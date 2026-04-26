#![cfg(feature = "experimental-v3")]

use energy_simulator::optimizer::{run_v3_global_grid, run_v3_oracle, V3SearchConfig};
use energy_simulator::{BatteryMode, CostParams, HOURS_PER_YEAR};

fn synthetic_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let mut solar = Vec::with_capacity(HOURS_PER_YEAR);
    let mut wind = Vec::with_capacity(HOURS_PER_YEAR);
    let mut load = Vec::with_capacity(HOURS_PER_YEAR);

    for h in 0..HOURS_PER_YEAR {
        let hod = h % 24;

        let solar_cf = if (6..=18).contains(&hod) {
            let shape = 1.0 - ((hod as f64 - 12.0).abs() / 6.0);
            (0.28 * shape).max(0.0)
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

#[test]
fn v3_global_grid_matches_oracle_on_identical_lattice() {
    let (solar, wind, load) = synthetic_profiles();
    let costs = CostParams::default_costs();

    let config = V3SearchConfig {
        solar_step: 25.0,
        wind_step: 25.0,
        storage_step: 25.0,
        cf_step: 5.0,
        target_tolerance: 0.5,
        max_solar: 100.0,
        max_wind: 200.0,
        max_storage: 100.0,
        max_clean_firm: 100.0,
        parallel: true,
        monotonic_scan_local_radius: 2,
        monotonic_full_scan_threshold: 5,
        ..V3SearchConfig::default()
    };

    let target = 95.0;
    let v3 = run_v3_global_grid(
        target,
        &solar,
        &wind,
        &load,
        &costs,
        BatteryMode::Hybrid,
        &config,
    )
    .expect("v3 global-grid run failed");

    let oracle = run_v3_oracle(
        target,
        &solar,
        &wind,
        &load,
        &costs,
        BatteryMode::Hybrid,
        &config,
    )
    .expect("v3 oracle run failed");

    assert!(
        (v3.result.lcoe - oracle.result.lcoe).abs() <= 1e-9,
        "LCOE mismatch: v3={} oracle={}",
        v3.result.lcoe,
        oracle.result.lcoe
    );
    assert_eq!(v3.result.solar_capacity, oracle.result.solar_capacity);
    assert_eq!(v3.result.wind_capacity, oracle.result.wind_capacity);
    assert_eq!(v3.result.storage_capacity, oracle.result.storage_capacity);
    assert_eq!(
        v3.result.clean_firm_capacity,
        oracle.result.clean_firm_capacity
    );
    assert!(
        (v3.result.achieved_clean_match - oracle.result.achieved_clean_match).abs() <= 1e-9,
        "Clean-match mismatch: v3={} oracle={}",
        v3.result.achieved_clean_match,
        oracle.result.achieved_clean_match
    );
    assert!(
        v3.diagnostics.certified,
        "v3 should report certified result"
    );
    assert!(
        oracle.diagnostics.certified,
        "oracle should report certified result"
    );
}
