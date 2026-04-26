use super::global_grid::run_v3_global_grid;
use super::oracle::run_v3_oracle;
use super::types::{median, percentile, CaseReport, SuiteGates, SuiteReport, V3SearchConfig};
use crate::optimizer::v2_hierarchical::run_v2_optimizer;
use crate::types::{BatteryMode, CostParams, OptimizerConfig, HOURS_PER_YEAR};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const ZONES_PATH: &str = "../data/zones.json";
const RUNTIME_RATIO_LIMIT: f64 = 3.0;
const P95_GAP_LIMIT_PCT: f64 = 0.5;

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
    v3_config: V3SearchConfig,
    oracle_coarse_config: V3SearchConfig,
    oracle_fine_config: Option<V3SearchConfig>,
}

#[derive(Debug, Deserialize)]
struct ZoneProfiles {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
struct QuickFailState {
    consecutive_failures: u32,
}

pub fn run_suite(suite: &str) -> Result<SuiteReport, String> {
    let scenarios = build_suite(suite)?;
    let mut cases = Vec::with_capacity(scenarios.len());

    for scenario in &scenarios {
        cases.push(execute_case(scenario));
    }

    let gates = compute_gates(&cases);
    let generated_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Clock error: {}", e))?
        .as_millis();

    Ok(SuiteReport {
        suite: suite.to_string(),
        generated_unix_ms,
        cases,
        gates,
        consecutive_quick_failures: None,
        abandon_recommended: false,
    })
}

pub fn write_suite_report(report: &SuiteReport, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create report directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("Failed to serialize suite report: {}", e))?;
    fs::write(path, json)
        .map_err(|e| format!("Failed to write suite report {}: {}", path.display(), e))
}

pub fn apply_quick_fail_state(report: &mut SuiteReport, state_path: &Path) -> Result<(), String> {
    if report.suite != "quick" {
        return Ok(());
    }

    let mut state = if state_path.exists() {
        let raw = fs::read_to_string(state_path)
            .map_err(|e| format!("Failed to read fail state {}: {}", state_path.display(), e))?;
        serde_json::from_str::<QuickFailState>(&raw)
            .map_err(|e| format!("Failed to parse fail state {}: {}", state_path.display(), e))?
    } else {
        QuickFailState {
            consecutive_failures: 0,
        }
    };

    if report.gates.pass {
        state.consecutive_failures = 0;
    } else {
        state.consecutive_failures += 1;
    }

    if let Some(parent) = state_path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create fail-state directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }

    let raw = serde_json::to_string_pretty(&state)
        .map_err(|e| format!("Failed to serialize fail-state JSON: {}", e))?;
    fs::write(state_path, raw)
        .map_err(|e| format!("Failed to write fail state {}: {}", state_path.display(), e))?;

    report.consecutive_quick_failures = Some(state.consecutive_failures);
    report.abandon_recommended = !report.gates.pass && state.consecutive_failures >= 2;

    Ok(())
}

pub fn load_real_zone(zone_name: &str) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), String> {
    let zones = load_zone_map()?;
    let profiles = get_zone_profiles(&zones, zone_name)?;
    Ok((profiles.solar, profiles.wind, profiles.load))
}

fn execute_case(scenario: &ScenarioDefinition) -> CaseReport {
    let mut report = CaseReport {
        case_name: scenario.case_name.clone(),
        target: scenario.target,
        battery_mode: scenario.battery_mode,
        tolerance: scenario.tolerance,
        v2_time_ms: None,
        v3_time_ms: None,
        runtime_ratio_v3_vs_v2: None,
        v2_lcoe: None,
        v3_lcoe: None,
        v3_match: None,
        v3_deviation: None,
        oracle_coarse_time_ms: None,
        oracle_coarse_lcoe: None,
        oracle_fine_time_ms: None,
        oracle_fine_lcoe: None,
        v2_gap_vs_fine_pct: None,
        v3_gap_vs_fine_pct: None,
        success: true,
        error: None,
    };

    let v2_result = {
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
        report.v2_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
        match result {
            Ok(v) => {
                report.v2_lcoe = Some(v.lcoe);
                Some(v)
            }
            Err(e) => {
                report.success = false;
                report.error = Some(format!("V2 failed: {}", e));
                None
            }
        }
    };

    let v3_result = {
        let start = Instant::now();
        let result = run_v3_global_grid(
            scenario.target,
            &scenario.profiles.solar,
            &scenario.profiles.wind,
            &scenario.profiles.load,
            &scenario.costs,
            scenario.battery_mode,
            &scenario.v3_config,
        );
        report.v3_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
        match result {
            Ok(v) => {
                report.v3_lcoe = Some(v.result.lcoe);
                report.v3_match = Some(v.result.achieved_clean_match);
                report.v3_deviation = Some((v.result.achieved_clean_match - scenario.target).abs());
                Some(v)
            }
            Err(e) => {
                report.success = false;
                report.error = Some(append_error(
                    report.error.take(),
                    format!("V3 failed: {}", e),
                ));
                None
            }
        }
    };

    {
        let start = Instant::now();
        let result = run_v3_oracle(
            scenario.target,
            &scenario.profiles.solar,
            &scenario.profiles.wind,
            &scenario.profiles.load,
            &scenario.costs,
            scenario.battery_mode,
            &scenario.oracle_coarse_config,
        );
        report.oracle_coarse_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);
        match result {
            Ok(v) => {
                report.oracle_coarse_lcoe = Some(v.result.lcoe);
            }
            Err(e) => {
                report.success = false;
                report.error = Some(append_error(
                    report.error.take(),
                    format!("Oracle coarse failed: {}", e),
                ));
            }
        }
    }

    if let Some(fine) = &scenario.oracle_fine_config {
        let start = Instant::now();
        let result = run_v3_oracle(
            scenario.target,
            &scenario.profiles.solar,
            &scenario.profiles.wind,
            &scenario.profiles.load,
            &scenario.costs,
            scenario.battery_mode,
            fine,
        );
        report.oracle_fine_time_ms = Some(start.elapsed().as_secs_f64() * 1000.0);

        match result {
            Ok(v) => {
                report.oracle_fine_lcoe = Some(v.result.lcoe);
            }
            Err(e) => {
                report.success = false;
                report.error = Some(append_error(
                    report.error.take(),
                    format!("Oracle fine failed: {}", e),
                ));
            }
        }
    }

    if let (Some(v2_ms), Some(v3_ms)) = (report.v2_time_ms, report.v3_time_ms) {
        if v2_ms > 0.0 {
            report.runtime_ratio_v3_vs_v2 = Some(v3_ms / v2_ms);
        }
    }

    if let Some(fine_lcoe) = report.oracle_fine_lcoe {
        if fine_lcoe.abs() > 0.0 {
            if let Some(v2_lcoe) = report.v2_lcoe {
                report.v2_gap_vs_fine_pct =
                    Some(((v2_lcoe - fine_lcoe).abs() / fine_lcoe.abs()) * 100.0);
            }
            if let Some(v3_lcoe) = report.v3_lcoe {
                report.v3_gap_vs_fine_pct =
                    Some(((v3_lcoe - fine_lcoe).abs() / fine_lcoe.abs()) * 100.0);
            }
        }
    }

    if let Some(v3_dev) = report.v3_deviation {
        if v3_dev > scenario.tolerance + 1e-6 {
            report.success = false;
            report.error = Some(append_error(
                report.error.take(),
                format!(
                    "V3 deviation {:.4} exceeded tolerance {:.4}",
                    v3_dev, scenario.tolerance
                ),
            ));
        }
    }

    if v2_result.is_none() || v3_result.is_none() {
        report.success = false;
    }

    report
}

fn compute_gates(cases: &[CaseReport]) -> SuiteGates {
    let runtime_values: Vec<f64> = cases
        .iter()
        .filter_map(|c| c.runtime_ratio_v3_vs_v2)
        .collect();

    let gap_values: Vec<f64> = cases.iter().filter_map(|c| c.v3_gap_vs_fine_pct).collect();

    let deviation_values: Vec<f64> = cases.iter().filter_map(|c| c.v3_deviation).collect();

    let max_tolerance = cases.iter().map(|c| c.tolerance).fold(0.0_f64, f64::max) + 1e-6;

    let runtime_ratio_median = median(&runtime_values);
    let p95_v3_gap_pct = percentile(&gap_values, 95.0);
    let max_v3_deviation = if deviation_values.is_empty() {
        None
    } else {
        Some(
            deviation_values
                .iter()
                .copied()
                .fold(f64::NEG_INFINITY, f64::max),
        )
    };

    let runtime_pass = runtime_ratio_median
        .map(|v| v <= RUNTIME_RATIO_LIMIT)
        .unwrap_or(false);

    let gap_pass = if gap_values.is_empty() {
        true
    } else {
        p95_v3_gap_pct
            .map(|v| v <= P95_GAP_LIMIT_PCT)
            .unwrap_or(false)
    };

    let deviation_pass = max_v3_deviation
        .map(|v| v <= max_tolerance)
        .unwrap_or(false);

    SuiteGates {
        runtime_ratio_median,
        runtime_ratio_limit: RUNTIME_RATIO_LIMIT,
        p95_v3_gap_pct,
        p95_gap_limit_pct: P95_GAP_LIMIT_PCT,
        max_v3_deviation,
        deviation_limit: max_tolerance,
        runtime_pass,
        gap_pass,
        deviation_pass,
        pass: runtime_pass && gap_pass && deviation_pass,
    }
}

fn build_suite(suite: &str) -> Result<Vec<ScenarioDefinition>, String> {
    let zones = load_zone_map()?;

    let synthetic = synthetic_profiles();
    let california = get_zone_profiles(&zones, "california")?;
    let texas = get_zone_profiles(&zones, "texas")?;
    let florida = get_zone_profiles(&zones, "florida")?;
    let southeast = get_zone_profiles(&zones, "southeast")?;
    let northeast = get_zone_profiles(&zones, "mid-atlantic")?;
    let new_york = get_zone_profiles(&zones, "new york")?;

    let default_costs = CostParams::default_costs();
    let mut expensive_gas_costs = CostParams::default_costs();
    expensive_gas_costs.gas_price *= 3.0;

    let base_config = OptimizerConfig {
        target_clean_match: 80.0,
        enable_solar: true,
        enable_wind: true,
        enable_storage: true,
        enable_clean_firm: true,
        max_solar: 100.0,
        max_wind: 400.0,
        max_storage: 100.0,
        max_clean_firm: 100.0,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
    };

    let mk_v3_search = |target: f64, optimizer_config: &OptimizerConfig, radius: usize| {
        let mut config = V3SearchConfig::from_optimizer_config(optimizer_config, target);
        config.monotonic_scan_local_radius = radius;
        config
    };

    let fine_cfg = {
        let mut cfg = mk_v3_search(95.0, &base_config, 3);
        cfg.solar_step = 10.0;
        cfg.wind_step = 10.0;
        cfg.storage_step = 10.0;
        cfg.cf_step = 5.0;
        cfg
    };

    let mk_case = |name: &str,
                   target: f64,
                   costs: &CostParams,
                   profiles: &Profiles,
                   battery_mode: BatteryMode,
                   optimizer_config: OptimizerConfig,
                   v3_config: V3SearchConfig,
                   oracle_fine_config: Option<V3SearchConfig>| {
        ScenarioDefinition {
            case_name: name.to_string(),
            target,
            tolerance: v3_config.target_tolerance,
            battery_mode,
            costs: costs.clone(),
            profiles: profiles.clone(),
            optimizer_config,
            v3_config: v3_config.clone(),
            oracle_coarse_config: v3_config.clone(),
            oracle_fine_config,
        }
    };

    let mk_default_case = |name: &str,
                          target: f64,
                          costs: &CostParams,
                          profiles: &Profiles,
                          use_fine_oracle: bool| {
        let optimizer_config = config_from_search(target, &base_config);
        let use_fine_oracle = if use_fine_oracle {
            Some(fine_cfg.clone())
        } else {
            None
        };
        let v3_config = mk_v3_search(target, &optimizer_config, 2);
        mk_case(
            name,
            target,
            costs,
            profiles,
            BatteryMode::Hybrid,
            optimizer_config,
            v3_config.clone(),
            use_fine_oracle,
        )
    };

    let quick = vec![
        mk_default_case(
            "synthetic_target_70_default",
            70.0,
            &default_costs,
            &synthetic,
            false,
        ),
        mk_default_case(
            "synthetic_target_95_default",
            95.0,
            &default_costs,
            &synthetic,
            false,
        ),
        mk_default_case(
            "synthetic_target_99_default",
            99.0,
            &default_costs,
            &synthetic,
            true,
        ),
        mk_default_case(
            "california_target_70_default",
            70.0,
            &default_costs,
            &california,
            false,
        ),
        mk_default_case(
            "california_target_95_default",
            95.0,
            &default_costs,
            &california,
            true,
        ),
        mk_default_case(
            "california_target_99_default",
            99.0,
            &default_costs,
            &california,
            true,
        ),
        mk_default_case(
            "texas_target_95_default",
            95.0,
            &default_costs,
            &texas,
            false,
        ),
        mk_default_case(
            "texas_target_99_expensive_gas",
            99.0,
            &expensive_gas_costs,
            &texas,
            true,
        ),
    ];

    let smoke = vec![
        mk_default_case(
            "smoke_synthetic_95",
            95.0,
            &default_costs,
            &synthetic,
            false,
        ),
        mk_default_case(
            "smoke_california_95",
            95.0,
            &default_costs,
            &california,
            false,
        ),
        mk_default_case("smoke_texas_95", 95.0, &default_costs, &texas, false),
    ];

    let mut standard = quick.clone();
    standard.push(mk_default_case(
        "standard_california_85_default",
        85.0,
        &default_costs,
        &california,
        false,
    ));
    standard.push(mk_default_case(
        "standard_texas_85_default",
        85.0,
        &default_costs,
        &texas,
        false,
    ));
    standard.push(mk_default_case(
        "standard_synthetic_90_expensive_gas",
        90.0,
        &expensive_gas_costs,
        &synthetic,
        false,
    ));
    standard.push(mk_default_case(
        "standard_texas_99_default",
        99.0,
        &default_costs,
        &texas,
        false,
    ));

    let hardest = vec![
        mk_default_case("hardest_synthetic_target_30", 30.0, &default_costs, &synthetic, false),
        mk_default_case("hardest_synthetic_target_40", 40.0, &default_costs, &synthetic, false),
        mk_default_case("hardest_synthetic_target_70", 70.0, &default_costs, &synthetic, true),
        mk_default_case("hardest_california_target_80", 80.0, &default_costs, &california, false),
        mk_default_case("hardest_southeast_target_95", 95.0, &default_costs, &southeast, true),
        mk_default_case("hardest_texas_target_30_expensive_gas", 30.0, &expensive_gas_costs, &texas, false),
        mk_default_case("hardest_florida_target_90", 90.0, &default_costs, &florida, false),
        mk_default_case("hardest_newyork_target_95", 95.0, &default_costs, &new_york, false),
    ];

    let mut comprehensive = quick.clone();
    let zone_mix = [
        ("standard_northeast_85", &northeast, 85.0, false),
        ("standard_northeast_95", &northeast, 95.0, true),
        ("standard_florida_99", &florida, 99.0, true),
        ("standard_delta_95", &get_zone_profiles(&zones, "delta")?, 95.0, false),
        ("standard_mountain_95", &get_zone_profiles(&zones, "mountain")?, 95.0, false),
        ("standard_northwest_99", &get_zone_profiles(&zones, "northwest")?, 99.0, false),
        ("standard_midwest_90", &get_zone_profiles(&zones, "midwest")?, 90.0, false),
        ("standard_plains_90", &get_zone_profiles(&zones, "plains")?, 90.0, false),
        ("standard_plains_95", &get_zone_profiles(&zones, "plains")?, 95.0, false),
        ("standard_mid_atlantic_90", &northeast, 90.0, false),
    ];

    comprehensive.extend(standard.clone());
    for (name, profile, target, use_fine_oracle) in zone_mix {
        comprehensive.push(mk_default_case(name, target, &default_costs, profile, use_fine_oracle));
    }
    comprehensive.push(mk_default_case(
        "comprehensive_synthetic_85",
        85.0,
        &expensive_gas_costs,
        &synthetic,
        true,
    ));
    comprehensive.push(mk_case(
        "comprehensive_california_lowcf_95_no_cf",
        95.0,
        &default_costs,
        &california,
        BatteryMode::Hybrid,
        {
            let mut cfg = config_from_search(95.0, &base_config);
            cfg.enable_clean_firm = false;
            cfg
        },
        {
            let optimizer_config = config_from_search(95.0, &base_config);
            mk_v3_search(95.0, &optimizer_config, 2)
        },
        None,
    ));

    match suite {
        "smoke" => Ok(smoke),
        "quick" => Ok(quick),
        "standard" => Ok(standard),
        "hardest" => Ok(hardest),
        "comprehensive" => Ok(comprehensive),
        other => Err(format!(
            "Unknown suite '{}'. Expected one of: smoke, quick, standard, hardest, comprehensive",
            other
        )),
    }
}

fn config_from_search(target: f64, search: &OptimizerConfig) -> OptimizerConfig {
    let mut config = search.clone();
    config.target_clean_match = target;
    config
}

fn synthetic_profiles() -> Profiles {
    let mut solar = Vec::with_capacity(HOURS_PER_YEAR);
    let mut wind = Vec::with_capacity(HOURS_PER_YEAR);
    let mut load = Vec::with_capacity(HOURS_PER_YEAR);

    for h in 0..HOURS_PER_YEAR {
        let hod = h % 24;
        let day = h / 24;
        let seasonal = 0.85 + 0.15 * (2.0 * std::f64::consts::PI * day as f64 / 365.0).sin();

        let solar_cf = if (6..=18).contains(&hod) {
            let peak_factor = 1.0 - ((hod as f64 - 12.0).abs() / 6.0);
            0.30 * peak_factor * seasonal.max(0.2)
        } else {
            0.0
        };
        solar.push(solar_cf.max(0.0));

        let base_wind = if hod < 6 || hod > 20 { 0.40 } else { 0.30 };
        let wind_seasonal = 0.95 + 0.10 * (2.0 * std::f64::consts::PI * day as f64 / 365.0).cos();
        wind.push((base_wind * wind_seasonal).max(0.05));

        let load_shape = if (17..=22).contains(&hod) {
            110.0
        } else if (0..=5).contains(&hod) {
            90.0
        } else {
            100.0
        };
        load.push(load_shape);
    }

    normalize_load_to_100mw(&mut load);

    Profiles { solar, wind, load }
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
            "Zone '{}' has invalid profile lengths: solar={}, wind={}, load={}, expected {}",
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

fn append_error(existing: Option<String>, addition: String) -> String {
    match existing {
        Some(prev) => format!("{} | {}", prev, addition),
        None => addition,
    }
}
