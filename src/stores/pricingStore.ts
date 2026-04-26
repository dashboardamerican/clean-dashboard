import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import {
  PricingMethod,
  PricingResult,
  OrdcConfig,
  DEFAULT_ORDC_CONFIG,
} from '../types'
import { useSimulationStore } from './simulationStore'
import { useSettingsStore } from './settingsStore'
import { useElccStore } from './elccStore'
import { serializeCostParams, serializePricingMethod } from '../lib/wasmSerde'

interface PricingState {
  // Configuration
  pricingMethod: PricingMethod;
  ordcConfig: OrdcConfig;

  // Results
  pricingResult: PricingResult | null;
  isCalculating: boolean;
  error: string | null;

  // Actions
  setPricingMethod: (method: PricingMethod) => void;
  setOrdcConfig: (config: Partial<OrdcConfig>) => void;
  calculatePrices: () => Promise<void>;
  clearResults: () => void;
}

// Get WASM module from global
function getWasmModule(): any {
  return (window as any).__wasmModule || null;
}

export const usePricingStore = create<PricingState>()(
  immer((set, get) => ({
    pricingMethod: PricingMethod.ScarcityBased,
    ordcConfig: { ...DEFAULT_ORDC_CONFIG },
    pricingResult: null,
    isCalculating: false,
    error: null,

    setPricingMethod: (method) => {
      set((state) => {
        state.pricingMethod = method;
      });
    },

    setOrdcConfig: (config) => {
      set((state) => {
        Object.assign(state.ordcConfig, config);
      });
    },

    calculatePrices: async () => {
      const wasm = getWasmModule();
      if (!wasm) {
        set((state) => {
          state.error = 'WASM module not loaded';
        });
        return;
      }

      const simStore = useSimulationStore.getState();
      const { simulationResult, lcoeResult, config, loadProfile } = simStore;

      if (!simulationResult || !lcoeResult) {
        set((state) => {
          state.error = 'Run simulation first';
        });
        return;
      }

      set((state) => {
        state.isCalculating = true;
        state.error = null;
      });

      try {
        const { pricingMethod, ordcConfig } = get();
        const costs = useSettingsStore.getState().costs;
        const elccResult = useElccStore.getState().elccResult;

        const wasmCosts = serializeCostParams(costs);
        const pricingMethodStr = serializePricingMethod(pricingMethod);

        const result: PricingResult = wasm.compute_prices(
          simulationResult,
          wasmCosts,
          lcoeResult.total_lcoe,
          pricingMethodStr,
          new Float64Array(loadProfile),
          pricingMethod === PricingMethod.Ordc ? ordcConfig : null,
          elccResult,
          config.solar_capacity,
          config.wind_capacity,
          config.storage_capacity,
          config.clean_firm_capacity,
        );

        set((state) => {
          state.pricingResult = result;
          state.isCalculating = false;
        });
      } catch (error) {
        console.error('Pricing calculation error:', error);
        set((state) => {
          state.error = error instanceof Error ? error.message : String(error);
          state.isCalculating = false;
        });
      }
    },

    clearResults: () => {
      set((state) => {
        state.pricingResult = null;
        state.error = null;
      });
    },
  }))
);
