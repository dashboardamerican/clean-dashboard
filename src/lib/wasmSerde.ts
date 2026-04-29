import {
  BatteryMode,
  CostParams,
  OptimizerConfig,
  PricingMethod,
  SimulationConfig,
} from '../types';

const BATTERY_MODE_NAMES = ['Default', 'PeakShaver', 'Hybrid', 'LimitedForecast'] as const;
const DEPRECIATION_METHOD_NAMES = ['Macrs5', 'Macrs15', 'StraightLine'] as const;
const PRICING_METHOD_NAMES = [
  'ScarcityBased',
  'MarginalCost',
  'Ordc',
  'MarginalPlusCapacity',
] as const;

export function serializeBatteryMode(mode: BatteryMode): string {
  return BATTERY_MODE_NAMES[mode] ?? 'Default';
}

export function serializeSimulationConfig(config: SimulationConfig): Record<string, number | string> {
  return {
    ...config,
    battery_mode: serializeBatteryMode(config.battery_mode),
  };
}

export function serializeCostParams(
  costs: CostParams
): Record<string, number | string | boolean> {
  return {
    ...costs,
    depreciation_method: DEPRECIATION_METHOD_NAMES[costs.depreciation_method] ?? 'Macrs5',
    project_lifetime: Math.floor(costs.project_lifetime),
    solar_lifetime: Math.floor(costs.solar_lifetime),
    wind_lifetime: Math.floor(costs.wind_lifetime),
    storage_lifetime: Math.floor(costs.storage_lifetime),
    clean_firm_lifetime: Math.floor(costs.clean_firm_lifetime),
    gas_lifetime: Math.floor(costs.gas_lifetime),
  };
}

export function serializePricingMethod(method: PricingMethod): string {
  return PRICING_METHOD_NAMES[method] ?? 'ScarcityBased';
}

export function withOptimizerRuntimeConfig(
  optimizerConfig: OptimizerConfig,
  simulationConfig: Pick<SimulationConfig, 'battery_efficiency' | 'max_demand_response'>
): OptimizerConfig {
  return {
    ...optimizerConfig,
    battery_efficiency: simulationConfig.battery_efficiency,
    max_demand_response: simulationConfig.max_demand_response,
  };
}
