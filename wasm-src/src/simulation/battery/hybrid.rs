/// Hybrid Battery Dispatch Mode
///
/// # Objective
///
/// Hybrid mode combines peak shaving with opportunistic cycling:
///
/// 1. **Primary constraint**: Achieve the same peak gas reduction as Peak Shaver mode
/// 2. **Secondary goal**: Maximize battery cycling during "idle" periods
///
/// # The Problem with Pure Peak Shaving
///
/// In Peak Shaver mode, the battery often sits full, waiting for peak events.
/// During this idle time, there may be opportunities to:
/// - Discharge the battery (replacing gas generation)
/// - Recharge from upcoming renewable excess
/// - Still be ready for the next peak shaving need
///
/// # Algorithm
///
/// Pass 1: Run peak shaver to establish baseline
/// Pass 2: For each period where battery is "idle but full":
///   - Look ahead for renewable excess period before next peak-shaving need
///   - If found, discharge now and plan to recharge later
///   - Constraint: battery must be ready when needed for peak shaving
///
/// # Verification Criteria
///
/// A correct hybrid implementation should have:
/// - Peak gas ≈ Peak Shaver's peak gas (within ~1%)
/// - Total battery cycling >= Peak Shaver's cycling
/// - Clean match % >= Peak Shaver's clean match %
use super::peak_shaver::apply_peak_shaver_dispatch;

/// Apply hybrid battery dispatch
///
/// Returns: (battery_charge, battery_discharge, soc, curtailed, gas_for_charging)
pub fn apply_hybrid_dispatch(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    storage_capacity: f64,
    battery_eff: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = renewable_gen.len();

    if storage_capacity <= 0.0 || n == 0 {
        // No storage - return zeros
        let zeros = vec![0.0; n];
        let mut curtailed = vec![0.0; n];

        for i in 0..n {
            let total_clean = renewable_gen[i] + clean_firm_gen;
            curtailed[i] = (total_clean - load[i]).max(0.0);
        }

        return (zeros.clone(), zeros.clone(), zeros.clone(), curtailed, zeros);
    }

    // Pass 1: Peak shaver establishes baseline
    let (mut battery_charge, mut battery_discharge, mut soc, mut curtailed, gas_for_charging) =
        apply_peak_shaver_dispatch(
            renewable_gen,
            load,
            clean_firm_gen,
            storage_capacity,
            battery_eff,
        );

    // Calculate gas generation after pass 1
    let mut gas_generation = vec![0.0; n];
    for i in 0..n {
        let total_clean = renewable_gen[i] + clean_firm_gen;
        let battery_net = battery_discharge[i] * battery_eff - battery_charge[i];
        gas_generation[i] = (load[i] - total_clean - battery_net).max(0.0);
    }

    // Pass 2: Opportunistic dispatch
    // Find periods where battery is idle (full or nearly full, not discharging)
    // and look for opportunities to cycle

    hybrid_opportunistic_pass(
        renewable_gen,
        load,
        clean_firm_gen,
        &mut battery_charge,
        &mut battery_discharge,
        &mut soc,
        &mut curtailed,
        &mut gas_generation,
        storage_capacity,
        battery_eff,
    );

    (battery_charge, battery_discharge, soc, curtailed, gas_for_charging)
}

/// Pass 2: Look for opportunistic discharge/recharge cycles
fn hybrid_opportunistic_pass(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    battery_charge: &mut [f64],
    battery_discharge: &mut [f64],
    soc: &mut [f64],
    curtailed: &mut [f64],
    gas_generation: &mut [f64],
    storage_capacity: f64,
    battery_eff: f64,
) {
    let n = renewable_gen.len();
    if n == 0 {
        return;
    }

    // Recalculate SOC to ensure consistency
    let mut current_soc = storage_capacity;
    for i in 0..n {
        current_soc = (current_soc + battery_charge[i] - battery_discharge[i])
            .max(0.0)
            .min(storage_capacity);
        soc[i] = current_soc;
    }

    // Find transitions from renewable/neutral to gas (potential discharge opportunities)
    let mut hour = 1;
    while hour < n {
        // Check for transition: was renewable/neutral, now using gas
        let prev_has_gas = gas_generation[hour - 1] > 0.001;
        let curr_has_gas = gas_generation[hour] > 0.001;

        // Look for start of gas block (transition into gas usage)
        if !prev_has_gas && curr_has_gas {
            // Find the gas block extent
            let gas_start = hour;
            let mut gas_end = hour;
            while gas_end < n && gas_generation[gas_end] > 0.001 {
                gas_end += 1;
            }
            gas_end -= 1; // inclusive

            // Find renewable block after gas (where we can recharge)
            let mut ren_start = gas_end + 1;
            while ren_start < n && curtailed[ren_start] <= 0.001 {
                ren_start += 1;
            }

            if ren_start < n {
                let mut ren_end = ren_start;
                while ren_end < n && curtailed[ren_end] > 0.001 {
                    ren_end += 1;
                }

                // Check if this is a valid opportunity
                // SOC at start of gas block
                let soc_at_gas_start = if gas_start > 0 {
                    soc[gas_start - 1]
                } else {
                    storage_capacity
                };

                // Only proceed if we have available SOC to discharge
                if soc_at_gas_start > 0.001 {
                    apply_opportunistic_cycle(
                        gas_start,
                        gas_end,
                        ren_start,
                        ren_end,
                        soc_at_gas_start,
                        battery_charge,
                        battery_discharge,
                        soc,
                        curtailed,
                        gas_generation,
                        storage_capacity,
                        battery_eff,
                    );
                }

                hour = ren_end;
            } else {
                hour = gas_end + 1;
            }
        } else {
            hour += 1;
        }
    }
}

/// Apply one opportunistic discharge/recharge cycle
#[inline]
fn apply_opportunistic_cycle(
    gas_start: usize,
    gas_end: usize,
    ren_start: usize,
    ren_end: usize,
    soc_at_start: f64,
    battery_charge: &mut [f64],
    battery_discharge: &mut [f64],
    soc: &mut [f64],
    curtailed: &mut [f64],
    gas_generation: &mut [f64],
    storage_capacity: f64,
    battery_eff: f64,
) {
    // Calculate how much we could discharge in gas block.
    // Skip hours that are already charging to avoid simultaneous charge/discharge.
    let mut total_dischargeable = 0.0;
    for hour in gas_start..=gas_end {
        if battery_charge[hour] > 0.001 {
            continue;
        }
        total_dischargeable += gas_generation[hour]; // Gas we could replace
    }

    if total_dischargeable <= 0.001 {
        return;
    }

    // Account for existing discharge activity
    let mut existing_discharge = 0.0;
    for hour in gas_start..=gas_end {
        existing_discharge += battery_discharge[hour];
    }

    let available_soc = (soc_at_start - existing_discharge).max(0.0);
    if available_soc <= 0.001 {
        return;
    }

    // Max energy we can deliver = available_soc * efficiency
    let max_delivery = available_soc * battery_eff;
    let discharge_energy = total_dischargeable.min(max_delivery);

    // Calculate how much charge we need to restore
    let charge_needed = discharge_energy / battery_eff;

    // Calculate available charge potential in renewable block
    let mut charge_potential = 0.0;
    let soc_after_discharge = available_soc - (discharge_energy / battery_eff);
    let mut sim_soc = soc_after_discharge;

    for hour in ren_start..ren_end {
        sim_soc += battery_charge[hour]; // existing charge
        let room = storage_capacity - sim_soc;
        charge_potential += curtailed[hour].min(room).max(0.0);
        sim_soc = sim_soc.min(storage_capacity);
    }

    if charge_potential <= 0.001 {
        return;
    }

    // Scale based on limiting factor
    let actual_energy = if charge_potential >= charge_needed {
        discharge_energy // Can fully restore
    } else {
        charge_potential * battery_eff // Limited by charge potential
    };

    if actual_energy <= 0.001 {
        return;
    }

    let scale = actual_energy / total_dischargeable;

    // Apply discharge in gas block
    let mut running_soc = soc_at_start;
    for hour in gas_start..=gas_end {
        running_soc -= battery_discharge[hour]; // existing

        if battery_charge[hour] <= 0.001 {
            let add_discharge = (gas_generation[hour] / battery_eff * scale)
                .min(running_soc)
                .max(0.0);
            if add_discharge > 0.001 {
                battery_discharge[hour] += add_discharge;
                gas_generation[hour] =
                    (gas_generation[hour] - add_discharge * battery_eff).max(0.0);
                running_soc -= add_discharge;
            }
        }
        running_soc = running_soc.max(0.0);
    }

    // Apply charge in renewable block
    let actual_discharged = soc_at_start - running_soc - existing_discharge;
    let mut charge_remaining = actual_discharged;

    for hour in ren_start..ren_end {
        if charge_remaining <= 0.001 {
            break;
        }

        running_soc += battery_charge[hour]; // existing
        let room = storage_capacity - running_soc;
        let add_charge = curtailed[hour].min(room).min(charge_remaining).max(0.0);

        if add_charge > 0.001 {
            battery_charge[hour] += add_charge;
            curtailed[hour] -= add_charge;
            running_soc += add_charge;
            charge_remaining -= add_charge;
        }
        running_soc = running_soc.min(storage_capacity);
    }

    // Update SOC for affected range
    let start_update = if gas_start > 0 { gas_start - 1 } else { 0 };
    let mut current_soc = if start_update > 0 {
        soc[start_update - 1]
    } else {
        storage_capacity
    };

    for hour in start_update..ren_end.min(soc.len()) {
        current_soc = (current_soc + battery_charge[hour] - battery_discharge[hour])
            .max(0.0)
            .min(storage_capacity);
        soc[hour] = current_soc;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HOURS_PER_YEAR;

    #[test]
    fn test_hybrid_no_storage() {
        let renewable = vec![100.0; HOURS_PER_YEAR];
        let load = vec![80.0; HOURS_PER_YEAR];

        let (charge, discharge, soc, curtailed, gas_charge) =
            apply_hybrid_dispatch(&renewable, &load, 0.0, 0.0, 0.85);

        assert!(charge.iter().all(|&x| x == 0.0));
        assert!(discharge.iter().all(|&x| x == 0.0));
        assert!(soc.iter().all(|&x| x == 0.0));
        assert!(gas_charge.iter().all(|&x| x == 0.0));
        assert!(curtailed.iter().all(|&x| (x - 20.0).abs() < 0.01));
    }

    #[test]
    fn test_hybrid_increases_cycling() {
        // Create a profile with daily solar pattern
        let mut renewable = vec![0.0; HOURS_PER_YEAR];
        let load = vec![100.0; HOURS_PER_YEAR];

        // Solar: 150 MW during hours 10-14
        for day in 0..365 {
            for hour in 10..14 {
                renewable[day * 24 + hour] = 150.0;
            }
        }

        // Run both modes
        let (ps_charge, ps_discharge, _, _, _) =
            apply_peak_shaver_dispatch(&renewable, &load, 0.0, 100.0, 0.85);

        let (hy_charge, hy_discharge, _, _, _) =
            apply_hybrid_dispatch(&renewable, &load, 0.0, 100.0, 0.85);

        let ps_total_charge: f64 = ps_charge.iter().sum();
        let ps_total_discharge: f64 = ps_discharge.iter().sum();
        let hy_total_charge: f64 = hy_charge.iter().sum();
        let hy_total_discharge: f64 = hy_discharge.iter().sum();

        // Hybrid should have >= cycling compared to peak shaver
        assert!(
            hy_total_charge >= ps_total_charge * 0.99,
            "Hybrid charge {} should be >= Peak Shaver charge {}",
            hy_total_charge,
            ps_total_charge
        );
        assert!(
            hy_total_discharge >= ps_total_discharge * 0.99,
            "Hybrid discharge {} should be >= Peak Shaver discharge {}",
            hy_total_discharge,
            ps_total_discharge
        );
    }
}
