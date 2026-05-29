/// Limited-forecast battery dispatch mode.
///
/// This is the dashboard-grade analogue of a rolling-horizon dispatch model:
/// it sees a finite window, compares clean-only peak shaving with economically
/// justified gas/grid charging, commits the first block, then re-solves. It
/// intentionally avoids LP/MILP solvers so runtime stays in the same class as
/// the existing dispatch strategies.

const DEFAULT_HORIZON_HOURS: usize = 48;
const DEFAULT_COMMIT_HOURS: usize = 24;
const DEFAULT_TERMINAL_RESERVE_FRACTION: f64 = 0.0;
const DEFAULT_PEAK_VALUE_MWH_PER_MW: f64 = 24.0;
// Dashboard-tuned search budget: enough to avoid visible stepped peak lines
// while keeping the mode cheap enough for interactive sweeps.
const PEAK_LINE_ITERATIONS: usize = 12;
const PEAK_LINE_CANDIDATES: usize = 9;

fn limited_peak_line_feasible(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    start: usize,
    end: usize,
    initial_soc: f64,
    storage_capacity: f64,
    battery_eff: f64,
    soc_reserve: f64,
    peak_line: f64,
    gas_charging: bool,
) -> bool {
    let mut soc = initial_soc;

    for hour in start..end {
        let total_clean = renewable_gen[hour] + clean_firm_gen;
        let surplus = (total_clean - load[hour]).max(0.0);
        let deficit = (load[hour] - total_clean).max(0.0);

        if deficit > peak_line {
            let discharge_needed = (deficit - peak_line) / battery_eff;
            if discharge_needed > (soc - soc_reserve).max(0.0) + 1e-9 {
                return false;
            }
            soc = (soc - discharge_needed).max(soc_reserve);
        } else {
            let gas_charge_headroom = if gas_charging {
                peak_line - deficit
            } else {
                0.0
            };
            let charge = (surplus + gas_charge_headroom).min(storage_capacity - soc);
            soc = (soc + charge).min(storage_capacity);
        }
    }

    soc + 1e-9 >= soc_reserve
}

fn find_limited_peak_line(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    start: usize,
    end: usize,
    initial_soc: f64,
    storage_capacity: f64,
    battery_eff: f64,
    soc_reserve: f64,
    gas_charging: bool,
) -> f64 {
    let mut high = 0.0;
    for hour in start..end {
        let total_clean = renewable_gen[hour] + clean_firm_gen;
        high = f64::max(high, load[hour] - total_clean);
    }
    high = high.max(0.0);

    if high <= 0.0 || storage_capacity <= 0.0 {
        return high;
    }

    // If the terminal reserve is unreachable even with no discharge, avoid
    // forcing impossible reserve recovery and simply preserve SOC.
    if !limited_peak_line_feasible(
        renewable_gen,
        load,
        clean_firm_gen,
        start,
        end,
        initial_soc,
        storage_capacity,
        battery_eff,
        soc_reserve,
        high,
        gas_charging,
    ) {
        return high;
    }

    let mut low = 0.0;
    for _ in 0..PEAK_LINE_ITERATIONS {
        let mid = (low + high) / 2.0;
        if limited_peak_line_feasible(
            renewable_gen,
            load,
            clean_firm_gen,
            start,
            end,
            initial_soc,
            storage_capacity,
            battery_eff,
            soc_reserve,
            mid,
            gas_charging,
        ) {
            high = mid;
        } else {
            low = mid;
        }
    }

    // `high` is the feasible side of the binary search. Returning the midpoint
    // can land just below feasibility and make the best tight candidate look
    // infeasible in the later economic scoring pass.
    high
}

fn score_window_policy(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    start: usize,
    end: usize,
    initial_soc: f64,
    storage_capacity: f64,
    battery_eff: f64,
    soc_reserve: f64,
    peak_line: f64,
    gas_charging: bool,
) -> Option<(f64, f64)> {
    let mut soc = initial_soc;
    let mut gas_mwh = 0.0;
    let mut peak_gas: f64 = 0.0;

    for hour in start..end {
        let total_clean = renewable_gen[hour] + clean_firm_gen;
        let surplus = (total_clean - load[hour]).max(0.0);
        let deficit = (load[hour] - total_clean).max(0.0);
        let mut discharge = 0.0;
        let mut charge_from_gas = 0.0;

        if deficit > peak_line {
            let discharge_needed = (deficit - peak_line) / battery_eff;
            if discharge_needed > (soc - soc_reserve).max(0.0) + 1e-9 {
                return None;
            }
            discharge = discharge_needed.min((soc - soc_reserve).max(0.0));
            soc = (soc - discharge).max(soc_reserve);
        } else {
            let charge_from_clean = surplus.min(storage_capacity - soc);
            soc = (soc + charge_from_clean).min(storage_capacity);
            if gas_charging {
                charge_from_gas = (peak_line - deficit).min(storage_capacity - soc);
                soc = (soc + charge_from_gas).min(storage_capacity);
            }
        }

        let gas_for_load = (deficit - discharge * battery_eff).max(0.0);
        let total_gas = gas_for_load + charge_from_gas;
        gas_mwh += total_gas;
        peak_gas = peak_gas.max(total_gas);
    }

    Some((gas_mwh, peak_gas))
}

fn choose_limited_window_rule(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    start: usize,
    end: usize,
    initial_soc: f64,
    storage_capacity: f64,
    battery_eff: f64,
    soc_reserve: f64,
    peak_value_mwh_per_mw: f64,
    allow_gas_charging: bool,
) -> (f64, bool) {
    let clean_only_peak = find_limited_peak_line(
        renewable_gen,
        load,
        clean_firm_gen,
        start,
        end,
        initial_soc,
        storage_capacity,
        battery_eff,
        soc_reserve,
        false,
    );
    let mut best_line = clean_only_peak;
    let mut best_gas_charging = false;
    let mut best_obj = score_window_policy(
        renewable_gen,
        load,
        clean_firm_gen,
        start,
        end,
        initial_soc,
        storage_capacity,
        battery_eff,
        soc_reserve,
        clean_only_peak,
        false,
    )
    .map(|(gas_mwh, peak_gas)| gas_mwh + peak_value_mwh_per_mw * peak_gas)
    .unwrap_or(f64::INFINITY);

    if peak_value_mwh_per_mw <= 0.0 || !allow_gas_charging {
        return (best_line, best_gas_charging);
    }

    let gas_peak = find_limited_peak_line(
        renewable_gen,
        load,
        clean_firm_gen,
        start,
        end,
        initial_soc,
        storage_capacity,
        battery_eff,
        soc_reserve,
        true,
    );
    if gas_peak >= clean_only_peak - 1e-6 {
        return (best_line, best_gas_charging);
    }

    for candidate in 0..PEAK_LINE_CANDIDATES {
        let fraction = if PEAK_LINE_CANDIDATES <= 1 {
            0.0
        } else {
            candidate as f64 / (PEAK_LINE_CANDIDATES - 1) as f64
        };
        let line = gas_peak + (clean_only_peak - gas_peak) * fraction;
        if let Some((gas_mwh, peak_gas)) = score_window_policy(
            renewable_gen,
            load,
            clean_firm_gen,
            start,
            end,
            initial_soc,
            storage_capacity,
            battery_eff,
            soc_reserve,
            line,
            true,
        ) {
            let obj = gas_mwh + peak_value_mwh_per_mw * peak_gas;
            if obj < best_obj {
                best_obj = obj;
                best_line = line;
                best_gas_charging = true;
            }
        }
    }

    (best_line, best_gas_charging)
}

fn forecast_renewable_generation(renewable_gen: &[f64], renewable_error_pct: f64) -> Vec<f64> {
    let error_blend = (renewable_error_pct / 100.0).clamp(0.0, 1.0);
    if error_blend <= 1e-12 {
        return renewable_gen.to_vec();
    }

    renewable_gen
        .iter()
        .enumerate()
        .map(|(hour, actual)| {
            let persistence = if hour >= 24 {
                renewable_gen[hour - 24]
            } else {
                *actual
            };
            actual * (1.0 - error_blend) + persistence * error_blend
        })
        .collect()
}

fn apply_limited_forecast_dispatch_with_params(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    storage_capacity: f64,
    battery_eff: f64,
    horizon_hours: usize,
    commit_hours: usize,
    soc_reserve_fraction: f64,
    renewable_forecast_error_pct: f64,
    peak_value_mwh_per_mw: f64,
    allow_gas_charging: bool,
    initial_soc: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    let n = renewable_gen.len();
    let mut battery_charge = vec![0.0; n];
    let mut battery_discharge = vec![0.0; n];
    let mut soc_array = vec![0.0; n];
    let mut curtailed = vec![0.0; n];
    let mut gas_for_charging = vec![0.0; n];

    if storage_capacity <= 0.0 || n == 0 {
        for hour in 0..n {
            let total_clean = renewable_gen[hour] + clean_firm_gen;
            curtailed[hour] = (total_clean - load[hour]).max(0.0);
        }
        return (
            battery_charge,
            battery_discharge,
            soc_array,
            curtailed,
            gas_for_charging,
        );
    }

    let horizon_hours = horizon_hours.max(1);
    let commit_hours = commit_hours.max(1).min(horizon_hours);
    let soc_reserve = soc_reserve_fraction.clamp(0.0, 1.0) * storage_capacity;
    let forecast_renewable =
        forecast_renewable_generation(renewable_gen, renewable_forecast_error_pct);
    let mut current_soc = initial_soc.clamp(0.0, storage_capacity);
    let mut start = 0;

    while start < n {
        let end = (start + horizon_hours).min(n);
        let commit_end = (start + commit_hours).min(n);
        let (peak_line, gas_charging) = choose_limited_window_rule(
            &forecast_renewable,
            load,
            clean_firm_gen,
            start,
            end,
            current_soc,
            storage_capacity,
            battery_eff,
            soc_reserve,
            peak_value_mwh_per_mw,
            allow_gas_charging,
        );

        for hour in start..commit_end {
            let total_clean = renewable_gen[hour] + clean_firm_gen;
            let surplus = (total_clean - load[hour]).max(0.0);
            let deficit = (load[hour] - total_clean).max(0.0);

            if deficit > peak_line {
                let available_soc = (current_soc - soc_reserve).max(0.0);
                let discharge = ((deficit - peak_line) / battery_eff).min(available_soc);
                battery_discharge[hour] = discharge;
                current_soc = (current_soc - discharge).max(soc_reserve);
            } else {
                let charge_from_clean = surplus.min(storage_capacity - current_soc);
                current_soc = (current_soc + charge_from_clean).min(storage_capacity);

                let charge_from_gas = if gas_charging {
                    let gas_charge_headroom = peak_line - deficit;
                    let charge = gas_charge_headroom.min(storage_capacity - current_soc);
                    current_soc = (current_soc + charge).min(storage_capacity);
                    charge
                } else {
                    0.0
                };

                battery_charge[hour] = charge_from_clean + charge_from_gas;
                gas_for_charging[hour] = charge_from_gas;
                curtailed[hour] = surplus - charge_from_clean;
            }

            soc_array[hour] = current_soc;
        }

        start = commit_end;
    }

    (
        battery_charge,
        battery_discharge,
        soc_array,
        curtailed,
        gas_for_charging,
    )
}

/// Apply limited-forecast dispatch using fixed dashboard defaults.
///
/// Defaults:
/// - 48-hour forecast horizon
/// - 24-hour committed schedule before re-solving
/// - no SOC reserve holdback
/// - clean surplus charging first
/// - gas/grid charging only when its forecast peak value justifies the extra fuel
pub fn apply_limited_forecast_dispatch(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    storage_capacity: f64,
    battery_eff: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    apply_limited_forecast_dispatch_with_settings(
        renewable_gen,
        load,
        clean_firm_gen,
        storage_capacity,
        battery_eff,
        DEFAULT_HORIZON_HOURS,
        DEFAULT_COMMIT_HOURS,
        DEFAULT_TERMINAL_RESERVE_FRACTION,
        DEFAULT_PEAK_VALUE_MWH_PER_MW,
        0.0,
        true,
    )
}

pub fn apply_limited_forecast_dispatch_with_settings(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    storage_capacity: f64,
    battery_eff: f64,
    horizon_hours: usize,
    commit_hours: usize,
    soc_reserve_fraction: f64,
    peak_value_mwh_per_mw: f64,
    renewable_forecast_error_pct: f64,
    allow_gas_charging: bool,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    apply_limited_forecast_dispatch_with_params(
        renewable_gen,
        load,
        clean_firm_gen,
        storage_capacity,
        battery_eff,
        horizon_hours,
        commit_hours,
        soc_reserve_fraction,
        renewable_forecast_error_pct,
        peak_value_mwh_per_mw,
        allow_gas_charging,
        storage_capacity,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::HOURS_PER_YEAR;

    #[test]
    fn test_limited_forecast_no_storage() {
        let renewable = vec![120.0; 24];
        let load = vec![100.0; 24];

        let (charge, discharge, soc, curtailed, gas_for_charging) =
            apply_limited_forecast_dispatch(&renewable, &load, 0.0, 0.0, 0.85);

        assert!(charge.iter().all(|&x| x == 0.0));
        assert!(discharge.iter().all(|&x| x == 0.0));
        assert!(soc.iter().all(|&x| x == 0.0));
        assert!(gas_for_charging.iter().all(|&x| x == 0.0));
        assert!(curtailed.iter().all(|&x| (x - 20.0).abs() < 1e-9));
    }

    #[test]
    fn test_limited_forecast_cannot_see_distant_peak() {
        let renewable = vec![90.0, 90.0, 90.0, 90.0, 90.0, 0.0, 0.0, 0.0];
        let load = vec![100.0; 8];

        let (_, short_discharge, _, _, short_gas_for_charging) =
            apply_limited_forecast_dispatch_with_params(
                &renewable, &load, 0.0, 50.0, 1.0, 3, 1, 0.0, 0.0, 24.0, true, 50.0,
            );
        let (_, long_discharge, _, _, long_gas_for_charging) =
            apply_limited_forecast_dispatch_with_params(
                &renewable, &load, 0.0, 50.0, 1.0, 8, 1, 0.0, 0.0, 24.0, true, 50.0,
            );

        let short_early_discharge: f64 = short_discharge[..5].iter().sum();
        let long_early_discharge: f64 = long_discharge[..5].iter().sum();
        let short_early_gas_charge: f64 = short_gas_for_charging[..5].iter().sum();
        let long_early_gas_charge: f64 = long_gas_for_charging[..5].iter().sum();

        assert!(short_early_discharge > long_early_discharge);
        assert!(short_early_gas_charge > long_early_gas_charge);
    }

    #[test]
    fn test_limited_forecast_respects_soc_reserve() {
        let renewable = vec![80.0; 6];
        let load = vec![100.0; 6];

        let (_, discharge, soc, _, _) = apply_limited_forecast_dispatch_with_params(
            &renewable, &load, 0.0, 40.0, 1.0, 6, 6, 0.5, 0.0, 24.0, true, 40.0,
        );

        assert!(discharge.iter().sum::<f64>() > 0.0);
        assert!(soc.iter().all(|&x| x >= 20.0 - 1e-6));
        assert!(soc[5] >= 20.0 - 1e-6);
    }

    #[test]
    fn test_limited_forecast_charges_from_gas_below_peak_line() {
        let renewable = vec![100.0, 100.0, 0.0];
        let load = vec![100.0, 100.0, 200.0];

        let (charge, discharge, soc, curtailed, gas_for_charging) =
            apply_limited_forecast_dispatch_with_params(
                &renewable, &load, 0.0, 100.0, 1.0, 3, 3, 0.0, 0.0, 24.0, true, 0.0,
            );

        assert!(gas_for_charging[0] > 99.0);
        assert!(charge[0] > 99.0);
        assert!(discharge[2] > 99.0);
        assert!(soc[2] < 1.0);
        assert!(curtailed.iter().all(|&x| x.abs() < 1e-6));
    }

    #[test]
    fn test_limited_forecast_avoids_gas_charge_without_peak_improvement() {
        let renewable = vec![200.0, 100.0, 0.0];
        let load = vec![100.0, 100.0, 200.0];

        let (_, _, _, _, gas_for_charging) = apply_limited_forecast_dispatch_with_params(
            &renewable, &load, 0.0, 100.0, 1.0, 3, 3, 0.0, 0.0, 24.0, true, 0.0,
        );

        assert!(gas_for_charging.iter().all(|&x| x.abs() < 1e-6));
    }

    #[test]
    fn test_limited_forecast_gas_charging_toggle() {
        let renewable = vec![100.0, 100.0, 0.0];
        let load = vec![100.0, 100.0, 200.0];

        let (_, _, _, _, gas_for_charging_allowed) = apply_limited_forecast_dispatch_with_params(
            &renewable, &load, 0.0, 100.0, 1.0, 3, 3, 0.0, 0.0, 24.0, true, 0.0,
        );
        let (_, _, _, _, gas_for_charging_blocked) = apply_limited_forecast_dispatch_with_params(
            &renewable, &load, 0.0, 100.0, 1.0, 3, 3, 0.0, 0.0, 24.0, false, 0.0,
        );

        assert!(gas_for_charging_allowed.iter().sum::<f64>() > 99.0);
        assert!(gas_for_charging_blocked.iter().all(|&x| x.abs() < 1e-6));
    }

    #[test]
    fn test_limited_forecast_flat_deficit_has_smooth_peak_line() {
        let renewable = vec![0.0; 24];
        let load = vec![100.0; 24];

        let (_, discharge, _, _, gas_for_charging) = apply_limited_forecast_dispatch_with_params(
            &renewable, &load, 0.0, 120.0, 0.85, 24, 24, 0.0, 0.0, 0.0, false, 120.0,
        );

        let residual_gas: Vec<f64> = load
            .iter()
            .zip(discharge.iter())
            .zip(gas_for_charging.iter())
            .map(|((&load, &discharge), &gas_charge)| {
                (load - discharge * 0.85).max(0.0) + gas_charge
            })
            .collect();
        let peak = residual_gas.iter().copied().fold(0.0, f64::max);
        let trough = residual_gas.iter().copied().fold(f64::INFINITY, f64::min);

        assert!(peak - trough < 0.01);
        assert!((peak - 95.75).abs() < 0.01);
    }

    #[test]
    fn test_hourly_commitment_reacts_before_daily_commitment() {
        let mut renewable = vec![90.0; 36];
        let mut load = vec![100.0; 36];
        renewable[30] = 0.0;
        load[30] = 180.0;

        let (_, daily_discharge, _, _, _) = apply_limited_forecast_dispatch_with_params(
            &renewable, &load, 0.0, 80.0, 1.0, 24, 24, 0.0, 0.0, 0.0, false, 80.0,
        );
        let (_, hourly_discharge, _, _, _) = apply_limited_forecast_dispatch_with_params(
            &renewable, &load, 0.0, 80.0, 1.0, 24, 1, 0.0, 0.0, 0.0, false, 80.0,
        );

        let daily_early: f64 = daily_discharge[..24].iter().sum();
        let hourly_early: f64 = hourly_discharge[..24].iter().sum();
        let daily_peak_hour = daily_discharge[30];
        let hourly_peak_hour = hourly_discharge[30];

        assert!(hourly_early < daily_early);
        assert!(hourly_peak_hour > daily_peak_hour);
    }

    #[test]
    fn test_renewable_forecast_error_uses_persistence_blend() {
        let renewable = vec![10.0, 20.0, 30.0, 40.0, 100.0, 120.0];

        let perfect = forecast_renewable_generation(&renewable, 0.0);
        let persistence = forecast_renewable_generation(&renewable, 100.0);
        let blended = forecast_renewable_generation(&renewable, 50.0);

        assert_eq!(perfect, renewable);
        assert_eq!(persistence, renewable);
        assert_eq!(blended, renewable);

        let mut multi_day = vec![0.0; 48];
        multi_day[1] = 20.0;
        multi_day[25] = 100.0;

        let persistence = forecast_renewable_generation(&multi_day, 100.0);
        let blended = forecast_renewable_generation(&multi_day, 50.0);

        assert!((persistence[25] - 20.0).abs() < 1e-9);
        assert!((blended[25] - 60.0).abs() < 1e-9);
    }

    #[test]
    fn test_limited_forecast_full_year_shape() {
        let mut renewable = vec![0.0; HOURS_PER_YEAR];
        let load = vec![100.0; HOURS_PER_YEAR];
        for day in 0..(HOURS_PER_YEAR / 24) {
            for hour in 10..16 {
                renewable[day * 24 + hour] = 150.0;
            }
        }

        let (charge, discharge, soc, curtailed, gas_for_charging) =
            apply_limited_forecast_dispatch(&renewable, &load, 0.0, 100.0, 0.85);

        assert_eq!(charge.len(), HOURS_PER_YEAR);
        assert_eq!(discharge.len(), HOURS_PER_YEAR);
        assert_eq!(soc.len(), HOURS_PER_YEAR);
        assert_eq!(curtailed.len(), HOURS_PER_YEAR);
        assert!(charge.iter().sum::<f64>() > 0.0);
        assert!(discharge.iter().sum::<f64>() > 0.0);
        assert!(gas_for_charging.iter().all(|&x| x >= 0.0));
        assert!(soc.iter().all(|&x| (-1e-9..=100.0 + 1e-9).contains(&x)));
    }
}
