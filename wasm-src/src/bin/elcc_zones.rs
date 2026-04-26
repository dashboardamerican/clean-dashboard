//! ELCC across different zones - testing with real profile data

use energy_simulator::economics::elcc::calculate_elcc;
use energy_simulator::types::BatteryMode;
use std::fs;

fn main() {
    // Load zone data
    let zones_json =
        fs::read_to_string("../web/public/data/zones.json").expect("Failed to read zones.json");

    let zones: serde_json::Value =
        serde_json::from_str(&zones_json).expect("Failed to parse zones.json");

    let zone_names = ["California", "Texas", "Florida", "New England", "Northwest"];

    println!("=== ELCC by Zone (100 solar + 100 wind + 100 storage) ===\n");
    println!(
        "{:<15} {:>10} {:>12} {:>10} {:>10} {:>10}",
        "Zone", "Port ELCC", "Div Benefit", "Solar %", "Wind %", "Storage %"
    );
    println!("{}", "-".repeat(70));

    for zone_name in zone_names.iter() {
        if let Some(zone) = zones
            .as_array()
            .unwrap()
            .iter()
            .find(|z| z["name"] == *zone_name)
        {
            let solar: Vec<f64> = zone["solar_profile"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let wind: Vec<f64> = zone["wind_profile"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();
            let load: Vec<f64> = zone["load_profile"]
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_f64().unwrap())
                .collect();

            if let Ok(result) = calculate_elcc(
                100.0,
                100.0,
                100.0,
                0.0,
                &solar,
                &wind,
                &load,
                BatteryMode::Hybrid,
                0.85,
                0.0,
            ) {
                println!(
                    "{:<15} {:>10.1} {:>+12.1} {:>10.1} {:>10.1} {:>10.1}",
                    zone_name,
                    result.portfolio_elcc_mw,
                    result.diversity_benefit_mw,
                    result.solar.contribution,
                    result.wind.contribution,
                    result.storage.contribution,
                );
            }
        }
    }

    // Deep dive into Texas (ERCOT) - known for wind resources
    println!("\n=== Texas Deep Dive: Solar vs Wind Value ===\n");

    if let Some(zone) = zones
        .as_array()
        .unwrap()
        .iter()
        .find(|z| z["name"] == "Texas")
    {
        let solar: Vec<f64> = zone["solar_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let wind: Vec<f64> = zone["wind_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let load: Vec<f64> = zone["load_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();

        // Print solar/wind capacity factors
        let solar_cf: f64 = solar.iter().sum::<f64>() / solar.len() as f64;
        let wind_cf: f64 = wind.iter().sum::<f64>() / wind.len() as f64;
        println!(
            "Texas capacity factors: Solar {:.1}%, Wind {:.1}%",
            solar_cf * 100.0,
            wind_cf * 100.0
        );

        // Find peak load hours
        let peak_load = load.iter().cloned().fold(0.0_f64, f64::max);
        let peak_hours: Vec<usize> = load
            .iter()
            .enumerate()
            .filter(|(_, &l)| l > peak_load * 0.95)
            .map(|(i, _)| i)
            .collect();

        // Average solar/wind during peak hours
        let solar_at_peak: f64 =
            peak_hours.iter().map(|&h| solar[h]).sum::<f64>() / peak_hours.len() as f64;
        let wind_at_peak: f64 =
            peak_hours.iter().map(|&h| wind[h]).sum::<f64>() / peak_hours.len() as f64;

        println!(
            "Peak hours: {} hours above 95% of max load",
            peak_hours.len()
        );
        println!("Solar at peak: {:.1}% CF", solar_at_peak * 100.0);
        println!("Wind at peak: {:.1}% CF\n", wind_at_peak * 100.0);

        // Test different mixes
        let mixes = vec![
            ("Solar only", 200.0, 0.0, 100.0),
            ("Wind only", 0.0, 200.0, 100.0),
            ("Balanced", 100.0, 100.0, 100.0),
            ("Heavy solar", 150.0, 50.0, 100.0),
            ("Heavy wind", 50.0, 150.0, 100.0),
        ];

        println!(
            "{:<15} {:>10} {:>12} {:>12}",
            "Mix", "Port ELCC", "Div Benefit", "ELCC/Cap"
        );
        println!("{}", "-".repeat(52));

        for (name, s, w, st) in mixes {
            if let Ok(result) = calculate_elcc(
                s,
                w,
                st,
                0.0,
                &solar,
                &wind,
                &load,
                BatteryMode::Hybrid,
                0.85,
                0.0,
            ) {
                let total_cap = s + w + st;
                println!(
                    "{:<15} {:>10.1} {:>+12.1} {:>11.1}%",
                    name,
                    result.portfolio_elcc_mw,
                    result.diversity_benefit_mw,
                    result.portfolio_elcc_mw / total_cap * 100.0,
                );
            }
        }
    }

    // California deep dive
    println!("\n=== California Deep Dive: Storage Value ===\n");

    if let Some(zone) = zones
        .as_array()
        .unwrap()
        .iter()
        .find(|z| z["name"] == "California")
    {
        let solar: Vec<f64> = zone["solar_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let wind: Vec<f64> = zone["wind_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let load: Vec<f64> = zone["load_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();

        println!(
            "{:<20} {:>10} {:>12} {:>12} {:>12}",
            "Storage (MWh)", "Port ELCC", "Div Benefit", "Solar Cont", "Stor Cont"
        );
        println!("{}", "-".repeat(70));

        for storage in [0, 50, 100, 200, 400].iter() {
            if let Ok(result) = calculate_elcc(
                100.0,
                100.0,
                *storage as f64,
                0.0,
                &solar,
                &wind,
                &load,
                BatteryMode::Hybrid,
                0.85,
                0.0,
            ) {
                println!(
                    "{:<20} {:>10.1} {:>+12.1} {:>12.1}% {:>12.1}%",
                    storage,
                    result.portfolio_elcc_mw,
                    result.diversity_benefit_mw,
                    result.solar.contribution,
                    result.storage.contribution,
                );
            }
        }
    }

    println!("\n=== Key Finding: Solar ELCC by Method (California) ===\n");

    if let Some(zone) = zones
        .as_array()
        .unwrap()
        .iter()
        .find(|z| z["name"] == "California")
    {
        let solar: Vec<f64> = zone["solar_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let wind: Vec<f64> = zone["wind_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();
        let load: Vec<f64> = zone["load_profile"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap())
            .collect();

        // Compare with and without storage
        println!("Without storage (100 solar + 100 wind):");
        if let Ok(r) = calculate_elcc(
            100.0,
            100.0,
            0.0,
            0.0,
            &solar,
            &wind,
            &load,
            BatteryMode::Hybrid,
            0.85,
            0.0,
        ) {
            println!(
                "  Solar: First-In={:.1}%, Marginal={:.1}%, Contribution={:.1}%, Delta={:.1}%",
                r.solar.first_in, r.solar.marginal, r.solar.contribution, r.solar.delta
            );
            println!(
                "  Portfolio ELCC: {:.1} MW, Diversity Benefit: {:+.1} MW\n",
                r.portfolio_elcc_mw, r.diversity_benefit_mw
            );
        }

        println!("With storage (100 solar + 100 wind + 200 storage):");
        if let Ok(r) = calculate_elcc(
            100.0,
            100.0,
            200.0,
            0.0,
            &solar,
            &wind,
            &load,
            BatteryMode::Hybrid,
            0.85,
            0.0,
        ) {
            println!(
                "  Solar: First-In={:.1}%, Marginal={:.1}%, Contribution={:.1}%, Delta={:.1}%",
                r.solar.first_in, r.solar.marginal, r.solar.contribution, r.solar.delta
            );
            println!(
                "  Portfolio ELCC: {:.1} MW, Diversity Benefit: {:+.1} MW",
                r.portfolio_elcc_mw, r.diversity_benefit_mw
            );
        }
    }
}
