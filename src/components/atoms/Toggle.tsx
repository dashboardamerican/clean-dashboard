import React from 'react';

interface ToggleProps {
  label: string;
  checked: boolean;
  onChange: (checked: boolean) => void;
  description?: string;
}

export const Toggle: React.FC<ToggleProps> = ({ label, checked, onChange, description }) => {
  return (
    <div className="flex flex-col">
      <div className="flex items-center justify-between">
        <span className="text-sm font-medium text-gray-700">{label}</span>
        <button
          type="button"
          role="switch"
          aria-checked={checked}
          onClick={() => onChange(!checked)}
          className={`
            relative inline-flex h-6 w-11 items-center rounded-full transition-colors
            ${checked ? 'bg-blue-600' : 'bg-gray-300'}
          `}
        >
          <span
            className={`
              inline-block h-4 w-4 transform rounded-full bg-white transition-transform
              ${checked ? 'translate-x-6' : 'translate-x-1'}
            `}
          />
        </button>
      </div>
      {description && (
        <p className="text-xs text-gray-500 mt-1">{description}</p>
      )}
    </div>
  );
};
