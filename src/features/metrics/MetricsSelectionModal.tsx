import React from 'react';
import {
  METRIC_DEFINITIONS,
  METRIC_CATEGORIES,
  MetricCategory,
  getMetricsByCategory,
} from '../../types/metrics';
import { useMetricsStore } from '../../stores/metricsStore';
import { ElccMethod } from '../../types';

interface MetricsSelectionModalProps {
  isOpen: boolean;
  onClose: () => void;
}

export const MetricsSelectionModal: React.FC<MetricsSelectionModalProps> = ({
  isOpen,
  onClose,
}) => {
  const selectedMetrics = useMetricsStore((state) => state.selectedMetrics);
  const toggleMetric = useMetricsStore((state) => state.toggleMetric);
  const setSelectedMetrics = useMetricsStore((state) => state.setSelectedMetrics);
  const elccMethod = useMetricsStore((state) => state.elccMethod);
  const setElccMethod = useMetricsStore((state) => state.setElccMethod);
  const resetToDefaults = useMetricsStore((state) => state.resetToDefaults);

  if (!isOpen) return null;

  const selectAll = () => {
    setSelectedMetrics(METRIC_DEFINITIONS.map(m => m.id));
  };

  const deselectAll = () => {
    setSelectedMetrics([]);
  };

  // Check if any ELCC metrics are selected
  const hasElccMetrics = selectedMetrics.some(id => {
    const def = METRIC_DEFINITIONS.find(m => m.id === id);
    return def?.requiresElcc;
  });

  const categories: MetricCategory[] = [
    'core',
    'system_performance',
    'economic_analysis',
    'reliability_analysis',
    'environmental',
  ];

  return (
    <div className="fixed inset-0 z-50 overflow-y-auto">
      {/* Backdrop */}
      <div
        className="fixed inset-0 bg-black bg-opacity-50 transition-opacity"
        onClick={onClose}
      />

      {/* Modal */}
      <div className="flex min-h-full items-center justify-center p-4">
        <div className="relative bg-white rounded-lg shadow-xl max-w-2xl w-full max-h-[90vh] overflow-hidden">
          {/* Header */}
          <div className="flex items-center justify-between px-6 py-4 border-b">
            <h2 className="text-xl font-semibold text-gray-900">
              Select Metrics
            </h2>
            <button
              onClick={onClose}
              className="text-gray-400 hover:text-gray-500 focus:outline-none"
            >
              <svg className="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
              </svg>
            </button>
          </div>

          {/* Content */}
          <div className="px-6 py-4 overflow-y-auto max-h-[60vh]">
            {/* Action buttons */}
            <div className="flex gap-2 mb-4">
              <button
                onClick={selectAll}
                className="px-3 py-1 text-sm bg-blue-100 text-blue-700 rounded hover:bg-blue-200"
              >
                Select All
              </button>
              <button
                onClick={deselectAll}
                className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
              >
                Deselect All
              </button>
              <button
                onClick={resetToDefaults}
                className="px-3 py-1 text-sm bg-gray-100 text-gray-700 rounded hover:bg-gray-200"
              >
                Reset to Defaults
              </button>
            </div>

            {/* Legend */}
            <div className="flex gap-4 mb-4 text-xs text-gray-500">
              <span><span className="text-blue-500">*</span> Requires ELCC calculation</span>
              <span><span className="text-purple-500">*</span> Requires pricing calculation</span>
            </div>

            {/* Categories */}
            {categories.map((category) => {
              const metrics = getMetricsByCategory(category);
              if (metrics.length === 0) return null;

              return (
                <div key={category} className="mb-6">
                  <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-2">
                    {METRIC_CATEGORIES[category]}
                  </h3>
                  <div className="space-y-2">
                    {metrics.map((metric) => (
                      <label
                        key={metric.id}
                        className="flex items-start gap-3 cursor-pointer group"
                      >
                        <input
                          type="checkbox"
                          checked={selectedMetrics.includes(metric.id)}
                          onChange={() => toggleMetric(metric.id)}
                          className="mt-1 h-4 w-4 text-blue-600 rounded border-gray-300 focus:ring-blue-500"
                        />
                        <div className="flex-1">
                          <div className="flex items-center gap-1">
                            <span className="text-sm font-medium text-gray-900 group-hover:text-blue-600">
                              {metric.label}
                            </span>
                            {metric.unit && (
                              <span className="text-xs text-gray-400">
                                ({metric.unit})
                              </span>
                            )}
                            {metric.requiresElcc && (
                              <span className="text-blue-500 text-xs">*</span>
                            )}
                            {metric.requiresPricing && (
                              <span className="text-purple-500 text-xs">*</span>
                            )}
                          </div>
                          <p className="text-xs text-gray-500">
                            {metric.description}
                          </p>
                        </div>
                      </label>
                    ))}
                  </div>
                </div>
              );
            })}

            {/* ELCC Method selector (shown when ELCC metrics selected) */}
            {hasElccMetrics && (
              <div className="mt-6 pt-4 border-t">
                <h3 className="text-sm font-semibold text-gray-700 uppercase tracking-wide mb-2">
                  ELCC Calculation Method
                </h3>
                <div className="grid grid-cols-2 gap-2">
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="elccMethod"
                      checked={elccMethod === ElccMethod.FirstIn}
                      onChange={() => setElccMethod(ElccMethod.FirstIn)}
                      className="h-4 w-4 text-blue-600 border-gray-300 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-700">First-In (Standalone)</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="elccMethod"
                      checked={elccMethod === ElccMethod.Marginal}
                      onChange={() => setElccMethod(ElccMethod.Marginal)}
                      className="h-4 w-4 text-blue-600 border-gray-300 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-700">Marginal (Last-In)</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="elccMethod"
                      checked={elccMethod === ElccMethod.Contribution}
                      onChange={() => setElccMethod(ElccMethod.Contribution)}
                      className="h-4 w-4 text-blue-600 border-gray-300 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-700">Contribution (Removal)</span>
                  </label>
                  <label className="flex items-center gap-2 cursor-pointer">
                    <input
                      type="radio"
                      name="elccMethod"
                      checked={elccMethod === ElccMethod.Delta}
                      onChange={() => setElccMethod(ElccMethod.Delta)}
                      className="h-4 w-4 text-blue-600 border-gray-300 focus:ring-blue-500"
                    />
                    <span className="text-sm text-gray-700">Delta (E3 Method)</span>
                  </label>
                </div>
                <p className="mt-2 text-xs text-gray-500">
                  First-In: standalone. Marginal: last-in increment. Contribution: removal impact. Delta: E3 proportional allocation.
                </p>
              </div>
            )}
          </div>

          {/* Footer */}
          <div className="flex justify-between items-center px-6 py-4 border-t bg-gray-50">
            <span className="text-sm text-gray-500">
              {selectedMetrics.length} metrics selected
            </span>
            <button
              onClick={onClose}
              className="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-blue-500"
            >
              Done
            </button>
          </div>
        </div>
      </div>
    </div>
  );
};
