/// V1 Adaptive Optimizer (DEPRECATED)
///
/// **DEPRECATED**: Use V2 hierarchical optimizer instead (`run_v2_optimizer`).
/// V2 is faster (~20ms vs ~100ms) and more precise.
///
/// This module is kept for reference and backward compatibility only.
///
/// Algorithm: CF Grid Search + Gap-Aware Greedy
/// - Phase 1: Coarse CF grid search (0-120 MW in 10 MW steps)
/// - Phase 2: Greedy optimization on solar/wind/storage for each CF level
/// - Phase 3: Refinement around best CF level
use crate::optimizer::greedy::{evaluate_portfolio, run_greedy_optimizer, EvalResult};
use crate::types::{BatteryMode, CostParams, OptimizerConfig, OptimizerResult};

/// Penalty for not meeting target
const TARGET_MISS_PENALTY: f64 = 1000.0;

/// Calculate penalized LCOE
fn penalized_lcoe(lcoe: f64, clean_match: f64, target: f64, tolerance: f64) -> f64 {
    if clean_match < target - tolerance {
        // Below target - add penalty proportional to gap
        let gap = target - clean_match;
        lcoe + gap * TARGET_MISS_PENALTY
    } else {
        lcoe
    }
}

/// Run the V1 adaptive optimizer
///
/// # Arguments
/// * `target_match` - Target clean match percentage (0-100)
/// * `solar_profile` - Solar capacity factors (8760 hours)
/// * `wind_profile` - Wind capacity factors (8760 hours)
/// * `load_profile` - Load MW (8760 hours)
/// * `costs` - Cost parameters
/// * `config` - Optimizer configuration
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * OptimizerResult with optimal portfolio
pub fn run_v1_optimizer(
    target_match: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
) -> Result<OptimizerResult, String> {
    let tolerance = 0.2; // Clean match tolerance (tighter for precision)
    let mut total_evals = 0u32;

    // Phase 1: CF grid search (finer resolution for smoother sweeps)
    let cf_grid: Vec<f64> = if config.enable_clean_firm {
        // 5 MW steps for better resolution, extend to 150 MW for high targets
        vec![
            0.0, 5.0, 10.0, 15.0, 20.0, 25.0, 30.0, 35.0, 40.0, 45.0, 50.0, 55.0, 60.0, 65.0, 70.0,
            75.0, 80.0, 85.0, 90.0, 95.0, 100.0, 105.0, 110.0, 115.0, 120.0, 130.0, 140.0, 150.0,
        ]
        .into_iter()
        .filter(|&cf| cf <= config.max_clean_firm)
        .collect()
    } else {
        vec![0.0]
    };

    let mut best_result: Option<EvalResult> = None;
    let mut best_cf = 0.0;
    let mut best_penalized_lcoe = f64::INFINITY;

    for &cf in &cf_grid {
        let (result, evals) = run_greedy_optimizer(
            cf,
            target_match,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
        )?;
        total_evals += evals;

        let penalized = penalized_lcoe(result.lcoe, result.clean_match, target_match, tolerance);

        if penalized < best_penalized_lcoe {
            best_penalized_lcoe = penalized;
            best_cf = cf;
            best_result = Some(result);
        }
    }

    // Phase 2: Refinement around best CF level (finer offsets)
    let refinement_offsets: Vec<f64> = vec![-7.5, -2.5, 2.5, 7.5, 12.5, 17.5];

    for offset in refinement_offsets {
        let cf = (best_cf + offset).max(0.0).min(config.max_clean_firm);
        if cf_grid.contains(&cf) {
            continue; // Already evaluated
        }
        if !config.enable_clean_firm && cf > 0.0 {
            continue;
        }

        let (result, evals) = run_greedy_optimizer(
            cf,
            target_match,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
        )?;
        total_evals += evals;

        let penalized = penalized_lcoe(result.lcoe, result.clean_match, target_match, tolerance);

        if penalized < best_penalized_lcoe {
            best_penalized_lcoe = penalized;
            best_cf = cf;
            best_result = Some(result);
        }
    }

    // Phase 3: Continuous local refinement (not grid-snapped)
    // Fine-tune each capacity with 1 MW precision
    if let Some(ref mut result) = best_result {
        let refinement_step = 1.0; // 1 MW refinement
        let max_refinement_iters = 50;

        for _ in 0..max_refinement_iters {
            let mut improved = false;

            // Try adjusting each capacity independently
            for resource in 0..4 {
                for direction in [-1.0, 1.0] {
                    let delta = refinement_step * direction;

                    let (test_solar, test_wind, test_storage, test_cf) = match resource {
                        0 if config.enable_solar => {
                            let new_val = (result.solar + delta).max(0.0).min(config.max_solar);
                            if (new_val - result.solar).abs() < 0.01 {
                                continue;
                            }
                            (new_val, result.wind, result.storage, result.clean_firm)
                        }
                        1 if config.enable_wind => {
                            let new_val = (result.wind + delta).max(0.0).min(config.max_wind);
                            if (new_val - result.wind).abs() < 0.01 {
                                continue;
                            }
                            (result.solar, new_val, result.storage, result.clean_firm)
                        }
                        2 if config.enable_storage => {
                            let new_val = (result.storage + delta).max(0.0).min(config.max_storage);
                            if (new_val - result.storage).abs() < 0.01 {
                                continue;
                            }
                            (result.solar, result.wind, new_val, result.clean_firm)
                        }
                        3 if config.enable_clean_firm => {
                            let new_val = (result.clean_firm + delta)
                                .max(0.0)
                                .min(config.max_clean_firm);
                            if (new_val - result.clean_firm).abs() < 0.01 {
                                continue;
                            }
                            (result.solar, result.wind, result.storage, new_val)
                        }
                        _ => continue,
                    };

                    if let Ok(test_result) = evaluate_portfolio(
                        test_solar,
                        test_wind,
                        test_storage,
                        test_cf,
                        solar_profile,
                        wind_profile,
                        load_profile,
                        costs,
                        config,
                        battery_mode,
                    ) {
                        total_evals += 1;
                        let test_penalized = penalized_lcoe(
                            test_result.lcoe,
                            test_result.clean_match,
                            target_match,
                            tolerance,
                        );

                        if test_penalized < best_penalized_lcoe {
                            best_penalized_lcoe = test_penalized;
                            *result = test_result;
                            improved = true;
                        }
                    }
                }
            }

            if !improved {
                break;
            }
        }
    }

    // Build result
    match best_result {
        Some(result) => {
            // Success requires hitting target within 0.1% precision
            let success = (result.clean_match - target_match).abs() < 0.1;

            Ok(OptimizerResult {
                solar_capacity: result.solar,
                wind_capacity: result.wind,
                storage_capacity: result.storage,
                clean_firm_capacity: result.clean_firm,
                achieved_clean_match: result.clean_match,
                lcoe: result.lcoe,
                num_evaluations: total_evals,
                success,
            })
        }
        None => Err("No valid portfolio found".to_string()),
    }
}

/// Run optimizer sweep across multiple targets
///
/// # Arguments
/// * `targets` - Clean match targets to optimize for
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs` - Cost parameters
/// * `config` - Optimizer configuration
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * Vector of optimizer results
pub fn run_optimizer_sweep(
    targets: &[f64],
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
) -> Result<Vec<OptimizerResult>, String> {
    let mut results = Vec::with_capacity(targets.len());

    for &target in targets {
        let mut target_config = config.clone();
        target_config.target_clean_match = target;

        let result = run_v1_optimizer(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            &target_config,
            battery_mode,
        )?;
        results.push(result);
    }

    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HOURS_PER_YEAR;

    fn create_test_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let solar = vec![0.25; HOURS_PER_YEAR];
        let wind = vec![0.35; HOURS_PER_YEAR];
        let load = vec![100.0; HOURS_PER_YEAR];
        (solar, wind, load)
    }

    #[test]
    fn test_v1_optimizer_low_target() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let result = run_v1_optimizer(
            30.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Default,
        )
        .unwrap();

        assert!(result.num_evaluations > 0);
        // Should achieve close to 30% with primarily renewables
        assert!(result.achieved_clean_match >= 25.0);
        assert!(result.lcoe > 0.0);
    }

    #[test]
    fn test_v1_optimizer_high_target() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let result = run_v1_optimizer(
            80.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Default,
        )
        .unwrap();

        // High targets should use clean firm
        assert!(result.achieved_clean_match >= 70.0 || result.clean_firm_capacity > 0.0);
    }

    #[test]
    fn test_v1_optimizer_no_clean_firm() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let mut config = OptimizerConfig::default();
        config.enable_clean_firm = false;

        let result = run_v1_optimizer(
            50.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Default,
        )
        .unwrap();

        // Should not use clean firm
        assert_eq!(result.clean_firm_capacity, 0.0);
    }

    #[test]
    fn test_penalized_lcoe() {
        let lcoe = 50.0;
        let target = 80.0;
        let tolerance = 0.5;

        // Below target
        let penalized = penalized_lcoe(lcoe, 70.0, target, tolerance);
        assert!(penalized > lcoe);

        // At target
        let at_target = penalized_lcoe(lcoe, 80.0, target, tolerance);
        assert_eq!(at_target, lcoe);

        // Above target
        let above = penalized_lcoe(lcoe, 90.0, target, tolerance);
        assert_eq!(above, lcoe);
    }

    #[test]
    fn test_optimizer_sweep() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let targets = vec![20.0, 40.0, 60.0];
        let results = run_optimizer_sweep(
            &targets,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Default,
        )
        .unwrap();

        assert_eq!(results.len(), 3);

        // LCOE should generally increase with target
        // (not strictly guaranteed but typical)
        for result in &results {
            assert!(result.lcoe > 0.0);
        }
    }
}
