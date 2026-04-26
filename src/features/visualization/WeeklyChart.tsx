import React, { useMemo } from 'react';
import Plot from 'react-plotly.js';
import { useSimulationStore } from '../../stores/simulationStore';
import { COLORS } from '../../types';

interface WeeklyChartProps {
  week: number;
}

export const WeeklyChart: React.FC<WeeklyChartProps> = ({ week }) => {
  const simulationResult = useSimulationStore((state) => state.simulationResult);
  const loadProfile = useSimulationStore((state) => state.loadProfile);
  const batteryEfficiency = useSimulationStore((state) => state.config.battery_efficiency);

  const chartData = useMemo(() => {
    if (!simulationResult) return null;

    // Calculate start and end hours for the week
    const hoursPerWeek = week === 52 ? 192 : 168; // Week 52 gets extra days
    const startHour = (week - 1) * 168;
    const endHour = Math.min(startHour + hoursPerWeek, 8760);

    // Extract week data
    const hours = Array.from({ length: endHour - startHour }, (_, i) => i);
    const solar = simulationResult.solar_out.slice(startHour, endHour);
    const wind = simulationResult.wind_out.slice(startHour, endHour);
    const cleanFirm = simulationResult.clean_firm_generation.slice(startHour, endHour);
    const batteryDischargeRaw = simulationResult.battery_discharge.slice(startHour, endHour)
      .map((d) => d * batteryEfficiency);
    const gasForCharging = simulationResult.gas_for_charging.slice(startHour, endHour);
    // Gas for load = total gas - gas for charging
    const gas = simulationResult.gas_generation.slice(startHour, endHour)
      .map((g, i) => Math.max(0, g - gasForCharging[i]));
    const load = loadProfile.slice(startHour, endHour);
    const batteryCharge = simulationResult.battery_charge.slice(startHour, endHour);
    const curtailed = simulationResult.curtailed.slice(startHour, endHour);

    // Process data for new visualization:
    // - Below load: solar, wind, CF, gas (capped proportionally), battery fills gap from top
    // - Above load: charging (purple) + curtailed (black) + gas for charging

    const solarCapped: number[] = [];
    const windCapped: number[] = [];
    const cfCapped: number[] = [];
    const gasCapped: number[] = [];
    const batteryFill: number[] = [];       // Battery discharge filling gap (hangs from load)
    const batteryFillBase: number[] = [];   // Base position for battery fill
    const chargingAbove: number[] = [];     // Charging shown above load
    const curtailedAbove: number[] = [];    // Curtailed shown above charging
    const gasChargingAbove: number[] = [];  // Gas for charging above load

    for (let h = 0; h < hours.length; h++) {
      const loadVal = load[h];
      const baseGen = solar[h] + wind[h] + cleanFirm[h] + gas[h];

      if (baseGen <= loadVal) {
        // All generation goes to load - no capping needed
        solarCapped.push(solar[h]);
        windCapped.push(wind[h]);
        cfCapped.push(cleanFirm[h]);
        gasCapped.push(gas[h]);

        // Battery fills the remaining gap (hangs from load line)
        const gap = loadVal - baseGen;
        const batteryToFill = Math.min(batteryDischargeRaw[h], gap);
        batteryFill.push(batteryToFill);
        batteryFillBase.push(baseGen); // Start from top of generation stack

        // Nothing above load (no excess)
        chargingAbove.push(0);
        curtailedAbove.push(0);
      } else {
        // Generation exceeds load - cap proportionally and show excess above
        const ratio = loadVal / baseGen;
        solarCapped.push(solar[h] * ratio);
        windCapped.push(wind[h] * ratio);
        cfCapped.push(cleanFirm[h] * ratio);
        gasCapped.push(gas[h] * ratio);

        // No battery discharge needed (no gap to fill)
        batteryFill.push(0);
        batteryFillBase.push(loadVal);

        // Show excess above load: charging (purple) on bottom, curtailed (black) on top
        // battery_charge already includes all charging; curtailed is what's wasted
        const renewableCharge = Math.max(0, batteryCharge[h] - gasForCharging[h]);
        chargingAbove.push(renewableCharge);
        curtailedAbove.push(curtailed[h]);
      }

      // Gas for charging always shown above load
      gasChargingAbove.push(gasForCharging[h]);
    }

    // Create x-axis labels (day names)
    const dayNames = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
    const tickvals: number[] = [];
    const ticktext: string[] = [];
    for (let d = 0; d < Math.ceil(hoursPerWeek / 24); d++) {
      tickvals.push(d * 24 + 12);
      ticktext.push(dayNames[d % 7]);
    }

    return {
      hours,
      solarCapped,
      windCapped,
      cfCapped,
      gasCapped,
      batteryFill,
      batteryFillBase,
      chargingAbove,
      curtailedAbove,
      gasChargingAbove,
      load,
      tickvals,
      ticktext,
    };
  }, [simulationResult, loadProfile, week, batteryEfficiency]);

  if (!chartData) {
    return <div className="h-96 flex items-center justify-center text-gray-500">No data</div>;
  }

  const traces = [
    // Stacked generation below load (from bottom)
    {
      x: chartData.hours,
      y: chartData.solarCapped,
      name: 'Solar',
      type: 'bar' as const,
      marker: { color: COLORS.solar },
    },
    {
      x: chartData.hours,
      y: chartData.windCapped,
      name: 'Wind',
      type: 'bar' as const,
      marker: { color: COLORS.wind },
    },
    {
      x: chartData.hours,
      y: chartData.cfCapped,
      name: 'Clean Firm',
      type: 'bar' as const,
      marker: { color: COLORS.cleanFirm },
    },
    {
      x: chartData.hours,
      y: chartData.gasCapped,
      name: 'Gas',
      type: 'bar' as const,
      marker: { color: COLORS.gas },
    },
    // Battery discharge fills gap (hangs from load line)
    {
      x: chartData.hours,
      y: chartData.batteryFill,
      base: chartData.batteryFillBase,
      name: 'Battery',
      type: 'bar' as const,
      marker: { color: COLORS.battery },
    },
    // Above load line: charging (purple) on bottom
    {
      x: chartData.hours,
      y: chartData.chargingAbove,
      base: chartData.load,
      name: 'Charging',
      type: 'bar' as const,
      marker: { color: COLORS.storage }, // Purple
    },
    // Above load line: curtailed (black) stacked on top of charging
    {
      x: chartData.hours,
      y: chartData.curtailedAbove,
      base: chartData.load.map((l, i) => l + chartData.chargingAbove[i]),
      name: 'Curtailed',
      type: 'bar' as const,
      marker: { color: '#333333' }, // Black
    },
    // Gas for charging (above load, stacked on top)
    {
      x: chartData.hours,
      y: chartData.gasChargingAbove,
      base: chartData.load.map((l, i) => l + chartData.chargingAbove[i] + chartData.curtailedAbove[i]),
      name: 'Gas for Charging',
      type: 'bar' as const,
      marker: { color: '#9c27b0' }, // Darker purple
    },
    // Load line
    {
      x: chartData.hours,
      y: chartData.load,
      name: 'Load',
      type: 'scatter' as const,
      mode: 'lines' as const,
      line: { color: '#333', width: 2 },
    },
  ];

  const layout = {
    barmode: 'stack' as const,
    height: 400,
    margin: { t: 20, r: 20, b: 50, l: 60 },
    xaxis: {
      title: { text: '' },
      tickmode: 'array' as const,
      tickvals: chartData.tickvals,
      ticktext: chartData.ticktext,
    },
    yaxis: {
      title: { text: 'Power (MW)' },
    },
    legend: {
      orientation: 'h' as const,
      y: -0.15,
      x: 0.5,
      xanchor: 'center' as const,
    },
    font: {
      family: 'Google Sans, Roboto, sans-serif',
    },
  };

  return (
    <Plot
      data={traces}
      layout={layout}
      config={{
        responsive: true,
        displayModeBar: false,
      }}
      style={{ width: '100%' }}
    />
  );
};
