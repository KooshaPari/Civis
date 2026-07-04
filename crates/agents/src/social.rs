//! Emergent social graph for civilian agents.
//!
//! Relationships are sparse, directed, and updated only through interaction
//! events. No authored friend/enemy taxonomy is stored in the graph itself.

use serde::{Deserialize, Serialize};

use civ_needs::{LifecycleParams, Needs};

use crate::Civilian;

/// Maximum number of ties retained per agent.
pub const MAX_TIES: usize = 150;

/// Directed interaction used to mutate a social graph edge.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum Interaction {
    /// Cooperative contact with a positive benefit.
    Cooperated {
        /// Positive utility transferred by the interaction.
        benefit: f32,
    },
    /// Contested contact with pressure.
    Competed {
        /// Competitive pressure applied by the interaction.
        pressure: f32,
    },
    /// Betrayal, theft, or attack.
    Defected {
        /// Harm inflicted by the interaction.
        harm: f32,
    },
    /// Mere co-location.
    Coexisted,
    /// Kinship link established at birth.
    Kin,
}

/// A directed social event between two agents.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SocialEvent {
    /// Source agent.
    pub a: u64,
    /// Target agent.
    pub b: u64,
    /// Event kind.
    pub kind: Interaction,
    /// Simulation tick.
    pub tick: u32,
}

/// Emergent relation label derived from a tie at query time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationLabel {
    /// Positive kin link.
    Family,
    /// Strong positive relation.
    CloseFriend,
    /// Strong positive relation with further familiarity.
    Partner,
    /// Mild or mixed relation.
    Acquaintance,
    /// Negative relation.
    Rival,
    /// Severe hostility.
    Enemy,
}

/// One directed tie in the social graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Tie {
    /// Other agent.
    pub other: u64,
    /// Genetic relatedness.
    pub kinship: f32,
    /// Familiarity.
    pub familiarity: f32,
    /// Like/dislike signal.
    pub affinity: f32,
    /// Reliability belief.
    pub trust: f32,
    /// Last interaction tick.
    pub last_seen: u32,
}

impl Tie {
    /// Zeroed tie placeholder.
    #[must_use]
    pub fn new(other: u64, tick: u32) -> Self {
        Self {
            other,
            kinship: 0.0,
            familiarity: 0.0,
            affinity: 0.0,
            trust: 0.0,
            last_seen: tick,
        }
    }
}

/// Per-agent social graph.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct SocialGraph {
    /// Sparse directed ties sorted by `other`.
    pub ties: Vec<Tie>,
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn clamp11(value: f32) -> f32 {
    value.clamp(-1.0, 1.0)
}

fn salience(tie: &Tie) -> f32 {
    tie.familiarity + tie.affinity.abs() + tie.kinship
}

/// Derive a relation label from the current tie state.
#[must_use]
pub fn relation_label(tie: &Tie) -> RelationLabel {
    if tie.kinship >= 0.5 && tie.affinity >= 0.0 {
        RelationLabel::Family
    } else if tie.affinity <= -0.85 {
        RelationLabel::Enemy
    } else if tie.affinity < 0.0 && tie.trust < 0.0 {
        RelationLabel::Rival
    } else if tie.affinity >= 0.75 && tie.trust >= 0.35 && tie.familiarity >= 0.5 {
        RelationLabel::Partner
    } else if tie.affinity >= 0.55 && tie.trust >= 0.1 {
        RelationLabel::CloseFriend
    } else {
        RelationLabel::Acquaintance
    }
}

fn touch_tie(graph: &mut SocialGraph, other: u64, tick: u32) -> &mut Tie {
    match graph.ties.binary_search_by_key(&other, |tie| tie.other) {
        Ok(idx) => &mut graph.ties[idx],
        Err(idx) => {
            graph.ties.insert(idx, Tie::new(other, tick));
            &mut graph.ties[idx]
        }
    }
}

fn evict_weakest(graph: &mut SocialGraph) {
    if graph.ties.len() <= MAX_TIES {
        return;
    }
    let mut weakest_idx = 0usize;
    let mut weakest_score = salience(&graph.ties[0]);
    for (idx, tie) in graph.ties.iter().enumerate().skip(1) {
        let score = salience(tie);
        if score < weakest_score {
            weakest_idx = idx;
            weakest_score = score;
        }
    }
    graph.ties.remove(weakest_idx);
}

/// Apply one social event from `event.a`'s perspective to `graph`.
pub fn apply_social_event(graph: &mut SocialGraph, event: SocialEvent) {
    let tie = touch_tie(graph, event.b, event.tick);
    match event.kind {
        Interaction::Cooperated { benefit } => {
            tie.affinity = clamp11(tie.affinity + 0.20 * benefit);
            tie.trust = clamp11(tie.trust + 0.18 * benefit);
            tie.familiarity = clamp01(tie.familiarity + 0.12 * benefit);
        }
        Interaction::Competed { pressure } => {
            tie.affinity = clamp11(tie.affinity - 0.18 * pressure);
            tie.trust = clamp11(tie.trust - 0.06 * pressure);
            tie.familiarity = clamp01(tie.familiarity + 0.05 * pressure);
        }
        Interaction::Defected { harm } => {
            tie.affinity = clamp11(tie.affinity - 0.30 * harm);
            tie.trust = clamp11(tie.trust - 0.35 * harm);
            tie.familiarity = clamp01(tie.familiarity + 0.08 * harm);
        }
        Interaction::Coexisted => {
            tie.familiarity = clamp01(tie.familiarity + 0.04);
            tie.affinity = clamp11(tie.affinity + 0.01);
        }
        Interaction::Kin => {
            tie.kinship = 1.0;
            tie.familiarity = clamp01(tie.familiarity + 0.2);
            tie.affinity = clamp11(tie.affinity + 0.1);
        }
    }
    tie.last_seen = event.tick;
    evict_weakest(graph);
}

/// Decay ties that have not been seen for `current_tick`.
pub fn decay_social_graph(graph: &mut SocialGraph, current_tick: u32) {
    if graph.ties.is_empty() {
        return;
    }
    for tie in &mut graph.ties {
        let gap = current_tick.saturating_sub(tie.last_seen);
        if gap == 0 {
            continue;
        }
        let gap = gap.min(i32::MAX as u32) as i32;
        tie.familiarity = clamp01(tie.familiarity * 0.98_f32.powi(gap));
        if tie.kinship < 0.5 {
            tie.affinity *= 0.995_f32.powi(gap);
        }
        tie.trust *= 0.992_f32.powi(gap);
        tie.affinity = clamp11(tie.affinity);
        tie.trust = clamp11(tie.trust);
    }
    graph.ties.sort_by_key(|tie| tie.other);
}

/// Determine whether two co-located partnered agents should reproduce.
#[must_use]
pub fn should_reproduce(
    a: &Civilian,
    b: &Civilian,
    graph_a: &SocialGraph,
    graph_b: &SocialGraph,
    needs_a: &Needs,
    needs_b: &Needs,
    params: &LifecycleParams,
) -> bool {
    if a.id == b.id {
        return false;
    }

    let relation_a = graph_a
        .ties
        .iter()
        .find(|tie| tie.other == b.id)
        .map(relation_label);
    let relation_b = graph_b
        .ties
        .iter()
        .find(|tie| tie.other == a.id)
        .map(relation_label);

    matches!(relation_a, Some(RelationLabel::Partner))
        && matches!(relation_b, Some(RelationLabel::Partner))
        && needs_a.food > params.fertility_food_threshold
        && needs_b.food > params.fertility_food_threshold
        && needs_a.safety > params.fertility_safety_threshold
        && needs_b.safety > params.fertility_safety_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cooperation_and_defection_mutate_ties() {
        let mut graph = SocialGraph::default();
        apply_social_event(
            &mut graph,
            SocialEvent {
                a: 1,
                b: 2,
                kind: Interaction::Cooperated { benefit: 1.0 },
                tick: 1,
            },
        );
        let after_coop = graph.ties[0].clone();
        assert!(after_coop.affinity > 0.0);
        assert!(after_coop.trust > 0.0);

        apply_social_event(
            &mut graph,
            SocialEvent {
                a: 1,
                b: 2,
                kind: Interaction::Defected { harm: 1.0 },
                tick: 2,
            },
        );
        let tie = &graph.ties[0];
        assert!(tie.trust < after_coop.trust);
    }

    #[test]
    fn labels_are_derived_not_stored() {
        let family = Tie {
            other: 2,
            kinship: 1.0,
            familiarity: 0.2,
            affinity: 0.4,
            trust: 0.1,
            last_seen: 0,
        };
        let rival = Tie {
            affinity: -0.6,
            trust: -0.5,
            ..family.clone()
        };
        assert_eq!(relation_label(&family), RelationLabel::Family);
        assert_eq!(relation_label(&rival), RelationLabel::Rival);
    }

    #[test]
    fn asymmetry_is_preserved() {
        let mut a = SocialGraph::default();
        let mut b = SocialGraph::default();
        apply_social_event(
            &mut a,
            SocialEvent {
                a: 1,
                b: 2,
                kind: Interaction::Cooperated { benefit: 1.0 },
                tick: 1,
            },
        );
        apply_social_event(
            &mut b,
            SocialEvent {
                a: 2,
                b: 1,
                kind: Interaction::Defected { harm: 1.0 },
                tick: 1,
            },
        );
        assert!(a.ties[0].affinity > b.ties[0].affinity);
    }

    #[test]
    fn decay_reduces_old_ties_and_keeps_kinship() {
        let mut graph = SocialGraph {
            ties: vec![
                Tie {
                    other: 2,
                    kinship: 1.0,
                    familiarity: 1.0,
                    affinity: 0.8,
                    trust: 0.8,
                    last_seen: 1,
                },
                Tie {
                    other: 3,
                    kinship: 0.0,
                    familiarity: 1.0,
                    affinity: 0.8,
                    trust: 0.8,
                    last_seen: 1,
                },
            ],
        };
        decay_social_graph(&mut graph, 20);
        assert!(graph.ties[0].familiarity < 1.0);
        assert!(graph.ties[1].familiarity < 1.0);
        assert!(graph.ties[0].affinity > graph.ties[1].affinity);
    }

    #[test]
    fn partnered_couples_with_enough_food_and_safety_can_reproduce() {
        let params = LifecycleParams::default();
        let a = Civilian {
            id: 1,
            alignment: Default::default(),
            age: 24,
        };
        let b = Civilian {
            id: 2,
            alignment: Default::default(),
            age: 25,
        };
        let mut graph_a = SocialGraph::default();
        let mut graph_b = SocialGraph::default();
        apply_social_event(
            &mut graph_a,
            SocialEvent {
                a: 1,
                b: 2,
                kind: Interaction::Cooperated { benefit: 4.3 },
                tick: 1,
            },
        );
        apply_social_event(
            &mut graph_b,
            SocialEvent {
                a: 2,
                b: 1,
                kind: Interaction::Cooperated { benefit: 4.3 },
                tick: 1,
            },
        );
        let needs = Needs {
            food: 0.6,
            water: 0.5,
            rest: 0.5,
            safety: 0.5,
            social: 0.4,
            health: 0.5,
        };
        assert!(should_reproduce(
            &a,
            &b,
            &graph_a,
            &graph_b,
            &needs,
            &needs,
            &params
        ));
    }
}
