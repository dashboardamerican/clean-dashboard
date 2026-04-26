/// Depreciation schedules for LCOE calculations
///
/// Implements MACRS (Modified Accelerated Cost Recovery System) schedules
/// used for tax depreciation of capital assets.
use crate::types::DepreciationMethod;

/// MACRS 5-year depreciation schedule
/// Used for solar, wind, and storage assets
pub const MACRS_5_YEAR: [f64; 6] = [0.20, 0.32, 0.192, 0.1152, 0.1152, 0.0576];

/// MACRS 15-year depreciation schedule
/// Used for certain utility equipment
pub const MACRS_15_YEAR: [f64; 16] = [
    0.05, 0.095, 0.0855, 0.077, 0.0693, 0.0623, 0.059, 0.059, 0.0591, 0.059, 0.0591, 0.059, 0.0591,
    0.059, 0.0591, 0.0295,
];

/// Get depreciation schedule for a given method
///
/// # Arguments
/// * `method` - Depreciation method
///
/// # Returns
/// * Vector of annual depreciation fractions
pub fn get_depreciation_schedule(method: DepreciationMethod) -> Vec<f64> {
    match method {
        DepreciationMethod::Macrs5 => MACRS_5_YEAR.to_vec(),
        DepreciationMethod::Macrs15 => MACRS_15_YEAR.to_vec(),
        DepreciationMethod::StraightLine => {
            // 5-year straight line for compatibility
            vec![0.2, 0.2, 0.2, 0.2, 0.2]
        }
    }
}

/// Calculate depreciation for an asset over its lifetime
///
/// # Arguments
/// * `capex` - Capital expenditure (after ITC)
/// * `method` - Depreciation method
/// * `project_lifetime` - Project lifetime in years
///
/// # Returns
/// * Vector of annual depreciation amounts for each project year
pub fn calculate_depreciation(
    capex: f64,
    method: DepreciationMethod,
    project_lifetime: u32,
) -> Vec<f64> {
    let schedule = get_depreciation_schedule(method);
    let mut depreciation = vec![0.0; project_lifetime as usize];

    for (year, &rate) in schedule.iter().enumerate() {
        if year < project_lifetime as usize {
            depreciation[year] = capex * rate;
        }
    }

    depreciation
}

/// Calculate depreciation for an asset with replacement
///
/// If asset lifetime < project lifetime, the asset is replaced.
/// Depreciation restarts on each replacement.
///
/// # Arguments
/// * `capex` - Initial capital expenditure (before ITC)
/// * `itc_rate` - Investment Tax Credit rate (0-1)
/// * `method` - Depreciation method
/// * `asset_lifetime` - Asset lifetime in years
/// * `project_lifetime` - Project lifetime in years
/// * `inflation_rate` - Annual inflation rate for replacement cost
///
/// # Returns
/// * Vector of annual depreciation amounts for each project year
pub fn calculate_depreciation_with_replacement(
    capex: f64,
    itc_rate: f64,
    method: DepreciationMethod,
    asset_lifetime: u32,
    project_lifetime: u32,
    inflation_rate: f64,
) -> Vec<f64> {
    let schedule = get_depreciation_schedule(method);
    let mut depreciation = vec![0.0; project_lifetime as usize];

    let mut current_year = 0u32;
    let mut install_number = 0;

    while current_year < project_lifetime {
        // Calculate replacement cost (inflated)
        let inflation_factor = (1.0 + inflation_rate).powi(current_year as i32);
        let replacement_capex = capex * inflation_factor * (1.0 - itc_rate);

        // Apply depreciation schedule for this installation
        for (schedule_year, &rate) in schedule.iter().enumerate() {
            let project_year = current_year as usize + schedule_year;
            if project_year < project_lifetime as usize {
                depreciation[project_year] += replacement_capex * rate;
            }
        }

        // Move to next replacement
        current_year += asset_lifetime;
        install_number += 1;
    }

    depreciation
}

/// Calculate tax shield from depreciation
///
/// # Arguments
/// * `depreciation` - Annual depreciation amounts
/// * `tax_rate` - Corporate tax rate (e.g., 0.21 for 21%)
///
/// # Returns
/// * Vector of annual tax shield amounts
pub fn calculate_tax_shield(depreciation: &[f64], tax_rate: f64) -> Vec<f64> {
    depreciation.iter().map(|d| d * tax_rate).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macrs_5_year_sums_to_one() {
        let sum: f64 = MACRS_5_YEAR.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_macrs_15_year_sums_to_one() {
        let sum: f64 = MACRS_15_YEAR.iter().sum();
        assert!((sum - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_depreciation_basic() {
        let capex = 1_000_000.0;
        let depreciation = calculate_depreciation(capex, DepreciationMethod::Macrs5, 20);

        // First year should be 20% of capex
        assert!((depreciation[0] - 200_000.0).abs() < 0.01);

        // Total depreciation over schedule should equal capex
        let total: f64 = depreciation.iter().take(6).sum();
        assert!((total - capex).abs() < 0.01);
    }

    #[test]
    fn test_depreciation_with_replacement() {
        let capex = 1_000_000.0;
        let depreciation = calculate_depreciation_with_replacement(
            capex,
            0.3, // 30% ITC
            DepreciationMethod::Macrs5,
            15,   // Asset lasts 15 years
            20,   // Project is 20 years
            0.02, // 2% inflation
        );

        // Should have two installations: year 0 and year 15
        // Year 0: capex after ITC = 700,000, first depreciation = 140,000
        assert!((depreciation[0] - 140_000.0).abs() < 0.01);

        // Year 15: inflated capex = 1M * 1.02^15 = ~1.346M, after ITC = ~942K
        // First depreciation of replacement = ~188K
        assert!(depreciation[15] > 100_000.0); // Should have replacement depreciation
    }

    #[test]
    fn test_tax_shield() {
        let depreciation = vec![100_000.0, 80_000.0, 60_000.0];
        let tax_rate = 0.21;

        let shield = calculate_tax_shield(&depreciation, tax_rate);

        assert!((shield[0] - 21_000.0).abs() < 0.01);
        assert!((shield[1] - 16_800.0).abs() < 0.01);
        assert!((shield[2] - 12_600.0).abs() < 0.01);
    }
}
