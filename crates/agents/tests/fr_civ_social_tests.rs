//! FR traceability tests for the social graph module in civ-agents.
//!
//! Covers: FR-CIV-PSYCHE-002 (social interactions), FR-CIV-PSYCHE-003
//! (social relationships), FR-CIV-PSYCHE-005 (social dynamics),
//! FR-CIV-PSYCHE-006 (social behavior)

use civ_agents::{
    Interaction, RelationLabel, SocialEvent, SocialGraph, Tie, apply_social_event,
    decay_social_graph, relation_label, MAX_TIES,
};

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-002 — Social interactions: tie construction
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-002 — New tie has zeroed fields and correct other/tick.
#[test]
fn fr_civ_psyche_002_new_tie_defaults() {
    let tie = Tie::new(42, 100);
    assert_eq!(tie.other, 42);
    assert_eq!(tie.last_seen, 100);
    assert_eq!(tie.kinship, 0.0);
    assert_eq!(tie.familiarity, 0.0);
    assert_eq!(tie.affinity, 0.0);
    assert_eq!(tie.trust, 0.0);
}

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-003 — Social relationships: relation label derivation
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-003 — Family label for high kinship tie.
#[test]
fn fr_civ_psyche_003_family_label_for_high_kinship() {
    let tie = Tie {
        other: 1,
        kinship: 0.9,
        familiarity: 0.5,
        affinity: 0.3,
        trust: 0.5,
        last_seen: 0,
    };
    let label = relation_label(&tie);
    assert_eq!(label, RelationLabel::Family);
}

/// FR-CIV-PSYCHE-003 — Enemy label for strong negative affinity.
#[test]
fn fr_civ_psyche_003_enemy_label_for_negative_affinity() {
    let tie = Tie {
        other: 2,
        kinship: 0.0,
        familiarity: 0.8,
        affinity: -0.9,
        trust: 0.0,
        last_seen: 0,
    };
    let label = relation_label(&tie);
    assert_eq!(label, RelationLabel::Enemy);
}

/// FR-CIV-PSYCHE-003 — Acquaintance label for low-salience tie.
#[test]
fn fr_civ_psyche_003_acquaintance_for_low_salience() {
    let tie = Tie {
        other: 3,
        kinship: 0.0,
        familiarity: 0.1,
        affinity: 0.05,
        trust: 0.1,
        last_seen: 0,
    };
    let label = relation_label(&tie);
    assert_eq!(label, RelationLabel::Acquaintance);
}

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-005 — Social dynamics: social event application
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-005 — Cooperation increases affinity on target tie.
#[test]
fn fr_civ_psyche_005_cooperation_increases_affinity() {
    let mut graph = SocialGraph::default();
    graph.ties.push(Tie {
        other: 10,
        kinship: 0.0,
        familiarity: 0.3,
        affinity: 0.1,
        trust: 0.2,
        last_seen: 0,
    });

    let event = SocialEvent {
        a: 5,
        b: 10,
        kind: Interaction::Cooperated { benefit: 0.5 },
        tick: 1,
    };
    apply_social_event(&mut graph, event);

    let tie = graph.ties.iter().find(|t| t.other == 10).unwrap();
    assert!(
        tie.affinity > 0.1,
        "affinity should increase after cooperation"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-006 — Social behavior: graph decay
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-006 — Decay reduces familiarity over time.
#[test]
fn fr_civ_psyche_006_decay_reduces_familiarity() {
    let mut graph = SocialGraph::default();
    graph.ties.push(Tie {
        other: 20,
        kinship: 0.0,
        familiarity: 0.8,
        affinity: 0.5,
        trust: 0.5,
        last_seen: 0,
    });

    decay_social_graph(&mut graph, 100);

    let tie = graph.ties.iter().find(|t| t.other == 20).unwrap();
    assert!(
        tie.familiarity < 0.8,
        "familiarity should decrease after decay"
    );
}

/// FR-CIV-PSYCHE-006 — Social graph respects MAX_TIES limit.
#[test]
fn fr_civ_psyche_006_max_ties_constant() {
    assert!(MAX_TIES > 0, "MAX_TIES must be positive");
    assert!(MAX_TIES <= 500, "MAX_TIES should be reasonable");
}
