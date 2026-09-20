# Intent: FR-CIV-ARCH-D-003 — Era-index-from-pop-tech determinism

> Date: 2026-09-20
> FR: FR-CIV-ARCH-D-003
> Epic: FR-CIV-ARCH-D

## What This FR Captures

Determinism guarantee for `era_index_from_pop_tech(pop, tech)` — the
function that maps a civilisation's population count and tech tier
into an era index used by the architecture/build layer. The mapping
must be idempotent: same `(pop, tech)` → same era index.

## User Intent

Era index drives facade styles and unlock sets. A nondeterministic
mapping would change build behavior across replays of the same
input tick, breaking the engine's determinism contract.

## Acceptance Signal

- `cargo test -p build fr_arch_d003_era_index_is_deterministic`
  passes.

## Implementing Code

- `crates/build/src/tiers.rs:633` — test
  `fr_arch_d003_era_index_is_deterministic`
- Production code: `era_index_from_pop_tech` in the same file.

## Test Coverage

- `crates/build/src/tiers.rs:633` (dedicated test, swept 4
  `(pop, tech)` pairs spanning Stone Age to high-tech)

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-civ-arch-d-003-intent.md` |
| Implementing crate | `crates/build/src/` |