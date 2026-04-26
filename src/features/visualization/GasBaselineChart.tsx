import React, { useMemo } from 'react';
import Plot from 'react-plotly.js';
import { useSimulationStore } from '../../stores/simulationStore';
import { useSettingsStore } from '../../stores/settingsStore';
import { COLORS } from '../../types';

interface GasBaselineChartProps {
  week: number;
}

export const GasBaselineChart: React.FC<GasBaselineChartProps> = ({ week }) => {
  const simulationResult = useSimulationStore((state) => state.simulationResult);
  const loadProfile = useSimulationStore((state) => state.loadProfile);
  const costs = useSettingsStore((state) => state.costs);

  const chartData = useMemo(() => {
    if (!simulationResult) return null;

    const hoursPerWeek = week === 52 ? 192 : 168;
    const startHour = (week - 1) * 168;
    const endHour = Math.min(startHour + hoursPerWeek, 8760);
    const hourCount = endHour - startHour;

    const hours = Array.from({ length: hourCount }, (_, i) => i);
    const solar = simulationResult.solar_out.slice(startHour, endHour);
    const wind = simulationResult.wind_out.slice(startHour, endHour);
    const cleanFirm = simulationResult.clean_firm_generation.slice(startHour, endHour);
    const demandResponse = simulationResult.demand_response.slice(startHour, endHour);
    const gasGeneration = simulationResult.gas_generation.slice(startHour, endHour);
    const gasForCharging = simulationResult.gas_for_charging.slice(startHour, endHour);
    const load = loadProfile.slice(startHour, endHour);

    const netLoad = load.map((value, i) => Math.max(0, value - demandResponse[i]));
    const gasForLoad = gasGeneration.map((value, i) => Math.max(0, value - gasForCharging[i]));
    const gasBaseline = netLoad.map((value, i) =>
      Math.max(0, value - solar[i] - wind[i] - cleanFirm[i])
    );

    const ccsFraction = Math.min(1, Math.max(0, costs.ccs_percentage / 100));
    const gasServingLoadWithCcs = gasForLoad.map((value) => value * ccsFraction);
    const gasServingLoadWithoutCcs = gasForLoad.map((value) => value * (1 - ccsFraction));
    const gasChargingWithCcs = gasForCharging.map((value) => value * ccsFraction);
    const gasChargingWithoutCcs = gasForCharging.map((value) => value * (1 - ccsFraction));

    const netGasDeltaMwh = gasBaseline.reduce(
      (sum, baseline, i) => sum + (baseline - gasGeneration[i]),
      0,
    );
    const peakBaseline = gasBaseline.reduce((max, value) => Math.max(max, value), 0);
    const peakActual = gasGeneration.reduce((max, value) => Math.max(max, value), 0);

    const dayNames = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun', 'Mon'];
    const tickvals: number[] = [];
    const ticktext: string[] = [];
    for (let day = 0; day < Math.ceil(hourCount / 24); day++) {
      tickvals.push(day * 24 + 12);
      ticktext.push(dayNames[day % dayNames.length]);
    }

    return {
      hours,
      gasBaseline,
      gasServingLoadWithCcs,
      gasServingLoadWithoutCcs,
      gasChargingWithCcs,
      gasChargingWithoutCcs,
      netGasDeltaMwh,
      peakBaseline,
      peakActual,
      tickvals,
      ticktext,
      ccsEnabled: ccsFraction > 0,
    };
  }, [simulationResult, loadProfile, costs.ccs_percentage, week]);

  if (!chartData) {
    return <div className="h-96 flex items-center justify-center text-gray-500">No data</div>;
  }

  const traces = [
    {
      x: chartData.hours,
      y: chartData.gasServingLoadWithoutCcs,
      name: chartData.ccsEnabled ? 'Gas Serving Load' : 'Gas',
      type: 'bar' as const,
      marker: { color: COLORS.gas },
    },
    ...(chartData.ccsEnabled
      ? [{
          x: chartData.hours,
          y: chartData.gasServingLoadWithCcs,
          name: 'Gas + CCS Serving Load',
          type: 'bar' as const,
          marker: { color: COLORS.gasCcs },
        }]
      : []),
    {
      x: chartData.hours,
      y: chartData.gasChargingWithoutCcs,
      name: chartData.ccsEnabled ? 'Gas for Charging' : 'Gas for Charging',
      type: 'bar' as const,
      marker: { color: '#7e57c2' },
    },
    ...(chartData.ccsEnabled
      ? [{
          x: chartData.hours,
          y: chartData.gasChargingWithCcs,
          name: 'Gas + CCS for Charging',
          type: 'bar' as const,
          marker: { color: '#b794f4' },
        }]
      : []),
    {
      x: chartData.hours,
      y: chartData.gasBaseline,
      name: 'Gas Baseline (No Battery)',
      type: 'scatter' as const,
      mode: 'lines' as const,
      line: { color: '#111827', width: 2, dash: 'dash' as const },
    },
  ];

  return (
    <Plot
      data={traces}
      layout={{
        barmode: 'stack',
        height: 400,
        margin: { t: 36, r: 24, b: 56, l: 60 },
        xaxis: {
          tickmode: 'array',
          tickvals: chartData.tickvals,
          ticktext: chartData.ticktext,
        },
        yaxis: {
          title: { text: 'Gas Generation (MW)' },
        },
        legend: {
          orientation: 'h',
          y: -0.18,
          x: 0.5,
          xanchor: 'center',
        },
        font: {
          family: 'Google Sans, Roboto, sans-serif',
        },
        annotations: [
          {
            x: 1,
            y: 1.05,
            xref: 'paper',
            yref: 'paper',
            xanchor: 'right',
            showarrow: false,
            text: `Net gas delta: ${chartData.netGasDeltaMwh >= 0 ? '+' : ''}${chartData.netGasDeltaMwh.toFixed(0)} MWh | Peak baseline: ${chartData.peakBaseline.toFixed(1)} MW | Peak actual: ${chartData.peakActual.toFixed(1)} MW`,
            font: { size: 12, color: '#4b5563' },
          },
        ],
      }}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%' }}
    />
  );
};
