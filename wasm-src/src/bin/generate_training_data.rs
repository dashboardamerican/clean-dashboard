/// Training Data Generator for V2 Optimizer
///
/// Generates empirical model lookup tables by running simulations
/// across a grid of portfolio configurations for each zone and battery mode.
///
/// Usage:
///   cargo run --release --features native --bin generate_training_data -- \
///     --zones california,texas --modes hybrid
///
/// Options:
///   --zones <list>     Comma-separated zone names (default: california)
///   --modes <list>     Comma-separated modes: default,peak_shaver,hybrid (default: hybrid)
///   --grid <type>      Grid type: v1 (optimized, ~79KB) or default (~1.1MB) (default: v1)
///   --data <path>      Path to zones.json (default: ../data/zones.json)
///
/// Output:
///   models/<zone>_<mode>.bin files
use energy_simulator::{
    calculate_lcoe, simulate_system, BatteryMode, CostParams, EmpiricalModel, GridConfig,
    SimulationConfig, TrainingSample,
};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::time::Instant;

#[cfg(feature = "native")]
use rayon::prelude::*;

/// Zone data for simulation (profiles + metadata)
struct ZoneData {
    name: String,
    solar_profile: Vec<f64>,
    wind_profile: Vec<f64>,
    load_profile: Vec<f64>,
    is_synthetic: bool,
}

/// Zones data loaded from JSON
struct ZonesData {
    zones: HashMap<String, ZoneProfiles>,
}

/// Profile data for a single zone
#[derive(Clone)]
struct ZoneProfiles {
    solar: Vec<f64>,
    wind: Vec<f64>,
    load: Vec<f64>,
}

fn main() {
    println!("=== V2 Optimizer Training Data Generator ===\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let zones = parse_zones(&args);
    let modes = parse_modes(&args);
    let grid_type = parse_grid_type(&args);
    let data_path = parse_data_path(&args);

    println!("Zones: {:?}", zones);
    println!("Modes: {:?}", modes);
    println!("Grid type: {}", grid_type);
    println!("Data path: {}", data_path);
    println!();

    // Load zone data from JSON
    let zones_data = match load_zones_json(&data_path) {
        Ok(data) => {
            println!("Loaded {} zones from {}", data.zones.len(), data_path);
            data
        }
        Err(e) => {
            eprintln!("Warning: Failed to load zones.json: {}", e);
            eprintln!("Will use synthetic profiles as fallback");
            ZonesData {
                zones: HashMap::new(),
            }
        }
    };

    // Show available zones
    if !zones_data.zones.is_empty() {
        println!(
            "Available zones: {:?}",
            zones_data.zones.keys().collect::<Vec<_>>()
        );
    }
    println!();

    // Ensure output directory exists
    let model_dir = Path::new("models");
    if !model_dir.exists() {
        fs::create_dir_all(model_dir).expect("Failed to create models directory");
    }

    // Process each zone and mode
    for zone_name in &zones {
        for mode in &modes {
            generate_model(zone_name, *mode, model_dir, &grid_type, &zones_data);
        }
    }

    println!("\n=== Generation Complete ===");
}

fn parse_grid_type(args: &[String]) -> String {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--grid" && i + 1 < args.len() {
            return args[i + 1].to_lowercase();
        }
    }
    // Default: v1 optimized grid (smaller)
    "v1".to_string()
}

fn parse_data_path(args: &[String]) -> String {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--data" && i + 1 < args.len() {
            return args[i + 1].clone();
        }
    }
    // Default path relative to rust/ directory
    "../data/zones.json".to_string()
}

fn parse_zones(args: &[String]) -> Vec<String> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--zones" && i + 1 < args.len() {
            return args[i + 1].split(',').map(|s| s.to_string()).collect();
        }
    }
    // Default: California only (for quick testing)
    vec!["california".to_string()]
}

fn parse_modes(args: &[String]) -> Vec<BatteryMode> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--modes" && i + 1 < args.len() {
            return args[i + 1]
                .split(',')
                .filter_map(|s| match s.to_lowercase().as_str() {
                    "default" => Some(BatteryMode::Default),
                    "peak_shaver" | "peakshaver" => Some(BatteryMode::PeakShaver),
                    "hybrid" => Some(BatteryMode::Hybrid),
                    _ => None,
                })
                .collect();
        }
    }
    // Default: Hybrid only
    vec![BatteryMode::Hybrid]
}

/// Load zones data from JSON file
fn load_zones_json(path: &str) -> Result<ZonesData, String> {
    let content =
        fs::read_to_string(path).map_err(|e| format!("Failed to read {}: {}", path, e))?;

    // Parse as HashMap<String, ZoneJson>
    #[derive(serde::Deserialize)]
    struct ZoneJson {
        solar: Vec<f64>,
        wind: Vec<f64>,
        load: Vec<f64>,
    }

    let raw: HashMap<String, ZoneJson> =
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    let mut zones = HashMap::new();
    for (name, data) in raw {
        // Validate profile lengths
        if data.solar.len() != 8760 || data.wind.len() != 8760 || data.load.len() != 8760 {
            return Err(format!(
                "Zone {} has invalid profile lengths: solar={}, wind={}, load={}",
                name,
                data.solar.len(),
                data.wind.len(),
                data.load.len()
            ));
        }
        zones.insert(
            name.to_lowercase(),
            ZoneProfiles {
                solar: data.solar,
                wind: data.wind,
                load: data.load,
            },
        );
    }

    Ok(ZonesData { zones })
}

/// Normalize zone name for lookup (case-insensitive, handle variations)
fn normalize_zone_name(name: &str) -> String {
    let lower = name.to_lowercase();
    // Handle common variations and ISO region names
    match lower.as_str() {
        "midatlantic" | "mid_atlantic" | "pjm" => "mid-atlantic".to_string(),
        "newengland" | "new_england" | "isone" => "new england".to_string(),
        "newyork" | "new_york" | "nyiso" => "new york".to_string(),
        "spp" | "plains" => "plains".to_string(),
        "ercot" => "texas".to_string(),
        "miso" => "midwest".to_string(),
        "mountain" | "rocky" => "mountain".to_string(),
        "delta" | "south_central" => "delta".to_string(),
        _ => lower,
    }
}

fn generate_model(
    zone_name: &str,
    mode: BatteryMode,
    model_dir: &Path,
    grid_type: &str,
    zones_data: &ZonesData,
) {
    println!("Generating: {} / {:?}", zone_name, mode);

    let start = Instant::now();

    // Load zone data (prefer real data, fallback to synthetic)
    let zone = load_zone_data(zone_name, zones_data);

    // Create grid configuration based on type
    let config = match grid_type {
        "v1" => {
            println!("  Using V1 optimized grid (11×6×25×6 = 9,900 points)");
            GridConfig::v1_optimized()
        }
        "v2" | "fine" => {
            println!("  Using V2 fine grid (21×11×49×13 = 147,147 points)");
            GridConfig::v2_fine()
        }
        "default" | _ => {
            println!("  Using default grid (25×25×25×11 = 171,875 points)");
            GridConfig::default()
        }
    };
    let total_points = config.total_points();
    println!("  Grid points: {}", total_points);

    // Generate all grid points
    let mut model = EmpiricalModel::new(config.clone());
    let points = model.generate_grid_points();

    // Run simulations (parallel if feature enabled)
    #[cfg(feature = "native")]
    let samples: Vec<TrainingSample> = points
        .par_iter()
        .map(|(solar, wind, storage, cf)| run_simulation(*solar, *wind, *storage, *cf, &zone, mode))
        .collect();

    #[cfg(not(feature = "native"))]
    let samples: Vec<TrainingSample> = points
        .iter()
        .map(|(solar, wind, storage, cf)| run_simulation(*solar, *wind, *storage, *cf, &zone, mode))
        .collect();

    // Build lookup table (clean_match, peak_gas, and gas_generation)
    for sample in &samples {
        model.set_all(
            sample.solar,
            sample.wind,
            sample.storage,
            sample.clean_firm,
            sample.clean_match,
            sample.peak_gas,
            sample.gas_generation,
        );
    }

    // Save model
    let filename = format!("{}_{:?}.bin", zone_name, mode).to_lowercase();
    let filepath = model_dir.join(&filename);

    let bytes = model.to_bytes().expect("Failed to serialize model");
    fs::write(&filepath, bytes).expect("Failed to write model file");

    let elapsed = start.elapsed();
    let file_size = fs::metadata(&filepath).map(|m| m.len()).unwrap_or(0);

    println!(
        "  Completed in {:.1}s, {} samples, {} KB",
        elapsed.as_secs_f64(),
        samples.len(),
        file_size / 1024
    );
}

fn load_zone_data(zone_name: &str, zones_data: &ZonesData) -> ZoneData {
    // Try to find zone in loaded data (case-insensitive)
    let normalized = normalize_zone_name(zone_name);

    if let Some(profiles) = zones_data.zones.get(&normalized) {
        println!("  Using real profiles for {}", zone_name);
        return ZoneData {
            name: zone_name.to_string(),
            solar_profile: profiles.solar.clone(),
            wind_profile: profiles.wind.clone(),
            load_profile: profiles.load.clone(),
            is_synthetic: false,
        };
    }

    // Try alternative lookups (the JSON uses title case)
    for (key, profiles) in &zones_data.zones {
        if key.to_lowercase() == normalized {
            println!(
                "  Using real profiles for {} (matched as '{}')",
                zone_name, key
            );
            return ZoneData {
                name: zone_name.to_string(),
                solar_profile: profiles.solar.clone(),
                wind_profile: profiles.wind.clone(),
                load_profile: profiles.load.clone(),
                is_synthetic: false,
            };
        }
    }

    // Fallback to synthetic profiles
    println!(
        "  Warning: Zone '{}' not found, using synthetic profiles",
        zone_name
    );
    generate_synthetic_zone(zone_name)
}

fn generate_synthetic_zone(zone_name: &str) -> ZoneData {
    const HOURS: usize = 8760;

    // Generate realistic-ish profiles based on zone
    let (solar_cf, wind_cf) = match zone_name.to_lowercase().as_str() {
        "california" => (0.28, 0.32),
        "texas" => (0.26, 0.38),
        "florida" => (0.24, 0.20),
        "newyork" | "new york" | "new_york" => (0.18, 0.30),
        "pjm" | "mid-atlantic" | "midatlantic" => (0.20, 0.28),
        "miso" => (0.22, 0.35),
        "spp" | "plains" => (0.24, 0.42),
        "ercot" => (0.26, 0.36),
        "northwest" => (0.16, 0.28),
        "southwest" => (0.30, 0.25),
        "southeast" => (0.23, 0.22),
        "newengland" | "new england" | "new_england" => (0.17, 0.28),
        "midwest" => (0.20, 0.34),
        "delta" => (0.24, 0.30),
        "mountain" => (0.26, 0.32),
        _ => (0.25, 0.30),
    };

    // Simple diurnal pattern for solar
    let solar_profile: Vec<f64> = (0..HOURS)
        .map(|h| {
            let hour_of_day = h % 24;
            if hour_of_day >= 6 && hour_of_day <= 18 {
                let peak_factor = 1.0 - ((hour_of_day as f64 - 12.0).abs() / 6.0);
                solar_cf * peak_factor * 2.0
            } else {
                0.0
            }
        })
        .collect();

    // Simple pattern for wind (higher at night)
    let wind_profile: Vec<f64> = (0..HOURS)
        .map(|h| {
            let hour_of_day = h % 24;
            let base = wind_cf;
            if hour_of_day < 6 || hour_of_day > 20 {
                base * 1.2 // Higher at night
            } else {
                base * 0.8 // Lower during day
            }
        })
        .collect();

    // Constant load for simplicity
    let load_profile = vec![100.0; HOURS];

    ZoneData {
        name: zone_name.to_string(),
        solar_profile,
        wind_profile,
        load_profile,
        is_synthetic: true,
    }
}

fn run_simulation(
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
    zone: &ZoneData,
    mode: BatteryMode,
) -> TrainingSample {
    let config = SimulationConfig {
        solar_capacity: solar,
        wind_capacity: wind,
        storage_capacity: storage,
        clean_firm_capacity: clean_firm,
        battery_efficiency: 0.85,
        max_demand_response: 0.0,
        battery_mode: mode,
    };

    match simulate_system(
        &config,
        &zone.solar_profile,
        &zone.wind_profile,
        &zone.load_profile,
    ) {
        Ok(result) => TrainingSample {
            solar,
            wind,
            storage,
            clean_firm,
            clean_match: result.clean_match_pct,
            peak_gas: result.peak_gas,
            gas_generation: result.gas_generation.iter().sum(), // Sum hourly gas to get annual MWh
        },
        Err(_) => TrainingSample {
            solar,
            wind,
            storage,
            clean_firm,
            clean_match: 0.0,
            peak_gas: 100.0,           // Assume max gas needed on error
            gas_generation: 876_000.0, // Assume all gas on error (100 MW * 8760 h)
        },
    }
}
