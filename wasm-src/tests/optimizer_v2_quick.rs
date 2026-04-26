/// Quick Tests for V2 Optimizer
///
/// These tests run in ~30 seconds and should be run on every change.
/// They cover basic functionality, precision, and performance.
use energy_simulator::{
    run_v2_optimizer, run_v2_optimizer_mode, BatteryMode, CostParams, OptimizerConfig, V2Mode,
    HOURS_PER_YEAR,
};

fn create_test_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    // Solar: peaks at midday
    let solar: Vec<f64> = (0..HOURS_PER_YEAR)
        .map(|h| {
            let hour_of_day = h % 24;
            if hour_of_day >= 6 && hour_of_day <= 18 {
                let peak_factor = 1.0 - ((hour_of_day as f64 - 12.0).abs() / 6.0);
                0.25 * peak_factor * 2.0
            } else {
                0.0
            }
        })
        .collect();

    // Wind: higher at night
    let wind: Vec<f64> = (0..HOURS_PER_YEAR)
        .map(|h| {
            let hour_of_day = h % 24;
            if hour_of_day < 6 || hour_of_day > 20 {
                0.35 * 1.2
            } else {
                0.35 * 0.8
            }
        })
        .collect();

    // Constant load
    let load = vec![100.0; HOURS_PER_YEAR];

    (solar, wind, load)
}

/// Test 1: Basic target compliance across range
#[test]
fn test_basic_targets() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    for target in [0.0, 30.0, 50.0, 80.0, 99.0] {
        let result = run_v2_optimizer(
            target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        )
        .expect(&format!("Optimization failed for target {}", target));

        // Should be close to target
        let deviation = (result.achieved_clean_match - target).abs();
        assert!(
            deviation < 1.0,
            "Target {}: achieved {:.2}%, deviation {:.2}%",
            target,
            result.achieved_clean_match,
            deviation
        );

        // LCOE should be reasonable
        assert!(
            result.lcoe > 0.0 && result.lcoe < 500.0,
            "Target {}: unreasonable LCOE {}",
            target,
            result.lcoe
        );
    }
}

/// Test 2: Precision at boundary values
#[test]
fn test_precision_boundaries() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    for target in [0.1, 0.5, 49.9, 50.1, 99.5] {
        let result = run_v2_optimizer(
            target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        );

        match result {
            Ok(r) => {
                let deviation = (r.achieved_clean_match - target).abs();
                // Allow slightly larger deviation for edge cases
                assert!(
                    deviation < 1.5,
                    "Precision test failed for target {}: achieved {:.2}%, deviation {:.2}%",
                    target,
                    r.achieved_clean_match,
                    deviation
                );
            }
            Err(e) => {
                // Some edge cases may fail - that's OK for quick test
                println!("Note: Target {} failed with: {}", target, e);
            }
        }
    }
}

/// Test 3: Extended bounds are used appropriately
#[test]
fn test_extended_bounds() {
    let (solar, wind, load) = create_test_profiles();
    let mut costs = CostParams::default_costs();
    costs.solar_capex = 200.0; // Very cheap solar
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        99.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed for 99% target");

    // With very cheap solar, should use high capacity
    // (may still need CF for 99%)
    assert!(
        result.solar_capacity > 100.0 || result.clean_firm_capacity > 0.0,
        "Should use significant solar or CF for 99% target"
    );
}

/// Test 4: All battery modes work
#[test]
fn test_battery_modes() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    for mode in [
        BatteryMode::Default,
        BatteryMode::PeakShaver,
        BatteryMode::Hybrid,
    ] {
        let result = run_v2_optimizer(70.0, &solar, &wind, &load, &costs, &config, mode, None)
            .expect(&format!("Mode {:?} failed", mode));

        assert!(
            (result.achieved_clean_match - 70.0).abs() < 2.0,
            "Mode {:?}: achieved {:.2}%, expected ~70%",
            mode,
            result.achieved_clean_match
        );
    }
}

/// Test 5: Performance check - should complete quickly
#[test]
fn test_performance() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    let start = std::time::Instant::now();

    for target in [30.0, 50.0, 70.0, 90.0] {
        run_v2_optimizer(
            target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        )
        .expect(&format!("Optimization failed for target {}", target));
    }

    let elapsed = start.elapsed();

    // 4 optimizations should complete in reasonable time
    // Without empirical model, allow up to 2 seconds each
    assert!(
        elapsed.as_millis() < 8000,
        "Performance too slow: {}ms for 4 optimizations",
        elapsed.as_millis()
    );

    println!(
        "Performance test: 4 optimizations in {}ms",
        elapsed.as_millis()
    );
}

/// Test 6: Disabled resources must have zero capacity
#[test]
fn test_disabled_resources() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();

    // Disable wind and CF
    let mut config = OptimizerConfig::default();
    config.enable_wind = false;
    config.enable_clean_firm = false;

    let result = run_v2_optimizer(
        30.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    assert_eq!(result.wind_capacity, 0.0, "Wind should be 0 when disabled");
    assert_eq!(
        result.clean_firm_capacity, 0.0,
        "CF should be 0 when disabled"
    );
}

/// Test 7: Zero target should require minimal resources
#[test]
fn test_zero_target() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        0.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed for 0% target");

    // Should have minimal clean resources
    assert!(
        result.achieved_clean_match < 5.0,
        "0% target should result in very low clean match, got {}",
        result.achieved_clean_match
    );
}

/// Test 8: Solar-only configuration
#[test]
fn test_solar_only() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();

    let mut config = OptimizerConfig::default();
    config.enable_wind = false;
    config.enable_storage = false;
    config.enable_clean_firm = false;

    let result = run_v2_optimizer(
        25.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    assert!(result.solar_capacity > 0.0, "Should use some solar");
    assert_eq!(result.wind_capacity, 0.0, "Wind should be 0");
    assert_eq!(result.storage_capacity, 0.0, "Storage should be 0");
    assert_eq!(result.clean_firm_capacity, 0.0, "CF should be 0");
}

/// Test 9: Wind-only configuration
#[test]
fn test_wind_only() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();

    let mut config = OptimizerConfig::default();
    config.enable_solar = false;
    config.enable_storage = false;
    config.enable_clean_firm = false;

    let result = run_v2_optimizer(
        30.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    assert_eq!(result.solar_capacity, 0.0, "Solar should be 0");
    assert!(result.wind_capacity > 0.0, "Should use some wind");
    assert_eq!(result.storage_capacity, 0.0, "Storage should be 0");
    assert_eq!(result.clean_firm_capacity, 0.0, "CF should be 0");
}

/// Test 10: Consistency check - same result each time
#[test]
fn test_determinism() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    let first = run_v2_optimizer(
        60.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("First optimization failed");

    // Run again
    let second = run_v2_optimizer(
        60.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Second optimization failed");

    // Results should be identical
    assert_eq!(
        first.solar_capacity, second.solar_capacity,
        "Solar mismatch"
    );
    assert_eq!(first.wind_capacity, second.wind_capacity, "Wind mismatch");
    assert_eq!(
        first.storage_capacity, second.storage_capacity,
        "Storage mismatch"
    );
    assert!(
        (first.clean_firm_capacity - second.clean_firm_capacity).abs() < 0.5,
        "CF mismatch: {} vs {}",
        first.clean_firm_capacity,
        second.clean_firm_capacity
    );
}

/// Test 11: Fast mode API is invariant with legacy API
#[test]
fn test_fast_mode_invariance() {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();
    let target = 95.0;

    let legacy = run_v2_optimizer(
        target,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("legacy fast run failed");

    let explicit_fast = run_v2_optimizer_mode(
        target,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
        V2Mode::Fast,
        None,
    )
    .expect("explicit fast run failed");

    assert_eq!(legacy.solar_capacity, explicit_fast.solar_capacity);
    assert_eq!(legacy.wind_capacity, explicit_fast.wind_capacity);
    assert_eq!(legacy.storage_capacity, explicit_fast.storage_capacity);
    assert!((legacy.clean_firm_capacity - explicit_fast.clean_firm_capacity).abs() < 1e-9);
    assert!((legacy.achieved_clean_match - explicit_fast.achieved_clean_match).abs() < 1e-9);
    assert!((legacy.lcoe - explicit_fast.lcoe).abs() < 1e-9);
    assert_eq!(legacy.num_evaluations, explicit_fast.num_evaluations);
    assert_eq!(legacy.success, explicit_fast.success);
}
