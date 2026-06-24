# Bincode 1.3 ↔ 2.0 "conflict" — verification verdict

**Verdict: FALSE ALARM (no corruption hazard). Fixed by removing a dead dep.**

The 2026-06-16 dependency audit flagged a P0 "critical bincode version conflict"
(civ-voxel + civ-engine on 1.3.3 vs civ-protocol-3d on 2.0.1) as a silent
serialization-corruption hazard. Verified against the source — it is not.

## Evidence (grep of all `.rs` in the three crates)

- `crates/protocol-3d`: **zero** `bincode::` call sites anywhere (src, tests,
  benches). The `bincode = "2.0"` dependency was **unused / dead**.
- `crates/voxel`: bincode 1.3 used only on its OWN data — `stream.rs`
  (chunk serialize/deserialize) and `window/io.rs` (round-trips itself).
- `crates/engine`: bincode 1.3 used only on its OWN data — `save.rs`
  (SavedSimulation) and a `lib.rs` round-trip test.

No data crosses the 1.3↔2.0 boundary (no `Frame3d`/protocol type is bincode-
encoded by protocol-3d and decoded by voxel/engine, or vice versa). voxel and
engine each serialize+deserialize with the same 1.3, so they are internally
consistent. Cargo permits multiple semver-major versions to coexist; with
disjoint usage this is harmless.

## Fix applied

Removed the unused `bincode = { version = "2.0" }` dependency from
`crates/protocol-3d/Cargo.toml`. This eliminates the version skew entirely
(no more 2.0 in the tree) and drops a dead dependency — the lazy-correct fix,
versus a ~2hr "unify encoders" effort that had no real target.

No `thiserror` action taken: semver coexistence is likewise harmless absent a
cross-boundary error type, and was not part of this verification.
