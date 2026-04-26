use super::types::{
    compare_points, is_feasible, EvaluatedPoint, V3Diagnostics, V3Result, V3SearchConfig,
};
use super::{build_axis, evaluate_point};
use crate::types::{BatteryMode, CostParams};

#[cfg(feature = "native")]
use rayon::prelude::*;

pub fn run_v3_oracle(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    config: &V3SearchConfig,
) -> Result<V3Result, String> {
    let solar_values = build_axis(config.max_solar, config.solar_step)?;
    let wind_values = build_axis(config.max_wind, config.wind_step)?;
    let storage_values = build_axis(config.max_storage, config.storage_step)?;
    let cf_values = build_axis(config.max_clean_firm, config.cf_step)?;

    let outcomes = evaluate_oracle_points(
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        battery_mode,
        config,
        &solar_values,
        &wind_values,
        &storage_values,
        &cf_values,
    )?;

    let mut best: Option<EvaluatedPoint> = None;
    let mut total_lcoe_evals = 0u64;
    let mut feasible_points_checked = 0u64;

    for outcome in outcomes {
        total_lcoe_evals += outcome.lcoe_evals;
        feasible_points_checked += outcome.feasible_points;
        if let Some(candidate) = outcome.best {
            if best
                .as_ref()
                .map(|current| compare_points(&candidate, current, target).is_lt())
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
    }

    let Some(best_point) = best else {
        return Err(format!(
            "No feasible portfolio found for target {} with tolerance {}",
            target, config.target_tolerance
        ));
    };

    let mut diagnostics = V3Diagnostics::from_config(config);
    diagnostics.total_triples =
        (solar_values.len() * wind_values.len() * storage_values.len()) as u64;
    diagnostics.total_clean_evals = 0;
    diagnostics.total_lcoe_evals = total_lcoe_evals;
    diagnostics.feasible_points_checked = feasible_points_checked;
    diagnostics.monotonic_fallback_count = 0;
    diagnostics.certified = true;

    let result = best_point.to_optimizer_result(
        diagnostics.total_evaluations().min(u32::MAX as u64) as u32,
        target,
        config.target_tolerance,
    );

    Ok(V3Result {
        result,
        diagnostics,
    })
}

struct OracleOutcome {
    best: Option<EvaluatedPoint>,
    lcoe_evals: u64,
    feasible_points: u64,
}

#[allow(clippy::too_many_arguments)]
fn evaluate_oracle_points(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    config: &V3SearchConfig,
    solar_values: &[f64],
    wind_values: &[f64],
    storage_values: &[f64],
    cf_values: &[f64],
) -> Result<Vec<OracleOutcome>, String> {
    #[cfg(feature = "native")]
    {
        if config.parallel {
            return solar_values
                .par_iter()
                .map(|&solar| {
                    evaluate_single_solar(
                        solar,
                        target,
                        solar_profile,
                        wind_profile,
                        load_profile,
                        costs,
                        battery_mode,
                        wind_values,
                        storage_values,
                        cf_values,
                        config.target_tolerance,
                    )
                })
                .collect::<Result<Vec<_>, _>>();
        }
    }

    let mut outcomes = Vec::with_capacity(solar_values.len());
    for &solar in solar_values {
        outcomes.push(evaluate_single_solar(
            solar,
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            battery_mode,
            wind_values,
            storage_values,
            cf_values,
            config.target_tolerance,
        )?);
    }

    Ok(outcomes)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_single_solar(
    solar: f64,
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    wind_values: &[f64],
    storage_values: &[f64],
    cf_values: &[f64],
    tolerance: f64,
) -> Result<OracleOutcome, String> {
    let mut local_best: Option<EvaluatedPoint> = None;
    let mut lcoe_evals = 0u64;
    let mut feasible_points = 0u64;

    for &wind in wind_values {
        for &storage in storage_values {
            for &cf in cf_values {
                let point = evaluate_point(
                    solar,
                    wind,
                    storage,
                    cf,
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    battery_mode,
                )?;
                lcoe_evals += 1;

                if is_feasible(point.clean_match, target, tolerance) {
                    feasible_points += 1;
                    if local_best
                        .as_ref()
                        .map(|current| compare_points(&point, current, target).is_lt())
                        .unwrap_or(true)
                    {
                        local_best = Some(point);
                    }
                }
            }
        }
    }

    Ok(OracleOutcome {
        best: local_best,
        lcoe_evals,
        feasible_points,
    })
}
