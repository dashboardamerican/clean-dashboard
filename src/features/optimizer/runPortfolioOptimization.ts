import {
  BatteryMode,
  CostParams,
  OptimizerConfig,
  OptimizerResult,
  SimulationConfig,
} from '../../types';
import { serializeCostParams, withOptimizerRuntimeConfig } from '../../lib/wasmSerde';
import { ensureModelLoaded } from '../../lib/modelLoader';

interface RunPortfolioOptimizationArgs {
  wasm: any;
  zone: string;
  optimizerConfig: OptimizerConfig;
  simulationConfig: SimulationConfig;
  solarProfile: number[];
  windProfile: number[];
  loadProfile: number[];
  costs: CostParams;
  batteryMode: BatteryMode;
}

export interface PortfolioOptimizationRun {
  result: OptimizerResult;
  elapsedMs: number;
  usedModel: boolean;
  optimizerPath: 'v2-model' | 'v2';
}

export async function runPortfolioOptimization({
  wasm,
  zone,
  optimizerConfig,
  simulationConfig,
  solarProfile,
  windProfile,
  loadProfile,
  costs,
  batteryMode,
}: RunPortfolioOptimizationArgs): Promise<PortfolioOptimizationRun> {
  const wasmCosts = serializeCostParams(costs);
  const wasmOptimizerConfig = withOptimizerRuntimeConfig(optimizerConfig, simulationConfig);
  const modelStatus = await ensureModelLoaded(zone, batteryMode);
  const useModel = modelStatus.loaded && typeof wasm.optimize_with_model === 'function';
  const useExplicitV2 = !useModel && typeof wasm.optimize_v2 === 'function';

  const solarFloat = new Float64Array(solarProfile);
  const windFloat = new Float64Array(windProfile);
  const loadFloat = new Float64Array(loadProfile);

  const startTime = performance.now();
  const result: OptimizerResult = useModel
    ? wasm.optimize_with_model(
        zone,
        wasmOptimizerConfig.target_clean_match,
        solarFloat,
        windFloat,
        loadFloat,
        wasmCosts,
        wasmOptimizerConfig,
        batteryMode
      )
    : useExplicitV2
      ? wasm.optimize_v2(
          wasmOptimizerConfig.target_clean_match,
          solarFloat,
          windFloat,
          loadFloat,
          wasmCosts,
          wasmOptimizerConfig,
          batteryMode
        )
      : wasm.optimize(
        wasmOptimizerConfig.target_clean_match,
        solarFloat,
        windFloat,
        loadFloat,
        wasmCosts,
        wasmOptimizerConfig,
        batteryMode
      );

  return {
    result,
    elapsedMs: performance.now() - startTime,
    usedModel: useModel,
    optimizerPath: useModel ? 'v2-model' : 'v2',
  };
}
