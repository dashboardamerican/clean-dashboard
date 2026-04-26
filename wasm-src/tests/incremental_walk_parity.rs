//! Integration tests for the Rust port of `run_incremental_cost_walk`.
//!
//! These tests use the real zone profiles from a `zones.json` fixture and
//! verify:
//! - The walk trace is well-formed (baseline at index 0, monotonic capacities,
//!   no NaN/inf, length > 1).
//! - The achieved match is within 1.0 percentage point of the requested target.
//! - The final LCOE is positive and within a reasonable range.
//! - Disabled resources stay at 0 capacity.
//! - Two identical runs produce identical walk traces (determinism).
//!
//! Run with: `cargo test --release --test incremental_walk_parity`

use energy_simulator::optimizer::run_incremental_walk;
use energy_simulator::types::{BatteryMode, CostParams, IncrementalWalkResult, WalkStep};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone)]
struct ZoneProfile {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

/// Search a few well-known locations for `zones.json` so the test is portable
/// between the rust_refactor and clean-dashboard layouts.
fn locate_zones_json() -> PathBuf {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let candidates = [
        manifest_dir.join("..").join("data").join("zones.json"),
        manifest_dir
            .join("..")
            .join("public")
            .join("data")
            .join("zones.json"),
        manifest_dir
            .join("..")
            .join("dist")
            .join("data")
            .join("zones.json"),
        manifest_dir.join("data").join("zones.json"),
        manifest_dir.join("..").join("..").join("data").join("zones.json"),
    ];

    for candidate in candidates.iter() {
        if candidate.exists() {
            return candidate.clone();
        }
    }

    panic!(
        "Could not find zones.json. Searched: {:?}",
        candidates.iter().map(|p| p.display().to_string()).collect::<Vec<_>>()
    );
}

fn load_zone(zone: &str) -> ZoneProfile {
    let fixture_path = locate_zones_json();

    let content = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|e| panic!("Failed to read zones.json at {:?}: {}", fixture_path, e));

    let zones: HashMap<String, ZoneProfile> =
        serde_json::from_str(&content).expect("Failed to parse zones.json");

    zones
        .get(zone)
        .unwrap_or_else(|| panic!("Zone {} not found in zones.json", zone))
        .clone()
}

fn assert_finite(val: f64, label: &str) {
    assert!(
        val.is_finite(),
        "Expected finite value for {} but got {}",
        label,
        val
    );
}

fn assert_walk_trace_invariants(
    trace: &[WalkStep],
    target: f64,
    use_solar: bool,
    use_wind: bool,
    use_storage: bool,
    use_clean_firm: bool,
    label: &str,
) {
    assert!(
        trace.len() > 1,
        "{}: walk_trace must have more than just the baseline (got {})",
        label,
        trace.len()
    );

    // First entry must be the baseline at zero capacity.
    let first = &trace[0];
    assert_eq!(
        first.resource_added, "baseline",
        "{}: first walk_trace entry must be 'baseline' (got '{}')",
        label, first.resource_added
    );
    assert_eq!(first.solar_capacity, 0.0, "{}: baseline solar must be 0", label);
    assert_eq!(first.wind_capacity, 0.0, "{}: baseline wind must be 0", label);
    assert_eq!(
        first.storage_capacity, 0.0,
        "{}: baseline storage must be 0",
        label
    );
    assert_eq!(
        first.clean_firm_capacity, 0.0,
        "{}: baseline clean_firm must be 0",
        label
    );

    let mut prev = first;
    for (i, step) in trace.iter().enumerate() {
        assert_finite(step.match_percentage, &format!("{}: match_percentage[{}]", label, i));
        assert_finite(step.lcoe, &format!("{}: lcoe[{}]", label, i));
        assert_finite(step.lcoe_premium, &format!("{}: lcoe_premium[{}]", label, i));
        assert_finite(step.solar_capacity, &format!("{}: solar_capacity[{}]", label, i));
        assert_finite(step.wind_capacity, &format!("{}: wind_capacity[{}]", label, i));
        assert_finite(
            step.storage_capacity,
            &format!("{}: storage_capacity[{}]", label, i),
        );
        assert_finite(
            step.clean_firm_capacity,
            &format!("{}: clean_firm_capacity[{}]", label, i),
        );

        // Capacities must be monotonically non-decreasing.
        if i > 0 {
            assert!(
                step.solar_capacity >= prev.solar_capacity - 1e-9,
                "{}: solar_capacity decreased at step {} ({} -> {})",
                label,
                i,
                prev.solar_capacity,
                step.solar_capacity
            );
            assert!(
                step.wind_capacity >= prev.wind_capacity - 1e-9,
                "{}: wind_capacity decreased at step {} ({} -> {})",
                label,
                i,
                prev.wind_capacity,
                step.wind_capacity
            );
            assert!(
                step.storage_capacity >= prev.storage_capacity - 1e-9,
                "{}: storage_capacity decreased at step {} ({} -> {})",
                label,
                i,
                prev.storage_capacity,
                step.storage_capacity
            );
            assert!(
                step.clean_firm_capacity >= prev.clean_firm_capacity - 1e-9,
                "{}: clean_firm_capacity decreased at step {} ({} -> {})",
                label,
                i,
                prev.clean_firm_capacity,
                step.clean_firm_capacity
            );
        }

        // Disabled resources must remain zero throughout.
        if !use_solar {
            assert_eq!(
                step.solar_capacity, 0.0,
                "{}: solar disabled but step {} has {} MW",
                label, i, step.solar_capacity
            );
        }
        if !use_wind {
            assert_eq!(
                step.wind_capacity, 0.0,
                "{}: wind disabled but step {} has {} MW",
                label, i, step.wind_capacity
            );
        }
        if !use_storage {
            assert_eq!(
                step.storage_capacity, 0.0,
                "{}: storage disabled but step {} has {} MWh",
                label, i, step.storage_capacity
            );
        }
        if !use_clean_firm {
            assert_eq!(
                step.clean_firm_capacity, 0.0,
                "{}: clean_firm disabled but step {} has {} MW",
                label, i, step.clean_firm_capacity
            );
        }

        prev = step;
    }

    // Final achieved match must be within 1.0 percentage point of the target band.
    let final_match = trace.last().unwrap().match_percentage;
    assert!(
        (final_match - target).abs() <= 1.0,
        "{}: final match {:.3} not within 1.0% of target {:.3}",
        label,
        final_match,
        target
    );
}

fn assert_result_invariants(
    result: &IncrementalWalkResult,
    target: f64,
    label: &str,
) {
    assert_finite(result.solar_capacity, &format!("{}: solar_capacity", label));
    assert_finite(result.wind_capacity, &format!("{}: wind_capacity", label));
    assert_finite(result.storage_capacity, &format!("{}: storage_capacity", label));
    assert_finite(result.clean_firm_capacity, &format!("{}: clean_firm_capacity", label));
    assert_finite(result.final_lcoe, &format!("{}: final_lcoe", label));
    assert_finite(result.achieved_match, &format!("{}: achieved_match", label));

    // LCOE must be positive and within a sane range for a real grid system.
    assert!(
        result.final_lcoe > 0.0 && result.final_lcoe < 10000.0,
        "{}: final_lcoe {} out of reasonable range",
        label,
        result.final_lcoe
    );

    // Achieved match must be within 1.0% of the target.
    assert!(
        (result.achieved_match - target).abs() <= 1.0,
        "{}: achieved_match {:.3} not within 1.0% of target {:.3}",
        label,
        result.achieved_match,
        target
    );

    // Walk-trace and result final state should agree on the final portfolio.
    let last = result.walk_trace.last().expect("walk_trace must be non-empty");
    assert!(
        (last.solar_capacity - result.solar_capacity).abs() < 1e-9,
        "{}: walk_trace last solar {} != result solar {}",
        label,
        last.solar_capacity,
        result.solar_capacity
    );
    assert!(
        (last.wind_capacity - result.wind_capacity).abs() < 1e-9,
        "{}: walk_trace last wind != result wind",
        label
    );
    assert!(
        (last.storage_capacity - result.storage_capacity).abs() < 1e-9,
        "{}: walk_trace last storage != result storage",
        label
    );
    assert!(
        (last.clean_firm_capacity - result.clean_firm_capacity).abs() < 1e-9,
        "{}: walk_trace last clean_firm != result clean_firm",
        label
    );
}

#[test]
fn california_70_percent_all_resources() {
    let zone = load_zone("California");
    let costs = CostParams::default_costs();
    let target = 70.0;

    let result = run_incremental_walk(
        target,
        &zone.solar,
        &zone.wind,
        &zone.load,
        &costs,
        true, // solar
        true, // wind
        true, // storage
        true, // clean_firm
        BatteryMode::Hybrid,
        0.85,
        0.0,
    )
    .expect("Incremental walk failed");

    assert_walk_trace_invariants(
        &result.walk_trace,
        target,
        true,
        true,
        true,
        true,
        "California 70% all-resources",
    );
    assert_result_invariants(&result, target, "California 70% all-resources");

    // At a moderate target, the solver should add real capacity.
    let total_capacity = result.solar_capacity
        + result.wind_capacity
        + result.storage_capacity
        + result.clean_firm_capacity;
    assert!(
        total_capacity > 0.0,
        "Expected positive total capacity for 70% target (got {})",
        total_capacity
    );
}

#[test]
fn california_90_percent_no_clean_firm() {
    let zone = load_zone("California");
    let costs = CostParams::default_costs();
    let target = 90.0;

    let result = run_incremental_walk(
        target,
        &zone.solar,
        &zone.wind,
        &zone.load,
        &costs,
        true,  // solar
        true,  // wind
        true,  // storage
        false, // clean_firm DISABLED
        BatteryMode::Hybrid,
        0.85,
        0.0,
    )
    .expect("Incremental walk failed");

    assert_walk_trace_invariants(
        &result.walk_trace,
        target,
        true,
        true,
        true,
        false,
        "California 90% no-CF",
    );
    assert_result_invariants(&result, target, "California 90% no-CF");

    // Clean firm must be exactly zero (disabled).
    assert_eq!(
        result.clean_firm_capacity, 0.0,
        "Clean firm disabled but result has {} MW",
        result.clean_firm_capacity
    );
}

#[test]
fn texas_80_percent_all_resources() {
    let zone = load_zone("Texas");
    let costs = CostParams::default_costs();
    let target = 80.0;

    let result = run_incremental_walk(
        target,
        &zone.solar,
        &zone.wind,
        &zone.load,
        &costs,
        true,
        true,
        true,
        true,
        BatteryMode::Hybrid,
        0.85,
        0.0,
    )
    .expect("Incremental walk failed");

    assert_walk_trace_invariants(
        &result.walk_trace,
        target,
        true,
        true,
        true,
        true,
        "Texas 80% all-resources",
    );
    assert_result_invariants(&result, target, "Texas 80% all-resources");

    // At 80% target, expect at least one real resource to ramp.
    let total_capacity = result.solar_capacity
        + result.wind_capacity
        + result.storage_capacity
        + result.clean_firm_capacity;
    assert!(
        total_capacity > 0.0,
        "Expected positive total capacity for Texas 80% (got {})",
        total_capacity
    );
}

#[test]
fn determinism_same_inputs_yield_identical_walk() {
    let zone = load_zone("California");
    let costs = CostParams::default_costs();
    let target = 75.0;

    let a = run_incremental_walk(
        target,
        &zone.solar,
        &zone.wind,
        &zone.load,
        &costs,
        true,
        true,
        true,
        true,
        BatteryMode::Hybrid,
        0.85,
        0.0,
    )
    .expect("First run failed");
    let b = run_incremental_walk(
        target,
        &zone.solar,
        &zone.wind,
        &zone.load,
        &costs,
        true,
        true,
        true,
        true,
        BatteryMode::Hybrid,
        0.85,
        0.0,
    )
    .expect("Second run failed");

    assert_eq!(
        a.walk_trace.len(),
        b.walk_trace.len(),
        "Determinism: walk_trace length mismatch ({} vs {})",
        a.walk_trace.len(),
        b.walk_trace.len()
    );
    assert_eq!(
        a.steps, b.steps,
        "Determinism: step count mismatch ({} vs {})",
        a.steps, b.steps
    );

    for (i, (sa, sb)) in a.walk_trace.iter().zip(b.walk_trace.iter()).enumerate() {
        assert_eq!(
            sa.resource_added, sb.resource_added,
            "Determinism: resource_added differs at step {}",
            i
        );
        assert!(
            (sa.match_percentage - sb.match_percentage).abs() < 1e-12,
            "Determinism: match_percentage differs at step {} ({} vs {})",
            i,
            sa.match_percentage,
            sb.match_percentage
        );
        assert!(
            (sa.lcoe - sb.lcoe).abs() < 1e-9,
            "Determinism: lcoe differs at step {}",
            i
        );
        assert!(
            (sa.lcoe_premium - sb.lcoe_premium).abs() < 1e-9,
            "Determinism: lcoe_premium differs at step {}",
            i
        );
        assert!(
            (sa.solar_capacity - sb.solar_capacity).abs() < 1e-12,
            "Determinism: solar_capacity differs at step {}",
            i
        );
        assert!(
            (sa.wind_capacity - sb.wind_capacity).abs() < 1e-12,
            "Determinism: wind_capacity differs at step {}",
            i
        );
        assert!(
            (sa.storage_capacity - sb.storage_capacity).abs() < 1e-12,
            "Determinism: storage_capacity differs at step {}",
            i
        );
        assert!(
            (sa.clean_firm_capacity - sb.clean_firm_capacity).abs() < 1e-12,
            "Determinism: clean_firm_capacity differs at step {}",
            i
        );
    }

    assert!(
        (a.final_lcoe - b.final_lcoe).abs() < 1e-9,
        "Determinism: final_lcoe differs ({} vs {})",
        a.final_lcoe,
        b.final_lcoe
    );
    assert!(
        (a.achieved_match - b.achieved_match).abs() < 1e-12,
        "Determinism: achieved_match differs ({} vs {})",
        a.achieved_match,
        b.achieved_match
    );
}
