import React, { useState } from 'react';
import Plot from 'react-plotly.js';
import { useSweepStore } from '../../stores/sweepStore';
import { COLORS, CostSweepParam } from '../../types';

type ViewMode = 'capacity' | 'lcoe';

const PARAM_LABELS: Record<CostSweepParam, string> = {
  solar_capex: 'Solar CAPEX ($/kW)',
  wind_capex: 'Wind CAPEX ($/kW)',
  storage_capex: 'Storage CAPEX ($/kWh)',
  clean_firm_capex: 'Clean Firm CAPEX ($/kW)',
  gas_capex: 'Gas CAPEX ($/kW)',
  gas_price: 'Gas Price ($/MMBtu)',
  solar_itc: 'Solar ITC (%)',
  wind_itc: 'Wind ITC (%)',
  storage_itc: 'Storage ITC (%)',
  clean_firm_itc: 'Clean Firm ITC (%)',
  discount_rate: 'Discount Rate (%)',
};

export const CostSweepChart: React.FC = () => {
  const {
    costSweepResult,
    costSweepParam,
    costSweepRange,
    costSweepTarget,
    costSweepSteps,
    isRunning,
    error,
    setCostSweepParam,
    setCostSweepRange,
    setCostSweepTarget,
    setCostSweepSteps,
    runCostSweep,
  } = useSweepStore();

  const [viewMode, setViewMode] = useState<ViewMode>('capacity');

  const handleRun = async () => {
    await runCostSweep();
  };

  if (isRunning) {
    return (
      <div className="h-96 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-gray-600">Running cost sweep...</span>
      </div>
    );
  }

  if (error) {
    return (
      <div className="h-96 flex flex-col items-center justify-center">
        <p className="text-red-500 font-medium">Error</p>
        <p className="text-sm text-gray-600 mt-1">{error}</p>
        <button
          onClick={handleRun}
          className="mt-4 px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Try Again
        </button>
      </div>
    );
  }

  // Controls always visible
  const controls = (
    <div className="flex flex-wrap justify-between items-start mb-4 px-4 gap-4">
      <div className="flex flex-wrap items-center gap-4">
        {/* Parameter selector */}
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Parameter:</label>
          <select
            value={costSweepParam}
            onChange={(e) => setCostSweepParam(e.target.value as CostSweepParam)}
            className="text-sm border rounded px-2 py-1"
          >
            {Object.entries(PARAM_LABELS).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </div>

        {/* Target */}
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Target:</label>
          <input
            type="number"
            value={costSweepTarget}
            onChange={(e) => setCostSweepTarget(parseFloat(e.target.value))}
            className="w-16 text-sm border rounded px-2 py-1"
            min="0"
            max="100"
            step="5"
          />
          <span className="text-xs text-gray-500">%</span>
        </div>

        {/* Range */}
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Range:</label>
          <input
            type="number"
            value={costSweepRange[0]}
            onChange={(e) => setCostSweepRange([parseFloat(e.target.value), costSweepRange[1]])}
            className="w-20 text-sm border rounded px-2 py-1"
          />
          <span className="text-gray-400">—</span>
          <input
            type="number"
            value={costSweepRange[1]}
            onChange={(e) => setCostSweepRange([costSweepRange[0], parseFloat(e.target.value)])}
            className="w-20 text-sm border rounded px-2 py-1"
          />
        </div>

        {/* Steps */}
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Steps:</label>
          <input
            type="number"
            value={costSweepSteps}
            onChange={(e) => setCostSweepSteps(parseInt(e.target.value))}
            className="w-14 text-sm border rounded px-2 py-1"
            min="3"
            max="20"
          />
        </div>
      </div>

      <div className="flex items-center gap-2">
        {costSweepResult && (
          <div className="inline-flex rounded-md shadow-sm mr-2" role="group">
            {(['capacity', 'lcoe'] as ViewMode[]).map((mode) => (
              <button
                key={mode}
                type="button"
                onClick={() => setViewMode(mode)}
                className={`px-3 py-1.5 text-xs font-medium border ${
                  mode === 'capacity' ? 'rounded-l-md' : 'rounded-r-md'
                } ${
                  viewMode === mode
                    ? 'bg-blue-600 text-white border-blue-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                {mode === 'capacity' ? 'Capacity' : 'LCOE'}
              </button>
            ))}
          </div>
        )}

        <button
          onClick={handleRun}
          className="text-xs px-3 py-1.5 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Run Sweep
        </button>
      </div>
    </div>
  );

  if (!costSweepResult) {
    return (
      <div>
        {controls}
        <div className="h-80 flex flex-col items-center justify-center text-gray-500">
          <svg className="w-16 h-16 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1}
              d="M7 12l3-3 3 3 4-4M8 21l4-4 4 4M3 4h18M4 4h16v12a1 1 0 01-1 1H5a1 1 0 01-1-1V4z"
            />
          </svg>
          <p className="text-lg font-medium">Cost Sensitivity Sweep</p>
          <p className="text-sm mt-1">See how optimal portfolios change with costs</p>
        </div>
      </div>
    );
  }

  const { points, param_name, target_match, elapsed_ms } = costSweepResult;
  const paramValues = points.map((p) => p.param_value);

  // Build traces based on view mode
  let traces: any[] = [];

  if (viewMode === 'capacity') {
    traces = [
      {
        x: paramValues,
        y: points.map((p) => p.solar),
        name: 'Solar',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.solar, width: 2 },
        marker: { size: 6 },
      },
      {
        x: paramValues,
        y: points.map((p) => p.wind),
        name: 'Wind',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.wind, width: 2 },
        marker: { size: 6 },
      },
      {
        x: paramValues,
        y: points.map((p) => p.storage),
        name: 'Storage (MWh)',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.storage, width: 2 },
        marker: { size: 6 },
      },
      {
        x: paramValues,
        y: points.map((p) => p.clean_firm),
        name: 'Clean Firm',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.cleanFirm, width: 2 },
        marker: { size: 6 },
      },
    ];
  } else {
    traces = [
      {
        x: paramValues,
        y: points.map((p) => p.lcoe),
        name: 'LCOE',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: '#333', width: 2 },
        marker: { size: 6 },
        fill: 'tozeroy',
        fillcolor: 'rgba(100, 100, 100, 0.1)',
      },
    ];
  }

  const layout: any = {
    height: 350,
    margin: { t: 40, r: 30, b: 60, l: 60 },
    xaxis: {
      title: { text: PARAM_LABELS[param_name as CostSweepParam] || param_name },
    },
    yaxis: {
      title: {
        text: viewMode === 'lcoe' ? '$/MWh' : 'Capacity (MW)',
      },
    },
    font: {
      family: 'Google Sans, Roboto, sans-serif',
    },
    legend: {
      orientation: 'h',
      y: -0.2,
      x: 0.5,
      xanchor: 'center',
    },
    annotations: [
      {
        x: 1,
        y: 1.02,
        xref: 'paper',
        yref: 'paper',
        text: `Target: ${target_match}% | Completed in ${elapsed_ms.toFixed(0)}ms`,
        showarrow: false,
        font: { size: 11, color: '#666' },
        xanchor: 'right',
      },
    ],
  };

  return (
    <div>
      {controls}

      {/* Chart */}
      <Plot
        data={traces}
        layout={layout}
        config={{
          responsive: true,
          displayModeBar: false,
        }}
        style={{ width: '100%' }}
      />

      {/* Summary table */}
      <div className="mt-4 px-4 overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b bg-gray-50">
              <th className="text-left py-2 px-2">{PARAM_LABELS[param_name as CostSweepParam]}</th>
              <th className="text-right py-2 px-2">Solar</th>
              <th className="text-right py-2 px-2">Wind</th>
              <th className="text-right py-2 px-2">Storage</th>
              <th className="text-right py-2 px-2">CF</th>
              <th className="text-right py-2 px-2">LCOE</th>
              <th className="text-right py-2 px-2">Status</th>
            </tr>
          </thead>
          <tbody>
            {points.map((p) => (
              <tr key={p.param_value} className="border-b border-gray-100 hover:bg-gray-50">
                <td className="py-1 px-2">{p.param_value.toFixed(0)}</td>
                <td className="text-right py-1 px-2">{p.solar.toFixed(0)}</td>
                <td className="text-right py-1 px-2">{p.wind.toFixed(0)}</td>
                <td className="text-right py-1 px-2">{p.storage.toFixed(0)}</td>
                <td className="text-right py-1 px-2">{p.clean_firm.toFixed(0)}</td>
                <td className="text-right py-1 px-2">${p.lcoe.toFixed(1)}</td>
                <td className="text-right py-1 px-2">
                  {p.success ? (
                    <span className="text-green-600">✓</span>
                  ) : (
                    <span className="text-red-500">✗</span>
                  )}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
