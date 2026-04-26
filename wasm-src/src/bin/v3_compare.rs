#[cfg(feature = "experimental-v3")]
use energy_simulator::optimizer::{apply_quick_fail_state, run_suite, write_suite_report};
#[cfg(feature = "experimental-v3")]
use serde_json::json;
#[cfg(feature = "experimental-v3")]
use std::path::PathBuf;
#[cfg(feature = "experimental-v3")]
use std::{fs::OpenOptions, io::Write};

#[cfg(feature = "experimental-v3")]
fn print_usage() {
    eprintln!(
        "Usage: cargo run --release --features \"experimental-v3 native\" --bin v3_compare -- --suite <smoke|quick|standard|hardest|comprehensive> [--json-out <path>] [--jsonl-out <path>] [--fail-state <path>]"
    );
}

#[cfg(feature = "experimental-v3")]
fn append_jsonl(path: &PathBuf, value: serde_json::Value) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| format!("Failed to open jsonl output {}: {}", path.display(), e))?;

    let line = serde_json::to_string(&value)
        .map_err(|e| format!("Failed to serialize jsonl payload: {}", e))?;
    writeln!(file, "{}", line).map_err(|e| format!("Failed to write jsonl line {}: {}", path.display(), e))
}

#[cfg(feature = "experimental-v3")]
fn main() {
    let mut suite = String::from("quick");
    let mut json_out: Option<PathBuf> = None;
    let mut jsonl_out: Option<PathBuf> = None;
    let mut fail_state = PathBuf::from("target/v3_quick_fail_state.json");

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
            "--json-out" => {
                if let Some(value) = args.next() {
                    json_out = Some(PathBuf::from(value));
                } else {
                    eprintln!("Missing value for --json-out");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--fail-state" => {
                if let Some(value) = args.next() {
                    fail_state = PathBuf::from(value);
                } else {
                    eprintln!("Missing value for --fail-state");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--jsonl-out" => {
                if let Some(value) = args.next() {
                    jsonl_out = Some(PathBuf::from(value));
                } else {
                    eprintln!("Missing value for --jsonl-out");
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

    let mut report = match run_suite(&suite) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Failed to run suite '{}': {}", suite, e);
            std::process::exit(2);
        }
    };

    if suite == "quick" {
        if let Err(e) = apply_quick_fail_state(&mut report, &fail_state) {
            eprintln!("Warning: failed to update quick fail state: {}", e);
        }
    }

    let output_path =
        json_out.unwrap_or_else(|| PathBuf::from(format!("target/v3_{}_report.json", suite)));

    if let Err(e) = write_suite_report(&report, &output_path) {
        eprintln!("Failed to write report to {}: {}", output_path.display(), e);
        std::process::exit(2);
    }

    println!("Suite: {}", report.suite);
    println!("Cases: {}", report.cases.len());
    println!(
        "Gates: runtime_pass={} gap_pass={} deviation_pass={} overall_pass={}",
        report.gates.runtime_pass,
        report.gates.gap_pass,
        report.gates.deviation_pass,
        report.gates.pass,
    );
    if let Some(v) = report.gates.runtime_ratio_median {
        println!("Median runtime ratio (v3/v2): {:.4}", v);
    }
    if let Some(v) = report.gates.p95_v3_gap_pct {
        println!("P95 v3 gap vs fine oracle (%): {:.4}", v);
    }
    if let Some(v) = report.gates.max_v3_deviation {
        println!("Max v3 deviation: {:.6}", v);
    }
    if let Some(v) = report.consecutive_quick_failures {
        println!("Consecutive quick failures: {}", v);
    }
    println!("Report: {}", output_path.display());
    if let Some(path) = &jsonl_out {
        let line = json!({
            "suite": report.suite,
            "generated_unix_ms": report.generated_unix_ms,
            "runtime_ratio_median": report.gates.runtime_ratio_median,
            "runtime_ratio_limit": report.gates.runtime_ratio_limit,
            "p95_v3_gap_pct": report.gates.p95_v3_gap_pct,
            "p95_gap_limit_pct": report.gates.p95_gap_limit_pct,
            "max_v3_deviation": report.gates.max_v3_deviation,
            "deviation_limit": report.gates.deviation_limit,
            "runtime_pass": report.gates.runtime_pass,
            "gap_pass": report.gates.gap_pass,
            "deviation_pass": report.gates.deviation_pass,
            "pass": report.gates.pass,
            "consecutive_quick_failures": report.consecutive_quick_failures,
            "abandon_recommended": report.abandon_recommended,
        });
        if let Err(e) = append_jsonl(path, line) {
            eprintln!("Warning: failed to append jsonl {}: {}", path.display(), e);
        }
    }

    if report.abandon_recommended {
        eprintln!("Abandon recommendation triggered: quick suite failed twice consecutively.");
    }

    if !report.gates.pass {
        std::process::exit(2);
    }
}

#[cfg(not(feature = "experimental-v3"))]
fn main() {
    eprintln!("This binary requires feature 'experimental-v3'.");
    std::process::exit(2);
}
