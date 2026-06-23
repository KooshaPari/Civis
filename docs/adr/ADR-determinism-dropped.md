# ADR: Determinism Requirement Dropped

**Number:** Supersedes ADR-003 (no separate numeric number — see file naming note below)
**Title:** Determinism Requirement Dropped
**Date:** 2026-05-30
**Status:** **Accepted**
**Supersedes:** [ADR-003-deterministic-replay.md](ADR-003-deterministic-replay.md)

> File naming note: this ADR intentionally keeps the original non-numeric filename
> `ADR-determinism-dropped.md` because that is how it was originally proposed and
> reviewed. It is treated as the authoritative decision that supersedes ADR-003.
> See `docs/adr/README.md` for the canonical index.

## Context

When ADR-003 was accepted (2026-02-21) it imposed a strict, **zero-tolerance**
non-determinism policy on every simulation run: bit-identical replay, mandatory
`ChaCha20Rng` seeding, no `HashMap` iteration in simulation paths, fixed-point
money, CI-gated replay tests, etc. That framing made sense when the project was
positioned primarily as an audit-grade artifact producer.

Since then the project's center of gravity has shifted toward **emergent
civilization simulation** (faction emergence, religion emergence, language
phoneme drift, N-series emergence coupling, the emergence charter). The Civis
emergence charter explicitly says:

> determinism is not a requirement and that real randomness, floats, and
> snapshot-based persistence are acceptable where they support richer
> emergence.

That is a deliberate correction to the older replay-determinism framing, and
the rest of the codebase has followed it in practice (e.g. snapshot-based
`sim.snapshot` persistence rather than replay-from-seed; floating-point
quantities where convenient; non-seeded randomness for variety). The two ADRs
contradict each other and the contradiction must be resolved for the record.

Relevant docs:

- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md)
- [`docs/adr/ADR-003-deterministic-replay.md`](ADR-003-deterministic-replay.md)
- [`docs/guides/voxel-emergent-vision-and-migration.md`](../guides/voxel-emergent-vision-and-migration.md)

## Decision

**Civis does not require bit-identical determinism, seeded-only reproducibility,
fixed-point arithmetic, or determinism-test gates for the main simulation path.**

Concretely:

1. The simulation engine **may** use real (non-seeded) randomness where it
   produces richer emergence (weather, personality, faction drift, mutation,
   etc.).
2. The simulation engine **may** use floating-point arithmetic where it
   improves implementation simplicity or numerical stability; fixed-point is
   **no longer** mandated.
3. Persistence uses **snapshot-based save/load of actual state**
   (`sim.snapshot` and friends) as the primary recovery mechanism. Replay
   from an event log is **not** a substitute for snapshot persistence.
4. Replay-from-seed and bit-identical replay **remain available** as a tool
   for debugging or for a specific subsystem that explicitly opts into that
   contract (e.g. a test fixture), but they are **not** a global CI gate.
5. ADR-003 is **superseded** by this ADR. The original ADR-003 text is kept
   in place so the historical decision trail is preserved.

## Consequences

### Positive

- The codebase is free to prioritize emergent variety over replay lockstep.
- Engineering effort is no longer spent policing `HashMap` vs `BTreeMap`,
  chasing hidden RNG leaks, or writing replay-replay tests just to merge.
- Subsystems that genuinely benefit from determinism (replay fixtures,
  specific unit tests, networking lockstep) can still opt in locally
  without dragging the whole engine into a replay contract.
- Aligns the ADR set with the actual codebase, the emergence charter, and
  the in-flight work on snapshot-based persistence.

### Negative

- Bugs that *would* have been caught by a replay-determinism CI gate may now
  surface later (e.g. flaky unit tests driven by accidental randomness). We
  mitigate by encouraging per-test seeding where the test benefits from it.
- External systems (e.g. downstream Venture artifact builds) that previously
  relied on bit-identical CIV replay must now treat CIV runs as
  non-deterministic and either snapshot or opt into a determinism contract
  explicitly.
- Contributors familiar with the old "zero-tolerance" framing may be
  surprised. The README index and this ADR's supersede notice are the
  canonical pointer.

## Implementation Notes

- Do **not** add new CI gates that assert bit-identical replay for the main
  simulation path.
- Existing per-test determinism (single-seed unit tests, snapshot golden
  tests) is fine and should be kept where it already exists.
- If a future subsystem needs determinism (e.g. multiplayer lockstep), it
  should ship its own local ADR or note in this one rather than re-asserting
  ADR-003.

## References

- [ADR-003-deterministic-replay.md](ADR-003-deterministic-replay.md) — the
  superseded decision.
- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — the
  charter that effectively dropped the determinism requirement first.
- [`docs/guides/voxel-emergent-vision-and-migration.md`](../guides/voxel-emergent-vision-and-migration.md)
  — snapshot-based persistence in practice.

