# Intent: FR-NFR-S-03 — Citizen count scaling (1k → 10k)

> Date: 2026-09-20
> FR: FR-NFR-S-03
> Epic: FR-NFR-S

## What This FR Captures

The scalability NFR requiring tick time to scale **sub-linearly**
from 1k to 10k citizens. The measurable target is the ratio
`tick_time_10k / tick_time_1k < 8` — expected ~5 with rayon
parallelism. Statement in
`docs/models/civ-sim/TECHNICAL_SPEC.md:2068`.

## User Intent

If a 10× increase in population produced a 10× tick-time blowup,
the engine would be unusable for the 10k+ citizen scenarios that
showcase emergent behavior. Sub-linear scaling means the
parallelism (rayon data-parallel phases, SoA layout) is doing real
work.

## Acceptance Signal

- Criterion comparison benchmark
  (`tick_time_1k` vs `tick_time_10k`) records a value < 8 (target
  ~5) on the reference CI hardware.

## Implementing Code

- `docs/models/civ-sim/TECHNICAL_SPEC.md:2068` — NFR row.

## Test Coverage

- Criterion comparison benchmark (referenced in the NFR table).

## Traceability

| Artifact | Path |
|----------|------|
| Spec | `fr-nfr-s-03-intent.md` |
| General spec | `docs/models/civ-sim/TECHNICAL_SPEC.md` §10.3 |