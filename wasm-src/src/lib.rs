/// Energy System Simulator - Rust/WASM Core
///
/// High-performance energy system simulation and optimization for
/// React frontend integration. Provides WASM bindings for:
/// - 8760-hour chronological simulation with multiple battery strategies
/// - Investment-grade LCOE calculation
/// - Portfolio optimization (V2 hierarchical optimizer)
///
/// Target performance: <20ms simulation, <20ms optimization per target
pub mod economics;
pub mod optimizer;
pub mod simulation;
pub mod types;

use serde_wasm_bindgen::{from_value, to_value};
use wasm_bindgen::prelude::*;

pub use economics::{calculate_elcc, calculate_lcoe, compute_hourly_prices};
pub use optimizer::{
    clear_models,
    get_model,
    is_model_loaded,
    load_model as load_model_internal,
    loaded_models,
    run_incremental_walk,
    run_v2_accuracy_audit_suite,
    // Model cache exports
    run_v2_adaptive_audit,
    run_v2_optimizer,
    run_v2_optimizer_mode,
    run_v2_optimizer_mode_detailed,
    run_v2_sweep,
    run_v2_sweep_mode,
    AdaptiveFixRecommendation,
    AdaptiveOracleConfig,
    AdaptiveTrialReport,
    EmpiricalModel,
    GridConfig,
    LeverMultipliers,
    TrainingSample,
    V2AccuracyAuditCaseReport,
    V2AccuracyAuditSuiteReport,
    V2AccuracySummary,
    V2AccurateConfig,
    V2AccurateDiagnostics,
    V2AccurateStopReason,
    V2AdaptiveAuditConfig,
    V2AdaptiveAuditReport,
    V2AdaptiveAuditSummary,
    V2Mode,
    V2StepSchedule,
};
#[cfg(feature = "experimental-v3")]
pub use optimizer::run_v3_optimizer;
pub use simulation::simulate_system;
pub use types::*;

// Deprecated V1 optimizer - use run_v2_optimizer instead
#[allow(deprecated)]
pub use optimizer::run_v1_optimizer;

/// Initialize panic hook for better error messages in browser console
#[wasm_bindgen(start)]
pub fn init() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

/// Get the library version
#[wasm_bindgen]
pub fn get_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Run a single simulation and return results as JSON
///
/// # Arguments
/// * `config_js` - SimulationConfig as JsValue
/// * `solar_profile` - Solar capacity factors (Float64Array)
/// * `wind_profile` - Wind capacity factors (Float64Array)
/// * `load_profile` - Load MW (Float64Array)
///
/// # Returns
/// * SimulationResult as JsValue (JSON-serializable)
#[wasm_bindgen]
pub fn simulate(
    config_js: JsValue,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
) -> Result<JsValue, JsError> {
    let config: SimulationConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse config: {}", e)))?;

    let result = simulate_system(&config, &solar_profile, &wind_profile, &load_profile)
        .map_err(|e| JsError::new(&e))?;

    to_value(&result).map_err(|e| JsError::new(&format!("Failed to serialize result: {}", e)))
}

/// Calculate LCOE for a simulation result
///
/// # Arguments
/// * `sim_result_js` - SimulationResult as JsValue
/// * `solar_capacity` - Solar capacity MW
/// * `wind_capacity` - Wind capacity MW
/// * `storage_capacity` - Storage capacity MWh
/// * `clean_firm_capacity` - Clean firm capacity MW
/// * `costs_js` - CostParams as JsValue
///
/// # Returns
/// * LcoeResult as JsValue
#[wasm_bindgen]
pub fn compute_lcoe(
    sim_result_js: JsValue,
    solar_capacity: f64,
    wind_capacity: f64,
    storage_capacity: f64,
    clean_firm_capacity: f64,
    costs_js: JsValue,
) -> Result<JsValue, JsError> {
    let sim_result: SimulationResult = from_value(sim_result_js)
        .map_err(|e| JsError::new(&format!("Failed to parse simulation result: {}", e)))?;
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;

    let result = calculate_lcoe(
        &sim_result,
        solar_capacity,
        wind_capacity,
        storage_capacity,
        clean_firm_capacity,
        &costs,
    );

    to_value(&result).map_err(|e| JsError::new(&format!("Failed to serialize LCOE result: {}", e)))
}

/// Run full simulation and LCOE calculation in one call
///
/// # Arguments
/// * `config_js` - SimulationConfig as JsValue
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
///
/// # Returns
/// * Object with both simulation and LCOE results
#[wasm_bindgen]
pub fn simulate_and_calculate_lcoe(
    config_js: JsValue,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
) -> Result<JsValue, JsError> {
    let config: SimulationConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse config: {}", e)))?;
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;

    let sim_result = simulate_system(&config, &solar_profile, &wind_profile, &load_profile)
        .map_err(|e| JsError::new(&e))?;

    let lcoe_result = calculate_lcoe(
        &sim_result,
        config.solar_capacity,
        config.wind_capacity,
        config.storage_capacity,
        config.clean_firm_capacity,
        &costs,
    );

    // Combine results
    #[derive(serde::Serialize)]
    struct CombinedResult {
        simulation: SimulationResult,
        lcoe: LcoeResult,
    }

    let combined = CombinedResult {
        simulation: sim_result,
        lcoe: lcoe_result,
    };

    to_value(&combined)
        .map_err(|e| JsError::new(&format!("Failed to serialize combined result: {}", e)))
}

/// Run the optimizer (V2 hierarchical optimizer)
///
/// # Arguments
/// * `target_match` - Target clean match percentage (0-100)
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * OptimizerResult as JsValue
///
/// Note: If a model is loaded for the current zone/mode via `wasm_load_model()`,
/// it will be used automatically for faster candidate filtering.
#[wasm_bindgen]
pub fn optimize(
    target_match: f64,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    let result = run_v2_optimizer(
        target_match,
        &solar_profile,
        &wind_profile,
        &load_profile,
        &costs,
        &config,
        battery_mode,
        None, // Model-based optimization: model is looked up internally by optimizer
    )
    .map_err(|e| JsError::new(&e))?;

    to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize optimizer result: {}", e)))
}

#[cfg(feature = "experimental-v3")]
/// Run the optimizer (experimental V3 global-grid optimizer)
///
/// # Arguments
/// * `target_match` - Target clean match percentage (0-100)
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
/// * `config_js` - Optional OptimizerConfig as JsValue for runtime assumptions
///
/// # Returns
/// * OptimizerResult as JsValue
#[wasm_bindgen]
pub fn optimize_v3(
    target_match: f64,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    let result = run_v3_optimizer(
        target_match,
        &solar_profile,
        &wind_profile,
        &load_profile,
        &costs,
        battery_mode,
        &config,
    )
    .map_err(|e| JsError::new(&e))?;

    to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize optimizer result: {}", e)))
}

/// Get default cost parameters
#[wasm_bindgen]
pub fn get_default_costs() -> Result<JsValue, JsError> {
    let costs = CostParams::default_costs();
    to_value(&costs).map_err(|e| JsError::new(&format!("Failed to serialize costs: {}", e)))
}

/// Get default simulation config
#[wasm_bindgen]
pub fn get_default_simulation_config() -> Result<JsValue, JsError> {
    let config = SimulationConfig::with_defaults();
    to_value(&config).map_err(|e| JsError::new(&format!("Failed to serialize config: {}", e)))
}

/// Get default optimizer config
#[wasm_bindgen]
pub fn get_default_optimizer_config() -> Result<JsValue, JsError> {
    let config = OptimizerConfig::default();
    to_value(&config).map_err(|e| JsError::new(&format!("Failed to serialize config: {}", e)))
}

/// Run the V2 hierarchical optimizer
///
/// # Arguments
/// * `target_match` - Target clean match percentage (0-100)
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * OptimizerResult as JsValue
#[wasm_bindgen]
pub fn optimize_v2(
    target_match: f64,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    let result = run_v2_optimizer(
        target_match,
        &solar_profile,
        &wind_profile,
        &load_profile,
        &costs,
        &config,
        battery_mode,
        None,
    )
    .map_err(|e| JsError::new(&e))?;

    to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize optimizer result: {}", e)))
}

/// Run optimizer with model-based acceleration (if model is cached)
///
/// This is the preferred method when you have loaded a model via `wasm_load_model()`.
/// Falls back to greedy search if no model is cached for the zone/mode.
///
/// # Arguments
/// * `zone` - Zone name (must match the zone used when loading the model)
/// * `target_match` - Target clean match percentage (0-100)
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * OptimizerResult as JsValue
#[wasm_bindgen]
pub fn optimize_with_model(
    zone: String,
    target_match: f64,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    // Try to get cached model for this zone/mode
    let model = get_model(&zone, battery_mode);

    let result = run_v2_optimizer(
        target_match,
        &solar_profile,
        &wind_profile,
        &load_profile,
        &costs,
        &config,
        battery_mode,
        model.as_ref(),
    )
    .map_err(|e| JsError::new(&e))?;

    to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize optimizer result: {}", e)))
}

/// Run the incremental cost walk optimizer.
///
/// Mirrors the Python `run_incremental_cost_walk` strategy: starts from a zero
/// portfolio and incrementally adds the most cost-effective resource (smallest
/// LCOE-per-percentage-point ratio) until reaching the clean-match target,
/// halving step sizes when overshooting.
///
/// # Arguments
/// * `target_match` - Target clean match percentage (values >= 100 are capped to 99.5)
/// * `solar_profile` - Solar capacity factors (8760 hours)
/// * `wind_profile` - Wind capacity factors (8760 hours)
/// * `load_profile` - Load MW (8760 hours)
/// * `costs_js` - CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue (provides battery_efficiency,
///   max_demand_response, and the resource-enable flags)
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * IncrementalWalkResult as JsValue (includes the full walk_trace)
#[wasm_bindgen]
pub fn run_incremental_walk_wasm(
    target_match: f64,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    let result = run_incremental_walk(
        target_match,
        &solar_profile,
        &wind_profile,
        &load_profile,
        &costs,
        config.enable_solar,
        config.enable_wind,
        config.enable_storage,
        config.enable_clean_firm,
        battery_mode,
        config.battery_efficiency,
        config.max_demand_response,
    )
    .map_err(|e| JsError::new(&e))?;

    to_value(&result).map_err(|e| {
        JsError::new(&format!(
            "Failed to serialize incremental walk result: {}",
            e
        ))
    })
}

/// Run V2 optimizer sweep across multiple targets
///
/// # Arguments
/// * `targets` - Array of target percentages
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * Array of OptimizerResult as JsValue
#[wasm_bindgen]
pub fn optimize_v2_sweep(
    targets: Vec<f64>,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    let results = run_v2_sweep(
        &targets,
        &solar_profile,
        &wind_profile,
        &load_profile,
        &costs,
        &config,
        battery_mode,
        None,
    )
    .map_err(|e| JsError::new(&e))?;

    to_value(&results).map_err(|e| JsError::new(&format!("Failed to serialize results: {}", e)))
}

/// Run optimizer sweep with model-based acceleration
///
/// Uses cached model for faster candidate filtering if available.
/// Returns the same SweepResult structure as run_optimizer_sweep.
///
/// # Arguments
/// * `zone` - Zone name (must match loaded model)
/// * `targets` - Array of target percentages
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * SweepResult as JsValue (same format as run_optimizer_sweep)
#[wasm_bindgen]
pub fn optimize_sweep_with_model(
    zone: String,
    targets: Vec<f64>,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    // Get cached model
    let model = get_model(&zone, battery_mode);

    // Use V2 optimizer with model for each target
    let mut points = Vec::with_capacity(targets.len());

    for target in &targets {
        let result = run_v2_optimizer(
            *target,
            &solar_profile,
            &wind_profile,
            &load_profile,
            &costs,
            &config,
            battery_mode,
            model.as_ref(),
        );

        match result {
            Ok(r) => {
                // Run simulation and LCOE calculation to get breakdown
                let sim_config = SimulationConfig {
                    solar_capacity: r.solar_capacity,
                    wind_capacity: r.wind_capacity,
                    storage_capacity: r.storage_capacity,
                    clean_firm_capacity: r.clean_firm_capacity,
                    battery_efficiency: config.battery_efficiency,
                    max_demand_response: config.max_demand_response,
                    battery_mode,
                };

                let (solar_lcoe, wind_lcoe, storage_lcoe, clean_firm_lcoe, gas_lcoe) =
                    if let Ok(sim_result) =
                        simulate_system(&sim_config, &solar_profile, &wind_profile, &load_profile)
                    {
                        let lcoe_result = calculate_lcoe(
                            &sim_result,
                            r.solar_capacity,
                            r.wind_capacity,
                            r.storage_capacity,
                            r.clean_firm_capacity,
                            &costs,
                        );
                        (
                            lcoe_result.solar_lcoe,
                            lcoe_result.wind_lcoe,
                            lcoe_result.storage_lcoe,
                            lcoe_result.clean_firm_lcoe,
                            lcoe_result.gas_lcoe,
                        )
                    } else {
                        (0.0, 0.0, 0.0, 0.0, 0.0)
                    };

                points.push(SweepPoint {
                    target: *target,
                    achieved: r.achieved_clean_match,
                    solar: r.solar_capacity,
                    wind: r.wind_capacity,
                    storage: r.storage_capacity,
                    clean_firm: r.clean_firm_capacity,
                    lcoe: r.lcoe,
                    solar_lcoe,
                    wind_lcoe,
                    storage_lcoe,
                    clean_firm_lcoe,
                    gas_lcoe,
                    success: r.success,
                });
            }
            Err(_) => {
                points.push(SweepPoint {
                    target: *target,
                    achieved: 0.0,
                    solar: 0.0,
                    wind: 0.0,
                    storage: 0.0,
                    clean_firm: 0.0,
                    lcoe: 0.0,
                    solar_lcoe: 0.0,
                    wind_lcoe: 0.0,
                    storage_lcoe: 0.0,
                    clean_firm_lcoe: 0.0,
                    gas_lcoe: 0.0,
                    success: false,
                });
            }
        }
    }

    let sweep_result = SweepResult {
        points,
        elapsed_ms: 0.0,
    };

    to_value(&sweep_result)
        .map_err(|e| JsError::new(&format!("Failed to serialize results: {}", e)))
}

/// Evaluate a batch of portfolios (for Web Worker parallel processing)
///
/// # Arguments
/// * `portfolios_js` - Array of portfolio configurations
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `costs_js` - CostParams as JsValue
/// * `battery_mode` - Battery dispatch mode
/// * `config_js` - Optional OptimizerConfig as JsValue for runtime assumptions
///
/// # Returns
/// * Array of evaluation results
#[wasm_bindgen]
pub fn evaluate_batch(
    portfolios_js: JsValue,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    battery_mode: BatteryMode,
    config_js: Option<JsValue>,
) -> Result<JsValue, JsError> {
    #[derive(serde::Deserialize)]
    struct PortfolioInput {
        solar: f64,
        wind: f64,
        storage: f64,
        clean_firm: f64,
    }

    #[derive(serde::Serialize)]
    struct EvalOutput {
        solar: f64,
        wind: f64,
        storage: f64,
        clean_firm: f64,
        lcoe: f64,
        clean_match: f64,
    }

    let portfolios: Vec<PortfolioInput> = from_value(portfolios_js)
        .map_err(|e| JsError::new(&format!("Failed to parse portfolios: {}", e)))?;
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let runtime_config = match config_js {
        Some(value) if !value.is_undefined() && !value.is_null() => Some(
            from_value::<OptimizerConfig>(value)
                .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?,
        ),
        _ => None,
    };

    let mut results = Vec::with_capacity(portfolios.len());

    for p in portfolios {
        let config = SimulationConfig {
            solar_capacity: p.solar,
            wind_capacity: p.wind,
            storage_capacity: p.storage,
            clean_firm_capacity: p.clean_firm,
            battery_efficiency: runtime_config
                .as_ref()
                .map(|cfg| cfg.battery_efficiency)
                .unwrap_or(0.85),
            max_demand_response: runtime_config
                .as_ref()
                .map(|cfg| cfg.max_demand_response)
                .unwrap_or(0.0),
            battery_mode,
        };

        match simulate_system(&config, &solar_profile, &wind_profile, &load_profile) {
            Ok(sim_result) => {
                let lcoe_result = calculate_lcoe(
                    &sim_result,
                    p.solar,
                    p.wind,
                    p.storage,
                    p.clean_firm,
                    &costs,
                );

                results.push(EvalOutput {
                    solar: p.solar,
                    wind: p.wind,
                    storage: p.storage,
                    clean_firm: p.clean_firm,
                    lcoe: lcoe_result.total_lcoe,
                    clean_match: sim_result.clean_match_pct,
                });
            }
            Err(e) => {
                return Err(JsError::new(&format!("Simulation failed: {}", e)));
            }
        }
    }

    to_value(&results).map_err(|e| JsError::new(&format!("Failed to serialize results: {}", e)))
}

// Re-export types for TypeScript generation
#[wasm_bindgen]
pub fn battery_mode_default() -> BatteryMode {
    BatteryMode::Default
}

#[wasm_bindgen]
pub fn battery_mode_peak_shaver() -> BatteryMode {
    BatteryMode::PeakShaver
}

#[wasm_bindgen]
pub fn battery_mode_hybrid() -> BatteryMode {
    BatteryMode::Hybrid
}

// ============================================================================
// Model Cache WASM Exports
// ============================================================================

/// Load an empirical model into the cache for model-based optimization
///
/// # Arguments
/// * `zone` - Zone name (case-insensitive, e.g., "california", "texas")
/// * `battery_mode` - Battery mode (must match the mode used to generate the model)
/// * `bytes` - Model binary data (bincode serialized EmpiricalModel)
///
/// # Returns
/// * `Ok(())` if model loaded successfully
/// * `Err` if deserialization fails
///
/// # Example (TypeScript)
/// ```typescript
/// const response = await fetch('/models/california_hybrid.bin');
/// const bytes = new Uint8Array(await response.arrayBuffer());
/// wasm.wasm_load_model('california', BatteryMode.Hybrid, bytes);
/// ```
#[wasm_bindgen]
pub fn wasm_load_model(
    zone: String,
    battery_mode: BatteryMode,
    bytes: Vec<u8>,
) -> Result<(), JsError> {
    load_model_internal(&zone, battery_mode, &bytes)
        .map_err(|e| JsError::new(&format!("Failed to load model: {}", e)))
}

/// Check if a model is loaded in the cache
///
/// # Arguments
/// * `zone` - Zone name (case-insensitive)
/// * `battery_mode` - Battery mode
///
/// # Returns
/// * `true` if model is cached and ready for use
/// * `false` if model needs to be loaded
#[wasm_bindgen]
pub fn wasm_is_model_loaded(zone: String, battery_mode: BatteryMode) -> bool {
    is_model_loaded(&zone, battery_mode)
}

/// Clear all cached models to free memory
///
/// Call this when switching contexts or to reduce memory usage.
/// Models will need to be reloaded before model-based optimization can be used.
#[wasm_bindgen]
pub fn wasm_clear_models() {
    clear_models();
}

/// Get list of currently loaded models
///
/// # Returns
/// * Array of [zone, battery_mode] pairs as JSON
#[wasm_bindgen]
pub fn wasm_loaded_models() -> Result<JsValue, JsError> {
    let models = loaded_models();
    to_value(&models).map_err(|e| JsError::new(&format!("Failed to serialize: {}", e)))
}

/// Get model cache statistics
///
/// # Returns
/// * Object with { loaded: number, max: number }
#[wasm_bindgen]
pub fn wasm_cache_stats() -> Result<JsValue, JsError> {
    let (loaded, max) = optimizer::model_cache::cache_stats();

    #[derive(serde::Serialize)]
    struct CacheStats {
        loaded: usize,
        max: usize,
    }

    let stats = CacheStats { loaded, max };
    to_value(&stats).map_err(|e| JsError::new(&format!("Failed to serialize: {}", e)))
}

/// Calculate ELCC metrics for all resources
///
/// # Arguments
/// * `solar_capacity` - Solar capacity MW
/// * `wind_capacity` - Wind capacity MW
/// * `storage_capacity` - Storage capacity MWh
/// * `clean_firm_capacity` - Clean firm capacity MW
/// * `solar_profile` - Solar capacity factors (Float64Array)
/// * `wind_profile` - Wind capacity factors (Float64Array)
/// * `load_profile` - Load MW (Float64Array)
/// * `battery_mode_js` - Battery mode as string
/// * `battery_efficiency` - Battery round-trip efficiency
/// * `max_demand_response` - Maximum demand response fraction
///
/// # Returns
/// * ElccResult as JsValue
#[wasm_bindgen]
pub fn calculate_elcc_metrics(
    solar_capacity: f64,
    wind_capacity: f64,
    storage_capacity: f64,
    clean_firm_capacity: f64,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    battery_mode_js: JsValue,
    battery_efficiency: f64,
    max_demand_response: f64,
) -> Result<JsValue, JsError> {
    let battery_mode: BatteryMode = from_value(battery_mode_js).unwrap_or(BatteryMode::Default);

    let result = calculate_elcc(
        solar_capacity,
        wind_capacity,
        storage_capacity,
        clean_firm_capacity,
        &solar_profile,
        &wind_profile,
        &load_profile,
        battery_mode,
        battery_efficiency,
        max_demand_response,
    )
    .map_err(|e| JsError::new(&e))?;

    to_value(&result).map_err(|e| JsError::new(&format!("Failed to serialize ELCC result: {}", e)))
}

/// Compute hourly electricity prices
///
/// # Arguments
/// * `sim_result_js` - SimulationResult as JsValue
/// * `costs_js` - CostParams as JsValue
/// * `lcoe` - System LCOE $/MWh
/// * `pricing_method_js` - PricingMethod as JsValue
/// * `load_profile` - Load MW (Float64Array)
/// * `ordc_config_js` - Optional OrdcConfig as JsValue
/// * `elcc_result_js` - Optional ElccResult as JsValue
/// * `solar_capacity` - Solar capacity MW
/// * `wind_capacity` - Wind capacity MW
/// * `storage_capacity` - Storage capacity MWh
/// * `clean_firm_capacity` - Clean firm capacity MW
///
/// # Returns
/// * PricingResult as JsValue
#[wasm_bindgen]
pub fn compute_prices(
    sim_result_js: JsValue,
    costs_js: JsValue,
    lcoe: f64,
    pricing_method_js: JsValue,
    load_profile: Vec<f64>,
    ordc_config_js: JsValue,
    elcc_result_js: JsValue,
    solar_capacity: f64,
    wind_capacity: f64,
    storage_capacity: f64,
    clean_firm_capacity: f64,
) -> Result<JsValue, JsError> {
    let sim_result: SimulationResult = from_value(sim_result_js)
        .map_err(|e| JsError::new(&format!("Failed to parse simulation result: {}", e)))?;
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let pricing_method: PricingMethod =
        from_value(pricing_method_js).unwrap_or(PricingMethod::ScarcityBased);
    let ordc_config: Option<OrdcConfig> = from_value(ordc_config_js).ok();
    let elcc_result: Option<ElccResult> = from_value(elcc_result_js).ok();

    let result = compute_hourly_prices(
        &sim_result,
        &costs,
        lcoe,
        pricing_method,
        &load_profile,
        ordc_config.as_ref(),
        elcc_result.as_ref(),
        (
            solar_capacity,
            wind_capacity,
            storage_capacity,
            clean_firm_capacity,
        ),
    );

    to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize pricing result: {}", e)))
}

/// Run cost sweep with model-based acceleration
///
/// Uses cached model for faster candidate filtering if available.
///
/// # Arguments
/// * `zone` - Zone name (must match loaded model)
/// * `target_match` - Target clean match percentage
/// * `param_name` - Name of parameter to sweep
/// * `min_value` - Minimum parameter value
/// * `max_value` - Maximum parameter value
/// * `steps` - Number of steps in sweep
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `base_costs_js` - Base CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * CostSweepResult as JsValue
#[wasm_bindgen]
pub fn run_cost_sweep_with_model(
    zone: String,
    target_match: f64,
    param_name: String,
    min_value: f64,
    max_value: f64,
    steps: u32,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    base_costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    let base_costs: CostParams = from_value(base_costs_js)
        .map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    // Get cached model for this zone/mode
    let model = get_model(&zone, battery_mode);

    let mut points = Vec::with_capacity(steps as usize);
    let step_size = (max_value - min_value) / (steps - 1) as f64;

    for i in 0..steps {
        let param_value = min_value + step_size * i as f64;

        // Create modified costs with the swept parameter
        let mut costs = base_costs.clone();
        match param_name.as_str() {
            "solar_capex" => costs.solar_capex = param_value,
            "wind_capex" => costs.wind_capex = param_value,
            "storage_capex" => costs.storage_capex = param_value,
            "clean_firm_capex" => costs.clean_firm_capex = param_value,
            "gas_capex" => costs.gas_capex = param_value,
            "gas_price" => costs.gas_price = param_value,
            "solar_itc" => costs.solar_itc = param_value / 100.0,
            "wind_itc" => costs.wind_itc = param_value / 100.0,
            "storage_itc" => costs.storage_itc = param_value / 100.0,
            "clean_firm_itc" => costs.clean_firm_itc = param_value / 100.0,
            "discount_rate" => costs.discount_rate = param_value,
            _ => {
                return Err(JsError::new(&format!("Unknown parameter: {}", param_name)));
            }
        }

        // Run V2 optimizer with model (if available)
        let opt_result = run_v2_optimizer(
            target_match,
            &solar_profile,
            &wind_profile,
            &load_profile,
            &costs,
            &config,
            battery_mode,
            model.as_ref(),
        );

        match opt_result {
            Ok(r) => {
                points.push(CostSweepPoint {
                    param_value,
                    solar: r.solar_capacity,
                    wind: r.wind_capacity,
                    storage: r.storage_capacity,
                    clean_firm: r.clean_firm_capacity,
                    achieved: r.achieved_clean_match,
                    lcoe: r.lcoe,
                    success: r.success,
                });
            }
            Err(_) => {
                points.push(CostSweepPoint {
                    param_value,
                    solar: 0.0,
                    wind: 0.0,
                    storage: 0.0,
                    clean_firm: 0.0,
                    achieved: 0.0,
                    lcoe: 0.0,
                    success: false,
                });
            }
        }
    }

    let result = CostSweepResult {
        param_name,
        points,
        target_match,
        elapsed_ms: 0.0,
    };

    to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize cost sweep result: {}", e)))
}

/// Run cost sweep - optimize across a range of parameter values
///
/// # Arguments
/// * `target_match` - Target clean match percentage
/// * `param_name` - Name of parameter to sweep
/// * `min_value` - Minimum parameter value
/// * `max_value` - Maximum parameter value
/// * `steps` - Number of steps in sweep
/// * `solar_profile` - Solar capacity factors
/// * `wind_profile` - Wind capacity factors
/// * `load_profile` - Load MW
/// * `base_costs_js` - Base CostParams as JsValue
/// * `config_js` - OptimizerConfig as JsValue
/// * `battery_mode` - Battery dispatch mode
///
/// # Returns
/// * CostSweepResult as JsValue
#[wasm_bindgen]
pub fn run_cost_sweep(
    target_match: f64,
    param_name: String,
    min_value: f64,
    max_value: f64,
    steps: u32,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    base_costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    // Note: std::time::Instant not available in WASM, so we skip timing
    let base_costs: CostParams = from_value(base_costs_js)
        .map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    let mut points = Vec::with_capacity(steps as usize);
    let step_size = (max_value - min_value) / (steps - 1) as f64;

    for i in 0..steps {
        let param_value = min_value + step_size * i as f64;

        // Create modified costs with the swept parameter
        let mut costs = base_costs.clone();
        match param_name.as_str() {
            "solar_capex" => costs.solar_capex = param_value,
            "wind_capex" => costs.wind_capex = param_value,
            "storage_capex" => costs.storage_capex = param_value,
            "clean_firm_capex" => costs.clean_firm_capex = param_value,
            "gas_capex" => costs.gas_capex = param_value,
            "gas_price" => costs.gas_price = param_value,
            "solar_itc" => costs.solar_itc = param_value / 100.0,
            "wind_itc" => costs.wind_itc = param_value / 100.0,
            "storage_itc" => costs.storage_itc = param_value / 100.0,
            "clean_firm_itc" => costs.clean_firm_itc = param_value / 100.0,
            "discount_rate" => costs.discount_rate = param_value,
            _ => {
                return Err(JsError::new(&format!("Unknown parameter: {}", param_name)));
            }
        }

        // Run V2 optimizer at this parameter value (same as main optimize() function)
        let opt_result = run_v2_optimizer(
            target_match,
            &solar_profile,
            &wind_profile,
            &load_profile,
            &costs,
            &config,
            battery_mode,
            None,
        );

        match opt_result {
            Ok(r) => {
                points.push(CostSweepPoint {
                    param_value,
                    solar: r.solar_capacity,
                    wind: r.wind_capacity,
                    storage: r.storage_capacity,
                    clean_firm: r.clean_firm_capacity,
                    achieved: r.achieved_clean_match,
                    lcoe: r.lcoe,
                    success: r.success,
                });
            }
            Err(_) => {
                points.push(CostSweepPoint {
                    param_value,
                    solar: 0.0,
                    wind: 0.0,
                    storage: 0.0,
                    clean_firm: 0.0,
                    achieved: 0.0,
                    lcoe: 0.0,
                    success: false,
                });
            }
        }
    }

    let result = CostSweepResult {
        param_name,
        points,
        target_match,
        elapsed_ms: 0.0, // Timing not available in WASM
    };

    to_value(&result)
        .map_err(|e| JsError::new(&format!("Failed to serialize cost sweep result: {}", e)))
}

/// Run optimizer sweep and return structured result (uses V2 optimizer)
#[wasm_bindgen]
pub fn run_optimizer_sweep(
    targets: Vec<f64>,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    costs_js: JsValue,
    config_js: JsValue,
    battery_mode: BatteryMode,
) -> Result<JsValue, JsError> {
    // Note: std::time::Instant not available in WASM, so we skip timing
    let costs: CostParams =
        from_value(costs_js).map_err(|e| JsError::new(&format!("Failed to parse costs: {}", e)))?;
    let config: OptimizerConfig = from_value(config_js)
        .map_err(|e| JsError::new(&format!("Failed to parse optimizer config: {}", e)))?;

    // Use V2 optimizer for each target (same as main optimize() function)
    let mut points = Vec::with_capacity(targets.len());

    for target in &targets {
        let result = run_v2_optimizer(
            *target,
            &solar_profile,
            &wind_profile,
            &load_profile,
            &costs,
            &config,
            battery_mode,
            None,
        );

        match result {
            Ok(r) => {
                // Run simulation and LCOE calculation to get breakdown
                let sim_config = SimulationConfig {
                    solar_capacity: r.solar_capacity,
                    wind_capacity: r.wind_capacity,
                    storage_capacity: r.storage_capacity,
                    clean_firm_capacity: r.clean_firm_capacity,
                    battery_efficiency: config.battery_efficiency,
                    max_demand_response: config.max_demand_response,
                    battery_mode,
                };

                let (solar_lcoe, wind_lcoe, storage_lcoe, clean_firm_lcoe, gas_lcoe) =
                    if let Ok(sim_result) =
                        simulate_system(&sim_config, &solar_profile, &wind_profile, &load_profile)
                    {
                        let lcoe_result = calculate_lcoe(
                            &sim_result,
                            r.solar_capacity,
                            r.wind_capacity,
                            r.storage_capacity,
                            r.clean_firm_capacity,
                            &costs,
                        );
                        (
                            lcoe_result.solar_lcoe,
                            lcoe_result.wind_lcoe,
                            lcoe_result.storage_lcoe,
                            lcoe_result.clean_firm_lcoe,
                            lcoe_result.gas_lcoe,
                        )
                    } else {
                        (0.0, 0.0, 0.0, 0.0, 0.0)
                    };

                points.push(SweepPoint {
                    target: *target,
                    achieved: r.achieved_clean_match,
                    solar: r.solar_capacity,
                    wind: r.wind_capacity,
                    storage: r.storage_capacity,
                    clean_firm: r.clean_firm_capacity,
                    lcoe: r.lcoe,
                    solar_lcoe,
                    wind_lcoe,
                    storage_lcoe,
                    clean_firm_lcoe,
                    gas_lcoe,
                    success: r.success,
                });
            }
            Err(_) => {
                points.push(SweepPoint {
                    target: *target,
                    achieved: 0.0,
                    solar: 0.0,
                    wind: 0.0,
                    storage: 0.0,
                    clean_firm: 0.0,
                    lcoe: 0.0,
                    solar_lcoe: 0.0,
                    wind_lcoe: 0.0,
                    storage_lcoe: 0.0,
                    clean_firm_lcoe: 0.0,
                    gas_lcoe: 0.0,
                    success: false,
                });
            }
        }
    }

    let sweep_result = SweepResult {
        points,
        elapsed_ms: 0.0,
    }; // Timing not available in WASM

    to_value(&sweep_result)
        .map_err(|e| JsError::new(&format!("Failed to serialize sweep result: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        let version = get_version();
        assert!(!version.is_empty());
    }
}
