//! CIV-007 Emergent Diplomacy — BDD behaviour tests.
//!
//! These integration tests assert the three core feedback loops from the
//! diplomacy drift equation (CIV-007 §1.3):
//!
//! 1. War → grievance → negative score → War (self-reinforcing hostility).
//! 2. Scarcity → resource competition → rivalry (zero-sum pressure).
//! 3. Exhaustion → grievance decay → engagement drops → score recovers
//!    toward neutral (attrition cooling).

use civ_agents::diplomacy::{DiplomacyMatrix, DiplomacySignal, GriefAccumulator, RelationKind};
use civ_agents::ClusterId;

/// CIV-007 §1.3 — sustained combat grievance pushes the pairwise relation
/// score deep into negative territory, crossing the War threshold.
#[test]
fn war_drives_relation_score_negative() {
    let mut matrix = DiplomacyMatrix::new();
    let a = ClusterId(0);
    let b = ClusterId(1);

    let mut last_score = 0.0_f32;
    for _ in 0..30 {
        let outcome = matrix.apply_signal(
            a,
            b,
            DiplomacySignal {
                combat_grievance: 1.0,
                ..Default::default()
            },
        );
        last_score = outcome.score;
    }

    assert!(
        last_score < -0.60,
        "30 ticks of grievance should push score into War territory, got {last_score}"
    );
    assert_eq!(
        matrix.relation(a, b),
        RelationKind::War,
        "score {last_score} should map to War"
    );
}

/// CIV-007 §1.3 — when both factions are energy-scarce, the
/// `scarcity_pressure` term becomes positive and sharpens competition.
/// Combined with high resource overlap, the pair drifts into Rivalry.
#[test]
fn scarcity_raises_rivalry() {
    let mut matrix = DiplomacyMatrix::new();
    let a = ClusterId(10);
    let b = ClusterId(20);

    for _ in 0..20 {
        matrix.apply_signal(
            a,
            b,
            DiplomacySignal {
                resource_competition: 1.0,
                scarcity_pressure: 1.0,
                proximity: 0.8,
                ..Default::default()
            },
        );
    }

    let record = matrix.record(a, b).expect("record present");
    assert!(
        record.score <= -0.20,
        "scarcity + competition should push into Rivalry or War, got {}",
        record.score
    );
    let kind = matrix.relation(a, b);
    assert!(
        matches!(kind, RelationKind::Rivalry | RelationKind::War),
        "expected Rivalry or War, got {kind:?}"
    );
}

/// CIV-007 §2.2 — after a war phase, halted engagements plus exponential
/// grievance decay model "exhaustion".  The absence of new combat signals
/// lets the score drift back toward neutral, lowering effective engagement.
#[test]
fn exhaustion_lowers_engagement() {
    let mut matrix = DiplomacyMatrix::new();
    let mut grief = GriefAccumulator::new();
    let a = ClusterId(3);
    let b = ClusterId(4);

    // Phase 1: sustained war (50 ticks of grievance engagements).
    for _ in 0..50 {
        grief.add_engagement(3, 4);
        grief.tick_decay();
        let g = grief.get(3, 4);
        matrix.apply_signal(
            a,
            b,
            DiplomacySignal {
                combat_grievance: g,
                ..Default::default()
            },
        );
    }
    let war_score = matrix.record(a, b).expect("record").score;
    assert!(war_score < -0.20, "should be at least Rivalry after war phase, got {war_score}");

    // Phase 2: engagements stop — only grievance decay, no new signals.
    // Score should drift upward (toward zero) over 200 ticks.
    for _ in 0..200 {
        grief.tick_decay();
        let g = grief.get(3, 4);
        matrix.apply_signal(
            a,
            b,
            DiplomacySignal {
                combat_grievance: g,
                ..Default::default()
            },
        );
    }
    let recovery_score = matrix.record(a, b).expect("record").score;
    assert!(
        recovery_score > war_score,
        "score should recover after engagements stop: war={war_score}, recovery={recovery_score}"
    );
    // Exhaustion should pull the pair out of War territory entirely.
    assert_ne!(
        matrix.relation(a, b),
        RelationKind::War,
        "exhaustion should lift the pair out of War, got {:?}",
        matrix.relation(a, b)
    );
}
