use energy_simulator::optimizer::{
    run_v2_adaptive_audit, AdaptiveTrialReport, V2AdaptiveAuditConfig,
};
use energy_simulator::BatteryMode;
use std::path::PathBuf;

fn print_usage() {
    eprintln!(
        "Usage: cargo run --release --features \"native\" --bin adaptive_audit_v2 -- \
--json-out <path> [--jsonl-out <path>] [--trials N] [--batch-size N] [--max-generations N] \
[--zones california,texas] [--targets 70,85,95,99] [--min-multiplier X] [--max-multiplier X] \
[--coarse-top-k N] [--fine-top-k N] [--seed N] [--battery-mode default|peak_shaver|hybrid] [--fail-on-gate]"
    );
}

fn parse_f64_list(raw: &str) -> Result<Vec<f64>, String> {
    let mut values = Vec::new();
    for part in raw.split(',') {
        let trimmed = part.trim();
        if trimmed.is_empty() {
            continue;
        }
        let value = trimmed
            .parse::<f64>()
            .map_err(|e| format!("Invalid float '{}': {}", trimmed, e))?;
        values.push(value);
    }
    if values.is_empty() {
        return Err("Expected at least one numeric value".to_string());
    }
    Ok(values)
}

fn parse_string_list(raw: &str) -> Result<Vec<String>, String> {
    let values: Vec<String> = raw
        .split(',')
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| part.to_string())
        .collect();
    if values.is_empty() {
        return Err("Expected at least one value".to_string());
    }
    Ok(values)
}

fn parse_battery_mode(raw: &str) -> Option<BatteryMode> {
    match raw.to_lowercase().as_str() {
        "default" => Some(BatteryMode::Default),
        "peak_shaver" | "peak" => Some(BatteryMode::PeakShaver),
        "hybrid" => Some(BatteryMode::Hybrid),
        _ => None,
    }
}

fn write_json(
    report: &energy_simulator::optimizer::V2AdaptiveAuditReport,
    path: &PathBuf,
) -> Result<(), String> {
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

fn write_jsonl(trials: &[AdaptiveTrialReport], path: &PathBuf) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| {
            format!(
                "Failed to create output directory {}: {}",
                parent.display(),
                e
            )
        })?;
    }
    let mut out = String::new();
    for trial in trials {
        let line = serde_json::to_string(trial)
            .map_err(|e| format!("Failed to serialize trial {}: {}", trial.trial_id, e))?;
        out.push_str(&line);
        out.push('\n');
    }
    std::fs::write(path, out)
        .map_err(|e| format!("Failed to write trial log {}: {}", path.display(), e))
}

fn print_top_trials(report: &energy_simulator::optimizer::V2AdaptiveAuditReport) {
    let mut ranked: Vec<&AdaptiveTrialReport> = report.trials.iter().collect();
    ranked.sort_by(|a, b| {
        b.weakness_score
            .partial_cmp(&a.weakness_score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    println!("Top weak spots:");
    for trial in ranked.into_iter().take(5) {
        println!(
            "  trial={} zone={} target={:.1} score={:.3} fine_gap={:?} runtime_ratio={:?} flags={}",
            trial.trial_id,
            trial.zone,
            trial.target,
            trial.weakness_score,
            trial.selected_gap_vs_fine_pct,
            trial.runtime_ratio_accurate_vs_fast,
            trial.flags.join(",")
        );
    }
}

fn main() {
    let mut config = V2AdaptiveAuditConfig::default();
    let mut json_out = PathBuf::from("target/v2_adaptive_audit_report.json");
    let mut jsonl_out: Option<PathBuf> =
        Some(PathBuf::from("target/v2_adaptive_audit_trials.jsonl"));
    let mut fail_on_gate = false;

    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let next_required = |name: &str, args: &mut std::iter::Skip<std::env::Args>| -> String {
            args.next().unwrap_or_else(|| {
                eprintln!("Missing value for {}", name);
                print_usage();
                std::process::exit(2);
            })
        };

        match arg.as_str() {
            "--json-out" => json_out = PathBuf::from(next_required("--json-out", &mut args)),
            "--jsonl-out" => {
                jsonl_out = Some(PathBuf::from(next_required("--jsonl-out", &mut args)))
            }
            "--no-jsonl" => jsonl_out = None,
            "--trials" => {
                config.trial_budget =
                    next_required("--trials", &mut args)
                        .parse()
                        .unwrap_or_else(|_| {
                            eprintln!("Invalid --trials value");
                            std::process::exit(2);
                        })
            }
            "--batch-size" => {
                config.batch_size = next_required("--batch-size", &mut args)
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid --batch-size value");
                        std::process::exit(2);
                    })
            }
            "--max-generations" => {
                config.max_generations = next_required("--max-generations", &mut args)
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid --max-generations value");
                        std::process::exit(2);
                    })
            }
            "--zones" => {
                config.zones = parse_string_list(&next_required("--zones", &mut args))
                    .unwrap_or_else(|e| {
                        eprintln!("{}", e);
                        std::process::exit(2);
                    })
            }
            "--targets" => {
                config.targets = parse_f64_list(&next_required("--targets", &mut args))
                    .unwrap_or_else(|e| {
                        eprintln!("{}", e);
                        std::process::exit(2);
                    })
            }
            "--min-multiplier" => {
                config.min_multiplier = next_required("--min-multiplier", &mut args)
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid --min-multiplier value");
                        std::process::exit(2);
                    })
            }
            "--max-multiplier" => {
                config.max_multiplier = next_required("--max-multiplier", &mut args)
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid --max-multiplier value");
                        std::process::exit(2);
                    })
            }
            "--coarse-top-k" => {
                config.coarse_oracle_top_k = next_required("--coarse-top-k", &mut args)
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid --coarse-top-k value");
                        std::process::exit(2);
                    })
            }
            "--fine-top-k" => {
                config.fine_oracle_top_k = next_required("--fine-top-k", &mut args)
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid --fine-top-k value");
                        std::process::exit(2);
                    })
            }
            "--seed" => {
                config.seed = next_required("--seed", &mut args)
                    .parse()
                    .unwrap_or_else(|_| {
                        eprintln!("Invalid --seed value");
                        std::process::exit(2);
                    })
            }
            "--battery-mode" => {
                let raw = next_required("--battery-mode", &mut args);
                config.battery_mode = parse_battery_mode(&raw).unwrap_or_else(|| {
                    eprintln!(
                        "Invalid battery mode '{}'. Use default|peak_shaver|hybrid",
                        raw
                    );
                    std::process::exit(2);
                });
            }
            "--fail-on-gate" => fail_on_gate = true,
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

    let report = match run_v2_adaptive_audit(&config) {
        Ok(report) => report,
        Err(e) => {
            eprintln!("Adaptive audit failed: {}", e);
            std::process::exit(2);
        }
    };

    if let Err(e) = write_json(&report, &json_out) {
        eprintln!("{}", e);
        std::process::exit(2);
    }

    if let Some(path) = &jsonl_out {
        if let Err(e) = write_jsonl(&report.trials, path) {
            eprintln!("{}", e);
            std::process::exit(2);
        }
    }

    println!("Adaptive audit completed");
    println!("  Trials: {}", report.summary.trial_count);
    println!("  Successful trials: {}", report.summary.successful_trials);
    println!(
        "  Coarse oracle trials: {}",
        report.summary.coarse_oracle_trials
    );
    println!(
        "  Fine oracle trials: {}",
        report.summary.fine_oracle_trials
    );
    println!(
        "  Median runtime ratio (accurate/fast): {:?}",
        report.summary.median_runtime_ratio_accurate_vs_fast
    );
    println!(
        "  P95 fine-gap (%): {:?}",
        report.summary.p95_selected_gap_vs_fine_pct
    );
    println!(
        "  Gates: runtime_pass={} gap_pass={} overall_pass={}",
        report.summary.runtime_pass, report.summary.gap_pass, report.summary.pass
    );
    println!("  JSON report: {}", json_out.display());
    if let Some(path) = jsonl_out {
        println!("  Trial log: {}", path.display());
    }

    print_top_trials(&report);

    if !report.recommendations.is_empty() {
        println!("Recommendations:");
        for recommendation in &report.recommendations {
            println!(
                "  [{}] {} | action: {} | evidence={} cases",
                recommendation.key,
                recommendation.rationale,
                recommendation.suggested_action,
                recommendation.evidence_count
            );
        }
    }

    if fail_on_gate && !report.summary.pass {
        std::process::exit(2);
    }
}
