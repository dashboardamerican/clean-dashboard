/// Model Cache for WASM Optimizer
///
/// Provides thread-local storage for EmpiricalModel instances,
/// enabling fast model-based optimization without repeated loading.
///
/// Cache is keyed by (zone, battery_mode) tuple, with optional LRU eviction
/// to limit memory usage.
use super::EmpiricalModel;
use crate::BatteryMode;
use std::cell::RefCell;
use std::collections::HashMap;

/// Maximum number of models to cache (limits WASM memory usage)
/// Each V1 model is ~79KB, so 3 models = ~240KB
const MAX_CACHED_MODELS: usize = 3;

/// Cache key combining zone and battery mode
#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ModelKey {
    zone: String,
    battery_mode: u8,
}

impl ModelKey {
    pub fn new(zone: &str, battery_mode: BatteryMode) -> Self {
        Self {
            zone: zone.to_lowercase(),
            battery_mode: battery_mode as u8,
        }
    }
}

/// Entry in the model cache (model + access order for LRU)
struct CacheEntry {
    model: EmpiricalModel,
    access_order: u64,
}

/// Thread-local model cache
/// Uses RefCell for interior mutability in WASM single-threaded context
thread_local! {
    static MODEL_CACHE: RefCell<ModelCache> = RefCell::new(ModelCache::new());
}

/// LRU cache for empirical models
pub struct ModelCache {
    entries: HashMap<ModelKey, CacheEntry>,
    access_counter: u64,
}

impl ModelCache {
    fn new() -> Self {
        Self {
            entries: HashMap::new(),
            access_counter: 0,
        }
    }

    fn insert(&mut self, key: ModelKey, model: EmpiricalModel) {
        // Evict oldest entry if at capacity
        if self.entries.len() >= MAX_CACHED_MODELS && !self.entries.contains_key(&key) {
            self.evict_oldest();
        }

        self.access_counter += 1;
        self.entries.insert(
            key,
            CacheEntry {
                model,
                access_order: self.access_counter,
            },
        );
    }

    fn get(&mut self, key: &ModelKey) -> Option<&EmpiricalModel> {
        if let Some(entry) = self.entries.get_mut(key) {
            self.access_counter += 1;
            entry.access_order = self.access_counter;
            Some(&entry.model)
        } else {
            None
        }
    }

    fn contains(&self, key: &ModelKey) -> bool {
        self.entries.contains_key(key)
    }

    fn clear(&mut self) {
        self.entries.clear();
        self.access_counter = 0;
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.access_order)
            .map(|(key, _)| key.clone())
        {
            self.entries.remove(&oldest_key);
        }
    }

    /// Get list of currently loaded models
    fn loaded_models(&self) -> Vec<(String, u8)> {
        self.entries
            .keys()
            .map(|k| (k.zone.clone(), k.battery_mode))
            .collect()
    }
}

/// Load a model into the cache
///
/// # Arguments
/// * `zone` - Zone name (case-insensitive)
/// * `battery_mode` - Battery mode
/// * `bytes` - Serialized model bytes (bincode format)
///
/// # Returns
/// * `Ok(())` if model loaded successfully
/// * `Err(String)` if deserialization fails
pub fn load_model(zone: &str, battery_mode: BatteryMode, bytes: &[u8]) -> Result<(), String> {
    let model = EmpiricalModel::from_bytes(bytes)?;
    let key = ModelKey::new(zone, battery_mode);

    MODEL_CACHE.with(|cache| {
        cache.borrow_mut().insert(key, model);
    });

    Ok(())
}

/// Check if a model is loaded in the cache
pub fn is_model_loaded(zone: &str, battery_mode: BatteryMode) -> bool {
    let key = ModelKey::new(zone, battery_mode);
    MODEL_CACHE.with(|cache| cache.borrow().contains(&key))
}

/// Get a model from the cache for use in optimization
///
/// Returns a clone of the model if found (necessary due to RefCell borrowing)
pub fn get_model(zone: &str, battery_mode: BatteryMode) -> Option<EmpiricalModel> {
    let key = ModelKey::new(zone, battery_mode);
    MODEL_CACHE.with(|cache| cache.borrow_mut().get(&key).map(|m| m.clone()))
}

/// Clear all cached models (free memory)
pub fn clear_models() {
    MODEL_CACHE.with(|cache| {
        cache.borrow_mut().clear();
    });
}

/// Get list of currently loaded models
pub fn loaded_models() -> Vec<(String, u8)> {
    MODEL_CACHE.with(|cache| cache.borrow().loaded_models())
}

/// Get cache statistics
pub fn cache_stats() -> (usize, usize) {
    MODEL_CACHE.with(|cache| {
        let cache = cache.borrow();
        (cache.entries.len(), MAX_CACHED_MODELS)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::GridConfig;

    fn create_test_model() -> EmpiricalModel {
        // Create a minimal model for testing
        let config = GridConfig {
            solar_min: 0.0,
            solar_max: 100.0,
            solar_step: 100.0,
            wind_min: 0.0,
            wind_max: 100.0,
            wind_step: 100.0,
            storage_min: 0.0,
            storage_max: 100.0,
            storage_step: 100.0,
            cf_min: 0.0,
            cf_max: 100.0,
            cf_step: 100.0,
        };
        EmpiricalModel::new(config)
    }

    #[test]
    fn test_load_and_retrieve_model() {
        // Clear any existing models
        clear_models();

        let model = create_test_model();
        let bytes = model.to_bytes().unwrap();

        // Load model
        load_model("california", BatteryMode::Hybrid, &bytes).unwrap();

        // Check it's loaded
        assert!(is_model_loaded("california", BatteryMode::Hybrid));
        assert!(is_model_loaded("CALIFORNIA", BatteryMode::Hybrid)); // case insensitive
        assert!(!is_model_loaded("texas", BatteryMode::Hybrid));

        // Retrieve model
        let retrieved = get_model("california", BatteryMode::Hybrid);
        assert!(retrieved.is_some());

        clear_models();
    }

    #[test]
    fn test_lru_eviction() {
        clear_models();

        let model = create_test_model();
        let bytes = model.to_bytes().unwrap();

        // Load MAX_CACHED_MODELS + 1 models
        load_model("zone1", BatteryMode::Hybrid, &bytes).unwrap();
        load_model("zone2", BatteryMode::Hybrid, &bytes).unwrap();
        load_model("zone3", BatteryMode::Hybrid, &bytes).unwrap();

        // All three should be loaded
        assert!(is_model_loaded("zone1", BatteryMode::Hybrid));
        assert!(is_model_loaded("zone2", BatteryMode::Hybrid));
        assert!(is_model_loaded("zone3", BatteryMode::Hybrid));

        // Access zone1 to make it recently used
        let _ = get_model("zone1", BatteryMode::Hybrid);

        // Load a fourth model - zone2 should be evicted (oldest unused)
        load_model("zone4", BatteryMode::Hybrid, &bytes).unwrap();

        assert!(is_model_loaded("zone1", BatteryMode::Hybrid)); // recently accessed
        assert!(!is_model_loaded("zone2", BatteryMode::Hybrid)); // evicted
        assert!(is_model_loaded("zone3", BatteryMode::Hybrid));
        assert!(is_model_loaded("zone4", BatteryMode::Hybrid));

        clear_models();
    }

    #[test]
    fn test_clear_models() {
        clear_models();

        let model = create_test_model();
        let bytes = model.to_bytes().unwrap();

        load_model("california", BatteryMode::Hybrid, &bytes).unwrap();
        assert!(is_model_loaded("california", BatteryMode::Hybrid));

        clear_models();
        assert!(!is_model_loaded("california", BatteryMode::Hybrid));
    }
}
