# Intent: FR-CIV-DET-002 — Per-tick BLAKE3 hash chain

> Date: 2026-09-19
> FR: FR-CIV-DET-002
> Epic: FR-CIV-DET (Determinism)

## User Intent

Each tick's event bytes feed into a BLAKE3 hash chain that starts from a
fixed `GENESIS` anchor. Identical inputs must yield identical chain links,
and any single-byte tamper of the event bytes must change the hash. The
chain root from a tick sequence must match the root produced by
incrementally advancing the chain link-by-link.

## Acceptance Signal

- `crates/engine/src/hash_chain.rs` exposes `tick_event_bytes`, `tick_hash`,
  and `GENESIS`.
- `crates/engine/tests/fr_fr_civ_det_002.rs` 3 tests pass:
  `identical_inputs_same_chain_link`, `tamper_changes_hash`,
  `chain_root_matches_incremental_advance`.
- `// Covers: FR-CIV-DET-002` on the test module header.
