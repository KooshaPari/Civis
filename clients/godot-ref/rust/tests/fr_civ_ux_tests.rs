//! Traceability tests for the Godot client UX FRs.
//!
//! These FRs were flagged `IMPL-NO-TEST` in the FR audit because the behaviour
//! is covered by generically named inline tests in `src/ux.rs` (for example
//! `spawn_helpers_preserve_payload_and_sequence`), which the audit's
//! FR-id-to-test-name matching cannot see. The tests below carry the FR ids in
//! their names and assert the contracts the module documents.
//!
//! - FR-CIV-UX-000 — the spawn API is exposed via protocol; spawning N
//!   civilians from the UI yields N entity-create events.
//! - FR-CIV-UX-001 — spawn palette exposes the wired kinds.
//! - FR-CIV-UX-004 — drag-release placement and convoy drag for
//!   non-civilian kinds.
//! - FR-CIV-UX-005 — timelapse speed validation and tick advancement.

use civis_godot_rust::ux::{
    convoy_positions, entity_create_from_spawn, spawn_batch_events, spawn_civilian_body,
    spawn_drag_exceeds_threshold, validate_timelapse_speed, SpawnKind, TimelapseView,
    CONVOY_MAX_SPAWNS, DEFAULT_ERA_LENGTH_TICKS, SPAWN_DRAG_MIN_NORM,
};

/// FR-CIV-UX-000 — spawning N civilians emits exactly N entity-create events,
/// with a stable monotonic id sequence.
#[test]
fn fr_civ_ux_000_spawn_n_civilians_yields_n_entity_creates() {
    const N: usize = 12;

    let spawns: Vec<_> = (0..N)
        .map(|i| spawn_civilian_body(i as f32 * 0.01, 0.5, 0))
        .collect();
    let events = spawn_batch_events(&spawns, 1_000);

    assert_eq!(events.len(), N, "N spawn requests must yield exactly N events");
    for (i, ev) in events.iter().enumerate() {
        assert_eq!(
            ev.entity_id,
            1_000 + i as u64,
            "entity ids must be monotonic and dense"
        );
    }

    // A single spawn maps one-to-one and carries the faction through.
    let body = spawn_civilian_body(0.25, 0.75, -3);
    let ev = entity_create_from_spawn(&body, 7);
    assert_eq!(ev.entity_id, 7);
    assert_eq!(ev.faction, u32::MAX - 2, "negative faction wraps consistently");

    // Zero spawns is a valid no-op, not a panic.
    assert!(spawn_batch_events(&[], 0).is_empty());
}

/// FR-CIV-UX-001 — every palette kind is wired and reports a stable label.
#[test]
fn fr_civ_ux_001_spawn_palette_kinds_are_wired_and_labelled() {
    let kinds = [
        (SpawnKind::Civilian, "civilian"),
        (SpawnKind::Vehicle, "vehicle"),
        (SpawnKind::Airport, "airport"),
        (SpawnKind::Port, "port"),
        (SpawnKind::Hangar, "hangar"),
    ];

    for (kind, label) in kinds {
        assert_eq!(kind.label(), label, "palette label must be stable");
        assert!(kind.is_wired(), "{label} must be spawnable via the protocol");
    }

    // Labels are unique, so the UI cannot show two identical entries.
    let mut labels: Vec<&str> = kinds.iter().map(|(k, _)| k.label()).collect();
    labels.sort_unstable();
    labels.dedup();
    assert_eq!(labels.len(), kinds.len(), "palette labels must be unique");
}

/// FR-CIV-UX-004 — drag-release placement requires exceeding the drag
/// threshold, and a long drag spawns a spaced convoy capped at the maximum.
#[test]
fn fr_civ_ux_004_drag_release_and_convoy_placement() {
    // Below the threshold: a click, not a drag, so no placement fires.
    assert!(!spawn_drag_exceeds_threshold((0.0, 0.0), (0.001, 0.001)));
    // Exactly at the threshold and beyond: placement fires.
    assert!(spawn_drag_exceeds_threshold((0.0, 0.0), (0.0, SPAWN_DRAG_MIN_NORM)));
    assert!(spawn_drag_exceeds_threshold((0.0, 0.0), (0.5, 0.5)));

    // A short drag yields the single endpoint (no convoy).
    assert_eq!(
        convoy_positions((0.2, 0.2), (0.21, 0.22)),
        vec![(0.21, 0.22)],
        "a sub-spacing drag places one entity"
    );

    // A long drag yields a spaced convoy that starts and ends at the gesture.
    let convoy = convoy_positions((0.1, 0.1), (0.9, 0.9));
    assert!(convoy.len() > 1, "a long drag must produce a convoy");
    assert_eq!(convoy.first().copied(), Some((0.1, 0.1)));
    assert_eq!(convoy.last().copied(), Some((0.9, 0.9)));
    assert!(
        convoy.len() <= CONVOY_MAX_SPAWNS,
        "convoy must respect the placement cap"
    );

    // Only the non-civilian kinds opt into drag/convoy placement.
    assert!(!SpawnKind::Civilian.uses_drag_place());
    assert!(!SpawnKind::Civilian.uses_convoy_drag());
    for kind in [
        SpawnKind::Vehicle,
        SpawnKind::Airport,
        SpawnKind::Port,
        SpawnKind::Hangar,
    ] {
        assert!(kind.uses_drag_place(), "{kind:?} must use drag-release placement");
        assert!(kind.uses_convoy_drag(), "{kind:?} must support convoy drag");
    }
}

/// FR-CIV-UX-005 — timelapse speed is validated against the allowed set and
/// tick advancement tracks the requested speed.
#[test]
fn fr_civ_ux_005_timelapse_speed_and_advancement() {
    // The allowed speeds are exactly {0, 1, 2, 4, 8}.
    for speed in [0u8, 1, 2, 4, 8] {
        assert!(
            validate_timelapse_speed(speed).is_ok(),
            "speed {speed} must be accepted"
        );
    }
    for speed in [3u8, 5, 6, 7, 9, 255] {
        assert!(
            validate_timelapse_speed(speed).is_err(),
            "speed {speed} must be rejected"
        );
    }

    // Speed 0 freezes the tick; speed N advances by N per frame.
    let mut view = TimelapseView::at_tick(9_999, DEFAULT_ERA_LENGTH_TICKS);
    let frozen = view.tick;
    view.advance_frame(0, DEFAULT_ERA_LENGTH_TICKS);
    assert_eq!(view.tick, frozen, "speed 0 must not advance the tick");

    view.advance_frame(4, DEFAULT_ERA_LENGTH_TICKS);
    assert_eq!(view.tick, frozen + 4, "speed 4 advances four ticks");

    // Advancing to an absolute tick is exact.
    let mut target = TimelapseView::at_tick(0, 1);
    target.advance_to(5, 2, 1);
    assert_eq!(target.tick, 5);
}
