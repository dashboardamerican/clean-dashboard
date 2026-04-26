/// Default battery dispatch mode using water-fill algorithm
///
/// Battery only charges from renewable excess (when generation > load).
/// During deficit, water-fill algorithm prioritizes shaving highest peaks.
/// Binary search finds optimal "water level" threshold.

/// Calculate water-fill allocation for battery discharge
///
/// Allocates battery energy to shave the highest peaks first.
/// Uses binary search to find optimal threshold.
///
/// # Arguments
/// * `deficits` - Array of energy deficits for each hour (MW)
/// * `total_to_allocate` - Total battery energy available to allocate (MWh)
///
/// # Returns
/// * Allocation array (MW for each hour)
pub fn calculate_waterfill_allocation(deficits: &[f64], total_to_allocate: f64) -> Vec<f64> {
    let n = deficits.len();
    let mut allocation = vec![0.0; n];

    if total_to_allocate <= 0.0 {
        return allocation;
    }

    // Sum of all deficits
    let total_deficit: f64 = deficits.iter().sum();

    // If we have enough to cover everything, just return all deficits
    if total_deficit <= total_to_allocate {
        return deficits.to_vec();
    }

    // Find max deficit for binary search upper bound
    let max_deficit = deficits.iter().cloned().fold(0.0, f64::max);

    // Binary search to find optimal threshold
    // We want: sum(max(deficit - threshold, 0)) = total_to_allocate
    let mut low = 0.0;
    let mut high = max_deficit;
    let tolerance = 0.1; // MW tolerance
    let max_iterations = 10;

    for _ in 0..max_iterations {
        let mid = (low + high) / 2.0;

        // Calculate how much we'd allocate at this threshold
        let mut allocated = 0.0;
        for &deficit in deficits {
            if deficit > mid {
                allocated += deficit - mid;
            }
        }

        if (allocated - total_to_allocate).abs() < tolerance {
            // Found good threshold
            low = mid;
            high = mid;
            break;
        } else if allocated > total_to_allocate {
            // Threshold too low, need higher threshold (less allocation)
            low = mid;
        } else {
            // Threshold too high, need lower threshold (more allocation)
            high = mid;
        }
    }

    // Use final threshold to compute allocation
    let threshold = (low + high) / 2.0;
    for i in 0..n {
        if deficits[i] > threshold {
            allocation[i] = deficits[i] - threshold;
        }
    }

    allocation
}

/// Apply default battery dispatch (water-fill algorithm)
///
/// This implements Python's single-pass, per-block waterfill algorithm:
/// 1. Single pass through all hours
/// 2. During surplus: charge battery from excess
/// 3. At START of each deficit block: calculate waterfill using CURRENT soc
/// 4. Apply discharge allocation for each deficit hour
///
/// # Arguments
/// * `renewable_gen` - Total renewable generation MW (8760 hours)
/// * `load` - Load MW (8760 hours)
/// * `clean_firm_gen` - Clean firm generation MW
/// * `storage_capacity` - Storage capacity MWh
/// * `battery_eff` - Round-trip efficiency (0-1)
///
/// # Returns
/// * (battery_charge, battery_discharge, soc, curtailed) arrays
pub fn apply_default_dispatch(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    storage_capacity: f64,
    battery_eff: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = renewable_gen.len();
    let mut battery_charge = vec![0.0; n];
    let mut battery_discharge = vec![0.0; n];
    let mut soc_array = vec![0.0; n];
    let mut curtailed = vec![0.0; n];

    if storage_capacity <= 0.0 {
        // No storage - just calculate curtailment
        for i in 0..n {
            let total_gen = renewable_gen[i] + clean_firm_gen;
            if total_gen > load[i] {
                curtailed[i] = total_gen - load[i];
            }
        }
        return (battery_charge, battery_discharge, soc_array, curtailed);
    }

    // Pre-calculate surplus/deficit masks and amounts
    let mut surplus_mask = vec![false; n];
    let mut surplus_amount = vec![0.0; n];
    let mut deficit_amount = vec![0.0; n];

    for i in 0..n {
        let total_gen = renewable_gen[i] + clean_firm_gen;
        let net = total_gen - load[i];
        if net >= 0.0 {
            surplus_mask[i] = true;
            surplus_amount[i] = net;
        } else {
            deficit_amount[i] = -net;
        }
    }

    // SINGLE PASS with per-block waterfill (matches Python algorithm)
    let mut current_soc = storage_capacity; // Battery starts full
    let mut current_allocation: Vec<f64> = Vec::new();
    let mut block_start: usize = 0;

    for i in 0..n {
        if surplus_mask[i] {
            // SURPLUS: Charge battery from renewable excess
            let charge_possible = storage_capacity - current_soc;
            let surplus = surplus_amount[i];

            let charge_amount = if surplus <= charge_possible {
                curtailed[i] = 0.0;
                surplus
            } else {
                curtailed[i] = surplus - charge_possible;
                charge_possible
            };

            battery_charge[i] = charge_amount;
            current_soc += charge_amount;
            current_soc = current_soc.min(storage_capacity);
        } else {
            // DEFICIT: Discharge battery

            // At START of new deficit block: calculate waterfill with CURRENT soc
            if i == 0 || surplus_mask[i - 1] {
                // Find end of this deficit block
                let mut block_end = n;
                for j in i..n {
                    if surplus_mask[j] {
                        block_end = j;
                        break;
                    }
                }
                block_start = i;

                // Calculate waterfill allocation for this block using CURRENT soc
                let block_deficits = &deficit_amount[block_start..block_end];
                let block_initial_battery = current_soc;
                let available_delivery = block_initial_battery * battery_eff;
                current_allocation =
                    calculate_waterfill_allocation(block_deficits, available_delivery);
            }

            // Apply allocation for this hour
            let index_in_block = i - block_start;
            let battery_delivery = if index_in_block < current_allocation.len() {
                current_allocation[index_in_block]
            } else {
                0.0
            };

            let discharge_amount = (battery_delivery / battery_eff).min(current_soc);
            battery_discharge[i] = discharge_amount;
            current_soc -= discharge_amount;
            current_soc = current_soc.max(0.0);
        }

        soc_array[i] = current_soc;
    }

    (battery_charge, battery_discharge, soc_array, curtailed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HOURS_PER_YEAR;

    #[test]
    fn test_waterfill_full_coverage() {
        let deficits = vec![10.0, 20.0, 30.0];
        let total = 100.0; // More than sum of deficits (60)

        let allocation = calculate_waterfill_allocation(&deficits, total);

        // Should return all deficits since we have enough
        assert_eq!(allocation, deficits);
    }

    #[test]
    fn test_waterfill_partial_coverage() {
        let deficits = vec![10.0, 20.0, 30.0, 40.0];
        let total = 30.0; // Less than sum (100)

        let allocation = calculate_waterfill_allocation(&deficits, total);

        // Sum should equal total_to_allocate (within tolerance)
        let sum: f64 = allocation.iter().sum();
        assert!((sum - total).abs() < 1.0);

        // Higher deficits should get more allocation
        assert!(allocation[3] >= allocation[2]);
        assert!(allocation[2] >= allocation[1]);
    }

    #[test]
    fn test_waterfill_zero_allocation() {
        let deficits = vec![10.0, 20.0, 30.0];
        let total = 0.0;

        let allocation = calculate_waterfill_allocation(&deficits, total);

        assert!(allocation.iter().all(|&x| x == 0.0));
    }

    #[test]
    fn test_default_dispatch_no_storage() {
        let renewable = vec![100.0; HOURS_PER_YEAR];
        let load = vec![80.0; HOURS_PER_YEAR];

        let (charge, discharge, soc, curtailed) =
            apply_default_dispatch(&renewable, &load, 0.0, 0.0, 0.85);

        // No storage means no charging/discharging
        assert!(charge.iter().all(|&x| x == 0.0));
        assert!(discharge.iter().all(|&x| x == 0.0));
        assert!(soc.iter().all(|&x| x == 0.0));
        // Should curtail excess
        assert!(curtailed.iter().all(|&x| (x - 20.0).abs() < 0.01));
    }

    #[test]
    fn test_default_dispatch_basic() {
        // Simple test: alternating excess and deficit
        let mut renewable = vec![0.0; 24];
        let mut load = vec![100.0; 24];

        // First 12 hours: excess
        for i in 0..12 {
            renewable[i] = 150.0;
        }
        // Last 12 hours: deficit
        for i in 12..24 {
            renewable[i] = 50.0;
        }

        // Pad to 8760
        renewable.resize(HOURS_PER_YEAR, 100.0);
        load.resize(HOURS_PER_YEAR, 100.0);

        let (charge, discharge, soc, _curtailed) =
            apply_default_dispatch(&renewable, &load, 0.0, 100.0, 0.85);

        // Should charge during excess hours
        for i in 0..12 {
            assert!(charge[i] >= 0.0, "Hour {}: charge should be >= 0", i);
        }

        // Should discharge during deficit hours (at least some)
        let total_discharge: f64 = discharge[12..24].iter().sum();
        assert!(total_discharge > 0.0, "Should have some discharge");
    }
}
