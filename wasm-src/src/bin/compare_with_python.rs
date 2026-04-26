//! Compare Rust simulation results with Python baseline
//!
//! Loads Python baseline results and runs the same scenarios in Rust,
//! then compares the outputs across all 8760 hours.

use energy_simulator::simulation::core::simulate_system;
use energy_simulator::types::{BatteryMode, SimulationConfig, HOURS_PER_YEAR};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Deserialize)]
struct PythonBaseline {
    config: Config,
    profiles: Profiles,
    results: Results,
}

#[derive(Debug, Deserialize)]
struct Config {
    solar_capacity: f64,
    wind_capacity: f64,
    storage_capacity: f64,
    clean_firm_capacity: f64,
    battery_eff: f64,
    max_demand_response: f64,
}

#[derive(Debug, Deserialize)]
struct Profiles {
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct Results {
    default: ModeResult,
    peak_shaver: ModeResult,
    hybrid: ModeResult,
}

#[derive(Debug, Deserialize)]
struct ModeResult {
    arrays: Arrays,
    stats: Stats,
}

#[derive(Debug, Deserialize)]
struct Arrays {
    battery_charge: Vec<f64>,
    battery_discharge: Vec<f64>,
    gas_generation: Vec<f64>,
    curtailed: Vec<f64>,
    solar_out: Vec<f64>,
    wind_out: Vec<f64>,
    #[serde(alias = "renewable_delivered")]
    clean_delivered: Vec<f64>,
}

#[derive(Debug, Deserialize)]
struct Stats {
    total_charge: f64,
    total_discharge: f64,
    total_curtailed: f64,
    total_gas: f64,
    #[serde(default)]
    peak_gas: f64,
    hours_charging: usize,
    hours_discharging: usize,
}

#[derive(Debug, Serialize)]
struct ComparisonResult {
    mode: String,
    python_stats: StatsReport,
    rust_stats: StatsReport,
    differences: DifferenceReport,
}

#[derive(Debug, Serialize)]
struct StatsReport {
    total_charge: f64,
    total_discharge: f64,
    total_curtailed: f64,
    total_gas: f64,
    peak_gas: f64,
    hours_charging: usize,
    hours_discharging: usize,
}

#[derive(Debug, Serialize)]
struct DifferenceReport {
    charge_diff_pct: f64,
    discharge_diff_pct: f64,
    curtailed_diff_pct: f64,
    gas_diff_pct: f64,
    max_hourly_charge_diff: f64,
    max_hourly_discharge_diff: f64,
    max_hourly_gas_diff: f64,
    hours_with_charge_diff: usize,
    hours_with_discharge_diff: usize,
    first_diff_hour_charge: Option<usize>,
    first_diff_hour_discharge: Option<usize>,
}

fn compute_stats(charge: &[f64], discharge: &[f64], curtailed: &[f64], gas: &[f64]) -> StatsReport {
    StatsReport {
        total_charge: charge.iter().sum(),
        total_discharge: discharge.iter().sum(),
        total_curtailed: curtailed.iter().sum(),
        total_gas: gas.iter().sum(),
        peak_gas: gas.iter().cloned().fold(0.0, f64::max),
        hours_charging: charge.iter().filter(|&&x| x > 0.01).count(),
        hours_discharging: discharge.iter().filter(|&&x| x > 0.01).count(),
    }
}

fn compare_arrays(python: &[f64], rust: &[f64], tolerance: f64) -> (f64, usize, Option<usize>) {
    let mut max_diff = 0.0f64;
    let mut hours_with_diff = 0usize;
    let mut first_diff_hour = None;

    for i in 0..python.len().min(rust.len()) {
        let diff = (python[i] - rust[i]).abs();
        if diff > tolerance {
            hours_with_diff += 1;
            if first_diff_hour.is_none() {
                first_diff_hour = Some(i);
            }
        }
        max_diff = max_diff.max(diff);
    }

    (max_diff, hours_with_diff, first_diff_hour)
}

fn percent_diff(python: f64, rust: f64) -> f64 {
    if python.abs() < 0.01 {
        if rust.abs() < 0.01 {
            0.0
        } else {
            100.0
        }
    } else {
        ((rust - python) / python * 100.0).abs()
    }
}

fn run_comparison(
    mode_name: &str,
    mode: BatteryMode,
    config: &Config,
    profiles: &Profiles,
    python_result: &ModeResult,
) -> ComparisonResult {
    println!("\n=== {} MODE ===", mode_name.to_uppercase());

    // Create Rust simulation config
    let sim_config = SimulationConfig {
        solar_capacity: config.solar_capacity,
        wind_capacity: config.wind_capacity,
        storage_capacity: config.storage_capacity,
        clean_firm_capacity: config.clean_firm_capacity,
        battery_efficiency: config.battery_eff,
        max_demand_response: config.max_demand_response,
        battery_mode: mode,
    };

    // Run Rust simulation
    let rust_result = simulate_system(
        &sim_config,
        &profiles.solar_profile,
        &profiles.wind_profile,
        &profiles.load_profile,
    )
    .expect("Rust simulation failed");

    // Compute Rust stats
    let rust_stats = compute_stats(
        &rust_result.battery_charge,
        &rust_result.battery_discharge,
        &rust_result.curtailed,
        &rust_result.gas_generation,
    );

    // Python stats
    let python_stats = StatsReport {
        total_charge: python_result.stats.total_charge,
        total_discharge: python_result.stats.total_discharge,
        total_curtailed: python_result.stats.total_curtailed,
        total_gas: python_result.stats.total_gas,
        peak_gas: python_result.stats.peak_gas,
        hours_charging: python_result.stats.hours_charging,
        hours_discharging: python_result.stats.hours_discharging,
    };

    // Compare arrays
    let tolerance = 0.1; // MW tolerance
    let (max_charge_diff, hours_charge_diff, first_charge_diff) = compare_arrays(
        &python_result.arrays.battery_charge,
        &rust_result.battery_charge,
        tolerance,
    );
    let (max_discharge_diff, hours_discharge_diff, first_discharge_diff) = compare_arrays(
        &python_result.arrays.battery_discharge,
        &rust_result.battery_discharge,
        tolerance,
    );
    let (max_gas_diff, _, _) = compare_arrays(
        &python_result.arrays.gas_generation,
        &rust_result.gas_generation,
        tolerance,
    );

    let differences = DifferenceReport {
        charge_diff_pct: percent_diff(python_stats.total_charge, rust_stats.total_charge),
        discharge_diff_pct: percent_diff(python_stats.total_discharge, rust_stats.total_discharge),
        curtailed_diff_pct: percent_diff(python_stats.total_curtailed, rust_stats.total_curtailed),
        gas_diff_pct: percent_diff(python_stats.total_gas, rust_stats.total_gas),
        max_hourly_charge_diff: max_charge_diff,
        max_hourly_discharge_diff: max_discharge_diff,
        max_hourly_gas_diff: max_gas_diff,
        hours_with_charge_diff: hours_charge_diff,
        hours_with_discharge_diff: hours_discharge_diff,
        first_diff_hour_charge: first_charge_diff,
        first_diff_hour_discharge: first_discharge_diff,
    };

    // Print comparison
    println!("\nPython Results:");
    println!(
        "  Total charge:     {:>10.2} MWh ({} hours)",
        python_stats.total_charge, python_stats.hours_charging
    );
    println!(
        "  Total discharge:  {:>10.2} MWh ({} hours)",
        python_stats.total_discharge, python_stats.hours_discharging
    );
    println!(
        "  Total curtailed:  {:>10.2} MWh",
        python_stats.total_curtailed
    );
    println!("  Total gas:        {:>10.2} MWh", python_stats.total_gas);

    println!("\nRust Results:");
    println!(
        "  Total charge:     {:>10.2} MWh ({} hours)",
        rust_stats.total_charge, rust_stats.hours_charging
    );
    println!(
        "  Total discharge:  {:>10.2} MWh ({} hours)",
        rust_stats.total_discharge, rust_stats.hours_discharging
    );
    println!(
        "  Total curtailed:  {:>10.2} MWh",
        rust_stats.total_curtailed
    );
    println!("  Total gas:        {:>10.2} MWh", rust_stats.total_gas);

    println!("\nDifferences:");
    println!(
        "  Charge diff:      {:>10.2}% (max hourly: {:.2} MW)",
        differences.charge_diff_pct, differences.max_hourly_charge_diff
    );
    println!(
        "  Discharge diff:   {:>10.2}% (max hourly: {:.2} MW)",
        differences.discharge_diff_pct, differences.max_hourly_discharge_diff
    );
    println!(
        "  Curtailed diff:   {:>10.2}%",
        differences.curtailed_diff_pct
    );
    println!(
        "  Gas diff:         {:>10.2}% (max hourly: {:.2} MW)",
        differences.gas_diff_pct, differences.max_hourly_gas_diff
    );
    println!(
        "  Hours with charge diff (>0.1 MW): {}",
        differences.hours_with_charge_diff
    );
    println!(
        "  Hours with discharge diff (>0.1 MW): {}",
        differences.hours_with_discharge_diff
    );

    if let Some(hour) = differences.first_diff_hour_charge {
        println!("\n  First charge difference at hour {}:", hour);
        println!(
            "    Python: {:.4} MW",
            python_result.arrays.battery_charge[hour]
        );
        println!("    Rust:   {:.4} MW", rust_result.battery_charge[hour]);
    }

    if let Some(hour) = differences.first_diff_hour_discharge {
        println!("\n  First discharge difference at hour {}:", hour);
        println!(
            "    Python: {:.4} MW",
            python_result.arrays.battery_discharge[hour]
        );
        println!("    Rust:   {:.4} MW", rust_result.battery_discharge[hour]);

        // Show context around first difference
        let start = hour.saturating_sub(5);
        let end = (hour + 10).min(HOURS_PER_YEAR);
        println!("\n  Context (hours {}-{}):", start, end);
        println!(
            "  {:>6} {:>12} {:>12} {:>12} {:>12}",
            "Hour", "Py Charge", "Rs Charge", "Py Disch", "Rs Disch"
        );
        for h in start..end {
            let marker = if h == hour { " <--" } else { "" };
            println!(
                "  {:>6} {:>12.2} {:>12.2} {:>12.2} {:>12.2}{}",
                h,
                python_result.arrays.battery_charge[h],
                rust_result.battery_charge[h],
                python_result.arrays.battery_discharge[h],
                rust_result.battery_discharge[h],
                marker
            );
        }
    }

    ComparisonResult {
        mode: mode_name.to_string(),
        python_stats,
        rust_stats,
        differences,
    }
}

fn main() {
    println!("Loading Python baseline results...");

    let baseline_path = "../python_baseline_results.json";
    let baseline_json = fs::read_to_string(baseline_path)
        .expect("Failed to read Python baseline results. Run compare_implementations.py first.");

    let baseline: PythonBaseline =
        serde_json::from_str(&baseline_json).expect("Failed to parse Python baseline JSON");

    println!(
        "Loaded baseline with {} hours of profile data",
        baseline.profiles.solar_profile.len()
    );
    println!("\nConfiguration:");
    println!("  Solar: {} MW", baseline.config.solar_capacity);
    println!("  Wind: {} MW", baseline.config.wind_capacity);
    println!("  Storage: {} MWh", baseline.config.storage_capacity);
    println!("  Clean Firm: {} MW", baseline.config.clean_firm_capacity);
    println!("  Battery Efficiency: {}", baseline.config.battery_eff);

    // Run comparisons for all 3 modes
    let results = vec![
        run_comparison(
            "default",
            BatteryMode::Default,
            &baseline.config,
            &baseline.profiles,
            &baseline.results.default,
        ),
        run_comparison(
            "peak_shaver",
            BatteryMode::PeakShaver,
            &baseline.config,
            &baseline.profiles,
            &baseline.results.peak_shaver,
        ),
        run_comparison(
            "hybrid",
            BatteryMode::Hybrid,
            &baseline.config,
            &baseline.profiles,
            &baseline.results.hybrid,
        ),
    ];

    // Summary
    println!("\n\n========== SUMMARY ==========");
    for result in &results {
        let status = if result.differences.charge_diff_pct < 1.0
            && result.differences.discharge_diff_pct < 1.0
            && result.differences.gas_diff_pct < 1.0
        {
            "PASS"
        } else {
            "FAIL"
        };
        println!(
            "{}: {} (charge: {:.1}%, discharge: {:.1}%, gas: {:.1}%)",
            result.mode.to_uppercase(),
            status,
            result.differences.charge_diff_pct,
            result.differences.discharge_diff_pct,
            result.differences.gas_diff_pct,
        );
    }

    // Save detailed results
    let output_path = "../rust_comparison_results.json";
    let output_json = serde_json::to_string_pretty(&results).unwrap();
    fs::write(output_path, output_json).expect("Failed to write comparison results");
    println!("\nDetailed results saved to {}", output_path);
}
