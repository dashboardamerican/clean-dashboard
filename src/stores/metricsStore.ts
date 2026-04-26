import { create } from 'zustand';
import { persist } from 'zustand/middleware';
import { immer } from 'zustand/middleware/immer';
import { DEFAULT_SELECTED_METRICS } from '../types/metrics';
import { ElccMethod } from '../types';

interface MetricsState {
  // Selected metrics to display
  selectedMetrics: string[];

  // ELCC display method preference
  elccMethod: ElccMethod;

  // Actions
  setSelectedMetrics: (metrics: string[]) => void;
  toggleMetric: (id: string) => void;
  setElccMethod: (method: ElccMethod) => void;
  resetToDefaults: () => void;
}

export const useMetricsStore = create<MetricsState>()(
  persist(
    immer((set) => ({
      selectedMetrics: [...DEFAULT_SELECTED_METRICS],
      elccMethod: ElccMethod.Contribution,

      setSelectedMetrics: (metrics) => {
        set((state) => {
          state.selectedMetrics = metrics;
        });
      },

      toggleMetric: (id) => {
        set((state) => {
          const index = state.selectedMetrics.indexOf(id);
          if (index === -1) {
            state.selectedMetrics.push(id);
          } else {
            state.selectedMetrics.splice(index, 1);
          }
        });
      },

      setElccMethod: (method) => {
        set((state) => {
          state.elccMethod = method;
        });
      },

      resetToDefaults: () => {
        set((state) => {
          state.selectedMetrics = [...DEFAULT_SELECTED_METRICS];
          state.elccMethod = ElccMethod.Contribution;
        });
      },
    })),
    {
      name: 'metrics-storage',
      version: 3, // Bump for 4-method ELCC (FirstIn, Marginal, Contribution, Delta)
    }
  )
);
