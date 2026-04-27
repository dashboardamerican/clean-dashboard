// Metric definitions for the configurable metrics system

export interface MetricDefinition {
  id: string;
  label: string;
  unit: string;
  category: MetricCategory;
  description: string;
  requiresElcc?: boolean;
  requiresPricing?: boolean;
  isToggle?: boolean; // For non-numeric metrics like battery mode selector
}

export type MetricCategory =
  | 'core'
  | 'system_performance'
  | 'economic_analysis'
  | 'reliability_analysis'
  | 'environmental';

export const METRIC_CATEGORIES: Record<MetricCategory, string> = {
  core: 'Core Metrics',
  system_performance: 'System Performance',
  economic_analysis: 'Economic Analysis',
  reliability_analysis: 'Reliability Analysis',
  environmental: 'Environmental & Infrastructure',
};

// All available metrics organized by category
export const METRIC_DEFINITIONS: MetricDefinition[] = [
  // Core (4) - default enabled
  {
    id: 'annual_match',
    label: 'Annual Match',
    unit: '%',
    category: 'core',
    description:
      'Clean energy generated (incl. curtailed) minus battery losses, divided by annual load. The generous "100% renewable" headline.',
  },
  {
    id: 'hourly_match',
    label: 'Hourly Matched',
    unit: '%',
    category: 'core',
    description:
      'Clean energy that actually served load each hour, summed and divided by load. The strict 24/7 carbon-free-energy metric.',
  },
  {
    id: 'ghg_intensity',
    label: 'GHG Intensity',
    unit: 'g/kWh',
    category: 'core',
    description: 'Lifecycle greenhouse gas emissions per unit energy',
  },
  {
    id: 'lcoe',
    label: 'System LCOE',
    unit: '$/MWh',
    category: 'core',
    description: 'Levelized cost of electricity for the entire system',
  },

  // System Performance (5)
  {
    id: 'curtailed',
    label: 'Curtailed',
    unit: '%',
    category: 'system_performance',
    description: 'Percentage of clean generation that is curtailed',
  },
  {
    id: 'zero_price_gen',
    label: 'Zero Price Gen',
    unit: '%',
    category: 'system_performance',
    description: 'Percentage of renewable generation during excess supply (curtailment) hours',
  },
  {
    id: 'load_utilization',
    label: 'Load Utilization',
    unit: '%',
    category: 'system_performance',
    description: 'Percentage of original load served (after demand response)',
  },
  {
    id: 'gas_capacity',
    label: 'Gas Capacity',
    unit: 'MW',
    category: 'system_performance',
    description:
      'Firm gas capacity built = operational peak × (1 + reserve margin). Matches the gas capex billed in LCOE.',
  },
  {
    id: 'peak_shave',
    label: 'Peak Shave',
    unit: 'MW',
    category: 'system_performance',
    description:
      'Operational peak shaving from battery dispatch (peak load − peak gas hour). Independent of reserve margin.',
  },

  // Economic Analysis (6)
  {
    id: 'operating_costs',
    label: 'Operating Costs',
    unit: '$/MWh',
    category: 'economic_analysis',
    description: 'Variable operating costs (excludes capital recovery)',
  },
  {
    id: 'customer_costs',
    label: 'Customer Costs',
    unit: '$/MWh',
    category: 'economic_analysis',
    description: 'Average electricity price paid by customers',
    requiresPricing: true,
  },
  {
    id: 'solar_market_value',
    label: 'Solar Market Value',
    unit: '$/MWh',
    category: 'economic_analysis',
    description: 'Energy-weighted average price for solar generation',
    requiresPricing: true,
  },
  {
    id: 'wind_market_value',
    label: 'Wind Market Value',
    unit: '$/MWh',
    category: 'economic_analysis',
    description: 'Energy-weighted average price for wind generation',
    requiresPricing: true,
  },
  {
    id: 'solar_system_value',
    label: 'Solar System Value',
    unit: '$/MWh',
    category: 'economic_analysis',
    description: 'Solar value including capacity and fuel displacement',
    requiresElcc: true,
    requiresPricing: true,
  },
  {
    id: 'wind_system_value',
    label: 'Wind System Value',
    unit: '$/MWh',
    category: 'economic_analysis',
    description: 'Wind value including capacity and fuel displacement',
    requiresElcc: true,
    requiresPricing: true,
  },

  // Reliability Analysis (2)
  {
    id: 'elcc_analysis',
    label: 'ELCC Analysis',
    unit: '',
    category: 'reliability_analysis',
    description: 'Effective Load Carrying Capability table by resource',
    requiresElcc: true,
  },
  {
    id: 'delta_elcc_method',
    label: 'ELCC Method',
    unit: '',
    category: 'reliability_analysis',
    description: 'Currently selected ELCC allocation method',
    isToggle: true,
    requiresElcc: true,
  },

  // Environmental & Infrastructure (2)
  {
    id: 'land_use',
    label: 'Land Use',
    unit: 'mi\u00B2',
    category: 'environmental',
    description: 'Direct footprint / Total (including wind spacing and exclusion zones)',
  },
  {
    id: 'battery_mode',
    label: 'Battery Mode',
    unit: '',
    category: 'environmental',
    description: 'Current battery dispatch strategy',
    isToggle: true,
  },
];

// Default metrics to show (core metrics)
export const DEFAULT_SELECTED_METRICS = [
  'annual_match',
  'hourly_match',
  'ghg_intensity',
  'lcoe',
];

// Get metrics by category
export function getMetricsByCategory(category: MetricCategory): MetricDefinition[] {
  return METRIC_DEFINITIONS.filter(m => m.category === category);
}

// Get metric definition by ID
export function getMetricById(id: string): MetricDefinition | undefined {
  return METRIC_DEFINITIONS.find(m => m.id === id);
}

// Check if any selected metrics require ELCC
export function requiresElccCalculation(selectedMetrics: string[]): boolean {
  return selectedMetrics.some(id => {
    const metric = getMetricById(id);
    return metric?.requiresElcc === true;
  });
}

// Check if any selected metrics require pricing
export function requiresPricingCalculation(selectedMetrics: string[]): boolean {
  return selectedMetrics.some(id => {
    const metric = getMetricById(id);
    return metric?.requiresPricing === true;
  });
}

// GHG intensity color scale (7 bands from green to brown)
export function getGhgColor(intensity: number): string {
  if (intensity <= 25) return '#6BAF68';  // deep green
  if (intensity <= 75) return '#81BA68';
  if (intensity <= 150) return '#BED853';
  if (intensity <= 300) return '#C3D768';
  if (intensity <= 400) return '#DFC85D';
  if (intensity <= 500) return '#90492F';
  return '#4E2912';  // dark brown
}
