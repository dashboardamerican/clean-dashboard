//! Incremental Cost Walk Optimizer
//!
//! Rust port of Python's `run_incremental_cost_walk` (in `optimizer.py`).
//!
//! This optimizer starts from a zero-capacity baseline and incrementally adds
//! the most cost-effective resource (smallest LCOE-per-percentage-point ratio)
//! until either the clean-match target is reached, no improvement is possible,
//! or a hard step limit is hit. Step sizes are halved when a candidate move
//! overshoots the target by more than the overshoot tolerance.
//!
//! The algorithm mirrors the Python reference exactly:
//! - Initial step sizes: solar=100 MW, wind=100 MW, storage=200 MWh, clean_firm=50 MW
//! - Minimum step sizes: solar=1, wind=1, storage=2, clean_firm=1
//! - Resource caps: solar=1000, wind=500, storage=2400, clean_firm=125
//! - Match-improvement floor: 0.01 percentage points
//! - Overshoot tolerance: 0.5 percentage points above target
//! - Maximum steps: 100
//! - Targets at or above 100% are capped to 99.5%

use crate::economics::calculate_lcoe;
use crate::simulation::simulate_system;
use crate::types::{
    BatteryMode, CostParams, IncrementalWalkResult, SimulationConfig, WalkStep, HOURS_PER_YEAR,
};

/// Resource identifier inside the walk loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Resource {
    Solar,
    Wind,
    Storage,
    CleanFirm,
}

impl Resource {
    fn name(self) -> &'static str {
        match self {
            Resource::Solar => "solar",
            Resource::Wind => "wind",
            Resource::Storage => "storage",
            Resource::CleanFirm => "clean_firm",
        }
    }
}

/// Capacities for the four resources tracked by the walk.
#[derive(Clone, Copy, Debug, Default)]
struct Capacities {
    solar: f64,
    wind: f64,
    storage: f64,
    clean_firm: f64,
}

impl Capacities {
    fn add(mut self, res: Resource, delta: f64) -> Self {
        match res {
            Resource::Solar => self.solar += delta,
            Resource::Wind => self.wind += delta,
            Resource::Storage => self.storage += delta,
            Resource::CleanFirm => self.clean_firm += delta,
        }
        self
    }
}

/// Per-resource step state.
#[derive(Clone, Copy, Debug)]
struct StepState {
    current: f64,
    min: f64,
    max_capacity: f64,
}

/// Result of evaluating a candidate portfolio.
#[derive(Clone, Copy, Debug)]
struct EvalResult {
    lcoe: f64,
    clean_match: f64,
}

fn evaluate(
    caps: Capacities,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    battery_mode: BatteryMode,
    battery_efficiency: f64,
    max_demand_response: f64,
) -> Result<EvalResult, String> {
    let sim_config = SimulationConfig {
        solar_capacity: caps.solar,
        wind_capacity: caps.wind,
        storage_capacity: caps.storage,
        clean_firm_capacity: caps.clean_firm,
        battery_efficiency,
        max_demand_response,
        battery_mode,
    };

    let sim = simulate_system(&sim_config, solar_profile, wind_profile, load_profile)?;
    let lcoe = calculate_lcoe(
        &sim,
        caps.solar,
        caps.wind,
        caps.storage,
        caps.clean_firm,
        costs,
    );

    Ok(EvalResult {
        lcoe: lcoe.total_lcoe,
        clean_match: sim.clean_match_pct,
    })
}

fn make_walk_step(
    resource_added: &str,
    eval: EvalResult,
    baseline_lcoe: f64,
    caps: Capacities,
) -> WalkStep {
    WalkStep {
        match_percentage: eval.clean_match,
        lcoe: eval.lcoe,
        lcoe_premium: eval.lcoe - baseline_lcoe,
        resource_added: resource_added.to_string(),
        solar_capacity: caps.solar,
        wind_capacity: caps.wind,
        storage_capacity: caps.storage,
        clean_firm_capacity: caps.clean_firm,
    }
}

/// Run the incremental cost walk optimizer.
///
/// # Arguments
/// * `clean_match_target` - Target clean match percentage in 0-100. Values
///   `>= 100` are capped to 99.5 to mirror the Python reference behaviour.
/// * `solar_profile`, `wind_profile`, `load_profile` - 8760-hour input arrays
/// * `costs` - LCOE cost parameters
/// * `use_solar`, `use_wind`, `use_storage`, `use_clean_firm` - resource flags
/// * `battery_mode` - dispatch strategy
/// * `battery_efficiency` - round-trip efficiency (0-1); pass 0.85 for the default
/// * `max_demand_response` - MW of demand response (typically 0.0)
///
/// # Returns
/// `IncrementalWalkResult` with the full walk trace and the final portfolio.
#[allow(clippy::too_many_arguments)]
pub fn run_incremental_walk(
    clean_match_target: f64,
    solar_profile: &[f64],
    wind_profile: &[f64],
    load_profile: &[f64],
    costs: &CostParams,
    use_solar: bool,
    use_wind: bool,
    use_storage: bool,
    use_clean_firm: bool,
    battery_mode: BatteryMode,
    battery_efficiency: f64,
    max_demand_response: f64,
) -> Result<IncrementalWalkResult, String> {
    if solar_profile.len() != HOURS_PER_YEAR
        || wind_profile.len() != HOURS_PER_YEAR
        || load_profile.len() != HOURS_PER_YEAR
    {
        return Err(format!(
            "Profiles must each have {} hours (got solar={}, wind={}, load={})",
            HOURS_PER_YEAR,
            solar_profile.len(),
            wind_profile.len(),
            load_profile.len()
        ));
    }

    // Cap target at 99.5 when 100 is requested (matches Python reference).
    let target = if clean_match_target >= 100.0 {
        99.5
    } else {
        clean_match_target
    };

    // Build the enabled-resource list and per-resource step state.
    let mut enabled: Vec<(Resource, StepState)> = Vec::with_capacity(4);
    if use_solar {
        enabled.push((
            Resource::Solar,
            StepState {
                current: 100.0,
                min: 1.0,
                max_capacity: 1000.0,
            },
        ));
    }
    if use_wind {
        enabled.push((
            Resource::Wind,
            StepState {
                current: 100.0,
                min: 1.0,
                max_capacity: 500.0,
            },
        ));
    }
    if use_storage {
        enabled.push((
            Resource::Storage,
            StepState {
                current: 200.0,
                min: 2.0,
                max_capacity: 2400.0,
            },
        ));
    }
    if use_clean_firm {
        enabled.push((
            Resource::CleanFirm,
            StepState {
                current: 50.0,
                min: 1.0,
                max_capacity: 125.0,
            },
        ));
    }

    let mut caps = Capacities::default();

    // Baseline evaluation at zero capacities.
    let baseline = evaluate(
        caps,
        solar_profile,
        wind_profile,
        load_profile,
        costs,
        battery_mode,
        battery_efficiency,
        max_demand_response,
    )?;

    let baseline_lcoe = baseline.lcoe;
    let mut current_lcoe = baseline.lcoe;
    let mut current_match = baseline.clean_match;

    let mut walk_trace: Vec<WalkStep> = Vec::new();
    walk_trace.push(make_walk_step("baseline", baseline, baseline_lcoe, caps));

    const MAX_STEPS: u32 = 100;
    const OVERSHOOT_TOLERANCE: f64 = 0.5;
    const MATCH_IMPROVEMENT_FLOOR: f64 = 0.01;

    let mut step: u32 = 0;

    while step < MAX_STEPS {
        // Stop if the current portfolio already meets the target.
        if current_match >= target {
            break;
        }

        step += 1;

        let mut best_resource: Option<Resource> = None;
        let mut best_eval: Option<EvalResult> = None;
        let mut best_caps: Capacities = caps;
        let mut best_cost_effectiveness: f64 = f64::INFINITY;
        let mut best_increment: f64 = 0.0;

        for &(res, state) in enabled.iter() {
            let increment = state.current;
            let cap_field = match res {
                Resource::Solar => caps.solar,
                Resource::Wind => caps.wind,
                Resource::Storage => caps.storage,
                Resource::CleanFirm => caps.clean_firm,
            };

            // Respect the per-resource maximum capacity.
            if cap_field + increment > state.max_capacity {
                continue;
            }

            let test_caps = caps.add(res, increment);

            let test_eval = match evaluate(
                test_caps,
                solar_profile,
                wind_profile,
                load_profile,
                costs,
                battery_mode,
                battery_efficiency,
                max_demand_response,
            ) {
                Ok(v) => v,
                Err(_) => continue,
            };

            let match_improvement = test_eval.clean_match - current_match;
            let lcoe_increase = test_eval.lcoe - current_lcoe;

            if match_improvement > MATCH_IMPROVEMENT_FLOOR {
                let cost_effectiveness = lcoe_increase / match_improvement;
                if cost_effectiveness < best_cost_effectiveness {
                    best_cost_effectiveness = cost_effectiveness;
                    best_resource = Some(res);
                    best_eval = Some(test_eval);
                    best_caps = test_caps;
                    best_increment = increment;
                }
            }
        }

        // No candidate improved match enough -> stop.
        let (best_resource, best_eval) = match (best_resource, best_eval) {
            (Some(r), Some(e)) => (r, e),
            _ => break,
        };
        let _ = best_increment; // kept for parity with Python (informational)

        // Overshoot logic: if match exceeds target by more than tolerance and
        // the resource's step is still above its minimum, halve the step and retry.
        if best_eval.clean_match > target + OVERSHOOT_TOLERANCE {
            let entry = enabled
                .iter_mut()
                .find(|(r, _)| *r == best_resource)
                .expect("resource state must exist for selected resource");
            let state = &mut entry.1;
            if state.current > state.min {
                let new_step = state.current / 2.0;
                state.current = if new_step > state.min {
                    new_step
                } else {
                    state.min
                };
                // Retry without consuming this step (Python: `continue` after decrement
                // keeps `step` incremented, matching its `max_steps` accounting).
                continue;
            }
            // At the minimum step: accept the overshoot.
        }

        // Apply the best option.
        caps = best_caps;
        current_lcoe = best_eval.lcoe;
        current_match = best_eval.clean_match;

        walk_trace.push(make_walk_step(
            best_resource.name(),
            best_eval,
            baseline_lcoe,
            caps,
        ));
    }

    Ok(IncrementalWalkResult {
        solar_capacity: caps.solar,
        wind_capacity: caps.wind,
        storage_capacity: caps.storage,
        clean_firm_capacity: caps.clean_firm,
        final_lcoe: current_lcoe,
        achieved_match: current_match,
        steps: step,
        walk_trace,
    })
}
