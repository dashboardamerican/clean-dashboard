import React from 'react';
import Plot from 'react-plotly.js';
import { useSweepStore } from '../../stores/sweepStore';
import { COLORS } from '../../types';

/**
 * Capacity Sweep
 *
 * Answers: at the costs currently set in the dashboard, how much of the peak
 * firm-capacity bucket still has to come from gas at every clean-match level
 * from 0 % to 100 %?
 *
 * The peak the system has to firm is the gas value at target = 0 % — that's
 * the load that has to be met when nothing clean is built. From there:
 *
 *   gas_share = gas_capacity_at_target / peak_load × 100
 *
 * Goes from 100 % at clean target = 0, monotonically down toward 0 % near
 * 100 % clean. The shape of the descent — slow until ~95 %, then steep — is
 * the punchline of "the last MW of gas is the hardest to displace".
 *
 * Reuses the optimizer sweep store; running this sweep also fills the
 * Optimizer Sweep view, and vice versa.
 */
export const CapacitySweepChart: React.FC = () => {
  const {
    sweepResult,
    useFineTargets,
    sweepResources,
    isRunning,
    error,
    setUseFineTargets,
    setSweepResource,
    runOptimizerSweep,
  } = useSweepStore();

  const handleRun = async () => {
    await runOptimizerSweep();
  };

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

  if (isRunning) {
    return (
      <div className="h-96 flex items-center justify-center">
        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
        <span className="ml-3 text-gray-600">Running capacity sweep...</span>
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
            d="M3 21v-7m0 0V8a2 2 0 012-2h2v15M3 14h4m6 7V3m0 0H8a2 2 0 00-2 2v3m7-5h4a2 2 0 012 2v13"
          />
        </svg>
        <p className="text-lg font-medium">Capacity Sweep</p>
        <p className="text-sm mt-1 max-w-md text-center">
          For each clean-match target 0–100 %, what share of peak firm capacity
          still has to come from gas in the least-cost portfolio.
        </p>
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
  const gasMW = points.map((p) => p.gas_capacity ?? 0);

  // Peak the system has to firm = gas value at target = 0 (no clean built).
  // Fall back to overall max if a target = 0 row is missing.
  const peakLoad = (() => {
    const at0 = points.find((p) => Math.abs(p.target) < 0.5);
    if (at0 && (at0.gas_capacity ?? 0) > 0) return at0.gas_capacity ?? 0;
    return Math.max(...gasMW, 1);
  })();

  const gasPct = gasMW.map((mw) => (mw / peakLoad) * 100);

  // Punchline annotation
  const find = (t: number) => points.find((p) => Math.abs(p.target - t) < 0.5);
  const pctAt = (t: number) => {
    const p = find(t);
    if (!p) return null;
    return ((p.gas_capacity ?? 0) / peakLoad) * 100;
  };
  const punchlineParts: string[] = [];
  [0, 50, 80, 95, 99, 100].forEach((t) => {
    const v = pctAt(t);
    if (v !== null) punchlineParts.push(`${t}%: ${v.toFixed(0)}% gas`);
  });
  const punchline = punchlineParts.join(' • ');

  const traces: any[] = [
    {
      x: targets,
      y: gasPct,
      name: 'Gas share of peak',
      type: 'scatter',
      mode: 'lines+markers',
      stackgroup: undefined,
      fill: 'tozeroy',
      fillcolor: 'rgba(234, 67, 53, 0.18)',
      line: { color: COLORS.gas, width: 3, shape: 'spline' },
      marker: { size: 7, color: COLORS.gas },
      hovertemplate:
        'Clean target: %{x}%<br>Gas share of peak: <b>%{y:.1f}%</b><extra></extra>',
    },
  ];

  const layout: any = {
    height: 380,
    margin: { t: 60, r: 30, b: 60, l: 60 },
    xaxis: {
      title: { text: 'Clean Match Target (%)' },
      range: [0, 100],
      gridcolor: 'lightgray',
      zeroline: false,
    },
    yaxis: {
      title: { text: 'Gas as % of Peak Capacity' },
      range: [0, 105],
      gridcolor: 'lightgray',
      zeroline: false,
      ticksuffix: '%',
    },
    font: { family: 'Google Sans, Roboto, sans-serif' },
    hovermode: 'x unified',
    showlegend: false,
    annotations: [
      {
        x: 0.01,
        y: 1.14,
        xref: 'paper',
        yref: 'paper',
        text: punchline,
        showarrow: false,
        font: { size: 11, color: '#444' },
        xanchor: 'left',
      },
      {
        x: 1,
        y: 1.14,
        xref: 'paper',
        yref: 'paper',
        text: `Peak: ${peakLoad.toFixed(0)} MW · sweep: ${sweepResult.elapsed_ms.toFixed(0)}ms`,
        showarrow: false,
        font: { size: 11, color: '#666' },
        xanchor: 'right',
      },
    ],
  };

  return (
    <div>
      {/* Controls */}
      <div className="flex flex-wrap justify-between items-center mb-4 px-4 gap-4">
        <div className="flex flex-wrap items-center gap-4">
          {resourceToggles}
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

        <button
          onClick={handleRun}
          className="text-xs px-3 py-1.5 bg-blue-600 text-white rounded hover:bg-blue-700"
        >
          Run Sweep
        </button>
      </div>

      <Plot
        data={traces}
        layout={layout}
        config={{ responsive: true, displayModeBar: false }}
        style={{ width: '100%' }}
      />

      {/* Summary table */}
      <div className="mt-4 px-4 overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b bg-gray-50">
              <th className="text-left py-2 px-2">Clean Target</th>
              <th className="text-right py-2 px-2">Gas (MW)</th>
              <th className="text-right py-2 px-2">Gas % of Peak</th>
              <th className="text-right py-2 px-2">Solar (MW)</th>
              <th className="text-right py-2 px-2">Wind (MW)</th>
              <th className="text-right py-2 px-2">Storage (MWh)</th>
              <th className="text-right py-2 px-2">CF (MW)</th>
              <th className="text-right py-2 px-2">LCOE</th>
            </tr>
          </thead>
          <tbody>
            {points.map((p) => {
              const gas = p.gas_capacity ?? 0;
              const pct = (gas / peakLoad) * 100;
              return (
                <tr key={p.target} className="border-b border-gray-100 hover:bg-gray-50">
                  <td className="py-1 px-2">{p.target}%</td>
                  <td className="text-right py-1 px-2 text-red-700">{gas.toFixed(0)}</td>
                  <td className="text-right py-1 px-2 text-red-700 font-medium">
                    {pct.toFixed(1)}%
                  </td>
                  <td className="text-right py-1 px-2">{p.solar.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">{p.wind.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">{p.storage.toFixed(0)}</td>
                  <td className="text-right py-1 px-2">{p.clean_firm.toFixed(0)}</td>
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
