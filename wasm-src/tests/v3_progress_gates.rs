#![cfg(feature = "experimental-v3")]

use energy_simulator::optimizer::run_suite;

#[test]
#[ignore = "heavy integration test; run explicitly in release mode"]
fn quick_suite_meets_progress_gates() {
    let report = run_suite("quick").expect("quick suite execution failed");
    assert_eq!(report.cases.len(), 8, "quick suite must include 8 cases");

    let gates = &report.gates;

    let runtime_ratio = gates
        .runtime_ratio_median
        .expect("runtime ratio median should be present");
    assert!(
        runtime_ratio <= gates.runtime_ratio_limit,
        "runtime gate failed: median ratio={} limit={}",
        runtime_ratio,
        gates.runtime_ratio_limit
    );

    let p95_gap = gates
        .p95_v3_gap_pct
        .expect("p95 gap should be present for quick suite");
    assert!(
        p95_gap <= gates.p95_gap_limit_pct,
        "gap gate failed: p95 gap={} limit={}",
        p95_gap,
        gates.p95_gap_limit_pct
    );

    let max_dev = gates
        .max_v3_deviation
        .expect("max v3 deviation should be present");
    assert!(
        max_dev <= gates.deviation_limit,
        "deviation gate failed: max_dev={} limit={}",
        max_dev,
        gates.deviation_limit
    );

    assert!(gates.pass, "overall quick gate should pass");
}
