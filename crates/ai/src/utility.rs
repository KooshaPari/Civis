//! FR-AI-001 — Utility scoring function for AI move selection.
//!
//! AI civilizations SHALL select actions using a utility scoring function
//! over available moves. `UtilityScorer` wraps configurable weights and
//! scores every candidate action, returning ranked results.

use serde::{Deserialize, Serialize};

/// Weights controlling how each dimension contributes to the utility score.
/// All weights are non-negative f64; the scorer sums weighted components.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UtilityWeights {
    /// Weight for resource value of a move.
    pub resource_value: f64,
    /// Weight for strategic positioning.
    pub strategic_value: f64,
    /// Weight for safety / risk reduction.
    pub safety_value: f64,
    /// Weight for diplomatic benefit.
    pub diplomatic_value: f64,
}

impl Default for UtilityWeights {
    fn default() -> Self {
        Self {
            resource_value: 1.0,
            strategic_value: 1.0,
            safety_value: 1.0,
            diplomatic_value: 0.5,
        }
    }
}

/// A candidate move an AI civilization can make.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Move {
    /// Unique move identifier.
    pub id: String,
    /// Resource value in [0.0, 1.0].
    pub resource_value: f64,
    /// Strategic value in [0.0, 1.0].
    pub strategic_value: f64,
    /// Safety value in [0.0, 1.0] (1.0 = perfectly safe).
    pub safety_value: f64,
    /// Diplomatic value in [0.0, 1.0].
    pub diplomatic_value: f64,
}

/// Scored result for a single move.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScoredMove {
    /// The move id.
    pub move_id: String,
    /// Total utility score.
    pub score: f64,
}

/// Utility scorer that evaluates moves using configurable weights.
#[derive(Debug, Clone)]
pub struct UtilityScorer {
    /// The weights applied to each dimension.
    pub weights: UtilityWeights,
}

impl UtilityScorer {
    /// Create a new scorer with the given weights.
    #[must_use]
    pub fn new(weights: UtilityWeights) -> Self {
        Self { weights }
    }

    /// Create a scorer with default weights.
    #[must_use]
    pub fn default_scorer() -> Self {
        Self::new(UtilityWeights::default())
    }

    /// Score a single move using the weighted sum of its dimensions.
    #[must_use]
    pub fn score_move(&self, m: &Move) -> ScoredMove {
        let score = self.weights.resource_value * m.resource_value
            + self.weights.strategic_value * m.strategic_value
            + self.weights.safety_value * m.safety_value
            + self.weights.diplomatic_value * m.diplomatic_value;
        ScoredMove {
            move_id: m.id.clone(),
            score,
        }
    }

    /// Score all moves and return them ranked by descending score.
    #[must_use]
    pub fn score_all(&self, moves: &[Move]) -> Vec<ScoredMove> {
        let mut scored: Vec<ScoredMove> = moves.iter().map(|m| self.score_move(m)).collect();
        scored.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        scored
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scorer_returns_all_moves() {
        let scorer = UtilityScorer::default_scorer();
        let moves = vec![
            Move { id: "a".into(), resource_value: 0.5, strategic_value: 0.5, safety_value: 0.5, diplomatic_value: 0.5 },
            Move { id: "b".into(), resource_value: 0.8, strategic_value: 0.2, safety_value: 0.9, diplomatic_value: 0.1 },
        ];
        let scored = scorer.score_all(&moves);
        assert_eq!(scored.len(), 2);
    }

    #[test]
    fn higher_values_produce_higher_scores() {
        let scorer = UtilityScorer::default_scorer();
        let low = Move { id: "low".into(), resource_value: 0.1, strategic_value: 0.1, safety_value: 0.1, diplomatic_value: 0.1 };
        let high = Move { id: "high".into(), resource_value: 0.9, strategic_value: 0.9, safety_value: 0.9, diplomatic_value: 0.9 };
        assert!(scorer.score_move(&high).score > scorer.score_move(&low).score);
    }
}
