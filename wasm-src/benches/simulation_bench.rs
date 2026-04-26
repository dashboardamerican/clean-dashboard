use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use energy_simulator::{
    calculate_lcoe, run_v1_optimizer, simulate_system, BatteryMode, CostParams, OptimizerConfig,
    SimulationConfig, HOURS_PER_YEAR,
};

/// Generate realistic solar profile (peaks midday, zero at night)
fn generate_solar_profile() -> Vec<f64> {
    let mut profile = vec![0.0; HOURS_PER_YEAR];
    for day in 0..(HOURS_PER_YEAR / 24) {
        for hour in 0..24 {
            let idx = day * 24 + hour;
            // Solar peaks around noon
            if hour >= 6 && hour <= 18 {
                let solar_hour = hour as f64 - 6.0;
                let peak_factor = (solar_hour * std::f64::consts::PI / 12.0).sin();
                // Add some seasonal variation
                let day_of_year = day % 365;
                let seasonal = 0.7
                    + 0.3 * ((day_of_year as f64 * 2.0 * std::f64::consts::PI / 365.0).sin() + 1.0)
                        / 2.0;
                profile[idx] = peak_factor * seasonal * 0.85;
            }
        }
    }
    profile
}

/// Generate realistic wind profile (variable, somewhat correlated)
fn generate_wind_profile() -> Vec<f64> {
    let mut profile = vec![0.0; HOURS_PER_YEAR];
    let mut current = 0.4;
    for i in 0..HOURS_PER_YEAR {
        // Random walk with mean reversion
        let noise = ((i * 7919) % 1000) as f64 / 1000.0 - 0.5; // Deterministic pseudo-random
        current = 0.95 * current + 0.05 * 0.35 + 0.1 * noise;
        current = current.clamp(0.05, 0.85);
        profile[i] = current;
    }
    profile
}

/// Generate realistic load profile (daily cycle, seasonal variation)
fn generate_load_profile() -> Vec<f64> {
    let mut profile = vec![0.0; HOURS_PER_YEAR];
    let base_load = 100.0;
    for day in 0..(HOURS_PER_YEAR / 24) {
        for hour in 0..24 {
            let idx = day * 24 + hour;
            // Daily pattern: low at night, peak in evening
            let daily = match hour {
                0..=5 => 0.7,
                6..=8 => 0.85,
                9..=16 => 0.95,
                17..=21 => 1.15,
                _ => 0.9,
            };
            // Seasonal: higher in summer/winter
            let day_of_year = day % 365;
            let seasonal = 1.0
                + 0.1
                    * (2.0 * std::f64::consts::PI * day_of_year as f64 / 182.5)
                        .cos()
                        .abs();
            profile[idx] = base_load * daily * seasonal;
        }
    }
    profile
}

fn bench_simulation(c: &mut Criterion) {
    let solar = generate_solar_profile();
    let wind = generate_wind_profile();
    let load = generate_load_profile();

    let mut group = c.benchmark_group("simulation");

    // Test different battery modes
    for mode in [
        BatteryMode::Default,
        BatteryMode::PeakShaver,
        BatteryMode::Hybrid,
    ] {
        let mode_name = match mode {
            BatteryMode::Default => "default",
            BatteryMode::PeakShaver => "peak_shaver",
            BatteryMode::Hybrid => "hybrid",
        };

        let mut config = SimulationConfig::with_defaults();
        config.solar_capacity = 100.0;
        config.wind_capacity = 100.0;
        config.storage_capacity = 50.0;
        config.clean_firm_capacity = 20.0;
        config.battery_mode = mode;

        group.bench_with_input(
            BenchmarkId::new("8760h", mode_name),
            &(&config, &solar, &wind, &load),
            |b, (config, solar, wind, load)| {
                b.iter(|| black_box(simulate_system(config, solar, wind, load).unwrap()))
            },
        );
    }

    // Test with varying storage sizes
    for storage in [0.0, 50.0, 100.0, 200.0] {
        let mut config = SimulationConfig::with_defaults();
        config.solar_capacity = 100.0;
        config.wind_capacity = 100.0;
        config.storage_capacity = storage;
        config.clean_firm_capacity = 20.0;

        group.bench_with_input(
            BenchmarkId::new("storage_MWh", storage as u64),
            &(&config, &solar, &wind, &load),
            |b, (config, solar, wind, load)| {
                b.iter(|| black_box(simulate_system(config, solar, wind, load).unwrap()))
            },
        );
    }

    group.finish();
}

fn bench_lcoe(c: &mut Criterion) {
    let solar = generate_solar_profile();
    let wind = generate_wind_profile();
    let load = generate_load_profile();

    let mut config = SimulationConfig::with_defaults();
    config.solar_capacity = 100.0;
    config.wind_capacity = 100.0;
    config.storage_capacity = 50.0;
    config.clean_firm_capacity = 20.0;

    let sim_result = simulate_system(&config, &solar, &wind, &load).unwrap();
    let costs = CostParams::default_costs();

    c.bench_function("lcoe_calculation", |b| {
        b.iter(|| {
            black_box(calculate_lcoe(
                &sim_result,
                config.solar_capacity,
                config.wind_capacity,
                config.storage_capacity,
                config.clean_firm_capacity,
                &costs,
            ))
        })
    });
}

fn bench_simulation_plus_lcoe(c: &mut Criterion) {
    let solar = generate_solar_profile();
    let wind = generate_wind_profile();
    let load = generate_load_profile();

    let mut config = SimulationConfig::with_defaults();
    config.solar_capacity = 100.0;
    config.wind_capacity = 100.0;
    config.storage_capacity = 50.0;
    config.clean_firm_capacity = 20.0;

    let costs = CostParams::default_costs();

    c.bench_function("simulation_plus_lcoe", |b| {
        b.iter(|| {
            let sim_result = simulate_system(&config, &solar, &wind, &load).unwrap();
            black_box(calculate_lcoe(
                &sim_result,
                config.solar_capacity,
                config.wind_capacity,
                config.storage_capacity,
                config.clean_firm_capacity,
                &costs,
            ))
        })
    });
}

fn bench_optimizer(c: &mut Criterion) {
    let solar = generate_solar_profile();
    let wind = generate_wind_profile();
    let load = generate_load_profile();

    let costs = CostParams::default_costs();

    let mut group = c.benchmark_group("optimizer");
    group.sample_size(10); // Optimizer is slower, use fewer samples

    for target in [30.0, 50.0, 80.0] {
        let config = OptimizerConfig {
            target_clean_match: target,
            ..OptimizerConfig::default()
        };

        group.bench_with_input(
            BenchmarkId::new("target_pct", target as u64),
            &(&config, &solar, &wind, &load, &costs),
            |b, (config, solar, wind, load, costs)| {
                b.iter(|| {
                    black_box(run_v1_optimizer(
                        config.target_clean_match,
                        solar,
                        wind,
                        load,
                        costs,
                        config,
                        BatteryMode::Hybrid,
                    ))
                })
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_simulation,
    bench_lcoe,
    bench_simulation_plus_lcoe,
    bench_optimizer,
);
criterion_main!(benches);
