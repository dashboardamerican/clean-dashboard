/// Benchmark: Model-Based vs Greedy Optimization
///
/// Compares timing and results of optimization with and without
/// the EmpiricalModel lookup table.
///
/// Run with:
///   cargo run --release --bin bench_model
use energy_simulator::{
    run_v2_optimizer, BatteryMode, CostParams, EmpiricalModel, OptimizerConfig,
};
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

const MODEL_PATH: &str = "models/california_hybrid.bin";
const ZONES_PATH: &str = "../data/zones.json";

fn load_california_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    let content = fs::read_to_string(ZONES_PATH).expect("Failed to read zones.json");

    #[derive(serde::Deserialize)]
    struct ZoneJson {
        solar: Vec<f64>,
        wind: Vec<f64>,
        load: Vec<f64>,
    }

    let raw: HashMap<String, ZoneJson> =
        serde_json::from_str(&content).expect("Failed to parse JSON");

    let zone = raw.get("California").expect("California not found");
    (zone.solar.clone(), zone.wind.clone(), zone.load.clone())
}

fn main() {
    println!("=== Model-Based Optimization Benchmark ===\n");

    // Load profiles
    let (solar, wind, load) = load_california_profiles();
    println!("Loaded California profiles: {} hours\n", solar.len());

    // Load model (if available)
    let model = if std::path::Path::new(MODEL_PATH).exists() {
        let bytes = fs::read(MODEL_PATH).expect("Failed to read model");
        let m = EmpiricalModel::from_bytes(&bytes).expect("Failed to deserialize model");

        // Verify data is present
        println!("Model has gas data: {}", m.has_gas_data());
        println!("Model has gas gen data: {}", m.has_gas_gen_data());

        // Show a few sample predictions
        println!("\nSample predictions:");
        println!(
            "  {:40} {:>8} {:>10} {:>12}",
            "Portfolio", "Match %", "Gas MW", "Gas MWh/yr"
        );
        println!("  {}", "-".repeat(70));
        for (s, w, st, cf) in [
            (0.0, 0.0, 0.0, 0.0),
            (500.0, 200.0, 800.0, 0.0),
            (500.0, 200.0, 800.0, 50.0),
        ] {
            println!(
                "  (S={}, W={}, St={}, CF={}): {:>6.1}% {:>8.1} {:>10.0}",
                s,
                w,
                st,
                cf,
                m.predict(s, w, st, cf),
                m.predict_gas(s, w, st, cf),
                m.predict_gas_gen(s, w, st, cf)
            );
        }
        println!();

        Some(m)
    } else {
        println!("Warning: Model not found at {}", MODEL_PATH);
        None
    };

    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    // Test targets
    let targets = vec![30.0, 50.0, 70.0, 80.0, 90.0, 95.0, 99.0];

    println!("=== Single Target Optimization ===\n");
    println!(
        "{:>8} {:>12} {:>12} {:>10} {:>10} {:>10}",
        "Target", "No Model", "With Model", "Speedup", "LCOE Diff", "Match Diff"
    );
    println!("{}", "-".repeat(72));

    let mut total_no_model = 0.0;
    let mut total_with_model = 0.0;

    for target in &targets {
        // Without model
        let start = Instant::now();
        let result_no_model = run_v2_optimizer(
            *target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        )
        .unwrap();
        let time_no_model = start.elapsed().as_secs_f64() * 1000.0;
        total_no_model += time_no_model;

        // With model
        let start = Instant::now();
        let result_with_model = run_v2_optimizer(
            *target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            model.as_ref(),
        )
        .unwrap();
        let time_with_model = start.elapsed().as_secs_f64() * 1000.0;
        total_with_model += time_with_model;

        let speedup = time_no_model / time_with_model;
        let lcoe_diff = result_with_model.lcoe - result_no_model.lcoe;
        let match_diff =
            result_with_model.achieved_clean_match - result_no_model.achieved_clean_match;

        println!(
            "{:>7.0}% {:>10.2}ms {:>10.2}ms {:>9.2}x {:>9.2}$ {:>9.2}%",
            target, time_no_model, time_with_model, speedup, lcoe_diff, match_diff
        );

        // Show portfolio details for 90% target
        if *target == 90.0 && lcoe_diff.abs() > 0.1 {
            println!(
                "    Greedy:  S={:.0} W={:.0} St={:.0} CF={:.0} -> LCOE=${:.2} Match={:.1}%",
                result_no_model.solar_capacity,
                result_no_model.wind_capacity,
                result_no_model.storage_capacity,
                result_no_model.clean_firm_capacity,
                result_no_model.lcoe,
                result_no_model.achieved_clean_match
            );
            println!(
                "    Model:   S={:.0} W={:.0} St={:.0} CF={:.0} -> LCOE=${:.2} Match={:.1}%",
                result_with_model.solar_capacity,
                result_with_model.wind_capacity,
                result_with_model.storage_capacity,
                result_with_model.clean_firm_capacity,
                result_with_model.lcoe,
                result_with_model.achieved_clean_match
            );
        }
    }

    println!("{}", "-".repeat(72));
    println!(
        "{:>8} {:>10.2}ms {:>10.2}ms {:>9.2}x",
        "TOTAL",
        total_no_model,
        total_with_model,
        total_no_model / total_with_model
    );

    // Sweep benchmark
    println!("\n=== 11-Point Sweep Benchmark ===\n");

    let sweep_targets: Vec<f64> = (0..=10).map(|i| i as f64 * 10.0).collect();

    // Without model
    let start = Instant::now();
    for target in &sweep_targets {
        let _ = run_v2_optimizer(
            *target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            None,
        );
    }
    let sweep_no_model = start.elapsed().as_secs_f64() * 1000.0;

    // With model
    let start = Instant::now();
    for target in &sweep_targets {
        let _ = run_v2_optimizer(
            *target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            model.as_ref(),
        );
    }
    let sweep_with_model = start.elapsed().as_secs_f64() * 1000.0;

    println!("Without model: {:.2}ms", sweep_no_model);
    println!("With model:    {:.2}ms", sweep_with_model);
    println!("Speedup:       {:.2}x", sweep_no_model / sweep_with_model);

    // Warmup note
    println!("\n=== Notes ===");
    println!("- Times include first-run compilation overhead");
    println!("- Real-world usage after warmup will be faster");
    println!(
        "- Model size: {} KB",
        fs::metadata(MODEL_PATH)
            .map(|m| m.len() / 1024)
            .unwrap_or(0)
    );
}
