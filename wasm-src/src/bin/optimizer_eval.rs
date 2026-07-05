use energy_simulator::eval::{
    find_refactor_root, read_report, run_suite_from_path, scenario_path_for_suite, validate_report,
    write_report, ValidationIssue, ValidationSeverity,
};
use std::env;
use std::fs;
use std::path::PathBuf;
use std::process;

fn main() {
    if let Err(e) = run() {
        eprintln!("optimizer_eval error: {}", e);
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let mut suite_name = "core".to_string();
    let mut scenario_file: Option<PathBuf> = None;
    let mut out_dir: Option<PathBuf> = None;
    let mut validate = false;

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--suite" => {
                i += 1;
                suite_name = args
                    .get(i)
                    .ok_or_else(|| "--suite requires a value".to_string())?
                    .clone();
            }
            "--scenario-file" => {
                i += 1;
                scenario_file =
                    Some(PathBuf::from(args.get(i).ok_or_else(|| {
                        "--scenario-file requires a value".to_string()
                    })?));
            }
            "--out" => {
                i += 1;
                out_dir = Some(PathBuf::from(
                    args.get(i)
                        .ok_or_else(|| "--out requires a value".to_string())?,
                ));
            }
            "--validate" => {
                validate = true;
            }
            "--help" | "-h" => {
                print_help();
                return Ok(());
            }
            other => return Err(format!("Unknown argument: {}", other)),
        }
        i += 1;
    }

    let scenario_path = match scenario_file {
        Some(path) => path,
        None if suite_name == "all" => return run_all_suites(out_dir, validate),
        None => scenario_path_for_suite(&suite_name)?,
    };

    let report = run_suite_from_path(&scenario_path)?;
    let output_path = if let Some(out) = out_dir {
        Some(write_report(&report, &out)?)
    } else {
        None
    };

    println!(
        "suite={} scenarios={} points={} success={} off_target={} errors={} validation_errors={} mean_runtime_ms={:.2}",
        report.suite,
        report.summary.scenario_count,
        report.summary.point_count,
        report.summary.success_count,
        report.summary.off_target_count,
        report.summary.error_count,
        report.summary.validation_error_count,
        report.summary.mean_runtime_ms
    );
    if let Some(path) = &output_path {
        println!("wrote {}", path.display());
    }

    if validate {
        let validation_report = if let Some(path) = output_path {
            read_report(&path)?
        } else {
            report
        };
        let issues = validate_report(&validation_report);
        let error_count = count_issues(&issues, ValidationSeverity::Error);
        print_issues(&issues);
        if error_count > 0 {
            return Err(format!("validation failed with {} error(s)", error_count));
        }
    }

    Ok(())
}

fn run_all_suites(out_dir: Option<PathBuf>, validate: bool) -> Result<(), String> {
    let root = find_refactor_root()?;
    let scenarios_dir = root.join("optimizer_evals").join("scenarios");
    let mut scenario_paths = Vec::new();
    for entry in fs::read_dir(&scenarios_dir)
        .map_err(|e| format!("Failed to read {}: {}", scenarios_dir.display(), e))?
    {
        let entry = entry.map_err(|e| format!("Failed to read scenario entry: {}", e))?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
            scenario_paths.push(path);
        }
    }
    scenario_paths.sort();

    let mut total_points = 0;
    let mut total_errors = 0;
    let mut total_warnings = 0;

    for scenario_path in scenario_paths {
        let report = run_suite_from_path(&scenario_path)?;
        total_points += report.summary.point_count;

        let output_path = if let Some(out) = &out_dir {
            Some(write_report(&report, &out.join(&report.suite))?)
        } else {
            None
        };

        println!(
            "suite={} scenarios={} points={} success={} off_target={} errors={} validation_errors={} mean_runtime_ms={:.2}",
            report.suite,
            report.summary.scenario_count,
            report.summary.point_count,
            report.summary.success_count,
            report.summary.off_target_count,
            report.summary.error_count,
            report.summary.validation_error_count,
            report.summary.mean_runtime_ms
        );
        if let Some(path) = &output_path {
            println!("wrote {}", path.display());
        }

        if validate {
            let validation_report = if let Some(path) = output_path {
                read_report(&path)?
            } else {
                report
            };
            let issues = validate_report(&validation_report);
            print_issues(&issues);
            total_errors += count_issues(&issues, ValidationSeverity::Error);
            total_warnings += count_issues(&issues, ValidationSeverity::Warning);
        }
    }

    println!(
        "all suites complete: {} point(s), {} validation error(s), {} validation warning(s)",
        total_points, total_errors, total_warnings
    );

    if total_errors > 0 {
        Err(format!(
            "validation failed across all suites with {} error(s)",
            total_errors
        ))
    } else {
        Ok(())
    }
}

fn count_issues(issues: &[ValidationIssue], severity: ValidationSeverity) -> usize {
    issues
        .iter()
        .filter(|issue| issue.severity == severity)
        .count()
}

fn print_issues(issues: &[ValidationIssue]) {
    for issue in issues {
        println!(
            "{:?}: {} target {}: {}",
            issue.severity, issue.scenario_id, issue.target, issue.message
        );
    }
}

fn print_help() {
    println!(
        "Usage: optimizer_eval [--suite NAME | --scenario-file PATH] [--out DIR] [--validate]\n\
         \n\
         Examples:\n\
         cargo run --release --bin optimizer_eval -- --suite regression_cases --out ../optimizer_evals/runs/latest --validate\n\
         cargo run --release --bin optimizer_eval -- --suite all --out ../optimizer_evals/runs/all_latest --validate"
    );
}
