# Intent: FR-CIV-PSYCHE-N11 — Macro psyche rollup belief bonus

> Date: 2026-09-19
> FR: FR-CIV-PSYCHE-N11
> Epic: FR-CIV-PSYCHE

## User Intent

The dormant-phase `phase_psyche` projects the average `Psyche` maturity
across all `Psyche`-component agents upward into a macro belief bonus.
Per-agent mood/belief mutation already runs in `phase_emergence`; this
phase only rolls up aggregate maturity. The bonus is `floor(maturity * 10)`
additive belief, applied only when maturity > 0.

## Acceptance Signal

- `crates/engine/src/dormant_phases.rs:88` `phase_psyche` exists and uses
  `avg_psyche_maturity(&self.world)` to drive an additive `add_belief`.
- Zero-maturity short-circuit prevents spurious belief gains.
- `// Covers: FR-CIV-PSYCHE-N11` on the function doc-comment.
