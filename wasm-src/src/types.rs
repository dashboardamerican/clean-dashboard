use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

/// Number of hours in a year
pub const HOURS_PER_YEAR: usize = 8760;

fn default_battery_efficiency() -> f64 {
    0.85
}

/// Battery dispatch mode
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BatteryMode {
    /// Water-fill algorithm prioritizes shaving highest peaks
    Default,
    /// Binary search finds optimal constant peak shaving line
    PeakShaver,
    /// Two-pass: peak shaving + opportunistic dispatch
    Hybrid,
}

impl Default for BatteryMode {
    fn default() -> Self {
        BatteryMode::Default
    }
}

/// Configuration for a simulation run
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Solar capacity in MW
    pub solar_capacity: f64,
    /// Wind capacity in MW
    pub wind_capacity: f64,
    /// Storage capacity in MWh
    pub storage_capacity: f64,
    /// Clean firm capacity in MW (constant output)
    pub clean_firm_capacity: f64,
    /// Battery round-trip efficiency (0-1)
    pub battery_efficiency: f64,
    /// Maximum demand response in MW
    pub max_demand_response: f64,
    /// Battery dispatch mode
    pub battery_mode: BatteryMode,
}

#[wasm_bindgen]
impl SimulationConfig {
    #[wasm_bindgen(constructor)]
    pub fn new(
        solar_capacity: f64,
        wind_capacity: f64,
        storage_capacity: f64,
        clean_firm_capacity: f64,
        battery_efficiency: f64,
        max_demand_response: f64,
        battery_mode: BatteryMode,
    ) -> Self {
        Self {
            solar_capacity,
            wind_capacity,
            storage_capacity,
            clean_firm_capacity,
            battery_efficiency,
            max_demand_response,
            battery_mode,
        }
    }

    pub fn with_defaults() -> Self {
        Self {
            solar_capacity: 0.0,
            wind_capacity: 0.0,
            storage_capacity: 0.0,
            clean_firm_capacity: 0.0,
            battery_efficiency: 0.85,
            max_demand_response: 0.0,
            battery_mode: BatteryMode::Default,
        }
    }
}

/// Results from a simulation run
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Solar generation MW (8760 hours)
    pub solar_out: Vec<f64>,
    /// Wind generation MW (8760 hours)
    pub wind_out: Vec<f64>,
    /// Battery charging MW (8760 hours)
    pub battery_charge: Vec<f64>,
    /// Battery discharging MW (8760 hours)
    pub battery_discharge: Vec<f64>,
    /// Gas generation MW (8760 hours)
    pub gas_generation: Vec<f64>,
    /// Curtailed renewable MW (8760 hours)
    pub curtailed: Vec<f64>,
    /// Clean energy delivered to load MW (8760 hours)
    #[serde(alias = "renewable_delivered")]
    pub clean_delivered: Vec<f64>,
    /// Clean firm generation MW (8760 hours)
    pub clean_firm_generation: Vec<f64>,
    /// Demand response deployed MW (8760 hours)
    pub demand_response: Vec<f64>,
    /// Gas used for battery charging MW (8760 hours)
    pub gas_for_charging: Vec<f64>,
    /// State of charge MW (8760 hours)
    pub state_of_charge: Vec<f64>,

    // Scalar metrics
    /// Total annual renewable generation MWh
    pub annual_renewable_gen: f64,
    /// Total annual load MWh
    pub annual_load: f64,
    /// Peak gas generation MW
    pub peak_gas: f64,
    /// Clean match percentage (0-100)
    pub clean_match_pct: f64,
    /// Total curtailment MWh
    pub total_curtailment: f64,
}

impl SimulationResult {
    pub fn new() -> Self {
        Self {
            solar_out: vec![0.0; HOURS_PER_YEAR],
            wind_out: vec![0.0; HOURS_PER_YEAR],
            battery_charge: vec![0.0; HOURS_PER_YEAR],
            battery_discharge: vec![0.0; HOURS_PER_YEAR],
            gas_generation: vec![0.0; HOURS_PER_YEAR],
            curtailed: vec![0.0; HOURS_PER_YEAR],
            clean_delivered: vec![0.0; HOURS_PER_YEAR],
            clean_firm_generation: vec![0.0; HOURS_PER_YEAR],
            demand_response: vec![0.0; HOURS_PER_YEAR],
            gas_for_charging: vec![0.0; HOURS_PER_YEAR],
            state_of_charge: vec![0.0; HOURS_PER_YEAR],
            annual_renewable_gen: 0.0,
            annual_load: 0.0,
            peak_gas: 0.0,
            clean_match_pct: 0.0,
            total_curtailment: 0.0,
        }
    }
}

impl Default for SimulationResult {
    fn default() -> Self {
        Self::new()
    }
}

/// MACRS depreciation method
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepreciationMethod {
    Macrs5,
    Macrs15,
    StraightLine,
}

impl Default for DepreciationMethod {
    fn default() -> Self {
        DepreciationMethod::Macrs5
    }
}

/// Cost parameters for LCOE calculation
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostParams {
    // CAPEX ($/kW or $/kWh)
    pub solar_capex: f64,
    pub wind_capex: f64,
    pub storage_capex: f64,
    pub clean_firm_capex: f64,
    pub gas_capex: f64,

    // Fixed O&M ($/kW-year or $/kWh-year)
    pub solar_fixed_om: f64,
    pub wind_fixed_om: f64,
    pub storage_fixed_om: f64,
    pub clean_firm_fixed_om: f64,
    pub gas_fixed_om: f64,

    // Variable O&M ($/MWh)
    pub solar_var_om: f64,
    pub wind_var_om: f64,
    pub storage_var_om: f64,
    pub clean_firm_var_om: f64,
    pub gas_var_om: f64,

    // Fuel costs
    pub gas_price: f64,       // $/MMBtu
    pub clean_firm_fuel: f64, // $/MWh

    // Financial
    pub discount_rate: f64,                 // % (e.g., 7 for 7%)
    pub project_lifetime: u32,              // years
    pub inflation_rate: f64,                // % (e.g., 2 for 2%)
    pub tax_rate: f64,                      // % (e.g., 21 for 21%)
    pub electricity_price: f64,             // $/MWh (for revenue/tax calculations)
    pub excess_power_price: f64,            // $/MWh (for selling curtailed power)
    pub monetize_excess_depreciation: bool, // Enable depreciation monetization
    pub monetization_rate: f64,             // % of excess depreciation to monetize

    // Asset lifetimes
    pub solar_lifetime: u32,
    pub wind_lifetime: u32,
    pub storage_lifetime: u32,
    pub clean_firm_lifetime: u32,
    pub gas_lifetime: u32,

    // ITCs (0-1)
    pub solar_itc: f64,
    pub wind_itc: f64,
    pub storage_itc: f64,
    pub clean_firm_itc: f64,

    // Depreciation
    pub depreciation_method: DepreciationMethod,

    // Fuel efficiency
    pub gas_heat_rate: f64, // MMBtu/MWh

    // Emissions
    pub gas_emissions_factor: f64,     // kg CO2/MMBtu
    pub solar_embodied_emissions: f64, // g CO2eq/kWh
    pub wind_embodied_emissions: f64,
    pub clean_firm_embodied_emissions: f64,
    pub battery_embodied_emissions: f64, // kg CO2eq/kWh
    pub gas_leakage_rate: f64,           // %
    pub methane_gwp: f64,                // GWP multiplier

    // Land use (acres/MW)
    pub solar_land_direct: f64,
    pub wind_land_direct: f64,
    pub wind_land_total: f64,
    pub clean_firm_land_direct: f64,
    pub clean_firm_land_total: f64,
    pub gas_land_direct: f64,

    // CCS
    pub ccs_percentage: f64,
    pub ccs_capex: f64,
    pub ccs_fixed_om: f64,
    pub ccs_var_om: f64,
    pub ccs_energy_penalty: f64,
    pub ccs_capture_rate: f64,
}

impl CostParams {
    pub fn default_costs() -> Self {
        Self {
            // CAPEX
            solar_capex: 1000.0,
            wind_capex: 1200.0,
            storage_capex: 300.0,
            clean_firm_capex: 5000.0,
            gas_capex: 1200.0,

            // Fixed O&M
            solar_fixed_om: 15.0,
            wind_fixed_om: 40.0,
            storage_fixed_om: 10.0,
            clean_firm_fixed_om: 60.0,
            gas_fixed_om: 20.0,

            // Variable O&M
            solar_var_om: 0.0,
            wind_var_om: 0.0,
            storage_var_om: 5.0,
            clean_firm_var_om: 10.0,
            gas_var_om: 2.0,

            // Fuel
            gas_price: 4.0,
            clean_firm_fuel: 20.0,

            // Financial
            discount_rate: 7.0,
            project_lifetime: 20,
            inflation_rate: 2.0,
            tax_rate: 21.0,
            electricity_price: 50.0,
            excess_power_price: 0.0,
            monetize_excess_depreciation: false,
            monetization_rate: 50.0,

            // Asset lifetimes
            solar_lifetime: 30,
            wind_lifetime: 30,
            storage_lifetime: 15,
            clean_firm_lifetime: 60,
            gas_lifetime: 40,

            // ITCs
            solar_itc: 0.0,
            wind_itc: 0.0,
            storage_itc: 0.0,
            clean_firm_itc: 0.0,

            // Depreciation
            depreciation_method: DepreciationMethod::Macrs5,

            // Fuel efficiency
            gas_heat_rate: 7.5,

            // Emissions
            gas_emissions_factor: 53.1,
            solar_embodied_emissions: 30.0,
            wind_embodied_emissions: 11.0,
            clean_firm_embodied_emissions: 11.0,
            battery_embodied_emissions: 100.0,
            gas_leakage_rate: 1.0,
            methane_gwp: 29.8, // GWP100

            // Land use
            solar_land_direct: 6.5,
            wind_land_direct: 1.25,
            wind_land_total: 50.0,
            clean_firm_land_direct: 1.0,
            clean_firm_land_total: 1.0,
            gas_land_direct: 0.19,

            // CCS
            ccs_percentage: 0.0,
            ccs_capex: 2500.0,
            ccs_fixed_om: 50.0,
            ccs_var_om: 15.0,
            ccs_energy_penalty: 20.0,
            ccs_capture_rate: 100.0,
        }
    }
}

impl Default for CostParams {
    fn default() -> Self {
        Self::default_costs()
    }
}

/// Cost breakdown for a single technology
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct TechnologyCostBreakdown {
    /// Capital expenditure $/MWh
    pub capex: f64,
    /// Fixed O&M $/MWh
    pub fixed_om: f64,
    /// Variable O&M $/MWh
    pub var_om: f64,
    /// Fuel cost $/MWh
    pub fuel: f64,
    /// ITC benefit (negative = reduces cost) $/MWh
    pub itc_benefit: f64,
    /// Depreciation tax shield (negative = reduces cost) $/MWh
    pub tax_shield: f64,
    /// Total for this technology $/MWh
    pub total: f64,
}

/// LCOE calculation result with detailed breakdown
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LcoeResult {
    /// Total system LCOE in $/MWh
    pub total_lcoe: f64,
    /// Solar LCOE contribution $/MWh
    pub solar_lcoe: f64,
    /// Wind LCOE contribution $/MWh
    pub wind_lcoe: f64,
    /// Storage LCOE contribution $/MWh
    pub storage_lcoe: f64,
    /// Clean firm LCOE contribution $/MWh
    pub clean_firm_lcoe: f64,
    /// Gas LCOE contribution $/MWh
    pub gas_lcoe: f64,
    /// CCS LCOE contribution $/MWh
    pub ccs_lcoe: f64,

    /// Total present value of costs $
    pub pv_total_costs: f64,
    /// Total present value of energy MWh
    pub pv_total_energy: f64,

    /// Emissions intensity g CO2/kWh
    pub emissions_intensity: f64,
    /// Direct land use acres (physical footprint only)
    pub direct_land_use: f64,
    /// Total land use acres (includes indirect: wind spacing, exclusion zones)
    pub total_land_use: f64,

    // Detailed cost breakdowns per technology
    /// Solar cost breakdown
    pub solar_breakdown: TechnologyCostBreakdown,
    /// Wind cost breakdown
    pub wind_breakdown: TechnologyCostBreakdown,
    /// Storage cost breakdown
    pub storage_breakdown: TechnologyCostBreakdown,
    /// Clean firm cost breakdown
    pub clean_firm_breakdown: TechnologyCostBreakdown,
    /// Gas cost breakdown
    pub gas_breakdown: TechnologyCostBreakdown,
    /// CCS cost breakdown
    pub ccs_breakdown: TechnologyCostBreakdown,
}

impl Default for LcoeResult {
    fn default() -> Self {
        Self {
            total_lcoe: 0.0,
            solar_lcoe: 0.0,
            wind_lcoe: 0.0,
            storage_lcoe: 0.0,
            clean_firm_lcoe: 0.0,
            gas_lcoe: 0.0,
            ccs_lcoe: 0.0,
            pv_total_costs: 0.0,
            pv_total_energy: 0.0,
            emissions_intensity: 0.0,
            direct_land_use: 0.0,
            total_land_use: 0.0,
            solar_breakdown: TechnologyCostBreakdown::default(),
            wind_breakdown: TechnologyCostBreakdown::default(),
            storage_breakdown: TechnologyCostBreakdown::default(),
            clean_firm_breakdown: TechnologyCostBreakdown::default(),
            gas_breakdown: TechnologyCostBreakdown::default(),
            ccs_breakdown: TechnologyCostBreakdown::default(),
        }
    }
}

/// Optimizer configuration
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizerConfig {
    /// Target clean match percentage (0-100)
    pub target_clean_match: f64,
    /// Enable solar in optimization
    pub enable_solar: bool,
    /// Enable wind in optimization
    pub enable_wind: bool,
    /// Enable storage in optimization
    pub enable_storage: bool,
    /// Enable clean firm in optimization
    pub enable_clean_firm: bool,
    /// Maximum solar capacity MW
    pub max_solar: f64,
    /// Maximum wind capacity MW
    pub max_wind: f64,
    /// Maximum storage capacity MWh
    pub max_storage: f64,
    /// Maximum clean firm capacity MW
    pub max_clean_firm: f64,
    /// Battery round-trip efficiency used during optimizer evaluations
    #[serde(default = "default_battery_efficiency")]
    pub battery_efficiency: f64,
    /// Maximum demand response used during optimizer evaluations
    #[serde(default)]
    pub max_demand_response: f64,
}

impl Default for OptimizerConfig {
    fn default() -> Self {
        Self {
            target_clean_match: 80.0,
            enable_solar: true,
            enable_wind: true,
            enable_storage: true,
            enable_clean_firm: true,
            max_solar: 1000.0,
            max_wind: 700.0,
            max_storage: 2400.0,
            max_clean_firm: 200.0,
            battery_efficiency: default_battery_efficiency(),
            max_demand_response: 0.0,
        }
    }
}

/// Optimizer result
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OptimizerResult {
    /// Optimal solar capacity MW
    pub solar_capacity: f64,
    /// Optimal wind capacity MW
    pub wind_capacity: f64,
    /// Optimal storage capacity MWh
    pub storage_capacity: f64,
    /// Optimal clean firm capacity MW
    pub clean_firm_capacity: f64,
    /// Achieved clean match percentage
    pub achieved_clean_match: f64,
    /// Resulting LCOE $/MWh
    pub lcoe: f64,
    /// Number of evaluations
    pub num_evaluations: u32,
    /// Optimization successful
    pub success: bool,
}

impl Default for OptimizerResult {
    fn default() -> Self {
        Self {
            solar_capacity: 0.0,
            wind_capacity: 0.0,
            storage_capacity: 0.0,
            clean_firm_capacity: 0.0,
            achieved_clean_match: 0.0,
            lcoe: 0.0,
            num_evaluations: 0,
            success: false,
        }
    }
}

/// Zone profile data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ZoneData {
    /// Zone name
    pub name: String,
    /// Solar capacity factors (8760 hours, 0-1)
    pub solar_profile: Vec<f64>,
    /// Wind capacity factors (8760 hours, 0-1)
    pub wind_profile: Vec<f64>,
    /// Load profile MW (8760 hours)
    pub load_profile: Vec<f64>,
}

impl ZoneData {
    pub fn validate(&self) -> Result<(), String> {
        if self.solar_profile.len() != HOURS_PER_YEAR {
            return Err(format!(
                "Solar profile has {} hours, expected {}",
                self.solar_profile.len(),
                HOURS_PER_YEAR
            ));
        }
        if self.wind_profile.len() != HOURS_PER_YEAR {
            return Err(format!(
                "Wind profile has {} hours, expected {}",
                self.wind_profile.len(),
                HOURS_PER_YEAR
            ));
        }
        if self.load_profile.len() != HOURS_PER_YEAR {
            return Err(format!(
                "Load profile has {} hours, expected {}",
                self.load_profile.len(),
                HOURS_PER_YEAR
            ));
        }
        Ok(())
    }
}

// =============================================================================
// ELCC (Effective Load Carrying Capability) Types
// =============================================================================

/// ELCC calculation method
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElccMethod {
    /// Average (First-In): Simulate with only that resource, measure gas reduction
    Average,
    /// Marginal (Last-In): Add 10 MW to full portfolio, measure gas reduction / 10
    Marginal,
    /// Delta: Allocate portfolio interactive effect proportionally
    Delta,
}

impl Default for ElccMethod {
    fn default() -> Self {
        ElccMethod::Delta
    }
}

/// Per-resource ELCC values
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourceElcc {
    /// First-In (Standalone) ELCC percentage - resource alone
    pub first_in: f64,
    /// Last-In (Marginal) ELCC percentage - adding increment to portfolio
    pub marginal: f64,
    /// Contribution (Removal) ELCC percentage - portfolio minus portfolio-without-resource
    pub contribution: f64,
    /// Delta (E3) ELCC percentage - Last-In + proportional interactive effect allocation
    pub delta: f64,
}

/// ELCC calculation result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ElccResult {
    /// Solar ELCC values
    pub solar: ResourceElcc,
    /// Wind ELCC values
    pub wind: ResourceElcc,
    /// Storage ELCC values
    pub storage: ResourceElcc,
    /// Clean firm ELCC values (always 100% for all methods)
    pub clean_firm: ResourceElcc,
    /// Total portfolio ELCC in MW
    pub portfolio_elcc_mw: f64,
    /// Diversity benefit MW (positive = complementary resources, negative = overlap)
    pub diversity_benefit_mw: f64,
    /// Baseline peak gas (no resources) MW
    pub baseline_peak_gas: f64,
    /// Portfolio peak gas MW
    pub portfolio_peak_gas: f64,
}

impl Default for ElccResult {
    fn default() -> Self {
        Self {
            solar: ResourceElcc::default(),
            wind: ResourceElcc::default(),
            storage: ResourceElcc::default(),
            clean_firm: ResourceElcc {
                first_in: 100.0,
                marginal: 100.0,
                contribution: 100.0,
                delta: 100.0,
            },
            portfolio_elcc_mw: 0.0,
            diversity_benefit_mw: 0.0,
            baseline_peak_gas: 0.0,
            portfolio_peak_gas: 0.0,
        }
    }
}

// =============================================================================
// Market Pricing Types
// =============================================================================

/// Electricity pricing method
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PricingMethod {
    /// SRMC + capacity adder during tight supply, scaled to match LCOE
    ScarcityBased,
    /// Pure energy-only market (SRMC)
    MarginalCost,
    /// Operating Reserve Demand Curve pricing
    Ordc,
    /// Dual revenue stream: energy + capacity payments
    MarginalPlusCapacity,
}

impl Default for PricingMethod {
    fn default() -> Self {
        PricingMethod::ScarcityBased
    }
}

/// ORDC configuration parameters
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct OrdcConfig {
    /// Reserve requirement as % of load
    pub reserve_requirement: f64,
    /// Steepness parameter (lambda)
    pub lambda: f64,
    /// Maximum price cap $/MWh
    pub max_price: f64,
}

impl Default for OrdcConfig {
    fn default() -> Self {
        Self {
            reserve_requirement: 6.0,
            lambda: 2.0,
            max_price: 5000.0,
        }
    }
}

/// Per-resource values (MW or $)
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResourceValues {
    pub solar: f64,
    pub wind: f64,
    pub storage: f64,
    pub clean_firm: f64,
    pub gas: f64,
}

/// Capacity market payment data
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CapacityMarketData {
    /// Qualified capacity MW per resource (capacity * ELCC%)
    pub qualified_capacity: ResourceValues,
    /// Annual payments $ per resource
    pub annual_payments: ResourceValues,
    /// ELCC percentages per resource
    pub elcc_percentages: ResourceValues,
    /// Capacity market clearing price $/MW-yr
    pub clearing_price: f64,
    /// Uniform adder $/MWh when spread across energy
    pub adder_per_mwh: f64,
}

impl Default for CapacityMarketData {
    fn default() -> Self {
        Self {
            qualified_capacity: ResourceValues::default(),
            annual_payments: ResourceValues::default(),
            elcc_percentages: ResourceValues::default(),
            clearing_price: 0.0,
            adder_per_mwh: 0.0,
        }
    }
}

/// Market pricing result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PricingResult {
    /// Hourly prices $/MWh (8760 hours)
    pub hourly_prices: Vec<f64>,
    /// Energy-weighted average price $/MWh
    pub average_price: f64,
    /// Peak price $/MWh
    pub peak_price: f64,
    /// Minimum price $/MWh
    pub min_price: f64,
    /// Capacity market data (only for MarginalPlusCapacity)
    pub capacity_data: Option<CapacityMarketData>,
    /// Pricing method used
    pub method: PricingMethod,
}

impl Default for PricingResult {
    fn default() -> Self {
        Self {
            hourly_prices: vec![0.0; HOURS_PER_YEAR],
            average_price: 0.0,
            peak_price: 0.0,
            min_price: 0.0,
            capacity_data: None,
            method: PricingMethod::default(),
        }
    }
}

// =============================================================================
// Optimizer Sweep Types
// =============================================================================

/// Single point in an optimizer sweep
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepPoint {
    /// Target clean match percentage
    pub target: f64,
    /// Achieved clean match percentage
    pub achieved: f64,
    /// Optimal solar capacity MW
    pub solar: f64,
    /// Optimal wind capacity MW
    pub wind: f64,
    /// Optimal storage capacity MWh
    pub storage: f64,
    /// Optimal clean firm capacity MW
    pub clean_firm: f64,
    /// Resulting LCOE $/MWh
    pub lcoe: f64,
    /// LCOE breakdown by resource $/MWh
    pub solar_lcoe: f64,
    pub wind_lcoe: f64,
    pub storage_lcoe: f64,
    pub clean_firm_lcoe: f64,
    pub gas_lcoe: f64,
    /// Optimization successful
    pub success: bool,
}

/// Optimizer sweep result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SweepResult {
    /// Sweep points for each target
    pub points: Vec<SweepPoint>,
    /// Total elapsed time in milliseconds
    pub elapsed_ms: f64,
}

/// Cost sweep point (for parameter sensitivity)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostSweepPoint {
    /// Parameter value at this point
    pub param_value: f64,
    /// Optimal solar capacity MW
    pub solar: f64,
    /// Optimal wind capacity MW
    pub wind: f64,
    /// Optimal storage capacity MWh
    pub storage: f64,
    /// Optimal clean firm capacity MW
    pub clean_firm: f64,
    /// Achieved clean match percentage
    pub achieved: f64,
    /// Resulting LCOE $/MWh
    pub lcoe: f64,
    /// Optimization successful
    pub success: bool,
}

/// Cost sweep result
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CostSweepResult {
    /// Parameter being swept
    pub param_name: String,
    /// Sweep points
    pub points: Vec<CostSweepPoint>,
    /// Target clean match used
    pub target_match: f64,
    /// Total elapsed time in milliseconds
    pub elapsed_ms: f64,
}
