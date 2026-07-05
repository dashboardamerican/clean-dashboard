import React, { useEffect, useState } from 'react';
import { Slider } from '../../components/atoms';
import { useCostOptimizerStore, OptimizerResourceKey } from '../../stores/costOptimizerStore';
import { useSettingsStore } from '../../stores/settingsStore';
import { useSimulationStore } from '../../stores/simulationStore';
import { COLORS, CostParams } from '../../types';

interface CostLeverDefinition {
  key: keyof Pick<
    CostParams,
    | 'solar_capex'
    | 'wind_capex'
    | 'storage_capex'
    | 'clean_firm_capex'
    | 'gas_capex'
    | 'gas_price'
  >;
  label: string;
  min: number;
  max: number;
  step: number;
  color: string;
  format: (value: number) => string;
}

const COST_LEVERS: CostLeverDefinition[] = [
  {
    key: 'solar_capex',
    label: 'Solar CAPEX',
    min: 400,
    max: 1800,
    step: 25,
    color: COLORS.solar,
    format: (value) => `$${value.toLocaleString()}/kW`,
  },
  {
    key: 'wind_capex',
    label: 'Wind CAPEX',
    min: 500,
    max: 2200,
    step: 25,
    color: COLORS.wind,
    format: (value) => `$${value.toLocaleString()}/kW`,
  },
  {
    key: 'storage_capex',
    label: 'Storage CAPEX',
    min: 75,
    max: 700,
    step: 25,
    color: COLORS.storage,
    format: (value) => `$${value.toLocaleString()}/kWh`,
  },
  {
    key: 'clean_firm_capex',
    label: 'Clean Firm CAPEX',
    min: 500,
    max: 12000,
    step: 100,
    color: COLORS.cleanFirm,
    format: (value) => `$${value.toLocaleString()}/kW`,
  },
  {
    key: 'gas_capex',
    label: 'Gas CAPEX',
    min: 300,
    max: 1800,
    step: 25,
    color: COLORS.gas,
    format: (value) => `$${value.toLocaleString()}/kW`,
  },
  {
    key: 'gas_price',
    label: 'Gas Fuel',
    min: 1,
    max: 16,
    step: 0.25,
    color: COLORS.gas,
    format: (value) => `$${value.toLocaleString(undefined, { maximumFractionDigits: 2 })}/MMBtu`,
  },
];

const RESOURCE_TOGGLES: Array<{
  key: OptimizerResourceKey;
  label: string;
  color: string;
}> = [
  { key: 'solar', label: 'Solar', color: COLORS.solar },
  { key: 'wind', label: 'Wind', color: COLORS.wind },
  { key: 'storage', label: 'Storage', color: COLORS.storage },
  { key: 'cleanFirm', label: 'Clean Firm', color: COLORS.cleanFirm },
];

interface CostLeverSliderProps {
  lever: CostLeverDefinition;
  value: number;
  onCommit: (value: number) => void;
}

const CostLeverSlider: React.FC<CostLeverSliderProps> = ({ lever, value, onCommit }) => {
  const [draft, setDraft] = useState(value);

  useEffect(() => {
    setDraft(value);
  }, [value]);

  const commit = () => {
    if (Math.abs(draft - value) > 1e-9) {
      onCommit(draft);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between gap-3 mb-1">
        <label className="text-sm font-medium text-gray-700">{lever.label}</label>
        <span className="text-sm font-semibold tabular-nums" style={{ color: lever.color }}>
          {lever.format(draft)}
        </span>
      </div>
      <input
        type="range"
        min={lever.min}
        max={lever.max}
        step={lever.step}
        value={draft}
        onChange={(event) => setDraft(Number(event.target.value))}
        onPointerUp={commit}
        onBlur={commit}
        onKeyUp={commit}
        className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
        style={{ accentColor: lever.color }}
      />
      <div className="flex justify-between text-xs text-gray-500 mt-1">
        <span>{lever.format(lever.min)}</span>
        <span>{lever.format(lever.max)}</span>
      </div>
    </div>
  );
};

const PortfolioRow: React.FC<{
  label: string;
  value: string;
  color?: string;
}> = ({ label, value, color }) => (
  <div className="flex items-center justify-between gap-3 py-1.5">
    <dt className="text-sm text-gray-500">{label}</dt>
    <dd className="text-sm font-semibold tabular-nums" style={{ color }}>
      {value}
    </dd>
  </div>
);

export const CostOptimizedControls: React.FC = () => {
  const costs = useSettingsStore((state) => state.costs);
  const setCost = useSettingsStore((state) => state.setCost);
  const config = useSimulationStore((state) => state.config);
  const simulationResult = useSimulationStore((state) => state.simulationResult);

  const optimizerConfig = useCostOptimizerStore((state) => state.optimizerConfig);
  const setTargetCleanMatch = useCostOptimizerStore((state) => state.setTargetCleanMatch);
  const setResourceEnabled = useCostOptimizerStore((state) => state.setResourceEnabled);
  const result = useCostOptimizerStore((state) => state.result);
  const isRunning = useCostOptimizerStore((state) => state.isRunning);
  const error = useCostOptimizerStore((state) => state.error);
  const elapsedMs = useCostOptimizerStore((state) => state.elapsedMs);
  const optimizerPath = useCostOptimizerStore((state) => state.optimizerPath);

  const gasCapacity = simulationResult
    ? simulationResult.peak_gas * (1 + costs.reserve_margin / 100)
    : null;

  const statusText = isRunning
    ? 'Optimizing...'
    : error
      ? 'Needs attention'
      : result && elapsedMs !== null
        ? `Optimized in ${elapsedMs < 1000 ? `${elapsedMs.toFixed(0)} ms` : `${(elapsedMs / 1000).toFixed(2)} s`}${optimizerPath ? ` · ${optimizerPath === 'v2-model' ? 'v2 model' : 'v2'}` : ''}`
        : 'Ready';

  const statusClass = isRunning
    ? 'bg-blue-50 text-blue-700'
    : error
      ? 'bg-red-50 text-red-700'
      : result
        ? 'bg-green-50 text-green-700'
        : 'bg-gray-100 text-gray-600';

  return (
    <div data-tutorial-id="capacity-sliders" className="space-y-5">
      <div className="flex items-center justify-between gap-3">
        <h3 className="text-sm font-medium text-gray-700 uppercase tracking-wide">
          Cost-Optimized Portfolio
        </h3>
        <span className={`px-2 py-1 rounded text-xs font-medium ${statusClass}`}>
          {statusText}
        </span>
      </div>

      <Slider
        label="Clean Match Target"
        value={optimizerConfig.target_clean_match}
        min={0}
        max={100}
        step={5}
        unit="%"
        color="#2563eb"
        onChange={setTargetCleanMatch}
      />

      <div>
        <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wide mb-2">
          Allowed Resources
        </h4>
        <div className="grid grid-cols-2 gap-2">
          {RESOURCE_TOGGLES.map((resource) => {
            const checked =
              resource.key === 'solar'
                ? optimizerConfig.enable_solar
                : resource.key === 'wind'
                  ? optimizerConfig.enable_wind
                  : resource.key === 'storage'
                    ? optimizerConfig.enable_storage
                    : optimizerConfig.enable_clean_firm;

            return (
              <label
                key={resource.key}
                className={`flex items-center justify-between gap-2 rounded-md border px-3 py-2 text-sm font-medium ${
                  checked ? 'border-gray-300 bg-gray-50 text-gray-900' : 'border-gray-200 text-gray-400'
                }`}
              >
                <span className="flex items-center gap-2">
                  <span
                    className="h-2.5 w-2.5 rounded-full"
                    style={{ backgroundColor: checked ? resource.color : '#d1d5db' }}
                  />
                  {resource.label}
                </span>
                <input
                  type="checkbox"
                  checked={checked}
                  onChange={(event) => setResourceEnabled(resource.key, event.target.checked)}
                  className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
                />
              </label>
            );
          })}
        </div>
      </div>

      <div className="space-y-4">
        <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wide">
          Cost Levers
        </h4>
        {COST_LEVERS.map((lever) => (
          <CostLeverSlider
            key={lever.key}
            lever={lever}
            value={Number(costs[lever.key])}
            onCommit={(value) => setCost(lever.key, value)}
          />
        ))}
      </div>

      <div className="pt-4 border-t border-gray-100">
        <div className="flex items-center justify-between mb-2">
          <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wide">
            Optimized Build
          </h4>
          {isRunning && (
            <span className="h-4 w-4 rounded-full border-2 border-blue-200 border-t-blue-600 animate-spin" />
          )}
        </div>
        <dl className="divide-y divide-gray-100">
          <PortfolioRow
            label="Solar"
            value={`${config.solar_capacity.toFixed(0)} MW`}
            color={COLORS.solar}
          />
          <PortfolioRow
            label="Wind"
            value={`${config.wind_capacity.toFixed(0)} MW`}
            color={COLORS.wind}
          />
          <PortfolioRow
            label="Storage"
            value={`${config.storage_capacity.toFixed(0)} MWh`}
            color={COLORS.storage}
          />
          <PortfolioRow
            label="Clean Firm"
            value={`${config.clean_firm_capacity.toFixed(0)} MW`}
            color={COLORS.cleanFirm}
          />
          <PortfolioRow
            label="Gas Capacity"
            value={gasCapacity === null ? '--' : `${gasCapacity.toFixed(0)} MW`}
            color={COLORS.gas}
          />
          <PortfolioRow
            label="Clean Match"
            value={result ? `${result.achieved_clean_match.toFixed(1)}%` : '--'}
          />
          <PortfolioRow
            label="LCOE"
            value={result ? `$${result.lcoe.toFixed(1)}/MWh` : '--'}
          />
        </dl>
        {error && <p className="mt-3 text-sm text-red-600">{error}</p>}
      </div>
    </div>
  );
};
