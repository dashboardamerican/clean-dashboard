/// Core Optimizer (V2 Hierarchical)
///
/// This is the production optimizer for the energy system simulator.
/// It finds the minimum-cost portfolio that achieves a precise clean energy target.
///
/// **Performance**: ~20ms per target, 0.1% precision
///
/// Algorithm (three-stage):
/// 1. Greedy expansion: Iteratively add resources by cost-efficiency
/// 2. Local refinement: Search neighborhood of best solution
/// 3. Binary search CF: Size clean firm precisely to hit target
///
/// With empirical model (optional):
/// 1. Empirical filtering: Pre-trained model prunes candidate space
/// 2. Parallel evaluation: Full simulation on candidates
/// 3. Refinement: Fine-grid search around top candidates
use crate::economics::calculate_lcoe;
use crate::optimizer::cache::{CachedResult, EvalCache};
use crate::optimizer::empirical_model::{EmpiricalModel, Portfolio};
use crate::simulation::simulate_system;
use crate::types::{BatteryMode, CostParams, OptimizerConfig, OptimizerResult};
use std::cmp::Ordering;
use std::collections::HashSet;
#[cfg(not(target_arch = "wasm32"))]
use std::sync::OnceLock;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

/// V2 optimizer configuration
#[derive(Clone, Debug)]
pub struct V2Config {
    /// Coarse tolerance for initial candidate filter
    pub coarse_tolerance: f64,
    /// Fine tolerance for refinement pass
    pub fine_tolerance: f64,
    /// Final precision target
    pub precision: f64,
    /// Number of top candidates for refinement
    pub top_k: usize,
    /// Maximum binary search iterations
    pub max_binary_search_iters: usize,
}

impl Default for V2Config {
    fn default() -> Self {
        Self {
            coarse_tolerance: 1.0,
            fine_tolerance: 0.3,
            precision: 0.1,
            top_k: 5,
            max_binary_search_iters: 20,
        }
    }
}

/// Runtime mode for V2 optimizer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum V2Mode {
    /// Existing production behavior.
    Fast,
    /// Fast pass followed by bounded local polish for better LCOE quality.
    Accurate,
}

impl Default for V2Mode {
    fn default() -> Self {
        Self::Fast
    }
}

/// Step schedule for bounded local polish in accurate mode.
#[derive(Clone, Debug)]
pub struct V2StepSchedule {
    pub solar_wind_steps: Vec<f64>,
    pub storage_steps: Vec<f64>,
    pub clean_firm_steps: Vec<f64>,
}

impl Default for V2StepSchedule {
    fn default() -> Self {
        Self {
            solar_wind_steps: vec![40.0, 20.0, 10.0],
            storage_steps: vec![80.0, 40.0, 20.0],
            clean_firm_steps: vec![10.0, 5.0, 2.0],
        }
    }
}

/// Tunables for accurate mode local polish.
#[derive(Clone, Debug)]
pub struct V2AccurateConfig {
    /// Maximum number of new simulator evaluations in polish stage.
    pub max_extra_evals: u32,
    /// Extra runtime budget in milliseconds. If <= 0.0, use fast-pass runtime.
    pub max_extra_ms: f64,
    /// Minimum absolute LCOE improvement required to accept a move.
    pub lcoe_improve_min: f64,
    /// Step schedule for multi-scale neighborhood search.
    pub step_schedule: V2StepSchedule,
    /// Feasibility tolerance for polish stage.
    pub target_tolerance: f64,
    /// Number of best local seeds to polish in accurate mode.
    pub multi_start_top_k: usize,
    /// Targets at/above this threshold are treated as hard and get larger eval budget.
    pub hard_target_threshold: f64,
    /// Maximum eval budget for hard targets or adaptive budget expansion.
    pub adaptive_hard_max_extra_evals: u32,
}

impl Default for V2AccurateConfig {
    fn default() -> Self {
        Self {
            max_extra_evals: 300,
            max_extra_ms: 0.0,
            lcoe_improve_min: 0.01,
            step_schedule: V2StepSchedule::default(),
            target_tolerance: 0.5,
            multi_start_top_k: 5,
            hard_target_threshold: 90.0,
            adaptive_hard_max_extra_evals: 900,
        }
    }
}

/// Reason accurate mode stopped searching.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum V2AccurateStopReason {
    CompletedSchedule,
    NoFurtherImprovement,
    EvalBudgetExhausted,
    TimeBudgetExhausted,
}

impl V2AccurateStopReason {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CompletedSchedule => "completed_schedule",
            Self::NoFurtherImprovement => "no_further_improvement",
            Self::EvalBudgetExhausted => "eval_budget_exhausted",
            Self::TimeBudgetExhausted => "time_budget_exhausted",
        }
    }
}

/// Diagnostics emitted by accurate-mode local polish.
#[derive(Clone, Debug)]
pub struct V2AccurateDiagnostics {
    pub seed_count: usize,
    pub start_lcoe: f64,
    pub end_lcoe: f64,
    pub accepted_moves: u32,
    pub rejected_feasible_moves: u32,
    pub extra_evals: u32,
    pub effective_max_extra_evals: u32,
    pub stop_reason: V2AccurateStopReason,
    pub improved_dimensions: Vec<&'static str>,
}

#[derive(Clone, Copy, Debug, Default)]
struct ImprovedDimensions {
    solar: bool,
    wind: bool,
    storage: bool,
    clean_firm: bool,
}

impl ImprovedDimensions {
    fn mark_delta(&mut self, before: &EvalResult, after: &EvalResult) {
        if (after.solar - before.solar).abs() > 1e-9 {
            self.solar = true;
        }
        if (after.wind - before.wind).abs() > 1e-9 {
            self.wind = true;
        }
        if (after.storage - before.storage).abs() > 1e-9 {
            self.storage = true;
        }
        if (after.clean_firm - before.clean_firm).abs() > 1e-9 {
            self.clean_firm = true;
        }
    }

    fn as_names(self) -> Vec<&'static str> {
        let mut out = Vec::new();
        if self.solar {
            out.push("solar");
        }
        if self.wind {
            out.push("wind");
        }
        if self.storage {
            out.push("storage");
        }
        if self.clean_firm {
            out.push("clean_firm");
        }
        out
    }
}

/// Evaluation result with full details
#[derive(Clone, Debug)]
pub struct EvalResult {
    pub solar: f64,
    pub wind: f64,
    pub storage: f64,
    pub clean_firm: f64,
    pub lcoe: f64,
    pub clean_match: f64,
}

impl EvalResult {
    /// Check if result is valid for target
    pub fn is_valid(&self, target: f64, tolerance: f64) -> bool {
        (self.clean_match - target).abs() < tolerance
    }
}

#[cfg(target_arch = "wasm32")]
fn now_ms() -> f64 {
    js_sys::Date::now()
}

#[cfg(not(target_arch = "wasm32"))]
fn now_ms() -> f64 {
    static START: OnceLock<Instant> = OnceLock::new();
    START.get_or_init(Instant::now).elapsed().as_secs_f64() * 1000.0
}

/// Evaluate a portfolio with caching
fn evaluate_cached(
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    cache: &mut EvalCache,
    battery_mode: BatteryMode,
) -> Result<EvalResult, String> {
    // Check cache first
    if let Some(cached) = cache.get(solar, wind, storage, clean_firm) {
        return Ok(EvalResult {
            solar,
            wind,
            storage,
            clean_firm,
            lcoe: cached.lcoe,
            clean_match: cached.clean_match,
        });
    }

    // Full evaluation
    let simulation_config =
        config.simulation_config_for_portfolio(solar, wind, storage, clean_firm, battery_mode);

    let sim_result = simulate_system(
        &simulation_config,
        solar_profile,
        wind_profile,
        load_profile,
    )?;
    let lcoe_result = calculate_lcoe(&sim_result, solar, wind, storage, clean_firm, costs);

    // Store in cache
    cache.put(
        solar,
        wind,
        storage,
        clean_firm,
        CachedResult {
            lcoe: lcoe_result.total_lcoe,
            clean_match: sim_result.clean_match_pct,
        },
    );

    Ok(EvalResult {
        solar,
        wind,
        storage,
        clean_firm,
        lcoe: lcoe_result.total_lcoe,
        clean_match: sim_result.clean_match_pct,
    })
}

/// Binary search to find CF that hits target clean match
fn binary_search_cf(
    solar: f64,
    wind: f64,
    storage: f64,
    target: f64,
    tolerance: f64,
    max_cf: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    cache: &mut EvalCache,
    battery_mode: BatteryMode,
    max_iters: usize,
) -> Result<(f64, EvalResult), String> {
    // Evaluate at CF=0 first
    let base = evaluate_cached(
        solar,
        wind,
        storage,
        0.0,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;

    if base.clean_match >= target - tolerance {
        cache.set_last_eval_count(1);
        return Ok((0.0, base));
    }

    let mut low = 0.0;
    let mut high = max_cf;
    let mut best_cf = 0.0;
    let mut best_result = base;
    let mut eval_count = 1u32;

    for _ in 0..max_iters {
        let mid = (low + high) / 2.0;

        let result = evaluate_cached(
            solar,
            wind,
            storage,
            mid,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
        )?;
        eval_count += 1;

        // Track closest to target
        if (result.clean_match - target).abs() < (best_result.clean_match - target).abs() {
            best_cf = mid;
            best_result = result.clone();
        }

        // Adjust search bounds
        if result.clean_match < target {
            low = mid;
        } else {
            high = mid;
        }

        // Check convergence
        if (result.clean_match - target).abs() < tolerance / 2.0 {
            break;
        }

        // Check if bounds have converged
        if high - low < 0.1 {
            break;
        }
    }

    cache.set_last_eval_count(eval_count);
    Ok((best_cf, best_result))
}

#[allow(clippy::too_many_arguments)]
fn polish_vre_overshoot_by_scaling(
    solar: f64,
    wind: f64,
    storage: f64,
    target: f64,
    tolerance: f64,
    max_cf: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    cache: &mut EvalCache,
    battery_mode: BatteryMode,
    max_iters: usize,
) -> Result<(Option<(f64, f64, f64, f64, EvalResult)>, u32), String> {
    let renewable_capacity = if config.enable_solar { solar } else { 0.0 }
        + if config.enable_wind { wind } else { 0.0 }
        + if config.enable_storage { storage } else { 0.0 };
    if renewable_capacity <= 1e-9 {
        return Ok((None, 0));
    }

    let base = evaluate_cached(
        solar,
        wind,
        storage,
        0.0,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;
    let mut eval_count = 1u32;
    if base.clean_match <= target + tolerance {
        return Ok((None, eval_count));
    }

    let zero = evaluate_cached(
        0.0,
        0.0,
        0.0,
        0.0,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;
    eval_count += 1;
    if zero.clean_match > target {
        return Ok((None, eval_count));
    }

    let mut low = 0.0;
    let mut high = 1.0;
    let mut best_scale = 1.0;
    let mut best_result = base;

    for _ in 0..max_iters {
        let scale = (low + high) / 2.0;
        let scaled_solar = if config.enable_solar {
            solar * scale
        } else {
            0.0
        };
        let scaled_wind = if config.enable_wind {
            wind * scale
        } else {
            0.0
        };
        let scaled_storage = if config.enable_storage {
            storage * scale
        } else {
            0.0
        };
        let result = evaluate_cached(
            scaled_solar,
            scaled_wind,
            scaled_storage,
            0.0,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
        )?;
        eval_count += 1;

        if (result.clean_match - target).abs() < (best_result.clean_match - target).abs() {
            best_scale = scale;
            best_result = result.clone();
        }

        if result.clean_match < target {
            low = scale;
        } else {
            high = scale;
        }

        if (result.clean_match - target).abs() < tolerance / 2.0 {
            break;
        }
    }

    let scaled_solar = if config.enable_solar {
        solar * best_scale
    } else {
        0.0
    };
    let scaled_wind = if config.enable_wind {
        wind * best_scale
    } else {
        0.0
    };
    let scaled_storage = if config.enable_storage {
        storage * best_scale
    } else {
        0.0
    };

    let (cf, result) =
        if config.enable_clean_firm && max_cf > 0.0 && best_result.clean_match < target - tolerance
        {
            let (cf, result) = binary_search_cf(
                scaled_solar,
                scaled_wind,
                scaled_storage,
                target,
                tolerance,
                max_cf,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
                12,
            )?;
            eval_count += cache.last_eval_count();
            (cf, result)
        } else {
            (0.0, best_result)
        };

    cache.set_last_eval_count(eval_count);
    Ok((
        Some((scaled_solar, scaled_wind, scaled_storage, cf, result)),
        eval_count,
    ))
}

/// Run the V2 hierarchical optimizer
///
/// Uses a two-phase approach:
/// 1. Fast greedy expansion (like V1) to find approximate solution
/// 2. Localized grid refinement around the greedy result
///
/// # Arguments
/// * `target` - Target clean match percentage (0-100)
/// * `solar_profile` - Solar capacity factors (8760 hours)
/// * `wind_profile` - Wind capacity factors (8760 hours)
/// * `load_profile` - Load MW (8760 hours)
/// * `costs` - Cost parameters
/// * `config` - Optimizer configuration
/// * `battery_mode` - Battery dispatch mode
/// * `model` - Pre-trained empirical model (optional, for future use)
///
/// # Returns
/// * OptimizerResult with optimal portfolio
pub fn run_v2_optimizer(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: Option<&EmpiricalModel>,
) -> Result<OptimizerResult, String> {
    // Production default is Accurate: one quality bar everywhere (2026-07-05).
    // The certified numbers (97%+ within 1% of ground truth) describe Accurate
    // mode; Fast is an internal building block, not a user-facing tier.
    run_v2_optimizer_mode(
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
        model,
        V2Mode::Accurate,
        None,
    )
}

/// Run the V2 optimizer with an explicit runtime mode.
pub fn run_v2_optimizer_mode(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: Option<&EmpiricalModel>,
    mode: V2Mode,
    accurate_config: Option<&V2AccurateConfig>,
) -> Result<OptimizerResult, String> {
    let (result, _) = run_v2_optimizer_mode_detailed(
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
        model,
        mode,
        accurate_config,
    )?;
    Ok(result)
}

/// Run the V2 optimizer with explicit runtime mode and return optional diagnostics.
pub fn run_v2_optimizer_mode_detailed(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: Option<&EmpiricalModel>,
    mode: V2Mode,
    accurate_config: Option<&V2AccurateConfig>,
) -> Result<(OptimizerResult, Option<V2AccurateDiagnostics>), String> {
    let mut cache = EvalCache::new();
    run_v2_optimizer_mode_with_cache(
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
        model,
        mode,
        accurate_config,
        &mut cache,
        None,
    )
}

/// Fast-mode mini-polish tunables (targets in [90, 99) and warm-started sweep
/// targets). Bounded: single stage of coarse steps (build_stage_schedule appends
/// the micro stages, but the eval cap is the real limiter), small eval budget,
/// short time backstop. Fast mode is +1.5-3.2% vs corpus at t=90-98 without any
/// polish (optimizer_research F20); this buys back most of that for ~80 sims.
fn fast_mini_polish_config() -> V2AccurateConfig {
    V2AccurateConfig {
        max_extra_evals: 80,
        max_extra_ms: 300.0,
        lcoe_improve_min: 0.01,
        step_schedule: V2StepSchedule {
            solar_wind_steps: vec![40.0],
            storage_steps: vec![80.0],
            clean_firm_steps: vec![10.0],
        },
        target_tolerance: 0.5,
        multi_start_top_k: 1,
        // Disable the hard-target budget expansion paths: the caps above ARE
        // the budget in mini-polish mode.
        hard_target_threshold: f64::INFINITY,
        adaptive_hard_max_extra_evals: 80,
    }
}

/// Polish tunables for warm-started fast-mode sweep targets below t=99. The
/// warm seed replaces the whole greedy pass, so this polish is the only local
/// search on the path — it gets a second (medium) stage and a slightly larger
/// budget than the single-target mini-polish, while staying far cheaper than
/// the greedy pass it replaces (~160 vs ~700-1200 sims). Targets in the
/// storage-restructuring / CF-entry band (t >= 85) get a bigger budget and
/// multi-start seeds: that is where the optimal mix pivots fastest between
/// adjacent sweep targets (F15/F17) and a single-seed coarse polish stalls.
fn fast_warm_polish_config(target: f64) -> V2AccurateConfig {
    if target >= 55.0 {
        // Split budget: t=55-70 is the mix-entry region (solar/storage joining
        // a wind-heavy path) where 300 evals suffice; t>=75 approaches the
        // CF-entry fold where the pivot is larger (e.g. southeast t=85) and
        // needs the fuller 400-eval budget.
        let evals = if target >= 75.0 { 400 } else { 300 };
        V2AccurateConfig {
            max_extra_evals: evals,
            max_extra_ms: 600.0,
            lcoe_improve_min: 0.01,
            step_schedule: V2StepSchedule::default(),
            target_tolerance: 0.5,
            multi_start_top_k: 3,
            hard_target_threshold: f64::INFINITY,
            adaptive_hard_max_extra_evals: evals,
        }
    } else {
        V2AccurateConfig {
            max_extra_evals: 160,
            max_extra_ms: 400.0,
            lcoe_improve_min: 0.01,
            step_schedule: V2StepSchedule {
                solar_wind_steps: vec![40.0, 20.0],
                storage_steps: vec![80.0, 40.0],
                clean_firm_steps: vec![10.0, 5.0],
            },
            target_tolerance: 0.5,
            multi_start_top_k: 1,
            hard_target_threshold: f64::INFINITY,
            adaptive_hard_max_extra_evals: 160,
        }
    }
}

/// Re-size a previous sweep target's portfolio for a new target by adjusting CF
/// (and, on overshoot, scaling VRE down) using the existing precision machinery.
/// Returns None when the warm start cannot be made feasible for the new target
/// (caller falls back to the full pipeline). Warm-start continuation across
/// sweep targets is safe: the optimal path never folds (optimizer_research F12).
#[allow(clippy::too_many_arguments)]
fn resize_warm_start_for_target(
    warm: &OptimizerResult,
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    cache: &mut EvalCache,
) -> Result<Option<OptimizerResult>, String> {
    let precision = V2Config::default().precision;
    let feasibility_tolerance = V2AccurateConfig::default().target_tolerance;

    // An essentially empty warm portfolio (e.g. the t=0 sweep point) would make
    // CF the sole closer and seed the polish with an all-CF cover — exactly the
    // F20 degenerate mode. Let the full pipeline handle that target instead.
    // (A 25 MW gate was tried and rejected: pushing the continuation entry
    // point later reshuffles the whole path along the flat solar<->wind ridge
    // and produced worse mid-target results in texas/florida.)
    let warm_total = warm.solar_capacity
        + warm.wind_capacity
        + warm.storage_capacity
        + warm.clean_firm_capacity;
    if warm_total < 1.0 {
        return Ok(None);
    }

    // Clamp to the current config's bounds/enabled flags before re-sizing.
    let (solar, wind, storage, _) = clamp_candidate(
        warm.solar_capacity,
        warm.wind_capacity,
        warm.storage_capacity,
        warm.clean_firm_capacity,
        config,
    );
    let max_cf = if config.enable_clean_firm {
        config.max_clean_firm
    } else {
        0.0
    };

    let mut warm_evals = 0u32;

    // Evaluate the warm portfolio as-is (with its CF) at the new target.
    let warm_cf = if config.enable_clean_firm {
        warm.clean_firm_capacity.clamp(0.0, max_cf)
    } else {
        0.0
    };
    let base = evaluate_cached(
        solar,
        wind,
        storage,
        warm_cf,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;
    warm_evals += 1;
    let mut best = (solar, wind, storage, warm_cf, base.clone());

    // Undershoot (ascending sweep): scale VRE up with CF held fixed. This
    // preserves the near-optimal portfolio SHAPE from the previous target;
    // closing the whole gap with CF instead would hand the polish a CF-heavy
    // seed it must laboriously exchange away (the F20 degenerate-cover trap).
    if base.clean_match < target - precision {
        let vre_total = solar + wind + storage;
        if vre_total > 1e-9 {
            let mut max_scale: f64 = 4.0;
            if solar > 1e-9 {
                max_scale = max_scale.min(config.max_solar / solar);
            }
            if wind > 1e-9 {
                max_scale = max_scale.min(config.max_wind / wind);
            }
            if storage > 1e-9 {
                max_scale = max_scale.min(config.max_storage / storage);
            }
            if max_scale > 1.0 {
                let mut low = 1.0f64;
                let mut high = max_scale;
                for _ in 0..12 {
                    let scale = (low + high) / 2.0;
                    let (cs, cw, cst, ccf) = clamp_candidate(
                        solar * scale,
                        wind * scale,
                        storage * scale,
                        warm_cf,
                        config,
                    );
                    let eval = evaluate_cached(
                        cs,
                        cw,
                        cst,
                        ccf,
                        solar_profile,
                        wind_profile,
                        load_profile,
                        costs,
                        config,
                        cache,
                        battery_mode,
                    )?;
                    warm_evals += 1;
                    if (eval.clean_match - target).abs() < (best.4.clean_match - target).abs() {
                        best = (cs, cw, cst, ccf, eval.clone());
                    }
                    if eval.clean_match < target {
                        low = scale;
                    } else {
                        high = scale;
                    }
                    if (eval.clean_match - target).abs() < precision / 2.0 {
                        break;
                    }
                }
            }
        }

        // Still short (VRE saturation or scale capped): let CF close the
        // residual gap precisely, as usual.
        if best.4.clean_match < target - precision && config.enable_clean_firm && max_cf > 0.0 {
            let (cf, result) = binary_search_cf(
                best.0,
                best.1,
                best.2,
                target,
                precision / 2.0,
                max_cf,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
                15,
            )?;
            warm_evals += cache.last_eval_count();
            if (result.clean_match - target).abs() < (best.4.clean_match - target).abs() {
                best = (best.0, best.1, best.2, cf, result);
            }
        }
        // Candidates B/C: alternative VRE levels with CF closing the gap.
        // Near the CF-entry fold the optimal mix pivots toward CF (F15/F17):
        // the best portfolio may keep the previous VRE (B: ratio 1.0) or SHED
        // a chunk of it (C: ratios < 1.0 — e.g. northwest t=95's optimum is
        // roughly half of t=90's VRE plus CF). Pick the cheapest feasible seed.
        if config.enable_clean_firm && max_cf > 0.0 {
            let mut vre_ratios = vec![1.0];
            if target >= 75.0 {
                vre_ratios.push(0.75);
                vre_ratios.push(0.5);
            }
            for ratio in vre_ratios {
                let (cs, cw, cst, _) =
                    clamp_candidate(solar * ratio, wind * ratio, storage * ratio, 0.0, config);
                let (cf_b, result_b) = binary_search_cf(
                    cs,
                    cw,
                    cst,
                    target,
                    precision / 2.0,
                    max_cf,
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    config,
                    cache,
                    battery_mode,
                    15,
                )?;
                warm_evals += cache.last_eval_count();
                let b_feasible = (result_b.clean_match - target).abs() <= feasibility_tolerance;
                let best_feasible = (best.4.clean_match - target).abs() <= feasibility_tolerance;
                if b_feasible && (!best_feasible || result_b.lcoe + 1e-9 < best.4.lcoe) {
                    best = (cs, cw, cst, cf_b, result_b);
                }
            }
        }
    }

    if best.4.clean_match > target + precision {
        // Overshoot (descending sweeps / large target drops). First see if
        // shrinking CF alone recovers the target (binary_search_cf handles
        // this: it starts from the CF=0 base and re-sizes upward as needed).
        if best.3 > 0.0 {
            let (cf, result) = binary_search_cf(
                best.0,
                best.1,
                best.2,
                target,
                precision / 2.0,
                max_cf,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
                15,
            )?;
            warm_evals += cache.last_eval_count();
            if (result.clean_match - target).abs() < (best.4.clean_match - target).abs() {
                best = (best.0, best.1, best.2, cf, result);
            }
        }
        // Still overshooting: the VRE alone overshoots — scale it down,
        // mirroring the fast-pass finishing step.
        if best.4.clean_match > target + precision {
            let (polished, polish_evals) = polish_vre_overshoot_by_scaling(
                best.0,
                best.1,
                best.2,
                target,
                precision / 2.0,
                max_cf,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
                16,
            )?;
            warm_evals += polish_evals;
            if let Some((ps, pw, pst, pcf, pres)) = polished {
                if (pres.clean_match - target).abs() < (best.4.clean_match - target).abs() {
                    best = (ps, pw, pst, pcf, pres);
                }
            }
        }
    }

    if (best.4.clean_match - target).abs() > feasibility_tolerance {
        return Ok(None);
    }

    Ok(Some(OptimizerResult {
        solar_capacity: best.0,
        wind_capacity: best.1,
        storage_capacity: best.2,
        clean_firm_capacity: best.3,
        achieved_clean_match: best.4.clean_match,
        lcoe: best.4.lcoe,
        num_evaluations: warm_evals,
        success: (best.4.clean_match - target).abs() < precision,
    }))
}

#[allow(clippy::too_many_arguments)]
fn run_v2_optimizer_mode_with_cache(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: Option<&EmpiricalModel>,
    mode: V2Mode,
    accurate_config: Option<&V2AccurateConfig>,
    cache: &mut EvalCache,
    warm_start: Option<&OptimizerResult>,
) -> Result<(OptimizerResult, Option<V2AccurateDiagnostics>), String> {
    // Self-consistent LCOE costs ~3 extra financial-loop passes per evaluation
    // and re-ranks candidates by <0.5 normalized units (measured, F26). Search
    // on the plain objective; reprice only the final answer honestly.
    if costs.self_consistent_lcoe {
        let mut search_costs = costs.clone();
        search_costs.self_consistent_lcoe = false;
        let (mut result, diagnostics) = run_v2_optimizer_mode_with_cache(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            &search_costs,
            config,
            battery_mode,
            model,
            mode,
            accurate_config,
            cache,
            warm_start,
        )?;
        // Reprice under full semantics, bypassing the cache (it holds
        // plain-LCOE values for these portfolios).
        let sim_config = config.simulation_config_for_portfolio(
            result.solar_capacity,
            result.wind_capacity,
            result.storage_capacity,
            result.clean_firm_capacity,
            battery_mode,
        );
        let sim = simulate_system(&sim_config, solar_profile, wind_profile, load_profile)?;
        let lcoe = calculate_lcoe(
            &sim,
            result.solar_capacity,
            result.wind_capacity,
            result.storage_capacity,
            result.clean_firm_capacity,
            costs,
        );
        result.lcoe = lcoe.total_lcoe;
        return Ok((result, diagnostics));
    }

    let fast_start_ms = now_ms();

    // Warm-start continuation (sweep mode): re-size the previous target's
    // portfolio for this target and, when feasible, skip the greedy/model fast
    // pass entirely — the polish stage walks the warm seed to the new optimum
    // (safe: the optimal path never folds across targets, F12).
    let warm_result = if let Some(warm) = warm_start {
        resize_warm_start_for_target(
            warm,
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            cache,
        )?
    } else {
        None
    };
    let from_warm = warm_result.is_some();

    let fast_result = match warm_result {
        Some(result) => result,
        None => run_v2_optimizer_fast_with_cache(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            model,
            cache,
        )?,
    };

    // Terminal targets (>= 99%) always get the polish stage, even in Fast mode: the
    // fast pass can degenerate to an all-CF cover there (up to +48-56% LCOE vs dense
    // ground truth; optimizer_research F20) and only the oblique-move polish walks CF
    // back down the storage+CF frontier.
    let force_terminal_polish = target >= 99.0;
    // Fast mode below the terminal threshold gets a BOUNDED mini-polish when the
    // target is high (fast is +1.5-3.2% vs corpus at t=90-98, F20) or when the
    // result came from a warm start (which had no greedy search of its own).
    let fast_mini_polish =
        mode == V2Mode::Fast && !force_terminal_polish && (target >= 90.0 || from_warm);
    if mode == V2Mode::Fast && !force_terminal_polish && !fast_mini_polish {
        let closed = close_band_without_cf(
            fast_result,
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
        )?;
        return Ok((closed, None));
    }

    let cfg = if fast_mini_polish {
        if from_warm {
            fast_warm_polish_config(target)
        } else {
            fast_mini_polish_config()
        }
    } else if from_warm && !force_terminal_polish && accurate_config.is_none() {
        // Warm-started accurate sweep targets: the seed is already near the
        // valley floor, so the stall-triggered budget expansion (designed for
        // cold greedy results) mostly buys cache churn, not quality. Trim the
        // budgets; explicit caller configs are honored unchanged.
        V2AccurateConfig {
            max_extra_evals: 250,
            adaptive_hard_max_extra_evals: 500,
            multi_start_top_k: 3,
            ..V2AccurateConfig::default()
        }
    } else {
        accurate_config.cloned().unwrap_or_default()
    };
    let fast_runtime_ms = now_ms() - fast_start_ms;
    let mut extra_ms_budget = if cfg.max_extra_ms > 0.0 {
        cfg.max_extra_ms
    } else {
        fast_runtime_ms
    };
    if from_warm && !fast_mini_polish {
        // No fast pass ran, so fast_runtime_ms is just the cheap warm re-size.
        // Give the polish (the only search stage on this path) a real time
        // backstop; the eval caps remain the deterministic work limiter.
        extra_ms_budget = extra_ms_budget.max(800.0);
    }
    if force_terminal_polish {
        // The terminal CF walk-down (all-CF cover -> storage+CF frontier) needs more
        // than the fast-pass runtime; don't let wall-clock truncate it. The eval cap
        // (adaptive_hard_max_extra_evals) is the real work limiter — a generous time
        // backstop keeps results deterministic under CPU contention (parallel tests,
        // busy browsers) without unbounded latency.
        extra_ms_budget = extra_ms_budget.max(2000.0);
    }

    let (result, diagnostics) = run_v2_accurate_polish(
        target,
        &fast_result,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
        cache,
        &cfg,
        extra_ms_budget,
    )?;
    let result = close_band_without_cf(
        result,
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;
    Ok((result, Some(diagnostics)))
}

/// Final-mile band closer for configs where clean firm is DISABLED. CF is
/// normally the precision instrument that lands clean_match inside the target
/// band (binary-searched to +-precision); without it the pipeline can finish
/// just outside the band on chunky renewable/storage steps (combos corpus:
/// california no_cf t90 stalled at |dev| = 0.516). Bisect the smallest uniform
/// scale-up of the enabled resources that re-enters the band; constraint
/// compliance takes priority over the small LCOE increase it costs. No-op when
/// CF is enabled or the result is already feasible.
#[allow(clippy::too_many_arguments)]
fn close_band_without_cf(
    result: OptimizerResult,
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    cache: &mut EvalCache,
    battery_mode: BatteryMode,
) -> Result<OptimizerResult, String> {
    let tolerance = V2AccurateConfig::default().target_tolerance;
    let undershoot = target - tolerance - result.achieved_clean_match;
    if config.enable_clean_firm || undershoot <= 0.0 {
        return Ok(result);
    }

    let (s, w, st) = (
        result.solar_capacity,
        result.wind_capacity,
        result.storage_capacity,
    );
    if s + w + st <= 1e-9 {
        return Ok(result);
    }
    let mut max_scale: f64 = 3.0;
    if s > 1e-9 {
        max_scale = max_scale.min(config.max_solar / s);
    }
    if w > 1e-9 {
        max_scale = max_scale.min(config.max_wind / w);
    }
    if st > 1e-9 {
        max_scale = max_scale.min(config.max_storage / st);
    }
    if max_scale <= 1.0 {
        return Ok(result); // saturated: band genuinely unreachable
    }

    // Smallest scale that re-enters the band = cheapest feasible correction.
    let mut low = 1.0f64;
    let mut high = max_scale;
    let mut best: Option<(f64, f64, f64, EvalResult)> = None;
    let mut extra_evals = 0u32;
    for _ in 0..16 {
        let scale = (low + high) / 2.0;
        let (cs, cw, cst, _) =
            clamp_candidate(s * scale, w * scale, st * scale, 0.0, config);
        let eval = evaluate_cached(
            cs,
            cw,
            cst,
            0.0,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
        )?;
        extra_evals += 1;
        if eval.clean_match >= target - tolerance {
            if eval.clean_match <= target + tolerance {
                best = Some((cs, cw, cst, eval.clone()));
            }
            high = scale;
        } else {
            low = scale;
        }
        if high - low < 1e-4 {
            break;
        }
    }

    match best {
        Some((cs, cw, cst, eval)) => Ok(OptimizerResult {
            solar_capacity: cs,
            wind_capacity: cw,
            storage_capacity: cst,
            clean_firm_capacity: 0.0,
            achieved_clean_match: eval.clean_match,
            lcoe: eval.lcoe,
            num_evaluations: result.num_evaluations.saturating_add(extra_evals),
            success: (eval.clean_match - target).abs() < V2Config::default().precision,
        }),
        None => Ok(result),
    }
}

/// Internal fast-path optimizer entrypoint that accepts an external cache.
/// Used by sweep mode to reuse cached evaluations across multiple targets.
fn run_v2_optimizer_fast_with_cache(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: Option<&EmpiricalModel>,
    cache: &mut EvalCache,
) -> Result<OptimizerResult, String> {
    let v2_config = V2Config::default();
    let mut total_evals = 0u32;

    // Use model-based approach for high targets (>=85%) now that we have full LCOE
    // estimation including gas fuel costs. The model stores clean_match, peak_gas,
    // AND gas_generation, enabling accurate LCOE ranking.
    let use_model = model.is_some() && target >= 85.0;

    let best = if use_model {
        run_model_based_optimization(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            model.unwrap(),
            &v2_config,
            cache,
            &mut total_evals,
        )?
    } else {
        run_greedy_based_optimization(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            &v2_config,
            cache,
            &mut total_evals,
        )?
    };

    // Final precision polish
    let (final_cf, final_result) = binary_search_cf(
        best.solar,
        best.wind,
        best.storage,
        target,
        v2_config.precision / 2.0,
        if config.enable_clean_firm {
            config.max_clean_firm
        } else {
            0.0
        },
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
        15,
    )?;
    total_evals += cache.last_eval_count();

    let (final_solar, final_wind, final_storage, final_cf, final_result) =
        if final_result.clean_match > target + v2_config.precision {
            let (polished, polish_evals) = polish_vre_overshoot_by_scaling(
                best.solar,
                best.wind,
                best.storage,
                target,
                v2_config.precision / 2.0,
                if config.enable_clean_firm {
                    config.max_clean_firm
                } else {
                    0.0
                },
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
                16,
            )?;
            total_evals += polish_evals;
            if let Some((solar, wind, storage, cf, result)) = polished {
                if (result.clean_match - target).abs() < (final_result.clean_match - target).abs() {
                    (solar, wind, storage, cf, result)
                } else {
                    (best.solar, best.wind, best.storage, final_cf, final_result)
                }
            } else {
                (best.solar, best.wind, best.storage, final_cf, final_result)
            }
        } else {
            (best.solar, best.wind, best.storage, final_cf, final_result)
        };

    Ok(OptimizerResult {
        solar_capacity: final_solar,
        wind_capacity: final_wind,
        storage_capacity: final_storage,
        clean_firm_capacity: final_cf,
        achieved_clean_match: final_result.clean_match,
        lcoe: final_result.lcoe,
        num_evaluations: total_evals,
        success: (final_result.clean_match - target).abs() < v2_config.precision,
    })
}

const MICRO_SOLAR_WIND_STEPS: [f64; 3] = [5.0, 2.0, 1.0];
const MICRO_STORAGE_STEPS: [f64; 3] = [5.0, 2.0, 1.0];
const MICRO_CLEAN_FIRM_STEPS: [f64; 3] = [2.0, 1.0, 0.5];

fn build_stage_schedule(step_schedule: &V2StepSchedule) -> Vec<(f64, f64, f64)> {
    let mut out = Vec::new();
    let base_len = step_schedule
        .solar_wind_steps
        .len()
        .min(step_schedule.storage_steps.len())
        .min(step_schedule.clean_firm_steps.len());

    for idx in 0..base_len {
        out.push((
            step_schedule.solar_wind_steps[idx],
            step_schedule.storage_steps[idx],
            step_schedule.clean_firm_steps[idx],
        ));
    }

    for idx in 0..MICRO_SOLAR_WIND_STEPS.len() {
        let stage = (
            MICRO_SOLAR_WIND_STEPS[idx],
            MICRO_STORAGE_STEPS[idx],
            MICRO_CLEAN_FIRM_STEPS[idx],
        );
        if !out.iter().any(|existing| {
            (existing.0 - stage.0).abs() < 1e-9
                && (existing.1 - stage.1).abs() < 1e-9
                && (existing.2 - stage.2).abs() < 1e-9
        }) {
            out.push(stage);
        }
    }

    out
}

fn compare_eval_candidates(a: &EvalResult, b: &EvalResult, target: f64) -> Ordering {
    compare_f64(a.lcoe, b.lcoe)
        .then_with(|| {
            compare_f64(
                (a.clean_match - target).abs(),
                (b.clean_match - target).abs(),
            )
        })
        .then_with(|| compare_f64(a.solar, b.solar))
        .then_with(|| compare_f64(a.wind, b.wind))
        .then_with(|| compare_f64(a.storage, b.storage))
        .then_with(|| compare_f64(a.clean_firm, b.clean_firm))
}

fn better_eval_candidate(candidate: &EvalResult, incumbent: &EvalResult, target: f64) -> bool {
    compare_eval_candidates(candidate, incumbent, target).is_lt()
}

fn portfolio_key(solar: f64, wind: f64, storage: f64, clean_firm: f64) -> (i32, i32, i32, i32) {
    (
        (solar * 10.0).round() as i32,
        (wind * 10.0).round() as i32,
        (storage * 10.0).round() as i32,
        (clean_firm * 10.0).round() as i32,
    )
}

#[allow(clippy::too_many_arguments)]
fn select_multistart_seeds(
    target: f64,
    tolerance: f64,
    top_k: usize,
    start_eval: &EvalResult,
    stage_schedule: &[(f64, f64, f64)],
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    cache: &mut EvalCache,
    extra_evals: &mut u32,
    polish_start_ms: f64,
    extra_ms_budget: f64,
    max_extra_evals: u32,
) -> Result<Vec<EvalResult>, String> {
    let mut points = Vec::new();
    points.push((
        start_eval.solar,
        start_eval.wind,
        start_eval.storage,
        start_eval.clean_firm,
    ));

    let seed_stage_count = stage_schedule.len().min(2);
    for stage in stage_schedule.iter().take(seed_stage_count) {
        let moves = generate_stage_moves(stage.0, stage.1, stage.2, config);
        for (ds, dw, dst, dcf) in moves {
            let candidate = clamp_candidate(
                start_eval.solar + ds,
                start_eval.wind + dw,
                start_eval.storage + dst,
                start_eval.clean_firm + dcf,
                config,
            );
            points.push(candidate);
        }
    }

    if config.enable_clean_firm {
        points.push((start_eval.solar, start_eval.wind, start_eval.storage, 0.0));
    }

    let mut seen = HashSet::new();
    let mut seeds = Vec::new();
    for candidate in points {
        if budget_exhausted(
            polish_start_ms,
            extra_ms_budget,
            *extra_evals,
            max_extra_evals,
        ) {
            break;
        }
        if !seen.insert(portfolio_key(
            candidate.0,
            candidate.1,
            candidate.2,
            candidate.3,
        )) {
            continue;
        }
        let eval = evaluate_with_miss_tracking(
            candidate.0,
            candidate.1,
            candidate.2,
            candidate.3,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
            extra_evals,
        )?;
        if is_feasible(eval.clean_match, target, tolerance) {
            seeds.push(eval);
        }
    }

    if seeds.is_empty() {
        seeds.push(start_eval.clone());
    }

    seeds.sort_by(|a, b| compare_eval_candidates(a, b, target));
    let capped = top_k.max(1);
    if seeds.len() > capped {
        seeds.truncate(capped);
    }
    Ok(seeds)
}

#[allow(clippy::too_many_arguments)]
fn polish_seed_local(
    target: f64,
    seed: &EvalResult,
    stage_schedule: &[(f64, f64, f64)],
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    cache: &mut EvalCache,
    accurate_cfg: &V2AccurateConfig,
    improved_dimensions: &mut ImprovedDimensions,
    accepted_moves: &mut u32,
    rejected_feasible_moves: &mut u32,
    extra_evals: &mut u32,
    polish_start_ms: f64,
    extra_ms_budget: f64,
    max_extra_evals: u32,
) -> Result<(EvalResult, bool, bool), String> {
    let mut local_best = seed.clone();
    let banked_best = seed.clone();
    let mut any_stage_improved = false;
    let mut first_stage_improved = false;

    // Band-edge recentering (F22 round 3): accurate-mode solutions ride
    // clean_match = target - tolerance to buy LCOE, which leaves no slack for
    // approximately-iso-match exchange moves — any tiny match loss goes
    // infeasible and the exchange walk never starts. Step slightly off the edge
    // first; `banked_best` guarantees this can never worsen the returned result.
    let lower_edge = target - accurate_cfg.target_tolerance;
    let slack = local_best.clean_match - lower_edge;
    if slack < 0.15
        && !budget_exhausted(
            polish_start_ms,
            extra_ms_budget,
            *extra_evals,
            max_extra_evals,
        )
    {
        let need = 0.25 - slack;
        let recenter_probes: [(f64, f64, f64, f64, bool); 4] = [
            (0.0, 0.0, 0.0, 2.0, config.enable_clean_firm),
            (10.0, 0.0, 0.0, 0.0, config.enable_solar),
            (0.0, 10.0, 0.0, 0.0, config.enable_wind),
            (0.0, 0.0, 20.0, 0.0, config.enable_storage),
        ];
        'recenter: for (ds, dw, dst, dcf, res_enabled) in recenter_probes {
            if !res_enabled {
                continue;
            }
            let probe = clamp_candidate(
                local_best.solar + ds,
                local_best.wind + dw,
                local_best.storage + dst,
                local_best.clean_firm + dcf,
                config,
            );
            if same_point(&local_best, probe) {
                continue;
            }
            let probe_eval = evaluate_with_miss_tracking(
                probe.0,
                probe.1,
                probe.2,
                probe.3,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
                extra_evals,
            )?;
            let gained = probe_eval.clean_match - local_best.clean_match;
            if gained <= 1e-6 {
                continue;
            }
            // Scale the probe delta to gain roughly `need` match points.
            let scale = (need / gained).clamp(0.25, 4.0);
            let stepped = clamp_candidate(
                local_best.solar + ds * scale,
                local_best.wind + dw * scale,
                local_best.storage + dst * scale,
                local_best.clean_firm + dcf * scale,
                config,
            );
            let stepped_eval = if same_point(&local_best, stepped) {
                probe_eval.clone()
            } else {
                evaluate_with_miss_tracking(
                    stepped.0,
                    stepped.1,
                    stepped.2,
                    stepped.3,
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    config,
                    cache,
                    battery_mode,
                    extra_evals,
                )?
            };
            for cand in [stepped_eval, probe_eval] {
                if is_feasible(cand.clean_match, target, accurate_cfg.target_tolerance)
                    && cand.clean_match > local_best.clean_match + 1e-6
                {
                    local_best = cand;
                    break 'recenter;
                }
            }
        }
    }

    for (stage_idx, stage) in stage_schedule.iter().enumerate() {
        if budget_exhausted(
            polish_start_ms,
            extra_ms_budget,
            *extra_evals,
            max_extra_evals,
        ) {
            break;
        }
        let sw_step = stage.0;
        let st_step = stage.1;
        let cf_step = stage.2;

        let mut improved_in_stage = false;
        if config.enable_clean_firm && cf_step > 0.0 {
            let cf_improved = cf_local_scan_if_non_monotonic(
                target,
                accurate_cfg.target_tolerance,
                accurate_cfg.lcoe_improve_min,
                cf_step,
                &mut local_best,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                battery_mode,
                cache,
                accepted_moves,
                rejected_feasible_moves,
                improved_dimensions,
                extra_evals,
                polish_start_ms,
                extra_ms_budget,
                max_extra_evals,
            )?;
            improved_in_stage |= cf_improved;
            any_stage_improved |= cf_improved;
        }

        let mut moves = generate_stage_moves(sw_step, st_step, cf_step, config);
        // Point-adaptive iso-match exchange moves from locally measured slopes
        // (F22 round 3): fixes zone-ratio drift that strands hard-coded oblique
        // templates outside the feasibility band.
        if !budget_exhausted(
            polish_start_ms,
            extra_ms_budget,
            *extra_evals,
            max_extra_evals,
        ) {
            let slopes = measure_match_sensitivities(
                &local_best,
                sw_step,
                st_step,
                cf_step,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                battery_mode,
                cache,
                extra_evals,
            )?;
            moves.extend(generate_adaptive_exchange_moves(
                slopes, sw_step, st_step, cf_step, config,
            ));
        }
        // Repeat the sweep until no move improves: long walks along an exchange
        // direction (e.g. storage -> CF at terminal targets) need many consecutive
        // steps, and a single sweep advances at most one step per direction
        // (optimizer_research F22: the "distant exchange" failure class).
        let mut stage_rounds = 0u32;
        'rounds: loop {
            let mut improved_this_round = false;
            for &(ds, dw, dst, dcf) in &moves {
                if budget_exhausted(
                    polish_start_ms,
                    extra_ms_budget,
                    *extra_evals,
                    max_extra_evals,
                ) {
                    break 'rounds;
                }

                let candidate = clamp_candidate(
                    local_best.solar + ds,
                    local_best.wind + dw,
                    local_best.storage + dst,
                    local_best.clean_firm + dcf,
                    config,
                );
                if same_point(&local_best, candidate) {
                    continue;
                }

                let eval = evaluate_with_miss_tracking(
                    candidate.0,
                    candidate.1,
                    candidate.2,
                    candidate.3,
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    config,
                    cache,
                    battery_mode,
                    extra_evals,
                )?;

                if is_feasible(eval.clean_match, target, accurate_cfg.target_tolerance)
                    && local_best.lcoe - eval.lcoe >= accurate_cfg.lcoe_improve_min
                {
                    let previous = local_best.clone();
                    local_best = eval;
                    improved_dimensions.mark_delta(&previous, &local_best);
                    *accepted_moves = accepted_moves.saturating_add(1);
                    improved_in_stage = true;
                    any_stage_improved = true;
                    improved_this_round = true;

                    // Accelerate: keep stepping along the same direction while it
                    // keeps improving (cheap directional line search).
                    loop {
                        if budget_exhausted(
                            polish_start_ms,
                            extra_ms_budget,
                            *extra_evals,
                            max_extra_evals,
                        ) {
                            break;
                        }
                        let next = clamp_candidate(
                            local_best.solar + ds,
                            local_best.wind + dw,
                            local_best.storage + dst,
                            local_best.clean_firm + dcf,
                            config,
                        );
                        if same_point(&local_best, next) {
                            break;
                        }
                        let next_eval = evaluate_with_miss_tracking(
                            next.0,
                            next.1,
                            next.2,
                            next.3,
                            solar_profile,
                            wind_profile,
                            load_profile,
                            costs,
                            config,
                            cache,
                            battery_mode,
                            extra_evals,
                        )?;
                        if is_feasible(
                            next_eval.clean_match,
                            target,
                            accurate_cfg.target_tolerance,
                        ) && local_best.lcoe - next_eval.lcoe
                            >= accurate_cfg.lcoe_improve_min
                        {
                            let prev = local_best.clone();
                            local_best = next_eval;
                            improved_dimensions.mark_delta(&prev, &local_best);
                            *accepted_moves = accepted_moves.saturating_add(1);
                        } else {
                            break;
                        }
                    }
                } else if is_feasible(eval.clean_match, target, accurate_cfg.target_tolerance) {
                    *rejected_feasible_moves = rejected_feasible_moves.saturating_add(1);
                }
            }
            stage_rounds += 1;
            if !improved_this_round || stage_rounds >= 25 {
                break;
            }
        }

        if stage_idx == 0 && improved_in_stage {
            first_stage_improved = true;
        }
        if !improved_in_stage {
            break;
        }
    }

    // Recentering safety net: never return worse than the seed we started from.
    if banked_best.lcoe + 1e-9 < local_best.lcoe {
        local_best = banked_best;
    }

    Ok((local_best, any_stage_improved, first_stage_improved))
}

#[allow(clippy::too_many_arguments)]
fn run_v2_accurate_polish(
    target: f64,
    fast_result: &OptimizerResult,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    cache: &mut EvalCache,
    accurate_cfg: &V2AccurateConfig,
    extra_ms_budget: f64,
) -> Result<(OptimizerResult, V2AccurateDiagnostics), String> {
    let precision = V2Config::default().precision;
    let polish_start_ms = now_ms();
    let mut extra_evals = 0u32;

    let start_eval = evaluate_with_miss_tracking(
        fast_result.solar_capacity,
        fast_result.wind_capacity,
        fast_result.storage_capacity,
        fast_result.clean_firm_capacity,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
        &mut extra_evals,
    )?;

    let mut best = start_eval.clone();
    let mut stop_reason = V2AccurateStopReason::CompletedSchedule;
    let mut improved_dimensions = ImprovedDimensions::default();
    let mut accepted_moves = 0u32;
    let mut rejected_feasible_moves = 0u32;
    let mut any_improvement = false;

    let mut max_extra_evals = accurate_cfg.max_extra_evals.max(1);
    if target >= accurate_cfg.hard_target_threshold {
        max_extra_evals = max_extra_evals.max(accurate_cfg.adaptive_hard_max_extra_evals);
    }
    let stage_schedule = build_stage_schedule(&accurate_cfg.step_schedule);
    // multi_start_top_k <= 1 means "no multi-start": descend from the start
    // point directly instead of spending the eval budget probing seed
    // candidates around it (the bounded fast-mode mini-polish relies on this —
    // with an 80-eval cap, breadth probing would consume the entire budget).
    let seeds = if accurate_cfg.multi_start_top_k <= 1 {
        vec![start_eval.clone()]
    } else {
        select_multistart_seeds(
            target,
            accurate_cfg.target_tolerance,
            accurate_cfg.multi_start_top_k,
            &start_eval,
            &stage_schedule,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            cache,
            &mut extra_evals,
            polish_start_ms,
            extra_ms_budget,
            max_extra_evals,
        )?
    };

    for (seed_idx, seed) in seeds.iter().enumerate() {
        if let Some(reason) = budget_stop_reason(
            polish_start_ms,
            extra_ms_budget,
            extra_evals,
            max_extra_evals,
        ) {
            stop_reason = reason;
            break;
        }

        let (local_best, local_improved, first_stage_improved) = polish_seed_local(
            target,
            seed,
            &stage_schedule,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            cache,
            accurate_cfg,
            &mut improved_dimensions,
            &mut accepted_moves,
            &mut rejected_feasible_moves,
            &mut extra_evals,
            polish_start_ms,
            extra_ms_budget,
            max_extra_evals,
        )?;

        if seed_idx == 0
            && !first_stage_improved
            && max_extra_evals < accurate_cfg.adaptive_hard_max_extra_evals
        {
            max_extra_evals = accurate_cfg.adaptive_hard_max_extra_evals;
        }

        if local_improved {
            any_improvement = true;
        }
        if better_eval_candidate(&local_best, &best, target) {
            best = local_best;
        }
    }

    if stop_reason == V2AccurateStopReason::CompletedSchedule {
        if let Some(reason) = budget_stop_reason(
            polish_start_ms,
            extra_ms_budget,
            extra_evals,
            max_extra_evals,
        ) {
            stop_reason = reason;
        } else if !any_improvement && !better_eval_candidate(&best, &start_eval, target) {
            stop_reason = V2AccurateStopReason::NoFurtherImprovement;
        }
    }

    let total_evals = fast_result.num_evaluations.saturating_add(extra_evals);
    Ok((
        OptimizerResult {
            solar_capacity: best.solar,
            wind_capacity: best.wind,
            storage_capacity: best.storage,
            clean_firm_capacity: best.clean_firm,
            achieved_clean_match: best.clean_match,
            lcoe: best.lcoe,
            num_evaluations: total_evals,
            success: (best.clean_match - target).abs() < precision,
        },
        V2AccurateDiagnostics {
            seed_count: seeds.len(),
            start_lcoe: start_eval.lcoe,
            end_lcoe: best.lcoe,
            accepted_moves,
            rejected_feasible_moves,
            extra_evals,
            effective_max_extra_evals: max_extra_evals,
            stop_reason,
            improved_dimensions: improved_dimensions.as_names(),
        },
    ))
}

fn evaluate_with_miss_tracking(
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    cache: &mut EvalCache,
    battery_mode: BatteryMode,
    extra_evals: &mut u32,
) -> Result<EvalResult, String> {
    let (_, misses_before, _) = cache.stats();
    let result = evaluate_cached(
        solar,
        wind,
        storage,
        clean_firm,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;
    let (_, misses_after, _) = cache.stats();
    *extra_evals = extra_evals.saturating_add(misses_after.saturating_sub(misses_before));
    Ok(result)
}

fn budget_exhausted(
    stage_start_ms: f64,
    max_extra_ms: f64,
    extra_evals: u32,
    max_extra_evals: u32,
) -> bool {
    budget_stop_reason(stage_start_ms, max_extra_ms, extra_evals, max_extra_evals).is_some()
}

fn budget_stop_reason(
    stage_start_ms: f64,
    max_extra_ms: f64,
    extra_evals: u32,
    max_extra_evals: u32,
) -> Option<V2AccurateStopReason> {
    if extra_evals >= max_extra_evals {
        return Some(V2AccurateStopReason::EvalBudgetExhausted);
    }
    if max_extra_ms > 0.0 && now_ms() - stage_start_ms >= max_extra_ms {
        return Some(V2AccurateStopReason::TimeBudgetExhausted);
    }
    None
}

fn compare_f64(a: f64, b: f64) -> Ordering {
    if (a - b).abs() <= 1e-9 {
        Ordering::Equal
    } else {
        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
    }
}

fn is_feasible(clean_match: f64, target: f64, tolerance: f64) -> bool {
    (clean_match - target).abs() <= tolerance
}

fn same_point(current: &EvalResult, candidate: (f64, f64, f64, f64)) -> bool {
    (current.solar - candidate.0).abs() < 1e-9
        && (current.wind - candidate.1).abs() < 1e-9
        && (current.storage - candidate.2).abs() < 1e-9
        && (current.clean_firm - candidate.3).abs() < 1e-9
}

fn clamp_candidate(
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
    config: &OptimizerConfig,
) -> (f64, f64, f64, f64) {
    let s = if config.enable_solar {
        solar.clamp(0.0, config.max_solar)
    } else {
        0.0
    };
    let w = if config.enable_wind {
        wind.clamp(0.0, config.max_wind)
    } else {
        0.0
    };
    let st = if config.enable_storage {
        storage.clamp(0.0, config.max_storage)
    } else {
        0.0
    };
    let cf = if config.enable_clean_firm {
        clean_firm.clamp(0.0, config.max_clean_firm)
    } else {
        0.0
    };
    (s, w, st, cf)
}

fn generate_stage_moves(
    sw_step: f64,
    st_step: f64,
    cf_step: f64,
    config: &OptimizerConfig,
) -> Vec<(f64, f64, f64, f64)> {
    let mut moves = Vec::new();
    let signs = [-1.0, 1.0];

    if config.enable_solar && sw_step > 0.0 {
        moves.push((-sw_step, 0.0, 0.0, 0.0));
        moves.push((sw_step, 0.0, 0.0, 0.0));
    }
    if config.enable_wind && sw_step > 0.0 {
        moves.push((0.0, -sw_step, 0.0, 0.0));
        moves.push((0.0, sw_step, 0.0, 0.0));
    }
    if config.enable_storage && st_step > 0.0 {
        moves.push((0.0, 0.0, -st_step, 0.0));
        moves.push((0.0, 0.0, st_step, 0.0));
    }
    if config.enable_clean_firm && cf_step > 0.0 {
        moves.push((0.0, 0.0, 0.0, -cf_step));
        moves.push((0.0, 0.0, 0.0, cf_step));
    }

    if config.enable_solar && config.enable_wind && sw_step > 0.0 {
        for &ss in &signs {
            for &ww in &signs {
                moves.push((ss * sw_step, ww * sw_step, 0.0, 0.0));
            }
        }
    }
    if config.enable_solar && config.enable_storage && sw_step > 0.0 && st_step > 0.0 {
        for &ss in &signs {
            for &st in &signs {
                moves.push((ss * sw_step, 0.0, st * st_step, 0.0));
            }
        }
    }
    if config.enable_wind && config.enable_storage && sw_step > 0.0 && st_step > 0.0 {
        for &ww in &signs {
            for &st in &signs {
                moves.push((0.0, ww * sw_step, st * st_step, 0.0));
            }
        }
    }
    if config.enable_solar && config.enable_clean_firm && sw_step > 0.0 && cf_step > 0.0 {
        for &ss in &signs {
            for &cc in &signs {
                moves.push((ss * sw_step, 0.0, 0.0, cc * cf_step));
            }
        }
    }
    if config.enable_wind && config.enable_clean_firm && sw_step > 0.0 && cf_step > 0.0 {
        for &ww in &signs {
            for &cc in &signs {
                moves.push((0.0, ww * sw_step, 0.0, cc * cf_step));
            }
        }
    }
    if config.enable_storage && config.enable_clean_firm && st_step > 0.0 && cf_step > 0.0 {
        for &st in &signs {
            for &cc in &signs {
                moves.push((0.0, 0.0, st * st_step, cc * cf_step));
            }
        }
    }

    // Oblique exchange-direction moves (optimizer_research/LANDSCAPE_FINDINGS.md
    // F12/F19/F21). The near-optimal set is a flat plane of resource exchanges;
    // per-axis and uniform-pair moves change clean_match too much to survive the
    // feasibility gate, so descent stalls on that plane. These templates are scaled
    // to be roughly iso-clean-match.
    if config.enable_clean_firm && cf_step > 0.0 {
        for &sign in &signs {
            let c = sign * cf_step;
            // ~1 MW CF <-> ~3.7 MW wind (+ ~1.7 MWh storage when available)
            if config.enable_wind {
                moves.push((0.0, 3.7 * c, 0.0, -c));
                if config.enable_storage {
                    moves.push((0.0, 3.7 * c, 1.7 * c, -c));
                }
            }
            // ~1 MW CF <-> solar + storage
            if config.enable_solar && config.enable_storage {
                moves.push((1.4 * c, 0.0, 1.7 * c, -c));
            }
        }
    }
    if config.enable_solar && config.enable_wind && sw_step > 0.0 {
        // Iso-match solar<->wind exchange (~1.27 MW solar per MW wind)
        for &sign in &signs {
            let s = sign * sw_step;
            moves.push((1.27 * s, -s, 0.0, 0.0));
        }
    }
    if config.enable_storage && config.enable_clean_firm && st_step > 0.0 {
        // Fine storage<->CF ratios: 1 MW CF moves clean_match ~10x more than
        // 2 MWh storage, so escaping storage/CF stall ridges needs small CF
        // deltas per storage step.
        for &sign in &signs {
            let st = sign * st_step;
            moves.push((0.0, 0.0, st, -0.05 * st));
            moves.push((0.0, 0.0, st, -0.1 * st));
        }
    }

    moves
}

/// Per-unit clean-match slopes (per MW solar/wind/CF, per MWh storage), measured
/// at `base` by one-step finite differences. Cache-friendly: the probe points
/// coincide with axis moves the stage sweep evaluates anyway. Disabled resources
/// (or zero steps) report slope 0.
#[allow(clippy::too_many_arguments)]
fn measure_match_sensitivities(
    base: &EvalResult,
    sw_step: f64,
    st_step: f64,
    cf_step: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    cache: &mut EvalCache,
    extra_evals: &mut u32,
) -> Result<[f64; 4], String> {
    let mut slopes = [0.0f64; 4];
    let axes: [(f64, f64, f64, f64, bool, f64); 4] = [
        (sw_step, 0.0, 0.0, 0.0, config.enable_solar, sw_step),
        (0.0, sw_step, 0.0, 0.0, config.enable_wind, sw_step),
        (0.0, 0.0, st_step, 0.0, config.enable_storage, st_step),
        (0.0, 0.0, 0.0, cf_step, config.enable_clean_firm, cf_step),
    ];
    for (i, (ds, dw, dst, dcf, enabled, step)) in axes.into_iter().enumerate() {
        if !enabled || step <= 0.0 {
            continue;
        }
        // Probe upward; at an upper bound fall back to a downward probe.
        let mut sign = 1.0;
        let mut cand = clamp_candidate(
            base.solar + ds,
            base.wind + dw,
            base.storage + dst,
            base.clean_firm + dcf,
            config,
        );
        if same_point(base, cand) {
            sign = -1.0;
            cand = clamp_candidate(
                base.solar - ds,
                base.wind - dw,
                base.storage - dst,
                base.clean_firm - dcf,
                config,
            );
            if same_point(base, cand) {
                continue;
            }
        }
        let eval = evaluate_with_miss_tracking(
            cand.0,
            cand.1,
            cand.2,
            cand.3,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
            extra_evals,
        )?;
        slopes[i] = sign * (eval.clean_match - base.clean_match) / step;
    }
    Ok(slopes)
}

/// Point-adaptive iso-clean-match exchange moves: give one step of resource A,
/// take the match back with resource B at the LOCALLY measured ratio. The
/// hard-coded oblique ratios above are CA-calibrated averages; in other zones
/// (or far from those calibration points) they drift out of the feasibility
/// band and the exchange walk never starts (F22 round 3).
fn generate_adaptive_exchange_moves(
    slopes: [f64; 4],
    sw_step: f64,
    st_step: f64,
    cf_step: f64,
    config: &OptimizerConfig,
) -> Vec<(f64, f64, f64, f64)> {
    const MIN_SLOPE: f64 = 1e-5;
    let enabled = [
        config.enable_solar,
        config.enable_wind,
        config.enable_storage,
        config.enable_clean_firm,
    ];
    let steps = [sw_step, sw_step, st_step, cf_step];
    let mut moves = Vec::new();
    for give in 0..4 {
        if !enabled[give] || steps[give] <= 0.0 || slopes[give] <= MIN_SLOPE {
            continue;
        }
        for take in 0..4 {
            if take == give || !enabled[take] || steps[take] <= 0.0 || slopes[take] <= MIN_SLOPE
            {
                continue;
            }
            let comp = slopes[give] * steps[give] / slopes[take];
            if comp > 8.0 * steps[take] {
                continue;
            }
            let mut delta = [0.0f64; 4];
            delta[give] = -steps[give];
            delta[take] = comp;
            moves.push((delta[0], delta[1], delta[2], delta[3]));
        }
    }
    moves
}

#[allow(clippy::too_many_arguments)]
fn cf_local_scan_if_non_monotonic(
    target: f64,
    tolerance: f64,
    lcoe_improve_min: f64,
    cf_step: f64,
    best: &mut EvalResult,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    cache: &mut EvalCache,
    accepted_moves: &mut u32,
    rejected_feasible_moves: &mut u32,
    improved_dimensions: &mut ImprovedDimensions,
    extra_evals: &mut u32,
    stage_start_ms: f64,
    max_extra_ms: f64,
    max_extra_evals: u32,
) -> Result<bool, String> {
    if cf_step <= 0.0 || !config.enable_clean_firm {
        return Ok(false);
    }

    let cf_low = (best.clean_firm - cf_step).max(0.0);
    let cf_mid = best.clean_firm.clamp(0.0, config.max_clean_firm);
    let cf_high = (best.clean_firm + cf_step).min(config.max_clean_firm);

    if (cf_low - cf_mid).abs() < 1e-9 || (cf_mid - cf_high).abs() < 1e-9 {
        return Ok(false);
    }

    let low = evaluate_with_miss_tracking(
        best.solar,
        best.wind,
        best.storage,
        cf_low,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
        extra_evals,
    )?;
    let mid = evaluate_with_miss_tracking(
        best.solar,
        best.wind,
        best.storage,
        cf_mid,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
        extra_evals,
    )?;
    let high = evaluate_with_miss_tracking(
        best.solar,
        best.wind,
        best.storage,
        cf_high,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
        extra_evals,
    )?;

    let monotonic =
        mid.clean_match + 1e-9 >= low.clean_match && high.clean_match + 1e-9 >= mid.clean_match;
    if monotonic {
        return Ok(false);
    }

    let mut improved = false;
    let mut scan_cf = (best.clean_firm - 3.0 * cf_step).max(0.0);
    let scan_max = (best.clean_firm + 3.0 * cf_step).min(config.max_clean_firm);
    while scan_cf <= scan_max + 1e-9 {
        if budget_exhausted(stage_start_ms, max_extra_ms, *extra_evals, max_extra_evals) {
            break;
        }
        let eval = evaluate_with_miss_tracking(
            best.solar,
            best.wind,
            best.storage,
            scan_cf,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
            extra_evals,
        )?;
        if is_feasible(eval.clean_match, target, tolerance)
            && best.lcoe - eval.lcoe >= lcoe_improve_min
        {
            let previous = best.clone();
            *best = eval;
            improved_dimensions.mark_delta(&previous, best);
            *accepted_moves = accepted_moves.saturating_add(1);
            improved = true;
        } else if is_feasible(eval.clean_match, target, tolerance) {
            *rejected_feasible_moves = rejected_feasible_moves.saturating_add(1);
        }
        scan_cf += cf_step;
    }

    Ok(improved)
}

/// Model-based optimization: uses empirical model to find best candidates quickly,
/// then applies greedy refinement for quality.
///
/// Strategy: Model for coarse search + Greedy for fine-tuning
/// - Model quickly identifies promising region of search space
/// - Diversified candidate selection ensures different resource mixes are explored
/// - Greedy refinement ensures we don't miss nearby better solutions
/// - Result: ~3x speedup with minimal quality loss
fn run_model_based_optimization(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: &EmpiricalModel,
    v2_config: &V2Config,
    cache: &mut EvalCache,
    total_evals: &mut u32,
) -> Result<EvalResult, String> {
    // Step 1: Use model to find candidates with estimated CF to hit target
    let candidates = find_ranked_candidates(model, target, costs, config);

    if candidates.is_empty() {
        // Fall back to greedy if no candidates found
        return run_greedy_based_optimization(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            v2_config,
            cache,
            total_evals,
        );
    }

    // Step 2: Select candidates for evaluation
    // With finer grids, we need to evaluate more candidates to ensure good coverage
    let num_candidates = if target >= 97.0 { 30 } else { 50 };

    let diverse_candidates = select_diverse_candidates(&candidates, num_candidates);

    let mut results: Vec<EvalResult> = Vec::new();

    for (portfolio, estimated_cf) in diverse_candidates.iter() {
        let (cf, result) = binary_search_cf_narrow(
            portfolio.solar,
            portfolio.wind,
            portfolio.storage,
            target,
            *estimated_cf,
            v2_config.fine_tolerance,
            if config.enable_clean_firm {
                config.max_clean_firm
            } else {
                0.0
            },
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
            8,
        )?;
        *total_evals += cache.last_eval_count();

        if result.is_valid(target, v2_config.coarse_tolerance) {
            results.push(EvalResult {
                solar: portfolio.solar,
                wind: portfolio.wind,
                storage: portfolio.storage,
                clean_firm: cf,
                lcoe: result.lcoe,
                clean_match: result.clean_match,
            });
        }
    }

    if results.is_empty() {
        return run_greedy_based_optimization(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            battery_mode,
            v2_config,
            cache,
            total_evals,
        );
    }

    // Sort by LCOE, take best as starting point for refinement
    results.sort_by(|a, b| {
        a.lcoe
            .partial_cmp(&b.lcoe)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let model_best = results[0].clone();

    // Step 3: Greedy refinement starting from model's best candidate
    // This is the key improvement: use greedy to fine-tune the model's result
    let refined = run_greedy_refinement_from_start(
        target,
        &model_best,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
        v2_config,
        cache,
        total_evals,
    )?;

    // Return the better of model's best or greedy-refined result
    if refined.lcoe < model_best.lcoe {
        Ok(refined)
    } else {
        Ok(model_best)
    }
}

/// Run lightweight greedy refinement starting from an initial portfolio
/// Uses multiple passes with decreasing step sizes for better coverage
fn run_greedy_refinement_from_start(
    target: f64,
    start: &EvalResult,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    v2_config: &V2Config,
    cache: &mut EvalCache,
    total_evals: &mut u32,
) -> Result<EvalResult, String> {
    let mut best = start.clone();

    // Two-pass refinement: first with larger steps, then finer steps
    // This helps find solutions off the model's grid
    let step_sizes: [&[f64]; 2] = [
        &[-75.0, -50.0, -25.0, 25.0, 50.0, 75.0], // First pass: wider search
        &[-15.0, -10.0, 10.0, 15.0],              // Second pass: fine-tune
    ];

    for deltas in step_sizes {
        // Try solar adjustments
        if config.enable_solar {
            for &ds in deltas {
                let s = (best.solar + ds).clamp(0.0, config.max_solar);
                if s == best.solar {
                    continue;
                }

                let (cf, result) = binary_search_cf(
                    s,
                    best.wind,
                    best.storage,
                    target,
                    v2_config.fine_tolerance,
                    if config.enable_clean_firm {
                        config.max_clean_firm
                    } else {
                        0.0
                    },
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    config,
                    cache,
                    battery_mode,
                    6,
                )?;
                *total_evals += cache.last_eval_count();

                if result.is_valid(target, v2_config.fine_tolerance)
                    && result.lcoe < best.lcoe - 0.05
                {
                    best = EvalResult {
                        solar: s,
                        wind: best.wind,
                        storage: best.storage,
                        clean_firm: cf,
                        lcoe: result.lcoe,
                        clean_match: result.clean_match,
                    };
                }
            }
        }

        // Try wind adjustments
        if config.enable_wind {
            for &dw in deltas {
                let w = (best.wind + dw).clamp(0.0, config.max_wind);
                if w == best.wind {
                    continue;
                }

                let (cf, result) = binary_search_cf(
                    best.solar,
                    w,
                    best.storage,
                    target,
                    v2_config.fine_tolerance,
                    if config.enable_clean_firm {
                        config.max_clean_firm
                    } else {
                        0.0
                    },
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    config,
                    cache,
                    battery_mode,
                    6,
                )?;
                *total_evals += cache.last_eval_count();

                if result.is_valid(target, v2_config.fine_tolerance)
                    && result.lcoe < best.lcoe - 0.05
                {
                    best = EvalResult {
                        solar: best.solar,
                        wind: w,
                        storage: best.storage,
                        clean_firm: cf,
                        lcoe: result.lcoe,
                        clean_match: result.clean_match,
                    };
                }
            }
        }

        // Try storage adjustments
        if config.enable_storage {
            for &dst in deltas {
                let st = (best.storage + dst).clamp(0.0, config.max_storage);
                if st == best.storage {
                    continue;
                }

                let (cf, result) = binary_search_cf(
                    best.solar,
                    best.wind,
                    st,
                    target,
                    v2_config.fine_tolerance,
                    if config.enable_clean_firm {
                        config.max_clean_firm
                    } else {
                        0.0
                    },
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    config,
                    cache,
                    battery_mode,
                    6,
                )?;
                *total_evals += cache.last_eval_count();

                if result.is_valid(target, v2_config.fine_tolerance)
                    && result.lcoe < best.lcoe - 0.05
                {
                    best = EvalResult {
                        solar: best.solar,
                        wind: best.wind,
                        storage: st,
                        clean_firm: cf,
                        lcoe: result.lcoe,
                        clean_match: result.clean_match,
                    };
                }
            }
        }
    } // Close outer step_sizes loop

    Ok(best)
}

/// Select diverse candidates to ensure we explore different parts of the solution space
///
/// The key insight is that candidates with similar estimated LCOE may all be from
/// the same "region" of the solution space (e.g., all solar-heavy). This can cause
/// us to miss better solutions in other regions.
///
/// This function selects candidates ensuring diversity across:
/// - Resource composition (solar-heavy, wind-heavy, balanced, CF-heavy)
/// - Storage levels (low, medium, high)
fn select_diverse_candidates(
    candidates: &[(Portfolio, f64)],
    max_count: usize,
) -> Vec<(Portfolio, f64)> {
    if candidates.len() <= max_count {
        return candidates.to_vec();
    }

    let mut selected: Vec<(Portfolio, f64)> = Vec::with_capacity(max_count);
    let mut selected_keys: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Helper to categorize a portfolio
    let categorize = |p: &Portfolio| -> String {
        let total_renewable = p.solar + p.wind;
        let solar_ratio = if total_renewable > 0.0 {
            p.solar / total_renewable
        } else {
            0.5
        };
        let storage_category = if p.storage < 400.0 {
            "low_st"
        } else if p.storage < 1200.0 {
            "med_st"
        } else {
            "high_st"
        };
        let cf_category = if p.cf < 25.0 {
            "low_cf"
        } else if p.cf < 75.0 {
            "med_cf"
        } else {
            "high_cf"
        };
        let solar_category = if solar_ratio < 0.3 {
            "wind_heavy"
        } else if solar_ratio > 0.7 {
            "solar_heavy"
        } else {
            "balanced"
        };

        format!("{}_{}_{}", solar_category, storage_category, cf_category)
    };

    // Pass 1: Take top 10 by estimated LCOE (best candidates)
    for (portfolio, cf) in candidates.iter().take(10) {
        let key = categorize(portfolio);
        selected.push((portfolio.clone(), *cf));
        selected_keys.insert(key);
    }

    // Pass 2: Add candidates from underrepresented categories
    for (portfolio, cf) in candidates.iter().skip(10) {
        if selected.len() >= max_count {
            break;
        }

        let key = categorize(portfolio);
        if !selected_keys.contains(&key) {
            selected.push((portfolio.clone(), *cf));
            selected_keys.insert(key);
        }
    }

    // Pass 3: If still need more, fill with remaining top candidates
    for (portfolio, cf) in candidates.iter() {
        if selected.len() >= max_count {
            break;
        }

        // Check if this exact portfolio is already selected
        let already_selected = selected.iter().any(|(p, _)| {
            (p.solar - portfolio.solar).abs() < 0.1
                && (p.wind - portfolio.wind).abs() < 0.1
                && (p.storage - portfolio.storage).abs() < 0.1
        });

        if !already_selected {
            selected.push((portfolio.clone(), *cf));
        }
    }

    selected
}

/// Find candidates using model predictions, ranked by estimated total LCOE
///
/// Key insight: The model now stores BOTH clean_match AND peak_gas capacity.
/// This allows us to estimate the full system LCOE including:
/// - Clean resource costs (solar, wind, storage, clean firm)
/// - Gas capacity costs (from model.predict_gas())
///
/// This is a major improvement over pure CAPEX ranking because gas capacity
/// varies significantly between portfolios with the same clean_match %.
fn find_ranked_candidates(
    model: &EmpiricalModel,
    target: f64,
    costs: &CostParams,
    config: &OptimizerConfig,
) -> Vec<(Portfolio, f64)> {
    let mut candidates: Vec<(Portfolio, f64, f64)> = Vec::new(); // (portfolio, estimated_cf, estimated_lcoe)

    // Annualized cost per unit (simplified CRF ≈ 1/lifetime)
    let solar_annual_cost =
        costs.solar_capex * 1000.0 / costs.solar_lifetime as f64 + costs.solar_fixed_om * 1000.0;
    let wind_annual_cost =
        costs.wind_capex * 1000.0 / costs.wind_lifetime as f64 + costs.wind_fixed_om * 1000.0;
    let storage_annual_cost = costs.storage_capex * 1000.0 / costs.storage_lifetime as f64
        + costs.storage_fixed_om * 1000.0;
    let cf_annual_cost = costs.clean_firm_capex * 1000.0 / costs.clean_firm_lifetime as f64
        + costs.clean_firm_fixed_om * 1000.0;

    // Gas capacity cost (important: this is what differentiates portfolios with same clean_match!)
    let gas_annual_cost = costs.gas_capex * 1000.0 / 30.0  // Assume 30-year gas plant life
        + costs.gas_fixed_om * 1000.0;

    // Check if model has gas data (for backward compatibility)
    let has_gas_data = model.has_gas_data();

    let solar_range: Vec<f64> = if config.enable_solar {
        (0..=((config.max_solar.min(model.config.solar_max) / model.config.solar_step) as usize))
            .map(|i| model.config.solar_min + i as f64 * model.config.solar_step)
            .collect()
    } else {
        vec![0.0]
    };

    let wind_range: Vec<f64> = if config.enable_wind {
        (0..=((config.max_wind.min(model.config.wind_max) / model.config.wind_step) as usize))
            .map(|i| model.config.wind_min + i as f64 * model.config.wind_step)
            .collect()
    } else {
        vec![0.0]
    };

    let storage_range: Vec<f64> = if config.enable_storage {
        (0..=((config.max_storage.min(model.config.storage_max) / model.config.storage_step)
            as usize))
            .map(|i| model.config.storage_min + i as f64 * model.config.storage_step)
            .collect()
    } else {
        vec![0.0]
    };

    let max_cf = if config.enable_clean_firm {
        config.max_clean_firm
    } else {
        0.0
    };

    for solar in &solar_range {
        for wind in &wind_range {
            for storage in &storage_range {
                // Predict clean match at CF=0
                let base_match = model.predict(*solar, *wind, *storage, 0.0);

                // Skip if already overshoots target significantly
                if base_match > target + 3.0 {
                    continue;
                }

                // If we already hit target without CF, use CF=0
                let estimated_cf = if base_match >= target - 0.5 {
                    0.0
                } else if !config.enable_clean_firm {
                    continue; // Can't hit target without CF
                } else {
                    // Binary search in model to find CF that hits target
                    let max_match = model.predict(*solar, *wind, *storage, max_cf);
                    if max_match < target - 1.0 {
                        continue; // Can't hit target even with max CF
                    }
                    model_binary_search_cf(model, *solar, *wind, *storage, target, max_cf)
                };

                // === FULL LCOE ESTIMATION ===
                //
                // With the model storing clean_match, peak_gas, AND gas_generation,
                // we can now calculate a nearly-exact LCOE estimate:
                //
                // LCOE = (Clean CAPEX + Gas CAPEX + Gas Fuel + O&M) / Energy Delivered
                //
                // This is the key insight: gas fuel cost is the main differentiator
                // between portfolios with similar clean_match %.

                // 1. Clean resource capital + fixed O&M (annualized).
                // CF is firm-thermal so its capex/fixed-O&M scale by the
                // planning-reserve factor — same multiplier applied in
                // calculate_lcoe so the filter ranking matches the final
                // LCOE that re-evaluates top candidates.
                let reserve_factor = 1.0 + costs.reserve_margin / 100.0;
                let clean_cost = *solar * solar_annual_cost
                    + *wind * wind_annual_cost
                    + *storage * storage_annual_cost
                    + estimated_cf * cf_annual_cost * reserve_factor;

                // 2. Gas capacity cost (CAPEX + fixed O&M, annualized).
                // Also reserve-scaled — gas peakers carry the same buffer.
                let gas_capacity = if has_gas_data {
                    model.predict_gas(*solar, *wind, *storage, estimated_cf)
                } else {
                    let current_match = model.predict(*solar, *wind, *storage, estimated_cf);
                    100.0 * (1.0 - current_match / 100.0)
                };
                let gas_capex_cost = gas_capacity * gas_annual_cost * reserve_factor;

                // 3. Gas FUEL cost (this is the big differentiator!)
                // Gas fuel cost = gas_generation (MWh) * heat_rate (MMBtu/MWh) * gas_price ($/MMBtu)
                let gas_generation = if model.has_gas_gen_data() {
                    model.predict_gas_gen(*solar, *wind, *storage, estimated_cf)
                } else {
                    // Fallback: estimate from clean_match
                    let current_match = model.predict(*solar, *wind, *storage, estimated_cf);
                    876_000.0 * (1.0 - current_match / 100.0)
                };
                let gas_fuel_cost = gas_generation * costs.gas_heat_rate * costs.gas_price;

                // 4. Gas variable O&M
                let gas_var_om_cost = gas_generation * costs.gas_var_om;

                // Total system cost (annual)
                let total_cost = clean_cost + gas_capex_cost + gas_fuel_cost + gas_var_om_cost;

                // Estimated LCOE = Total Cost / Energy Delivered
                // Use assumed annual load of 876,000 MWh (100 MW × 8760 hours)
                const ASSUMED_ANNUAL_LOAD: f64 = 876_000.0;
                let estimated_lcoe = total_cost / ASSUMED_ANNUAL_LOAD;

                candidates.push((
                    Portfolio {
                        solar: *solar,
                        wind: *wind,
                        storage: *storage,
                        cf: estimated_cf,
                    },
                    estimated_cf,
                    estimated_lcoe,
                ));
            }
        }
    }

    // Sort by estimated LCOE (ascending) - lower is better
    candidates.sort_by(|a, b| a.2.partial_cmp(&b.2).unwrap_or(std::cmp::Ordering::Equal));

    // Return top candidates with their estimated CF
    candidates
        .into_iter()
        .take(50) // Keep more for diversity, but we only evaluate top 20
        .map(|(p, cf, _)| (p, cf))
        .collect()
}

/// Binary search in model to find CF that achieves target
fn model_binary_search_cf(
    model: &EmpiricalModel,
    solar: f64,
    wind: f64,
    storage: f64,
    target: f64,
    max_cf: f64,
) -> f64 {
    let mut low = 0.0;
    let mut high = max_cf;

    for _ in 0..10 {
        let mid = (low + high) / 2.0;
        let predicted = model.predict(solar, wind, storage, mid);

        if predicted < target {
            low = mid;
        } else {
            high = mid;
        }

        if high - low < 1.0 {
            break;
        }
    }

    (low + high) / 2.0
}

/// Narrow binary search starting near estimated CF
fn binary_search_cf_narrow(
    solar: f64,
    wind: f64,
    storage: f64,
    target: f64,
    estimated_cf: f64,
    tolerance: f64,
    max_cf: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    cache: &mut EvalCache,
    battery_mode: BatteryMode,
    max_iters: usize,
) -> Result<(f64, EvalResult), String> {
    // Start with a narrow search window around estimated CF
    let search_radius = 50.0; // ±50 MW window
    let mut low = (estimated_cf - search_radius).max(0.0);
    let mut high = (estimated_cf + search_radius).min(max_cf);

    let mut best_cf = estimated_cf;
    let mut best_result = evaluate_cached(
        solar,
        wind,
        storage,
        estimated_cf,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;
    let mut eval_count = 1u32;

    // Check if estimate is already good enough
    if (best_result.clean_match - target).abs() < tolerance {
        cache.set_last_eval_count(eval_count);
        return Ok((best_cf, best_result));
    }

    // Expand search if estimate was way off
    if best_result.clean_match < target - 5.0 {
        high = max_cf;
    } else if best_result.clean_match > target + 5.0 {
        low = 0.0;
    }

    for _ in 0..max_iters {
        let mid = (low + high) / 2.0;

        let result = evaluate_cached(
            solar,
            wind,
            storage,
            mid,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
        )?;
        eval_count += 1;

        // Track closest to target
        if (result.clean_match - target).abs() < (best_result.clean_match - target).abs() {
            best_cf = mid;
            best_result = result.clone();
        }

        // Adjust search bounds
        if result.clean_match < target {
            low = mid;
        } else {
            high = mid;
        }

        // Check convergence
        if (result.clean_match - target).abs() < tolerance / 2.0 {
            break;
        }

        // Check if bounds have converged
        if high - low < 0.1 {
            break;
        }
    }

    cache.set_last_eval_count(eval_count);
    Ok((best_cf, best_result))
}

/// Greedy-based optimization: builds portfolio incrementally
fn run_greedy_based_optimization(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    v2_config: &V2Config,
    cache: &mut EvalCache,
    total_evals: &mut u32,
) -> Result<EvalResult, String> {
    // Phase 1: Greedy expansion
    let greedy_result = run_greedy_phase(
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
        v2_config,
        cache,
    )?;
    *total_evals += cache.last_eval_count();
    let mut best = greedy_result;

    // Phase 2: Local refinement
    let seed = best.clone();
    for &ds in &[-30.0, -15.0, 0.0, 15.0, 30.0] {
        for &dw in &[-30.0, -15.0, 0.0, 15.0, 30.0] {
            for &dst in &[-100.0, -50.0, 0.0, 50.0, 100.0] {
                let s = (seed.solar + ds).clamp(0.0, config.max_solar);
                let w = (seed.wind + dw).clamp(0.0, config.max_wind);
                let st = (seed.storage + dst).clamp(0.0, config.max_storage);

                if !config.enable_solar && s > 0.0 {
                    continue;
                }
                if !config.enable_wind && w > 0.0 {
                    continue;
                }
                if !config.enable_storage && st > 0.0 {
                    continue;
                }

                let (cf, result) = binary_search_cf(
                    s,
                    w,
                    st,
                    target,
                    v2_config.fine_tolerance,
                    if config.enable_clean_firm {
                        config.max_clean_firm
                    } else {
                        0.0
                    },
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    config,
                    cache,
                    battery_mode,
                    12,
                )?;
                *total_evals += cache.last_eval_count();

                if result.is_valid(target, v2_config.fine_tolerance) && result.lcoe < best.lcoe {
                    best = EvalResult {
                        solar: s,
                        wind: w,
                        storage: st,
                        clean_firm: cf,
                        lcoe: result.lcoe,
                        clean_match: result.clean_match,
                    };
                }
            }
        }
    }

    Ok(best)
}

fn evaluate_frontier_candidate(
    current: &EvalResult,
    target: f64,
    search_tolerance: f64,
    accept_tolerance: f64,
    max_cf: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    cache: &mut EvalCache,
    battery_mode: BatteryMode,
    enable_clean_firm: bool,
    eval_count: &mut u32,
    best_valid: &mut Option<EvalResult>,
    best_valid_lcoe: &mut f64,
    best_any: &mut EvalResult,
    best_any_deviation: &mut f64,
    no_improve_steps: &mut u32,
) -> Result<Option<f64>, String> {
    let mut candidate = current.clone();
    let mut cf_efficiency = None;

    if enable_clean_firm && max_cf > 0.0 && current.clean_match < target - search_tolerance {
        let (cf, result) = binary_search_cf(
            current.solar,
            current.wind,
            current.storage,
            target,
            search_tolerance,
            max_cf,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
            12,
        )?;
        *eval_count += cache.last_eval_count();

        candidate = EvalResult {
            solar: current.solar,
            wind: current.wind,
            storage: current.storage,
            clean_firm: cf,
            lcoe: result.lcoe,
            clean_match: result.clean_match,
        };

        let match_delta = candidate.clean_match - current.clean_match;
        if match_delta > 0.0 {
            cf_efficiency = Some((candidate.lcoe - current.lcoe) / match_delta);
        }
    }

    if candidate.is_valid(target, accept_tolerance) {
        if candidate.lcoe < *best_valid_lcoe {
            *best_valid_lcoe = candidate.lcoe;
            *best_valid = Some(candidate.clone());
            *no_improve_steps = 0;
        } else if best_valid.is_some() {
            *no_improve_steps += 1;
        }
    } else if best_valid.is_none() {
        let deviation = (candidate.clean_match - target).abs();
        if deviation < *best_any_deviation {
            *best_any_deviation = deviation;
            *best_any = candidate;
        }
    }

    Ok(cf_efficiency)
}

/// Greedy frontier sweep - expands renewables while checking CF fill.
fn run_greedy_phase(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    v2_config: &V2Config,
    cache: &mut EvalCache,
) -> Result<EvalResult, String> {
    let mut solar = 0.0;
    let mut wind = 0.0;
    let mut storage = 0.0;
    let mut eval_count = 0u32;

    // Start with larger steps, refine as we approach target
    let mut step_size = 50.0;
    let min_step = 10.0;
    let min_match_gain = 0.1;
    // F9 fix: minimum LCOE decrease ($/MWh) for a "free improvement" move —
    // a move that lowers LCOE without losing clean match (e.g. storage trimming
    // peak-gas capacity cost). Such moves have ~0 match gain and are invisible
    // to the efficiency = dLCOE/dMatch ranking (see LANDSCAPE_FINDINGS F9).
    let free_move_min_lcoe_drop = 0.01;
    let max_no_improve_steps = 3u32;
    let max_cf_guard_steps = 2u32;
    let cf_guard_multiplier = 1.05;

    // Get initial state
    let mut current = evaluate_cached(
        solar,
        wind,
        storage,
        0.0,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
    )?;
    eval_count += 1;

    let max_cf = if config.enable_clean_firm {
        config.max_clean_firm
    } else {
        0.0
    };
    let search_tolerance = v2_config.fine_tolerance;
    let accept_tolerance = v2_config.coarse_tolerance;
    let mut best_valid: Option<EvalResult> = None;
    let mut best_valid_lcoe = f64::INFINITY;
    let mut best_any = current.clone();
    let mut best_any_deviation = (current.clean_match - target).abs();
    let mut no_improve_steps = 0u32;
    let mut cf_guard_steps = 0u32;

    let mut cf_efficiency = evaluate_frontier_candidate(
        &current,
        target,
        search_tolerance,
        accept_tolerance,
        max_cf,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        cache,
        battery_mode,
        config.enable_clean_firm,
        &mut eval_count,
        &mut best_valid,
        &mut best_valid_lcoe,
        &mut best_any,
        &mut best_any_deviation,
        &mut no_improve_steps,
    )?;

    for _ in 0..100 {
        let mut best_move: Option<(f64, f64, f64, EvalResult)> = None;
        let mut best_efficiency = f64::INFINITY;
        // F9 fix: track "free improvement" candidates (LCOE down, match not down)
        // separately — they are accepted ahead of the efficiency ranking.
        let mut best_free: Option<(f64, f64, f64, EvalResult)> = None;
        let mut best_free_drop = 0.0f64;
        // F9 fix: once the trajectory is inside the target band, moves that exit
        // it upward are never accepted — overshoot states can never become valid
        // frontier candidates (CF fill only adds match), and marching past the
        // band preempts the in-band free-improvement trims (e.g. storage moves
        // that cut peak-gas capacity cost at ~zero match gain).
        let in_band = current.clean_match >= target - accept_tolerance;

        // Try adding solar
        if config.enable_solar && solar + step_size <= config.max_solar {
            let test_solar = solar + step_size;
            if let Ok(result) = evaluate_cached(
                test_solar,
                wind,
                storage,
                0.0,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
            ) {
                eval_count += 1;
                let match_gain = result.clean_match - current.clean_match;
                let lcoe_drop = current.lcoe - result.lcoe;
                let band_ok = !in_band || result.clean_match <= target + accept_tolerance;
                // In-band, coarse match-gaining moves just climb the band at rising
                // LCOE and trigger the no-improve break before the step shrinks far
                // enough to expose fine free-improvement trims; only accept them at
                // min-step granularity so the step schedule shrinks first.
                let eff_ok = band_ok && (!in_band || step_size <= min_step);
                if band_ok
                    && lcoe_drop >= free_move_min_lcoe_drop
                    && match_gain >= -1e-9
                    && lcoe_drop > best_free_drop
                {
                    best_free_drop = lcoe_drop;
                    best_free = Some((test_solar, wind, storage, result.clone()));
                }
                if eff_ok && match_gain > min_match_gain {
                    let efficiency = (result.lcoe - current.lcoe) / match_gain;
                    if efficiency < best_efficiency {
                        best_efficiency = efficiency;
                        best_move = Some((test_solar, wind, storage, result));
                    }
                }
            }
        }

        // Try adding wind
        if config.enable_wind && wind + step_size <= config.max_wind {
            let test_wind = wind + step_size;
            if let Ok(result) = evaluate_cached(
                solar,
                test_wind,
                storage,
                0.0,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
            ) {
                eval_count += 1;
                let match_gain = result.clean_match - current.clean_match;
                let lcoe_drop = current.lcoe - result.lcoe;
                let band_ok = !in_band || result.clean_match <= target + accept_tolerance;
                // In-band, coarse match-gaining moves just climb the band at rising
                // LCOE and trigger the no-improve break before the step shrinks far
                // enough to expose fine free-improvement trims; only accept them at
                // min-step granularity so the step schedule shrinks first.
                let eff_ok = band_ok && (!in_band || step_size <= min_step);
                if band_ok
                    && lcoe_drop >= free_move_min_lcoe_drop
                    && match_gain >= -1e-9
                    && lcoe_drop > best_free_drop
                {
                    best_free_drop = lcoe_drop;
                    best_free = Some((solar, test_wind, storage, result.clone()));
                }
                if eff_ok && match_gain > min_match_gain {
                    let efficiency = (result.lcoe - current.lcoe) / match_gain;
                    if efficiency < best_efficiency {
                        best_efficiency = efficiency;
                        best_move = Some((solar, test_wind, storage, result));
                    }
                }
            }
        }

        // Try adding storage
        if config.enable_storage && storage + step_size * 2.0 <= config.max_storage {
            let test_storage = storage + step_size * 2.0;
            if let Ok(result) = evaluate_cached(
                solar,
                wind,
                test_storage,
                0.0,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                config,
                cache,
                battery_mode,
            ) {
                eval_count += 1;
                let match_gain = result.clean_match - current.clean_match;
                let lcoe_drop = current.lcoe - result.lcoe;
                let band_ok = !in_band || result.clean_match <= target + accept_tolerance;
                // In-band, coarse match-gaining moves just climb the band at rising
                // LCOE and trigger the no-improve break before the step shrinks far
                // enough to expose fine free-improvement trims; only accept them at
                // min-step granularity so the step schedule shrinks first.
                let eff_ok = band_ok && (!in_band || step_size <= min_step);
                if band_ok
                    && lcoe_drop >= free_move_min_lcoe_drop
                    && match_gain >= -1e-9
                    && lcoe_drop > best_free_drop
                {
                    best_free_drop = lcoe_drop;
                    best_free = Some((solar, wind, test_storage, result.clone()));
                }
                if eff_ok && match_gain > min_match_gain {
                    let efficiency = (result.lcoe - current.lcoe) / match_gain;
                    if efficiency < best_efficiency {
                        best_efficiency = efficiency;
                        best_move = Some((solar, wind, test_storage, result));
                    }
                }
            }
        }

        // F9 fix: a "free improvement" (LCOE strictly down, clean match not down)
        // is accepted immediately, ahead of the efficiency ranking. It stops being
        // taken as soon as the LCOE decrease falls below the threshold, so
        // termination stays bounded by the iteration cap and step shrinking.
        let took_free = best_free.is_some();
        let Some((s, w, st, result)) = best_free.or(best_move) else {
            // No improving move at this step size, reduce it
            step_size = (step_size / 2.0).max(min_step);
            if step_size <= min_step {
                break;
            }
            continue;
        };

        if !took_free {
            if let Some(cf_eff) = cf_efficiency {
                if cf_eff > 0.0 && best_efficiency > cf_eff * cf_guard_multiplier {
                    cf_guard_steps += 1;
                    if cf_guard_steps >= max_cf_guard_steps {
                        break;
                    }
                } else {
                    cf_guard_steps = 0;
                }
            }
        }

        solar = s;
        wind = w;
        storage = st;
        current = result;

        cf_efficiency = evaluate_frontier_candidate(
            &current,
            target,
            search_tolerance,
            accept_tolerance,
            max_cf,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            config,
            cache,
            battery_mode,
            config.enable_clean_firm,
            &mut eval_count,
            &mut best_valid,
            &mut best_valid_lcoe,
            &mut best_any,
            &mut best_any_deviation,
            &mut no_improve_steps,
        )?;

        if current.clean_match >= target - accept_tolerance
            && best_valid.is_some()
            && no_improve_steps >= max_no_improve_steps
        {
            break;
        }
    }

    cache.set_last_eval_count(eval_count);

    Ok(best_valid.unwrap_or(best_any))
}

/// Generate candidate grid without empirical model
/// Uses COARSE grid (50-100 MW steps) to keep evaluations manageable
fn generate_candidate_grid(config: &OptimizerConfig) -> Vec<Portfolio> {
    let mut candidates = Vec::new();

    // Use coarse steps for initial search: ~5-7 points per dimension
    // This gives ~125-350 candidates instead of 11,000
    let solar_step = (config.max_solar / 5.0).max(50.0);
    let wind_step = (config.max_wind / 5.0).max(50.0);
    let storage_step = (config.max_storage / 5.0).max(200.0);

    let solar_range: Vec<f64> = if config.enable_solar {
        (0..=((config.max_solar / solar_step) as usize))
            .map(|i| i as f64 * solar_step)
            .collect()
    } else {
        vec![0.0]
    };

    let wind_range: Vec<f64> = if config.enable_wind {
        (0..=((config.max_wind / wind_step) as usize))
            .map(|i| i as f64 * wind_step)
            .collect()
    } else {
        vec![0.0]
    };

    let storage_range: Vec<f64> = if config.enable_storage {
        (0..=((config.max_storage / storage_step) as usize))
            .map(|i| i as f64 * storage_step)
            .collect()
    } else {
        vec![0.0]
    };

    for solar in &solar_range {
        for wind in &wind_range {
            for storage in &storage_range {
                candidates.push(Portfolio {
                    solar: *solar,
                    wind: *wind,
                    storage: *storage,
                    cf: 0.0,
                });
            }
        }
    }

    candidates
}

/// Fallback when no candidates meet target
fn run_v2_fallback(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    cache: &mut EvalCache,
    total_evals: &mut u32,
) -> Result<OptimizerResult, String> {
    // Try a dense grid search for low targets
    let mut best: Option<EvalResult> = None;
    let mut best_portfolio = Portfolio {
        solar: 0.0,
        wind: 0.0,
        storage: 0.0,
        cf: 0.0,
    };

    let solar_steps = if config.enable_solar { 5 } else { 1 };
    let wind_steps = if config.enable_wind { 5 } else { 1 };
    let storage_steps = if config.enable_storage { 3 } else { 1 };
    let cf_steps = if config.enable_clean_firm { 5 } else { 1 };

    for si in 0..solar_steps {
        let solar = if config.enable_solar {
            config.max_solar * si as f64 / (solar_steps - 1) as f64
        } else {
            0.0
        };

        for wi in 0..wind_steps {
            let wind = if config.enable_wind {
                config.max_wind * wi as f64 / (wind_steps - 1) as f64
            } else {
                0.0
            };

            for sti in 0..storage_steps {
                let storage = if config.enable_storage {
                    config.max_storage * sti as f64 / (storage_steps - 1) as f64
                } else {
                    0.0
                };

                for cfi in 0..cf_steps {
                    let cf = if config.enable_clean_firm {
                        config.max_clean_firm * cfi as f64 / (cf_steps - 1) as f64
                    } else {
                        0.0
                    };

                    let result = evaluate_cached(
                        solar,
                        wind,
                        storage,
                        cf,
                        solar_profile,
                        wind_profile,
                        load_profile,
                        costs,
                        config,
                        cache,
                        battery_mode,
                    )?;
                    *total_evals += 1;

                    // Find best that's closest to target
                    let deviation = (result.clean_match - target).abs();
                    let current_best_deviation = best
                        .as_ref()
                        .map(|b| (b.clean_match - target).abs())
                        .unwrap_or(f64::INFINITY);

                    if deviation < current_best_deviation {
                        best = Some(result);
                        best_portfolio = Portfolio {
                            solar,
                            wind,
                            storage,
                            cf,
                        };
                    }
                }
            }
        }
    }

    match best {
        Some(result) => Ok(OptimizerResult {
            solar_capacity: best_portfolio.solar,
            wind_capacity: best_portfolio.wind,
            storage_capacity: best_portfolio.storage,
            clean_firm_capacity: best_portfolio.cf,
            achieved_clean_match: result.clean_match,
            lcoe: result.lcoe,
            num_evaluations: *total_evals,
            success: (result.clean_match - target).abs() < 0.5,
        }),
        None => Err("No valid portfolio found in fallback".to_string()),
    }
}

/// Run optimizer sweep across multiple targets
pub fn run_v2_sweep(
    targets: &[f64],
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: Option<&EmpiricalModel>,
) -> Result<Vec<OptimizerResult>, String> {
    // Production default is Accurate; warm-start continuation keeps full-ladder
    // sweeps fast (~1.8s per 21-target zone sweep) at the certified quality bar.
    run_v2_sweep_mode(
        targets,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        config,
        battery_mode,
        model,
        V2Mode::Accurate,
        None,
    )
}

/// Run optimizer sweep across multiple targets with explicit runtime mode.
pub fn run_v2_sweep_mode(
    targets: &[f64],
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    model: Option<&EmpiricalModel>,
    mode: V2Mode,
    accurate_config: Option<&V2AccurateConfig>,
) -> Result<Vec<OptimizerResult>, String> {
    let mut results = Vec::with_capacity(targets.len());
    let mut sweep_cache = EvalCache::new();

    for &target in targets {
        let mut target_config = config.clone();
        target_config.target_clean_match = target;

        // Warm-start continuation: carry the previous target's portfolio forward
        // (F12: the optimal path never folds across targets). Only feasible
        // previous results are usable seeds; infeasible ones fall back to the
        // full per-target pipeline inside run_v2_optimizer_mode_with_cache.
        let warm_start = results
            .last()
            .filter(|prev: &&OptimizerResult| prev.lcoe.is_finite() && prev.lcoe > 0.0);

        let (result, _) = run_v2_optimizer_mode_with_cache(
            target,
            solar_profile,
            wind_profile,
            load_profile,
            costs,
            &target_config,
            battery_mode,
            model,
            mode,
            accurate_config,
            &mut sweep_cache,
            warm_start,
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
    fn test_v2_optimizer_basic() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let result = run_v2_optimizer(
            50.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        )
        .unwrap();

        assert!(result.num_evaluations > 0);
        assert!(result.lcoe > 0.0);
        // Should achieve close to 50%
        assert!(
            (result.achieved_clean_match - 50.0).abs() < 1.0,
            "Expected ~50%, got {}",
            result.achieved_clean_match
        );
    }

    #[test]
    fn test_v2_optimizer_low_target() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let result = run_v2_optimizer(
            10.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        )
        .unwrap();

        // Low target should primarily use renewables, minimal CF
        assert!(result.achieved_clean_match <= 15.0);
    }

    #[test]
    fn test_v2_optimizer_high_target() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();

        let result = run_v2_optimizer(
            90.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        )
        .unwrap();

        // High target should achieve close to 90% with some combination of resources
        assert!(
            result.achieved_clean_match >= 85.0,
            "Expected ~90%, got {}",
            result.achieved_clean_match
        );

        // Should use significant resources to achieve high target
        let total_capacity = result.solar_capacity
            + result.wind_capacity
            + result.clean_firm_capacity
            + result.storage_capacity;
        assert!(
            total_capacity > 100.0,
            "High target should require significant capacity, got {}",
            total_capacity
        );
    }

    #[test]
    fn test_v2_optimizer_disabled_resources() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let mut config = OptimizerConfig::default();
        config.enable_wind = false;
        config.enable_clean_firm = false;

        let result = run_v2_optimizer(
            40.0,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        )
        .unwrap();

        // Disabled resources should be 0
        assert_eq!(result.wind_capacity, 0.0);
        assert_eq!(result.clean_firm_capacity, 0.0);
    }

    #[test]
    fn test_binary_search_cf() {
        let (solar, wind, load) = create_test_profiles();
        let costs = CostParams::default_costs();
        let config = OptimizerConfig::default();
        let mut cache = EvalCache::new();

        let (cf, result) = binary_search_cf(
            100.0,
            100.0,
            50.0,
            70.0,  // target
            0.5,   // tolerance
            200.0, // max cf
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            &mut cache,
            BatteryMode::Hybrid,
            20,
        )
        .unwrap();

        // Should find a CF that gets close to 70%
        assert!((result.clean_match - 70.0).abs() < 2.0);
        assert!(cf >= 0.0 && cf <= 200.0);
    }

    #[test]
    fn test_generate_candidate_grid() {
        let config = OptimizerConfig::default();
        let candidates = generate_candidate_grid(&config);

        // Should have multiple candidates
        assert!(!candidates.is_empty());

        // All should have valid bounds
        for c in &candidates {
            assert!(c.solar >= 0.0 && c.solar <= config.max_solar);
            assert!(c.wind >= 0.0 && c.wind <= config.max_wind);
            assert!(c.storage >= 0.0 && c.storage <= config.max_storage);
        }
    }
}
