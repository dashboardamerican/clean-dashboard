import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import { persist } from 'zustand/middleware'
import { CostParams, DEFAULT_COSTS } from '../types'

// Preset scenarios
export const PRESETS = {
  default: { ...DEFAULT_COSTS },
  lowCostClean: {
    ...DEFAULT_COSTS,
    solar_capex: 700,
    wind_capex: 900,
    storage_capex: 200,
    clean_firm_capex: 3500,
    solar_itc: 0.30,
    wind_itc: 0.30,
    storage_itc: 0.30,
    clean_firm_itc: 0.40,
  },
  highCostClean: {
    ...DEFAULT_COSTS,
    solar_capex: 1300,
    wind_capex: 1500,
    storage_capex: 400,
    clean_firm_capex: 7000,
    solar_itc: 0,
    wind_itc: 0,
    storage_itc: 0,
    clean_firm_itc: 0,
  },
  highGasPrices: {
    ...DEFAULT_COSTS,
    gas_price: 8,
  },
  lowGasPrices: {
    ...DEFAULT_COSTS,
    gas_price: 2,
  },
} as const;

export type PresetName = keyof typeof PRESETS;

// Callback for when costs change - will be set by simulation store
let onCostsChangeCallback: (() => void) | null = null;

export function setOnCostsChangeCallback(callback: () => void) {
  onCostsChangeCallback = callback;
}

interface SettingsState {
  costs: CostParams;
  currentPreset: PresetName | 'custom';

  // Actions
  setCost: <K extends keyof CostParams>(key: K, value: CostParams[K]) => void;
  setCosts: (costs: Partial<CostParams>) => void;
  applyPreset: (preset: PresetName) => void;
  resetToDefaults: () => void;
}

export const useSettingsStore = create<SettingsState>()(
  persist(
    immer((set) => ({
      costs: { ...DEFAULT_COSTS },
      currentPreset: 'default' as PresetName | 'custom',

      setCost: (key, value) => {
        set((state) => {
          (state.costs as any)[key] = value;
          state.currentPreset = 'custom';
        });
        // Trigger simulation update
        if (onCostsChangeCallback) {
          onCostsChangeCallback();
        }
      },

      setCosts: (newCosts) => {
        set((state) => {
          Object.assign(state.costs, newCosts);
          state.currentPreset = 'custom';
        });
        // Trigger simulation update
        if (onCostsChangeCallback) {
          onCostsChangeCallback();
        }
      },

      applyPreset: (preset) => {
        set((state) => {
          state.costs = { ...PRESETS[preset] };
          state.currentPreset = preset;
        });
        // Trigger simulation update
        if (onCostsChangeCallback) {
          onCostsChangeCallback();
        }
      },

      resetToDefaults: () => {
        set((state) => {
          state.costs = { ...DEFAULT_COSTS };
          state.currentPreset = 'default';
        });
        // Trigger simulation update
        if (onCostsChangeCallback) {
          onCostsChangeCallback();
        }
      },
    })),
    {
      name: 'energy-simulator-settings',
      version: 3, // Increment when adding new fields to force migration
      partialize: (state) => ({ costs: state.costs, currentPreset: state.currentPreset }),
      // Merge persisted state with defaults to ensure new fields are present
      merge: (persistedState, currentState) => {
        const persisted = persistedState as Partial<SettingsState>;
        return {
          ...currentState,
          currentPreset: persisted.currentPreset ?? 'default',
          // Merge costs with defaults to ensure new fields exist
          costs: {
            ...DEFAULT_COSTS,
            ...(persisted.costs ?? {}),
          },
        };
      },
    }
  )
);
