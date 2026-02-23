#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WorldState {
    pub tick: u64,
    pub population: u64,
    pub energy_budget_joules: f64,
}

impl Default for WorldState {
    fn default() -> Self {
        Self {
            tick: 0,
            population: 1_000_000,
            energy_budget_joules: 1.0e12,
        }
    }
}

pub fn step(mut state: WorldState, consumption_joules: f64) -> WorldState {
    state.tick += 1;
    state.energy_budget_joules = (state.energy_budget_joules - consumption_joules).max(0.0);
    state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_advances_tick() {
        let s = WorldState::default();
        let n = step(s, 100.0);
        assert_eq!(n.tick, 1);
    }
}
