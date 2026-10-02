# SOTA / existence gate — consolidation pass

Date: 2026-10-01. Research refreshed against current public material.

## Best realistic absent-product stacks

Civis has two distinct substitute classes.

### Research / simulation substitute
- Mesa (or NetLogo/Repast/MASON-class ABM tooling) for agent scheduling/spaces, experimentation, visualization and analysis;
- ODD protocol for rigorous model description and fitness-for-purpose documentation;
- domain-specific scientific models and notebooks;
- standard data-analysis stack.

This is substantially better than building Civis merely to run experiments.

### Playable-world substitute
No single research framework is the right substitute. A realistic stack would be a game engine/ECS + bespoke simulation subsystems + persistence + UI/rendering + mod/plugin system, or simply existing simulation/god/strategy games where their authored model is acceptable.

The product therefore survives only if the integrated **playable causal world + intervention + emergence + extensibility** is itself valuable.

## Commodity / bootstrap decisions

### USE / INTEGRATE
- Wasmtime/Wasm Component Model for sandboxable cross-language plugin/component execution where its capability/performance model fits. Wasmtime already provides a sandboxed WebAssembly runtime and Component Model embedding API; Civis should not invent a VM.
- mature ECS/rendering/storage/serialization primitives already in the chosen Rust/game stack.
- standard statistical/data-analysis tools for experiments.

### ADAPT / LEARN
- ODD as a model-documentation projection: purpose, entities/state, process/scheduling, design concepts, initialization/input/submodels, rationale/evaluation. Civis's model ontology can map to it without forcing product UX/implementation into ODD.
- Mesa as an alternative/reference for experiment ergonomics and model analysis, not as the playable runtime.

### CUSTOM THESIS
- coupled world model and its accepted causal semantics;
- multi-resolution simulation contracts where needed;
- player/god intervention bound to the same authoritative world;
- persistence of causal state;
- emergence oracles that distinguish authored labels from causal outcomes;
- game-facing visualization/interaction and explainability;
- product-specific extension API over the accepted world model.

## Differentiation ledger

### Commodity / falsified differentiation
- "has agents";
- "has a scheduler";
- "can visualize an ABM";
- "supports plugins";
- "can serialize state";
- generic browser dashboards.

### Candidate differentiation
1. One coherent playable world spanning physical/material, biological, individual, social, economic, institutional and warfare scales.
2. Emergence-first causal contract: important higher-level phenomena should arise from accepted lower-level mechanisms rather than only scripted labels.
3. God-game intervention with inspectable causal consequences.
4. Multi-LOD world/simulation architecture that preserves accepted identities/conservation across fidelity boundaries.
5. A mod/model extension system integrated with world identity, persistence and evidence.
6. Research-grade model documentation/evaluation coexisting with a usable game, without claiming scientific validity merely because the simulation is detailed.

### Unverified
- that the coupled model produces interesting rather than noisy/degenerate worlds;
- that deeper physics materially improves play;
- that emergence is legible to users;
- that the scale is computationally practical;
- that the product is more useful for experiments than Mesa-class tools;
- that users prefer the integrated product to narrower established games.

## Architecture consequence

Civis should not become "Mesa in Rust" or "a pile of game systems." Its durable conceptual spine is:

model/version + world identity
-> authoritative causal state
-> explicit scheduler/process boundaries
-> interventions/events
-> projections/rendering/analysis
-> faithful save/recovery
-> extension capability boundary
-> model/game/evidence oracles

Research documentation is an orthogonal projection. ODD can structure the model description; it should not be misused as the product requirement tree.

## Existence gate

Civis survives provisionally if its value proposition remains the integrated playable mega-simulation. If empirical pilots show that:
- emergence is mostly scripted labels,
- deep substrate adds cost without useful behavior,
- multi-LOD coupling is unstable,
- or a game-engine + Mesa/notebook split provides the same user/research outcomes more cheaply,
then the current integrated architecture should be reduced or decomposed.

No design document can close that empirical question.

## Sources refreshed

- Mesa repository/docs, retrieved 2026-10-01: https://github.com/mesa/mesa
- ODD protocol update: Grimm et al., JASSS 23(2)7, 2020: https://jasss.soc.surrey.ac.uk/23/2/7.html
- Wasmtime introduction/security/component API, retrieved 2026-10-01: https://docs.wasmtime.dev/ ; https://docs.wasmtime.dev/security.html ; https://docs.wasmtime.dev/api/wasmtime/component/index.html
- Wasmtime plugin/component example: https://docs.wasmtime.dev/wasip2-plugins.html

External evidence establishes available primitives/prior art, not Civis model validity.
