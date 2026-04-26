/// Empirical Model for Fast Clean Match Prediction
///
/// Uses a pre-computed 4D lookup table with trilinear interpolation
/// to quickly predict clean match % for any portfolio configuration.
/// This enables fast filtering of candidate portfolios before
/// running full simulations.
use serde::{Deserialize, Serialize};

/// Grid configuration for lookup table
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GridConfig {
    pub solar_min: f64,
    pub solar_max: f64,
    pub solar_step: f64,
    pub wind_min: f64,
    pub wind_max: f64,
    pub wind_step: f64,
    pub storage_min: f64,
    pub storage_max: f64,
    pub storage_step: f64,
    pub cf_min: f64,
    pub cf_max: f64,
    pub cf_step: f64,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            solar_min: 0.0,
            solar_max: 2400.0,
            solar_step: 100.0, // Coarser step for larger range
            wind_min: 0.0,
            wind_max: 2400.0,
            wind_step: 100.0, // Coarser step for larger range
            storage_min: 0.0,
            storage_max: 2400.0,
            storage_step: 100.0,
            cf_min: 0.0,
            cf_max: 200.0,
            cf_step: 20.0,
        }
    }
}

impl GridConfig {
    /// V1-style memory-optimized configuration
    ///
    /// Grid points: 11 × 6 × 25 × 6 = 9,900 (vs 172,875 default)
    /// Model size: ~232 KB per zone (with gas data)
    ///
    /// Bounds from OPTIMIZER_VERSIONS.md V1 optimizer:
    /// - Solar: 0-1000 MW (100 MW steps) → 11 points
    /// - Wind: 0-500 MW (100 MW steps) → 6 points
    /// - Storage: 0-2400 MWh (100 MWh steps) → 25 points
    /// - Clean Firm: 0-125 MW (25 MW steps) → 6 points
    pub fn v1_optimized() -> Self {
        Self {
            solar_min: 0.0,
            solar_max: 1000.0,
            solar_step: 100.0,
            wind_min: 0.0,
            wind_max: 500.0,
            wind_step: 100.0,
            storage_min: 0.0,
            storage_max: 2400.0,
            storage_step: 100.0,
            cf_min: 0.0,
            cf_max: 125.0,
            cf_step: 25.0,
        }
    }

    /// V2 fine-grained configuration for better optimization quality
    ///
    /// Grid points: 21 × 11 × 49 × 13 = 147,147
    /// Model size: ~3.5 MB per zone (with gas data)
    ///
    /// Finer steps capture inter-grid optima:
    /// - Solar: 0-1000 MW (50 MW steps) → 21 points
    /// - Wind: 0-500 MW (50 MW steps) → 11 points
    /// - Storage: 0-2400 MWh (50 MWh steps) → 49 points
    /// - Clean Firm: 0-125 MW (10 MW steps) → 13 points
    pub fn v2_fine() -> Self {
        Self {
            solar_min: 0.0,
            solar_max: 1000.0,
            solar_step: 50.0,
            wind_min: 0.0,
            wind_max: 500.0,
            wind_step: 50.0,
            storage_min: 0.0,
            storage_max: 2400.0,
            storage_step: 50.0,
            cf_min: 0.0,
            cf_max: 125.0,
            cf_step: 10.0,
        }
    }

    /// Calculate number of steps for each dimension
    pub fn dimensions(&self) -> (usize, usize, usize, usize) {
        let solar_steps = ((self.solar_max - self.solar_min) / self.solar_step) as usize + 1;
        let wind_steps = ((self.wind_max - self.wind_min) / self.wind_step) as usize + 1;
        let storage_steps =
            ((self.storage_max - self.storage_min) / self.storage_step) as usize + 1;
        let cf_steps = ((self.cf_max - self.cf_min) / self.cf_step) as usize + 1;
        (solar_steps, wind_steps, storage_steps, cf_steps)
    }

    /// Calculate total number of grid points
    pub fn total_points(&self) -> usize {
        let (s, w, st, cf) = self.dimensions();
        s * w * st * cf
    }
}

/// Portfolio candidate for optimization
#[derive(Clone, Debug)]
pub struct Portfolio {
    pub solar: f64,
    pub wind: f64,
    pub storage: f64,
    pub cf: f64,
}

/// Empirical model with 4D lookup tables for clean_match, gas capacity, and gas generation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmpiricalModel {
    /// 4D lookup table for clean_match % stored as flat vector
    /// Index: [solar_idx * wind_size * storage_size * cf_size +
    ///         wind_idx * storage_size * cf_size +
    ///         storage_idx * cf_size +
    ///         cf_idx]
    pub data: Vec<f64>,
    /// 4D lookup table for peak gas capacity (MW) needed
    /// Same indexing as data vector
    #[serde(default)]
    pub gas_data: Vec<f64>,
    /// 4D lookup table for total gas generation (MWh) per year
    /// This enables accurate fuel cost estimation in LCOE ranking
    #[serde(default)]
    pub gas_gen_data: Vec<f64>,
    pub config: GridConfig,
    solar_steps: usize,
    wind_steps: usize,
    storage_steps: usize,
    cf_steps: usize,
}

impl EmpiricalModel {
    /// Create a new empty model with given configuration
    pub fn new(config: GridConfig) -> Self {
        let (solar_steps, wind_steps, storage_steps, cf_steps) = config.dimensions();
        let size = config.total_points();
        Self {
            data: vec![0.0; size],
            gas_data: vec![0.0; size],
            gas_gen_data: vec![0.0; size],
            config,
            solar_steps,
            wind_steps,
            storage_steps,
            cf_steps,
        }
    }

    /// Check if model has gas capacity data (for backward compatibility)
    pub fn has_gas_data(&self) -> bool {
        !self.gas_data.is_empty() && self.gas_data.iter().any(|&v| v > 0.0)
    }

    /// Check if model has gas generation data (for full LCOE estimation)
    pub fn has_gas_gen_data(&self) -> bool {
        !self.gas_gen_data.is_empty() && self.gas_gen_data.iter().any(|&v| v > 0.0)
    }

    /// Load model from binary data
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        bincode::deserialize(bytes).map_err(|e| format!("Failed to deserialize model: {}", e))
    }

    /// Serialize model to binary data
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        bincode::serialize(self).map_err(|e| format!("Failed to serialize model: {}", e))
    }

    /// Calculate flat index for grid position
    fn index(&self, si: usize, wi: usize, sti: usize, cfi: usize) -> usize {
        si * self.wind_steps * self.storage_steps * self.cf_steps
            + wi * self.storage_steps * self.cf_steps
            + sti * self.cf_steps
            + cfi
    }

    /// Set clean_match value at grid position
    pub fn set(&mut self, solar: f64, wind: f64, storage: f64, cf: f64, value: f64) {
        let si = ((solar - self.config.solar_min) / self.config.solar_step).round() as usize;
        let wi = ((wind - self.config.wind_min) / self.config.wind_step).round() as usize;
        let sti = ((storage - self.config.storage_min) / self.config.storage_step).round() as usize;
        let cfi = ((cf - self.config.cf_min) / self.config.cf_step).round() as usize;

        if si < self.solar_steps
            && wi < self.wind_steps
            && sti < self.storage_steps
            && cfi < self.cf_steps
        {
            let idx = self.index(si, wi, sti, cfi);
            self.data[idx] = value;
        }
    }

    /// Set peak gas capacity (MW) at grid position
    pub fn set_gas(&mut self, solar: f64, wind: f64, storage: f64, cf: f64, peak_gas: f64) {
        let si = ((solar - self.config.solar_min) / self.config.solar_step).round() as usize;
        let wi = ((wind - self.config.wind_min) / self.config.wind_step).round() as usize;
        let sti = ((storage - self.config.storage_min) / self.config.storage_step).round() as usize;
        let cfi = ((cf - self.config.cf_min) / self.config.cf_step).round() as usize;

        if si < self.solar_steps
            && wi < self.wind_steps
            && sti < self.storage_steps
            && cfi < self.cf_steps
        {
            let idx = self.index(si, wi, sti, cfi);
            // Ensure gas_data is properly sized
            if self.gas_data.len() != self.data.len() {
                self.gas_data.resize(self.data.len(), 0.0);
            }
            self.gas_data[idx] = peak_gas;
        }
    }

    /// Set both clean_match and peak_gas at grid position (convenience method)
    pub fn set_both(
        &mut self,
        solar: f64,
        wind: f64,
        storage: f64,
        cf: f64,
        clean_match: f64,
        peak_gas: f64,
    ) {
        self.set(solar, wind, storage, cf, clean_match);
        self.set_gas(solar, wind, storage, cf, peak_gas);
    }

    /// Set gas generation (MWh) at grid position
    pub fn set_gas_gen(&mut self, solar: f64, wind: f64, storage: f64, cf: f64, gas_gen: f64) {
        let si = ((solar - self.config.solar_min) / self.config.solar_step).round() as usize;
        let wi = ((wind - self.config.wind_min) / self.config.wind_step).round() as usize;
        let sti = ((storage - self.config.storage_min) / self.config.storage_step).round() as usize;
        let cfi = ((cf - self.config.cf_min) / self.config.cf_step).round() as usize;

        if si < self.solar_steps
            && wi < self.wind_steps
            && sti < self.storage_steps
            && cfi < self.cf_steps
        {
            let idx = self.index(si, wi, sti, cfi);
            if self.gas_gen_data.len() != self.data.len() {
                self.gas_gen_data.resize(self.data.len(), 0.0);
            }
            self.gas_gen_data[idx] = gas_gen;
        }
    }

    /// Set all values at grid position: clean_match, peak_gas, and gas_generation
    pub fn set_all(
        &mut self,
        solar: f64,
        wind: f64,
        storage: f64,
        cf: f64,
        clean_match: f64,
        peak_gas: f64,
        gas_gen: f64,
    ) {
        self.set(solar, wind, storage, cf, clean_match);
        self.set_gas(solar, wind, storage, cf, peak_gas);
        self.set_gas_gen(solar, wind, storage, cf, gas_gen);
    }

    /// Get clean_match value at exact grid position
    fn get_exact(&self, si: usize, wi: usize, sti: usize, cfi: usize) -> f64 {
        if si < self.solar_steps
            && wi < self.wind_steps
            && sti < self.storage_steps
            && cfi < self.cf_steps
        {
            self.data[self.index(si, wi, sti, cfi)]
        } else {
            0.0
        }
    }

    /// Get gas capacity value at exact grid position
    fn get_gas_exact(&self, si: usize, wi: usize, sti: usize, cfi: usize) -> f64 {
        if self.gas_data.is_empty() {
            return 0.0;
        }
        if si < self.solar_steps
            && wi < self.wind_steps
            && sti < self.storage_steps
            && cfi < self.cf_steps
        {
            let idx = self.index(si, wi, sti, cfi);
            if idx < self.gas_data.len() {
                self.gas_data[idx]
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Predict clean match with 4D linear interpolation
    pub fn predict(&self, solar: f64, wind: f64, storage: f64, cf: f64) -> f64 {
        // Clamp to grid bounds
        let solar = solar.clamp(self.config.solar_min, self.config.solar_max);
        let wind = wind.clamp(self.config.wind_min, self.config.wind_max);
        let storage = storage.clamp(self.config.storage_min, self.config.storage_max);
        let cf = cf.clamp(self.config.cf_min, self.config.cf_max);

        // Convert to grid coordinates
        let si = (solar - self.config.solar_min) / self.config.solar_step;
        let wi = (wind - self.config.wind_min) / self.config.wind_step;
        let sti = (storage - self.config.storage_min) / self.config.storage_step;
        let cfi = (cf - self.config.cf_min) / self.config.cf_step;

        // Get integer indices and fractional parts
        let si0 = si.floor() as usize;
        let si1 = (si0 + 1).min(self.solar_steps - 1);
        let sf = si - si0 as f64;

        let wi0 = wi.floor() as usize;
        let wi1 = (wi0 + 1).min(self.wind_steps - 1);
        let wf = wi - wi0 as f64;

        let sti0 = sti.floor() as usize;
        let sti1 = (sti0 + 1).min(self.storage_steps - 1);
        let stf = sti - sti0 as f64;

        let cfi0 = cfi.floor() as usize;
        let cfi1 = (cfi0 + 1).min(self.cf_steps - 1);
        let cff = cfi - cfi0 as f64;

        // 4D linear interpolation (16 lookups)
        let mut sum = 0.0;
        for (s_idx, s_weight) in [(si0, 1.0 - sf), (si1, sf)] {
            for (w_idx, w_weight) in [(wi0, 1.0 - wf), (wi1, wf)] {
                for (st_idx, st_weight) in [(sti0, 1.0 - stf), (sti1, stf)] {
                    for (cf_idx, cf_weight) in [(cfi0, 1.0 - cff), (cfi1, cff)] {
                        let weight = s_weight * w_weight * st_weight * cf_weight;
                        sum += weight * self.get_exact(s_idx, w_idx, st_idx, cf_idx);
                    }
                }
            }
        }

        sum
    }

    /// Predict peak gas capacity (MW) with 4D linear interpolation
    /// Returns 0 if model doesn't have gas data
    pub fn predict_gas(&self, solar: f64, wind: f64, storage: f64, cf: f64) -> f64 {
        if !self.has_gas_data() {
            return 0.0;
        }

        // Clamp to grid bounds
        let solar = solar.clamp(self.config.solar_min, self.config.solar_max);
        let wind = wind.clamp(self.config.wind_min, self.config.wind_max);
        let storage = storage.clamp(self.config.storage_min, self.config.storage_max);
        let cf = cf.clamp(self.config.cf_min, self.config.cf_max);

        // Convert to grid coordinates
        let si = (solar - self.config.solar_min) / self.config.solar_step;
        let wi = (wind - self.config.wind_min) / self.config.wind_step;
        let sti = (storage - self.config.storage_min) / self.config.storage_step;
        let cfi = (cf - self.config.cf_min) / self.config.cf_step;

        // Get integer indices and fractional parts
        let si0 = si.floor() as usize;
        let si1 = (si0 + 1).min(self.solar_steps - 1);
        let sf = si - si0 as f64;

        let wi0 = wi.floor() as usize;
        let wi1 = (wi0 + 1).min(self.wind_steps - 1);
        let wf = wi - wi0 as f64;

        let sti0 = sti.floor() as usize;
        let sti1 = (sti0 + 1).min(self.storage_steps - 1);
        let stf = sti - sti0 as f64;

        let cfi0 = cfi.floor() as usize;
        let cfi1 = (cfi0 + 1).min(self.cf_steps - 1);
        let cff = cfi - cfi0 as f64;

        // 4D linear interpolation (16 lookups)
        let mut sum = 0.0;
        for (s_idx, s_weight) in [(si0, 1.0 - sf), (si1, sf)] {
            for (w_idx, w_weight) in [(wi0, 1.0 - wf), (wi1, wf)] {
                for (st_idx, st_weight) in [(sti0, 1.0 - stf), (sti1, stf)] {
                    for (cf_idx, cf_weight) in [(cfi0, 1.0 - cff), (cfi1, cff)] {
                        let weight = s_weight * w_weight * st_weight * cf_weight;
                        sum += weight * self.get_gas_exact(s_idx, w_idx, st_idx, cf_idx);
                    }
                }
            }
        }

        sum
    }

    /// Get gas generation value at exact grid position
    fn get_gas_gen_exact(&self, si: usize, wi: usize, sti: usize, cfi: usize) -> f64 {
        if self.gas_gen_data.is_empty() {
            return 0.0;
        }
        if si < self.solar_steps
            && wi < self.wind_steps
            && sti < self.storage_steps
            && cfi < self.cf_steps
        {
            let idx = self.index(si, wi, sti, cfi);
            if idx < self.gas_gen_data.len() {
                self.gas_gen_data[idx]
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Predict total gas generation (MWh/year) with 4D linear interpolation
    /// Returns 0 if model doesn't have gas generation data
    pub fn predict_gas_gen(&self, solar: f64, wind: f64, storage: f64, cf: f64) -> f64 {
        if !self.has_gas_gen_data() {
            return 0.0;
        }

        // Clamp to grid bounds
        let solar = solar.clamp(self.config.solar_min, self.config.solar_max);
        let wind = wind.clamp(self.config.wind_min, self.config.wind_max);
        let storage = storage.clamp(self.config.storage_min, self.config.storage_max);
        let cf = cf.clamp(self.config.cf_min, self.config.cf_max);

        // Convert to grid coordinates
        let si = (solar - self.config.solar_min) / self.config.solar_step;
        let wi = (wind - self.config.wind_min) / self.config.wind_step;
        let sti = (storage - self.config.storage_min) / self.config.storage_step;
        let cfi = (cf - self.config.cf_min) / self.config.cf_step;

        // Get integer indices and fractional parts
        let si0 = si.floor() as usize;
        let si1 = (si0 + 1).min(self.solar_steps - 1);
        let sf = si - si0 as f64;

        let wi0 = wi.floor() as usize;
        let wi1 = (wi0 + 1).min(self.wind_steps - 1);
        let wf = wi - wi0 as f64;

        let sti0 = sti.floor() as usize;
        let sti1 = (sti0 + 1).min(self.storage_steps - 1);
        let stf = sti - sti0 as f64;

        let cfi0 = cfi.floor() as usize;
        let cfi1 = (cfi0 + 1).min(self.cf_steps - 1);
        let cff = cfi - cfi0 as f64;

        // 4D linear interpolation (16 lookups)
        let mut sum = 0.0;
        for (s_idx, s_weight) in [(si0, 1.0 - sf), (si1, sf)] {
            for (w_idx, w_weight) in [(wi0, 1.0 - wf), (wi1, wf)] {
                for (st_idx, st_weight) in [(sti0, 1.0 - stf), (sti1, stf)] {
                    for (cf_idx, cf_weight) in [(cfi0, 1.0 - cff), (cfi1, cff)] {
                        let weight = s_weight * w_weight * st_weight * cf_weight;
                        sum += weight * self.get_gas_gen_exact(s_idx, w_idx, st_idx, cf_idx);
                    }
                }
            }
        }

        sum
    }

    /// Find all portfolios where predicted match is within range of target
    pub fn filter_candidates(
        &self,
        target: f64,
        tolerance: f64,
        max_solar: f64,
        max_wind: f64,
        max_storage: f64,
        max_cf: f64,
        enable_solar: bool,
        enable_wind: bool,
        enable_storage: bool,
        enable_cf: bool,
    ) -> Vec<Portfolio> {
        let mut candidates = Vec::new();

        let solar_range: Vec<f64> = if enable_solar {
            (0..=((max_solar.min(self.config.solar_max) / self.config.solar_step) as usize))
                .map(|i| self.config.solar_min + i as f64 * self.config.solar_step)
                .collect()
        } else {
            vec![0.0]
        };

        let wind_range: Vec<f64> = if enable_wind {
            (0..=((max_wind.min(self.config.wind_max) / self.config.wind_step) as usize))
                .map(|i| self.config.wind_min + i as f64 * self.config.wind_step)
                .collect()
        } else {
            vec![0.0]
        };

        let storage_range: Vec<f64> = if enable_storage {
            (0..=((max_storage.min(self.config.storage_max) / self.config.storage_step) as usize))
                .map(|i| self.config.storage_min + i as f64 * self.config.storage_step)
                .collect()
        } else {
            vec![0.0]
        };

        for solar in &solar_range {
            for wind in &wind_range {
                for storage in &storage_range {
                    // Predict with CF=0 first
                    let base_match = self.predict(*solar, *wind, *storage, 0.0);

                    // Skip if already overshoots target by more than tolerance
                    if base_match > target + tolerance {
                        continue;
                    }

                    // Check if we can reach target with CF
                    let max_match = if enable_cf {
                        self.predict(*solar, *wind, *storage, max_cf.min(self.config.cf_max))
                    } else {
                        base_match
                    };

                    // Skip if can't reach target even with max CF
                    if max_match < target - tolerance {
                        continue;
                    }

                    // This portfolio can potentially hit target
                    candidates.push(Portfolio {
                        solar: *solar,
                        wind: *wind,
                        storage: *storage,
                        cf: 0.0, // Will be determined by binary search
                    });
                }
            }
        }

        candidates
    }

    /// Generate grid points for training data generation
    pub fn generate_grid_points(&self) -> Vec<(f64, f64, f64, f64)> {
        let mut points = Vec::with_capacity(self.config.total_points());

        for si in 0..self.solar_steps {
            let solar = self.config.solar_min + si as f64 * self.config.solar_step;
            for wi in 0..self.wind_steps {
                let wind = self.config.wind_min + wi as f64 * self.config.wind_step;
                for sti in 0..self.storage_steps {
                    let storage = self.config.storage_min + sti as f64 * self.config.storage_step;
                    for cfi in 0..self.cf_steps {
                        let cf = self.config.cf_min + cfi as f64 * self.config.cf_step;
                        points.push((solar, wind, storage, cf));
                    }
                }
            }
        }

        points
    }
}

/// Training sample for model building
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TrainingSample {
    pub solar: f64,
    pub wind: f64,
    pub storage: f64,
    pub clean_firm: f64,
    pub clean_match: f64,
    /// Peak gas capacity needed (MW)
    pub peak_gas: f64,
    /// Total gas generation (MWh/year) - for fuel cost estimation
    pub gas_generation: f64,
}

/// Build a lookup table from training samples (includes clean_match, peak_gas, and gas_generation)
pub fn build_lookup_table(samples: &[TrainingSample], config: GridConfig) -> EmpiricalModel {
    let mut model = EmpiricalModel::new(config);

    for sample in samples {
        model.set_all(
            sample.solar,
            sample.wind,
            sample.storage,
            sample.clean_firm,
            sample.clean_match,
            sample.peak_gas,
            sample.gas_generation,
        );
    }

    model
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_model() -> EmpiricalModel {
        let config = GridConfig {
            solar_min: 0.0,
            solar_max: 100.0,
            solar_step: 50.0,
            wind_min: 0.0,
            wind_max: 100.0,
            wind_step: 50.0,
            storage_min: 0.0,
            storage_max: 100.0,
            storage_step: 50.0,
            cf_min: 0.0,
            cf_max: 100.0,
            cf_step: 50.0,
        };
        let mut model = EmpiricalModel::new(config);

        // Fill with simple formula: match% = (solar + wind + cf) / 3
        for (solar, wind, storage, cf) in model.generate_grid_points() {
            let clean_match = (solar + wind + cf) / 3.0;
            model.set(solar, wind, storage, cf, clean_match);
        }

        model
    }

    #[test]
    fn test_exact_lookup() {
        let model = create_test_model();

        // Test exact grid points
        let result = model.predict(0.0, 0.0, 0.0, 0.0);
        assert!((result - 0.0).abs() < 0.1);

        let result = model.predict(100.0, 100.0, 0.0, 100.0);
        assert!((result - 100.0).abs() < 0.1);

        let result = model.predict(50.0, 50.0, 0.0, 50.0);
        assert!((result - 50.0).abs() < 0.1);
    }

    #[test]
    fn test_interpolation() {
        let model = create_test_model();

        // Test interpolation between grid points
        let result = model.predict(25.0, 25.0, 0.0, 25.0);
        // Should interpolate to approximately 25
        assert!((result - 25.0).abs() < 5.0);

        let result = model.predict(75.0, 75.0, 0.0, 75.0);
        // Should interpolate to approximately 75
        assert!((result - 75.0).abs() < 5.0);
    }

    #[test]
    fn test_filter_candidates() {
        let model = create_test_model();

        let candidates = model.filter_candidates(
            50.0,  // target
            5.0,   // tolerance
            100.0, // max_solar
            100.0, // max_wind
            100.0, // max_storage
            100.0, // max_cf
            true,  // enable_solar
            true,  // enable_wind
            true,  // enable_storage
            true,  // enable_cf
        );

        // Should have some candidates
        assert!(!candidates.is_empty());
    }

    #[test]
    fn test_serialization() {
        let model = create_test_model();

        let bytes = model.to_bytes().unwrap();
        let loaded = EmpiricalModel::from_bytes(&bytes).unwrap();

        // Verify same predictions
        assert!(
            (model.predict(50.0, 50.0, 0.0, 50.0) - loaded.predict(50.0, 50.0, 0.0, 50.0)).abs()
                < 0.01
        );
    }

    #[test]
    fn test_grid_dimensions() {
        let config = GridConfig::default();
        let (s, w, st, cf) = config.dimensions();

        // 2400/100 + 1 = 25
        assert_eq!(s, 25);
        assert_eq!(w, 25);
        assert_eq!(st, 25);
        // 200/20 + 1 = 11
        assert_eq!(cf, 11);

        assert_eq!(config.total_points(), 25 * 25 * 25 * 11);
    }

    #[test]
    fn test_v1_optimized_grid_dimensions() {
        let config = GridConfig::v1_optimized();
        let (s, w, st, cf) = config.dimensions();

        // V1 bounds: Solar 0-1000/100 = 11, Wind 0-500/100 = 6, Storage 0-2400/100 = 25, CF 0-125/25 = 6
        assert_eq!(s, 11, "solar steps");
        assert_eq!(w, 6, "wind steps");
        assert_eq!(st, 25, "storage steps");
        assert_eq!(cf, 6, "cf steps");

        // 11 * 6 * 25 * 6 = 9,900 points
        assert_eq!(config.total_points(), 9_900);
    }

    #[test]
    fn test_v1_model_size_estimate() {
        let config = GridConfig::v1_optimized();
        // Each f64 is 8 bytes, plus some overhead for config serialization
        let data_size = config.total_points() * 8;
        // ~79 KB for data alone
        assert!(data_size < 100_000, "Model data should be under 100KB");
        assert!(data_size > 70_000, "Model data should be at least 70KB");
    }
}
