import React from 'react';

interface SubMetric {
  label: string;
  value: string;
}

interface ActionButton {
  label: string;
  onClick: () => void;
}

interface MetricProps {
  label: string;
  value: string | number;
  unit?: string;
  color?: string;
  subtext?: string;
  size?: 'sm' | 'md' | 'lg';
  colorIndicator?: string; // Color box indicator (e.g., for GHG severity)
  subMetrics?: SubMetric[]; // Additional sub-metrics (e.g., LCOE premium/abatement)
  actionButton?: ActionButton; // Action button (e.g., "Go" to peak week)
}

const sizeClasses = {
  sm: {
    value: 'text-lg font-semibold',
    label: 'text-xs',
    unit: 'text-xs',
  },
  md: {
    value: 'text-2xl font-bold',
    label: 'text-sm',
    unit: 'text-sm',
  },
  lg: {
    value: 'text-4xl font-bold',
    label: 'text-base',
    unit: 'text-base',
  },
};

export const Metric: React.FC<MetricProps> = ({
  label,
  value,
  unit,
  color,
  subtext,
  size = 'md',
  colorIndicator,
  subMetrics,
  actionButton,
}) => {
  const classes = sizeClasses[size];

  return (
    <div className="text-center">
      <div className={`${classes.label} text-gray-500 uppercase tracking-wide flex items-center justify-center gap-1`}>
        {colorIndicator && (
          <span
            className="inline-block w-3 h-3 rounded-sm"
            style={{ backgroundColor: colorIndicator }}
          />
        )}
        {label}
      </div>
      <div className="flex items-baseline justify-center gap-1">
        <span
          className={classes.value}
          style={{ color: color || '#1f2937' }}
        >
          {typeof value === 'number' ? value.toLocaleString(undefined, { maximumFractionDigits: 1 }) : value}
        </span>
        {unit && (
          <span className={`${classes.unit} text-gray-500`}>{unit}</span>
        )}
      </div>
      {subtext && (
        <div className="text-xs text-gray-400 mt-1">{subtext}</div>
      )}
      {subMetrics && subMetrics.length > 0 && (
        <div className="mt-1 space-y-0.5">
          {subMetrics.map((sub, idx) => (
            <div key={idx} className="text-xs text-gray-500">
              <span className="text-gray-400">{sub.label}:</span> {sub.value}
            </div>
          ))}
        </div>
      )}
      {actionButton && (
        <button
          onClick={actionButton.onClick}
          className="mt-1 px-2 py-0.5 text-xs bg-blue-100 text-blue-700 rounded hover:bg-blue-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
        >
          {actionButton.label}
        </button>
      )}
    </div>
  );
};
