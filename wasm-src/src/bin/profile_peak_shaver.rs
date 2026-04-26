/// Profile the peak shaver algorithm to find the bottleneck
use energy_simulator::simulation::battery::{
    apply_peak_shaver_dispatch, find_optimal_peak_line, test_battery_line,
};
use std::io::BufRead;
use std::time::Instant;

fn main() {
    // Load California zone data
    let csv_path =
        "/Users/nathaniyer/Desktop/Multi-Heatmap Test/normalized_data_2035_by_zone_new.csv";
    let (solar, wind, load) = load_california_data(csv_path);

    // Calculate renewable_gen and gas_baseline like simulation does
    let solar_cap = 100.0;
    let wind_cap = 100.0;
    let clean_firm = 20.0;
    let storage_cap = 100.0;
    let battery_eff = 0.85;

    let n = 8760;
    let mut renewable_gen = vec![0.0; n];
    let mut gas_baseline = vec![0.0; n];
    let mut renewable_excess = vec![0.0; n];

    for i in 0..n {
        renewable_gen[i] = solar_cap * solar[i] + wind_cap * wind[i];
        let total_clean = renewable_gen[i] + clean_firm;
        if total_clean >= load[i] {
            renewable_excess[i] = total_clean - load[i];
        } else {
            gas_baseline[i] = load[i] - total_clean;
        }
    }

    let max_gas = gas_baseline.iter().cloned().fold(0.0, f64::max);
    let min_gas = gas_baseline
        .iter()
        .cloned()
        .filter(|&x| x > 0.0)
        .fold(f64::INFINITY, f64::min);
    println!("Gas baseline: min={:.2}, max={:.2}", min_gas, max_gas);

    // Profile test_battery_line (single call)
    println!("\nProfiling test_battery_line (single call at mid point)...");
    let test_line = (min_gas + max_gas) / 2.0;
    let start = Instant::now();
    let iterations = 1000;
    for _ in 0..iterations {
        let _ = test_battery_line(
            test_line,
            &gas_baseline,
            &renewable_excess,
            storage_cap,
            battery_eff,
        );
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    println!("  test_battery_line: {:.4} ms per call", elapsed);

    // Profile find_optimal_peak_line (binary search)
    println!("\nProfiling find_optimal_peak_line (full binary search)...");
    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let _ = find_optimal_peak_line(&gas_baseline, &renewable_excess, storage_cap, battery_eff);
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    println!("  find_optimal_peak_line: {:.4} ms per call", elapsed);

    // Profile full apply_peak_shaver_dispatch
    println!("\nProfiling apply_peak_shaver_dispatch (full function)...");
    let start = Instant::now();
    let iterations = 100;
    for _ in 0..iterations {
        let _ =
            apply_peak_shaver_dispatch(&renewable_gen, &load, clean_firm, storage_cap, battery_eff);
    }
    let elapsed = start.elapsed().as_secs_f64() * 1000.0 / iterations as f64;
    println!("  apply_peak_shaver_dispatch: {:.4} ms per call", elapsed);

    // Count how many hours have gas_baseline > 0
    let gas_hours = gas_baseline.iter().filter(|&&x| x > 0.0).count();
    println!("\nData characteristics:");
    println!("  Hours with gas need: {} / {}", gas_hours, n);
    println!(
        "  Hours with renewable excess: {}",
        renewable_excess.iter().filter(|&&x| x > 0.0).count()
    );
}

fn load_california_data(path: &str) -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let file = std::fs::File::open(path).expect("Failed to open CSV file");
    let reader = std::io::BufReader::new(file);

    let mut solar = Vec::with_capacity(8760);
    let mut wind = Vec::with_capacity(8760);
    let mut load = Vec::with_capacity(8760);

    let mut first_line = true;
    for line in reader.lines() {
        let line = line.expect("Failed to read line");
        if first_line {
            first_line = false;
            continue;
        }
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 5 && parts[1] == "California" {
            solar.push(parts[3].parse().unwrap_or(0.0));
            wind.push(parts[2].parse().unwrap_or(0.0));
            load.push(parts[4].parse().unwrap_or(0.0));
        }
    }

    let load_sum: f64 = load.iter().sum();
    let scale = 100.0 * 8760.0 / load_sum;
    load.iter_mut().for_each(|x| *x *= scale);

    (solar, wind, load)
}
