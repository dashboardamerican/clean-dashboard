import React from 'react';
import { Select } from '../atoms';
import { useUiStore } from '../../stores/uiStore';
import { useSimulationStore } from '../../stores/simulationStore';
import { VisualizationType } from '../../types';
import { WeeklyChart } from '../../features/visualization/WeeklyChart';
import { AnnualHeatmap } from '../../features/visualization/AnnualHeatmap';
import { BatteryChart } from '../../features/visualization/BatteryChart';
import { LcoeChart } from '../../features/visualization/LcoeChart';
import { PriceChart } from '../../features/visualization/PriceChart';
import { GasBaselineChart } from '../../features/visualization/GasBaselineChart';
import { OptimizerSweepChart } from '../../features/visualization/OptimizerSweepChart';
import { CostSweepChart } from '../../features/visualization/CostSweepChart';

const VISUALIZATION_OPTIONS: { value: VisualizationType; label: string }[] = [
  { value: 'weekly', label: 'Weekly Operation' },
  { value: 'heatmap', label: 'Annual Heatmap' },
  { value: 'battery', label: 'Battery Profile' },
  { value: 'lcoe', label: 'LCOE Components' },
  { value: 'price', label: 'Market Price' },
  { value: 'optimizerSweep', label: 'Optimizer Sweep' },
  { value: 'costSweep', label: 'Cost Sensitivity' },
  { value: 'gasBaseline', label: 'Gas Baseline' },
];

export const VisualizationPanel: React.FC = () => {
  const currentViz = useUiStore((state) => state.currentViz);
  const setVisualization = useUiStore((state) => state.setVisualization);
  const selectedWeek = useUiStore((state) => state.selectedWeek);
  const setSelectedWeek = useUiStore((state) => state.setSelectedWeek);
  const simulationResult = useSimulationStore((state) => state.simulationResult);

  const renderVisualization = () => {
    // Sweep charts don't require simulation result
    if (currentViz === 'optimizerSweep') {
      return <OptimizerSweepChart />;
    }
    if (currentViz === 'costSweep') {
      return <CostSweepChart />;
    }

    if (!simulationResult) {
      return (
        <div className="flex items-center justify-center h-96 text-gray-500">
          Run a simulation to see visualization
        </div>
      );
    }

    switch (currentViz) {
      case 'weekly':
        return <WeeklyChart week={selectedWeek} />;
      case 'heatmap':
        return <AnnualHeatmap />;
      case 'battery':
        return <BatteryChart />;
      case 'lcoe':
        return <LcoeChart />;
      case 'price':
        return <PriceChart />;
      case 'gasBaseline':
        return <GasBaselineChart week={selectedWeek} />;
      default:
        return <PlaceholderChart title="Coming Soon" />;
    }
  };

  return (
    <div className="bg-white rounded-lg shadow">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <Select
          value={currentViz}
          options={VISUALIZATION_OPTIONS}
          onChange={(v) => setVisualization(v as VisualizationType)}
          className="w-48"
        />

        {(currentViz === 'weekly' || currentViz === 'gasBaseline') && (
          <div className="flex items-center gap-2">
            <button
              onClick={() => setSelectedWeek(selectedWeek - 1)}
              disabled={selectedWeek <= 1}
              className="p-1 rounded hover:bg-gray-100 disabled:opacity-50"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M15 19l-7-7 7-7" />
              </svg>
            </button>
            <span className="text-sm font-medium">Week {selectedWeek}</span>
            <button
              onClick={() => setSelectedWeek(selectedWeek + 1)}
              disabled={selectedWeek >= 52}
              className="p-1 rounded hover:bg-gray-100 disabled:opacity-50"
            >
              <svg className="w-5 h-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 5l7 7-7 7" />
              </svg>
            </button>
          </div>
        )}
      </div>

      {/* Chart */}
      <div className="p-4">{renderVisualization()}</div>
    </div>
  );
};

// Placeholder component for unimplemented charts
const PlaceholderChart: React.FC<{ title: string }> = ({ title }) => (
  <div className="flex flex-col items-center justify-center h-96 text-gray-400">
    <svg className="w-16 h-16 mb-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
      <path
        strokeLinecap="round"
        strokeLinejoin="round"
        strokeWidth={1}
        d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
      />
    </svg>
    <p className="text-lg font-medium">{title}</p>
    <p className="text-sm mt-1">Chart implementation in progress</p>
  </div>
);
