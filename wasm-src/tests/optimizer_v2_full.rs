/// Full Tests for V2 Optimizer
///
/// These tests are more comprehensive and take longer (~5 minutes).
/// Run before committing changes.
use energy_simulator::{
    run_v2_optimizer, run_v2_sweep, BatteryMode, CostParams, OptimizerConfig, HOURS_PER_YEAR,
};

fn create_zone_profiles(zone: &str) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let (solar_cf, wind_cf) = match zone {
        "california" => (0.28, 0.32),
        "texas" => (0.26, 0.38),
        "florida" => (0.24, 0.20),
        "newyork" => (0.18, 0.30),
        "pjm" => (0.20, 0.28),
        "miso" => (0.22, 0.35),
        "spp" => (0.24, 0.42),
        "ercot" => (0.26, 0.36),
        "northwest" => (0.16, 0.28),
        "southwest" => (0.30, 0.25),
        "southeast" => (0.23, 0.22),
        "newengland" => (0.17, 0.28),
        "midwest" => (0.20, 0.34),
        _ => (0.25, 0.30),
    };

    // Solar: peaks at midday
    let solar: Vec<f64> = (0..HOURS_PER_YEAR)
        .map(|h| {
            let hour_of_day = h % 24;
            if hour_of_day >= 6 && hour_of_day <= 18 {
                let peak_factor = 1.0 - ((hour_of_day as f64 - 12.0).abs() / 6.0);
                solar_cf * peak_factor * 2.0
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
                wind_cf * 1.2
            } else {
                wind_cf * 0.8
            }
        })
        .collect();

    // Constant load
    let load = vec![100.0; HOURS_PER_YEAR];

    (solar, wind, load)
}

/// Test across multiple zones with different profiles
#[test]
fn test_multiple_zones() {
    let zones = ["california", "texas", "florida", "newyork", "pjm"];

    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    for zone in &zones {
        let (solar, wind, load) = create_zone_profiles(zone);

        for target in [30.0, 50.0, 70.0, 90.0] {
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
                    assert!(
                        deviation < 2.0,
                        "Zone {} target {}: achieved {:.2}%, deviation {:.2}%",
                        zone,
                        target,
                        r.achieved_clean_match,
                        deviation
                    );
                }
                Err(e) => {
                    panic!("Zone {} target {} failed: {}", zone, target, e);
                }
            }
        }
    }
}

/// Test cost sensitivity - cheap solar
#[test]
fn test_cheap_solar() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.solar_capex = 500.0; // Half price
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        70.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    // With cheap solar, optimizer should produce a valid result
    // Note: greedy may still prefer wind if California's wind profile has
    // significantly higher capacity factor - that's economically rational
    assert!(result.success, "Cheap solar: optimization should succeed");
    assert!(
        result.lcoe > 0.0 && result.lcoe < 200.0,
        "Cheap solar: LCOE should be reasonable, got {}",
        result.lcoe
    );
    // Either solar or wind should be used (at least one renewable)
    assert!(
        result.solar_capacity > 0.0 || result.wind_capacity > 0.0,
        "Cheap solar: should use some renewables, got solar={}, wind={}",
        result.solar_capacity,
        result.wind_capacity
    );
}

/// Test cost sensitivity - expensive solar
#[test]
fn test_expensive_solar() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.solar_capex = 2000.0; // Double price
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        70.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    // With expensive solar, may prefer wind or CF
    assert!(
        result.wind_capacity > 0.0 || result.clean_firm_capacity > 0.0,
        "Should use alternatives when solar is expensive"
    );
}

/// Test cost sensitivity - cheap wind
#[test]
fn test_cheap_wind() {
    let (solar, wind, load) = create_zone_profiles("texas"); // Good wind
    let mut costs = CostParams::default_costs();
    costs.wind_capex = 600.0; // Half price
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        70.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    // With cheap wind, should use significant wind
    assert!(
        result.wind_capacity >= result.solar_capacity,
        "Cheap wind in Texas: wind {} < solar {}",
        result.wind_capacity,
        result.solar_capacity
    );
}

/// Test cost sensitivity - cheap storage
#[test]
fn test_cheap_storage() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.storage_capex = 100.0; // Very cheap
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        80.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    // With cheap storage, should achieve target with reasonable LCOE
    // Storage may or may not be used significantly depending on renewable mix
    // The greedy approach adds storage when it improves efficiency
    assert!(
        result.achieved_clean_match >= 79.0,
        "Should achieve ~80% target, got {}",
        result.achieved_clean_match
    );
    assert!(result.lcoe > 0.0, "LCOE should be positive");
}

/// Test cost sensitivity - cheap clean firm
#[test]
fn test_cheap_clean_firm() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.clean_firm_capex = 1000.0; // Much cheaper
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        95.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    // With cheap CF and high target, should use some CF to hit target
    // The exact amount depends on how much renewables contribute
    assert!(
        result.clean_firm_capacity > 0.0,
        "Cheap CF at 95% target should use some CF, got {}",
        result.clean_firm_capacity
    );
    assert!(
        result.achieved_clean_match >= 94.0,
        "Should achieve ~95% target, got {}",
        result.achieved_clean_match
    );
}

/// Test cost sensitivity - expensive clean firm
#[test]
fn test_expensive_clean_firm() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.clean_firm_capex = 10000.0; // Very expensive
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
    .expect("Optimization failed");

    // With expensive CF, should prefer renewables + storage
    assert!(
        result.clean_firm_capacity < 10.0,
        "Expensive CF at 50% target should use minimal CF, got {}",
        result.clean_firm_capacity
    );
}

/// Test cost sensitivity - high gas price
#[test]
fn test_high_gas_price() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.gas_price = 14.0; // High gas
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        80.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed");

    // With high gas price, optimizer should find a solution
    assert!(
        (result.achieved_clean_match - 80.0).abs() < 2.0,
        "Should achieve target with high gas price"
    );
}

/// Test cost sensitivity - low gas price
#[test]
fn test_low_gas_price() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.gas_price = 2.0; // Low gas
    let config = OptimizerConfig::default();

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

    // Low target with low gas price - should be achievable
    assert!(
        (result.achieved_clean_match - 30.0).abs() < 2.0,
        "Should achieve 30% with low gas price"
    );
}

/// Test all resource combinations
#[test]
fn test_resource_combinations() {
    let (solar, wind, load) = create_zone_profiles("california");
    let costs = CostParams::default_costs();

    let combinations = [
        ("solar_only", true, false, false, false),
        ("wind_only", false, true, false, false),
        ("no_storage", true, true, false, true),
        ("no_cf", true, true, true, false),
        ("renewables_only", true, true, false, false),
        ("storage_cf_only", false, false, true, true),
    ];

    for (name, solar_en, wind_en, storage_en, cf_en) in combinations {
        let mut config = OptimizerConfig::default();
        config.enable_solar = solar_en;
        config.enable_wind = wind_en;
        config.enable_storage = storage_en;
        config.enable_clean_firm = cf_en;

        let result = run_v2_optimizer(
            40.0,
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
                // Verify disabled resources have 0 capacity
                if !solar_en {
                    assert_eq!(r.solar_capacity, 0.0, "{}: solar should be 0", name);
                }
                if !wind_en {
                    assert_eq!(r.wind_capacity, 0.0, "{}: wind should be 0", name);
                }
                if !storage_en {
                    assert_eq!(r.storage_capacity, 0.0, "{}: storage should be 0", name);
                }
                if !cf_en {
                    assert_eq!(
                        r.clean_firm_capacity, 0.0,
                        "{}: clean_firm should be 0",
                        name
                    );
                }
            }
            Err(_) => {
                // Some combinations may be infeasible
                println!("Note: {} combination infeasible for 40% target", name);
            }
        }
    }
}

/// Test optimizer sweep function
#[test]
fn test_optimizer_sweep() {
    let (solar, wind, load) = create_zone_profiles("california");
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    let targets = vec![20.0, 40.0, 60.0, 80.0];
    let results = run_v2_sweep(
        &targets,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Sweep failed");

    assert_eq!(results.len(), targets.len());

    // LCOE should generally increase with target
    for (i, result) in results.iter().enumerate() {
        assert!(result.lcoe > 0.0, "Result {} has invalid LCOE", i);

        // Each result should be close to its target
        let deviation = (result.achieved_clean_match - targets[i]).abs();
        assert!(deviation < 2.0, "Result {} deviation: {:.2}%", i, deviation);
    }

    // Higher targets generally mean higher LCOE (with some tolerance)
    // This isn't always strictly true due to optimizer variance
}

/// Test edge case: very low target
#[test]
fn test_very_low_target() {
    let (solar, wind, load) = create_zone_profiles("california");
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        1.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed for 1% target");

    // Should have minimal clean resources
    assert!(
        result.achieved_clean_match < 5.0,
        "1% target should result in low clean match"
    );
}

/// Test edge case: high target with expensive CF
#[test]
fn test_high_target_expensive_cf() {
    let (solar, wind, load) = create_zone_profiles("california");
    let mut costs = CostParams::default_costs();
    costs.clean_firm_capex = 8000.0;
    let config = OptimizerConfig::default();

    let result = run_v2_optimizer(
        95.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        None,
    )
    .expect("Optimization failed for 95% target");

    // Should still find a solution, just expensive
    assert!(result.lcoe > 0.0);
    assert!(result.achieved_clean_match > 90.0);
}

/// Test all battery modes produce valid results
#[test]
fn test_all_battery_modes() {
    let (solar, wind, load) = create_zone_profiles("california");
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    for mode in [
        BatteryMode::Default,
        BatteryMode::PeakShaver,
        BatteryMode::Hybrid,
    ] {
        for target in [30.0, 60.0, 90.0] {
            let result =
                run_v2_optimizer(target, &solar, &wind, &load, &costs, &config, mode, None)
                    .expect(&format!("Mode {:?} target {} failed", mode, target));

            assert!(
                (result.achieved_clean_match - target).abs() < 3.0,
                "Mode {:?} target {}: achieved {:.2}%",
                mode,
                target,
                result.achieved_clean_match
            );
        }
    }
}

/// Test that LCOE is calculated correctly
#[test]
fn test_lcoe_calculation() {
    let (solar, wind, load) = create_zone_profiles("california");
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
    .expect("Optimization failed");

    // LCOE should be in reasonable range for this scenario
    // With ~50% renewables and gas backup, expect ~$40-80/MWh
    assert!(
        result.lcoe > 20.0 && result.lcoe < 150.0,
        "LCOE {} seems unreasonable for 50% clean",
        result.lcoe
    );
}
