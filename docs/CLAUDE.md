# CLAUDE.md - Rust/WASM Energy System Simulator

This folder contains a high-performance Rust/WASM rewrite of the Python energy system simulator, paired with a React/TypeScript frontend.

## Quick Start

```bash
# Build WASM module
cd rust && wasm-pack build --target web --out-dir ../web/src/lib/wasm/pkg

# Start web frontend
cd web && npm install && npm run dev
```

App runs at `http://localhost:3000`

## Architecture Overview

```
rust_refactor/
├── rust/               # Rust/WASM core (energy simulation, LCOE, optimizer)
│   ├── src/
│   │   ├── lib.rs              # WASM entry points (FFI exports)
│   │   ├── types.rs            # Core data structures
│   │   ├── simulation/         # 8760-hour chronological simulation
│   │   │   ├── core.rs         # simulate_system() entry point
│   │   │   └── battery/        # Battery dispatch algorithms
│   │   │       ├── default.rs  # Water-fill algorithm
│   │   │       ├── peak_shaver.rs
│   │   │       └── hybrid.rs   # Two-pass hybrid mode
│   │   ├── economics/          # Financial calculations
│   │   │   ├── lcoe.rs         # LCOE with ITCs, depreciation, tax
│   │   │   ├── depreciation.rs # MACRS schedules
│   │   │   ├── elcc.rs         # ELCC metrics
│   │   │   └── pricing.rs      # Hourly electricity pricing
│   │   └── optimizer/          # Portfolio optimization
│   │       ├── v2_hierarchical.rs  # Core optimizer (production)
│   │       ├── v1_adaptive.rs      # Deprecated (kept for reference)
│   │       ├── greedy.rs           # Greedy resource expansion
│   │       └── cache.rs            # Evaluation caching
│   └── Cargo.toml
│
├── web/                # React/TypeScript frontend (interactive dashboard)
│   ├── src/
│   │   ├── App.tsx             # Main application
│   │   ├── types/index.ts      # TypeScript types (must match Rust!)
│   │   ├── stores/             # Zustand state management
│   │   ├── hooks/useWasm.ts    # WASM loader
│   │   ├── components/         # UI components (atoms/molecules/organisms)
│   │   └── features/           # Feature modules (simulation, optimizer, viz)
│   └── package.json
│
├── microsite/          # Educational scrollytelling explainer
│   ├── generate_*.py           # Data generators (Python → JSON/TS)
│   ├── OVERVIEW.md             # Full narrative framework
│   ├── CLAUDE.md               # Microsite-specific guidance
│   └── site/                   # React app (D3, Framer Motion, Tailwind)
│       ├── src/
│       │   ├── components/
│       │   │   ├── chapters/       # Scroll-triggered narrative sections
│       │   │   └── visualizations/ # D3/Framer charts
│       │   └── data/               # Pre-computed simulation data
│       └── dist/                   # Built output
│
├── fixtures/           # Test fixtures & cross-validation
└── data/               # Zone profile data (zones.json)
```

## Key Modules

### Rust Core (`rust/src/`)

**`types.rs`** - Core data structures
- `SimulationConfig`: Capacity parameters, battery mode, efficiency
- `CostParams`: 68 economic parameters (CAPEX, O&M, ITCs, depreciation, emissions)
- `SimulationResult`: 8760 hourly arrays + aggregated metrics
- `BatteryMode`: Enum (Default=0, PeakShaver=1, Hybrid=2)
- `LcoeResult`: Technology-specific LCOE breakdown

**`simulation/core.rs`** - Main simulation entry point
- `simulate_system()`: Validates inputs, dispatches to battery mode handler
- Returns 8760-hour arrays: solar_out, wind_out, battery_charge/discharge, gas_generation, curtailed, etc.

**`simulation/battery/`** - Battery dispatch strategies
1. **default.rs**: Water-fill algorithm - shaves highest peaks first
2. **peak_shaver.rs**: Binary search for optimal constant peak line
3. **hybrid.rs**: Two-pass (peak shave + opportunistic dispatch at transitions)

**`economics/lcoe.rs`** - Investment-grade LCOE calculation
- Revenue-based taxable income
- ITC applied upfront, MACRS depreciation schedules
- Corporate tax shield effects
- Present value discounting
- Formula: `LCOE = PV(Total Costs) / PV(Total Energy)`

**`optimizer/v2_hierarchical.rs`** - Core Optimizer (PRODUCTION)
- This is THE optimizer used by all WASM functions (`optimize()`, `run_optimizer_sweep()`, `run_cost_sweep()`)
- Algorithm: Greedy expansion → Local refinement → Binary search CF sizing
- Performance: ~20ms per target, ~200 evaluations
- Precision: ±0.2% target compliance (exact, not minimum)
- Supports all battery modes, resource combinations, and cost scenarios

**`optimizer/v1_adaptive.rs`** - DEPRECATED
- Kept for reference only, not used in production
- Was: CF Grid Search + Gap-Aware Greedy algorithm

**`lib.rs`** - WASM exports
Key functions: `simulate()`, `compute_lcoe()`, `optimize()`, `run_optimizer_sweep()`, `run_cost_sweep()`, `compute_prices()`, `calculate_elcc_metrics()`

### Web Frontend (`web/src/`)

**State Management** (Zustand stores)
- `simulationStore.ts`: Zone data, config, results, runSimulation()
- `settingsStore.ts`: 68 cost parameters with localStorage persistence
- `sweepStore.ts`, `elccStore.ts`, `pricingStore.ts`, `uiStore.ts`

**Visualizations** (`features/visualization/`)
- `WeeklyChart.tsx`: 168-hour stacked bar
- `AnnualHeatmap.tsx`: Full-year generation patterns
- `BatteryChart.tsx`: State of charge over 8760 hours
- `LcoeChart.tsx`: LCOE breakdown by technology
- `PriceChart.tsx`: Hourly electricity prices
- `CostSweepChart.tsx`, `OptimizerSweepChart.tsx`

## Build Commands

```bash
# Rust/WASM
cd rust
wasm-pack build --target web --out-dir ../web/src/lib/wasm/pkg
cargo test --release
cargo bench

# Web
cd web
npm install
npm run dev          # Dev server with hot reload
npm run build        # Production build → dist/
npm run type-check   # TypeScript validation
```

## Critical Implementation Details

### 1. Battery Initial State
Battery MUST start at 100% full (`storage_capacity`), not 0%. This affects early deficit responses and clean match calculations.

### 2. Clean Match Target is EXACT
The optimizer MUST hit the specified target precisely (±0.2%), not treat it as a minimum. Overshooting is penalized.

### 3. Type Synchronization (CRITICAL)
Rust types in `rust/src/types.rs` must EXACTLY match TypeScript types in `web/src/types/index.ts`:

```rust
// Rust
pub enum BatteryMode { Default = 0, PeakShaver = 1, Hybrid = 2 }
```
```typescript
// TypeScript
export enum BatteryMode { Default = 0, PeakShaver = 1, Hybrid = 2 }
```

When adding new fields to `CostParams`:
1. Update `rust/src/types.rs`
2. Update `web/src/types/index.ts`
3. Increment version in `settingsStore.ts` persist config
4. Rebuild WASM

### 4. WASM Initialization
```typescript
const wasm = await import('../lib/wasm/pkg')
await wasm.default()  // MUST call as function!
```

### 5. Round-Trip Efficiency
Applied on discharge only: `energy_delivered = discharge * efficiency`

## Performance Targets

| Operation | Python | Rust/WASM |
|-----------|--------|-----------|
| Simulation (8760h) | 200-300ms | ~15ms |
| LCOE calculation | 50ms | ~3ms |
| Optimizer sweep | 3-8s | ~300-700ms |
| UI Response | 200-300ms | <16ms (60fps) |

## Common Modification Patterns

**Adding a new cost parameter:**
1. Add field to `CostParams` in `rust/src/types.rs`
2. Mirror in `web/src/types/index.ts`
3. Add to `DEFAULT_COSTS` in both places
4. Update `SettingsModal.tsx` if user-configurable
5. Increment persist version in `settingsStore.ts`
6. Use in LCOE calculation (`economics/lcoe.rs`)

**Adding a new battery mode:**
1. Add variant to `BatteryMode` enum in both Rust and TypeScript
2. Create new module in `rust/src/simulation/battery/`
3. Update `simulate_system()` dispatch in `core.rs`
4. Update UI selector if needed

**Adding a new chart:**
1. Create component in `web/src/features/visualization/`
2. Add to visualization selector in `VisualizationPanel.tsx`
3. Add any required store state

## Testing

```bash
# Rust tests
cd rust && cargo test --release

# Generate fixtures from Python baseline
python fixtures/generate_fixtures.py

# Cross-validate Rust vs Python
python compare_implementations.py
python fixtures/validate_lcoe.py
```

## Deployment

The web app is a static site (pure client-side WASM). Build with `npm run build` and deploy `dist/` to any static host (GitHub Pages, Vercel, Netlify).

## Key Constants

- `HOURS_PER_YEAR = 8760`
- Default battery efficiency: 85%
- Debounce delay: 50ms (prevents UI blocking)
- Optimizer tolerance: 0.2% (exact constraint compliance)

---

## Microsite: Educational Scrollytelling

The `microsite/` directory contains an interactive explainer demonstrating clean energy accounting concepts (inputs vs offsets framework).

### Structure

```
microsite/
├── generate_weekly_data.py   # Data generator (Python → JSON/TS)
├── OVERVIEW.md               # Full narrative framework
├── CLAUDE.md                 # Microsite-specific guidance
├── quick.md                  # Tech stack reference
└── site/                     # React app
    ├── src/
    │   ├── components/
    │   │   ├── chapters/         # Narrative sections (scroll sequence)
    │   │   ├── visualizations/   # D3/Framer charts
    │   │   ├── ui/               # Shared components
    │   │   └── layout/           # Header, Footer, Section
    │   ├── hooks/                # useInView, useScrollProgress
    │   ├── data/                 # Pre-computed simulation data (JSON/TS)
    │   └── lib/colors.ts         # Semantic color palette
    ├── dist/                     # Built output
    └── package.json
```

**Tech Stack**: React + TypeScript, Framer Motion (animations), D3.js (charts), Tailwind CSS, Vite

### Narrative Flow (12 Chapters)

The microsite scrolls through these chapters to build understanding:

1. **Hook** → "You're 100% renewable. Or are you?"
2. **Framework** → Introduce inputs vs offsets
3. **The Split** → Signature animation: 100% → 60/40 reveal
4. **Divergence** → Solar vs Wind vs CF efficiency curves
5. **Volume** → Why 100% annual matching isn't enough
6. **Scale Problem** → Small buyer vs system-wide impact
7. **Blueprint** → Did you build replicable infrastructure?
8. **Shape** → Greedy vs matching portfolios
9. **Storage Reality** → What batteries actually do (contract vs economic dispatch)
10. **Storage Fix** → Capability matching solution
11. **Build It** → Investment priority principle
12. **Synthesis** → Complete framework summary

---

## Data Pipeline: Simulation → Microsite Visualization

The microsite uses **pre-computed data** from the simulation engine rather than running WASM in-browser. This keeps the microsite lightweight and allows editorial control over which scenarios to visualize.

### Data Flow Architecture

```
┌─────────────────────────────────────────────────────────────────────┐
│  1. SIMULATION ENGINE (Python or Rust)                              │
│     - 8760-hour chronological simulation                            │
│     - Battery dispatch (default/peak_shaver/hybrid)                 │
│     - LCOE calculation with ITCs, depreciation, tax                 │
│     - Optimizer finds portfolios for target clean match %           │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│  2. DATA GENERATOR SCRIPT (microsite/generate_*.py)                 │
│     - Imports simulation/optimizer from parent directory            │
│     - Configures scenario (zone, target %, load shape, costs)       │
│     - Runs simulation → extracts representative data                │
│     - Outputs JSON + TypeScript to site/src/data/                   │
└─────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────┐
│  3. MICROSITE (React + D3/Framer Motion)                            │
│     - Imports pre-computed data as TypeScript modules               │
│     - Renders scroll-triggered visualizations                       │
│     - Animates to reveal insights (split, build-up, transition)     │
└─────────────────────────────────────────────────────────────────────┘
```

### Primary Tool: `generate_simulation_data.py`

This is the main script agents should use to generate visualization data. It supports all the "levers" via CLI args or Python API.

**CLI Examples:**
```bash
cd microsite

# Optimize for 80% clean match in Texas
python generate_simulation_data.py --zone Texas --target 80

# Manual portfolio: 150 MW solar, 50 MW wind, 100 MWh storage
python generate_simulation_data.py --solar 150 --wind 50 --storage 100

# Specific week (week 25 = late June)
python generate_simulation_data.py --zone California --target 90 --week 25

# Daily view of a specific day
python generate_simulation_data.py --target 70 --day 180 --view daily

# High gas price scenario
python generate_simulation_data.py --target 90 --gas-price 8.0

# Solar-only portfolio (no wind, no CF)
python generate_simulation_data.py --target 60 --no-wind --no-cf

# Custom output name
python generate_simulation_data.py --target 50 --output scenario_low
python generate_simulation_data.py --target 90 --output scenario_high
```

**Python API for Agents:**
```python
from generate_simulation_data import (
    generate_weekly_data,
    generate_daily_data,
    generate_comparison,
    GeneratorConfig,
    generate
)

# Simple weekly generation
data = generate_weekly_data(zone="Texas", target=80, week=25)

# Daily view
data = generate_daily_data(zone="California", target=70, day=180)

# Compare multiple scenarios
data = generate_comparison([
    {'name': 'low', 'target': 50, 'zone': 'California'},
    {'name': 'medium', 'target': 75, 'zone': 'California'},
    {'name': 'high', 'target': 95, 'zone': 'California'}
])

# Full control via GeneratorConfig
config = GeneratorConfig(
    zone="ERCOT",
    target_clean_match=85,
    use_solar=True,
    use_wind=True,
    use_storage=True,
    use_clean_firm=False,  # No clean firm
    battery_mode="hybrid",
    gas_price=8.0,  # High gas
    view="weekly",
    week=None,  # Auto-select best
    output_name="ercot_no_cf"
)
data = generate(config)
```

**Available Levers:**
| Lever | CLI Flag | Default | Description |
|-------|----------|---------|-------------|
| Zone | `--zone` | California | 13 US regions with different profiles |
| Target | `--target` | (required) | Clean match target 0-100% |
| View | `--view` | weekly | `weekly` (168h) or `daily` (24h) |
| Week/Day | `--week` / `--day` | auto | Specific period or auto-select best |
| Solar | `--solar` | (optimizer) | Manual capacity MW |
| Wind | `--wind` | (optimizer) | Manual capacity MW |
| Storage | `--storage` | (optimizer) | Manual capacity MWh |
| Clean Firm | `--clean-firm` | (optimizer) | Manual capacity MW |
| Disable resources | `--no-solar`, `--no-wind`, `--no-storage`, `--no-cf` | all enabled | Constrain optimizer |
| Battery mode | `--battery-mode` | hybrid | `default`, `peak_shaver`, `hybrid` |
| Gas price | `--gas-price` | $4/MMBtu | $/MMBtu override |
| Solar CAPEX | `--solar-capex` | $1000/kW | $/kW override |
| Output name | `--output` | simulation | Filename (no extension) |

### Agent Workflow: Creating New Visualizations

When an agent needs to create visualizations for complex energy concepts:

**Step 1: Define the insight to communicate**
- What question does the chart answer?
- What's the "aha moment" for the reader?
- What parameters need to vary (target %, costs, zone)?

**Step 2: Create a data generator script**
```bash
# Pattern: microsite/generate_<concept>_data.py
microsite/generate_divergence_curves.py    # Solar vs Wind saturation
microsite/generate_overbuild_analysis.py   # Volume effect
microsite/generate_portfolio_compare.py    # Greedy vs matching
```

**Step 3: Generate the data**
```bash
cd microsite
python generate_<concept>_data.py
# Outputs: site/src/data/<concept>.json + .ts
```

**Step 4: Build the visualization component**
```typescript
// site/src/components/visualizations/<Concept>Chart.tsx
import { conceptData } from '../../data/conceptData';
// Use D3 for data-driven elements, Framer Motion for animations
```

**Step 5: Wire into scroll sequence**
```typescript
// site/src/components/chapters/Chapter<N>.tsx
import { ConceptChart } from '../visualizations/ConceptChart';
// Trigger on scroll intersection
```

### Available Data Generation Patterns

| Pattern | Use Case | Generator Example |
|---------|----------|-------------------|
| **Weekly slice** | Show 168h dispatch dynamics | `generate_weekly_data.py` |
| **Sweep curve** | Show how metric changes with parameter | Run `optimizer_sweep`, extract points |
| **Comparison** | Side-by-side scenarios | Generate two scenarios, combine |
| **Heatmap** | Full-year patterns | Extract 8760h array, reshape to 365×24 |
| **Divergence** | Technology crossover points | Sweep clean_match 0→100%, track each tech |

### Design Rules for Microsite Charts

1. **One point per chart** - Each visualization makes exactly ONE argument
2. **Three variable maximum** - No chart shows >3 data dimensions
3. **Animate with purpose** - Animate to reveal transformation, not decoration
4. **Consistent colors** across all charts:
   ```typescript
   solar: '#F59E0B'      // Amber
   wind: '#3B82F6'       // Blue
   storage: '#8B5CF6'    // Purple
   cleanFirm: '#10B981'  // Emerald
   gas: '#6B7280'        // Gray
   inputs: '#10B981'     // Green (matched = good)
   offsets: '#FBBF24'    // Yellow (caution)
   ```

### Build & Deploy Microsite

```bash
cd microsite/site
npm install
npm run dev      # Dev server at localhost:5173
npm run build    # Production build → dist/
```

Deploy `dist/` to any static host (GitHub Pages, Vercel, Netlify).
