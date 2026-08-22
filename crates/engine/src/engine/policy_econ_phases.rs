//! Economic focus & policy phases extracted from engine.rs (Pass 8).

use super::{EconomicFocus, EconomicFocusEvent};
use crate::emergence_coupling::candidate_economic_focus;
use crate::policy::ControlSignals;
use crate::Simulation;
use crate::SCALE;

impl Simulation {
    /// Economic-focus pre-pass (FR-CIV-ECON-001 / ADR-020).
    pub(crate) fn phase_economic_focus_pre(&mut self) {
        let research_tier = self.research_tier();
        let economy_i = self.economy_state.energy_budget_joules;
        let belief = self.belief;
        let treasury_total: i64 = self
            .state
            .faction_treasury
            .values()
            .map(|v| i64::from(v.to_bits()) / crate::SCALE)
            .sum();

        let settlement_ids: Vec<u32> = self.settlements.keys().copied().collect();
        for sid in settlement_ids {
            let current = self
                .econ_focus
                .get(&sid)
                .copied()
                .unwrap_or(EconomicFocus::Balanced);

            let pop = self.settlements[&sid];
            let stocked = self.settlement_food_stocked.get(&sid).copied().unwrap_or(0);
            let food_surplus = economy_i.saturating_mul(pop as i64) + stocked as i64;
            let food = food_surplus.max(0);

            let ideal = candidate_economic_focus(food, research_tier, belief, treasury_total);

            if ideal != current {
                let cause = format!(
                    "pre: food={} tier={} belief={} treasury={} -> {:?}",
                    food, research_tier, belief, treasury_total, ideal
                );
                self.econ_focus_stability.push(EconomicFocusEvent {
                    settlement_id: sid,
                    from: current,
                    to: ideal,
                    cause,
                });
            }
        }
    }

    /// Economic-focus phase (FR-CIV-ECON-001 / ADR-020).
    pub(crate) fn phase_economic_focus(&mut self) {
        for event in &self.econ_focus_stability {
            self.econ_focus.insert(event.settlement_id, event.to);
        }
    }

    /// Policy phase (FR-CORE-005).
    pub(crate) fn phase_policy(&mut self) {
        self.last_control_signals = self.policy.evaluate(&self.state);
    }
}
