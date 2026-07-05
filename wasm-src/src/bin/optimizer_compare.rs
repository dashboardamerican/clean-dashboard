use energy_simulator::eval::{
    compare_reports, read_report, validate_report, ReportComparisonConfig, ValidationSeverity,
};
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("optimizer_compare error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    if args.len() == 1 || args.iter().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return Ok(());
    }

    let mut baseline_path: Option<PathBuf> = None;
    let mut candidate_path: Option<PathBuf> = None;
    let mut config = ReportComparisonConfig::default();

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--lcoe-tolerance" => {
                i += 1;
                config.lcoe_regression_tolerance = parse_f64(&args, i, "--lcoe-tolerance")?;
            }
            "--achieved-tolerance" => {
                i += 1;
                config.achieved_drift_tolerance_pct = parse_f64(&args, i, "--achieved-tolerance")?;
            }
            "--runtime-factor" => {
                i += 1;
                config.runtime_regression_factor = parse_f64(&args, i, "--runtime-factor")?;
            }
            "--runtime-min-delta" => {
                i += 1;
                config.runtime_regression_min_delta_ms =
                    parse_f64(&args, i, "--runtime-min-delta")?;
            }
            value if baseline_path.is_none() => baseline_path = Some(PathBuf::from(value)),
            value if candidate_path.is_none() => candidate_path = Some(PathBuf::from(value)),
            other => return Err(format!("Unknown argument: {}", other)),
        }
        i += 1;
    }

    let baseline_path = baseline_path.ok_or_else(|| "missing baseline report path".to_string())?;
    let candidate_path =
        candidate_path.ok_or_else(|| "missing candidate report path".to_string())?;

    let baseline = read_report(&baseline_path)?;
    let candidate = read_report(&candidate_path)?;

    let mut issues = validate_report(&candidate);
    issues.extend(compare_reports(&baseline, &candidate, &config));

    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == ValidationSeverity::Error)
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == ValidationSeverity::Warning)
        .count();

    for issue in &issues {
        println!(
            "{:?}: {} target {}: {}",
            issue.severity, issue.scenario_id, issue.target, issue.message
        );
    }

    println!(
        "compared {} baseline points against {} candidate points: {} error(s), {} warning(s)",
        baseline.points.len(),
        candidate.points.len(),
        error_count,
        warning_count
    );

    if error_count > 0 {
        Err(format!("comparison failed with {} error(s)", error_count))
    } else {
        Ok(())
    }
}

fn parse_f64(args: &[String], index: usize, label: &str) -> Result<f64, String> {
    args.get(index)
        .ok_or_else(|| format!("{} requires a value", label))?
        .parse::<f64>()
        .map_err(|e| format!("{} must be numeric: {}", label, e))
}

fn print_help() {
    println!(
        "Usage: optimizer_compare BASELINE_RESULTS_JSON CANDIDATE_RESULTS_JSON [options]\n\
         \n\
         Options:\n\
           --lcoe-tolerance N      Allowed candidate LCOE increase in $/MWh (default 0.10)\n\
           --achieved-tolerance N  Warning threshold for achieved clean-match drift in percentage points (default 0.25)\n\
           --runtime-factor N      Warning threshold as a multiple of baseline runtime (default 1.5)\n\
           --runtime-min-delta N   Minimum runtime increase before warning in ms (default 50)\n\
         \n\
         Example:\n\
         cargo run --release --bin optimizer_compare -- ../optimizer_evals/baselines/core/results.json ../optimizer_evals/runs/latest/results.json"
    );
}
