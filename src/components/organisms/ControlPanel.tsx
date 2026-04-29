import React from 'react';
import { Slider, Select } from '../atoms';
import { useSimulationStore, LoadType } from '../../stores/simulationStore';
import { BatteryMode, COLORS, ZONE_NAMES } from '../../types';

export const ControlPanel: React.FC = () => {
  const config = useSimulationStore((state) => state.config);
  const setConfig = useSimulationStore((state) => state.setConfig);
  const zone = useSimulationStore((state) => state.zone);
  const setZone = useSimulationStore((state) => state.setZone);
  const loadType = useSimulationStore((state) => state.loadType);
  const setLoadType = useSimulationStore((state) => state.setLoadType);
  const setBatteryMode = useSimulationStore((state) => state.setBatteryMode);

  const batteryModeOptions = [
    { value: BatteryMode.Default, label: 'Default (Water-fill)' },
    { value: BatteryMode.PeakShaver, label: 'Peak Shaver' },
    { value: BatteryMode.Hybrid, label: 'Hybrid' },
    { value: BatteryMode.LimitedForecast, label: 'Limited Forecast (48h)' },
  ];

  const loadTypeOptions = [
    { value: 'hourly', label: 'Hourly Load (zone profile)' },
    { value: 'flat', label: 'Flat Load (100 MW constant)' },
  ];

  const zoneOptions = ZONE_NAMES.map((z) => ({ value: z, label: z }));

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

      {/* Capacity sliders */}
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

      {/* Storage */}
      <div data-tutorial-id="storage-and-battery" className="space-y-4">
        <h3 className="text-sm font-medium text-gray-700 uppercase tracking-wide">
          Energy Storage
        </h3>

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

        <Select
          label="Battery Mode"
          value={config.battery_mode}
          options={batteryModeOptions}
          onChange={(v) => setBatteryMode(Number(v) as BatteryMode)}
        />
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
