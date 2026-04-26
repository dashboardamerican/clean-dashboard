/// Quick benchmark comparing V2 and V3 optimizer paths
use energy_simulator::{
    run_v2_optimizer, run_v3_optimizer, BatteryMode, CostParams, EmpiricalModel, OptimizerConfig,
    HOURS_PER_YEAR,
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
                eprintln!("Failed to parse model {}: {}", path, e);
                None
            }
        },
        Err(_) => None,
    }
}

fn time<T>(mut f: impl FnMut() -> T) -> (T, u128) {
    let start = Instant::now();
    let value = f();
    (value, start.elapsed().as_millis())
}

fn run_suite() -> Result<(), String> {
    println!("V2 vs V3 Optimizer Benchmark");
    println!("==============================\n");

    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    // Try to load empirical model for V2 baseline
    let model = load_model("models/california_hybrid.bin");
    let model_ref = model.as_ref();

    if model.is_some() {
        println!("Loaded empirical model: california_hybrid.bin");
    } else {
        println!("No empirical model found, using greedy fallback for V2");
    }

    // Warmup both paths
    let _ = run_v2_optimizer(
        50.0,
        &solar,
        &wind,
        &load,
        &costs,
        &config,
        BatteryMode::Hybrid,
        model_ref,
    )
    .map_err(|e| format!("V2 warmup failed: {}", e))?;

    let _ = run_v3_optimizer(
        50.0,
        &solar,
        &wind,
        &load,
        &costs,
        BatteryMode::Hybrid,
        &config,
    )
    .map_err(|e| format!("V3 warmup failed: {}", e))?;

    // Targeted comparisons
    println!("Single Optimization Times:");
    for target in [30.0, 50.0, 70.0, 90.0, 95.0] {
        let (v2, v2_ms) = time(|| {
            run_v2_optimizer(
                target,
                &solar,
                &wind,
                &load,
                &costs,
                &config,
                BatteryMode::Hybrid,
                model_ref,
            )
        });
        let (v3, v3_ms) = time(|| {
            run_v3_optimizer(
                target,
                &solar,
                &wind,
                &load,
                &costs,
                BatteryMode::Hybrid,
                &config,
            )
        });

        let (v2, v3) = match (v2, v3) {
            (Ok(v2), Ok(v3)) => (Some(v2), Some(v3)),
            (Err(err), Ok(v3)) => {
                println!("  Target {:>3.0}%: V2 failed: {}", target, err);
                (None, Some(v3))
            }
            (Ok(v2), Err(err)) => {
                println!("  Target {:>3.0}%: V3 failed: {}", target, err);
                (Some(v2), None)
            }
            (Err(v2_err), Err(v3_err)) => {
                println!(
                    "  Target {:>3.0}%: V2 failed: {} | V3 failed: {}",
                    target, v2_err, v3_err
                );
                (None, None)
            }
        };

        if let (Some(v2), Some(v3)) = (v2, v3) {
            let ratio = v3_ms as f64 / v2_ms as f64;
            println!(
                "  Target {:>3.0}%: V2={:>5}ms | V3={:>5}ms | ratio={:>4.2} | lcoe(v2={:.2}, v3={:.2}) | evals(v2={}, v3={})",
                target,
                v2_ms,
                v3_ms,
                ratio,
                v2.lcoe,
                v3.lcoe,
                v2.num_evaluations,
                v3.num_evaluations
            );
        }
    }

    // Full sweep
    println!("\nFull Sweep (0-100% in 10% steps):");
    let mut v2_total_ms = 0u128;
    let mut v3_total_ms = 0u128;
    let mut v2_total_evals = 0u32;
    let mut v3_total_evals = 0u32;
    let mut sweep_count = 0u32;

    for target in (0..=100).step_by(10) {
        let target = target as f64;
        let (v2_result, v2_ms) = time(|| {
            run_v2_optimizer(
                target,
                &solar,
                &wind,
                &load,
                &costs,
                &config,
                BatteryMode::Hybrid,
                model_ref,
            )
        });
        let (v3_result, v3_ms) = time(|| {
            run_v3_optimizer(
                target,
                &solar,
                &wind,
                &load,
                &costs,
                BatteryMode::Hybrid,
                &config,
            )
        });

        if v2_result.is_err() || v3_result.is_err() {
            println!(
                "  Sweep target {:>3.0}% skipped (V2 error: {:?}, V3 error: {:?})",
                target,
                v2_result.as_ref().err(),
                v3_result.as_ref().err()
            );
            continue;
        }

        let v2_result = v2_result.unwrap();
        let v3_result = v3_result.unwrap();
        v2_total_ms += v2_ms;
        v3_total_ms += v3_ms;
        sweep_count += 1;

        // Lightweight eval totals for rough throughput comparison
        v2_total_evals += v2_result.num_evaluations;
        v3_total_evals += v3_result.num_evaluations;
    }

    println!(
        "  V2: {:>6}ms total ({:>6.1}ms avg), {} total evals",
        v2_total_ms,
        v2_total_ms as f64 / sweep_count.max(1) as f64,
        v2_total_evals
    );
    println!(
        "  V3: {:>6}ms total ({:>6.1}ms avg), {} total evals",
        v3_total_ms,
        v3_total_ms as f64 / sweep_count.max(1) as f64,
        v3_total_evals
    );
    if v3_total_ms > 0 {
        println!(
            "  Sweep ratio (V3/V2): {:>4.2}x",
            v3_total_ms as f64 / v2_total_ms as f64
        );
    }

    Ok(())
}

fn main() {
    if let Err(e) = run_suite() {
        eprintln!("Benchmark failed: {}", e);
        std::process::exit(1);
    }
}
