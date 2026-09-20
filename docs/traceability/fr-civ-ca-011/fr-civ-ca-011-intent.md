# Intent: FR-CIV-CA-011 -- Pairwise material reaction cascade

> Date: 2026-09-19
> FR: FR-CIV-CA-011
> Epic: FR-CIV-CA

## User Intent

The cellular automaton in `crates/voxel/src/fluid_ca.rs` must apply binary
material-pair reactions per dirty cell (e.g. fire↔wood/plant combustion,
lava↔ice/water thermal, acid↔stone, fire+water boil-off) grounded in the
Powder-Toy/Noita "friction" reference. The reaction pass sits between the
`phase_transition_pass` and the `evaporation_pass` in the per-tick CA
scheduler (`step_with_parity`).

### What This FR Achieves

`reaction_pass(grid, &cells)` walks the dirty-cell set, looks up the
material-pair rule via `reaction_for(left, right)`, and writes the result
pair into both cells. AIR (id 0) never reacts. Writes are immediate
(matching the `phase_transition_pass` snapshot semantics) so a hot
neighbour set can ignite a full fuel cell in one pass.

## Acceptance Signal

- `cargo test -p voxel reaction_pass` passes (covers ignition cascades,
  AIR non-reactivity, the 6-neighbourhood, dirty-cell marking).
- Step order in `step_with_parity`: thermal → phase → reaction → evaporation
  → saturation → percolation → boundary.
- Reactions are deterministic; no RNG; identical inputs → identical outputs.

## Traceability

| Artifact | Path |
|----------|------|
| Implementing crate | `crates/voxel/` |
| Reaction pass | `crates/voxel/src/fluid_ca.rs:1360` |
| Stepper wiring | `crates/voxel/src/fluid_ca.rs:1698` |
