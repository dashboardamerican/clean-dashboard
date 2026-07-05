use crate::economics::calculate_lcoe;
use crate::optimizer::{run_v2_optimizer, run_v2_optimizer_mode, V2Mode};
use crate::simulation::simulate_system;
use crate::types::{BatteryMode, CostParams, OptimizerConfig, OptimizerResult, HOURS_PER_YEAR};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const DEFAULT_TARGET_TOLERANCE_PCT: f64 = 0.2;
const DEFAULT_RUNTIME_BUDGET_MS: f64 = 500.0;
const SWEEP_GAS_CAPACITY_BUMP_TOLERANCE_MW: f64 = 1.0;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvalOptimizerMode {
    V2Fast,
    V2Accurate,
}

impl Default for EvalOptimizerMode {
    fn default() -> Self {
        Self::V2Fast
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioExpectation {
    Success,
    SuccessOrOffTarget,
    Any,
}

impl Default for ScenarioExpectation {
    fn default() -> Self {
        Self::Success
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EvalStatus {
    Success,
    OffTarget,
    Error,
    Infeasible,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ResourceToggles {
    #[serde(default = "default_true")]
    pub solar: bool,
    #[serde(default = "default_true")]
    pub wind: bool,
    #[serde(default = "default_true")]
    pub storage: bool,
    #[serde(default = "default_true")]
    pub clean_firm: bool,
}

fn default_true() -> bool {
    true
}

impl Default for ResourceToggles {
    fn default() -> Self {
        Self {
            solar: true,
            wind: true,
            storage: true,
            clean_firm: true,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioSuite {
    pub suite: String,
    #[serde(default)]
    pub description: String,
    #[serde(default = "default_target_tolerance")]
    pub default_target_tolerance_pct: f64,
    #[serde(default = "default_runtime_budget")]
    pub default_runtime_budget_ms: f64,
    pub scenarios: Vec<ScenarioSpec>,
}

fn default_target_tolerance() -> f64 {
    DEFAULT_TARGET_TOLERANCE_PCT
}

fn default_runtime_budget() -> f64 {
    DEFAULT_RUNTIME_BUDGET_MS
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScenarioSpec {
    pub id: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub zone: Option<String>,
    #[serde(default)]
    pub zones: Vec<String>,
    pub targets: Vec<f64>,
    #[serde(default)]
    pub battery_mode: BatteryMode,
    #[serde(default)]
    pub optimizer_mode: EvalOptimizerMode,
    #[serde(default)]
    pub resources: ResourceToggles,
    #[serde(default)]
    pub cost_overrides: Map<String, Value>,
    #[serde(default)]
    pub optimizer_overrides: Map<String, Value>,
    #[serde(default)]
    pub expectation: ScenarioExpectation,
    pub target_tolerance_pct: Option<f64>,
    pub runtime_budget_ms: Option<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ZoneProfiles {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RunManifest {
    pub generated_unix_ms: u128,
    pub crate_version: String,
    pub git_commit: Option<String>,
    pub scenario_path: String,
    pub scenario_hash: u64,
    pub profile_data_path: String,
    pub profile_data_hash: u64,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RunSummary {
    pub scenario_count: usize,
    pub point_count: usize,
    pub success_count: usize,
    pub off_target_count: usize,
    pub error_count: usize,
    pub infeasible_count: usize,
    pub validation_error_count: usize,
    pub validation_warning_count: usize,
    pub max_abs_deviation_pct: f64,
    pub mean_runtime_ms: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvalReport {
    pub manifest: RunManifest,
    pub suite: String,
    pub description: String,
    pub summary: RunSummary,
    pub points: Vec<EvalPoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EvalPoint {
    pub suite: String,
    pub scenario_id: String,
    pub scenario_description: String,
    pub zone: String,
    pub target: f64,
    pub achieved: Option<f64>,
    pub deviation_pct: Option<f64>,
    pub target_tolerance_pct: f64,
    pub runtime_budget_ms: f64,
    pub runtime_ms: f64,
    pub status: EvalStatus,
    pub expectation: ScenarioExpectation,
    pub optimizer_success: bool,
    pub optimizer_mode: EvalOptimizerMode,
    pub battery_mode: BatteryMode,
    pub resources: ResourceToggles,
    pub solar: f64,
    pub wind: f64,
    pub storage: f64,
    pub clean_firm: f64,
    pub gas_capacity: f64,
    pub lcoe: f64,
    pub solar_lcoe: f64,
    pub wind_lcoe: f64,
    pub storage_lcoe: f64,
    pub clean_firm_lcoe: f64,
    pub gas_lcoe: f64,
    pub num_evaluations: u32,
    pub cost_hash: u64,
    pub profile_hash: u64,
    pub warnings: Vec<String>,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: ValidationSeverity,
    pub scenario_id: String,
    pub target: f64,
    pub message: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ReportComparisonConfig {
    pub lcoe_regression_tolerance: f64,
    pub achieved_drift_tolerance_pct: f64,
    pub runtime_regression_factor: f64,
    pub runtime_regression_min_delta_ms: f64,
}

impl Default for ReportComparisonConfig {
    fn default() -> Self {
        Self {
            lcoe_regression_tolerance: 0.10,
            achieved_drift_tolerance_pct: 0.25,
            runtime_regression_factor: 1.5,
            runtime_regression_min_delta_ms: 50.0,
        }
    }
}

pub fn find_refactor_root() -> Result<PathBuf, String> {
    let cwd = env::current_dir().map_err(|e| format!("Failed to read cwd: {}", e))?;
    for ancestor in cwd.ancestors() {
        let direct = ancestor;
        if direct.join("optimizer_evals").is_dir() && direct.join("data/zones.json").exists() {
            return Ok(direct.to_path_buf());
        }

        let nested = ancestor.join("rust_refactor");
        if nested.join("optimizer_evals").is_dir() && nested.join("data/zones.json").exists() {
            return Ok(nested);
        }
    }

    Err(format!(
        "Could not find rust_refactor root from {}",
        cwd.display()
    ))
}

pub fn scenario_path_for_suite(suite: &str) -> Result<PathBuf, String> {
    let root = find_refactor_root()?;
    Ok(root
        .join("optimizer_evals")
        .join("scenarios")
        .join(format!("{}.json", suite)))
}

pub fn load_suite(path: &Path) -> Result<ScenarioSuite, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read scenario file {}: {}", path.display(), e))?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse scenario file {}: {}", path.display(), e))
}

pub fn load_suite_by_name(suite: &str) -> Result<(PathBuf, ScenarioSuite), String> {
    let path = scenario_path_for_suite(suite)?;
    let suite = load_suite(&path)?;
    Ok((path, suite))
}

pub fn run_suite_from_path(path: &Path) -> Result<EvalReport, String> {
    let root = find_refactor_root()?;
    let suite = load_suite(path)?;
    let zones_path = root.join("data/zones.json");
    let zones_raw = fs::read_to_string(&zones_path)
        .map_err(|e| format!("Failed to read zone data {}: {}", zones_path.display(), e))?;
    let zones: HashMap<String, ZoneProfiles> = serde_json::from_str(&zones_raw)
        .map_err(|e| format!("Failed to parse zone data {}: {}", zones_path.display(), e))?;

    let mut points = Vec::new();
    for scenario in &suite.scenarios {
        let zone_names = expand_scenario_zones(scenario, &zones)?;
        let is_multi_zone = zone_names.len() > 1;
        for zone_name in zone_names {
            let zone = zones.get(&zone_name).ok_or_else(|| {
                format!(
                    "Scenario {} references unknown zone {}",
                    scenario.id, zone_name
                )
            })?;
            let scenario_id = scenario_id_for_zone(scenario, &zone_name, is_multi_zone);
            validate_profiles(zone, &scenario_id)?;
            let scenario_points = run_scenario(&suite, scenario, &scenario_id, &zone_name, zone)?;
            points.extend(scenario_points);
        }
    }

    let manifest = RunManifest {
        generated_unix_ms: now_unix_ms()?,
        crate_version: env!("CARGO_PKG_VERSION").to_string(),
        git_commit: git_commit(&root),
        scenario_path: path.display().to_string(),
        scenario_hash: hash_bytes(
            &fs::read(path)
                .map_err(|e| format!("Failed to hash scenario file {}: {}", path.display(), e))?,
        ),
        profile_data_path: zones_path.display().to_string(),
        profile_data_hash: hash_bytes(zones_raw.as_bytes()),
    };

    let mut report = EvalReport {
        manifest,
        suite: suite.suite.clone(),
        description: suite.description.clone(),
        summary: RunSummary::default(),
        points,
    };
    refresh_summary(&mut report);
    Ok(report)
}

fn run_scenario(
    suite: &ScenarioSuite,
    scenario: &ScenarioSpec,
    scenario_id: &str,
    zone_name: &str,
    zone: &ZoneProfiles,
) -> Result<Vec<EvalPoint>, String> {
    let mut costs = apply_overrides(CostParams::default_costs(), &scenario.cost_overrides)?;
    // Keep the default CCS setting explicit in case old persisted settings omit it.
    if !costs.ccs_percentage.is_finite() {
        costs.ccs_percentage = 0.0;
    }

    let mut config = apply_overrides(OptimizerConfig::default(), &scenario.optimizer_overrides)?;
    config.enable_solar = scenario.resources.solar;
    config.enable_wind = scenario.resources.wind;
    config.enable_storage = scenario.resources.storage;
    config.enable_clean_firm = scenario.resources.clean_firm;

    let target_tolerance = scenario
        .target_tolerance_pct
        .unwrap_or(suite.default_target_tolerance_pct);
    let runtime_budget = scenario
        .runtime_budget_ms
        .unwrap_or(suite.default_runtime_budget_ms);
    let cost_hash = hash_json(&costs)?;
    let profile_hash = hash_profiles(zone);

    let mut points = Vec::with_capacity(scenario.targets.len());
    for &target in &scenario.targets {
        config.target_clean_match = target;
        let start = Instant::now();
        let result = match scenario.optimizer_mode {
            EvalOptimizerMode::V2Fast => run_v2_optimizer(
                target,
                &zone.solar,
                &zone.wind,
                &zone.load,
                &costs,
                &config,
                scenario.battery_mode,
                None,
            ),
            EvalOptimizerMode::V2Accurate => run_v2_optimizer_mode(
                target,
                &zone.solar,
                &zone.wind,
                &zone.load,
                &costs,
                &config,
                scenario.battery_mode,
                None,
                V2Mode::Accurate,
                None,
            ),
        };
        let runtime_ms = start.elapsed().as_secs_f64() * 1000.0;
        points.push(point_from_result(
            suite,
            scenario,
            scenario_id,
            zone_name,
            target,
            target_tolerance,
            runtime_budget,
            runtime_ms,
            &costs,
            &config,
            zone,
            result,
            cost_hash,
            profile_hash,
        ));
    }

    Ok(points)
}

#[allow(clippy::too_many_arguments)]
fn point_from_result(
    suite: &ScenarioSuite,
    scenario: &ScenarioSpec,
    scenario_id: &str,
    zone_name: &str,
    target: f64,
    target_tolerance: f64,
    runtime_budget: f64,
    runtime_ms: f64,
    costs: &CostParams,
    config: &OptimizerConfig,
    zone: &ZoneProfiles,
    result: Result<OptimizerResult, String>,
    cost_hash: u64,
    profile_hash: u64,
) -> EvalPoint {
    match result {
        Ok(r) => {
            let deviation = (r.achieved_clean_match - target).abs();
            let status = if r.success && deviation <= target_tolerance {
                EvalStatus::Success
            } else {
                EvalStatus::OffTarget
            };

            let mut warnings = Vec::new();
            if deviation <= target_tolerance && !r.success {
                warnings.push(
                    "achieved within scenario tolerance but optimizer_success=false".to_string(),
                );
            }
            if runtime_ms > runtime_budget {
                warnings.push(format!(
                    "runtime {:.2}ms exceeds budget {:.2}ms",
                    runtime_ms, runtime_budget
                ));
            }

            let (solar_lcoe, wind_lcoe, storage_lcoe, clean_firm_lcoe, gas_lcoe, gas_capacity) =
                match simulate_for_breakdown(config, scenario.battery_mode, &r, zone, costs) {
                    Ok(values) => values,
                    Err(e) => {
                        warnings.push(e);
                        (0.0, 0.0, 0.0, 0.0, 0.0, 0.0)
                    }
                };

            add_resource_warnings(&mut warnings, &scenario.resources, &r);

            EvalPoint {
                suite: suite.suite.clone(),
                scenario_id: scenario_id.to_string(),
                scenario_description: scenario.description.clone(),
                zone: zone_name.to_string(),
                target,
                achieved: Some(r.achieved_clean_match),
                deviation_pct: Some(deviation),
                target_tolerance_pct: target_tolerance,
                runtime_budget_ms: runtime_budget,
                runtime_ms,
                status,
                expectation: scenario.expectation,
                optimizer_success: r.success,
                optimizer_mode: scenario.optimizer_mode,
                battery_mode: scenario.battery_mode,
                resources: scenario.resources.clone(),
                solar: r.solar_capacity,
                wind: r.wind_capacity,
                storage: r.storage_capacity,
                clean_firm: r.clean_firm_capacity,
                gas_capacity,
                lcoe: r.lcoe,
                solar_lcoe,
                wind_lcoe,
                storage_lcoe,
                clean_firm_lcoe,
                gas_lcoe,
                num_evaluations: r.num_evaluations,
                cost_hash,
                profile_hash,
                warnings,
                error: None,
            }
        }
        Err(error) => EvalPoint {
            suite: suite.suite.clone(),
            scenario_id: scenario_id.to_string(),
            scenario_description: scenario.description.clone(),
            zone: zone_name.to_string(),
            target,
            achieved: None,
            deviation_pct: None,
            target_tolerance_pct: target_tolerance,
            runtime_budget_ms: runtime_budget,
            runtime_ms,
            status: EvalStatus::Error,
            expectation: scenario.expectation,
            optimizer_success: false,
            optimizer_mode: scenario.optimizer_mode,
            battery_mode: scenario.battery_mode,
            resources: scenario.resources.clone(),
            solar: 0.0,
            wind: 0.0,
            storage: 0.0,
            clean_firm: 0.0,
            gas_capacity: 0.0,
            lcoe: 0.0,
            solar_lcoe: 0.0,
            wind_lcoe: 0.0,
            storage_lcoe: 0.0,
            clean_firm_lcoe: 0.0,
            gas_lcoe: 0.0,
            num_evaluations: 0,
            cost_hash,
            profile_hash,
            warnings: Vec::new(),
            error: Some(error),
        },
    }
}

fn expand_scenario_zones(
    scenario: &ScenarioSpec,
    zones: &HashMap<String, ZoneProfiles>,
) -> Result<Vec<String>, String> {
    if scenario.zone.is_some() && !scenario.zones.is_empty() {
        return Err(format!(
            "Scenario {} must use either zone or zones, not both",
            scenario.id
        ));
    }

    if let Some(zone) = &scenario.zone {
        ensure_known_zone(&scenario.id, zone, zones)?;
        return Ok(vec![zone.clone()]);
    }

    if scenario.zones.is_empty() {
        return Err(format!(
            "Scenario {} must define zone or zones",
            scenario.id
        ));
    }

    if scenario.zones.iter().any(|zone| zone == "all") {
        if scenario.zones.len() != 1 {
            return Err(format!(
                "Scenario {} may use zones=[\"all\"] only by itself",
                scenario.id
            ));
        }
        let mut all_zones: Vec<String> = zones.keys().cloned().collect();
        all_zones.sort();
        return Ok(all_zones);
    }

    for zone in &scenario.zones {
        ensure_known_zone(&scenario.id, zone, zones)?;
    }
    Ok(scenario.zones.clone())
}

fn ensure_known_zone(
    scenario_id: &str,
    zone: &str,
    zones: &HashMap<String, ZoneProfiles>,
) -> Result<(), String> {
    if zones.contains_key(zone) {
        Ok(())
    } else {
        Err(format!(
            "Scenario {} references unknown zone {}",
            scenario_id, zone
        ))
    }
}

fn scenario_id_for_zone(scenario: &ScenarioSpec, zone_name: &str, is_multi_zone: bool) -> String {
    if is_multi_zone {
        format!("{}__{}", scenario.id, slugify_zone(zone_name))
    } else {
        scenario.id.clone()
    }
}

fn slugify_zone(zone_name: &str) -> String {
    let mut slug = String::with_capacity(zone_name.len());
    let mut last_was_separator = false;
    for ch in zone_name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            last_was_separator = false;
        } else if !last_was_separator {
            slug.push('_');
            last_was_separator = true;
        }
    }
    slug.trim_matches('_').to_string()
}

fn simulate_for_breakdown(
    config: &OptimizerConfig,
    battery_mode: BatteryMode,
    result: &OptimizerResult,
    zone: &ZoneProfiles,
    costs: &CostParams,
) -> Result<(f64, f64, f64, f64, f64, f64), String> {
    let sim_config = config.simulation_config_for_portfolio(
        result.solar_capacity,
        result.wind_capacity,
        result.storage_capacity,
        result.clean_firm_capacity,
        battery_mode,
    );
    let sim = simulate_system(&sim_config, &zone.solar, &zone.wind, &zone.load)
        .map_err(|e| format!("simulation breakdown failed: {}", e))?;
    let lcoe = calculate_lcoe(
        &sim,
        result.solar_capacity,
        result.wind_capacity,
        result.storage_capacity,
        result.clean_firm_capacity,
        costs,
    );
    Ok((
        lcoe.solar_lcoe,
        lcoe.wind_lcoe,
        lcoe.storage_lcoe,
        lcoe.clean_firm_lcoe,
        lcoe.gas_lcoe,
        sim.peak_gas * (1.0 + costs.reserve_margin / 100.0),
    ))
}

fn add_resource_warnings(
    warnings: &mut Vec<String>,
    resources: &ResourceToggles,
    result: &OptimizerResult,
) {
    let eps = 1e-9;
    if !resources.solar && result.solar_capacity.abs() > eps {
        warnings.push(format!(
            "disabled solar has nonzero capacity {:.6}",
            result.solar_capacity
        ));
    }
    if !resources.wind && result.wind_capacity.abs() > eps {
        warnings.push(format!(
            "disabled wind has nonzero capacity {:.6}",
            result.wind_capacity
        ));
    }
    if !resources.storage && result.storage_capacity.abs() > eps {
        warnings.push(format!(
            "disabled storage has nonzero capacity {:.6}",
            result.storage_capacity
        ));
    }
    if !resources.clean_firm && result.clean_firm_capacity.abs() > eps {
        warnings.push(format!(
            "disabled clean firm has nonzero capacity {:.6}",
            result.clean_firm_capacity
        ));
    }
}

pub fn validate_report(report: &EvalReport) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    for point in &report.points {
        validate_point(point, &mut issues);
    }
    validate_sweep_shape(report, &mut issues);
    issues
}

fn validate_sweep_shape(report: &EvalReport, issues: &mut Vec<ValidationIssue>) {
    let mut points_by_scenario_zone: HashMap<String, Vec<&EvalPoint>> = HashMap::new();
    for point in &report.points {
        points_by_scenario_zone
            .entry(format!("{}|{}", point.scenario_id, point.zone))
            .or_default()
            .push(point);
    }

    for mut points in points_by_scenario_zone.into_values() {
        points.sort_by(|a, b| a.target.total_cmp(&b.target));

        let mut previous_success: Option<&EvalPoint> = None;
        for point in points {
            if point.status != EvalStatus::Success {
                continue;
            }

            if let Some(previous) = previous_success {
                if is_all_resources_enabled(previous)
                    && is_all_resources_enabled(point)
                    && point.gas_capacity
                        > previous.gas_capacity + SWEEP_GAS_CAPACITY_BUMP_TOLERANCE_MW
                {
                    push_issue(
                        issues,
                        point,
                        ValidationSeverity::Warning,
                        format!(
                            "gas capacity increases across successful all-resource sweep from target {:.4} ({:.4} MW) to {:.4} ({:.4} MW)",
                            previous.target,
                            previous.gas_capacity,
                            point.target,
                            point.gas_capacity
                        ),
                    );
                }
            }

            previous_success = Some(point);
        }
    }
}

fn is_all_resources_enabled(point: &EvalPoint) -> bool {
    point.resources.solar
        && point.resources.wind
        && point.resources.storage
        && point.resources.clean_firm
}

pub fn compare_reports(
    baseline: &EvalReport,
    candidate: &EvalReport,
    config: &ReportComparisonConfig,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();
    let mut candidate_by_key: HashMap<String, &EvalPoint> = HashMap::new();
    for point in &candidate.points {
        candidate_by_key.insert(point_key(point), point);
    }

    let mut baseline_keys = std::collections::HashSet::new();
    for baseline_point in &baseline.points {
        let key = point_key(baseline_point);
        baseline_keys.insert(key.clone());
        let Some(candidate_point) = candidate_by_key.get(&key) else {
            push_issue(
                &mut issues,
                baseline_point,
                ValidationSeverity::Error,
                "candidate is missing baseline point".to_string(),
            );
            continue;
        };
        compare_point(baseline_point, candidate_point, config, &mut issues);
    }

    for candidate_point in &candidate.points {
        if !baseline_keys.contains(&point_key(candidate_point)) {
            push_issue(
                &mut issues,
                candidate_point,
                ValidationSeverity::Warning,
                "candidate has extra point not present in baseline".to_string(),
            );
        }
    }

    issues
}

fn compare_point(
    baseline: &EvalPoint,
    candidate: &EvalPoint,
    config: &ReportComparisonConfig,
    issues: &mut Vec<ValidationIssue>,
) {
    match baseline.status {
        EvalStatus::Success if candidate.status != EvalStatus::Success => {
            push_issue(
                issues,
                candidate,
                ValidationSeverity::Error,
                format!("status regressed from Success to {:?}", candidate.status),
            );
        }
        EvalStatus::OffTarget
            if matches!(candidate.status, EvalStatus::Error | EvalStatus::Infeasible) =>
        {
            push_issue(
                issues,
                candidate,
                ValidationSeverity::Error,
                format!("status regressed from OffTarget to {:?}", candidate.status),
            );
        }
        EvalStatus::Infeasible if candidate.status == EvalStatus::Error => {
            push_issue(
                issues,
                candidate,
                ValidationSeverity::Error,
                "status regressed from Infeasible to Error".to_string(),
            );
        }
        _ => {}
    }

    if candidate.lcoe - baseline.lcoe > config.lcoe_regression_tolerance {
        push_issue(
            issues,
            candidate,
            ValidationSeverity::Error,
            format!(
                "LCOE regressed by {:.4} $/MWh (baseline {:.4}, candidate {:.4}, tolerance {:.4})",
                candidate.lcoe - baseline.lcoe,
                baseline.lcoe,
                candidate.lcoe,
                config.lcoe_regression_tolerance
            ),
        );
    }

    if let (Some(baseline_achieved), Some(candidate_achieved)) =
        (baseline.achieved, candidate.achieved)
    {
        let drift = (candidate_achieved - baseline_achieved).abs();
        if drift > config.achieved_drift_tolerance_pct {
            push_issue(
                issues,
                candidate,
                ValidationSeverity::Warning,
                format!(
                    "achieved clean match drifted by {:.4} percentage points (baseline {:.4}, candidate {:.4})",
                    drift, baseline_achieved, candidate_achieved
                ),
            );
        }
    }

    let runtime_delta = candidate.runtime_ms - baseline.runtime_ms;
    if candidate.runtime_ms > baseline.runtime_ms * config.runtime_regression_factor
        && runtime_delta > config.runtime_regression_min_delta_ms
    {
        push_issue(
            issues,
            candidate,
            ValidationSeverity::Warning,
            format!(
                "runtime regressed from {:.2}ms to {:.2}ms",
                baseline.runtime_ms, candidate.runtime_ms
            ),
        );
    }
}

fn point_key(point: &EvalPoint) -> String {
    format!("{}|{}|{:.6}", point.scenario_id, point.zone, point.target)
}

fn validate_point(point: &EvalPoint, issues: &mut Vec<ValidationIssue>) {
    match point.expectation {
        ScenarioExpectation::Success if point.status != EvalStatus::Success => {
            push_issue(
                issues,
                point,
                ValidationSeverity::Error,
                format!(
                    "expected success but status is {:?}, achieved={:?}, deviation={:?}",
                    point.status, point.achieved, point.deviation_pct
                ),
            );
        }
        ScenarioExpectation::SuccessOrOffTarget
            if matches!(point.status, EvalStatus::Error | EvalStatus::Infeasible) =>
        {
            push_issue(
                issues,
                point,
                ValidationSeverity::Error,
                format!(
                    "expected success/off_target but status is {:?}: {:?}",
                    point.status, point.error
                ),
            );
        }
        _ => {}
    }

    if point.status == EvalStatus::Success {
        if !point.optimizer_success {
            push_issue(
                issues,
                point,
                ValidationSeverity::Error,
                "status success but optimizer_success=false".to_string(),
            );
        }
        if point
            .deviation_pct
            .map(|d| d > point.target_tolerance_pct)
            .unwrap_or(true)
        {
            push_issue(
                issues,
                point,
                ValidationSeverity::Error,
                format!(
                    "status success but deviation {:?} exceeds tolerance {:.4}",
                    point.deviation_pct, point.target_tolerance_pct
                ),
            );
        }
    }

    for (label, value) in [
        ("achieved", point.achieved.unwrap_or(0.0)),
        ("solar", point.solar),
        ("wind", point.wind),
        ("storage", point.storage),
        ("clean_firm", point.clean_firm),
        ("gas_capacity", point.gas_capacity),
        ("lcoe", point.lcoe),
    ] {
        if !value.is_finite() {
            push_issue(
                issues,
                point,
                ValidationSeverity::Error,
                format!("{} is not finite: {}", label, value),
            );
        }
    }

    if point.lcoe < 0.0 {
        push_issue(
            issues,
            point,
            ValidationSeverity::Error,
            format!("LCOE is negative: {}", point.lcoe),
        );
    }

    let eps = 1e-9;
    if !point.resources.solar && point.solar.abs() > eps {
        push_issue(
            issues,
            point,
            ValidationSeverity::Error,
            format!("disabled solar nonzero: {:.6}", point.solar),
        );
    }
    if !point.resources.wind && point.wind.abs() > eps {
        push_issue(
            issues,
            point,
            ValidationSeverity::Error,
            format!("disabled wind nonzero: {:.6}", point.wind),
        );
    }
    if !point.resources.storage && point.storage.abs() > eps {
        push_issue(
            issues,
            point,
            ValidationSeverity::Error,
            format!("disabled storage nonzero: {:.6}", point.storage),
        );
    }
    if !point.resources.clean_firm && point.clean_firm.abs() > eps {
        push_issue(
            issues,
            point,
            ValidationSeverity::Error,
            format!("disabled clean firm nonzero: {:.6}", point.clean_firm),
        );
    }

    if point.runtime_ms > point.runtime_budget_ms {
        push_issue(
            issues,
            point,
            ValidationSeverity::Warning,
            format!(
                "runtime {:.2}ms exceeds budget {:.2}ms",
                point.runtime_ms, point.runtime_budget_ms
            ),
        );
    }
}

fn push_issue(
    issues: &mut Vec<ValidationIssue>,
    point: &EvalPoint,
    severity: ValidationSeverity,
    message: String,
) {
    issues.push(ValidationIssue {
        severity,
        scenario_id: point.scenario_id.clone(),
        target: point.target,
        message,
    });
}

pub fn refresh_summary(report: &mut EvalReport) {
    let mut summary = RunSummary {
        scenario_count: report
            .points
            .iter()
            .map(|p| p.scenario_id.as_str())
            .collect::<std::collections::HashSet<_>>()
            .len(),
        point_count: report.points.len(),
        ..RunSummary::default()
    };

    let mut total_runtime = 0.0;
    for point in &report.points {
        match point.status {
            EvalStatus::Success => summary.success_count += 1,
            EvalStatus::OffTarget => summary.off_target_count += 1,
            EvalStatus::Error => summary.error_count += 1,
            EvalStatus::Infeasible => summary.infeasible_count += 1,
        }
        if let Some(deviation) = point.deviation_pct {
            summary.max_abs_deviation_pct = summary.max_abs_deviation_pct.max(deviation);
        }
        total_runtime += point.runtime_ms;
    }
    if summary.point_count > 0 {
        summary.mean_runtime_ms = total_runtime / summary.point_count as f64;
    }

    let issues = validate_report(report);
    summary.validation_error_count = issues
        .iter()
        .filter(|i| i.severity == ValidationSeverity::Error)
        .count();
    summary.validation_warning_count = issues
        .iter()
        .filter(|i| i.severity == ValidationSeverity::Warning)
        .count();
    report.summary = summary;
}

pub fn write_report(report: &EvalReport, out_dir: &Path) -> Result<PathBuf, String> {
    fs::create_dir_all(out_dir).map_err(|e| {
        format!(
            "Failed to create output directory {}: {}",
            out_dir.display(),
            e
        )
    })?;
    let path = out_dir.join("results.json");
    let raw = serde_json::to_string_pretty(report)
        .map_err(|e| format!("Failed to serialize report: {}", e))?;
    fs::write(&path, raw)
        .map_err(|e| format!("Failed to write report {}: {}", path.display(), e))?;
    Ok(path)
}

pub fn read_report(path: &Path) -> Result<EvalReport, String> {
    let raw = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read report {}: {}", path.display(), e))?;
    serde_json::from_str(&raw)
        .map_err(|e| format!("Failed to parse report {}: {}", path.display(), e))
}

fn validate_profiles(zone: &ZoneProfiles, scenario_id: &str) -> Result<(), String> {
    for (label, len) in [
        ("solar", zone.solar.len()),
        ("wind", zone.wind.len()),
        ("load", zone.load.len()),
    ] {
        if len != HOURS_PER_YEAR {
            return Err(format!(
                "Scenario {} zone {} profile has {} hours, expected {}",
                scenario_id, label, len, HOURS_PER_YEAR
            ));
        }
    }
    Ok(())
}

fn apply_overrides<T>(base: T, overrides: &Map<String, Value>) -> Result<T, String>
where
    T: Serialize + DeserializeOwned,
{
    let mut value = serde_json::to_value(base)
        .map_err(|e| format!("Failed to serialize defaults for override: {}", e))?;
    let object = value
        .as_object_mut()
        .ok_or_else(|| "Default configuration did not serialize to an object".to_string())?;

    for (key, override_value) in overrides {
        if !object.contains_key(key) {
            return Err(format!("Unknown override field: {}", key));
        }
        object.insert(key.clone(), override_value.clone());
    }

    serde_json::from_value(value).map_err(|e| format!("Failed to apply overrides: {}", e))
}

fn hash_json<T: Serialize>(value: &T) -> Result<u64, String> {
    let raw = serde_json::to_vec(value).map_err(|e| format!("Failed to hash JSON: {}", e))?;
    Ok(hash_bytes(&raw))
}

fn hash_bytes(bytes: &[u8]) -> u64 {
    let mut hasher = DefaultHasher::new();
    bytes.hash(&mut hasher);
    hasher.finish()
}

fn hash_profiles(zone: &ZoneProfiles) -> u64 {
    let mut hasher = DefaultHasher::new();
    for series in [&zone.solar, &zone.wind, &zone.load] {
        series.len().hash(&mut hasher);
        for value in series {
            value.to_le_bytes().hash(&mut hasher);
        }
    }
    hasher.finish()
}

fn now_unix_ms() -> Result<u128, String> {
    Ok(SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Clock error: {}", e))?
        .as_millis())
}

fn git_commit(root: &Path) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .arg("rev-parse")
        .arg("--short")
        .arg("HEAD")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
