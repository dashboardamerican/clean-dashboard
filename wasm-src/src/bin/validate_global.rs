//! Exhaustive grid search to validate optimizer results
use energy_simulator::{
    calculate_lcoe, run_v2_optimizer, simulate_system, BatteryMode, CostParams, EmpiricalModel,
    OptimizerConfig, SimulationConfig, HOURS_PER_YEAR,
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

fn evaluate(
    solar: f64,
    wind: f64,
    storage: f64,
    cf: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
) -> Option<(f64, f64)> {
    let config = SimulationConfig {
        solar_capacity: solar,
        wind_capacity: wind,
        storage_capacity: storage,
        clean_firm_capacity: cf,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
        battery_mode: BatteryMode::Hybrid,
    };

    let sim = simulate_system(&config, solar_profile, wind_profile, load_profile).ok()?;
    let lcoe = calculate_lcoe(&sim, solar, wind, storage, cf, costs);
    Some((sim.clean_match_pct, lcoe.total_lcoe))
}

fn main() {
    let target = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(95.0);

    let tolerance = 0.5; // Accept solutions within 0.5% of target

    println!("Exhaustive Search Validation for {}% target", target);
    println!("=========================================\n");

    let (solar_profile, wind_profile, load_profile) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();
    let model = load_model("models/california_hybrid.bin");
    let model_ref = model.as_ref();

    // First, get V2 result
    let start = Instant::now();
    let v2 = run_v2_optimizer(
        target,
        &solar_profile,
        &wind_profile,
        &load_profile,
        &costs,
        &config,
        BatteryMode::Hybrid,
        model_ref,
    )
    .unwrap();
    let v2_time = start.elapsed();

    println!("V2 Optimizer Result:");
    println!(
        "  Match: {:.2}%, LCOE: ${:.2}",
        v2.achieved_clean_match, v2.lcoe
    );
    println!(
        "  Portfolio: Solar={:.0}, Wind={:.0}, Storage={:.0}, CF={:.1}",
        v2.solar_capacity, v2.wind_capacity, v2.storage_capacity, v2.clean_firm_capacity
    );
    println!("  Time: {:?}, Evals: {}\n", v2_time, v2.num_evaluations);

    // Exhaustive search on a focused grid around where solutions likely are
    // For high targets, wind dominates, so focus there
    println!("Running exhaustive search...");
    let start = Instant::now();

    let mut best_lcoe = f64::INFINITY;
    let mut best_portfolio = (0.0, 0.0, 0.0, 0.0);
    let mut best_match = 0.0;
    let mut valid_count = 0u32;
    let mut total_count = 0u32;

    // Grid: Solar 0-200 by 25, Wind 0-500 by 25, Storage 0-400 by 50, CF 0-100 by 5
    for solar in (0..=200).step_by(25) {
        for wind in (0..=500).step_by(25) {
            for storage in (0..=400).step_by(50) {
                for cf in (0..=100).step_by(5) {
                    total_count += 1;

                    if let Some((clean_match, lcoe)) = evaluate(
                        solar as f64,
                        wind as f64,
                        storage as f64,
                        cf as f64,
                        &solar_profile,
                        &wind_profile,
                        &load_profile,
                        &costs,
                    ) {
                        // Check if it hits the target within tolerance
                        if (clean_match - target).abs() <= tolerance {
                            valid_count += 1;
                            if lcoe < best_lcoe {
                                best_lcoe = lcoe;
                                best_portfolio =
                                    (solar as f64, wind as f64, storage as f64, cf as f64);
                                best_match = clean_match;
                            }
                        }
                    }
                }
            }
        }
    }

    let exhaustive_time = start.elapsed();

    println!(
        "\nExhaustive Search Result ({} portfolios, {} valid):",
        total_count, valid_count
    );
    println!("  Match: {:.2}%, LCOE: ${:.2}", best_match, best_lcoe);
    println!(
        "  Portfolio: Solar={:.0}, Wind={:.0}, Storage={:.0}, CF={:.0}",
        best_portfolio.0, best_portfolio.1, best_portfolio.2, best_portfolio.3
    );
    println!("  Time: {:?}\n", exhaustive_time);

    // Compare
    let lcoe_gap = v2.lcoe - best_lcoe;
    println!("Comparison:");
    println!(
        "  LCOE gap: ${:.2}/MWh ({})",
        lcoe_gap.abs(),
        if lcoe_gap.abs() < 0.5 {
            "OPTIMAL"
        } else if lcoe_gap < 0.0 {
            "V2 BETTER than exhaustive!"
        } else {
            "V2 suboptimal"
        }
    );
    println!(
        "  V2 speedup: {:.0}x faster",
        exhaustive_time.as_secs_f64() / v2_time.as_secs_f64()
    );
}
