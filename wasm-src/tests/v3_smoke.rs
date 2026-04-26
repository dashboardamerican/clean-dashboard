#![cfg(feature = "experimental-v3")]

use energy_simulator::optimizer::v3::scenarios::load_real_zone;
use energy_simulator::optimizer::{run_v3_global_grid, V3SearchConfig};
use energy_simulator::{BatteryMode, CostParams, HOURS_PER_YEAR};

fn synthetic_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut solar = Vec::with_capacity(HOURS_PER_YEAR);
    let mut wind = Vec::with_capacity(HOURS_PER_YEAR);
    let mut load = Vec::with_capacity(HOURS_PER_YEAR);

    for h in 0..HOURS_PER_YEAR {
        let hod = h % 24;
        let day = h / 24;
        let seasonal = 0.85 + 0.15 * (2.0 * std::f64::consts::PI * day as f64 / 365.0).sin();

        let solar_cf = if (6..=18).contains(&hod) {
            let shape = 1.0 - ((hod as f64 - 12.0).abs() / 6.0);
            (0.30 * shape * seasonal).max(0.0)
        } else {
            0.0
        };
        solar.push(solar_cf);

        let wind_cf = if hod < 6 || hod > 20 { 0.40 } else { 0.30 };
        wind.push(wind_cf);

        let load_mw = if (17..=22).contains(&hod) {
            110.0
        } else if (0..=5).contains(&hod) {
            90.0
        } else {
            100.0
        };
        load.push(load_mw);
    }

    (solar, wind, load)
}

fn smoke_config() -> V3SearchConfig {
    V3SearchConfig {
        solar_step: 25.0,
        wind_step: 25.0,
        storage_step: 25.0,
        cf_step: 5.0,
        target_tolerance: 0.5,
        max_solar: 100.0,
        max_wind: 400.0,
        max_storage: 100.0,
        max_clean_firm: 100.0,
        parallel: true,
        monotonic_scan_local_radius: 2,
        monotonic_full_scan_threshold: 5,
        ..V3SearchConfig::default()
    }
}

#[test]
fn v3_smoke_synthetic_and_real_zone_are_feasible_and_deterministic() {
    let costs = CostParams::default_costs();
    let target = 95.0;
    let config = smoke_config();
    let tolerance = config.target_tolerance + 1e-6;

    let (syn_solar, syn_wind, syn_load) = synthetic_profiles();
    let syn_a = run_v3_global_grid(
        target,
        &syn_solar,
        &syn_wind,
        &syn_load,
        &costs,
        BatteryMode::Hybrid,
        &config,
    )
    .expect("synthetic run A failed");
    let syn_b = run_v3_global_grid(
        target,
        &syn_solar,
        &syn_wind,
        &syn_load,
        &costs,
        BatteryMode::Hybrid,
        &config,
    )
    .expect("synthetic run B failed");

    assert!(
        (syn_a.result.achieved_clean_match - target).abs() <= tolerance,
        "Synthetic deviation too high: target={} achieved={}",
        target,
        syn_a.result.achieved_clean_match
    );
    assert_eq!(syn_a.result.solar_capacity, syn_b.result.solar_capacity);
    assert_eq!(syn_a.result.wind_capacity, syn_b.result.wind_capacity);
    assert_eq!(syn_a.result.storage_capacity, syn_b.result.storage_capacity);
    assert_eq!(
        syn_a.result.clean_firm_capacity,
        syn_b.result.clean_firm_capacity
    );
    assert!(
        (syn_a.result.lcoe - syn_b.result.lcoe).abs() <= 1e-9,
        "Synthetic LCOE not deterministic: A={} B={}",
        syn_a.result.lcoe,
        syn_b.result.lcoe
    );

    let (ca_solar, ca_wind, ca_load) = load_real_zone("california")
        .expect("failed to load california profile from data/zones.json");
    let real_result = run_v3_global_grid(
        target,
        &ca_solar,
        &ca_wind,
        &ca_load,
        &costs,
        BatteryMode::Hybrid,
        &config,
    )
    .expect("real-zone run failed");

    assert!(
        (real_result.result.achieved_clean_match - target).abs() <= tolerance,
        "Real-zone deviation too high: target={} achieved={}",
        target,
        real_result.result.achieved_clean_match
    );
}
