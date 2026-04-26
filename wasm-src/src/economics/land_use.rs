//! Standalone land-use calculation.
//!
//! Mirrors Python's `calculate_system_land_use` in `multi_test.py:157`.
//! The same math runs inside `calculate_lcoe` and is exposed on `LcoeResult`,
//! but having it as a standalone function lets callers compute land use
//! without paying for a full LCOE pass.
//!
//! # Units
//! All capacities are in MW. Land cost params (`solar_land_direct`, etc.)
//! are in **acres/MW**. The returned struct exposes both `acres` and `mi²`
//! so the web UI doesn't have to convert.

use crate::types::CostParams;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

const ACRES_PER_SQUARE_MILE: f64 = 640.0;

/// Result of a land-use calculation.
#[wasm_bindgen]
#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize)]
pub struct LandUseResult {
    /// Direct (physical-footprint) land use in acres.
    pub direct_acres: f64,
    /// Total (direct + indirect, e.g. wind spacing) land use in acres.
    pub total_acres: f64,
    /// Direct land use in mi² (Python's headline number).
    pub direct_mi2: f64,
    /// Total land use in mi².
    pub total_mi2: f64,

    /// Per-technology direct contributions (acres). Useful for charts.
    pub solar_direct_acres: f64,
    pub wind_direct_acres: f64,
    pub clean_firm_direct_acres: f64,
    pub gas_direct_acres: f64,

    /// Per-technology total contributions (acres).
    /// Solar and gas have no significant indirect footprint, so total == direct
    /// for those two.
    pub solar_total_acres: f64,
    pub wind_total_acres: f64,
    pub clean_firm_total_acres: f64,
    pub gas_total_acres: f64,
}

/// Compute land use for a portfolio.
///
/// `gas_capacity_mw` should be the *peak* gas capacity needed by the system,
/// not annual gas energy. This matches Python's `gas_capacity_needed` input.
pub fn calculate_land_use(
    solar_capacity_mw: f64,
    wind_capacity_mw: f64,
    clean_firm_capacity_mw: f64,
    gas_capacity_mw: f64,
    costs: &CostParams,
) -> LandUseResult {
    let solar_direct = solar_capacity_mw * costs.solar_land_direct;
    let wind_direct = wind_capacity_mw * costs.wind_land_direct;
    let cf_direct = clean_firm_capacity_mw * costs.clean_firm_land_direct;
    let gas_direct = gas_capacity_mw * costs.gas_land_direct;

    // Solar: only direct (no significant indirect impact)
    // Wind: includes spacing between turbines (wind_land_total >> wind_land_direct)
    // Clean firm: includes exclusion zones, mining footprint
    // Gas: only direct (negligible indirect)
    let solar_total = solar_direct;
    let wind_total = wind_capacity_mw * costs.wind_land_total;
    let cf_total = clean_firm_capacity_mw * costs.clean_firm_land_total;
    let gas_total = gas_direct;

    let direct_acres = solar_direct + wind_direct + cf_direct + gas_direct;
    let total_acres = solar_total + wind_total + cf_total + gas_total;

    LandUseResult {
        direct_acres,
        total_acres,
        direct_mi2: direct_acres / ACRES_PER_SQUARE_MILE,
        total_mi2: total_acres / ACRES_PER_SQUARE_MILE,
        solar_direct_acres: solar_direct,
        wind_direct_acres: wind_direct,
        clean_firm_direct_acres: cf_direct,
        gas_direct_acres: gas_direct,
        solar_total_acres: solar_total,
        wind_total_acres: wind_total,
        clean_firm_total_acres: cf_total,
        gas_total_acres: gas_total,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::CostParams;

    /// Hand-computed parity fixture against Python's `calculate_system_land_use`.
    ///
    /// Inputs: solar=100 MW, wind=50 MW, clean_firm=20 MW, gas_capacity=137.436 MW.
    /// Costs: defaults (solar_land_direct=6.5, wind_land_direct=1.25,
    /// wind_land_total=50, clean_firm_land=1.0/1.0, gas_land_direct=0.19).
    ///
    /// Expected from running the Python `multi_test.calculate_system_land_use`
    /// on these inputs:
    ///   direct_mi2 = 1.1853325606436937
    ///   total_mi2  = 4.993926310643694
    ///   direct_acres = 758.612838811964
    ///   total_acres  = 3196.112838811964
    #[test]
    fn matches_python_reference() {
        let costs = CostParams::default_costs();
        let result = calculate_land_use(100.0, 50.0, 20.0, 137.43599374717837, &costs);

        let expected_direct_acres = 758.612838811964;
        let expected_total_acres = 3196.112838811964;
        let tol = 1e-9; // Tight: the math is bit-identical to Python.

        assert!(
            (result.direct_acres - expected_direct_acres).abs() < tol,
            "direct_acres: got {}, expected {}",
            result.direct_acres,
            expected_direct_acres
        );
        assert!(
            (result.total_acres - expected_total_acres).abs() < tol,
            "total_acres: got {}, expected {}",
            result.total_acres,
            expected_total_acres
        );
        assert!((result.direct_mi2 - 1.1853325606436937).abs() < 1e-12);
        assert!((result.total_mi2 - 4.993926310643694).abs() < 1e-12);
    }

    #[test]
    fn zero_capacities_returns_zero() {
        let costs = CostParams::default_costs();
        let r = calculate_land_use(0.0, 0.0, 0.0, 0.0, &costs);
        assert_eq!(r.direct_acres, 0.0);
        assert_eq!(r.total_acres, 0.0);
        assert_eq!(r.direct_mi2, 0.0);
        assert_eq!(r.total_mi2, 0.0);
    }

    #[test]
    fn wind_dominates_total_via_spacing() {
        // 100 MW wind only: direct = 125 ac, total = 5000 ac (40x bigger).
        let costs = CostParams::default_costs();
        let r = calculate_land_use(0.0, 100.0, 0.0, 0.0, &costs);
        assert_eq!(r.direct_acres, 125.0);
        assert_eq!(r.total_acres, 5000.0);
        assert!((r.total_acres / r.direct_acres - 40.0).abs() < 1e-9);
    }

    #[test]
    fn solar_and_gas_have_no_indirect() {
        let costs = CostParams::default_costs();
        let r = calculate_land_use(50.0, 0.0, 0.0, 30.0, &costs);
        // Solar: 50 * 6.5 = 325; gas: 30 * 0.19 = 5.7.
        assert!((r.solar_direct_acres - r.solar_total_acres).abs() < 1e-12);
        assert!((r.gas_direct_acres - r.gas_total_acres).abs() < 1e-12);
        assert!((r.direct_acres - r.total_acres).abs() < 1e-12);
    }

    #[test]
    fn additivity() {
        // Land use is linear in capacity: 2x inputs → 2x outputs.
        let costs = CostParams::default_costs();
        let a = calculate_land_use(50.0, 25.0, 10.0, 70.0, &costs);
        let b = calculate_land_use(100.0, 50.0, 20.0, 140.0, &costs);
        let tol = 1e-9;
        assert!((b.direct_acres - 2.0 * a.direct_acres).abs() < tol);
        assert!((b.total_acres - 2.0 * a.total_acres).abs() < tol);
    }
}
