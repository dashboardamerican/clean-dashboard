import React from 'react';
import { ElccResult, ElccMethod } from '../../types';
import { useMetricsStore } from '../../stores/metricsStore';

interface ElccAnalysisMetricProps {
  elccResult: ElccResult;
}

// Method descriptions for tooltips/info
const METHOD_DESCRIPTIONS = {
  [ElccMethod.FirstIn]: 'Standalone: Resource alone vs baseline',
  [ElccMethod.Marginal]: 'Last-In: Adding 10MW to full portfolio',
  [ElccMethod.Contribution]: 'Removal: Portfolio minus portfolio-without-resource',
  [ElccMethod.Delta]: 'E3 Method: Marginal + proportional interactive effects',
};

const METHOD_LABELS = {
  [ElccMethod.FirstIn]: 'First-In',
  [ElccMethod.Marginal]: 'Marginal',
  [ElccMethod.Contribution]: 'Contrib',
  [ElccMethod.Delta]: 'Delta',
};

export const ElccAnalysisMetric: React.FC<ElccAnalysisMetricProps> = ({
  elccResult,
}) => {
  const elccMethod = useMetricsStore((state) => state.elccMethod);
  const setElccMethod = useMetricsStore((state) => state.setElccMethod);

  // Get the appropriate ELCC value based on selected method
  const getElccValue = (resource: 'solar' | 'wind' | 'storage' | 'clean_firm'): number => {
    const data = elccResult[resource];
    switch (elccMethod) {
      case ElccMethod.FirstIn:
        return data.first_in;
      case ElccMethod.Marginal:
        return data.marginal;
      case ElccMethod.Contribution:
        return data.contribution;
      case ElccMethod.Delta:
        return data.delta;
      default:
        return data.contribution;
    }
  };

  const formatPercent = (value: number): string => {
    return value.toFixed(1) + '%';
  };

  const formatMw = (value: number): string => {
    return value.toFixed(1) + ' MW';
  };

  const resources = [
    { key: 'solar' as const, label: 'Solar', color: '#fbbc05' },
    { key: 'wind' as const, label: 'Wind', color: '#4285f4' },
    { key: 'storage' as const, label: 'Storage', color: '#673ab7' },
    { key: 'clean_firm' as const, label: 'Clean Firm', color: '#FF7900' },
  ];

  const methods = [ElccMethod.FirstIn, ElccMethod.Marginal, ElccMethod.Contribution, ElccMethod.Delta];

  return (
    <div className="bg-gray-50 rounded-lg p-3">
      {/* Header with 4-way toggle */}
      <div className="flex items-center justify-between mb-2">
        <div className="text-xs text-gray-500 uppercase tracking-wide font-medium">
          ELCC Analysis
        </div>
        {/* 4-way button group */}
        <div className="flex rounded-md overflow-hidden border border-gray-300">
          {methods.map((method) => (
            <button
              key={method}
              onClick={() => setElccMethod(method)}
              title={METHOD_DESCRIPTIONS[method]}
              className={`px-2 py-1 text-xs transition-colors ${
                elccMethod === method
                  ? 'bg-blue-600 text-white'
                  : 'bg-white text-gray-600 hover:bg-gray-100'
              } ${method !== ElccMethod.FirstIn ? 'border-l border-gray-300' : ''}`}
            >
              {METHOD_LABELS[method]}
            </button>
          ))}
        </div>
      </div>

      {/* Resource ELCC values in 4-column layout */}
      <div className="grid grid-cols-4 gap-2 mb-3">
        {resources.map(({ key, label, color }) => (
          <div key={key} className="text-center">
            <div
              className="text-xs font-medium mb-1"
              style={{ color }}
            >
              {label}
            </div>
            <div className="text-lg font-semibold text-gray-900">
              {formatPercent(getElccValue(key))}
            </div>
          </div>
        ))}
      </div>

      {/* Portfolio metrics */}
      <div className="border-t pt-2 mt-2 grid grid-cols-2 gap-2 text-xs">
        <div>
          <span className="text-gray-500">Portfolio ELCC:</span>
          <span className="ml-1 font-medium">{formatMw(elccResult.portfolio_elcc_mw)}</span>
        </div>
        <div>
          <span className="text-gray-500">Diversity Benefit:</span>
          <span className={`ml-1 font-medium ${elccResult.diversity_benefit_mw >= 0 ? 'text-green-600' : 'text-orange-600'}`}>
            {elccResult.diversity_benefit_mw >= 0 ? '+' : ''}{formatMw(elccResult.diversity_benefit_mw)}
          </span>
        </div>
      </div>
    </div>
  );
};
