use energy_simulator::{run_v2_accuracy_audit_suite, V2Mode};

#[test]
#[ignore = "heavy audit suite; run explicitly in release mode"]
fn test_accuracy_audit_hard_suite_gates() {
    let report =
        run_v2_accuracy_audit_suite("hard", V2Mode::Accurate).expect("accuracy audit suite failed");

    assert!(
        report.summary.case_count >= 4,
        "expected at least 4 hard cases, got {}",
        report.summary.case_count
    );
    assert_eq!(
        report.summary.successful_cases, report.summary.case_count,
        "all audit cases should succeed"
    );

    let p95_gap = report
        .summary
        .p95_selected_gap_pct
        .expect("p95 gap should be present");
    assert!(
        p95_gap <= report.summary.p95_gap_limit_pct,
        "p95 gap gate failed: {:.4}% > {:.4}%",
        p95_gap,
        report.summary.p95_gap_limit_pct
    );

    let runtime_ratio = report
        .summary
        .median_runtime_ratio_selected_vs_fast
        .expect("median runtime ratio should be present");
    assert!(
        runtime_ratio <= report.summary.runtime_ratio_limit,
        "runtime gate failed: {:.4} > {:.4}",
        runtime_ratio,
        report.summary.runtime_ratio_limit
    );
}

#[test]
#[ignore = "very heavy confidence suite; run explicitly in release mode"]
fn test_accuracy_audit_confidence_suite_gates() {
    let report = run_v2_accuracy_audit_suite("confidence", V2Mode::Accurate)
        .expect("accuracy confidence suite failed");

    assert!(
        report.summary.case_count >= 10,
        "expected at least 10 confidence cases, got {}",
        report.summary.case_count
    );
    assert_eq!(
        report.summary.successful_cases, report.summary.case_count,
        "all confidence cases should succeed"
    );

    let p95_gap = report
        .summary
        .p95_selected_gap_pct
        .expect("p95 gap should be present");
    assert!(
        p95_gap <= report.summary.p95_gap_limit_pct,
        "confidence p95 gap failed: {:.4}% > {:.4}%",
        p95_gap,
        report.summary.p95_gap_limit_pct
    );
}
