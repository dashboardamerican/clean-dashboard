use crate::types::{BatteryMode, OptimizerConfig, OptimizerResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::env;

const FLOAT_TIE_EPS: f64 = 1e-9;
const DEFAULT_MONOTONIC_SCAN_LOCAL_RADIUS: usize = 2;
const DEFAULT_MONOTONIC_FULL_SCAN_THRESHOLD: usize = 5;
const DEFAULT_LOCAL_REFINEMENT_RADIUS: usize = 1;
const DEFAULT_LOCAL_REFINEMENT_TOP_K: usize = 1;
const DEFAULT_LOCAL_REFINEMENT_STEP_DIVISOR: f64 = 2.0;

#[derive(Clone, Copy, Debug)]
enum V3Profile {
    Default,
    Speed,
    Accuracy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V3SearchConfig {
    pub solar_step: f64,
    pub wind_step: f64,
    pub storage_step: f64,
    pub cf_step: f64,
    pub target_tolerance: f64,
    pub max_solar: f64,
    pub max_wind: f64,
    pub max_storage: f64,
    pub max_clean_firm: f64,
    pub parallel: bool,
    pub monotonic_scan_local_radius: usize,
    pub monotonic_full_scan_threshold: usize,
    pub local_refine_radius: usize,
    pub local_refine_top_k: usize,
    pub local_refine_step_divisor: f64,
}

impl Default for V3SearchConfig {
    fn default() -> Self {
        Self {
            solar_step: 25.0,
            wind_step: 20.0,
            storage_step: 25.0,
            cf_step: 5.0,
            target_tolerance: 0.5,
            max_solar: 100.0,
            max_wind: 400.0,
            max_storage: 100.0,
            max_clean_firm: 100.0,
            parallel: true,
            monotonic_scan_local_radius: DEFAULT_MONOTONIC_SCAN_LOCAL_RADIUS,
            monotonic_full_scan_threshold: DEFAULT_MONOTONIC_FULL_SCAN_THRESHOLD,
            local_refine_radius: 0,
            local_refine_top_k: 0,
            local_refine_step_divisor: 0.0,
        }
    }
}

impl V3SearchConfig {
    pub fn from_optimizer_config(config: &OptimizerConfig, target: f64) -> Self {
        let profile = parse_v3_profile();

        let (default_solar_points, default_wind_points, default_storage_points, default_cf_points) =
            match profile {
                V3Profile::Default => {
                    if target >= 95.0 {
                        (7usize, 7, 6, 12)
                    } else if target >= 85.0 {
                        (7usize, 7, 6, 10)
                    } else {
                        (6usize, 6, 5, 14)
                    }
                }
                V3Profile::Speed => {
                    if target >= 95.0 {
                        (5usize, 5, 4, 10)
                    } else if target >= 85.0 {
                        (5usize, 5, 4, 9)
                    } else {
                        (4usize, 4, 3, 12)
                    }
                }
                V3Profile::Accuracy => {
                    if target >= 95.0 {
                        (9usize, 9, 7, 14)
                    } else if target >= 85.0 {
                        (9usize, 9, 7, 12)
                    } else {
                        (8usize, 8, 6, 16)
                    }
                }
            };

        let (default_scan_radius, default_scan_threshold) = match profile {
            V3Profile::Default => (
                DEFAULT_MONOTONIC_SCAN_LOCAL_RADIUS,
                DEFAULT_MONOTONIC_FULL_SCAN_THRESHOLD,
            ),
            V3Profile::Speed => (1, 2),
            V3Profile::Accuracy => (3, 7),
        };

        let (default_refine_radius, default_refine_top_k, default_refine_step_divisor) =
            match profile {
                V3Profile::Default => (
                    DEFAULT_LOCAL_REFINEMENT_RADIUS,
                    DEFAULT_LOCAL_REFINEMENT_TOP_K,
                    DEFAULT_LOCAL_REFINEMENT_STEP_DIVISOR,
                ),
                V3Profile::Speed => (0, 0, DEFAULT_LOCAL_REFINEMENT_STEP_DIVISOR),
                V3Profile::Accuracy => (
                    DEFAULT_LOCAL_REFINEMENT_RADIUS + 1,
                    2,
                    DEFAULT_LOCAL_REFINEMENT_STEP_DIVISOR,
                ),
            };

    let scan_radius = parse_usize_env("V3_MONO_SCAN_RADIUS")
        .unwrap_or(default_scan_radius)
        .max(1);
        let mut scan_threshold = parse_usize_env("V3_MONO_FULL_SCAN_THRESHOLD")
            .unwrap_or(default_scan_threshold)
            .max(scan_radius);

        if scan_threshold < scan_radius {
            scan_threshold = scan_radius * 2;
        }

        let (mut solar_points, mut wind_points, mut storage_points, mut cf_points) =
            parse_step_override(profile).unwrap_or((
                default_solar_points,
                default_wind_points,
                default_storage_points,
                default_cf_points,
            ));

        if solar_points == 0 {
            solar_points = 1;
        }
        if wind_points == 0 {
            wind_points = 1;
        }
        if storage_points == 0 {
            storage_points = 1;
        }
        if cf_points == 0 {
            cf_points = 1;
        }

        let max_solar = if config.enable_solar {
            config.max_solar
        } else {
            0.0
        };
        let max_wind = if config.enable_wind {
            config.max_wind
        } else {
            0.0
        };
        let max_storage = if config.enable_storage {
            config.max_storage
        } else {
            0.0
        };
        let max_clean_firm = if config.enable_clean_firm {
            config.max_clean_firm
        } else {
            0.0
        };

        let derive_step = |max: f64, points: usize| {
            if max <= 0.0 {
                1.0
            } else {
                (max / points as f64).max(1.0)
            }
        };

        Self {
            solar_step: derive_step(max_solar, solar_points),
            wind_step: derive_step(max_wind, wind_points),
            storage_step: derive_step(max_storage, storage_points),
            cf_step: derive_step(max_clean_firm, cf_points),
            target_tolerance: 0.5,
            max_solar,
            max_wind,
            max_storage,
            max_clean_firm,
            parallel: true,
            monotonic_scan_local_radius: scan_radius,
            monotonic_full_scan_threshold: scan_threshold,
            local_refine_radius: parse_usize_env_allow_zero("V3_LOCAL_REFINEMENT_RADIUS")
                .unwrap_or(default_refine_radius),
            local_refine_top_k: parse_usize_env_allow_zero("V3_LOCAL_REFINEMENT_TOP_K")
                .unwrap_or(default_refine_top_k),
            local_refine_step_divisor: parse_f64_env("V3_LOCAL_REFINEMENT_STEP_DIVISOR")
                .unwrap_or(default_refine_step_divisor),
        }
    }
}

fn parse_usize_env_allow_zero(name: &str) -> Option<usize> {
    env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
}

fn parse_f64_env(name: &str) -> Option<f64> {
    env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<f64>().ok())
        .filter(|value| value.is_finite() && *value > 0.0)
}

fn parse_v3_profile() -> V3Profile {
    match env::var("V3_PROFILE")
        .ok()
        .as_deref()
        .unwrap_or("default")
        .trim()
        .to_lowercase()
        .as_str()
    {
        "speed" | "fast" => V3Profile::Speed,
        "accuracy" | "accurate" => V3Profile::Accuracy,
        _ => V3Profile::Default,
    }
}

fn parse_usize_env(name: &str) -> Option<usize> {
    env::var(name)
        .ok()
        .and_then(|raw| raw.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
}

fn parse_step_override(profile: V3Profile) -> Option<(usize, usize, usize, usize)> {
    let var_name = match profile {
        V3Profile::Default => "V3_POINTS",
        V3Profile::Speed => "V3_POINTS_SPEED",
        V3Profile::Accuracy => "V3_POINTS_ACCURACY",
    };

    let raw = env::var(var_name).ok()?;
    let pieces: Vec<_> = raw.split(',').map(str::trim).collect();
    if pieces.len() != 4 {
        return None;
    }

    let parse_piece = |idx: usize| pieces.get(idx)?.parse::<usize>().ok();
    let values = (
        parse_piece(0)?,
        parse_piece(1)?,
        parse_piece(2)?,
        parse_piece(3)?,
    );
    Some(values)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V3Diagnostics {
    pub solar_step: f64,
    pub wind_step: f64,
    pub storage_step: f64,
    pub cf_step: f64,
    pub target_tolerance: f64,
    pub max_solar: f64,
    pub max_wind: f64,
    pub max_storage: f64,
    pub max_clean_firm: f64,
    pub total_triples: u64,
    pub total_clean_evals: u64,
    pub total_lcoe_evals: u64,
    pub feasible_points_checked: u64,
    pub monotonic_fallback_count: u64,
    pub local_refine_candidates: u64,
    pub local_refine_candidates_evaluated: u64,
    pub local_refine_improved: bool,
    pub local_refine_radius: usize,
    pub local_refine_top_k: usize,
    pub local_refine_step_divisor: f64,
    pub certified: bool,
}

impl V3Diagnostics {
    pub fn from_config(config: &V3SearchConfig) -> Self {
        Self {
            solar_step: config.solar_step,
            wind_step: config.wind_step,
            storage_step: config.storage_step,
            cf_step: config.cf_step,
            target_tolerance: config.target_tolerance,
            max_solar: config.max_solar,
            max_wind: config.max_wind,
            max_storage: config.max_storage,
            max_clean_firm: config.max_clean_firm,
            total_triples: 0,
            total_clean_evals: 0,
            total_lcoe_evals: 0,
            feasible_points_checked: 0,
            monotonic_fallback_count: 0,
            local_refine_candidates: 0,
            local_refine_candidates_evaluated: 0,
            local_refine_improved: false,
            local_refine_radius: 0,
            local_refine_top_k: 0,
            local_refine_step_divisor: 0.0,
            certified: false,
        }
    }

    pub fn total_evaluations(&self) -> u64 {
        self.total_clean_evals + self.total_lcoe_evals
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct V3Result {
    pub result: OptimizerResult,
    pub diagnostics: V3Diagnostics,
}

#[derive(Clone, Debug)]
pub struct EvaluatedPoint {
    pub solar: f64,
    pub wind: f64,
    pub storage: f64,
    pub clean_firm: f64,
    pub clean_match: f64,
    pub lcoe: f64,
}

impl EvaluatedPoint {
    pub fn to_optimizer_result(
        &self,
        num_evaluations: u32,
        target: f64,
        tolerance: f64,
    ) -> OptimizerResult {
        OptimizerResult {
            solar_capacity: self.solar,
            wind_capacity: self.wind,
            storage_capacity: self.storage,
            clean_firm_capacity: self.clean_firm,
            achieved_clean_match: self.clean_match,
            lcoe: self.lcoe,
            num_evaluations,
            success: (self.clean_match - target).abs() <= tolerance,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CaseReport {
    pub case_name: String,
    pub target: f64,
    pub battery_mode: BatteryMode,
    pub tolerance: f64,
    pub v2_time_ms: Option<f64>,
    pub v3_time_ms: Option<f64>,
    pub runtime_ratio_v3_vs_v2: Option<f64>,
    pub v2_lcoe: Option<f64>,
    pub v3_lcoe: Option<f64>,
    pub v3_match: Option<f64>,
    pub v3_deviation: Option<f64>,
    pub oracle_coarse_time_ms: Option<f64>,
    pub oracle_coarse_lcoe: Option<f64>,
    pub oracle_fine_time_ms: Option<f64>,
    pub oracle_fine_lcoe: Option<f64>,
    pub v2_gap_vs_fine_pct: Option<f64>,
    pub v3_gap_vs_fine_pct: Option<f64>,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuiteGates {
    pub runtime_ratio_median: Option<f64>,
    pub runtime_ratio_limit: f64,
    pub p95_v3_gap_pct: Option<f64>,
    pub p95_gap_limit_pct: f64,
    pub max_v3_deviation: Option<f64>,
    pub deviation_limit: f64,
    pub runtime_pass: bool,
    pub gap_pass: bool,
    pub deviation_pass: bool,
    pub pass: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SuiteReport {
    pub suite: String,
    pub generated_unix_ms: u128,
    pub cases: Vec<CaseReport>,
    pub gates: SuiteGates,
    pub consecutive_quick_failures: Option<u32>,
    pub abandon_recommended: bool,
}

pub fn is_feasible(clean_match: f64, target: f64, tolerance: f64) -> bool {
    (clean_match - target).abs() <= tolerance
}

pub fn compare_points(a: &EvaluatedPoint, b: &EvaluatedPoint, target: f64) -> Ordering {
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

fn compare_f64(a: f64, b: f64) -> Ordering {
    if (a - b).abs() <= FLOAT_TIE_EPS {
        Ordering::Equal
    } else {
        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
    }
}

pub fn median(values: &[f64]) -> Option<f64> {
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

pub fn percentile(values: &[f64], pct: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));
    let clamped = pct.clamp(0.0, 100.0);
    let rank = ((clamped / 100.0) * (sorted.len() as f64 - 1.0)).round() as usize;
    sorted.get(rank).copied()
}
