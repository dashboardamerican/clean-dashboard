//! ELCC Explorer - Test different portfolio configurations and analyze diversity benefits

use energy_simulator::economics::elcc::calculate_elcc;
use energy_simulator::types::BatteryMode;

fn main() {
    // Load California zone data (using simple synthetic profiles for testing)
    let hours = 8760;

    // Create realistic-ish profiles
    // Solar: peaks midday, zero at night
    let solar_profile: Vec<f64> = (0..hours)
        .map(|h| {
            let hour_of_day = h % 24;
            if hour_of_day >= 6 && hour_of_day <= 18 {
                let peak_hour = 12.0;
                let width = 6.0;
                let x = (hour_of_day as f64 - peak_hour) / width;
                (1.0 - x * x).max(0.0) * 0.8
            } else {
                0.0
            }
        })
        .collect();

    // Wind: inverse correlation with solar (stronger at night/evening)
    let wind_profile: Vec<f64> = (0..hours)
        .map(|h| {
            let hour_of_day = h % 24;
            let day_of_year = h / 24;
            // Base wind with some daily/seasonal variation
            let base = 0.35;
            let daily_var = if hour_of_day >= 18 || hour_of_day <= 6 {
                0.15
            } else {
                -0.1
            };
            let seasonal = 0.1 * ((day_of_year as f64 * 2.0 * std::f64::consts::PI / 365.0).sin());
            (base + daily_var + seasonal).max(0.05).min(0.6)
        })
        .collect();

    // Load: peaks in afternoon/evening
    let load_profile: Vec<f64> = (0..hours)
        .map(|h| {
            let hour_of_day = h % 24;
            let base = 80.0;
            let daily_shape = match hour_of_day {
                0..=5 => -20.0,
                6..=8 => 0.0,
                9..=11 => 10.0,
                12..=14 => 15.0,
                15..=19 => 20.0, // Peak demand
                20..=22 => 10.0,
                _ => -10.0,
            };
            base + daily_shape
        })
        .collect();

    println!("=== ELCC Portfolio Analysis ===\n");

    // Test scenarios
    let scenarios = vec![
        ("Solar only (100 MW)", 100.0, 0.0, 0.0, 0.0),
        ("Wind only (100 MW)", 0.0, 100.0, 0.0, 0.0),
        ("Storage only (100 MWh)", 0.0, 0.0, 100.0, 0.0),
        ("Clean Firm only (50 MW)", 0.0, 0.0, 0.0, 50.0),
        ("Solar + Wind (100 each)", 100.0, 100.0, 0.0, 0.0),
        ("Solar + Storage (100/100)", 100.0, 0.0, 100.0, 0.0),
        ("Wind + Storage (100/100)", 0.0, 100.0, 100.0, 0.0),
        ("Solar + Wind + Storage", 100.0, 100.0, 100.0, 0.0),
        ("Full portfolio", 100.0, 100.0, 100.0, 50.0),
        ("Heavy solar (200 MW)", 200.0, 50.0, 100.0, 0.0),
        ("Heavy wind (200 MW)", 50.0, 200.0, 100.0, 0.0),
        ("Large storage (400 MWh)", 100.0, 100.0, 400.0, 0.0),
    ];

    println!(
        "{:<30} {:>10} {:>10} {:>12} {:>12}",
        "Scenario", "Port ELCC", "Div Benef", "Solar ELCC", "Storage ELCC"
    );
    println!("{}", "-".repeat(76));

    for (name, solar, wind, storage, cf) in scenarios {
        match calculate_elcc(
            solar,
            wind,
            storage,
            cf,
            &solar_profile,
            &wind_profile,
            &load_profile,
            BatteryMode::Hybrid,
            0.85,
            0.0,
        ) {
            Ok(result) => {
                println!(
                    "{:<30} {:>10.1} {:>+10.1} {:>12.1}% {:>12.1}%",
                    name,
                    result.portfolio_elcc_mw,
                    result.diversity_benefit_mw,
                    result.solar.contribution,
                    result.storage.contribution,
                );
            }
            Err(e) => println!("{}: Error - {}", name, e),
        }
    }

    println!("\n=== Deep Dive: Solar + Storage Synergy ===\n");

    // Test how storage amplifies solar value
    println!(
        "{:<20} {:>12} {:>12} {:>12} {:>12}",
        "Storage (MWh)", "Port ELCC", "Div Benefit", "Solar Contrib", "Stor Contrib"
    );
    println!("{}", "-".repeat(72));

    for storage in [0, 50, 100, 200, 400, 800].iter() {
        if let Ok(result) = calculate_elcc(
            100.0,
            0.0,
            *storage as f64,
            0.0,
            &solar_profile,
            &wind_profile,
            &load_profile,
            BatteryMode::Hybrid,
            0.85,
            0.0,
        ) {
            println!(
                "{:<20} {:>12.1} {:>+12.1} {:>12.1}% {:>12.1}%",
                storage,
                result.portfolio_elcc_mw,
                result.diversity_benefit_mw,
                result.solar.contribution,
                result.storage.contribution,
            );
        }
    }

    println!("\n=== ELCC Methods Comparison (100 solar + 100 wind + 100 storage) ===\n");

    if let Ok(result) = calculate_elcc(
        100.0,
        100.0,
        100.0,
        0.0,
        &solar_profile,
        &wind_profile,
        &load_profile,
        BatteryMode::Hybrid,
        0.85,
        0.0,
    ) {
        println!(
            "{:<12} {:>12} {:>12} {:>12} {:>12}",
            "Resource", "First-In", "Marginal", "Contribution", "Delta"
        );
        println!("{}", "-".repeat(62));
        println!(
            "{:<12} {:>12.1}% {:>12.1}% {:>12.1}% {:>12.1}%",
            "Solar",
            result.solar.first_in,
            result.solar.marginal,
            result.solar.contribution,
            result.solar.delta
        );
        println!(
            "{:<12} {:>12.1}% {:>12.1}% {:>12.1}% {:>12.1}%",
            "Wind",
            result.wind.first_in,
            result.wind.marginal,
            result.wind.contribution,
            result.wind.delta
        );
        println!(
            "{:<12} {:>12.1}% {:>12.1}% {:>12.1}% {:>12.1}%",
            "Storage",
            result.storage.first_in,
            result.storage.marginal,
            result.storage.contribution,
            result.storage.delta
        );

        println!("\nPortfolio ELCC: {:.1} MW", result.portfolio_elcc_mw);
        println!("Diversity Benefit: {:+.1} MW", result.diversity_benefit_mw);
        println!("Baseline Peak Gas: {:.1} MW", result.baseline_peak_gas);
        println!("Portfolio Peak Gas: {:.1} MW", result.portfolio_peak_gas);
    }

    println!("\n=== Solar vs Wind: Correlation Analysis ===\n");

    // Compare solar-heavy vs wind-heavy portfolios
    let test_cases = vec![
        ("All Solar (200 MW)", 200.0, 0.0),
        ("Solar-heavy (150/50)", 150.0, 50.0),
        ("Balanced (100/100)", 100.0, 100.0),
        ("Wind-heavy (50/150)", 50.0, 150.0),
        ("All Wind (200 MW)", 0.0, 200.0),
    ];

    println!(
        "{:<25} {:>12} {:>12} {:>14}",
        "Mix", "Port ELCC", "Div Benefit", "ELCC/Capacity"
    );
    println!("{}", "-".repeat(65));

    for (name, solar, wind) in test_cases {
        if let Ok(result) = calculate_elcc(
            solar,
            wind,
            100.0,
            0.0,
            &solar_profile,
            &wind_profile,
            &load_profile,
            BatteryMode::Hybrid,
            0.85,
            0.0,
        ) {
            let total_cap = solar + wind + 100.0; // +100 for storage
            let elcc_ratio = result.portfolio_elcc_mw / total_cap * 100.0;
            println!(
                "{:<25} {:>12.1} {:>+12.1} {:>13.1}%",
                name, result.portfolio_elcc_mw, result.diversity_benefit_mw, elcc_ratio,
            );
        }
    }
}
