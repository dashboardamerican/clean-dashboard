# AGENTS Guide

## Purpose
This repository is a Rust/WASM energy system simulator with a React frontend and a separate narrative microsite.

Primary goals for agents:
- keep changes scoped and minimal
- preserve correctness and performance assumptions
- avoid accidental breakage across Rust/WASM and web bindings

This guide is for agent execution only: it should drive *how* to make a change safely, not replace project docs.

## Repo Surfaces
- `rust/`: Core engine (simulation, economics, optimizer), compiled to WASM for browser use.
- `web/`: Main interactive simulator app (React + Zustand stores).
- `microsite/site/`: Standalone educational/narrative frontend.
- `data/`: Zone input profiles (`zones.json`) and shared assets.
- `fixtures/`: Test fixture generation and cross-validation assets.
- `docs/`: Technical/process docs.

## Architecture to preserve
- WASM boundary in `rust/src/lib.rs` is the contract between compute and UI.
- Simulation/economics/optimization should stay in Rust for performance.
- UI orchestration should stay in `web/src/*`; avoid moving core logic into components.
- Do not remove/rename public WASM exports without updating all call sites.

## Editing priorities
1. `rust/src/lib.rs`: exported API signatures and glue code.
2. `rust/src/{types,simulation,economics,optimizer}/`: core algorithm changes.
3. `web/src/{hooks,stores,components}/`: data loading, parameter wiring, rendering updates.
4. Docs updates when behavior/contract changes.

## Safe change rules
- Do not use destructive git operations.
- Do not change unrelated files unless explicitly necessary.
- Do not run tests/builds unless the user explicitly asks.
- Use existing patterns (typed interfaces, enum mappings, caching patterns).
- For Rust/WASM changes, keep payload shapes aligned between:
  - `rust/src/types.rs`
  - `rust/src/lib.rs`
  - `web/src/types/index.ts`
  - `web/src/hooks/useWasm.ts` and `web/src/stores/simulationStore.ts`

## Quick RALPH Loop (5 Iterations)
**RALPH here means an iterative build-check-fix loop, not a hard acronym.**

Use this for non-trivial changes, including docs, Rust/WASM logic, and optimizer experiments.

### Iteration 1 — Re-anchor
- Read the relevant contracts first: `AGENTS.md` + `rust/src/lib.rs`, `rust/src/types.rs`, and paired TS types.
- Define 1–3 acceptance criteria that are measurable (runtime bounds, error-free build, one output metric improvement target).

### Iteration 2 — Risk map
- Call out likely failures before edits:
  - Rust ↔ TS API drift
  - numeric unit/regression risk
  - timing regressions from added constraints
  - fallback behavior changes
- Assign risk levels and concrete mitigations.

### Iteration 3 — Minimal edit
- Make the smallest behavioral change to satisfy criteria.
- Keep scope narrow; avoid refactors or unrelated formatting churn.

### Iteration 4 — Local consistency pass
- Update dependent mappings/docs only when required by the primary change.
- Re-run only the fastest relevant checks for this loop.
- Record objective evidence (commands and outputs) in the PR/notes.

### Iteration 5 — Handoff
- Summarize what changed, why, files touched, and remaining risk.
- Include explicit next steps if optimization quality or speed is still not within target.

## RALPH completion criteria
- If any iteration reveals unknown risk or scope drift, pause and ask for direction.
- Exit only when acceptance criteria are met and checks are recorded.
- Do not expand into nice-to-have cleanup while in loop mode.

## Robust testing requirements
- For any optimizer-related change, capture all of the following before marking iteration complete:
  1. `cargo test --release --features "experimental-v3 native" --test v3_smoke -- --nocapture`
  2. `cargo test --release --features "experimental-v3 native" --test v3_oracle_consistency -- --nocapture`
  3. `cargo run --release --features "experimental-v3 native" --bin bench_v3`
- Record pass/fail, runtime, and notable deltas in AGENTS notes for traceability.
- Treat unexpected warnings as follow-up items; fail the loop if new warnings indicate behavior change risk (new compiler errors still fail immediately).

## Suggested optimizer loop (v3)
When working on optimizer performance improvements, use this concrete 5-step pattern:

1) Define objective: e.g., "V3 succeeds on 0–95% targets with ≤ 3x V2 runtime at 95%."  
2) Baseline: run the relevant target set and capture `bench_v3` output in `rust/` (or add comparable command output).  
3) Edit: adjust one thing only (parameter range, pruning rule, step granularity).  
4) Verify: run `cargo run --release --features "experimental-v3 native" --bin bench_v3` and at least one focused unit test.  
5) Compare: document deltas for success-rate, runtime, and output quality before deciding next change.  

Stop after 5 iterations unless risk/benefit clearly still improving and you want an extension.

## Performance defaults
- Avoid unnecessary allocations in hot paths (8760-hour arrays).
- Favor reusing pre-allocated buffers when touching simulation internals.
- Keep debounce and async cancellation in the UI to avoid blocking interactions.

## Security and reliability constraints
- Treat fetched model/profile assets as untrusted input at runtime.
- Preserve graceful fallback behavior if assets fail to load.
- Keep WASM errors surfaced as user-visible state, not silent no-ops.

## Command references
- Build Rust WASM (from repo root): `cd rust && wasm-pack build --target web --out-dir ../web/src/wasm/pkg`
- Run Rust tests (when asked): `cd rust && cargo test --release`
- Build web app: `cd web && npm run build`
- Validate TypeScript types: `cd web && npm run type-check`

## Quick change checklist (default)
- [ ] I changed the right layer (`rust/` for compute, `web/` for UI wiring, not both unless needed).
- [ ] I kept Rust/TS contracts aligned (`types.rs`, `lib.rs`, `web/src/types/index.ts`).
- [ ] I preserved existing fallback behavior for missing assets/modules.
- [ ] I updated docs/comments where user-facing behavior changed.
- [ ] I listed residual risks and recommended follow-up validation.

## What to include in PRs
- Short summary of what changed and why.
- Files touched and data-flow impact.
- Any compatibility or behavior changes (API/UX/numerical outputs).
- Manual quick test scenario if automated tests were not run.

## Layer ownership map
- Core simulation/economics/optimizer logic: `rust/src/*`
- WASM API boundary: `rust/src/lib.rs`
- Simulator control and state flow: `web/src/stores/*`
- Visualization components: `web/src/features/*` and `web/src/components/*`
- Microsite narrative only: `microsite/site/*`

When in doubt, prefer changing the highest layer possible that solves the issue.
