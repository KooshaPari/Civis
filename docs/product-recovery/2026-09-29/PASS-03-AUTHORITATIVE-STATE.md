# Civis pass 3 — authoritative state denominator

Date 2026-09-29. Original frozen source remains `b3cd62a7394878cc64d024fbfcfd398b8bb88bf1`. This pass inspects concurrent main `54d5758970249c8d1f24688ea45920b530e77299`; findings are revision-bound.

## The persistence denominator is not WorldState

At concurrent main, `WorldState` (engine.rs blob `b5fabb65b3f8982c7feb5fdf913d3e8ed32efa13`) contains a substantial durable-looking surface, but `Simulation` owns a second large state universe: hecs World, RNG, planet/moon/climate, current_tick, damage queue, allocator, research cache, economy/market, policy object/signals, voxel substrate, mod host, doctrines, coastal/weather state, construction/social/religion/cohesion histories and many last-tick buffers.

Several Simulation fields mirror fields in WorldState. The save path explicitly calls `save_state_mirror_to` before serializing world_state.json, proving that live authority is not simply "whatever WorldState currently contains."

Six extended WorldState fields are marked `#[serde(default, skip)]`: faction religions, faction language systems, civilian psyches, settlement building layouts, historical log and faction writing systems. Their presence in WorldState therefore does not make them durable through world_state.json.

The mature save contract needs a generated/maintained **authoritative-state manifest**, not a hand-curated claim that WorldState is exhaustive.

## C-F09 — WorldState equality is not a save oracle

`impl PartialEq for WorldState` compares only:
- tick;
- population;
- energy_budget_joules;
- rng_seed;
- resources.

It ignores factions, treasuries, faction resources, trade routes, belief/cohesion/unrest, diplomacy, languages, institutions, build sites, economic focus, culture, ideology, aggression, accumulators, research, era, emergence, significance and more.

Therefore any test using whole-object `assert_eq!(loaded.state, original.state)` can pass while most WorldState fields differ. Some current persistence tests correctly assert selected fields individually, but equality itself must not be treated as exhaustive evidence. Either define semantic equality explicitly/exhaustively for persistence or use an independent manifest-driven comparator.

## C-F10 — orphan persistence implementation must be quarantined from active-path evidence

`crates/engine/src/save.rs` contains a separate `SavedSimulation`, world/ECS snapshot, voxel snapshot and `save_game/load_game` implementation. Its fetched struct/body are internally inconsistent around environment fields. More importantly, `crates/engine/src/lib.rs` at the same revision does **not** declare `mod save` or export save_game/load_game; it declares and exports `save_bundle`.

Thus the apparent compile contradiction is falsified for the active crate: save.rs is currently orphan/dead source under the inspected module graph. Its tests and apparent capabilities cannot qualify production save/load unless another build target includes it. Preserve it as historical/experimental evidence and decide whether useful pieces should be migrated or the file retired after history review.

Active save evidence must bind to `CivSaveBundle` and mounted callers.

## C-F11 — active bundle integrity covers serialized files, not omitted live state

Concurrent `save_bundle.rs` v5 computes BLAKE3 digests for every serialized file except metadata/integrity and rejects unlisted files. This is valuable **artifact integrity**.

It cannot prove state completeness. If a live authoritative Simulation field is never serialized into any component, the integrity manifest faithfully proves the completeness of an incomplete bundle.

The state denominator therefore precedes integrity qualification:
1. enumerate every Simulation/WorldState/ECS/mod/config field;
2. classify canonical durable / derived-reconstructible / cache-projection / ephemeral / historical event / duplicate mirror / unknown;
3. for every durable field, identify exact save component and restore symbol;
4. for every derived field, specify deterministic derivation inputs/invariant without reinstating global future determinism;
5. mutate each durable field away from default before save and observe it after load.

## C-F12 — mutable policy/economy state is a priority completeness target

`Simulation::economy_policy` is mutable at runtime through `sim.set_policy` and god-tool difficulty law; `market_state` advances each economy phase. Neither is a WorldState field. They are examples of state that can change future behavior after load and therefore must be explicitly classified as durable or intentionally reconstructed/reset.

This pass does not claim they are definitely omitted from all active sidecars until the full save_bundle write/read body is mechanically mapped. They are priority probes because simple WorldState round trips cannot cover them.

Likewise the active `policy: Box<dyn Policy>`, last control signals, RNG position/state, allocator, pending damage, research cache, mod set vs guest memory, doctrine libraries, ECS component subsets, voxel state, queues and social histories require explicit classification.

## Initial state-manifest schema

For each state item record:
- stable state ID and semantic owner;
- runtime field/symbol;
- authoritative vs mirror/projection;
- mutation sites;
- affects present observation? future behavior? both?;
- persistence disposition;
- serialized component/path;
- restore symbol;
- migration/version rule;
- consistency peers;
- positive non-default fixture;
- omission/corruption/conflict fixture;
- evidence candidate/config/world identity.

No field is green because it derives Serialize, appears in a struct, or has a round-trip test with weak equality.

## Next pass

Mechanically extract Simulation and WorldState fields, then map every write/read in save_bundle plus save_state_mirror_to and load restoration. Cross-check mounted Bevy/server/watch callers. Quarantine orphan save.rs evidence. Run omission probes for economy_policy/market_state/policy/RNG/ECS components and every serde-skip WorldState extension.

No native execution was performed by this recovery worker.
