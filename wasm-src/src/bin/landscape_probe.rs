//! Landscape probe: evaluate arbitrary portfolio grids/points and dump ALL results.
//!
//! Unlike the oracle (which returns only the argmin), this tool exists to study the
//! SHAPE of the objective landscape: basins, cliffs, plateaus, fold lines.
//!
//! Usage:
//!   cargo run --release --features native --bin landscape_probe -- --spec spec.json --out out.csv
//!
//! Spec JSON:
//! {
//!   "zone": "california",                  // zone name in zones.json, or "synthetic"
//!   "data": "../data/zones.json",          // optional, default ../data/zones.json
//!   "battery_mode": "hybrid",              // default | peak_shaver | hybrid
//!   "cost_overrides": {"gas_price": 14.0}, // merged onto CostParams::default_costs()
//!   "grid": {                              // optional: axes as [start, stop, step]
//!     "solar": [0, 200, 10],
//!     "wind": [0, 300, 10],
//!     "storage": [0, 400, 25],
//!     "clean_firm": [0, 100, 5]
//!   },
//!   "points": [[100, 50, 25, 0]]           // optional explicit [solar,wind,storage,cf] points
//! }
//!
//! Output CSV columns:
//!   solar,wind,storage,clean_firm,clean_match,lcoe,peak_gas,gas_mwh,curtailed_mwh,battery_discharge_mwh

use energy_simulator::{
    calculate_lcoe, simulate_system, BatteryMode, CostParams, SimulationConfig, HOURS_PER_YEAR,
};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::time::Instant;

#[derive(Deserialize)]
struct Spec {
    #[serde(default = "default_zone")]
    zone: String,
    #[serde(default = "default_data_path")]
    data: String,
    #[serde(default = "default_mode")]
    battery_mode: String,
    #[serde(default)]
    cost_overrides: serde_json::Map<String, serde_json::Value>,
    #[serde(default)]
    grid: Option<GridSpec>,
    #[serde(default)]
    points: Vec<[f64; 4]>,
    /// If set (e.g. 99.9), gas capacity is priced off this percentile of hourly gas
    /// generation instead of the strict annual max (H6 smoothed-peak experiment).
    #[serde(default)]
    peak_gas_percentile: Option<f64>,
}

#[derive(Deserialize)]
struct GridSpec {
    solar: [f64; 3],
    wind: [f64; 3],
    storage: [f64; 3],
    clean_firm: [f64; 3],
}

fn default_zone() -> String {
    "california".to_string()
}
fn default_data_path() -> String {
    "../data/zones.json".to_string()
}
fn default_mode() -> String {
    "hybrid".to_string()
}

fn axis(range: &[f64; 3]) -> Vec<f64> {
    let (start, stop, step) = (range[0], range[1], range[2]);
    if step <= 0.0 || stop <= start {
        return vec![start];
    }
    let mut values = Vec::new();
    let mut v = start;
    while v <= stop + 1e-9 {
        values.push(v);
        v += step;
    }
    values
}

fn load_zone(zone: &str, data_path: &str) -> Result<(Vec<f64>, Vec<f64>, Vec<f64>), String> {
    if zone == "synthetic" {
        let solar: Vec<f64> = (0..HOURS_PER_YEAR)
            .map(|h| {
                let hour_of_day = h % 24;
                if (6..=18).contains(&hour_of_day) {
                    let peak = 1.0 - ((hour_of_day as f64 - 12.0).abs() / 6.0);
                    0.5 * peak
                } else {
                    0.0
                }
            })
            .collect();
        let wind: Vec<f64> = (0..HOURS_PER_YEAR)
            .map(|h| {
                let hour_of_day = h % 24;
                if !(6..=20).contains(&hour_of_day) {
                    0.42
                } else {
                    0.28
                }
            })
            .collect();
        let load = vec![100.0; HOURS_PER_YEAR];
        return Ok((solar, wind, load));
    }

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

fn build_costs(overrides: &serde_json::Map<String, serde_json::Value>) -> Result<CostParams, String> {
    let mut value = serde_json::to_value(CostParams::default_costs())
        .map_err(|e| format!("serialize default costs: {}", e))?;
    let obj = value
        .as_object_mut()
        .ok_or_else(|| "default costs not an object".to_string())?;
    for (k, v) in overrides {
        if !obj.contains_key(k) {
            return Err(format!("Unknown cost override field: '{}'", k));
        }
        obj.insert(k.clone(), v.clone());
    }
    serde_json::from_value(value).map_err(|e| format!("deserialize costs: {}", e))
}

fn parse_mode(s: &str) -> Result<BatteryMode, String> {
    match s.to_lowercase().as_str() {
        "default" => Ok(BatteryMode::Default),
        "peak_shaver" | "peakshaver" => Ok(BatteryMode::PeakShaver),
        "hybrid" => Ok(BatteryMode::Hybrid),
        other => Err(format!("Unknown battery mode: {}", other)),
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut spec_path = None;
    let mut out_path = None;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--spec" => {
                i += 1;
                spec_path = args.get(i).cloned();
            }
            "--out" => {
                i += 1;
                out_path = args.get(i).cloned();
            }
            other => {
                eprintln!("Unknown argument: {}", other);
                std::process::exit(1);
            }
        }
        i += 1;
    }
    let spec_path = spec_path.unwrap_or_else(|| {
        eprintln!("--spec required");
        std::process::exit(1);
    });
    let out_path = out_path.unwrap_or_else(|| {
        eprintln!("--out required");
        std::process::exit(1);
    });

    let spec: Spec = serde_json::from_str(
        &fs::read_to_string(&spec_path).unwrap_or_else(|e| {
            eprintln!("read spec: {}", e);
            std::process::exit(1);
        }),
    )
    .unwrap_or_else(|e| {
        eprintln!("parse spec: {}", e);
        std::process::exit(1);
    });

    let (solar_profile, wind_profile, load_profile) =
        load_zone(&spec.zone, &spec.data).unwrap_or_else(|e| {
            eprintln!("{}", e);
            std::process::exit(1);
        });
    let costs = build_costs(&spec.cost_overrides).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });
    let mode = parse_mode(&spec.battery_mode).unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1);
    });

    let mut points: Vec<[f64; 4]> = spec.points.clone();
    if let Some(grid) = &spec.grid {
        for &s in &axis(&grid.solar) {
            for &w in &axis(&grid.wind) {
                for &st in &axis(&grid.storage) {
                    for &cf in &axis(&grid.clean_firm) {
                        points.push([s, w, st, cf]);
                    }
                }
            }
        }
    }
    if points.is_empty() {
        eprintln!("Spec produced zero points (need grid and/or points)");
        std::process::exit(1);
    }

    eprintln!(
        "landscape_probe: zone={} mode={} points={}",
        spec.zone,
        spec.battery_mode,
        points.len()
    );
    let start = Instant::now();

    let rows: Vec<String> = points
        .par_iter()
        .map(|p| {
            let [solar, wind, storage, cf] = *p;
            let config = SimulationConfig {
                solar_capacity: solar,
                wind_capacity: wind,
                storage_capacity: storage,
                clean_firm_capacity: cf,
                battery_mode: mode,
                ..SimulationConfig::with_defaults()
            };
            match simulate_system(&config, &solar_profile, &wind_profile, &load_profile) {
                Ok(mut sim) => {
                    if let Some(pct) = spec.peak_gas_percentile {
                        let mut sorted = sim.gas_generation.clone();
                        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
                        let rank = ((pct.clamp(0.0, 100.0) / 100.0)
                            * (sorted.len() as f64 - 1.0))
                            .round() as usize;
                        sim.peak_gas = sorted[rank];
                    }
                    let lcoe = calculate_lcoe(&sim, solar, wind, storage, cf, &costs);
                    let gas_mwh: f64 = sim.gas_generation.iter().sum();
                    let discharge_mwh: f64 = sim.battery_discharge.iter().sum();
                    format!(
                        "{},{},{},{},{:.6},{:.6},{:.4},{:.2},{:.2},{:.2}",
                        solar,
                        wind,
                        storage,
                        cf,
                        sim.clean_match_pct,
                        lcoe.total_lcoe,
                        sim.peak_gas,
                        gas_mwh,
                        sim.total_curtailment,
                        discharge_mwh
                    )
                }
                Err(e) => format!("{},{},{},{},ERROR,{},,,,", solar, wind, storage, cf, e),
            }
        })
        .collect();

    let mut csv = String::from(
        "solar,wind,storage,clean_firm,clean_match,lcoe,peak_gas,gas_mwh,curtailed_mwh,battery_discharge_mwh\n",
    );
    for row in &rows {
        csv.push_str(row);
        csv.push('\n');
    }
    fs::write(&out_path, csv).unwrap_or_else(|e| {
        eprintln!("write out: {}", e);
        std::process::exit(1);
    });

    eprintln!(
        "landscape_probe: {} points in {:.1}s -> {}",
        rows.len(),
        start.elapsed().as_secs_f64(),
        out_path
    );
}
