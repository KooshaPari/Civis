# Civis pass 10 — mirror causality and trace correction

Date 2026-09-29. Concurrent implementation examined: `54d5758970249c8d1f24688ea45920b530e77299`.

## The 22-field comment is false

Full-function extraction establishes:
- `save_state_mirror_to(&self, target)`: 20 assignments.
- `save_state_mirror(&mut self)`: 17 assignments.
- `load_dir`: restores the same 20 fields written by `save_state_mirror_to`.

The three present only in the 20-field direct-save mirror are:
- `era_progression`
- `emergence_sample`
- `significance`

## Existing test behavior is useful; its causal narrative is wrong

`crates/engine/tests/era_emergence_significance_persistence.rs` says it advances one tick "to fire save_state_mirror" before archiving and treats that as the save-side mechanism for those three fields.

But the tick-time `save_state_mirror()` does not copy those three fields.

The test can nevertheless round-trip them because `CivSaveBundle::save_archive` ultimately invokes `save_dir`, and `save_dir` creates a cloned WorldState then invokes the 20-field `save_state_mirror_to` immediately before serialization.

So classify this test as:
- useful behavioral round-trip evidence for the archive path if/when its run is candidate-bound;
- **not** evidence that the tick-time 17-field mirror carries those three;
- documentation/traceability defect in its stated causal path.

This distinction matters for any other consumer of `WorldState` or replay state between tick completion and save serialization.

## Open semantic question

Trace whether anything reads `sim.state.era_progression`, `sim.state.emergence_sample`, or `sim.state.significance` after tick but before a direct save-side mirror. If such a consumer exists, the live Simulation and WorldState projections can diverge during normal runtime despite archive round-trip being correct.

Do not "fix" the 17-vs-20 difference solely for symmetry before identifying the accepted authority model.
