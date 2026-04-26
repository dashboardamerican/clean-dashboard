use super::types::{
    compare_points, is_feasible, EvaluatedPoint, V3Diagnostics, V3Result, V3SearchConfig,
};
use std::cmp::Ordering;
use super::{build_axis, evaluate_clean_match, evaluate_point};
use crate::types::{BatteryMode, CostParams};
use std::collections::HashSet;

#[cfg(feature = "native")]
use rayon::prelude::*;

#[derive(Clone)]
struct SeedCandidate {
    triple: (f64, f64, f64),
    point: EvaluatedPoint,
}

pub fn run_v3_global_grid(
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    config: &V3SearchConfig,
) -> Result<V3Result, String> {
    let solar_values = build_axis(config.max_solar, config.solar_step)?;
    let wind_values = build_axis(config.max_wind, config.wind_step)?;
    let storage_values = build_axis(config.max_storage, config.storage_step)?;
    let cf_values = build_axis(config.max_clean_firm, config.cf_step)?;

    let mut triples =
        Vec::with_capacity(solar_values.len() * wind_values.len() * storage_values.len());
    for &solar in &solar_values {
        for &wind in &wind_values {
            for &storage in &storage_values {
                triples.push((solar, wind, storage));
            }
        }
    }

    let outcomes = evaluate_triples(
        &triples,
        &cf_values,
        target,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        battery_mode,
        config,
    )?;

    let mut outcomes = outcomes;
    let mut seed_candidates: Vec<SeedCandidate> = outcomes
        .iter()
        .filter_map(|outcome| {
            outcome.best.as_ref().map(|point| SeedCandidate {
                triple: outcome.triple,
                point: point.clone(),
            })
        })
        .collect();

    let mut best: Option<EvaluatedPoint> = None;
    let mut diagnostics = V3Diagnostics::from_config(config);
    diagnostics.total_triples = triples.len() as u64;
    diagnostics.local_refine_radius = config.local_refine_radius;
    diagnostics.local_refine_top_k = config.local_refine_top_k;
    diagnostics.local_refine_step_divisor = config.local_refine_step_divisor;

    for outcome in outcomes {
        diagnostics.total_clean_evals += outcome.clean_evals;
        diagnostics.total_lcoe_evals += outcome.lcoe_evals;
        diagnostics.feasible_points_checked += outcome.feasible_points;
        if outcome.used_fallback {
            diagnostics.monotonic_fallback_count += 1;
        }

        if let Some(candidate) = outcome.best {
            if best
                .as_ref()
                .map(|current| compare_points(&candidate, current, target).is_lt())
                .unwrap_or(true)
            {
                best = Some(candidate);
            }
        }
    }

    if !seed_candidates.is_empty()
        && config.local_refine_radius > 0
        && config.local_refine_top_k > 0
        && config.local_refine_step_divisor > 1.0
    {
        seed_candidates.sort_by(|a, b| {
            let a_deviation = (a.point.clean_match - target).abs();
            let b_deviation = (b.point.clean_match - target).abs();

            a_deviation
                .partial_cmp(&b_deviation)
                .unwrap_or(Ordering::Equal)
                .then_with(|| compare_points(&a.point, &b.point, target))
        });
        seed_candidates.truncate(config.local_refine_top_k.min(seed_candidates.len()));

        diagnostics.local_refine_candidates = seed_candidates.len() as u64;

        let mut refine_triples = Vec::new();
        let mut seen = HashSet::with_capacity(triples.len() + seed_candidates.len() * 64);
        for triple in triples.iter().copied() {
            seen.insert(triple_key(triple));
        }

        for seed in &seed_candidates {
            for triple in build_refine_triples(seed.triple, config) {
                if seen.insert(triple_key(triple)) {
                    refine_triples.push(triple);
                }
            }
        }

        if !refine_triples.is_empty() {
            diagnostics.local_refine_candidates_evaluated = refine_triples.len() as u64;

            let local_outcomes = evaluate_triples(
                &refine_triples,
                &cf_values,
                target,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                battery_mode,
                config,
            )?;

            diagnostics.total_triples += refine_triples.len() as u64;
            for outcome in local_outcomes {
                diagnostics.total_clean_evals += outcome.clean_evals;
                diagnostics.total_lcoe_evals += outcome.lcoe_evals;
                diagnostics.feasible_points_checked += outcome.feasible_points;
                if outcome.used_fallback {
                    diagnostics.monotonic_fallback_count += 1;
                }

                if let Some(candidate) = outcome.best {
                    if best
                        .as_ref()
                        .map(|current| compare_points(&candidate, current, target).is_lt())
                        .unwrap_or(true)
                    {
                        diagnostics.local_refine_improved = true;
                        best = Some(candidate);
                    }
                }
            }
        }
    }

    diagnostics.certified = true;

    let Some(best_point) = best else {
        return Err(format!(
            "No feasible portfolio found for target {} with tolerance {}",
            target, config.target_tolerance
        ));
    };

    let result = best_point.to_optimizer_result(
        diagnostics.total_evaluations().min(u32::MAX as u64) as u32,
        target,
        config.target_tolerance,
    );

    Ok(V3Result {
        result,
        diagnostics,
    })
}

struct TripleOutcome {
    triple: (f64, f64, f64),
    best: Option<EvaluatedPoint>,
    clean_evals: u64,
    lcoe_evals: u64,
    feasible_points: u64,
    used_fallback: bool,
}

#[allow(clippy::too_many_arguments)]
fn evaluate_triples(
    triples: &[(f64, f64, f64)],
    cf_values: &[f64],
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    config: &V3SearchConfig,
) -> Result<Vec<TripleOutcome>, String> {
    #[cfg(feature = "native")]
    {
        if config.parallel {
            return triples
                .par_iter()
                .map(|&(solar, wind, storage)| {
                    evaluate_single_triple(
                        solar,
                        wind,
                        storage,
                        cf_values,
                        target,
                        solar_profile,
                        wind_profile,
                        load_profile,
                        costs,
                        battery_mode,
                        config.target_tolerance,
                        config,
                    )
                })
                .collect::<Result<Vec<_>, _>>();
        }
    }

    let mut outcomes = Vec::with_capacity(triples.len());
    for &(solar, wind, storage) in triples {
        outcomes.push(evaluate_single_triple(
            solar,
                wind,
                storage,
                cf_values,
            target,
            solar_profile,
                wind_profile,
                load_profile,
                costs,
                battery_mode,
                config.target_tolerance,
                config,
            )?);
    }

    Ok(outcomes)
}

#[allow(clippy::too_many_arguments)]
fn evaluate_single_triple(
    solar: f64,
    wind: f64,
    storage: f64,
    cf_values: &[f64],
    target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    tolerance: f64,
    config: &V3SearchConfig,
) -> Result<TripleOutcome, String> {
    let n_cf = cf_values.len();
    if n_cf == 0 {
        return Ok(TripleOutcome {
            triple: (solar, wind, storage),
            best: None,
            clean_evals: 0,
            lcoe_evals: 0,
            feasible_points: 0,
            used_fallback: false,
        });
    }

    let mut clean_cache = vec![None; n_cf];
    let mut point_cache = vec![None; n_cf];
    let mut clean_evals = 0u64;
    let mut lcoe_evals = 0u64;
    let mut feasible_points = 0u64;

    let lower_threshold = target - tolerance;
    let upper_threshold = target + tolerance;

    let lower_idx = first_index_geq(
        cf_values,
        lower_threshold,
        solar,
        wind,
        storage,
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
        &mut clean_cache,
        &mut clean_evals,
    )?;

    let upper_idx = last_index_leq(
        cf_values,
        upper_threshold,
        solar,
        wind,
        storage,
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
        &mut clean_cache,
        &mut clean_evals,
    )?;

    let target_idx = first_index_geq(
        cf_values,
        target,
        solar,
        wind,
        storage,
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
        &mut clean_cache,
        &mut clean_evals,
    )?;

    let monotonic = validate_monotonic_non_decreasing(
        cf_values,
        solar,
        wind,
        storage,
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
        &mut clean_cache,
        &mut clean_evals,
    )?;

    let mut best: Option<EvaluatedPoint> = None;
    let mut used_fallback = false;

    if monotonic {
        if let (Some(lo), Some(hi)) = (lower_idx, upper_idx) {
            if lo <= hi {
                let mut scan_indices = Vec::new();

                if hi - lo <= config.monotonic_full_scan_threshold {
                    for idx in lo..=hi {
                        scan_indices.push(idx);
                    }
                } else {
                    scan_indices.push(lo);
                    scan_indices.push(hi);

                    let center_idx = target_idx.unwrap_or(hi.min(lo));
                    let start = center_idx.saturating_sub(config.monotonic_scan_local_radius);
                    let end = (center_idx + config.monotonic_scan_local_radius).min(hi);
                    let end = end.max(lo);

                    for idx in start..=end {
                        scan_indices.push(idx);
                    }
                }

                scan_indices.sort_unstable();
                scan_indices.dedup();

                for idx in scan_indices {
                    let point = get_point(
                        idx,
                        cf_values,
                        solar,
                        wind,
                        storage,
                        solar_profile,
                        wind_profile,
                        load_profile,
                        costs,
                        battery_mode,
                        &mut point_cache,
                        &mut lcoe_evals,
                    )?;
                    feasible_points += 1;
                    if best
                        .as_ref()
                        .map(|current| compare_points(&point, current, target).is_lt())
                        .unwrap_or(true)
                    {
                        best = Some(point);
                    }
                }
            }
        }
    } else {
        used_fallback = true;
        for idx in 0..n_cf {
            let clean = get_clean_match(
                idx,
                cf_values,
                solar,
                wind,
                storage,
                solar_profile,
                wind_profile,
                load_profile,
                battery_mode,
                &mut clean_cache,
                &mut clean_evals,
            )?;

            if is_feasible(clean, target, tolerance) {
                let point = get_point(
                    idx,
                    cf_values,
                    solar,
                    wind,
                    storage,
                    solar_profile,
                    wind_profile,
                    load_profile,
                    costs,
                    battery_mode,
                    &mut point_cache,
                    &mut lcoe_evals,
                )?;
                feasible_points += 1;
                if best
                    .as_ref()
                    .map(|current| compare_points(&point, current, target).is_lt())
                    .unwrap_or(true)
                {
                    best = Some(point);
                }
            }
        }
    }

    Ok(TripleOutcome {
        triple: (solar, wind, storage),
        best,
        clean_evals,
        lcoe_evals,
        feasible_points,
        used_fallback,
    })
}

fn build_refine_triples(
    seed: (f64, f64, f64),
    config: &V3SearchConfig,
) -> Vec<(f64, f64, f64)> {
    if config.local_refine_radius == 0 || config.local_refine_step_divisor <= 1.0 {
        return Vec::new();
    }

    let radius = config.local_refine_radius as i32;
    let solar_step = config.solar_step / config.local_refine_step_divisor;
    let wind_step = config.wind_step / config.local_refine_step_divisor;
    let storage_step = config.storage_step / config.local_refine_step_divisor;

    let min_solar = 0.0_f64;
    let max_solar = config.max_solar;
    let min_wind = 0.0_f64;
    let max_wind = config.max_wind;
    let min_storage = 0.0_f64;
    let max_storage = config.max_storage;

    let mut triples = Vec::with_capacity(
        usize::try_from(radius).map(|r| (2 * r + 1).pow(3)).unwrap_or(0),
    );

    for ds in -radius..=radius {
        for dw in -radius..=radius {
            for dcf in -radius..=radius {
                let solar = (seed.0 + ds as f64 * solar_step).clamp(min_solar, max_solar);
                let wind = (seed.1 + dw as f64 * wind_step).clamp(min_wind, max_wind);
                let storage = (seed.2 + dcf as f64 * storage_step).clamp(min_storage, max_storage);
                triples.push((
                    round_capacity(solar),
                    round_capacity(wind),
                    round_capacity(storage),
                ));
            }
        }
    }

    triples
}

fn triple_key(triple: (f64, f64, f64)) -> (u64, u64, u64) {
    (
        round_capacity(triple.0).to_bits(),
        round_capacity(triple.1).to_bits(),
        round_capacity(triple.2).to_bits(),
    )
}

fn round_capacity(v: f64) -> f64 {
    (v * 1_000_000.0).round() / 1_000_000.0
}

#[allow(clippy::too_many_arguments)]
fn get_clean_match(
    idx: usize,
    cf_values: &[f64],
    solar: f64,
    wind: f64,
    storage: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    battery_mode: BatteryMode,
    clean_cache: &mut [Option<f64>],
    clean_evals: &mut u64,
) -> Result<f64, String> {
    if let Some(cached) = clean_cache[idx] {
        return Ok(cached);
    }

    let clean = evaluate_clean_match(
        solar,
        wind,
        storage,
        cf_values[idx],
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
    )?;
    clean_cache[idx] = Some(clean);
    *clean_evals += 1;
    Ok(clean)
}

#[allow(clippy::too_many_arguments)]
fn get_point(
    idx: usize,
    cf_values: &[f64],
    solar: f64,
    wind: f64,
    storage: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    point_cache: &mut [Option<EvaluatedPoint>],
    lcoe_evals: &mut u64,
) -> Result<EvaluatedPoint, String> {
    if let Some(cached) = &point_cache[idx] {
        return Ok(cached.clone());
    }

    let point = evaluate_point(
        solar,
        wind,
        storage,
        cf_values[idx],
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        battery_mode,
    )?;
    point_cache[idx] = Some(point.clone());
    *lcoe_evals += 1;
    Ok(point)
}

#[allow(clippy::too_many_arguments)]
fn first_index_geq(
    cf_values: &[f64],
    threshold: f64,
    solar: f64,
    wind: f64,
    storage: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    battery_mode: BatteryMode,
    clean_cache: &mut [Option<f64>],
    clean_evals: &mut u64,
) -> Result<Option<usize>, String> {
    let n = cf_values.len();
    let last = get_clean_match(
        n - 1,
        cf_values,
        solar,
        wind,
        storage,
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
        clean_cache,
        clean_evals,
    )?;
    if last < threshold {
        return Ok(None);
    }

    let mut lo = 0usize;
    let mut hi = n - 1;
    while lo < hi {
        let mid = (lo + hi) / 2;
        let value = get_clean_match(
            mid,
            cf_values,
            solar,
            wind,
            storage,
            solar_profile,
            wind_profile,
            load_profile,
            battery_mode,
            clean_cache,
            clean_evals,
        )?;
        if value >= threshold {
            hi = mid;
        } else {
            lo = mid + 1;
        }
    }

    Ok(Some(lo))
}

#[allow(clippy::too_many_arguments)]
fn last_index_leq(
    cf_values: &[f64],
    threshold: f64,
    solar: f64,
    wind: f64,
    storage: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    battery_mode: BatteryMode,
    clean_cache: &mut [Option<f64>],
    clean_evals: &mut u64,
) -> Result<Option<usize>, String> {
    let first = get_clean_match(
        0,
        cf_values,
        solar,
        wind,
        storage,
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
        clean_cache,
        clean_evals,
    )?;
    if first > threshold {
        return Ok(None);
    }

    let mut lo = 0usize;
    let mut hi = cf_values.len() - 1;
    while lo < hi {
        let mid = (lo + hi + 1) / 2;
        let value = get_clean_match(
            mid,
            cf_values,
            solar,
            wind,
            storage,
            solar_profile,
            wind_profile,
            load_profile,
            battery_mode,
            clean_cache,
            clean_evals,
        )?;
        if value <= threshold {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }

    Ok(Some(lo))
}

#[allow(clippy::too_many_arguments)]
fn validate_monotonic_non_decreasing(
    cf_values: &[f64],
    solar: f64,
    wind: f64,
    storage: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    battery_mode: BatteryMode,
    clean_cache: &mut [Option<f64>],
    clean_evals: &mut u64,
) -> Result<bool, String> {
    if cf_values.len() <= 1 {
        return Ok(true);
    }

    let mut previous = get_clean_match(
        0,
        cf_values,
        solar,
        wind,
        storage,
        solar_profile,
        wind_profile,
        load_profile,
        battery_mode,
        clean_cache,
        clean_evals,
    )?;

    for idx in 1..cf_values.len() {
        let current = get_clean_match(
            idx,
            cf_values,
            solar,
            wind,
            storage,
            solar_profile,
            wind_profile,
            load_profile,
            battery_mode,
            clean_cache,
            clean_evals,
        )?;
        if current + 1e-9 < previous {
            return Ok(false);
        }
        previous = current;
    }

    Ok(true)
}
