# Intent: FR-CIV-BEVY-034 -- Faction frame → diplomacy panel mapping

> Date: 2026-09-19
> FR: FR-CIV-BEVY-034
> Epic: FR-CIV-BEVY

## User Intent

The product owner requires Faction frame → diplomacy panel mapping as part of the FR-CIV-BEVY epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement ensures that Faction frame → diplomacy panel mapping is properly specified, implemented, and testable within the simulation engine.

`clients/bevy-ref/src/diplomacy_ui.rs::diplomacy_state_from_faction_frame`
converts a `FactionStateFrame` from the WS F3D0 wire payload into a
`DiplomacyState` containing the sorted panel rows (`DipFaction` list)
plus a symmetric NxN `relations` matrix initialised to neutral by
`neutral_relations_matrix(n)`. The function is deterministic, sorted
by faction id, and is the single entry point the Bevy reference client
uses to populate the diplomacy panel each tick.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with a
WebSocket / HTTP server. FR-CIV-BEVY-034 contributes to the diplomacy
UI contract by guaranteeing that the rendered panel rows and the
relation matrix stay derived from the same source-of-truth frame — no
client-side drift between Bevy / Godot / Unreal clients.

## Acceptance Signal

### Definition of Done

- [x] Implementation in `clients/bevy-ref/` compiles and passes all checks
- [x] Unit tests pass for the new functionality
- [x] Integration with the simulation tick system works correctly
- [x] No regressions in existing FRs
- [x] `diplomacy_state_from_faction_frame_maps_entries` covers sort + matrix init

### How We Know This FR Is Satisfied

1. `cargo test -p bevy-ref diplomacy_state_from_faction_frame` passes
2. Panel rows are sorted by faction id
3. Relation matrix is `N x N` with all-zero off-diagonal before any
   stance events mutate it
4. Output `DiplomacyState` is `Clone + Eq` for snapshot testing

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-bevy-034-intent.md` |
| Source | `clients/bevy-ref/src/diplomacy_ui.rs:625` |
| Test | `clients/bevy-ref/src/diplomacy_ui.rs:625` |

<!-- Covers: FR-CIV-BEVY-034 -->
