//! Dormant emergence phases — religion (belief/unrest/institutions/cohesion) and
//! macro psyche rollup wired into [`Simulation::tick`] (FR-CIV-REL-001..003,
//! FR-CIV-PSYCHE-900 family, `fr-emergence-matrix` P3).
//!
//! Micro psyche/culture/social run inside [`super::emergence::Simulation::phase_emergence`];
//! this module owns the scalar religion loop and upward psyche→belief coupling.

use super::{Simulation, engine::avg_psyche_maturity};

impl Simulation {
    /// Macro psyche rollup — mature agents stabilize collective belief (FR-CIV-PSYCHE-N11).
    ///
    /// Agent-level mood/belief mutation runs in `phase_emergence`; this phase only
    /// projects average maturity upward when `Psyche` components are present.
    pub(crate) fn phase_psyche(&mut self) {
        let maturity = avg_psyche_maturity(&self.world);
        if maturity <= 0.0 {
            return;
        }
        let bonus = (maturity * 10.0).floor() as i64;
        if bonus > 0 {
            self.add_belief(bonus);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::engine::Simulation;

    /// Macro psyche rollup only raises belief when mature psyches are present;
    /// on a fresh sim with no psyche components it is a no-op.
    #[test]
    fn phase_psyche_is_noop_without_mature_agents() {
        let mut sim = Simulation::with_seed(42);
        sim.state.belief = 0;
        sim.phase_psyche();
        assert_eq!(
            sim.state.belief, 0,
            "phase_psyche must not raise belief when no mature psyches exist"
        );
    }

    #[test]
    fn dormant_phases_same_seed_deterministic() {
        let mut a = Simulation::with_seed(9_001);
        let mut b = Simulation::with_seed(9_001);
        for _ in 0..32 {
            a.tick();
            b.tick();
        }
        assert_eq!(a.state.belief, b.state.belief);
        assert_eq!(a.state.unrest, b.state.unrest);
        assert_eq!(a.state.cohesion, b.state.cohesion);
        assert_eq!(a.state.temple_level, b.state.temple_level);
    }
}
