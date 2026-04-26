/* tslint:disable */
/* eslint-disable */

/**
 * Battery dispatch mode
 */
export enum BatteryMode {
    /**
     * Water-fill algorithm prioritizes shaving highest peaks
     */
    Default = 0,
    /**
     * Binary search finds optimal constant peak shaving line
     */
    PeakShaver = 1,
    /**
     * Two-pass: peak shaving + opportunistic dispatch
     */
    Hybrid = 2,
}

/**
 * Cost parameters for LCOE calculation
 */
export class CostParams {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    battery_embodied_emissions: number;
    ccs_capex: number;
    ccs_capture_rate: number;
    ccs_energy_penalty: number;
    ccs_fixed_om: number;
    ccs_percentage: number;
    ccs_var_om: number;
    clean_firm_capex: number;
    clean_firm_embodied_emissions: number;
    clean_firm_fixed_om: number;
    clean_firm_fuel: number;
    clean_firm_itc: number;
    clean_firm_land_direct: number;
    clean_firm_land_total: number;
    clean_firm_lifetime: number;
    clean_firm_var_om: number;
    depreciation_method: DepreciationMethod;
    discount_rate: number;
    electricity_price: number;
    excess_power_price: number;
    gas_capex: number;
    gas_emissions_factor: number;
    gas_fixed_om: number;
    gas_heat_rate: number;
    gas_land_direct: number;
    gas_leakage_rate: number;
    gas_lifetime: number;
    gas_price: number;
    gas_var_om: number;
    inflation_rate: number;
    methane_gwp: number;
    monetization_rate: number;
    monetize_excess_depreciation: boolean;
    project_lifetime: number;
    solar_capex: number;
    solar_embodied_emissions: number;
    solar_fixed_om: number;
    solar_itc: number;
    solar_land_direct: number;
    solar_lifetime: number;
    solar_var_om: number;
    storage_capex: number;
    storage_fixed_om: number;
    storage_itc: number;
    storage_lifetime: number;
    storage_var_om: number;
    tax_rate: number;
    wind_capex: number;
    wind_embodied_emissions: number;
    wind_fixed_om: number;
    wind_itc: number;
    wind_land_direct: number;
    wind_land_total: number;
    wind_lifetime: number;
    wind_var_om: number;
}

/**
 * MACRS depreciation method
 */
export enum DepreciationMethod {
    Macrs5 = 0,
    Macrs15 = 1,
    StraightLine = 2,
}

/**
 * ELCC calculation method
 */
export enum ElccMethod {
    /**
     * Average (First-In): Simulate with only that resource, measure gas reduction
     */
    Average = 0,
    /**
     * Marginal (Last-In): Add 10 MW to full portfolio, measure gas reduction / 10
     */
    Marginal = 1,
    /**
     * Delta: Allocate portfolio interactive effect proportionally
     */
    Delta = 2,
}

/**
 * Result of a land-use calculation.
 */
export class LandUseResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    clean_firm_direct_acres: number;
    clean_firm_total_acres: number;
    /**
     * Direct (physical-footprint) land use in acres.
     */
    direct_acres: number;
    /**
     * Direct land use in mi² (Python's headline number).
     */
    direct_mi2: number;
    gas_direct_acres: number;
    gas_total_acres: number;
    /**
     * Per-technology direct contributions (acres). Useful for charts.
     */
    solar_direct_acres: number;
    /**
     * Per-technology total contributions (acres).
     * Solar and gas have no significant indirect footprint, so total == direct
     * for those two.
     */
    solar_total_acres: number;
    /**
     * Total (direct + indirect, e.g. wind spacing) land use in acres.
     */
    total_acres: number;
    /**
     * Total land use in mi².
     */
    total_mi2: number;
    wind_direct_acres: number;
    wind_total_acres: number;
}

/**
 * LCOE calculation result with detailed breakdown
 */
export class LcoeResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * CCS cost breakdown
     */
    ccs_breakdown: TechnologyCostBreakdown;
    /**
     * CCS LCOE contribution $/MWh
     */
    ccs_lcoe: number;
    /**
     * Clean firm cost breakdown
     */
    clean_firm_breakdown: TechnologyCostBreakdown;
    /**
     * Clean firm LCOE contribution $/MWh
     */
    clean_firm_lcoe: number;
    /**
     * Direct land use acres (physical footprint only)
     */
    direct_land_use: number;
    /**
     * Emissions intensity g CO2/kWh
     */
    emissions_intensity: number;
    /**
     * Gas cost breakdown
     */
    gas_breakdown: TechnologyCostBreakdown;
    /**
     * Gas LCOE contribution $/MWh
     */
    gas_lcoe: number;
    /**
     * Total present value of costs $
     */
    pv_total_costs: number;
    /**
     * Total present value of energy MWh
     */
    pv_total_energy: number;
    /**
     * Solar cost breakdown
     */
    solar_breakdown: TechnologyCostBreakdown;
    /**
     * Solar LCOE contribution $/MWh
     */
    solar_lcoe: number;
    /**
     * Storage cost breakdown
     */
    storage_breakdown: TechnologyCostBreakdown;
    /**
     * Storage LCOE contribution $/MWh
     */
    storage_lcoe: number;
    /**
     * Total land use acres (includes indirect: wind spacing, exclusion zones)
     */
    total_land_use: number;
    /**
     * Total system LCOE in $/MWh
     */
    total_lcoe: number;
    /**
     * Wind cost breakdown
     */
    wind_breakdown: TechnologyCostBreakdown;
    /**
     * Wind LCOE contribution $/MWh
     */
    wind_lcoe: number;
}

/**
 * Optimizer configuration
 */
export class OptimizerConfig {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Battery round-trip efficiency used during optimizer evaluations
     */
    battery_efficiency: number;
    /**
     * Enable clean firm in optimization
     */
    enable_clean_firm: boolean;
    /**
     * Enable solar in optimization
     */
    enable_solar: boolean;
    /**
     * Enable storage in optimization
     */
    enable_storage: boolean;
    /**
     * Enable wind in optimization
     */
    enable_wind: boolean;
    /**
     * Maximum clean firm capacity MW
     */
    max_clean_firm: number;
    /**
     * Maximum demand response used during optimizer evaluations
     */
    max_demand_response: number;
    /**
     * Maximum solar capacity MW
     */
    max_solar: number;
    /**
     * Maximum storage capacity MWh
     */
    max_storage: number;
    /**
     * Maximum wind capacity MW
     */
    max_wind: number;
    /**
     * Target clean match percentage (0-100)
     */
    target_clean_match: number;
}

/**
 * Optimizer result
 */
export class OptimizerResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Achieved clean match percentage
     */
    achieved_clean_match: number;
    /**
     * Optimal clean firm capacity MW
     */
    clean_firm_capacity: number;
    /**
     * Resulting LCOE $/MWh
     */
    lcoe: number;
    /**
     * Number of evaluations
     */
    num_evaluations: number;
    /**
     * Optimal solar capacity MW
     */
    solar_capacity: number;
    /**
     * Optimal storage capacity MWh
     */
    storage_capacity: number;
    /**
     * Optimization successful
     */
    success: boolean;
    /**
     * Optimal wind capacity MW
     */
    wind_capacity: number;
}

/**
 * ORDC configuration parameters
 */
export class OrdcConfig {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Steepness parameter (lambda)
     */
    lambda: number;
    /**
     * Maximum price cap $/MWh
     */
    max_price: number;
    /**
     * Reserve requirement as % of load
     */
    reserve_requirement: number;
}

/**
 * Electricity pricing method
 */
export enum PricingMethod {
    /**
     * SRMC + capacity adder during tight supply, scaled to match LCOE
     */
    ScarcityBased = 0,
    /**
     * Pure energy-only market (SRMC)
     */
    MarginalCost = 1,
    /**
     * Operating Reserve Demand Curve pricing
     */
    Ordc = 2,
    /**
     * Dual revenue stream: energy + capacity payments
     */
    MarginalPlusCapacity = 3,
}

/**
 * Configuration for a simulation run
 */
export class SimulationConfig {
    free(): void;
    [Symbol.dispose](): void;
    constructor(solar_capacity: number, wind_capacity: number, storage_capacity: number, clean_firm_capacity: number, battery_efficiency: number, max_demand_response: number, battery_mode: BatteryMode);
    static with_defaults(): SimulationConfig;
    /**
     * Battery round-trip efficiency (0-1)
     */
    battery_efficiency: number;
    /**
     * Battery dispatch mode
     */
    battery_mode: BatteryMode;
    /**
     * Clean firm capacity in MW (constant output)
     */
    clean_firm_capacity: number;
    /**
     * Maximum demand response in MW
     */
    max_demand_response: number;
    /**
     * Solar capacity in MW
     */
    solar_capacity: number;
    /**
     * Storage capacity in MWh
     */
    storage_capacity: number;
    /**
     * Wind capacity in MW
     */
    wind_capacity: number;
}

/**
 * Cost breakdown for a single technology
 */
export class TechnologyCostBreakdown {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Capital expenditure $/MWh
     */
    capex: number;
    /**
     * Fixed O&M $/MWh
     */
    fixed_om: number;
    /**
     * Fuel cost $/MWh
     */
    fuel: number;
    /**
     * ITC benefit (negative = reduces cost) $/MWh
     */
    itc_benefit: number;
    /**
     * Depreciation tax shield (negative = reduces cost) $/MWh
     */
    tax_shield: number;
    /**
     * Total for this technology $/MWh
     */
    total: number;
    /**
     * Variable O&M $/MWh
     */
    var_om: number;
}

export function battery_mode_default(): BatteryMode;

export function battery_mode_hybrid(): BatteryMode;

export function battery_mode_peak_shaver(): BatteryMode;

/**
 * Calculate ELCC metrics for all resources
 *
 * # Arguments
 * * `solar_capacity` - Solar capacity MW
 * * `wind_capacity` - Wind capacity MW
 * * `storage_capacity` - Storage capacity MWh
 * * `clean_firm_capacity` - Clean firm capacity MW
 * * `solar_profile` - Solar capacity factors (Float64Array)
 * * `wind_profile` - Wind capacity factors (Float64Array)
 * * `load_profile` - Load MW (Float64Array)
 * * `battery_mode_js` - Battery mode as string
 * * `battery_efficiency` - Battery round-trip efficiency
 * * `max_demand_response` - Maximum demand response fraction
 *
 * # Returns
 * * ElccResult as JsValue
 */
export function calculate_elcc_metrics(solar_capacity: number, wind_capacity: number, storage_capacity: number, clean_firm_capacity: number, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, battery_mode_js: any, battery_efficiency: number, max_demand_response: number): any;

/**
 * Calculate land use for a portfolio without running the simulation.
 *
 * # Arguments
 * * `solar_capacity` - Solar capacity (MW)
 * * `wind_capacity` - Wind capacity (MW)
 * * `clean_firm_capacity` - Clean firm capacity (MW)
 * * `gas_capacity` - Peak gas capacity needed (MW). Pass the same value
 *   you would read from `SimulationResult.peak_gas`.
 * * `costs_js` - CostParams as JsValue
 *
 * # Returns
 * * LandUseResult as JsValue with `direct_acres`, `total_acres`,
 *   `direct_mi2`, `total_mi2`, plus per-technology breakdowns.
 */
export function compute_land_use(solar_capacity: number, wind_capacity: number, clean_firm_capacity: number, gas_capacity: number, costs_js: any): any;

/**
 * Calculate LCOE for a simulation result
 *
 * # Arguments
 * * `sim_result_js` - SimulationResult as JsValue
 * * `solar_capacity` - Solar capacity MW
 * * `wind_capacity` - Wind capacity MW
 * * `storage_capacity` - Storage capacity MWh
 * * `clean_firm_capacity` - Clean firm capacity MW
 * * `costs_js` - CostParams as JsValue
 *
 * # Returns
 * * LcoeResult as JsValue
 */
export function compute_lcoe(sim_result_js: any, solar_capacity: number, wind_capacity: number, storage_capacity: number, clean_firm_capacity: number, costs_js: any): any;

/**
 * Compute hourly electricity prices
 *
 * # Arguments
 * * `sim_result_js` - SimulationResult as JsValue
 * * `costs_js` - CostParams as JsValue
 * * `lcoe` - System LCOE $/MWh
 * * `pricing_method_js` - PricingMethod as JsValue
 * * `load_profile` - Load MW (Float64Array)
 * * `ordc_config_js` - Optional OrdcConfig as JsValue
 * * `elcc_result_js` - Optional ElccResult as JsValue
 * * `solar_capacity` - Solar capacity MW
 * * `wind_capacity` - Wind capacity MW
 * * `storage_capacity` - Storage capacity MWh
 * * `clean_firm_capacity` - Clean firm capacity MW
 *
 * # Returns
 * * PricingResult as JsValue
 */
export function compute_prices(sim_result_js: any, costs_js: any, lcoe: number, pricing_method_js: any, load_profile: Float64Array, ordc_config_js: any, elcc_result_js: any, solar_capacity: number, wind_capacity: number, storage_capacity: number, clean_firm_capacity: number): any;

/**
 * Evaluate a batch of portfolios (for Web Worker parallel processing)
 *
 * # Arguments
 * * `portfolios_js` - Array of portfolio configurations
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `battery_mode` - Battery dispatch mode
 * * `config_js` - Optional OptimizerConfig as JsValue for runtime assumptions
 *
 * # Returns
 * * Array of evaluation results
 */
export function evaluate_batch(portfolios_js: any, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, battery_mode: BatteryMode, config_js?: any | null): any;

/**
 * Get default cost parameters
 */
export function get_default_costs(): any;

/**
 * Get default optimizer config
 */
export function get_default_optimizer_config(): any;

/**
 * Get default simulation config
 */
export function get_default_simulation_config(): any;

/**
 * Get the library version
 */
export function get_version(): string;

/**
 * Initialize panic hook for better error messages in browser console
 */
export function init(): void;

/**
 * Run the optimizer (V2 hierarchical optimizer)
 *
 * # Arguments
 * * `target_match` - Target clean match percentage (0-100)
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * OptimizerResult as JsValue
 *
 * Note: If a model is loaded for the current zone/mode via `wasm_load_model()`,
 * it will be used automatically for faster candidate filtering.
 */
export function optimize(target_match: number, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run optimizer sweep with model-based acceleration
 *
 * Uses cached model for faster candidate filtering if available.
 * Returns the same SweepResult structure as run_optimizer_sweep.
 *
 * # Arguments
 * * `zone` - Zone name (must match loaded model)
 * * `targets` - Array of target percentages
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * SweepResult as JsValue (same format as run_optimizer_sweep)
 */
export function optimize_sweep_with_model(zone: string, targets: Float64Array, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run the V2 hierarchical optimizer
 *
 * # Arguments
 * * `target_match` - Target clean match percentage (0-100)
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * OptimizerResult as JsValue
 */
export function optimize_v2(target_match: number, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run V2 optimizer sweep across multiple targets
 *
 * # Arguments
 * * `targets` - Array of target percentages
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * Array of OptimizerResult as JsValue
 */
export function optimize_v2_sweep(targets: Float64Array, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run optimizer with model-based acceleration (if model is cached)
 *
 * This is the preferred method when you have loaded a model via `wasm_load_model()`.
 * Falls back to greedy search if no model is cached for the zone/mode.
 *
 * # Arguments
 * * `zone` - Zone name (must match the zone used when loading the model)
 * * `target_match` - Target clean match percentage (0-100)
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * OptimizerResult as JsValue
 */
export function optimize_with_model(zone: string, target_match: number, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run cost sweep - optimize across a range of parameter values
 *
 * # Arguments
 * * `target_match` - Target clean match percentage
 * * `param_name` - Name of parameter to sweep
 * * `min_value` - Minimum parameter value
 * * `max_value` - Maximum parameter value
 * * `steps` - Number of steps in sweep
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `base_costs_js` - Base CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * CostSweepResult as JsValue
 */
export function run_cost_sweep(target_match: number, param_name: string, min_value: number, max_value: number, steps: number, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, base_costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run cost sweep with model-based acceleration
 *
 * Uses cached model for faster candidate filtering if available.
 *
 * # Arguments
 * * `zone` - Zone name (must match loaded model)
 * * `target_match` - Target clean match percentage
 * * `param_name` - Name of parameter to sweep
 * * `min_value` - Minimum parameter value
 * * `max_value` - Maximum parameter value
 * * `steps` - Number of steps in sweep
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `base_costs_js` - Base CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * CostSweepResult as JsValue
 */
export function run_cost_sweep_with_model(zone: string, target_match: number, param_name: string, min_value: number, max_value: number, steps: number, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, base_costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run the incremental cost walk optimizer.
 *
 * Mirrors the Python `run_incremental_cost_walk` strategy: starts from a zero
 * portfolio and incrementally adds the most cost-effective resource (smallest
 * LCOE-per-percentage-point ratio) until reaching the clean-match target,
 * halving step sizes when overshooting.
 *
 * # Arguments
 * * `target_match` - Target clean match percentage (values >= 100 are capped to 99.5)
 * * `solar_profile` - Solar capacity factors (8760 hours)
 * * `wind_profile` - Wind capacity factors (8760 hours)
 * * `load_profile` - Load MW (8760 hours)
 * * `costs_js` - CostParams as JsValue
 * * `config_js` - OptimizerConfig as JsValue (provides battery_efficiency,
 *   max_demand_response, and the resource-enable flags)
 * * `battery_mode` - Battery dispatch mode
 *
 * # Returns
 * * IncrementalWalkResult as JsValue (includes the full walk_trace)
 */
export function run_incremental_walk_wasm(target_match: number, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run optimizer sweep and return structured result (uses V2 optimizer)
 */
export function run_optimizer_sweep(targets: Float64Array, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any, config_js: any, battery_mode: BatteryMode): any;

/**
 * Run a single simulation and return results as JSON
 *
 * # Arguments
 * * `config_js` - SimulationConfig as JsValue
 * * `solar_profile` - Solar capacity factors (Float64Array)
 * * `wind_profile` - Wind capacity factors (Float64Array)
 * * `load_profile` - Load MW (Float64Array)
 *
 * # Returns
 * * SimulationResult as JsValue (JSON-serializable)
 */
export function simulate(config_js: any, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array): any;

/**
 * Run full simulation and LCOE calculation in one call
 *
 * # Arguments
 * * `config_js` - SimulationConfig as JsValue
 * * `solar_profile` - Solar capacity factors
 * * `wind_profile` - Wind capacity factors
 * * `load_profile` - Load MW
 * * `costs_js` - CostParams as JsValue
 *
 * # Returns
 * * Object with both simulation and LCOE results
 */
export function simulate_and_calculate_lcoe(config_js: any, solar_profile: Float64Array, wind_profile: Float64Array, load_profile: Float64Array, costs_js: any): any;

/**
 * Get model cache statistics
 *
 * # Returns
 * * Object with { loaded: number, max: number }
 */
export function wasm_cache_stats(): any;

/**
 * Clear all cached models to free memory
 *
 * Call this when switching contexts or to reduce memory usage.
 * Models will need to be reloaded before model-based optimization can be used.
 */
export function wasm_clear_models(): void;

/**
 * Check if a model is loaded in the cache
 *
 * # Arguments
 * * `zone` - Zone name (case-insensitive)
 * * `battery_mode` - Battery mode
 *
 * # Returns
 * * `true` if model is cached and ready for use
 * * `false` if model needs to be loaded
 */
export function wasm_is_model_loaded(zone: string, battery_mode: BatteryMode): boolean;

/**
 * Load an empirical model into the cache for model-based optimization
 *
 * # Arguments
 * * `zone` - Zone name (case-insensitive, e.g., "california", "texas")
 * * `battery_mode` - Battery mode (must match the mode used to generate the model)
 * * `bytes` - Model binary data (bincode serialized EmpiricalModel)
 *
 * # Returns
 * * `Ok(())` if model loaded successfully
 * * `Err` if deserialization fails
 *
 * # Example (TypeScript)
 * ```typescript
 * const response = await fetch('/models/california_hybrid.bin');
 * const bytes = new Uint8Array(await response.arrayBuffer());
 * wasm.wasm_load_model('california', BatteryMode.Hybrid, bytes);
 * ```
 */
export function wasm_load_model(zone: string, battery_mode: BatteryMode, bytes: Uint8Array): void;

/**
 * Get list of currently loaded models
 *
 * # Returns
 * * Array of [zone, battery_mode] pairs as JSON
 */
export function wasm_loaded_models(): any;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_costparams_free: (a: number, b: number) => void;
    readonly __wbg_get_costparams_battery_embodied_emissions: (a: number) => number;
    readonly __wbg_get_costparams_ccs_capex: (a: number) => number;
    readonly __wbg_get_costparams_ccs_capture_rate: (a: number) => number;
    readonly __wbg_get_costparams_ccs_energy_penalty: (a: number) => number;
    readonly __wbg_get_costparams_ccs_fixed_om: (a: number) => number;
    readonly __wbg_get_costparams_ccs_percentage: (a: number) => number;
    readonly __wbg_get_costparams_ccs_var_om: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_capex: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_embodied_emissions: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_fixed_om: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_fuel: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_itc: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_land_direct: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_land_total: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_lifetime: (a: number) => number;
    readonly __wbg_get_costparams_clean_firm_var_om: (a: number) => number;
    readonly __wbg_get_costparams_depreciation_method: (a: number) => number;
    readonly __wbg_get_costparams_discount_rate: (a: number) => number;
    readonly __wbg_get_costparams_electricity_price: (a: number) => number;
    readonly __wbg_get_costparams_excess_power_price: (a: number) => number;
    readonly __wbg_get_costparams_gas_capex: (a: number) => number;
    readonly __wbg_get_costparams_gas_emissions_factor: (a: number) => number;
    readonly __wbg_get_costparams_gas_fixed_om: (a: number) => number;
    readonly __wbg_get_costparams_gas_heat_rate: (a: number) => number;
    readonly __wbg_get_costparams_gas_land_direct: (a: number) => number;
    readonly __wbg_get_costparams_gas_leakage_rate: (a: number) => number;
    readonly __wbg_get_costparams_gas_lifetime: (a: number) => number;
    readonly __wbg_get_costparams_gas_price: (a: number) => number;
    readonly __wbg_get_costparams_gas_var_om: (a: number) => number;
    readonly __wbg_get_costparams_inflation_rate: (a: number) => number;
    readonly __wbg_get_costparams_methane_gwp: (a: number) => number;
    readonly __wbg_get_costparams_monetization_rate: (a: number) => number;
    readonly __wbg_get_costparams_monetize_excess_depreciation: (a: number) => number;
    readonly __wbg_get_costparams_project_lifetime: (a: number) => number;
    readonly __wbg_get_costparams_solar_capex: (a: number) => number;
    readonly __wbg_get_costparams_solar_embodied_emissions: (a: number) => number;
    readonly __wbg_get_costparams_solar_fixed_om: (a: number) => number;
    readonly __wbg_get_costparams_solar_itc: (a: number) => number;
    readonly __wbg_get_costparams_solar_land_direct: (a: number) => number;
    readonly __wbg_get_costparams_solar_lifetime: (a: number) => number;
    readonly __wbg_get_costparams_solar_var_om: (a: number) => number;
    readonly __wbg_get_costparams_storage_capex: (a: number) => number;
    readonly __wbg_get_costparams_storage_fixed_om: (a: number) => number;
    readonly __wbg_get_costparams_storage_itc: (a: number) => number;
    readonly __wbg_get_costparams_storage_lifetime: (a: number) => number;
    readonly __wbg_get_costparams_storage_var_om: (a: number) => number;
    readonly __wbg_get_costparams_tax_rate: (a: number) => number;
    readonly __wbg_get_costparams_wind_capex: (a: number) => number;
    readonly __wbg_get_costparams_wind_embodied_emissions: (a: number) => number;
    readonly __wbg_get_costparams_wind_fixed_om: (a: number) => number;
    readonly __wbg_get_costparams_wind_itc: (a: number) => number;
    readonly __wbg_get_costparams_wind_land_direct: (a: number) => number;
    readonly __wbg_get_costparams_wind_land_total: (a: number) => number;
    readonly __wbg_get_costparams_wind_lifetime: (a: number) => number;
    readonly __wbg_get_costparams_wind_var_om: (a: number) => number;
    readonly __wbg_get_lcoeresult_ccs_breakdown: (a: number) => number;
    readonly __wbg_get_lcoeresult_clean_firm_breakdown: (a: number) => number;
    readonly __wbg_get_lcoeresult_gas_breakdown: (a: number) => number;
    readonly __wbg_get_lcoeresult_solar_breakdown: (a: number) => number;
    readonly __wbg_get_lcoeresult_storage_breakdown: (a: number) => number;
    readonly __wbg_get_lcoeresult_wind_breakdown: (a: number) => number;
    readonly __wbg_get_optimizerconfig_enable_clean_firm: (a: number) => number;
    readonly __wbg_get_optimizerconfig_enable_solar: (a: number) => number;
    readonly __wbg_get_optimizerconfig_enable_storage: (a: number) => number;
    readonly __wbg_get_optimizerconfig_enable_wind: (a: number) => number;
    readonly __wbg_get_optimizerresult_num_evaluations: (a: number) => number;
    readonly __wbg_get_optimizerresult_success: (a: number) => number;
    readonly __wbg_get_simulationconfig_battery_mode: (a: number) => number;
    readonly __wbg_lcoeresult_free: (a: number, b: number) => void;
    readonly __wbg_optimizerconfig_free: (a: number, b: number) => void;
    readonly __wbg_optimizerresult_free: (a: number, b: number) => void;
    readonly __wbg_ordcconfig_free: (a: number, b: number) => void;
    readonly __wbg_set_costparams_battery_embodied_emissions: (a: number, b: number) => void;
    readonly __wbg_set_costparams_ccs_capex: (a: number, b: number) => void;
    readonly __wbg_set_costparams_ccs_capture_rate: (a: number, b: number) => void;
    readonly __wbg_set_costparams_ccs_energy_penalty: (a: number, b: number) => void;
    readonly __wbg_set_costparams_ccs_fixed_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_ccs_percentage: (a: number, b: number) => void;
    readonly __wbg_set_costparams_ccs_var_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_capex: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_embodied_emissions: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_fixed_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_fuel: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_itc: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_land_direct: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_land_total: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_lifetime: (a: number, b: number) => void;
    readonly __wbg_set_costparams_clean_firm_var_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_depreciation_method: (a: number, b: number) => void;
    readonly __wbg_set_costparams_discount_rate: (a: number, b: number) => void;
    readonly __wbg_set_costparams_electricity_price: (a: number, b: number) => void;
    readonly __wbg_set_costparams_excess_power_price: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_capex: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_emissions_factor: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_fixed_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_heat_rate: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_land_direct: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_leakage_rate: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_lifetime: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_price: (a: number, b: number) => void;
    readonly __wbg_set_costparams_gas_var_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_inflation_rate: (a: number, b: number) => void;
    readonly __wbg_set_costparams_methane_gwp: (a: number, b: number) => void;
    readonly __wbg_set_costparams_monetization_rate: (a: number, b: number) => void;
    readonly __wbg_set_costparams_monetize_excess_depreciation: (a: number, b: number) => void;
    readonly __wbg_set_costparams_project_lifetime: (a: number, b: number) => void;
    readonly __wbg_set_costparams_solar_capex: (a: number, b: number) => void;
    readonly __wbg_set_costparams_solar_embodied_emissions: (a: number, b: number) => void;
    readonly __wbg_set_costparams_solar_fixed_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_solar_itc: (a: number, b: number) => void;
    readonly __wbg_set_costparams_solar_land_direct: (a: number, b: number) => void;
    readonly __wbg_set_costparams_solar_lifetime: (a: number, b: number) => void;
    readonly __wbg_set_costparams_solar_var_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_storage_capex: (a: number, b: number) => void;
    readonly __wbg_set_costparams_storage_fixed_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_storage_itc: (a: number, b: number) => void;
    readonly __wbg_set_costparams_storage_lifetime: (a: number, b: number) => void;
    readonly __wbg_set_costparams_storage_var_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_tax_rate: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_capex: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_embodied_emissions: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_fixed_om: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_itc: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_land_direct: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_land_total: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_lifetime: (a: number, b: number) => void;
    readonly __wbg_set_costparams_wind_var_om: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_ccs_breakdown: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_clean_firm_breakdown: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_gas_breakdown: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_solar_breakdown: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_storage_breakdown: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_wind_breakdown: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_enable_clean_firm: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_enable_solar: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_enable_storage: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_enable_wind: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_num_evaluations: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_success: (a: number, b: number) => void;
    readonly __wbg_set_simulationconfig_battery_mode: (a: number, b: number) => void;
    readonly __wbg_simulationconfig_free: (a: number, b: number) => void;
    readonly __wbg_technologycostbreakdown_free: (a: number, b: number) => void;
    readonly simulationconfig_new: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => number;
    readonly simulationconfig_with_defaults: () => number;
    readonly __wbg_set_lcoeresult_ccs_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_clean_firm_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_direct_land_use: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_emissions_intensity: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_gas_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_pv_total_costs: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_pv_total_energy: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_solar_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_storage_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_total_land_use: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_total_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_lcoeresult_wind_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_battery_efficiency: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_max_clean_firm: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_max_demand_response: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_max_solar: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_max_storage: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_max_wind: (a: number, b: number) => void;
    readonly __wbg_set_optimizerconfig_target_clean_match: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_achieved_clean_match: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_clean_firm_capacity: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_lcoe: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_solar_capacity: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_storage_capacity: (a: number, b: number) => void;
    readonly __wbg_set_optimizerresult_wind_capacity: (a: number, b: number) => void;
    readonly __wbg_set_ordcconfig_lambda: (a: number, b: number) => void;
    readonly __wbg_set_ordcconfig_max_price: (a: number, b: number) => void;
    readonly __wbg_set_ordcconfig_reserve_requirement: (a: number, b: number) => void;
    readonly __wbg_set_simulationconfig_battery_efficiency: (a: number, b: number) => void;
    readonly __wbg_set_simulationconfig_clean_firm_capacity: (a: number, b: number) => void;
    readonly __wbg_set_simulationconfig_max_demand_response: (a: number, b: number) => void;
    readonly __wbg_set_simulationconfig_solar_capacity: (a: number, b: number) => void;
    readonly __wbg_set_simulationconfig_storage_capacity: (a: number, b: number) => void;
    readonly __wbg_set_simulationconfig_wind_capacity: (a: number, b: number) => void;
    readonly __wbg_set_technologycostbreakdown_capex: (a: number, b: number) => void;
    readonly __wbg_set_technologycostbreakdown_fixed_om: (a: number, b: number) => void;
    readonly __wbg_set_technologycostbreakdown_fuel: (a: number, b: number) => void;
    readonly __wbg_set_technologycostbreakdown_itc_benefit: (a: number, b: number) => void;
    readonly __wbg_set_technologycostbreakdown_tax_shield: (a: number, b: number) => void;
    readonly __wbg_set_technologycostbreakdown_total: (a: number, b: number) => void;
    readonly __wbg_set_technologycostbreakdown_var_om: (a: number, b: number) => void;
    readonly __wbg_get_lcoeresult_ccs_lcoe: (a: number) => number;
    readonly __wbg_get_lcoeresult_clean_firm_lcoe: (a: number) => number;
    readonly __wbg_get_lcoeresult_direct_land_use: (a: number) => number;
    readonly __wbg_get_lcoeresult_emissions_intensity: (a: number) => number;
    readonly __wbg_get_lcoeresult_gas_lcoe: (a: number) => number;
    readonly __wbg_get_lcoeresult_pv_total_costs: (a: number) => number;
    readonly __wbg_get_lcoeresult_pv_total_energy: (a: number) => number;
    readonly __wbg_get_lcoeresult_solar_lcoe: (a: number) => number;
    readonly __wbg_get_lcoeresult_storage_lcoe: (a: number) => number;
    readonly __wbg_get_lcoeresult_total_land_use: (a: number) => number;
    readonly __wbg_get_lcoeresult_total_lcoe: (a: number) => number;
    readonly __wbg_get_lcoeresult_wind_lcoe: (a: number) => number;
    readonly __wbg_get_optimizerconfig_battery_efficiency: (a: number) => number;
    readonly __wbg_get_optimizerconfig_max_clean_firm: (a: number) => number;
    readonly __wbg_get_optimizerconfig_max_demand_response: (a: number) => number;
    readonly __wbg_get_optimizerconfig_max_solar: (a: number) => number;
    readonly __wbg_get_optimizerconfig_max_storage: (a: number) => number;
    readonly __wbg_get_optimizerconfig_max_wind: (a: number) => number;
    readonly __wbg_get_optimizerconfig_target_clean_match: (a: number) => number;
    readonly __wbg_get_optimizerresult_achieved_clean_match: (a: number) => number;
    readonly __wbg_get_optimizerresult_clean_firm_capacity: (a: number) => number;
    readonly __wbg_get_optimizerresult_lcoe: (a: number) => number;
    readonly __wbg_get_optimizerresult_solar_capacity: (a: number) => number;
    readonly __wbg_get_optimizerresult_storage_capacity: (a: number) => number;
    readonly __wbg_get_optimizerresult_wind_capacity: (a: number) => number;
    readonly __wbg_get_ordcconfig_lambda: (a: number) => number;
    readonly __wbg_get_ordcconfig_max_price: (a: number) => number;
    readonly __wbg_get_ordcconfig_reserve_requirement: (a: number) => number;
    readonly __wbg_get_simulationconfig_battery_efficiency: (a: number) => number;
    readonly __wbg_get_simulationconfig_clean_firm_capacity: (a: number) => number;
    readonly __wbg_get_simulationconfig_max_demand_response: (a: number) => number;
    readonly __wbg_get_simulationconfig_solar_capacity: (a: number) => number;
    readonly __wbg_get_simulationconfig_storage_capacity: (a: number) => number;
    readonly __wbg_get_simulationconfig_wind_capacity: (a: number) => number;
    readonly __wbg_get_technologycostbreakdown_capex: (a: number) => number;
    readonly __wbg_get_technologycostbreakdown_fixed_om: (a: number) => number;
    readonly __wbg_get_technologycostbreakdown_fuel: (a: number) => number;
    readonly __wbg_get_technologycostbreakdown_itc_benefit: (a: number) => number;
    readonly __wbg_get_technologycostbreakdown_tax_shield: (a: number) => number;
    readonly __wbg_get_technologycostbreakdown_total: (a: number) => number;
    readonly __wbg_get_technologycostbreakdown_var_om: (a: number) => number;
    readonly battery_mode_default: () => number;
    readonly battery_mode_hybrid: () => number;
    readonly battery_mode_peak_shaver: () => number;
    readonly calculate_elcc_metrics: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: any, l: number, m: number) => [number, number, number];
    readonly compute_land_use: (a: number, b: number, c: number, d: number, e: any) => [number, number, number];
    readonly compute_lcoe: (a: any, b: number, c: number, d: number, e: number, f: any) => [number, number, number];
    readonly compute_prices: (a: any, b: any, c: number, d: any, e: number, f: number, g: any, h: any, i: number, j: number, k: number, l: number) => [number, number, number];
    readonly evaluate_batch: (a: any, b: number, c: number, d: number, e: number, f: number, g: number, h: any, i: number, j: number) => [number, number, number];
    readonly get_default_costs: () => [number, number, number];
    readonly get_default_optimizer_config: () => [number, number, number];
    readonly get_default_simulation_config: () => [number, number, number];
    readonly get_version: () => [number, number];
    readonly optimize: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: any, i: any, j: number) => [number, number, number];
    readonly optimize_sweep_with_model: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: any, l: any, m: number) => [number, number, number];
    readonly optimize_v2_sweep: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: any, j: any, k: number) => [number, number, number];
    readonly optimize_with_model: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: any, k: any, l: number) => [number, number, number];
    readonly run_cost_sweep: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: any, n: any, o: number) => [number, number, number];
    readonly run_cost_sweep_with_model: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number, m: number, n: number, o: any, p: any, q: number) => [number, number, number];
    readonly run_incremental_walk_wasm: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: any, i: any, j: number) => [number, number, number];
    readonly run_optimizer_sweep: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: any, j: any, k: number) => [number, number, number];
    readonly simulate: (a: any, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number, number];
    readonly simulate_and_calculate_lcoe: (a: any, b: number, c: number, d: number, e: number, f: number, g: number, h: any) => [number, number, number];
    readonly wasm_cache_stats: () => [number, number, number];
    readonly wasm_is_model_loaded: (a: number, b: number, c: number) => number;
    readonly wasm_load_model: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly wasm_loaded_models: () => [number, number, number];
    readonly init: () => void;
    readonly wasm_clear_models: () => void;
    readonly optimize_v2: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: any, i: any, j: number) => [number, number, number];
    readonly __wbg_get_landuseresult_clean_firm_direct_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_clean_firm_total_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_direct_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_direct_mi2: (a: number) => number;
    readonly __wbg_get_landuseresult_gas_direct_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_gas_total_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_solar_direct_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_solar_total_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_total_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_total_mi2: (a: number) => number;
    readonly __wbg_get_landuseresult_wind_direct_acres: (a: number) => number;
    readonly __wbg_get_landuseresult_wind_total_acres: (a: number) => number;
    readonly __wbg_landuseresult_free: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_clean_firm_direct_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_clean_firm_total_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_direct_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_direct_mi2: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_gas_direct_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_gas_total_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_solar_direct_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_solar_total_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_total_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_total_mi2: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_wind_direct_acres: (a: number, b: number) => void;
    readonly __wbg_set_landuseresult_wind_total_acres: (a: number, b: number) => void;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
