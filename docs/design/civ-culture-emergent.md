# CIV-CULTURE: Emergent Culture + Language + Script — Design Spec

> **Status:** Design (planner-only, 2026-06-14). No implementation code in this document.
> **Spec ID:** `civ-culture-emergent` | **Epic:** E2 (emergence), layered on E3 (psyche) and E5 (language) from `emergent-systems-spec.md` §3 | **Pattern ancestor:** [`civ-003-emergent-lifecycle.md`](civ-003-emergent-lifecycle.md) (charter constraint + read-only classifier + shared-gradient coupling + criticality knobs + phased WBS).
> **Governing canon:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md), [`docs/design/emergent-systems-spec.md`](../design/emergent-systems-spec.md) §2.4 + §3, [`docs/design/emergence-dashboard.md`](../design/emergence-dashboard.md), [`docs/design/psyche-social.md`](../design/psyche-social.md), [`docs/design/civ-003-emergent-lifecycle.md`](../design/civ-003-emergent-lifecycle.md), [`docs/design/civ-economy-emergent-markets.md`](../design/civ-economy-emergent-markets.md), [`docs/design/polities-markets.md`](../design/polities-markets.md).
> **Code substrate (read-only inputs):** `crates/agents/src/culture.rs` (`CultureProfile`, `TraitVector`, `ContactEdge`, `drift_populations`, `mutate_traits`, `mix_trait_vectors`, `cultural_distance`, `language_distance`), `crates/agents/src/psyche.rs` (`Psyche`, `beliefs: [f32; PSYCHE_DIM]`, `belief_culture_exposure`, `update_beliefs`, `nudge_temperament`, `psyche_from_dna`, `PSYCHE_DIM = 4`), `crates/agents/src/social.rs` (`SocialGraph`, `Tie`, `apply_social_event`, `decay_social_graph`, `relation_label`, `MAX_TIES = 150`), `crates/agents/src/cluster.rs` (`ClusterId`, `ClusterMember`, `should_join`, `should_leave`, `MembershipPayoff`), `crates/civ-emergence-metrics/src/dashboard.rs` (`EmergenceDashboard` with `ideology_homophily`, `cluster_entropy`, `psyche_stability`, `sentience_fraction`, `diplomacy_tension`), `crates/engine/src/emergence.rs` (`phase_emergence` orchestrating `emergence_culture` → `emergence_social` → `emergence_psyche`).
> **Traceability:** FR-CIV-CA-001..010 (carry-overs from `emergent-systems-spec.md` §3 E2.4 — fully consumed), FR-CIV-LANG-001..008 (carry-overs from E5.1-E5.3 — *superset*; this spec defines the macro read-out + bidirectional coupling; the per-crate internals stay in `emergent-systems-spec.md` E5.1/E5.2), FR-CIV-EMERGENCE-001 (micro-driver → macro-pattern mapping), FR-CIV-EMERGENCE-002 (shared-gradient coupling — no API edges), FR-CIV-PSYCHE-020 (beliefs emerge from culture sampled via graph — already in `psyche-social.md` §3.3; this spec defines the cluster-aggregation feedback loop that closes it).

---

## 0. Charter constraint

The Civis Emergence Charter forbids hardcoding life / sentience / society / culture / language / religion as authored state machines or enums (`emergence-charter.md` §"Layer 1+ — What EMERGES"). The current `crates/agents/src/culture.rs` already obeys the substrate side: `CultureProfile` is a 4-axis `TraitVector`, `drift_populations` mutates + diffuses + creolizes from contact edges, `mutate_traits` adds bounded jitter, and `creole_threshold` triggers a separate blend pass for diverging languages. The psyche side is also substrate-correct: `Psyche::beliefs: [f32; PSYCHE_DIM]` drifts from a graph-weighted `belief_culture_exposure` via `update_beliefs` (`psyche.rs:202-218`), and `nudge_temperament` provides slow experiential plasticity gated by maturity.

What is **missing** — and what this spec specifies — is the *macro read-out layer* on top of these micro-dynamics. Today the engine has no projected function for "dialect," "value-system," "ritual," or "script" because those are *labels* on a continuous state, not fields on a citizen. The micro-substrate already produces the substrate patterns those labels point at; the missing work is (a) defining the *pure classifier functions* that map the live state into observable macro-labels for the dashboard, legends engine, inspector, and 3D client overlays, (b) defining the *bidirectional shared-gradient coupling* to lifecycle / markets / diplomacy through the *existing* substrate channels (no new API edges), (c) defining the *criticality knobs* that keep the system on the edge of chaos, and (d) defining the *observable metrics* that surface the macro phenomena on the Emergence Dashboard.

The charter test applied to every field below: *can this emerge from Layer-0 rules?* Yes — dialects from `language_distance` between drift states, value-systems from `belief_culture_exposure` aggregates, rituals from repeated `apply_social_event(Interaction::Cooperated)` patterns, scripts from a *derived* orthography projection over the language vector (no authored script enum). Nothing here is a `Language`, `Script`, `Religion`, or `Ritual` enum on any agent.

This spec **does not** re-implement `CultureProfile::drift_populations` (the substrate is correct); it does not re-implement `belief_culture_exposure` (the substrate is correct); it does not write a new ASCA-style diachronic engine (the per-crate plan in `emergent-systems-spec.md` §2.4 / E5.0-E5.3 is the implementation home, and this spec consumes its outputs as **derived state**). What it *does* add is the macro read-out, the shared-gradient coupling, the criticality knobs, and the dashboard metrics.

---

## 1. Core emergence model

### 1.1 Macro culture is a measurement, not a stored state

There is no `dialect: DialectId`, no `script: ScriptId`, no `religion: ReligionId`, no `value_system: ValueSystemId` field on any agent, cluster, or polity. The macro phenomena that humans name "dialect," "script," "ritual," "value-system," and "religious tradition" are all *labels* applied at query time to a continuous state, by pure classifier functions over the existing substrate:

| Continuous driver | Crate + field | What it does |
|---|---|---|
| Per-cluster culture vector | `CultureProfile::traits: TraitVector` (`agents/src/culture.rs:19-30`) | 4-axis trait vector in `[0,1]`, mutates per drift step |
| Per-cluster language vector | `CultureProfile::language: TraitVector` (`culture.rs:24`) | Separate 4-axis drift vector — language and culture are *coupled* but not identical, so dialects can survive a culture shift (creole) and a culture can survive a language shift (cognate borrowing) |
| Contact graph | `ContactEdge { from, to, weight }` (`culture.rs:46-54`) | Population-pair contact intensity in `[0,1]`; built by the existing `phase_emergence` from cluster co-location (currently the fixed `0.15` in `emergence.rs:186` is the placeholder) |
| Per-agent belief vector | `Psyche::beliefs: [f32; PSYCHE_DIM]` (`psyche.rs:75`) | 4-axis personal sample of the agent's exposure through its graph ties, updated by `update_beliefs` (`psyche.rs:202-218`) using `belief_culture_exposure` (`psyche.rs:220-246`) |
| Kinship insulation | `CultureProfile::kinship: f32` (`culture.rs:29`) | Higher kinship ⇒ less drift from outside contact; emergent from intra-cluster mean kinship |
| Cluster membership | `ClusterMember` + `ClusterId` (`agents/src/cluster.rs`) | The existing aggregator for "who counts as in the same culture"; emergent from co-location + graph |
| Per-agent social graph | `SocialGraph::ties` with `Tie { familiarity, affinity, trust, kinship, last_seen }` (`social.rs:60-75`) | The channel through which culture reaches the individual (the belief update samples the cultures of the agent's strongest ties, not the cluster average) |
| Ideology bin histogram | `ideology_homophily(ideologies, bin_width)` (`civ-emergence-metrics/src/dashboard.rs:125-156`) | Population-wide ideological histogram; the existing dashboard metric that this spec *extends* with new fields |
| Co-located cooperation events | `apply_social_event(Interaction::Cooperated)` + `apply_social_event(Interaction::Coexisted)` (`social.rs:158-176`) | The substrate for "ritual" — repeated synchronous cooperation at a location, observed as a frequency pattern, not scripted as a `Ritual::X` enum |

### 1.2 The macro read-out: pure classifier functions

The macro read-out is a *family of pure functions* over the live substrate. Each takes the world / cluster / graph state and returns a labeled enum for the dashboard / legends / inspector. The labels are **derived**, not stored, and the function set is **additive** — calling a classifier is free, never mutates state, and can be added incrementally without touching the substrate:

```text
// DIAGNOSTIC — pure read-out, never stored on a cluster
fn classify_dialect(
    language_vectors: &BTreeMap<ClusterId, TraitVector>,
    distance_threshold: f32,
) -> BTreeMap<ClusterId, DialectLabel>          // 'dialect_A' / 'dialect_B' / ... (string IDs, not enum variants)

// VALUE-SYSTEMS — pure read-out over a cluster's belief centroid
fn classify_value_system(
    cluster_belief_centroid: [f32; PSYCHE_DIM],
    axes: &ValueAxesProfile,
) -> ValueSystemLabel                           // 'individualist-collectivist' axis readings; never a fixed enum

// SCRIPT — pure read-out over a cluster's language vector + an emergent orthography projection
fn project_script(
    language: TraitVector,
    phonotactic_profile: &PhonotacticProfile,
) -> ScriptSignature                            // a derived glyph-stem signature; renderable as a vector-stroke atlas; never an enum

// RITUAL — pure read-out over a sliding window of social events at a location
fn classify_ritual(
    events: &[SocialEvent],
    location: Position3d,
    window_ticks: u32,
) -> RitualLabel                                // 'greeting' / 'feast' / 'mourning' / 'pilgrimage' — emergent frequency pattern, not authored

// RELIGION / TRADITION — pure read-out over a cluster's belief centroid + script signature
fn classify_religion(
    cluster_belief_centroid: [f32; PSYCHE_DIM],
    cluster_script: ScriptSignature,
    cluster_ritual_frequencies: &BTreeMap<RitualLabel, f32>,
) -> TraditionLabel                             // 'fire-cult' / 'river-cult' / 'ancestor-cult' — tags from the substrate, not a fixed taxonomy
```

The signatures are illustrative — the *function set* is the contract, not the exact identifier strings. Each function is **pure** (same input → same output) and **read-only** (no ECS mutation), so they are *safe to call from any of the read-out surfaces* (dashboard panel, legends, inspector, 3D client overlay, replay viewer). This is the same pattern as `civ-003-emergent-lifecycle.md` §1.2 `classify_lifecycle` and `civ-economy-emergent-markets.md` §1.2 `emergent_clearing_price` + `select_numeraire`.

### 1.3 The script signature: vector strokes, not authored glyph atlases

A "script" in Civis is *not* an authored font. It is an **orthography projection** derived from the cluster's `CultureProfile::language` vector through a `PhonotacticProfile` (a small data table mapping phoneme-feature combinations to stroke-stem counts and curvature biases — the per-civ generalization of the Lexifer/gleb inventory heuristics called out in `emergent-systems-spec.md` §2.4). The output `ScriptSignature` is a small byte-vector — say 16–64 bytes — that the renderer feeds to a stroke composer (the `ab_glyph` / `cosmic-text` wrapping in `emergent-systems-spec.md` §2.4) to materialize a runtime TTF. Two clusters with `language_distance > script_threshold` project to **different** signatures; the renderer never consults an authored glyph atlas for civ-native text.

The per-cluster `ScriptSignature` is *cached* on `CultureProfile` as a *derived* field (one byte-vector, not a separate enum) — computed lazily on first read, invalidated when `language_distance` from the prior signature exceeds `script_invalidation_threshold` (default 0.15). This is the only "stored macro" field anywhere in the system, and it is *derived*, not authored. The cache is invalidated by the same drift loop that mutates the language vector, so a stored signature can never outlive a substantial language change.

### 1.4 What produces each macro phenomenon

**Dialects** are the *partition* of clusters by pairwise `language_distance(a, b) >= dialect_split_threshold` (default 0.35). A dialect boundary is a single-link cluster on the language-distance graph: clusters within `dialect_split_threshold` share a dialect, clusters above it form new dialects. New dialects are *measured* (the partition changes when drift changes distances) — never *declared* by code. The `culture.rs:91-93` `language_distance` function is the substrate. The `creole_threshold` path (`culture.rs:157-164`) is the substrate for two dialects *blending* into a creole when contact resumes — the third macro phenomenon, distinct from "language" and "dialect."

**Value-systems** are the *per-cluster belief centroid* projected through a `ValueAxesProfile` (a small data table mapping belief-vector axes to a small set of named axes — e.g. `(provision, security, affiliation, novelty)` — the runtime labels from `psyche-social.md` §6.1 that the inspector uses anyway). The centroid is a weighted mean of `Psyche::beliefs` over agents in the cluster, weighted by `Tie::familiarity` for the agent's intra-cluster ties (a loner does not anchor the cluster's value-system; a high-familiarity node does). The projection is a pure function — no cluster stores a `ValueSystem` field.

**Rituals** are the *frequency pattern* of `Interaction::Coexisted` + `Interaction::Cooperated` events at a `(location, time-window)`. A "ritual" emerges when the *same shape* of cooperation pattern repeats: e.g. `count(events with benefit > 0.5 in radius r over window w) >= ritual_repetition_threshold` and the events cluster on the *same* location bin across multiple windows. The classifier names the pattern by *which shape* the frequencies match (clustered-around-food = "feast"; clustered-around-corpse = "mourning"; clustered-around-crossing = "pilgrimage"; clustered-around-mate = "courtship"). The name is *descriptive* of the shape, not *prescriptive* of an enum — `emergence-dashboard.md` §3.4 explicitly accepts descriptive labels for novelty metrics, and the same logic applies to ritual frequency patterns.

**Scripts** are the *projection* of `CultureProfile::language` through the `PhonotacticProfile`, as in §1.3. Two clusters with different language vectors project to different signatures; the renderer composes a runtime TTF for the viewed cluster and re-uses it across agents in that cluster.

**Religious / mythic traditions** are the *composition* of (cluster belief centroid) + (cluster script signature) + (cluster ritual frequencies) into a single typed bundle. The classifier names the tradition by the *features* the bundle exhibits — the same descriptive-principle as rituals. The legends engine reads the `TraditionLabel` for narrative purposes (the cultural register of chronicles; per `emergent-systems-spec.md` §1.2 the cultural register carries chronicle + rumor chains, low-conscientness historian embellishment, sphere tags). Traditions are *not* stored on any agent; they are a query-time composition.

**Music** is *out of scope* for this spec — it is the domain of [`audio-direction.md`](../design/audio-direction.md) + `emergent-systems-spec.md` §2.5 (E6). The `MusicalTradition` is derived from `(culture_vec, available_materials)` and does not require culture-language co-emergence; it is a separate substrate read-out. This spec notes the boundary and stops.

---

## 2. Bidirectional coupling — the substrate gradient, not the API edge

The charter explicitly forbids one emergent layer calling another through an API boundary with no lag. The mechanism here is the same as `civ-003-emergent-lifecycle.md` §2 and `civ-economy-emergent-markets.md` §2: **shared conserved gradients with explicit lags**. The four gradients that carry culture-language coupling are: `CultureProfile` (per-cluster trait + language vectors, `culture.rs:19-30`), `Psyche::beliefs` (per-agent belief vector, `psyche.rs:75`), `SocialGraph::ties` (per-agent directed graph, `social.rs:60-97`), and `Tie::last_seen` (the contact-decay clock, `social.rs:74`). Nothing in this spec introduces a new edge between crates.

### 2.1 Culture/Language → Lifecycle / Markets / Diplomacy / Cluster (downward causation, no API call)

| Macro signal (read by what) | Effect on the layer | Mechanism (shared gradient, not call) |
|---|---|---|
| `CultureProfile::traits` shifts toward a high-provision vector (read by lifecycle / needs) | Agents in that cluster spend more cycles on production activities; `Needs::food` decay slightly faster because work is more distributed | The trait shift is observable on the cluster culture; the next `score_poi` (`daily_path.rs`) call from a cluster member reads its *own* `Psyche::beliefs` (which has drifted toward the cluster centroid via `belief_culture_exposure`), and that belief weights the food-seek score. No `culture.on_shift` callback exists. |
| `CultureProfile::language` distance between two clusters rises above `creole_threshold` (read by diplomacy) | Trade volume between the two clusters falls (mutual intelligibility cost) — an emergent transaction cost; a `creole` event may fire on the next contact | The language-distance metric is computed by the existing `creole_threshold` path in `culture.rs:157-164`; the diplomacy `DiplomacySignal::trade_volume` (`diplomacy.rs`) reads the *measured* distance and biases its payoff calculation. No direct culture→diplomacy API. |
| Cluster belief centroid shifts (read by markets) | Bid / offer composition in the cluster's locale shifts — agents bid more on the good whose axis matches the cluster's high-belief direction | The `record_bid_for_need` helper in `civ-economy-emergent-markets.md` §6 P3-A reads `Psyche::beliefs` indirectly through the actor's `Needs` vector; the new shared channel is `belief_culture_exposure` in the actor's psyche, which the actor's utility-AI uses to weight need priorities. No direct culture→market API. |
| Cluster `kinship` rises (read by `phase_emergence` contact-edge builder) | External contact-edge weights in `emergence_culture` are dampened by `(1 - kinship)`, so the cluster's `drift_populations` step mutates more and diffuses less | The `culture.rs:122` `* (1.0 - base[idx].kinship)` line is the existing substrate for this; no new code. The contact-edge builder in `emergence.rs:180-189` currently uses a fixed `weight: 0.15` — this spec's §6 Phase 2 will replace that with a kinship- and proximity-weighted edge builder. |
| Ritual repetition at a location exceeds `ritual_repetition_threshold` (read by legends engine) | The cultural register emits a `ritual_observed.v1` event into the saga graph; historians in the same cluster may pick it up and embellish (low conscientiousness → more embellishment) | The classifier in §1.2 emits the event into the existing legends `RawSimEvent` stream (`legends` crate, `SourceCrate::Agents`); legends ingestion and rumor hop mutation (`emergent-systems-spec.md` §2.6) is the downstream consumer. No direct culture→legends API. |
| `ScriptSignature` projected for a cluster (read by 3D / web clients) | The client requests the per-civ glyph atlas from the server; the server materializes the runtime TTF on first read and caches it | The renderer fetches through the existing `civis-lang` materialization path (`emergent-systems-spec.md` §2.4 E5.3); the cache lives in `civ-watch` and is keyed by `(cluster_id, ScriptSignature)`. No direct culture→renderer API. |

### 2.2 Lifecycle / Markets / Diplomacy / Cluster → Culture (upward causation with lag)

| Layer signal | Culture effect | Lag mechanism |
|---|---|---|
| A new faction split (diplomacy `ClusterPair` falls below `RelationKind::Neutral` threshold) | The two resulting clusters' `CultureProfile::contact` rises; `drift_populations` diffuses more aggressively between the two profiles (if they remain in the same `emergence_culture` contact graph); otherwise the contact-edge weight falls to zero and they diverge | 1 tick lag: diplomacy `apply_signal` runs after `phase_emergence` (`emergence.rs:121-127`); next tick the culture phase sees the new cluster IDs and the new edge weights |
| Mass mortality / migration (lifecycle FR-CIV-LIFE-003) | Cluster `kinship` falls because the surviving graph has fewer `Tie::kinship` entries per agent; `drift_populations` mutates faster (less kinship insulation); surviving agents' belief centroids move faster | 1 generation lag (~20 in-game years for a human cohort): the new kinship is recomputed from the surviving `Tie` set on the next `phase_emergence` tick; belief drift is then re-weighted |
| `Belief` distribution in a cluster becomes bimodal (schism signal) | Cluster `ClusterMember::should_leave` payoff may go negative for the minority belief-holders (per `psyche-social.md` §4.2); cluster splits via the existing `cluster_by_colocation` + `should_leave` path; the new cluster starts a new `CultureProfile` keyed by `ClusterId` | 1 tick lag: cluster reconcile runs in `phase_life`; next `phase_emergence` sees the new cluster IDs and creates new `CultureProfile` entries from their belief centroids |
| Trade volume rises between two clusters (markets `numeraire_share` / `cross_locale_arbitrage_opportunity`) | Contact-edge weight between the two clusters rises in the next `phase_emergence` contact-builder pass; cultural diffusion accelerates; the two `CultureProfile::language` vectors may cross `creole_threshold` and blend | 1 tick lag: trade signal is observed through the `Allocator::trades_log`; next `phase_emergence` reads `Allocator::trades_log` to compute contact weights |
| Agent welfare is high in a cluster (lifecycle / markets — needs above critical) | Agent's `belief_culture_exposure` stabilizes (the agent is not seeking new contact); cluster `CultureProfile::contact` may fall slightly; language drift continues via mutation only | Structural lag: 1 generation for belief-stability to register as cluster-contact-stability |
| Player freehand tool lays a script (e.g. writes a name in a player-language) on a cluster object (per `emergent-systems-spec.md` §1.2 cheat-lever layer) | The cluster's contact-edge weight to the player's faction rises sharply (the player is now a *kind* of cluster); a new tradition may form around the player-authored object | 1 tick lag: the freehand tool emits a `CulturalDiffusion` event into the watch bus; next `phase_emergence` reads the event and updates contact edges |

### 2.3 The dialect → polity coupling (read-out, not call)

A "polity" in `polities-markets.md` is a *measured* cluster of clusters, not an authored `Faction(u32)` field (per `emergence-charter.md` §"Polities / states — decentralized"). The dialect read-out (§1.4) is a *secondary signal* that the polity cohesion graph can use to weight `w_econ · payoff_if_coordinated` (`polities-markets.md` §1.2). Dialects that are mutually unintelligible increase polity-separation pressure; mutually-intelligible dialects reduce it. The polity cohesion graph already aggregates cluster-pair signals; adding a "dialect-distance × mutual-intelligibility" term is a one-line weighted add to the existing `w_*` coefficient table, not a new edge. (This spec defines the input; `polities-markets.md` owns the polity cohesion math.)

### 2.4 The market → numeraire-acceptability coupling (read-out, not call)

`civ-economy-emergent-markets.md` §3 defines `select_numeraire` as the good maximizing `trade_count(g) * sqrt(cross_good_acceptance(g))` over a windowed log. The new shared channel is *per-locale* cultural acceptability: a good that is *not* culturally associated with the locale's `value-system` will be accepted at a lower rate, so its `cross_good_acceptance` will fall, so the `select_numeraire` argmax may shift away from it. The shift is a `numeraire_shift.v1` event (existing in `civ-economy-emergent-markets.md` §6 P3-D) and is *caused* by the cultural read-out — but the market does not *call* the culture layer; it consumes the trade-frequency log, which is the substrate. No new API edge.

---

## 3. The cluster culture ↔ belief feedback loop (closing the existing open half-loop)

`psyche-social.md` §3.3 / §4.4 already specifies the *belief half* of the loop: `belief_culture_exposure` samples a cluster member's strongest ties, `update_beliefs` mixes the personal belief vector toward that exposure, and §4.4 promises "the belief loop feeds back into `culture.rs` aggregation." That feedback is the *missing half*, and this spec closes it.

The current `emergence_culture` (`emergence.rs:147-203`) maintains `cluster_cultures: BTreeMap<u64, CultureProfile>` keyed by `ClusterId`, with `traits` drifting independently of agent belief vectors. The closure: when `phase_psyche` runs `update_beliefs` for every agent, the new belief vector is a *personal sample* of the cluster culture, but the *cluster culture itself* is not updated from those samples. The current code stores the cluster culture as a pure-drift object that ignores the agents' lived belief.

This spec adds the *belief → cluster culture* feedback as a **read-only-aggregator pass** in `phase_emergence`. The aggregator is a *pure function* over the current `Psyche::beliefs` values of all agents in the cluster, weighted by `Tie::familiarity` (a loner with `familiarity = 0` in the cluster does not pull the cluster culture; a high-familiarity node does):

```text
// ADDED in emergence_culture, after the drift_populations call
let belief_centroid = weighted_belief_centroid(world, cluster_id, cluster_members);
// Pull the cluster's CultureProfile::traits toward the centroid by a small amount,
// bounded by the drift-populations mutation rate so feedback cannot dominate noise.
let pull_strength = belief_feedback_rate * (1.0 - profile.kinship);
profile.traits = mix_trait_vectors(profile.traits, belief_centroid, pull_strength);
```

The function `weighted_belief_centroid(world, cluster_id, members)` is a pure query; it iterates members, reads each agent's `Psyche::beliefs` and the familiarity of the member's strongest *intra-cluster* ties, and returns the weighted mean. The pull is bounded by `(1 - kinship)` (kinship-insulated clusters resist the pull, exactly mirroring the existing `culture.rs:122` kinship damping on inbound contact). No new field, no new component, no new edge.

This is the *only* mutation introduced by this spec into the existing substrate, and it is bounded: the pull strength is multiplied by `belief_feedback_rate` (default 0.04, see §4) and the pull is capped at the existing `mix_trait_vectors` clamp.

---

## 4. Criticality knobs — edge of chaos

All knobs concentrate in a new `CultureEmergenceParams` struct on `EmergenceState` (loaded from scenario RON, same pattern as `civ-003-emergent-lifecycle.md` §3 `LifecycleParams` and `civ-economy-emergent-markets.md` §4 `EmergentMarketParams`). Defaults target weak emergence (Class 4): the system is in a *culturally-varied, not-static, not-exploding* band — multiple stable dialects, multiple traditions, ritual frequency patterns visible in the event log, scripts that vary by cluster but not so rapidly that the renderer cache thrashes.

| Parameter | Type | Default | Effect on culture/language dynamics | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `culture_mutation_rate` | `f32` | `0.02` (existing in `emergence.rs:175,190`) | Per-tick jitter on `CultureProfile::traits` and `language` in `drift_populations` (`culture.rs:117-118`) | `0.0` (frozen culture; no drift) | `> 0.1` (cultures blur every tick; no stable identity) |
| `culture_diffusion_rate` | `f32` | `0.08` (existing in `emergence.rs:190`) | Per-edge blend weight on inbound culture in `drift_populations` (`culture.rs:127-129`) | `0.0` (no contact-induced change; isolation hardens) | `> 0.5` (contact dominates; no local identity) |
| `creole_threshold` | `f32` | `0.85` (existing in `emergence.rs:175,190`) | `language_distance` at which the second-pass creolization in `culture.rs:157-164` fires | `> 1.5` (never creolizes; languages fragment) | `0.0` (every contact creolizes; all languages merge) |
| `dialect_split_threshold` | `f32` | `0.35` | `language_distance` at which the macro `classify_dialect` partition splits one dialect into two | `> 0.8` (one dialect per world) | `< 0.05` (every micro-drift is a new dialect; 1000s of micro-dialects) |
| `script_invalidation_threshold` | `f32` | `0.15` | `language_distance` from a cluster's cached `ScriptSignature` at which the cache is invalidated and the projection recomputed | `> 0.5` (signature pins even as language drifts; same script for diverged languages) | `< 0.02` (signature thrashes every tick; renderer cache invalidates continuously) |
| `belief_feedback_rate` | `f32` | `0.04` (NEW — see §3) | Pull strength of `CultureProfile::traits` toward the weighted belief centroid of cluster members | `0.0` (cluster culture ignores lived belief; agents drift to a value-system that no one actually holds) | `> 0.2` (belief dominates drift; mutation and contact are invisible; cultures become the agent-level average) |
| `belief_sociability_lr` | `f32` | `0.08` (existing in `psyche.rs:204`) | Per-tick learning rate in `update_beliefs`; multiplied by `temperament.sociability` | `0.0` (beliefs are static after birth) | `> 0.3` (beliefs snap to the cluster centroid in a few ticks; no heterodoxy) |
| `ritual_repetition_threshold` | `u32` | `8` | Minimum number of `Interaction::Coexisted` + `Interaction::Cooperated` events in a `(location_bin, window_ticks)` for the `classify_ritual` function to label a pattern as a "ritual" | `> 64` (almost no ritual ever labels; cultural register is silent) | `1` (every repeated cooperation labels as a ritual; legend-spam) |
| `ritual_window_ticks` | `u32` | `512` | Sliding window over which `classify_ritual` counts event frequencies | `> 4096` (rituals only fire on generational scales; invisible at scene tempo) | `< 32` (ritual labels flicker on / off every few seconds) |
| `tradition_min_evidence` | `u32` | `32` | Minimum number of `TraditionLabel`-shaped events (rituals + belief-cluster centroids + script signatures) before `classify_religion` emits a stable label | `> 512` (traditions never label; cultural register is silent) | `1` (every minor pattern labels; tradition-spam) |
| `creole_resume_decay` | `f32` | `0.0` (NEW) | EWMA weight when `creole_threshold` *fails* to fire — i.e. how fast the two languages *un-creolize* if contact is severed after a creole formed | `1.0` (creole is permanent; languages never re-diverge after contact resumes-then-severs) | `0.0` is the recommended default (creole is a *moment*, not a permanent fusion; the two languages drift back to their pre-creole states) |
| `contact_edge_min_weight` | `f32` | `0.05` | Minimum `ContactEdge::weight` for an edge to be retained in the `phase_emergence` contact-builder; edges below this are dropped (no contact = no drift) | `> 0.5` (most contact is ignored; isolation hardens) | `0.0` (every weak contact registers; diffusion noise dominates) |

All knobs are grouped in one `CultureEmergenceParams` struct (not scattered across modules) and loaded from the scenario RON config. The emergence dashboard (§5) plots a real-time criticality indicator so the designer can see whether the system is heading toward heat-death (one dialect, one tradition, no ritual labels) or explosion (1000s of micro-dialects, tradition-spam, script-cache thrashing) before adjusting.

---

## 5. Observable emergence metrics for the dashboard

These metrics feed the **Emergence Dashboard** (`crates/civ-emergence-metrics/src/dashboard.rs` extension) and the `emergence.metrics.v1` replay-bus event (per `emergence-dashboard.md` §5). The existing dashboard already reports `cluster_entropy`, `ideology_homophily`, `sentience_fraction`, `psyche_stability`, `diplomacy_tension`. This spec extends the dashboard with **culture-language-specific** metrics that aggregate from the *existing* `CultureProfile`, `Psyche::beliefs`, `Tie`, and `SocialEvent` substrate — no new per-agent state. The `EmergenceDashboard` struct gains a new field, `culture: CultureDashboard`, computed in a new pure function `compute_culture_metrics`:

| Metric | How to compute | Target signature (healthy culture) | Failure mode |
|---|---|---|---|
| **Dialect count** | Count distinct `DialectLabel`s produced by `classify_dialect` across all `cluster_cultures` at the sample tick | Power-law: a few major dialects + many micro-dialects (α ∈ [1.5, 2.0] per `emergence-dashboard.md` §3.1) | 1 = one dialect per world (cultural heat-death); > `cluster_count` (= every cluster its own dialect, no shared identity) |
| **Dialect turnover rate** | Count of clusters whose `DialectLabel` changed since the previous sample, divided by `cluster_count` | Low (< 0.05 per sample); dialects are sticky | High (> 0.20 per sample) = dialect-shuffle churn (script-cache thrash) |
| **Language distance entropy** | Shannon entropy of the *pairwise language-distance histogram* across all cluster pairs in `cluster_cultures` | Moderate (0.4..0.7 of normalised); enough variety to support multiple dialects without chaos | Low (all within `dialect_split_threshold` = one dialect) = no linguistic differentiation; high (spread flat) = maximum noise |
| **Creole event rate** | Count of `creole_blend` events emitted by the `drift_populations` second pass over the rolling window, normalised by `cluster_count` | Low and stable; creole is a *moment*, not a steady state | Sustained high = languages merge continuously; zero = linguistic isolation hardening |
| **Belief variance within cluster** | Per-cluster variance of `Psyche::beliefs` across cluster members (weighted by `Tie::familiarity`); report the population median | Moderate (0.05..0.20) — enough heterodoxy to support schism; not so much that no shared value-system exists | Near zero (one belief vector per cluster) = cult-like unanimity (charter violation in the *other* direction: cultures should not be a hive-mind); near 1.0 = no shared culture at all |
| **Belief variance between clusters** | Per-axis pairwise distance between cluster belief centroids, normalised | High (≥ 0.3) — clusters have measurably different value-systems | Near zero = all clusters share the same belief centroid; culture is uniform across the world |
| **Ideology × belief MI** | `ideology_homophily` already reports *intra-cluster* ideological homophily; the new metric is the histogram-based `MI(ideology_bin, belief_bin)` across the population, normalised by `H(ideology_bin)` (per `emergence-dashboard.md` §3.5) | Moderate (0.2..0.6 normalised) — agents with similar beliefs cluster ideologically, but ideology is not *determined* by belief | Near 0 = no coupling (ideology and belief are independent layers — fine, but a charter warning that the cluster-aggregation feedback may be broken); near 1.0 = ideology *is* belief (single-variable over-coupling) |
| **Ritual label rate** | Count of `RitualLabel`s emitted by `classify_ritual` over the rolling `ritual_window_ticks`, normalised by `(cluster_count × ritual_window_ticks)` | Moderate (≥ 0.01 / cluster / window) — rituals are visible in the cultural register | Zero = no ritual labels ever fire; > 0.5 / cluster / window = legend-spam |
| **Ritual diversity (entropy)** | Shannon entropy of the `RitualLabel` histogram over the rolling window, normalised | Moderate (0.4..0.8) — multiple ritual shapes per cluster, not all rituals are the same kind | Near zero = one ritual dominates (monoculture); near 1.0 = chaos |
| **Script signature diversity** | Count of distinct `ScriptSignature` byte-vectors across all clusters at the sample tick | Power-law: a few major scripts + many micro-scripts | 1 = one script per world (linguistic monoculture); > `cluster_count` (every cluster its own script; render-cache thrashes) |
| **Script cache invalidation rate** | Count of clusters whose `ScriptSignature` was recomputed since the previous sample, divided by `cluster_count` | Low (< 0.05 per sample) — signatures are sticky | High (> 0.20 per sample) = language drift outpaces `script_invalidation_threshold`; renderer cache thrash |
| **Tradition label rate** | Count of `TraditionLabel`s emitted by `classify_religion` over the rolling `tradition_lookback`, normalised by `cluster_count` | Low and stable; traditions are *rare* macro labels (one per major belief + ritual pattern) | Zero = no tradition labels; > 0.10 / cluster / lookback = tradition-spam |
| **`ideology_homophily` ↔ cluster `belief_feedback_rate` correlation** | Pearson r of `ideology_homophily` and the rolling-mean of `belief_feedback_rate × cluster_belief_variance` over 4096 ticks | Moderate positive (0.2..0.6) — the feedback loop is producing *some* homogenization, not none and not all | Near 0 = feedback loop broken (cultures drift independently of lived belief); near 1.0 = feedback dominates (cultures are a hive-mind echo of agent belief) |

The `emergence.metrics.v1` replay-bus event (per `emergence-dashboard.md` §5) is extended with the `culture` layer so the Godot / Unreal / web clients show the same dashboard series offline. The `emergence.alarm.v1` event is extended with three new alarm IDs derived from this spec:

- **MT-CUL-001** — dialect count stuck at 1 for 4096 ticks (cultural heat-death) OR script signatures stuck at 1 (linguistic heat-death)
- **MT-CUL-002** — script cache invalidation rate > 0.20 per sample for 256 ticks (renderer-cache thrash from over-fast language drift)
- **MT-CUL-003** — `ideology_homophily` × `belief_feedback_rate` correlation outside [0.05, 0.85] for 4096 ticks (the belief feedback loop in §3 is broken or has taken over the dynamics)

---

## 6. Phased implementation plan

This is a DAG-structured WBS. No code is written here; file paths, struct names, and function signatures are identified for the implementing agent. Every phase extends *existing* substrate — no new crate, no API edge to lifecycle / diplomacy / markets.

### Phase 0 — Prerequisite audit (no new structs)

| Task | File | Depends on | Agent effort |
|---|---|---|---|
| P0-A: Verify `drift_populations` is called once per `phase_emergence` tick from `emergence_culture` (`emergence.rs:170-193`) and that `culture.rs:100-166` is the sole mutation path for `CultureProfile` | `crates/engine/src/emergence.rs`, `crates/agents/src/culture.rs` | — | 2 tool calls |
| P0-B: Confirm `belief_culture_exposure` (`psyche.rs:220-246`) and `update_beliefs` (`psyche.rs:202-218`) are the sole mutation paths for `Psyche::beliefs`; verify the existing `phase_emergence` `emergence_psyche` call site (`emergence.rs:382-414`) | `crates/agents/src/psyche.rs`, `crates/engine/src/emergence.rs` | — | 2 tool calls |
| P0-C: Map which contact-edge sources feed `phase_emergence`: the current fixed `0.15` weight in `emergence.rs:186` is the placeholder; the audit identifies the existing co-location / proximity / trade-volume signals available as replacements | `crates/engine/src/emergence.rs` | — | 3 tool calls |
| P0-D: Audit `crates/civ-emergence-metrics/src/dashboard.rs` to confirm the existing `EmergenceDashboard` struct's serialization format; the new `culture: CultureDashboard` field is an additive change, not a breaking one | `crates/civ-emergence-metrics/src/dashboard.rs` | — | 2 tool calls |
| P0-E: Confirm `crates/civ-emergence-metrics` has no current `CultureDashboard` struct, no current `compute_culture_metrics` function, and the dashboard's `compute` signature is `pub fn compute(cluster_sizes, ideologies, sentient, total, moods, diplomacy)` (`dashboard.rs:63-79`) — the new function is a new pure fn, not a parameter to `compute` | `crates/civ-emergence-metrics/src/dashboard.rs` | — | 2 tool calls |

### Phase 1 — Add the macro read-out classifier functions to `crates/agents/src/culture.rs`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P1-A: Add `pub enum DialectLabel(pub String)` and `pub fn classify_dialect(languages: &BTreeMap<ClusterId, TraitVector>, distance_threshold: f32) -> BTreeMap<ClusterId, DialectLabel>` | `crates/agents/src/culture.rs` | P0-A | Single-link cluster on `language_distance >= distance_threshold`; pure fn, no ECS. Returns a string ID per cluster, never an enum variant. |
| P1-B: Add `pub struct ScriptSignature(pub Vec<u8>)` and `pub struct PhonotacticProfile { /* small data table: feature-bias → stroke-count, curvature, stem-angles */ }` and `pub fn project_script(language: TraitVector, profile: &PhonotacticProfile) -> ScriptSignature` | same | P1-A | Pure fn, deterministic. The `PhonotacticProfile` is a small data struct (Rosenfelder/Zompist LCK + gleb inventory heuristics as data tables; per `emergent-systems-spec.md` §2.4). |
| P1-C: Add `pub struct ValueAxesProfile { /* data table: (belief_axis, named_axis, weight) */ }` and `pub fn classify_value_system(centroid: [f32; PSYCHE_DIM], axes: &ValueAxesProfile) -> ValueSystemBundle` | same | P1-A | `ValueSystemBundle` is a small struct of `(name, axis_readings)`, not an enum. |
| P1-D: Add `pub fn classify_ritual(events: &[SocialEvent], location_bin: (i32, i32), window_ticks: u32, threshold: u32) -> Option<RitualPattern>` | same | P1-A | Pure fn over a slice. `RitualPattern` is a small struct of `(name, frequency, location_bin)`, not an enum. |
| P1-E: Add `pub fn classify_religion(centroid: [f32; PSYCHE_DIM], script: &ScriptSignature, ritual_freqs: &BTreeMap<String, f32>, min_evidence: u32) -> Option<TraditionLabel>` | same | P1-A, P1-C, P1-D | Pure composition of the three classifiers; `TraditionLabel` is a `pub struct` with descriptive fields, not an enum. |
| P1-F: Tests: P1-A splits 4 clusters at `distance_threshold = 0.35` into the correct partition; P1-B same language → same signature; P1-C centroid `0` returns a valid bundle; P1-D empty events return `None`; P1-D `threshold = 1` labels any single event; P1-E min_evidence filters sparse bundles | `crates/agents/src/culture.rs` tests | P1-A..E | Property-based via `proptest` consistent with the rest of the crate |

### Phase 2 — Replace the fixed `weight: 0.15` contact-edge builder with a kinship- and proximity-weighted pass

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P2-A: Add `pub fn build_culture_contact_edges(world: &World, cluster_cultures: &BTreeMap<u64, CultureProfile>, params: &CultureEmergenceParams) -> Vec<ContactEdge>` to `crates/agents/src/culture.rs` | `crates/agents/src/culture.rs` | P1-A, P0-C | Pure fn over the world; returns edges weighted by `mean_intra_pair_kinship × co_location_signal` (the existing `position` co-location as a proxy until the proximity heatmap lands; see P2-C). Drops edges below `params.contact_edge_min_weight`. |
| P2-B: Replace the inline `for i in 0..keys.len() { for j in (i+1)..keys.len() { edges.push(ContactEdge { weight: 0.15, .. }) } }` in `emergence.rs:180-189` with a call to `build_culture_contact_edges` | `crates/engine/src/emergence.rs` | P2-A | The function reads from the `&World` already in scope; the only new dependency is `civ_agents::culture::build_culture_contact_edges`. |
| P2-C: The `co_location_signal` proxy is the *current* mean inter-cluster `Position3d` distance; a future PR may swap it for the actual `phase_proximity` heatmap (out of scope for v1 of this spec) | `crates/engine/src/emergence.rs` | P2-A | Documented as a known proxy; tracked as a follow-up. |
| P2-D: Tests: P2-A returns 0 edges when only 1 cluster exists; P2-A drops edges below `contact_edge_min_weight`; P2-B the integration test in `emergence.rs:699-710` (`culture_phase_drifts_cluster_profiles`) still passes; the new contact weights differ measurably from the old fixed `0.15` (dialects can split when co-location is high) | `crates/agents/src/culture.rs` tests + engine integration test | P2-A, P2-B | |

### Phase 3 — Close the cluster culture ↔ belief feedback loop (§3)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P3-A: Add `pub fn weighted_belief_centroid(world: &World, cluster_id: ClusterId, members: &[hecs::Entity]) -> [f32; PSYCHE_DIM]` to `crates/agents/src/psyche.rs` | `crates/agents/src/psyche.rs` | P0-B | Pure fn: walks members, reads `Psyche::beliefs` + `SocialGraph` ties; weights by `Tie::familiarity` for the member's strongest *intra-cluster* ties. Returns the agent-mean weighted mean. |
| P3-B: Add `pub fn tick_culture_belief_feedback(world: &World, cluster_cultures: &mut BTreeMap<u64, CultureProfile>, params: &CultureEmergenceParams)` to `crates/agents/src/culture.rs` | `crates/agents/src/culture.rs` | P3-A, P1-A | Pure fn: for each cluster, computes the weighted belief centroid, pulls `CultureProfile::traits` toward it by `params.belief_feedback_rate * (1.0 - profile.kinship)`. Bounded by `mix_trait_vectors` clamp. |
| P3-C: Wire P3-B into `emergence_culture` after the `drift_populations` call (`emergence.rs:190-193`); the pull runs *after* drift so the drift noise is preserved and the pull is the slow, *meaningful* part | `crates/engine/src/emergence.rs` | P3-B | One-line integration: `tick_culture_belief_feedback(&self.world, &mut self.emergence.cluster_cultures, &params)`. |
| P3-D: Tests: P3-A weighted centroid favours high-familiarity nodes; P3-B with `belief_feedback_rate = 0.0` is a no-op (preserves the existing `culture.rs:222-249` `kinship_resists_external_contact` test); P3-B with `belief_feedback_rate = 1.0` and zero kinship pulls traits to the centroid in one tick; P3-C the integration test in `emergence.rs:699-710` still passes AND the new feedback changes the long-run cluster trait values measurably | `crates/agents/src/psyche.rs` + `crates/agents/src/culture.rs` + engine integration | P3-A..C | Property-based |

### Phase 4 — Criticality knobs + dashboard metrics

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P4-A: Add `pub struct CultureEmergenceParams { /* all §4 knobs */ }` to `crates/agents/src/culture.rs` | `crates/agents/src/culture.rs` | P1-A, P0-E | `Default::default()` returns the §4 defaults. `#[serde(default)]` for scenario RON loading. |
| P4-B: Add `pub struct CultureDashboard { /* all §5 fields, f32 and u32 */ }` and `pub fn compute_culture_metrics(world: &World, cluster_cultures: &BTreeMap<u64, CultureProfile>, sample_window: &SampleWindow, params: &CultureEmergenceParams) -> CultureDashboard` to `crates/civ-emergence-metrics/src/dashboard.rs` | `crates/civ-emergence-metrics/src/dashboard.rs` | P1-A..E, P0-D, P0-E | Pure fn; calls the classifiers from P1. `SampleWindow` is a small struct holding the rolling `SocialEvent` window + the previous `ScriptSignature` cache for invalidation rate. |
| P4-C: Add `pub culture: CultureDashboard` field to `EmergenceDashboard` (`dashboard.rs:39-54`); add a new `compute` overload that takes the culture inputs and routes to P4-B; preserve the existing 5-field `compute` for back-compat | `crates/civ-emergence-metrics/src/dashboard.rs` | P4-B | Additive: existing 5-tile dashboard consumers see no change; the new `culture` field is the 6th tile. |
| P4-D: Wire `compute_culture_metrics` into the existing `sample_emergence` pass in `crates/engine/src/emergence_metrics.rs`; the call site already walks the world for cluster cultures, so no new ECS iteration | `crates/engine/src/emergence_metrics.rs` | P4-B, P4-C | The new field is added to `EmergenceSample::dashboard` (an additive change to the JSON-RPC `sim.snapshot` result; see `emergence.rs:230-231` for the existing 5-tile serialization) |
| P4-E: Extend `emergence.metrics.v1` replay-bus event with the 13 `culture_*` fields from §5; extend `emergence.alarm.v1` with `MT-CUL-001` / `MT-CUL-002` / `MT-CUL-003` thresholds | `crates/engine/src/emergence.rs` | P4-C, P4-D | Same wire format, new fields. Charter: no API break. |
| P4-F: Tests: P4-A default params match §4 table; P4-B all 13 metrics are bounded in `[0, 1]` for populations of 10, 100, 1000 agents; P4-C the existing `emerg_emerg_001_dashboard_compute_combines_all_five` test (`dashboard.rs:399-423`) still passes; P4-D `sample_emergence` populates the new `culture` field without breaking the existing 5 tiles; P4-E replay event round-trips through `bincode` for the new fields | `crates/civ-emergence-metrics/src/dashboard.rs` tests + engine integration | P4-A..E | Property-based |

### Phase 5 — Bidirectional coupling sanity (cross-layer, no new edges)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P5-A: Audit `crates/diplomacy/` for any read of culture that should now read the *measured* `language_distance` from P1-A — replace any `culture.traits[…]` read that was being used as a linguistic-distance proxy with `language_distance(profile_a.language, profile_b.language)` | `crates/diplomacy/` or `crates/agents/src/diplomacy.rs` | P1-A | No new crate dep; just a one-line read-site migration. |
| P5-B: Audit `crates/economy/` for any read of culture that should now flow through the existing `belief_culture_exposure` → `update_beliefs` → actor need-priority channel described in §2.1; the actor's needs are already weighted by `Psyche::drives` (genetic), so adding belief-driven need-biasing is a one-line weight add in the existing `record_bid_for_need` helper (`civ-economy-emergent-markets.md` §6 P3-A) | `crates/economy/src/emergent.rs` | P3-A | Charter-safe: the actor's own `Psyche::beliefs` is the new input; the existing substrate already has the actor reading its own psyche. |
| P5-C: Audit `crates/build/` (architecture) for any read of culture that should now drive the WFC tile-set selection (per `emergent-systems-spec.md` §2.3 the build layer is data-driven from culture vec, but the per-tile weight table may have a hand-coded "X cultural style" entry); replace any hardcoded style table with a read of `weighted_belief_centroid` + a `tile_style_axes` data table | `crates/build/` | P3-A | If no such read exists, document the absence and close the task. |
| P5-D: Audit `crates/legends/` for any read of culture that should now consume the `TraditionLabel` + `RitualPattern` events from P1-D + P1-E; the cultural register is the existing `legends` ingest path (`emergent-systems-spec.md` §2.6) — the new events slot in as `SourceCrate::Agents` `RawSimEvent`s with `EventKind::RitualObserved` + `EventKind::TraditionFormed` (new variants, additive) | `crates/legends/` | P1-D, P1-E | If no such read exists, document and close. |
| P5-E: Add a CI-level invariant test: assert that NO production code path mutates `CultureProfile` outside of `drift_populations` + the new `tick_culture_belief_feedback` (P3-B). Static check via a `#[test]` that walks the crate's public surface and asserts only those two functions call `mix_trait_vectors` on a `CultureProfile` field | `crates/agents/src/culture.rs` tests | Phase 3 complete | Closes the loop on the charter: there is no other way for culture to change. |

### Phase 6 — Acceptance criteria reachability (validation)

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P6-A: Scenario test: "dialects emerge from drift, not authoring" — start with 2 clusters sharing a `CultureProfile`; let the system run with NO contact between them; assert `classify_dialect` initially returns 1 dialect, then after 2000 ticks returns 2 (one per cluster, both well above `dialect_split_threshold`) | `crates/agents/src/culture.rs` integration test | P1-A, P2 complete | Charter AC: dialects emerge from drift, not from a `DialectId` enum |
| P6-B: Scenario test: "creole is a moment, not a permanent fusion" — start with 2 clusters with diverged language vectors (distance > `creole_threshold`); bring them into high contact for 1000 ticks (creole fires); then sever contact; assert over the next 2000 ticks the two languages re-diverge (the creole does not pin them) | same | P6-A, P1-A, P2 complete | Charter AC: the creole path is the existing `culture.rs:157-164`; the test verifies the moment-vs-fusion semantics through `creole_resume_decay` (default 0.0) |
| P6-C: Scenario test: "script signatures differ by cluster" — start with 2 clusters with diverged language vectors; after drift, assert `project_script` returns 2 distinct `ScriptSignature` byte-vectors; the renderer materialization would produce 2 visually different scripts | same | P1-B, P1-A | Charter AC: scripts are derived, not authored |
| P6-D: Scenario test: "belief feedback loop closes" — start with a cluster whose `Psyche::beliefs` distribution is bimodal (schism in progress); with `belief_feedback_rate = 0.0` (no feedback), the cluster's `CultureProfile::traits` drifts by mutation only; with `belief_feedback_rate = 0.1` (default-ish), the cluster's `traits` is measurably pulled toward the high-familiarity nodes' belief centroid; the difference is statistically significant at 4096 ticks | same | P3-B, P3-C | Charter AC: cluster culture responds to lived belief |
| P6-E: Scenario test: "ritual labels emerge from frequency patterns" — construct a 500-tick scenario where 12 `Interaction::Coexisted` events cluster at a single `(x, z)` location bin within `ritual_window_ticks`; assert `classify_ritual` returns a `RitualPattern` with the right shape | same | P1-D | Charter AC: rituals are measured frequency patterns |
| P6-F: Scenario test: "tradition labels form from belief + ritual + script composition" — construct a scenario where a cluster has a stable belief centroid, a stable script signature, and ≥ 4 ritual events of the same shape; assert `classify_religion` returns a stable `TraditionLabel` after `tradition_min_evidence` events | same | P1-E, P6-E | Charter AC: traditions are *composed*, not authored |
| P6-G: Scenario test: "criticality edge of chaos" — run a 4096-tick scenario with default `CultureEmergenceParams`; assert `dialect_count` is in `[1, cluster_count]`, `ritual_label_rate` is ≥ 0.01 / cluster / window, `ideology_homophily × belief_feedback_rate` correlation is in `[0.2, 0.6]` (the §5 target band) | same | P4 complete | Performance-gated; runs in CI as a regression |
| P6-H: Scenario test: "criticality knob reachability" — for each knob in §4, set it to its heat-death and explosion extremes; assert the system reaches the corresponding failure mode within 1024 ticks (dialect_count = 1 for heat-death, script_cache_invalidation_rate > 0.20 for explosion); assert setting it back to default returns the system to the target band within 2048 ticks | same | P6-G | Ensures the knobs are *real* knobs, not constants |

### Phase 7 — Coupling to the existing `phase_emergence` orchestrator

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| P7-A: The `phase_emergence` orchestrator (`emergence.rs:115-127`) is already `genetics → culture → social → psyche → legends → civ-ai`. The new culture-side work (P2-B contact builder, P3-C belief feedback) slots into the existing `emergence_culture` call without changing the orchestrator order. The dashboard side (P4-D `compute_culture_metrics`) is wired into `sample_emergence` in `emergence_metrics.rs` (already called from the same `phase_emergence` body). Verify the call order is correct: culture must finish before psyche (psyche reads cluster cultures) and the sample must run after psyche (so the belief feedback has propagated) | `crates/engine/src/emergence.rs`, `crates/engine/src/emergence_metrics.rs` | P1, P2, P3, P4 complete | No new orchestrator code; just verification that the existing order supports the new work |
| P7-B: The `CulturalDiffusion` event emission from the freehand tool path (§2.2 final row) is a follow-up owned by the freehand tool PR, not by this spec; this spec consumes the event but does not produce it | `crates/build/` (or wherever freehand tools live) | P2-A | Tracked as a follow-up |
| P7-C: The polity cohesion graph's dialect-distance coefficient (§2.3) is a one-line weight add in `polities-markets.md` §1.2; owned by the polity PR, not by this spec | `crates/diplomacy/` or `crates/agents/src/diplomacy.rs` | P1-A | Tracked as a follow-up |

### DAG summary (critical path)

```
P0-* (all parallel) → P1-A..F → P2-A..D (depends on P1-A)
                                  ↓
                                P3-A..D (depends on P1, P2) → P4-A..F (depends on P1, P3)
                                                                   ↓
                                                                 P5-A..E (depends on P1, P3, P4; can run in parallel with P6)
                                                                   ↓
                                                                 P6-A..H (depends on P4 complete)
                                                                   ↓
                                                                 P7-A..C (depends on Phase 6 complete)
```

**Critical path to acceptance:** P0 → P1 → P2 → P3 → P4 → P6 → P7. P5 audit can run in parallel with P6.

---

## 7. Test strategy summary

- **Unit tests** (property-based via `proptest`): each new pure function in `culture.rs` (P1-A..E) and `psyche.rs` (P3-A) has invariant tests — same input → same output, distance bounds respected, classifications are non-empty for non-degenerate input, cache invalidation rate is bounded.
- **Integration tests** (hecs World with seeded RNG): the P6-A..H scenarios run for 2000–4096 ticks and assert statistical properties of the macro culture (dialect count bands, ritual label rates, tradition label rates, MI correlations). These are charter-aligned: statistical properties, not bit-identical outcomes.
- **Emergence regression**: `cargo test -p civ-agents -- culture_emergence_regression` runs the P6-G 4096-tick scenario and asserts `dialect_count ∈ [1, cluster_count]`, `ritual_label_rate ≥ 0.01`, `ideology_homophily × belief_feedback_rate` correlation ∈ [0.2, 0.6]. This runs in CI as a performance-gated test, mirroring the lifecycle regression test in `civ-needs` and the markets regression test in `civ-economy`.
- **No determinism requirement** (per charter): tests assert statistical properties of the macro culture (entropy bands, MI bands, label rates), not bit-identical outcomes of the per-tick culture vector. The *function-purity* of the §1.2 classifiers + §3 feedback is asserted separately (same inputs → same outputs) so the dashboard / snapshot tests can rely on it.
- **Existing test preservation**: the existing tests in `crates/agents/src/culture.rs:168-249` (`mutation_moves_traits_but_keeps_bounds`, `isolated_populations_diverge_over_time`, `contact_diffuses_culture_and_creolizes_language`, `kinship_resists_external_contact`) and the integration tests in `crates/engine/src/emergence.rs:647-733` (`legends_phase_ingests_death_events`, `psyche_phase_mutates_mood_over_ticks`, `culture_phase_drifts_cluster_profiles`, `civ_ai_phase_leaves_observable_emergence_state`) must all continue to pass — the substrate is unchanged, only the macro read-out is added.

---

## 8. What this spec does NOT include

- Any `enum Dialect`, `enum Script`, `enum Religion`, `enum ValueSystem`, `enum Ritual` stored on a cluster, agent, or polity. All macro labels are pure read-out functions (§1.2).
- Any `set_culture(profile)` / `set_dialect(id)` / `set_script(id)` callable that mutates macro state independent of the underlying drift / drift / contact / graph substrate.
- Any direct `civ-agents` → `civ-economy` or `civ-agents` → `civ-diplomacy` API edge. All coupling is the shared substrate gradient (`CultureProfile`, `Psyche::beliefs`, `Tie::familiarity`, `SocialEvent`), per the charter.
- Any re-implementation of `drift_populations`, `belief_culture_exposure`, `update_beliefs`, or `decay_social_graph`. The substrate is correct; this spec adds read-out + a single bounded feedback loop.
- Any hardcoded script / glyph atlas / TTF. Scripts are derived projections through `PhonotacticProfile` data tables and materialized lazily by the renderer (per `emergent-systems-spec.md` §2.4 / E5.3).
- Any LLM call in the culture-language path. The cultural register of the legends engine may use the existing LLM garnish for narrative embellishment (per `emergent-systems-spec.md` §2.6 / §2.10), but the *macro read-out* itself is a pure function over the substrate.
- Any micro-drivers beyond what `culture.rs` + `psyche.rs` + `social.rs` already provide. The micro-drivers are: `drift_populations` (mutation + diffusion + creolization), `update_beliefs` (graph-weighted culture sampling), `apply_social_event` (interaction → tie mutation), `decay_social_graph` (contact-decay). This spec consumes their outputs and projects them into macro labels; it does not add new micro-drivers.
- Any change to the existing `CultureProfile` *substrate* (the `traits`, `language`, `contact`, `kinship` fields, the `drift_populations` algorithm, the `creole_threshold` semantics). The substrate is charter-correct as of this spec's date; the new work is purely read-out + a single bounded feedback pull.
- Any music / audio component. Music is the domain of [`audio-direction.md`](../design/audio-direction.md) + `emergent-systems-spec.md` §2.5 (E6); the `MusicalTradition` is a separate substrate read-out from `(culture_vec, available_materials)` and does not require culture-language co-emergence.
- Any change to the freehand tool path's `CulturalDiffusion` event emission. This spec consumes the event in §2.2 (a future freehand tool PR produces it).
- Any change to the polity cohesion graph's dialect-distance coefficient. This spec defines the input (`language_distance` between cluster pairs); `polities-markets.md` owns the polity cohesion math.
- Any direct write to the existing `EmergenceDashboard` 5-tile struct. The new `culture: CultureDashboard` field is additive; the existing 5 tiles (`cluster_entropy`, `ideology_homophily`, `sentience_fraction`, `psyche_stability`, `diplomacy_tension`) keep their existing contracts and existing tests.

---

*Document authority: this spec defines the macro read-out + bidirectional coupling + criticality knobs + dashboard metrics for emergent culture / language / script / ritual / tradition, on top of the existing `crates/agents/src/culture.rs` + `crates/agents/src/psyche.rs` + `crates/agents/src/social.rs` substrate. The micro-drivers (`drift_populations`, `update_beliefs`, `apply_social_event`, `decay_social_graph`) are unchanged. The feedback loop in §3 is the only mutation introduced, and it is bounded by `(1 - kinship) * belief_feedback_rate`. There is no other way for a dialect, a value-system, a script, a ritual, or a tradition to exist — they are pure read-out functions over the substrate, per the charter.*
