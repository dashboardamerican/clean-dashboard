/// Peak Shaver battery dispatch mode
///
/// Binary search finds optimal constant peak shaving line (30 iterations).
/// Tests if a given line is achievable with two-pass charging:
/// (1) renewable excess, (2) gas if needed.

/// Test if a given peak line is achievable with the battery
///
/// Simulates a full year to check if the battery can maintain
/// gas generation at or below the test line.
///
/// # Arguments
/// * `test_line` - Target peak gas line MW
/// * `gas_baseline` - Gas generation without battery MW (8760 hours)
/// * `renewable_excess` - Excess renewable generation MW (8760 hours)
/// * `storage_capacity` - Storage capacity MWh
/// * `battery_eff` - Round-trip efficiency (0-1)
///
/// # Returns
/// * `true` if line is achievable, `false` otherwise
pub fn test_battery_line(
    test_line: f64,
    gas_baseline: &[f64],
    renewable_excess: &[f64],
    storage_capacity: f64,
    battery_eff: f64,
) -> bool {
    let n = gas_baseline.len();
    // Battery starts at 100% full (matches Python behavior)
    let mut soc = storage_capacity;

    for i in 0..n {
        if gas_baseline[i] < test_line {
            // Below target - can charge
            let charge_headroom = test_line - gas_baseline[i];
            let available_charge = charge_headroom + renewable_excess[i];
            let available_capacity = storage_capacity - soc;
            let charge_amount = available_charge.min(available_capacity);
            soc += charge_amount;
        } else if gas_baseline[i] > test_line {
            // Above target - must discharge
            let needed_reduction = gas_baseline[i] - test_line;
            let discharge_needed = needed_reduction / battery_eff;

            if discharge_needed > soc {
                // Can't meet the target
                return false;
            }
            soc -= discharge_needed;
        }
        // If exactly at target, no action needed
    }

    true
}

/// Find optimal peak shaving line using binary search
///
/// # Arguments
/// * `gas_baseline` - Gas generation without battery MW (8760 hours)
/// * `renewable_excess` - Excess renewable generation MW (8760 hours)
/// * `storage_capacity` - Storage capacity MWh
/// * `battery_eff` - Round-trip efficiency (0-1)
///
/// # Returns
/// * Optimal peak line MW
pub fn find_optimal_peak_line(
    gas_baseline: &[f64],
    renewable_excess: &[f64],
    storage_capacity: f64,
    battery_eff: f64,
) -> f64 {
    // Find the range for binary search
    let max_gas = gas_baseline.iter().cloned().fold(0.0, f64::max);
    let min_gas = gas_baseline.iter().cloned().fold(f64::INFINITY, f64::min);

    if storage_capacity <= 0.0 {
        return max_gas;
    }

    let mut low = min_gas;
    let mut high = max_gas;
    let max_iterations = 30;
    let tolerance = 0.01; // MW tolerance

    for _ in 0..max_iterations {
        if high - low < tolerance {
            break;
        }

        let mid = (low + high) / 2.0;

        if test_battery_line(
            mid,
            gas_baseline,
            renewable_excess,
            storage_capacity,
            battery_eff,
        ) {
            // Can achieve this line, try lower
            high = mid;
        } else {
            // Can't achieve, need higher line
            low = mid;
        }
    }

    (low + high) / 2.0
}

/// Apply peak shaver battery dispatch
///
/// # Arguments
/// * `renewable_gen` - Total renewable generation MW (8760 hours)
/// * `load` - Load MW (8760 hours)
/// * `clean_firm_gen` - Clean firm generation MW
/// * `storage_capacity` - Storage capacity MWh
/// * `battery_eff` - Round-trip efficiency (0-1)
///
/// # Returns
/// * (battery_charge, battery_discharge, soc, curtailed, gas_for_charging) arrays
pub fn apply_peak_shaver_dispatch(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    storage_capacity: f64,
    battery_eff: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = renewable_gen.len();
    let mut battery_charge = vec![0.0; n];
    let mut battery_discharge = vec![0.0; n];
    let mut soc = vec![0.0; n];
    let mut curtailed = vec![0.0; n];
    let mut gas_for_charging = vec![0.0; n];

    if storage_capacity <= 0.0 {
        // No storage - just calculate curtailment
        for i in 0..n {
            let total_gen = renewable_gen[i] + clean_firm_gen;
            if total_gen > load[i] {
                curtailed[i] = total_gen - load[i];
            }
        }
        return (
            battery_charge,
            battery_discharge,
            soc,
            curtailed,
            gas_for_charging,
        );
    }

    // Calculate gas baseline (gas needed without battery)
    let mut gas_baseline = vec![0.0; n];
    let mut renewable_excess = vec![0.0; n];

    for i in 0..n {
        let total_clean = renewable_gen[i] + clean_firm_gen;
        if total_clean >= load[i] {
            renewable_excess[i] = total_clean - load[i];
        } else {
            gas_baseline[i] = load[i] - total_clean;
        }
    }

    // Find optimal peak line
    let peak_line = find_optimal_peak_line(
        &gas_baseline,
        &renewable_excess,
        storage_capacity,
        battery_eff,
    );

    // Apply the peak shaving dispatch
    // Battery starts at 100% full (matches Python behavior)
    let mut current_soc = storage_capacity;

    for i in 0..n {
        let total_clean = renewable_gen[i] + clean_firm_gen;

        if gas_baseline[i] < peak_line {
            // Below peak line - can charge
            // First use renewable excess
            let charge_from_renewable = renewable_excess[i].min(storage_capacity - current_soc);
            current_soc += charge_from_renewable;

            // Then potentially charge from gas (to prepare for later discharge)
            let remaining_capacity = storage_capacity - current_soc;
            let charge_headroom = peak_line - gas_baseline[i];
            let charge_from_gas = charge_headroom.min(remaining_capacity);
            current_soc += charge_from_gas;
            gas_for_charging[i] = charge_from_gas;

            battery_charge[i] = charge_from_renewable + charge_from_gas;
            curtailed[i] = renewable_excess[i] - charge_from_renewable;
        } else if gas_baseline[i] > peak_line {
            // Above peak line - must discharge
            let reduction_needed = gas_baseline[i] - peak_line;
            let discharge_needed = reduction_needed / battery_eff;
            let actual_discharge = discharge_needed.min(current_soc);
            battery_discharge[i] = actual_discharge;
            current_soc -= actual_discharge;
        } else {
            // Exactly at peak line - no battery action needed
            // But still capture any renewable excess if we have capacity
            if renewable_excess[i] > 0.0 {
                let charge_amount = renewable_excess[i].min(storage_capacity - current_soc);
                battery_charge[i] = charge_amount;
                current_soc += charge_amount;
                curtailed[i] = renewable_excess[i] - charge_amount;
            }
        }

        soc[i] = current_soc;
    }

    (
        battery_charge,
        battery_discharge,
        soc,
        curtailed,
        gas_for_charging,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HOURS_PER_YEAR;

    #[test]
    fn test_battery_line_trivial() {
        // If line is at or above max gas, should always be achievable
        let gas_baseline = vec![10.0, 20.0, 30.0, 20.0, 10.0];
        let renewable_excess = vec![0.0; 5];

        assert!(test_battery_line(
            30.0,
            &gas_baseline,
            &renewable_excess,
            0.0,
            0.85
        ));
        assert!(test_battery_line(
            40.0,
            &gas_baseline,
            &renewable_excess,
            10.0,
            0.85
        ));
    }

    #[test]
    fn test_battery_line_impossible() {
        // Line below min gas, no storage
        let gas_baseline = vec![10.0, 20.0, 30.0];
        let renewable_excess = vec![0.0; 3];

        assert!(!test_battery_line(
            5.0,
            &gas_baseline,
            &renewable_excess,
            0.0,
            0.85
        ));
    }

    #[test]
    fn test_find_optimal_line() {
        // Create a simple scenario
        let mut gas_baseline = vec![0.0; 24];
        let mut renewable_excess = vec![0.0; 24];

        // Low gas in morning, high in evening
        for i in 0..12 {
            gas_baseline[i] = 10.0;
            renewable_excess[i] = 20.0; // Can charge
        }
        for i in 12..24 {
            gas_baseline[i] = 50.0;
            renewable_excess[i] = 0.0; // Need discharge
        }

        let line = find_optimal_peak_line(&gas_baseline, &renewable_excess, 100.0, 0.85);

        // Line should be between min (10) and max (50)
        assert!(line > 10.0);
        assert!(line < 50.0);
    }

    #[test]
    fn test_peak_shaver_no_storage() {
        let renewable = vec![100.0; HOURS_PER_YEAR];
        let load = vec![80.0; HOURS_PER_YEAR];

        let (charge, discharge, soc, curtailed, gas_charge) =
            apply_peak_shaver_dispatch(&renewable, &load, 0.0, 0.0, 0.85);

        // No storage means no charging/discharging
        assert!(charge.iter().all(|&x| x == 0.0));
        assert!(discharge.iter().all(|&x| x == 0.0));
        assert!(soc.iter().all(|&x| x == 0.0));
        assert!(gas_charge.iter().all(|&x| x == 0.0));
        // Should curtail excess
        assert!(curtailed.iter().all(|&x| (x - 20.0).abs() < 0.01));
    }
}
