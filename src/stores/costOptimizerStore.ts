import { create } from 'zustand';
import { immer } from 'zustand/middleware/immer';
import {
  DEFAULT_OPTIMIZER_CONFIG,
  OptimizerConfig,
  OptimizerResult,
} from '../types';
import { runPortfolioOptimization } from '../features/optimizer/runPortfolioOptimization';
import { useSettingsStore } from './settingsStore';
import { useSimulationStore } from './simulationStore';

export type ControlMode = 'manualCapacity' | 'costOptimized';
export type OptimizerResourceKey = 'solar' | 'wind' | 'storage' | 'cleanFirm';

interface CostOptimizerState {
  mode: ControlMode;
  optimizerConfig: OptimizerConfig;
  result: OptimizerResult | null;
  isRunning: boolean;
  error: string | null;
  elapsedMs: number | null;
  usedModel: boolean;
  optimizerPath: 'v2-model' | 'v2' | null;

  setMode: (mode: ControlMode) => void;
  setTargetCleanMatch: (target: number) => void;
  setResourceEnabled: (resource: OptimizerResourceKey, enabled: boolean) => void;
  scheduleAutoOptimize: (delayMs?: number) => void;
  runAndApply: () => Promise<void>;
  clearResult: () => void;
}

let debounceTimer: ReturnType<typeof setTimeout> | null = null;
let runId = 0;

function getWasmModule(): any {
  return (window as any).__wasmModule || null;
}

function resourceConfigKey(resource: OptimizerResourceKey): keyof Pick<
  OptimizerConfig,
  'enable_solar' | 'enable_wind' | 'enable_storage' | 'enable_clean_firm'
> {
  switch (resource) {
    case 'solar':
      return 'enable_solar';
    case 'wind':
      return 'enable_wind';
    case 'storage':
      return 'enable_storage';
    case 'cleanFirm':
      return 'enable_clean_firm';
  }
}

export const useCostOptimizerStore = create<CostOptimizerState>()(
  immer((set, get) => ({
    mode: 'manualCapacity',
    optimizerConfig: { ...DEFAULT_OPTIMIZER_CONFIG },
    result: null,
    isRunning: false,
    error: null,
    elapsedMs: null,
    usedModel: false,
    optimizerPath: null,

    setMode: (mode) => {
      runId++;
      set((state) => {
        state.mode = mode;
        if (mode === 'manualCapacity') {
          state.isRunning = false;
        }
      });

      if (debounceTimer) {
        clearTimeout(debounceTimer);
        debounceTimer = null;
      }

    },

    setTargetCleanMatch: (target) => {
      set((state) => {
        state.optimizerConfig.target_clean_match = target;
      });
      get().scheduleAutoOptimize();
    },

    setResourceEnabled: (resource, enabled) => {
      const key = resourceConfigKey(resource);
      set((state) => {
        state.optimizerConfig[key] = enabled;
      });
      get().scheduleAutoOptimize();
    },

    scheduleAutoOptimize: (delayMs = 600) => {
      if (get().mode !== 'costOptimized') return;

      if (debounceTimer) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        debounceTimer = null;
        void get().runAndApply();
      }, delayMs);
    },

    runAndApply: async () => {
      const wasm = getWasmModule();
      if (!wasm) {
        set((state) => {
          state.error = 'WASM module not loaded';
          state.isRunning = false;
        });
        return;
      }

      const simStore = useSimulationStore.getState();
      if (!simStore.zoneDataLoaded) {
        set((state) => {
          state.error = 'Zone data is still loading';
          state.isRunning = false;
        });
        return;
      }

      const currentRunId = ++runId;
      set((state) => {
        state.isRunning = true;
        state.error = null;
      });

      await new Promise((resolve) => setTimeout(resolve, 0));
      if (currentRunId !== runId || get().mode !== 'costOptimized') return;

      try {
        const latestSimStore = useSimulationStore.getState();
        const costs = useSettingsStore.getState().costs;
        const optimization = await runPortfolioOptimization({
          wasm,
          zone: latestSimStore.zone,
          optimizerConfig: get().optimizerConfig,
          simulationConfig: latestSimStore.config,
          solarProfile: latestSimStore.solarProfile,
          windProfile: latestSimStore.windProfile,
          loadProfile: latestSimStore.loadProfile,
          costs,
          batteryMode: latestSimStore.config.battery_mode,
        });

        if (currentRunId !== runId) return;

        const targetMiss = Math.abs(
          optimization.result.achieved_clean_match - get().optimizerConfig.target_clean_match
        );
        const usableResult =
          Number.isFinite(optimization.result.solar_capacity) &&
          Number.isFinite(optimization.result.wind_capacity) &&
          Number.isFinite(optimization.result.storage_capacity) &&
          Number.isFinite(optimization.result.clean_firm_capacity);

        set((state) => {
          state.result = optimization.result;
          state.elapsedMs = optimization.elapsedMs;
          state.usedModel = optimization.usedModel;
          state.optimizerPath = optimization.optimizerPath;
          state.isRunning = false;
          state.error =
            optimization.result.success || targetMiss <= 0.75
              ? null
              : `Closest portfolio reached ${optimization.result.achieved_clean_match.toFixed(1)}% clean match`;
        });

        if (usableResult) {
          latestSimStore.applyOptimizerResult({
            solar: optimization.result.solar_capacity,
            wind: optimization.result.wind_capacity,
            storage: optimization.result.storage_capacity,
            cleanFirm: optimization.result.clean_firm_capacity,
          });
          await useSimulationStore.getState().runSimulation();
        }
      } catch (error) {
        if (currentRunId !== runId) return;
        set((state) => {
          state.error = error instanceof Error ? error.message : String(error);
          state.isRunning = false;
        });
      }
    },

    clearResult: () => {
      set((state) => {
        state.result = null;
        state.error = null;
        state.elapsedMs = null;
        state.usedModel = false;
        state.optimizerPath = null;
      });
    },
  }))
);
