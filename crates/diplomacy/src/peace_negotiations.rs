//! Peace Negotiations — structured conflict resolution system.
//!
//! Provides peace proposal, counter-proposal, evaluation, and execution.
//! War weariness is computed from conflict duration and casualties.
//! All values use fixed-point arithmetic (i32, scaled x100 where 100 = 1.0).
//!
//! # Determinism
//!
//! All computation is integer-only. Given the same inputs, the same functions
//! produce identical outputs. No RNG, no floating-point, no wall-clock.

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::PolityId;

/// Terms of a peace proposal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeaceTerms {
    /// Ceasefire duration in ticks.
    pub ceasefire_duration: u32,
    /// Reparations amount (resource units).
    pub reparations_amount: i32,
    /// Number of territory concessions.
    pub territory_concessions: u32,
    /// Disarmament level (0–100, percentage of forces to demobilize).
    pub disarmament_level: u32,
}

/// Status of a peace negotiation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PeaceNegotiationStatus {
    /// Initial proposal made.
    Proposed,
    /// Counter-proposal received.
    Countered,
    /// Terms accepted by both parties.
    Accepted,
    /// Terms rejected.
    Rejected,
}

/// A peace proposal between two factions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeaceProposal {
    /// Proposing faction.
    pub proposer: PolityId,
    /// Target faction.
    pub target: PolityId,
    /// Proposed terms.
    pub terms: PeaceTerms,
    /// Tick when the proposal was made.
    pub tick: u64,
}

/// An active peace negotiation wrapping proposals and status.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeaceNegotiation {
    /// The current (most recent) proposal.
    pub current_proposal: PeaceProposal,
    /// Negotiation status.
    pub status: PeaceNegotiationStatus,
    /// Number of rounds so far.
    pub round_count: u32,
    /// Maximum allowed rounds before forced resolution.
    pub max_rounds: u32,
    /// Tick when the negotiation started.
    pub started_at_tick: u64,
}

/// Outcome of a peace execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeaceOutcome {
    /// Whether the ceasefire was established.
    pub ceasefire_established: bool,
    /// Resources transferred as reparations.
    pub reparations_transferred: i32,
    /// Territory concessions applied.
    pub territories_conceded: u32,
    /// Forces demobilized (percentage points).
    pub forces_demobilized: u32,
}

/// Errors during peace operations.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PeaceError {
    /// Negotiation is already resolved (accepted or rejected).
    #[error("negotiation already resolved")]
    AlreadyResolved,
    /// Round limit has been reached.
    #[error("round limit reached ({0})")]
    RoundLimitReached(u32),
    /// The caller is not a party to this negotiation.
    #[error("actor {0} is not part of this negotiation")]
    ActorNotParty(PolityId),
    /// No proposal exists to evaluate or counter.
    #[error("no proposal exists yet")]
    NoProposal,
}

impl PeaceNegotiation {
    /// Create a new negotiation from an initial proposal.
    pub fn new(proposal: PeaceProposal, max_rounds: u32) -> Self {
        let tick = proposal.tick;
        Self {
            current_proposal: proposal,
            status: PeaceNegotiationStatus::Proposed,
            round_count: 1,
            max_rounds,
            started_at_tick: tick,
        }
    }

    /// Propose peace terms (initial or replacement).
    pub fn propose_peace(
        &mut self,
        proposer: PolityId,
        target: PolityId,
        terms: PeaceTerms,
        tick: u64,
    ) -> Result<(), PeaceError> {
        if self.status == PeaceNegotiationStatus::Accepted
            || self.status == PeaceNegotiationStatus::Rejected
        {
            return Err(PeaceError::AlreadyResolved);
        }
        if proposer != self.current_proposal.proposer && proposer != self.current_proposal.target {
            return Err(PeaceError::ActorNotParty(proposer));
        }
        if target != self.current_proposal.proposer && target != self.current_proposal.target {
            return Err(PeaceError::ActorNotParty(target));
        }
        self.current_proposal = PeaceProposal {
            proposer,
            target,
            terms,
            tick,
        };
        self.status = PeaceNegotiationStatus::Proposed;
        self.round_count = self.round_count.saturating_add(1);
        Ok(())
    }

    /// Counter-propose with modified terms.
    pub fn counter_propose(&mut self, terms: PeaceTerms, tick: u64) -> Result<(), PeaceError> {
        if self.status == PeaceNegotiationStatus::Accepted
            || self.status == PeaceNegotiationStatus::Rejected
        {
            return Err(PeaceError::AlreadyResolved);
        }
        if self.round_count >= self.max_rounds {
            return Err(PeaceError::RoundLimitReached(self.max_rounds));
        }
        // Counter from the other party.
        let (new_proposer, new_target) =
            (self.current_proposal.target, self.current_proposal.proposer);
        self.current_proposal = PeaceProposal {
            proposer: new_proposer,
            target: new_target,
            terms,
            tick,
        };
        self.status = PeaceNegotiationStatus::Countered;
        self.round_count = self.round_count.saturating_add(1);
        Ok(())
    }

    /// Evaluate whether the current terms are acceptable based on
    /// war weariness and strength ratio.
    ///
    /// Returns `true` if peace should be accepted. Acceptance occurs when:
    /// - The weaker side's war weariness exceeds 6000 (60%), OR
    /// - The strength ratio is ≥ 150 (1.5:1) favoring the proposer's side.
    pub fn evaluate_peace(
        &self,
        war_weariness_a: i32,
        war_weariness_b: i32,
        strength_ratio: i32,
    ) -> bool {
        let avg_weariness = (war_weariness_a + war_weariness_b) / 2;
        avg_weariness >= 6000 || strength_ratio >= 150
    }

    /// Accept the current proposal and mark the negotiation as resolved.
    pub fn accept(&mut self) -> Result<(), PeaceError> {
        if self.status == PeaceNegotiationStatus::Accepted
            || self.status == PeaceNegotiationStatus::Rejected
        {
            return Err(PeaceError::AlreadyResolved);
        }
        self.status = PeaceNegotiationStatus::Accepted;
        Ok(())
    }

    /// Reject the current proposal and mark the negotiation as resolved.
    pub fn reject(&mut self) -> Result<(), PeaceError> {
        if self.status == PeaceNegotiationStatus::Accepted
            || self.status == PeaceNegotiationStatus::Rejected
        {
            return Err(PeaceError::AlreadyResolved);
        }
        self.status = PeaceNegotiationStatus::Rejected;
        Ok(())
    }

    /// Execute the peace agreement (call after acceptance).
    ///
    /// Converts the current terms into a concrete [`PeaceOutcome`].
    pub fn execute_peace(&self) -> Result<PeaceOutcome, PeaceError> {
        if self.status != PeaceNegotiationStatus::Accepted {
            return Err(PeaceError::AlreadyResolved);
        }
        let terms = &self.current_proposal.terms;
        Ok(PeaceOutcome {
            ceasefire_established: terms.ceasefire_duration > 0,
            reparations_transferred: terms.reparations_amount,
            territories_conceded: terms.territory_concessions,
            forces_demobilized: terms.disarmament_level,
        })
    }
}

/// Compute war weariness for each side of a conflict.
///
/// Weariness is proportional to conflict duration and casualties, capped
/// at `max_weariness` (default 10000 = maximum fatigue). Returns
/// `(weariness_a, weariness_b)`.
///
/// Formula (per side): `min(max_weariness, duration_ticks / 100 + casualties * 10)`
pub fn war_weariness(
    conflict_duration_ticks: u64,
    casualties_a: u32,
    casualties_b: u32,
    max_weariness: i32,
) -> (i32, i32) {
    let duration_component = (conflict_duration_ticks / 100) as i32;
    let wa = (duration_component + casualties_a as i32 * 10).min(max_weariness);
    let wb = (duration_component + casualties_b as i32 * 10).min(max_weariness);
    (wa, wb)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn p(id: u32) -> PolityId {
        PolityId::new(id)
    }

    fn basic_terms() -> PeaceTerms {
        PeaceTerms {
            ceasefire_duration: 500,
            reparations_amount: 100,
            territory_concessions: 2,
            disarmament_level: 25,
        }
    }

    fn basic_proposal() -> PeaceProposal {
        PeaceProposal {
            proposer: p(1),
            target: p(2),
            terms: basic_terms(),
            tick: 100,
        }
    }

    #[test]
    fn new_negotiation_starts_with_proposed_status() {
        let neg = PeaceNegotiation::new(basic_proposal(), 5);
        assert_eq!(neg.status, PeaceNegotiationStatus::Proposed);
        assert_eq!(neg.round_count, 1);
        assert_eq!(neg.max_rounds, 5);
    }

    #[test]
    fn counter_propose_swaps_parties_and_increments_round() {
        let mut neg = PeaceNegotiation::new(basic_proposal(), 5);
        let counter_terms = PeaceTerms {
            ceasefire_duration: 300,
            reparations_amount: 50,
            territory_concessions: 1,
            disarmament_level: 10,
        };
        neg.counter_propose(counter_terms, 200)
            .expect("counter should succeed");
        assert_eq!(neg.status, PeaceNegotiationStatus::Countered);
        assert_eq!(neg.round_count, 2);
        // Proposer/target should be swapped.
        assert_eq!(neg.current_proposal.proposer, p(2));
        assert_eq!(neg.current_proposal.target, p(1));
    }

    #[test]
    fn counter_propose_rejects_after_resolution() {
        let mut neg = PeaceNegotiation::new(basic_proposal(), 5);
        neg.accept().expect("accept");
        let result = neg.counter_propose(basic_terms(), 300);
        assert!(matches!(result, Err(PeaceError::AlreadyResolved)));
    }

    #[test]
    fn counter_propose_rejects_at_round_limit() {
        let mut neg = PeaceNegotiation::new(basic_proposal(), 2);
        // Round 1: initial proposal. Round 2: counter.
        neg.counter_propose(basic_terms(), 200).expect("counter 1");
        let result = neg.counter_propose(basic_terms(), 300);
        assert!(matches!(result, Err(PeaceError::RoundLimitReached(2))));
    }

    #[test]
    fn evaluate_peace_high_weariness_approves() {
        let neg = PeaceNegotiation::new(basic_proposal(), 5);
        // Both sides at 70% weariness.
        assert!(neg.evaluate_peace(7000, 7000, 100));
    }

    #[test]
    fn evaluate_peace_favorable_ratio_approves() {
        let neg = PeaceNegotiation::new(basic_proposal(), 5);
        // Low weariness but 2:1 strength ratio.
        assert!(neg.evaluate_peace(1000, 1000, 200));
    }

    #[test]
    fn evaluate_peace_low_weariness_and_low_ratio_rejects() {
        let neg = PeaceNegotiation::new(basic_proposal(), 5);
        // 30% weariness, 1:1 ratio → neither threshold met.
        assert!(!neg.evaluate_peace(3000, 3000, 100));
    }

    #[test]
    fn execute_peace_after_accept_produces_outcome() {
        let mut neg = PeaceNegotiation::new(basic_proposal(), 5);
        neg.accept().expect("accept");
        let outcome = neg.execute_peace().expect("execute");
        assert!(outcome.ceasefire_established);
        assert_eq!(outcome.reparations_transferred, 100);
        assert_eq!(outcome.territories_conceded, 2);
        assert_eq!(outcome.forces_demobilized, 25);
    }

    #[test]
    fn execute_peace_before_accept_errors() {
        let neg = PeaceNegotiation::new(basic_proposal(), 5);
        let result = neg.execute_peace();
        assert!(matches!(result, Err(PeaceError::AlreadyResolved)));
    }

    #[test]
    fn war_weariness_increases_with_duration_and_casualties() {
        let (wa, wb) = war_weariness(1000, 50, 30, 10000);
        // duration_component = 1000/100 = 10
        // wa = 10 + 50*10 = 510
        // wb = 10 + 30*10 = 310
        assert_eq!(wa, 510);
        assert_eq!(wb, 310);
    }

    #[test]
    fn war_weariness_caps_at_max() {
        let (wa, _) = war_weariness(100_000, 5000, 0, 8000);
        // duration_component = 1000, casualties = 50000 → capped at 8000
        assert_eq!(wa, 8000);
    }

    #[test]
    fn reject_then_accept_errors() {
        let mut neg = PeaceNegotiation::new(basic_proposal(), 5);
        neg.reject().expect("reject");
        let result = neg.accept();
        assert!(matches!(result, Err(PeaceError::AlreadyResolved)));
    }

    #[test]
    fn propose_peace_from_non_party_errors() {
        let mut neg = PeaceNegotiation::new(basic_proposal(), 5);
        let result = neg.propose_peace(p(99), p(1), basic_terms(), 300);
        assert!(matches!(result, Err(PeaceError::ActorNotParty(id)) if id == p(99)));
    }

    #[test]
    fn peace_outcome_serialization_roundtrips() {
        let outcome = PeaceOutcome {
            ceasefire_established: true,
            reparations_transferred: 500,
            territories_conceded: 3,
            forces_demobilized: 40,
        };
        let json = serde_json::to_string(&outcome).expect("serialize");
        let decoded: PeaceOutcome = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(outcome, decoded);
    }
}
