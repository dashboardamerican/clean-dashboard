import React, { useState } from 'react';
import { Modal, Button, Slider, Select, Toggle } from '../../components/atoms';
import { useSettingsStore, PresetName } from '../../stores/settingsStore';
import { DepreciationMethod } from '../../types';

interface SettingsModalProps {
  isOpen: boolean;
  onClose: () => void;
}

type TabName = 'capex' | 'fixedOm' | 'variableOm' | 'fuel' | 'financial' | 'lifetimes' | 'itc' | 'emissions' | 'land' | 'ccs';

const TABS: { id: TabName; label: string }[] = [
  { id: 'capex', label: 'CAPEX' },
  { id: 'fixedOm', label: 'Fixed O&M' },
  { id: 'variableOm', label: 'Var O&M' },
  { id: 'fuel', label: 'Fuel' },
  { id: 'financial', label: 'Financial' },
  { id: 'lifetimes', label: 'Lifetimes' },
  { id: 'itc', label: 'ITCs' },
  { id: 'emissions', label: 'Emissions' },
  { id: 'land', label: 'Land' },
  { id: 'ccs', label: 'CCS' },
];

const PRESET_OPTIONS = [
  { value: 'default', label: 'Default Costs' },
  { value: 'lowCostClean', label: 'Low Cost Clean' },
  { value: 'highCostClean', label: 'High Cost Clean' },
  { value: 'highGasPrices', label: 'High Gas Prices' },
  { value: 'lowGasPrices', label: 'Low Gas Prices' },
  { value: 'custom', label: 'Custom' },
];

export const SettingsModal: React.FC<SettingsModalProps> = ({ isOpen, onClose }) => {
  const [activeTab, setActiveTab] = useState<TabName>('capex');
  const costs = useSettingsStore((state) => state.costs);
  const currentPreset = useSettingsStore((state) => state.currentPreset);
  const setCost = useSettingsStore((state) => state.setCost);
  const applyPreset = useSettingsStore((state) => state.applyPreset);
  const resetToDefaults = useSettingsStore((state) => state.resetToDefaults);

  const renderTabContent = () => {
    switch (activeTab) {
      case 'capex':
        return (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
              <Slider label="Solar ($/kW)" value={costs.solar_capex} min={500} max={2000} onChange={(v) => setCost('solar_capex', v)} />
              <Slider label="Wind ($/kW)" value={costs.wind_capex} min={800} max={2500} onChange={(v) => setCost('wind_capex', v)} />
              <Slider label="Storage ($/kWh)" value={costs.storage_capex} min={100} max={600} onChange={(v) => setCost('storage_capex', v)} />
              <Slider label="Clean Firm ($/kW)" value={costs.clean_firm_capex} min={2000} max={10000} step={100} onChange={(v) => setCost('clean_firm_capex', v)} />
              <Slider label="Gas ($/kW)" value={costs.gas_capex} min={500} max={4500} onChange={(v) => setCost('gas_capex', v)} />
            </div>

            <div className="border-t pt-4">
              <h4 className="text-sm font-semibold text-gray-700 mb-1">Planning Reserve</h4>
              <p className="text-xs text-gray-500 mb-3">
                Over-build factor on firm-thermal capacity (gas <em>and</em> clean firm
                — nuclear, geothermal). Covers forced outages, load forecast
                error, and weather extremes. NERC reference is ~15%; ERCOT
                13.75%, California ~18%. Affects capex / fixed O&M / depreciation /
                land for gas + CF only — dispatch and energy costs are unchanged.
                Renewables and storage are unaffected (their reliability is
                already captured in capacity factors and ELCC).
              </p>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
                <Slider
                  label="Reserve Margin (%)"
                  value={costs.reserve_margin}
                  min={0}
                  max={30}
                  step={1}
                  onChange={(v) => setCost('reserve_margin', v)}
                />
              </div>
            </div>
          </div>
        );

      case 'fixedOm':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
            <Slider label="Solar ($/kW-yr)" value={costs.solar_fixed_om} min={5} max={30} onChange={(v) => setCost('solar_fixed_om', v)} />
            <Slider label="Wind ($/kW-yr)" value={costs.wind_fixed_om} min={20} max={80} onChange={(v) => setCost('wind_fixed_om', v)} />
            <Slider label="Storage ($/kWh-yr)" value={costs.storage_fixed_om} min={5} max={25} onChange={(v) => setCost('storage_fixed_om', v)} />
            <Slider label="Clean Firm ($/kW-yr)" value={costs.clean_firm_fixed_om} min={30} max={120} onChange={(v) => setCost('clean_firm_fixed_om', v)} />
            <Slider label="Gas ($/kW-yr)" value={costs.gas_fixed_om} min={10} max={50} onChange={(v) => setCost('gas_fixed_om', v)} />
          </div>
        );

      case 'variableOm':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
            <Slider label="Solar Var O&M ($/MWh)" value={costs.solar_var_om} min={0} max={5} onChange={(v) => setCost('solar_var_om', v)} />
            <Slider label="Wind Var O&M ($/MWh)" value={costs.wind_var_om} min={0} max={5} onChange={(v) => setCost('wind_var_om', v)} />
            <Slider label="Storage Var O&M ($/MWh)" value={costs.storage_var_om} min={0} max={15} onChange={(v) => setCost('storage_var_om', v)} />
            <Slider label="Clean Firm Var O&M ($/MWh)" value={costs.clean_firm_var_om} min={0} max={30} onChange={(v) => setCost('clean_firm_var_om', v)} />
            <Slider label="Gas Var O&M ($/MWh)" value={costs.gas_var_om} min={0} max={10} onChange={(v) => setCost('gas_var_om', v)} />
          </div>
        );

      case 'fuel':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
            <Slider label="Gas Price ($/MMBtu)" value={costs.gas_price} min={0.5} max={14} step={0.5} onChange={(v) => setCost('gas_price', v)} />
            <Slider label="Clean Firm Fuel ($/MWh)" value={costs.clean_firm_fuel} min={0} max={50} onChange={(v) => setCost('clean_firm_fuel', v)} />
            <Slider label="Gas Heat Rate (MMBtu/MWh)" value={costs.gas_heat_rate} min={6} max={12} step={0.1} onChange={(v) => setCost('gas_heat_rate', v)} />
          </div>
        );

      case 'financial':
        return (
          <div className="space-y-4">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
              <Slider label="Discount Rate (%)" value={costs.discount_rate} min={3} max={12} step={0.5} onChange={(v) => setCost('discount_rate', v)} />
              <Slider label="Inflation Rate (%)" value={costs.inflation_rate} min={0} max={5} step={0.5} onChange={(v) => setCost('inflation_rate', v)} />
              <Slider label="Tax Rate (%)" value={costs.tax_rate} min={0} max={35} onChange={(v) => setCost('tax_rate', v)} />
              <Select
                label="Depreciation Method"
                value={costs.depreciation_method}
                options={[
                  { value: DepreciationMethod.Macrs5, label: 'MACRS 5-Year' },
                  { value: DepreciationMethod.Macrs15, label: 'MACRS 15-Year' },
                  { value: DepreciationMethod.StraightLine, label: 'Straight Line' },
                ]}
                onChange={(v) => setCost('depreciation_method', Number(v) as DepreciationMethod)}
              />
            </div>

            <div className="border-t pt-4 mt-4">
              <h4 className="text-sm font-medium text-gray-700 mb-3">Revenue & Tax Calculation</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
                <Slider label="Electricity Price ($/MWh)" value={costs.electricity_price} min={20} max={150} step={5} onChange={(v) => setCost('electricity_price', v)} />
                <Slider label="Excess Power Price ($/MWh)" value={costs.excess_power_price} min={0} max={50} step={5} onChange={(v) => setCost('excess_power_price', v)} />
              </div>
            </div>

            <div className="border-t pt-4 mt-4">
              <h4 className="text-sm font-medium text-gray-700 mb-3">Depreciation Monetization</h4>
              <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
                <Toggle
                  label="Monetize Excess Depreciation"
                  checked={costs.monetize_excess_depreciation}
                  onChange={(v) => setCost('monetize_excess_depreciation', v)}
                  description="Sell excess depreciation via tax equity"
                />
                {costs.monetize_excess_depreciation && (
                  <Slider
                    label="Monetization Rate (%)"
                    value={costs.monetization_rate}
                    min={30}
                    max={90}
                    step={5}
                    onChange={(v) => setCost('monetization_rate', v)}
                  />
                )}
              </div>
            </div>
          </div>
        );

      case 'lifetimes':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
            <Slider label="Project Lifetime (years)" value={costs.project_lifetime} min={10} max={60} step={5} onChange={(v) => setCost('project_lifetime', v)} />
            <Slider label="Solar Lifetime (years)" value={costs.solar_lifetime} min={20} max={40} step={5} onChange={(v) => setCost('solar_lifetime', v)} />
            <Slider label="Wind Lifetime (years)" value={costs.wind_lifetime} min={20} max={40} step={5} onChange={(v) => setCost('wind_lifetime', v)} />
            <Slider label="Storage Lifetime (years)" value={costs.storage_lifetime} min={10} max={25} step={5} onChange={(v) => setCost('storage_lifetime', v)} />
            <Slider label="Clean Firm Lifetime (years)" value={costs.clean_firm_lifetime} min={40} max={80} step={10} onChange={(v) => setCost('clean_firm_lifetime', v)} />
            <Slider label="Gas Lifetime (years)" value={costs.gas_lifetime} min={20} max={50} step={5} onChange={(v) => setCost('gas_lifetime', v)} />
          </div>
        );

      case 'itc':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
            <Slider label="Solar ITC (%)" value={costs.solar_itc * 100} min={0} max={50} onChange={(v) => setCost('solar_itc', v / 100)} />
            <Slider label="Wind ITC (%)" value={costs.wind_itc * 100} min={0} max={50} onChange={(v) => setCost('wind_itc', v / 100)} />
            <Slider label="Storage ITC (%)" value={costs.storage_itc * 100} min={0} max={50} onChange={(v) => setCost('storage_itc', v / 100)} />
            <Slider label="Clean Firm ITC (%)" value={costs.clean_firm_itc * 100} min={0} max={50} onChange={(v) => setCost('clean_firm_itc', v / 100)} />
          </div>
        );

      case 'emissions':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
            <Slider label="Gas Emissions (kg CO2/MMBtu)" value={costs.gas_emissions_factor} min={40} max={60} step={0.1} onChange={(v) => setCost('gas_emissions_factor', v)} />
            <Slider label="Gas Leakage Rate (%)" value={costs.gas_leakage_rate} min={0} max={5} step={0.1} onChange={(v) => setCost('gas_leakage_rate', v)} />
            <Slider label="Methane GWP" value={costs.methane_gwp} min={20} max={100} onChange={(v) => setCost('methane_gwp', v)} />
            <Slider label="Solar Embodied (g CO2/kWh)" value={costs.solar_embodied_emissions} min={0} max={50} onChange={(v) => setCost('solar_embodied_emissions', v)} />
            <Slider label="Wind Embodied (g CO2/kWh)" value={costs.wind_embodied_emissions} min={0} max={50} onChange={(v) => setCost('wind_embodied_emissions', v)} />
            <Slider label="Clean Firm Embodied (g CO2/kWh)" value={costs.clean_firm_embodied_emissions} min={0} max={50} onChange={(v) => setCost('clean_firm_embodied_emissions', v)} />
            <Slider label="Battery Embodied (kg CO2/kWh)" value={costs.battery_embodied_emissions} min={50} max={200} onChange={(v) => setCost('battery_embodied_emissions', v)} />
          </div>
        );

      case 'land':
        return (
          <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
            <Slider label="Solar (acres/MW)" value={costs.solar_land_direct} min={3} max={10} step={0.5} onChange={(v) => setCost('solar_land_direct', v)} />
            <Slider label="Wind Direct (acres/MW)" value={costs.wind_land_direct} min={0.5} max={3} step={0.25} onChange={(v) => setCost('wind_land_direct', v)} />
            <Slider label="Wind Total (acres/MW)" value={costs.wind_land_total} min={20} max={150} onChange={(v) => setCost('wind_land_total', v)} />
            <Slider label="Clean Firm Direct (acres/MW)" value={costs.clean_firm_land_direct} min={0.5} max={2} step={0.1} onChange={(v) => setCost('clean_firm_land_direct', v)} />
            <Slider label="Clean Firm Total (acres/MW)" value={costs.clean_firm_land_total} min={1} max={8} step={0.5} onChange={(v) => setCost('clean_firm_land_total', v)} />
            <Slider label="Gas (acres/MW)" value={costs.gas_land_direct} min={0.1} max={1} step={0.05} onChange={(v) => setCost('gas_land_direct', v)} />
          </div>
        );

      case 'ccs':
        return (
          <div className="space-y-4">
            <p className="text-sm text-gray-600">
              CCS applies to the selected share of gas backup generation. It affects LCOE, operating costs, emissions, and the gas baseline chart, but does not change dispatch.
            </p>
            <div className="grid grid-cols-1 md:grid-cols-2 gap-x-6 gap-y-3">
              <Slider label="CCS Coverage (% of Gas)" value={costs.ccs_percentage} min={0} max={100} step={5} onChange={(v) => setCost('ccs_percentage', v)} />
              <Slider label="CCS CAPEX ($/kW of CCS gas)" value={costs.ccs_capex} min={1500} max={4000} step={100} onChange={(v) => setCost('ccs_capex', v)} />
              <Slider label="CCS Fixed O&M ($/kW-yr)" value={costs.ccs_fixed_om} min={20} max={100} onChange={(v) => setCost('ccs_fixed_om', v)} />
              <Slider label="CCS Variable O&M ($/MWh)" value={costs.ccs_var_om} min={5} max={30} onChange={(v) => setCost('ccs_var_om', v)} />
              <Slider label="Energy Penalty (%)" value={costs.ccs_energy_penalty} min={10} max={35} onChange={(v) => setCost('ccs_energy_penalty', v)} />
              <Slider label="Capture Rate (%)" value={costs.ccs_capture_rate} min={70} max={100} onChange={(v) => setCost('ccs_capture_rate', v)} />
            </div>
          </div>
        );

      default:
        return null;
    }
  };

  return (
    <Modal isOpen={isOpen} onClose={onClose} title="Cost Settings" size="xl">
      {/* Presets */}
      <div className="flex items-center gap-2 mb-4 pb-4 border-b">
        <span className="text-sm font-medium text-gray-700">Preset:</span>
        <Select
          value={currentPreset}
          options={PRESET_OPTIONS}
          onChange={(v) => {
            if (v !== 'custom') {
              applyPreset(v as PresetName);
            }
          }}
          className="flex-1 max-w-xs"
        />
        <Button variant="secondary" size="sm" onClick={resetToDefaults}>
          Reset All
        </Button>
      </div>

      {/* Tabs */}
      <div className="flex border-b mb-4">
        {TABS.map((tab) => (
          <button
            key={tab.id}
            onClick={() => setActiveTab(tab.id)}
            className={`
              px-3 py-2 text-sm font-medium whitespace-nowrap
              ${activeTab === tab.id
                ? 'text-blue-600 border-b-2 border-blue-600'
                : 'text-gray-500 hover:text-gray-700'
              }
            `}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {/* Tab content - no max height to avoid scrolling */}
      <div className="min-h-[200px]">{renderTabContent()}</div>

      {/* Footer */}
      <div className="flex justify-end gap-2 mt-4 pt-4 border-t">
        <Button variant="secondary" onClick={onClose}>
          Close
        </Button>
      </div>
    </Modal>
  );
};
