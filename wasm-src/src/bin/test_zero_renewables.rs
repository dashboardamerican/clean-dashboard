//! Test peak shaver with zero renewables

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
    let baseline_path = "../python_baseline_results.json";
    let baseline_json = fs::read_to_string(baseline_path).unwrap();
    let baseline: PythonBaseline = serde_json::from_str(&baseline_json).unwrap();

    println!("Peak Shaver - Zero Renewables (Rust):");
    println!("{}", "=".repeat(60));

    for storage in [0, 50, 100, 150] {
        let config = SimulationConfig {
            solar_capacity: 0.0, // Zero renewables
            wind_capacity: 0.0,
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
        .unwrap();

        let peak_gas = result.gas_generation.iter().cloned().fold(0.0, f64::max);
        let total_charge: f64 = result.battery_charge.iter().sum();
        let total_discharge: f64 = result.battery_discharge.iter().sum();
        let total_gas_charge: f64 = result.gas_for_charging.iter().sum();

        println!("Storage: {:3} MWh -> Peak Gas: {:.1} MW", storage, peak_gas);
        println!(
            "    Charge: {:.0} MWh, Discharge: {:.0} MWh",
            total_charge, total_discharge
        );
        println!("    Gas for charging: {:.0} MWh", total_gas_charge);
    }
}
