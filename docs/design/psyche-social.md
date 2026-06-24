# Civis Psyche + Social-Graph Layer — Emergent Mind & Relationship Spec

> **Status:** Design spec (2026-05-30). Owned by Design (Planner stance — specs / AC / pseudocode only, **no implementation code**).
> Governed by [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md): **only physical / environmental / genomic laws are authored; psyche, beliefs, and relationships EMERGE.** This layer hardcodes *update rules over the substrate*, never the outcomes (no `personality: enum`, no `friend_of: u32` authored by hand).
> Builds on the existing crates — `civ-needs` (`crates/needs`), `civ-agents` (`crates/agents`: `cluster.rs`, `culture.rs`, `diplomacy.rs`, `daily_path.rs`), `civ-genetics` (`crates/genetics`). Surfaces through the inspector / overlay suite ([`docs/info-views.md`](../info-views.md)).
> Traceability: **FR-CIV-PSYCHE-001 … FR-CIV-PSYCHE-040**.

---

## 1. Thesis: the mind as a measured field, not an authored enum

Civis already simulates **survival needs** (`civ-needs`: food / water / rest / safety / social / health), **emergent clusters** (`cluster.rs`), **culture/language drift** (`culture.rs`), and **inter-cluster diplomacy** (`diplomacy.rs`). What is missing is the *individual interior*: **why two agents with the same needs make different choices**, and **who each agent trusts, loves, or resents**.

This layer adds two emergent structures and wires them into the *existing* utility-AI / cluster / diplomacy machinery — it does **not** replace them:

1. **Psyche vector** — a per-agent interior state with four sub-layers, each EMERGING from an authored substrate channel:
   - **Drives** — *what an agent chronically wants* (stable, lifelong) — emerge from **genetics** (DNA byte slots, same pattern as `sentience.rs` cognition traits).
   - **Temperament** — *how an agent reacts* (reactivity, sociability, risk, impulsivity) — emerge from **genetics** modulated slowly by lived experience.
   - **Mood** — *transient affect* (valence / arousal) — emerges from the live **needs** vector + recent social events; fast-moving.
   - **Beliefs** — *what an agent holds true / values* — emerge from **culture** (`CultureProfile`) sampled through the agent's social graph; drift over life.

2. **Social graph** — a per-agent directed, weighted relationship set (kinship / familiarity / affinity / trust / rivalry) that **accumulates from interaction events** and **decays without contact**. Clusters and diplomacy are *aggregates* of this graph, not parallel authored state.

**Charter test applied to every field below:** *can this emerge from Layer-0 rules?* Yes — drives/temperament from genomics, mood from needs, beliefs from culture-over-graph, relationships from interaction history. Nothing here is a fixed taxonomy of personalities or a scripted friendship.

---

## 2. Data model

All types are **engine-free POD** (`Serialize`/`Deserialize`, `Copy` where small), to live as `hecs` components alongside `Civilian`, `Needs`, `ClusterMember`, mirroring the existing `civ-agents` style. New module split: `crates/agents/src/psyche.rs` and `crates/agents/src/social.rs` (planner-proposed; implementation is a later WBS item).

### 2.1 Psyche vector

```text
PSYCHE_DIM = 4   // width of each sub-vector; matches culture::TraitVector for cheap mixing

struct Psyche {
    drives:      [f32; 4],   // EMERGENT-from-genetics, ~immutable post-maturity. [0,1]
    temperament: Temperament,// EMERGENT-from-genetics + slow experiential nudge
    mood:        Mood,       // EMERGENT-from-needs + events, fast
    beliefs:     [f32; 4],   // EMERGENT-from-culture-via-graph, lifelong drift. [0,1]
    maturity:    f32,        // 0=infant..1=adult; gates how plastic each layer is
}
```

- **`drives[4]`** — generic axes (intentionally *un-named in code*, like `culture::TraitVector`): the meaning emerges from which behaviours they end up biasing. For *readability only* the inspector labels them by the behaviour they most correlate with at runtime (see §6), e.g. provision / security / affiliation / novelty. They are NOT authored as those concepts.

```text
struct Temperament {
    reactivity:  f32,  // how strongly mood swings per event       [0,1]
    sociability: f32,  // baseline pull toward Social need + ties   [0,1]
    risk_tol:    f32,  // willingness to accept safety cost         [0,1]
    impulsivity: f32,  // discount on distance/effort in planning   [0,1]
}

struct Mood {
    valence: f32,   // -1 misery .. +1 contentment
    arousal: f32,   //  0 calm   ..  1 agitated
}
```

### 2.2 Genetic → psyche mapping (authored = the *mapping rule*, not the values)

Reuse the `civ-genetics` slot-weight pattern (`sentience::CognitionTraitProfile`) verbatim — a `PsychGenomeProfile` declares DNA byte indices → axis. The **values** come from each agent's evolving DNA; only the *projection* is authored, exactly as cognition scoring is.

```text
struct PsychGenomeProfile {           // data-driven, mod-friendly (like DnaClass)
    drive_slots:       [Vec<(usize, f32)>; 4],  // byte idx + weight per drive axis
    reactivity_slots:  Vec<(usize, f32)>,
    sociability_slots: Vec<(usize, f32)>,
    risk_slots:        Vec<(usize, f32)>,
    impulsivity_slots: Vec<(usize, f32)>,
}

// score_axis(dna, slots) == genetics::sentience cognition reducer:
//   sum(byte/255 * weight) / sum(weight)  -> [0,1]
```

### 2.3 Social graph

A relationship is **directed** (A's view of B ≠ B's view of A) and **sparse**. Each agent owns a bounded adjacency list (cap `MAX_TIES`, default 150 — a Dunbar bound that keeps per-agent cost O(1)); weakest ties evict when full.

```text
struct Tie {
    other:       u64,   // target agent id
    kinship:     f32,   // [0,1] genetic relatedness, set at birth, immutable
    familiarity: f32,   // [0,1] how well-known; grows w/ contact, slow decay
    affinity:    f32,   // [-1,1] like..dislike (love/friendship vs rivalry)
    trust:       f32,   // [-1,1] reliability belief; moves on kept/broken cooperation
    last_seen:   u32,   // tick of last interaction (drives decay)
}

struct SocialGraph {            // hecs component, per agent
    ties: Vec<Tie>,             // sorted by `other` for binary-search merge
}
```

**Relation read-out (emergent label, not stored kind):** a pure function classifies a tie at query time — never an authored field:

```text
fn relation_label(t: &Tie) -> RelationLabel
//  kin & affinity>0            => Family
//  affinity> KIN_HI & trust>0  => CloseFriend / Partner (by valence+familiarity)
//  affinity in mid band        => Acquaintance
//  affinity<0 & trust<0        => Rival
//  affinity< -RIVAL_HI         => Enemy
// Thresholds are tunables, NOT a taxonomy of who-must-be-what.
```

### 2.4 Interaction event (the only thing that mutates the graph)

```text
enum Interaction {
    Cooperated{ benefit: f32 },   // shared food, built together, defended
    Competed{ pressure: f32 },    // contested POI / resource
    Defected{ harm: f32 },        // took, betrayed, attacked
    Coexisted,                    // mere co-location this tick (weak +)
    Kin,                          // birth/family link (sets kinship once)
}

struct SocialEvent { a: u64, b: u64, kind: Interaction, tick: u32 }
```

Events are produced by *existing* systems — `daily_path` POI contention → `Competed`; `cluster` co-membership → `Coexisted`/`Cooperated`; combat/theft (tactics/economy) → `Defected`; genetics reproduction → `Kin`. **No new authored social scripts.**

---

## 3. Update rules (the authored "laws of the mind")

All updates are pure functions `(state, inputs, dt) -> state` so they slot into the LOD tick (`Hot`/`Warm`/`Cold`) and respect the charter's "real randomness is welcome" (per `feedback_civis_no_determinism`): jitter via `thread_rng` is allowed; no determinism gate required.

### 3.1 Drives & temperament (genetics-anchored)
- **At birth / speciation:** `drives`, `temperament` ← `score_axis(dna, profile)` for every axis (**FR-CIV-PSYCHE-001/002**). Recombination + mutation (`genetics::recombine/mutate`) already give offspring blended-then-drifted DNA → offspring psyche resembles parents but varies. No authored inheritance of personality; it falls out of DNA.
- **Lifelong plasticity (small):** temperament nudges toward the statistics of lived experience, gated by `(1 - maturity*0.8)` so children are plastic, adults stable (**FR-CIV-PSYCHE-003**):
  ```text
  reactivity += LR_T * (recent_mood_variance - reactivity) * plasticity
  sociability += LR_T * (recent_social_satisfaction - sociability) * plasticity
  ```
  `LR_T` tiny (≈0.002/day). Drives are effectively frozen post-maturity.

### 3.2 Mood (needs-driven, fast)
Mood is the agent's felt summary of its needs plus a decaying memory of social events (**FR-CIV-PSYCHE-010**):
```text
need_valence = weighted_mean(needs - 0.5) over 6 needs          // sated→+, deprived→-
event_term   = decayed_sum(recent SocialEvent affinity deltas)  // half-life ~1 day
target_val   = clamp(need_valence + EVENT_W * event_term, -1, 1)
mood.valence += (target_val - mood.valence) * (MOOD_LR * (0.5 + temperament.reactivity))
mood.arousal  = clamp(threat_pressure + |Δneeds| + EVENT_W*|event_term|, 0, 1)
```
Reactive agents (high `reactivity`) converge faster and overshoot → emergent volatile vs stoic temperaments **without** authoring either.

### 3.3 Beliefs (culture-via-graph)
Beliefs are the agent's personal sample of the culture it is *exposed to through its strongest ties*, not the cluster average (**FR-CIV-PSYCHE-020**). This makes belief sub-populations and heterodoxy emerge inside one cluster.
```text
exposure = Σ_ties w_i * culture_of(other_i),  w_i ∝ familiarity_i * max(affinity_i,0)
beliefs  = mix(beliefs, exposure, BELIEF_LR * plasticity)   // culture::mix_trait_vectors
beliefs  = mutate_traits(beliefs, BELIEF_JITTER)            // reuse culture.rs mutator
```
Conformity pressure scales with `sociability`; contrarians (low sociability, low trust) drift away → schisms emerge. Feeds *back* into `culture.rs` aggregation (cluster culture = belief centroid weighted by influence), closing the loop instead of duplicating state.

### 3.4 Social graph (interaction-accumulated, contact-decayed)
Per `SocialEvent` apply a bounded delta then renormalise (**FR-CIV-PSYCHE-030**):
```text
match kind {
  Cooperated{benefit} => affinity += A_COOP*benefit;  trust += T_COOP*benefit;
  Competed{pressure}  => affinity -= A_COMP*pressure;
  Defected{harm}      => affinity -= A_DEF*harm;       trust -= T_DEF*harm;     // sharp
  Coexisted           => familiarity += F_FAM;         affinity += A_COEX;      // weak
  Kin                 => kinship = relatedness();       // once
}
familiarity = clamp01(familiarity + F_FAM_ON_ANY_CONTACT)
last_seen = tick;  clamp affinity/trust to [-1,1]
```
**Decay each Warm/Cold tick for ties not seen recently** (**FR-CIV-PSYCHE-031**):
```text
gap = tick - last_seen
familiarity *= DECAY_FAM^gap         // forget acquaintances
affinity    -> pull toward 0 by DECAY_AFF*gap   // kinship is exempt (kin stays kin)
trust       -> pull toward 0 by DECAY_TRUST*gap
```
**Eviction:** when `ties.len() > MAX_TIES`, drop the tie minimising `familiarity + |affinity| + kinship` (keep the meaningful, shed the forgotten).

Asymmetry is intrinsic: A may adore B while B is indifferent — emergent unrequited bonds, hero-worship, one-sided rivalry.

---

## 4. How psyche + social drive the existing systems

This layer's value is that it **modulates** machinery already in `civ-agents`, via small, surgical hooks — each a pure multiplier/bias, charter-safe.

### 4.1 Utility-AI / daily path (`daily_path.rs`) — **FR-CIV-PSYCHE-011**
`score_poi` today = `need_pressure - DISTANCE_WEIGHT*dist`. Psyche injects three biases:
```text
score = need_pressure * drive_gain(need_kind, psyche.drives)        // drives weight needs
      - DISTANCE_WEIGHT * dist * (1 - 0.5*temperament.impulsivity)   // impulsive ignore distance
      + social_pull(poi, social_graph) * temperament.sociability     // go where my ties are
      - mood_risk_term(poi, psyche.mood, temperament.risk_tol)       // anxious avoid SafeZone-low
```
- `drive_gain`: an agent with a high "affiliation" drive over-weights the Social need → emergent extroverts; high "provision" → workaholic foragers. Same needs, different routines.
- `social_pull`: a `SocialHub`/POI co-occupied by high-affinity ties scores higher → friends congregate, families co-locate **(emergent, from the graph)**.
- This is the same greedy selector — only the score is enriched, so the existing eat→rest→socialize routine still holds.

### 4.2 Cluster membership (`cluster.rs`) — **FR-CIV-PSYCHE-032**
`should_join`/`should_leave` consume a `MembershipPayoff`. Provide a `SocialPayoff` impl whose payoff = mean(affinity·trust over my ties already in cluster) + belief-similarity bonus − rivalry penalty. Result: clusters coalesce along **friendship + shared belief**, fracture along **rivalry + schism** — exactly the charter's "membership is emergent cluster overlap." Co-location (existing `cluster_by_colocation`) seeds candidates; psyche/social decides whether the bond *holds*.

### 4.3 Diplomacy (`diplomacy.rs`) — **FR-CIV-PSYCHE-033**
Cluster-pair `DiplomacySignal` is currently `resource_competition/trade_volume/proximity`. Add a **social bridge term**: aggregate cross-cluster ties — many positive inter-cluster affinities → bias `trade_volume` up; dense inter-cluster rivalry/defection → bias `resource_competition` up. Leaders/high-degree nodes weight more (degree centrality from the graph). Inter-group relations thus **emerge from individual relationships**, not from a faction matrix. Existing `RelationKind` thresholds and EMA folding are untouched.

### 4.4 Culture feedback (`culture.rs`) — **FR-CIV-PSYCHE-021**
Cluster `CultureProfile.kinship` (insulation) is set from mean intra-cluster kinship+familiarity; `contact` from cross-cluster tie density. The belief loop (§3.3) supplies per-agent variation that aggregates into `traits`. The two language/culture drift systems now have a *source* rather than abstract scalars.

---

## 5. Tick integration & cost (LOD-tiered)

Respects the existing `LodTier::{Hot, Warm, Cold}` budget:

| Tier | Mood | Beliefs | Social events | Graph decay | Drives/Temp |
|------|------|---------|---------------|-------------|-------------|
| Hot  | every tick | every N ticks | full per-event | per tick | birth + slow |
| Warm | every N ticks | coarse | batched | periodic | birth only |
| Cold | statistical (cluster-mean mood) | frozen | aggregated as cluster stats | lazy on promote | birth only |

- Per-agent cost is O(MAX_TIES) bounded; decay is amortised (touch on access / periodic sweep). **FR-CIV-PSYCHE-005**.
- Cold agents collapse to cluster-level aggregates (mean mood, belief centroid, intra/inter tie density) — no per-tie sim far from camera, matching the agent-LOD charter rule. Promotion rehydrates ties lazily from a compact summary. **FR-CIV-PSYCHE-006**.

---

## 6. Readable surface — "see the mind & the bonds"

Per the legibility thesis ([`docs/info-views.md`](../info-views.md)), emergence that can't be *seen* is worth zero. Two surfaces:

### 6.1 Agent inspector — "Mind" panel (**FR-CIV-PSYCHE-024**)
Click any agent → panel sections:
- **Mood:** valence/arousal as a 2-axis dot ("content & calm" … "miserable & agitated"), color = valence ramp; sparkline of recent valence (`egui_plot`, already a dep).
- **Drives:** 4 horizontal bars, each **runtime-labelled** by its strongest behavioural correlation this session (provision / security / affiliation / novelty) with an "emergent — derived from genome" footnote. Honest about the charter: the label is a *description of observed behaviour*, not an authored class.
- **Temperament:** radar/4-bar (reactivity / sociability / risk / impulsivity).
- **Beliefs:** the 4-axis belief vector vs the agent's cluster centroid (shows heterodoxy at a glance — a contrarian visibly diverges).
- **One-line read-out:** an auto-generated plain-language summary, e.g. *"Anxious, sociable forager; devout by local standards; volatile mood (hungry)."* Composed from thresholds, not authored bios.

### 6.2 Relationships panel + graph (**FR-CIV-PSYCHE-034**)
- **Tie list:** sorted by salience (`familiarity+|affinity|+kinship`), each row = other agent, `relation_label` (Family/Friend/Partner/Acquaintance/Rival/Enemy — *derived*, §2.3), and trust/affinity mini-bars. Click → jump to that agent.
- **Ego-graph mini-map:** the inspected agent at center, ties as edges (green=affinity+, red=rivalry, thickness=familiarity, gold=kin). Reuses the gizmo render-kind from the overlay suite.
- **World overlays (register into `info_views.rs`, charter-safe categorical):**
  - **Social ties** overlay (Gizmo): draw strong edges between nearby agents — friendship networks become visible terrain. **FR-CIV-PSYCHE-035**.
  - **Mood / contentment** overlay (EntityTint): tint agents by mood valence — see a happy district vs a miserable one. **FR-CIV-PSYCHE-036**.
  - **Belief field** overlay (LatticeRecolor over belief centroid): cultural/ideological regions and their boundaries, derived from per-agent beliefs (the "where do worldviews split" map). **FR-CIV-PSYCHE-037**.
  All three are *measurements*, gated behind the §7-style availability flag until the producing fields land.

---

## 7. Acceptance criteria (spec-level)

- **AC-1 (genetics→psyche):** Two siblings (recombined DNA) have correlated but non-identical drives/temperament; two unrelated agents are uncorrelated. *(FR-CIV-PSYCHE-001/002)*
- **AC-2 (mood tracks needs):** Driving any need to 0 pushes `mood.valence` negative within a bounded window; satiation recovers it. High-reactivity agents move faster. *(FR-CIV-PSYCHE-010)*
- **AC-3 (beliefs drift from graph, not cluster):** An agent whose strong ties hold a minority belief drifts toward the minority, diverging from its cluster centroid (heterodoxy emerges). *(FR-CIV-PSYCHE-020)*
- **AC-4 (relationships accumulate & decay):** Repeated `Cooperated` raises affinity+trust; a `Defected` sharply drops trust; absence decays familiarity/affinity toward 0 while kinship persists. *(FR-CIV-PSYCHE-030/031)*
- **AC-5 (asymmetry):** A→B and B→A ties can differ in sign and magnitude. *(FR-CIV-PSYCHE-030)*
- **AC-6 (drives the AI):** Same `Needs` snapshot + different drives ⇒ different `score_poi` ranking; friends co-locate via `social_pull`. *(FR-CIV-PSYCHE-011)*
- **AC-7 (drives clusters):** A cluster splits when intra-cluster rivalry/belief-distance exceeds the join payoff; coalesces on shared belief+affinity. *(FR-CIV-PSYCHE-032)*
- **AC-8 (drives diplomacy):** Dense positive cross-cluster ties move the pair toward Trade/Alliance; dense defection toward Rivalry/War, via the social-bridge term. *(FR-CIV-PSYCHE-033)*
- **AC-9 (no authored taxonomy):** No `enum Personality`, no authored friendship/belief; every label is a pure function of emergent state. Grep proves it. *(charter)*
- **AC-10 (LOD-bounded):** Per-agent cost O(MAX_TIES); Cold agents run cluster-aggregate only; promotion rehydrates. *(FR-CIV-PSYCHE-005/006)*
- **AC-11 (legible):** Inspector renders mind + relationships; 3 world overlays register without core changes; each carries an "emergent — derived" provenance note. *(FR-CIV-PSYCHE-024/034)*

---

## 8. FR catalog

| FR | Title |
|----|-------|
| FR-CIV-PSYCHE-001 | Drives emerge from DNA via `PsychGenomeProfile` slot-weights |
| FR-CIV-PSYCHE-002 | Temperament emerges from DNA (reactivity/sociability/risk/impulsivity) |
| FR-CIV-PSYCHE-003 | Lifelong temperament plasticity gated by maturity |
| FR-CIV-PSYCHE-005 | Per-agent psyche+social cost bounded O(MAX_TIES) |
| FR-CIV-PSYCHE-006 | Cold-LOD collapse to cluster aggregates + lazy rehydrate |
| FR-CIV-PSYCHE-010 | Mood emerges from needs vector + decayed event memory |
| FR-CIV-PSYCHE-011 | Psyche biases `score_poi` (drives/impulsivity/sociability/risk) |
| FR-CIV-PSYCHE-020 | Beliefs emerge from culture sampled via the social graph |
| FR-CIV-PSYCHE-021 | Belief loop feeds back into `culture.rs` aggregation |
| FR-CIV-PSYCHE-024 | Inspector "Mind" panel (mood/drives/temperament/beliefs/read-out) |
| FR-CIV-PSYCHE-030 | Directed weighted ties accumulate from interaction events |
| FR-CIV-PSYCHE-031 | Contact decay of familiarity/affinity/trust; kinship exempt; eviction |
| FR-CIV-PSYCHE-032 | Social payoff drives cluster join/leave |
| FR-CIV-PSYCHE-033 | Social bridge term drives inter-cluster diplomacy |
| FR-CIV-PSYCHE-034 | Inspector relationships panel + ego-graph |
| FR-CIV-PSYCHE-035 | Social-ties world overlay (Gizmo) |
| FR-CIV-PSYCHE-036 | Mood/contentment world overlay (EntityTint) |
| FR-CIV-PSYCHE-037 | Belief-field world overlay (LatticeRecolor) |
| FR-CIV-PSYCHE-040 | `relation_label` derives relation kind at query time (no stored taxonomy) |

---

## 9. WBS (implementation breakdown — *not done here*)

1. **P1 — types & genetics map.** `psyche.rs` (Psyche/Temperament/Mood/PsychGenomeProfile), `social.rs` (Tie/SocialGraph/Interaction). Birth-time `score_axis` reusing `sentience` reducer. Tests: AC-1. *(FR-001/002)*
2. **P2 — mood + drives update.** Pure update fns + recent-event ring buffer. Tests: AC-2. *(FR-010)*
3. **P3 — social graph mutation + decay + eviction.** Event apply, sorted-merge ties, decay sweep. Tests: AC-4/5. *(FR-030/031, 040)*
4. **P4 — belief loop.** Graph-weighted exposure mix via `culture::mix_trait_vectors`; feedback into culture aggregation. Tests: AC-3. *(FR-020/021)*
5. **P5 — AI/cluster/diplomacy hooks.** `score_poi` bias, `SocialPayoff`, diplomacy social-bridge term. Tests: AC-6/7/8. *(FR-011/032/033)*
6. **P6 — LOD integration.** Hot/Warm/Cold scheduling, Cold aggregate collapse + rehydrate. Tests: AC-10. *(FR-005/006)*
7. **P7 — inspector + overlays.** Mind panel, relationships/ego-graph, 3 overlay registrations in `info_views.rs`. Tests: AC-11. *(FR-024/034/035/036/037)*

Each work item: disjoint files, xDD-first (tests/AC before impl), wraps existing `culture.rs`/`genetics` helpers rather than hand-rolling (per the quality charter).
