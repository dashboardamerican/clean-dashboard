# Battery Dispatch Implementation Notes

## Critical: Battery Initial State

**The battery must start at 100% full (storage_capacity), not 0%.**

### Python Implementation (simulation.py)
```python
soc = storage_capacity  # Start with a full battery
```
This appears at:
- Line 19: `_test_battery_line_numba`
- Line 193: `_simulate_system_numba_core` (for peak shaver)
- Line 306: Default mode simulation

### Rust Implementation
Fixed on 2025-01-25 to match Python:

**default.rs:**
```rust
// Battery starts at 100% full (matches Python behavior)
let mut current_soc = storage_capacity;
```

**peak_shaver.rs:**
```rust
// Battery starts at 100% full (matches Python behavior)
let mut soc = storage_capacity;  // in test_battery_line
let mut current_soc = storage_capacity;  // in apply_peak_shaver_dispatch
```

**hybrid.rs:**
All helper functions that get initial SOC now use `storage_capacity` instead of `0.0` for hour 0.

## Why This Matters

Starting the battery at 0% causes:
1. No discharge possible in early deficit hours
2. Battery only starts working after it accumulates charge from renewable excess
3. Incorrect clean match percentage (lower than expected)
4. Higher gas generation than necessary

Starting at 100% (full) is realistic for:
- Day-ahead scheduling where storage is pre-charged
- Simulation that represents optimal dispatch with full state knowledge
- Matching Python baseline for cross-validation

## Battery Dispatch Strategies

### Default Mode (Water-Fill)
- Battery charges only from renewable excess (generation > load)
- Discharge uses water-fill algorithm to shave highest peaks first
- Binary search finds optimal threshold (10 iterations, 0.1 MW tolerance)

### Peak Shaver Mode
- Binary search finds optimal peak line (30 iterations)
- Battery maintains gas generation at or below the line
- Two-pass charging: (1) renewable excess, (2) gas if needed
- `test_battery_line()` validates if a line is achievable

### Hybrid Mode
- Pass 1: Peak shaver algorithm
- Pass 2: Opportunistic dispatch at renewable→gas transitions
- Tracks battery energy source (renewable vs gas) for accurate clean match
- Only renewable-sourced discharge counts toward clean percentage

## Round-Trip Efficiency

Efficiency is applied on DISCHARGE, not charge:
```rust
// Charging: no loss
battery_soc += charge_amount;

// Discharging: apply efficiency
energy_delivered = discharge_amount * battery_eff;  // typically 0.85
```

## Cross-Validation

All 38 unit tests pass plus 5 cross-validation tests comparing Rust output to Python fixtures.
