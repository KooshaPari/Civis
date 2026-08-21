//! Test helpers for cross-domain integration tests.
//!
//! Provides [`SimulationBuilder`] — a thin wrapper around
//! [`Simulation::with_seed`] that adds convenience methods for
//! bootstrapping test-ready simulations.

use super::Simulation;

/// Builder for creating test-ready `Simulation` instances.
///
/// # Example
/// ```ignore
/// let sim = SimulationBuilder::new(42).build();
/// assert_eq!(sim.current_tick(), 0);
/// ```
pub struct SimulationBuilder {
    sim: Simulation,
}

impl SimulationBuilder {
    pub fn new(seed: u64) -> Self {
        Self {
            sim: Simulation::with_seed(seed),
        }
    }

    /// Advance the simulation by N ticks to populate lifecycle state.
    pub fn advance_ticks(mut self, n: u64) -> Self {
        for _ in 0..n {
            self.sim.tick();
        }
        self
    }

    pub fn build(self) -> Simulation {
        self.sim
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_creates_simulation() {
        let sim = SimulationBuilder::new(42).build();
        assert_eq!(sim.current_tick(), 0);
    }

    #[test]
    fn builder_advances_ticks() {
        let sim = SimulationBuilder::new(42).advance_ticks(5).build();
        assert!(sim.current_tick() >= 5);
    }
}
