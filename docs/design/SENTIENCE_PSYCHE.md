# Sentience & Psyche — Emergence Architecture for the Individual Mind

> **Status:** Design (Planner) — emergent individual psychology from the substrate upward.
> No code in this commit; the document is the architecture reference a follow-up
> implementation PR will follow.
>
> **Owner crates (read-only here):** `crates/needs` (Layer-0 needs substrate),
> `crates/genetics` (Dna / sentience primitives), `crates/species` (DNA → phenotype),
> `crates/agents` (`psyche.rs`, `social.rs`, `culture.rs`, `cluster.rs`, `diplomacy.rs`),
> `crates/engine` (`emergence.rs` orchestrator, `phase_*` macro-phases).
>
> **Charter binding:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md).
> Hardcoded laws only: physics, materials/energy, genomics. **Sentience and psyche emerge.**
> This is a charter-compliance audit doc — every field and rule below must be answerable
> to the question "which Layer-0 substrate does this read from?"
>
> **Companion docs:**
> - [`species-sentience.md`](species-sentience.md) — DNA → phenotype → cognition accumulation → sentience threshold (the **species-level** story).
> - [`psyche-social.md`](psyche-social.md) — per-agent Psyche vector + social graph data model + AI hooks (the **individual-mind data model**). This doc *complements* that one by tracing the **flow of causation** from substrate to social-field to cluster culture.
> - [`civ-culture-emergent.md`](civ-culture-emergent.md) — cluster-level culture drift, dialects/creoles.
> - [`SEEDS_EMERGENCE_FLOW.md`](SEEDS_EMERGENCE_FLOW.md) — what seeds do and don't touch in the emergence stack.
> - [`EMERGENCE_WIRING_PATCHPLAN.md`](EMERGENCE_WIRING_PATCHPLAN.md) — `phase_life → phase_research → phase_tech → phase_belief → phase_unrest → phase_cohesion → phase_social_mood → phase_stratification → phase_institutions → phase_economic_focus → phase_emergence → phase_diffusion` DAG that the psyche layer feeds into.
>
> **FR families:** `FR-CIV-PSYCHE-*` (psyche / social-graph dynamics, already enumerated
> in `psyche-social.md`), `FR-CIV-SPECIES-4xx` (sentience ladder, `species-sentience.md`),
> `FR-CIV-AGENTS-EMERGENCE-*` (the macro social field hooks — new IDs allocated in this
> doc), `FR-CIV-NEEDS-*` (substrate reads).

---

## 1. Thesis — the mind is an emergent field, not an authored enum

The `psyche-social.md` doc already specifies the **data model**: a per-agent
`Psyche { drives, temperament, mood, beliefs, maturity }` derived from `Dna` via
`PsychGenomeProfile`, plus a per-agent `SocialGraph` that accumulates from
`Interaction` events. What it does not yet specify is **the full causal chain
upward** — how physics, materials, geology, hydrology, climate, needs decay,
disasters, social-graph decay, culture drift, and the wired emergence macro-phases
combine to produce a per-agent mind that the player can *see* behaving in ways
no author wrote.

This doc is the missing link: the **architecture of emergence** from Layer-0
substrate up to cluster culture, written so a follow-up implementation lands
**only authoring hooks** (where the substrate meets the layer), never authored
outcomes. It does not duplicate the data model — it traces **where each field
in `Psyche` and each tie in `SocialGraph` gets its inputs from** and **which
downstream consumer reads it back**.

```
                          LAYER 0 — hardcoded substrate
                          physics · materials · energy · needs · genomics
                                          │
                                          ▼
   LAYER 1 — substrate reads (no authored content)
     needs decay/regen · threat pressure · material/fluid proximity · RNG
                                          │
                                          ▼
   LAYER 2 — agent-internal psyche updates (psyche.rs + social.rs)
     drives (DNA, immutable) · temperament (DNA + slow nudge)
     mood (needs + threat + event_term) · beliefs (graph-weighted culture)
     social graph (Interaction events, contact decay)
                                          │
                                          ▼
   LAYER 3 — cluster-level emergence (cluster.rs + culture.rs)
     ClusterMember · CultureProfile.traits / .language / .kinship / .contact
                                          │
                                          ▼
   LAYER 4 — macro emergence (phase_emergence + phase_cohesion + phase_social_mood)
     awakening pulses · belief · cohesion · society_mood · unrest · strat
                                          │
                                          ▼
   LAYER 5 — civic narrative (legends saga graph + civ-ai naming)
     Founder / Sage / Martyr promotions · feed events · chronicled milestones
```

**The single rule.** Every box above is **derived from the one below by a pure
function** (possibly RNG-jittered, per the determinism-dropped charter). If any
arrow requires the implementer to write "if X is a Hero, then Y", the rule is
already wrong — replace with a measurement.

---

## 2. Where sentience enters — the lineage-level ladder (recap + linkage)

Per [`species-sentience.md §4`](species-sentience.md), a lineage crosses the
sentience threshold when its cognition score (genome-derived, never assigned)
exceeds `SentienceThreshold.minimum_cognition`. The ladder is:

| Rung | Trait gate | Unlocks | Downstream consumer |
|------|-----------|---------|---------------------|
| **Learning** | memory_capacity | `Psyche.temperament` plasticity actually does something — temperament nudges from lived experience | `nudge_temperament` (`crates/agents/src/psyche.rs:169`) |
| **Tool-use** | tool_affinity + learning | `daily_path.rs::score_poi` learns to treat objects as means (existing hook already in `psyche-social.md §4.1`) | `crates/agents/src/daily_path.rs` |
| **Culture** | social_coordination + tool_use | cluster `CultureProfile` is **re-derived** from agent-level beliefs (not the other way around); close the belief ↔ culture loop | `crates/agents/src/culture.rs` |
| **Language** | signal/communication + culture | `CultureProfile.language` drift becomes meaningful; creole threshold drops | `crates/agents/src/culture.rs:drift_populations` |
| **Civilization** | abstraction + language | polity/market/architecture layers come online (out of scope here, see `polities-markets.md`) | `crates/economy` |

**Key insight:** every rung reads only the genome — never an authored `is_sentient: bool`.
The rung latches feed *downward* into `Psyche.temperament.sociability` and *upward* into
`awakening_belief_gain` / `awakening_cohesion_gain` (per
[`EMERGENCE_WIRING_PATCHPLAN.md §3.11`](EMERGENCE_WIRING_PATCHPLAN.md)), which are the
**only** places a sentience crossing touches macro state.

This doc assumes the lineage-level ladder exists; it focuses on the
**individual-psychology consequences** of a rung being crossed.

---

## 3. The four psyche sub-layers — what each one reads, what each one writes

### 3.1 `drives` (PSYCHE_DIM = 4 floats, [0, 1])

**Read from:**
- `Dna` via `PsychGenomeProfile.drive_slots` — a *projection rule* (data-driven
  byte-slot weights, identical pattern to `cognition_score`). See
  [`crates/agents/src/psyche.rs:121-124`](../../crates/agents/src/psyche.rs) (`score_axis`).
- Offspring DNA comes from `civ-genetics::recombine` + `mutate`; therefore
  offspring drives resemble parents but vary (no authored inheritance of
  personality).

**Written to:** never after birth (`psyche.rs:134-150` — `psyche_from_dna`
sets `drives` once at birth).

**Writes into:**
- `score_poi` bias in `daily_path.rs::drive_gain(need_kind, psyche.drives)` —
  the same `Needs` vector gets a different *per-agent* interpretation. Two
  agents with identical `Needs { food: 0.9, … }` make different `top_action`
  choices if their drives differ.
- `belief_culture_exposure` weighting via `temperament.sociability` (a derived
  trait; see §3.2).

**Charter check.** Authored: `PsychGenomeProfile.drive_slots` (a *projection
rule*, equivalent to `CognitionTraitProfile.trait_weights`). Emergent: every
numeric value of `drives[i]` (a measurement of the DNA bytes). No
`enum Personality { Bold, Cautious, … }` exists.

### 3.2 `temperament` (4 floats, [0, 1]: reactivity / sociability / risk_tol / impulsivity)

**Read from:**
- DNA at birth (same slot-weight reducer as drives).
- `recent_mood_variance` and `recent_social_satisfaction` over a rolling
  window — the *only* inputs that move temperament after birth
  ([`psyche.rs:169-182`](../../crates/agents/src/psyche.rs)).
- `psyche.maturity` gates the plasticity: `(1 - maturity*0.8) ∈ [0.2, 1.0]`
  so children are plastic, adults are stable.

**Written to:** `temperament` itself (one update per Warm or Cold LOD tick).

**Writes into:**
- `score_poi` (`daily_path.rs`): `risk_tol` modulates SafeZone avoidance,
  `impulsivity` discounts distance/effort, `sociability` boosts
  `social_pull(poi, social_graph)` term.
- `update_mood` reactivity term (`psyche.rs:196`): high reactivity = faster
  mood convergence + overshoot → emergent volatile/stoic temperaments without
  authoring either.
- `update_beliefs` sociability LR (`psyche.rs:208`): high sociability = more
  conformist belief drift; low = contrarian.
- Cluster `MembershipPayoff` (`psyche-social.md §4.2`): temperament + social
  graph decide whether to stay, leave, or join.

**Charter check.** Authored: the slot map and the LR constants. Emergent:
every value of `reactivity/sociability/risk_tol/impulsivity`. No "personality
archetype" enum exists.

### 3.3 `mood` (valence ∈ [-1, 1], arousal ∈ [0, 1])

**Read from:**
- `Needs` (food, shelter, safety, belonging — from `crates/needs/src/lib.rs`),
  with **all six** needs `food/water/rest/safety/social/health` averaged. The
  engine uses the 4-component `civ_agents::Needs` for utility scoring and the
  6-component `civ_needs::Needs` for the survival pipeline — both feed the
  psyche update (`emergence.rs:397-416`).
- `Health` (mirror of `needs.health`) and threat pressure
  (`1 - needs.safety`).
- Decayed sum of recent `SocialEvent` affinity deltas (the `event_term`,
  half-life ~1 day).
- `temperament.reactivity` (the *only* temperament field the mood update
  reads directly).

**Written to:** `mood` itself (Hot: every tick; Warm: every N ticks; Cold:
frozen as cluster-mean, see §6).

**Writes into:**
- `phase_social_mood` aggregate: `state.society_mood` is the per-tick mean of
  `psyche.mood.valence`, bounded step `MAX_MOOD_STEP_PER_TICK = 0.05` per tick
  ([`EMERGENCE_WIRING_PATCHPLAN.md §3.7`](EMERGENCE_WIRING_PATCHPLAN.md)). The
  mood is therefore a *slow-moving aggregate* — individual spikes get absorbed,
  but a chronic dearth shows up.
- `phase_unrest` `agent_misery_unrest` term: mean `(-mood.valence)` over the
  ECS, single-tick lag, contributes to `state.unrest`.
- `nudge_temperament` `recent_mood_variance` input: chronic mood swings nudge
  `reactivity` upward; chronic calm nudges it down.
- Inspector "Mind" panel (read-only projection).
- `relation_label` derivation: nothing stored, but `Friend`/`Rival` boundaries
  reflect the *cumulative* social event history that produced the mood.

**Charter check.** Authored: the update formula constants. Emergent: every
value of `valence/arousal`. No `enum Emotion { Happy, Angry, Sad }` exists;
the `Mood` struct is two floats and a valence/arousal axis is *what the data
looks like*, not what it is named.

### 3.4 `beliefs` (PSYCHE_DIM = 4 floats, [0, 1])

This is the field that **closes the belief ↔ culture loop**. Per
[`psyche-social.md §3.3`](psyche-social.md) and the existing
`update_beliefs` (`psyche.rs:202-216`), beliefs are blended toward a
graph-weighted culture exposure:

```
exposure = Σ_ties w_i * CultureProfile(other_i.cluster).traits
         where w_i ∝ familiarity_i * max(affinity_i, 0)
beliefs  = mix(beliefs, exposure, BELIEF_LR * sociability)
         + small jitter from culture::mutate_traits
```

**Read from:**
- `CultureProfile.traits` of the cluster(s) of the agent's *strongest social
  ties* (`emergence.rs:376-390`). This is **not** the agent's own cluster —
  it's the cluster(s) of the agent's friends/family. Therefore a cluster with
  a strongly-connected minority tie set will exhibit **emergent heterodoxy**:
  the minority-tied agents drift toward a different belief vector than their
  cluster centroid.
- `temperament.sociability` for the LR magnitude.
- Local RNG for the mutational wobble (the same `culture::mutate_traits` used
  in `culture.rs:70` — one RNG pattern shared across layers).

**Written to:** `beliefs` (per Hot tick + small jitter).

**Writes into:**
- Cluster `CultureProfile.traits` aggregation: per the existing orchestrator,
  cluster culture is re-derived from the belief centroid weighted by influence
  (i.e. mean of `psyche.beliefs` over cluster members with `[0,1]` bounds).
  This **closes the loop**: individual beliefs → cluster culture → individual
  beliefs. There is no parallel authored `ClusterBelief` table.
- `phase_stratification` does **not** read `beliefs` directly; `beliefs` feed
  `culture.rs` only.
- Inspector "Beliefs" view: contrast per-agent vector against cluster centroid
  to expose heterodoxy at a glance.

**Charter check.** Authored: the LR constants and the `mix_trait_vectors`
helper. Emergent: every value of `beliefs[i]`. No `enum Ideology { Anarchist,
Monarchist, Theocrat }` exists — the inspector derives labels at query time
from the *correlation with observed behaviour*, not from stored taxonomy.

---

## 4. The social graph — what produces a tie, what a tie produces

The graph is **directed, sparse, and contact-decaying** (per
`psyche-social.md §2.3` and the existing `social.rs`). This section documents
the **producers and consumers** the graph touches.

### 4.1 Producers (the only things that mutate the graph)

```
crate/system → Interaction → apply_social_event
```

| Producer | Crate / function | Output | Notes |
|----------|------------------|--------|-------|
| Cluster pair interaction | `crates/engine/src/emergence.rs:249-288` (`emergence_social`) | `Coexisted` (70%) / `Cooperated{0.5}` (30%) per 0.12 prob per co-located pair | Existing; this is the *baseline* tie generator. |
| Daily-path POI contention | `crates/agents/src/daily_path.rs` (POI score competition) | `Competed{pressure}` when two agents contest the same POI this tick | Charter: competition is measured by who picks the POI, not by an authored `competing: bool`. |
| Combat / theft | `crates/engine/src/tactics.rs`, `crates/economy` (planned) | `Defected{harm}` on damage / theft | Severity is the magnitude of harm taken/inflicted. |
| Birth / family link | `crates/engine/src/demographics.rs` (planned reproduction), `civ-agents::should_reproduce` (`social.rs:209-239`) | `Kin` event sets `kinship = 1.0` and bumps familiarity | One-shot at birth. |
| Faction alignment | `crates/agents/src/lib.rs::Alignment::form_faction/join_faction` (existing `FR-CIV-EMERGENCE-001/002`) | Reads `graph.ties` but does **not** emit `Interaction` | Faction membership is a *query result* over the graph, not a parallel authored state. |

**Anti-regression rule.** No code path may write to `SocialGraph.ties` outside
of `apply_social_event` / `decay_social_graph` / `evict_weakest`. No producer
may store a "friend id" field on the agent. All ties are derived from event
history.

### 4.2 Consumers

| Consumer | What it reads | How |
|----------|---------------|-----|
| `belief_culture_exposure` | familiarity + max(affinity, 0) | Weight for sampling `CultureProfile(other_cluster).traits` |
| `MembershipPayoff` (cluster join/leave) | mean affinity × trust over ties already in cluster | Decides whether to commit or break (per `psyche-social.md §4.2`) |
| `DiplomacySignal.social_bridge` (proposed) | cross-cluster tie density | Biases `trade_volume` (positive ties) vs `resource_competition` (negative ties). Per `psyche-social.md §4.3`. |
| `phase_cohesion` `micro_cohesion_delta` | positive tie count vs fray threshold | Bumps `state.cohesion` (per `EMERGENCE_WIRING_PATCHPLAN.md §3.6`); existing scan pattern, no double-counting. |
| `relation_label(tie)` | kinship + affinity + trust + familiarity | Returns `Family / CloseFriend / Partner / Acquaintance / Rival / Enemy` (existing, derived at query time, never stored). |
| `Alignment::form_faction` | positive tie count | Counts ties passing `is_positive_tie` (existing, `lib.rs:81-83`). |
| `Alignment::join_faction` | tie count to current faction members | Counts ties with `other ∈ members` passing `is_positive_tie`. |
| Inspector "Relationships" panel | the full graph | Ego-graph mini-map (green = affinity+, red = rivalry, gold = kin). |

**Charter check.** Authored: `apply_social_event` per-kind delta constants
(0.20/0.18/0.12 etc., `social.rs:159-186`); `relation_label` thresholds. Emergent:
every numeric value in every `Tie`. No `enum RelationKind { Friend, Foe }` stored.

### 4.3 Decay & eviction

- `decay_social_graph` runs every Warm/Cold tick (`social.rs:189-205`):
  - `familiarity *= 0.98^gap`
  - `affinity *= 0.995^gap` (only if kinship < 0.5 — kin stays kin)
  - `trust *= 0.992^gap`
  - Re-sorts by `other` (binary-search invariant for merge).
- Eviction drops the lowest-salience tie when `ties.len() > MAX_TIES = 150`
  (`social.rs:139-153`). Salience = `familiarity + |affinity| + kinship` —
  kin + deep friendships + active rivals survive, acquaintances fall.

**The Dunbar bound emerges as a self-tuning cap.** Per
`psyche-social.md §2.3`, `MAX_TIES = 150` is the natural cognitive limit; an
agent cannot meaningfully track more than ~150 stable relationships. The
mechanism is *measurement-driven*: a passive observer of a long simulation
will see graph sizes cluster around 150.

---

## 5. Upward causation — what psyche + social *feed* into

This section maps the upward arrows from `psyche.rs` and `social.rs` into the
wired macro-phases. Each arrow is a **read**, not a write — the macro-phases
do not store psyche state; they *aggregate* from it.

### 5.1 `psyche.mood.valence` → `phase_social_mood` (aggregate → `state.society_mood`)

Per [`EMERGENCE_WIRING_PATCHPLAN.md §3.7`](EMERGENCE_WIRING_PATCHPLAN.md):
`phase_social_mood` reads mean `psyche.mood.valence` and `.arousal` from
the world ECS, computes `delta = (mean_valence - state.society_mood).clamp(-MAX, +MAX)`
with `MAX_MOOD_STEP_PER_TICK = 0.05`, and writes back to `state.society_mood`
clamped to [-1, 1]. The slow-moving aggregate prevents individual spikes from
distorting macro state, but a chronic dearth of social/safety/satiety needs
will register as a sustained mood slide.

**Producer end:** `update_mood` at `psyche.rs:185-199` reads `Needs` and
`temperament.reactivity` per tick. **Lag**: single tick (psyche updates
post-`phase_life`, social mood aggregates next tick).

**Downstream consumers of `state.society_mood`:**
- `phase_stratification` (implicitly via `unrest` and `cohesion`).
- Inspector overlays (`EntityTint` over agent valence).
- Legends feed (`legend_promotion` events: chronic low mood → Martyr candidate).

### 5.2 `psyche.mood.valence` (negative) → `phase_unrest`

`agent_misery_unrest` reads mean `(-psyche.mood.valence)` from the ECS and
adds it to the unrest delta alongside food scarcity, commodity scarcity,
energy scarcity, overcrowding, inequality, and dispossession
([`EMERGENCE_WIRING_PATCHPLAN.md §3.5`](EMERGENCE_WIRING_PATCHPLAN.md)).
Single-tick lag; acceptable because the mean is slow-moving.

**Producer end:** same `update_mood`. **Downstream:**
- `phase_cohesion` `cohesion_delta(belief, unrest)` reads `unrest` and
  frays `cohesion` proportionally.
- `phase_institutions` `garrison_target = institution_target_level(unrest +
  dispossessed_permille, …)` — chronic unrest pushes the garrison up.
- `phase_diplomacy` `N12 threshold` reads unrest for rivalry escalation.
- Inspector unrest overlay (`EntityTint` warm).

### 5.3 `SocialGraph` (positive ties) → `phase_cohesion` `micro_cohesion_delta`

Per `EMERGENCE_WIRING_PATCHPLAN.md §3.6` and the existing pattern at
`engine.rs:2161-2188`, `micro_cohesion_delta(&self.world)` re-scans for
positive tie density within clusters; bounded at `MICRO_BIND_CAP = +12` /
`MICRO_FRAY_CAP = -18` per tick.

**Producer end:** `emergence_social` at `emergence.rs:249-288` applies
interaction events; `decay_social_graph` reduces old ties. **Lag:** single
tick, same as mood/unrest.

**Downstream consumers of `state.cohesion`:**
- `phase_stratification` `dispossession_target_permille(spread, cohesion)`
  reads cohesion as a *decay term* on dispossession.
- `phase_research` `cohesion_research_bonus_permille(stale)` reads cohesion
  (stale-allowed on first tick) for the per-tick research bonus.
- `phase_cohesion` itself (recurrent via `awakening_cohesion_gain` from
  `apply_awakening_coupling`).
- Legends feed: high-cohesion eras → Founder/Golden Age entries.

### 5.4 `SocialGraph` (cross-cluster ties) → `phase_diplomacy` `social_bridge` (new)

Per [`psyche-social.md §4.3`](psyche-social.md), `DiplomacySignal` gains a
**social bridge term**: aggregate cross-cluster ties, weighted by degree
centrality, biasing `trade_volume` (positive ties) or `resource_competition`
(negative ties / defection). The existing `RelationKind` thresholds and
EMA folding are untouched — the bridge term is additive input only.

**Producer end:** `emergence_social` per-tick tie accumulation. **Lag:**
single tick. **No new authored state** in `diplomacy.rs`.

### 5.5 `psyche.beliefs` → `CultureProfile.traits` (re-aggregation, close-the-loop)

The belief ↔ culture loop is **the most subtle** of the upward arrows.
Per `psyche-social.md §3.3` and the existing `emergence_psyche` at
`emergence.rs:328-470`:

```
per tick:
  psyche.beliefs = mix(psyche.beliefs, exposure, BELIEF_LR * sociability) + jitter
  cluster_cultures[c].traits = mean of cluster members' psyche.beliefs (re-derived)
```

This means **cluster culture is a measurement of the agents' beliefs**, not
the source of them. The source is the graph-weighted exposure to *other
agents' cluster cultures*. A cluster with internal heterogeneity (agents
connected to outside clusters) sees its culture move toward the outside
contact's culture *via the agents whose beliefs already moved* — a second-order
diffusion.

**Charter check.** The first-touch seed for `cluster_cultures` is
`cluster_id_derived_colours` (`emergence.rs:204-211`), a deterministic
non-seed-dependent value. Two agents with identical genomes spawning in
different clusters (or in the same biome of different seed selections) start
with *different* culture seeds — because the **cluster id** is different. The
seed does not author the culture. The cluster id (an emergent identifier) does.
This is the anti-hardcoding contract from `SEEDS_EMERGENCE_FLOW.md §5.2`.

### 5.6 Sentience threshold crossings → `apply_awakening_coupling`

Per `emergence.rs:539-546` and `EMERGENCE_WIRING_PATCHPLAN.md §3.11`:

```text
last_sentience = list of SentienceEvent from emergence_genetics_sentience
awakenings = last_sentience.len()
add_belief(awakening_belief_gain(awakenings))    // bounded per tick
add_cohesion(awakening_cohesion_gain(awakenings)) // bounded per tick
```

The sentience crossing mints a **bounded pulse** into macro state. It does
**not** mutate `Psyche` directly — but it does mutate `state.belief` and
`state.cohesion`, which are read back into:

- `phase_belief` next tick (one-tick lag, recursive `belief → belief` feedback).
- `phase_cohesion` next tick (one-tick lag, `awakening_cohesion_gain` re-applied).
- `phase_social_mood` indirectly via `cohesion_delta` and `cohesion_unrest_damp`.
- Legends: each crossing is a `sentience` feed event; cumulative crossings on
  the same agent trigger `Founder` / `Sage` promotion (out of scope here).

**Charter check.** `awakening_belief_gain(awakenings)` is a *pure function of
the count of threshold crossings this tick*. It is not an authored "if lineage
is human then belief +5" — it is `min(MAX_AWAKENING_BELIEF_PER_TICK, base × awakenings)`.
The seed (or lineage) is never named.

---

## 6. Downward causation — what changes the agent's mind

The reverse arrows: how macro state feeds back into per-agent psyche, without
ever assigning a psyche field directly.

### 6.1 Physics & climate → `Needs` → `update_mood`

```
Layer-0 (materials/energy/fluids/climate)
    → civ-needs::tick (decay + regen + sickness + death)
    → update_mood (psyche.rs:185)
    → mood.valence ∈ [-1, 1]
```

This is the **longest causal chain** in the engine and the most important
*downward* arrow. Specifically:

- `crates/laws` (chemistry/materials/energy conservation) constrains which
  resources exist and at what cost.
- `crates/planet` (climate/hydrology) determines aridity, flood risk, thermal
  comfort.
- `crates/engine/src/disasters.rs` (planned per ADR-020 §3.4) injects
  emergent earthquakes/floods/storms/volcanism/plague (see
  [`DISASTER_EMERGENCE.md`](DISASTER_EMERGENCE.md)).
- `crates/needs/src/lib.rs::tick` decays every need toward zero at per-need
  rates; sustained deprivation drains `Health.integrity` → sickness → death.
- `crates/engine/src/emergence.rs::emergence_psyche` calls `update_mood`
  with the post-tick `Needs`, computing `need_valence` and feeding the
  `target_val` formula.

**Consequence.** An agent's mood is a *direct measurement* of the world's
harshness applied to that agent's needs. A drought drives the whole population's
mood negative; a flourishing season restores it. **No author wrote a "happy
season" event.** The mood emerged because the need decay + satisfaction
dynamics *are* the world.

### 6.2 Threat pressure → `arousal`

`update_mood` sets `mood.arousal = (threat_pressure + |Δneeds| + 0.25 × |event_term|).clamp(0, 1)`
where `threat_pressure = max(0, 1 - life_needs.safety)`. The arousal term
therefore tracks *immediate physical danger* plus *acute deprivation shocks*
plus *acute social events* (fight / betrayal).

**Charter check.** Threat comes from the world's material/agent state via
`life_needs.safety`, which decays per `DecayRates.safety = 0.002` when no safe
place is occupied. No authored `is_in_combat: bool`.

### 6.3 `phase_belief`/`phase_unrest`/`phase_cohesion` → `phase_social_mood` → mood aggregate

The macro state feeds back into the **mean** mood aggregate via
`phase_social_mood`. Individual agents don't read macro state directly — the
emergence of mean mood is an aggregation, and the per-agent mood update path
is independent. The "feedback loop" is therefore:

```
macro (belief, unrest, cohesion)  ──►  mean_mood  ──►  state.society_mood
                                                          │
agent update:  update_mood(needs, temperament, …)         │  (next tick)
                                                          ▼
                                                  mean_valence_scan
```

The two ends (per-agent mood + macro mood) are coupled only through `Needs`,
which is the substrate.

### 6.4 `phase_diffusion` (technology adoption) → `Needs.satisfaction`

`phase_diffusion` (the tail of the tick DAG) propagates wardrobe / tools era
adoption via `DiffusionParams` S-curve (`crates/diffusion`). The `Tools` and
`Wardrobe` components directly satisfy `safety` and `shelter` needs (the
existing `daily_path.rs` POI model). Therefore an agent living through a
technology-era transition sees *sustained* need satisfaction, which feeds
through `update_mood` to higher baseline `valence`.

**Charter check.** The diffusion parameters (S-curve inflection, adoption
ceiling) are data in `DiffusionParams`; the *capability* of an era to satisfy
a need is measured by `daily_path.rs::score_poi`, not authored as "Era 3
satisfies Safety".

---

## 7. LOD cost — how psyche survives far-from-camera aggregation

The psyche + social machinery is the **most expensive per-agent state** in
the engine (a `Psyche` + a `SocialGraph` + a `CultureProfile` reference per
agent). Per `psyche-social.md §5` and the existing `LodTier` policy at
`crates/agents/src/lib.rs:240-247`, the contract is:

| Tier | Cadence | Per-agent cost |
|------|---------|----------------|
| Hot | every tick | full `update_mood` + `nudge_temperament` + `update_beliefs` + `apply_social_event` + `decay_social_graph` |
| Warm | every 4 ticks | coarse mood update; social events batched; decay on a fixed schedule |
| Cold | every 16 ticks | `state.society_mood` is the cluster mean; per-agent `Psyche` frozen; `CultureProfile` aggregated at cluster level only |

**Cold rehydration.** On promote-to-Hot, an agent's `Psyche` is reconstructed
from a compact summary: `seed_value(needs) × scale(society_mood) + per_cluster_culture_residual`.
Per `EMERGENCE_WIRING_PATCHPLAN.md §3.7` the mood mean is the source of truth,
not the individual `valence`. This is safe because mood is *slow-moving* and
the cold region is observed to move in concert with its cluster.

**Cap constants (already grep-guarded).**
- `MAX_TIES = 150` (`crates/agents/src/social.rs:13`).
- `MAX_MOOD_STEP_PER_TICK = 0.05` (`EMERGENCE_WIRING_PATCHPLAN.md §3.7`).
- `MAX_AWAKENING_BELIEF_PER_TICK = 50` (`crates/engine/src/emergence.rs`).
- `MAX_AWAKENING_COHESION_PER_TICK = 10` (`crates/engine/src/emergence.rs:2713`).
- `MAX_BELIEF_PER_TICK = 200` (`EMERGENCE_WIRING_PATCHPLAN.md §3.4`).
- `MAX_RISE` per `phase_unrest` delta term (≤200 composed per tick, per
  ADR-020 §3 table).

**Perf budget.** Per ADR-020 §4: 11 new macro phases add 0.6–1.2 ms at
5,000-agent populations; total tick moves from ~2 ms to ~3.2 ms; the 4 ms
tick-budget guard at `crates/engine/src/perf.rs` is enforced.

---

## 8. Memory — what we store, what we don't

The brief mentions **memory** alongside drives/mood/belief-formation.
The existing `crates/agents/src/psyche.rs` does **not** model episodic memory
explicitly — it models only:

- `psyche.maturity` (a single scalar, cumulative).
- A ring-buffer of recent `SocialEvent` affinity deltas (planned; the
  `event_term` in `update_mood` reads the buffer, but the buffer itself
  has no API surface today).

The charter says memory **emerges**, not authored. The following
design proposal (NOT in scope for the first implementation PR — see §10)
specifies how episodic memory falls out of the existing substrate:

### 8.1 Three memory tiers (emergent, not authored kinds)

| Tier | Granularity | Source | Lifetime | Where it lives |
|------|-------------|--------|----------|----------------|
| **Subconscious pulse** | 1 float `arousal`, 1 float `valence` | `update_mood` | seconds-to-minutes (decay with `event_term` half-life ~1 day) | `Psyche.mood` (existing) |
| **Trait echo** | per-trait nudge accumulated over years | `nudge_temperament` | years-to-lifetime | `Psyche.temperament` (existing) |
| **Episodic trace** | sparse landmark events (births of children, deaths of friends, sentience threshold crossings, first-cooperation-with-X) | planner proposal — see §10 | years-to-lifetime | `civ_legends::SagaGraph` (existing, **but with new ingestion**) |

The episodic trace is **not a per-agent field**; it lives in the *saga graph*
shared across the simulation, and each agent has a *sparse index* into it
(recent `RawSimEvent`s the agent witnessed, capped at e.g. `MAX_AGENT_EPISODES = 50`).
This is the same pattern as `Tie { other, last_seen }`: bounded, decaying,
sparse.

### 8.2 Why memory is not an authored kind list

A common temptation is to define `enum Memory { Family, Betrayal, Achievement, Trauma }`.
This violates the charter. Instead, *every memory is a `RawSimEvent` reference*
(legend type, tick, participants, magnitude), and the "kind" of memory is
**derived at query time** by which event type it is.

### 8.3 Belief-formation IS the long-term memory

`psyche.beliefs` is the *consolidated memory* — the slow-moving residue of
countless `update_beliefs` updates. Agents don't "remember" a specific
Cooperation event from 1000 ticks ago; they *believe* what they believe
because the beliefs vector has been nudged by culture exposure weighted by
the social graph. This is **belief-formation as memory compression** and it
falls out of the substrate automatically.

### 8.4 Anti-regression rule

No `enum MemoryKind`. No `agent.memory: Vec<Memory>` field. The agent's
"memory" is the joint distribution of:
- `Psyche.mood` (short-term),
- `Psyche.temperament` (medium-term),
- `Psyche.beliefs` (long-term),
- `SocialGraph` (relational long-term),
- Optional sparse episodic index into `SagaGraph` (out of scope here, see §10).

---

## 9. The four anti-hardcoding rules (audit checklist)

Per `SEEDS_EMERGENCE_FLOW.md §9`, the same audit applies to psyche. Any
code change touching psyche must pass all four:

### 9.1 No-id-routing
> No code path may branch on `SeedDefinition::id`, `NamedSeed::Ardani/...`,
> or any other named content when reading or writing `Psyche`, `SocialGraph`,
> or `CultureProfile`.

The current code is clean: `psyche_from_dna` reads only `Dna` and
`PsychGenomeProfile` (a projection rule). No `match seed.id { "ardani" => ... }`
exists anywhere in `crates/agents/src/psyche.rs` or `crates/agents/src/social.rs`.

### 9.2 Measurement-only
> All reads from a seed are reads of the *result* of the seed (Dna bytes,
> phenotype behavior weights, cognition score), never reads of the seed
> itself.

`psyche_from_dna` reads only `Dna`. `emergence_genetics_sentience` reads
`Dna` via `evaluate_sentience`. No `SeedDefinition` reference crosses into
the psyche layer.

### 9.3 Dial-as-knob
> The divergence dial scales a substrate rate (mutation rate per byte); it
> does not gate, threshold, or branch on its own.

`psyche.rs` does not consult `divergence` at all — the psyche is computed
from the *post-divergence* `Dna`. The dial is upstream of psyche, never
inside it.

### 9.4 No-authoring-in-emergence
> No emergence phase may have an authored taxonomy of personality types,
> memory kinds, belief kinds, or relation kinds. All such labels are pure
> functions of emergent state.

The current code: `RelationLabel` is derived at query time (`relation_label`,
`social.rs:113-127`). No `enum Personality`. No `enum Memory`. The
inspector-side labels (e.g. "provision / security / affiliation / novelty"
for the four drive axes per `psyche-social.md §6.1`) are *runtime labels*
computed from observed behaviour correlations — never stored kinds.

---

## 10. Open questions / future work

| Question | Where it lands | Effect on this doc |
|----------|----------------|--------------------|
| Should `Psyche` gain an explicit `episodic_index: SmallVec<[RawSimEventRef; 50]>` field? | TBD — open per §8.4 above | If yes, add to §3 as a fifth sub-layer; the implementation reuses `civ_legends::SagaGraph::query_for_agent()`. |
| Should `apply_awakening_coupling` also bump `psyche.beliefs[i]` directly? | TBD | **No** in default design — the existing path (`add_belief` → `state.belief` → `phase_social_mood` → mean mood → re-scan `phase_social_mood`) is closed-loop already. Direct bumps would be authored outcomes. |
| Should `phase_diplomacy` `social_bridge` term be a per-faction aggregate or per-cluster-pair? | TBD | Per-cluster-pair keeps the resolution matching the social graph; per-faction aggregates lose signal. Recommend per-cluster-pair. |
| Should `MAX_TIES = 150` be a function of cognition score? | TBD | Charter-clean variant: a tool-using + language-capable lineage might support more ties; a memory-poor lineage supports fewer. This is a *measurement* of the cognition bytes, not a hardcoded cap per species. |
| Should `Psyche.mood` track `needs.water` and `needs.rest` explicitly (currently the existing `update_mood` averages over the 4-component `Needs`)? | TBD — `crates/needs::Needs` has 6 components, `civ_agents::Needs` has 4 | A reconciliation PR should pick the 6-component version and call `Needs::sated()` etc. consistently. Out of scope here; tracked in `EMERGENCE_AUDIT`. |

---

## 11. Implementation WBS (NOT done here)

The doc-only deliverable. Each item is disjoint from the existing
`psyche-social.md §9` WBS — that doc covered the **types and updates**; this
doc's WBS covers the **wiring into macro-phases + inspector** which the
earlier doc left for follow-up.

1. **P1 — wire `phase_social_mood` `mean_valence` read.** `crates/engine/src/emergence.rs` `phase_social_mood` body: scan `psyche.mood.valence`, compute mean, clamp delta to `MAX_MOOD_STEP_PER_TICK`. Add the 3-test minimum (ADR-011).
2. **P2 — wire `phase_unrest` `agent_misery_unrest`.** Same scan pattern, contributes `mean(-valence) × W_MISERY`. Add `MAX_MISERY_UNREST_PER_TICK` const to grep-guard.
3. **P3 — wire `phase_diplomacy` `social_bridge`.** Cross-cluster tie scan in `crates/engine/src/diplomacy.rs` (or wherever diplomacy aggregation lives). Bias `trade_volume` (positive ties) and `resource_competition` (defection ties). No new `RelationKind` thresholds.
4. **P4 — close the belief ↔ culture loop with a re-aggregation pass.** After `update_beliefs` in `emergence_psyche`, re-derive `cluster_cultures[c].traits = mean of cluster members' beliefs` (centroid with `[0,1]` clamp). Document the existing drift in `culture.rs` as the **second pass** of the loop.
5. **P5 — inspector "Mind" + "Relationships" panels + 3 overlays.** Per `psyche-social.md §6`; the three overlays (`SocialTies` Gizmo, `MoodContentment` EntityTint, `BeliefField` LatticeRecolor) register into `info_views.rs`. Each carries the "emergent — derived from genome/needs" provenance note.
6. **P6 — episodic memory index (out of scope here, listed in §8.4/§10).** A `RawSimEventRef: { event_id, tick, salience }` small-vec per agent, populated from `civ_legends::SagaGraph::query_for_agent()`. Decays on the same schedule as `SocialGraph`. **Deferred to a follow-up PR.**

Each PR: disjoint files, xDD-first (tests/AC before impl), wraps existing
`culture.rs`/`genetics` helpers rather than hand-rolling.

---

## 12. Acceptance criteria

| AC | What it asserts |
|----|-----------------|
| **AC-PSY-1** | Two siblings (recombined DNA) have correlated but non-identical drives/temperament; two unrelated agents are uncorrelated. *(FR-CIV-PSYCHE-001/002 — already tested in `crates/agents/src/psyche.rs::genetics_projection_is_bounded_and_sibling_sensitive`.)* |
| **AC-PSY-2** | `mood.valence` responds to `Needs` within bounded window; high-reactivity agents move faster; chronic deprivation shows up in `state.society_mood`. *(FR-CIV-PSYCHE-010 + `phase_social_mood` cap test.)* |
| **AC-PSY-3** | An agent whose strong ties hold a minority belief drifts toward the minority, diverging from its cluster centroid. *(FR-CIV-PSYCHE-020.)* |
| **AC-PSY-4** | `Interaction::Cooperated` raises affinity + trust; `Defected` sharply drops trust; absence decays familiarity toward 0 while kinship persists. *(FR-CIV-PSYCHE-030/031, already covered in `crates/agents/src/social.rs::decay_reduces_old_ties_and_keeps_kinship`.)* |
| **AC-PSY-5** | A→B and B→A ties can differ in sign and magnitude (asymmetry). *(FR-CIV-PSYCHE-030, already covered.)* |
| **AC-PSY-6** | Same `Needs` snapshot + different `drives` ⇒ different `score_poi` ranking; friends co-locate via `social_pull`. *(FR-CIV-PSYCHE-011.)* |
| **AC-PSY-7** | A cluster splits when intra-cluster rivalry/belief-distance exceeds the join payoff; coalesces on shared belief + affinity. *(FR-CIV-PSYCHE-032.)* |
| **AC-PSY-8** | Dense positive cross-cluster ties move the pair toward Trade/Alliance; dense defection toward Rivalry/War, via the social-bridge term. *(FR-CIV-PSYCHE-033.)* |
| **AC-PSY-9** | `grep -nr 'enum Personality\|enum MemoryKind\|enum EmotionKind' crates/agents crates/engine` returns **0 hits** (charter). |
| **AC-PSY-10** | Per-agent `psyche + social` cost ≤ O(MAX_TIES); Cold agents run cluster-aggregate only; promotion rehydrates within ≤ 1 tick of warm-up. *(FR-CIV-PSYCHE-005/006.)* |
| **AC-PSY-11** | `apply_awakening_coupling` reads only `last_sentience.len()`; no second world scan; `MAX_AWAKENING_*` caps enforced. *(Per `EMERGENCE_WIRING_PATCHPLAN.md §3.11`.)* |
| **AC-PSY-12** | `mean_valence` scan in `phase_social_mood` and `agent_misery_unrest` share the same scan pattern (single world iteration per tick); no double-counting across phases. *(Per `EMERGENCE_WIRING_PATCHPLAN.md §3.7`.)* |

---

## 13. References

- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — the single governing principle.
- [`docs/design/species-sentience.md`](species-sentience.md) — the lineage-level ladder (§2 above is a recap).
- [`docs/design/psyche-social.md`](psyche-social.md) — the per-agent data model; this doc complements it with the upward/downward causation story.
- [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md) — cluster-level culture drift; this doc's §5.5 closes the belief ↔ culture loop with this layer.
- [`docs/design/SEEDS_EMERGENCE_FLOW.md`](SEEDS_EMERGENCE_FLOW.md) — what seeds touch and don't touch; §3/§5 above cite this for the anti-hardcoding audit.
- [`docs/design/EMERGENCE_WIRING_PATCHPLAN.md`](EMERGENCE_WIRING_PATCHPLAN.md) — the macro-phase DAG this doc feeds into via §5/§6.
- [`crates/agents/src/psyche.rs`](../../crates/agents/src/psyche.rs) — the data model + update rules for `Psyche`.
- [`crates/agents/src/social.rs`](../../crates/agents/src/social.rs) — the `SocialGraph` + `Interaction` machinery.
- [`crates/agents/src/culture.rs`](../../crates/agents/src/culture.rs) — `CultureProfile` drift + `mix_trait_vectors`.
- [`crates/agents/src/cluster.rs`](../../crates/agents/src/cluster.rs) — cluster formation + `MembershipPayoff`.
- [`crates/agents/src/diplomacy.rs`](../../crates/agents/src/diplomacy.rs) — diplomacy matrix + `DiplomacySignal`.
- [`crates/needs/src/lib.rs`](../../crates/needs/src/lib.rs) — 6-component `Needs` + decay/health pipeline.
- [`crates/genetics/src/sentience.rs`](../../crates/genetics/src/sentience.rs) — `CognitionTraitProfile` + `evaluate_sentience`.
- [`crates/engine/src/emergence.rs`](../../crates/engine/src/emergence.rs) — `phase_emergence` orchestrator; the producer of all `last_*` buffers.
- [`crates/engine/src/engine.rs`](../../crates/engine/src/engine.rs) — `Simulation::tick` DAG; the wiring context.
- [`crates/legends/`](../../crates/legends/) — `SagaGraph` + `RawSimEvent` for the episodic-trace proposal in §8.
- [`docs/traceability/fr-3d-matrix.md`](../traceability/fr-3d-matrix.md) — the FR matrix this doc unblocks.

**Traceability IDs (new in this doc).** `FR-CIV-AGENTS-EMERGENCE-001` (mean
valence scan shared between `phase_social_mood` and `agent_misery_unrest`),
`FR-CIV-AGENTS-EMERGENCE-002` (social bridge term in `phase_diplomacy`),
`FR-CIV-AGENTS-EMERGENCE-003` (belief ↔ culture re-aggregation pass),
`FR-CIV-AGENTS-EMERGENCE-004` (episodic memory index, deferred). Existing
`FR-CIV-PSYCHE-*` from `psyche-social.md` cover AC-1 through AC-11.