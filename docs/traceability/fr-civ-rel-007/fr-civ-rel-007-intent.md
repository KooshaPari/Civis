# Intent: FR-CIV-REL-007 -- Public religion belief helpers clamp profile scalars

> Date: 2026-09-20
> FR: FR-CIV-REL-007
> Epic: FR-CIV-REL

## User Intent

The public religion helpers
(`apply_big_gods_response`, `last_religion_sample`,
`substrate_gradients_for`, `ReligiousProfile`, `SubstrateGradients`)
must clamp every profile scalar in `[0.0, 1.0]` after applying the
big-gods response, so extreme substrate gradients never produce
out-of-range belief states.

### What This FR Achieves

Bounded belief scalars mean downstream emergence paths (legends,
psyche, religion oracle) always receive a valid `ReligiousProfile`.

### Product Context

Religion drives FR-EMG-001 (religion emergence oracle) and the
FR-CIV-RELIGION-002 patron gate; unbounded scalars would corrupt
both.

## Acceptance Signal

- Test `fr_civ_religion_007_response_clamps_profile_scalars` in
  `crates/engine/tests/fr_civ_religion_007_phase_belief.rs:1`
  passes.

## Traceability

| Artifact | Path |
|----------|------|
| Test | `crates/engine/tests/fr_civ_religion_007_phase_belief.rs:1` |
| Implementing crate | `crates/engine/src/religion.rs` |
