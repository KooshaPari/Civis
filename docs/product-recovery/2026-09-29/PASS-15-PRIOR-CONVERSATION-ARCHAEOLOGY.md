# Civis archaeology pass — mature horizon before later implementation corrections

Recovered2026-09-30 from prior conversation context. Authority labels preserved.

## User intent anchor —2026-02-19

User wanted Civis/CivLab to combine patterns from Cities: Skylines, WorldBox, Diplomacy Is Not an Option, Civilization7 and Empire at War:
- deep politics/governance;
- public/private competition;
- war/defense;
- deep economics;
- hybrid crowd + agent simulation;
- macro/detail zooms.

This predates the current repository charter and confirms that the mature product horizon is not merely a city renderer or narrow ABM notebook.

## Assistant synthesis — useful, not automatically normative

February assistant outputs proposed:
- civilization-scale coupling among military/economic/political/urban systems;
- macro sectors, meso firms, selective household agents;
- sanctions, mobilization, logistics, attrition, legitimacy and ideology;
- two/three-level LOD with macro distributions, active meso regions and selective micro agents;
- conservation across zoom levels;
- at that time, deterministic seeded transitions/replay.

The coupled multi-scale model is compatible with the user's explicit mature horizon and is useful prior design evidence. The deterministic-replay portion is **superseded by the later May29-30 accepted charter/ADR that drops global bit-identical replay**. Do not revive it simply because it appeared in an older assistant architecture.

## Reconciliation

Mature intent remains broad: civilization-scale coupled simulation with zoom/resolution changes and observable intervention.

Later accepted implementation semantics refine that horizon:
- actual-state snapshot restoration, not universal future bit identity;
- randomness/floating point allowed;
- v1 single-player; multiplayer/co-op/spectator excluded;
- emergence-first "model rules, not predetermined outcomes."

Thus the recovery should preserve macro/meso/micro and coupled economics/politics/war as mature-horizon candidates while independently validating exact models, conservation laws, LOD transfer semantics and scientific claims.

## Evidence consequence

A save/state architecture that cannot preserve policy, research, mod identity or cross-resolution ownership undermines the mature user intent even if a narrow renderer/demo works. Conversely, a broad historical assistant model must not be treated as scientifically validated merely because it is ambitious.
