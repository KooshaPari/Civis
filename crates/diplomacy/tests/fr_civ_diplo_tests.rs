//! FR traceability tests for the diplomacy emergent module.
//!
//! Covers: FR-CIV-DIPLO-002-SHADOW

use civ_diplomacy::{EmergentStance, RelationDrivers, Territory, PolityId};

// ---------------------------------------------------------------------------
// FR-CIV-DIPLO-002-SHADOW — Shadow diplomacy: emergent stance derivation
// ---------------------------------------------------------------------------

/// FR-CIV-DIPLO-002-SHADOW — Low score produces Rival stance.
#[test]
fn fr_civ_diplo_002_shadow_low_score_is_rival() {
    let stance = EmergentStance::from_score(-0.8, -0.3, 0.5);
    assert_eq!(stance, EmergentStance::Rival);
}

/// FR-CIV-DIPLO-002-SHADOW — High score produces Ally stance.
#[test]
fn fr_civ_diplo_002_shadow_high_score_is_ally() {
    let stance = EmergentStance::from_score(0.8, -0.3, 0.5);
    assert_eq!(stance, EmergentStance::Ally);
}

/// FR-CIV-DIPLO-002-SHADOW — Mid-range score produces Neutral stance.
#[test]
fn fr_civ_diplo_002_shadow_mid_score_is_neutral() {
    let stance = EmergentStance::from_score(0.1, -0.3, 0.5);
    assert_eq!(stance, EmergentStance::Neutral);
}

/// FR-CIV-DIPLO-002-SHADOW — RelationDrivers score is clamped to [-1, 1].
#[test]
fn fr_civ_diplo_002_shadow_score_clamped() {
    let drivers = RelationDrivers {
        shared_enemy: 1.0,
        border_friction: 0.0,
        trade: 1.0,
        culture_similarity: 1.0,
    };
    let score = drivers.score();
    assert!(score >= -1.0 && score <= 1.0, "score {score} out of range");
}

/// FR-CIV-DIPLO-002-SHADOW — Border friction cools relations (negative contribution).
#[test]
fn fr_civ_diplo_002_shadow_border_friction_cools() {
    let no_friction = RelationDrivers {
        shared_enemy: 0.5,
        border_friction: 0.0,
        trade: 0.5,
        culture_similarity: 0.5,
    };
    let with_friction = RelationDrivers {
        shared_enemy: 0.5,
        border_friction: 0.8,
        trade: 0.5,
        culture_similarity: 0.5,
    };
    assert!(
        with_friction.score() < no_friction.score(),
        "friction should lower the score"
    );
}

/// FR-CIV-DIPLO-002-SHADOW — Territory overlap counts shared cells.
#[test]
fn fr_civ_diplo_002_shadow_territory_overlap() {
    let t1 = Territory::new(PolityId::new(1)).with_cells([1, 2, 3, 4]);
    let t2 = Territory::new(PolityId::new(2)).with_cells([3, 4, 5, 6]);
    assert_eq!(t1.overlap(&t2), 2, "should share cells 3 and 4");
}

/// FR-CIV-DIPLO-002-SHADOW — No overlap returns zero.
#[test]
fn fr_civ_diplo_002_shadow_no_overlap() {
    let t1 = Territory::new(PolityId::new(1)).with_cells([1, 2]);
    let t2 = Territory::new(PolityId::new(2)).with_cells([3, 4]);
    assert_eq!(t1.overlap(&t2), 0);
}
