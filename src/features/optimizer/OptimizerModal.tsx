import React, { useState } from 'react';
import { Modal, Button, Slider } from '../../components/atoms';
import { useSimulationStore } from '../../stores/simulationStore';
import { useSettingsStore } from '../../stores/settingsStore';
import { OptimizerConfig, OptimizerResult, DEFAULT_OPTIMIZER_CONFIG } from '../../types';
import { serializeCostParams, withOptimizerRuntimeConfig } from '../../lib/wasmSerde';

interface OptimizerModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const OptimizerModal: React.FC<OptimizerModalProps> = ({ isOpen, onClose }) => {
  const [config, setConfig] = useState<OptimizerConfig>({ ...DEFAULT_OPTIMIZER_CONFIG });
  const [isRunning, setIsRunning] = useState(false);
  const [result, setResult] = useState<OptimizerResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [elapsedMs, setElapsedMs] = useState<number | null>(null);

  const solarProfile = useSimulationStore((state) => state.solarProfile);
  const windProfile = useSimulationStore((state) => state.windProfile);
  const loadProfile = useSimulationStore((state) => state.loadProfile);
  const batteryMode = useSimulationStore((state) => state.config.battery_mode);
  const simulationConfig = useSimulationStore((state) => state.config);
  const applyOptimizerResult = useSimulationStore((state) => state.applyOptimizerResult);
  const costs = useSettingsStore((state) => state.costs);

  const handleRun = async () => {
    const wasm = (window as any).__wasmModule;
    if (!wasm) {
      setError('WASM module not loaded');
      return;
    }

    setIsRunning(true);
    setError(null);
    setResult(null);
    setElapsedMs(null);

    const startTime = performance.now();

    try {
      const wasmCosts = serializeCostParams(costs);
      const wasmOptimizerConfig = withOptimizerRuntimeConfig(config, simulationConfig);

      // Convert profiles to Float64Array
      const solarFloat = new Float64Array(solarProfile);
      const windFloat = new Float64Array(windProfile);
      const loadFloat = new Float64Array(loadProfile);

      const optimizerResult: OptimizerResult = wasm.optimize(
        wasmOptimizerConfig.target_clean_match,
        solarFloat,
        windFloat,
        loadFloat,
        wasmCosts,
        wasmOptimizerConfig,
        batteryMode
      );

      const endTime = performance.now();
      setElapsedMs(endTime - startTime);
      setResult(optimizerResult);
    } catch (err) {
      const endTime = performance.now();
      setElapsedMs(endTime - startTime);
      setError(err instanceof Error ? err.message : 'Optimization failed');
    } finally {
      setIsRunning(false);
    }
  };

  const handleApply = () => {
    if (result) {
      applyOptimizerResult({
        solar: result.solar_capacity,
        wind: result.wind_capacity,
        storage: result.storage_capacity,
        cleanFirm: result.clean_firm_capacity,
      });
      onClose();
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Portfolio Optimizer" size="md">
      <div className="space-y-6">
        {/* Target */}
        <div>
          <Slider
            label="Clean Match Target"
            value={config.target_clean_match}
            min={0}
            max={100}
            step={5}
            unit="%"
            onChange={(v) => setConfig({ ...config, target_clean_match: v })}
          />
        </div>

        {/* Resource selection */}
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-2">Enable Resources</h4>
          <div className="grid grid-cols-2 gap-2">
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.enable_solar}
                onChange={(e) => setConfig({ ...config, enable_solar: e.target.checked })}
                className="rounded"
              />
              <span className="text-sm">Solar</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.enable_wind}
                onChange={(e) => setConfig({ ...config, enable_wind: e.target.checked })}
                className="rounded"
              />
              <span className="text-sm">Wind</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.enable_storage}
                onChange={(e) => setConfig({ ...config, enable_storage: e.target.checked })}
                className="rounded"
              />
              <span className="text-sm">Storage</span>
            </label>
            <label className="flex items-center gap-2">
              <input
                type="checkbox"
                checked={config.enable_clean_firm}
                onChange={(e) => setConfig({ ...config, enable_clean_firm: e.target.checked })}
                className="rounded"
              />
              <span className="text-sm">Clean Firm</span>
            </label>
          </div>
        </div>

        {/* Capacity limits */}
        <div>
          <h4 className="text-sm font-medium text-gray-700 mb-2">Maximum Capacities</h4>
          <div className="grid grid-cols-2 gap-4">
            <Slider
              label="Max Solar (MW)"
              value={config.max_solar}
              min={50}
              max={1000}
              step={25}
              onChange={(v) => setConfig({ ...config, max_solar: v })}
            />
            <Slider
              label="Max Wind (MW)"
              value={config.max_wind}
              min={50}
              max={700}
              step={25}
              onChange={(v) => setConfig({ ...config, max_wind: v })}
            />
            <Slider
              label="Max Storage (MWh)"
              value={config.max_storage}
              min={50}
              max={2400}
              step={25}
              onChange={(v) => setConfig({ ...config, max_storage: v })}
            />
            <Slider
              label="Max Clean Firm (MW)"
              value={config.max_clean_firm}
              min={0}
              max={200}
              step={5}
              onChange={(v) => setConfig({ ...config, max_clean_firm: v })}
            />
          </div>
        </div>

        {/* Results */}
        {result && (
          <div className="bg-gray-50 rounded-lg p-4">
            <h4 className="text-sm font-semibold text-gray-700 mb-3">Optimization Result</h4>
            <div className="grid grid-cols-2 gap-3 text-sm">
              <div>
                <span className="text-gray-500">Solar:</span>{' '}
                <span className="font-medium">{result.solar_capacity.toFixed(0)} MW</span>
              </div>
              <div>
                <span className="text-gray-500">Wind:</span>{' '}
                <span className="font-medium">{result.wind_capacity.toFixed(0)} MW</span>
              </div>
              <div>
                <span className="text-gray-500">Storage:</span>{' '}
                <span className="font-medium">{result.storage_capacity.toFixed(0)} MWh</span>
              </div>
              <div>
                <span className="text-gray-500">Clean Firm:</span>{' '}
                <span className="font-medium">{result.clean_firm_capacity.toFixed(0)} MW</span>
              </div>
              <div className="col-span-2 pt-2 border-t mt-2">
                <span className="text-gray-500">Clean Match:</span>{' '}
                <span className="font-semibold text-green-600">
                  {result.achieved_clean_match.toFixed(1)}%
                </span>
              </div>
              <div>
                <span className="text-gray-500">LCOE:</span>{' '}
                <span className="font-medium">${result.lcoe.toFixed(1)}/MWh</span>
              </div>
              <div>
                <span className="text-gray-500">Evaluations:</span>{' '}
                <span className="font-medium">{result.num_evaluations}</span>
              </div>
              {elapsedMs !== null && (
                <div>
                  <span className="text-gray-500">Time:</span>{' '}
                  <span className="font-medium">
                    {elapsedMs < 1000
                      ? `${elapsedMs.toFixed(0)} ms`
                      : `${(elapsedMs / 1000).toFixed(2)} s`}
                  </span>
                </div>
              )}
            </div>
          </div>
        )}

        {/* Error */}
        {error && (
          <div className="bg-red-50 text-red-700 rounded-lg p-4 text-sm">{error}</div>
        )}

        {/* Actions */}
        <div className="flex justify-end gap-2 pt-4 border-t">
          <Button variant="secondary" onClick={onClose}>
            Cancel
          </Button>
          <Button onClick={handleRun} disabled={isRunning}>
            {isRunning ? (
              <span className="flex items-center gap-2">
                <svg className="animate-spin h-4 w-4" viewBox="0 0 24 24">
                  <circle
                    className="opacity-25"
                    cx="12"
                    cy="12"
                    r="10"
                    stroke="currentColor"
                    strokeWidth="4"
                    fill="none"
                  />
                  <path
                    className="opacity-75"
                    fill="currentColor"
                    d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
                  />
                </svg>
                Running...
              </span>
            ) : (
              'Run Optimization'
            )}
          </Button>
          {result && (
            <Button onClick={handleApply} variant="primary">
              Apply Result
            </Button>
          )}
        </div>
      </div>
    </Modal>
  );
};
