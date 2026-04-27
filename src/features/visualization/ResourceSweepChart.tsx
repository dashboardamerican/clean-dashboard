import React from 'react';
import Plot from 'react-plotly.js';
import { useSweepStore } from '../../stores/sweepStore';
import { COLORS, ResourceSweepResource, ResourceSweepMetric } from '../../types';

const RESOURCE_LABELS: Record<ResourceSweepResource, string> = {
  solar: 'Solar',
  wind: 'Wind',
  storage: 'Storage',
  clean_firm: 'Clean Firm',
};

const RESOURCE_UNITS: Record<ResourceSweepResource, string> = {
  solar: 'MW',
  wind: 'MW',
  storage: 'MWh',
  clean_firm: 'MW',
};

const RESOURCE_COLORS: Record<ResourceSweepResource, string> = {
  solar: COLORS.solar,
  wind: COLORS.wind,
  storage: COLORS.storage,
  clean_firm: COLORS.cleanFirm,
};

const METRIC_LABELS: Record<ResourceSweepMetric, { axis: string; hover: string }> = {
  clean_match: { axis: 'Clean Match (%)', hover: 'Clean Match' },
  lcoe: { axis: 'LCOE ($/MWh)', hover: 'LCOE' },
};

export const ResourceSweepChart: React.FC = () => {
  const {
    resourceSweepResult,
    resourceSweepResource,
    resourceSweepSteps,
    resourceSweepMetric,
    isRunning,
    error,
    setResourceSweepResource,
    setResourceSweepSteps,
    setResourceSweepMetric,
    runResourceSweep,
  } = useSweepStore();

  const handleRun = async () => {
    await runResourceSweep();
  };

  if (isRunning) {
    return (
      <div className="h-96 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-gray-600">Running resource sweep...</span>
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

  const controls = (
    <div className="flex flex-wrap justify-between items-start mb-4 px-4 gap-4">
      <div className="flex flex-wrap items-center gap-4">
        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Resource:</label>
          <select
            value={resourceSweepResource}
            onChange={(e) => setResourceSweepResource(e.target.value as ResourceSweepResource)}
            className="text-sm border rounded px-2 py-1"
          >
            {(Object.keys(RESOURCE_LABELS) as ResourceSweepResource[]).map((value) => (
              <option key={value} value={value}>
                {RESOURCE_LABELS[value]}
              </option>
            ))}
          </select>
        </div>

        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Metric:</label>
          <select
            value={resourceSweepMetric}
            onChange={(e) => setResourceSweepMetric(e.target.value as ResourceSweepMetric)}
            className="text-sm border rounded px-2 py-1"
          >
            <option value="clean_match">Clean Match</option>
            <option value="lcoe">LCOE</option>
          </select>
        </div>

        <div className="flex items-center gap-2">
          <label className="text-xs font-medium text-gray-600">Steps:</label>
          <input
            type="number"
            value={resourceSweepSteps}
            onChange={(e) => setResourceSweepSteps(parseInt(e.target.value, 10) || 2)}
            className="w-14 text-sm border rounded px-2 py-1"
            min="3"
            max="30"
          />
        </div>
      </div>

      <button
        onClick={handleRun}
        className="text-xs px-3 py-1.5 bg-blue-600 text-white rounded hover:bg-blue-700"
      >
        Run Sweep
      </button>
    </div>
  );

  if (!resourceSweepResult) {
    return (
      <div>
        {controls}
        <div className="h-80 flex flex-col items-center justify-center text-gray-500">
          <svg className="w-16 h-16 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={1}
              d="M3 3v18h18M7 14l4-4 4 4 6-6"
            />
          </svg>
          <p className="text-lg font-medium">Resource Sweep</p>
          <p className="text-sm mt-1">
            See how {METRIC_LABELS[resourceSweepMetric].hover.toLowerCase()} changes as one resource ramps
          </p>
        </div>
      </div>
    );
  }

  const {
    resource,
    points,
    fixed_solar,
    fixed_wind,
    fixed_storage,
    fixed_clean_firm,
    current_value,
    elapsed_ms,
  } = resourceSweepResult;

  const metricKey: 'clean_match' | 'lcoe' = resourceSweepMetric;
  const xValues = points.map((p) => p.capacity);
  const yValues = points.map((p) => p[metricKey]);

  const resourceLabel = RESOURCE_LABELS[resource];
  const resourceUnit = RESOURCE_UNITS[resource];
  const resourceColor = RESOURCE_COLORS[resource];
  const yAxis = METRIC_LABELS[resourceSweepMetric].axis;
  const yHover = METRIC_LABELS[resourceSweepMetric].hover;
  const yFmt = resourceSweepMetric === 'lcoe' ? '$%{y:.1f}/MWh' : '%{y:.1f}%';

  // Build subtitle showing fixed values
  const fixedParts: string[] = [];
  ([
    ['solar', 'Solar', fixed_solar, 'MW'],
    ['wind', 'Wind', fixed_wind, 'MW'],
    ['storage', 'Storage', fixed_storage, 'MWh'],
    ['clean_firm', 'Clean Firm', fixed_clean_firm, 'MW'],
  ] as Array<[ResourceSweepResource, string, number, string]>).forEach(([key, label, value, unit]) => {
    const tag = key === resource ? 'Swept' : 'Fixed';
    fixedParts.push(`${label}: ${value.toFixed(0)} ${unit} (${tag})`);
  });
  const subtitle = fixedParts.join(' • ');

  const traces: any[] = [
    {
      x: xValues,
      y: yValues,
      name: `${resourceLabel} sweep`,
      type: 'scatter',
      mode: 'lines+markers',
      line: { color: resourceColor, width: 3 },
      marker: { size: 7, color: resourceColor, line: { color: 'white', width: 1 } },
      hovertemplate: `${resourceLabel}: %{x:.0f} ${resourceUnit}<br>${yHover}: ${yFmt}<extra></extra>`,
    },
  ];

  // y-axis range with 5% padding
  const finiteYs = yValues.filter((y) => Number.isFinite(y));
  let yMin = 0;
  let yMax = 100;
  if (finiteYs.length > 0) {
    const lo = Math.min(...finiteYs);
    const hi = Math.max(...finiteYs);
    const span = hi - lo || 1;
    yMin = Math.max(0, lo - span * 0.05);
    yMax = hi + span * 0.05;
  }

  const layout: any = {
    height: 350,
    margin: { t: 60, r: 30, b: 60, l: 60 },
    xaxis: {
      title: { text: `${resourceLabel} Capacity (${resourceUnit})` },
      gridcolor: 'lightgray',
      zeroline: false,
    },
    yaxis: {
      title: { text: yAxis },
      gridcolor: 'lightgray',
      zeroline: false,
      range: [yMin, yMax],
    },
    font: { family: 'Google Sans, Roboto, sans-serif' },
    legend: { orientation: 'h', y: -0.2, x: 0.5, xanchor: 'center' },
    shapes: [
      // vertical line at the slider's current value
      {
        type: 'line',
        x0: current_value,
        x1: current_value,
        y0: yMin,
        y1: yMax,
        yref: 'y',
        line: { color: '#ea4335', dash: 'dash', width: 2 },
      },
    ],
    annotations: [
      {
        x: 0.01,
        y: 1.1,
        xref: 'paper',
        yref: 'paper',
        text: subtitle,
        showarrow: false,
        font: { size: 11, color: '#555' },
        xanchor: 'left',
      },
      {
        x: 1,
        y: 1.1,
        xref: 'paper',
        yref: 'paper',
        text: `Completed in ${elapsed_ms.toFixed(0)}ms`,
        showarrow: false,
        font: { size: 11, color: '#666' },
        xanchor: 'right',
      },
      {
        x: current_value,
        y: yMax,
        xref: 'x',
        yref: 'y',
        text: `Current: ${current_value.toFixed(0)} ${resourceUnit}`,
        showarrow: false,
        font: { size: 11, color: '#ea4335' },
        xanchor: 'left',
        yanchor: 'top',
      },
    ],
  };

  return (
    <div>
      {controls}

      <Plot
        data={traces}
        layout={layout}
        config={{ responsive: true, displayModeBar: false }}
        style={{ width: '100%' }}
      />

      <div className="mt-4 px-4 overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b bg-gray-50">
              <th className="text-left py-2 px-2">{resourceLabel} Capacity ({resourceUnit})</th>
              <th className="text-right py-2 px-2">Clean Match</th>
              <th className="text-right py-2 px-2">LCOE</th>
            </tr>
          </thead>
          <tbody>
            {points.map((p, i) => {
              const isCurrent = Math.abs(p.capacity - current_value) < 1e-6;
              return (
                <tr
                  key={i}
                  className={`border-b border-gray-100 hover:bg-gray-50 ${isCurrent ? 'bg-red-50' : ''}`}
                >
                  <td className="py-1 px-2">{p.capacity.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">{p.clean_match.toFixed(1)}%</td>
                  <td className="text-right py-1 px-2">${p.lcoe.toFixed(1)}</td>
                </tr>
              );
            })}
          </tbody>
        </table>
      </div>
    </div>
  );
};
