# Clean Dashboard

Interactive energy-system simulator and optimizer. Rust core compiled to WebAssembly, React frontend, deployed as a static site on GitHub Pages.

## Live site

After deploying, the site will be available at `https://<owner>.github.io/<repo-name>/`.

## Local development

```bash
npm install
npm run dev
```

Open http://localhost:3000.

## Build

```bash
npm run build
```

The build output is written to `dist/`. To preview the production build:

```bash
npm run preview
```

## Rebuilding the WASM module

The pre-built WASM module is committed at `src/lib/wasm/pkg/`, so no Rust toolchain is required for normal development or deployment. To rebuild from `wasm-src/`:

```bash
# Install wasm-pack if needed
cargo install wasm-pack

# Rebuild
npm run wasm:build
```

## Deployment

Pushing to `main` triggers `.github/workflows/deploy.yml`, which builds the site and deploys it to GitHub Pages.

For the workflow to publish, the repo's **Settings → Pages → Source** must be set to **GitHub Actions**.

The workflow sets `BASE_PATH` to `/<repo-name>/` automatically, so you can rename or fork the repo without editing the config.

## Layout

```
.
├── .github/workflows/deploy.yml   # CI: build + deploy to Pages
├── docs/                          # Agent guides (CLAUDE.md, AGENTS.md, BATTERY_DISPATCH.md)
├── public/                        # Static assets served at base path
│   ├── data/zones.json            # 13-zone hourly profiles (~4 MB)
│   └── models/                    # Pre-computed empirical models (~44 MB total)
├── src/                           # React + TypeScript frontend
│   ├── App.tsx
│   ├── main.tsx
│   ├── components/
│   ├── features/                  # Visualization, optimizer, sweep panels
│   ├── stores/                    # Zustand state stores
│   ├── lib/
│   │   ├── wasm/pkg/              # Compiled WASM module (committed)
│   │   ├── modelLoader.ts
│   │   ├── worker-pool.ts         # Web-worker parallel sweeps
│   │   └── ...
│   └── types/                     # Shared TS types (must match wasm-src/src/types.rs)
├── wasm-src/                      # Rust source for the WASM core
│   ├── Cargo.toml
│   └── src/
│       ├── simulation/            # 8760-hour chronological simulator
│       ├── economics/             # LCOE, depreciation, pricing
│       └── optimizer/             # v2_hierarchical (production)
├── index.html
├── package.json
└── vite.config.ts
```

## Stack

React 18, TypeScript, Vite, Tailwind, Zustand, Plotly. Rust core compiled with `wasm-pack`. Optimization runs in a pool of Web Workers (`src/lib/worker-pool.ts`) for parallel sweeps.

## Origin

Extracted from the [Multi-Heatmap-Test](https://github.com/) monorepo, where it lives alongside the original Python/Dash implementation. This repo contains only the Rust/WASM rewrite, which is the version intended for deployment.
