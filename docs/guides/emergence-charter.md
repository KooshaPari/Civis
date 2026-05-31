# Civis Emergence Charter

**The single governing principle:** Civis hardcodes *only* the environmental and physical simulation rules that model reality. **Everything else emerges** from those rules. Because the rules model reality, outcomes will often track reality closely — but the rules leave enough freedom for *variety that makes sense*. Nothing about life, society, economy, polity, language, or technology is a fixed enum or scripted concept; each is a measured, emergent pattern over the substrate.

## Layer 0 — The hardcoded substrate (the "laws of nature")
These, and essentially only these, are authored:
- **Physics engine.** Voxel material-fluid CA with full gravity: liquids flow + find level, powders pile at angle-of-repose + pack toward solid, gases rise/diffuse, solids hold. World-boundary walls. Deterministic, integer/fixed-point, mass-conserving. Forces, structural stress, thermal, pressure where they matter.
- **Chemistry/energy + materials DB** (`crates/laws`): conservation laws, material properties, reaction/phase rules, energy costs — the constraints every emergent process must satisfy.
- **Climate/planet** (`crates/planet`): insolation, weather, hydrology, day/night, tides.
- **Genomics** (`crates/genetics`): DNA byte-vectors, mutation, recombination, fitness, speciation thresholds.

## Layer 1+ — What EMERGES (never hardcoded)
- **Life & custom species + paths to sentience.** Abiogenesis from local material+energy conditions; species = phenotype clusters (Hamming-distance speciation), not a humanoid enum. Sentience is a *threshold* a lineage may cross via accumulated cognitive/genomic traits — not a given. Many viable body/mind plans.
- **Psyche.** Per-agent drives, temperament, beliefs, memory — shaping individual choices and divergence.
- **Ideology & culture & language.** Belief systems, norms, and languages drift + diffuse across populations (cultural evolution over kinship/contact networks); dialects/creoles emerge from contact.
- **Markets of varying types.** Not one market model — gift, barter, commodity, mercantile, credit, planned — each emerging from local resource/trust/scarcity conditions. Comparative advantage + surplus/deficit drive trade.
- **Polities / states — decentralized, not necessarily explicit mutual collectives.** Organizing structures emerge from co-location + kinship + culture + economic payoff + coercion. A "faction" is one possible shape; anarchic/decentralized/networked/tributary forms are equally valid. Membership is emergent cluster overlap, NOT `faction: u32`.
- **Architecture & civ-driven engineering.** Houses, roads/trails/highways, vehicles, tools, machines — built by agents/settlements when needs+resources allow (self-organizing, anarchist regions possible) AND placeable by the user. Roads form along desire-paths. Structures share data tags regardless of author.

## Scale target
- **~20 mi × 20 mi** real-world-equivalent maps (or the max realistic below that). Tractable via the **SVO + dense-leaf-chunk substrate + chunk streaming + LOD + frustum culling** already built: keep an active working set in memory, stream the rest from disk. At a coarse base voxel (e.g. 1–4 m) this is feasible; **disk space is the primary bound**, not compute, given aggressive chunking/LOD. Agent sim runs LOD-tiered (full near the camera/active areas, statistical far away).

## Determinism is NOT a requirement (correction, 2026-05-29)
Strict lockstep determinism — seeded-RNG-only, fixed-point everything, `thread_rng` forbidden, bit-identical replay — is **explicitly NOT a requirement** for Civis. It was inherited from the old CivLab replay-determinism ADR and **conflicts with the emergent-variety goal**: the point is a living world that produces *variety that makes sense*, not one that replays identically every run.
- Agents/systems MAY use real randomness (`thread_rng`, OS entropy, floats) where it serves richer emergence. No need to fixed-point or seed-gate for reproducibility.
- **Seeded worldgen stays useful** (so "New World, seed X" yields a recognizable starting world) — but that's a *best-effort convenience*, not a bit-identical invariant. Floats are fine.
- Save/load persists actual state (snapshots), not a replay-from-seed log; an event log can exist for history/feed but need not guarantee bit-identical re-simulation.
- Do NOT spend effort enforcing determinism, writing determinism tests, or rejecting non-deterministic crates/plugins on determinism grounds. Prefer the simpler/livelier implementation.

## Design rule for every contributor (human or agent)
Before adding any "thing," ask: *can this emerge from Layer-0 rules instead of being hardcoded?* If yes, model the rule, not the outcome. Authored content is limited to physical law, material/energy constraints, and genomics primitives. See [[civis-voxel-fluid-vision]], [[hierarchical-agents]].

## Scope boundary (2026-05-31)
- **Multiplayer / co-op / spectator is OUT OF SCOPE for v1.** Civis is single-player. The WebSocket layer is client↔server transport for one player, NOT multiplayer netcode; with determinism dropped, true MP would need state-sync/rollback — explicitly deferred. Do not build co-op features.
- **Actor rigging + animation + "rich aspects" are IN scope** — skeletal/glTF animation on agents & creatures (idle/walk/work/combat driven by emergent activity), richer agent representation, lifelike world feel.
