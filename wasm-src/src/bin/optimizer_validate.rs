use energy_simulator::eval::{read_report, validate_report, ValidationSeverity};
use std::env;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("optimizer_validate error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "Usage: optimizer_validate <results.json>".to_string())?;

    let report = read_report(&path)?;
    let issues = validate_report(&report);
    let error_count = issues
        .iter()
        .filter(|issue| issue.severity == ValidationSeverity::Error)
        .count();
    let warning_count = issues
        .iter()
        .filter(|issue| issue.severity == ValidationSeverity::Warning)
        .count();

    println!(
        "validated {} points: {} error(s), {} warning(s)",
        report.summary.point_count, error_count, warning_count
    );
    for issue in &issues {
        println!(
            "{:?}: {} target {}: {}",
            issue.severity, issue.scenario_id, issue.target, issue.message
        );
    }

    if error_count > 0 {
        return Err(format!("validation failed with {} error(s)", error_count));
    }

    Ok(())
}
