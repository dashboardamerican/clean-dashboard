//! Optimizer benchmark CLI: run the V2 optimizer over zones × targets and dump results.
//!
//! Companion to landscape_probe — probe maps the landscape, this benches the optimizer
//! against it. Output feeds the certification harness (gap vs ground-truth corpus).
//!
//! Usage:
//!   cargo run --release --features native --bin optimizer_bench -- \
//!     --zones california,texas --targets 20,30,...,100 --mode both \
//!     [--battery-mode hybrid] [--cost-overrides '{"gas_price":8.0}'] \
//!     [--disable solar,wind] [--data ../data/zones.json] --out results.json
//!
//! `--disable` takes a comma list of resources (solar, wind, storage,
//! clean_firm) and flips the matching `OptimizerConfig.enable_*` field off
//! before running (default OptimizerConfig has all four enabled).
//!
//! Output: JSON array of
//!   {zone, target, mode, battery_mode, disabled, solar, wind, storage, clean_firm,
//!    clean_match, lcoe, evaluations, time_ms, success, error}

use energy_simulator::{
    run_v2_optimizer_mode, BatteryMode, CostParams, EmpiricalModel, OptimizerConfig, V2Mode,
    HOURS_PER_YEAR,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

fn load_zone(zone: &str, data_path: &str) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), String> {
    #[derive(Deserialize)]
    struct ZoneJson {
        solar: Vec<f64>,
        wind: Vec<f64>,
        load: Vec<f64>,
    }
    let content = fs::read_to_string(data_path)
        .map_err(|e| format!("Failed to read {}: {}", data_path, e))?;
    let raw: HashMap<String, ZoneJson> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse zones json: {}", e))?;
    let want = zone.to_lowercase().replace([' ', '-'], "_");
    for (key, z) in raw {
        if key.to_lowercase().replace([' ', '-'], "_") == want {
            if z.solar.len() != HOURS_PER_YEAR
                || z.wind.len() != HOURS_PER_YEAR
                || z.load.len() != HOURS_PER_YEAR
            {
                return Err(format!("Zone {} has invalid profile lengths", key));
            }
            return Ok((z.solar, z.wind, z.load));
        }
    }
    Err(format!("Zone '{}' not found in {}", zone, data_path))
}

fn build_costs(overrides_json: &str) -> Result<CostParams, String> {
    let overrides: serde_json::Map<String, serde_json::Value> =
        serde_json::from_str(overrides_json).map_err(|e| format!("parse overrides: {}", e))?;
    let mut value = serde_json::to_value(CostParams::default_costs())
        .map_err(|e| format!("serialize default costs: {}", e))?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| "default costs not an object".to_string())?;
    for (k, v) in &overrides {
        if !obj.contains_key(k) {
            return Err(format!("Unknown cost override field: '{}'", k));
        }
        obj.insert(k.clone(), v.clone());
    }
    serde_json::from_value(value).map_err(|e| format!("deserialize costs: {}", e))
}

/// Load a pre-computed EmpiricalModel for a zone/battery-mode from a directory.
/// Tries the naming variants used across the repo (underscores, hyphens, squashed).
fn load_model_for_zone(dir: &str, zone: &str, battery_mode: &str) -> Option<EmpiricalModel> {
    let zl = zone.to_lowercase();
    let underscored = zl.replace([' ', '-'], "_");
    let hyphenated = zl.replace([' ', '_'], "-");
    let squashed = zl.replace([' ', '-', '_'], "");
    let mode = battery_mode.to_lowercase().replace('_', "");
    for stem in [&underscored, &hyphenated, &squashed] {
        let path = format!("{}/{}_{}.bin", dir, stem, mode);
        if let Ok(bytes) = fs::read(&path) {
            match EmpiricalModel::from_bytes(&bytes) {
                Ok(m) => {
                    eprintln!("loaded model {}", path);
                    return Some(m);
                }
                Err(e) => eprintln!("failed to parse model {}: {}", path, e),
            }
        }
    }
    None
}

fn parse_battery_mode(s: &str) -> Result<BatteryMode, String> {
    match s.to_lowercase().as_str() {
        "default" => Ok(BatteryMode::Default),
        "peak_shaver" | "peakshaver" => Ok(BatteryMode::PeakShaver),
        "hybrid" => Ok(BatteryMode::Hybrid),
        other => Err(format!("Unknown battery mode: {}", other)),
    }
}

/// Parse a comma-separated `--disable` list (e.g. "solar,wind") and flip the
/// matching `enable_*` fields off on the config.
fn apply_disable_flags(config: &mut OptimizerConfig, disable_list: &str) -> Result<(), String> {
    for raw in disable_list.split(',') {
        let name = raw.trim().to_lowercase().replace(['-', ' '], "_");
        if name.is_empty() {
            continue;
        }
        match name.as_str() {
            "solar" => config.enable_solar = false,
            "wind" => config.enable_wind = false,
            "storage" => config.enable_storage = false,
            "clean_firm" | "cleanfirm" | "cf" => config.enable_clean_firm = false,
            other => return Err(format!("Unknown --disable resource: '{}'", other)),
        }
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut zones = vec!["california".to_string()];
    let mut targets: Vec<f64> = vec![50.0, 90.0];
    let mut mode = "both".to_string();
    let mut battery_mode = "hybrid".to_string();
    let mut cost_overrides = "{}".to_string();
    let mut data_path = "../data/zones.json".to_string();
    let mut out_path = None;
    let mut model_dir: Option<String> = None;
    let mut disable_list: Option<String> = None;

    let mut i = 1;
    while i < args.len() {
        let arg = args[i].as_str();
        i += 1;
        let value = || args.get(i).cloned();
        match arg {
            "--zones" => {
                zones = value()
                    .expect("--zones needs a value")
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect();
                i += 1;
            }
            "--targets" => {
                targets = value()
                    .expect("--targets needs a value")
                    .split(',')
                    .filter_map(|s| s.trim().parse().ok())
                    .collect();
                i += 1;
            }
            "--mode" => {
                mode = value().expect("--mode needs a value");
                i += 1;
            }
            "--battery-mode" => {
                battery_mode = value().expect("--battery-mode needs a value");
                i += 1;
            }
            "--cost-overrides" => {
                cost_overrides = value().expect("--cost-overrides needs a value");
                i += 1;
            }
            "--data" => {
                data_path = value().expect("--data needs a value");
                i += 1;
            }
            "--out" => {
                out_path = value();
                i += 1;
            }
            "--model-dir" => {
                model_dir = value();
                i += 1;
            }
            "--disable" => {
                disable_list = value();
                i += 1;
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                std::process::exit(1);
            }
        }
    }
    let out_path = out_path.unwrap_or_else(|| {
        eprintln!("--out required");
        std::process::exit(1);
    });

    let costs = build_costs(&cost_overrides).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let bmode = parse_battery_mode(&battery_mode).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let modes: Vec<V2Mode> = match mode.as_str() {
        "fast" => vec![V2Mode::Fast],
        "accurate" => vec![V2Mode::Accurate],
        "both" => vec![V2Mode::Fast, V2Mode::Accurate],
        other => {
            eprintln!("Unknown mode: {} (fast|accurate|both)", other);
            std::process::exit(1);
        }
    };

    let mut config = OptimizerConfig::default();
    if let Some(dl) = &disable_list {
        apply_disable_flags(&mut config, dl).unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
    }
    let mut rows: Vec<serde_json::Value> = Vec::new();

    for zone in &zones {
        let (solar_p, wind_p, load_p) = match load_zone(zone, &data_path) {
            Ok(p) => p,
            Err(e) => {
                eprintln!("skip zone {}: {}", zone, e);
                continue;
            }
        };
        let model = model_dir
            .as_deref()
            .and_then(|dir| load_model_for_zone(dir, zone, &battery_mode));
        if model_dir.is_some() && model.is_none() {
            eprintln!("no model found for zone {} in {:?}", zone, model_dir);
        }
        for &target in &targets {
            for &m in &modes {
                let mode_name = match m {
                    V2Mode::Fast => "fast",
                    V2Mode::Accurate => "accurate",
                };
                let start = Instant::now();
                let result = run_v2_optimizer_mode(
                    target,
                    &solar_p,
                    &wind_p,
                    &load_p,
                    &costs,
                    &config,
                    bmode,
                    model.as_ref(),
                    m,
                    None,
                );
                let ms = start.elapsed().as_secs_f64() * 1000.0;
                let row = match result {
                    Ok(r) => serde_json::json!({
                        "zone": zone, "target": target, "mode": mode_name,
                        "battery_mode": battery_mode, "model_loaded": model.is_some(),
                        "disabled": disable_list,
                        "solar": r.solar_capacity, "wind": r.wind_capacity,
                        "storage": r.storage_capacity, "clean_firm": r.clean_firm_capacity,
                        "clean_match": r.achieved_clean_match, "lcoe": r.lcoe,
                        "evaluations": r.num_evaluations, "time_ms": ms,
                        "success": r.success, "error": null
                    }),
                    Err(e) => serde_json::json!({
                        "zone": zone, "target": target, "mode": mode_name,
                        "battery_mode": battery_mode, "disabled": disable_list,
                        "solar": null, "wind": null, "storage": null, "clean_firm": null,
                        "clean_match": null, "lcoe": null, "evaluations": null,
                        "time_ms": ms, "success": false, "error": e
                    }),
                };
                rows.push(row);
                eprintln!(
                    "{} t={} {} -> {:.1}ms",
                    zone, target, mode_name, ms
                );
            }
        }
    }

    fs::write(
        &out_path,
        serde_json::to_string_pretty(&rows).expect("serialize"),
    )
    .unwrap_or_else(|e| {
        eprintln!("write out: {}", e);
        std::process::exit(1);
    });
    eprintln!("optimizer_bench: {} rows -> {}", rows.len(), out_path);
}
