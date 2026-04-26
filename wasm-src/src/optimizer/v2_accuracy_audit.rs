use super::v2_hierarchical::{run_v2_optimizer, run_v2_optimizer_mode_detailed, V2Mode};
use crate::economics::calculate_lcoe;
use crate::simulation::simulate_system;
use crate::types::{BatteryMode, CostParams, OptimizerConfig, SimulationConfig, HOURS_PER_YEAR};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const ZONES_PATH: &str = "../data/zones.json";
const ORACLE_SOLAR_STEP: f64 = 10.0;
const ORACLE_WIND_STEP: f64 = 10.0;
const ORACLE_STORAGE_STEP: f64 = 10.0;
const ORACLE_CF_STEP: f64 = 5.0;
const RUNTIME_RATIO_LIMIT: f64 = 2.0;
const P95_GAP_LIMIT_PCT: f64 = 1.0;

#[derive(Clone)]
struct Profiles {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

#[derive(Clone)]
struct ScenarioDefinition {
    case_name: String,
    target: f64,
    tolerance: f64,
    battery_mode: BatteryMode,
    costs: CostParams,
    profiles: Profiles,
    optimizer_config: OptimizerConfig,
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

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V2AccuracyAuditCaseReport {
    pub case_name: String,
    pub target: f64,
    pub tolerance: f64,
    pub battery_mode: BatteryMode,
    pub fast_time_ms: Option<f64>,
    pub accurate_time_ms: Option<f64>,
    pub oracle_time_ms: Option<f64>,
    pub fast_lcoe: Option<f64>,
    pub accurate_lcoe: Option<f64>,
    pub oracle_lcoe: Option<f64>,
    pub fast_match: Option<f64>,
    pub accurate_match: Option<f64>,
    pub oracle_match: Option<f64>,
    pub fast_gap_vs_oracle_pct: Option<f64>,
    pub accurate_gap_vs_oracle_pct: Option<f64>,
    pub selected_mode: String,
    pub selected_lcoe: Option<f64>,
    pub selected_match: Option<f64>,
    pub selected_deviation: Option<f64>,
    pub selected_gap_vs_oracle_pct: Option<f64>,
    pub runtime_ratio_selected_vs_fast: Option<f64>,
    pub oracle_evaluations: Option<u64>,
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
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V2AccuracySummary {
    pub mode: String,
    pub case_count: usize,
    pub successful_cases: usize,
    pub mean_selected_gap_pct: Option<f64>,
    pub median_selected_gap_pct: Option<f64>,
    pub p95_selected_gap_pct: Option<f64>,
    pub median_runtime_ratio_selected_vs_fast: Option<f64>,
    pub runtime_ratio_limit: f64,
    pub p95_gap_limit_pct: f64,
    pub runtime_pass: bool,
    pub gap_pass: bool,
    pub pass: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V2AccuracyAuditSuiteReport {
    pub suite: String,
    pub mode: String,
    pub generated_unix_ms: u128,
    pub cases: Vec<V2AccuracyAuditCaseReport>,
    pub summary: V2AccuracySummary,
}

pub fn run_v2_accuracy_audit_suite(
    suite: &str,
    mode: V2Mode,
) -> Result<V2AccuracyAuditSuiteReport, String> {
    let scenarios = build_suite(suite)?;
    let mut cases = Vec::with_capacity(scenarios.len());
    for scenario in &scenarios {
        cases.push(execute_case(scenario, mode));
    }

    let summary = summarize(&cases, mode);
    let generated_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Clock error: {}", e))?
        .as_millis();

    Ok(V2AccuracyAuditSuiteReport {
        suite: suite.to_string(),
        mode: mode_name(mode).to_string(),
        generated_unix_ms,
        cases,
        summary,
    })
}

fn execute_case(scenario: &ScenarioDefinition, mode: V2Mode) -> V2AccuracyAuditCaseReport {
    let mut report = V2AccuracyAuditCaseReport {
        case_name: scenario.case_name.clone(),
        target: scenario.target,
        tolerance: scenario.tolerance,
        battery_mode: scenario.battery_mode,
        fast_time_ms: None,
        accurate_time_ms: None,
        oracle_time_ms: None,
        fast_lcoe: None,
        accurate_lcoe: None,
        oracle_lcoe: None,
        fast_match: None,
        accurate_match: None,
        oracle_match: None,
        fast_gap_vs_oracle_pct: None,
        accurate_gap_vs_oracle_pct: None,
        selected_mode: mode_name(mode).to_string(),
        selected_lcoe: None,
        selected_match: None,
        selected_deviation: None,
        selected_gap_vs_oracle_pct: None,
        runtime_ratio_selected_vs_fast: None,
        oracle_evaluations: None,
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
            &scenario.costs,
            &scenario.optimizer_config,
            scenario.battery_mode,
            None,
        );
        report.fast_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
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
            &scenario.costs,
            &scenario.optimizer_config,
            scenario.battery_mode,
            None,
            V2Mode::Accurate,
            None,
        );
        report.accurate_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
        match result {
            Ok((v, diagnostics)) => {
                report.accurate_lcoe = Some(v.lcoe);
                report.accurate_match = Some(v.achieved_clean_match);
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

    let oracle = {
        let start = Instant::now();
        let result = run_fine_oracle(
            scenario.target,
            scenario.tolerance,
            &scenario.profiles,
            &scenario.costs,
            &scenario.optimizer_config,
            scenario.battery_mode,
        );
        report.oracle_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
        match result {
            Ok(v) => {
                report.oracle_lcoe = Some(v.lcoe);
                report.oracle_match = Some(v.clean_match);
                report.oracle_evaluations = Some(v.evaluations);
                Some(v)
            }
            Err(e) => {
                report.success = false;
                report.error = Some(append_error(
                    report.error.take(),
                    format!("Oracle failed: {}", e),
                ));
                None
            }
        }
    };

    if let (Some(fast_ms), Some(acc_ms), Some(fast)) =
        (report.fast_time_ms, report.accurate_time_ms, &fast_result)
    {
        let selected_ms = match mode {
            V2Mode::Fast => fast_ms,
            V2Mode::Accurate => acc_ms,
        };
        if fast_ms > 0.0 {
            report.runtime_ratio_selected_vs_fast = Some(selected_ms / fast_ms);
        }
        report.selected_lcoe = Some(match mode {
            V2Mode::Fast => fast.lcoe,
            V2Mode::Accurate => accurate_result
                .as_ref()
                .map(|r| r.lcoe)
                .unwrap_or(fast.lcoe),
        });
        report.selected_match = Some(match mode {
            V2Mode::Fast => fast.achieved_clean_match,
            V2Mode::Accurate => accurate_result
                .as_ref()
                .map(|r| r.achieved_clean_match)
                .unwrap_or(fast.achieved_clean_match),
        });
    }

    if let Some(selected_match) = report.selected_match {
        report.selected_deviation = Some((selected_match - scenario.target).abs());
    }

    if let Some(oracle_result) = &oracle {
        if oracle_result.lcoe.abs() > 0.0 {
            if let Some(v) = report.fast_lcoe {
                report.fast_gap_vs_oracle_pct =
                    Some(((v - oracle_result.lcoe).abs() / oracle_result.lcoe.abs()) * 100.0);
            }
            if let Some(v) = report.accurate_lcoe {
                report.accurate_gap_vs_oracle_pct =
                    Some(((v - oracle_result.lcoe).abs() / oracle_result.lcoe.abs()) * 100.0);
            }
            if let Some(v) = report.selected_lcoe {
                report.selected_gap_vs_oracle_pct =
                    Some(((v - oracle_result.lcoe).abs() / oracle_result.lcoe.abs()) * 100.0);
            }
        }
    }

    if let Some(dev) = report.selected_deviation {
        if dev > scenario.tolerance + 1e-6 {
            report.success = false;
            report.error = Some(append_error(
                report.error.take(),
                format!(
                    "Selected mode deviation {:.4} exceeded tolerance {:.4}",
                    dev, scenario.tolerance
                ),
            ));
        }
    }

    if fast_result.is_none() || accurate_result.is_none() || oracle.is_none() {
        report.success = false;
    }

    report
}

fn run_fine_oracle(
    target: f64,
    tolerance: f64,
    profiles: &Profiles,
    costs: &CostParams,
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
) -> Result<OracleResult, String> {
    let solar_values = build_axis(config.max_solar, ORACLE_SOLAR_STEP)?;
    let wind_values = build_axis(config.max_wind, ORACLE_WIND_STEP)?;
    let storage_values = build_axis(config.max_storage, ORACLE_STORAGE_STEP)?;
    let cf_values = build_axis(config.max_clean_firm, ORACLE_CF_STEP)?;

    let mut best_lcoe = f64::INFINITY;
    let mut best_match = 0.0;
    let mut best_deviation = f64::INFINITY;
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
                        &profiles.solar,
                        &profiles.wind,
                        &profiles.load,
                    )?;
                    evaluations += 1;

                    let deviation = (sim.clean_match_pct - target).abs();
                    if deviation > tolerance {
                        continue;
                    }

                    let lcoe = calculate_lcoe(&sim, solar, wind, storage, cf, costs).total_lcoe;
                    let better = compare_oracle_candidates(
                        lcoe,
                        deviation,
                        (solar, wind, storage, cf),
                        best_lcoe,
                        best_deviation,
                        best_point,
                    )
                    .is_lt();
                    if better {
                        found = true;
                        best_lcoe = lcoe;
                        best_match = sim.clean_match_pct;
                        best_deviation = deviation;
                        best_point = (solar, wind, storage, cf);
                    }
                }
            }
        }
    }

    if !found {
        return Err(format!(
            "No feasible fine-oracle point found for target {} and tolerance {}",
            target, tolerance
        ));
    }

    Ok(OracleResult {
        lcoe: best_lcoe,
        clean_match: best_match,
        evaluations,
    })
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

fn summarize(cases: &[V2AccuracyAuditCaseReport], mode: V2Mode) -> V2AccuracySummary {
    let selected_gaps: Vec<f64> = cases
        .iter()
        .filter_map(|c| c.selected_gap_vs_oracle_pct)
        .collect();
    let runtime_ratios: Vec<f64> = cases
        .iter()
        .filter_map(|c| c.runtime_ratio_selected_vs_fast)
        .collect();
    let successful_cases = cases.iter().filter(|c| c.success).count();

    let mean_selected_gap_pct = if selected_gaps.is_empty() {
        None
    } else {
        Some(selected_gaps.iter().sum::<f64>() / selected_gaps.len() as f64)
    };
    let median_selected_gap_pct = median(&selected_gaps);
    let p95_selected_gap_pct = percentile(&selected_gaps, 95.0);
    let median_runtime_ratio_selected_vs_fast = median(&runtime_ratios);

    let runtime_pass = match mode {
        V2Mode::Fast => true,
        V2Mode::Accurate => median_runtime_ratio_selected_vs_fast
            .map(|v| v <= RUNTIME_RATIO_LIMIT)
            .unwrap_or(false),
    };
    let gap_pass = p95_selected_gap_pct
        .map(|v| v <= P95_GAP_LIMIT_PCT)
        .unwrap_or(false);
    let pass = runtime_pass && gap_pass && successful_cases == cases.len();

    V2AccuracySummary {
        mode: mode_name(mode).to_string(),
        case_count: cases.len(),
        successful_cases,
        mean_selected_gap_pct,
        median_selected_gap_pct,
        p95_selected_gap_pct,
        median_runtime_ratio_selected_vs_fast,
        runtime_ratio_limit: RUNTIME_RATIO_LIMIT,
        p95_gap_limit_pct: P95_GAP_LIMIT_PCT,
        runtime_pass,
        gap_pass,
        pass,
    }
}

fn build_suite(suite: &str) -> Result<Vec<ScenarioDefinition>, String> {
    let zones = load_zone_map()?;
    let california = get_zone_profiles(&zones, "california")?;
    let texas = get_zone_profiles(&zones, "texas")?;

    let default_costs = CostParams::default_costs();
    let mut expensive_gas_costs = CostParams::default_costs();
    expensive_gas_costs.gas_price *= 3.0;
    let mut cheap_storage_costs = CostParams::default_costs();
    cheap_storage_costs.storage_capex *= 0.35;
    let mut cheap_clean_firm_costs = CostParams::default_costs();
    cheap_clean_firm_costs.clean_firm_capex *= 0.5;
    cheap_clean_firm_costs.clean_firm_fuel *= 0.75;
    let mut cheap_vre_costs = CostParams::default_costs();
    cheap_vre_costs.solar_capex *= 0.6;
    cheap_vre_costs.wind_capex *= 0.6;

    let mut base_config = OptimizerConfig::default();
    base_config.enable_solar = true;
    base_config.enable_wind = true;
    base_config.enable_storage = true;
    base_config.enable_clean_firm = true;
    base_config.max_solar = 100.0;
    base_config.max_wind = 400.0;
    base_config.max_storage = 100.0;
    base_config.max_clean_firm = 100.0;

    let mk_case = |name: &str, target: f64, costs: &CostParams, profiles: &Profiles| {
        let mut cfg = base_config.clone();
        cfg.target_clean_match = target;
        ScenarioDefinition {
            case_name: name.to_string(),
            target,
            tolerance: 0.5,
            battery_mode: BatteryMode::Hybrid,
            costs: costs.clone(),
            profiles: profiles.clone(),
            optimizer_config: cfg,
        }
    };

    let hard = vec![
        mk_case(
            "california_target_95_default",
            95.0,
            &default_costs,
            &california,
        ),
        mk_case(
            "california_target_99_default",
            99.0,
            &default_costs,
            &california,
        ),
        mk_case("texas_target_95_default", 95.0, &default_costs, &texas),
        mk_case(
            "texas_target_99_expensive_gas",
            99.0,
            &expensive_gas_costs,
            &texas,
        ),
    ];

    let smoke = vec![
        mk_case(
            "smoke_california_95_default",
            95.0,
            &default_costs,
            &california,
        ),
        mk_case(
            "smoke_texas_99_expensive_gas",
            99.0,
            &expensive_gas_costs,
            &texas,
        ),
    ];

    let confidence = vec![
        mk_case(
            "confidence_california_70_default",
            70.0,
            &default_costs,
            &california,
        ),
        mk_case(
            "confidence_california_85_cheap_vre",
            85.0,
            &cheap_vre_costs,
            &california,
        ),
        mk_case(
            "confidence_california_95_default",
            95.0,
            &default_costs,
            &california,
        ),
        mk_case(
            "confidence_california_99_expensive_gas",
            99.0,
            &expensive_gas_costs,
            &california,
        ),
        mk_case(
            "confidence_california_95_cheap_cf",
            95.0,
            &cheap_clean_firm_costs,
            &california,
        ),
        mk_case("confidence_texas_70_default", 70.0, &default_costs, &texas),
        mk_case(
            "confidence_texas_85_cheap_storage",
            85.0,
            &cheap_storage_costs,
            &texas,
        ),
        mk_case(
            "confidence_texas_95_expensive_gas",
            95.0,
            &expensive_gas_costs,
            &texas,
        ),
        mk_case("confidence_texas_99_default", 99.0, &default_costs, &texas),
        mk_case(
            "confidence_texas_99_cheap_cf",
            99.0,
            &cheap_clean_firm_costs,
            &texas,
        ),
    ];

    match suite {
        "hard" | "quick" => Ok(hard),
        "smoke" => Ok(smoke),
        "confidence" => Ok(confidence),
        other => Err(format!(
            "Unknown audit suite '{}'. Expected one of: smoke, hard, quick, confidence",
            other
        )),
    }
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

fn mode_name(mode: V2Mode) -> &'static str {
    match mode {
        V2Mode::Fast => "fast",
        V2Mode::Accurate => "accurate",
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
    let clamped = pct.clamp(0.0, 100.0);
    let rank = ((clamped / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted.get(rank).copied()
}
