/// Quick benchmark for V2 optimizer speed
use energy_simulator::{
    run_v2_optimizer, BatteryMode, CostParams, EmpiricalModel, OptimizerConfig, HOURS_PER_YEAR,
};
use std::fs;
use std::time::Instant;

fn create_test_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
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

    let load = vec![100.0; HOURS_PER_YEAR];
    (solar, wind, load)
}

fn load_model(path: &str) -> Option<EmpiricalModel> {
    match fs::read(path) {
        Ok(bytes) => match EmpiricalModel::from_bytes(&bytes) {
            Ok(model) => Some(model),
            Err(e) => {
                eprintln!("Failed to parse model: {}", e);
                None
            }
        },
        Err(_) => None,
    }
}

fn main() {
    println!("V2 Optimizer Speed Benchmark");
    println!("============================\n");

    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    // Try to load empirical model
    let model = load_model("models/california_hybrid.bin");
    let model_ref = model.as_ref();

    if model.is_some() {
        println!("Loaded empirical model: california_hybrid.bin");
    } else {
        println!("No empirical model found, using greedy fallback");
    }

    // Warmup
    println!("\nWarming up...");
    let _ = run_v2_optimizer(
        50.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        model_ref,
    );

    // Single optimization benchmarks
    println!("\nSingle Optimization Times:");
    for target in [30.0, 50.0, 70.0, 90.0] {
        let start = Instant::now();
        let result = run_v2_optimizer(
            target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            model_ref,
        )
        .unwrap();
        let elapsed = start.elapsed();

        println!(
            "  Target {:2}%: {:>6.0}ms (achieved {:.1}%, {} evals, LCOE ${:.1})",
            target as i32,
            elapsed.as_millis(),
            result.achieved_clean_match,
            result.num_evaluations,
            result.lcoe
        );
    }

    // Full sweep benchmark
    println!("\nFull Sweep (0-100% in 10% steps):");
    let start = Instant::now();
    let mut total_evals = 0u32;
    for target in (0..=100).step_by(10) {
        let result = run_v2_optimizer(
            target as f64,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
            model_ref,
        )
        .unwrap();
        total_evals += result.num_evaluations;
    }
    let elapsed = start.elapsed();
    println!(
        "  11 optimizations: {:.0}ms total ({:.0}ms avg), {} total evals",
        elapsed.as_millis(),
        elapsed.as_millis() as f64 / 11.0,
        total_evals
    );

    // Simulation-only benchmark for reference
    println!("\nReference - Single Simulation Time:");
    let sim_config = energy_simulator::SimulationConfig::new(
        100.0,
        100.0,
        100.0,
        50.0,
        0.85,
        0.0,
        BatteryMode::Hybrid,
    );
    let start = Instant::now();
    for _ in 0..100 {
        let _ = energy_simulator::simulate_system(&sim_config, &solar, &wind, &load);
    }
    let elapsed = start.elapsed();
    println!(
        "  100 simulations: {:.0}ms ({:.2}ms each)",
        elapsed.as_millis(),
        elapsed.as_millis() as f64 / 100.0
    );
}
