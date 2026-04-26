/// Direct benchmark binary for Rust simulation performance testing
/// This uses the same California zone data as the Python benchmark
use energy_simulator::{
    calculate_lcoe, simulate_system, BatteryMode, CostParams, SimulationConfig,
};
use std::io::BufRead;
use std::time::Instant;

fn main() {
    // Load California zone data from the CSV file
    let csv_path =
        "/Users/nathaniyer/Desktop/Multi-Heatmap Test/normalized_data_2035_by_zone_new.csv";

    println!("Loading California zone data from {}...", csv_path);
    let (solar, wind, load) = load_california_data(csv_path);
    println!("Loaded {} hours of data", solar.len());

    // Verify data looks reasonable
    let solar_mean: f64 = solar.iter().sum::<f64>() / solar.len() as f64;
    let wind_mean: f64 = wind.iter().sum::<f64>() / wind.len() as f64;
    let load_mean: f64 = load.iter().sum::<f64>() / load.len() as f64;
    println!("  Solar CF mean: {:.3}", solar_mean);
    println!("  Wind CF mean: {:.3}", wind_mean);
    println!("  Load mean: {:.1} MW", load_mean);

    let iterations = 100;

    // Test scenarios (matching Python benchmark)
    let scenarios = vec![
        (
            "Basic (Default mode)",
            100.0,
            100.0,
            50.0,
            20.0,
            BatteryMode::Default,
        ),
        (
            "High Storage (Default)",
            150.0,
            100.0,
            200.0,
            30.0,
            BatteryMode::Default,
        ),
        (
            "Peak Shaver mode",
            100.0,
            100.0,
            100.0,
            20.0,
            BatteryMode::PeakShaver,
        ),
        (
            "Hybrid mode",
            100.0,
            100.0,
            100.0,
            20.0,
            BatteryMode::Hybrid,
        ),
    ];

    println!("\n{}", "=".repeat(60));
    println!("RUST BENCHMARK (NATIVE)");
    println!("{}", "=".repeat(60));

    for (name, solar_cap, wind_cap, storage_cap, cf_cap, mode) in scenarios {
        let mut config = SimulationConfig::with_defaults();
        config.solar_capacity = solar_cap;
        config.wind_capacity = wind_cap;
        config.storage_capacity = storage_cap;
        config.clean_firm_capacity = cf_cap;
        config.battery_mode = mode;

        println!("\nScenario: {}", name);

        // Warmup
        print!("  Warming up...");
        for _ in 0..3 {
            let _ = simulate_system(&config, &solar, &wind, &load);
        }
        println!(" done");

        // Timed runs
        print!("  Running {} iterations...", iterations);
        let mut times: Vec<f64> = Vec::with_capacity(iterations);

        for _ in 0..iterations {
            let start = Instant::now();
            let _ = simulate_system(&config, &solar, &wind, &load);
            let elapsed = start.elapsed().as_secs_f64() * 1000.0; // ms
            times.push(elapsed);
        }
        println!(" done");

        // Calculate statistics
        times.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean = times.iter().sum::<f64>() / times.len() as f64;
        let median = times[times.len() / 2];
        let min = times[0];
        let max = times[times.len() - 1];

        println!("  Mean:   {:.3} ms", mean);
        println!("  Median: {:.3} ms", median);
        println!("  Min:    {:.3} ms", min);
        println!("  Max:    {:.3} ms", max);
    }

    // Also test LCOE calculation
    println!("\n{}", "-".repeat(40));
    println!("LCOE Calculation benchmark:");

    let mut config = SimulationConfig::with_defaults();
    config.solar_capacity = 100.0;
    config.wind_capacity = 100.0;
    config.storage_capacity = 50.0;
    config.clean_firm_capacity = 20.0;

    let sim_result = simulate_system(&config, &solar, &wind, &load).unwrap();
    let costs = CostParams::default_costs();

    print!("  Running {} iterations...", iterations);
    let mut times: Vec<f64> = Vec::with_capacity(iterations);

    for _ in 0..iterations {
        let start = Instant::now();
        let _ = calculate_lcoe(
            &sim_result,
            config.solar_capacity,
            config.wind_capacity,
            config.storage_capacity,
            config.clean_firm_capacity,
            &costs,
        );
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        times.push(elapsed);
    }
    println!(" done");

    times.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let mean = times.iter().sum::<f64>() / times.len() as f64;
    let median = times[times.len() / 2];
    println!("  Mean:   {:.3} ms", mean);
    println!("  Median: {:.3} ms", median);
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
            continue; // Skip header
        }

        // Format: timestamp,zone,normalized_wind,normalized_solar,normalized_load,normalized_net_demand
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() >= 5 && parts[1] == "California" {
            let wind_val: f64 = parts[2].parse().unwrap_or(0.0);
            let solar_val: f64 = parts[3].parse().unwrap_or(0.0);
            let load_val: f64 = parts[4].parse().unwrap_or(0.0);

            solar.push(solar_val);
            wind.push(wind_val);
            load.push(load_val);
        }
    }

    // Normalize load to average 100 MW (same as Python)
    let load_sum: f64 = load.iter().sum();
    let target_sum = 100.0 * 8760.0;
    let scale = target_sum / load_sum;
    load.iter_mut().for_each(|x| *x *= scale);

    (solar, wind, load)
}
