# Emergent Tech / Era Progression + Civ-Driven Engineering — Design Spec

**Status:** Design spec (docs-only, planner stance). Owner: Design Lead.
**Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only physical/environmental/genomic laws are authored; *technology is not a fixed tree*. **Determinism is NOT required** (charter §"Determinism is NOT a requirement"): floats and real randomness are welcome where they enrich emergent variety; seeded worldgen stays a best-effort convenience.
**Companion specs:** [`docs/research/ai-rnd.md`](../research/ai-rnd.md) (the hybrid canonical/LLM tech-card pipeline), [`docs/design/civ-ai-crate.md`](./civ-ai-crate.md), [`docs/design/legends-engine.md`](./legends-engine.md).
**Existing crates this spec maps onto (do NOT duplicate):** `crates/laws` (the physics/materials laws DB + validator), `crates/research` (`TechCard`, `LlmClient`, `validate`, `ResearchCache`, `LlmEvent`, `ReplayMode`, `run_research_cycle`), `crates/diffusion` (Bass/Rogers S-curve), `crates/needs`, `crates/economy` (`Stocks`, `surplus`/`deficit`, `comparative_advantage`), `crates/species`/`crates/agents` (lineages, psyche, knowledge carriers).

---

## 0. Thesis

Technology in Civis is **not a tree the player climbs**. It is a *measured emergent pattern*: a population invents a technique when three independent pressures coincide — **NEED** (an unmet need or economic deficit a technique would relieve), **RESOURCE** (the material inputs are locally available), and **KNOWLEDGE** (the prerequisite techniques/laws are already known to that population). Every candidate technique is **validated against the authored `civ-laws` DB** (conservation, material properties, energy cost) before it can become real — the laws DB is the only authority on what is physically possible; the *order and timing* of discovery emerge.

"Eras" are therefore **not gates** — they are **labels we read off** the distribution of known techniques across a population (a percentile threshold over the era-rank of what is known and adopted). A civ does not "advance to the Bronze Age"; we *observe* that its adopted-knowledge frontier has crossed the bronze threshold and we name the epoch accordingly.

This spec covers, in order:
1. **The emergence model** — how a technique candidate forms from need × resource × knowledge and is law-validated (§1–§3).
2. **The hybrid canonical/LLM tech-card pipeline** — replay-safe, per [`ai-rnd.md`](../research/ai-rnd.md) (§4).
3. **Diffusion-driven aging** — Bass/Rogers spread of an invented technique through a population, and era transitions as emergent thresholds over that spread (§5–§6).
4. **Civ-driven engineering** — agents/settlements building tools, machines, structures when knowledge + resources allow (§7).
5. Crate mapping, FR catalogue, phased WBS/DAG (§8–§10).

---

## 1. Domain model (emergent, not enumerated)

These are **measured records**, not hardcoded enums. A "technology" is a cluster ID over the substrate, exactly as the charter treats species/factions.

| Concept | What it is | Backing store |
|---|---|---|
| **Technique** | A validated `TechCard` (`crates/research::TechCard`): `{id, era, inputs, energy_cost, byproducts, dependencies}`. `era` here is an **era-rank label** (see §6), not a gate. | `crates/research` |
| **KnowledgeSet** | The set of technique IDs a *population* (lineage cluster / settlement / culture cluster) currently *knows* (has discovered, not necessarily adopted). | new: `crates/research` (`KnowledgeSet`), held per cluster in `civ-engine` |
| **AdoptionState** | Per-technique adopted fraction `f ∈ [0,1]` within a population, advanced by `civ-diffusion`. Knowing ≠ using. | `crates/diffusion` curve state, held per (cluster, technique) in `civ-engine` |
| **NeedPressure** | A scalar `[0,1]` per technique-affordance derived from `civ-needs` deficits + `civ-economy` `deficit()`/scarcity. "How badly would inventing this help?" | derived in `civ-engine` from `crates/needs` + `crates/economy` |
| **ResourceAvailability** | Whether a technique's `inputs` are locally extractable/in-stock (over `civ-economy::Stocks` + voxel material presence). | derived from `crates/economy` + `crates/voxel` material map |
| **EraLabel** | A *read-off* percentile over the era-rank of a population's adopted techniques (§6). Not stored as authoritative state. | computed in `civ-engine` |

> **Charter compliance:** none of the above is a fixed list. The *laws* are authored; the *technique catalogue* is open-ended (canonical seed cards + LLM-proposed cards), and which techniques a given population holds is an emergent cluster, not a global tech level.

---

## 2. The emergence trigger: NEED × RESOURCE × KNOWLEDGE

### 2.1 Invention-readiness score

A population becomes *ready to invent* a candidate technique `T` when a multiplicative gate clears. Multiplicative (not additive) so that **any** missing factor blocks invention — you cannot invent steelmaking with no need, no ore, or no prerequisite knowledge.

**Pseudocode (planner sketch — not implementation):**

```
readiness(pop, T) =
    need_pressure(pop, T)          // [0,1] from civ-needs deficits + civ-economy deficit/scarcity
  * resource_availability(pop, T)  // [0,1] fraction of T.inputs locally obtainable
  * knowledge_fraction(pop, T)     // [0,1] fraction of T.dependencies already in pop.KnowledgeSet

invention_attempt(pop, T) occurs this tick with hazard proportional to:
    base_rate * readiness(pop, T) * cognitive_capacity(pop)
// cognitive_capacity derives from emergent genomic/sentience traits (civ-species) +
// population size + free labour surplus (civ-economy). A pre-sentient lineage ≈ 0.
```

- `need_pressure`: high when agents in the population are starving/cold/unsafe (`civ-needs`) **or** the economy shows a persistent `deficit()` / scarcity in a good `T` would produce (`civ-economy::deficit`, `surplus`). This is the "necessity is the mother of invention" term.
- `resource_availability`: fraction of `T.inputs` (resource IDs) currently in `Stocks` or extractable from nearby voxel materials. A technique whose inputs are absent cannot be invented locally — but **can arrive by diffusion/trade** (§5), which is how resource-poor regions still acquire tech.
- `knowledge_fraction`: fraction of `T.dependencies` (which are **law IDs / prerequisite technique IDs**) already in the population's `KnowledgeSet`. At `1.0`, all prerequisites are known; below `1.0`, `T` is "not yet conceivable here."
- **No global timer.** A population with high need, abundant ore, and metallurgy knowledge invents iron tools *early*; a sheltered, resource-poor one may never invent them. Variety-that-makes-sense, per charter.

### 2.2 Candidate sourcing

Candidates `T` for a population come from three sources, in priority order:
1. **Adjacent-possible canonical cards** — seed `TechCard`s whose `dependencies` are *one step* beyond the population's `KnowledgeSet` (cheap, replay-safe, always-available backbone).
2. **Diffused techniques** — a neighbour already knows `T`; the population can *adopt by learning* rather than *invent from scratch* (§5; far higher hazard, lower cost).
3. **LLM-proposed novel cards** — when need is high but no canonical/diffused candidate fits, request a *new* `TechCard` from the AI pipeline (§4). This is how genuinely emergent / alt-history / fictional techniques enter the world (gated to `Hybrid`/`Free` saves).

### 2.3 Acceptance — law validation is the only authority

Every candidate (canonical, diffused, or LLM) passes through the existing `crates/research::validate(card, &LawDb)` **unchanged**:
- declared `dependencies` must exist in the `LawDb` (`UnknownDependency`);
- each dependency's `era_min` must be ≤ the card's `era` (`DependencyEraGated`);
- the card must have effects (`NoEffects`).

On `Accept`, `T.id` is added to the population's `KnowledgeSet` and a fresh `AdoptionState` (f≈ε) is seeded for it. On `Reject`, the candidate is discarded and (for LLM cards) the rejection is logged for the dev-assist balance analyst (`ai-rnd.md` §3). **No technique exists that the laws DB does not permit** — this is the charter's "model the rule, not the outcome."

### Acceptance criteria — emergence trigger
- **AC-E1**: `readiness` returns `0.0` if *any* of need/resource/knowledge is `0.0` (multiplicative gate).
- **AC-E2**: A population with `knowledge_fraction < 1.0` for `T` never adds `T` to its `KnowledgeSet` by invention (only by diffusion learning, which still requires the prerequisite to be learnable next).
- **AC-E3**: Every accepted technique passed `civ-research::validate` against the *current* `LawDb`; no bypass path exists.
- **AC-E4**: Two populations with identical (need, resource, knowledge) state but different `cognitive_capacity` invent at different expected rates.
- **AC-E5**: Removing all need pressure halts net new invention even with abundant resources + knowledge (no invention-for-its-own-sake).

---

## 3. Knowledge, not a global tech level

- Knowledge is **per-population and partial**. There is no `world.tech_level`. Each lineage/settlement/culture cluster carries its own `KnowledgeSet` + per-technique `AdoptionState`.
- **Knowledge can be lost** (collapse, depopulation, severed contact): if a population holding `T` dies out or fragments below a viability threshold and no neighbour retains `T`, `T.id` leaves the regional knowledge frontier — a *dark age* emerges as a measured regression, not a scripted event. (Charter: regressions are as valid as advances.)
- Knowledge is **carried by agents**, so it moves with migration, captives, traders, and texts (once writing emerges as a technique). This wires knowledge transmission into the existing kinship/contact networks the charter already uses for culture/language drift.

### Acceptance criteria — knowledge
- **AC-K1**: No global tech-level field exists; era is always derived per population (§6).
- **AC-K2**: A population's `KnowledgeSet` is a strict function of its invention + diffusion history; it can both grow and shrink.
- **AC-K3**: Severing all contact + extinguishing the last holder of `T` removes `T` from that region's frontier (knowledge loss is representable).

---

## 4. Hybrid canonical / LLM tech-card pipeline (replay-safe)

Per [`ai-rnd.md`](../research/ai-rnd.md) §1/§4 and the existing `crates/research` machinery — **reuse, do not reinvent.**

### 4.1 Two lanes behind one validator
- **Canonical lane** (always on, all `ReplayMode`s): hand-authored seed `TechCard`s in RON, mirroring the `civ-laws` RON convention. Provides the backbone (fire → stone tools → pottery → smelting → …) so a world is playable with zero model calls and `Canonical` saves are fully replayable.
- **LLM lane** (opt-in, `Hybrid`/`Free` saves only): when §2.2 finds no canonical/diffused candidate for a high-need population, enqueue an async request to the AI worker pool (`ai-rnd.md` §4.3). The model proposes a *novel* `TechCard`; it is then validated by the **same** `civ-research::validate`. The LLM never bypasses the laws DB — it only proposes; the laws DB disposes.

### 4.2 Replay-safety (existing `ReplayMode` + `LlmEvent`, unchanged)
- Every LLM proposal is recorded as an `LlmEvent` (blake3 `cache_key = prompt_hash ++ input_snapshot_hash ++ model_id ++ model_version`) and gated by `replay_advance_llm_event`:
  - **`Canonical`** — refuses any `LlmEvent` on replay (`CanonicalLlmEvent`); the backbone alone replays bit-stably.
  - **`Hybrid` / `Free`** — replay requires a cache hit (`HybridCacheMiss` otherwise); live play caches the proposal so reload is free and consistent.
- Per the charter, **determinism is not a hard requirement** for live play — but the cache is mandatory (cost/latency) and the event log gives an *optional* replay/history path for saves that want it. Cosmetic naming of techniques (via `ai-rnd.md` §1.2 grammar) need not be replay-gated; the *card itself* (which has sim effects) is.

### 4.3 Never blocks the sim
- Invention requests are `AiTask`s on the bounded queue (`ai-rnd.md` §4.3). The sim tick advances regardless; an accepted card lands in the population's `KnowledgeSet` whenever it is ready (this tick, later, or from cache on reload). A pending request never stalls a tick.

### Acceptance criteria — pipeline
- **AC-P1**: A `Canonical` save never emits an `LlmEvent`; invention draws only from canonical + diffused candidates.
- **AC-P2**: An LLM-proposed card that fails `validate` is discarded and never enters any `KnowledgeSet`.
- **AC-P3**: Replaying a `Hybrid` save with a populated cache reproduces the same accepted cards (cache hits); a cold cache halts replay loudly (`HybridCacheMiss`), per existing tests.
- **AC-P4**: No sim tick blocks on an invention request (async worker pool; bounded queue with backpressure).

---

## 5. Diffusion: spreading an invented technique through populations

Once `T` is known by *one* population, it spreads. This is the existing `crates/diffusion` Bass/Rogers engine, applied per (population, technique).

### 5.1 Adoption within a population (intra-cluster)
- Each `(pop, T)` holds an `AdoptionState` `f ∈ [0,1]` = fraction of that population *using* `T` (not merely aware). Advanced each culture-tick by `civ-diffusion::advance(f, params)`:
  - `f'(t) = (p + q·f) · (1 − f)` — innovation coefficient `p` (spontaneous uptake), imitation coefficient `q` (copying adopters).
- **Params are emergent, not constant.** `DiffusionParams { p, q }` per `(pop, T)` are *modulated* by substrate state (the meta-analysis defaults `p≈0.03, q≈0.38` are only a starting prior):
  - `p` ↑ with `need_pressure(pop, T)` (urgent tech catches on faster) and `resource_availability` (can't adopt what you can't supply).
  - `q` ↑ with population density / contact-network connectivity (more neighbours to imitate) and cultural openness (an emergent ideology metric).
  - `q` ↓ with cultural conservatism / taboo (emergent), modelling tech that stalls despite being known.
- This makes adoption **gradual and uneven** — visible tech (tools, wardrobe, architecture) propagates across a civ rather than snap-upgrading, exactly the stated purpose of `crates/diffusion`.

### 5.2 Cross-population spread (inter-cluster) — knowledge transmission
- A technique crosses from population A to neighbour B along the **contact/kinship/trade network** (the same graph used for culture/language drift). When `A.adoption[T]` is high and an A↔B contact edge exists, B *learns* `T` (adds it to `B.KnowledgeSet` with f≈ε) — provided B can satisfy `T`'s resource needs (or import the inputs via `civ-economy` trade).
- Spread rate along an edge scales with contact intensity (trade volume from `civ-economy::propose_trade`/`apply_trade`, migration, shared culture). This is why **trade routes are tech vectors**: comparative-advantage trade (`civ-economy::comparative_advantage`) both moves goods *and* carries techniques.
- **Resource gating on adoption** (not on learning): B may *know* `T` yet keep `f≈0` if it cannot source `T.inputs` — knowledge outruns capability until trade or local extraction catches up. Realistic and emergent.

### Acceptance criteria — diffusion
- **AC-D1**: `AdoptionState.f` is monotonic non-decreasing under non-negative `(p,q)` (inherits `civ-diffusion` FR-CIV-DIFFUSION-003) and saturates at ≤1.0.
- **AC-D2**: Higher `need_pressure` ⇒ higher effective `p` ⇒ faster early adoption for the same `q`.
- **AC-D3**: A technique known by A spreads to a *contacting* neighbour B but not to an isolated population C with no path to A.
- **AC-D4**: B can hold `T` in `KnowledgeSet` with `f≈0` while `T.inputs` are unavailable, then `f` rises once trade supplies the inputs (knowledge-before-capability).
- **AC-D5**: Cutting the A↔B contact edge halts further cross-population spread along it.

---

## 6. Era transitions as emergent thresholds (diffusion-driven aging)

Eras are **read off**, never set.

### 6.1 Era-rank of a technique
- Each `TechCard` carries `era` (existing field) = an **era-rank** — a monotone label from `civ-laws` (`era_min` of the dominant dependency), e.g. stone≈0, bronze≈2, iron≈3, steam≈6, electric≈7, fictional≈9+. This is *only* a sortable difficulty rank, **not a gate**: a population can hold a high-rank technique without holding lower-rank ones if its path there was unusual (charter: alt-paths welcome).

### 6.2 Era label of a population (a percentile read-off)
- `era_label(pop)` = a high percentile (e.g. p70) of the era-rank of techniques that pop has **adopted** (`f` above an adoption floor, e.g. `f ≥ 0.5`) — weighted by adoption fraction. Knowing-but-not-using doesn't age a civ; *broad adoption* does.
- An **era transition** is the event "`era_label(pop)` crosses an integer boundary." It is detected, named, and emitted to the **legends/event log** (`legends-engine.md`, `ai-rnd.md` §1.1) — never used to unlock anything (unlocking is governed by knowledge §2 + laws §2.3, not by era).
- Because the label is driven by the **diffusion** `f` values, era advance is **gradual and population-specific**: a civ "enters the Iron Age" when iron tools have *diffused widely enough*, not when the first smith invents them. Regions age at different rates; a world holds multiple coexisting eras simultaneously (a steam-power core trading with a stone-tool periphery) — emergent, not scripted.

### 6.3 Regression
- If adoption collapses (knowledge loss §3, depopulation, resource exhaustion cutting `f` back toward 0), `era_label` *falls* — an emergent dark age, surfaced to legends as a measured regression.

### Acceptance criteria — eras
- **AC-A1**: No code path *gates* any capability on `era_label`; era is purely descriptive/observational.
- **AC-A2**: `era_label(pop)` rises only as adopted-technique `f` values rise (driven by `civ-diffusion`), not on invention alone.
- **AC-A3**: Two contacting populations can hold different `era_label`s simultaneously (uneven aging) and the world reports both.
- **AC-A4**: A modelled adoption collapse lowers `era_label` (regression is representable and emitted to legends).

---

## 7. Civ-driven engineering (agents invent and build)

Engineering = applying *known* techniques to produce **artifacts** (tools, machines, structures, roads, vehicles) when knowledge + resources + need align. This reuses the charter's "Architecture & civ-driven engineering" layer and the existing build/protocol-3d crates.

### 7.1 From technique to artifact
- A `TechCard` describes a *capability* (`inputs → outputs/byproducts` at `energy_cost`). An **artifact** is a concrete instance built by an agent/settlement that *realises* that capability in the world (a forge realises smelting; a cart realises wheeled transport; a granary realises storage).
- An agent/settlement initiates a build when: (a) the enabling technique is in its `KnowledgeSet` **and** adopted (`f` high enough to have practitioners); (b) the artifact's material inputs are in `Stocks` / extractable; (c) a need or economic payoff justifies it (`civ-needs` deficit or `civ-economy` surplus-seeking — e.g. build a road where a desire-path's traffic cost is high; build a granary where food spoils between surplus and deficit seasons).
- The build itself is a **physics/economy transaction**: it consumes inputs, takes labour-time, emits byproducts, and is mass/energy-conserving against `civ-laws` — same validation discipline as invention. A structure that violates structural-stress or material laws cannot stand (defer to the physics engine / `civ-laws` material properties).

### 7.2 Self-organising, author-agnostic
- Per charter: structures built by agents and structures placed by the **user** share the same data tags — the sim does not distinguish author. Roads form along desire-paths (emergent path-cost reinforcement, `ai-rnd.md` §1.5 narrow-RL note — classic flow-field, not neural). Anarchic/decentralised regions can engineer without a central polity; a "settlement" is an emergent co-location cluster, not `faction:u32`.
- Engineering **complexity tracks the era frontier emergently**: a population only builds water-wheels once it has adopted the relevant mechanical techniques; it builds rail once metallurgy + steam have diffused. No build menu is unlocked by era — builds are unlocked by *known + adopted techniques* (§2/§5), which is what era merely *describes*.

### Acceptance criteria — engineering
- **AC-G1**: An artifact build requires the enabling technique to be both *known* and *adopted* (`f` ≥ floor) in the building population; a known-but-unadopted technique yields no builders.
- **AC-G2**: A build consumes its declared inputs from `Stocks` and is rejected if inputs are unavailable (no free construction).
- **AC-G3**: Agent-built and user-placed structures of the same type carry identical data tags (author-agnostic).
- **AC-G4**: Build availability is gated by known+adopted techniques, never directly by `era_label`.
- **AC-G5**: A structure is subject to `civ-laws` material/structural validation (cannot violate conservation/stress).

---

## 8. Crate mapping (extend, never duplicate)

| Capability | Crate | New surface (additive) |
|---|---|---|
| Technique record + validation | `crates/research` | `KnowledgeSet` type; `readiness()` helper; novel-card request hook into AI worker pool |
| Physics/material authority | `crates/laws` | none — used as-is as the validator authority; seed canonical era-rank `era_min`s |
| Per-technique adoption spread | `crates/diffusion` | emergent-`DiffusionParams` modulation hook (params derived from need/density/culture); no math change |
| Need pressure | `crates/needs` | read-only consumer; expose aggregate deficit per population |
| Resource availability + trade vectors | `crates/economy` | read-only consumer of `Stocks`/`deficit`/`comparative_advantage`/`propose_trade` |
| Cognitive capacity / knowledge carriers | `crates/species`, `crates/agents` | derive `cognitive_capacity`; agents carry `KnowledgeSet` membership |
| Orchestration (per-pop knowledge/adoption/era state) | `crates/engine` | hold `KnowledgeSet` + `AdoptionState` per cluster; compute `era_label`; drive invention/diffusion/build ticks |
| Artifact builds | `crates/build`, `crates/protocol-3d` | technique→artifact realisation; author-agnostic structure tags |
| AI proposal lane | `crates/ai` (per `ai-rnd.md`) + `crates/research` | async novel-card proposals behind the existing `LlmClient`/`AiProvider` port |
| Narration of inventions/era transitions | legends/event log | emit invention, diffusion-milestone, era-transition, regression events |

> **Cross-project reuse:** the `readiness = need × resource × knowledge` gate + the diffusion-params-from-substrate modulation are generic "emergent capability unlock" primitives — candidate for a shared Phenotype substrate alongside the `civ-ai` extraction (flag per the reuse protocol; confirm destination before extracting).

---

## 9. FR catalogue — `FR-CIV-TECH-*`

| FR | Requirement | Verifies | Maps to AC |
|---|---|---|---|
| **FR-CIV-TECH-001** | Invention readiness is the multiplicative product of need, resource, and knowledge factors; any zero factor blocks invention. | `readiness` unit tests | AC-E1, AC-E5 |
| **FR-CIV-TECH-002** | A technique is added to a `KnowledgeSet` only after passing `civ-research::validate` against the current `LawDb`. | validation-gate test | AC-E3, AC-P2 |
| **FR-CIV-TECH-003** | Invention requires `knowledge_fraction == 1.0` (all prerequisites known) for from-scratch invention. | prerequisite-gate test | AC-E2 |
| **FR-CIV-TECH-004** | `cognitive_capacity` modulates invention hazard (pre-sentient ⇒ ~0). | hazard-scaling test | AC-E4 |
| **FR-CIV-TECH-005** | Knowledge is per-population and partial; no global tech-level field exists. | architecture/test | AC-K1, AC-K2 |
| **FR-CIV-TECH-006** | Knowledge loss is representable: extinguishing the last holder + severing contact removes a technique from a region's frontier. | knowledge-loss test | AC-K3, AC-A4 |
| **FR-CIV-TECH-007** | Canonical saves never emit `LlmEvent`s; invention draws from canonical + diffused candidates only. | replay-mode test | AC-P1 |
| **FR-CIV-TECH-008** | LLM-proposed cards are validated by the same `validate`; rejects never enter any `KnowledgeSet`. | LLM-gate test | AC-P2 |
| **FR-CIV-TECH-009** | Hybrid/Free replay reproduces accepted cards from cache; cold cache halts loudly (`HybridCacheMiss`). | replay test (reuses existing) | AC-P3 |
| **FR-CIV-TECH-010** | No sim tick blocks on an invention/proposal request (async, bounded queue, backpressure). | concurrency test | AC-P4 |
| **FR-CIV-TECH-011** | Per-`(pop,technique)` adoption advances via `civ-diffusion`; monotone non-decreasing, saturates ≤1.0. | diffusion-integration test | AC-D1 |
| **FR-CIV-TECH-012** | `DiffusionParams` are modulated by emergent substrate (need ↑p; density/openness ↑q; conservatism ↓q). | param-modulation test | AC-D2 |
| **FR-CIV-TECH-013** | A technique spreads across a contact edge to a neighbour but not to an isolated population. | cross-population test | AC-D3, AC-D5 |
| **FR-CIV-TECH-014** | Knowledge can precede capability: a population holds a technique with `f≈0` until trade/extraction supplies inputs. | knowledge-before-capability test | AC-D4 |
| **FR-CIV-TECH-015** | `era_label(pop)` is a percentile read-off over adopted techniques' era-rank; no capability is gated on it. | era-derivation test | AC-A1, AC-A2 |
| **FR-CIV-TECH-016** | Multiple eras coexist across contacting populations (uneven aging). | multi-era world test | AC-A3 |
| **FR-CIV-TECH-017** | Adoption collapse lowers `era_label`; regression is emitted to legends. | regression test | AC-A4 |
| **FR-CIV-TECH-018** | Era transitions (boundary crossings) are emitted to the event log, never used to unlock. | era-event test | AC-A1, AC-A2 |
| **FR-CIV-TECH-019** | An artifact build requires its enabling technique to be known *and* adopted (`f` ≥ floor). | build-gate test | AC-G1, AC-G4 |
| **FR-CIV-TECH-020** | A build consumes declared inputs from `Stocks`; unavailable inputs reject the build (mass-conserving). | build-economy test | AC-G2, AC-G5 |
| **FR-CIV-TECH-021** | Agent-built and user-placed structures of the same type share identical data tags (author-agnostic). | tag-parity test | AC-G3 |

---

## 10. Phased WBS (DAG)

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| P1 Foundation | T1 | `KnowledgeSet` + per-`(pop,technique)` `AdoptionState` state in `civ-engine`; seed canonical era-rank `TechCard`s (RON) | — |
| P1 | T2 | `readiness()` = need × resource × knowledge; wire `civ-needs`/`civ-economy` read-only consumers | T1 |
| P1 | T3 | Invention tick: candidate sourcing (adjacent-possible canonical) → `validate` → `KnowledgeSet` insert | T1, T2 |
| P2 Diffusion | T4 | Per-`(pop,technique)` `civ-diffusion` integration + substrate-modulated `DiffusionParams` | T1 |
| P2 | T5 | Cross-population spread over contact/trade network (`civ-economy` trade as tech vector) | T4 |
| P2 | T6 | `era_label` percentile read-off + era-transition/regression events to legends | T4 |
| P3 LLM lane | T7 | Novel-card proposal path (async AI worker, `ai-rnd.md` §4) gated to Hybrid/Free; `LlmEvent` replay | T3, `civ-ai` P1 (`ai-rnd.md`) |
| P3 Engineering | T8 | Technique→artifact build (`crates/build`/`protocol-3d`); input consumption; author-agnostic tags | T1, T4 |
| P3 | T9 | Desire-path road formation (flow-field reinforcement) as emergent engineering | T8 |
| P4 Dev-assist | T10 | Balance analyst over tech-spread telemetry (dead-end tech, runaway leader) — `ai-rnd.md` §3 | T3, T6 |

**Aggressive agent-effort:** P1 ≈ 3–4 parallel subagents / ~15 min wall; P2 ≈ 2–3 subagents / ~10 min; P3 ≈ 2–3 subagents per task; P4 ≈ 1–2 subagents. T7 depends on the `civ-ai` extraction landing first (`ai-rnd.md` P1).

---

## 11. Summary of the two headline models

**Emergence model (invention):** a population invents a technique when **need × resource × knowledge** clears a multiplicative gate, scaled by emergent cognitive capacity; the candidate is **law-validated** by `civ-research::validate` against the authored `civ-laws` DB before it can exist. Canonical seed cards form a replay-safe backbone; the LLM lane proposes genuinely novel cards (Hybrid/Free only, replay-cached). No global tech tree — knowledge is per-population, partial, transmissible, and losable.

**Diffusion-driven aging (spread + eras):** a known technique spreads via the Bass/Rogers S-curve (`civ-diffusion`) — intra-population adoption with substrate-modulated `p`/`q`, and inter-population transmission along the contact/trade network (trade routes are tech vectors). **Eras are read off, not gated**: `era_label` is a percentile over the era-rank of *adopted* techniques, so a civ ages gradually as tech *diffuses widely*, not when it is first invented; multiple eras coexist, and adoption collapse produces emergent dark-age regression. Capability is always governed by known+adopted techniques and the laws DB — never by the era label itself.
