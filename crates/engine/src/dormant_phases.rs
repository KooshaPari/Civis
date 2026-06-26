//! Dormant emergence phases — religion (belief/unrest/institutions/cohesion) and
//! macro psyche rollup wired into [`Simulation::tick`] (FR-CIV-REL-001..003,
//! FR-CIV-PSYCHE-900 family, `fr-emergence-matrix` P3).
//!
//! Micro psyche/culture/social run inside [`super::emergence::Simulation::phase_emergence`];
//! this module owns the scalar religion loop and upward psyche→belief coupling.

use super::{
    add_unrest_delta, avg_psyche_maturity, cohesion_delta, cohesion_unrest_damp,
    commodity_unrest_delta, energy_scarcity_unrest, institution_step, institution_target_level,
    kinship_cohesion_boost, micro_cohesion_delta, unrest_delta, Simulation,
    FOOD_SCARCITY_BASELINE,
};

/// Belief units required per temple level (FR-CIV-REL-003 / emergent-systems-tracelinks).
const BELIEF_PER_TEMPLE_LEVEL: u64 = 5_000;
/// Unrest units required per garrison level.
const UNREST_PER_GARRISON_LEVEL: u64 = 200;
/// Population divisor for per-tick belief accrual (FR-CIV-REL-001).
const BELIEF_POP_DIVISOR: u64 = 2_000;

impl Simulation {
    /// Belief accrual from population and temple institutions (FR-CIV-REL-001/003).
    pub(crate) fn phase_belief(&mut self) {
        let from_pop = (self.state.population / BELIEF_POP_DIVISOR) as i64;
        let from_temple = self.state.temple_level as i64;
        self.add_belief(from_pop.saturating_add(from_temple));
    }

    /// Societal unrest from scarcity/misery; hardship feeds belief (FR-CIV-REL-002).
    pub(crate) fn phase_unrest(&mut self) {
        let food_price = self
            .market_state
            .prices()
            .get("food")
            .copied()
            .unwrap_or(FOOD_SCARCITY_BASELINE);
        let mut delta = unrest_delta(food_price);
        delta = delta.saturating_add(commodity_unrest_delta(self.market_state.prices()));
        delta = delta.saturating_add(super::agent_misery_unrest(&self.world));
        delta = delta.saturating_add(energy_scarcity_unrest(self.state.energy_budget_joules));
        delta = super::research_unrest_mitigation(delta, self.research_tier());
        delta = cohesion_unrest_damp(delta, self.state.cohesion);
        add_unrest_delta(&mut self.state.unrest, delta);

        if self.state.unrest > 0 {
            self.add_belief((self.state.unrest / 100) as i64);
        }
    }

    /// Social fabric bind/fray from belief vs unrest plus micro/kinship upward causation.
    pub(crate) fn phase_cohesion(&mut self) {
        let macro_delta = cohesion_delta(self.state.belief, self.state.unrest);
        let micro = micro_cohesion_delta(&self.world);
        let kinship = kinship_cohesion_boost(&self.world);
        self.add_cohesion(macro_delta.saturating_add(micro).saturating_add(kinship));
    }

    /// Temple/garrison levels step toward belief/unrest targets (FR-CIV-REL-003).
    pub(crate) fn phase_institutions(&mut self) {
        let temple_target =
            institution_target_level(self.state.belief, BELIEF_PER_TEMPLE_LEVEL);
        let garrison_target =
            institution_target_level(self.state.unrest, UNREST_PER_GARRISON_LEVEL);
        self.state.temple_level = institution_step(self.state.temple_level, temple_target);
        self.state.garrison_level = institution_step(self.state.garrison_level, garrison_target);
    }

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
    use super::*;
    use crate::engine::Simulation;

    #[test]
    fn phase_belief_accrues_from_population() {
        let mut sim = Simulation::with_seed(42);
        sim.state.population = 10_000;
        sim.state.belief = 0;
        sim.state.temple_level = 0;
        sim.phase_belief();
        assert_eq!(sim.state.belief, 5, "+1 belief per pop/2000");
    }

    #[test]
    fn phase_unrest_feeds_belief_under_hardship() {
        let mut sim = Simulation::with_seed(42);
        sim.state.unrest = 500;
        sim.state.belief = 0;
        sim.phase_unrest();
        assert!(
            sim.state.belief > 0,
            "unrest > 0 must feed belief same tick (got {})",
            sim.state.belief
        );
    }

    #[test]
    fn phase_institutions_grows_temple_with_belief() {
        let mut sim = Simulation::with_seed(42);
        sim.state.belief = 10_000;
        sim.state.temple_level = 0;
        sim.phase_institutions();
        assert!(
            sim.state.temple_level > 0,
            "belief should drive temple growth"
        );

        sim.state.population = 0;
        sim.state.belief = 0;
        sim.state.temple_level = 3;
        sim.phase_belief();
        assert_eq!(
            sim.state.belief, 3,
            "temple_level adds +level belief per tick"
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
