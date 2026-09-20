# Intent: FR-EMG-023 -- Genetics inheritance oracle

> Date: 2026-09-20
> FR: FR-EMG-023
> Epic: FR-EMG

## User Intent

The genetics oracle must confirm that the inheritance path keeps
offspring within parent-derived bounds: every child genome must (a)
preserve the parent genome length, (b) inherit each locus from one
of the parents before mutation, (c) stay within the class mutation
budget. 64 deterministic Monte-Carlo trials must all succeed after
tick > 0.

### What This FR Achieves

Verifies the recombination + mutation invariants that
`civ-genetics` enforces during the emergence phase.

### Product Context

Wired into `oracle_report` CI baseline; threshold 0 at tick 0,
all 64 trials thereafter.

## Acceptance Signal

- Library unit test
  `inheritance_trial_respects_parent_loci_before_mutation` in
  `crates/emergence-oracle/src/oracles/genetics.rs` passes.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/emergence-oracle/src/oracles/genetics.rs:1` |
| Code | `crates/emergence-oracle/src/oracles/genetics.rs:60` |
| Implementing crate | `crates/emergence-oracle/src/` |
