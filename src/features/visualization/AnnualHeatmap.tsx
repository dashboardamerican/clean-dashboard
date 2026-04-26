import React, { useMemo } from 'react';
import Plot from 'react-plotly.js';
import { useSimulationStore } from '../../stores/simulationStore';

// Zone timezone mapping
const ZONE_TIMEZONE_MAP: Record<string, string> = {
  California: 'PST',
  Florida: 'EST',
  'Mid-Atlantic': 'EST',
  Delta: 'CST',
  Southeast: 'EST',
  'New England': 'EST',
  'New York': 'EST',
  Northwest: 'PST',
  Southwest: 'MST',
  Midwest: 'CST',
  Texas: 'CST',
  Mountain: 'MST',
  Plains: 'CST',
};

export const AnnualHeatmap: React.FC = () => {
  const simulationResult = useSimulationStore((state) => state.simulationResult);
  const loadProfile = useSimulationStore((state) => state.loadProfile);
  const zone = useSimulationStore((state) => state.zone);

  const chartData = useMemo(() => {
    if (!simulationResult || !loadProfile) return null;

    const cleanDelivered = simulationResult.clean_delivered;

    // Calculate clean match percentage (cap at 100%)
    const matchPercentage = cleanDelivered.map((cd, i) => {
      const load = loadProfile[i] || 1;
      return Math.min((cd / load) * 100, 100);
    });

    // Reshape into 365 rows x 24 cols
    // z[day][hour] = match percentage
    const heatmapData: number[][] = [];
    for (let day = 0; day < 365; day++) {
      const dayData: number[] = [];
      for (let hour = 0; hour < 24; hour++) {
        const idx = day * 24 + hour;
        dayData.push(idx < matchPercentage.length ? matchPercentage[idx] : 0);
      }
      heatmapData.push(dayData);
    }

    // Transpose for Plotly (z[hour][day])
    const transposed: number[][] = [];
    for (let hour = 0; hour < 24; hour++) {
      transposed.push(heatmapData.map((day) => day[hour]));
    }

    return transposed;
  }, [simulationResult, loadProfile]);

  if (!chartData) {
    return <div className="h-96 flex items-center justify-center text-gray-500">No data</div>;
  }

  // Get timezone abbreviation
  const timezoneAbbr = ZONE_TIMEZONE_MAP[zone] || 'LT';

  // Create hour labels
  const hourLabels = Array(24).fill('');
  hourLabels[0] = `Midnight ${timezoneAbbr}`;
  hourLabels[12] = `Noon ${timezoneAbbr}`;
  hourLabels[23] = `Midnight ${timezoneAbbr}`;

  // Create day labels (only show Winter/Summer)
  const dayLabels = Array(365).fill('');
  dayLabels[0] = 'Winter';
  dayLabels[182] = 'Summer';
  dayLabels[364] = 'Winter';

  const trace = {
    z: chartData,
    x: Array.from({ length: 365 }, (_, i) => i),
    y: Array.from({ length: 24 }, (_, i) => i),
    type: 'heatmap' as const,
    colorscale: 'Greens' as const,
    zmin: 0,
    zmax: 100,
    colorbar: {
      title: { text: 'Clean<br>Match (%)', font: { size: 12 } },
    },
    hovertemplate: 'Day: %{x}<br>Hour: %{y}<br>Match: %{z:.1f}%<extra></extra>',
  };

  const layout = {
    height: 440,
    margin: { t: 30, r: 80, b: 50, l: 80 },
    xaxis: {
      title: { text: 'Day of the Year' },
      tickmode: 'array' as const,
      tickvals: [0, 182, 364],
      ticktext: ['Winter', 'Summer', 'Winter'],
    },
    yaxis: {
      title: { text: 'Hour of Day' },
      autorange: 'reversed' as const,
      tickmode: 'array' as const,
      tickvals: [0, 12, 23],
      ticktext: [hourLabels[0], hourLabels[12], hourLabels[23]],
    },
    font: {
      family: 'Google Sans, Roboto, sans-serif',
    },
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
