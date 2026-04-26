/// Core simulation module
///
/// Main entry point for running 8760-hour chronological energy system simulation.
/// Supports multiple battery dispatch strategies and calculates all output arrays.
use crate::simulation::battery::{
    apply_default_dispatch, apply_hybrid_dispatch, apply_peak_shaver_dispatch,
    calculate_clean_delivered,
};
use crate::types::{BatteryMode, SimulationConfig, SimulationResult, HOURS_PER_YEAR};

/// Run the full energy system simulation
///
/// # Arguments
/// * `config` - Simulation configuration
/// * `solar_profile` - Solar capacity factors (8760 hours, 0-1)
/// * `wind_profile` - Wind capacity factors (8760 hours, 0-1)
/// * `load_profile` - Load MW (8760 hours)
///
/// # Returns
/// * SimulationResult with all output arrays and metrics
pub fn simulate_system(
    config: &SimulationConfig,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
) -> Result<SimulationResult, String> {
    // Validate inputs
    if solar_profile.len() != HOURS_PER_YEAR {
        return Err(format!(
            "Solar profile has {} hours, expected {}",
            solar_profile.len(),
            HOURS_PER_YEAR
        ));
    }
    if wind_profile.len() != HOURS_PER_YEAR {
        return Err(format!(
            "Wind profile has {} hours, expected {}",
            wind_profile.len(),
            HOURS_PER_YEAR
        ));
    }
    if load_profile.len() != HOURS_PER_YEAR {
        return Err(format!(
            "Load profile has {} hours, expected {}",
            load_profile.len(),
            HOURS_PER_YEAR
        ));
    }

    let mut result = SimulationResult::new();

    // Calculate generation arrays
    for i in 0..HOURS_PER_YEAR {
        result.solar_out[i] = config.solar_capacity * solar_profile[i];
        result.wind_out[i] = config.wind_capacity * wind_profile[i];
        result.clean_firm_generation[i] = config.clean_firm_capacity;
    }

    // Calculate total renewable generation
    let mut renewable_gen = vec![0.0; HOURS_PER_YEAR];
    for i in 0..HOURS_PER_YEAR {
        renewable_gen[i] = result.solar_out[i] + result.wind_out[i];
    }

    // Apply demand response (if any)
    let mut effective_load = vec![0.0; HOURS_PER_YEAR];
    for i in 0..HOURS_PER_YEAR {
        // DR reduces load during peaks
        let total_clean = renewable_gen[i] + config.clean_firm_capacity;
        if load_profile[i] > total_clean {
            let deficit = load_profile[i] - total_clean;
            let dr_applied = deficit.min(config.max_demand_response);
            result.demand_response[i] = dr_applied;
            effective_load[i] = load_profile[i] - dr_applied;
        } else {
            effective_load[i] = load_profile[i];
        }
    }

    // Run battery dispatch based on mode
    match config.battery_mode {
        BatteryMode::Default => {
            let (charge, discharge, soc, curtailed) = apply_default_dispatch(
                &renewable_gen,
                &effective_load,
                config.clean_firm_capacity,
                config.storage_capacity,
                config.battery_efficiency,
            );
            result.battery_charge = charge;
            result.battery_discharge = discharge;
            result.state_of_charge = soc;
            result.curtailed = curtailed;
        }
        BatteryMode::PeakShaver => {
            let (charge, discharge, soc, curtailed, gas_for_charge) = apply_peak_shaver_dispatch(
                &renewable_gen,
                &effective_load,
                config.clean_firm_capacity,
                config.storage_capacity,
                config.battery_efficiency,
            );
            result.battery_charge = charge;
            result.battery_discharge = discharge;
            result.state_of_charge = soc;
            result.curtailed = curtailed;
            result.gas_for_charging = gas_for_charge;
        }
        BatteryMode::Hybrid => {
            let (charge, discharge, soc, curtailed, gas_for_charge) =
                apply_hybrid_dispatch(
                    &renewable_gen,
                    &effective_load,
                    config.clean_firm_capacity,
                    config.storage_capacity,
                    config.battery_efficiency,
                );
            result.battery_charge = charge;
            result.battery_discharge = discharge;
            result.state_of_charge = soc;
            result.curtailed = curtailed;
            result.gas_for_charging = gas_for_charge;
        }
    }

    result.clean_delivered = calculate_clean_delivered(
        &renewable_gen,
        &effective_load,
        config.clean_firm_capacity,
        &result.battery_charge,
        &result.battery_discharge,
        &result.gas_for_charging,
        config.battery_efficiency,
        config.storage_capacity,
    );

    // Calculate gas generation (to fill remaining deficit)
    // Gas provides energy for: (1) load not met by renewables/battery, (2) battery charging
    for i in 0..HOURS_PER_YEAR {
        let total_clean = renewable_gen[i] + config.clean_firm_capacity;
        // Battery discharge provides energy to meet load
        let battery_delivery = result.battery_discharge[i] * config.battery_efficiency;
        // Gas needed for load = load - renewable - battery delivery
        let gas_for_load = (effective_load[i] - total_clean - battery_delivery).max(0.0);
        // Total gas = gas for load + gas used to charge battery
        result.gas_generation[i] = gas_for_load + result.gas_for_charging[i];
    }

    // Calculate scalar metrics
    result.annual_renewable_gen = renewable_gen.iter().sum();
    result.annual_load = effective_load.iter().sum();
    result.peak_gas = result.gas_generation.iter().cloned().fold(0.0, f64::max);
    result.total_curtailment = result.curtailed.iter().sum();

    // Calculate clean match percentage from exact clean-delivered accounting.
    let clean_served: f64 = result.clean_delivered.iter().sum();
    result.clean_match_pct = if result.annual_load > 0.0 {
        (clean_served / result.annual_load) * 100.0
    } else {
        0.0
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        // Simple flat profiles for testing
        let solar = vec![0.25; HOURS_PER_YEAR]; // 25% capacity factor
        let wind = vec![0.35; HOURS_PER_YEAR]; // 35% capacity factor
        let load = vec![100.0; HOURS_PER_YEAR]; // 100 MW constant load
        (solar, wind, load)
    }

    #[test]
    fn test_simulate_zero_capacity() {
        let config = SimulationConfig::with_defaults();
        let (solar, wind, load) = create_test_profiles();

        let result = simulate_system(&config, &solar, &wind, &load).unwrap();

        // With zero capacity, all load should be served by gas
        assert!(result.solar_out.iter().all(|&x| x == 0.0));
        assert!(result.wind_out.iter().all(|&x| x == 0.0));
        assert!(result.gas_generation.iter().all(|&x| x == 100.0));
        assert_eq!(result.clean_match_pct, 0.0);
    }

    #[test]
    fn test_simulate_solar_only() {
        let mut config = SimulationConfig::with_defaults();
        config.solar_capacity = 100.0; // 100 MW solar
        let (solar, wind, load) = create_test_profiles();

        let result = simulate_system(&config, &solar, &wind, &load).unwrap();

        // Solar should generate 25 MW each hour (100 * 0.25)
        assert!(result.solar_out.iter().all(|&x| (x - 25.0).abs() < 0.01));
        // Wind should be zero
        assert!(result.wind_out.iter().all(|&x| x == 0.0));
        // Gas should cover remaining 75 MW
        assert!(result
            .gas_generation
            .iter()
            .all(|&x| (x - 75.0).abs() < 0.01));
    }

    #[test]
    fn test_simulate_excess_generation() {
        let mut config = SimulationConfig::with_defaults();
        config.solar_capacity = 200.0;
        config.wind_capacity = 200.0;
        let (solar, wind, load) = create_test_profiles();

        let result = simulate_system(&config, &solar, &wind, &load).unwrap();

        // Total generation = 200*0.25 + 200*0.35 = 50 + 70 = 120 MW
        // Load = 100 MW
        // Should have curtailment = 20 MW
        assert!(result.curtailed.iter().all(|&x| (x - 20.0).abs() < 0.01));
        // No gas needed
        assert!(result.gas_generation.iter().all(|&x| x < 0.01));
    }

    #[test]
    fn test_simulate_with_storage() {
        let mut config = SimulationConfig::with_defaults();
        config.solar_capacity = 300.0; // 300 MW solar
        config.storage_capacity = 50.0;

        // Create varying profile
        let mut solar = vec![0.0; HOURS_PER_YEAR];
        let wind = vec![0.0; HOURS_PER_YEAR];
        let load = vec![100.0; HOURS_PER_YEAR];

        // Solar only during day hours (6am-6pm)
        // 300 MW * 0.5 CF = 150 MW generation during day
        // Load = 100 MW, so 50 MW excess to charge battery
        for day in 0..(HOURS_PER_YEAR / 24) {
            for hour in 6..18 {
                solar[day * 24 + hour] = 0.5; // 50% CF during day
            }
        }

        let result = simulate_system(&config, &solar, &wind, &load).unwrap();

        // Should have some battery activity
        let total_charge: f64 = result.battery_charge.iter().sum();
        let total_discharge: f64 = result.battery_discharge.iter().sum();

        assert!(
            total_charge > 0.0,
            "Should have battery charging (excess solar during day)"
        );
        assert!(
            total_discharge > 0.0,
            "Should have battery discharging (deficit at night)"
        );
    }

    #[test]
    fn test_invalid_profile_length() {
        let config = SimulationConfig::with_defaults();
        let short_profile = vec![0.25; 100]; // Wrong length
        let wind = vec![0.35; HOURS_PER_YEAR];
        let load = vec![100.0; HOURS_PER_YEAR];

        let result = simulate_system(&config, &short_profile, &wind, &load);
        assert!(result.is_err());
    }

    #[test]
    fn test_all_battery_modes() {
        let (solar, wind, load) = create_test_profiles();

        for mode in [
            BatteryMode::Default,
            BatteryMode::PeakShaver,
            BatteryMode::Hybrid,
        ] {
            let mut config = SimulationConfig::with_defaults();
            config.solar_capacity = 150.0;
            config.wind_capacity = 100.0;
            config.storage_capacity = 50.0;
            config.battery_mode = mode;

            let result = simulate_system(&config, &solar, &wind, &load);
            assert!(result.is_ok(), "Mode {:?} should succeed", mode);

            let r = result.unwrap();
            assert!(r.annual_load > 0.0);
            assert!(r.clean_match_pct >= 0.0 && r.clean_match_pct <= 100.0);
        }
    }
}
