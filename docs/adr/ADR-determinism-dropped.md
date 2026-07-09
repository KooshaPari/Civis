# ADR: Determinism Requirement Dropped

**Status:** Accepted
**Date:** 2026-05-30

## Context

The Civis emergence charter explicitly says determinism is not a requirement and that real randomness, floats, and snapshot-based persistence are acceptable where they support richer emergence. That is a deliberate correction to the older replay-determinism framing. Relevant docs:

- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md)
- [`docs/adr/ADR-003-deterministic-replay.md`](ADR-003-deterministic-replay.md)
- [`docs/guides/voxel-emergent-vision-and-migration.md`](../guides/voxel-emergent-vision-and-migration.md)

## Decision

Civis does not require bit-identical determinism, seed-only reproducibility, or determinism test gates for the main simulation path. The engine may use real randomness and floating-point arithmetic where that improves emergence or implementation simplicity. Persistence should rely on save/load of actual state rather than replay-from-seed guarantees.

## Consequences

- The codebase is free to prioritize emergent variety over replay lockstep.
- Determinism-focused constraints should not be added unless a subsystem explicitly needs them.
- Old replay-determinism assumptions remain useful only as historical context, not as a current system requirement.
- Documentation and tests should avoid asserting bit-identical replay unless a specific subsystem explicitly opts into that contract.

