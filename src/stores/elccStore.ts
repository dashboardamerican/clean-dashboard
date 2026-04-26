import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import {
  ElccResult,
  ElccMethod,
} from '../types'
import { useSimulationStore } from './simulationStore'
import { serializeBatteryMode } from '../lib/wasmSerde'

interface ElccState {
  // Results
  elccResult: ElccResult | null;
  displayMethod: ElccMethod;
  isCalculating: boolean;
  error: string | null;

  // Actions
  setDisplayMethod: (method: ElccMethod) => void;
  calculateElcc: () => Promise<void>;
  clearResults: () => void;
}

// Get WASM module from global
function getWasmModule(): any {
  return (window as any).__wasmModule || null;
}

export const useElccStore = create<ElccState>()(
  immer((set) => ({
    elccResult: null,
    displayMethod: ElccMethod.Delta,
    isCalculating: false,
    error: null,

    setDisplayMethod: (method) => {
      set((state) => {
        state.displayMethod = method;
      });
    },

    calculateElcc: async () => {
      const wasm = getWasmModule();
      if (!wasm) {
        set((state) => {
          state.error = 'WASM module not loaded';
        });
        return;
      }

      set((state) => {
        state.isCalculating = true;
        state.error = null;
      });

      try {
        const simStore = useSimulationStore.getState();
        const { config, solarProfile, windProfile, loadProfile } = simStore;

        const result: ElccResult = wasm.calculate_elcc_metrics(
          config.solar_capacity,
          config.wind_capacity,
          config.storage_capacity,
          config.clean_firm_capacity,
          new Float64Array(solarProfile),
          new Float64Array(windProfile),
          new Float64Array(loadProfile),
          serializeBatteryMode(config.battery_mode),
          config.battery_efficiency,
          config.max_demand_response,
        );

        set((state) => {
          state.elccResult = result;
          state.isCalculating = false;
        });
      } catch (error) {
        console.error('ELCC calculation error:', error);
        set((state) => {
          state.error = error instanceof Error ? error.message : String(error);
          state.isCalculating = false;
        });
      }
    },

    clearResults: () => {
      set((state) => {
        state.elccResult = null;
        state.error = null;
      });
    },
  }))
);
