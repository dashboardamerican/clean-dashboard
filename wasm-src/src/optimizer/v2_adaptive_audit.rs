use super::v2_hierarchical::{run_v2_optimizer, run_v2_optimizer_mode_detailed, V2Mode};
use crate::economics::calculate_lcoe;
use crate::simulation::simulate_system;
use crate::types::{BatteryMode, CostParams, OptimizerConfig, SimulationConfig, HOURS_PER_YEAR};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const ZONES_PATH: &str = "../data/zones.json";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveOracleConfig {
    pub max_solar: f64,
    pub max_wind: f64,
    pub max_storage: f64,
    pub max_clean_firm: f64,
    pub coarse_solar_step: f64,
    pub coarse_wind_step: f64,
    pub coarse_storage_step: f64,
    pub coarse_cf_step: f64,
    pub fine_solar_step: f64,
    pub fine_wind_step: f64,
    pub fine_storage_step: f64,
    pub fine_cf_step: f64,
}

impl Default for AdaptiveOracleConfig {
    fn default() -> Self {
        Self {
            max_solar: 120.0,
            max_wind: 400.0,
            max_storage: 160.0,
            max_clean_firm: 120.0,
            coarse_solar_step: 20.0,
            coarse_wind_step: 20.0,
            coarse_storage_step: 20.0,
            coarse_cf_step: 10.0,
            fine_solar_step: 10.0,
            fine_wind_step: 10.0,
            fine_storage_step: 10.0,
            fine_cf_step: 5.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V2AdaptiveAuditConfig {
    pub trial_budget: usize,
    pub batch_size: usize,
    pub max_generations: usize,
    pub zones: Vec<String>,
    pub targets: Vec<f64>,
    pub target_tolerance: f64,
    pub min_multiplier: f64,
    pub max_multiplier: f64,
    pub random_exploration_per_gen: usize,
    pub coarse_oracle_top_k: usize,
    pub fine_oracle_top_k: usize,
    pub runtime_ratio_limit: f64,
    pub fine_gap_limit_pct: f64,
    pub seed: u64,
    pub battery_mode: BatteryMode,
    pub oracle: AdaptiveOracleConfig,
}

impl Default for V2AdaptiveAuditConfig {
    fn default() -> Self {
        Self {
            trial_budget: 120,
            batch_size: 12,
            max_generations: 16,
            zones: vec!["california".to_string(), "texas".to_string()],
            targets: vec![70.0, 85.0, 95.0, 99.0],
            target_tolerance: 0.5,
            min_multiplier: 0.02,
            max_multiplier: 40.0,
            random_exploration_per_gen: 4,
            coarse_oracle_top_k: 3,
            fine_oracle_top_k: 2,
            runtime_ratio_limit: 2.0,
            fine_gap_limit_pct: 1.0,
            seed: 20260206,
            battery_mode: BatteryMode::Hybrid,
            oracle: AdaptiveOracleConfig::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LeverMultipliers {
    pub solar_capex: f64,
    pub wind_capex: f64,
    pub storage_capex: f64,
    pub clean_firm_capex: f64,
    pub gas_capex: f64,
    pub gas_price: f64,
    pub clean_firm_fuel: f64,
    pub gas_var_om: f64,
}

impl Default for LeverMultipliers {
    fn default() -> Self {
        Self {
            solar_capex: 1.0,
            wind_capex: 1.0,
            storage_capex: 1.0,
            clean_firm_capex: 1.0,
            gas_capex: 1.0,
            gas_price: 1.0,
            clean_firm_fuel: 1.0,
            gas_var_om: 1.0,
        }
    }
}

impl LeverMultipliers {
    fn apply(&self, baseline: &CostParams) -> CostParams {
        let mut costs = baseline.clone();
        costs.solar_capex *= self.solar_capex;
        costs.wind_capex *= self.wind_capex;
        costs.storage_capex *= self.storage_capex;
        costs.clean_firm_capex *= self.clean_firm_capex;
        costs.gas_capex *= self.gas_capex;
        costs.gas_price *= self.gas_price;
        costs.clean_firm_fuel *= self.clean_firm_fuel;
        costs.gas_var_om *= self.gas_var_om;
        clamp_costs(&mut costs);
        costs
    }

    fn values(&self) -> [f64; 8] {
        [
            self.solar_capex,
            self.wind_capex,
            self.storage_capex,
            self.clean_firm_capex,
            self.gas_capex,
            self.gas_price,
            self.clean_firm_fuel,
            self.gas_var_om,
        ]
    }

    fn from_values(values: [f64; 8]) -> Self {
        Self {
            solar_capex: values[0],
            wind_capex: values[1],
            storage_capex: values[2],
            clean_firm_capex: values[3],
            gas_capex: values[4],
            gas_price: values[5],
            clean_firm_fuel: values[6],
            gas_var_om: values[7],
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveTrialReport {
    pub trial_id: u64,
    pub generation: usize,
    pub parent_trial_id: Option<u64>,
    pub zone: String,
    pub target: f64,
    pub multipliers: LeverMultipliers,
    pub runtime_fast_ms: Option<f64>,
    pub runtime_accurate_ms: Option<f64>,
    pub runtime_ratio_accurate_vs_fast: Option<f64>,
    pub fast_lcoe: Option<f64>,
    pub accurate_lcoe: Option<f64>,
    pub selected_lcoe: Option<f64>,
    pub fast_match: Option<f64>,
    pub accurate_match: Option<f64>,
    pub selected_deviation: Option<f64>,
    pub fast_vs_accurate_gap_pct: Option<f64>,
    pub coarse_oracle_lcoe: Option<f64>,
    pub fine_oracle_lcoe: Option<f64>,
    pub selected_gap_vs_coarse_pct: Option<f64>,
    pub selected_gap_vs_fine_pct: Option<f64>,
    pub accurate_start_lcoe: Option<f64>,
    pub accurate_end_lcoe: Option<f64>,
    pub accurate_lcoe_improvement: Option<f64>,
    pub accurate_accepted_moves: Option<u32>,
    pub accurate_rejected_feasible_moves: Option<u32>,
    pub accurate_extra_evals: Option<u32>,
    pub accurate_max_extra_evals: Option<u32>,
    pub accurate_seed_count: Option<usize>,
    pub accurate_stop_reason: Option<String>,
    pub accurate_improved_dimensions: Option<Vec<String>>,
    pub weakness_score: f64,
    pub flags: Vec<String>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AdaptiveFixRecommendation {
    pub key: String,
    pub rationale: String,
    pub suggested_action: String,
    pub evidence_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V2AdaptiveAuditSummary {
    pub trial_count: usize,
    pub successful_trials: usize,
    pub coarse_oracle_trials: usize,
    pub fine_oracle_trials: usize,
    pub median_runtime_ratio_accurate_vs_fast: Option<f64>,
    pub median_selected_gap_vs_fine_pct: Option<f64>,
    pub p95_selected_gap_vs_fine_pct: Option<f64>,
    pub worst_selected_gap_vs_fine_pct: Option<f64>,
    pub highest_weakness_score: Option<f64>,
    pub highest_weakness_trial_id: Option<u64>,
    pub runtime_ratio_limit: f64,
    pub fine_gap_limit_pct: f64,
    pub runtime_pass: bool,
    pub gap_pass: bool,
    pub pass: bool,
    pub lever_coverage_min: LeverMultipliers,
    pub lever_coverage_max: LeverMultipliers,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V2AdaptiveAuditReport {
    pub generated_unix_ms: u128,
    pub config: V2AdaptiveAuditConfig,
    pub trials: Vec<AdaptiveTrialReport>,
    pub summary: V2AdaptiveAuditSummary,
    pub recommendations: Vec<AdaptiveFixRecommendation>,
}

#[derive(Clone)]
struct Profiles {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

#[derive(Clone)]
struct Scenario {
    zone: String,
    target: f64,
    tolerance: f64,
    optimizer_config: OptimizerConfig,
    baseline_costs: CostParams,
    profiles: Profiles,
}

#[derive(Debug, Deserialize)]
struct ZoneProfiles {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

#[derive(Clone, Debug)]
struct OracleResult {
    lcoe: f64,
    clean_match: f64,
    evaluations: u64,
}

#[derive(Clone, Copy, Debug)]
struct OracleSteps {
    solar_step: f64,
    wind_step: f64,
    storage_step: f64,
    cf_step: f64,
}

#[derive(Clone)]
struct TrialCandidate {
    scenario_idx: usize,
    multipliers: LeverMultipliers,
    generation: usize,
    parent_trial_id: Option<u64>,
}

#[derive(Clone, Copy)]
enum OracleLevel {
    Coarse,
    Fine,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
struct OracleCacheKey {
    scenario_idx: usize,
    level: u8,
    solar_capex_x1000: i32,
    wind_capex_x1000: i32,
    storage_capex_x1000: i32,
    clean_firm_capex_x1000: i32,
    gas_capex_x1000: i32,
    gas_price_x1000: i32,
    clean_firm_fuel_x1000: i32,
    gas_var_om_x1000: i32,
}

#[derive(Clone)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        let state = if seed == 0 { 0x9E3779B97F4A7C15 } else { seed };
        Self { state }
    }

    fn next_u64(&mut self) -> u64 {
        // xorshift64*
        let mut x = self.state;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.state = x;
        x.wrapping_mul(0x2545F4914F6CDD1D)
    }

    fn next_f64(&mut self) -> f64 {
        let value = self.next_u64() >> 11;
        (value as f64) / ((1u64 << 53) as f64)
    }

    fn choose_index(&mut self, len: usize) -> usize {
        if len <= 1 {
            return 0;
        }
        (self.next_u64() as usize) % len
    }
}

pub fn run_v2_adaptive_audit(
    config: &V2AdaptiveAuditConfig,
) -> Result<V2AdaptiveAuditReport, String> {
    validate_config(config)?;

    let scenarios = build_scenarios(config)?;
    if scenarios.is_empty() {
        return Err("No scenarios were built for adaptive audit".to_string());
    }

    let mut rng = DeterministicRng::new(config.seed);
    let mut queue: VecDeque<TrialCandidate> =
        VecDeque::from(make_initial_candidates(config, scenarios.len(), &mut rng));
    let mut seen: HashSet<String> = HashSet::new();
    let mut trials: Vec<AdaptiveTrialReport> = Vec::new();
    let mut next_trial_id = 1u64;
    let mut oracle_cache: HashMap<OracleCacheKey, OracleResult> = HashMap::new();
    let mut best_score_seen = f64::NEG_INFINITY;
    let mut stagnant_generations = 0usize;

    for generation in 0..config.max_generations {
        if trials.len() >= config.trial_budget {
            break;
        }

        let mut batch = Vec::new();
        while batch.len() < config.batch_size && trials.len() + batch.len() < config.trial_budget {
            let candidate = if let Some(candidate) = queue.pop_front() {
                candidate
            } else {
                random_candidate(config, scenarios.len(), generation, None, &mut rng)
            };
            let key = candidate_key(&candidate);
            if seen.insert(key) {
                batch.push(candidate);
            }
        }

        if batch.is_empty() {
            break;
        }

        let start_idx = trials.len();
        for candidate in batch {
            let scenario = &scenarios[candidate.scenario_idx];
            let report = evaluate_candidate(next_trial_id, &candidate, scenario, config);
            trials.push(report);
            next_trial_id = next_trial_id.saturating_add(1);
        }

        let batch_indices: Vec<usize> = (start_idx..trials.len()).collect();
        run_oracle_escalation(
            &batch_indices,
            config,
            &scenarios,
            &mut trials,
            &mut oracle_cache,
        )?;

        let current_best = batch_indices
            .iter()
            .filter_map(|&idx| trials.get(idx))
            .map(|trial| trial.weakness_score)
            .fold(f64::NEG_INFINITY, f64::max);

        if current_best > best_score_seen + 1e-9 {
            best_score_seen = current_best;
            stagnant_generations = 0;
        } else {
            stagnant_generations = stagnant_generations.saturating_add(1);
        }

        if stagnant_generations >= 3 && queue.is_empty() {
            break;
        }

        let mut next_wave =
            derive_next_candidates(config, &trials, generation, &scenarios, &mut rng);

        for _ in 0..config.random_exploration_per_gen {
            next_wave.push(random_candidate(
                config,
                scenarios.len(),
                generation + 1,
                None,
                &mut rng,
            ));
        }

        for candidate in next_wave {
            queue.push_back(candidate);
        }
    }

    let summary = summarize_trials(&trials, config);
    let recommendations = generate_recommendations(&trials);
    let generated_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Clock error: {}", e))?
        .as_millis();

    Ok(V2AdaptiveAuditReport {
        generated_unix_ms,
        config: config.clone(),
        trials,
        summary,
        recommendations,
    })
}

fn evaluate_candidate(
    trial_id: u64,
    candidate: &TrialCandidate,
    scenario: &Scenario,
    config: &V2AdaptiveAuditConfig,
) -> AdaptiveTrialReport {
    let costs = candidate.multipliers.apply(&scenario.baseline_costs);

    let mut report = AdaptiveTrialReport {
        trial_id,
        generation: candidate.generation,
        parent_trial_id: candidate.parent_trial_id,
        zone: scenario.zone.clone(),
        target: scenario.target,
        multipliers: candidate.multipliers.clone(),
        runtime_fast_ms: None,
        runtime_accurate_ms: None,
        runtime_ratio_accurate_vs_fast: None,
        fast_lcoe: None,
        accurate_lcoe: None,
        selected_lcoe: None,
        fast_match: None,
        accurate_match: None,
        selected_deviation: None,
        fast_vs_accurate_gap_pct: None,
        coarse_oracle_lcoe: None,
        fine_oracle_lcoe: None,
        selected_gap_vs_coarse_pct: None,
        selected_gap_vs_fine_pct: None,
        accurate_start_lcoe: None,
        accurate_end_lcoe: None,
        accurate_lcoe_improvement: None,
        accurate_accepted_moves: None,
        accurate_rejected_feasible_moves: None,
        accurate_extra_evals: None,
        accurate_max_extra_evals: None,
        accurate_seed_count: None,
        accurate_stop_reason: None,
        accurate_improved_dimensions: None,
        weakness_score: 0.0,
        flags: Vec::new(),
        success: true,
        error: None,
    };

    let fast_result = {
        let start = Instant::now();
        let result = run_v2_optimizer(
            scenario.target,
            &scenario.profiles.solar,
            &scenario.profiles.wind,
            &scenario.profiles.load,
            &costs,
            &scenario.optimizer_config,
            config.battery_mode,
            None,
        );
        report.runtime_fast_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
        match result {
            Ok(v) => {
                report.fast_lcoe = Some(v.lcoe);
                report.fast_match = Some(v.achieved_clean_match);
                Some(v)
            }
            Err(e) => {
                report.success = false;
                report.error = Some(format!("Fast mode failed: {}", e));
                None
            }
        }
    };

    let accurate_result = {
        let start = Instant::now();
        let result = run_v2_optimizer_mode_detailed(
            scenario.target,
            &scenario.profiles.solar,
            &scenario.profiles.wind,
            &scenario.profiles.load,
            &costs,
            &scenario.optimizer_config,
            config.battery_mode,
            None,
            V2Mode::Accurate,
            None,
        );
        report.runtime_accurate_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
        match result {
            Ok((v, diagnostics)) => {
                report.accurate_lcoe = Some(v.lcoe);
                report.accurate_match = Some(v.achieved_clean_match);
                report.selected_lcoe = Some(v.lcoe);
                if let Some(diag) = diagnostics {
                    report.accurate_start_lcoe = Some(diag.start_lcoe);
                    report.accurate_end_lcoe = Some(diag.end_lcoe);
                    report.accurate_lcoe_improvement = Some(diag.start_lcoe - diag.end_lcoe);
                    report.accurate_accepted_moves = Some(diag.accepted_moves);
                    report.accurate_rejected_feasible_moves = Some(diag.rejected_feasible_moves);
                    report.accurate_extra_evals = Some(diag.extra_evals);
                    report.accurate_max_extra_evals = Some(diag.effective_max_extra_evals);
                    report.accurate_seed_count = Some(diag.seed_count);
                    report.accurate_stop_reason = Some(diag.stop_reason.as_str().to_string());
                    report.accurate_improved_dimensions = Some(
                        diag.improved_dimensions
                            .iter()
                            .map(|s| s.to_string())
                            .collect(),
                    );
                }
                Some(v)
            }
            Err(e) => {
                report.success = false;
                report.error = Some(append_error(
                    report.error.take(),
                    format!("Accurate mode failed: {}", e),
                ));
                None
            }
        }
    };

    if let (Some(fast), Some(accurate), Some(fast_ms), Some(acc_ms)) = (
        fast_result.as_ref(),
        accurate_result.as_ref(),
        report.runtime_fast_ms,
        report.runtime_accurate_ms,
    ) {
        report.selected_lcoe = Some(accurate.lcoe);
        report.selected_deviation = Some((accurate.achieved_clean_match - scenario.target).abs());

        if fast_ms > 0.0 {
            report.runtime_ratio_accurate_vs_fast = Some(acc_ms / fast_ms);
        }

        if accurate.lcoe.abs() > 0.0 {
            report.fast_vs_accurate_gap_pct =
                Some(((fast.lcoe - accurate.lcoe).abs() / accurate.lcoe.abs()) * 100.0);
        }
    }

    recompute_weakness_score(&mut report, scenario.target, scenario.tolerance);
    report
}

fn run_oracle_escalation(
    batch_indices: &[usize],
    config: &V2AdaptiveAuditConfig,
    scenarios: &[Scenario],
    trials: &mut [AdaptiveTrialReport],
    oracle_cache: &mut HashMap<OracleCacheKey, OracleResult>,
) -> Result<(), String> {
    if batch_indices.is_empty() {
        return Ok(());
    }

    let mut ranked: Vec<usize> = batch_indices.to_vec();
    ranked.sort_by(|&a, &b| compare_f64(trials[b].weakness_score, trials[a].weakness_score));

    for &idx in ranked.iter().take(config.coarse_oracle_top_k) {
        if trials[idx].coarse_oracle_lcoe.is_some() || !trials[idx].success {
            continue;
        }
        let scenario_idx = scenario_index(scenarios, &trials[idx]);
        let scenario = &scenarios[scenario_idx];
        let costs = trials[idx].multipliers.apply(&scenario.baseline_costs);
        let oracle = run_oracle_cached(
            oracle_cache,
            scenario_idx,
            &trials[idx].multipliers,
            scenario,
            &costs,
            config.battery_mode,
            &config.oracle,
            OracleLevel::Coarse,
        )?;

        trials[idx].coarse_oracle_lcoe = Some(oracle.lcoe);
        if let Some(selected_lcoe) = trials[idx].selected_lcoe {
            if oracle.lcoe.abs() > 0.0 {
                trials[idx].selected_gap_vs_coarse_pct =
                    Some(((selected_lcoe - oracle.lcoe).abs() / oracle.lcoe.abs()) * 100.0);
            }
        }
        recompute_weakness_score(&mut trials[idx], scenario.target, scenario.tolerance);
    }

    let mut ranked_for_fine: Vec<usize> = ranked
        .into_iter()
        .filter(|&idx| trials[idx].coarse_oracle_lcoe.is_some() && trials[idx].success)
        .collect();
    ranked_for_fine.sort_by(|&a, &b| {
        let a_gap = trials[a]
            .selected_gap_vs_coarse_pct
            .unwrap_or(trials[a].weakness_score);
        let b_gap = trials[b]
            .selected_gap_vs_coarse_pct
            .unwrap_or(trials[b].weakness_score);
        compare_f64(b_gap, a_gap)
    });

    for &idx in ranked_for_fine.iter().take(config.fine_oracle_top_k) {
        if trials[idx].fine_oracle_lcoe.is_some() {
            continue;
        }
        let scenario_idx = scenario_index(scenarios, &trials[idx]);
        let scenario = &scenarios[scenario_idx];
        let costs = trials[idx].multipliers.apply(&scenario.baseline_costs);
        let oracle = run_oracle_cached(
            oracle_cache,
            scenario_idx,
            &trials[idx].multipliers,
            scenario,
            &costs,
            config.battery_mode,
            &config.oracle,
            OracleLevel::Fine,
        )?;

        trials[idx].fine_oracle_lcoe = Some(oracle.lcoe);
        if let Some(selected_lcoe) = trials[idx].selected_lcoe {
            if oracle.lcoe.abs() > 0.0 {
                trials[idx].selected_gap_vs_fine_pct =
                    Some(((selected_lcoe - oracle.lcoe).abs() / oracle.lcoe.abs()) * 100.0);
            }
        }
        recompute_weakness_score(&mut trials[idx], scenario.target, scenario.tolerance);
    }

    Ok(())
}

fn run_oracle_cached(
    oracle_cache: &mut HashMap<OracleCacheKey, OracleResult>,
    scenario_idx: usize,
    multipliers: &LeverMultipliers,
    scenario: &Scenario,
    costs: &CostParams,
    battery_mode: BatteryMode,
    oracle_cfg: &AdaptiveOracleConfig,
    level: OracleLevel,
) -> Result<OracleResult, String> {
    let key = oracle_cache_key(scenario_idx, multipliers, level);
    if let Some(cached) = oracle_cache.get(&key) {
        return Ok(cached.clone());
    }

    let steps = match level {
        OracleLevel::Coarse => OracleSteps {
            solar_step: oracle_cfg.coarse_solar_step,
            wind_step: oracle_cfg.coarse_wind_step,
            storage_step: oracle_cfg.coarse_storage_step,
            cf_step: oracle_cfg.coarse_cf_step,
        },
        OracleLevel::Fine => OracleSteps {
            solar_step: oracle_cfg.fine_solar_step,
            wind_step: oracle_cfg.fine_wind_step,
            storage_step: oracle_cfg.fine_storage_step,
            cf_step: oracle_cfg.fine_cf_step,
        },
    };

    let result = run_oracle_scan(
        scenario.target,
        scenario.tolerance,
        scenario,
        costs,
        battery_mode,
        oracle_cfg,
        steps,
    )?;

    oracle_cache.insert(key, result.clone());
    Ok(result)
}

fn run_oracle_scan(
    target: f64,
    tolerance: f64,
    scenario: &Scenario,
    costs: &CostParams,
    battery_mode: BatteryMode,
    oracle_cfg: &AdaptiveOracleConfig,
    steps: OracleSteps,
) -> Result<OracleResult, String> {
    let solar_values = build_axis(oracle_cfg.max_solar, steps.solar_step)?;
    let wind_values = build_axis(oracle_cfg.max_wind, steps.wind_step)?;
    let storage_values = build_axis(oracle_cfg.max_storage, steps.storage_step)?;
    let cf_values = build_axis(oracle_cfg.max_clean_firm, steps.cf_step)?;

    let mut best_lcoe = f64::INFINITY;
    let mut best_match = 0.0;
    let mut best_dev = f64::INFINITY;
    let mut best_point = (0.0, 0.0, 0.0, 0.0);
    let mut evaluations = 0u64;
    let mut found = false;

    for &solar in &solar_values {
        for &wind in &wind_values {
            for &storage in &storage_values {
                for &cf in &cf_values {
                    let sim_config = SimulationConfig {
                        solar_capacity: solar,
                        wind_capacity: wind,
                        storage_capacity: storage,
                        clean_firm_capacity: cf,
                        battery_efficiency: 0.85,
                        max_demand_response: 0.0,
                        battery_mode,
                    };
                    let sim = simulate_system(
                        &sim_config,
                        &scenario.profiles.solar,
                        &scenario.profiles.wind,
                        &scenario.profiles.load,
                    )?;
                    evaluations = evaluations.saturating_add(1);

                    let deviation = (sim.clean_match_pct - target).abs();
                    if deviation > tolerance {
                        continue;
                    }

                    let lcoe = calculate_lcoe(&sim, solar, wind, storage, cf, costs).total_lcoe;
                    if compare_oracle_candidates(
                        lcoe,
                        deviation,
                        (solar, wind, storage, cf),
                        best_lcoe,
                        best_dev,
                        best_point,
                    )
                    .is_lt()
                    {
                        found = true;
                        best_lcoe = lcoe;
                        best_match = sim.clean_match_pct;
                        best_dev = deviation;
                        best_point = (solar, wind, storage, cf);
                    }
                }
            }
        }
    }

    if !found {
        return Err(format!(
            "Oracle found no feasible point (target={}, tolerance={})",
            target, tolerance
        ));
    }

    Ok(OracleResult {
        lcoe: best_lcoe,
        clean_match: best_match,
        evaluations,
    })
}

fn make_initial_candidates(
    config: &V2AdaptiveAuditConfig,
    scenario_count: usize,
    rng: &mut DeterministicRng,
) -> Vec<TrialCandidate> {
    let mut candidates = Vec::new();
    let base = LeverMultipliers::default();

    for scenario_idx in 0..scenario_count {
        candidates.push(TrialCandidate {
            scenario_idx,
            multipliers: base.clone(),
            generation: 0,
            parent_trial_id: None,
        });

        for lever_idx in 0..8 {
            let mut low = base.values();
            low[lever_idx] = config.min_multiplier;
            candidates.push(TrialCandidate {
                scenario_idx,
                multipliers: LeverMultipliers::from_values(low),
                generation: 0,
                parent_trial_id: None,
            });

            let mut high = base.values();
            high[lever_idx] = config.max_multiplier;
            candidates.push(TrialCandidate {
                scenario_idx,
                multipliers: LeverMultipliers::from_values(high),
                generation: 0,
                parent_trial_id: None,
            });
        }

        let mut all_low = base.values();
        all_low.fill(config.min_multiplier);
        candidates.push(TrialCandidate {
            scenario_idx,
            multipliers: LeverMultipliers::from_values(all_low),
            generation: 0,
            parent_trial_id: None,
        });

        let mut all_high = base.values();
        all_high.fill(config.max_multiplier);
        candidates.push(TrialCandidate {
            scenario_idx,
            multipliers: LeverMultipliers::from_values(all_high),
            generation: 0,
            parent_trial_id: None,
        });
    }

    for _ in 0..(scenario_count * 3) {
        candidates.push(random_candidate(config, scenario_count, 0, None, rng));
    }

    candidates
}

fn derive_next_candidates(
    config: &V2AdaptiveAuditConfig,
    trials: &[AdaptiveTrialReport],
    generation: usize,
    scenarios: &[Scenario],
    rng: &mut DeterministicRng,
) -> Vec<TrialCandidate> {
    let mut ranked: Vec<&AdaptiveTrialReport> = trials.iter().filter(|t| t.success).collect();
    ranked.sort_by(|a, b| compare_f64(b.weakness_score, a.weakness_score));

    let top_count = config.batch_size.min(6).max(2);
    let mut out = Vec::new();

    for trial in ranked.into_iter().take(top_count) {
        let scenario_idx = scenario_index(scenarios, trial);
        let parent_id = Some(trial.trial_id);
        let base = trial.multipliers.values();

        let lever_idx = rng.choose_index(8);
        let mut down = base;
        down[lever_idx] = clamp_multiplier(base[lever_idx] * 0.5, config);
        out.push(TrialCandidate {
            scenario_idx,
            multipliers: LeverMultipliers::from_values(down),
            generation: generation + 1,
            parent_trial_id: parent_id,
        });

        let mut up = base;
        up[lever_idx] = clamp_multiplier(base[lever_idx] * 2.0, config);
        out.push(TrialCandidate {
            scenario_idx,
            multipliers: LeverMultipliers::from_values(up),
            generation: generation + 1,
            parent_trial_id: parent_id,
        });

        let mut jitter = base;
        for value in &mut jitter {
            let noise = (rng.next_f64() - 0.5) * 1.3;
            *value = clamp_multiplier(*value * (1.0 + noise), config);
        }
        out.push(TrialCandidate {
            scenario_idx,
            multipliers: LeverMultipliers::from_values(jitter),
            generation: generation + 1,
            parent_trial_id: parent_id,
        });

        let mut extreme = base;
        extreme[lever_idx] = if base[lever_idx] > 1.0 {
            config.max_multiplier
        } else {
            config.min_multiplier
        };
        out.push(TrialCandidate {
            scenario_idx,
            multipliers: LeverMultipliers::from_values(extreme),
            generation: generation + 1,
            parent_trial_id: parent_id,
        });

        if let Some(peer_scenario_idx) = sibling_scenario_with_same_zone(scenarios, scenario_idx) {
            out.push(TrialCandidate {
                scenario_idx: peer_scenario_idx,
                multipliers: trial.multipliers.clone(),
                generation: generation + 1,
                parent_trial_id: parent_id,
            });
        }
    }

    out
}

fn random_candidate(
    config: &V2AdaptiveAuditConfig,
    scenario_count: usize,
    generation: usize,
    parent_trial_id: Option<u64>,
    rng: &mut DeterministicRng,
) -> TrialCandidate {
    let scenario_idx = rng.choose_index(scenario_count);
    let values = std::array::from_fn(|_| {
        let u = rng.next_f64();
        config.min_multiplier * (config.max_multiplier / config.min_multiplier).powf(u)
    });
    TrialCandidate {
        scenario_idx,
        multipliers: LeverMultipliers::from_values(values),
        generation,
        parent_trial_id,
    }
}

fn scenario_index(scenarios: &[Scenario], trial: &AdaptiveTrialReport) -> usize {
    scenarios
        .iter()
        .position(|s| s.zone == trial.zone && (s.target - trial.target).abs() < 1e-9)
        .unwrap_or(0)
}

fn sibling_scenario_with_same_zone(scenarios: &[Scenario], base_idx: usize) -> Option<usize> {
    let base = scenarios.get(base_idx)?;
    scenarios
        .iter()
        .enumerate()
        .filter(|(idx, scenario)| {
            *idx != base_idx && scenario.zone == base.zone && scenario.target > base.target
        })
        .min_by(|a, b| compare_f64(a.1.target, b.1.target))
        .map(|(idx, _)| idx)
}

fn candidate_key(candidate: &TrialCandidate) -> String {
    let values = candidate.multipliers.values();
    format!(
        "{}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}:{:.3}",
        candidate.scenario_idx,
        values[0],
        values[1],
        values[2],
        values[3],
        values[4],
        values[5],
        values[6],
        values[7]
    )
}

fn oracle_cache_key(
    scenario_idx: usize,
    multipliers: &LeverMultipliers,
    level: OracleLevel,
) -> OracleCacheKey {
    let v = multipliers.values();
    OracleCacheKey {
        scenario_idx,
        level: match level {
            OracleLevel::Coarse => 0,
            OracleLevel::Fine => 1,
        },
        solar_capex_x1000: (v[0] * 1000.0).round() as i32,
        wind_capex_x1000: (v[1] * 1000.0).round() as i32,
        storage_capex_x1000: (v[2] * 1000.0).round() as i32,
        clean_firm_capex_x1000: (v[3] * 1000.0).round() as i32,
        gas_capex_x1000: (v[4] * 1000.0).round() as i32,
        gas_price_x1000: (v[5] * 1000.0).round() as i32,
        clean_firm_fuel_x1000: (v[6] * 1000.0).round() as i32,
        gas_var_om_x1000: (v[7] * 1000.0).round() as i32,
    }
}

fn recompute_weakness_score(report: &mut AdaptiveTrialReport, target: f64, tolerance: f64) {
    let mut score = 0.0;
    let mut flags = Vec::new();

    if !report.success {
        report.weakness_score = 1000.0;
        if let Some(error) = &report.error {
            flags.push(format!("runtime_error:{}", error));
        } else {
            flags.push("runtime_error".to_string());
        }
        report.flags = flags;
        return;
    }

    if let Some(gap) = report.fast_vs_accurate_gap_pct {
        score += gap * 0.5;
        if gap > 1.0 {
            flags.push("fast_accurate_disagreement".to_string());
        }
    }

    if let Some(ratio) = report.runtime_ratio_accurate_vs_fast {
        if ratio > 1.0 {
            score += (ratio - 1.0) * 0.35;
        }
        if ratio > 2.0 {
            flags.push("runtime_over_budget".to_string());
        }
    }

    if let Some(deviation) = report.selected_deviation {
        if deviation > tolerance {
            score += (deviation / tolerance.max(1e-6)) * 4.0;
            flags.push("target_deviation".to_string());
        }
    }

    if let Some(gap) = report.selected_gap_vs_coarse_pct {
        score += gap * 1.25;
        if gap > 1.0 {
            flags.push("coarse_oracle_gap_high".to_string());
        }
    }

    if let Some(gap) = report.selected_gap_vs_fine_pct {
        score += gap * 2.0;
        if gap > 1.0 {
            flags.push("fine_oracle_gap_high".to_string());
        }
    }

    if let Some(stop_reason) = &report.accurate_stop_reason {
        if stop_reason == "eval_budget_exhausted" {
            score += 0.75;
            flags.push("accurate_eval_budget_exhausted".to_string());
        }
        if stop_reason == "time_budget_exhausted" {
            score += 0.5;
            flags.push("accurate_time_budget_exhausted".to_string());
        }
        if stop_reason == "no_further_improvement" {
            score += 0.3;
            flags.push("accurate_stalled".to_string());
        }
    }

    if let (Some(accepted), Some(rejected)) = (
        report.accurate_accepted_moves,
        report.accurate_rejected_feasible_moves,
    ) {
        if rejected > accepted.saturating_mul(8) && rejected >= 24 {
            score += 0.4;
            flags.push("many_rejected_feasible_moves".to_string());
        }
    }

    if let Some(improvement) = report.accurate_lcoe_improvement {
        if improvement < 0.05 {
            score += 0.15;
        }
    }

    if let Some(selected_lcoe) = report.selected_lcoe {
        if !selected_lcoe.is_finite() || selected_lcoe <= 0.0 {
            score += 5.0;
            flags.push("invalid_lcoe".to_string());
        }
    }

    if flags.is_empty() && (target >= 95.0) {
        flags.push("high_target_monitored".to_string());
    }

    report.weakness_score = score;
    report.flags = flags;
}

fn summarize_trials(
    trials: &[AdaptiveTrialReport],
    config: &V2AdaptiveAuditConfig,
) -> V2AdaptiveAuditSummary {
    let successful_trials = trials.iter().filter(|t| t.success).count();
    let coarse_oracle_trials = trials
        .iter()
        .filter(|t| t.coarse_oracle_lcoe.is_some())
        .count();
    let fine_oracle_trials = trials
        .iter()
        .filter(|t| t.fine_oracle_lcoe.is_some())
        .count();

    let runtime_ratios: Vec<f64> = trials
        .iter()
        .filter_map(|t| t.runtime_ratio_accurate_vs_fast)
        .collect();
    let fine_gaps: Vec<f64> = trials
        .iter()
        .filter_map(|t| t.selected_gap_vs_fine_pct)
        .collect();

    let median_runtime_ratio = median(&runtime_ratios);
    let median_fine_gap = median(&fine_gaps);
    let p95_fine_gap = percentile(&fine_gaps, 95.0);
    let worst_fine_gap = fine_gaps.iter().copied().reduce(f64::max);

    let (highest_weakness_score, highest_weakness_trial_id) = trials
        .iter()
        .max_by(|a, b| compare_f64(a.weakness_score, b.weakness_score))
        .map(|trial| (Some(trial.weakness_score), Some(trial.trial_id)))
        .unwrap_or((None, None));

    let (lever_min, lever_max) = lever_coverage(trials);

    let runtime_pass = median_runtime_ratio
        .map(|v| v <= config.runtime_ratio_limit)
        .unwrap_or(false);
    let gap_pass = p95_fine_gap
        .map(|v| v <= config.fine_gap_limit_pct)
        .unwrap_or(false);
    let pass = runtime_pass && gap_pass && successful_trials == trials.len();

    V2AdaptiveAuditSummary {
        trial_count: trials.len(),
        successful_trials,
        coarse_oracle_trials,
        fine_oracle_trials,
        median_runtime_ratio_accurate_vs_fast: median_runtime_ratio,
        median_selected_gap_vs_fine_pct: median_fine_gap,
        p95_selected_gap_vs_fine_pct: p95_fine_gap,
        worst_selected_gap_vs_fine_pct: worst_fine_gap,
        highest_weakness_score,
        highest_weakness_trial_id,
        runtime_ratio_limit: config.runtime_ratio_limit,
        fine_gap_limit_pct: config.fine_gap_limit_pct,
        runtime_pass,
        gap_pass,
        pass,
        lever_coverage_min: lever_min,
        lever_coverage_max: lever_max,
    }
}

fn generate_recommendations(trials: &[AdaptiveTrialReport]) -> Vec<AdaptiveFixRecommendation> {
    let mut recommendations = Vec::new();

    let eval_exhausted = trials
        .iter()
        .filter(|trial| {
            trial
                .accurate_stop_reason
                .as_deref()
                .map(|reason| reason == "eval_budget_exhausted")
                .unwrap_or(false)
                && trial
                    .selected_gap_vs_fine_pct
                    .map(|gap| gap > 1.0)
                    .unwrap_or(false)
        })
        .count();
    if eval_exhausted >= 2 {
        recommendations.push(AdaptiveFixRecommendation {
            key: "increase-accurate-budget".to_string(),
            rationale: "Multiple high-gap trials stopped due to evaluation budget exhaustion.".to_string(),
            suggested_action: "Increase `adaptive_hard_max_extra_evals` or add more multi-start seeds for high targets.".to_string(),
            evidence_count: eval_exhausted,
        });
    }

    let stalled = trials
        .iter()
        .filter(|trial| {
            trial
                .accurate_stop_reason
                .as_deref()
                .map(|reason| reason == "no_further_improvement")
                .unwrap_or(false)
                && trial
                    .selected_gap_vs_fine_pct
                    .map(|gap| gap > 0.75)
                    .unwrap_or(false)
        })
        .count();
    if stalled >= 2 {
        recommendations.push(AdaptiveFixRecommendation {
            key: "add-exploration-restarts".to_string(),
            rationale: "Several cases stalled locally with still-meaningful gap to oracle.".to_string(),
            suggested_action: "Increase exploration seeds or add another randomized restart stage in accurate polish.".to_string(),
            evidence_count: stalled,
        });
    }

    let rejected_dense = trials
        .iter()
        .filter(|trial| {
            match (
                trial.accurate_rejected_feasible_moves,
                trial.accurate_accepted_moves,
                trial.selected_gap_vs_fine_pct,
            ) {
                (Some(rejected), Some(accepted), Some(gap)) => {
                    rejected > accepted.saturating_mul(8) && rejected > 30 && gap > 0.75
                }
                _ => false,
            }
        })
        .count();
    if rejected_dense >= 2 {
        recommendations.push(AdaptiveFixRecommendation {
            key: "retune-acceptance-threshold".to_string(),
            rationale: "Many feasible moves are rejected in high-gap regions, suggesting threshold/schedule mismatch.".to_string(),
            suggested_action: "Reduce `lcoe_improve_min` in final micro stages or enlarge micro step schedule around stall points.".to_string(),
            evidence_count: rejected_dense,
        });
    }

    let top_failures: Vec<&AdaptiveTrialReport> = trials
        .iter()
        .filter(|trial| trial.selected_gap_vs_fine_pct.is_some())
        .collect();
    if !top_failures.is_empty() {
        let mut sorted = top_failures;
        sorted.sort_by(|a, b| {
            compare_f64(
                b.selected_gap_vs_fine_pct.unwrap_or(0.0),
                a.selected_gap_vs_fine_pct.unwrap_or(0.0),
            )
        });
        let take_n = sorted.len().min(8);
        let mut lever_sums = [0.0; 8];
        for trial in sorted.iter().take(take_n) {
            let v = trial.multipliers.values();
            for i in 0..8 {
                lever_sums[i] += v[i];
            }
        }
        let mut dominant = Vec::new();
        for (idx, sum) in lever_sums.iter().enumerate() {
            let avg = *sum / take_n as f64;
            if avg > 1.8 || avg < 0.55 {
                dominant.push((lever_name(idx), avg));
            }
        }
        if !dominant.is_empty() {
            let description = dominant
                .iter()
                .map(|(name, avg)| format!("{}={:.2}x", name, avg))
                .collect::<Vec<_>>()
                .join(", ");
            recommendations.push(AdaptiveFixRecommendation {
                key: "stress-region-regression".to_string(),
                rationale: format!(
                    "Worst oracle-gap cases concentrate in stressed lever regions: {}.",
                    description
                ),
                suggested_action: "Add targeted regression scenarios for these lever regions and retune accurate-mode seed generation there.".to_string(),
                evidence_count: dominant.len(),
            });
        }
    }

    recommendations
}

fn lever_coverage(trials: &[AdaptiveTrialReport]) -> (LeverMultipliers, LeverMultipliers) {
    if trials.is_empty() {
        return (LeverMultipliers::default(), LeverMultipliers::default());
    }

    let mut min_values = [f64::INFINITY; 8];
    let mut max_values = [f64::NEG_INFINITY; 8];

    for trial in trials {
        let values = trial.multipliers.values();
        for i in 0..8 {
            min_values[i] = min_values[i].min(values[i]);
            max_values[i] = max_values[i].max(values[i]);
        }
    }

    (
        LeverMultipliers::from_values(min_values),
        LeverMultipliers::from_values(max_values),
    )
}

fn build_scenarios(config: &V2AdaptiveAuditConfig) -> Result<Vec<Scenario>, String> {
    let zones = load_zone_map()?;

    let mut scenarios = Vec::new();
    let baseline = CostParams::default_costs();

    for zone_name in &config.zones {
        let profiles = get_zone_profiles(&zones, zone_name)?;

        for &target in &config.targets {
            let mut optimizer_config = OptimizerConfig::default();
            optimizer_config.target_clean_match = target;
            optimizer_config.max_solar = config.oracle.max_solar;
            optimizer_config.max_wind = config.oracle.max_wind;
            optimizer_config.max_storage = config.oracle.max_storage;
            optimizer_config.max_clean_firm = config.oracle.max_clean_firm;

            scenarios.push(Scenario {
                zone: zone_name.clone(),
                target,
                tolerance: config.target_tolerance,
                optimizer_config,
                baseline_costs: baseline.clone(),
                profiles: profiles.clone(),
            });
        }
    }

    Ok(scenarios)
}

fn validate_config(config: &V2AdaptiveAuditConfig) -> Result<(), String> {
    if config.trial_budget == 0 {
        return Err("trial_budget must be > 0".to_string());
    }
    if config.batch_size == 0 {
        return Err("batch_size must be > 0".to_string());
    }
    if config.max_generations == 0 {
        return Err("max_generations must be > 0".to_string());
    }
    if config.targets.is_empty() {
        return Err("targets must not be empty".to_string());
    }
    if config.zones.is_empty() {
        return Err("zones must not be empty".to_string());
    }
    if config.target_tolerance <= 0.0 {
        return Err("target_tolerance must be > 0".to_string());
    }
    if config.min_multiplier <= 0.0 || config.max_multiplier <= 0.0 {
        return Err("multiplier bounds must be > 0".to_string());
    }
    if config.max_multiplier < config.min_multiplier {
        return Err("max_multiplier must be >= min_multiplier".to_string());
    }
    Ok(())
}

fn clamp_multiplier(value: f64, config: &V2AdaptiveAuditConfig) -> f64 {
    value
        .clamp(config.min_multiplier, config.max_multiplier)
        .max(1e-6)
}

fn clamp_costs(costs: &mut CostParams) {
    costs.solar_capex = costs.solar_capex.max(1e-6);
    costs.wind_capex = costs.wind_capex.max(1e-6);
    costs.storage_capex = costs.storage_capex.max(1e-6);
    costs.clean_firm_capex = costs.clean_firm_capex.max(1e-6);
    costs.gas_capex = costs.gas_capex.max(1e-6);
    costs.gas_price = costs.gas_price.max(1e-6);
    costs.clean_firm_fuel = costs.clean_firm_fuel.max(1e-6);
    costs.gas_var_om = costs.gas_var_om.max(1e-6);
}

fn build_axis(max: f64, step: f64) -> Result<Vec<f64>, String> {
    if max < 0.0 {
        return Err(format!("Negative max bound: {}", max));
    }
    if step <= 0.0 {
        return Err(format!("Step must be > 0.0, got {}", step));
    }

    if max == 0.0 {
        return Ok(vec![0.0]);
    }

    let mut values = vec![0.0];
    let mut current = 0.0;
    while current + step < max - 1e-9 {
        current += step;
        values.push(current);
    }
    if (values[values.len() - 1] - max).abs() > 1e-9 {
        values.push(max);
    }
    Ok(values)
}

fn load_zone_map() -> Result<HashMap<String, ZoneProfiles>, String> {
    let raw = fs::read_to_string(ZONES_PATH)
        .map_err(|e| format!("Failed to read {}: {}", ZONES_PATH, e))?;
    serde_json::from_str::<HashMap<String, ZoneProfiles>>(&raw)
        .map_err(|e| format!("Failed to parse {}: {}", ZONES_PATH, e))
}

fn get_zone_profiles(
    zones: &HashMap<String, ZoneProfiles>,
    requested_name: &str,
) -> Result<Profiles, String> {
    let requested = requested_name.to_lowercase();
    let zone = zones
        .iter()
        .find(|(name, _)| name.to_lowercase() == requested)
        .map(|(_, zone)| zone)
        .ok_or_else(|| format!("Zone '{}' not found in {}", requested_name, ZONES_PATH))?;

    if zone.solar.len() != HOURS_PER_YEAR
        || zone.wind.len() != HOURS_PER_YEAR
        || zone.load.len() != HOURS_PER_YEAR
    {
        return Err(format!(
            "Zone '{}' profile lengths invalid: solar={}, wind={}, load={}, expected {}",
            requested_name,
            zone.solar.len(),
            zone.wind.len(),
            zone.load.len(),
            HOURS_PER_YEAR
        ));
    }

    let mut load = zone.load.clone();
    normalize_load_to_100mw(&mut load);

    Ok(Profiles {
        solar: zone.solar.clone(),
        wind: zone.wind.clone(),
        load,
    })
}

fn normalize_load_to_100mw(load: &mut [f64]) {
    let sum: f64 = load.iter().sum();
    if sum <= 0.0 {
        return;
    }
    let target_sum = 100.0 * HOURS_PER_YEAR as f64;
    let scale = target_sum / sum;
    for x in load.iter_mut() {
        *x *= scale;
    }
}

fn compare_oracle_candidates(
    lcoe_a: f64,
    dev_a: f64,
    point_a: (f64, f64, f64, f64),
    lcoe_b: f64,
    dev_b: f64,
    point_b: (f64, f64, f64, f64),
) -> Ordering {
    compare_f64(lcoe_a, lcoe_b)
        .then_with(|| compare_f64(dev_a, dev_b))
        .then_with(|| compare_f64(point_a.0, point_b.0))
        .then_with(|| compare_f64(point_a.1, point_b.1))
        .then_with(|| compare_f64(point_a.2, point_b.2))
        .then_with(|| compare_f64(point_a.3, point_b.3))
}

fn compare_f64(a: f64, b: f64) -> Ordering {
    if (a - b).abs() <= 1e-9 {
        Ordering::Equal
    } else {
        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
    }
}

fn append_error(existing: Option<String>, addition: String) -> String {
    match existing {
        Some(prev) => format!("{} | {}", prev, addition),
        None => addition,
    }
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let n = sorted.len();
    if n % 2 == 0 {
        Some((sorted[n / 2 - 1] + sorted[n / 2]) / 2.0)
    } else {
        Some(sorted[n / 2])
    }
}

fn percentile(values: &[f64], pct: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

    let rank = (pct / 100.0) * (sorted.len() - 1) as f64;
    let low = rank.floor() as usize;
    let high = rank.ceil() as usize;
    if low == high {
        Some(sorted[low])
    } else {
        let weight = rank - low as f64;
        Some(sorted[low] * (1.0 - weight) + sorted[high] * weight)
    }
}

fn lever_name(idx: usize) -> &'static str {
    match idx {
        0 => "solar_capex",
        1 => "wind_capex",
        2 => "storage_capex",
        3 => "clean_firm_capex",
        4 => "gas_capex",
        5 => "gas_price",
        6 => "clean_firm_fuel",
        7 => "gas_var_om",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_audit_smoke() {
        let config = V2AdaptiveAuditConfig {
            trial_budget: 8,
            batch_size: 4,
            max_generations: 3,
            zones: vec!["california".to_string()],
            targets: vec![70.0],
            coarse_oracle_top_k: 0,
            fine_oracle_top_k: 0,
            random_exploration_per_gen: 1,
            oracle: AdaptiveOracleConfig {
                max_solar: 40.0,
                max_wind: 60.0,
                max_storage: 40.0,
                max_clean_firm: 40.0,
                coarse_solar_step: 20.0,
                coarse_wind_step: 20.0,
                coarse_storage_step: 20.0,
                coarse_cf_step: 10.0,
                fine_solar_step: 20.0,
                fine_wind_step: 20.0,
                fine_storage_step: 20.0,
                fine_cf_step: 10.0,
            },
            ..V2AdaptiveAuditConfig::default()
        };

        let report = run_v2_adaptive_audit(&config).expect("adaptive audit should succeed");
        assert!(!report.trials.is_empty(), "expected non-empty trial list");
        assert!(
            report.summary.trial_count <= config.trial_budget,
            "trial count should respect budget"
        );
    }
}
