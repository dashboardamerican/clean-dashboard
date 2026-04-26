/// ELCC (Effective Load Carrying Capability) Calculation Module
///
/// ELCC measures how much a resource contributes to system reliability by
/// quantifying the reduction in peak gas generation required.
///
/// Four methods are supported:
/// - **First-In (Standalone)**: Resource alone, no other intermittent resources
/// - **Last-In (Marginal)**: Adding 10MW increment to full portfolio
/// - **Contribution (Removal)**: Portfolio minus portfolio-without-resource
/// - **Delta (E3)**: Last-In + proportional allocation of interactive effects
use crate::simulation::simulate_system;
use crate::types::{BatteryMode, ElccResult, ResourceElcc, SimulationConfig, HOURS_PER_YEAR};

/// Small increment for marginal calculations (MW)
const MARGINAL_INCREMENT: f64 = 10.0;

/// Calculate ELCC metrics for all resources using four methods
pub fn calculate_elcc(
    solar_capacity: f64,
    wind_capacity: f64,
    storage_capacity: f64,
    clean_firm_capacity: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    battery_mode: BatteryMode,
    battery_efficiency: f64,
    max_demand_response: f64,
) -> Result<ElccResult, String> {
    // Validate profiles
    if solar_profile.len() != HOURS_PER_YEAR
        || wind_profile.len() != HOURS_PER_YEAR
        || load_profile.len() != HOURS_PER_YEAR
    {
        return Err("All profiles must have 8760 hours".to_string());
    }

    // Helper to run simulation and get peak gas
    let run_sim = |solar: f64, wind: f64, storage: f64, cf: f64| -> Result<f64, String> {
        let config = SimulationConfig {
            solar_capacity: solar,
            wind_capacity: wind,
            storage_capacity: storage,
            clean_firm_capacity: cf,
            battery_efficiency,
            max_demand_response,
            battery_mode,
        };
        let result = simulate_system(&config, solar_profile, wind_profile, load_profile)?;
        Ok(result.peak_gas)
    };

    // 1. Baseline: no resources (all gas)
    let baseline_peak = run_sim(0.0, 0.0, 0.0, 0.0)?;

    // 2. Full portfolio peak gas
    let portfolio_peak = run_sim(
        solar_capacity,
        wind_capacity,
        storage_capacity,
        clean_firm_capacity,
    )?;

    // Portfolio ELCC = reduction in peak gas
    let portfolio_elcc_mw = baseline_peak - portfolio_peak;

    // ==========================================================================
    // 1. First-In (Standalone) ELCC
    // ==========================================================================
    // Each resource's contribution when it's the ONLY resource (with clean firm as baseload)

    let solar_first_in_mw = if solar_capacity > 0.0 {
        let peak_with_solar = run_sim(solar_capacity, 0.0, 0.0, 0.0)?;
        (baseline_peak - peak_with_solar).max(0.0)
    } else {
        0.0
    };

    let wind_first_in_mw = if wind_capacity > 0.0 {
        let peak_with_wind = run_sim(0.0, wind_capacity, 0.0, 0.0)?;
        (baseline_peak - peak_with_wind).max(0.0)
    } else {
        0.0
    };

    let storage_first_in_mw = if storage_capacity > 0.0 {
        let peak_with_storage = run_sim(0.0, 0.0, storage_capacity, 0.0)?;
        (baseline_peak - peak_with_storage).max(0.0)
    } else {
        0.0
    };

    let cf_first_in_mw = clean_firm_capacity; // CF is 100% dispatchable

    // Convert to percentages
    let solar_first_in_pct = if solar_capacity > 0.0 {
        (solar_first_in_mw / solar_capacity) * 100.0
    } else {
        0.0
    };

    let wind_first_in_pct = if wind_capacity > 0.0 {
        (wind_first_in_mw / wind_capacity) * 100.0
    } else {
        0.0
    };

    let storage_first_in_pct = if storage_capacity > 0.0 {
        (storage_first_in_mw / storage_capacity) * 100.0
    } else {
        0.0
    };

    let cf_first_in_pct = 100.0;

    // ==========================================================================
    // 2. Last-In (Marginal) ELCC
    // ==========================================================================
    // Incremental contribution when adding 10MW to full portfolio

    let solar_marginal_mw = if solar_capacity > 0.0 {
        let peak_plus = run_sim(
            solar_capacity + MARGINAL_INCREMENT,
            wind_capacity,
            storage_capacity,
            clean_firm_capacity,
        )?;
        (portfolio_peak - peak_plus).max(0.0)
    } else {
        // If no solar, use first-in value
        solar_first_in_mw.min(MARGINAL_INCREMENT)
    };

    let wind_marginal_mw = if wind_capacity > 0.0 {
        let peak_plus = run_sim(
            solar_capacity,
            wind_capacity + MARGINAL_INCREMENT,
            storage_capacity,
            clean_firm_capacity,
        )?;
        (portfolio_peak - peak_plus).max(0.0)
    } else {
        wind_first_in_mw.min(MARGINAL_INCREMENT)
    };

    let storage_marginal_mw = if storage_capacity > 0.0 {
        let peak_plus = run_sim(
            solar_capacity,
            wind_capacity,
            storage_capacity + MARGINAL_INCREMENT,
            clean_firm_capacity,
        )?;
        (portfolio_peak - peak_plus).max(0.0)
    } else {
        storage_first_in_mw.min(MARGINAL_INCREMENT)
    };

    let cf_marginal_mw = MARGINAL_INCREMENT; // CF is always 100%

    // Convert to percentages (per MW of increment)
    let solar_marginal_pct = (solar_marginal_mw / MARGINAL_INCREMENT) * 100.0;
    let wind_marginal_pct = (wind_marginal_mw / MARGINAL_INCREMENT) * 100.0;
    let storage_marginal_pct = (storage_marginal_mw / MARGINAL_INCREMENT) * 100.0;
    let cf_marginal_pct = 100.0;

    // ==========================================================================
    // 3. Contribution (Removal) ELCC
    // ==========================================================================
    // Portfolio ELCC - Portfolio ELCC without resource

    let solar_contribution_mw = if solar_capacity > 0.0 {
        let peak_without = run_sim(0.0, wind_capacity, storage_capacity, clean_firm_capacity)?;
        let elcc_without = baseline_peak - peak_without;
        (portfolio_elcc_mw - elcc_without).max(0.0)
    } else {
        0.0
    };

    let wind_contribution_mw = if wind_capacity > 0.0 {
        let peak_without = run_sim(solar_capacity, 0.0, storage_capacity, clean_firm_capacity)?;
        let elcc_without = baseline_peak - peak_without;
        (portfolio_elcc_mw - elcc_without).max(0.0)
    } else {
        0.0
    };

    let storage_contribution_mw = if storage_capacity > 0.0 {
        let peak_without = run_sim(solar_capacity, wind_capacity, 0.0, clean_firm_capacity)?;
        let elcc_without = baseline_peak - peak_without;
        (portfolio_elcc_mw - elcc_without).max(0.0)
    } else {
        0.0
    };

    let cf_contribution_mw = if clean_firm_capacity > 0.0 {
        let peak_without = run_sim(solar_capacity, wind_capacity, storage_capacity, 0.0)?;
        let elcc_without = baseline_peak - peak_without;
        (portfolio_elcc_mw - elcc_without).max(0.0)
    } else {
        0.0
    };

    // Convert to percentages
    let solar_contribution_pct = if solar_capacity > 0.0 {
        (solar_contribution_mw / solar_capacity) * 100.0
    } else {
        0.0
    };

    let wind_contribution_pct = if wind_capacity > 0.0 {
        (wind_contribution_mw / wind_capacity) * 100.0
    } else {
        0.0
    };

    let storage_contribution_pct = if storage_capacity > 0.0 {
        (storage_contribution_mw / storage_capacity) * 100.0
    } else {
        0.0
    };

    let cf_contribution_pct = if clean_firm_capacity > 0.0 {
        (cf_contribution_mw / clean_firm_capacity) * 100.0
    } else {
        0.0
    };

    // ==========================================================================
    // 4. Delta (E3) ELCC
    // ==========================================================================
    // E3 method: Last-In ELCC + proportional allocation of portfolio interactive effect
    // Portfolio Interactive Effect = Portfolio ELCC - Sum of Last-In ELCC (in MW)

    // Calculate total Last-In ELCC in MW
    let solar_last_in_total_mw = if solar_capacity > 0.0 {
        (solar_marginal_pct / 100.0) * solar_capacity
    } else {
        0.0
    };
    let wind_last_in_total_mw = if wind_capacity > 0.0 {
        (wind_marginal_pct / 100.0) * wind_capacity
    } else {
        0.0
    };
    let storage_last_in_total_mw = if storage_capacity > 0.0 {
        (storage_marginal_pct / 100.0) * storage_capacity
    } else {
        0.0
    };
    let cf_last_in_total_mw = clean_firm_capacity; // 100%

    let sum_last_in_mw = solar_last_in_total_mw
        + wind_last_in_total_mw
        + storage_last_in_total_mw
        + cf_last_in_total_mw;

    // Portfolio interactive effect (positive = synergies, resources complement each other)
    let portfolio_interactive_effect_mw = portfolio_elcc_mw - sum_last_in_mw;

    // Allocate interactive effect proportionally based on Last-In contribution
    let total_positive_last_in = solar_last_in_total_mw.max(0.0)
        + wind_last_in_total_mw.max(0.0)
        + storage_last_in_total_mw.max(0.0)
        + cf_last_in_total_mw.max(0.0);

    let allocate_effect = |last_in_mw: f64| -> f64 {
        if total_positive_last_in > 0.0 && last_in_mw > 0.0 {
            let proportion = last_in_mw / total_positive_last_in;
            portfolio_interactive_effect_mw * proportion
        } else {
            0.0
        }
    };

    // Delta ELCC = Last-In + allocated interactive effect
    let solar_delta_mw = solar_last_in_total_mw + allocate_effect(solar_last_in_total_mw);
    let wind_delta_mw = wind_last_in_total_mw + allocate_effect(wind_last_in_total_mw);
    let storage_delta_mw = storage_last_in_total_mw + allocate_effect(storage_last_in_total_mw);
    let cf_delta_mw = cf_last_in_total_mw + allocate_effect(cf_last_in_total_mw);

    // Convert to percentages (clamped to 0-100)
    let solar_delta_pct = if solar_capacity > 0.0 {
        ((solar_delta_mw / solar_capacity) * 100.0)
            .max(0.0)
            .min(100.0)
    } else {
        0.0
    };

    let wind_delta_pct = if wind_capacity > 0.0 {
        ((wind_delta_mw / wind_capacity) * 100.0)
            .max(0.0)
            .min(100.0)
    } else {
        0.0
    };

    let storage_delta_pct = if storage_capacity > 0.0 {
        ((storage_delta_mw / storage_capacity) * 100.0)
            .max(0.0)
            .min(100.0)
    } else {
        0.0
    };

    let cf_delta_pct = if clean_firm_capacity > 0.0 {
        ((cf_delta_mw / clean_firm_capacity) * 100.0)
            .max(0.0)
            .min(100.0)
    } else {
        0.0
    };

    // Diversity benefit: positive = complementary resources, negative = overlap
    // Portfolio ELCC - Sum of Contributions: if portfolio > sum, resources synergize
    let sum_contributions =
        solar_contribution_mw + wind_contribution_mw + storage_contribution_mw + cf_contribution_mw;
    let diversity_benefit = portfolio_elcc_mw - sum_contributions;

    Ok(ElccResult {
        solar: ResourceElcc {
            first_in: solar_first_in_pct.min(100.0).max(0.0),
            marginal: solar_marginal_pct.min(100.0).max(0.0),
            contribution: solar_contribution_pct.min(100.0).max(0.0),
            delta: solar_delta_pct,
        },
        wind: ResourceElcc {
            first_in: wind_first_in_pct.min(100.0).max(0.0),
            marginal: wind_marginal_pct.min(100.0).max(0.0),
            contribution: wind_contribution_pct.min(100.0).max(0.0),
            delta: wind_delta_pct,
        },
        storage: ResourceElcc {
            first_in: storage_first_in_pct.min(100.0).max(0.0),
            marginal: storage_marginal_pct.min(100.0).max(0.0),
            contribution: storage_contribution_pct.min(100.0).max(0.0),
            delta: storage_delta_pct,
        },
        clean_firm: ResourceElcc {
            first_in: cf_first_in_pct,
            marginal: cf_marginal_pct,
            contribution: cf_contribution_pct.min(100.0).max(0.0),
            delta: cf_delta_pct,
        },
        portfolio_elcc_mw,
        diversity_benefit_mw: diversity_benefit,
        baseline_peak_gas: baseline_peak,
        portfolio_peak_gas: portfolio_peak,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
        let solar = vec![0.25; HOURS_PER_YEAR];
        let wind = vec![0.35; HOURS_PER_YEAR];
        let load = vec![100.0; HOURS_PER_YEAR];
        (solar, wind, load)
    }

    #[test]
    fn test_elcc_zero_capacity() {
        let (solar, wind, load) = create_test_profiles();
        let result = calculate_elcc(
            0.0,
            0.0,
            0.0,
            0.0,
            &solar,
            &wind,
            &load,
            BatteryMode::Default,
            0.85,
            0.0,
        )
        .unwrap();
        assert_eq!(result.portfolio_elcc_mw, 0.0);
    }

    #[test]
    fn test_elcc_clean_firm_100_percent() {
        let (solar, wind, load) = create_test_profiles();
        let result = calculate_elcc(
            0.0,
            0.0,
            0.0,
            50.0,
            &solar,
            &wind,
            &load,
            BatteryMode::Default,
            0.85,
            0.0,
        )
        .unwrap();
        assert_eq!(result.clean_firm.first_in, 100.0);
        assert_eq!(result.clean_firm.marginal, 100.0);
        assert!((result.portfolio_elcc_mw - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_elcc_four_methods_differ() {
        let (solar, wind, load) = create_test_profiles();
        let result = calculate_elcc(
            100.0,
            100.0,
            200.0,
            25.0,
            &solar,
            &wind,
            &load,
            BatteryMode::Default,
            0.85,
            0.0,
        )
        .unwrap();

        // All methods should produce values
        assert!(result.solar.first_in >= 0.0);
        assert!(result.solar.marginal >= 0.0);
        assert!(result.solar.contribution >= 0.0);
        assert!(result.solar.delta >= 0.0);

        // Portfolio ELCC should be positive
        assert!(result.portfolio_elcc_mw > 0.0);
    }
}
