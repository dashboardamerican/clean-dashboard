// TypeScript types mirroring Rust types

export const HOURS_PER_YEAR = 8760;

// These must match WASM-generated enum values (numeric)
export enum BatteryMode {
  Default = 0,
  PeakShaver = 1,
  Hybrid = 2,
}

export enum DepreciationMethod {
  Macrs5 = 0,
  Macrs15 = 1,
  StraightLine = 2,
}

export interface SimulationConfig {
  solar_capacity: number;
  wind_capacity: number;
  storage_capacity: number;
  clean_firm_capacity: number;
  battery_efficiency: number;
  max_demand_response: number;
  battery_mode: BatteryMode;
}

export interface SimulationResult {
  solar_out: number[];
  wind_out: number[];
  battery_charge: number[];
  battery_discharge: number[];
  gas_generation: number[];
  curtailed: number[];
  clean_delivered: number[];
  clean_firm_generation: number[];
  demand_response: number[];
  gas_for_charging: number[];
  state_of_charge: number[];
  annual_renewable_gen: number;
  annual_load: number;
  peak_gas: number;
  clean_match_pct: number;
  total_curtailment: number;
}

export interface CostParams {
  // CAPEX
  solar_capex: number;
  wind_capex: number;
  storage_capex: number;
  clean_firm_capex: number;
  gas_capex: number;

  // Fixed O&M
  solar_fixed_om: number;
  wind_fixed_om: number;
  storage_fixed_om: number;
  clean_firm_fixed_om: number;
  gas_fixed_om: number;

  // Variable O&M
  solar_var_om: number;
  wind_var_om: number;
  storage_var_om: number;
  clean_firm_var_om: number;
  gas_var_om: number;

  // Fuel
  gas_price: number;
  clean_firm_fuel: number;

  // Financial
  discount_rate: number;
  project_lifetime: number;
  inflation_rate: number;
  tax_rate: number;
  electricity_price: number;        // $/MWh - for revenue/tax calculations
  excess_power_price: number;       // $/MWh - for selling curtailed power
  monetize_excess_depreciation: boolean;  // Enable depreciation monetization
  monetization_rate: number;        // % of excess depreciation to monetize (haircut)

  // Planning reserve margin (% — e.g. 15 means built firm capacity exceeds
  // operational peak by 15%). Scales gas/CF capex and gas land in the LCOE
  // pipeline. Does not affect dispatch.
  reserve_margin: number;

  // Asset lifetimes
  solar_lifetime: number;
  wind_lifetime: number;
  storage_lifetime: number;
  clean_firm_lifetime: number;
  gas_lifetime: number;

  // ITCs
  solar_itc: number;
  wind_itc: number;
  storage_itc: number;
  clean_firm_itc: number;

  // Depreciation
  depreciation_method: DepreciationMethod;

  // Fuel efficiency
  gas_heat_rate: number;

  // Emissions
  gas_emissions_factor: number;
  solar_embodied_emissions: number;
  wind_embodied_emissions: number;
  clean_firm_embodied_emissions: number;
  battery_embodied_emissions: number;
  gas_leakage_rate: number;
  methane_gwp: number;

  // Land use
  solar_land_direct: number;
  wind_land_direct: number;
  wind_land_total: number;
  clean_firm_land_direct: number;
  clean_firm_land_total: number;
  gas_land_direct: number;

  // CCS
  ccs_percentage: number;
  ccs_capex: number;
  ccs_fixed_om: number;
  ccs_var_om: number;
  ccs_energy_penalty: number;
  ccs_capture_rate: number;
}

export interface TechnologyCostBreakdown {
  capex: number;
  fixed_om: number;
  var_om: number;
  fuel: number;
  itc_benefit: number;   // Negative = cost reduction
  tax_shield: number;    // Negative = cost reduction
  total: number;
}

export interface LcoeResult {
  total_lcoe: number;
  solar_lcoe: number;
  wind_lcoe: number;
  storage_lcoe: number;
  clean_firm_lcoe: number;
  gas_lcoe: number;
  ccs_lcoe: number;
  pv_total_costs: number;
  pv_total_energy: number;
  emissions_intensity: number;
  direct_land_use: number;
  total_land_use: number;
  // Detailed breakdowns
  solar_breakdown: TechnologyCostBreakdown;
  wind_breakdown: TechnologyCostBreakdown;
  storage_breakdown: TechnologyCostBreakdown;
  clean_firm_breakdown: TechnologyCostBreakdown;
  gas_breakdown: TechnologyCostBreakdown;
  ccs_breakdown: TechnologyCostBreakdown;
}

export interface OptimizerConfig {
  target_clean_match: number;
  enable_solar: boolean;
  enable_wind: boolean;
  enable_storage: boolean;
  enable_clean_firm: boolean;
  max_solar: number;
  max_wind: number;
  max_storage: number;
  max_clean_firm: number;
  battery_efficiency: number;
  max_demand_response: number;
}

export interface OptimizerResult {
  solar_capacity: number;
  wind_capacity: number;
  storage_capacity: number;
  clean_firm_capacity: number;
  achieved_clean_match: number;
  lcoe: number;
  num_evaluations: number;
  success: boolean;
}

export interface ZoneData {
  name: string;
  solar_profile: number[];
  wind_profile: number[];
  load_profile: number[];
}

export interface CombinedResult {
  simulation: SimulationResult;
  lcoe: LcoeResult;
}

// Default values
export const DEFAULT_COSTS: CostParams = {
  solar_capex: 1000,
  wind_capex: 1200,
  storage_capex: 300,
  clean_firm_capex: 5000,
  gas_capex: 1200,

  solar_fixed_om: 15,
  wind_fixed_om: 40,
  storage_fixed_om: 10,
  clean_firm_fixed_om: 60,
  gas_fixed_om: 20,

  solar_var_om: 0,
  wind_var_om: 0,
  storage_var_om: 5,
  clean_firm_var_om: 10,
  gas_var_om: 2,

  gas_price: 4,
  clean_firm_fuel: 20,

  discount_rate: 7,
  project_lifetime: 20,
  inflation_rate: 2,
  tax_rate: 21,
  electricity_price: 50,
  excess_power_price: 0,
  monetize_excess_depreciation: false,
  monetization_rate: 50,
  reserve_margin: 15,

  solar_lifetime: 30,
  wind_lifetime: 30,
  storage_lifetime: 15,
  clean_firm_lifetime: 60,
  gas_lifetime: 40,

  solar_itc: 0,
  wind_itc: 0,
  storage_itc: 0,
  clean_firm_itc: 0,

  depreciation_method: DepreciationMethod.Macrs5,

  gas_heat_rate: 7.5,

  gas_emissions_factor: 53.1,
  solar_embodied_emissions: 30,
  wind_embodied_emissions: 11,
  clean_firm_embodied_emissions: 11,
  battery_embodied_emissions: 100,
  gas_leakage_rate: 1,
  methane_gwp: 29.8,

  solar_land_direct: 6.5,
  wind_land_direct: 1.25,
  wind_land_total: 50,
  clean_firm_land_direct: 1.0,
  clean_firm_land_total: 1.0,
  gas_land_direct: 0.19,

  ccs_percentage: 0,
  ccs_capex: 2500,
  ccs_fixed_om: 50,
  ccs_var_om: 15,
  ccs_energy_penalty: 20,
  ccs_capture_rate: 100,
};

export const DEFAULT_SIMULATION_CONFIG: SimulationConfig = {
  solar_capacity: 0,
  wind_capacity: 0,
  storage_capacity: 0,
  clean_firm_capacity: 0,
  battery_efficiency: 0.85,
  max_demand_response: 0,
  battery_mode: BatteryMode.Hybrid, // Hybrid is default - best battery utilization and has pre-computed models
};

export const DEFAULT_OPTIMIZER_CONFIG: OptimizerConfig = {
  target_clean_match: 80,
  enable_solar: true,
  enable_wind: true,
  enable_storage: true,
  enable_clean_firm: true,
  max_solar: 1000,
  max_wind: 700,
  max_storage: 2400,
  max_clean_firm: 200,
  battery_efficiency: 0.85,
  max_demand_response: 0,
};

// Zone names
export const ZONE_NAMES = [
  'California',
  'Delta',
  'Florida',
  'Mid-Atlantic',
  'Midwest',
  'Mountain',
  'New England',
  'New York',
  'Northwest',
  'Plains',
  'Southeast',
  'Southwest',
  'Texas',
] as const;

export type ZoneName = (typeof ZONE_NAMES)[number];

// Chart colors
export const COLORS = {
  solar: '#fbbc05',
  wind: '#4285f4',
  battery: '#34a853',
  gas: '#ea4335',
  cleanFirm: '#FF7900',
  storage: '#673ab7',
  gasCcs: '#009688',
  dr: '#9AA0A6',
  curtailed: '#BDBDBD',
} as const;

// Visualization types
export type VisualizationType =
  | 'weekly'
  | 'heatmap'
  | 'battery'
  | 'lcoe'
  | 'price'
  | 'gasBaseline'
  | 'resourceSweep'
  | 'optimizerSweep'
  | 'capacitySweep'
  | 'costSweep';

// =============================================================================
// ELCC Types
// =============================================================================

// Four ELCC methods per E3 methodology:
// - FirstIn (Standalone): Resource alone vs baseline (no other intermittent resources)
// - Marginal (Last-In): Adding 10MW increment to full portfolio
// - Contribution (Removal): Portfolio ELCC minus portfolio-without-resource ELCC
// - Delta (E3): Last-In + proportional allocation of portfolio interactive effect
export enum ElccMethod {
  FirstIn = 0,
  Marginal = 1,
  Contribution = 2,
  Delta = 3,
}

export interface ResourceElcc {
  first_in: number;
  marginal: number;
  contribution: number;
  delta: number;
}

export interface ElccResult {
  solar: ResourceElcc;
  wind: ResourceElcc;
  storage: ResourceElcc;
  clean_firm: ResourceElcc;
  portfolio_elcc_mw: number;
  diversity_benefit_mw: number;  // Positive = complementary, Negative = overlap
  baseline_peak_gas: number;
  portfolio_peak_gas: number;
}

// =============================================================================
// Market Pricing Types
// =============================================================================

export enum PricingMethod {
  ScarcityBased = 0,
  MarginalCost = 1,
  Ordc = 2,
  MarginalPlusCapacity = 3,
}

export interface OrdcConfig {
  reserve_requirement: number; // % of load
  lambda: number;              // steepness
  max_price: number;           // $/MWh cap
}

export interface ResourceValues {
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
  gas: number;
}

export interface CapacityMarketData {
  qualified_capacity: ResourceValues;
  annual_payments: ResourceValues;
  elcc_percentages: ResourceValues;
  clearing_price: number;  // $/MW-yr
  adder_per_mwh: number;   // $/MWh
}

export interface PricingResult {
  hourly_prices: number[];
  average_price: number;
  peak_price: number;
  min_price: number;
  capacity_data: CapacityMarketData | null;
  method: PricingMethod;
}

export const DEFAULT_ORDC_CONFIG: OrdcConfig = {
  reserve_requirement: 6.0,
  lambda: 2.0,
  max_price: 5000.0,
};

// =============================================================================
// Optimizer Sweep Types
// =============================================================================

export interface SweepPoint {
  target: number;
  achieved: number;
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
  lcoe: number;
  // LCOE breakdown by resource $/MWh
  solar_lcoe: number;
  wind_lcoe: number;
  storage_lcoe: number;
  clean_firm_lcoe: number;
  gas_lcoe: number;
  // Peak gas capacity (MW) — read from SimulationResult.peak_gas
  gas_capacity: number;
  success: boolean;
}

export interface SweepResult {
  points: SweepPoint[];
  elapsed_ms: number;
}

export interface CostSweepPoint {
  param_value: number;
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
  achieved: number;
  lcoe: number;
  success: boolean;
}

export interface CostSweepResult {
  param_name: string;
  points: CostSweepPoint[];
  target_match: number;
  elapsed_ms: number;
}

export type CostSweepParam =
  | 'solar_capex'
  | 'wind_capex'
  | 'storage_capex'
  | 'clean_firm_capex'
  | 'gas_capex'
  | 'gas_price'
  | 'solar_itc'
  | 'wind_itc'
  | 'storage_itc'
  | 'clean_firm_itc'
  | 'discount_rate';

// =============================================================================
// Resource Sweep Types
// =============================================================================

export type ResourceSweepResource = 'solar' | 'wind' | 'storage' | 'clean_firm';
export type ResourceSweepMetric = 'clean_match' | 'lcoe';

export interface ResourceSweepPoint {
  capacity: number;
  clean_match: number;
  lcoe: number;
}

export interface ResourceSweepResult {
  resource: ResourceSweepResource;
  points: ResourceSweepPoint[];
  fixed_solar: number;
  fixed_wind: number;
  fixed_storage: number;
  fixed_clean_firm: number;
  current_value: number;
  elapsed_ms: number;
}
