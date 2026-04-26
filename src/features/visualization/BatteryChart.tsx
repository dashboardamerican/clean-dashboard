import React, { useMemo } from 'react';
import Plot from 'react-plotly.js';
import { useSimulationStore } from '../../stores/simulationStore';
import { COLORS, HOURS_PER_YEAR } from '../../types';

export const BatteryChart: React.FC = () => {
  const simulationResult = useSimulationStore((state) => state.simulationResult);
  const config = useSimulationStore((state) => state.config);

  const chartData = useMemo(() => {
    if (!simulationResult || config.storage_capacity === 0) return null;

    const batteryCharge = simulationResult.battery_charge;
    const batteryDischarge = simulationResult.battery_discharge;
    const storageCapacity = config.storage_capacity;

    // Calculate state of charge
    // Battery starts at 100% capacity
    const soc: number[] = [];
    let currentSoc = storageCapacity;

    for (let t = 0; t < HOURS_PER_YEAR; t++) {
      currentSoc += batteryCharge[t] - batteryDischarge[t];
      currentSoc = Math.max(0, Math.min(currentSoc, storageCapacity));
      soc.push(currentSoc);
    }

    // Convert to percentage
    const socPercentage = soc.map((s) => (s / storageCapacity) * 100);

    // Create x-axis (hours)
    const hours = Array.from({ length: HOURS_PER_YEAR }, (_, i) => i);

    // Create tick labels at 1000-hour intervals
    const tickvals = Array.from({ length: 9 }, (_, i) => i * 1000);
    const ticktext = tickvals.map((v) => v.toString());

    return {
      hours,
      socPercentage,
      tickvals,
      ticktext,
    };
  }, [simulationResult, config.storage_capacity]);

  if (!chartData) {
    return (
      <div className="h-96 flex flex-col items-center justify-center text-gray-500">
        <svg className="w-16 h-16 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={1}
            d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
          />
        </svg>
        <p className="text-lg font-medium">No Battery Storage</p>
        <p className="text-sm mt-1">Add storage capacity to see battery state of charge</p>
      </div>
    );
  }

  const trace = {
    x: chartData.hours,
    y: chartData.socPercentage,
    type: 'scatter' as const,
    mode: 'lines' as const,
    name: 'Battery SOC (%)',
    line: { color: COLORS.storage, width: 1 },
    fill: 'tozeroy' as const,
    fillcolor: 'rgba(103, 58, 183, 0.2)',
  };

  const layout = {
    height: 440,
    margin: { t: 30, r: 30, b: 50, l: 60 },
    xaxis: {
      title: { text: 'Hour of the Year' },
      tickmode: 'array' as const,
      tickvals: chartData.tickvals,
      ticktext: chartData.ticktext,
    },
    yaxis: {
      title: { text: 'State of Charge (%)' },
      range: [0, 100],
    },
    font: {
      family: 'Google Sans, Roboto, sans-serif',
    },
    hovermode: 'x unified' as const,
    plot_bgcolor: 'white',
    paper_bgcolor: 'white',
  };

  return (
    <Plot
      data={[trace]}
      layout={layout}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%' }}
    />
  );
};
