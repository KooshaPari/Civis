# Civis Legends / Emergent-History Saga-Graph Engine — Design Spec

> **Status:** Design spec (docs-only, 2026-05-30). Owner: Design R&D Lead.
> **Stance:** PLANNER — this document is specs, architecture, acceptance criteria, schemas, and
> brief pseudocode only. It contains **no implementation code**; it equips engineer/codex agents to build.
> **Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only
> physical/environmental/genomic laws are authored; everything else EMERGES. The legends engine is a
> **measured record of what the sim already produced**, never a generator of outcomes.
> **Companions (do NOT duplicate):**
> - [`docs/research/game-rnd.md`](../research/game-rnd.md) §1.2 names this engine as adopt-now #1 (the
>   non-LLM saga-graph *data structure*). This spec is the concrete design of that item.
> - [`docs/research/ai-rnd.md`](../research/ai-rnd.md) §1.1 owns the **SLM narrator** that renders
>   epoch-windowed subgraphs of *this* engine into prose. This engine is BLIND to prose; it only holds
>   structured entities/events/causal edges + a query API the narrator and inspector read.
> - [`docs/research/competitive-benchmark.md`](../research/competitive-benchmark.md) §5 — emergent
>   histories made *readable* is the #1 depth-moat (the Dwarf-Fortress "Legends" lesson); legibility
>   (inspect-anything surfacing a legend) is the non-negotiable parity bar.

---

## 0. Why this is the #1 moat-gate

The competitive benchmark is unambiguous: **Civis's differentiator is emergent depth, and depth that
cannot be *read* is worth zero.** Dwarf Fortress is visually crude yet wins because *Legends mode* lets
you read the emergent history; the reverse (pretty but illegible) loses. Civis already has the substrate
(genetics, needs, tactics, economy, an event stream in `crates/watch`) but no layer that turns the raw
firehose into a **named, causal, queryable history**. This engine is that layer. It is the gate that:

1. Makes emergence **legible** — every inspected entity surfaces *its* saga (FR-CIV-LEGENDS-INSPECT-*).
2. Makes emergence **shareable** — a zero-player sim becomes a chronicle players retell.
3. **Feeds the AI narrator** — the SLM (ai-rnd §1.1) reads epoch-windowed subgraphs as digests; without
   this engine the narrator has no structured source and would hallucinate.

It is pure architecture over in-tree deps (`petgraph` + `crates/watch` + `crates/save-db`), zero charter
risk, and unblocks the entire legibility + narration roadmap. Hence: **build it first.**

---

## 1. Charter alignment (binding constraints)

| Charter rule | How this engine obeys it |
|---|---|
| "Model the rule, not the outcome." | The engine **records** outcomes the sim produced; it authors *no* outcome. Significance is a *measured threshold* over emitted events, exactly like the genomic speciation threshold the charter blesses. |
| Faction/polity is an **emergent cluster**, not `faction:u32`. | Polity/cluster entity nodes carry an opaque `cluster_id` (membership is emergent overlap); the engine stores the ID + provenance only, never an authored faction enum. |
| Determinism NOT required. | Significance scoring, decay, and clustering may use floats and real randomness. The graph is persisted as **state snapshots** (charter §save/load), not a replay-from-seed log. Cache keys for the narrator stay hash-based for cost, not for bit-identical replay. |
| Names emerge from drifted language. | Entity nodes hold a `name: Option<NameRef>` filled by the ai-rnd §1.2 grammar/Markov namer keyed on the entity's emergent culture; this engine never invents names. |
| Loud, not silent (repo CLAUDE.md). | If a producer crate stops emitting lifecycle events, the legends view **shows a gap with a logged warning** (`legends: no civ-agents lifecycle events for epoch N..M`), never a silently-empty saga. |

---

## 2. System overview (the pipeline)

```
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │ PRODUCERS (existing sim crates, per tick)                                     │
 │  civ-agents (birth/death/sickness/migration)  civ-tactics (battles)           │
 │  civ-economy (booms/busts/price shocks)  civ-engine (ideology shifts)         │
 │  civ-genetics (speciation)  civ-planet (disasters)  user god-tools            │
 └───────────────┬─────────────────────────────────────────────────────────────┘
                 │  RawSimEvent  (emitted onto the existing crates/watch broadcast bus)
                 ▼
 ┌─────────────────────────────────────────────────────────────────────────────┐
 │ INGEST (legends crate) — drains the watch bus / .civreplay stream off-tick    │
 │  1. Normalize RawSimEvent → LegendEvent (typed, with participants[])          │
 │  2. Entity resolution: map participant sim-IDs → stable LegendEntityId        │
 │  3. Significance scoring → promotion of entities to "historically significant" │
 │  4. Causal linking: attach caused_by / participated_in / succeeded edges      │
 │  5. Insert nodes+edges into the petgraph StableDiGraph (the saga graph)        │
 └───────────────┬─────────────────────────────────────────────────────────────┘
                 │
                 ▼
 ┌──────────────────────┐     ┌───────────────────────────────────────────────┐
 │ PERSIST              │     │ QUERY API (read-only, sync, cheap)             │
 │ snapshot → save-db   │◄────┤  saga_of(entity)  causal_chain(event)          │
 │ (sqlite, blake3 sum) │     │  timeline(entity, epoch_range)  significant()  │
 └──────────────────────┘     │  epoch_digest(epoch, region) → JSON for SLM    │
                              └───────────────┬───────────────────────────────┘
                                              │
                ┌─────────────────────────────┼─────────────────────────────┐
                ▼                             ▼                             ▼
        Inspector (egui)            Legends browser (egui_graphs)     SLM narrator
        click any entity →          entity → events → causal           epoch digest →
        its saga panel              neighbors (graph viz)              prose (ai-rnd §1.1)
```

**Key architectural decision:** the engine runs **off the sim hot path**. Producers do nothing but emit a
cheap `RawSimEvent` onto the bus that already exists (`crates/watch`'s broadcast `tx`). A dedicated
**legends worker** drains the bus and does all resolution/scoring/linking. The sim tick never waits on the
legends engine (mirrors the ai-rnd §4.3 "never block the sim" rule).

---

## 3. DATA MODEL (the core deliverable)

The saga graph is a `petgraph::stable_graph::StableDiGraph<LegendNode, LegendEdge>` plus side indices for
O(1) lookup. `StableDiGraph` is chosen because IDs must survive node removal (decayed/merged entities) —
indices stay stable across mutation, which a plain `Graph` does not guarantee.

### 3.1 Identifiers

| Type | Definition | Notes |
|---|---|---|
| `LegendEntityId` | newtype `u64` (or `Ulid`) | Stable across the whole game + saves; NOT the same as a sim runtime ID (agents recycle). The entity-resolution map (§4.2) bridges sim-ID → LegendEntityId. |
| `LegendEventId` | newtype `u64` (monotonic) | Append-only; never reused. |
| `Epoch` | `u64` | Coarse game-time bucket (the unit the narrator and pre-sim work in). Derived from sim tick / a configured ticks-per-epoch. |
| `RegionId` | existing chunk/Voronoi region id | Reuses the spatial index already in-tree (game-rnd §1.3 Voronoi / chunk grid). |
| `ClusterId` | opaque id of an emergent polity cluster | Membership emergent; engine stores id only (charter). |
| `NameRef` | handle into the ai-rnd namer's name store | `Option` — entity may be unnamed until promoted. |

### 3.2 Node types — `LegendNode`

Two top-level node kinds (charter §game-rnd 1.2): **Entity** and **Event**. Both are nodes in one graph so
causal edges can connect events *and* participation edges connect entities to events.

```
enum LegendNode {
    Entity(EntityNode),
    Event(EventNode),
}

struct EntityNode {
    id:            LegendEntityId,
    kind:          EntityKind,
    name:          Option<NameRef>,      // filled by ai-rnd namer on promotion
    born_epoch:    Epoch,                // first appearance in the record
    died_epoch:    Option<Epoch>,        // None = still extant
    significance:  f32,                  // 0..1 rolling score (§5); >= threshold ⇒ "significant"
    promoted:      bool,                 // crossed the significance threshold at least once
    home_region:   Option<RegionId>,
    cluster:       Option<ClusterId>,    // emergent polity overlap, if any
    sim_ref:       SimRef,               // back-pointer so the inspector can pull live components
    tags:          SmallVec<Tag>,        // shared data tags (charter: structures share tags regardless of author)
}

enum EntityKind {
    Agent,          // a historical figure (an agent that crossed notability)
    Lineage,        // a kinship/descent line (links to civ-genetics)
    Species,        // a phenotype cluster (Hamming-distance speciation)
    Settlement,     // an emergent built place (civ-protocol-3d building graph)
    PolityCluster,  // an emergent polity overlap (NOT faction:u32)
    War,            // a sustained conflict aggregate (promoted from battle events)
    Disaster,       // flood/quake/eruption/plague aggregate
    Artifact,       // a notable built/crafted object or discovered law
    Discovery,      // an emergent technique/law the sim surfaced
}

struct EventNode {
    id:           LegendEventId,
    epoch:        Epoch,
    region:       Option<RegionId>,
    kind:         EventKind,             // open taxonomy, see §3.4
    magnitude:    f32,                   // normalized 0..1 raw impact (feeds significance)
    participants: SmallVec<LegendEntityId>,   // resolved entity ids (subjects/objects of the event)
    summary_key:  Hash,                  // blake3 of (kind, participants, magnitude, epoch-bucket)
                                         //   → the narrator's prose cache key (ai-rnd §4.2)
    source_crate: SourceCrate,           // provenance (civ-agents / civ-tactics / …) for the loud-gap check
    raw_ref:      Option<RawEventRef>,   // pointer into the .civreplay stream for drill-down
}
```

### 3.3 Edge types — `LegendEdge`

```
enum LegendEdge {
    // event → event (the spine of a saga; makes it a causal DAG, not a flat log)
    CausedBy        { confidence: f32 },   // X happened because Y (heuristic-scored, §4.4)
    Succeeded,                             // temporal succession in the same thread (no causality claim)
    // entity ↔ event
    ParticipatedIn  { role: Role },        // Agent A fought in Battle B (role = aggressor/victim/leader/…)
    // entity → entity (relationship spine, mostly mirrored from sim, lightly held here)
    DescendsFrom,                          // lineage / kinship (from civ-genetics)
    MemberOf,                              // entity ∈ cluster (emergent overlap)
    Founded, Destroyed, Ruled, Built,      // typed entity-entity relations promoted from events
}

enum Role { Aggressor, Defender, Victim, Leader, Founder, Builder, Witness, Cause, Effect }
```

**Acyclicity:** `CausedBy`/`Succeeded` edges form a **DAG** (causality only points backward in epoch order;
the linker (§4.4) refuses an edge whose target epoch ≥ source epoch, guaranteeing no cycle). Entity-entity
edges (`DescendsFrom`, `MemberOf`) may form their own DAG/forest but never feed the causal walk.

### 3.4 Event taxonomy (`EventKind`) — open, producer-owned

Not a closed enum the engine authors; it is a registry each producer crate contributes to (extend-never-
duplicate). Seed set, mapped to producers:

| EventKind | Producer crate | Magnitude driver |
|---|---|---|
| `Birth` / `Death` / `Sickness` | civ-agents (lifecycle) | kinship reach, role of the agent |
| `Migration` | civ-agents | population fraction moved |
| `Battle` / `Siege` / `Raid` | civ-tactics | casualties, forces, territory swing |
| `WarDeclared` / `WarEnded` | civ-tactics (aggregate) | duration, scale |
| `EconomicBoom` / `Bust` / `PriceShock` / `Famine` | civ-economy | Gini/HHI swing, joule throughput (CIV-0107) |
| `IdeologyShift` / `CulturalSpeciation` | civ-engine ideology | meme-cluster cosine drift past threshold (ai-rnd §1.4) |
| `SpeciationEvent` / `Extinction` | civ-genetics | Hamming distance crossed / lineage terminated |
| `Disaster` (`Flood`/`Quake`/`Eruption`/`Plague`) | civ-planet | affected area × severity |
| `SettlementFounded` / `Abandoned` | civ-protocol-3d | size, persistence |
| `Discovery` / `LawObserved` | civ-engine / civ-research | novelty |
| `GodAct` | user god-tools | flagged as player-caused (charter: user-placeable structures) |

A producer registers a `(SourceCrate, EventKind, magnitude_fn)` triple. New producers extend the registry;
the engine treats `EventKind` as opaque + a display label, so adding kinds needs no engine change.

### 3.5 Side indices (kept in sync with the graph)

- `entity_index: HashMap<LegendEntityId, NodeIndex>` — O(1) entity → node.
- `event_index: HashMap<LegendEventId, NodeIndex>` — O(1) event → node.
- `sim_resolution: HashMap<(SourceCrate, SimRuntimeId), LegendEntityId>` — entity resolution (§4.2).
- `epoch_buckets: BTreeMap<Epoch, Vec<LegendEventId>>` — range scans for `timeline` + `epoch_digest`.
- `region_buckets: HashMap<RegionId, Vec<LegendEventId>>` — region-scoped digests.
- `significant_set: BTreeSet<(OrderedF32 /*score*/, LegendEntityId)>` — top-N significant entities, cheap.

---

## 4. INGEST & ENTITY/CAUSAL RESOLUTION

### 4.1 Producer contract (`RawSimEvent`)

Producers emit a minimal payload onto the existing `crates/watch` broadcast bus (the same `tx` that drives
SSE/snapshots). This is the *only* coupling producers take on; they do not depend on the legends crate.

```
struct RawSimEvent {
    tick:          u64,
    region:        Option<RegionId>,
    kind:          EventKind,
    source:        SourceCrate,
    participants:  SmallVec<(SourceCrate, SimRuntimeId, Role)>,
    raw_magnitude: f32,            // crate-local raw impact; engine normalizes
    payload:       MiniPayload,    // small typed bag (e.g. casualties, price delta) for magnitude + drill-down
}
```

**Acceptance:** producers MUST emit on the listed lifecycle hooks (§3.4). A missing producer is detected by
the loud-gap check (§7) — silence is an error, not "no history."

### 4.2 Entity resolution

Sim runtime IDs are recycled (agents die, slots reused); legend IDs must be permanent. The resolver:

1. Look up `(source, sim_runtime_id)` in `sim_resolution`.
2. **Hit** → return existing `LegendEntityId`.
3. **Miss** → mint a new `LegendEntityId`, create a *provisional* `EntityNode` (significance 0, unnamed,
   `promoted=false`), record the mapping. Provisional entities are cheap and pruned by decay (§5.3) if they
   never accumulate significance — so the graph is not flooded by every transient agent.
4. **Aggregate entities** (War, Disaster, PolityCluster) are resolved by a *stable aggregate key* (e.g.
   `(WarBetween, clusterA, clusterB, start_epoch_bucket)`) so repeated battle events fold into one War node.

### 4.3 Promotion (entity → "historically significant")

A provisional entity becomes a first-class historical figure when its rolling `significance` crosses
`PROMOTION_THRESHOLD` (config, default tunable per §5). On promotion:

- `promoted = true`; request a name from the ai-rnd namer (keyed on the entity's emergent culture) →
  fill `name`.
- Emit a `Promotion` legend event (so "X rose to prominence" is itself part of the saga).
- Add to `significant_set`.

Promotion is **monotonic in `promoted`** (once historically significant, the entity stays in the record even
if its live score later decays — DF keeps dead legends). But `significance` itself still decays for *ranking*
(so the Legends browser's "most significant now" view stays fresh).

### 4.4 Causal linking (what makes it a saga, not a log)

When a new `EventNode` is inserted, the linker proposes `CausedBy` edges to *prior* events using cheap,
explainable heuristics (NO ML on the hot path; this is measurement):

1. **Shared participants** — a prior event sharing ≥1 participant entity within a recency window is a
   candidate cause (a regicide event shares the king entity with the succession-war event).
2. **Spatial-temporal proximity** — same/adjacent region within `CAUSAL_WINDOW_EPOCHS`.
3. **Kind-affinity table** — an authored *prior* table of plausible cause→effect kind pairs
   (`Famine → Migration`, `WarEnded → SettlementFounded`, `Disaster → Bust`). This table is the **only**
   authored content in the engine and it is *advisory weighting*, not an outcome (it ranks candidates the
   sim already produced; it never creates events).
4. Score each candidate `confidence = w1·shared + w2·proximity + w3·affinity`; attach `CausedBy` edges above
   `CAUSAL_MIN_CONFIDENCE`, capped at `MAX_CAUSES_PER_EVENT` (keeps the DAG sparse + the "why" answer short).
5. **Acyclicity guard:** reject any candidate whose epoch ≥ the new event's epoch.

`Succeeded` edges are added unconditionally between consecutive events that share the dominant participant
(the thread spine) regardless of causality confidence.

### 4.5 Zero-player pre-sim backstory (DF-style deep history)

Per game-rnd §1.2 and the Toady principle ("history is a record of a zero-player game"): before the player
arrives, run worldgen forward `N` epochs at **far-LOD statistical sim**, feeding the *same* `RawSimEvent`
bus. The legends engine ingests it identically, producing **instant deep backstory**. Bound by a wall-clock
budget (`PRESIM_EPOCH_BUDGET`); if the budget elapses mid-epoch, stop at the last completed epoch and log
`legends: pre-sim truncated at epoch K of N (budget)`. Pre-sim events are tagged `provenance = PreSim` so the
UI can distinguish "deep past" from "lived" history.

---

## 5. SIGNIFICANCE / PROMOTION HEURISTICS

Significance is a **measured, decaying score** — the same pattern as the genomic speciation threshold the
charter endorses, applied to historical notability.

### 5.1 Per-event significance contribution

When an event with participant `e` is ingested, each participant's score gains:

```
Δsig(e) = magnitude
        × role_weight(role)          // Leader/Founder > Witness
        × kind_weight(kind)          // Death/War/Speciation > Sickness
        × reach(e)                   // log(kinship + cluster-membership + territory touched)
        × novelty(kind, region)      // first-of-kind in a region scores higher (rarity bonus)
```

All weights live in a config table (RON, `.env`-overridable) — `template > hardcode` (repo philosophy). No
weight is a charter outcome; they tune *what counts as notable*, not *what happens*.

### 5.2 Rolling aggregation + decay

`significance` is an exponentially-decayed accumulator:

```
significance(e, epoch) = significance(e, epoch-1) · DECAY
                       + Σ Δsig(e) over events this epoch
```

`DECAY ∈ (0,1)` (config). Decay keeps the **"significant now"** ranking fresh (a once-great lineage fades
unless it keeps doing notable things) while `promoted=true` preserves entities in the *record* forever.

### 5.3 Pruning provisional noise

Provisional (`!promoted`) entities whose `significance` decays below `PRUNE_FLOOR` and that have no edges to a
promoted entity are garbage-collected from the graph (their resolution-map entry is dropped). This keeps the
graph bounded at landscape scale (100k agents would otherwise explode the node count). **Promoted entities
and any entity on a causal chain reaching a promoted entity are never pruned.** Pruning logs a periodic
aggregate count (`legends: pruned 4,210 provisional entities this epoch`), not per-entity (loud-but-bounded).

### 5.4 Acceptance criteria (significance)

- AC-SIG-1: A lineage that founds a settlement, wins a war, and crosses a speciation threshold is `promoted`
  before a transient farmer who only had `Birth`+`Death` events.
- AC-SIG-2: With no new events, every non-promoted entity's score reaches `PRUNE_FLOOR` within a bounded
  number of epochs (decay terminates).
- AC-SIG-3: At 100k live agents the graph's *retained* node count stays within `MAX_GRAPH_NODES` (config)
  via §5.3 pruning, verified by the headless bot (game-rnd §2.1).

---

## 6. QUERY API (read-only, what consumers call)

All queries are **synchronous, allocation-light, read-only** against the in-memory graph (the worker holds
the write lock; readers take a read lock / a snapshot handle). Shapes (signatures only, no bodies):

| Query | Signature (conceptual) | Returns | Consumer |
|---|---|---|---|
| `saga_of` | `saga_of(entity: LegendEntityId) -> Saga` | the entity's full sub-saga: its events (chronological) + the causal neighbors of those events + related entities | Inspector saga panel |
| `timeline` | `timeline(entity, epochs: Range<Epoch>) -> Vec<EventNode>` | events touching the entity in the window, epoch-ordered | Inspector timeline strip |
| `causal_chain` | `causal_chain(event: LegendEventId, max_depth) -> CausalDag` | the "why did this happen" DAG — walks `CausedBy` predecessors breadth-bounded | Inspector "why?" + browser |
| `forward_chain` | `forward_chain(event, max_depth) -> CausalDag` | "what did this lead to" — walks `CausedBy` successors | Browser |
| `significant` | `significant(top_n, filter: EntityKind?) -> Vec<EntityRef>` | current top-N by `significance` (from `significant_set`) | Legends browser landing |
| `epoch_digest` | `epoch_digest(epoch, region: RegionId?) -> EpochDigest` | compact JSON bucket of the epoch's events + the deltas of significant entities — **the SLM narrator's input** (ai-rnd §1.1) | SLM narrator |
| `entity_for_sim` | `entity_for_sim(source, sim_id) -> Option<LegendEntityId>` | resolution lookup — the bridge the inspector uses on a click (§8) | Inspector pick |
| `neighbors` | `neighbors(node, edge_filter) -> Vec<NodeRef>` | generic graph step for the `egui_graphs` browser | Browser |

**`EpochDigest` schema (the narrator contract, must be stable + hashable):**

```
struct EpochDigest {
    epoch:        Epoch,
    region:       Option<RegionId>,
    headline_events: Vec<DigestEvent>,   // top events by magnitude, capped (≤ ~20)
    risen:        Vec<EntityRef>,        // entities promoted this epoch
    fallen:       Vec<EntityRef>,        // entities that died/were destroyed
    causal_notes: Vec<(LegendEventId, LegendEventId, f32)>, // high-confidence cause→effect pairs
    digest_hash:  Hash,                  // blake3 over the above → SLM prose cache key (ai-rnd §4.2)
}
```

The digest is **deterministic given the graph state** (sorted, capped) so the same epoch hashes the same and
the narrator's prose cache hits on reload — without requiring sim determinism.

**Acceptance criteria (query):**
- AC-Q-1: `causal_chain` on a succession-war event returns the regicide that shares the king participant.
- AC-Q-2: `epoch_digest` for an unchanged epoch produces an identical `digest_hash` across reloads (cache-safe).
- AC-Q-3: every query is O(neighborhood), not O(graph); `significant` is O(top_n) via the side set.

---

## 7. PERSISTENCE, FAILURE & LOUD-GAP BEHAVIOR

- **Storage:** the saga graph snapshots into `crates/save-db` (sqlite, in-tree). Persist as **state
  snapshots** (charter: not replay-from-seed). Nodes/edges/indices serialize via the project's `serde-evolve`
  versioned tagged-enum envelope (game-rnd §3.2) so old saves migrate forward; a **blake3 checksum** over the
  graph body guards integrity — mismatch → **loud refusal**, never best-effort partial load.
- **Event source of truth:** the `.civreplay` stream / `crates/watch` bus is the authoritative event log;
  the graph is a derived index that can be **rebuilt** from the stream if a save's graph is corrupt (loud:
  `legends: graph checksum mismatch, rebuilding from .civreplay`).
- **Loud-gap detector (required, per CLAUDE.md):** the engine tracks the last epoch each `SourceCrate`
  emitted. If a required producer goes silent for `> GAP_EPOCHS`, the Legends view **renders a visible gap
  marker** and logs `legends: no <crate> events for epoch N..M` — it does **not** silently show an empty
  saga. This is the named-failing-item discipline applied to history.
- **No silent degrade:** if the ai-rnd namer is unavailable, promoted entities display a *named placeholder*
  (`Unnamed Lineage #1234`) with a logged warning — visible, not hidden (mirrors ai-rnd §4.4 cosmetic-degrade).

---

## 8. INSPECTOR INTEGRATION (legibility — the parity bar)

This is how "click any entity → surface its legend" works end-to-end (the highest-leverage polish feature,
game-rnd §4 #2 / benchmark §5):

1. **Pick:** the player clicks a voxel/agent/settlement. The inspector already resolves the hit to a sim
   entity + its `SourceCrate` (the existing inspect-anything pick path, game-rnd §4).
2. **Bridge:** inspector calls `entity_for_sim(source, sim_id)`:
   - **Some(legend_id)** → the entity has a legend. Call `saga_of(legend_id)`.
   - **None** → the entity is below notability; the inspector shows its live components only, plus a passive
     line "no recorded history yet" (not an error — most agents are minor).
3. **Surface:** the inspector's **Legend panel** renders, from `saga_of`:
   - the entity name + kind + birth/death epochs + significance bar;
   - a **timeline strip** of its events (`timeline`);
   - per event, a **"why?"** affordance → `causal_chain` mini-DAG ("this war was caused by the famine of
     epoch 412, which was caused by the eruption of 409");
   - related entities (descendants, cluster, rivals) as clickable chips → re-inspect (graph navigation).
4. **Browser hop:** a "open in Legends browser" button hands `legend_id` to the `egui_graphs` saga browser
   (game-rnd §4 stats/graphs) for free-form graph exploration.

**Acceptance criteria (inspector):**
- AC-INS-1: clicking a promoted settlement shows its founding event, its wars, and the lineage that founded
  it, all clickable.
- AC-INS-2: clicking a transient agent shows components + "no recorded history yet", no error, no empty panel.
- AC-INS-3: the "why?" affordance on any event renders a non-empty causal chain whenever `CausedBy` edges
  exist, and a clean "no recorded cause" when they do not.

---

## 9. FUNCTIONAL REQUIREMENTS (FR-CIV-LEGENDS-*)

| ID | Requirement | Acceptance |
|---|---|---|
| **FR-CIV-LEGENDS-GRAPH-01** | A `petgraph::StableDiGraph` saga graph with `Entity`/`Event` nodes + typed edges, plus the §3.5 side indices, kept consistent on every mutation. | Graph + indices invariant holds under fuzzed insert/prune (proptest, game-rnd §2.3). |
| **FR-CIV-LEGENDS-INGEST-02** | A legends worker drains the `crates/watch` / `.civreplay` event bus off the sim hot path; the sim tick never blocks on it. | Tick-time unaffected with the worker running (perf bench). |
| **FR-CIV-LEGENDS-PRODUCER-03** | Each producer crate emits `RawSimEvent` on its lifecycle hooks (§3.4) via the existing bus; producers do not depend on the legends crate. | Each listed crate emits ≥1 event kind; loud-gap detector green. |
| **FR-CIV-LEGENDS-RESOLVE-04** | Stable entity resolution (sim-id → `LegendEntityId`) surviving sim-id recycling + aggregate-key folding for War/Disaster/Cluster. | Recycled sim-id does not merge two distinct legends; repeated battles fold into one War. |
| **FR-CIV-LEGENDS-SIG-05** | Measured, decaying significance score; promotion at threshold; monotonic `promoted`; provisional pruning bounded by `MAX_GRAPH_NODES`. | AC-SIG-1..3. |
| **FR-CIV-LEGENDS-CAUSAL-06** | Heuristic `CausedBy` linking (shared-participant + proximity + affinity), confidence-scored, capped, acyclic. | AC-Q-1; acyclicity invariant proptest. |
| **FR-CIV-LEGENDS-QUERY-07** | The §6 read-only query API, O(neighborhood), with the stable hashable `EpochDigest` for the SLM. | AC-Q-1..3. |
| **FR-CIV-LEGENDS-INSPECT-08** | Inspector bridge: click any entity → `entity_for_sim` → `saga_of` → Legend panel with timeline + "why?" causal chain + clickable relations. | AC-INS-1..3. |
| **FR-CIV-LEGENDS-BROWSER-09** | An `egui_graphs` Legends browser over the saga graph (entity → events → causal/related neighbors), seeded from `significant()`. | Browser renders a promoted entity's neighborhood; nodes clickable. |
| **FR-CIV-LEGENDS-PRESIM-10** | Zero-player far-LOD pre-sim feeds the same bus to produce deep backstory, wall-clock-budgeted, events tagged `PreSim`. | Backstory present at world start; truncation logged if budget hit. |
| **FR-CIV-LEGENDS-PERSIST-11** | Snapshot persistence to `save-db` via `serde-evolve` + blake3 integrity; rebuild-from-stream on mismatch. | Round-trip fuzz passes; corrupt graph triggers loud rebuild. |
| **FR-CIV-LEGENDS-GAP-12** | Loud-gap detector renders a visible gap + logs a named warning when a required producer goes silent; namer-absent → named placeholder, not silent. | AC: kill a producer → gap marker + log; no silent empty saga. |
| **FR-CIV-LEGENDS-NARRATOR-13** | `epoch_digest` is the contract the ai-rnd §1.1 SLM narrator consumes; digest is deterministic-given-state + hashable for the prose cache. | AC-Q-2 (stable hash); narrator handoff documented. |

**NFR:**
- **NFR-CIV-LEGENDS-PERF-01:** zero measurable sim-tick regression from ingest (off-thread).
- **NFR-CIV-LEGENDS-SCALE-02:** retained graph ≤ `MAX_GRAPH_NODES` at 100k live agents (pruning, §5.3).
- **NFR-CIV-LEGENDS-LOUD-03:** every degrade path is announced + names the failing item (CLAUDE.md).
- **NFR-CIV-LEGENDS-CONFIG-04:** all thresholds/weights via RON + `.env` override; none hardcoded.

---

## 10. PHASED WBS / DAG

| Phase | Task ID | Description | Depends On |
|---|---|---|---|
| **P1 Core graph** | LG1 | `legends` crate skeleton; `LegendNode`/`LegendEdge`/`StableDiGraph` + side indices (§3) | — |
| P1 | LG2 | `RawSimEvent` producer contract + bus tap on `crates/watch` (§4.1) | LG1 |
| P1 | LG3 | Entity resolution + aggregate-key folding (§4.2) | LG1 |
| **P2 Scoring + causality** | LG4 | Significance scoring + decay + promotion + pruning (§5) | LG3 |
| P2 | LG5 | Causal linker (shared-participant + proximity + affinity table, acyclic) (§4.4) | LG3 |
| P2 | LG6 | Off-thread legends worker draining the bus (§2 pipeline) | LG2, LG4, LG5 |
| **P3 Query + persist** | LG7 | Read-only query API + `EpochDigest` (§6) | LG4, LG5 |
| P3 | LG8 | Snapshot persistence (`serde-evolve` + blake3) + rebuild-from-stream (§7) | LG6 |
| P3 | LG9 | Loud-gap detector + namer-absent placeholder (§7) | LG6 |
| **P4 Surfacing** | LG10 | Inspector bridge + Legend panel (timeline + "why?" + relations) (§8) | LG7 |
| P4 | LG11 | `egui_graphs` Legends browser seeded from `significant()` (§9 FR-09) | LG7 |
| P4 | LG12 | Producer event-emission wiring across civ-agents/tactics/economy/engine/genetics/planet (§3.4) | LG2 |
| **P5 Depth + handoff** | LG13 | Zero-player pre-sim backstory feed (§4.5) | LG6, LG12 |
| P5 | LG14 | SLM narrator handoff: `epoch_digest` → ai-rnd §1.1 narrator (contract only here) | LG7 |
| P5 | LG15 | Proptest/fuzz: graph invariant, acyclicity, round-trip, scale-bound (FR-01/06/11, NFR-02) | LG7, LG8 |

**DAG notes:** P1 is the foundation (LG1 unblocks everything). P2 scoring + causality are parallelizable
after LG3 (~2 subagents). P3 query/persist/gap fan out after the worker (LG6). P4 surfacing depends only on
the query API (LG7) and can run parallel to producer wiring (LG12). Aggressive estimate: P1 ≈ 2–3 parallel
subagents / ~8–12 min wall; P2 ≈ 2 subagents / ~8 min; P3 ≈ 3 subagents / ~10 min; P4 ≈ 3 subagents / ~10 min
(egui work); P5 ≈ 2 subagents / ~8 min. Whole engine ≈ 5 phased subagent batches.

---

## 11. Cross-Project Reuse Opportunities (Phenotype org)

Per the Cross-Project Reuse Protocol (confirm destination with user before extraction):
- **The generic `entity / event / causal-DAG over an event stream`** (everything in §3–§6 minus the Civis
  `EventKind` registry) is reusable by any sibling world-sim (WSM3D, DINOForge). Candidate shared crate:
  **`phenotype-history`** — game-rnd §9 already flags this. The Civis `legends` crate would become a thin
  consumer adding the Civis-specific producer registry + kind-affinity table.
- The **`EpochDigest` → SLM narrator** contract pairs with the ai-rnd `phenotype-ai` candidate; the digest
  schema is the clean seam between the two shared crates.

---

## 12. The three highest-impact features (build-order priority)

1. **The saga graph + significance/promotion engine (§3 + §5).** The data model itself is the moat: a
   `StableDiGraph` of named, *measured-significant* entities/events is what converts the raw firehose into a
   readable history. Nothing else works without it, and it is the DF "Legends" moat in one structure.
2. **Inspect-anything → `saga_of` Legend panel (§8).** This is what makes the depth *perceptible* — the #1
   credibility gap per the competitive benchmark. Click any entity, read its legend with a causal "why?".
   Depth that can't be seen is worth zero; this is the feature that makes the whole engine pay off.
3. **Causal linking → `causal_chain` / `epoch_digest` (§4.4 + §6).** The `CausedBy` DAG is what makes it a
   *saga* (a regicide → succession war → migration → new settlement), not a flat log — and `epoch_digest` is
   the clean, hashable contract that feeds the SLM narrator (ai-rnd §1.1), unlocking emergent-history *prose*
   on top of the structured record.
