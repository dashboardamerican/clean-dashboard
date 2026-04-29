import { useState, useEffect } from 'react';
import { BatteryMode, PricingMethod } from '../types';

// Type for the WASM module exports
export interface WasmModule {
  get_version: () => string;
  simulate: (
    config: any,
    solarProfile: Float64Array,
    windProfile: Float64Array,
    loadProfile: Float64Array
  ) => any;
  compute_lcoe: (
    simResult: any,
    solarCapacity: number,
    windCapacity: number,
    storageCapacity: number,
    cleanFirmCapacity: number,
    costs: any
  ) => any;
  simulate_and_calculate_lcoe: (
    config: any,
    solarProfile: Float64Array,
    windProfile: Float64Array,
    loadProfile: Float64Array,
    costs: any
  ) => any;
  optimize: (
    targetMatch: number,
    solarProfile: Float64Array,
    windProfile: Float64Array,
    loadProfile: Float64Array,
    costs: any,
    config: any,
    batteryMode: any
  ) => any;
  optimize_with_model: (
    zone: string,
    targetMatch: number,
    solarProfile: Float64Array,
    windProfile: Float64Array,
    loadProfile: Float64Array,
    costs: any,
    config: any,
    batteryMode: any
  ) => any;
  optimize_sweep_with_model: (
    zone: string,
    targets: Float64Array,
    solarProfile: Float64Array,
    windProfile: Float64Array,
    loadProfile: Float64Array,
    costs: any,
    config: any,
    batteryMode: any
  ) => any;
  get_default_costs: () => any;
  get_default_simulation_config: () => any;
  get_default_optimizer_config: () => any;
  battery_mode_default: () => any;
  battery_mode_peak_shaver: () => any;
  battery_mode_hybrid: () => any;
  battery_mode_limited_forecast: () => any;

  // Model cache functions
  wasm_load_model: (zone: string, batteryMode: any, bytes: Uint8Array) => void;
  wasm_is_model_loaded: (zone: string, batteryMode: any) => boolean;
  wasm_clear_models: () => void;
  wasm_loaded_models: () => Array<[string, number]>;
  wasm_cache_stats: () => { loaded: number; max: number };
  calculate_elcc_metrics: (
    solarCapacity: number,
    windCapacity: number,
    storageCapacity: number,
    cleanFirmCapacity: number,
    solarProfile: Float64Array,
    windProfile: Float64Array,
    loadProfile: Float64Array,
    batteryMode: any,
    batteryEfficiency: number,
    maxDemandResponse: number
  ) => any;
  compute_prices: (
    simResult: any,
    costs: any,
    lcoe: number,
    pricingMethod: any,
    loadProfile: Float64Array,
    ordcConfig: any,
    elccResult: any,
    solarCapacity: number,
    windCapacity: number,
    storageCapacity: number,
    cleanFirmCapacity: number
  ) => any;
}

interface UseWasmResult {
  wasmModule: WasmModule | null;
  loading: boolean;
  error: string | null;
}

// Global module storage for access outside React
declare global {
  interface Window {
    __wasmModule: WasmModule | null;
  }
}

export function useWasm(): UseWasmResult {
  const [wasmModule, setWasmModule] = useState<WasmModule | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    async function loadWasm() {
      try {
        // Dynamic import of the WASM module
        const wasm = await import('../lib/wasm/pkg');

        // Initialize the module - MUST call as function!
        await wasm.default();

        // Store globally for access outside React
        window.__wasmModule = wasm as unknown as WasmModule;

        setWasmModule(wasm as unknown as WasmModule);
        setLoading(false);

        console.log(`WASM module loaded. Version: ${wasm.get_version()}`);
      } catch (err) {
        console.error('Failed to load WASM module:', err);
        setError(err instanceof Error ? err.message : 'Failed to load WASM module');
        setLoading(false);

        // In development without WASM, create a mock
        if ((import.meta as any).env?.DEV) {
          console.warn('Creating mock WASM module for development');
          const mockModule = createMockWasmModule();
          window.__wasmModule = mockModule;
          setWasmModule(mockModule);
          setError(null);
        }
      }
    }

    loadWasm();
  }, []);

  return { wasmModule, loading, error };
}

// Mock WASM module for development/testing
function createMockWasmModule(): WasmModule {
  const HOURS = 8760;

  return {
    get_version: () => 'mock-0.1.0',

    simulate: (config, solarProfile, windProfile, loadProfile) => {
      // Simple mock simulation
      const solarArr = Array.from(solarProfile);
      const windArr = Array.from(windProfile);
      const loadArr = Array.from(loadProfile);
      const solar_out = solarArr.map((cf) => cf * config.solar_capacity);
      const wind_out = windArr.map((cf) => cf * config.wind_capacity);
      const gas_generation = loadArr.map((load, i) => {
        const clean = solar_out[i] + wind_out[i] + config.clean_firm_capacity;
        return Math.max(0, load - clean);
      });

      return {
        solar_out,
        wind_out,
        battery_charge: new Array(HOURS).fill(0),
        battery_discharge: new Array(HOURS).fill(0),
        gas_generation,
        curtailed: new Array(HOURS).fill(0),
        clean_delivered: solar_out.map((s, i) =>
          Math.min(loadArr[i], s + wind_out[i] + config.clean_firm_capacity)
        ),
        clean_firm_generation: new Array(HOURS).fill(config.clean_firm_capacity),
        demand_response: new Array(HOURS).fill(0),
        gas_for_charging: new Array(HOURS).fill(0),
        state_of_charge: new Array(HOURS).fill(0),
        annual_renewable_gen: solar_out.reduce((a, b) => a + b, 0) + wind_out.reduce((a, b) => a + b, 0),
        annual_load: loadArr.reduce((a, b) => a + b, 0),
        peak_gas: Math.max(...gas_generation),
        clean_match_pct: 50, // Placeholder
        total_curtailment: 0,
      };
    },

    compute_lcoe: () => ({
      total_lcoe: 50,
      solar_lcoe: 10,
      wind_lcoe: 15,
      storage_lcoe: 5,
      clean_firm_lcoe: 10,
      gas_lcoe: 10,
      ccs_lcoe: 0,
      pv_total_costs: 1000000,
      pv_total_energy: 20000,
      emissions_intensity: 200,
      direct_land_use: 250,
      total_land_use: 500,
      solar_breakdown: { capex: 0, fixed_om: 0, var_om: 0, fuel: 0, itc_benefit: 0, tax_shield: 0, total: 0 },
      wind_breakdown: { capex: 0, fixed_om: 0, var_om: 0, fuel: 0, itc_benefit: 0, tax_shield: 0, total: 0 },
      storage_breakdown: { capex: 0, fixed_om: 0, var_om: 0, fuel: 0, itc_benefit: 0, tax_shield: 0, total: 0 },
      clean_firm_breakdown: { capex: 0, fixed_om: 0, var_om: 0, fuel: 0, itc_benefit: 0, tax_shield: 0, total: 0 },
      gas_breakdown: { capex: 0, fixed_om: 0, var_om: 0, fuel: 0, itc_benefit: 0, tax_shield: 0, total: 0 },
      ccs_breakdown: { capex: 0, fixed_om: 0, var_om: 0, fuel: 0, itc_benefit: 0, tax_shield: 0, total: 0 },
    }),

    simulate_and_calculate_lcoe: (config, solarProfile, windProfile, loadProfile, costs) => {
      const simulation = createMockWasmModule().simulate(config, solarProfile, windProfile, loadProfile);
      const lcoe = createMockWasmModule().compute_lcoe(simulation, config.solar_capacity, config.wind_capacity, config.storage_capacity, config.clean_firm_capacity, costs);
      return { simulation, lcoe };
    },

    optimize: () => ({
      solar_capacity: 100,
      wind_capacity: 75,
      storage_capacity: 50,
      clean_firm_capacity: 20,
      achieved_clean_match: 80,
      lcoe: 55,
      num_evaluations: 150,
      success: true,
    }),

    optimize_with_model: () => ({
      solar_capacity: 100,
      wind_capacity: 75,
      storage_capacity: 50,
      clean_firm_capacity: 20,
      achieved_clean_match: 80,
      lcoe: 55,
      num_evaluations: 150,
      success: true,
    }),

    optimize_sweep_with_model: () => [],

    get_default_costs: () => ({}),
    get_default_simulation_config: () => ({}),
    get_default_optimizer_config: () => ({}),
    battery_mode_default: () => BatteryMode.Default,
    battery_mode_peak_shaver: () => BatteryMode.PeakShaver,
    battery_mode_hybrid: () => BatteryMode.Hybrid,
    battery_mode_limited_forecast: () => BatteryMode.LimitedForecast,

    // Mock model cache functions
    wasm_load_model: () => {},
    wasm_is_model_loaded: () => false,
    wasm_clear_models: () => {},
    wasm_loaded_models: () => [],
    wasm_cache_stats: () => ({ loaded: 0, max: 3 }),
    calculate_elcc_metrics: (solarCapacity, windCapacity, storageCapacity, cleanFirmCapacity) => ({
      solar: { first_in: 0, marginal: 0, contribution: 0, delta: 0 },
      wind: { first_in: 0, marginal: 0, contribution: 0, delta: 0 },
      storage: {
        first_in: storageCapacity > 0 ? 100 : 0,
        marginal: storageCapacity > 0 ? 100 : 0,
        contribution: storageCapacity > 0 ? 100 : 0,
        delta: storageCapacity > 0 ? 100 : 0,
      },
      clean_firm: {
        first_in: cleanFirmCapacity > 0 ? 100 : 0,
        marginal: cleanFirmCapacity > 0 ? 100 : 0,
        contribution: cleanFirmCapacity > 0 ? 100 : 0,
        delta: cleanFirmCapacity > 0 ? 100 : 0,
      },
      portfolio_elcc_mw: solarCapacity + windCapacity + storageCapacity + cleanFirmCapacity,
      diversity_benefit_mw: 0,
      baseline_peak_gas: 0,
      portfolio_peak_gas: 0,
    }),
    compute_prices: (_simResult, _costs, lcoe, _pricingMethod, loadProfile, _ordcConfig, elccResult, solarCapacity, windCapacity, storageCapacity, cleanFirmCapacity) => {
      const averagePrice = Number.isFinite(lcoe) ? lcoe : 0;
      const hourly_prices = Array.from(loadProfile, () => averagePrice);
      return {
        hourly_prices,
        average_price: averagePrice,
        peak_price: averagePrice,
        min_price: averagePrice,
        capacity_data: elccResult ? {
          qualified_capacity: {
            solar: solarCapacity,
            wind: windCapacity,
            storage: storageCapacity,
            clean_firm: cleanFirmCapacity,
            gas: 0,
          },
          annual_payments: {
            solar: solarCapacity * averagePrice,
            wind: windCapacity * averagePrice,
            storage: storageCapacity * averagePrice,
            clean_firm: cleanFirmCapacity * averagePrice,
            gas: 0,
          },
          elcc_percentages: {
            solar: elccResult.solar?.delta ?? 0,
            wind: elccResult.wind?.delta ?? 0,
            storage: elccResult.storage?.delta ?? 0,
            clean_firm: elccResult.clean_firm?.delta ?? 0,
            gas: 100,
          },
          clearing_price: averagePrice * 1000,
          adder_per_mwh: averagePrice,
        } : null,
        method: PricingMethod.ScarcityBased,
      };
    },
  };
}
