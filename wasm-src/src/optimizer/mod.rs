//! Portfolio Optimization Module
//!
//! The core optimizer is V2 (hierarchical), which provides:
//! - ~20ms optimization per target
//! - Precise target compliance (±0.2%)
//! - Support for all battery modes and resource combinations
//!
//! V1 (adaptive) is deprecated and kept for reference only.
//!
//! ## Model-Based Optimization
//!
//! The optimizer can optionally use pre-computed EmpiricalModel lookup tables
//! for faster candidate filtering. Models are stored in a thread-local cache
//! and loaded via WASM exports:
//! - `load_model(zone, battery_mode, bytes)` - Load model from binary
//! - `is_model_loaded(zone, battery_mode)` - Check if cached
//! - `clear_models()` - Free memory

pub mod cache;
pub mod empirical_model;
pub mod greedy;
pub mod incremental_walk;
pub mod model_cache;
pub mod v2_accuracy_audit;
pub mod v2_adaptive_audit;
pub mod v2_hierarchical;
#[cfg(feature = "experimental-v3")]
pub mod v3;

#[deprecated(since = "0.2.0", note = "Use run_v2_optimizer instead")]
pub mod v1_adaptive;

pub use cache::EvalCache;
pub use empirical_model::{EmpiricalModel, GridConfig, Portfolio, TrainingSample};
pub use incremental_walk::run_incremental_walk;
pub use model_cache::{clear_models, get_model, is_model_loaded, load_model, loaded_models};
pub use v2_accuracy_audit::{
    run_v2_accuracy_audit_suite, V2AccuracyAuditCaseReport, V2AccuracyAuditSuiteReport,
    V2AccuracySummary,
};
pub use v2_adaptive_audit::{
    run_v2_adaptive_audit, AdaptiveFixRecommendation, AdaptiveOracleConfig, AdaptiveTrialReport,
    LeverMultipliers, V2AdaptiveAuditConfig, V2AdaptiveAuditReport, V2AdaptiveAuditSummary,
};
pub use v2_hierarchical::{
    run_v2_optimizer, run_v2_optimizer_mode, run_v2_optimizer_mode_detailed, run_v2_sweep,
    run_v2_sweep_mode, V2AccurateConfig, V2AccurateDiagnostics, V2AccurateStopReason, V2Mode,
    V2StepSchedule,
};
#[cfg(feature = "experimental-v3")]
pub use v3::{
    apply_quick_fail_state, run_suite, run_v3_global_grid, run_v3_oracle, run_v3_optimizer,
    write_suite_report, CaseReport, SuiteReport, V3Diagnostics, V3Result, V3SearchConfig,
};

// Re-export V2 as the primary optimizer
pub use v2_hierarchical::run_v2_optimizer as run_optimizer;

// Deprecated V1 export
#[allow(deprecated)]
pub use v1_adaptive::run_v1_optimizer;
