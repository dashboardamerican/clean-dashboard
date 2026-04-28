import React, { useMemo } from 'react';
import Plot from 'react-plotly.js';
import { useSimulationStore } from '../../stores/simulationStore';
import { COLORS } from '../../types';

type Scope = 'week' | 'year';

interface WeeklyChartProps {
  week: number;
  scope?: Scope;
}

const MONTH_NAMES = [
  'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
  'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
];

// Hour-of-year for the first day of each month (non-leap year)
const MONTH_START_HOURS = [
  0, 744, 1416, 2160, 2880, 3624,
  4344, 5088, 5832, 6552, 7296, 8016, 8760,
];

export const WeeklyChart: React.FC<WeeklyChartProps> = ({ week, scope = 'week' }) => {
  const simulationResult = useSimulationStore((state) => state.simulationResult);
  const loadProfile = useSimulationStore((state) => state.loadProfile);
  const batteryEfficiency = useSimulationStore((state) => state.config.battery_efficiency);

  const chartData = useMemo(() => {
    if (!simulationResult) return null;

    let startHour: number;
    let endHour: number;

    if (scope === 'year') {
      startHour = 0;
      endHour = 8760;
    } else {
      const hoursPerWeek = week === 52 ? 192 : 168;
      startHour = (week - 1) * 168;
      endHour = Math.min(startHour + hoursPerWeek, 8760);
    }

    const len = endHour - startHour;
    const hours = Array.from({ length: len }, (_, i) => i);
    const solar = simulationResult.solar_out.slice(startHour, endHour);
    const wind = simulationResult.wind_out.slice(startHour, endHour);
    const cleanFirm = simulationResult.clean_firm_generation.slice(startHour, endHour);
    const batteryDischargeRaw = simulationResult.battery_discharge.slice(startHour, endHour)
      .map((d) => d * batteryEfficiency);
    const gasForCharging = simulationResult.gas_for_charging.slice(startHour, endHour);
    const gas = simulationResult.gas_generation.slice(startHour, endHour)
      .map((g, i) => Math.max(0, g - gasForCharging[i]));
    const load = loadProfile.slice(startHour, endHour);
    const batteryCharge = simulationResult.battery_charge.slice(startHour, endHour);
    const curtailed = simulationResult.curtailed.slice(startHour, endHour);

    const solarCapped: number[] = [];
    const windCapped: number[] = [];
    const cfCapped: number[] = [];
    const gasCapped: number[] = [];
    const batteryFill: number[] = [];
    const batteryFillBase: number[] = [];
    const chargingAbove: number[] = [];
    const curtailedAbove: number[] = [];
    const gasChargingAbove: number[] = [];

    for (let h = 0; h < hours.length; h++) {
      const loadVal = load[h];
      const baseGen = solar[h] + wind[h] + cleanFirm[h] + gas[h];

      if (baseGen <= loadVal) {
        solarCapped.push(solar[h]);
        windCapped.push(wind[h]);
        cfCapped.push(cleanFirm[h]);
        gasCapped.push(gas[h]);

        const gap = loadVal - baseGen;
        const batteryToFill = Math.min(batteryDischargeRaw[h], gap);
        batteryFill.push(batteryToFill);
        batteryFillBase.push(baseGen);

        chargingAbove.push(0);
        curtailedAbove.push(0);
      } else {
        const ratio = loadVal / baseGen;
        solarCapped.push(solar[h] * ratio);
        windCapped.push(wind[h] * ratio);
        cfCapped.push(cleanFirm[h] * ratio);
        gasCapped.push(gas[h] * ratio);

        batteryFill.push(0);
        batteryFillBase.push(loadVal);

        const renewableCharge = Math.max(0, batteryCharge[h] - gasForCharging[h]);
        chargingAbove.push(renewableCharge);
        curtailedAbove.push(curtailed[h]);
      }

      gasChargingAbove.push(gasForCharging[h]);
    }

    // Tick labels
    let tickvals: number[] = [];
    let ticktext: string[] = [];
    if (scope === 'year') {
      // Place month label at the middle of each month
      for (let m = 0; m < 12; m++) {
        const mid = (MONTH_START_HOURS[m] + MONTH_START_HOURS[m + 1]) / 2;
        tickvals.push(mid);
        ticktext.push(MONTH_NAMES[m]);
      }
    } else {
      const dayNames = ['Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat', 'Sun'];
      const hoursPerWeek = week === 52 ? 192 : 168;
      for (let d = 0; d < Math.ceil(hoursPerWeek / 24); d++) {
        tickvals.push(d * 24 + 12);
        ticktext.push(dayNames[d % 7]);
      }
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
  }, [simulationResult, loadProfile, week, scope, batteryEfficiency]);

  if (!chartData) {
    return <div className="h-96 flex items-center justify-center text-gray-500">No data</div>;
  }

  // For year view, the battery fill amount is what's actually delivered to the load
  // from storage. Stack it on top of the other (capped) generation traces.
  // Skip the "above load" traces (charging/curtailed/gas-for-charging) in year view —
  // 8760 hourly points with all stacks would be visually overwhelming. Those details
  // are accessible via the Weekly view or the Annual Heatmap.
  const isYear = scope === 'year';

  const traces: any[] = isYear
    ? [
        {
          x: chartData.hours,
          y: chartData.solarCapped,
          name: 'Solar',
          type: 'scatter' as const,
          mode: 'lines' as const,
          stackgroup: 'supply',
          line: { width: 0 },
          fillcolor: COLORS.solar,
          hoverinfo: 'x+y+name' as const,
        },
        {
          x: chartData.hours,
          y: chartData.windCapped,
          name: 'Wind',
          type: 'scatter' as const,
          mode: 'lines' as const,
          stackgroup: 'supply',
          line: { width: 0 },
          fillcolor: COLORS.wind,
          hoverinfo: 'x+y+name' as const,
        },
        {
          x: chartData.hours,
          y: chartData.cfCapped,
          name: 'Clean Firm',
          type: 'scatter' as const,
          mode: 'lines' as const,
          stackgroup: 'supply',
          line: { width: 0 },
          fillcolor: COLORS.cleanFirm,
          hoverinfo: 'x+y+name' as const,
        },
        {
          x: chartData.hours,
          y: chartData.batteryFill,
          name: 'Battery',
          type: 'scatter' as const,
          mode: 'lines' as const,
          stackgroup: 'supply',
          line: { width: 0 },
          fillcolor: COLORS.battery,
          hoverinfo: 'x+y+name' as const,
        },
        {
          x: chartData.hours,
          y: chartData.gasCapped,
          name: 'Gas',
          type: 'scatter' as const,
          mode: 'lines' as const,
          stackgroup: 'supply',
          line: { width: 0 },
          fillcolor: COLORS.gas,
          hoverinfo: 'x+y+name' as const,
        },
        {
          x: chartData.hours,
          y: chartData.load,
          name: 'Load',
          type: 'scatter' as const,
          mode: 'lines' as const,
          line: { color: '#222', width: 1.5 },
          hoverinfo: 'x+y+name' as const,
        },
      ]
    : [
        // Existing weekly bar traces
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
        {
          x: chartData.hours,
          y: chartData.batteryFill,
          base: chartData.batteryFillBase,
          name: 'Battery',
          type: 'bar' as const,
          marker: { color: COLORS.battery },
        },
        {
          x: chartData.hours,
          y: chartData.chargingAbove,
          base: chartData.load,
          name: 'Charging',
          type: 'bar' as const,
          marker: { color: COLORS.storage },
        },
        {
          x: chartData.hours,
          y: chartData.curtailedAbove,
          base: chartData.load.map((l, i) => l + chartData.chargingAbove[i]),
          name: 'Curtailed',
          type: 'bar' as const,
          marker: { color: '#333333' },
        },
        {
          x: chartData.hours,
          y: chartData.gasChargingAbove,
          base: chartData.load.map((l, i) => l + chartData.chargingAbove[i] + chartData.curtailedAbove[i]),
          name: 'Gas for Charging',
          type: 'bar' as const,
          marker: { color: '#9c27b0' },
        },
        {
          x: chartData.hours,
          y: chartData.load,
          name: 'Load',
          type: 'scatter' as const,
          mode: 'lines' as const,
          line: { color: '#333', width: 2 },
        },
      ];

  const layout: any = {
    barmode: 'stack' as const,
    height: 400,
    margin: { t: 20, r: 20, b: 50, l: 60 },
    xaxis: {
      title: { text: '' },
      tickmode: 'array' as const,
      tickvals: chartData.tickvals,
      ticktext: chartData.ticktext,
      ...(isYear ? { range: [0, 8760] } : {}),
    },
    yaxis: {
      title: { text: 'Power (MW)' },
      rangemode: 'tozero' as const,
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
    hovermode: isYear ? ('x unified' as const) : ('closest' as const),
  };

  return (
    <Plot
      data={traces}
      layout={layout}
      config={{
        responsive: true,
        displayModeBar: isYear, // Enable zoom/pan on year view
        displaylogo: false,
        modeBarButtonsToRemove: ['lasso2d', 'select2d', 'autoScale2d', 'toggleSpikelines'],
      }}
      style={{ width: '100%' }}
    />
  );
};
