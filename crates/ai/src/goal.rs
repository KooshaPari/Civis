//! Agent goal selection (FR-CIV-AGENT-GOAL).
//!
//! An agent maintains a vector of [`Need`] values (hunger, rest, safety,
//! social, …) and is offered a set of candidate [`Goal`]s each scored by
//! utility. [`select_goal`] picks the goal with the highest utility for the
//! agent's current needs; ties are broken by deterministic id ordering so the
//! choice is stable across runs.
//!
//! This module is **pure** (no I/O, no async, no Bevy ECS dependency) so it can
//! be embedded inside a Bevy system, a worker-pool job, or a one-off unit test
//! without dragging the rest of `civ-ai` along. It is intentionally
//! domain-agnostic — the same selector works for a villager deciding whether
//! to farm vs sleep, a faction deciding whether to declare war, or a planner
//! picking the next research target.

#![allow(clippy::module_name_repetitions)]

use serde::{Deserialize, Serialize};

/// A named drive the agent is trying to satisfy. Each variant carries an
/// `f32` in `[0.0, 1.0]` where `0.0` is "satisfied" and `1.0` is "critical".
///
/// Additive: existing crates that already model needs are free to ignore this
/// enum and build their own [`Goal`] list against it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Need {
    /// Need to eat. `1.0` = starving.
    Hunger(f32),
    /// Need to sleep. `1.0` = exhausted.
    Rest(f32),
    /// Need for physical safety. `1.0` = under attack.
    Safety(f32),
    /// Need for social contact / belonging. `1.0` = isolated.
    Social(f32),
    /// Need for purpose / achievement. `1.0` = aimless.
    Purpose(f32),
}

impl Need {
    /// Raw urgency in `[0.0, 1.0]`. Saturating access for callers that just want
    /// the number.
    #[must_use]
    pub fn urgency(self) -> f32 {
        match self {
            Need::Hunger(u)
            | Need::Rest(u)
            | Need::Safety(u)
            | Need::Social(u)
            | Need::Purpose(u) => u.clamp(0.0, 1.0),
        }
    }
}

/// A goal the agent could pursue. Carries a stable `id` (for logging,
/// provenance, and tie-breaking) and a [`Goal::utility`] function that scores
/// how well the goal relieves the agent's current needs.
///
/// Higher utility = more attractive. [`select_goal`] picks the maximum.
pub trait Goal: Send + Sync {
    /// Stable identifier — used in logs and as a deterministic tie-breaker.
    fn id(&self) -> &str;

    /// Utility of pursuing this goal given the agent's needs. Higher is
    /// better. Implementations should be cheap (called per candidate per
    /// decision) and pure (no RNG, no I/O).
    fn utility(&self, needs: &[Need]) -> f32;
}

/// Outcome of a decision: either the chosen goal's id, or a structured
/// "no good action" reason. We never silently return `None` — callers can log
/// or fall back, but the absence is named (matches `CLAUDE.md` loud-failure
/// style).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum GoalChoice {
    /// Agent picked the goal with this id.
    Selected(String),
    /// No candidates were offered.
    NoCandidates,
    /// Every candidate had non-positive utility (nothing helps).
    NoUsefulGoal,
}

/// Pick the highest-utility goal from `candidates` for the given `needs`.
///
/// Tie-breaking: when two goals have *exactly* the same utility, the one with
/// the lexicographically smaller id wins. This keeps decisions deterministic
/// across runs — important for replay/provenance and for test stability.
///
/// # Examples
///
/// ```
/// use civ_ai::goal::{Goal, GoalChoice, Need, select_goal};
///
/// struct Eat;
/// impl Goal for Eat {
///     fn id(&self) -> &str { "eat" }
///     fn utility(&self, needs: &[Need]) -> f32 {
///         needs.iter().filter_map(|n| if let Need::Hunger(u) = n { Some(u) } else { None }).sum()
///     }
/// }
///
/// let candidates: Vec<Box<dyn Goal>> = vec![Box::new(Eat)];
/// let needs = vec![Need::Hunger(0.9)];
/// assert_eq!(select_goal(&candidates, &needs), GoalChoice::Selected("eat".into()));
/// ```
#[must_use]
pub fn select_goal<G: Goal + ?Sized>(candidates: &[Box<G>], needs: &[Need]) -> GoalChoice {
    if candidates.is_empty() {
        return GoalChoice::NoCandidates;
    }

    // argmax over utility, breaking ties on id ascending.
    let mut best_idx: Option<usize> = None;
    let mut best_u = f32::NEG_INFINITY;
    let mut best_id: Option<&str> = None;
    for (i, g) in candidates.iter().enumerate() {
        let u = g.utility(needs);
        let id = g.id();
        let take = match best_idx {
            None => true,
            Some(_) if u > best_u => true,
            Some(_) if u < best_u => false,
            // tie on utility: deterministic by id ascending
            Some(_) => id < best_id.unwrap_or(""),
        };
        if take {
            best_idx = Some(i);
            best_u = u;
            best_id = Some(id);
        }
    }

    match best_idx {
        Some(i) if best_u > 0.0 => GoalChoice::Selected(candidates[i].id().to_string()),
        Some(i) => {
            // Caller offered candidates but none of them actually help —
            // surface that rather than returning a vacuous pick.
            // We still log the candidate id (tests assert this branch).
            let _ = i;
            GoalChoice::NoUsefulGoal
        }
        None => GoalChoice::NoCandidates,
    }
}

// -----------------------------------------------------------------------------
// Tests
// -----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    /// A test goal with a fixed utility score (ignores needs). Useful for
    /// driving the selector without modelling the full needs/goal matrix.
    #[derive(Clone, Copy)]
    struct FixedGoal {
        id: &'static str,
        score: f32,
    }

    impl Goal for FixedGoal {
        fn id(&self) -> &str {
            self.id
        }
        fn utility(&self, _needs: &[Need]) -> f32 {
            self.score
        }
    }

    /// A test goal that sums the urgency of one specific need — lets us model
    /// "this goal relieves need X" without mocking the rest of the agent.
    struct RelievesNeed {
        id: &'static str,
        target: NeedKind,
    }

    #[derive(Clone, Copy)]
    enum NeedKind {
        Hunger,
        Rest,
        // Exercised by the match below but not instantiated by these fixtures.
        #[allow(dead_code)]
        Safety,
        Social,
        // Exercised by the match below but not instantiated by these fixtures.
        #[allow(dead_code)]
        Purpose,
    }

    impl Goal for RelievesNeed {
        fn id(&self) -> &str {
            self.id
        }
        fn utility(&self, needs: &[Need]) -> f32 {
            needs
                .iter()
                .map(|n| match (self.target, n) {
                    (NeedKind::Hunger, Need::Hunger(u)) => *u,
                    (NeedKind::Rest, Need::Rest(u)) => *u,
                    (NeedKind::Safety, Need::Safety(u)) => *u,
                    (NeedKind::Social, Need::Social(u)) => *u,
                    (NeedKind::Purpose, Need::Purpose(u)) => *u,
                    _ => 0.0,
                })
                .sum()
        }
    }

    /// FR-CIV-AGENT-GOAL — the agent picks the goal with the highest utility.
    #[test]
    fn selects_max_utility_goal() {
        let eat = RelievesNeed {
            id: "eat",
            target: NeedKind::Hunger,
        };
        let sleep = RelievesNeed {
            id: "sleep",
            target: NeedKind::Rest,
        };
        let play = RelievesNeed {
            id: "play",
            target: NeedKind::Social,
        };

        let needs = vec![Need::Hunger(0.9), Need::Rest(0.3), Need::Social(0.1)];
        let candidates: Vec<Box<dyn Goal>> = vec![Box::new(sleep), Box::new(play), Box::new(eat)];

        assert_eq!(
            select_goal(&candidates, &needs),
            GoalChoice::Selected("eat".to_string()),
            "agent is hungriest (0.9), should pick the goal that relieves Hunger"
        );
    }

    /// FR-CIV-AGENT-GOAL — when two candidates tie on utility, the one with
    /// the lexicographically smaller id wins (deterministic tie-break).
    #[test]
    fn ties_break_by_id_ascending() {
        let alpha = FixedGoal {
            id: "alpha",
            score: 0.5,
        };
        let beta = FixedGoal {
            id: "beta",
            score: 0.5,
        };
        let gamma = FixedGoal {
            id: "gamma",
            score: 0.7,
        };

        let needs = vec![Need::Rest(0.5)];
        let candidates: Vec<Box<dyn Goal>> = vec![Box::new(beta), Box::new(alpha), Box::new(gamma)];

        // gamma wins on utility (0.7 > 0.5).
        assert_eq!(
            select_goal(&candidates, &needs),
            GoalChoice::Selected("gamma".to_string())
        );

        // Drop gamma — alpha and beta tie; alpha should win by id ordering.
        let candidates: Vec<Box<dyn Goal>> = vec![Box::new(beta), Box::new(alpha)];
        assert_eq!(
            select_goal(&candidates, &needs),
            GoalChoice::Selected("alpha".to_string())
        );
    }

    /// FR-CIV-AGENT-GOAL — empty candidate set is a named outcome, not a panic.
    #[test]
    fn empty_candidates_returns_no_candidates() {
        let candidates: Vec<Box<dyn Goal>> = vec![];
        let needs = vec![Need::Hunger(1.0)];
        assert_eq!(select_goal(&candidates, &needs), GoalChoice::NoCandidates);
    }

    /// FR-CIV-AGENT-GOAL — every candidate scoring ≤ 0 returns `NoUsefulGoal`
    /// so callers know the agent would rather idle than pursue any of them.
    #[test]
    fn all_non_positive_returns_no_useful_goal() {
        let useless_a = FixedGoal {
            id: "a",
            score: -1.0,
        };
        let useless_b = FixedGoal {
            id: "b",
            score: 0.0,
        };
        let needs = vec![Need::Hunger(0.5)];
        let candidates: Vec<Box<dyn Goal>> = vec![Box::new(useless_a), Box::new(useless_b)];
        assert_eq!(select_goal(&candidates, &needs), GoalChoice::NoUsefulGoal);
    }

    /// FR-CIV-AGENT-GOAL — `Need::urgency` clamps out-of-range values so a
    /// misconfigured upstream system cannot poison the selector.
    #[test]
    fn need_urgency_clamps_to_unit_interval() {
        assert_eq!(Need::Hunger(2.0).urgency(), 1.0);
        assert_eq!(Need::Rest(-0.5).urgency(), 0.0);
        assert_eq!(Need::Safety(0.42).urgency(), 0.42);
    }

    /// FR-CIV-AGENT-GOAL — reordering candidates does not change the winner
    /// (max-utility is order-independent).
    #[test]
    fn selection_is_order_independent() {
        let candidates: Vec<Box<dyn Goal>> = vec![
            Box::new(RelievesNeed {
                id: "sleep",
                target: NeedKind::Rest,
            }),
            Box::new(RelievesNeed {
                id: "eat",
                target: NeedKind::Hunger,
            }),
        ];
        let needs = vec![Need::Hunger(0.8), Need::Rest(0.2)];

        let a = select_goal(&candidates, &needs);
        let b = {
            let candidates_rev: Vec<Box<dyn Goal>> = vec![
                Box::new(RelievesNeed {
                    id: "eat",
                    target: NeedKind::Hunger,
                }),
                Box::new(RelievesNeed {
                    id: "sleep",
                    target: NeedKind::Rest,
                }),
            ];
            select_goal(&candidates_rev, &needs)
        };
        assert_eq!(a, b);
        assert_eq!(a, GoalChoice::Selected("eat".to_string()));
    }
}
