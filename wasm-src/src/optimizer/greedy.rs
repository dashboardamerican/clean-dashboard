/// Greedy optimizer for renewable portfolio selection
///
/// Given a fixed clean firm capacity, uses a greedy algorithm to
/// find optimal solar, wind, and storage capacities to hit a target
/// clean match percentage.
use crate::economics::calculate_lcoe;
use crate::simulation::simulate_system;
use crate::types::{BatteryMode, CostParams, OptimizerConfig, SimulationConfig};

/// Evaluation result for a portfolio configuration
#[derive(Clone, Debug)]
pub struct EvalResult {
    pub solar: f64,
    pub wind: f64,
    pub storage: f64,
    pub clean_firm: f64,
    pub lcoe: f64,
    pub clean_match: f64,
}

/// Evaluate a portfolio configuration
pub fn evaluate_portfolio(
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
) -> Result<EvalResult, String> {
    let simulation_config = SimulationConfig {
        solar_capacity: solar,
        wind_capacity: wind,
        storage_capacity: storage,
        clean_firm_capacity: clean_firm,
        battery_efficiency: config.battery_efficiency,
        max_demand_response: config.max_demand_response,
        battery_mode,
    };

    let sim_result = simulate_system(&simulation_config, solar_profile, wind_profile, load_profile)?;
    let lcoe_result = calculate_lcoe(&sim_result, solar, wind, storage, clean_firm, costs);

    Ok(EvalResult {
        solar,
        wind,
        storage,
        clean_firm,
        lcoe: lcoe_result.total_lcoe,
        clean_match: sim_result.clean_match_pct,
    })
}

/// Run greedy optimization for a fixed clean firm capacity
///
/// # Arguments
/// * `clean_firm` - Fixed clean firm capacity MW
/// * `target_match` - Target clean match percentage
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs` - Cost parameters
/// * `config` - Optimizer configuration (for bounds and enabled resources)
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * Best portfolio found and number of evaluations
pub fn run_greedy_optimizer(
    clean_firm: f64,
    target_match: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
) -> Result<(EvalResult, u32), String> {
    let mut num_evals = 0u32;

    // Initial capacities
    let mut solar = 0.0;
    let mut wind = 0.0;
    let mut storage = 0.0;

    // Step sizes (start broad for exploration, refine down to min_step)
    let mut solar_step: f64 = if config.enable_solar { 50.0 } else { 0.0 };
    let mut wind_step: f64 = if config.enable_wind { 50.0 } else { 0.0 };
    let mut storage_step: f64 = if config.enable_storage { 25.0 } else { 0.0 };

    // Evaluate initial state
    let mut current = evaluate_portfolio(
        solar,
        wind,
        storage,
        clean_firm,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
    )?;
    num_evals += 1;

    let min_step: f64 = 0.5; // Finer steps for better precision
    let max_iterations = 500; // More iterations for finer step sizes
    let tolerance = 0.5; // Clean match tolerance

    for _ in 0..max_iterations {
        if current.clean_match >= target_match - tolerance {
            // Check if we're close enough to target
            if (current.clean_match - target_match).abs() < tolerance {
                break;
            }
            // If we've overshot, we could refine, but for now accept it
            if current.clean_match > target_match + tolerance {
                // Reduce step sizes and continue
                solar_step = (solar_step / 2.0).max(min_step);
                wind_step = (wind_step / 2.0).max(min_step);
                storage_step = (storage_step / 2.0).max(min_step);

                if solar_step <= min_step && wind_step <= min_step && storage_step <= min_step {
                    break;
                }
            }
        }

        // Find best next move
        let mut best_option: Option<EvalResult> = None;
        let mut best_efficiency = f64::INFINITY;

        // Try adding solar
        if config.enable_solar && solar + solar_step <= config.max_solar {
            let test_solar = solar + solar_step;
            if let Ok(result) = evaluate_portfolio(
                test_solar,
                wind,
                storage,
                clean_firm,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                battery_mode,
            ) {
                num_evals += 1;
                let match_gain = result.clean_match - current.clean_match;
                if match_gain > 0.1 {
                    let lcoe_change = result.lcoe - current.lcoe;
                    let efficiency = lcoe_change / match_gain;
                    if efficiency < best_efficiency {
                        best_efficiency = efficiency;
                        best_option = Some(result);
                    }
                }
            }
        }

        // Try adding wind
        if config.enable_wind && wind + wind_step <= config.max_wind {
            let test_wind = wind + wind_step;
            if let Ok(result) = evaluate_portfolio(
                solar,
                test_wind,
                storage,
                clean_firm,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                battery_mode,
            ) {
                num_evals += 1;
                let match_gain = result.clean_match - current.clean_match;
                if match_gain > 0.1 {
                    let lcoe_change = result.lcoe - current.lcoe;
                    let efficiency = lcoe_change / match_gain;
                    if efficiency < best_efficiency {
                        best_efficiency = efficiency;
                        best_option = Some(result);
                    }
                }
            }
        }

        // Try adding storage
        if config.enable_storage && storage + storage_step <= config.max_storage {
            let test_storage = storage + storage_step;
            if let Ok(result) = evaluate_portfolio(
                solar,
                wind,
                test_storage,
                clean_firm,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                battery_mode,
            ) {
                num_evals += 1;
                let match_gain = result.clean_match - current.clean_match;
                if match_gain > 0.1 {
                    let lcoe_change = result.lcoe - current.lcoe;
                    let efficiency = lcoe_change / match_gain;
                    if efficiency < best_efficiency {
                        best_efficiency = efficiency;
                        best_option = Some(result);
                    }
                }
            }
        }

        // Apply best option or reduce step sizes
        match best_option {
            Some(result) => {
                // Check for overshoot
                if result.clean_match > target_match + tolerance {
                    // Reduce step sizes instead of applying
                    solar_step = (solar_step / 2.0).max(min_step);
                    wind_step = (wind_step / 2.0).max(min_step);
                    storage_step = (storage_step / 2.0).max(min_step);
                } else {
                    solar = result.solar;
                    wind = result.wind;
                    storage = result.storage;
                    current = result;
                }
            }
            None => {
                // No improving move found, reduce step sizes
                solar_step = (solar_step / 2.0).max(min_step);
                wind_step = (wind_step / 2.0).max(min_step);
                storage_step = (storage_step / 2.0).max(min_step);

                if solar_step <= min_step && wind_step <= min_step && storage_step <= min_step {
                    break;
                }
            }
        }
    }

    Ok((current, num_evals))
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
    fn test_evaluate_portfolio() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let result = evaluate_portfolio(
            50.0,
            50.0,
            25.0,
            10.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Default,
        )
        .unwrap();

        assert!(result.lcoe > 0.0);
        assert!(result.clean_match >= 0.0 && result.clean_match <= 100.0);
    }

    #[test]
    fn test_greedy_optimizer() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let (result, num_evals) = run_greedy_optimizer(
            0.0,  // No clean firm
            50.0, // 50% target
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Default,
        )
        .unwrap();

        assert!(num_evals > 0);
        // Should get reasonably close to target
        assert!(
            result.clean_match >= 40.0,
            "Should achieve close to 50% target"
        );
    }
}
