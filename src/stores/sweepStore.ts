import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import {
  SweepResult,
  CostSweepResult,
  CostSweepParam,
  DEFAULT_OPTIMIZER_CONFIG,
} from '../types'
import { useSimulationStore } from './simulationStore'
import { useSettingsStore } from './settingsStore'
import { ensureModelLoaded, fetchModel } from '../lib/modelLoader'
import { getWorkerPool } from '../lib/worker-pool'
import { serializeBatteryMode, serializeCostParams, withOptimizerRuntimeConfig } from '../lib/wasmSerde'

// Default sweep targets (0% to 100% in 10% increments + high-end detail)
const DEFAULT_TARGETS = [0, 10, 20, 30, 40, 50, 60, 70, 80, 85, 90, 95, 98, 99, 100];

// Fine-grained targets (5% increments + extra detail at high end)
const FINE_TARGETS = [0, 5, 10, 15, 20, 25, 30, 35, 40, 45, 50, 55, 60, 65, 70, 75, 80, 85, 90, 95, 98, 99, 100];

interface SweepState {
  // Optimizer sweep
  sweepResult: SweepResult | null;
  sweepTargets: number[];
  useFineTargets: boolean;

  // Cost sweep
  costSweepResult: CostSweepResult | null;
  costSweepParam: CostSweepParam;
  costSweepRange: [number, number];
  costSweepTarget: number;
  costSweepSteps: number;

  // Comparison
  savedSweep: SweepResult | null;
  savedLabel: string;

  // State
  isRunning: boolean;
  error: string | null;

  // Worker settings
  useWorkers: boolean;

  // Actions
  setUseFineTargets: (fine: boolean) => void;
  setUseWorkers: (use: boolean) => void;
  setCostSweepParam: (param: CostSweepParam) => void;
  setCostSweepRange: (range: [number, number]) => void;
  setCostSweepTarget: (target: number) => void;
  setCostSweepSteps: (steps: number) => void;
  runOptimizerSweep: () => Promise<void>;
  runCostSweep: () => Promise<void>;
  saveAsComparison: (label: string) => void;
  clearSavedComparison: () => void;
  clearResults: () => void;
}

// Get WASM module from global
function getWasmModule(): any {
  return (window as any).__wasmModule || null;
}

export const useSweepStore = create<SweepState>()(
  immer((set, get) => ({
    // Initial state
    sweepResult: null,
    sweepTargets: DEFAULT_TARGETS,
    useFineTargets: false,
    costSweepResult: null,
    costSweepParam: 'clean_firm_capex',
    costSweepRange: [1000, 12000],
    costSweepTarget: 80,
    costSweepSteps: 12,
    savedSweep: null,
    savedLabel: '',
    isRunning: false,
    error: null,
    useWorkers: true, // Enable parallel sweep across web workers

    setUseFineTargets: (fine) => {
      set((state) => {
        state.useFineTargets = fine;
        state.sweepTargets = fine ? FINE_TARGETS : DEFAULT_TARGETS;
      });
    },

    setUseWorkers: (use) => {
      set((state) => {
        state.useWorkers = use;
      });
    },

    setCostSweepParam: (param) => {
      set((state) => {
        state.costSweepParam = param;
        // Set default ranges based on parameter
        switch (param) {
          case 'solar_capex':
            state.costSweepRange = [500, 2000];
            break;
          case 'wind_capex':
            state.costSweepRange = [800, 2500];
            break;
          case 'storage_capex':
            state.costSweepRange = [100, 600];
            break;
          case 'clean_firm_capex':
            state.costSweepRange = [1000, 12000];
            break;
          case 'gas_capex':
            state.costSweepRange = [500, 2000];
            break;
          case 'gas_price':
            state.costSweepRange = [2, 14];
            break;
          case 'solar_itc':
          case 'wind_itc':
          case 'storage_itc':
          case 'clean_firm_itc':
            state.costSweepRange = [0, 50];
            break;
          case 'discount_rate':
            state.costSweepRange = [3, 12];
            break;
        }
      });
    },

    setCostSweepRange: (range) => {
      set((state) => {
        state.costSweepRange = range;
      });
    },

    setCostSweepTarget: (target) => {
      set((state) => {
        state.costSweepTarget = target;
      });
    },

    setCostSweepSteps: (steps) => {
      set((state) => {
        state.costSweepSteps = steps;
      });
    },

    runOptimizerSweep: async () => {
      const wasm = getWasmModule();
      if (!wasm) {
        set((state) => {
          state.error = 'WASM module not loaded';
        });
        return;
      }

      set((state) => {
        state.isRunning = true;
        state.error = null;
      });

      try {
        const simStore = useSimulationStore.getState();
        const { solarProfile, windProfile, loadProfile, config, zone } = simStore;
        const costs = useSettingsStore.getState().costs;
        const { sweepTargets, useWorkers } = get();

        const wasmCosts = serializeCostParams(costs);
        const batteryModeStr = serializeBatteryMode(config.battery_mode);
        const batteryMode = config.battery_mode;
        const optimizerConfig = withOptimizerRuntimeConfig(DEFAULT_OPTIMIZER_CONFIG, config);

        // Try to load model for faster optimization (non-blocking on failure)
        const modelStatus = await ensureModelLoaded(zone, config.battery_mode);
        console.log(`[OptimizerSweep] Model for ${zone}/${batteryModeStr}: loaded=${modelStatus.loaded}`);

        // Capture timing on frontend (WASM can't use std::time::Instant)
        const startTime = performance.now();

        let result: SweepResult;

        // Try worker-based execution if enabled
        const pool = getWorkerPool();

        // Wait for pool to be ready (with timeout)
        if (useWorkers && !pool.isPoolReady()) {
          console.log('[OptimizerSweep] Waiting for worker pool to initialize...');
          await pool.waitForReady(5000); // 5 second timeout
        }

        console.log(`[OptimizerSweep] useWorkers=${useWorkers}, poolReady=${pool.isPoolReady()}, workerCount=${pool.getWorkerCount()}`);
        if (useWorkers && pool.isPoolReady()) {
          console.log(`[OptimizerSweep] Using ${pool.getWorkerCount()} workers for parallel sweep`);

          // Load model in workers if available
          if (modelStatus.loaded) {
            try {
              const modelBytes = await fetchModel(zone, config.battery_mode);
              // Create a proper ArrayBuffer copy for transfer
              const buffer = modelBytes.buffer.slice(modelBytes.byteOffset, modelBytes.byteOffset + modelBytes.byteLength);
              await pool.loadModelInWorkers(zone, config.battery_mode, buffer as ArrayBuffer);
            } catch (err) {
              console.warn('[OptimizerSweep] Failed to load model in workers, continuing without:', err);
            }
          }

          // Run parallel sweep
          result = await pool.runParallelSweep(
            zone,
            sweepTargets,
            { solar: solarProfile, wind: windProfile, load: loadProfile },
            wasmCosts,
            optimizerConfig,
            config.battery_mode,
          );
        } else {
          // Main thread execution (fallback or when workers disabled)
          if (modelStatus.loaded && wasm.optimize_sweep_with_model) {
            result = wasm.optimize_sweep_with_model(
              zone,
              new Float64Array(sweepTargets),
              new Float64Array(solarProfile),
              new Float64Array(windProfile),
              new Float64Array(loadProfile),
              wasmCosts,
              optimizerConfig,
              batteryMode,
            );
          } else {
            result = wasm.run_optimizer_sweep(
              new Float64Array(sweepTargets),
              new Float64Array(solarProfile),
              new Float64Array(windProfile),
              new Float64Array(loadProfile),
              wasmCosts,
              optimizerConfig,
              batteryMode,
            );
          }
        }

        const elapsed_ms = performance.now() - startTime;

        // Update elapsed_ms with actual frontend timing
        const resultWithTiming: SweepResult = {
          ...result,
          elapsed_ms,
        };

        set((state) => {
          state.sweepResult = resultWithTiming;
          state.isRunning = false;
        });

        console.log(`Optimizer sweep completed in ${elapsed_ms.toFixed(0)}ms (model=${modelStatus.loaded}, workers=${useWorkers && pool.isPoolReady()})`);
      } catch (error) {
        console.error('Optimizer sweep error:', error);
        set((state) => {
          state.error = error instanceof Error ? error.message : String(error);
          state.isRunning = false;
        });
      }
    },

    runCostSweep: async () => {
      const wasm = getWasmModule();
      if (!wasm) {
        set((state) => {
          state.error = 'WASM module not loaded';
        });
        return;
      }

      set((state) => {
        state.isRunning = true;
        state.error = null;
      });

      try {
        const simStore = useSimulationStore.getState();
        const { solarProfile, windProfile, loadProfile, config, zone } = simStore;
        const costs = useSettingsStore.getState().costs;
        const { costSweepParam, costSweepRange, costSweepTarget, costSweepSteps } = get();

        const wasmCosts = serializeCostParams(costs);
        const batteryModeStr = serializeBatteryMode(config.battery_mode);
        const batteryMode = config.battery_mode;
        const optimizerConfig = withOptimizerRuntimeConfig(DEFAULT_OPTIMIZER_CONFIG, config);

        // Try to load model for faster optimization (non-blocking on failure)
        const modelStatus = await ensureModelLoaded(zone, config.battery_mode);
        console.log(`[CostSweep] Model for ${zone}/${batteryModeStr}: loaded=${modelStatus.loaded}`);

        // Capture timing on frontend (WASM can't use std::time::Instant)
        const startTime = performance.now();

        // Use model-accelerated sweep if model is loaded, otherwise fallback to standard sweep
        let result: CostSweepResult;
        if (modelStatus.loaded && wasm.run_cost_sweep_with_model) {
          result = wasm.run_cost_sweep_with_model(
            zone,
            costSweepTarget,
            costSweepParam,
            costSweepRange[0],
            costSweepRange[1],
            costSweepSteps,
            new Float64Array(solarProfile),
            new Float64Array(windProfile),
            new Float64Array(loadProfile),
            wasmCosts,
            optimizerConfig,
            batteryMode,
          );
        } else {
          result = wasm.run_cost_sweep(
            costSweepTarget,
            costSweepParam,
            costSweepRange[0],
            costSweepRange[1],
            costSweepSteps,
            new Float64Array(solarProfile),
            new Float64Array(windProfile),
            new Float64Array(loadProfile),
            wasmCosts,
            optimizerConfig,
            batteryMode,
          );
        }

        const elapsed_ms = performance.now() - startTime;

        // Update elapsed_ms with actual frontend timing
        const resultWithTiming: CostSweepResult = {
          ...result,
          elapsed_ms,
        };

        set((state) => {
          state.costSweepResult = resultWithTiming;
          state.isRunning = false;
        });

        console.log(`Cost sweep completed in ${elapsed_ms.toFixed(0)}ms (model=${modelStatus.loaded})`);
      } catch (error) {
        console.error('Cost sweep error:', error);
        set((state) => {
          state.error = error instanceof Error ? error.message : String(error);
          state.isRunning = false;
        });
      }
    },

    saveAsComparison: (label) => {
      const { sweepResult } = get();
      if (sweepResult) {
        set((state) => {
          state.savedSweep = sweepResult;
          state.savedLabel = label;
        });
      }
    },

    clearSavedComparison: () => {
      set((state) => {
        state.savedSweep = null;
        state.savedLabel = '';
      });
    },

    clearResults: () => {
      set((state) => {
        state.sweepResult = null;
        state.costSweepResult = null;
        state.error = null;
      });
    },
  }))
);
