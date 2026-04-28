import { TutorialActionName } from '../../stores/tutorialStore';

export type TutorialPlacement = 'top' | 'bottom' | 'left' | 'right' | 'center';
export type TutorialMode = 'spotlight' | 'centered' | 'modal';

export interface TutorialStep {
  targetId: string | null;
  title: string;
  body: string;
  placement?: TutorialPlacement;
  mode?: TutorialMode;
  onEnter?: TutorialActionName;
  onExit?: TutorialActionName;
}

const SHARED_FINISH: TutorialStep = {
  targetId: null,
  mode: 'centered',
  title: "You're all set",
  body:
    'Keyboard shortcuts: S = Settings · O = Optimizer · M = Metrics · R = Reset · Esc closes any modal. Click the "?" in the header to replay this tour. Happy modeling.',
};

export const SIMPLE_STEPS: TutorialStep[] = [
  {
    targetId: 'capacity-sliders',
    title: 'Build your generation mix',
    body:
      'Drag these sliders to size each clean resource. Solar and wind are intermittent; clean firm runs 24/7. Every change re-runs the full 8,760-hour simulation instantly.',
    placement: 'right',
  },
  {
    targetId: 'metrics-panel',
    title: 'How your system performs',
    body:
      'Annual match is the share of energy served clean over the year. Hourly match is the share served clean in each hour — the harder, more honest number. LCOE is what you’d need to charge per MWh to break even.',
    placement: 'bottom',
  },
  {
    targetId: 'visualization-picker',
    title: 'Explore the charts',
    body:
      'Switch between weekly dispatch, annual heatmaps, battery state-of-charge, LCOE breakdowns, and market prices. Each tells a different story about the same simulation.',
    placement: 'bottom',
  },
  {
    targetId: 'optimizer-button',
    title: 'Let the optimizer drive',
    body:
      "The optimizer finds the cheapest portfolio that hits a target clean match % exactly. Click it when you're ready to try 80% or 95%.",
    placement: 'bottom',
  },
  SHARED_FINISH,
];

export const FULL_STEPS: TutorialStep[] = [
  {
    targetId: 'region-selector',
    title: 'Pick a region',
    body:
      'Each of 13 US zones has its own solar, wind, and load profile. Texas is sunny and windy; the Pacific Northwest is hydro-leaning; the Southeast peaks late on summer afternoons.',
    placement: 'right',
  },
  {
    targetId: 'load-shape',
    title: 'Load shape',
    body:
      "Use the zone's actual hourly demand, or a flat 100 MW load if you want to isolate supply-side dynamics from real-world demand variability.",
    placement: 'right',
  },
  {
    targetId: 'capacity-sliders',
    title: 'Build your generation mix',
    body:
      'Solar and wind are intermittent — they only generate when the sun shines or the wind blows. Clean firm runs 24/7 (geothermal, advanced nuclear, long-duration storage). Every change re-runs the simulation instantly.',
    placement: 'right',
  },
  {
    targetId: 'storage-and-battery',
    title: 'Storage and dispatch logic',
    body:
      'Storage shifts excess renewables into high-demand hours. The battery mode controls dispatch: water-fill shaves the highest peaks first; peak shaver maintains a constant target line; hybrid does both for the most realistic operation.',
    placement: 'right',
  },
  {
    targetId: 'demand-response',
    title: 'Demand response',
    body:
      'Up to this many MW of load can be shed during tight supply hours, reducing how much firm generation you need to build. Useful for closing the last few percent of clean match.',
    placement: 'right',
  },
  {
    targetId: 'metrics-panel',
    title: 'How your system performs',
    body:
      "Annual match is the share of energy served clean over the year. Hourly match is the share served clean in each hour — the harder, more honest number. GHG intensity counts gas combustion plus methane leakage and embodied emissions. LCOE is what you'd need to charge per MWh to break even.",
    placement: 'bottom',
  },
  {
    targetId: null,
    mode: 'modal',
    onEnter: 'openMetrics',
    onExit: 'closeMetrics',
    title: 'Add deeper metrics',
    body:
      'This is the metrics chooser. Pick from curtailment, ELCC (effective load-carrying capacity), market value, customer cost, land use, peak shave, operating costs, and more — about twenty in total. Each one has a tooltip explaining what it captures.',
  },
  {
    targetId: 'visualization-picker',
    title: 'Ten chart types to explore',
    body:
      'Weekly Operation shows hour-by-hour dispatch. Annual Heatmap reveals seasonal patterns. LCOE Components breaks cost down by technology. The Sweep charts (Resource, Optimizer, Cost) show how the system responds across a parameter range — these are where the real insights live.',
    placement: 'bottom',
  },
  {
    targetId: null,
    mode: 'modal',
    onEnter: 'openSettings',
    onExit: 'closeSettings',
    title: 'Tune the cost model',
    body:
      'Settings has 68 cost parameters across ten tabs: CAPEX, fixed and variable O&M, fuel, financial, lifetimes, ITCs, emissions, land, and CCS. Use the preset selector at the top to jump between "low cost clean", "high cost clean", and high/low gas price scenarios.',
  },
  {
    targetId: null,
    mode: 'modal',
    onEnter: 'openOptimizer',
    onExit: 'closeOptimizer',
    title: 'Let the optimizer drive',
    body:
      'Set the target clean match %, choose which resources to allow (solar / wind / storage / clean firm), and click run. The optimizer hits the target exactly — not at-or-above — so an 80% target gives you a true 80% portfolio, not 88%. Try 80% then 95% to see how the mix shifts.',
  },
  SHARED_FINISH,
];
