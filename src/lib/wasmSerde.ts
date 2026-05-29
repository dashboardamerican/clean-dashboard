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

export function serializeSimulationConfig(
  config: SimulationConfig
): Record<string, number | string | boolean> {
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
  simulationConfig: Pick<
    SimulationConfig,
    | 'battery_efficiency'
    | 'max_demand_response'
    | 'limited_forecast_horizon_hours'
    | 'limited_forecast_commit_hours'
    | 'limited_forecast_renewable_error_pct'
    | 'limited_forecast_soc_reserve_pct'
    | 'limited_forecast_peak_value_mwh_per_mw'
    | 'limited_forecast_allow_gas_charging'
  >
): OptimizerConfig {
  return {
    ...optimizerConfig,
    battery_efficiency: simulationConfig.battery_efficiency,
    max_demand_response: simulationConfig.max_demand_response,
    limited_forecast_horizon_hours: simulationConfig.limited_forecast_horizon_hours,
    limited_forecast_commit_hours: simulationConfig.limited_forecast_commit_hours,
    limited_forecast_renewable_error_pct:
      simulationConfig.limited_forecast_renewable_error_pct,
    limited_forecast_soc_reserve_pct: simulationConfig.limited_forecast_soc_reserve_pct,
    limited_forecast_peak_value_mwh_per_mw:
      simulationConfig.limited_forecast_peak_value_mwh_per_mw,
    limited_forecast_allow_gas_charging: simulationConfig.limited_forecast_allow_gas_charging,
  };
}
