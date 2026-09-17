# ADR-022: Runtime Representation Deviations (RNG Family, ECS, Fixed-Point Type)

**Status:** Accepted
**Date:** 2026-09-17
**Accepted:** 2026-09-17 (product owner directive: "do the amend call")

## Context

Three strategic functional requirements in
`docs/traceability/TRACEABILITY_MATRIX.md` (CIV-0001, CIV-0500) name concrete
runtime types that the shipped implementation does not use. All three rows have
sat at `in_progress` because the requirement text and the code disagree, not
because the underlying behaviour is missing.

The discrepancies, with evidence:

| FR | Requirement text names | Implementation actually uses | Evidence |
|---|---|---|---|
| FR-CORE-003 | `ChaCha20Rng` | `rand_chacha::ChaCha8Rng` | `crates/engine/src/engine.rs:228` (`pub type SimRng = ChaCha8Rng`), plus 163 further references across 34 files in `crates/` and `clients/` |
| FR-CORE-008 | `bevy_ecs` 0.18.x `World` | `hecs::World` | `crates/engine/src/engine.rs:674` (`pub world: World`), `hecs = "0.10"` in `crates/engine/Cargo.toml` |
| FR-CORE-010 | `FixedI32<U16>` | `civ_engine::Fixed(i64)` | `crates/engine/src/fixed_math.rs:7` (`pub struct Fixed(pub(crate) i64)`), used for `energy_budget_joules` and `faction_treasury` |

Relevant prior art already in the repo:

- `docs/fragemented/research/RND-001-ecs-library-decision.md` (2026-02-21)
  *recommends* `bevy_ecs` 0.18 over `hecs`, explicitly disqualifying `hecs` as
  "too minimal". The recommendation was never implemented.
- `docs/adr/ADR-determinism-dropped.md` (Accepted, 2026-05-30) removes the
  bit-identical-determinism requirement, which was the main driver behind both
  the `bevy_ecs` recommendation (sorted iteration for deterministic order) and
  the fixed-point mandate.

So the current codebase is internally consistent, but two of the three FR
clauses now rest on a rationale that an accepted ADR has retired.

## Decision

**Accepted 2026-09-17.** The requirements are amended to name the types the
project actually ships. The deviations are recorded rather than resolved by
migration.

- **FR-CORE-003** — requirement text amended from `ChaCha20Rng` to
  `ChaCha8Rng`. `docs/specs/CIV-0001-core-simulation-loop.md` invariant I2
  ("ChaCha8Rng Seeded, Not Unseeded") was amended in step. The load-bearing rule
  is preserved verbatim: all randomness is seeded, never `rand::random()`.
- **FR-CORE-008** — requirement text amended from `bevy_ecs` 0.18.x `World` to
  `hecs::World`. `docs/fragemented/research/RND-001-ecs-library-decision.md` is
  **superseded for the engine's ECS choice**: its recommendation was never
  implemented, and the engine is `hecs`-native today. `hecs` is a maintained
  archetype ECS and the engine does not use the scheduler/resources features
  RND-001 called for. If a future workstream needs those, that is a new ADR.
- **FR-CORE-010** — requirement text amended from `FixedI32<U16>` to
  `civ_engine::Fixed` (i64-backed) plus the `i64` KiloJoule / MilliCredit units.
  The requirement's intent (no binary floats in durable economy state) is met
  and tested.

In every case the FR row in `docs/traceability/TRACEABILITY_MATRIX.md` now names
the shipped type, so the matrix no longer claims a type the code does not have.

The behaviour-level properties each FR protects remain independently tested:

- **FR-CORE-003** — `crates/engine/tests/fr_fr_core_003.rs`
  (`no_global_rng_state`) proves randomness is seeded per run and never drawn
  from process-global state: interleaving two seeded simulations does not
  change either trajectory, and a replay with the same seed reproduces it
  exactly.
- **FR-CORE-010** — `crates/engine/tests/fr_fr_core_010.rs`
  (`integer_quantities_use_fixed_point`) proves the energy budget and faction
treasuries persist as integers and that `Fixed` is integer-backed and exact.
- **FR-CORE-008** — `crates/engine/tests/fr_fr_core_008.rs`
  (`no_global_resources`) proves world state is per-`Simulation` with no shared
  global: two simulations run side by side without observing each other's
  entities.

### Alternatives rejected

- **Migrate all 163 `ChaCha8Rng` call sites to `ChaCha20Rng`.** Rejected: it
  changes every random stream in the project (seeded scenarios, emergence
  oracles, genetics, tactics, clients) with no behavioural benefit, and
  `ADR-determinism-dropped.md` means no golden values depend on the stream.
- **Migrate the engine from `hecs` to `bevy_ecs`.** Rejected for now as a
  large, high-risk change to the core simulation loop that no current workstream
  requires. Escalate as a new ADR if scheduler or resource features become
  necessary.
- **Replace `Fixed(i64)` with `fixed::FixedI32<U16>`.** Rejected: the in-tree
  type already provides exact integer-backed arithmetic, is used in the
  persistence path, and swapping it would change persisted state encoding.

## Consequences

- The traceability matrix is honest: every FR row names a type the code actually
  has. All 102 strategic rows are `implemented`.
- Each previously-deviating FR has a behavioural test.
- `RND-001` is superseded for the engine ECS choice; `docs/specs/CIV-0001-core-simulation-loop.md`
  invariant I2 now names `ChaCha8Rng`.
- Reversing this ADR means scheduling the corresponding migration, with the
  understanding that the FR-CORE-003 and FR-CORE-010 migrations change
  simulation output and persisted state encoding respectively.
