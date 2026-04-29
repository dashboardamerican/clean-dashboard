/**
 * Web Worker for WASM Simulations
 *
 * Runs in a separate thread to avoid blocking the main UI.
 * Loads WASM module and processes batch evaluation requests.
 * Supports model loading for faster optimization sweeps.
 */

// Types for communication
interface Portfolio {
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
}

interface EvalResult {
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
  lcoe: number;
  clean_match: number;
}

interface ZoneProfiles {
  solar: number[];
  wind: number[];
  load: number[];
}

interface CostParams {
  [key: string]: number | boolean | string;
}

interface OptimizerConfig {
  [key: string]: number | boolean | string;
}

interface SweepPoint {
  target: number;
  achieved: number;
  solar: number;
  wind: number;
  storage: number;
  clean_firm: number;
  lcoe: number;
  solar_lcoe: number;
  wind_lcoe: number;
  storage_lcoe: number;
  clean_firm_lcoe: number;
  gas_lcoe: number;
  gas_capacity: number;
  success: boolean;
}

interface SweepResult {
  points: SweepPoint[];
  elapsed_ms: number;
}

type BatteryMode = 'Default' | 'PeakShaver' | 'Hybrid' | 'LimitedForecast';

type WorkerRequest = {
  type: 'init';
  wasmPath: string;
} | {
  type: 'load_model';
  zone: string;
  batteryMode: BatteryMode;
  bytes: ArrayBuffer;
} | {
  type: 'sweep';
  zone: string;
  targets: number[];
  profiles: ZoneProfiles;
  costs: CostParams;
  config: OptimizerConfig;
  mode: BatteryMode;
} | {
  type: 'evaluate';
  portfolios: Portfolio[];
  profiles: ZoneProfiles;
  costs: CostParams;
  config: OptimizerConfig;
  mode: BatteryMode;
};

// WASM module state
let wasmModule: any = null;
let isInitialized = false;

function toWasmBatteryMode(mode: BatteryMode): any {
  switch (mode) {
    case 'Default':
      return wasmModule.battery_mode_default();
    case 'PeakShaver':
      return wasmModule.battery_mode_peak_shaver();
    case 'Hybrid':
      return wasmModule.battery_mode_hybrid();
    case 'LimitedForecast':
      return wasmModule.battery_mode_limited_forecast();
    default:
      return wasmModule.battery_mode_default();
  }
}

/**
 * Initialize WASM module
 */
async function initWasm(): Promise<void> {
  if (isInitialized) return;

  try {
    // Dynamic import of WASM bindings
    // Path is relative to this worker file in src/lib/
    const wasm = await import('./wasm/pkg/energy_simulator.js');
    await wasm.default();
    wasmModule = wasm;
    isInitialized = true;
  } catch (e) {
    console.error('Failed to load WASM in worker:', e);
    throw e;
  }
}

/**
 * Load a model into the worker's WASM cache
 */
function loadModel(zone: string, batteryMode: BatteryMode, bytes: ArrayBuffer): void {
  if (!wasmModule) {
    throw new Error('WASM not initialized');
  }

  const batteryModeEnum = toWasmBatteryMode(batteryMode);

  // Convert ArrayBuffer to Uint8Array for WASM
  const uint8Array = new Uint8Array(bytes);
  wasmModule.wasm_load_model(zone, batteryModeEnum, uint8Array);
}

/**
 * Run optimizer sweep in worker
 */
function runSweep(
  zone: string,
  targets: number[],
  profiles: ZoneProfiles,
  costs: CostParams,
  config: OptimizerConfig,
  mode: BatteryMode
): SweepResult {
  if (!wasmModule) {
    throw new Error('WASM not initialized');
  }

  const batteryMode = toWasmBatteryMode(mode);

  // Check if model is loaded for this zone
  const modelLoaded = wasmModule.wasm_is_model_loaded?.(zone, batteryMode) || false;

  // Use model-accelerated sweep if available
  if (modelLoaded && wasmModule.optimize_sweep_with_model) {
    return wasmModule.optimize_sweep_with_model(
      zone,
      new Float64Array(targets),
      new Float64Array(profiles.solar),
      new Float64Array(profiles.wind),
      new Float64Array(profiles.load),
      costs,
      config,
      batteryMode
    );
  } else {
    return wasmModule.run_optimizer_sweep(
      new Float64Array(targets),
      new Float64Array(profiles.solar),
      new Float64Array(profiles.wind),
      new Float64Array(profiles.load),
      costs,
      config,
      batteryMode
    );
  }
}

/**
 * Evaluate a batch of portfolios
 */
function evaluateBatch(
  portfolios: Portfolio[],
  profiles: ZoneProfiles,
  costs: CostParams,
  config: OptimizerConfig,
  mode: BatteryMode
): EvalResult[] {
  if (!wasmModule) {
    throw new Error('WASM not initialized');
  }

  const batteryMode = toWasmBatteryMode(mode);

  // Call the batch evaluation function
  const results = wasmModule.evaluate_batch(
    portfolios,
    profiles.solar,
    profiles.wind,
    profiles.load,
    costs,
    batteryMode,
    config,
  );

  return results;
}

/**
 * Handle messages from main thread
 */
self.onmessage = async (e: MessageEvent<WorkerRequest>) => {
  try {
    switch (e.data.type) {
      case 'init':
        await initWasm();
        self.postMessage({ type: 'ready' });
        break;

      case 'load_model': {
        // Ensure WASM is initialized
        if (!isInitialized) {
          await initWasm();
        }

        const { zone, batteryMode, bytes } = e.data;
        loadModel(zone, batteryMode, bytes);
        self.postMessage({ type: 'model_loaded', zone, batteryMode });
        break;
      }

      case 'sweep': {
        // Ensure WASM is initialized
        if (!isInitialized) {
          await initWasm();
        }

        const { zone, targets, profiles, costs, config, mode } = e.data;
        const sweepResult = runSweep(zone, targets, profiles, costs, config, mode);

        self.postMessage({
          type: 'sweep_results',
          result: sweepResult,
        });
        break;
      }

      case 'evaluate': {
        // Ensure WASM is initialized
        if (!isInitialized) {
          await initWasm();
        }

        const { portfolios, profiles, costs, config, mode } = e.data;
        const results = evaluateBatch(portfolios, profiles, costs, config, mode);

        self.postMessage({
          type: 'results',
          results,
        });
        break;
      }

      default:
        console.warn('Unknown message type:', (e.data as any).type);
    }
  } catch (error) {
    const message = error instanceof Error ? error.message : String(error);
    self.postMessage({
      type: 'error',
      error: message,
    });
  }
};

// Auto-initialize on load
initWasm()
  .then(() => {
    self.postMessage({ type: 'ready' });
  })
  .catch((e) => {
    self.postMessage({
      type: 'error',
      error: `WASM initialization failed: ${e.message}`,
    });
  });
