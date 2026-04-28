import React, { useMemo, useEffect, useRef } from 'react';
import { Metric } from '../atoms';
import { useSimulationStore } from '../../stores/simulationStore';
import { useSettingsStore } from '../../stores/settingsStore';
import { useMetricsStore } from '../../stores/metricsStore';
import { useElccStore } from '../../stores/elccStore';
import { usePricingStore } from '../../stores/pricingStore';
import { useUiStore } from '../../stores/uiStore';
import { COLORS, ElccMethod } from '../../types';
import { getMetricById, getGhgColor, requiresElccCalculation, requiresPricingCalculation } from '../../types/metrics';
import { calculateMetrics, findPeakGasHour, hourToWeek } from '../../features/metrics';
import { ElccAnalysisMetric } from '../../features/metrics';

interface MetricsPanelProps {
  onOpenMetricsModal: () => void;
}

export const MetricsPanel: React.FC<MetricsPanelProps> = ({ onOpenMetricsModal }) => {
  const simulationResult = useSimulationStore((state) => state.simulationResult);
  const lcoeResult = useSimulationStore((state) => state.lcoeResult);
  const config = useSimulationStore((state) => state.config);
  const loadProfile = useSimulationStore((state) => state.loadProfile);

  const costs = useSettingsStore((state) => state.costs);

  const selectedMetrics = useMetricsStore((state) => state.selectedMetrics);
  const elccMethod = useMetricsStore((state) => state.elccMethod);

  const elccResult = useElccStore((state) => state.elccResult);
  const calculateElcc = useElccStore((state) => state.calculateElcc);
  const isCalculatingElcc = useElccStore((state) => state.isCalculating);

  const pricingResult = usePricingStore((state) => state.pricingResult);
  const calculatePrices = usePricingStore((state) => state.calculatePrices);
  const isCalculatingPrices = usePricingStore((state) => state.isCalculating);

  const setSelectedWeek = useUiStore((state) => state.setSelectedWeek);
  const setVisualization = useUiStore((state) => state.setVisualization);

  // Track which simulation the ELCC/pricing was calculated for
  const lastElccSimRef = useRef<typeof simulationResult>(null);
  const lastPricingSimRef = useRef<typeof simulationResult>(null);

  // Auto-calculate ELCC if needed metrics are selected
  const needsElcc = requiresElccCalculation(selectedMetrics);
  useEffect(() => {
    // Only recalculate if simulation changed or ELCC not yet calculated
    const simChanged = simulationResult !== lastElccSimRef.current;
    if (simulationResult && needsElcc && !isCalculatingElcc && (simChanged || !elccResult)) {
      lastElccSimRef.current = simulationResult;
      calculateElcc();
    }
  }, [needsElcc, isCalculatingElcc, simulationResult, elccResult, calculateElcc]);

  // Auto-calculate pricing if needed metrics are selected
  const needsPricing = requiresPricingCalculation(selectedMetrics);
  useEffect(() => {
    // Only recalculate if simulation changed or pricing not yet calculated
    const simChanged = simulationResult !== lastPricingSimRef.current;
    if (simulationResult && lcoeResult && needsPricing && !isCalculatingPrices && (simChanged || !pricingResult)) {
      lastPricingSimRef.current = simulationResult;
      calculatePrices();
    }
  }, [needsPricing, isCalculatingPrices, simulationResult, lcoeResult, pricingResult, calculatePrices]);

  // Calculate all metrics
  const metrics = useMemo(() => {
    if (!simulationResult || !lcoeResult) return null;
    return calculateMetrics(
      simulationResult,
      lcoeResult,
      config,
      costs,
      loadProfile,
      pricingResult,
      elccResult
    );
  }, [simulationResult, lcoeResult, config, costs, loadProfile, pricingResult, elccResult]);

  // Handler for "Go to peak week" button
  const handleGoToPeakWeek = () => {
    if (!simulationResult) return;
    const peakHour = findPeakGasHour(simulationResult.gas_generation);
    const peakWeek = hourToWeek(peakHour);
    setSelectedWeek(peakWeek);
    setVisualization('weekly');
  };

  // Don't show spinner - simulations are fast (<50ms)
  // Just show stale data while running, or placeholder if no data yet
  if (!simulationResult || !lcoeResult || !metrics) {
    return (
      <div data-tutorial-id="metrics-panel" className="bg-white rounded-lg shadow p-4">
        <div className="flex items-center justify-center h-24 text-gray-500">
          Adjust capacity sliders to run simulation
        </div>
      </div>
    );
  }

  // Get battery mode name for display
  const batteryModeNames = ['Default', 'Peak Shaver', 'Hybrid'];
  const batteryModeName = batteryModeNames[config.battery_mode] || 'Default';
  const elccMethodNames: Record<ElccMethod, string> = {
    [ElccMethod.FirstIn]: 'First-In',
    [ElccMethod.Marginal]: 'Marginal',
    [ElccMethod.Contribution]: 'Contribution',
    [ElccMethod.Delta]: 'Delta',
  };
  const elccMethodName = elccMethodNames[elccMethod] || 'Contribution';

  // Separate metrics into primary (core) and secondary (non-core)
  const coreMetricIds = ['annual_match', 'hourly_match', 'ghg_intensity', 'lcoe'];
  const primaryMetrics = selectedMetrics.filter(id => coreMetricIds.includes(id));
  const secondaryMetrics = selectedMetrics.filter(id => !coreMetricIds.includes(id) && id !== 'elcc_analysis');
  const showElccTable = selectedMetrics.includes('elcc_analysis');

  // Render a metric by ID
  const renderMetric = (metricId: string, size: 'sm' | 'md' | 'lg' = 'lg') => {
    const def = getMetricById(metricId);
    if (!def) return null;

    // Handle special metrics
    if (def.isToggle) {
      if (metricId === 'battery_mode') {
        return (
          <Metric
            key={metricId}
            label={def.label}
            value={batteryModeName}
            size={size}
          />
        );
      }
      if (metricId === 'delta_elcc_method') {
        return (
          <Metric
            key={metricId}
            label={def.label}
            value={elccMethodName}
            size={size}
          />
        );
      }
      return null;
    }

    // Get the value for this metric
    let value: number | string | null = null;
    let color: string | undefined;
    let colorIndicator: string | undefined;
    let actionButton: { label: string; onClick: () => void } | undefined;

    switch (metricId) {
      case 'annual_match':
        value = metrics.annual_match;
        color = metrics.annual_match >= 80 ? COLORS.battery : COLORS.gas;
        break;
      case 'hourly_match':
        value = metrics.hourly_match;
        color = metrics.hourly_match >= 80 ? COLORS.battery : COLORS.gas;
        break;
      case 'ghg_intensity':
        value = metrics.ghg_intensity;
        colorIndicator = getGhgColor(metrics.ghg_intensity);
        break;
      case 'lcoe':
        value = metrics.lcoe;
        break;
      case 'curtailed':
        value = metrics.curtailed;
        color = metrics.curtailed > 10 ? COLORS.gas : undefined;
        break;
      case 'zero_price_gen':
        value = metrics.zero_price_gen;
        break;
      case 'load_utilization':
        value = metrics.load_utilization;
        break;
      case 'gas_capacity':
        value = metrics.gas_capacity;
        color = COLORS.gas;
        actionButton = { label: 'Go', onClick: handleGoToPeakWeek };
        break;
      case 'peak_shave':
        value = metrics.peak_shave;
        color = COLORS.battery;
        break;
      case 'operating_costs':
        value = metrics.operating_costs;
        break;
      case 'customer_costs':
        value = metrics.customer_costs;
        if (value === null) return renderCalculatingPlaceholder(def.label, 'pricing');
        break;
      case 'solar_market_value':
        value = metrics.solar_market_value;
        if (value === null) return renderCalculatingPlaceholder(def.label, 'pricing');
        color = COLORS.solar;
        break;
      case 'wind_market_value':
        value = metrics.wind_market_value;
        if (value === null) return renderCalculatingPlaceholder(def.label, 'pricing');
        color = COLORS.wind;
        break;
      case 'solar_system_value':
        value = metrics.solar_system_value;
        if (value === null) return renderCalculatingPlaceholder(def.label, 'ELCC+pricing');
        color = COLORS.solar;
        break;
      case 'wind_system_value':
        value = metrics.wind_system_value;
        if (value === null) return renderCalculatingPlaceholder(def.label, 'ELCC+pricing');
        color = COLORS.wind;
        break;
      case 'land_use':
        // Special rendering for land use - show both direct and total
        return (
          <div key={metricId} className="text-center">
            <div className="text-xs text-gray-500 uppercase tracking-wide">{def.label}</div>
            <div className="text-lg font-semibold">
              <span title="Direct footprint">{metrics.direct_land_use.toFixed(1)}</span>
              <span className="text-gray-400 mx-1">/</span>
              <span title="Total (incl. indirect)">{metrics.total_land_use.toFixed(1)}</span>
              <span className="text-sm text-gray-500 ml-1">{def.unit}</span>
            </div>
            <div className="text-xs text-gray-400">direct / total</div>
          </div>
        );
      default:
        return null;
    }

    if (value === null) return null;

    return (
      <Metric
        key={metricId}
        label={def.label}
        value={value}
        unit={def.unit}
        color={color}
        colorIndicator={colorIndicator}
        actionButton={actionButton}
        size={size}
      />
    );
  };

  const renderCalculatingPlaceholder = (label: string, type: string) => (
    <div key={label} className="text-center">
      <div className="text-xs text-gray-500 uppercase tracking-wide">{label}</div>
      <div className="text-sm text-gray-400 italic">
        {type === 'pricing' && isCalculatingPrices ? 'Calculating...' :
         type === 'ELCC+pricing' && (isCalculatingElcc || isCalculatingPrices) ? 'Calculating...' :
         `Requires ${type}`}
      </div>
    </div>
  );

  return (
    <div data-tutorial-id="metrics-panel" className="bg-white rounded-lg shadow p-4">
      {/* Header with "+ More Metrics" button */}
      <div className="flex justify-end mb-2">
        <button
          onClick={onOpenMetricsModal}
          data-tutorial-id="more-metrics-button"
          className="text-sm text-blue-600 hover:text-blue-800 flex items-center gap-1"
        >
          <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 6v6m0 0v6m0-6h6m-6 0H6" />
          </svg>
          More Metrics
        </button>
      </div>

      {/* Primary metrics (large) */}
      {primaryMetrics.length > 0 && (
        <div className={`grid gap-6 ${
          primaryMetrics.length === 1 ? 'grid-cols-1' :
          primaryMetrics.length === 2 ? 'grid-cols-2' :
          primaryMetrics.length === 3 ? 'grid-cols-3' :
          'grid-cols-2 md:grid-cols-4'
        }`}>
          {primaryMetrics.map(id => renderMetric(id, 'lg'))}
        </div>
      )}

      {/* Secondary metrics (small) */}
      {secondaryMetrics.length > 0 && (
        <div className={`mt-6 pt-4 border-t grid gap-4 ${
          secondaryMetrics.length <= 3 ? 'grid-cols-3' :
          secondaryMetrics.length <= 6 ? 'grid-cols-3 lg:grid-cols-6' :
          'grid-cols-4 lg:grid-cols-8'
        }`}>
          {secondaryMetrics.map(id => renderMetric(id, 'sm'))}
        </div>
      )}

      {/* ELCC Analysis table */}
      {showElccTable && (
        <div className="mt-4 pt-4 border-t">
          {elccResult ? (
            <ElccAnalysisMetric elccResult={elccResult} />
          ) : isCalculatingElcc ? (
            <div className="text-center text-gray-400 text-sm py-4">
              Calculating ELCC...
            </div>
          ) : (
            <div className="text-center text-gray-400 text-sm py-4">
              Run simulation to calculate ELCC
            </div>
          )}
        </div>
      )}

      {/* Show curtailment bar if curtailment exists (always shown as visual indicator) */}
      {simulationResult.total_curtailment > 0 && !selectedMetrics.includes('curtailed') && (
        <div className="mt-4 pt-4 border-t">
          <div className="flex items-center gap-2">
            <span className="text-sm text-gray-600">Curtailment:</span>
            <div className="flex-1 h-2 bg-gray-200 rounded-full overflow-hidden">
              <div
                className="h-full bg-gray-400"
                style={{
                  width: `${Math.min(100, metrics.curtailed)}%`,
                }}
              />
            </div>
            <span className="text-sm font-medium">
              {metrics.curtailed.toFixed(1)}%
            </span>
          </div>
        </div>
      )}

      {/* Empty state */}
      {selectedMetrics.length === 0 && (
        <div className="text-center text-gray-400 py-8">
          No metrics selected.{' '}
          <button
            onClick={onOpenMetricsModal}
            className="text-blue-600 hover:underline"
          >
            Select metrics
          </button>
        </div>
      )}
    </div>
  );
};
