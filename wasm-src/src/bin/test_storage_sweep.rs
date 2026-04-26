//! Test how peak gas changes with storage capacity in Rust

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
    let baseline_json = fs::read_to_string(baseline_path)
        .expect("Failed to read Python baseline. Run compare_implementations.py first.");

    let baseline: PythonBaseline =
        serde_json::from_str(&baseline_json).expect("Failed to parse baseline JSON");

    println!("Peak Shaver Mode - Storage vs Peak Gas (Rust):");
    println!("{}", "=".repeat(70));

    for storage in [0, 5, 25, 50, 85, 120] {
        let config = SimulationConfig {
            solar_capacity: 200.0,
            wind_capacity: 100.0,
            storage_capacity: storage as f64,
            clean_firm_capacity: 0.0,
            battery_efficiency: 0.85,
            max_demand_response: 0.0,
            battery_mode: BatteryMode::PeakShaver,
        };

        let result = simulate_system(
            &config,
            &baseline.profiles.solar_profile,
            &baseline.profiles.wind_profile,
            &baseline.profiles.load_profile,
        )
        .expect("Simulation failed");

        let peak_gas = result.gas_generation.iter().cloned().fold(0.0, f64::max);
        let total_gas: f64 = result.gas_generation.iter().sum();
        let total_discharge: f64 = result.battery_discharge.iter().sum();

        println!(
            "Storage: {:3} MWh -> Peak Gas: {:6.2} MW, Total Gas: {:6.0} MWh, Discharge: {:4.0} MWh",
            storage, peak_gas, total_gas, total_discharge
        );
    }
}
