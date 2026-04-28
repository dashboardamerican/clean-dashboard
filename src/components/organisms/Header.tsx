import React from 'react';
import { COLORS } from '../../types';

interface HeaderProps {
  onSettingsClick: () => void;
  onOptimizerClick: () => void;
  onResetClick: () => void;
  onTutorialClick: () => void;
}

export const Header: React.FC<HeaderProps> = ({
  onSettingsClick,
  onOptimizerClick,
  onResetClick,
  onTutorialClick,
}) => {
  return (
    <header className="bg-white shadow-sm">
      {/* Color strip */}
      <div className="color-strip">
        <div style={{ backgroundColor: COLORS.wind }} />
        <div style={{ backgroundColor: COLORS.gas }} />
        <div style={{ backgroundColor: COLORS.battery }} />
        <div style={{ backgroundColor: COLORS.solar }} />
        <div style={{ backgroundColor: COLORS.storage }} />
      </div>

      <div className="container mx-auto px-4">
        <div className="flex items-center justify-between h-14">
          {/* Logo / Title */}
          <div className="flex items-center gap-2">
            <svg
              className="w-8 h-8 text-blue-600"
              fill="none"
              viewBox="0 0 24 24"
              stroke="currentColor"
            >
              <path
                strokeLinecap="round"
                strokeLinejoin="round"
                strokeWidth={2}
                d="M13 10V3L4 14h7v7l9-11h-7z"
              />
            </svg>
            <h1 className="text-xl font-semibold text-gray-900">
              Energy System Simulator
            </h1>
          </div>

          {/* Navigation buttons */}
          <nav className="flex items-center gap-2">
            <button
              onClick={onTutorialClick}
              className="w-8 h-8 flex items-center justify-center text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded-full transition-colors"
              title="Show tutorial"
              aria-label="Show tutorial"
            >
              <svg
                className="w-5 h-5"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  strokeLinecap="round"
                  strokeLinejoin="round"
                  strokeWidth={2}
                  d="M8.228 9c.549-1.165 2.03-2 3.772-2 2.21 0 4 1.343 4 3 0 1.4-1.278 2.575-3.006 2.907-.542.104-.994.54-.994 1.093M12 17h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
                />
              </svg>
            </button>

            <button
              onClick={onResetClick}
              className="px-3 py-1.5 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded transition-colors"
              title="Reset to defaults (R)"
            >
              Reset
            </button>

            <button
              onClick={onSettingsClick}
              data-tutorial-id="settings-button"
              className="px-3 py-1.5 text-sm text-gray-600 hover:text-gray-900 hover:bg-gray-100 rounded transition-colors"
              title="Cost settings (S)"
            >
              <span className="flex items-center gap-1">
                <svg
                  className="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z"
                  />
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                  />
                </svg>
                Settings
              </span>
            </button>

            <button
              onClick={onOptimizerClick}
              data-tutorial-id="optimizer-button"
              className="px-3 py-1.5 text-sm bg-blue-600 text-white hover:bg-blue-700 rounded transition-colors"
              title="Run optimizer (O)"
            >
              <span className="flex items-center gap-1">
                <svg
                  className="w-4 h-4"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke="currentColor"
                >
                  <path
                    strokeLinecap="round"
                    strokeLinejoin="round"
                    strokeWidth={2}
                    d="M9 19v-6a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2a2 2 0 002-2zm0 0V9a2 2 0 012-2h2a2 2 0 012 2v10m-6 0a2 2 0 002 2h2a2 2 0 002-2m0 0V5a2 2 0 012-2h2a2 2 0 012 2v14a2 2 0 01-2 2h-2a2 2 0 01-2-2z"
                  />
                </svg>
                Optimizer
              </span>
            </button>
          </nav>
        </div>
      </div>
    </header>
  );
};
