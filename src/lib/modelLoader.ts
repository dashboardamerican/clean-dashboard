/**
 * EmpiricalModel Loader for WASM Optimizer
 *
 * Provides utilities for fetching and loading pre-computed empirical models
 * into the WASM model cache for faster optimizer candidate filtering.
 *
 * Models are stored as binary files (bincode serialized) in /models/<zone>_<mode>.bin
 */

import { BatteryMode } from '../types';

// Default model path (can be overridden for CDN deployment)
const MODEL_BASE_PATH = '/models';

// Battery mode names for file naming
const BATTERY_MODE_NAMES: Record<BatteryMode, string> = {
  [BatteryMode.Default]: 'default',
  [BatteryMode.PeakShaver]: 'peakshaver',
  [BatteryMode.Hybrid]: 'hybrid',
  [BatteryMode.LimitedForecast]: 'limitedforecast',
};

// Zone name normalization for file naming
function normalizeZoneName(zone: string): string {
  return zone.toLowerCase().replace(/[\s-]+/g, '_');
}

/**
 * Get the WASM module from the global window object
 */
function getWasmModule(): any {
  return (window as any).__wasmModule || null;
}

/**
 * Model loading status
 */
export interface ModelLoadingStatus {
  loading: boolean;
  loaded: boolean;
  error: string | null;
}

/**
 * Fetch model binary from server
 */
export async function fetchModel(
  zone: string,
  batteryMode: BatteryMode
): Promise<Uint8Array> {
  const normalizedZone = normalizeZoneName(zone);
  const modeName = BATTERY_MODE_NAMES[batteryMode];
  const url = `${MODEL_BASE_PATH}/${normalizedZone}_${modeName}.bin`;

  console.log(`[ModelLoader] Fetching model: ${url}`);

  const response = await fetch(url);
  if (!response.ok) {
    throw new Error(`Failed to fetch model: ${response.status} ${response.statusText}`);
  }

  const arrayBuffer = await response.arrayBuffer();
  console.log(`[ModelLoader] Model fetched: ${arrayBuffer.byteLength} bytes`);

  return new Uint8Array(arrayBuffer);
}

/**
 * Load a model into the WASM cache
 *
 * @param zone - Zone name (e.g., "California", "Texas")
 * @param batteryMode - Battery dispatch mode
 * @returns Promise that resolves when model is loaded
 * @throws Error if WASM module not available or model loading fails
 */
export async function loadModel(
  zone: string,
  batteryMode: BatteryMode
): Promise<void> {
  const wasm = getWasmModule();
  if (!wasm) {
    throw new Error('WASM module not loaded');
  }

  // Check if already loaded
  if (isModelLoaded(zone, batteryMode)) {
    console.log(`[ModelLoader] Model already cached: ${zone}/${batteryMode}`);
    return;
  }

  // Fetch model binary
  const bytes = await fetchModel(zone, batteryMode);

  // Load into WASM cache
  wasm.wasm_load_model(zone, batteryMode, bytes);

  console.log(`[ModelLoader] Model loaded into cache: ${zone}/${batteryMode}`);
}

/**
 * Check if a model is loaded in the WASM cache
 */
export function isModelLoaded(zone: string, batteryMode: BatteryMode): boolean {
  const wasm = getWasmModule();
  if (!wasm || !wasm.wasm_is_model_loaded) {
    return false;
  }

  return wasm.wasm_is_model_loaded(zone, batteryMode);
}

/**
 * Clear all models from the WASM cache
 */
export function clearModels(): void {
  const wasm = getWasmModule();
  if (wasm && wasm.wasm_clear_models) {
    wasm.wasm_clear_models();
    console.log('[ModelLoader] Model cache cleared');
  }
}

/**
 * Get list of currently loaded models
 */
export function getLoadedModels(): Array<[string, number]> {
  const wasm = getWasmModule();
  if (!wasm || !wasm.wasm_loaded_models) {
    return [];
  }
  return wasm.wasm_loaded_models();
}

/**
 * Get model cache statistics
 */
export function getCacheStats(): { loaded: number; max: number } {
  const wasm = getWasmModule();
  if (!wasm || !wasm.wasm_cache_stats) {
    return { loaded: 0, max: 3 };
  }
  return wasm.wasm_cache_stats();
}

/**
 * Preload model for a zone/mode combination
 *
 * This is a fire-and-forget operation that loads the model in the background.
 * Useful for preloading when user hovers over a zone or during idle time.
 *
 * @param zone - Zone name
 * @param batteryMode - Battery mode
 * @returns Promise that resolves when model is loaded or rejects on error
 */
export function preloadModel(
  zone: string,
  batteryMode: BatteryMode
): Promise<void> {
  // Only Hybrid mode models are available
  if (!hasModel(zone, batteryMode)) {
    console.log(`[ModelLoader] Skipping preload for ${zone}/${batteryMode} (no model available)`);
    return Promise.resolve();
  }

  console.log(`[ModelLoader] Preloading model for ${zone}/${batteryMode}`);
  return loadModel(zone, batteryMode).catch((err) => {
    console.warn(`[ModelLoader] Failed to preload model for ${zone}/${batteryMode}:`, err);
    // Don't throw - preload failures are non-critical
  });
}

/**
 * Ensure model is loaded for the current zone/mode
 *
 * This is the primary function to call before running optimization.
 * It will:
 * 1. Check if model exists for this zone/mode combination
 * 2. Check if the model is already loaded
 * 3. If not, attempt to load it
 * 4. Return silently if model cannot be loaded (optimizer will fallback to greedy)
 *
 * @param zone - Zone name
 * @param batteryMode - Battery mode
 * @returns Promise with loading status
 */
export async function ensureModelLoaded(
  zone: string,
  batteryMode: BatteryMode
): Promise<ModelLoadingStatus> {
  const status: ModelLoadingStatus = {
    loading: false,
    loaded: false,
    error: null,
  };

  // Only Hybrid mode models are available
  if (!hasModel(zone, batteryMode)) {
    status.error = `No model available for ${zone}/${batteryMode} (only Hybrid models exist)`;
    return status;
  }

  const wasm = getWasmModule();
  if (!wasm) {
    status.error = 'WASM module not available';
    return status;
  }

  // Check if already loaded
  if (isModelLoaded(zone, batteryMode)) {
    status.loaded = true;
    return status;
  }

  // Try to load
  status.loading = true;
  try {
    await loadModel(zone, batteryMode);
    status.loaded = true;
    status.loading = false;
  } catch (err) {
    status.loading = false;
    status.error = err instanceof Error ? err.message : String(err);
    console.warn(
      `[ModelLoader] Model not available for ${zone}/${batteryMode}, optimizer will use greedy fallback:`,
      err
    );
  }

  return status;
}

/**
 * Get available models list
 *
 * Note: This returns the expected models based on zones, not what's actually on disk.
 * Use this for UI purposes (e.g., showing which zones have models).
 */
export function getAvailableZones(): string[] {
  // These are the zones that have models generated (13 US zones)
  return [
    'California',
    'Texas',
    'Florida',
    'New York',
    'New England',
    'Northwest',
    'Southwest',
    'Southeast',
    'Midwest',
    'Mid-Atlantic',
    'Mountain',
    'Plains',
    'Delta',
  ];
}

/**
 * Check if models are available for a zone and battery mode
 *
 * Note: Currently we only have Hybrid models generated.
 * Models for Default and PeakShaver modes are not available.
 */
export function hasModel(zone: string, batteryMode?: BatteryMode): boolean {
  // Only Hybrid mode models are available
  if (batteryMode !== undefined && batteryMode !== BatteryMode.Hybrid) {
    console.log(`[ModelLoader] hasModel(${zone}, ${batteryMode}) = false (not Hybrid mode)`);
    return false;
  }
  const availableZones = getAvailableZones().map((z) => z.toLowerCase());
  const result = availableZones.includes(zone.toLowerCase());
  console.log(`[ModelLoader] hasModel(${zone}, ${batteryMode}) = ${result}`);
  return result;
}
