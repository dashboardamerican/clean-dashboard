/// Benchmarks for V2 Optimizer
///
/// Run with: cargo bench --bench optimizer_v2_bench
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use energy_simulator::{
    run_v2_optimizer, BatteryMode, CostParams, OptimizerConfig, HOURS_PER_YEAR,
};

fn create_test_profiles() -> (Vec<f64>, Vec<f64>, Vec<f64>) {
    // Solar: peaks at midday
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

    // Wind: higher at night
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

    // Constant load
    let load = vec![100.0; HOURS_PER_YEAR];

    (solar, wind, load)
}

fn bench_single_optimization(c: &mut Criterion) {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    let mut group = c.benchmark_group("v2_single");

    for target in [30.0, 50.0, 70.0, 90.0] {
        group.bench_with_input(
            BenchmarkId::new("target", target as i32),
            &target,
            |b, &t| {
                b.iter(|| {
                    run_v2_optimizer(
                        t,
                        &solar,
                        &wind,
                        &load,
                        &costs,
                        &config,
                        BatteryMode::Hybrid,
                        None,
                    )
                    .unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_full_sweep(c: &mut Criterion) {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    c.bench_function("v2_sweep_0_to_100", |b| {
        b.iter(|| {
            for target in (0..=100).step_by(10) {
                run_v2_optimizer(
                    target as f64,
                    &solar,
                    &wind,
                    &load,
                    &costs,
                    &config,
                    BatteryMode::Hybrid,
                    None,
                )
                .ok();
            }
        })
    });
}

fn bench_battery_modes(c: &mut Criterion) {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();
    let config = OptimizerConfig::default();

    let mut group = c.benchmark_group("v2_battery_modes");

    for mode in [
        BatteryMode::Default,
        BatteryMode::PeakShaver,
        BatteryMode::Hybrid,
    ] {
        group.bench_with_input(
            BenchmarkId::new("mode", format!("{:?}", mode)),
            &mode,
            |b, &m| {
                b.iter(|| {
                    run_v2_optimizer(50.0, &solar, &wind, &load, &costs, &config, m, None).unwrap()
                })
            },
        );
    }

    group.finish();
}

fn bench_resource_constraints(c: &mut Criterion) {
    let (solar, wind, load) = create_test_profiles();
    let costs = CostParams::default_costs();

    let mut group = c.benchmark_group("v2_resource_constraints");

    // Solar only
    let mut solar_only = OptimizerConfig::default();
    solar_only.enable_wind = false;
    solar_only.enable_storage = false;
    solar_only.enable_clean_firm = false;

    group.bench_function("solar_only", |b| {
        b.iter(|| {
            run_v2_optimizer(
                30.0,
                &solar,
                &wind,
                &load,
                &costs,
                &solar_only,
                BatteryMode::Hybrid,
                None,
            )
            .ok()
        })
    });

    // Full resources
    let full = OptimizerConfig::default();
    group.bench_function("all_resources", |b| {
        b.iter(|| {
            run_v2_optimizer(
                70.0,
                &solar,
                &wind,
                &load,
                &costs,
                &full,
                BatteryMode::Hybrid,
                None,
            )
            .unwrap()
        })
    });

    // No clean firm
    let mut no_cf = OptimizerConfig::default();
    no_cf.enable_clean_firm = false;

    group.bench_function("no_clean_firm", |b| {
        b.iter(|| {
            run_v2_optimizer(
                50.0,
                &solar,
                &wind,
                &load,
                &costs,
                &no_cf,
                BatteryMode::Hybrid,
                None,
            )
            .ok()
        })
    });

    group.finish();
}

criterion_group!(
    benches,
    bench_single_optimization,
    bench_full_sweep,
    bench_battery_modes,
    bench_resource_constraints,
);
criterion_main!(benches);
