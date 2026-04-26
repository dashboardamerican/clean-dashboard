pub mod default;
pub mod hybrid;
pub mod peak_shaver;

pub use default::apply_default_dispatch;
pub use hybrid::apply_hybrid_dispatch;
pub use peak_shaver::{apply_peak_shaver_dispatch, find_optimal_peak_line, test_battery_line};

/// Calculate clean energy delivered to load, tracking battery charging source over time.
pub fn calculate_clean_delivered(
    renewable_gen: &[f64],
    load: &[f64],
    clean_firm_gen: f64,
    battery_charge: &[f64],
    battery_discharge: &[f64],
    gas_for_charging: &[f64],
    battery_eff: f64,
    initial_soc: f64,
) -> Vec<f64> {
    let n = renewable_gen.len();
    let mut clean_delivered = vec![0.0; n];

    // Treat the initial full battery as clean inventory so Jan. 1 dispatch does not
    // artificially penalize clean delivery in the annual accounting.
    let mut battery_clean = initial_soc.max(0.0);
    let mut battery_gas = 0.0;

    for i in 0..n {
        if battery_charge[i] > 0.0 {
            let gas_charge = gas_for_charging[i].max(0.0).min(battery_charge[i]);
            let clean_charge = (battery_charge[i] - gas_charge).max(0.0);
            battery_clean += clean_charge;
            battery_gas += gas_charge;
        }

        let mut clean_discharge_delivery = 0.0;
        if battery_discharge[i] > 0.0 {
            let total_stored = battery_clean + battery_gas;
            if total_stored > 0.0 {
                let clean_fraction = battery_clean / total_stored;
                let clean_discharge = battery_discharge[i] * clean_fraction;
                let gas_discharge = battery_discharge[i] - clean_discharge;

                battery_clean = (battery_clean - clean_discharge).max(0.0);
                battery_gas = (battery_gas - gas_discharge).max(0.0);
                clean_discharge_delivery = clean_discharge * battery_eff;
            }
        }

        let direct_clean = (renewable_gen[i] + clean_firm_gen).min(load[i]);
        clean_delivered[i] = (direct_clean + clean_discharge_delivery).min(load[i]);
    }

    clean_delivered
}
