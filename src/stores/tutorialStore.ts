import { create } from 'zustand';
import { persist } from 'zustand/middleware';

export type TutorialVariant = 'simple' | 'full';

export type TutorialActionName =
  | 'openSettings'
  | 'closeSettings'
  | 'openOptimizer'
  | 'closeOptimizer'
  | 'openMetrics'
  | 'closeMetrics';

export type TutorialActions = Partial<Record<TutorialActionName, () => void>>;

interface TutorialState {
  hasSeenTutorial: boolean;
  isOpen: boolean;
  currentStep: number;
  variant: TutorialVariant | null;
  actions: TutorialActions;

  openTutorial: () => void;
  closeTutorial: () => void;
  startVariant: (variant: TutorialVariant) => void;
  nextStep: () => void;
  prevStep: () => void;
  registerActions: (actions: TutorialActions) => void;
  runAction: (name: TutorialActionName | undefined) => void;
}

export const useTutorialStore = create<TutorialState>()(
  persist(
    (set, get) => ({
      hasSeenTutorial: false,
      isOpen: false,
      currentStep: 0,
      variant: null,
      actions: {},

      openTutorial: () =>
        set({ isOpen: true, currentStep: 0, variant: null }),

      closeTutorial: () =>
        set({
          isOpen: false,
          hasSeenTutorial: true,
          currentStep: 0,
          variant: null,
        }),

      startVariant: (variant) => set({ variant, currentStep: 0 }),

      nextStep: () => set((state) => ({ currentStep: state.currentStep + 1 })),

      prevStep: () =>
        set((state) => {
          if (state.currentStep === 0 && state.variant !== null) {
            return { variant: null, currentStep: 0 };
          }
          return { currentStep: Math.max(0, state.currentStep - 1) };
        }),

      registerActions: (actions) =>
        set((state) => ({ actions: { ...state.actions, ...actions } })),

      runAction: (name) => {
        if (!name) return;
        const fn = get().actions[name];
        if (fn) fn();
      },
    }),
    {
      name: 'energy-simulator-tutorial',
      version: 2,
      partialize: (state) => ({ hasSeenTutorial: state.hasSeenTutorial }),
    }
  )
);
