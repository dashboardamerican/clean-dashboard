/// Evaluation cache for avoiding redundant simulations
///
/// Uses a hash map with portfolio parameters as keys to cache
/// simulation + LCOE results.
use std::collections::HashMap;

/// Cached evaluation result
#[derive(Clone, Debug)]
pub struct CachedResult {
    pub lcoe: f64,
    pub clean_match: f64,
}

/// Cache key for portfolio lookup
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
struct CacheKey {
    // Store as fixed-point integers for reliable hashing
    solar_x10: i32,
    wind_x10: i32,
    storage_x10: i32,
    clean_firm_x10: i32,
}

impl CacheKey {
    fn new(solar: f64, wind: f64, storage: f64, clean_firm: f64) -> Self {
        Self {
            solar_x10: (solar * 10.0).round() as i32,
            wind_x10: (wind * 10.0).round() as i32,
            storage_x10: (storage * 10.0).round() as i32,
            clean_firm_x10: (clean_firm * 10.0).round() as i32,
        }
    }
}

/// Evaluation cache
pub struct EvalCache {
    cache: HashMap<CacheKey, CachedResult>,
    hits: u32,
    misses: u32,
    last_eval_count: u32,
}

impl EvalCache {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            hits: 0,
            misses: 0,
            last_eval_count: 0,
        }
    }

    /// Get a cached result if it exists
    pub fn get(
        &mut self,
        solar: f64,
        wind: f64,
        storage: f64,
        clean_firm: f64,
    ) -> Option<CachedResult> {
        let key = CacheKey::new(solar, wind, storage, clean_firm);
        if let Some(result) = self.cache.get(&key) {
            self.hits += 1;
            Some(result.clone())
        } else {
            self.misses += 1;
            None
        }
    }

    /// Store a result in the cache
    pub fn put(
        &mut self,
        solar: f64,
        wind: f64,
        storage: f64,
        clean_firm: f64,
        result: CachedResult,
    ) {
        let key = CacheKey::new(solar, wind, storage, clean_firm);
        self.cache.insert(key, result);
    }

    /// Get number of evaluations in the last operation
    pub fn last_eval_count(&self) -> u32 {
        self.last_eval_count
    }

    /// Set the last evaluation count
    pub fn set_last_eval_count(&mut self, count: u32) {
        self.last_eval_count = count;
    }

    /// Get cache statistics
    pub fn stats(&self) -> (u32, u32, usize) {
        (self.hits, self.misses, self.cache.len())
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.cache.clear();
        self.hits = 0;
        self.misses = 0;
    }
}

impl Default for EvalCache {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_hit_miss() {
        let mut cache = EvalCache::new();

        // Miss on first access
        assert!(cache.get(100.0, 50.0, 25.0, 10.0).is_none());

        // Store result
        cache.put(
            100.0,
            50.0,
            25.0,
            10.0,
            CachedResult {
                lcoe: 45.0,
                clean_match: 65.0,
            },
        );

        // Hit on second access
        let result = cache.get(100.0, 50.0, 25.0, 10.0);
        assert!(result.is_some());
        assert!((result.unwrap().lcoe - 45.0).abs() < 0.01);

        // Check stats
        let (hits, misses, size) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
        assert_eq!(size, 1);
    }

    #[test]
    fn test_cache_precision() {
        let mut cache = EvalCache::new();

        // Store result at exact grid point
        cache.put(
            100.0,
            50.0,
            25.0,
            10.0,
            CachedResult {
                lcoe: 45.0,
                clean_match: 65.0,
            },
        );

        // Should hit with exact same values
        assert!(cache.get(100.0, 50.0, 25.0, 10.0).is_some());

        // Should hit with values that round to same 0.1 precision
        assert!(cache.get(100.04, 50.04, 25.04, 10.04).is_some());

        // Should miss with significantly different values (0.1 precision difference)
        assert!(cache.get(100.1, 50.0, 25.0, 10.0).is_none());
    }
}
