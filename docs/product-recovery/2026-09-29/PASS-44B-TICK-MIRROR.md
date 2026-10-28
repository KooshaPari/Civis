# Pass 44B — runtime tick mirror continuity defect

Date 2026-10-01.
Observed implementation: `590fad0643eb85cae89edd9e64ed6b991461de6e`.

## Production finding

`CivSaveBundle::load_dir` reconstructs Simulation from replay, then replaces `sim.state` from persisted `world_state.json` and explicitly mirrors many WorldState-owned fields back into Simulation.

It does **not** assign `sim.current_tick = sim.state.tick`.

Engine tick behavior explicitly assigns `current_tick = state.tick` during normal tick advancement, and runtime phases consume `current_tick` directly. Therefore after load and before the next full synchronization tick, the object can expose two tick truths.

Classification: production continuity defect candidate with source-complete causal chain; executable oracle added to vNext experiment.

## Candidate repair

Experiment candidate adds:
- oracle: save at nonzero tick -> opt-in load -> require `loaded.current_tick == loaded.state.tick` before another tick;
- semantic apply explicitly resynchronizes `current_tick` from authoritative `state.tick`.

This keeps `state.tick` canonical and treats `current_tick` as a MIRROR with explicit restore.

The previous 17/17 run remains valid only for its exact earlier candidate; it does not transfer green to this new control.
