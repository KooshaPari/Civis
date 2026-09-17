//! FR traceability tests for the tactics crate.
//!
//! Covers: FR-CIV-TACTICS-025, 051, 065, 076

use civ_tactics::{
    FogOfWar, FormationKind, MoraleState, UnitStance,
    FactionEngagementStats, formation_offsets,
};

/// FR-CIV-TACTICS-025 — FogOfWar can be constructed with grid size.
#[test]
fn fr_civ_tactics_025_fog_of_war_construction() {
    let fog = FogOfWar::new(32, None);
    // Construction succeeds - fog is ready for update()
    let _ = fog;
}

/// FR-CIV-TACTICS-051 — Formation offsets exist for basic formations.
#[test]
fn fr_civ_tactics_051_formation_offsets_line() {
    let line = formation_offsets(FormationKind::Line, 4);
    assert!(!line.is_empty(), "line formation must produce offsets");
    assert_eq!(line.len(), 4, "should have 4 offsets for 4 soldiers");
}

/// FR-CIV-TACTICS-051 — Column formation produces offsets.
#[test]
fn fr_civ_tactics_051_formation_offsets_column() {
    let column = formation_offsets(FormationKind::Column, 3);
    assert_eq!(column.len(), 3, "column should have 3 offsets");
}

/// FR-CIV-TACTICS-065 — Different formations produce different offsets.
#[test]
fn fr_civ_tactics_065_different_formations_differ() {
    let line = formation_offsets(FormationKind::Line, 4);
    let column = formation_offsets(FormationKind::Column, 4);
    assert_ne!(line, column, "line and column should produce different offsets");
}

/// FR-CIV-TACTICS-076 — MoraleState can be constructed for a unit.
#[test]
fn fr_civ_tactics_076_morale_construction() {
    let morale = MoraleState::new(100, 25);
    // New unit should be standing (above rout threshold)
    assert_eq!(morale.stance(), UnitStance::Standing);
}

/// FR-CIV-TACTICS-076 — Morale drops after taking casualties.
#[test]
fn fr_civ_tactics_076_morale_drops_after_casualties() {
    let mut morale = MoraleState::new(100, 25);
    // Take heavy casualties
    morale.apply_casualties(80);
    // Should be routing now
    assert_eq!(morale.stance(), UnitStance::Routing);
}

/// FR-CIV-TACTICS-025 — FactionEngagementStats tracks shooter/target counts.
#[test]
fn fr_civ_tactics_025_faction_engagement_stats() {
    let stats = FactionEngagementStats {
        engagements_as_shooter: 10,
        engagements_as_target: 7,
        voxels_removed: 500,
    };
    assert_eq!(stats.net_pressure(), 3);
}

/// FR-CIV-TACTICS-025 — FactionEngagementStats defaults to zeros.
#[test]
fn fr_civ_tactics_025_faction_engagement_stats_default() {
    let stats = FactionEngagementStats::default();
    assert_eq!(stats.engagements_as_shooter, 0);
    assert_eq!(stats.engagements_as_target, 0);
    assert_eq!(stats.voxels_removed, 0);
}
