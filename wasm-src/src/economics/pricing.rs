/// Market Pricing Module
///
/// Implements four electricity pricing methods:
/// - **ScarcityBased**: SRMC + capacity adder during tight supply, scaled to LCOE
/// - **MarginalCost**: Pure energy-only market (SRMC)
/// - **ORDC**: Operating Reserve Demand Curve pricing
/// - **MarginalPlusCapacity**: Dual revenue stream (energy + capacity payments)
use crate::types::{
    CapacityMarketData, CostParams, ElccResult, OrdcConfig, PricingMethod, PricingResult,
    ResourceValues, SimulationResult, HOURS_PER_YEAR,
};

/// Compute hourly electricity prices
///
/// # Arguments
/// * `sim_result` - Simulation results with hourly generation/load data
/// * `costs` - Cost parameters
/// * `lcoe` - System LCOE $/MWh (for scarcity-based scaling)
/// * `method` - Pricing method to use
/// * `load_profile` - Original load profile MW
/// * `ordc_config` - ORDC configuration (optional, for ORDC method)
/// * `elcc_result` - ELCC results (optional, for capacity market)
/// * `capacities` - (solar, wind, storage, clean_firm) MW capacities
///
/// # Returns
/// * PricingResult with hourly prices and summary statistics
pub fn compute_hourly_prices(
    sim_result: &SimulationResult,
    costs: &CostParams,
    lcoe: f64,
    method: PricingMethod,
    load_profile: &[f64],
    ordc_config: Option<&OrdcConfig>,
    elcc_result: Option<&ElccResult>,
    capacities: (f64, f64, f64, f64),
) -> PricingResult {
    // Calculate SRMC (Short-Run Marginal Cost) of gas
    // SRMC = (heat_rate × gas_price) + variable O&M
    let gas_srmc = (costs.gas_heat_rate * costs.gas_price) + costs.gas_var_om;

    let hourly_prices = match method {
        PricingMethod::ScarcityBased => {
            compute_scarcity_based(sim_result, gas_srmc, lcoe, load_profile)
        }
        PricingMethod::MarginalCost => compute_marginal_cost(sim_result, gas_srmc),
        PricingMethod::Ordc => {
            let config = ordc_config.cloned().unwrap_or_default();
            compute_ordc(sim_result, gas_srmc, load_profile, &config)
        }
        PricingMethod::MarginalPlusCapacity => {
            // Pure SRMC for energy prices; capacity handled separately
            compute_marginal_cost(sim_result, gas_srmc)
        }
    };

    // Calculate statistics
    let peak_price = hourly_prices.iter().cloned().fold(0.0_f64, f64::max);
    let min_price = hourly_prices.iter().cloned().fold(f64::INFINITY, f64::min);

    // Energy-weighted average price
    let total_load: f64 = load_profile.iter().sum();
    let weighted_sum: f64 = hourly_prices
        .iter()
        .zip(load_profile.iter())
        .map(|(p, l)| p * l)
        .sum();
    let average_price = if total_load > 0.0 {
        weighted_sum / total_load
    } else {
        0.0
    };

    // Capacity market data (only for MarginalPlusCapacity)
    let capacity_data = if matches!(method, PricingMethod::MarginalPlusCapacity) {
        Some(compute_capacity_market(
            costs,
            elcc_result,
            capacities,
            total_load,
        ))
    } else {
        None
    };

    PricingResult {
        hourly_prices,
        average_price,
        peak_price,
        min_price,
        capacity_data,
        method,
    }
}

/// Scarcity-based pricing: SRMC + capacity adder during tight supply
/// Scaled so energy-weighted average equals LCOE
fn compute_scarcity_based(
    sim_result: &SimulationResult,
    gas_srmc: f64,
    lcoe: f64,
    load_profile: &[f64],
) -> Vec<f64> {
    let mut prices = vec![0.0; HOURS_PER_YEAR];

    // Find maximum gas generation for scaling
    let max_gas = sim_result
        .gas_generation
        .iter()
        .cloned()
        .fold(0.0_f64, f64::max);

    if max_gas <= 0.0 {
        // No gas needed, use LCOE as flat price
        return vec![lcoe; HOURS_PER_YEAR];
    }

    // Calculate unscaled prices with capacity adder
    // Adder weight = (gas_gen / max_gas)^k where k controls steepness
    let k = 2.0;
    let mut unscaled_prices = vec![0.0; HOURS_PER_YEAR];

    for i in 0..HOURS_PER_YEAR {
        let gas_fraction = sim_result.gas_generation[i] / max_gas;
        let capacity_adder = gas_fraction.powf(k);
        // Base price = SRMC when gas is running, 0 when renewables cover load
        let base_price = if sim_result.gas_generation[i] > 0.0 {
            gas_srmc
        } else {
            gas_srmc * 0.1 // Low price when renewables are marginal
        };
        unscaled_prices[i] = base_price * (1.0 + capacity_adder * 10.0);
    }

    // Scale prices so energy-weighted average equals LCOE
    let total_load: f64 = load_profile.iter().sum();
    let unscaled_weighted: f64 = unscaled_prices
        .iter()
        .zip(load_profile.iter())
        .map(|(p, l)| p * l)
        .sum();

    if unscaled_weighted > 0.0 && total_load > 0.0 {
        let unscaled_avg = unscaled_weighted / total_load;
        let scale_factor = lcoe / unscaled_avg;
        for i in 0..HOURS_PER_YEAR {
            prices[i] = unscaled_prices[i] * scale_factor;
        }
    } else {
        prices = vec![lcoe; HOURS_PER_YEAR];
    }

    prices
}

/// Pure marginal cost pricing: SRMC when gas is marginal
fn compute_marginal_cost(sim_result: &SimulationResult, gas_srmc: f64) -> Vec<f64> {
    let mut prices = vec![0.0; HOURS_PER_YEAR];

    for i in 0..HOURS_PER_YEAR {
        if sim_result.gas_generation[i] > 0.0 {
            // Gas is marginal
            prices[i] = gas_srmc;
        } else if sim_result.curtailed[i] > 0.0 {
            // Excess renewables, price goes to zero or negative
            prices[i] = 0.0;
        } else {
            // Renewables covering load exactly
            prices[i] = gas_srmc * 0.5; // Half of SRMC when supply is tight but no gas
        }
    }

    prices
}

/// ORDC pricing: SRMC + exponential adder when reserves are low
fn compute_ordc(
    sim_result: &SimulationResult,
    gas_srmc: f64,
    load_profile: &[f64],
    config: &OrdcConfig,
) -> Vec<f64> {
    let mut prices = vec![0.0; HOURS_PER_YEAR];

    for i in 0..HOURS_PER_YEAR {
        let load = load_profile[i];
        let reserve_req = load * (config.reserve_requirement / 100.0);

        // Calculate available reserves
        // Reserves = spare gas capacity + battery headroom
        let max_gas = sim_result.peak_gas.max(load); // Assume installed gas = peak need
        let spare_gas = max_gas - sim_result.gas_generation[i];
        let battery_headroom = sim_result.state_of_charge[i]; // Can discharge up to SOC

        let available_reserves = spare_gas + battery_headroom;

        // Reserve deficit
        let deficit = (reserve_req - available_reserves).max(0.0);

        // ORDC adder: exponential function of deficit
        let ordc_adder = if deficit > 0.0 {
            let normalized_deficit = deficit / reserve_req;
            (config.max_price * (config.lambda * normalized_deficit).exp().min(1e10))
                .min(config.max_price)
        } else {
            0.0
        };

        // Base price
        let base_price = if sim_result.gas_generation[i] > 0.0 {
            gas_srmc
        } else {
            0.0
        };

        prices[i] = (base_price + ordc_adder).min(config.max_price);
    }

    prices
}

/// Calculate capacity market payments for MarginalPlusCapacity method
fn compute_capacity_market(
    costs: &CostParams,
    elcc_result: Option<&ElccResult>,
    capacities: (f64, f64, f64, f64),
    total_load: f64,
) -> CapacityMarketData {
    let (solar_cap, wind_cap, storage_cap, cf_cap) = capacities;

    // Calculate capacity market clearing price
    // Based on cost of new entry (CONE) = CRF × gas_capex + fixed_om
    let crf = calculate_crf(costs.discount_rate / 100.0, costs.gas_lifetime as f64);
    let gas_cone = crf * costs.gas_capex * 1000.0 + costs.gas_fixed_om * 1000.0; // $/MW-yr

    let clearing_price = gas_cone;

    // Get ELCC percentages
    let (solar_elcc, wind_elcc, storage_elcc, cf_elcc) = if let Some(elcc) = elcc_result {
        (
            elcc.solar.delta / 100.0,
            elcc.wind.delta / 100.0,
            elcc.storage.delta / 100.0,
            elcc.clean_firm.delta / 100.0,
        )
    } else {
        // Default ELCC estimates if not provided
        (0.20, 0.25, 0.50, 1.0)
    };

    // Storage: 1-hour duration assumed, so MWh = MW
    let storage_mw = storage_cap;

    // Qualified capacity = installed capacity × ELCC%
    let qualified_capacity = ResourceValues {
        solar: solar_cap * solar_elcc,
        wind: wind_cap * wind_elcc,
        storage: storage_mw * storage_elcc,
        clean_firm: cf_cap * cf_elcc,
        gas: 0.0, // Gas doesn't receive capacity payments in clean energy context
    };

    // Annual payments = qualified capacity × clearing price
    let annual_payments = ResourceValues {
        solar: qualified_capacity.solar * clearing_price,
        wind: qualified_capacity.wind * clearing_price,
        storage: qualified_capacity.storage * clearing_price,
        clean_firm: qualified_capacity.clean_firm * clearing_price,
        gas: 0.0,
    };

    // Calculate uniform adder $/MWh
    let total_payments = annual_payments.solar
        + annual_payments.wind
        + annual_payments.storage
        + annual_payments.clean_firm;
    let adder_per_mwh = if total_load > 0.0 {
        total_payments / total_load
    } else {
        0.0
    };

    CapacityMarketData {
        qualified_capacity,
        annual_payments,
        elcc_percentages: ResourceValues {
            solar: solar_elcc * 100.0,
            wind: wind_elcc * 100.0,
            storage: storage_elcc * 100.0,
            clean_firm: cf_elcc * 100.0,
            gas: 100.0, // Gas is 100% dispatchable
        },
        clearing_price,
        adder_per_mwh,
    }
}

/// Calculate Capital Recovery Factor
fn calculate_crf(discount_rate: f64, lifetime: f64) -> f64 {
    if discount_rate <= 0.0 {
        return 1.0 / lifetime;
    }
    let r = discount_rate;
    let n = lifetime;
    (r * (1.0 + r).powf(n)) / ((1.0 + r).powf(n) - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_mock_sim_result() -> SimulationResult {
        let mut result = SimulationResult::new();
        // Create simple pattern: gas needed half the time
        for i in 0..HOURS_PER_YEAR {
            if i % 2 == 0 {
                result.gas_generation[i] = 50.0;
            } else {
                result.gas_generation[i] = 0.0;
                result.curtailed[i] = 10.0;
            }
            result.state_of_charge[i] = 25.0;
        }
        result.peak_gas = 50.0;
        result
    }

    fn create_mock_costs() -> CostParams {
        CostParams::default_costs()
    }

    #[test]
    fn test_scarcity_based_pricing() {
        let sim = create_mock_sim_result();
        let costs = create_mock_costs();
        let load = vec![100.0; HOURS_PER_YEAR];

        let result = compute_hourly_prices(
            &sim,
            &costs,
            75.0, // LCOE
            PricingMethod::ScarcityBased,
            &load,
            None,
            None,
            (100.0, 100.0, 100.0, 50.0),
        );

        // Average should be close to LCOE
        assert!((result.average_price - 75.0).abs() < 1.0);
        assert!(result.peak_price > result.min_price);
    }

    #[test]
    fn test_marginal_cost_pricing() {
        let sim = create_mock_sim_result();
        let costs = create_mock_costs();
        let load = vec![100.0; HOURS_PER_YEAR];

        let result = compute_hourly_prices(
            &sim,
            &costs,
            75.0,
            PricingMethod::MarginalCost,
            &load,
            None,
            None,
            (100.0, 100.0, 100.0, 50.0),
        );

        // Gas SRMC = 7.5 × 4 + 2 = 32 $/MWh
        let expected_srmc = 7.5 * 4.0 + 2.0;
        assert!((result.hourly_prices[0] - expected_srmc).abs() < 0.1); // Gas hour
        assert_eq!(result.hourly_prices[1], 0.0); // Curtailment hour
    }

    #[test]
    fn test_ordc_pricing() {
        let sim = create_mock_sim_result();
        let costs = create_mock_costs();
        let load = vec![100.0; HOURS_PER_YEAR];
        let ordc = OrdcConfig::default();

        let result = compute_hourly_prices(
            &sim,
            &costs,
            75.0,
            PricingMethod::Ordc,
            &load,
            Some(&ordc),
            None,
            (100.0, 100.0, 100.0, 50.0),
        );

        // Peak price should be limited by max_price
        assert!(result.peak_price <= ordc.max_price);
        assert!(result.peak_price > 0.0);
    }

    #[test]
    fn test_capacity_market() {
        let sim = create_mock_sim_result();
        let costs = create_mock_costs();
        let load = vec![100.0; HOURS_PER_YEAR];

        let result = compute_hourly_prices(
            &sim,
            &costs,
            75.0,
            PricingMethod::MarginalPlusCapacity,
            &load,
            None,
            None,
            (100.0, 100.0, 100.0, 50.0),
        );

        assert!(result.capacity_data.is_some());
        let cap_data = result.capacity_data.unwrap();
        assert!(cap_data.clearing_price > 0.0);
        assert!(cap_data.qualified_capacity.clean_firm > 0.0);
        assert!(cap_data.annual_payments.clean_firm > 0.0);
    }
}
