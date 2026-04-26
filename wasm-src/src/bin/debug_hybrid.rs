//! Debug hybrid mode to understand why transitions aren't being processed

use energy_simulator::simulation::core::simulate_system;
use energy_simulator::types::{BatteryMode, SimulationConfig};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
struct PythonBaseline {
    config: Config,
    profiles: Profiles,
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

fn main() {
    println!("Loading baseline...");
    let baseline_json = fs::read_to_string("../python_baseline_results.json").unwrap();
    let baseline: PythonBaseline = serde_json::from_str(&baseline_json).unwrap();

    let n = baseline.profiles.solar_profile.len();
    println!("Loaded {} hours of data", n);

    // Calculate what y_data should look like
    let clean_firm = baseline.config.clean_firm_capacity;
    let mut y_data_positive = 0;
    let mut y_data_negative = 0;
    let mut y_data_zero = 0;
    let mut transitions = 0;

    for i in 0..n {
        let total_renewable = baseline.profiles.solar_profile[i] * baseline.config.solar_capacity
            + baseline.profiles.wind_profile[i] * baseline.config.wind_capacity
            + clean_firm;
        let net_demand = baseline.profiles.load_profile[i] - total_renewable;

        if net_demand > 0.0 {
            y_data_positive += 1;
        } else if net_demand < 0.0 {
            y_data_negative += 1;
        } else {
            y_data_zero += 1;
        }

        // Count transitions
        if i > 0 {
            let prev_renewable = baseline.profiles.solar_profile[i - 1]
                * baseline.config.solar_capacity
                + baseline.profiles.wind_profile[i - 1] * baseline.config.wind_capacity
                + clean_firm;
            let prev_net = baseline.profiles.load_profile[i - 1] - prev_renewable;

            // Transition from renewable (<=0) to gas (>0)
            if prev_net <= 0.0 && net_demand > 0.0 {
                transitions += 1;
                if transitions <= 10 {
                    println!(
                        "Transition at hour {}: prev_net={:.2}, net_demand={:.2}",
                        i, prev_net, net_demand
                    );
                }
            }
        }
    }

    println!("\nPre-battery y_data distribution:");
    println!("  Positive (gas hours): {}", y_data_positive);
    println!("  Negative (renewable excess): {}", y_data_negative);
    println!("  Zero: {}", y_data_zero);
    println!("  Transitions (renewable->gas): {}", transitions);

    // Now run peak_shaver and see what the post-battery y_data looks like
    println!("\n=== Running Peak Shaver ===");
    let sim_config = SimulationConfig {
        solar_capacity: baseline.config.solar_capacity,
        wind_capacity: baseline.config.wind_capacity,
        storage_capacity: baseline.config.storage_capacity,
        clean_firm_capacity: baseline.config.clean_firm_capacity,
        battery_efficiency: baseline.config.battery_eff,
        max_demand_response: baseline.config.max_demand_response,
        battery_mode: BatteryMode::PeakShaver,
    };

    let result = simulate_system(
        &sim_config,
        &baseline.profiles.solar_profile,
        &baseline.profiles.wind_profile,
        &baseline.profiles.load_profile,
    )
    .unwrap();

    // Calculate post-battery y_data
    let mut post_positive = 0;
    let mut post_negative = 0;
    let mut post_transitions = 0;

    for i in 0..n {
        let total_renewable = baseline.profiles.solar_profile[i] * baseline.config.solar_capacity
            + baseline.profiles.wind_profile[i] * baseline.config.wind_capacity
            + clean_firm;
        let net_demand = baseline.profiles.load_profile[i] - total_renewable;

        // Post-battery y_data
        let y_data = if net_demand > 0.0 {
            result.gas_generation[i]
        } else if net_demand < 0.0 {
            -result.curtailed[i]
        } else {
            0.0
        };

        if y_data > 0.0 {
            post_positive += 1;
        } else if y_data < 0.0 {
            post_negative += 1;
        }

        if i > 0 {
            let prev_renewable = baseline.profiles.solar_profile[i - 1]
                * baseline.config.solar_capacity
                + baseline.profiles.wind_profile[i - 1] * baseline.config.wind_capacity
                + clean_firm;
            let prev_net = baseline.profiles.load_profile[i - 1] - prev_renewable;

            let prev_y = if prev_net > 0.0 {
                result.gas_generation[i - 1]
            } else if prev_net < 0.0 {
                -result.curtailed[i - 1]
            } else {
                0.0
            };

            if prev_y <= 0.0 && y_data > 0.0 {
                post_transitions += 1;
                if post_transitions <= 10 {
                    println!(
                        "Post-battery transition at hour {}: prev_y={:.2}, y={:.2}",
                        i, prev_y, y_data
                    );
                    println!(
                        "  gas_gen[{}]={:.2}, curtailed[{}]={:.2}",
                        i - 1,
                        result.gas_generation[i - 1],
                        i - 1,
                        result.curtailed[i - 1]
                    );
                    println!(
                        "  gas_gen[{}]={:.2}, curtailed[{}]={:.2}",
                        i, result.gas_generation[i], i, result.curtailed[i]
                    );
                }
            }
        }
    }

    println!("\nPost-battery y_data distribution:");
    println!("  Positive (gas hours): {}", post_positive);
    println!("  Negative (renewable excess): {}", post_negative);
    println!("  Transitions: {}", post_transitions);

    println!("\nPeak Shaver totals:");
    println!(
        "  Total charge: {:.2} MWh",
        result.battery_charge.iter().sum::<f64>()
    );
    println!(
        "  Total discharge: {:.2} MWh",
        result.battery_discharge.iter().sum::<f64>()
    );
    println!(
        "  Total curtailed: {:.2} MWh",
        result.curtailed.iter().sum::<f64>()
    );
    println!(
        "  Total gas: {:.2} MWh",
        result.gas_generation.iter().sum::<f64>()
    );

    // Show first 50 hours
    println!("\nFirst 50 hours detail:");
    println!(
        "{:>5} {:>10} {:>10} {:>10} {:>10} {:>10}",
        "Hour", "Load", "Renewable", "Gas", "Curtail", "y_data"
    );
    for i in 0..50 {
        let total_renewable = baseline.profiles.solar_profile[i] * baseline.config.solar_capacity
            + baseline.profiles.wind_profile[i] * baseline.config.wind_capacity;
        let net_demand = baseline.profiles.load_profile[i] - total_renewable;
        let y_data = if net_demand > 0.0 {
            result.gas_generation[i]
        } else if net_demand < 0.0 {
            -result.curtailed[i]
        } else {
            0.0
        };
        println!(
            "{:>5} {:>10.2} {:>10.2} {:>10.2} {:>10.2} {:>10.2}",
            i,
            baseline.profiles.load_profile[i],
            total_renewable,
            result.gas_generation[i],
            result.curtailed[i],
            y_data
        );
    }
}
