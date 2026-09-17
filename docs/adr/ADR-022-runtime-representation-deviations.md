# ADR-022: Runtime Representation Deviations (RNG Family, ECS, Fixed-Point Type)

**Status:** Proposed
**Date:** 2026-09-17
**Supersedes (in part):** n/a

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

Record the deviations rather than silently rewriting the requirements. This ADR
is **Proposed**: until it is Accepted, FR-CORE-003, FR-CORE-008, and FR-CORE-010
remain `in_progress` in the traceability matrix, with this ADR cited.

The behaviour-level properties that each FR was protecting are independently
enforced by new tests, so the risk of marking the rows `implemented` later is
bounded:

- **FR-CORE-003** — `crates/engine/tests/fr_fr_core_003.rs`
  (`no_global_rng_state`) proves randomness is seeded per run and never drawn
  from process-global state: interleaving two seeded simulations does not
  change either trajectory, and a replay with the same seed reproduces it
  exactly.
- **FR-CORE-010** — `crates/engine/tests/fr_fr_core_010.rs`
  (`no_float_in_state`) proves no persisted world-state quantity requires a
  floating-point representation, and that `Fixed` serialises as an integer.
- **FR-CORE-008** — no behavioural test is possible without the migration; the
  property at stake ("no global singletons") is structural.

Recommended resolution, in cost order:

1. **FR-CORE-003 — amend the requirement to `ChaCha8Rng`.** Cheapest and
   lowest risk. `ChaCha8Rng` is the established workspace convention; migrating
   163 call sites to `ChaCha20Rng` changes every random stream in the project
   (all seeded scenarios, emergence oracles, genetics, tactics) with no
   behavioural benefit, and `ADR-determinism-dropped.md` means no golden
   values depend on the stream. Recommendation: **amend the FR text.**
2. **FR-CORE-010 — amend the requirement to the in-tree `Fixed(i64)`
   fixed-point type.** The requirement's intent ("no binary floats in durable
   simulation state") is met and now tested. Recommendation: **amend the FR
   text.**
3. **FR-CORE-008 — decide between migrating to `bevy_ecs` or amending the FR.**
   This is a genuine architectural question, not a naming one. `RND-001`
   recommends `bevy_ecs`; `hecs` is already load-bearing across the engine.
   Migrating is a large, high-risk change; amending the FR is cheap but
   abandons RND-001's reasoning. Recommendation: **requires an explicit product
   decision** — do not change this one unilaterally.

## Consequences

- The traceability matrix stays honest: `in_progress` continues to mean "not
  implemented as written".
- Each deviating FR now has either a behavioural test or a documented reason
  why none is possible.
- If this ADR is accepted, the three FR rows can move to `implemented` after the
  requirement text is amended to match, and `docs/traceability/TRACEABILITY_MATRIX.md`
  should cite this ADR from those rows.
- If this ADR is rejected, FR-CORE-008's migration must be scheduled, and the
  FR-CORE-003 / FR-CORE-010 typed migrations must be scheduled with the
  understanding that they change simulation output.
