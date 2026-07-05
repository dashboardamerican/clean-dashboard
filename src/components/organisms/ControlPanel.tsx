import React, { useEffect } from 'react';
import { Slider, Select } from '../atoms';
import { useSimulationStore, LoadType } from '../../stores/simulationStore';
import { useCostOptimizerStore } from '../../stores/costOptimizerStore';
import { BatteryMode, COLORS, ZONE_NAMES } from '../../types';
import { CostOptimizedControls } from '../../features/optimizer/CostOptimizedControls';

export const ControlPanel: React.FC = () => {
  const config = useSimulationStore((state) => state.config);
  const setConfig = useSimulationStore((state) => state.setConfig);
  const zone = useSimulationStore((state) => state.zone);
  const setZone = useSimulationStore((state) => state.setZone);
  const loadType = useSimulationStore((state) => state.loadType);
  const setLoadType = useSimulationStore((state) => state.setLoadType);
  const setBatteryMode = useSimulationStore((state) => state.setBatteryMode);
  const isLimitedForecast = config.battery_mode === BatteryMode.LimitedForecast;
  const controlMode = useCostOptimizerStore((state) => state.mode);
  const setControlMode = useCostOptimizerStore((state) => state.setMode);
  const scheduleAutoOptimize = useCostOptimizerStore((state) => state.scheduleAutoOptimize);

  const batteryModeOptions = [
    { value: BatteryMode.Default, label: 'Default (Water-fill)' },
    { value: BatteryMode.PeakShaver, label: 'Peak Shaver' },
    { value: BatteryMode.Hybrid, label: 'Hybrid' },
    { value: BatteryMode.LimitedForecast, label: 'Limited Forecast Lab' },
  ];

  const loadTypeOptions = [
    { value: 'hourly', label: 'Hourly Load (zone profile)' },
    { value: 'flat', label: 'Flat Load (100 MW constant)' },
  ];

  const limitedForecastHorizonOptions = [
    { value: 24, label: '24 hours' },
    { value: 48, label: '48 hours' },
    { value: 72, label: '72 hours' },
    { value: 168, label: '168 hours' },
  ];
  const limitedForecastCommitOptions = [
    { value: 1, label: 'Hourly' },
    { value: 6, label: 'Every 6 hours' },
    { value: 24, label: 'Daily' },
  ].filter((option) => option.value <= config.limited_forecast_horizon_hours);

  const zoneOptions = ZONE_NAMES.map((z) => ({ value: z, label: z }));

  useEffect(() => {
    if (controlMode === 'costOptimized') {
      scheduleAutoOptimize();
    }
  }, [
    controlMode,
    zone,
    loadType,
    config.battery_mode,
    config.battery_efficiency,
    config.max_demand_response,
    config.limited_forecast_horizon_hours,
    config.limited_forecast_commit_hours,
    config.limited_forecast_renewable_error_pct,
    config.limited_forecast_soc_reserve_pct,
    config.limited_forecast_peak_value_mwh_per_mw,
    config.limited_forecast_allow_gas_charging,
    scheduleAutoOptimize,
  ]);

  return (
    <div className="bg-white rounded-lg shadow p-4 space-y-6">
      <h2 className="text-lg font-semibold text-gray-900 border-b pb-2">
        System Configuration
      </h2>

      {/* Zone selection */}
      <div data-tutorial-id="region-selector">
        <Select
          label="Region"
          value={zone}
          options={zoneOptions}
          onChange={(v) => setZone(v as typeof zone)}
        />
      </div>

      {/* Load shape */}
      <div data-tutorial-id="load-shape">
        <Select
          label="Load Shape"
          value={loadType}
          options={loadTypeOptions}
          onChange={(v) => setLoadType(v as LoadType)}
        />
      </div>

      {/* Control mode */}
      <div>
        <div className="grid grid-cols-2 gap-1 rounded-md bg-gray-100 p-1">
          <button
            type="button"
            onClick={() => setControlMode('manualCapacity')}
            className={`rounded px-3 py-2 text-sm font-medium transition ${
              controlMode === 'manualCapacity'
                ? 'bg-white text-gray-900 shadow-sm'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            Manual Build
          </button>
          <button
            type="button"
            onClick={() => setControlMode('costOptimized')}
            className={`rounded px-3 py-2 text-sm font-medium transition ${
              controlMode === 'costOptimized'
                ? 'bg-white text-gray-900 shadow-sm'
                : 'text-gray-500 hover:text-gray-700'
            }`}
          >
            Optimize From Costs
          </button>
        </div>
      </div>

      {controlMode === 'manualCapacity' ? (
        <div data-tutorial-id="capacity-sliders" className="space-y-4">
          <h3 className="text-sm font-medium text-gray-700 uppercase tracking-wide">
            Generation Capacity
          </h3>

          <Slider
            label="Solar Capacity"
            value={config.solar_capacity}
            min={0}
            max={1000}
            step={10}
            unit="MW"
            color={COLORS.solar}
            onChange={(v) => setConfig({ solar_capacity: v })}
          />

          <Slider
            label="Wind Capacity"
            value={config.wind_capacity}
            min={0}
            max={700}
            step={10}
            unit="MW"
            color={COLORS.wind}
            onChange={(v) => setConfig({ wind_capacity: v })}
          />

          <Slider
            label="Clean Firm Capacity"
            value={config.clean_firm_capacity}
            min={0}
            max={200}
            step={5}
            unit="MW"
            color={COLORS.cleanFirm}
            onChange={(v) => setConfig({ clean_firm_capacity: v })}
          />
        </div>
      ) : (
        <CostOptimizedControls />
      )}

      {/* Storage */}
      <div data-tutorial-id="storage-and-battery" className="space-y-4">
        <h3 className="text-sm font-medium text-gray-700 uppercase tracking-wide">
          Energy Storage
        </h3>

        {controlMode === 'manualCapacity' && (
          <Slider
            label="Storage Capacity"
            value={config.storage_capacity}
            min={0}
            max={2400}
            step={50}
            unit="MWh"
            color={COLORS.storage}
            onChange={(v) => setConfig({ storage_capacity: v })}
          />
        )}

        <Select
          label="Battery Mode"
          value={config.battery_mode}
          options={batteryModeOptions}
          onChange={(v) => setBatteryMode(Number(v) as BatteryMode)}
        />

        {isLimitedForecast && (
          <div className="pt-3 border-t border-gray-100 space-y-4">
            <h4 className="text-xs font-medium text-gray-500 uppercase tracking-wide">
              Limited Forecast
            </h4>

            <Select
              label="Forecast Horizon"
              value={config.limited_forecast_horizon_hours}
              options={limitedForecastHorizonOptions}
              onChange={(v) => {
                const horizon = Number(v);
                setConfig({
                  limited_forecast_horizon_hours: horizon,
                  limited_forecast_commit_hours: Math.min(
                    config.limited_forecast_commit_hours,
                    horizon
                  ),
                });
              }}
            />

            <Select
              label="Re-optimization Cadence"
              value={config.limited_forecast_commit_hours}
              options={limitedForecastCommitOptions}
              onChange={(v) => setConfig({ limited_forecast_commit_hours: Number(v) })}
            />

            <Slider
              label="Renewables Forecast Error"
              value={config.limited_forecast_renewable_error_pct}
              min={0}
              max={100}
              step={10}
              unit="%"
              color={COLORS.wind}
              onChange={(v) => setConfig({ limited_forecast_renewable_error_pct: v })}
            />

            <Slider
              label="SOC Holdback"
              value={config.limited_forecast_soc_reserve_pct}
              min={0}
              max={50}
              step={5}
              unit="%"
              color={COLORS.storage}
              onChange={(v) => setConfig({ limited_forecast_soc_reserve_pct: v })}
            />

            <Slider
              label="Peak Value Weight"
              value={config.limited_forecast_peak_value_mwh_per_mw}
              min={0}
              max={96}
              step={6}
              unit="MWh/MW"
              color={COLORS.gas}
              onChange={(v) => setConfig({ limited_forecast_peak_value_mwh_per_mw: v })}
            />

            <label className="flex items-center justify-between gap-3 text-sm font-medium text-gray-700">
              <span>Economic Gas/Grid Charging</span>
              <input
                type="checkbox"
                checked={config.limited_forecast_allow_gas_charging}
                onChange={(e) =>
                  setConfig({ limited_forecast_allow_gas_charging: e.target.checked })
                }
                className="h-4 w-4 rounded border-gray-300 text-blue-600 focus:ring-blue-500"
              />
            </label>
          </div>
        )}
      </div>

      {/* Demand Response */}
      <div data-tutorial-id="demand-response" className="space-y-4">
        <h3 className="text-sm font-medium text-gray-700 uppercase tracking-wide">
          Demand Response
        </h3>

        <Slider
          label="Max Demand Response"
          value={config.max_demand_response}
          min={0}
          max={100}
          step={5}
          unit="MW"
          color={COLORS.dr}
          onChange={(v) => setConfig({ max_demand_response: v })}
        />
      </div>

      {/* Keyboard shortcuts hint */}
      <div className="pt-4 border-t text-xs text-gray-400">
        <p className="font-medium mb-1">Keyboard shortcuts:</p>
        <ul className="space-y-0.5">
          <li>
            <kbd className="bg-gray-100 px-1 rounded">S</kbd> Settings
          </li>
          <li>
            <kbd className="bg-gray-100 px-1 rounded">O</kbd> Optimizer
          </li>
          <li>
            <kbd className="bg-gray-100 px-1 rounded">M</kbd> Metrics
          </li>
          <li>
            <kbd className="bg-gray-100 px-1 rounded">R</kbd> Reset
          </li>
        </ul>
      </div>
    </div>
  );
};
