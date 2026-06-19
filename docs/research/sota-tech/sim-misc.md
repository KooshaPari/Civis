# SOTA Simulation (Misc) → Civis

**Scope:** large-scale sim techniques beyond roads/crowds/physics/gfx — world-history generation, population scale, ML-driven behavior, GPU ECS. Maps to the [emergence charter](../../guides/emergence-charter.md) (emergent history, ~20mi×20mi, LOD-tiered agents). Companion: existing repo docs [dwarf-fortress-rimworld.md](../dwarf-fortress-rimworld.md), [songs-of-syx.md](../songs-of-syx.md) (this extends them with the *technique/integration* angle).

## 1. Dwarf Fortress — world-history generation `[adopt-now: the pattern]`
DF's worldgen is the gold standard for *emergent history*, and its design directly validates the Civis charter. Toady One's own description: **"there's a giant zero-player strategy game going on with somewhat loose turn rules and bad AI (but thousands of agents), and history is just a record of that."** That is *exactly* the charter's "everything emerges from Layer-0 rules; history is a measured pattern over the substrate."

**Techniques to adopt:**
- **Run history as a fast zero-player pre-sim**, then record events into a queryable **Legends** log (historical figures, sites, civs, regions, events). Civis already has an event-log notion (`crates/watch` SSE/snapshots, charter "event log can exist for history/feed"). Make worldgen *play the sim forward* at coarse LOD for N years, logging emergent events → instant deep backstory before the player arrives.
- **Loose turn rules + many simple agents > few smart agents** — DF gets richness from *thousands of cheap agents*, not clever AI. Matches our LOD-statistical-far-agents plan. Don't over-invest in per-agent intelligence; invest in *number of agents × simple drive rules* (utility AI from crowds.md).
- **Legends as a first-class queryable artifact** — surface emergent history to the player (and to debugging) as a browsable record. Cheap to add given our event stream; huge for the "living world" feel.

`[adopt-now]` as a *design pattern* — no library, it's an architecture: coarse pre-sim → event log → Legends view. The cost is "history pre-sim can take a long time" (DF's caveat); bound it with LOD + time budget.

## 2. Songs of Syx — population scale `[adopt-now: the techniques]`
SoS simulates **40,000+ individually-simulated citizens** with 1:1 buildings and thousands moving through streets simultaneously — the proof that *individual* (not purely statistical) large-pop sim is shippable on commodity hardware. Civis targets bigger maps, so SoS's scale tricks matter:
- **Tiered/aggregated simulation:** SoS keeps per-individual detail but aggregates work — jobs, needs, and movement are batched/pooled rather than each citizen running a full independent brain every tick. **Civis maps this to LOD tiers** (charter: full near camera, statistical far). Adopt the *aggregate-the-hot-loops* discipline: pool pathing (flow fields, crowds.md), batch needs ticks, share decisions across similar agents.
- **Data-oriented layout:** scale at this level demands SoA/data-oriented memory (cache-friendly columns), which is *exactly what Bevy ECS gives us for free*. SoS hand-rolled it in Java; we get it from the ECS.
- **Continuous integrated systems:** city-build + economy + governance as one continuous system, not separate modes — aligns with the charter's single emergent substrate.

`[adopt-now]` as techniques: ECS SoA (have it), pooled/batched hot loops, LOD aggregation. The scale bar (40k individually simulated) is a concrete near-term target to benchmark against.

## 3. ML-driven behavior `[experimental, watch]`
- **Learned agent policies** (RL/imitation) instead of hand-authored utility/GOAP: agents that *learn* drives/strategies. Powerful for emergent variety but heavy (training infra, non-determinism — fine per charter, but cost/control concerns) and hard to debug. Civis already has a learned/MCTS-adjacent line ([RND-011-mcts-ai-feasibility](../RND-011-mcts-ai-feasibility.md)); ML behavior is a *later* experimental layer on top of the utility/GOAP substrate (crowds.md), not a replacement. **Small, local models** (per-agent tiny policies, or LLM-driven NPC dialogue/culture flavor via the existing Firepass/Kimi infra — see project memory) are the realistic near-term experimental use: flavor/dialogue/naming, not core sim control. `[experimental]`.
- **Behavior via cultural evolution** (charter Layer-1): note that the charter already gets *learned-looking* behavior emergently via cultural drift/diffusion over kinship networks — often cheaper and more controllable than ML. Prefer the emergent-rules route first; reach for ML only where rules can't produce the needed variety.

## 4. GPU ECS / massive agent sim `[adopt-next / experimental]`
For 20mi×20mi at SoS+ scale, push agent sim to the GPU:
- **GPU-driven agent updates** — needs ticks, drive evaluation, flow-field following as compute shaders over SoA buffers (wgpu, Bevy 0.18). Pairs with GPU crowds (crowds.md §4) and GPU CA (material-physics Tier-1). Keep authoritative/gameplay-critical agents on CPU ECS; offload the statistical mass to GPU. `[adopt-next]` for the hot statistical layer.
- **Bevy ECS is already SoA + parallel** — the CPU baseline is strong; profile before GPU-offloading agents. The first scale wins are *algorithmic* (LOD aggregation, flow fields, pooled pathing) not GPU. Reach for GPU ECS when LOD-tiered CPU ECS is proven insufficient near the camera. `[experimental]` beyond that.

---

## Verdict (sim-misc)
- **Adopt-now (patterns):** DF-style **coarse pre-sim → event-log → Legends** for emergent deep history (have the event stream; add the pre-sim + browsable view); SoS-style **LOD aggregation + pooled/batched hot loops** on Bevy ECS SoA, benchmarked against 40k+ individually-simulated agents.
- **Adopt-next:** GPU-offload the *statistical* agent mass (needs/drives/flow as compute) once CPU LOD-tiering is profiled insufficient.
- **Experimental/watch:** ML/RL learned policies as a later layer over utility/GOAP (prefer emergent cultural-evolution rules first); small local/LLM models for dialogue/culture/naming flavor via existing Firepass/Kimi infra — not core sim control.

## Sources
- [DF2014: Legends (Dwarf Fortress Wiki)](https://dwarffortresswiki.org/index.php/DF2014:Legends) · [World generation (DF Wiki)](https://dwarffortresswiki.org/index.php/World_generation) · [Toady One worldgen description](https://df-walkthrough.readthedocs.io/en/latest/tutorials/generating-a-world.html)
- [Songs of Syx — 40k+ individually simulated population (PC Gamer)](https://www.pcgamer.com/songs-of-syx-is-a-base-building-game-with-massive-scale-battles/) · [SoS scale review (Reality Remake)](https://www.realityremake.com/articles/songs-of-syx-review-a-brutally-complex-colony-sim-with-endless-replay)
- Repo: [dwarf-fortress-rimworld.md](../dwarf-fortress-rimworld.md), [songs-of-syx.md](../songs-of-syx.md), [RND-011-mcts-ai-feasibility.md](../RND-011-mcts-ai-feasibility.md)
