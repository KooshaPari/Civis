# Civis Master Implementation Roadmap — Phased DAG Synthesizing All Design Docs

> **Status:** Master synthesis (2026-05-30). Owner: Design R&D Lead. **PLANNER stance** — this is a
> sequencing/scheduling artifact (phased WBS + DAG, acceptance gates, handoffs); it contains **no
> implementation code** and authors no new requirements. It *orders* the work already specified in the
> per-area design docs into one execution plan.
> **Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only
> physical/environmental/genomic laws are authored; everything else EMERGES. Determinism is **not** required.
>
> **This document does not duplicate the per-area specs.** Each design doc owns its own internal WBS, FR
> catalog, and acceptance criteria; this roadmap references them and sequences their *phases* against each
> other. When a wave below says "see PSYCHE P1–P4," the authority for that work is the design doc, not here.

---

## 0. What this synthesizes

Source design docs (each its own internal WBS, all read into this plan):

| Design doc | FR block | Charter role | Critical-path role |
|---|---|---|---|
| [`info-views.md`](./info-views.md) | `FR-CIV-INFOVIEW-900..930` | Legibility (measure, never define) | **Critical path root** — the #1 credibility gap |
| [`legends-engine.md`](./legends-engine.md) | `FR-CIV-LEGENDS-GRAPH-01..NARRATOR-13` | Emergent history made readable | **Critical path** — the #1 depth-moat |
| [`psyche-social.md`](./psyche-social.md) | `FR-CIV-PSYCHE-001..040` | Emergent mind + relationships | **Critical path** — the "soul"; feeds legends + overlays |
| [`polities-markets.md`](./polities-markets.md) | `FR-CIV-POLITY-001..008`, `FR-CIV-MARKET-001..008` | Emergent society/economy | Depth (depends on psyche/social graph) |
| [`lighting-biomes-art.md`](./lighting-biomes-art.md) | `FR-CIV-RENDER-LIGHT-*`, `-BIOME-PBR`, `-ATMOS-FOG`, `-GRADE-COHESION` | Visual richness ("not a prototype") | Parallel visual track — cheap, high-wow |
| [`civ-ai-crate.md`](./civ-ai-crate.md) | `FR-CIV-AI-001..015` | AI flavor/advisory over substrate | Enabler — narrator/naming/chatter/drift; off hot path |

Supporting inputs: [`competitive-benchmark.md`](../research/competitive-benchmark.md) (gap ranking),
[`backlog.md`](../specs/backlog.md) (Wave 0–6 ladder + remaining BLIND fills not yet design-doc'd),
[`aaa-quality-roadmap.md`](../guides/aaa-quality-roadmap.md) (the S-stage visual ladder M1→S1).

---

## 1. The S0–S6 stage ladder (the spine)

The roadmap rides the project's stage ladder. Each stage is a set of waves; a wave is a parallelizable batch
of work pulled from one or more design docs.

| Stage | Theme | Exit gate | Status |
|---|---|---|---|
| **S0 Foundation** | substrate: voxel-fluid CA, genetics, needs, clusters, economy, watch bus, save-db | substrate ticks; crates exist | ✓ done |
| **S1 Playable slice** | a world you can see/poke: perception layer + sandbox feel + baseline PBR | inspect-anything + overlays + god-tools + non-flat render | ✓ done (Wave 0/1 + M1/P1/A1/L1) |
| **S2 Emergent depth** | the moat: psyche, social graph, legends engine, AI narrator, emergent polities/markets | click any agent → mind+saga; histories readable; polities/markets emerge | **← NEXT** |
| **S3 Content + AAA look** | city verbs + full visual closeout + content breadth | desire-path roads, vehicles, characters/vegetation/GI, full 31-overlay suite, god-tool breadth | |
| **S4 Scale + perf** | 20mi streaming, LOD-tiered agents at 100k, 60fps validated | captured 60fps benchmark at target scale | |
| **S5 Beta** | warfare layers, dual-scale camera, audio juice, onboarding, modding hardening | full RTS↔4X loop; QoL complete | |
| **S6 Ship** | save/replay, polish, balance pass, distribution | shippable | |

**The critical path runs diagonally through the stages, not along them:** legibility (S1) → emergent depth made
legible (S2) → visual polish that makes the depth attractive (S2/S3). Scale and warfare (S4/S5) are *late* because
they are worthless until there is deep, legible, attractive content to scale.

---

## 2. MASTER TABLE — Phase | Wave | Design-doc | Depends-On

Waves are labeled `S{stage}.W{n}`. "Depends-On" lists wave-level predecessors (a wave starts when all its
predecessor waves clear their exit gate). Intra-wave task DAGs live in each design doc and are not repeated here.

| Phase (Stage) | Wave | Design doc(s) · FR block | Scope (one line) | Depends-On |
|---|---|---|---|---|
| **S0 Foundation** ✓ | S0.W1 | (substrate crates) | voxel CA, genetics, needs, cluster/culture/diplomacy, economy, watch bus, save-db | — |
| **S1 Playable slice** ✓ | S1.W1 | info-views P1–P3 · `INFOVIEW-900/902/910..916` | overlay registry + grouped panel + 7 LIVE overlays + inspect-anything | S0.W1 |
| S1 ✓ | S1.W2 | backlog Wave 1 · `GODTOOL-900..921` | god-tool palette, brushes, time controls, undo | S1.W1 |
| S1 ✓ | S1.W3 | aaa-roadmap M1/P1/A1/L1 | PBR materials + post stack (ACES/bloom/SSAO/TAA) + atmosphere + CSM | S0.W1 |
| **S2 Emergent depth** | **S2.W1** | **psyche-social P1–P4** · `PSYCHE-001..031,040` | psyche vector (genetics→drives/temperament, needs→mood, culture→beliefs) + social graph + decay | S1.W1 |
| **S2** | **S2.W2** | **legends-engine P1–P3** · `LEGENDS-GRAPH-01..QUERY-07` | saga graph + ingest/resolution + significance/causality + query API + persist | S0.W1 (watch bus) |
| **S2** | **S2.W3** | **civ-ai P1** · `AI-001..010` | extract `civ-ai` (provider trait, cache, pool, preflight) from `civ-research` | S0.W1 |
| S2 | S2.W4 | psyche-social P5–P6 · `PSYCHE-005/006/011/032/033` | wire psyche→AI/cluster/diplomacy hooks + LOD integration | S2.W1 |
| S2 | S2.W5 | legends-engine P4 + info-views inspect · `LEGENDS-INSPECT-08/BROWSER-09/PRODUCER-12` | inspector saga panel + legends browser + producer event wiring | S2.W2, S2.W1 |
| S2 | S2.W6 | civ-ai P2 · `AI-011..014` + legends `NARRATOR-13` | naming, legends narration, meme-drift, chatter (epoch_digest→SLM) | S2.W3, S2.W2 |
| S2 | S2.W7 | polities-markets P1–P3 · `POLITY-001..004,008`, `MARKET-001..004` | coercion signal + cohesion graph + community detection + tâtonnement + market-type classifier | S2.W4 |
| S2 | S2.W8 | psyche-social P7 + polities-markets §6 + info-views P5 · `PSYCHE-024/034..037`, `INFOVIEW-913/930` | mind/relationship panels + 3 psyche overlays + polity/market read-outs + gated BLIND overlays light up | S2.W4, S2.W7, S2.W5 |
| S2 | S2.W9 | legends-engine P5 · `LEGENDS-PRESIM-10/PERSIST-11/GAP-12` | zero-player pre-sim backstory + persistence hardening + loud-gap detector | S2.W5, S2.W2 |
| **S3 Content + AAA** | S3.W1 | lighting-biomes-art §1–§6 · `RENDER-LIGHT-*/BIOME-PBR/ATMOS-FOG/GRADE-COHESION` | warm-key/cool-fill + day/night curve + per-biome PBR + fog + grade cohesion | S1.W3 |
| S3 | S3.W2 | backlog Wave 3 · `ROAD-900..921`, `INFOVIEW-914` | desire-path roads, emergent architecture, vehicles, infra overlays | S2.W4 |
| S3 | S3.W3 | polities-markets P4 · `POLITY-005..007`, `MARKET-005..008` | secession/merge/collapse + CDA + credit/numeraire | S2.W7 |
| S3 | S3.W4 | aaa-roadmap C1/V1/B1 | humanoid characters + vegetation + building kit (content density) | S3.W1 |
| S3 | S3.W5 | info-views P4 remainder + backlog Wave 1 breadth · `INFOVIEW-917..921` | remaining NEAR overlays (resource/roads/wealth/hazards/migration) + god-tool breadth toward WorldBox bar | S2.W8, S3.W2 |
| S3 | S3.W6 | civ-ai P3 · `AI-015` | headless balance analyst (dev-assist, offline) | S2.W6 |
| **S4 Scale + perf** | S4.W1 | backlog Wave 6 (scale) · `SCALE-900/901/902/910` | 20mi streaming working set + disk format + LOD-tiered agent sim | S2.W4, S3.W2 |
| S4 | S4.W2 | aaa-roadmap G1/U1/S1 + backlog `PERF-900..902` | Solari GI + DLSS/FSR + chunk streaming + 60fps validation | S3.W4, S4.W1 |
| **S5 Beta** | S5.W1 | backlog Wave 5 · `CIV-0101/0300/0105` | strategic↔tactical camera + operational logistics + RTS/direct-control coexist | S4.W1 |
| S5 | S5.W2 | backlog Wave 6 (polish) · `CIV-0800`, `NOTIFY-910..921` | adaptive audio + UI juice + onboarding + rebindable hotkeys + stats dashboards | S2.W8 |
| S5 | S5.W3 | backlog Wave 6 (modding) · `CIV-0700` | sandboxed mod API + share/distribution path hardening | S3.W5 |
| **S6 Ship** | S6.W1 | backlog Wave 6 + `CIV-1000` | save/replay (snapshot), balance pass, final polish, distribution | S5.* |

---

## 3. THE CRITICAL PATH (callout)

```
 S1.W1 ─────────────────────────────────────────────────────────────────────► (legibility root)
 info-views: overlay registry + inspect-anything
   │   "emergence that can't be SEEN is worth zero" (benchmark §5, the #1 gap)
   ▼
 S2.W1 ──► S2.W2 (parallel)        S2.W4 ──► S2.W7 ──► S2.W8
 psyche+social     legends saga    psyche→AI/      polities/    SURFACE: mind+saga+
 graph             graph+causal    cluster/dip     markets      polity/market panels
   │   the "soul"  │   the moat     hooks+LOD       emerge       + overlays light up
   └───────┬───────┘                                                    ▲
           ▼                                                            │
       S2.W5 ──────────────────────────────────────────────────────────┘
       inspector saga panel + legends browser + producer wiring
       (click any entity → its legend; the parity bar)
           │
           ▼  (visual makes the now-legible depth attractive)
       S3.W1  lighting + per-biome PBR + grade cohesion  ──► "not a prototype"
```

**The single critical path, in order, is:**

**`S1.W1 (legibility) → S2.W1+S2.W2 (psyche/social + legends saga graph, in parallel) → S2.W5 (inspect→saga
panel) → S2.W8 (surface the emergent depth in panels+overlays) → S3.W1 (visual closeout makes it attractive)`**

Why this is *the* path (from the competitive benchmark, in priority order):
1. **Legibility first (S1.W1, done).** The #1 credibility gap. It is also a hard prerequisite: per the
   vision-verify principle you cannot validate any sim feature you cannot see. Everything downstream surfaces
   *through* the overlay registry + inspector built here.
2. **Emergent depth, made structural (S2.W1+W2).** Psyche/social graph is the soul; the legends saga graph is
   the moat (the DF "Legends" lesson). These two are independent and run in parallel — psyche feeds the social
   events the legends engine ingests, but the graph machinery does not block on psyche.
3. **Depth made perceptible (S2.W5 → S2.W8).** Inspect-anything → `saga_of` legend panel + the mind/relationship
   panels + the polity/market read-outs. This is where the moat becomes *payoff* rather than *promise*. Until
   this wave lands, the headline claim ("everything emerges") is invisible.
4. **Visual closeout (S3.W1).** Only *after* the depth is legible does the cheap, high-wow PBR/lighting/grade
   work pay off — it makes the now-readable depth *attractive*. Done earlier it polishes an empty world; done
   here it is the frame around a living one. (It can *start* in parallel after S1.W3 since it shares no files
   with the sim track — see §4 — but on the critical path it is sequenced after the depth is legible.)

**Not on the critical path (deliberately late):** scale/streaming (S4), GI/DLSS (S4), warfare layers (S5),
audio/onboarding (S5). Each is worthless until there is deep, legible, attractive content to scale, light, and
fight over. Scaling an invisible world to 100k agents closes no benchmark gap.

---

## 4. Parallelizable waves (what runs concurrently)

Disjoint file ownership lets these batches run as concurrent subagent fleets:

- **The two S2 tracks are independent and run in parallel:**
  - *Sim-depth track:* S2.W1 (psyche/social) → S2.W4 → S2.W7 (`crates/agents`, `crates/economy`).
  - *History track:* S2.W2 (legends) → S2.W5 (new `crates/legends` + inspector) — depends only on the watch bus.
  - *AI track:* S2.W3 (civ-ai extraction) → S2.W6 (`crates/ai` + feature services) — depends only on `civ-research`.
  These three touch disjoint crates and can fan out as three concurrent subagent fleets the moment S1 clears.
- **The visual track (S3.W1, lighting-biomes-art) shares no files with any sim track** (`clients/bevy-ref`
  rendering files only). It can launch in parallel immediately after S1.W3 and merge whenever ready — it is
  sequenced *on the critical path* after depth-is-legible only for *attention/ROI* ordering, not for a code
  dependency. If subagent budget allows, run it concurrently with the S2 sim tracks.
- **Within S3:** roads (S3.W2), polity lifecycle (S3.W3), characters/vegetation (S3.W4), and balance analyst
  (S3.W6) are mutually independent given their S2 predecessors.
- **The info-view BLIND overlays are pre-registered (S1.W1) and light up automatically** as their producing
  fields land (psyche overlays at S2.W8, polity/market at S2.W8, hazards/roads at S3.W2/W5) — no overlay rework,
  so the "light it up" work folds into whichever sim wave surfaces the field.

**Concurrency ceiling for S2:** ~3 fleets (sim-depth, history, AI) + optionally the visual track = up to 4
parallel batches, each a multi-subagent wave per the design-doc internal WBS.

---

## 5. The order that closes competitive-benchmark gaps fastest

Mapping each behind-axis from the benchmark §4 to the wave that closes it, ordered by the benchmark's own
priority ("close legibility first, PBR/lighting second, emergent histories/psyche third"):

| # | Benchmark gap (leader) | Closing wave(s) | Stage |
|---|---|---|---|
| 1 | **UX/UI legibility** (CS2 ~33 overlays + inspect) | S1.W1 (core) → S2.W8 → S3.W5 (full 31-overlay suite) | S1→S3 |
| 2 | **Emergence: histories** (Dwarf Fortress Legends) | S2.W2 → S2.W5 → S2.W9 (pre-sim backstory) | S2 |
| 2b | **Emergence: psyche/social** (RimWorld/DF) | S2.W1 → S2.W4 → S2.W8 | S2 |
| 2c | **Emergence: polities/markets** (Victoria 3) | S2.W7 → S3.W3 | S2→S3 |
| 3 | **Visual fidelity** (Manor Lords/Frostpunk) | S3.W1 (cheap closeout) → S3.W4 → S4.W2 (GI/DLSS) | S3→S4 |
| 4 | **Content breadth** (WorldBox 374 powers) | S3.W2 (city verbs) + S3.W5 (god-tool breadth) | S3 |
| 5 | **Scale** (Songs of Syx 50k) | S4.W1 → S4.W2 (validated 60fps) | S4 |
| 6 | **Agent AI depth** (RimWorld) | folded into S2.W4 (psyche biases utility-AI) | S2 |
| 7 | **Moddability** (near-par; protect) | S5.W3 (hardening only) | S5 |

The sequence front-loads the two gaps the benchmark calls the worst (legibility, then emergent depth made
readable), defers visual parity to S3 (explicitly *not* the first gap and never the gap to win on), and pushes
scale/perf to S4 where it has something worth scaling.

---

## 6. Next 3 waves to execute

All three S2 entry waves are unblocked the moment S1 is accepted, touch disjoint crates, and can launch as a
single concurrent 3-fleet batch:

1. **S2.W1 — Psyche + social graph (psyche-social P1–P4).** Owner: Life-Sim. New `crates/agents/src/psyche.rs`
   + `social.rs`. Birth-time genetics→drives/temperament (reuse `sentience` reducer), needs→mood, culture→beliefs
   via graph, interaction-accumulated/decay ties. Gate: AC-1..AC-5. The soul; emits the social events the legends
   engine ingests. ~3–4 parallel subagents.
2. **S2.W2 — Legends saga graph (legends-engine P1–P3).** Owner: History. New `crates/legends` over `petgraph`
   `StableDiGraph` + the existing `crates/watch` bus + `crates/save-db`. Ingest/resolution → significance/causality
   → read-only query API + `EpochDigest`. Gate: AC-SIG-1..3, AC-Q-1..3. The moat data structure; off the sim hot
   path. ~3 parallel subagents.
3. **S2.W3 — `civ-ai` extraction (civ-ai P1).** Owner: AI/ML. Extract `AiProvider` trait + cache + worker pool +
   preflight from `civ-research` into `crates/ai`; `civ-research` becomes a consumer. Gate: FR-CIV-AI-001..010
   acceptance signals; sim never awaits a token. Unblocks the narrator/naming/chatter services (S2.W6) that make
   both psyche and legends *speak*. ~3 parallel subagents.

After this batch clears, the next convergence wave is **S2.W5** (inspect→saga panel — the legibility parity bar
that makes the depth perceptible), fed by W1 and W2.

---

## 7. Cross-project reuse opportunities (Phenotype org)

Per the Cross-Project Reuse Protocol (confirm destination with user before any cross-repo extraction):
- **`phenotype-history`** — the generic entity/event/causal-DAG-over-an-event-stream core of `legends-engine`
  (minus the Civis `EventKind` registry). Reusable by WSM3D/DINOForge. Flagged in legends-engine §11.
- **`phenotype-ai`** — the `AiProvider` trait + registry + cache + worker pool + preflight from `civ-ai`
  (domain-agnostic). Flagged in civ-ai-crate §9. Lift only after the 5 features prove it in-repo.
- **`phenotype-ui`** — `ramp_color` + `LegendStop` + `cluster_color` overlay/legend primitives from info-views.
  Flagged in info-views §10.
- **Visual:** the KTX2/PolyHaven importer, DLSS/FSR wrapper, and atmosphere+day/night coupling from the
  aaa-roadmap are `phenotype-assets`/`phenotype-upscale`/`phenotype-sky` candidates.

All cross-repo moves are forward-only (extract → update callers → remove duplicate) and require user
confirmation on destination + rollout before execution.
