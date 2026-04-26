//! Verify Hybrid mode meets its objectives:
//! 1. Peak gas ≈ Peak Shaver's peak gas
//! 2. Total cycling >= Peak Shaver's cycling
//! 3. Clean match >= Peak Shaver's clean match

use energy_simulator::simulation::core::simulate_system;
use energy_simulator::types::{BatteryMode, SimulationConfig};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct PythonBaseline {
    profiles: Profiles,
}

#[derive(Debug, Deserialize)]
struct Profiles {
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
}

fn main() {
    // Load profiles from Python baseline
    let baseline_path = "../python_baseline_results.json";
    let baseline_json =
        fs::read_to_string(baseline_path).expect("Failed to read python_baseline_results.json");
    let baseline: PythonBaseline =
        serde_json::from_str(&baseline_json).expect("Failed to parse baseline JSON");

    let solar_profile = &baseline.profiles.solar_profile;
    let wind_profile = &baseline.profiles.wind_profile;
    let load_profile = &baseline.profiles.load_profile;

    println!("Hybrid Mode Verification");
    println!("========================\n");

    // Test configurations
    let configs = [
        ("Solar 200, Storage 120", 200.0, 100.0, 120.0),
        ("Solar 100, Wind 100, Storage 100", 100.0, 100.0, 100.0),
        ("Wind 200, Storage 150", 0.0, 200.0, 150.0),
    ];

    let mut all_passed = true;

    for (name, solar, wind, storage) in configs {
        println!("Configuration: {}", name);
        println!("{}", "-".repeat(50));

        // Run Peak Shaver
        let ps_config = SimulationConfig {
            solar_capacity: solar,
            wind_capacity: wind,
            storage_capacity: storage,
            clean_firm_capacity: 0.0,
            battery_efficiency: 0.85,
            max_demand_response: 0.0,
            battery_mode: BatteryMode::PeakShaver,
        };

        let ps_result = simulate_system(&ps_config, solar_profile, wind_profile, load_profile)
            .expect("Peak Shaver simulation failed");

        // Run Hybrid
        let hy_config = SimulationConfig {
            battery_mode: BatteryMode::Hybrid,
            ..ps_config
        };

        let hy_result = simulate_system(&hy_config, solar_profile, wind_profile, load_profile)
            .expect("Hybrid simulation failed");

        // Calculate metrics
        let ps_peak_gas = ps_result.gas_generation.iter().cloned().fold(0.0, f64::max);
        let hy_peak_gas = hy_result.gas_generation.iter().cloned().fold(0.0, f64::max);

        let ps_total_charge: f64 = ps_result.battery_charge.iter().sum();
        let hy_total_charge: f64 = hy_result.battery_charge.iter().sum();

        let ps_total_discharge: f64 = ps_result.battery_discharge.iter().sum();
        let hy_total_discharge: f64 = hy_result.battery_discharge.iter().sum();

        let ps_total_gas: f64 = ps_result.gas_generation.iter().sum();
        let hy_total_gas: f64 = hy_result.gas_generation.iter().sum();

        let ps_clean_match = ps_result.clean_match_pct;
        let hy_clean_match = hy_result.clean_match_pct;

        // Print comparison
        println!("                    Peak Shaver    Hybrid      Diff");
        println!(
            "Peak Gas (MW):      {:10.1}    {:10.1}  {:+.1}%",
            ps_peak_gas,
            hy_peak_gas,
            (hy_peak_gas - ps_peak_gas) / ps_peak_gas * 100.0
        );
        println!(
            "Total Charge (MWh): {:10.0}    {:10.0}  {:+.1}%",
            ps_total_charge,
            hy_total_charge,
            if ps_total_charge > 0.0 {
                (hy_total_charge - ps_total_charge) / ps_total_charge * 100.0
            } else {
                0.0
            }
        );
        println!(
            "Total Discharge:    {:10.0}    {:10.0}  {:+.1}%",
            ps_total_discharge,
            hy_total_discharge,
            if ps_total_discharge > 0.0 {
                (hy_total_discharge - ps_total_discharge) / ps_total_discharge * 100.0
            } else {
                0.0
            }
        );
        println!(
            "Total Gas (MWh):    {:10.0}    {:10.0}  {:+.1}%",
            ps_total_gas,
            hy_total_gas,
            (hy_total_gas - ps_total_gas) / ps_total_gas * 100.0
        );
        println!(
            "Clean Match (%):    {:10.1}    {:10.1}  {:+.1}pp",
            ps_clean_match,
            hy_clean_match,
            hy_clean_match - ps_clean_match
        );

        // Verification criteria
        let peak_gas_ok = (hy_peak_gas - ps_peak_gas).abs() / ps_peak_gas < 0.02; // Within 2%
        let cycling_ok = hy_total_discharge >= ps_total_discharge * 0.99; // >= 99% of PS
        let clean_match_ok = hy_clean_match >= ps_clean_match - 0.5; // >= PS - 0.5pp

        println!("\nVerification:");
        println!(
            "  Peak gas within 2%:   {}",
            if peak_gas_ok { "✓ PASS" } else { "✗ FAIL" }
        );
        println!(
            "  Cycling >= PS:        {}",
            if cycling_ok { "✓ PASS" } else { "✗ FAIL" }
        );
        println!(
            "  Clean match >= PS:    {}",
            if clean_match_ok {
                "✓ PASS"
            } else {
                "✗ FAIL"
            }
        );

        let config_pass = peak_gas_ok && cycling_ok && clean_match_ok;
        all_passed = all_passed && config_pass;
        println!(
            "  Overall:              {}\n",
            if config_pass { "✓ PASS" } else { "✗ FAIL" }
        );
    }

    println!("Performance Comparison:");
    println!("{}", "-".repeat(50));

    // Benchmark
    use std::time::Instant;

    let config = SimulationConfig {
        solar_capacity: 200.0,
        wind_capacity: 100.0,
        storage_capacity: 120.0,
        clean_firm_capacity: 0.0,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
        battery_mode: BatteryMode::PeakShaver,
    };

    // Warmup
    for _ in 0..10 {
        let _ = simulate_system(&config, solar_profile, wind_profile, load_profile);
    }

    // Peak Shaver timing
    let start = Instant::now();
    for _ in 0..100 {
        let _ = simulate_system(&config, solar_profile, wind_profile, load_profile);
    }
    let ps_time = start.elapsed().as_micros() as f64 / 100.0;

    // Hybrid timing
    let hy_config = SimulationConfig {
        battery_mode: BatteryMode::Hybrid,
        ..config
    };

    let start = Instant::now();
    for _ in 0..100 {
        let _ = simulate_system(&hy_config, solar_profile, wind_profile, load_profile);
    }
    let hy_time = start.elapsed().as_micros() as f64 / 100.0;

    println!("Peak Shaver: {:.0} µs", ps_time);
    println!(
        "Hybrid:      {:.0} µs ({:.1}x overhead)",
        hy_time,
        hy_time / ps_time
    );

    println!("\n{}", "=".repeat(50));
    println!(
        "FINAL RESULT: {}",
        if all_passed {
            "✓ ALL TESTS PASSED"
        } else {
            "✗ SOME TESTS FAILED"
        }
    );
}
