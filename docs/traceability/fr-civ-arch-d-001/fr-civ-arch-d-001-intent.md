# Intent: FR-CIV-ARCH-D-001 — Facade emergence determinism

> Date: 2026-09-20
> FR: FR-CIV-ARCH-D-001
> Epic: FR-CIV-ARCH-D

## What This FR Captures

Determinism guarantee for `facade_for_emergence` — calling the
emergent-architecture facade function twice with identical inputs
(`EmergentStyleKey`, `DemandSignals`, tile sets) must produce the
exact same output. Tested at `crates/build/src/tiers.rs:602` and is
a precondition for replay integrity in the build subsystem.

## User Intent

Architecture facades feed the visual layer; any nondeterminism
would corrupt snapshot replays and break visual stability across
clients. This test pins the contract.

## Acceptance Signal

- `cargo test -p build fr_arch_d001_facade_for_emergence_is_deterministic`
  passes.
- The function is also idempotent (same input → same output).

## Implementing Code

- `crates/build/src/tiers.rs:602` — test
  `fr_arch_d001_facade_for_emergence_is_deterministic`
- Production code: `facade_for_emergence` in the same file (under test).

## Test Coverage

- `crates/build/src/tiers.rs:602` (dedicated test)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-arch-d-001-intent.md` |
| Implementing crate | `crates/build/src/` |