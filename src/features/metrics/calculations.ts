import {
  SimulationResult,
  LcoeResult,
  SimulationConfig,
  CostParams,
  PricingResult,
  ElccResult,
} from '../../types';

export interface CalculatedMetrics {
  // Core
  annual_match: number;
  hourly_match: number;
  ghg_intensity: number;
  lcoe: number;

  // System Performance
  curtailed: number;
  zero_price_gen: number;
  load_utilization: number;
  gas_capacity: number;
  peak_shave: number;

  // Economic
  operating_costs: number;
  customer_costs: number | null; // null if pricing not calculated
  solar_market_value: number | null;
  wind_market_value: number | null;
  solar_system_value: number | null; // null if ELCC not calculated
  wind_system_value: number | null;

  // Environmental
  direct_land_use: number;  // Physical footprint only (mi²)
  total_land_use: number;   // Includes indirect (wind spacing, exclusion zones) (mi²)

  // Raw references for ELCC table
  elccResult: ElccResult | null;
}

/**
 * Calculate all derived metrics from simulation results
 */
export function calculateMetrics(
  simulation: SimulationResult,
  lcoe: LcoeResult,
  config: SimulationConfig,
  costs: CostParams,
  loadProfile: number[],
  pricingResult: PricingResult | null = null,
  elccResult: ElccResult | null = null
): CalculatedMetrics {
  // Clean Match % - already calculated in simulation
  const annual_match = simulation.clean_match_pct;

  // Hourly Match % - percentage of hours where clean_delivered >= load
  const hourly_match = calculateHourlyMatch(simulation, loadProfile);

  // GHG Intensity - already in LCOE result
  const ghg_intensity = lcoe.emissions_intensity;

  // System LCOE - already in LCOE result
  const system_lcoe = lcoe.total_lcoe;

  // Curtailed % - relative to total clean generation
  const annualCleanFirmGen = config.clean_firm_capacity * 8760;
  const totalCleanGen = simulation.annual_renewable_gen + annualCleanFirmGen;
  const curtailed = totalCleanGen > 0
    ? (simulation.total_curtailment / totalCleanGen) * 100
    : 0;

  // Zero Price Gen % - renewable gen during curtailment hours (excess supply = zero price)
  const zero_price_gen = calculateZeroPriceGen(simulation);

  // Load Utilization % - (actual load served) / (original load)
  const totalDemandResponse = simulation.demand_response.reduce((a, b) => a + b, 0);
  const originalLoad = simulation.annual_load + totalDemandResponse;
  const load_utilization = originalLoad > 0
    ? (simulation.annual_load / originalLoad) * 100
    : 100;

  // Gas Capacity Needed (MW) - peak gas generation
  const gas_capacity = simulation.peak_gas;

  // Peak Shave (MW) - difference between peak load and peak gas
  const peakLoad = Math.max(...loadProfile);
  const peak_shave = Math.max(0, peakLoad - simulation.peak_gas);

  // Operating Costs ($/MWh) - variable O&M only (fuel + var O&M)
  const operating_costs = calculateOperatingCosts(simulation, config, costs);

  // Customer Costs ($/MWh) - from pricing result
  const customer_costs = pricingResult ? pricingResult.average_price : null;

  // Solar/Wind Market Values - energy-weighted average prices
  const solar_market_value = pricingResult
    ? calculateMarketValue(simulation.solar_out, pricingResult.hourly_prices)
    : null;
  const wind_market_value = pricingResult
    ? calculateMarketValue(simulation.wind_out, pricingResult.hourly_prices)
    : null;

  // Solar/Wind System Values - include capacity value from ELCC
  const solar_system_value = elccResult && pricingResult
    ? calculateSystemValue('solar', simulation, config, elccResult, pricingResult, costs)
    : null;
  const wind_system_value = elccResult && pricingResult
    ? calculateSystemValue('wind', simulation, config, elccResult, pricingResult, costs)
    : null;

  // Land Use - convert from acres to mi² (640 acres = 1 mi²)
  const direct_land_use = lcoe.direct_land_use / 640;
  const total_land_use = lcoe.total_land_use / 640;

  return {
    annual_match,
    hourly_match,
    ghg_intensity,
    lcoe: system_lcoe,
    curtailed,
    zero_price_gen,
    load_utilization,
    gas_capacity,
    peak_shave,
    operating_costs,
    customer_costs,
    solar_market_value,
    wind_market_value,
    solar_system_value,
    wind_system_value,
    direct_land_use,
    total_land_use,
    elccResult,
  };
}

/**
 * Calculate hourly match percentage
 * = hours where (renewable + battery discharge) >= load
 */
function calculateHourlyMatch(
  simulation: SimulationResult,
  loadProfile: number[]
): number {
  let matchedHours = 0;
  for (let i = 0; i < loadProfile.length; i++) {
    const load = loadProfile[i];
    if (load <= 0.01) continue; // Skip zero-load hours

    const cleanDelivered = simulation.clean_delivered[i];
    if (cleanDelivered >= load * 0.999) { // 99.9% threshold for rounding
      matchedHours++;
    }
  }

  const totalLoadHours = loadProfile.filter(l => l > 0.01).length;
  return totalLoadHours > 0 ? (matchedHours / totalLoadHours) * 100 : 0;
}

/**
 * Calculate percentage of renewable generation during zero/negative price hours
 * Uses curtailment as the indicator - when there's curtailment, supply exceeds demand
 * and market prices would be zero or negative in reality
 */
function calculateZeroPriceGen(
  simulation: SimulationResult
): number {
  let zeroPriceGen = 0;
  let totalRenewableGen = 0;

  for (let i = 0; i < simulation.solar_out.length; i++) {
    const renewableGen = simulation.solar_out[i] + simulation.wind_out[i];
    totalRenewableGen += renewableGen;

    // Count generation during curtailment hours (excess supply = zero price)
    if (simulation.curtailed[i] > 0.01) {
      zeroPriceGen += renewableGen;
    }
  }

  return totalRenewableGen > 0 ? (zeroPriceGen / totalRenewableGen) * 100 : 0;
}

/**
 * Calculate operating costs (fuel + variable O&M) in $/MWh
 */
function calculateOperatingCosts(
  simulation: SimulationResult,
  _config: SimulationConfig, // Reserved for future use
  costs: CostParams
): number {
  const totalLoad = simulation.annual_load;
  if (totalLoad <= 0) return 0;

  // Gas fuel costs
  const totalGasGen = simulation.gas_generation.reduce((a, b) => a + b, 0);
  const ccsFraction = Math.min(1, Math.max(0, costs.ccs_percentage / 100));
  const ccsPenalty = Math.max(0, costs.ccs_energy_penalty / 100);
  const gasWithoutCcs = totalGasGen * (1 - ccsFraction);
  const gasWithCcs = totalGasGen * ccsFraction;
  const gasFuelCost =
    gasWithoutCcs * costs.gas_heat_rate * costs.gas_price +
    gasWithCcs * costs.gas_heat_rate * (1 + ccsPenalty) * costs.gas_price;
  const ccsVarOmCost = gasWithCcs * costs.ccs_var_om;

  // Variable O&M costs
  const solarGen = simulation.solar_out.reduce((a, b) => a + b, 0);
  const windGen = simulation.wind_out.reduce((a, b) => a + b, 0);
  const storageThruput = simulation.battery_discharge.reduce((a, b) => a + b, 0);
  const cleanFirmGen = simulation.clean_firm_generation.reduce((a, b) => a + b, 0);

  const varOmCost =
    solarGen * costs.solar_var_om +
    windGen * costs.wind_var_om +
    storageThruput * costs.storage_var_om +
    cleanFirmGen * costs.clean_firm_var_om +
    totalGasGen * costs.gas_var_om +
    ccsVarOmCost;

  // Clean firm fuel costs
  const cleanFirmFuelCost = cleanFirmGen * costs.clean_firm_fuel;

  return (gasFuelCost + varOmCost + cleanFirmFuelCost) / totalLoad;
}

/**
 * Calculate energy-weighted average price for a generation profile
 */
function calculateMarketValue(
  generation: number[],
  prices: number[]
): number {
  let weightedSum = 0;
  let totalGen = 0;

  for (let i = 0; i < generation.length; i++) {
    const gen = generation[i];
    if (gen > 0.01) {
      weightedSum += gen * prices[i];
      totalGen += gen;
    }
  }

  return totalGen > 0 ? weightedSum / totalGen : 0;
}

/**
 * Calculate system value (market value + capacity value) for a resource
 */
function calculateSystemValue(
  resource: 'solar' | 'wind',
  simulation: SimulationResult,
  config: SimulationConfig,
  elcc: ElccResult,
  pricing: PricingResult,
  costs: CostParams
): number {
  const generation = resource === 'solar' ? simulation.solar_out : simulation.wind_out;
  const capacity = resource === 'solar' ? config.solar_capacity : config.wind_capacity;

  const totalGen = generation.reduce((a, b) => a + b, 0);
  if (totalGen <= 0 || capacity <= 0) return 0;

  // Energy value (market value)
  const marketValue = calculateMarketValue(generation, pricing.hourly_prices);

  // Capacity value based on ELCC
  const resourceElcc = resource === 'solar' ? elcc.solar : elcc.wind;
  const elccMw = resourceElcc.delta * capacity / 100;

  // Capacity price (use capacity market data if available, else estimate)
  const capacityPrice = pricing.capacity_data
    ? pricing.capacity_data.clearing_price
    : estimateCapacityValue(costs);

  // Capacity value per MWh = (ELCC MW * $/MW-yr) / annual MWh
  const capacityValuePerMwh = (elccMw * capacityPrice) / totalGen;

  return marketValue + capacityValuePerMwh;
}

/**
 * Estimate capacity value when no capacity market data available
 * Based on gas peaker cost of new entry (CONE)
 */
function estimateCapacityValue(costs: CostParams): number {
  // Simple CONE estimate: gas capex annualized + fixed O&M
  const annualizationFactor = costs.discount_rate / 100 /
    (1 - Math.pow(1 + costs.discount_rate / 100, -costs.gas_lifetime));
  const gasCapexAnnual = costs.gas_capex * 1000 * annualizationFactor;
  const gasFixedOmAnnual = costs.gas_fixed_om * 1000;

  return gasCapexAnnual + gasFixedOmAnnual; // $/MW-yr
}

/**
 * Find the hour with peak gas generation (for "Go to peak week" feature)
 */
export function findPeakGasHour(gasGeneration: number[]): number {
  let peakHour = 0;
  let maxGas = 0;

  for (let i = 0; i < gasGeneration.length; i++) {
    if (gasGeneration[i] > maxGas) {
      maxGas = gasGeneration[i];
      peakHour = i;
    }
  }

  return peakHour;
}

/**
 * Convert hour index to week number (1-52)
 */
export function hourToWeek(hour: number): number {
  return Math.min(52, Math.floor(hour / 168) + 1);
}
