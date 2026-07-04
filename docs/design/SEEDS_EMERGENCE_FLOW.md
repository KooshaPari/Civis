# Seeds → Emergence — How the Canonical Seeds Feed the Emergence Phases

> **Status:** Design (Planner) — per-layer data flow of the **canonical seeds** (named races + raw-organism primitive + `0..1` divergence dial) into the **emergence phases** (species / society / language / religion / economy) without hardcoding outcomes.
> **Governing canon:** [`CONTENT_SEEDS.md`](CONTENT_SEEDS.md) (the 2-layer content model — seeds above substrate, substrate above emergence), [`emergent-systems-spec.md §1.2`](emergent-systems-spec.md) (layering contract), [`emergence-charter.md`](../guides/emergence-charter.md) (only physics / environment / genome are authored; everything else emerges).
> **Sister docs:** [`species-sentience.md`](species-sentience.md) (DNA → phenotype → sentience), [`psyche-social.md`](psyche-social.md) (mind + relationship emergence), [`civ-culture-emergent.md`](civ-culture-emergent.md) (culture/language drift), [`polities-markets.md`](polities-markets.md) and [`civ-economy-emergent-markets.md`](civ-economy-emergent-markets.md) (institutional emergence).
> **FR families:** `FR-CIV-GENETICS-*`, `FR-CIV-SPECIES-*`, `FR-CIV-AGENTS-*`, `FR-CIV-LEGENDS-*`, `FR-CIV-FACTIONS-*`, `FR-ECON-*`, `FR-CONTENT-MODEL`, `FR-CONTENT-SEEDMIX`.
> **Owner crates:** `civ-genetics` (substrate + seed schema), `civ-species` (DNA → phenotype), `civ-agents` (psyche / social / culture / diplomacy), `civ-legends` (saga graph), `civ-economy` (markets / polities), `civ-engine` (phase orchestration, `select_seed_for_position`, `choose_named_seed`).
>
> **Read this doc together with [`CONTENT_SEEDS.md §6`](CONTENT_SEEDS.md#6-how-seeds-feed-emergence-and-where-they-stop).** That section enumerates the *only* legal touchpoints a seed has with runtime state; this doc walks layer-by-layer through **how** those touchpoints flow data into each emergence phase and what each phase then does with the genome (and explicitly does **not** do with the seed).

---

## 1. Thesis — a seed is a nudge, never an outcome

The 2-layer content model defines two layers; this doc defines the bridge between them.

```
LAYER 1 — canonical seeds (authored content)
   • SeedDefinition { id, genome, divergence, spawn_biome_affinity, … }
   • NamedSeed enum (Ardani / Velthari / Grundak — code-level archetypes)
   • raw_organism primitive
   • 0..1 divergence dial
              │  spawn_genome, mutate_with_divergence,
              │  select_seed_for_position, choose_named_seed
              ▼
LAYER 0 — algorithmic substrate (no authored content)
   • Dna (Vec<u8>), DnaClass { length, mutation_rate, speciation_threshold }
   • mutate, recombine, speciation_distance, fitness
   • express() (DNA → Phenotype) in civ-species
              │  emerge
              ▼
EMERGENCE PHASES (no authored outcomes)
   • species  — DNA → phenotype → speciation events
   • society  — cluster formation, social graph, diplomacy
   • language — culture + language drift through contact
   • religion — psyche beliefs, legends, sentience-awakening pulse
   • economy  — market geometry, polity emergence, trade routes
```

**The single rule that this doc is auditing:** *every arrow above is **one-way** and **decoupled***. The seed touches the genome; the genome touches emergence; emergence never reads the seed. The seed **does not** say "this is a sea-faring empire" — it provides a genome distribution, and the environment + agents + RNG decide what comes of it.

The remainder of this document walks the five layers, showing for each one:

1. The **data path** — which substrate field the seed feeds, and through which crate/function.
2. The **emergence update rule** — what the layer does with the genome (deterministic, seed-agnostic).
3. The **explicit non-control** — what the seed does *not* set on this layer.
4. **Testable invariants** that pin the rule in place.

---

## 2. Layer order in the engine

`Simulation::tick` runs the emergence phases in a fixed order in
`crates/engine/src/emergence.rs::phase_emergence` (`emergence.rs:159-171`):

```
1. emergence_ensure_genomes       — backfill Dna for any spawned agent missing one
2. emergence_culture               — per-cluster CultureProfile drift
3. emergence_social                — per-cluster pair interactions, social-graph decay
4. emergence_psyche                — derive / update psyche from Dna + culture + needs
5. emergence_genetics_sentience    — evaluate sentience threshold, mint awakenings
6. emergence_legends               — ingest birth/death/sentience into saga graph
7. emergence_civ_ai                — civ-ai flavor decisions on promotions (naming)
```

**The order is not a pipeline of hardcoded outcomes.** Each phase reads the substrate, updates its own state, and emits feed events. Phase 2 has no idea phase 1 decided a genome came from `human_baseline`; phase 5 has no idea phase 2's culture has a particular drift. They are coupled only through the genome and through `cluster_cultures` (which is itself a measured pattern, not authored).

The data flow per layer in the rest of this doc is anchored against this order.

---

## 3. Layer 1 — Species (DNA → Phenotype → Speciation)

Species is the **most direct** layer: the seed's `genome: Vec<u8>` is *the* input. There is no authoring in between; `express()` is a pure function.

### 3.1 Data path

```
SeedDefinition
  │   (loaded via SeedLibrary::from_ron_str or archetype_seed())
  │
  ▼
spawn_genome_with_divergence(rng, class, seed, divergence)        seeds.rs:270
  │   — clones seed.genome
  │   — applies mutate_with_divergence(dna, rng, class, divergence)
  │   — if divergence = 0.0 → genome is clamped to seed bytes
  │   — if divergence = 1.0 → per-byte point mutation at full class rate
  ▼
Dna(Vec<u8>)                                                       civ-genetics
  │   (stored as hecs component on the Civilian entity)
  │
  ▼
express(&dna) → Phenotype { morphology, behavior }                civ-species/src/lib.rs:76
  │   — bytes 0..4 → morphology (height, hue, legs, arms, eyes)
  │   — bytes 5..8 → BehaviorWeights (aggression, curiosity, sociability, intelligence)
  │   — bytes 9..  → reserved / substrate-internal (ignored by express)
  ▼
Phenotype                                                             civ-species
  │   — drives civ-agents utility-AI weighting
  │   — feeds the renderer (skin colour, body plan)
  ▼
speciation events                                                     civ-genetics
  │   — when normalised Hamming distance > DnaClass.speciation_threshold
  │   — emits SpeciationEvent (consumed by legends + civ-ai)
```

**Per-tick drift:** `mutate_with_divergence` is the **canonical** per-tick call site; the divergence dial scales the class's `mutation_rate` per byte. With `divergence = 0.0` the genome is **clamped** — `mutate` becomes a no-op. With `divergence = 1.0` the per-tick per-byte flip rate is `class.mutation_rate` (default `0.01`).

### 3.2 The emergence update rule

The species layer is **fully substrate-driven**; emergence happens through two substrates:

1. **Phenotype plasticity** — the `express()` mapping means the same `Dna` always produces the same `Phenotype`; as DNA drifts, phenotypes drift. A `0.0`-divergence race has a *clamped* genome but recombination with another clamped genome (crossover) still produces byte-level variation, so offspring still look different from parents. This is the substrate's own, not the seed's.
2. **Speciation** — `speciation_distance` compares two genomes; when distance > `DnaClass.speciation_threshold`, they are tagged as distinct species. The threshold is a **substrate parameter**, not a seed parameter; the seed *paces* the rate at which new species form but does not *guarantee* or *prevent* speciation.

### 3.3 What the seed does NOT control

| Concern | Owned by | Why the seed can't control it |
|---------|----------|-------------------------------|
| Phenotype interpretation | `civ-species::express()` | Pure 9-byte layout; the seed only sets the bytes. |
| Speciation threshold | `DnaClass.speciation_threshold` | Substrate-level parameter; the seed does not override it. |
| Drift direction | `class.mutation_rate` + RNG | The seed scales magnitude, not direction. |
| Recombination crossover | `civ-genetics::recombine` | Uniform crossover is substrate-internal. |
| Body-plan category labels (e.g. "arachnid", "serpent") | None — emergent | The renderer reads morphology; no taxonomy is stored. |

### 3.4 Testable invariants

| Invariant | Source | Guarantee |
|-----------|--------|-----------|
| `raw_organism_primitive_is_valid` | `seeds.rs:512` | Engine always has a valid fallback genome. |
| `divergence_dial_zero_means_no_drift_over_generations` | `seeds.rs:626` | `divergence = 0.0` keeps the genome *byte-stable* across 10 000 ticks. |
| `divergence_dial_one_means_free_drift` | `seeds.rs:642` | `divergence = 1.0` produces non-trivial byte flips within 1 000 ticks. |
| `divergence_dial_intermediate_scales_rate` | `seeds.rs:665` | Rate-monotonicity: more divergence → more flips per tick. |
| `test_named_seeds_differ` | `seeds.rs:776` | The three archetypes are pairwise-distinct (no re-skin). |
| `spawn_genome_with_locked_seed_returns_seed` | `seeds.rs:722` | `divergence = 0.0` produces an exact clone. |
| `spawn_genome_with_divergence_one_drifts` | `seeds.rs:831` | `divergence = 1.0` produces a non-clone. |

---

## 4. Layer 2 — Society (Cluster formation, social graph, diplomacy)

The society layer is the **first emergence layer that does not read DNA byte-by-byte**. It reads the **phenotype and psyche** of agents who are in spatial proximity and produces *measured* clusters, social ties, and inter-cluster relations.

### 4.1 Data path

```
Phenotype (from Layer 1)
  │
  ▼
BehaviorWeights { aggression, curiosity, sociability, intelligence }
  │
  │   feeds:
  │
  ├─► cluster formation           (civ-agents::cluster)
  │     • cluster_by_colocation — spatial proximity only
  │     • should_join, should_leave — BehaviorWeights.weighted utilities
  │     • ClusterId, ClusterMember
  │
  ├─► social graph edges          (civ-agents::social)
  │     • per-agent SocialGraph { ties: Vec<Tie> }
  │     • Interaction::{Coexisted, Cooperated, Competed, Defected, Kin}
  │     • Tie { kinship, familiarity, affinity, trust, last_seen }
  │     • decay_social_graph(tick) — exponential decay
  │     • apply_social_event — pure transition function
  │
  └─► diplomacy matrix            (civ-agents::diplomacy)
        • DiplomacyMatrix { scores: HashMap<(A, B), RelationRecord> }
        • DiplomacySignal { resource_competition, trade_volume, … }
        • RelationKind { Alliance, Trade, Neutral, Rivalry, War }
        • drift = trade_volume·W_trade − combat_grievance·W_grievance
                  − scarcity_pressure·W_scarcity − proximity·W_proximity
                  + resource_competition·W_competition + need_complementarity·W_complementarity
        (the seed influences *only* the inputs to these signals)
```

**Where the seed nudges:** the seed sets the *initial distribution* of `BehaviorWeights` (via `express()`). Two agents with the same `Phenotype` go through the *same* `should_join` / `should_leave` decision, but two agents with **different** phenotypes (because their seeds are different, or their divergence has carried them apart) take **different** paths.

**Critically:** the cluster graph, the social graph, and the diplomacy matrix are *not* seeded from `SeedDefinition`. They are derived. The engine does not store "this cluster is human" or "this faction is Ardani" — it stores `ClusterId { index: u64 }` and the cluster's `CultureProfile` (§5).

### 4.2 The emergence update rule

Per tick (`emergence.rs:249-288`):

1. For each pair `(a, b)` in the same cluster (size ≥ 2):
   - With probability 0.12, draw an interaction (70% `Coexisted`, 30% `Cooperated { benefit: 0.5 }`).
   - Apply the event to both `SocialGraph`s (`apply_social_event`).
2. Decay every `SocialGraph` (`decay_social_graph(tick_u32)`) — affinity / trust / familiarity move toward 0 in proportion to time since last contact.
3. Diplomacy is updated *separately* in the diplomacy layer (out of scope of `phase_emergence` but in scope of `tick`); it consumes the same social signals plus `last_tick_engagements` and energy budgets.

The seed's effect on the diplomacy drift is **statistical, not deterministic**: a `human_baseline` (low divergence, high cohesion from heredity) will tend to have a different *distribution* of affinity / trust over time than a `deep_one` (mid divergence, more variation in sociability), but no pair of agents is *guaranteed* a particular relation.

### 4.3 What the seed does NOT control

| Concern | Owned by | Why the seed can't control it |
|---------|----------|-------------------------------|
| Which agent joins which cluster | Spatial proximity + `BehaviorWeights.sociability` | No `cluster_id` is in the seed. |
| Tie strength, affinity, trust | Past interactions | `apply_social_event` is pure over `Interaction`. |
| Decay rate | `decay_social_graph` constant | Substrate; uniform across seeds. |
| Relation label (`Family` / `Enemy` / `Rival`) | Query-time derivation from `Tie` (`relation_label`) | Authored *rule*, not data. |
| Diplomacy drift formula | `diplomacy.rs::apply_signal` weights | Hardcoded weights, seed-agnostic. |
| Faction creation, alignment | Faction formation rules (`civ-agents` cluster + needs) | Emergent from cluster size + tie positivity. |

### 4.4 Testable invariants

| Invariant | Source | Guarantee |
|-----------|--------|-----------|
| `trade_emerges_toward_alliance` | `diplomacy.rs:301` | Trade signal moves a pair from Neutral toward Alliance. |
| `decay_social_graph_*` | `social.rs` | Ties decay toward zero without contact. |
| `apply_social_event_*` | `social.rs` | Interaction types produce deterministic edge deltas. |
| Faction formation thresholds | `agents/src/lib.rs:65-78` | `FORM_FACTION_COHESION`, `JOIN_FACTION_ACCEPTANCE` are constants, not seed-driven. |

**Anti-regression rule:** if a future change adds a code path like `if seed_id == "ardani" { affinity += 0.3 }`, that is a **hardcoding violation** and must be replaced with a *measurement* of the genome (e.g. higher mean `Phenotype.behavior.sociability`).

---

## 5. Layer 3 — Language (Culture drift + Language drift through contact)

The language layer is the **first emergence layer that the engine explicitly stores state for per cluster** (`BTreeMap<u64, CultureProfile>` in `emergence.rs:65`). The seed's role is **statistical**: it sets the initial distribution of `BehaviorWeights` across the cluster, and culture/language drift *measures* that distribution over time.

### 5.1 Data path

```
Phenotype (from Layer 1) + cluster membership
  │
  │   on first encounter of a cluster:
  ▼
CultureProfile::new(seed)                                          civ-agents/src/culture.rs:35
  │   seed: [f32; 4] = cluster-id-derived colours (NOT genome-derived)
  │   — initial TraitVector (4 floats in [0,1])
  │   — initial language vector = same
  │   — contact: 0.0, kinship: 0.0
  │
  │   per tick:
  ▼
drift_populations(profiles, edges, rng, 0.02, 0.08, 0.85)         civ-agents/src/culture.rs
  │   — mutation: ±0.02 per axis per tick (RNG-jittered)
  │   — contact mixing: weight 0.08 between profiles that share an edge
  │   — persistence: 85% of last value retained
  │
  ▼
updated CultureProfile (in self.emergence.cluster_cultures)
  │
  │   consumed by:
  │
  ├─► psyche.beliefs (Layer 4)    — agents sample their cluster's traits
  │   via belief_culture_exposure(social-graph weighted neighbours)
  │
  └─► emergence inspector / feed event
        — every 128 ticks: "N settlement cultures drifted"
```

### 5.2 The emergence update rule

Per tick (`emergence.rs:191-247`):

1. **Cluster census** — count `ClusterMember` per `ClusterId`; skip singletons.
2. **First-touch seeding** — for each new cluster (≥ 2 members), initialise `CultureProfile::new(cluster_id_derived_colours)`. Note: **the seed does not seed culture directly.** Culture starts from a deterministic, cluster-id-derived `[0,1]^4` vector and drifts from there. This is the **anti-hardcoding** guarantee: two different canonical seeds spawning in the same cluster (or in a single-position biome) start with *the same* culture seed.
3. **Drift** — if N ≥ 2, build a complete `ContactEdge` graph (weight 0.15) and call `drift_populations` with `mutation_rate=0.02`, `contact_weight=0.08`, `persistence=0.85`.
4. **Feed event** — every 128 ticks, emit a `culture_drift` feed event with the number of active cluster cultures.

The seed's only influence is **statistical**, through the distribution of phenotypes that show up at a cluster: a `0.0`-divergence race has a *narrower* phenotype distribution than a `1.0`-divergence race, so the cluster's behaviour-weighted `belief_culture_exposure` sample is also narrower. But this is a **measurement** of the genome, not a seed → culture arrow.

### 5.3 What the seed does NOT control

| Concern | Owned by | Why the seed can't control it |
|---------|----------|-------------------------------|
| Initial `TraitVector` | `cluster_id_derived_colours` (deterministic from id) | Seed has no `culture_seed` field. |
| Mutation rate per tick | `drift_populations` argument (0.02) | Hardcoded; uniform across seeds. |
| Contact graph | All-pairs with weight 0.15 | Seed-agnostic. |
| Language divergence | Same `drift_populations` on the `language` sub-vector | Same update rule, distinct component. |
| Kinship / creolisation rules | `culture::mutate_traits` (pure jitter) | No taxonomy. |
| Per-cluster labels ("Velthari-tongue", "Ardani-speak") | None | The inspector derives labels by *measurement*, not authoring. |

### 5.4 Testable invariants

| Invariant | Source | Guarantee |
|-----------|--------|-----------|
| `CultureProfile::new` is deterministic from seed | `culture.rs:35` | Same `seed: TraitVector` → same profile. |
| `drift_populations` is bit-deterministic with fixed RNG | `culture.rs` | All randomness threads through `rng`. |
| `culture_drift` feed event at 128-tick cadence | `emergence.rs:238` | Cadence is hardcoded; seed cannot suppress it. |
| `contact_weight = 0.08`, `persistence = 0.85` | `emergence.rs:234` | Hardcoded drift parameters. |

**Anti-regression rule:** if a future change adds `if seed.divergence < 0.1 { language_persistence = 0.95 }`, that is hardcoding: it should be replaced with a *measurement* (e.g. lower drift reduces phenotype variance, which produces a narrower culture sample, which emerges as lower language mixing — but only as an emergent effect, not an authored rule).

---

## 6. Layer 4 — Religion (Beliefs, psyche, sentience-awakening, legends)

The religion layer is the **most emergent** and the **least code-anchored** layer in the current engine. There is no `Religion` struct; religion is a **measured pattern over** (psyche beliefs + sentience threshold crossings + legends sagas). The seed's role is *only* through the genome's effect on the psyche and on sentience trait accumulation.

### 6.1 Data path

```
Dna (from Layer 0)
  │
  ▼
PsychGenomeProfile {                                              civ-agents/src/psyche.rs:81
  drive_slots, reactivity_slots, sociability_slots,              (data-driven genome projection)
  risk_slots, impulsivity_slots
}
  │
  │   on first Psyche creation for an agent:
  ▼
psyche_from_dna(&Dna, &profile) → Psyche {                        civ-agents/src/psyche.rs
  drives[4], temperament, mood, beliefs[4], maturity
}
  │
  │   per tick (emergence.rs:328-470):
  │     update_mood(needs, temperament, threat, delta_needs)
  │     nudge_temperament(arousal, belonging, maturity)
  │     update_beliefs(beliefs, exposure, sociability, rng)
  │
  ▼
Psyche (in hecs world)
  │
  │   observed by:
  │
  ├─► sentience scoring                                           civ-genetics::sentience
  │     CognitionTraitProfile::score(dna) ≥ threshold
  │     → SentienceEvent (lineage_id, cognition_score)
  │
  └─► awakening coupling                                          emergence.rs:539
        apply_awakening_coupling():
          awakenings = last_sentience.len()
          add_belief(awakening_belief_gain(awakenings))     // bounded per tick
          add_cohesion(awakening_cohesion_gain(awakenings)) // bounded per tick
  │
  ▼
sentience threshold crossings → SpeciationEvent → legends ingest
                                                                  civ-legends
  │
  ▼
SagaGraph (saga = persistent narrative; not authored per race)
```

### 6.2 The emergence update rule

1. **Psyche derivation** (`psyche_from_dna`) — a *pure* function of `Dna` and `PsychGenomeProfile` (a data-driven slot/weight table). Two agents with the same `Dna` get the same `Psyche`. The profile is a **projection rule**, not authored content per race.
2. **Mood update** — `update_mood` reads `needs` (food, shelter, safety, belonging) and `temperament` and produces `Mood { valence, arousal }`. No seed input.
3. **Temperament nudge** — `nudge_temperament` reads arousal, belonging, and maturity. The seed sets the starting value; experiential events nudge it within `[0, 1]`.
4. **Beliefs update** — `update_beliefs` reads *culture* (sampled through the social graph via `belief_culture_exposure`), *sociability* (from the psyche), and the local RNG. **The seed's only input is the initial sociability value**, derived from DNA bytes.
5. **Sentience** — `evaluate_sentience(agent_id, dna, profile, threshold)` computes a `CognitionTraitProfile` score; crossing `SentienceThreshold` produces a `SentienceEvent`.
6. **Awakening coupling** (`apply_awakening_coupling`, `emergence.rs:539-546`) — for each sentience crossing in `last_sentience`, mint a *bounded* belief pulse (`awakening_belief_gain`) and a *bounded* cohesion pulse (`awakening_cohesion_gain`). The bounds are per-tick caps (edge-of-chaos principle).
7. **Legends ingest** (`emergence_legends`) — for each `Birth` / `Death` / `Sentience` event this tick, ingest into `SagaGraph` with a `RawSimEvent` payload; mark deaths via `mark_died`. Promotions to named entities (Founders, Sages, Martyrs) are computed from the saga graph, not from the seed.

The seed influences religion **only** through (a) the distribution of cognition scores (via genome distribution) and (b) the rate of `SentienceEvent` production. The *content* of the saga — which agent becomes a "founder", what the saga calls a deity, what a martyr's epitaph says — is **all generated downstream** by `civ-legends` and `civ-ai` (naming layer).

### 6.3 What the seed does NOT control

| Concern | Owned by | Why the seed can't control it |
|---------|----------|-------------------------------|
| Initial `beliefs` | `update_beliefs(0)` (zeroed) | No `beliefs_seed` field on the seed. |
| `PsychGenomeProfile` slot map | `civ-agents::psyche` | Substrate-level projection rule. |
| Sentience threshold value | `SentienceThreshold` (EmergenceState) | Authored once for the whole simulation, not per seed. |
| Awakening gain caps | `MAX_AWAKENING_BELIEF_PER_TICK`, `MAX_AWAKENING_COHESION_PER_TICK` | Hardcoded; seed-agnostic. |
| Saga entity promotion | `SagaGraph` + `civ-ai` prompt | Emerges from event count and timing. |
| Named deities, sacred sites, myths | None — generated per-scenario by `civ-ai` | No static mythology table. |
| Faith, prayer, ritual | Not yet modelled | Out of scope; if added, must follow the same substrate rule. |

### 6.4 Testable invariants

| Invariant | Source | Guarantee |
|-----------|--------|-----------|
| `psyche_from_dna` is pure | `psyche.rs` | Same `Dna` + same `profile` → same `Psyche`. |
| `apply_awakening_coupling` reads `last_sentience`, no second scan | `emergence.rs:539-546` | No double-counting across phases. |
| Awakening gain is bounded per tick | `MAX_AWAKENING_BELIEF_PER_TICK`, `MAX_AWAKENING_COHESION_PER_TICK` | Edge-of-chaos cap. |
| `sentience_profile` is a data-driven trait-accumulation rule | `emergence.rs:87-91` | The profile (not the seed) defines the threshold. |
| Saga promotions are scored from event count | `civ-legends` | The seed does not appear in the scoring formula. |

**Anti-regression rule:** the *one* place the seed can show up in this layer is in the genome's contribution to `CognitionTraitProfile::score(dna)`. That's a *measurement* of the genome. If a future change says `if seed_id == "human_baseline" { sentience_threshold *= 0.9 }`, that is hardcoding — replace it with a content fix in the seed's genome bytes (so the measurement changes), not with a code branch.

---

## 7. Layer 5 — Economy (Markets, polities, trade routes)

The economy layer is the **least coupled** layer to seeds of the five. The seed's role is **even more indirect**: it sets the population distribution (via birth rate, which depends on genome-derived traits) and the per-faction `aggression` (via `express()` in `emergence_genetics_sentience`), both of which are *measurements* on the genome distribution.

### 7.1 Data path

```
Dna (per agent) ─────────────────────────────────────────────────┐
  │                                                                │
  ▼                                                                │
express(&dna) → BehaviorWeights.aggression                          │
  │                                                                │
  ▼                                                                │
faction_aggression[faction_id] = mean(BehaviorWeights.aggression)   │
                                  over all members                  │
  │   (rebuilt every tick in emergence.rs:491-506)                │
  │                                                                │
  ▼                                                                │
Per-faction behavioural tendency (a measured scalar)              │
  │                                                                │
  │   consumed by:                                                 │
  │                                                                │
  ├─► polities / market formation                                 │
  │     civ-economy crate — emergent market geometry              │
  │     (no seed-aware code paths; see §7.2)                       │
  │                                                                │
  ├─► trade-route emergence (FR-ECON-trade)                        │
  │     civ-economy::trade — between polities                      │
  │     uses DiplomacySignal.trade_volume (see §4)                │
  │                                                                │
  └─► institution / policy emergence                               │
        civ-economy::institutions — emergent polity struct         │
        (no seed-aware code paths)                                 │
  │                                                                │
  ▼                                                                │
measured institutional state (markets, trade routes, polities)   ─┘
```

### 7.2 The emergence update rule

The current engine does **not** model markets or polities inside `phase_emergence`. Those layers are under active design in [`polities-markets.md`](polities-markets.md) and [`civ-economy-emergent-markets.md`](civ-economy-emergent-markets.md). The present rule is:

1. **Population** — births in `phase_life` consume `Dna`; newborns inherit parent DNA (with mutation) and join clusters based on `BehaviorWeights.sociability` (from `express()`). Cluster sizes are **measured**, not seeded.
2. **`faction_aggression`** — `emergence_genetics_sentience` (`emergence.rs:491-506`) computes the *mean* of `BehaviorWeights.aggression` for each faction's members every tick. This is the *one* place a seed-derived value feeds a per-faction scalar. It is a **measurement**, not a hardcoded lookup.
3. **Markets / polities / trade** — emerge from population density, cluster distances, resource distribution, and diplomacy (§4). **No seed field is read in this code path.** A faction's *propensity* to trade vs raid is a function of its *measured* `faction_aggression`, not its `seed_id`.

**Design intent for future economy code:** the same anti-hardcoding rule applies. If a new feature wants to say "this race is good at fishing", it must be a property of the genome (e.g. a `wetland_affinity` byte that produces measurable food-collection efficiency) — never a `if seed_id == "deep_one" { fishing_yield *= 1.5 }` branch.

### 7.3 What the seed does NOT control

| Concern | Owned by | Why the seed can't control it |
|---------|----------|-------------------------------|
| Polity / market location | Spatial proximity + population density | No `polity_seed` field. |
| Trade-route topology | Cluster graph + diplomacy | Seed-agnostic. |
| Tariff / tax / institution parameters | Hardcoded policy + emergent need | No per-seed tax table. |
| Per-faction aggression | `faction_aggression` (mean of members' `BehaviorWeights.aggression`) | This is a *measurement* of the genome, not a seed field. |
| Resource endowment | `civ-planet` geology + biome | Hardcoded physical laws. |
| Currency, prices | Emergent from market geometry | Not yet modelled; must follow the substrate rule. |

### 7.4 Testable invariants

| Invariant | Source | Guarantee |
|-----------|--------|-----------|
| `faction_aggression` is a mean, not a lookup | `emergence.rs:491-506` | No per-seed branch; uniform per-faction formula. |
| Diplomacy drift formula | `diplomacy.rs::apply_signal` | Six scalar inputs; no seed-aware path. |
| `trade_emerges_toward_alliance` | `diplomacy.rs:301` | Trade signal moves relations positive. |
| `last_tick_engagements` decays | `diplomacy.rs` | Grievance decay is a per-tick function of state. |

**Anti-regression rule:** if a future economy crate wants to add a per-seed bonus, the bonus must be a **measurement** (e.g. `seed.divergence` → mutation rate is a measurement of the dial, not a hardcoded per-seed value). A direct `if seed.id == "X" { … }` is a violation.

---

## 8. Cross-layer data-flow summary

The five layers, side by side, with the seed's *measurement* of influence (not authoring):

| Layer | Reads from genome | State the layer maintains | Feed event cadence | Anti-regression test anchor |
|-------|-------------------|---------------------------|--------------------|------------------------------|
| **Species** | bytes 0..8 (morphology + behaviour) | `Dna` per agent; `Phenotype` is derived | `sentience` on threshold cross; `speciation` on distance cross | `divergence_dial_*`; `test_named_seeds_differ` |
| **Society** | bytes 5..8 (behaviour) | `SocialGraph` per agent; `ClusterMember`; `DiplomacyMatrix` | none per tick | `trade_emerges_toward_alliance`; `decay_social_graph_*` |
| **Language** | (statistical only — through phenotype distribution) | `CultureProfile` per cluster | `culture_drift` every 128 ticks | `drift_populations` determinism; cadence hardcoded |
| **Religion** | bytes via `CognitionTraitProfile` (sentience) and `PsychGenomeProfile` (psyche) | `Psyche` per agent; `SagaGraph`; awakening counters | `psyche_sample` every 64 ticks; `sentience` on threshold cross; `legend_promotion` on promotion | `apply_awakening_coupling`; per-tick cap tests |
| **Economy** | byte 5 (aggression) for `faction_aggression` mean | `faction_aggression` map; future polities/markets | none yet | `faction_aggression` mean formula is uniform across seeds |

The single sentence that summarises the whole table: **the seed is a byte-vector source whose values are *measured* by the emergence layers; the layers never read the seed's `id` for routing or weighting.**

---

## 9. The four anti-hardcoding rules

These are the *audit tests* for any future code change that touches a seed or an emergence layer.

### 9.1 The no-id-routing rule

> No code path in the emergence stack may branch on `SeedDefinition::id`, `NamedSeed::Ardani/Velthari/Grundak`, or any other *named* content. If a branch is found, the value it depends on must be a *property of the genome* (a measurement), not of the name.

The current code is clean: `seed_library.get(id)` is called by `select_seed_for_position` (`emergence.rs:127-149`) **only** to pick a seed for a *spawn position* (i.e. only at the seed → substrate boundary). After that, only the `Dna` and `DnaClass` are used. The `NamedSeed` enum is referenced by `choose_named_seed` and `seed_mix` *only* for spawn-time archetype selection (i.e. to choose which `SeedDefinition` to apply), and the spawn helper then discards the `NamedSeed` and uses the seed's `genome` + `divergence`. There is no code path that says `if seed == "ardani" { … }` in the emergence stack.

### 9.2 The measurement-only rule

> All downstream reads from a seed are reads of the *result* of the seed (genome bytes, divergence scalar, faction_aggression mean), never reads of the *seed itself* (id, display_name, notes). 

This is enforced by keeping `SeedDefinition` ownership at the spawn boundary: once a `Dna` exists in the `hecs` world, the `SeedDefinition` that produced it is not stored alongside.

### 9.3 The dial-as-knob rule

> The divergence dial is the **only** seed-derived scalar that flows past the spawn boundary (`effective_mutation_rate`). It scales a substrate rate; it does not gate, threshold, or branch on its own. `divergence_override` in scenario YAML is the only author-facing knob, and it is range-validated.

### 9.4 The no-authoring-in-emergence rule

> No emergence phase (1–5 above) is allowed to have an *authored taxonomy* (an enum of named cultures, an enum of named factions, an enum of named religions). The layers that do have authored taxonomies — `NamedSeed`, `MaterialId`, `BiomeKind` — are content *primitives* (substrate or content layer), not emergent outcomes. 

Examples of forbidden authoring:
- `enum CultureKind { ArdaniCaste, VelthariCorte, GrundakLith }` — author what culture looks like.
- `enum FactionArchetype { Theocratic, Mercantile, Militaristic }` — author what factions look like.
- `enum ReligionType { AncestorWorship, SkyGod, Animism }` — author what religion looks like.

The audit question: *can the layer explain every value in its state by citing the substrate?* If yes, it is emergent; if no, it has been authored.

---

## 10. Open questions / future work

| Question | Where it lands | Effect on this doc |
|----------|----------------|--------------------|
| Should `SeedDefinition` gain a `traits: Vec<String>` field for race-archetype intent (e.g. `["caste", "predator"]`)? | TBD — see [`CONTENT_SEEDS.md §8`](CONTENT_SEEDS.md#8-open-questions--future-work) | **No** for the emergence stack: a `traits` field is inspector-only. The emergence code must continue to read only the genome. |
| Should `PsychGenomeProfile` slot maps be seed-relative or class-relative? | TBD — see [`psyche-social.md §2.2`](psyche-social.md#22-genetic--psyche-mapping-authored--the-mapping-rule-not-the-values) | Class-relative (substrate-level); the seed does not project psyche. |
| Should sentience thresholds be seed-relative? | **No.** Substrate-level, like speciation. The seed is *measured*, not consulted. |
| Should economy crates gain a `seed_aware` mode for "scenario-driven economy" (e.g. a merchant-republic scenario)? | **No.** A scenario can pre-seed *economy state* (treasury values, market positions) via scenario YAML; that is not a seed branch. The canonical seeds never branch. |
| Can a future religion layer add a `Faith` or `Pantheon` struct? | Yes — but it must be *measured* (e.g. `Faith` = mean psyche.beliefs over a cluster) and not authored (no `enum Pantheon { Sun, Sea, Mountain }`). | Add to §6 as a measured struct; do not change §9.4. |
| Should `select_seed_for_position` fall back to a *random* seed rather than `active_seed`? | TBD. Current behaviour is `active_seed` fallback (deterministic). Random fallback would couple spawn to RNG draws in a way that the current `seed_mix` round-robin avoids. | If changed, update §3.1. |

---

## 11. References

- [`docs/design/CONTENT_SEEDS.md`](CONTENT_SEEDS.md) — 2-layer content model, seed schema, divergence dial semantics, anti-hardcoding audit (§6).
- [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) — phase ordering, layering contract (§1.2).
- [`docs/design/species-sentience.md`](species-sentience.md) — DNA → phenotype → sentience pipeline (Layer 1 + sentience part of Layer 4).
- [`docs/design/psyche-social.md`](psyche-social.md) — mind + relationship emergence (Layer 2 + psyche part of Layer 4).
- [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md) — culture/language drift (Layer 3).
- [`docs/design/polities-markets.md`](polities-markets.md) — polity emergence (Layer 5).
- [`docs/design/civ-economy-emergent-markets.md`](civ-economy-emergent-markets.md) — market emergence (Layer 5).
- [`crates/genetics/src/seeds.rs`](../../crates/genetics/src/seeds.rs) — `SeedDefinition`, `SeedLibrary`, `spawn_genome_with_divergence`, `mutate_with_divergence`, `seed_with_divergence`, `effective_mutation_rate`, archetype races.
- [`crates/genetics/src/sentience.rs`](../../crates/genetics/src/sentience.rs) — `CognitionTraitProfile`, `SentienceEvent`, `SentienceThreshold`.
- [`crates/species/src/lib.rs`](../../crates/species/src/lib.rs) — `express(dna) → Phenotype`, byte layout (0..9).
- [`crates/agents/src/culture.rs`](../../crates/agents/src/culture.rs) — `CultureProfile`, `drift_populations`, `TraitVector`, contact edges.
- [`crates/agents/src/social.rs`](../../crates/agents/src/social.rs) — `SocialGraph`, `Tie`, `apply_social_event`, `decay_social_graph`, `relation_label`.
- [`crates/agents/src/diplomacy.rs`](../../crates/agents/src/diplomacy.rs) — `DiplomacyMatrix`, `DiplomacySignal`, `RelationKind`, six-driver drift formula.
- [`crates/agents/src/psyche.rs`](../../crates/agents/src/psyche.rs) — `Psyche`, `Mood`, `Temperament`, `PsychGenomeProfile`, `psyche_from_dna`.
- [`crates/engine/src/emergence.rs`](../../crates/engine/src/emergence.rs) — `phase_emergence` orchestrator, `select_seed_for_position`, `apply_awakening_coupling`, `seed_library` / `register_seed_*` / `set_active_seed` accessors.
- [`crates/engine/src/engine.rs`](../../crates/engine/src/engine.rs) — `choose_named_seed` (round-robin / WeightedIndex helper).
- [`crates/engine/src/scenario.rs`](../../crates/engine/src/scenario.rs) — `Scenario::active_seed`, `SeedWeight`, `divergence_override`, `seed_mix` validation.
- [`crates/planet/src/geology.rs`](../../crates/planet/src/geology.rs) — `BiomeKind::matches_affinity` (biome-label resolution).
- [`scenarios/canonical_seeds.ron`](../../scenarios/canonical_seeds.ron) — canonical RON content pack (`raw_organism`, `human_baseline`, `deep_one`).
- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — emergence-default principle.
- [`docs/traceability/fr-3d-matrix.md`](../traceability/fr-3d-matrix.md) — `FR-CIV-GENETICS-*`, `FR-CIV-SPECIES-*`, `FR-CIV-AGENTS-*`, `FR-CIV-LEGENDS-*`, `FR-ECON-*` coverage.
