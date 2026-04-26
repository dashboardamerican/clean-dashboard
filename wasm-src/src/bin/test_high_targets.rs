use energy_simulator::{
    run_v1_optimizer, run_v2_optimizer, BatteryMode, CostParams, EmpiricalModel, OptimizerConfig,
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
        Ok(bytes) => EmpiricalModel::from_bytes(&bytes).ok(),
        Err(_) => None,
    }
}

fn main() {
    println!("High Target Comparison: V1 vs V2\n");

    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();
    let model = load_model("models/california_hybrid.bin");
    let model_ref = model.as_ref();

    for target in [95.0, 99.0] {
        println!("=== Target {}% ===", target);

        // V1 optimizer
        let start = Instant::now();
        let v1 = run_v1_optimizer(
            target,
            &solar,
            &wind,
            &load,
            &costs,
            &config,
            BatteryMode::Hybrid,
        )
        .unwrap();
        let v1_time = start.elapsed();

        // V2 optimizer with model
        let start = Instant::now();
        let v2 = run_v2_optimizer(
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
        let v2_time = start.elapsed();

        println!(
            "V1: {:.1}% match, LCOE ${:.2}, {} evals, {:?}",
            v1.achieved_clean_match, v1.lcoe, v1.num_evaluations, v1_time
        );
        println!(
            "    Solar={:.0}, Wind={:.0}, Storage={:.0}, CF={:.0}",
            v1.solar_capacity, v1.wind_capacity, v1.storage_capacity, v1.clean_firm_capacity
        );

        println!(
            "V2: {:.1}% match, LCOE ${:.2}, {} evals, {:?}",
            v2.achieved_clean_match, v2.lcoe, v2.num_evaluations, v2_time
        );
        println!(
            "    Solar={:.0}, Wind={:.0}, Storage={:.0}, CF={:.0}",
            v2.solar_capacity, v2.wind_capacity, v2.storage_capacity, v2.clean_firm_capacity
        );

        let lcoe_diff = v2.lcoe - v1.lcoe;
        println!(
            "LCOE difference: ${:.2}/MWh ({})",
            lcoe_diff.abs(),
            if lcoe_diff < 0.0 {
                "V2 better"
            } else if lcoe_diff > 0.0 {
                "V1 better"
            } else {
                "equal"
            }
        );
        println!();
    }
}
