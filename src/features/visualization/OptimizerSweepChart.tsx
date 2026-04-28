import React, { useState } from 'react';
import Plot from 'react-plotly.js';
import { useSweepStore } from '../../stores/sweepStore';
import { COLORS } from '../../types';

type ViewMode = 'capacity' | 'lcoe' | 'composition';
type CompositionUnit = 'mw' | 'mwh';

export const OptimizerSweepChart: React.FC = () => {
  const {
    sweepResult,
    savedSweep,
    savedLabel,
    useFineTargets,
    sweepResources,
    isRunning,
    error,
    setUseFineTargets,
    setSweepResource,
    runOptimizerSweep,
    saveAsComparison,
    clearSavedComparison,
  } = useSweepStore();

  const [viewMode, setViewMode] = useState<ViewMode>('lcoe');
  const [compositionUnit, setCompositionUnit] = useState<CompositionUnit>('mw');
  const [saveLabel, setSaveLabel] = useState('');

  const resourceToggles = (
    <div className="flex flex-wrap items-center gap-3">
      <span className="text-xs font-medium text-gray-600">Allow:</span>
      {([
        ['solar', 'Solar'],
        ['wind', 'Wind'],
        ['storage', 'Storage'],
        ['clean_firm', 'Clean Firm'],
      ] as const).map(([key, label]) => (
        <label key={key} className="flex items-center gap-1.5 text-xs">
          <input
            type="checkbox"
            checked={sweepResources[key]}
            onChange={(e) => setSweepResource(key, e.target.checked)}
            className="rounded"
          />
          {label}
        </label>
      ))}
    </div>
  );

  const handleRun = async () => {
    await runOptimizerSweep();
  };

  const handleSave = () => {
    if (sweepResult && saveLabel.trim()) {
      saveAsComparison(saveLabel.trim());
      setSaveLabel('');
    }
  };

  if (isRunning) {
    return (
      <div className="h-96 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-gray-600">Running optimizer sweep...</span>
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

  if (!sweepResult) {
    return (
      <div className="h-96 flex flex-col items-center justify-center text-gray-500">
        <svg className="w-16 h-16 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1}
            d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
          />
        </svg>
        <p className="text-lg font-medium">Optimizer Sweep</p>
        <p className="text-sm mt-1">Run a sweep to see optimal portfolios across targets</p>
        <div className="mt-4 flex flex-col items-center gap-3">
          {resourceToggles}
          <div className="flex items-center gap-4">
            <label className="flex items-center gap-2 text-sm">
              <input
                type="checkbox"
                checked={useFineTargets}
                onChange={(e) => setUseFineTargets(e.target.checked)}
                className="rounded"
              />
              Fine targets (5% steps)
            </label>
            <button
              onClick={handleRun}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700"
            >
              Run Sweep
            </button>
          </div>
        </div>
      </div>
    );
  }

  const points = sweepResult.points || [];
  if (points.length === 0) {
    return <div className="p-4 text-gray-500">No sweep results available</div>;
  }
  const targets = points.map((p) => p.target);

  // Build traces based on view mode
  let traces: any[] = [];

  if (viewMode === 'capacity') {
    // Line chart of capacity by target
    traces = [
      {
        x: targets,
        y: points.map((p) => p.solar),
        name: 'Solar',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.solar, width: 2 },
        marker: { size: 6 },
      },
      {
        x: targets,
        y: points.map((p) => p.wind),
        name: 'Wind',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.wind, width: 2 },
        marker: { size: 6 },
      },
      {
        x: targets,
        y: points.map((p) => p.storage),
        name: 'Storage (MWh)',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.storage, width: 2 },
        marker: { size: 6 },
      },
      {
        x: targets,
        y: points.map((p) => p.clean_firm),
        name: 'Clean Firm',
        type: 'scatter',
        mode: 'lines+markers',
        line: { color: COLORS.cleanFirm, width: 2 },
        marker: { size: 6 },
      },
    ];

    // Add comparison if saved
    if (savedSweep) {
      const savedPoints = savedSweep.points;
      traces.push(
        {
          x: savedPoints.map((p) => p.target),
          y: savedPoints.map((p) => p.solar),
          name: `Solar (${savedLabel})`,
          type: 'scatter',
          mode: 'lines',
          line: { color: COLORS.solar, width: 1, dash: 'dot' },
          opacity: 0.6,
        },
        {
          x: savedPoints.map((p) => p.target),
          y: savedPoints.map((p) => p.wind),
          name: `Wind (${savedLabel})`,
          type: 'scatter',
          mode: 'lines',
          line: { color: COLORS.wind, width: 1, dash: 'dot' },
          opacity: 0.6,
        },
        {
          x: savedPoints.map((p) => p.target),
          y: savedPoints.map((p) => p.clean_firm),
          name: `CF (${savedLabel})`,
          type: 'scatter',
          mode: 'lines',
          line: { color: COLORS.cleanFirm, width: 1, dash: 'dot' },
          opacity: 0.6,
        }
      );
    }
  } else if (viewMode === 'lcoe') {
    // LCOE stacked area showing breakdown by resource
    traces = [
      {
        x: targets,
        y: points.map((p) => p.gas_lcoe || 0),
        name: 'Gas',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.gas,
        line: { color: COLORS.gas, width: 0.5 },
        hovertemplate: 'Gas: $%{y:.1f}/MWh<extra></extra>',
      },
      {
        x: targets,
        y: points.map((p) => p.solar_lcoe || 0),
        name: 'Solar',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.solar,
        line: { color: COLORS.solar, width: 0.5 },
        hovertemplate: 'Solar: $%{y:.1f}/MWh<extra></extra>',
      },
      {
        x: targets,
        y: points.map((p) => p.wind_lcoe || 0),
        name: 'Wind',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.wind,
        line: { color: COLORS.wind, width: 0.5 },
        hovertemplate: 'Wind: $%{y:.1f}/MWh<extra></extra>',
      },
      {
        x: targets,
        y: points.map((p) => p.storage_lcoe || 0),
        name: 'Storage',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.storage,
        line: { color: COLORS.storage, width: 0.5 },
        hovertemplate: 'Storage: $%{y:.1f}/MWh<extra></extra>',
      },
      {
        x: targets,
        y: points.map((p) => p.clean_firm_lcoe || 0),
        name: 'Clean Firm',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.cleanFirm,
        line: { color: COLORS.cleanFirm, width: 0.5 },
        hovertemplate: 'Clean Firm: $%{y:.1f}/MWh<extra></extra>',
      },
    ];

    // Add total LCOE line on top
    traces.push({
      x: targets,
      y: points.map((p) => p.lcoe),
      name: 'Total LCOE',
      type: 'scatter',
      mode: 'lines+markers',
      line: { color: '#333', width: 2 },
      marker: { size: 5, color: '#333' },
      hovertemplate: 'Total: $%{y:.1f}/MWh<extra></extra>',
    });

    if (savedSweep) {
      traces.push({
        x: savedSweep.points.map((p) => p.target),
        y: savedSweep.points.map((p) => p.lcoe),
        name: `LCOE (${savedLabel})`,
        type: 'scatter',
        mode: 'lines',
        line: { color: '#999', width: 1, dash: 'dot' },
      });
    }
  } else {
    // Stacked area composition - with MW/MWh toggle
    const storageValues = compositionUnit === 'mw'
      ? points.map((p) => p.storage / 4) // Convert MWh to MW (4-hour storage)
      : points.map((p) => p.storage);    // Keep as MWh

    // For MWh view, approximate annual energy production
    // Solar: ~20% capacity factor, Wind: ~35%, Clean Firm: ~90%
    const solarValues = compositionUnit === 'mw'
      ? points.map((p) => p.solar)
      : points.map((p) => p.solar * 8760 * 0.20); // Annual MWh

    const windValues = compositionUnit === 'mw'
      ? points.map((p) => p.wind)
      : points.map((p) => p.wind * 8760 * 0.35);

    const cfValues = compositionUnit === 'mw'
      ? points.map((p) => p.clean_firm)
      : points.map((p) => p.clean_firm * 8760 * 0.90);

    traces = [
      {
        x: targets,
        y: solarValues,
        name: 'Solar',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.solar,
        line: { color: COLORS.solar, width: 0 },
      },
      {
        x: targets,
        y: windValues,
        name: 'Wind',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.wind,
        line: { color: COLORS.wind, width: 0 },
      },
      {
        x: targets,
        y: storageValues,
        name: compositionUnit === 'mw' ? 'Storage (MW)' : 'Storage (MWh)',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.storage,
        line: { color: COLORS.storage, width: 0 },
      },
      {
        x: targets,
        y: cfValues,
        name: 'Clean Firm',
        type: 'scatter',
        mode: 'lines',
        stackgroup: 'one',
        fillcolor: COLORS.cleanFirm,
        line: { color: COLORS.cleanFirm, width: 0 },
      },
    ];
  }

  const getYAxisTitle = () => {
    if (viewMode === 'lcoe') return '$/MWh';
    if (viewMode === 'composition') {
      return compositionUnit === 'mw' ? 'Capacity (MW)' : 'Annual Energy (MWh)';
    }
    return 'Capacity (MW)';
  };

  const layout: any = {
    height: 350,
    margin: { t: 40, r: 30, b: 60, l: 60 },
    xaxis: {
      title: { text: 'Clean Match Target (%)' },
      range: [0, 100],
    },
    yaxis: {
      title: { text: getYAxisTitle() },
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
        text: `Sweep completed in ${sweepResult.elapsed_ms.toFixed(0)}ms`,
        showarrow: false,
        font: { size: 11, color: '#666' },
        xanchor: 'right',
      },
    ],
  };

  return (
    <div>
      {/* Resource toggles */}
      <div className="px-4 mb-3">{resourceToggles}</div>

      {/* Controls */}
      <div className="flex flex-wrap justify-between items-center mb-4 px-4 gap-4">
        <div className="flex items-center gap-4">
          {/* View mode toggle */}
          <div className="inline-flex rounded-md shadow-sm" role="group">
            {(['capacity', 'lcoe', 'composition'] as ViewMode[]).map((mode) => (
              <button
                key={mode}
                type="button"
                onClick={() => setViewMode(mode)}
                className={`px-3 py-1.5 text-xs font-medium border ${
                  mode === 'capacity'
                    ? 'rounded-l-md'
                    : mode === 'composition'
                      ? 'rounded-r-md'
                      : ''
                } ${
                  viewMode === mode
                    ? 'bg-blue-600 text-white border-blue-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                {mode === 'capacity' ? 'Capacity' : mode === 'lcoe' ? 'LCOE' : 'Composition'}
              </button>
            ))}
          </div>

          {/* MW/MWh toggle for composition view */}
          {viewMode === 'composition' && (
            <div className="inline-flex rounded-md shadow-sm" role="group">
              <button
                type="button"
                onClick={() => setCompositionUnit('mw')}
                className={`px-2 py-1 text-xs font-medium border rounded-l-md ${
                  compositionUnit === 'mw'
                    ? 'bg-gray-600 text-white border-gray-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                MW
              </button>
              <button
                type="button"
                onClick={() => setCompositionUnit('mwh')}
                className={`px-2 py-1 text-xs font-medium border rounded-r-md ${
                  compositionUnit === 'mwh'
                    ? 'bg-gray-600 text-white border-gray-600'
                    : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
                }`}
              >
                MWh
              </button>
            </div>
          )}

          <label className="flex items-center gap-2 text-xs">
            <input
              type="checkbox"
              checked={useFineTargets}
              onChange={(e) => setUseFineTargets(e.target.checked)}
              className="rounded"
            />
            Fine (5%)
          </label>
        </div>

        <div className="flex items-center gap-2">
          {/* Save comparison */}
          <input
            type="text"
            placeholder="Comparison label..."
            value={saveLabel}
            onChange={(e) => setSaveLabel(e.target.value)}
            className="text-xs border rounded px-2 py-1 w-32"
          />
          <button
            onClick={handleSave}
            disabled={!saveLabel.trim()}
            className="text-xs px-2 py-1 bg-gray-100 rounded hover:bg-gray-200 disabled:opacity-50"
          >
            Save
          </button>
          {savedSweep && (
            <button
              onClick={clearSavedComparison}
              className="text-xs px-2 py-1 text-red-600 hover:bg-red-50 rounded"
            >
              Clear
            </button>
          )}

          <button
            onClick={handleRun}
            className="text-xs px-3 py-1.5 bg-blue-600 text-white rounded hover:bg-blue-700"
          >
            Run Sweep
          </button>
        </div>
      </div>

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
              <th className="text-left py-2 px-2">Target</th>
              <th className="text-right py-2 px-2">Solar</th>
              <th className="text-right py-2 px-2">Wind</th>
              <th className="text-right py-2 px-2">Storage</th>
              <th className="text-right py-2 px-2">CF</th>
              <th className="text-right py-2 px-2">LCOE</th>
              <th className="text-right py-2 px-2">Achieved</th>
            </tr>
          </thead>
          <tbody>
            {points
              .filter((_, i) => i % (useFineTargets ? 2 : 1) === 0 || points.length <= 11)
              .map((p) => (
                <tr key={p.target} className="border-b border-gray-100 hover:bg-gray-50">
                  <td className="py-1 px-2">{p.target}%</td>
                  <td className="text-right py-1 px-2">{p.solar.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">{p.wind.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">{p.storage.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">{p.clean_firm.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">${p.lcoe.toFixed(1)}</td>
                  <td
                    className={`text-right py-1 px-2 ${Math.abs(p.achieved - p.target) > 2 ? 'text-red-500' : ''}`}
                  >
                    {p.achieved.toFixed(1)}%
                  </td>
                </tr>
              ))}
          </tbody>
        </table>
      </div>
    </div>
  );
};
