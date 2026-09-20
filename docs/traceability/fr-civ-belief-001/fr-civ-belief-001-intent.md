# Intent: FR-CIV-BELIEF-001 -- Religious profile substrate gradients

> Date: 2026-09-19
> FR: FR-CIV-BELIEF-001
> Epic: FR-CIV-BELIEF

## User Intent

The product owner requires Religious profile substrate gradients as part of the FR-CIV-BELIEF epic for the Civis civilisation simulation.

### What This FR Achieves

This functional requirement defines the per-settlement substrate-gradient snapshot
(`SubstrateGradients`) and emergent `ReligiousProfile` that feeds the Norenzayan
Big-Gods response curve (`apply_big_gods_response`) in the religion emergence
module. The module exposes:

- Hard saturation ceilings (`MAX_MISERY_UNREST`, `MAX_D_*_PER_TICK`) that clamp
  per-tick deltas to avoid runaway monitoring / coherence growth.
- `SubstrateGradients` (§7) — hardship, scarcity, belief, kinship density, unrest,
  migration rate, language distance — all clamped to [0, 1].
- `ReligiousProfile` (§8) — per-settlement monitoring, mythic_coherence,
  uncertainty_reduction fields used by downstream emergence / metrics consumers.
- `substrate_gradients_for` and `last_religion_sample` surface functions that the
  `emergence.metrics` consumer calls each tick.

### Product Context

Civis is a Rust-based civilisation simulation built on Bevy ECS with 42 crates.
FR-CIV-BELIEF-001 contributes to the overall simulation capability by addressing:
the religion / substrate gradient pathway that lets downstream emergence layers
observe how mythic coherence builds under hardship, scarcity, and unrest.

## Acceptance Signal

### Definition of Done

- [x] `crates/engine/src/religion.rs` defines `SubstrateGradients`, `ReligiousProfile`, and `apply_big_gods_response`.
- [x] Source already tagged with `// FR-CIV-BELIEF-001 §N` doc references (§7, §8, §10.1).
- [x] `cargo build -p engine` passes.
- [x] No regressions in existing FRs.

### How We Know This FR Is Satisfied

1. `cargo build -p engine` compiles cleanly with the religion module exposed.
2. The substrate gradients are clamp-bounded and produce deterministic outputs for identical inputs.
3. Downstream `emergence.metrics` consumer compiles against `last_religion_sample` and `substrate_gradients_for`.

## Traceability

| Artifact | Path |
|----------|------|
| Code | `crates/engine/src/religion.rs` (constants, structs, response curve, surface) |
| Implementing crate | `crates/engine/` |
