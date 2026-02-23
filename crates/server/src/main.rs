use civ_engine::{step, WorldState};
use civ_metrics::compute;
use civ_policy::{effective_consumption, PolicyInput};

fn main() {
    let state = WorldState::default();
    let consumption = effective_consumption(PolicyInput {
        base_consumption_joules: 5.0e9,
        scarcity_multiplier: 1.0,
    });

    let next = step(state, consumption);
    let metrics = compute(next.energy_budget_joules, consumption);

    println!(
        "tick={} energy={} waste={} surplus={} tyranny={} legitimacy={}",
        next.tick,
        next.energy_budget_joules,
        metrics.waste_joules,
        metrics.surplus_joules,
        metrics.tyranny_index,
        metrics.legitimacy_index
    );
}
