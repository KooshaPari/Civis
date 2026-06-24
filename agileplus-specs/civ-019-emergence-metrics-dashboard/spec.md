---
spec_id: civ-019
state: ACTIVE
plan_status: IN_PROGRESS
last_audit: 2026-06-10
workstream_branch: feat/emergence-dashboard
---

# Specification: Emergence Metrics Dashboard

**Slug**: civ-019-emergence-metrics-dashboard | **Epic**: E5 | **Date**: 2026-06-10 | **State**: ACTIVE
**Workstream branch**: `feat/emergence-dashboard` (in `E:/civis-wt-emergence-dash`)

## Problem Statement

The wave-1 emergence foundation (af913fb2) added several emergent
behaviours — sentience threshold (genetics), inter-cluster diplomacy,
culture + language drift, psyche + social graph, and insurgency pressure
— but there is no single panel that lets an operator observe them
together. Each metric is computed in its own crate; the JSON-RPC
`sim.snapshot` surface does not yet include an `emergence` block; and
both the Bevy live HUD and the (deprecated) web dashboard read
`sim.snapshot` only via per-stream fields. This spec adds the
**catalogue** of metrics, the **engine-tick** wiring, the **snapshot
field** on the JSON-RPC surface, and the **dashboard + Bevy overlays**
that consume them. The catalogue is intentionally **read-only** — it
never mutates world state, never re-orders ticks, and never affects
replay determinism.

## Target Users

- Operators / agents observing emergent behaviour during long sim runs
- Engine / replay engineers adding new metrics (extension surface)
- Web dashboard authors consuming the snapshot field
- Bevy primary-client HUD authors adding the overlay

## Functional Requirements

- [ ] **FR-CIV-EMERG-001**: The engine SHALL compute, in read-only
  fashion at the end of `phase_diffusion`, the following metrics per
  `(tick, seed, scenario)`: `ClusterEntropy`, `IdeologyHomophilyIndex`,
  `SentienceFraction`, `PsycheStability`, `DiplomacyTensionIndex`.
- [ ] **FR-CIV-EMERG-002**: The metrics SHALL be deterministic
  functions of `(tick, seed, scenario, snapshot)` — no wall-clock, no
  RNG, no allocation in the hot path. The output is identical for two
  runs with the same seed and same scenario.
- [ ] **FR-CIV-EMERG-003**: The metrics SHALL be exposed on
  `sim.snapshot.emergence` (JSON-RPC) and the `emergence_metrics.v1`
  replay-bus event SHALL be emitted once per N ticks (N is a scenario
  field, default 100).
- [ ] **FR-CIV-EMERG-004**: The web dashboard SHALL provide an
  `EmergencePanel` component with a per-metric sparkline (last 120
  ticks) and a threshold-color chip; the panel SHALL read from
  `sim.snapshot.emergence` only — no private channels.
- [ ] **FR-CIV-EMERG-005**: The Bevy primary client SHALL provide a
  `live_emergence_overlay` HUD toggle (E) that shows the same five
  metrics as a glassmorphism chip group; the chip colors SHALL mirror
  the dashboard's threshold colors.

## Non-Functional Requirements

- Determinism: every metric is a pure function of `(tick, seed,
  scenario)`; no wall-clock in the hot path; no RNG in the metric
  computation itself (sampling from existing sim state only)
- Test surface: `crates/engine/src/emergence_metrics.rs`,
  `crates/server/src/jsonrpc.rs`, `web/dashboard/emergence_panel.tsx`,
  `clients/bevy-ref/src/live_emergence_overlay.rs`
- Performance: the metrics block adds ≤ 200 µs P99 to the diffusion
  phase at 1,000 agents (NFR-CIV-PERF-003 budget headroom)
- Replay: emitting `emergence_metrics.v1` does NOT change the replay
  hash chain (replay-compatibility verified by
  `replay_emergence_metrics_emit_does_not_change_hash_chain`)

## Constraints and Dependencies

- Depends on FR-CIV-AGENTS-001 (wardrobe/tools ticks) for the
  `ClusterEntropy` + `IdeologyHomophilyIndex` inputs
- Depends on FR-CIV-GENETICS-001 (mutation deterministic) for
  `SentienceFraction`
- Depends on FR-CIV-DIFFUSION-001 (Bass/Rogers S-curve) for
  `IdeologyHomophilyIndex`
- Depends on FR-CIV-CULT-001 (culture diffusion) for
  `IdeologyHomophilyIndex` + `DiplomacyTensionIndex`
- Depends on FR-PROTO-002 (JSON-RPC) for the snapshot field
- Refines FR-METRICS-001/002/003 (the `Metrics` struct is extended
  with the emergence block; existing 4×f64 contract is preserved)

## Acceptance Criteria

- [ ] `cargo test -p civ-engine emergence_metrics` exits 0; the five
  per-metric determinism tests pass with two different seeds and the
  same input snapshot
- [ ] `sim.snapshot.emergence` JSON-RPC payload includes all five
  metrics; `ws_jsonrpc_sim_snapshot_returns_emergence_block` passes
- [ ] `cargo test -p civ-engine replay_emergence_metrics_emit_does_not_
  change_hash_chain` exits 0
- [ ] `node --test web/dashboard` covers `emergence_panel` smoke
- [ ] `cargo test --manifest-path clients/bevy-ref/rust/Cargo.toml
  live_emergence_overlay` exits 0

## Implementation Notes

- The metrics block lives in `crates/engine/src/emergence_metrics.rs`
  and is wired at the end of `phase_diffusion`. It is gated by the
  `feature = "emergence-metrics"` (or similar) so the binary size
  cost is opt-in for non-research builds.
- The threshold-color chip uses the same palette as the
  colorblind-safe faction palette (NFR-CIV-ACC-001).
- The Bevy overlay is **on by default** during live attach; the toggle
  key (E) is documented in `docs/guides/keybindings.md`.
- The replay-bus event is emitted at scenario-defined cadence
  (`emergence_metrics_cadence`, default 100) — emitting it every tick
  is rejected by determinism reviewer as "expensive on the bus".

## Status

| Story | Status |
|-------|--------|
| E5.1 Engine metrics block + tick wiring | Planned |
| E5.2 JSON-RPC `sim.snapshot.emergence` + replay-bus | Planned |
| E5.3 Web `EmergencePanel` | Planned |
| E5.4 Bevy `live_emergence_overlay` | Planned |
