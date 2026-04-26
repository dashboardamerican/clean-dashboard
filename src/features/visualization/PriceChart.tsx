import React, { useMemo, useState, useEffect } from 'react';
import Plot from 'react-plotly.js';
import { usePricingStore } from '../../stores/pricingStore';
import { useSimulationStore } from '../../stores/simulationStore';
import { useElccStore } from '../../stores/elccStore';
import { useUiStore } from '../../stores/uiStore';
import { PricingMethod } from '../../types';

type ViewMode = 'hourly' | 'duration' | 'weekly';

const PRICING_METHOD_LABELS: Record<PricingMethod, string> = {
  [PricingMethod.ScarcityBased]: 'Scarcity-Based',
  [PricingMethod.MarginalCost]: 'Marginal Cost',
  [PricingMethod.Ordc]: 'ORDC',
  [PricingMethod.MarginalPlusCapacity]: 'Marginal + Capacity',
};

export const PriceChart: React.FC = () => {
  const {
    pricingMethod,
    ordcConfig,
    pricingResult,
    isCalculating,
    error,
    setPricingMethod,
    setOrdcConfig,
    calculatePrices,
  } = usePricingStore();

  const simulationResult = useSimulationStore((state) => state.simulationResult);
  const lcoeResult = useSimulationStore((state) => state.lcoeResult);
  const { calculateElcc } = useElccStore();

  const [viewMode, setViewMode] = useState<ViewMode>('hourly');
  const selectedWeek = useUiStore((state) => state.selectedWeek);
  const setSelectedWeek = useUiStore((state) => state.setSelectedWeek);

  // Calculate prices when simulation changes or method changes
  useEffect(() => {
    if (simulationResult && lcoeResult) {
      // First calculate ELCC if needed for capacity market
      if (pricingMethod === PricingMethod.MarginalPlusCapacity) {
        calculateElcc().then(() => calculatePrices());
      } else {
        calculatePrices();
      }
    }
  }, [simulationResult, lcoeResult, pricingMethod, ordcConfig]);

  const chartData = useMemo(() => {
    if (!pricingResult) return null;

    const { hourly_prices } = pricingResult;

    if (viewMode === 'hourly') {
      // Show week view
      const hoursPerWeek = selectedWeek === 52 ? 192 : 168;
      const startHour = (selectedWeek - 1) * 168;
      const endHour = Math.min(startHour + hoursPerWeek, 8760);
      const weekPrices = hourly_prices.slice(startHour, endHour);
      const hours = Array.from({ length: weekPrices.length }, (_, i) => startHour + i);

      return {
        x: hours,
        y: weekPrices,
        xLabel: 'Hour of Year',
      };
    } else if (viewMode === 'duration') {
      // Sort prices descending for duration curve
      const sorted = [...hourly_prices].sort((a, b) => b - a);
      const hours = Array.from({ length: sorted.length }, (_, i) => i);

      return {
        x: hours,
        y: sorted,
        xLabel: 'Hours (sorted by price)',
      };
    } else {
      // Weekly averages
      const weeklyAvgs: number[] = [];
      for (let w = 0; w < 52; w++) {
        const start = w * 168;
        const end = Math.min(start + 168, 8760);
        const weekSlice = hourly_prices.slice(start, end);
        const avg = weekSlice.reduce((a, b) => a + b, 0) / weekSlice.length;
        weeklyAvgs.push(avg);
      }

      return {
        x: Array.from({ length: 52 }, (_, i) => i + 1),
        y: weeklyAvgs,
        xLabel: 'Week',
      };
    }
  }, [pricingResult, viewMode, selectedWeek]);

  if (!simulationResult) {
    return (
      <div className="h-96 flex flex-col items-center justify-center text-gray-500">
        <svg className="w-16 h-16 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1}
            d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
        <p className="text-lg font-medium">No Simulation Data</p>
        <p className="text-sm mt-1">Run a simulation to see market prices</p>
      </div>
    );
  }

  if (isCalculating) {
    return (
      <div className="h-96 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-gray-600">Calculating prices...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-96 flex flex-col items-center justify-center text-red-500">
        <p className="text-lg font-medium">Error</p>
        <p className="text-sm mt-1">{error}</p>
      </div>
    );
  }

  const showCapacityPanel =
    pricingMethod === PricingMethod.MarginalPlusCapacity && pricingResult?.capacity_data;

  return (
    <div>
      {/* Controls */}
      <div className="flex flex-wrap justify-between items-center mb-4 px-4 gap-4">
        {/* Pricing method selector */}
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Method:</label>
          <select
            value={pricingMethod}
            onChange={(e) => setPricingMethod(Number(e.target.value) as PricingMethod)}
            className="text-sm border rounded px-2 py-1"
          >
            {Object.entries(PRICING_METHOD_LABELS).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </div>

        {/* View mode toggle */}
        <div className="inline-flex rounded-md shadow-sm" role="group">
          {(['hourly', 'duration', 'weekly'] as ViewMode[]).map((mode) => (
            <button
              key={mode}
              type="button"
              onClick={() => setViewMode(mode)}
              className={`px-3 py-1.5 text-xs font-medium border ${
                mode === 'hourly' ? 'rounded-l-md' : mode === 'weekly' ? 'rounded-r-md' : ''
              } ${
                viewMode === mode
                  ? 'bg-blue-600 text-white border-blue-600'
                  : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
              }`}
            >
              {mode === 'hourly' ? 'Hourly' : mode === 'duration' ? 'Duration' : 'Weekly'}
            </button>
          ))}
        </div>

        {viewMode === 'hourly' && (
          <div className="flex items-center gap-2">
            <button
              onClick={() => setSelectedWeek(selectedWeek - 1)}
              disabled={selectedWeek <= 1}
              className="p-1 rounded hover:bg-gray-100 disabled:opacity-50"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <span className="text-xs font-medium text-gray-600">Week {selectedWeek}</span>
            <button
              onClick={() => setSelectedWeek(selectedWeek + 1)}
              disabled={selectedWeek >= 52}
              className="p-1 rounded hover:bg-gray-100 disabled:opacity-50"
            >
              <svg className="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>
        )}
      </div>

      {/* ORDC config (only for ORDC method) */}
      {pricingMethod === PricingMethod.Ordc && (
        <div className="flex flex-wrap gap-4 mb-4 px-4 p-2 bg-gray-50 rounded">
          <div className="flex items-center gap-2">
            <label className="text-xs text-gray-600">Reserve Req (%):</label>
            <input
              type="number"
              value={ordcConfig.reserve_requirement}
              onChange={(e) => setOrdcConfig({ reserve_requirement: parseFloat(e.target.value) })}
              className="w-16 text-sm border rounded px-2 py-1"
              min="0"
              max="20"
              step="0.5"
            />
          </div>
          <div className="flex items-center gap-2">
            <label className="text-xs text-gray-600">Lambda:</label>
            <input
              type="number"
              value={ordcConfig.lambda}
              onChange={(e) => setOrdcConfig({ lambda: parseFloat(e.target.value) })}
              className="w-16 text-sm border rounded px-2 py-1"
              min="0.5"
              max="10"
              step="0.5"
            />
          </div>
          <div className="flex items-center gap-2">
            <label className="text-xs text-gray-600">Max Price ($/MWh):</label>
            <input
              type="number"
              value={ordcConfig.max_price}
              onChange={(e) => setOrdcConfig({ max_price: parseFloat(e.target.value) })}
              className="w-20 text-sm border rounded px-2 py-1"
              min="1000"
              max="10000"
              step="500"
            />
          </div>
        </div>
      )}

      {/* Charts */}
      <div className={`${showCapacityPanel ? 'grid grid-cols-3 gap-4' : ''}`}>
        {/* Price chart */}
        <div className={showCapacityPanel ? 'col-span-2' : ''}>
          {chartData && (
            <Plot
              data={[
                {
                  x: chartData.x,
                  y: chartData.y,
                  type: 'scatter',
                  mode: 'lines',
                  line: { color: '#4285f4', width: 1 },
                  fill: viewMode === 'duration' ? 'tozeroy' : undefined,
                  fillcolor: viewMode === 'duration' ? 'rgba(66, 133, 245, 0.2)' : undefined,
                  hovertemplate: `${chartData.xLabel}: %{x}<br>Price: $%{y:.2f}/MWh<extra></extra>`,
                },
              ]}
              layout={{
                height: 320,
                margin: { t: 30, r: 30, b: 50, l: 60 },
                xaxis: {
                  title: { text: chartData.xLabel },
                },
                yaxis: {
                  title: { text: '$/MWh' },
                },
                font: {
                  family: 'Google Sans, Roboto, sans-serif',
                },
                annotations: pricingResult
                  ? [
                      {
                        x: 1,
                        y: 1.02,
                        xref: 'paper',
                        yref: 'paper',
                        text: `Avg: $${pricingResult.average_price.toFixed(1)} | Peak: $${pricingResult.peak_price.toFixed(0)} | Min: $${pricingResult.min_price.toFixed(1)}`,
                        showarrow: false,
                        font: { size: 12, color: '#666' },
                        xanchor: 'right',
                      },
                    ]
                  : [],
              }}
              config={{
                responsive: true,
                displayModeBar: false,
              }}
              style={{ width: '100%' }}
            />
          )}
        </div>

        {/* Capacity market panel */}
        {showCapacityPanel && pricingResult?.capacity_data && (
          <div className="col-span-1 p-4 bg-gray-50 rounded">
            <h3 className="text-sm font-semibold mb-3">Capacity Market</h3>
            <div className="text-xs space-y-2">
              <div>
                <span className="text-gray-500">Clearing Price:</span>
                <span className="ml-2 font-medium">
                  ${pricingResult.capacity_data.clearing_price.toFixed(0)}/MW-yr
                </span>
              </div>
              <div>
                <span className="text-gray-500">Adder:</span>
                <span className="ml-2 font-medium">
                  ${pricingResult.capacity_data.adder_per_mwh.toFixed(2)}/MWh
                </span>
              </div>
              <hr className="my-2" />
              <div className="font-medium mb-2">Qualified Capacity (MW)</div>
              {Object.entries(pricingResult.capacity_data.qualified_capacity)
                .filter(([k, v]) => v > 0 && k !== 'gas')
                .map(([key, value]) => (
                  <div key={key} className="flex justify-between">
                    <span className="capitalize">{key.replace('_', ' ')}</span>
                    <span>{(value as number).toFixed(1)}</span>
                  </div>
                ))}
              <hr className="my-2" />
              <div className="font-medium mb-2">Annual Payments ($M)</div>
              {Object.entries(pricingResult.capacity_data.annual_payments)
                .filter(([k, v]) => v > 0 && k !== 'gas')
                .map(([key, value]) => (
                  <div key={key} className="flex justify-between">
                    <span className="capitalize">{key.replace('_', ' ')}</span>
                    <span>${((value as number) / 1e6).toFixed(2)}M</span>
                  </div>
                ))}
            </div>
          </div>
        )}
      </div>
    </div>
  );
};
