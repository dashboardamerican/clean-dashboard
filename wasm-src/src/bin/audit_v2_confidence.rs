use energy_simulator::optimizer::{run_v2_accuracy_audit_suite, V2Mode};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Serialize)]
struct ConfidenceRunSummary {
    run_index: usize,
    pass: bool,
    successful_cases: usize,
    case_count: usize,
    p95_gap_pct: Option<f64>,
    median_runtime_ratio: Option<f64>,
    worst_case_name: Option<String>,
    worst_case_gap_pct: Option<f64>,
    report_path: String,
}

#[derive(Clone, Debug, Serialize)]
struct ConfidenceAggregateReport {
    generated_unix_ms: u128,
    suite: String,
    mode: String,
    repeats: usize,
    pass_count: usize,
    fail_count: usize,
    all_pass: bool,
    median_p95_gap_pct: Option<f64>,
    max_p95_gap_pct: Option<f64>,
    median_runtime_ratio: Option<f64>,
    max_runtime_ratio: Option<f64>,
    runs: Vec<ConfidenceRunSummary>,
}

fn print_usage() {
    eprintln!(
        "Usage: cargo run --release --features \"native\" --bin audit_v2_confidence -- \
--suite <confidence|hard|quick|smoke> --mode <accurate|fast> [--repeats N] \
[--summary-out <path>] [--run-dir <path>] [--allow-fail]"
    );
}

fn parse_mode(raw: &str) -> Option<V2Mode> {
    match raw.to_lowercase().as_str() {
        "fast" => Some(V2Mode::Fast),
        "accurate" => Some(V2Mode::Accurate),
        _ => None,
    }
}

fn mode_name(mode: V2Mode) -> &'static str {
    match mode {
        V2Mode::Fast => "fast",
        V2Mode::Accurate => "accurate",
    }
}

fn write_json<T: Serialize>(value: &T, path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create output directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }
    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write {}: {}", path.display(), e))
}

fn median(values: &[f64]) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mid = sorted.len() / 2;
    if sorted.len() % 2 == 0 {
        Some((sorted[mid - 1] + sorted[mid]) / 2.0)
    } else {
        Some(sorted[mid])
    }
}

fn main() {
    let mut suite = String::from("confidence");
    let mut mode = V2Mode::Accurate;
    let mut repeats: usize = 3;
    let mut summary_out = PathBuf::from("target/v2_accuracy_confidence_summary.json");
    let mut run_dir = PathBuf::from("target/v2_accuracy_confidence_runs");
    let mut allow_fail = false;

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
            "--repeats" => {
                if let Some(value) = args.next() {
                    repeats = value.parse::<usize>().unwrap_or_else(|_| {
                        eprintln!("Invalid repeats value '{}'", value);
                        std::process::exit(2);
                    });
                    if repeats == 0 {
                        eprintln!("--repeats must be >= 1");
                        std::process::exit(2);
                    }
                } else {
                    eprintln!("Missing value for --repeats");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--summary-out" => {
                if let Some(value) = args.next() {
                    summary_out = PathBuf::from(value);
                } else {
                    eprintln!("Missing value for --summary-out");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--run-dir" => {
                if let Some(value) = args.next() {
                    run_dir = PathBuf::from(value);
                } else {
                    eprintln!("Missing value for --run-dir");
                    print_usage();
                    std::process::exit(2);
                }
            }
            "--allow-fail" => allow_fail = true,
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

    let mode_label = mode_name(mode).to_string();
    let mut run_summaries: Vec<ConfidenceRunSummary> = Vec::with_capacity(repeats);
    let mut p95_values: Vec<f64> = Vec::with_capacity(repeats);
    let mut runtime_values: Vec<f64> = Vec::with_capacity(repeats);

    for run_index in 1..=repeats {
        let report = match run_v2_accuracy_audit_suite(&suite, mode) {
            Ok(report) => report,
            Err(e) => {
                eprintln!(
                    "Run {} failed to execute suite '{}': {}",
                    run_index, suite, e
                );
                std::process::exit(2);
            }
        };

        let run_path = run_dir.join(format!("{}_{}_run{:02}.json", suite, mode_label, run_index));
        if let Err(e) = write_json(&report, &run_path) {
            eprintln!("{}", e);
            std::process::exit(2);
        }

        if let Some(v) = report.summary.p95_selected_gap_pct {
            p95_values.push(v);
        }
        if let Some(v) = report.summary.median_runtime_ratio_selected_vs_fast {
            runtime_values.push(v);
        }

        let worst_case = report
            .cases
            .iter()
            .filter_map(|c| c.selected_gap_vs_oracle_pct.map(|gap| (&c.case_name, gap)))
            .max_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));

        let run_summary = ConfidenceRunSummary {
            run_index,
            pass: report.summary.pass,
            successful_cases: report.summary.successful_cases,
            case_count: report.summary.case_count,
            p95_gap_pct: report.summary.p95_selected_gap_pct,
            median_runtime_ratio: report.summary.median_runtime_ratio_selected_vs_fast,
            worst_case_name: worst_case.map(|(name, _)| name.clone()),
            worst_case_gap_pct: worst_case.map(|(_, gap)| gap),
            report_path: run_path.display().to_string(),
        };
        println!(
            "Run {}: pass={} p95_gap={:?} runtime_ratio={:?} worst_case={:?} worst_gap={:?}",
            run_index,
            run_summary.pass,
            run_summary.p95_gap_pct,
            run_summary.median_runtime_ratio,
            run_summary.worst_case_name,
            run_summary.worst_case_gap_pct
        );
        run_summaries.push(run_summary);
    }

    let pass_count = run_summaries.iter().filter(|r| r.pass).count();
    let fail_count = run_summaries.len() - pass_count;
    let all_pass = fail_count == 0;

    let aggregate = ConfidenceAggregateReport {
        generated_unix_ms: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0),
        suite: suite.clone(),
        mode: mode_label,
        repeats,
        pass_count,
        fail_count,
        all_pass,
        median_p95_gap_pct: median(&p95_values),
        max_p95_gap_pct: p95_values.into_iter().reduce(f64::max),
        median_runtime_ratio: median(&runtime_values),
        max_runtime_ratio: runtime_values.into_iter().reduce(f64::max),
        runs: run_summaries,
    };

    if let Err(e) = write_json(&aggregate, &summary_out) {
        eprintln!("{}", e);
        std::process::exit(2);
    }

    println!("Aggregate summary:");
    println!(
        "  suite={} mode={} repeats={}",
        aggregate.suite, aggregate.mode, repeats
    );
    println!(
        "  pass_count={} fail_count={} all_pass={}",
        aggregate.pass_count, aggregate.fail_count, aggregate.all_pass
    );
    println!(
        "  median_p95_gap={:?} max_p95_gap={:?}",
        aggregate.median_p95_gap_pct, aggregate.max_p95_gap_pct
    );
    println!(
        "  median_runtime_ratio={:?} max_runtime_ratio={:?}",
        aggregate.median_runtime_ratio, aggregate.max_runtime_ratio
    );
    println!("  summary report: {}", summary_out.display());

    if !allow_fail && !aggregate.all_pass {
        std::process::exit(2);
    }
}
