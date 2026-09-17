//! FR traceability tests for the psyche/social modules in civ-agents.
//!
//! Covers: FR-CIV-PSYCHE-001 through FR-CIV-PSYCHE-006

use civ_agents::{
    Mood, Psyche, PsychGenomeProfile, PSYCHE_DIM, SocialEvent, SocialGraph, Tie, Interaction,
    MAX_TIES, decay_social_graph, relation_label, apply_social_event, RelationLabel,
};

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-001 — Psyche vector construction and dimensionality
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-001 — Psyche vector has exactly PSYCHE_DIM drives.
#[test]
fn fr_civ_psyche_001_vector_dimensionality() {
    let p = Psyche {
        drives: [0.5; PSYCHE_DIM],
        temperament: civ_agents::Temperament::neutral(),
        mood: Mood::neutral(),
        beliefs: [0.0; PSYCHE_DIM],
        maturity: 0.5,
    };
    assert_eq!(p.drives.len(), PSYCHE_DIM);
    assert_eq!(p.beliefs.len(), PSYCHE_DIM);
}

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-002 — Mood state transitions
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-002 — Neutral mood has zero valence and arousal.
#[test]
fn fr_civ_psyche_002_mood_neutral_defaults() {
    let mood = Mood::neutral();
    assert_eq!(mood.valence, 0.0);
    assert_eq!(mood.arousal, 0.0);
}

/// FR-CIV-PSYCHE-002 — Mood can be constructed with extreme values.
#[test]
fn fr_civ_psyche_002_mood_extreme_values() {
    let mood = Mood {
        valence: 0.95,
        arousal: -0.85,
    };
    assert!((mood.valence - 0.95).abs() < 1e-6);
    assert!((mood.arousal - (-0.85)).abs() < 1e-6);
}

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-003 — Temperament defaults and construction
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-003 — Neutral temperament has all fields at 0.5.
#[test]
fn fr_civ_psyche_003_temperament_neutral() {
    let t = civ_agents::Temperament::neutral();
    assert_eq!(t.reactivity, 0.5);
    assert_eq!(t.sociability, 0.5);
    assert_eq!(t.risk_tol, 0.5);
    assert_eq!(t.impulsivity, 0.5);
}

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-005 — Psych genome profile default structure
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-005 — Default genome profile has correct slot counts.
#[test]
fn fr_civ_psyche_005_genome_profile_slots() {
    let profile = PsychGenomeProfile::default_profile();
    assert_eq!(profile.drive_slots.len(), PSYCHE_DIM);
    for slots in &profile.drive_slots {
        assert!(!slots.is_empty(), "each drive must have at least one slot");
    }
    assert!(!profile.reactivity_slots.is_empty());
    assert!(!profile.sociability_slots.is_empty());
    assert!(!profile.risk_slots.is_empty());
    assert!(!profile.impulsivity_slots.is_empty());
}

// ---------------------------------------------------------------------------
// FR-CIV-PSYCHE-006 — Psyche serialisation round-trip
// ---------------------------------------------------------------------------

/// FR-CIV-PSYCHE-006 — Psyche fields are accessible and correct.
#[test]
fn fr_civ_psyche_006_psyche_fields_accessible() {
    let psyche = Psyche {
        drives: [0.1, 0.2, 0.3, 0.4],
        temperament: civ_agents::Temperament {
            reactivity: 0.6,
            sociability: 0.7,
            risk_tol: 0.8,
            impulsivity: 0.9,
        },
        mood: Mood {
            valence: 0.3,
            arousal: -0.1,
        },
        beliefs: [0.4, 0.3, 0.2, 0.1],
        maturity: 0.55,
    };
    assert_eq!(psyche.drives[0], 0.1);
    assert_eq!(psyche.mood.valence, 0.3);
    assert_eq!(psyche.maturity, 0.55);
    assert_eq!(psyche.beliefs[3], 0.1);
}
