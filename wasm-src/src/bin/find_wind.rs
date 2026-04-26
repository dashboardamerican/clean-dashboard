use energy_simulator::{
    calculate_lcoe, simulate_system, BatteryMode, CostParams, SimulationConfig, HOURS_PER_YEAR,
};

fn main() {
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
    let costs = CostParams::default_costs();

    println!("Finding wind capacity to hit 99% clean match:\n");

    for wind_mw in (320..=450).step_by(5) {
        let config = SimulationConfig {
            solar_capacity: 0.0,
            wind_capacity: wind_mw as f64,
            storage_capacity: 0.0,
            clean_firm_capacity: 0.0,
            battery_efficiency: 0.85,
            max_demand_response: 0.0,
            battery_mode: BatteryMode::Hybrid,
        };

        let sim = simulate_system(&config, &solar, &wind, &load).unwrap();
        let lcoe = calculate_lcoe(&sim, 0.0, wind_mw as f64, 0.0, 0.0, &costs);

        println!(
            "Wind {}: {:.2}% match, LCOE ${:.2}",
            wind_mw, sim.clean_match_pct, lcoe.total_lcoe
        );

        if sim.clean_match_pct >= 99.0 {
            println!(
                "\n--> Wind-only solution for 99%: {} MW at ${:.2}/MWh",
                wind_mw, lcoe.total_lcoe
            );
            break;
        }
    }

    // Also check V2's solution
    println!("\nV2's solution (Wind=325, CF=7.4):");
    let config = SimulationConfig {
        solar_capacity: 0.0,
        wind_capacity: 325.0,
        storage_capacity: 0.0,
        clean_firm_capacity: 7.4,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
        battery_mode: BatteryMode::Hybrid,
    };
    let sim = simulate_system(&config, &solar, &wind, &load).unwrap();
    let lcoe = calculate_lcoe(&sim, 0.0, 325.0, 0.0, 7.4, &costs);
    println!(
        "  {:.2}% match, LCOE ${:.2}",
        sim.clean_match_pct, lcoe.total_lcoe
    );
}
