# Intent: FR-CIV-DET-006 — Replay log capture and restore

> Date: 2026-09-19
> FR: FR-CIV-DET-006
> Epic: FR-CIV-DET (Determinism)

## User Intent

`ReplayLog` must start empty with `schema_version == 1`. Recording a tick
appends an event to `events` and computes a `running_hash`. The running
hash changes after each recorded tick — never silently drops.

## Acceptance Signal

- `ReplayLog::default().events.is_empty()` and `schema_version == 1`.
- After `log.record_tick(1)`, `events.len() == 1` and
  `running_hash.is_some()`.
- `crates/engine/tests/fr_fr_civ_det_006.rs` 3 tests pass:
  `replay_log_starts_empty`, `record_tick_appears_in_events`,
  `running_hash_changes_after_tick`.
- `// Covers: FR-CIV-DET-006` on the test module header.
