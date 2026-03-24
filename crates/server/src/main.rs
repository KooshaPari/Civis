use civ_engine::{metrics, step, Fixed, WorldState};

fn main() {
    let state = WorldState::default();

    // Direct Fixed calculation instead of going through policy module
    let base = Fixed::from_num(5_000_000_000i64); // 5e9 joules
    let consumption = base; // No scarcity multiplier for now

    let next = step(state, consumption);
    let m = metrics::compute(next.energy_budget_joules.to_f64(), consumption.to_f64());

    println!(
        "tick={} energy={} waste={} surplus={} tyranny={} legitimacy={}",
        next.tick,
        next.energy_budget_joules.to_f64(),
        m.waste_joules,
        m.surplus_joules,
        m.tyranny_index,
        m.legitimacy_index
    );
}
