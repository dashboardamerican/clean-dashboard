//! Cross-validation tests for emissions, land use, and the LCOE breakdown
//! against the Python reference (`multi_test.py`, `lcoe_calculator.py`).
//!
//! These tests pin numeric values produced by Python so any future drift in
//! the Rust implementation fails immediately. The expected values were
//! produced by running the Python pipeline on the California zone with the
//! `cost_settings_modal.DEFAULT_COSTS` defaults — see the per-test docstring
//! for the exact scenario.
//!
//! Run with: `cargo test --release --test economics_parity`

use energy_simulator::economics::{calculate_land_use, calculate_lcoe};
use energy_simulator::simulation::simulate_system;
use energy_simulator::types::{BatteryMode, CostParams, SimulationConfig};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
struct ZoneData {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

fn load_california() -> ZoneData {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("data")
        .join("zones.json");
    let content = fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Failed to read {:?}: {}", path, e));
    let zones: std::collections::HashMap<String, ZoneData> =
        serde_json::from_str(&content).expect("Failed to parse zones.json");
    zones
        .get("California")
        .expect("California not in zones.json")
        .clone()
}

fn run_simulation(
    zone: &ZoneData,
    solar_mw: f64,
    wind_mw: f64,
    storage_mwh: f64,
    cf_mw: f64,
) -> energy_simulator::types::SimulationResult {
    let cfg = SimulationConfig {
        solar_capacity: solar_mw,
        wind_capacity: wind_mw,
        storage_capacity: storage_mwh,
        clean_firm_capacity: cf_mw,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
        battery_mode: BatteryMode::Default,
    };
    simulate_system(&cfg, &zone.solar, &zone.wind, &zone.load).expect("simulation failed")
}

// ---------------------------------------------------------------------------
// Land use
// ---------------------------------------------------------------------------

/// Hand-fixture from running Python's `calculate_system_land_use` on
/// {solar=100, wind=50, clean_firm=20, gas_capacity_needed=137.4359937…}
/// with default land params.
#[test]
fn land_use_california_mixed_portfolio_matches_python() {
    let zone = load_california();
    let sim = run_simulation(&zone, 100.0, 50.0, 0.0, 20.0);
    // Python predates the reserve-margin lever; pin it to 0 to keep parity.
    let mut costs = CostParams::default_costs();
    costs.reserve_margin = 0.0;
    let land = calculate_land_use(100.0, 50.0, 20.0, sim.peak_gas, &costs);

    // Python output (multi_test.calculate_system_land_use), pinned:
    let py_direct_acres = 758.612838811964;
    let py_total_acres = 3196.112838811964;

    assert!(
        (land.direct_acres - py_direct_acres).abs() < 1e-6,
        "direct_acres parity: rust={} python={}",
        land.direct_acres,
        py_direct_acres
    );
    assert!(
        (land.total_acres - py_total_acres).abs() < 1e-6,
        "total_acres parity: rust={} python={}",
        land.total_acres,
        py_total_acres
    );
    // mi² is just acres / 640; sanity-check the conversion:
    assert!((land.direct_mi2 * 640.0 - land.direct_acres).abs() < 1e-9);
    assert!((land.total_mi2 * 640.0 - land.total_acres).abs() < 1e-9);
}

#[test]
fn land_use_via_lcoe_matches_standalone() {
    // The standalone fn and the version inside `calculate_lcoe` should
    // produce identical numbers — they share the same formula and inputs.
    let zone = load_california();
    let sim = run_simulation(&zone, 100.0, 50.0, 0.0, 20.0);
    let costs = CostParams::default_costs();

    let standalone = calculate_land_use(100.0, 50.0, 20.0, sim.peak_gas, &costs);
    let lcoe = calculate_lcoe(&sim, 100.0, 50.0, 0.0, 20.0, &costs);

    assert!(
        (lcoe.direct_land_use - standalone.direct_acres).abs() < 1e-9,
        "lcoe.direct_land_use ({}) != standalone direct ({})",
        lcoe.direct_land_use,
        standalone.direct_acres
    );
    assert!(
        (lcoe.total_land_use - standalone.total_acres).abs() < 1e-9,
        "lcoe.total_land_use ({}) != standalone total ({})",
        lcoe.total_land_use,
        standalone.total_acres
    );
}

// ---------------------------------------------------------------------------
// Emissions
// ---------------------------------------------------------------------------

/// Pinned against Python's `calculate_system_emissions` for California with
/// {solar=100, wind=50, storage=0, cf=20} and DEFAULT_COSTS (which uses
/// methane_gwp='GWP100' → 27.2).
///
/// Python output: 174.918247 g CO2eq/kWh.
#[test]
fn emissions_california_mixed_portfolio_matches_python() {
    let zone = load_california();
    let sim = run_simulation(&zone, 100.0, 50.0, 0.0, 20.0);
    let costs = CostParams::default_costs();
    let lcoe = calculate_lcoe(&sim, 100.0, 50.0, 0.0, 20.0, &costs);

    let py_emissions = 174.918247;
    let diff = (lcoe.emissions_intensity - py_emissions).abs();

    // ±0.5 g/kWh tolerance (~0.3%) accounts for tiny differences in how the
    // simulation's 8760-hour gas generation is summed across implementations.
    assert!(
        diff < 0.5,
        "emissions parity: rust={:.4} python={:.4} diff={:.4}",
        lcoe.emissions_intensity,
        py_emissions,
        diff
    );
}

#[test]
fn emissions_zero_renewables_pure_gas() {
    // Smoke test: an all-gas system should produce ~400+ g/kWh from gas
    // combustion alone. Catches order-of-magnitude regressions.
    let zone = load_california();
    let sim = run_simulation(&zone, 0.0, 0.0, 0.0, 0.0);
    let costs = CostParams::default_costs();
    let lcoe = calculate_lcoe(&sim, 0.0, 0.0, 0.0, 0.0, &costs);

    assert!(
        lcoe.emissions_intensity > 350.0 && lcoe.emissions_intensity < 600.0,
        "all-gas emissions look wrong: {}",
        lcoe.emissions_intensity
    );
}

#[test]
fn emissions_methane_factor_locked_in() {
    // Specifically guard against the 19.2 kg-CH4-per-MMBtu factor
    // disappearing again. Doubles the leakage rate and checks methane
    // share scales linearly.
    let zone = load_california();
    let sim = run_simulation(&zone, 0.0, 0.0, 0.0, 0.0);
    let mut low = CostParams::default_costs();
    low.gas_leakage_rate = 1.0;
    let mut high = CostParams::default_costs();
    high.gas_leakage_rate = 2.0;

    let lcoe_low = calculate_lcoe(&sim, 0.0, 0.0, 0.0, 0.0, &low);
    let lcoe_high = calculate_lcoe(&sim, 0.0, 0.0, 0.0, 0.0, &high);

    // Doubling the leakage rate should add a meaningful (>5 g/kWh)
    // chunk to total emissions. If the 19.2 factor is missing again,
    // the delta would be tiny (~0.3 g/kWh) and this fails.
    let delta = lcoe_high.emissions_intensity - lcoe_low.emissions_intensity;
    assert!(
        delta > 5.0,
        "methane leakage doubling produced too small a delta: {:.4} g/kWh \
         (likely the 19.2 kg/MMBtu factor is missing)",
        delta
    );
}

#[test]
fn emissions_battery_embodied_factor_locked_in() {
    // Guards against the storage *1000 factor disappearing again.
    // Storage capacity is in MWh; battery_embodied_emissions is per kWh.
    // 1000 MWh of storage with default 100 kg/kWh embodied @ 20yr =
    // 5000000 kg/year → meaningful contribution to emissions intensity.
    let zone = load_california();
    let sim_no_storage = run_simulation(&zone, 0.0, 0.0, 0.0, 0.0);
    let sim_lots_storage = run_simulation(&zone, 0.0, 0.0, 1000.0, 0.0);
    let costs = CostParams::default_costs();

    let lcoe_no = calculate_lcoe(&sim_no_storage, 0.0, 0.0, 0.0, 0.0, &costs);
    let lcoe_yes = calculate_lcoe(&sim_lots_storage, 0.0, 0.0, 1000.0, 0.0, &costs);

    let delta = lcoe_yes.emissions_intensity - lcoe_no.emissions_intensity;
    assert!(
        delta > 1.0,
        "1000 MWh of storage barely moved emissions ({:.4} g/kWh delta) — \
         the *1000 factor on battery embodied emissions is probably missing",
        delta
    );
}
