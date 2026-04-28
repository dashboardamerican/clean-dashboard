import { create } from 'zustand'
import { VisualizationType } from '../types'

export type WeeklyScope = 'week' | 'year';

interface UiState {
  // Visualization
  currentViz: VisualizationType;
  selectedWeek: number;
  weeklyScope: WeeklyScope;

  // Modal states
  settingsOpen: boolean;
  optimizerOpen: boolean;
  guideOpen: boolean;

  // Actions
  setVisualization: (viz: VisualizationType) => void;
  cycleVisualization: () => void;
  setSelectedWeek: (week: number) => void;
  setWeeklyScope: (scope: WeeklyScope) => void;
  setSettingsOpen: (open: boolean) => void;
  setOptimizerOpen: (open: boolean) => void;
  setGuideOpen: (open: boolean) => void;
}

const VISUALIZATIONS: VisualizationType[] = [
  'weekly',
  'heatmap',
  'battery',
  'lcoe',
  'price',
  'gasBaseline',
  'resourceSweep',
  'optimizerSweep',
  'costSweep',
];

export const useUiStore = create<UiState>((set, get) => ({
  currentViz: 'weekly',
  selectedWeek: 1,
  weeklyScope: 'week',
  settingsOpen: false,
  optimizerOpen: false,
  guideOpen: false,

  setVisualization: (viz) => set({ currentViz: viz }),

  cycleVisualization: () => {
    const current = get().currentViz;
    const currentIndex = VISUALIZATIONS.indexOf(current);
    const nextIndex = (currentIndex + 1) % VISUALIZATIONS.length;
    set({ currentViz: VISUALIZATIONS[nextIndex] });
  },

  setSelectedWeek: (week) => set({ selectedWeek: Math.max(1, Math.min(52, week)) }),

  setWeeklyScope: (scope) => set({ weeklyScope: scope }),

  setSettingsOpen: (open) => set({ settingsOpen: open }),
  setOptimizerOpen: (open) => set({ optimizerOpen: open }),
  setGuideOpen: (open) => set({ guideOpen: open }),
}));
