pub mod global_grid;
pub mod oracle;
pub mod scenarios;
pub mod types;

pub use global_grid::run_v3_global_grid;
pub use oracle::run_v3_oracle;
pub use scenarios::{apply_quick_fail_state, run_suite, write_suite_report};
pub use types::{CaseReport, EvaluatedPoint, SuiteReport, V3Diagnostics, V3Result, V3SearchConfig};

use crate::economics::calculate_lcoe;
use crate::simulation::simulate_system;
use crate::types::{BatteryMode, CostParams, SimulationConfig};
use crate::types::OptimizerConfig;

pub(crate) fn build_axis(max: f64, step: f64) -> Result<Vec<f64>, String> {
    if max < 0.0 {
        return Err(format!("Negative max bound: {}", max));
    }
    if step <= 0.0 {
        return Err(format!("Step must be > 0.0, got {}", step));
    }

    if max == 0.0 {
        return Ok(vec![0.0]);
    }

    let mut values = Vec::new();
    values.push(0.0);
    let mut current = 0.0;
    while current + step < max - 1e-9 {
        current += step;
        values.push(round_capacity(current));
    }

    let last = values[values.len() - 1];
    if (last - max).abs() > 1e-9 {
        values.push(round_capacity(max));
    }

    Ok(values)
}

pub(crate) fn evaluate_clean_match(
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    battery_mode: BatteryMode,
) -> Result<f64, String> {
    let config = SimulationConfig {
        solar_capacity: solar,
        wind_capacity: wind,
        storage_capacity: storage,
        clean_firm_capacity: clean_firm,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
        battery_mode,
    };

    let sim = simulate_system(&config, solar_profile, wind_profile, load_profile)?;
    Ok(sim.clean_match_pct)
}

pub(crate) fn evaluate_point(
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
) -> Result<EvaluatedPoint, String> {
    let config = SimulationConfig {
        solar_capacity: solar,
        wind_capacity: wind,
        storage_capacity: storage,
        clean_firm_capacity: clean_firm,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
        battery_mode,
    };

    let sim = simulate_system(&config, solar_profile, wind_profile, load_profile)?;
    let lcoe = calculate_lcoe(&sim, solar, wind, storage, clean_firm, costs);

    Ok(EvaluatedPoint {
        solar,
        wind,
        storage,
        clean_firm,
        clean_match: sim.clean_match_pct,
        lcoe: lcoe.total_lcoe,
    })
}

pub fn run_v3_optimizer(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    optimizer_config: &OptimizerConfig,
) -> Result<crate::types::OptimizerResult, String> {
    let search = V3SearchConfig::from_optimizer_config(optimizer_config, target);
    let result = run_v3_global_grid(
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        battery_mode,
        &search,
    )?;

    Ok(result.result)
}

fn round_capacity(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}
