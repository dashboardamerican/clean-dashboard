use energy_simulator::optimizer::{
    run_v2_accuracy_audit_suite, V2AccuracyAuditSuiteReport, V2Mode,
};
use std::path::PathBuf;

fn print_usage() {
    eprintln!(
        "Usage: cargo run --release --features \"native\" --bin audit_v2_accuracy -- --suite <smoke|hard|quick|confidence> --mode <fast|accurate> [--json-out <path>]"
    );
}

fn parse_mode(raw: &str) -> Option<V2Mode> {
    match raw.to_lowercase().as_str() {
        "fast" => Some(V2Mode::Fast),
        "accurate" => Some(V2Mode::Accurate),
        _ => None,
    }
}

fn write_report(report: &V2AccuracyAuditSuiteReport, path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create output directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }
    let json = serde_json::to_string_pretty(report)
        .map_err(|e| format!("Failed to serialize report: {}", e))?;
    std::fs::write(path, json)
        .map_err(|e| format!("Failed to write report {}: {}", path.display(), e))
}

fn main() {
    let mut suite = String::from("hard");
    let mut mode = V2Mode::Accurate;
    let mut json_out: Option<PathBuf> = None;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--suite" => {
                if let Some(value) = args.next() {
                    suite = value;
                } else {
                    eprintln!("Missing value for --suite");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--mode" => {
                if let Some(value) = args.next() {
                    if let Some(parsed) = parse_mode(&value) {
                        mode = parsed;
                    } else {
                        eprintln!("Invalid mode '{}'. Expected fast|accurate", value);
                        print_usage();
                        std::process::exit(2);
                    }
                } else {
                    eprintln!("Missing value for --mode");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--json-out" => {
                if let Some(value) = args.next() {
                    json_out = Some(PathBuf::from(value));
                } else {
                    eprintln!("Missing value for --json-out");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--help" | "-h" => {
                print_usage();
                std::process::exit(0);
            }
            unknown => {
                eprintln!("Unknown argument: {}", unknown);
                print_usage();
                std::process::exit(2);
            }
        }
    }

    let report = match run_v2_accuracy_audit_suite(&suite, mode) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Audit run failed: {}", e);
            std::process::exit(2);
        }
    };

    let output_path = json_out.unwrap_or_else(|| {
        PathBuf::from(format!(
            "target/v2_accuracy_{}_{}.json",
            suite,
            match mode {
                V2Mode::Fast => "fast",
                V2Mode::Accurate => "accurate",
            }
        ))
    });

    if let Err(e) = write_report(&report, &output_path) {
        eprintln!("{}", e);
        std::process::exit(2);
    }

    println!("Suite: {}", report.suite);
    println!("Mode: {}", report.mode);
    println!("Cases: {}", report.summary.case_count);
    println!("Successful cases: {}", report.summary.successful_cases);
    if let Some(v) = report.summary.mean_selected_gap_pct {
        println!("Mean selected gap vs oracle (%): {:.4}", v);
    }
    if let Some(v) = report.summary.median_selected_gap_pct {
        println!("Median selected gap vs oracle (%): {:.4}", v);
    }
    if let Some(v) = report.summary.p95_selected_gap_pct {
        println!("P95 selected gap vs oracle (%): {:.4}", v);
    }
    if let Some(v) = report.summary.median_runtime_ratio_selected_vs_fast {
        println!("Median runtime ratio selected/fast: {:.4}", v);
    }
    println!(
        "Gates: runtime_pass={} gap_pass={} overall_pass={}",
        report.summary.runtime_pass, report.summary.gap_pass, report.summary.pass
    );
    println!("Report: {}", output_path.display());

    if !report.summary.pass {
        std::process::exit(2);
    }
}
