# Achieving Global Optimum in V2 Portfolio Optimization

> Design document capturing learnings and approach for finding the true minimum-cost portfolio at any clean energy target.

## Table of Contents
1. [Problem Statement](#1-problem-statement)
2. [Background: The Optimization Landscape](#2-background-the-optimization-landscape)
3. [Issue Discovery: The Overbuild Blind Spot](#3-issue-discovery-the-overbuild-blind-spot)
4. [Root Cause Analysis](#4-root-cause-analysis)
5. [Key Insights](#5-key-insights)
6. [Solution Design](#6-solution-design)
7. [Validation Methodology](#7-validation-methodology)
8. [Implementation Guidance](#8-implementation-guidance)
9. [Future Considerations](#9-future-considerations)
10. [EmpiricalModel Integration](#10-empiricalmodel-integration-lookup-table-acceleration)

---

## 1. Problem Statement

The V2 optimizer finds minimum-cost portfolios (solar, wind, storage, clean firm) to achieve a target clean energy percentage. The optimizer must:

- **Hit the target precisely**: A 99% target means achieving approximately 99%, not just "at least 99%"
- **Find the global minimum LCOE**: Among all portfolios that hit the target, find the cheapest
- **Run fast**: Target execution time is under 50ms for interactive use

**The discovered issue**: At high clean match targets (95%+), V2 consistently returns portfolios that are $1-2/MWh more expensive than the true global optimum.

---

## 2. Background: The Optimization Landscape

### 2.1 The Resources

The optimizer controls four generation resources:

| Resource | Role | Characteristics |
|----------|------|-----------------|
| **Solar** | Variable renewable | Cheap, but saturates due to curtailment at high penetration |
| **Wind** | Variable renewable | Moderate cost, higher capacity factor, also saturates |
| **Storage** | Time-shifting | Recovers curtailed energy, modifies renewable saturation curves |
| **Clean Firm (CF)** | Dispatchable clean | Expensive, but linear contribution with no saturation |

### 2.2 The Cost Structure (Default Costs)

```rust
solar_capex: 1000.0,      // $/kW
wind_capex: 1200.0,       // $/kW
storage_capex: 300.0,     // $/kWh
clean_firm_capex: 5000.0, // $/kW
```

Clean firm is approximately 4-5x more expensive per kW than renewables, but provides guaranteed output.

### 2.3 The Constraint

The optimizer must satisfy:

```
achieved_clean_match = target ± tolerance
```

Where `tolerance` is typically 0.5%. This is an **exact** constraint, not a minimum constraint.

---

## 3. Issue Discovery: The Overbuild Blind Spot

### 3.1 The Scenario

For a **99% clean match target**, V2 produces:

```
V2 Solution:
  Wind: 325 MW, Clean Firm: 7.4 MW
  Clean Match: 99.0%
  LCOE: $69.64/MWh
```

However, exhaustive search reveals:

```
Global Optimum:
  Wind: 355 MW, Clean Firm: 0 MW
  Clean Match: 99.63%
  LCOE: $68.01/MWh
```

**The V2 solution costs $1.63/MWh more than the global optimum.**

### 3.2 Verification

The wind-only solution space investigation shows the progression:

```
Wind 320: 93.50% match, LCOE $65.41
Wind 325: 94.38% match, LCOE $65.78
Wind 330: 95.25% match, LCOE $66.15
Wind 335: 96.13% match, LCOE $66.52
Wind 340: 97.00% match, LCOE $66.89
Wind 345: 97.87% match, LCOE $67.27
Wind 350: 98.75% match, LCOE $67.64
Wind 355: 99.63% match, LCOE $68.01  // <- Pure wind global optimum
```

The pure wind solution at 355 MW beats V2's mixed solution by $1.63/MWh.

---

## 4. Root Cause Analysis

### 4.1 How V2 Works

The V2 optimizer uses a **greedy-based approach**:

1. **Greedy Phase**: Incrementally add renewable capacity, selecting the most cost-effective resource at each step
2. **Early Stopping**: Stop when clean match reaches a threshold below target
3. **CF Binary Search**: Use clean firm to precisely hit the target
4. **Local Refinement**: Search nearby portfolio variations

The critical logic in `run_greedy_phase()`:

```rust
// Stop BEFORE overshooting target (leave room for CF to fine-tune)
let stop_margin = if config.enable_clean_firm {
    // Stop 3-8% below target so CF can fill the gap
    (target * 0.05).max(3.0).min(8.0)
} else {
    1.0
};
let stop_threshold = target - stop_margin;

for _ in 0..100 {
    if current.clean_match >= stop_threshold {
        break;  // STOPS HERE
    }
    // ... greedy expansion
}
```

### 4.2 Why It Misses the Optimum

The greedy phase has an **intentional blind spot**:

1. For a 99% target, it stops adding renewables around 91-96%
2. It relies on CF binary search to fill the remaining gap
3. It **never explores** continuing to add renewables past the target

This design makes sense when CF is cheaper than the marginal renewable. But at high targets where renewables are still economical, the "stop early + CF fill" strategy is suboptimal.

### 4.3 The Overshoot Constraint

The overshoot prevention logic rejects moves that exceed the target:

```rust
if match_gain > 0.5 && result.clean_match <= overshoot_limit {
    // Accept move
}
```

This prevents the optimizer from discovering that **overshooting with renewables can be cheaper** than precise targeting with CF.

---

## 5. Key Insights

### 5.1 Two Competing Strategies Exist

For any high clean match target, there are two viable approaches:

| Strategy | Description | When Optimal |
|----------|-------------|--------------|
| **Stop Early + CF Fill** | Build renewables to ~92-95%, use CF to hit exact target | When CF is cheap relative to marginal renewable |
| **Renewable Overbuild** | Build renewables past the target, accept slight overshoot | When marginal renewable cost < CF LCOE contribution |

### 5.2 The Marginal Cost Crossover

The optimal strategy depends on **relative marginal costs**:

- **Marginal renewable cost**: Increases with penetration due to curtailment
- **CF marginal cost**: Constant (linear contribution)

At some penetration level, these curves cross. Below the crossover, add renewables. Above it, use CF.

For the 99% target case:
- Wind LCOE at 355 MW (99.63% match): $68.01/MWh
- V2's solution (325 MW wind + 7.4 MW CF): $69.64/MWh

The marginal cost of the additional 30 MW of wind is **less** than the cost of 7.4 MW of CF.

### 5.3 Clean Firm Has Unique Properties

Clean firm differs from renewables:
- **Linear contribution**: No saturation or curtailment
- **Precise control**: Can hit exact targets
- **Constant marginal cost**: Each MW costs the same

This makes CF ideal for "closing the gap" in the greedy strategy, but it's not always the cheapest option.

### 5.4 Overshooting Can Be Optimal

The constraint is `achieved >= target - tolerance`, not `achieved == target`. A solution that achieves 99.63% for a 99% target is **valid and may be cheaper** than one that achieves exactly 99%.

---

## 6. Solution Design

### 6.1 Dual-Strategy Approach

Modify `run_greedy_based_optimization()` to explore both strategies:

```
Algorithm: Dual-Strategy Optimization

1. Run current strategy (STOP_EARLY_CF_FILL):
   a. Greedy expansion until ~92-96% of target
   b. Binary search CF to hit exact target
   c. Local refinement
   -> Result: portfolio_cf

2. Run overbuild strategy (RENEWABLE_OVERBUILD):
   a. Greedy expansion with NO early stopping
   b. Continue adding renewables past target
   c. Stop only when marginal cost exceeds threshold
   -> Result: portfolio_overbuild

3. Compare and return cheaper:
   if portfolio_overbuild.lcoe < portfolio_cf.lcoe
       AND portfolio_overbuild.clean_match >= target:
       return portfolio_overbuild
   else:
       return portfolio_cf
```

### 6.2 Pseudo-Implementation

```rust
fn run_greedy_based_optimization(...) -> Result<EvalResult, String> {
    // Strategy 1: Current approach (stop early + CF)
    let result_cf = run_stop_early_cf_strategy(
        target, profiles, costs, config, battery_mode, cache,
    )?;

    // Strategy 2: Overbuild approach (no early stopping)
    let result_overbuild = run_overbuild_strategy(
        target, profiles, costs, config, battery_mode, cache,
    )?;

    // Select winner
    if result_overbuild.clean_match >= target - 0.5
        && result_overbuild.lcoe < result_cf.lcoe {
        Ok(result_overbuild)
    } else {
        Ok(result_cf)
    }
}

fn run_overbuild_strategy(...) -> Result<EvalResult, String> {
    // Modified greedy phase with NO stop_threshold
    // Continue adding capacity as long as marginal cost is reasonable

    loop {
        let best_move = find_best_greedy_move(...);

        if best_move.is_none() || past_diminishing_returns() {
            break;
        }

        apply_move(best_move);
    }

    // No CF binary search - pure renewable solution
    Ok(EvalResult { solar, wind, storage, clean_firm: 0.0, ... })
}
```

### 6.3 Cost Guard for Overbuild

To prevent excessive overbuilding, add a cost guard:

```rust
// Stop overbuild when marginal LCOE increase exceeds threshold
let marginal_lcoe_threshold = 2.0; // $/MWh per step

if result.lcoe - prev_lcoe > marginal_lcoe_threshold {
    break; // Diminishing returns
}
```

---

## 7. Validation Methodology

### 7.1 Exhaustive Grid Search

The primary validation tool is exhaustive grid search:

```rust
// Grid: Solar 0-200 by 25, Wind 0-500 by 25, Storage 0-400 by 50, CF 0-100 by 5
for solar in (0..=200).step_by(25) {
    for wind in (0..=500).step_by(25) {
        for storage in (0..=400).step_by(50) {
            for cf in (0..=100).step_by(5) {
                // Evaluate and track best that hits target
            }
        }
    }
}
```

This evaluates ~35,000 portfolios and finds the true global minimum.

### 7.2 Regression Tests

- **Target compliance**: Verify achieved matches target within tolerance
- **V1 comparison**: V2 should match or beat V1 on LCOE
- **Determinism**: Same result on repeated runs
- **Mode coverage**: All battery modes (Default, PeakShaver, Hybrid)
- **Cost scenarios**: Free resources, expensive gas, etc.

### 7.3 High-Target Specific Tests

```rust
#[test]
fn test_high_target_global_optimum() {
    let targets = [95.0, 97.0, 99.0];

    for target in targets {
        let v2_result = run_v2_optimizer(target, ...);
        let exhaustive_best = run_exhaustive_search(target, ...);

        // V2 should be within 1% of global optimum
        let gap = (v2_result.lcoe - exhaustive_best.lcoe) / exhaustive_best.lcoe;
        assert!(gap < 0.01, "V2 suboptimal at {}%: gap={:.2}%", target, gap * 100.0);
    }
}
```

---

## 8. Implementation Guidance

### 8.1 Files to Modify

| File | Changes |
|------|---------|
| `src/optimizer/v2_hierarchical.rs` | Add dual-strategy logic to `run_greedy_based_optimization()` |
| `tests/optimizer_v2_validation.rs` | Add high-target global optimum tests |

### 8.2 Performance Considerations

The dual-strategy approach roughly doubles the number of evaluations:
- Current: ~100-200 evaluations
- With overbuild: ~200-400 evaluations

This should still achieve <50ms execution time given ~0.1ms per evaluation.

### 8.3 Edge Cases

1. **CF disabled**: Only run overbuild strategy
2. **Low targets (<70%)**: Overbuild unlikely to help; CF strategy should dominate
3. **Expensive renewables**: CF fill will win; overbuild strategy will exit early
4. **Resource constraints**: Respect max_solar, max_wind, max_storage limits

---

## 9. Future Considerations

### 9.1 Merit Order Pre-Computation

Instead of runtime dual-strategy, pre-compute the crossover point where CF becomes cheaper than marginal renewables. This could reduce the search space significantly.

### 9.2 Analytical Solution

For well-characterized cost curves, the optimal portfolio may be solvable analytically:

1. Compute renewable saturation curve (clean match vs LCOE)
2. Find CF crossover point
3. If target < crossover: pure renewable overbuild
4. If target > crossover: renewable to crossover + CF fill

### 9.3 Empirical Model Enhancement

Train the empirical model to predict not just clean_match but also LCOE. This would enable faster global search by ranking candidates by predicted LCOE before simulation.

### 9.4 GPU Parallelization

The exhaustive search is embarrassingly parallel. GPU evaluation of 10,000+ portfolios simultaneously could enable true global optimization in <10ms.

---

## Appendix A: Code References

| Concept | File | Line |
|---------|------|------|
| V2 optimizer entry | `src/optimizer/v2_hierarchical.rs` | 207 |
| Greedy-based optimization | `src/optimizer/v2_hierarchical.rs` | 587 |
| Greedy phase | `src/optimizer/v2_hierarchical.rs` | 658 |
| Stop threshold logic | `src/optimizer/v2_hierarchical.rs` | 690-699 |
| Binary search CF | `src/optimizer/v2_hierarchical.rs` | 119 |
| Exhaustive validator | `src/bin/validate_global.rs` | - |
| Cost parameters | `src/types.rs` | 248-329 |

## Appendix B: Test Commands

```bash
# Run exhaustive validation at 99% target
cargo run --release --bin validate_global 99

# Compare V1 vs V2 at high targets
cargo run --release --bin test_high_targets

# Find wind-only solution space
cargo run --release --bin find_wind

# Full validation test suite
cargo test --release --test optimizer_v2_validation -- --include-ignored
```

---

## Summary

The V2 optimizer's greedy approach has a blind spot at high targets where pure renewable overbuild can beat the "stop early + CF fill" strategy. The fix is to explore both strategies and return the cheaper one. This maintains the speed advantage (~20-40ms) while finding the true global minimum.

---

## 10. EmpiricalModel Integration (Lookup Table Acceleration)

### 10.1 Overview

The EmpiricalModel is a pre-computed 4D lookup table that predicts clean_match % for any portfolio configuration without running full simulations. This enables fast candidate filtering before expensive simulation runs.

**Location:** `src/optimizer/empirical_model.rs`

### 10.2 Model Structure

The lookup table is a 4D grid with V1-style bounds (memory-optimized):
- Solar: 0-1000 MW (100 MW steps) → 11 points
- Wind: 0-500 MW (100 MW steps) → 6 points
- Storage: 0-2400 MWh (100 MWh steps) → 25 points
- Clean Firm: 0-125 MW (25 MW steps) → 6 points

**Total:** 11 × 6 × 25 × 6 = 9,900 grid points per model
**Size:** ~79 KB per model (bincode serialized)
**Battery mode:** Hybrid only (most realistic)
**Total models:** 13 zones × 1 mode = 13 models (~1.0 MB total)

### 10.3 Model Generation

Models are generated by running full simulations at each grid point:

```bash
# Generate models for all zones (Hybrid mode only)
cd rust
./generate_models.sh --all

# Or generate for specific zones
cargo run --release --features native --bin generate_training_data -- \
  --zones california,texas \
  --modes hybrid \
  --grid v1 \
  --data ../data/zones.json
```

Output: `models/<zone>_hybrid.bin` files (13 total, ~1 MB combined)

**Generation time:** ~1-2 seconds per model with V1 optimized grid (parallelized with rayon)

### 10.4 How the Model is Used

The `filter_candidates()` method pre-filters portfolios:

```rust
// Find portfolios where predicted match is within range of target
let candidates = model.filter_candidates(
    target,           // e.g., 80.0
    tolerance,        // e.g., 5.0 (search ±5% around target)
    max_solar,
    max_wind,
    max_storage,
    max_cf,
    enable_solar,
    enable_wind,
    enable_storage,
    enable_cf,
);
// Returns ~100-500 candidates instead of 9,900 grid points
```

For portfolios between grid points, 4D trilinear interpolation provides smooth predictions.

### 10.5 Interpolation Accuracy

Validation testing (200 random off-grid samples) shows excellent accuracy:

| Metric | Value |
|--------|-------|
| Mean absolute error | 0.21% |
| 95th percentile error | 1.59% |
| Max absolute error | 2.74% |
| Grid point error | 0.00% |

The model is highly accurate for candidate filtering purposes.

### 10.6 WASM Integration

**Model Cache (thread-local):**
```rust
thread_local! {
    static MODEL_CACHE: RefCell<HashMap<ModelKey, EmpiricalModel>> = RefCell::new(HashMap::new());
}
```

**WASM exports:**
- `wasm_load_model(zone, battery_mode, bytes)` - Load model from binary
- `wasm_is_model_loaded(zone, battery_mode)` - Check if model cached
- `wasm_clear_models()` - Free memory
- `wasm_loaded_models()` - Get list of cached models
- `wasm_cache_stats()` - Get cache statistics

**Frontend loading pattern:**
1. Fetch model binary when zone/mode changes
2. Pass bytes to WASM `wasm_load_model()`
3. Use `optimize_with_model()` for model-based optimization
4. Fallback to greedy search if model unavailable

### 10.7 Performance Impact

| Scenario | Without Model | With Model | Improvement |
|----------|---------------|------------|-------------|
| Single target | ~20-50ms | ~5-15ms | 2-4x faster |
| 11-point sweep | ~200-500ms | ~60-150ms | 3x faster |
| Initial load | - | +50-100ms | One-time cost |

### 10.8 Model Versioning

Models depend on:
- Zone profiles (solar/wind/load data)
- Grid configuration (step sizes, max values)
- Simulation logic (battery dispatch algorithm)

**Invalidation strategy:** Regenerate models when simulation logic changes significantly.

### 10.9 Fallback Behavior

If model loading fails, the optimizer falls back to greedy-based search:

```rust
let model = get_model(&zone, battery_mode);

let result = run_v2_optimizer(
    target,
    profiles,
    costs,
    config,
    battery_mode,
    model.as_ref(),  // None = greedy fallback
);
```

This ensures the optimizer always works, even without pre-computed models.

### 10.10 Files Reference

| File | Description |
|------|-------------|
| `src/optimizer/empirical_model.rs` | EmpiricalModel struct, GridConfig, interpolation |
| `src/optimizer/model_cache.rs` | Thread-local cache, LRU eviction |
| `src/bin/generate_training_data.rs` | Model generation binary |
| `generate_models.sh` | Shell script to generate all models |
| `tests/model_validation.rs` | Interpolation accuracy tests |
| `web/src/lib/modelLoader.ts` | Frontend model loading utilities |
