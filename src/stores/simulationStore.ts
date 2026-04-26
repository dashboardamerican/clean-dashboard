import { create } from 'zustand'
import { immer } from 'zustand/middleware/immer'
import {
  SimulationConfig,
  SimulationResult,
  LcoeResult,
  CombinedResult,
  BatteryMode,
  DEFAULT_SIMULATION_CONFIG,
  ZoneName,
  HOURS_PER_YEAR,
} from '../types'
import { preloadModel } from '../lib/modelLoader'
import { serializeCostParams, serializeSimulationConfig } from '../lib/wasmSerde'

// Zone data cache
let zoneDataCache: Record<string, { solar: number[]; wind: number[]; load: number[] }> | null = null;

// Load zone data from JSON file
async function loadZoneData(zoneName: ZoneName): Promise<{
  solar: number[];
  wind: number[];
  load: number[];
}> {
  // Use cached data if available
  if (zoneDataCache && zoneDataCache[zoneName]) {
    return zoneDataCache[zoneName];
  }

  try {
    // Fetch zone data from public directory
    const response = await fetch(`${import.meta.env.BASE_URL}data/zones.json`);
    if (!response.ok) {
      throw new Error(`Failed to load zone data: ${response.statusText}`);
    }

    const allZones = await response.json();
    zoneDataCache = allZones;

    if (allZones[zoneName]) {
      return allZones[zoneName];
    }
    throw new Error(`Zone ${zoneName} not found in data`);
  } catch (err) {
    console.warn('Failed to load zone data, using synthetic profiles:', err);
    return generateSyntheticProfiles();
  }
}

// Generate synthetic profiles for fallback
function generateSyntheticProfiles(): {
  solar: number[];
  wind: number[];
  load: number[];
} {
  const solar: number[] = [];
  const wind: number[] = [];
  const load: number[] = [];

  for (let hour = 0; hour < HOURS_PER_YEAR; hour++) {
    const dayOfYear = Math.floor(hour / 24);
    const hourOfDay = hour % 24;

    // Solar: peaks midday, seasonal variation
    const solarSeasonal = 0.7 + 0.3 * Math.sin((2 * Math.PI * dayOfYear) / 365);
    const solarDaily =
      hourOfDay >= 6 && hourOfDay <= 18
        ? Math.sin((Math.PI * (hourOfDay - 6)) / 12)
        : 0;
    solar.push(solarSeasonal * solarDaily * 0.8);

    // Wind: more variable, higher at night
    const windBase = 0.3 + 0.15 * Math.sin((2 * Math.PI * dayOfYear) / 365);
    const windDaily = 0.2 + 0.1 * Math.cos((2 * Math.PI * hourOfDay) / 24);
    const windRandom = 0.9 + 0.2 * Math.random();
    wind.push(Math.min(1, windBase * (1 + windDaily) * windRandom));

    // Load: peaks in morning and evening
    const loadSeasonal = 90 + 20 * Math.cos((2 * Math.PI * dayOfYear) / 365);
    const loadDaily =
      1 +
      0.2 * Math.sin((2 * Math.PI * (hourOfDay - 6)) / 24) +
      0.1 * Math.sin((4 * Math.PI * hourOfDay) / 24);
    load.push(loadSeasonal * loadDaily);
  }

  return { solar, wind, load };
}

interface SimulationState {
  // Configuration
  config: SimulationConfig;
  zone: ZoneName;
  week: number;

  // Profile data
  solarProfile: number[];
  windProfile: number[];
  loadProfile: number[];

  // Results
  simulationResult: SimulationResult | null;
  lcoeResult: LcoeResult | null;
  isRunning: boolean;
  error: string | null;

  // Zone data loading state
  zoneDataLoaded: boolean;

  // Actions
  setConfig: (config: Partial<SimulationConfig>) => void;
  setZone: (zone: ZoneName) => Promise<void>;
  setWeek: (week: number) => void;
  setBatteryMode: (mode: BatteryMode) => void;
  runSimulation: () => Promise<void>;
  resetToDefaults: () => void;
  loadInitialZoneData: () => Promise<void>;
  applyOptimizerResult: (result: {
    solar: number;
    wind: number;
    storage: number;
    cleanFirm: number;
  }) => void;
}

// Get WASM module from global
function getWasmModule(): typeof import('../lib/wasm/pkg') | null {
  return (window as any).__wasmModule || null;
}

export const useSimulationStore = create<SimulationState>()(
  immer((set, get) => {
    // Start with synthetic profiles, will be replaced with real data
    const { solar, wind, load } = generateSyntheticProfiles();

    return {
      // Initial state
      config: { ...DEFAULT_SIMULATION_CONFIG },
      zone: 'California',
      week: 1,
      solarProfile: solar,
      windProfile: wind,
      loadProfile: load,
      simulationResult: null,
      lcoeResult: null,
      isRunning: false,
      error: null,
      zoneDataLoaded: false,

      // Actions
      setConfig: (newConfig) => {
        set((state) => {
          Object.assign(state.config, newConfig);
        });
      },

      setZone: async (zone) => {
        try {
          const zoneData = await loadZoneData(zone);
          const batteryMode = get().config.battery_mode;
          set((state) => {
            state.zone = zone;
            state.solarProfile = zoneData.solar;
            state.windProfile = zoneData.wind;
            state.loadProfile = zoneData.load;
          });

          // Preload model for new zone in background (non-blocking)
          preloadModel(zone, batteryMode);

          // Re-run simulation with new zone data
          await get().runSimulation();
        } catch (err) {
          console.error('Failed to load zone data:', err);
          set((state) => {
            state.zone = zone;
            state.error = `Failed to load data for ${zone}`;
          });
        }
      },

      loadInitialZoneData: async () => {
        const state = get();
        console.log('loadInitialZoneData called, zoneDataLoaded:', state.zoneDataLoaded);
        if (state.zoneDataLoaded) return;

        try {
          console.log('Loading zone data for:', state.zone);
          const zoneData = await loadZoneData(state.zone);
          console.log('Zone data loaded:', {
            zone: state.zone,
            solarLength: zoneData.solar.length,
            windLength: zoneData.wind.length,
            loadLength: zoneData.load.length,
            sampleLoad: zoneData.load.slice(0, 5),
          });
          set((s) => {
            s.solarProfile = zoneData.solar;
            s.windProfile = zoneData.wind;
            s.loadProfile = zoneData.load;
            s.zoneDataLoaded = true;
          });

          // Preload model for initial zone in background (non-blocking)
          preloadModel(state.zone, state.config.battery_mode);
        } catch (err) {
          console.error('Failed to load initial zone data:', err);
          set((s) => {
            s.zoneDataLoaded = true; // Mark as loaded even on error to prevent retry loop
          });
        }
      },

      setWeek: (week) => {
        set((state) => {
          state.week = Math.max(1, Math.min(52, week));
        });
      },

      setBatteryMode: (mode) => {
        set((state) => {
          state.config.battery_mode = mode;
        });
      },

      runSimulation: async () => {
        const wasm = getWasmModule();
        if (!wasm) {
          console.error('WASM module not loaded');
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
          const state = get();
          const costs = (await import('./settingsStore')).useSettingsStore.getState().costs;

          // Convert arrays to Float64Array for WASM
          const solarFloat = new Float64Array(state.solarProfile);
          const windFloat = new Float64Array(state.windProfile);
          const loadFloat = new Float64Array(state.loadProfile);

          const wasmConfig = serializeSimulationConfig(state.config);

          // Debug: log inputs
          console.log('Simulation inputs:', {
            config: wasmConfig,
            profileLengths: {
              solar: solarFloat.length,
              wind: windFloat.length,
              load: loadFloat.length,
            },
            sampleLoad: loadFloat.slice(0, 5),
          });

          const wasmCosts = serializeCostParams(costs);

          const result: CombinedResult = wasm.simulate_and_calculate_lcoe(
            wasmConfig,
            solarFloat,
            windFloat,
            loadFloat,
            wasmCosts
          );

          // Debug: log outputs including battery data
          const sim = result.simulation;
          const totalCharge = sim?.battery_charge?.reduce((a: number, b: number) => a + b, 0) || 0;
          const totalDischarge = sim?.battery_discharge?.reduce((a: number, b: number) => a + b, 0) || 0;
          const totalGasCharge = sim?.gas_for_charging?.reduce((a: number, b: number) => a + b, 0) || 0;
          const chargeHours = sim?.battery_charge?.filter((x: number) => x > 0.01).length || 0;
          const dischargeHours = sim?.battery_discharge?.filter((x: number) => x > 0.01).length || 0;
          console.log('Simulation result:', {
            clean_match_pct: sim?.clean_match_pct,
            peak_gas: sim?.peak_gas,
            total_lcoe: result.lcoe?.total_lcoe,
            annual_load: sim?.annual_load,
            battery: {
              totalCharge: totalCharge.toFixed(0),
              totalDischarge: totalDischarge.toFixed(0),
              totalGasCharge: totalGasCharge.toFixed(0),
              chargeHours,
              dischargeHours,
            },
          });

          set((s) => {
            s.simulationResult = result.simulation;
            s.lcoeResult = result.lcoe;
            s.isRunning = false;
          });
        } catch (error) {
          console.error('Simulation error:', error);
          set((s) => {
            s.error = error instanceof Error ? error.message : String(error);
            s.isRunning = false;
          });
        }
      },

      resetToDefaults: () => {
        set((state) => {
          state.config = { ...DEFAULT_SIMULATION_CONFIG };
          state.simulationResult = null;
          state.lcoeResult = null;
          state.error = null;
        });
      },

      applyOptimizerResult: (result) => {
        set((state) => {
          state.config.solar_capacity = result.solar;
          state.config.wind_capacity = result.wind;
          state.config.storage_capacity = result.storage;
          state.config.clean_firm_capacity = result.cleanFirm;
        });
      },
    };
  })
);

// Register callback so cost changes trigger simulation re-run
import { setOnCostsChangeCallback } from './settingsStore';
setOnCostsChangeCallback(() => {
  useSimulationStore.getState().runSimulation();
});
