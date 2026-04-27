/// LCOE (Levelized Cost of Energy) Calculator
///
/// Implements investment-grade LCOE calculation matching Python's revenue-based tax treatment:
/// - Revenue-based taxable income calculation
/// - Tax only on positive taxable income
/// - Depreciation monetization via tax equity
/// - Investment Tax Credits (ITCs)
/// - MACRS depreciation schedules
/// - Inflation-adjusted cash flows
/// - Present value discounting
/// - Asset replacement and residual values
use crate::economics::depreciation::calculate_depreciation;
use crate::types::{
    CostParams, DepreciationMethod, LcoeResult, SimulationResult, TechnologyCostBreakdown,
};

/// Calculate present value factor for a given year (discounts to year 0)
#[inline]
fn discount_factor(discount_rate: f64, year: u32) -> f64 {
    1.0 / (1.0 + discount_rate).powi(year as i32 + 1)
}

/// Calculate inflation factor for a given year
#[inline]
fn inflation_factor(inflation_rate: f64, year: u32) -> f64 {
    (1.0 + inflation_rate).powi(year as i32)
}

/// Calculate CAPEX after ITC for an asset
fn calculate_capex_after_itc(capacity: f64, cost_per_kw: f64, itc_rate: f64) -> f64 {
    let base_capex = capacity * 1000.0 * cost_per_kw;
    base_capex * (1.0 - itc_rate)
}

/// Calculate replacement costs and residual value for an asset
/// Returns (total_replacement_pv, residual_value_pv)
fn calculate_replacement_and_residual(
    capex_after_itc: f64,
    asset_lifetime: u32,
    project_lifetime: u32,
    discount_rate: f64,
) -> (f64, f64) {
    if capex_after_itc <= 0.0 {
        return (0.0, 0.0);
    }

    if asset_lifetime >= project_lifetime {
        // Asset outlives project - calculate residual value
        let years_remaining = asset_lifetime - project_lifetime;
        let residual_fraction = years_remaining as f64 / asset_lifetime as f64;
        let residual_value = capex_after_itc * residual_fraction * 0.5; // 50% salvage value
        let residual_pv = residual_value * discount_factor(discount_rate, project_lifetime - 1);
        (0.0, residual_pv)
    } else {
        // Asset needs replacement during project
        let mut total_replacement_pv = 0.0;
        let mut year = asset_lifetime;

        while year < project_lifetime {
            // Replacement at 100% of original capex (after ITC)
            let replacement_pv = capex_after_itc * discount_factor(discount_rate, year - 1);
            total_replacement_pv += replacement_pv;
            year += asset_lifetime;
        }

        (total_replacement_pv, 0.0)
    }
}

/// Calculate depreciation schedule for an asset (using gross CAPEX before ITC)
fn calculate_asset_depreciation(
    gross_capex: f64,
    method: DepreciationMethod,
    project_lifetime: u32,
) -> Vec<f64> {
    calculate_depreciation(gross_capex, method, project_lifetime)
}

/// Calculate present value of energy (no inflation adjustment)
fn calculate_pv_energy(annual_energy: f64, project_lifetime: u32, discount_rate: f64) -> f64 {
    let mut pv_energy = 0.0;
    for year in 0..project_lifetime {
        pv_energy += annual_energy * discount_factor(discount_rate, year);
    }
    pv_energy
}

/// Calculate full system LCOE using Python's revenue-based tax treatment
///
/// This matches the Python lcoe_calculator.py implementation exactly:
/// - Year-by-year calculation of taxable income
/// - Tax only on positive taxable income
/// - Depreciation monetization via tax equity partnerships
/// - Revenue from electricity sales and excess power
///
/// # Arguments
/// * `sim_result` - Simulation results
/// * `solar_capacity` - Solar capacity MW
/// * `wind_capacity` - Wind capacity MW
/// * `storage_capacity` - Storage capacity MWh
/// * `clean_firm_capacity` - Clean firm capacity MW
/// * `costs` - Cost parameters
///
/// # Returns
/// * LcoeResult with all LCOE components
pub fn calculate_lcoe(
    sim_result: &SimulationResult,
    solar_capacity: f64,
    wind_capacity: f64,
    storage_capacity: f64,
    clean_firm_capacity: f64,
    costs: &CostParams,
) -> LcoeResult {
    let mut result = LcoeResult::default();

    let project_lifetime = costs.project_lifetime;
    let discount_rate = costs.discount_rate / 100.0;
    let inflation_rate = costs.inflation_rate / 100.0;
    let tax_rate = costs.tax_rate / 100.0;
    let monetization_rate = costs.monetization_rate / 100.0;

    // === ANNUAL GENERATION VALUES ===
    let annual_solar: f64 = sim_result.solar_out.iter().sum();
    let annual_wind: f64 = sim_result.wind_out.iter().sum();
    let annual_storage_throughput: f64 = sim_result.battery_discharge.iter().sum();
    let annual_clean_firm: f64 = sim_result.clean_firm_generation.iter().sum();
    let annual_gas: f64 = sim_result.gas_generation.iter().sum();
    let annual_load = sim_result.annual_load;
    let curtailed_energy = sim_result.total_curtailment;
    // Planning reserve margin scales every *firm thermal* resource that has
    // forced-outage exposure: gas peakers AND clean firm (nuclear,
    // geothermal). NERC's PRM treats both the same way — you over-build
    // their nameplate so a tripped unit doesn't blow past the LOLE budget.
    //
    // Renewables and storage don't get this multiplier here:
    //   - Solar/wind reliability discount is implicit in capacity factors
    //     (their actual delivered MW already shows up in the dispatch)
    //   - Storage is solid-state with very high availability; ELCC handles
    //     its capacity contribution separately
    //
    // Dispatch (the 8760-hour arrays, annual_clean_firm energy, annual_gas
    // energy) is untouched — this multiplier only sets the capex basis.
    let reserve_factor = 1.0 + costs.reserve_margin / 100.0;
    let gas_capacity = sim_result.peak_gas * reserve_factor;
    let cf_built_capacity = clean_firm_capacity * reserve_factor;
    let ccs_fraction = (costs.ccs_percentage / 100.0).clamp(0.0, 1.0);
    let ccs_energy_penalty = (costs.ccs_energy_penalty / 100.0).max(0.0);
    let ccs_capture_rate = (costs.ccs_capture_rate / 100.0).clamp(0.0, 1.0);

    // === GROSS CAPEX (before ITC) ===
    // gas_capacity and cf_built_capacity already include the reserve margin.
    let solar_capex_gross = solar_capacity * 1000.0 * costs.solar_capex;
    let wind_capex_gross = wind_capacity * 1000.0 * costs.wind_capex;
    let storage_capex_gross = storage_capacity * 1000.0 * costs.storage_capex;
    let cf_capex_gross = cf_built_capacity * 1000.0 * costs.clean_firm_capex;
    let gas_capex_gross = gas_capacity * 1000.0 * costs.gas_capex;
    let ccs_capex_gross = gas_capacity * ccs_fraction * 1000.0 * costs.ccs_capex;

    // === CAPEX AFTER ITC ===
    let solar_capex_after_itc = solar_capex_gross * (1.0 - costs.solar_itc);
    let wind_capex_after_itc = wind_capex_gross * (1.0 - costs.wind_itc);
    let storage_capex_after_itc = storage_capex_gross * (1.0 - costs.storage_itc);
    let cf_capex_after_itc = cf_capex_gross * (1.0 - costs.clean_firm_itc);
    let gas_capex_after_itc = gas_capex_gross; // No ITC for gas
    let ccs_capex_after_itc = ccs_capex_gross; // No ITC for CCS

    // === ITC BENEFITS (for breakdown reporting) ===
    let solar_itc_benefit = solar_capex_gross * costs.solar_itc;
    let wind_itc_benefit = wind_capex_gross * costs.wind_itc;
    let storage_itc_benefit = storage_capex_gross * costs.storage_itc;
    let cf_itc_benefit = cf_capex_gross * costs.clean_firm_itc;

    // === REPLACEMENT COSTS AND RESIDUAL VALUES ===
    let (solar_replacement, solar_residual) = calculate_replacement_and_residual(
        solar_capex_after_itc,
        costs.solar_lifetime,
        project_lifetime,
        discount_rate,
    );
    let (wind_replacement, wind_residual) = calculate_replacement_and_residual(
        wind_capex_after_itc,
        costs.wind_lifetime,
        project_lifetime,
        discount_rate,
    );
    let (storage_replacement, storage_residual) = calculate_replacement_and_residual(
        storage_capex_after_itc,
        costs.storage_lifetime,
        project_lifetime,
        discount_rate,
    );
    let (cf_replacement, cf_residual) = calculate_replacement_and_residual(
        cf_capex_after_itc,
        costs.clean_firm_lifetime,
        project_lifetime,
        discount_rate,
    );
    let (gas_replacement, gas_residual) = calculate_replacement_and_residual(
        gas_capex_after_itc,
        costs.gas_lifetime,
        project_lifetime,
        discount_rate,
    );
    let (ccs_replacement, ccs_residual) = calculate_replacement_and_residual(
        ccs_capex_after_itc,
        costs.gas_lifetime,
        project_lifetime,
        discount_rate,
    );

    // === EFFECTIVE CAPEX (initial + replacements - residual) ===
    let total_capex_after_itc = solar_capex_after_itc
        + wind_capex_after_itc
        + storage_capex_after_itc
        + cf_capex_after_itc
        + gas_capex_after_itc
        + ccs_capex_after_itc;
    let total_replacement = solar_replacement
        + wind_replacement
        + storage_replacement
        + cf_replacement
        + gas_replacement
        + ccs_replacement;
    let total_residual = solar_residual
        + wind_residual
        + storage_residual
        + cf_residual
        + gas_residual
        + ccs_residual;
    let effective_capex = total_capex_after_itc + total_replacement - total_residual;

    // === ANNUAL FIXED O&M ===
    // Fixed O&M scales with built capacity (you maintain every MW you own,
    // operational or in reserve), so CF and gas use the reserve-scaled values.
    let solar_fixed_om = solar_capacity * 1000.0 * costs.solar_fixed_om;
    let wind_fixed_om = wind_capacity * 1000.0 * costs.wind_fixed_om;
    let storage_fixed_om = storage_capacity * 1000.0 * costs.storage_fixed_om;
    let cf_fixed_om = cf_built_capacity * 1000.0 * costs.clean_firm_fixed_om;
    let gas_fixed_om = gas_capacity * 1000.0 * costs.gas_fixed_om;
    let ccs_fixed_om = gas_capacity * ccs_fraction * 1000.0 * costs.ccs_fixed_om;
    let total_fixed_om =
        solar_fixed_om + wind_fixed_om + storage_fixed_om + cf_fixed_om + gas_fixed_om + ccs_fixed_om;

    // === ANNUAL VARIABLE O&M ===
    let solar_var_om = annual_solar * costs.solar_var_om;
    let wind_var_om = annual_wind * costs.wind_var_om;
    let storage_var_om = annual_storage_throughput * costs.storage_var_om;
    let cf_var_om = annual_clean_firm * costs.clean_firm_var_om;
    let gas_var_om = annual_gas * costs.gas_var_om;
    let gas_without_ccs = annual_gas * (1.0 - ccs_fraction);
    let gas_with_ccs = annual_gas * ccs_fraction;
    let ccs_var_om = gas_with_ccs * costs.ccs_var_om;

    // === ANNUAL FUEL COSTS ===
    let cf_fuel = annual_clean_firm * costs.clean_firm_fuel;
    let gas_fuel = gas_without_ccs * costs.gas_heat_rate * costs.gas_price
        + gas_with_ccs * costs.gas_heat_rate * (1.0 + ccs_energy_penalty) * costs.gas_price;

    // === TOTAL ANNUAL OPERATING EXPENSE ===
    let annual_operating_expense = total_fixed_om
        + solar_var_om
        + wind_var_om
        + storage_var_om
        + cf_var_om
        + gas_var_om
        + ccs_var_om
        + cf_fuel
        + gas_fuel;

    // === REVENUE CALCULATION ===
    let annual_revenue = annual_load * costs.electricity_price;
    let annual_excess_revenue = curtailed_energy * costs.excess_power_price;

    // === DEPRECIATION SCHEDULES (using gross CAPEX) ===
    let solar_depreciation = calculate_asset_depreciation(
        solar_capex_gross,
        costs.depreciation_method,
        project_lifetime,
    );
    let wind_depreciation = calculate_asset_depreciation(
        wind_capex_gross,
        costs.depreciation_method,
        project_lifetime,
    );
    let storage_depreciation = calculate_asset_depreciation(
        storage_capex_gross,
        costs.depreciation_method,
        project_lifetime,
    );
    let cf_depreciation =
        calculate_asset_depreciation(cf_capex_gross, costs.depreciation_method, project_lifetime);
    // Use the configured depreciation method for gas to match the Python reference.
    // (Real US tax code typically applies 15-year MACRS to gas; if you want that
    // behavior, switch the default for `depreciation_method` to Macrs15 instead of
    // hardcoding it here, so the choice stays driven by CostParams.)
    let gas_depreciation = calculate_asset_depreciation(
        gas_capex_gross,
        costs.depreciation_method,
        project_lifetime,
    );
    let ccs_depreciation = calculate_asset_depreciation(
        ccs_capex_gross,
        costs.depreciation_method,
        project_lifetime,
    );

    // === YEAR-BY-YEAR TAX CALCULATION (matching Python) ===
    let mut net_opex_pv = 0.0;
    let mut total_monetized_value_pv = 0.0;

    // Track PV of components for breakdown
    let mut pv_fixed_om_total = 0.0;
    let mut pv_var_om_total = 0.0;
    let mut pv_fuel_total = 0.0;
    let mut pv_gross_dep_shield = 0.0;
    let mut pv_tax_payments = 0.0;

    // Per-technology tracking
    let mut solar_pv_fixed_om = 0.0;
    let mut solar_pv_var_om = 0.0;
    let mut solar_pv_dep_shield = 0.0;
    let mut wind_pv_fixed_om = 0.0;
    let mut wind_pv_var_om = 0.0;
    let mut wind_pv_dep_shield = 0.0;
    let mut storage_pv_fixed_om = 0.0;
    let mut storage_pv_var_om = 0.0;
    let mut storage_pv_dep_shield = 0.0;
    let mut cf_pv_fixed_om = 0.0;
    let mut cf_pv_var_om = 0.0;
    let mut cf_pv_fuel = 0.0;
    let mut cf_pv_dep_shield = 0.0;
    let mut gas_pv_fixed_om = 0.0;
    let mut gas_pv_var_om = 0.0;
    let mut gas_pv_fuel = 0.0;
    let mut gas_pv_dep_shield = 0.0;
    let mut ccs_pv_fixed_om = 0.0;
    let mut ccs_pv_var_om = 0.0;
    let mut ccs_pv_dep_shield = 0.0;

    for year in 0..project_lifetime {
        let inf_factor = inflation_factor(inflation_rate, year);
        let disc_factor = discount_factor(discount_rate, year);

        // Inflate operating expenses
        let inflated_opex = annual_operating_expense * inf_factor;
        let inflated_revenue = annual_revenue * inf_factor;
        let inflated_excess_revenue = annual_excess_revenue * inf_factor;

        // Track inflated components
        let inflated_fixed_om = total_fixed_om * inf_factor;
        let inflated_var_om =
            (solar_var_om + wind_var_om + storage_var_om + cf_var_om + gas_var_om + ccs_var_om)
                * inf_factor;
        let inflated_fuel = (cf_fuel + gas_fuel) * inf_factor;

        pv_fixed_om_total += inflated_fixed_om * disc_factor;
        pv_var_om_total += inflated_var_om * disc_factor;
        pv_fuel_total += inflated_fuel * disc_factor;

        // Per-technology O&M
        solar_pv_fixed_om += solar_fixed_om * inf_factor * disc_factor;
        solar_pv_var_om += solar_var_om * inf_factor * disc_factor;
        wind_pv_fixed_om += wind_fixed_om * inf_factor * disc_factor;
        wind_pv_var_om += wind_var_om * inf_factor * disc_factor;
        storage_pv_fixed_om += storage_fixed_om * inf_factor * disc_factor;
        storage_pv_var_om += storage_var_om * inf_factor * disc_factor;
        cf_pv_fixed_om += cf_fixed_om * inf_factor * disc_factor;
        cf_pv_var_om += cf_var_om * inf_factor * disc_factor;
        cf_pv_fuel += cf_fuel * inf_factor * disc_factor;
        gas_pv_fixed_om += gas_fixed_om * inf_factor * disc_factor;
        gas_pv_var_om += gas_var_om * inf_factor * disc_factor;
        gas_pv_fuel += gas_fuel * inf_factor * disc_factor;
        ccs_pv_fixed_om += ccs_fixed_om * inf_factor * disc_factor;
        ccs_pv_var_om += ccs_var_om * inf_factor * disc_factor;

        // Total depreciation for this year
        let year_idx = year as usize;
        let solar_dep = if year_idx < solar_depreciation.len() {
            solar_depreciation[year_idx]
        } else {
            0.0
        };
        let wind_dep = if year_idx < wind_depreciation.len() {
            wind_depreciation[year_idx]
        } else {
            0.0
        };
        let storage_dep = if year_idx < storage_depreciation.len() {
            storage_depreciation[year_idx]
        } else {
            0.0
        };
        let cf_dep = if year_idx < cf_depreciation.len() {
            cf_depreciation[year_idx]
        } else {
            0.0
        };
        let gas_dep = if year_idx < gas_depreciation.len() {
            gas_depreciation[year_idx]
        } else {
            0.0
        };
        let ccs_dep = if year_idx < ccs_depreciation.len() {
            ccs_depreciation[year_idx]
        } else {
            0.0
        };
        let total_dep = solar_dep + wind_dep + storage_dep + cf_dep + gas_dep + ccs_dep;

        // Taxable income before depreciation
        let taxable_before_dep = inflated_revenue + inflated_excess_revenue - inflated_opex;

        // Gross depreciation tax shield
        let gross_dep_shield = total_dep * tax_rate;
        pv_gross_dep_shield += gross_dep_shield * disc_factor;

        // Per-technology depreciation shields (proportional allocation)
        if total_dep > 0.0 {
            solar_pv_dep_shield += (solar_dep / total_dep) * gross_dep_shield * disc_factor;
            wind_pv_dep_shield += (wind_dep / total_dep) * gross_dep_shield * disc_factor;
            storage_pv_dep_shield += (storage_dep / total_dep) * gross_dep_shield * disc_factor;
            cf_pv_dep_shield += (cf_dep / total_dep) * gross_dep_shield * disc_factor;
            gas_pv_dep_shield += (gas_dep / total_dep) * gross_dep_shield * disc_factor;
            ccs_pv_dep_shield += (ccs_dep / total_dep) * gross_dep_shield * disc_factor;
        }

        // Calculate excess depreciation (when depreciation exceeds taxable income before dep)
        let potential_shield = if taxable_before_dep > 0.0 {
            total_dep.min(taxable_before_dep) * tax_rate
        } else {
            0.0
        };
        let excess_shield = (gross_dep_shield - potential_shield).max(0.0);

        // Monetization of excess depreciation (via tax equity)
        let mut monetized_value = 0.0;
        let mut applied_depreciation = total_dep;

        if costs.monetize_excess_depreciation && excess_shield > 0.0 {
            // Direct swap: sell percentage of excess tax shield
            monetized_value = excess_shield * monetization_rate;
            total_monetized_value_pv += monetized_value * disc_factor;

            // Reduce applied depreciation based on what was monetized
            let excess_depreciation = excess_shield / tax_rate;
            applied_depreciation = total_dep - (excess_depreciation * monetization_rate);
        }

        // Taxable income after depreciation
        let taxable_income = taxable_before_dep - applied_depreciation;

        // Tax payment (only on positive taxable income)
        let tax_payment = taxable_income.max(0.0) * tax_rate;
        pv_tax_payments += tax_payment * disc_factor;

        // Annual total cost = OpEx + Tax - Monetized Value - Excess Revenue
        let annual_cost = inflated_opex + tax_payment - monetized_value - inflated_excess_revenue;

        net_opex_pv += annual_cost * disc_factor;
    }

    // === TOTAL PRESENT VALUE COST ===
    let total_pv_cost = effective_capex + net_opex_pv;

    // === PRESENT VALUE OF ENERGY (no inflation) ===
    let pv_energy = calculate_pv_energy(annual_load, project_lifetime, discount_rate);

    result.pv_total_costs = total_pv_cost;
    result.pv_total_energy = pv_energy;

    if pv_energy > 0.0 {
        result.total_lcoe = total_pv_cost / pv_energy;

        // === TECHNOLOGY LCOE CONTRIBUTIONS ===
        // Calculate effective capex per technology
        let solar_effective_capex = solar_capex_after_itc + solar_replacement - solar_residual;
        let wind_effective_capex = wind_capex_after_itc + wind_replacement - wind_residual;
        let storage_effective_capex =
            storage_capex_after_itc + storage_replacement - storage_residual;
        let cf_effective_capex = cf_capex_after_itc + cf_replacement - cf_residual;
        let gas_effective_capex = gas_capex_after_itc + gas_replacement - gas_residual;
        let ccs_effective_capex = ccs_capex_after_itc + ccs_replacement - ccs_residual;

        // Total cost per technology (capex + O&M - depreciation shield)
        // Note: We use gross depreciation shield here for the breakdown
        let solar_pv_cost =
            solar_effective_capex + solar_pv_fixed_om + solar_pv_var_om - solar_pv_dep_shield;
        let wind_pv_cost =
            wind_effective_capex + wind_pv_fixed_om + wind_pv_var_om - wind_pv_dep_shield;
        let storage_pv_cost = storage_effective_capex + storage_pv_fixed_om + storage_pv_var_om
            - storage_pv_dep_shield;
        let cf_pv_cost =
            cf_effective_capex + cf_pv_fixed_om + cf_pv_var_om + cf_pv_fuel - cf_pv_dep_shield;
        let gas_pv_cost =
            gas_effective_capex + gas_pv_fixed_om + gas_pv_var_om + gas_pv_fuel - gas_pv_dep_shield;
        let ccs_pv_cost =
            ccs_effective_capex + ccs_pv_fixed_om + ccs_pv_var_om - ccs_pv_dep_shield;

        result.solar_lcoe = solar_pv_cost / pv_energy;
        result.wind_lcoe = wind_pv_cost / pv_energy;
        result.storage_lcoe = storage_pv_cost / pv_energy;
        result.clean_firm_lcoe = cf_pv_cost / pv_energy;
        result.gas_lcoe = gas_pv_cost / pv_energy;
        result.ccs_lcoe = ccs_pv_cost / pv_energy;

        // === DETAILED BREAKDOWNS ===
        result.solar_breakdown = TechnologyCostBreakdown {
            capex: solar_capex_gross / pv_energy,
            fixed_om: solar_pv_fixed_om / pv_energy,
            var_om: solar_pv_var_om / pv_energy,
            fuel: 0.0,
            itc_benefit: -solar_itc_benefit / pv_energy,
            tax_shield: -solar_pv_dep_shield / pv_energy,
            total: solar_pv_cost / pv_energy,
        };

        result.wind_breakdown = TechnologyCostBreakdown {
            capex: wind_capex_gross / pv_energy,
            fixed_om: wind_pv_fixed_om / pv_energy,
            var_om: wind_pv_var_om / pv_energy,
            fuel: 0.0,
            itc_benefit: -wind_itc_benefit / pv_energy,
            tax_shield: -wind_pv_dep_shield / pv_energy,
            total: wind_pv_cost / pv_energy,
        };

        result.storage_breakdown = TechnologyCostBreakdown {
            capex: storage_capex_gross / pv_energy,
            fixed_om: storage_pv_fixed_om / pv_energy,
            var_om: storage_pv_var_om / pv_energy,
            fuel: 0.0,
            itc_benefit: -storage_itc_benefit / pv_energy,
            tax_shield: -storage_pv_dep_shield / pv_energy,
            total: storage_pv_cost / pv_energy,
        };

        result.clean_firm_breakdown = TechnologyCostBreakdown {
            capex: cf_capex_gross / pv_energy,
            fixed_om: cf_pv_fixed_om / pv_energy,
            var_om: cf_pv_var_om / pv_energy,
            fuel: cf_pv_fuel / pv_energy,
            itc_benefit: -cf_itc_benefit / pv_energy,
            tax_shield: -cf_pv_dep_shield / pv_energy,
            total: cf_pv_cost / pv_energy,
        };

        result.gas_breakdown = TechnologyCostBreakdown {
            capex: gas_capex_gross / pv_energy,
            fixed_om: gas_pv_fixed_om / pv_energy,
            var_om: gas_pv_var_om / pv_energy,
            fuel: gas_pv_fuel / pv_energy,
            itc_benefit: 0.0, // No ITC for gas
            tax_shield: -gas_pv_dep_shield / pv_energy,
            total: gas_pv_cost / pv_energy,
        };

        result.ccs_breakdown = TechnologyCostBreakdown {
            capex: ccs_capex_gross / pv_energy,
            fixed_om: ccs_pv_fixed_om / pv_energy,
            var_om: ccs_pv_var_om / pv_energy,
            fuel: 0.0,
            itc_benefit: 0.0,
            tax_shield: -ccs_pv_dep_shield / pv_energy,
            total: ccs_pv_cost / pv_energy,
        };
    }

    // === EMISSIONS ===
    // Direct gas combustion emissions with optional CCS on a share of gas generation.
    let fuel_without_ccs = gas_without_ccs * costs.gas_heat_rate;
    let fuel_with_ccs = gas_with_ccs * costs.gas_heat_rate * (1.0 + ccs_energy_penalty);
    let gas_combustion = fuel_without_ccs * costs.gas_emissions_factor
        + fuel_with_ccs * costs.gas_emissions_factor * (1.0 - ccs_capture_rate);

    // Methane leakage (converted to CO2eq).
    // Python reference uses 19.2 kg CH4 per MMBtu of natural gas (multi_test.py:99).
    // Methane leakage occurs upstream and is NOT captured by CCS, so it sees the
    // full fuel volume including the CCS-equipped share.
    let gas_volume = fuel_without_ccs + fuel_with_ccs; // MMBtu
    const METHANE_KG_PER_MMBTU: f64 = 19.2;
    let leaked_methane = gas_volume * (costs.gas_leakage_rate / 100.0) * METHANE_KG_PER_MMBTU;
    let methane_emissions = leaked_methane * costs.methane_gwp; // kg CO2eq

    // Embodied emissions (annual contribution).
    // `annual_solar` etc. are in MWh; embodied factors are in g CO2eq per kWh.
    // MWh × 1000 = kWh, then × (g/kWh) = g, then ÷ 1000 = kg → the 1000s cancel,
    // so the result in kg per year is simply `annual_MWh * (g/kWh)`.
    let solar_embodied = annual_solar * costs.solar_embodied_emissions; // kg CO2eq/year
    let wind_embodied = annual_wind * costs.wind_embodied_emissions;
    let cf_embodied = annual_clean_firm * costs.clean_firm_embodied_emissions;
    // storage_capacity is in MWh; battery_embodied_emissions is kg CO2eq per kWh,
    // so multiply by 1000 to land in kg/year of amortized embodied emissions.
    let battery_embodied = storage_capacity * 1000.0 * costs.battery_embodied_emissions
        / project_lifetime as f64;

    let total_emissions = gas_combustion
        + methane_emissions
        + solar_embodied
        + wind_embodied
        + cf_embodied
        + battery_embodied;
    let annual_energy_kwh = annual_load * 1000.0; // MWh to kWh

    result.emissions_intensity = if annual_energy_kwh > 0.0 {
        (total_emissions / annual_energy_kwh) * 1000.0 // g CO2/kWh
    } else {
        0.0
    };

    // === LAND USE ===
    // Firm thermal resources (gas, CF) use their reserve-scaled built capacity
    // since the reserve plant takes physical land too. Variable resources
    // (solar, wind) use nameplate.
    let solar_land_direct = solar_capacity * costs.solar_land_direct;
    let wind_land_direct = wind_capacity * costs.wind_land_direct;
    let cf_land_direct = cf_built_capacity * costs.clean_firm_land_direct;
    let gas_land_direct = gas_capacity * costs.gas_land_direct;

    result.direct_land_use =
        solar_land_direct + wind_land_direct + cf_land_direct + gas_land_direct;

    // Total land use (includes indirect: wind spacing, exclusion zones)
    // Solar: only direct (no significant indirect effects)
    // Wind: includes spacing between turbines
    // Clean firm: includes indirect impacts (exclusion zones, mining)
    // Gas: only direct (minimal indirect effects)
    let solar_land_total = solar_land_direct;
    let wind_land_total = wind_capacity * costs.wind_land_total;
    let cf_land_total = cf_built_capacity * costs.clean_firm_land_total;
    let gas_land_total = gas_land_direct;

    result.total_land_use = solar_land_total + wind_land_total + cf_land_total + gas_land_total;

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{SimulationResult, HOURS_PER_YEAR};

    fn create_test_sim_result() -> SimulationResult {
        let mut result = SimulationResult::new();
        for i in 0..HOURS_PER_YEAR {
            result.solar_out[i] = 25.0;
            result.wind_out[i] = 35.0;
            result.clean_firm_generation[i] = 10.0;
            result.gas_generation[i] = 30.0;
        }
        result.annual_load = 100.0 * HOURS_PER_YEAR as f64;
        result.peak_gas = 50.0;
        result.total_curtailment = 1000.0; // Some curtailment
        result
    }

    #[test]
    fn test_lcoe_basic() {
        let sim_result = create_test_sim_result();
        let costs = CostParams::default_costs();

        let lcoe = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs);

        assert!(lcoe.total_lcoe > 0.0);
        assert!(lcoe.solar_lcoe >= 0.0);
        assert!(lcoe.wind_lcoe >= 0.0);
        assert!(lcoe.storage_lcoe >= 0.0);
        assert!(lcoe.clean_firm_lcoe >= 0.0);
        assert!(lcoe.gas_lcoe >= 0.0);
        assert!(lcoe.ccs_lcoe >= 0.0);
    }

    #[test]
    fn test_lcoe_zero_capacity() {
        let mut sim_result = SimulationResult::new();
        sim_result.annual_load = 100.0 * HOURS_PER_YEAR as f64;
        for i in 0..HOURS_PER_YEAR {
            sim_result.gas_generation[i] = 100.0;
        }
        sim_result.peak_gas = 100.0;

        let costs = CostParams::default_costs();
        let lcoe = calculate_lcoe(&sim_result, 0.0, 0.0, 0.0, 0.0, &costs);

        assert!(lcoe.total_lcoe > 0.0);
        assert_eq!(lcoe.solar_lcoe, 0.0);
        assert_eq!(lcoe.wind_lcoe, 0.0);
        assert_eq!(lcoe.storage_lcoe, 0.0);
        assert_eq!(lcoe.clean_firm_lcoe, 0.0);
        assert!(lcoe.gas_lcoe > 0.0);
        assert_eq!(lcoe.ccs_lcoe, 0.0);
    }

    #[test]
    fn test_lcoe_itc_reduces_cost() {
        let sim_result = create_test_sim_result();

        let mut costs_no_itc = CostParams::default_costs();
        costs_no_itc.solar_itc = 0.0;

        let mut costs_with_itc = CostParams::default_costs();
        costs_with_itc.solar_itc = 0.30;

        let lcoe_no_itc = calculate_lcoe(&sim_result, 100.0, 0.0, 0.0, 0.0, &costs_no_itc);
        let lcoe_with_itc = calculate_lcoe(&sim_result, 100.0, 0.0, 0.0, 0.0, &costs_with_itc);

        assert!(lcoe_with_itc.solar_lcoe < lcoe_no_itc.solar_lcoe);
    }

    #[test]
    fn test_revenue_based_tax() {
        // Test that tax is only paid on positive income
        let sim_result = create_test_sim_result();
        let mut costs = CostParams::default_costs();
        costs.electricity_price = 10.0; // Very low price = low revenue

        let lcoe_low_price = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs);

        costs.electricity_price = 100.0; // High price = high revenue
        let lcoe_high_price = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs);

        // LCOE should be higher when electricity price is higher (more tax paid)
        // But the effect should be relatively small compared to capital costs
        assert!(lcoe_high_price.total_lcoe != lcoe_low_price.total_lcoe);
    }

    #[test]
    fn test_monetization() {
        let sim_result = create_test_sim_result();

        let mut costs_no_monetize = CostParams::default_costs();
        costs_no_monetize.monetize_excess_depreciation = false;
        costs_no_monetize.electricity_price = 20.0; // Low revenue to create excess depreciation

        let mut costs_with_monetize = CostParams::default_costs();
        costs_with_monetize.monetize_excess_depreciation = true;
        costs_with_monetize.monetization_rate = 50.0;
        costs_with_monetize.electricity_price = 20.0;

        let lcoe_no = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_no_monetize);
        let lcoe_with = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_with_monetize);

        // Monetization should reduce LCOE when there's excess depreciation
        assert!(lcoe_with.total_lcoe <= lcoe_no.total_lcoe);
    }

    #[test]
    fn test_excess_power_revenue() {
        let mut sim_result = create_test_sim_result();
        sim_result.total_curtailment = 50000.0; // Significant curtailment

        let mut costs_no_excess = CostParams::default_costs();
        costs_no_excess.excess_power_price = 0.0;

        let mut costs_with_excess = CostParams::default_costs();
        costs_with_excess.excess_power_price = 20.0;

        let lcoe_no = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_no_excess);
        let lcoe_with = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_with_excess);

        // Selling excess power should reduce LCOE
        assert!(lcoe_with.total_lcoe < lcoe_no.total_lcoe);
    }

    #[test]
    fn test_emissions_calculation() {
        let sim_result = create_test_sim_result();
        let costs = CostParams::default_costs();

        let lcoe = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs);

        // Should have positive emissions (some gas generation)
        assert!(lcoe.emissions_intensity > 0.0);
    }

    #[test]
    fn test_ccs_adds_visible_cost_breakdown() {
        let sim_result = create_test_sim_result();

        let mut costs = CostParams::default_costs();
        costs.ccs_percentage = 100.0;

        let lcoe = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs);

        assert!(lcoe.ccs_lcoe > 0.0);
        assert!(lcoe.ccs_breakdown.total > 0.0);
        assert!(lcoe.total_lcoe > lcoe.gas_lcoe);
    }

    #[test]
    fn test_ccs_reduces_emissions_intensity() {
        let sim_result = create_test_sim_result();

        let costs_no_ccs = CostParams::default_costs();

        let mut costs_with_ccs = CostParams::default_costs();
        costs_with_ccs.ccs_percentage = 100.0;
        costs_with_ccs.ccs_capture_rate = 90.0;

        let no_ccs = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_no_ccs);
        let with_ccs = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_with_ccs);

        assert!(with_ccs.emissions_intensity < no_ccs.emissions_intensity);
    }

    #[test]
    fn test_land_use_calculation() {
        let sim_result = create_test_sim_result();
        let mut costs = CostParams::default_costs();
        // Test pure land-use math; reserve margin is exercised separately.
        costs.reserve_margin = 0.0;

        let lcoe = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs);

        assert!(lcoe.direct_land_use > 0.0);
        assert!(lcoe.total_land_use > 0.0);
        assert!(lcoe.total_land_use >= lcoe.direct_land_use);

        // Direct land use: 100 * 6.5 (solar) + 100 * 1.25 (wind) + 10 * 1.0 (CF) + 50 * 0.19 (gas)
        let expected_direct = 100.0 * 6.5 + 100.0 * 1.25 + 10.0 * 1.0 + 50.0 * 0.19;
        assert!((lcoe.direct_land_use - expected_direct).abs() < 0.01);

        // Total land use: 100 * 6.5 (solar) + 100 * 50.0 (wind total) + 10 * 1.0 (CF total) + 50 * 0.19 (gas)
        let expected_total = 100.0 * 6.5 + 100.0 * 50.0 + 10.0 * 1.0 + 50.0 * 0.19;
        assert!((lcoe.total_land_use - expected_total).abs() < 0.01);
    }

    #[test]
    fn test_reserve_margin_scales_firm_thermal() {
        let sim_result = create_test_sim_result();

        let mut costs_no_reserve = CostParams::default_costs();
        costs_no_reserve.reserve_margin = 0.0;
        let mut costs_with_reserve = CostParams::default_costs();
        costs_with_reserve.reserve_margin = 15.0;

        let no_res = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_no_reserve);
        let with_res = calculate_lcoe(&sim_result, 100.0, 100.0, 50.0, 10.0, &costs_with_reserve);

        // Both gas and clean firm are firm-thermal → both scale.
        assert!(with_res.gas_lcoe > no_res.gas_lcoe);
        assert!(with_res.clean_firm_lcoe > no_res.clean_firm_lcoe);
        assert!(with_res.total_lcoe > no_res.total_lcoe);

        // Capex line items scale exactly 1.15× — single-multiplier path.
        let gas_ratio = with_res.gas_breakdown.capex / no_res.gas_breakdown.capex;
        let cf_ratio = with_res.clean_firm_breakdown.capex / no_res.clean_firm_breakdown.capex;
        assert!((gas_ratio - 1.15).abs() < 1e-6, "gas capex 1.15× expected, got {}", gas_ratio);
        assert!((cf_ratio - 1.15).abs() < 1e-6, "CF capex 1.15× expected, got {}", cf_ratio);

        // Variable resources (solar, wind, storage) are NOT scaled — their
        // reliability discount lives in capacity factor / ELCC, not here.
        let solar_ratio = with_res.solar_breakdown.capex / no_res.solar_breakdown.capex;
        let wind_ratio = with_res.wind_breakdown.capex / no_res.wind_breakdown.capex;
        let storage_ratio = with_res.storage_breakdown.capex / no_res.storage_breakdown.capex;
        assert!((solar_ratio - 1.0).abs() < 1e-6);
        assert!((wind_ratio - 1.0).abs() < 1e-6);
        assert!((storage_ratio - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_discount_factor() {
        assert!((discount_factor(0.07, 0) - 0.9346).abs() < 0.001);
        assert!((discount_factor(0.07, 1) - 0.8734).abs() < 0.001);
    }

    #[test]
    fn test_inflation_factor() {
        assert!((inflation_factor(0.02, 0) - 1.0).abs() < 0.001);
        assert!((inflation_factor(0.02, 1) - 1.02).abs() < 0.001);
        assert!((inflation_factor(0.02, 10) - 1.2190).abs() < 0.001);
    }
}
