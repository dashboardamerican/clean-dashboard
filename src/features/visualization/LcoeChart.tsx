import React, { useMemo, useState } from 'react';
import Plot from 'react-plotly.js';
import { useSimulationStore } from '../../stores/simulationStore';
import { COLORS, TechnologyCostBreakdown } from '../../types';

// Cost category definition
interface CostCategory {
  key: keyof TechnologyCostBreakdown;
  label: string;
  brightness: number;
  isNegative?: boolean;
}

// Cost category colors - based on technology color with varying opacity
const COST_CATEGORIES: CostCategory[] = [
  { key: 'capex', label: 'CAPEX', brightness: 1.0 },
  { key: 'fixed_om', label: 'Fixed O&M', brightness: 0.85 },
  { key: 'var_om', label: 'Variable O&M', brightness: 0.7 },
  { key: 'fuel', label: 'Fuel', brightness: 0.55 },
  { key: 'itc_benefit', label: 'ITC Benefit', brightness: 1.0, isNegative: true },
  { key: 'tax_shield', label: 'Tax Shield', brightness: 0.8, isNegative: true },
];

// Adjust color brightness
function adjustBrightness(hex: string, factor: number): string {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);

  const adjust = (c: number) => Math.min(255, Math.round(c * factor));

  return `rgb(${adjust(r)}, ${adjust(g)}, ${adjust(b)})`;
}

// Create striped pattern for negative values
function getNegativeColor(baseColor: string): string {
  // For negative values (benefits), use a lighter, more desaturated version
  const r = parseInt(baseColor.slice(1, 3), 16);
  const g = parseInt(baseColor.slice(3, 5), 16);
  const b = parseInt(baseColor.slice(5, 7), 16);

  // Lighten and desaturate
  const lighten = (c: number) => Math.min(255, Math.round(c * 0.5 + 127));

  return `rgb(${lighten(r)}, ${lighten(g)}, ${lighten(b)})`;
}

type ViewMode = 'byTechnology' | 'byCategory';

export const LcoeChart: React.FC = () => {
  const lcoeResult = useSimulationStore((state) => state.lcoeResult);
  const [viewMode, setViewMode] = useState<ViewMode>('byTechnology');

  const chartData = useMemo(() => {
    if (!lcoeResult) return null;

    // Get breakdowns for each technology
    const technologies = [
      { name: 'Solar', breakdown: lcoeResult.solar_breakdown, color: COLORS.solar },
      { name: 'Wind', breakdown: lcoeResult.wind_breakdown, color: COLORS.wind },
      { name: 'Storage', breakdown: lcoeResult.storage_breakdown, color: COLORS.storage },
      { name: 'Clean Firm', breakdown: lcoeResult.clean_firm_breakdown, color: COLORS.cleanFirm },
      { name: 'Gas', breakdown: lcoeResult.gas_breakdown, color: COLORS.gas },
      { name: 'CCS', breakdown: lcoeResult.ccs_breakdown, color: COLORS.gasCcs },
    ].filter(t => t.breakdown && t.breakdown.total !== 0);

    return technologies;
  }, [lcoeResult]);

  if (!chartData || chartData.length === 0) {
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
        <p className="text-lg font-medium">No LCOE Data</p>
        <p className="text-sm mt-1">Add generation capacity to see cost breakdown</p>
      </div>
    );
  }

  const totalLcoe = lcoeResult?.total_lcoe || 0;

  // Build traces based on view mode
  const traces = [];

  if (viewMode === 'byTechnology') {
    // Grouped by technology, stacked by cost category
    for (const category of COST_CATEGORIES) {
      const values: number[] = [];
      const colors: string[] = [];
      const names: string[] = [];
      const hovertexts: string[] = [];

      for (const tech of chartData) {
        const breakdown = tech.breakdown;
        const value = breakdown[category.key] as number;

        if (Math.abs(value) > 0.001) {
          values.push(Math.abs(value));
          const baseColor = category.isNegative
            ? getNegativeColor(tech.color)
            : adjustBrightness(tech.color, category.brightness);
          colors.push(baseColor);
          names.push(tech.name);
          hovertexts.push(`${tech.name} ${category.label}: $${value.toFixed(2)}/MWh`);
        } else {
          values.push(0);
          colors.push(tech.color);
          names.push(tech.name);
          hovertexts.push('');
        }
      }

      traces.push({
        x: chartData.map(t => t.name),
        y: values,
        name: category.label,
        type: 'bar' as const,
        marker: {
          color: colors,
          line: category.isNegative ? { color: 'rgba(0,0,0,0.3)', width: 1 } : undefined,
          pattern: category.isNegative ? { shape: '/' } : undefined,
        },
        hovertemplate: hovertexts.map((t) => t || `%{x} ${category.label}: $%{y:.2f}/MWh<extra></extra>`),
        showlegend: values.some(v => v > 0),
      });
    }
  } else {
    // Grouped by category, stacked by technology
    for (const tech of chartData) {
      const breakdown = tech.breakdown;
      const values: number[] = [];
      const colors: string[] = [];

      for (const category of COST_CATEGORIES) {
        const value = breakdown[category.key] as number;
        values.push(Math.abs(value));
        colors.push(
          category.isNegative
            ? getNegativeColor(tech.color)
            : adjustBrightness(tech.color, category.brightness)
        );
      }

      traces.push({
        x: COST_CATEGORIES.map(c => c.label),
        y: values,
        name: tech.name,
        type: 'bar' as const,
        marker: { color: tech.color },
        hovertemplate: `${tech.name}: $%{y:.2f}/MWh<extra></extra>`,
      });
    }
  }

  const layout = {
    height: 350,
    margin: { t: 40, r: 30, b: 80, l: 60 },
    barmode: 'stack' as const,
    xaxis: {
      title: { text: '' },
      tickangle: viewMode === 'byCategory' ? -45 : 0,
    },
    yaxis: {
      title: { text: '$/MWh' },
    },
    font: {
      family: 'Google Sans, Roboto, sans-serif',
    },
    legend: {
      orientation: 'h' as const,
      y: -0.25,
      x: 0.5,
      xanchor: 'center' as const,
    },
    annotations: [
      {
        x: 1,
        y: 1.02,
        xref: 'paper' as const,
        yref: 'paper' as const,
        text: `Total LCOE: $${totalLcoe.toFixed(1)}/MWh`,
        showarrow: false,
        font: { size: 14, color: '#333', weight: 600 },
        xanchor: 'right' as const,
      },
    ],
  };

  return (
    <div>
      {/* View mode toggle */}
      <div className="flex justify-end mb-2 px-4">
        <div className="inline-flex rounded-md shadow-sm" role="group">
          <button
            type="button"
            onClick={() => setViewMode('byTechnology')}
            className={`px-3 py-1.5 text-xs font-medium rounded-l-md border ${
              viewMode === 'byTechnology'
                ? 'bg-blue-600 text-white border-blue-600'
                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
            }`}
          >
            By Technology
          </button>
          <button
            type="button"
            onClick={() => setViewMode('byCategory')}
            className={`px-3 py-1.5 text-xs font-medium rounded-r-md border-t border-r border-b ${
              viewMode === 'byCategory'
                ? 'bg-blue-600 text-white border-blue-600'
                : 'bg-white text-gray-700 border-gray-300 hover:bg-gray-50'
            }`}
          >
            By Category
          </button>
        </div>
      </div>

      <Plot
        data={traces}
        layout={layout}
        config={{
          responsive: true,
          displayModeBar: false,
        }}
        style={{ width: '100%' }}
      />

      {/* Detailed breakdown table */}
      <div className="mt-4 px-4 overflow-x-auto">
        <table className="w-full text-xs">
          <thead>
            <tr className="border-b bg-gray-50">
              <th className="text-left py-2 px-2">Technology</th>
              <th className="text-right py-2 px-1">CAPEX</th>
              <th className="text-right py-2 px-1">Fixed O&M</th>
              <th className="text-right py-2 px-1">Var O&M</th>
              <th className="text-right py-2 px-1">Fuel</th>
              <th className="text-right py-2 px-1">ITC</th>
              <th className="text-right py-2 px-1">Tax Shield</th>
              <th className="text-right py-2 px-2 font-semibold">Total</th>
            </tr>
          </thead>
          <tbody>
            {chartData.map((tech) => {
              const b = tech.breakdown;
              return (
                <tr key={tech.name} className="border-b border-gray-100 hover:bg-gray-50">
                  <td className="py-2 px-2 flex items-center gap-2">
                    <span
                      className="w-3 h-3 rounded-sm flex-shrink-0"
                      style={{ backgroundColor: tech.color }}
                    />
                    {tech.name}
                  </td>
                  <td className="text-right py-2 px-1">${b.capex.toFixed(2)}</td>
                  <td className="text-right py-2 px-1">${b.fixed_om.toFixed(2)}</td>
                  <td className="text-right py-2 px-1">${b.var_om.toFixed(2)}</td>
                  <td className="text-right py-2 px-1">${b.fuel.toFixed(2)}</td>
                  <td className="text-right py-2 px-1 text-green-600">
                    {b.itc_benefit !== 0 ? `$${b.itc_benefit.toFixed(2)}` : '-'}
                  </td>
                  <td className="text-right py-2 px-1 text-green-600">
                    {b.tax_shield !== 0 ? `$${b.tax_shield.toFixed(2)}` : '-'}
                  </td>
                  <td className="text-right py-2 px-2 font-semibold">${b.total.toFixed(2)}</td>
                </tr>
              );
            })}
            <tr className="font-semibold bg-gray-50">
              <td className="py-2 px-2">Total</td>
              <td className="text-right py-2 px-1">
                ${chartData.reduce((s, t) => s + t.breakdown.capex, 0).toFixed(2)}
              </td>
              <td className="text-right py-2 px-1">
                ${chartData.reduce((s, t) => s + t.breakdown.fixed_om, 0).toFixed(2)}
              </td>
              <td className="text-right py-2 px-1">
                ${chartData.reduce((s, t) => s + t.breakdown.var_om, 0).toFixed(2)}
              </td>
              <td className="text-right py-2 px-1">
                ${chartData.reduce((s, t) => s + t.breakdown.fuel, 0).toFixed(2)}
              </td>
              <td className="text-right py-2 px-1 text-green-600">
                ${chartData.reduce((s, t) => s + t.breakdown.itc_benefit, 0).toFixed(2)}
              </td>
              <td className="text-right py-2 px-1 text-green-600">
                ${chartData.reduce((s, t) => s + t.breakdown.tax_shield, 0).toFixed(2)}
              </td>
              <td className="text-right py-2 px-2">${totalLcoe.toFixed(2)}</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
  );
};
