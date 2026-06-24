# Emergent Language — Kirby Iterated-Learning Pathway

> **Status:** Design (planner-only, 2026-06-23). No implementation code in this document.
> **Spec ID:** `lang-emergence-design` (research branch).
> **Pattern ancestor:** [`civ-culture-emergent.md`](civ-culture-emergent.md) (culture + script macro read-out), [`psyche-social.md`](psyche-social.md) (graph-weighted belief exposure), [`emergent-systems-spec.md`](emergent-systems-spec.md) §2.4 E5 (language & writing systems), [`PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md) (shared-field coupling doctrine).
> **Governing canon:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md), [`docs/design/emergent-systems-spec.md`](emergent-systems-spec.md) §2.4 E5.0-E5.3, [`docs/design/civ-culture-emergent.md`](civ-culture-emergent.md), [`docs/design/psyche-social.md`](psyche-social.md), [`docs/design/PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md), [`docs/design/emergence-dashboard.md`](emergence-dashboard.md), [`docs/architecture/phenotype-protocols.md`](../architecture/phenotype-protocols.md) (semantic-key i18n).
> **Code substrate (read-only inputs):** `crates/engine/src/language.rs` (`Phoneme { features: [f32; 6] }`, `Morpheme { phonemes, meaning }`, `LanguageState { vocabulary, drift_rate, split_threshold, tick }`, `tick_language(lang, contact_pressure)`, `should_split`, `borrow_word`, the deterministic `blake3`-seeded `ChaCha8Rng`), `crates/agents/src/culture.rs` (`CultureProfile::language: TraitVector`, `language_distance`, `drift_populations` with creolization second pass), `crates/agents/src/psyche.rs` (`Psyche::beliefs`, `belief_culture_exposure`, `update_beliefs`), `crates/agents/src/social.rs` (`Tie`, `SocialGraph`, `apply_social_event`, `decay_social_graph`, `relation_label`), `crates/agents/src/cluster.rs` (`ClusterId`, `ClusterMember`, `should_join`, `should_leave`), `crates/planet/src/geology.rs` (iron/silicate/water/ice plate grid — phonemic substrate channel), `crates/climate/src/lib.rs` (T/M/E balance), `crates/voxel/src/{material,fluid_ca,reactions}.rs` (F/B flux), `crates/civ-emergence-metrics/src/dashboard.rs` (per-tick `EmergenceDashboard`), `crates/legends/src/worker.rs` (event-bus ingest path), `crates/legends/src/rumor.rs` (rumor-hop mutation), `crates/audio/src/lib.rs` (material-gated `MusicalTradition` per `audio-direction.md`).
> **Traceability:** This spec defines FR-CIV-LANG-011 … FR-CIV-LANG-040 (carry-overs from `emergent-systems-spec.md` §2.4 E5.0-E5.3 — *superset*; this spec adds Kirby iterated-learning, grammar induction, writing-system emergence, physics + social coupling, and the English-default/native toggle, on top of the existing `engine::language` ASCA substrate and `civ-culture-emergent.md` macro read-out).

---

## 0. Charter constraint

The Civis Emergence Charter forbids hardcoding language, dialects, scripts, and grammars as authored enums or state machines ([`emergence-charter.md`](../guides/emergence-charter.md) §"What EMERGES"). The current substrate already obeys the lowest layer:

- **`crates/engine/src/language.rs`** carries the ASCA-style diachronic distinctive-feature sound-change engine (`Phoneme { features: [f32; 6] }`, `Morpheme { phonemes, meaning }`, `LanguageState { vocabulary, drift_rate, split_threshold, tick }`, `tick_language(lang, contact_pressure)`). Drift is gaussian on features, bounded to `[0, 1]`, deterministic via `blake3`-seeded `ChaCha8Rng` keyed by `(tick, drift_rate, split_threshold, contact_pressure, vocabulary bytes)`. Near-identical phonemes collapse via cosine-similarity merge at `COSINE_MERGE_THRESHOLD = 0.95`. `should_split(a, b)` fires when the average per-meaning distance exceeds `split_threshold`. `borrow_word(target, source, meaning)` clones a morpheme from `source.vocabulary` into `target.vocabulary` for the given `meaning` key.
- **`crates/agents/src/culture.rs`** carries a parallel high-level language vector `CultureProfile::language: [f32; 4]` that drifts and creolizes through `drift_populations` (`culture.rs:100-166`) using the same kinship-damping pattern as cultural traits.

What is **missing** — and what this spec specifies — is the **Kirby iterated-learning pathway** (the *cultural evolution* feedback loop, Kirby 1999/2001) that closes the substrate's one-way sound-change into a *transmissible*, *compositional*, *conventional* language, plus the three macro layers above the lexicon that turn a vocabulary into a culture-bounded communication system:

1. **Phoneme → Lexicon bridge.** The ASCA engine mutates phonemes; nothing yet *learns* a lexicon. We add an **iterated-learning channel** in which each generation of agents (a) induces a lexicon from exposure to parental speech and (b) produces output that becomes the next generation's input. The bottleneck forces compositionality to emerge (Kirby 1999, *Language evolution and the Baldwin effect*; Brighton & Kirby 2006, *Cultural selection for learnability*).
2. **Lexicon → Grammar induction.** A **grammatical category distribution** (open-class / closed-class / function morpheme ratio) is a *measured field* over the lexicon, learned by frequency and compositionality heuristics. Grammar is *not* stored as a phrase-structure rule; it is the measured distribution of categories over morpheme bigram statistics (Reali & Christiansen 2009; Smith & Kirby 2008).
3. **Grammar → Writing system.** An **orthographic projection** is computed from the (a) phonotactic profile, (b) grammatical category distribution, and (c) carried writing-system substrate (logographic vs. alphabetic vs. syllabic vs. abjad). The projection lags phonology by `orthography_lag` ticks (per FR-CIV-LANG-006). Two civs with the same phonology can have different scripts (writing systems are not 1-1 with spoken language; English is Latin script, Japanese is mixed logographic + syllabic).
4. **English default + native toggle.** A *separate, small* system: the player UI is always rendered in English by default (system font atlas), but a per-civ **render toggle** projects the live emergent lexicon through the orthographic projection into a per-civ vector-stroke atlas (`ab_glyph` / `cosmic-text` per FR-CIV-LANG-007). Switching a civ's "view language" flips the renderer between the system English atlas and the civ-native atlas. The toggle is **read-only**; it never mutates the live language state.
5. **Physics + social coupling.** Per [`PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md), all emergent layers couple *only* through the shared field substrate `{T, M, E, F, P, B}`. This spec adds the coupling paths: T and F shape the *phonemic inventory* (a hot-arctic civ lacks labial fricatives; a coastal civ has liquid consonants); P (population pressure) shapes the *contact graph* (Kirby's bottleneck: small populations compress lexicon); M (moisture) shapes the *vowel space* (correlates with altitude / aridity in real-language typology — Everett 2013, *Language: The Cultural Tool*); B and E shape the *surviving functional load* (a starved civ loses function words faster than content words).
6. **Social coupling.** The iterated-learning pathway runs *through* the `SocialGraph` ties (parent → child exposure); `belief_culture_exposure` already couples the lexicon to belief drift; the new **register selector** (formal vs. cultural, per `emergent-systems-spec.md` §1.2) picks which register's lexicon to surface on a per-context basis (a treaty transcript uses the formal register; a chronicle uses the cultural register).

The charter test applied to every field below: *can this emerge from Layer-0 rules?* Yes — iterated learning is a Markov chain over (parent output, child induction); grammar is a measured distribution; scripts are a derived projection. **No** `enum LanguageFamily`, no `enum ScriptKind`, no `enum GrammarType`, no `set_dialect(id)`, no authored lexicon table on any agent or polity. The macro labels are *pure read-out functions over the substrate* (same pattern as `civ-culture-emergent.md` §1.2).

This spec **does not** re-implement the ASCA engine (substrate correct per `emergent-systems-spec.md` §2.4); does not replace `civ_agents::culture::drift_populations` (substrate correct per `civ-culture-emergent.md` §0); does not write a new per-civ text atlas manually (FR-CIV-LANG-007 is the *materialization* contract). What it *adds* is the iterated-learning bridge, the grammar induction, the writing-system projection, the physics + social coupling, and the English-default/native toggle.

---

## 1. The Kirby iterated-learning pathway

### 1.1 The three layers in a single diagram

```
┌──────────────────────────────────────────────────────────────────────┐
│  L0  ASCA SUBSTRATE (existing — crates/engine/src/language.rs)      │
│      Phoneme { features: [f32;6] }  +  Morpheme { phonemes, meaning}│
│      tick_language(lang, contact_pressure)  drift + cosine merge    │
│      should_split(a, b)  borrow_word(target, source, meaning)       │
└──────────────────────────┬───────────────────────────────────────────┘
                           │  per-civ LanguageState.vocabulary
┌──────────────────────────▼───────────────────────────────────────────┐
│  L1  KIRBY ITERATED-LEARNING  (NEW — crates/lang or extend engine)  │
│      • Parental output → Child induction  (per generation)          │
│      • Bottleneck: |child_input| << |parent_output|                 │
│      • Compositionality pressure emerges from iteration             │
│      • Lexicon compressed by |P| gradient (Kirby 1999)              │
└──────────────────────────┬───────────────────────────────────────────┘
                           │  per-civ emergent Lexicon
┌──────────────────────────▼───────────────────────────────────────────┐
│  L2  GRAMMAR INDUCTION  (NEW — pure read-out)                       │
│      • Open/closed class split (Zipf-distribution test)              │
│      • Function morpheme emergence (frequencies drop in next gen)   │
│      • Bigram transition entropy  (a measured field)                │
└──────────────────────────┬───────────────────────────────────────────┘
                           │  per-civ emergent GrammarProfile
┌──────────────────────────▼───────────────────────────────────────────┐
│  L3  WRITING-SYSTEM PROJECTION  (NEW — derived atlas)               │
│      • Phonotactic profile → stroke-stem count + curvature         │
│      • Grammar → symbol density (logographic vs. syllabic vs. alph.)│
│      • Lag phonology by `orthography_lag` ticks                     │
│      • Materialize as runtime TTF via ab_glyph / cosmic-text        │
└──────────────────────────────────────────────────────────────────────┘
```

The pathway is **read-only at the top, write-only at the bottom, with the iterated-learning bridge in the middle mutating the substrate**. L2 and L3 are pure classifier functions (the same pattern as `civ-culture-emergent.md` §1.2); only L1 mutates the substrate, and only through the existing `tick_language` + `borrow_word` + `should_split` channels.

### 1.2 The iterated-learning channel — what is a "generation"?

A *generation* in Civis is not a wall-clock span. It is the *time-to-adult* of a cohort in the existing `civ-needs` lifecycle system, gated by maturity and birth events in the cluster's per-agent stream. The existing `phase_emergence` orchestrator (`crates/engine/src/emergence.rs`) already runs a per-cluster emergence tick on `genetics → culture → social → psyche → legends → civ-ai`; the iterated-learning channel slots into this orchestrator as **one new sub-phase** (`emergence_language`) between `emergence_psyche` and `emergence_legends`, so the agent's belief state (already drifted by `update_beliefs` from `belief_culture_exposure`) is the input to the child's induction step. This ordering matters: a child's induced lexicon reads the *parental belief state*, not the *parental language state* — this is the substrate for the coupling §3.

The generation definition:

```text
// NEW — crates/agents/src/language_lifecycle.rs (proposed)
struct LanguageCohort {
    parent_generation: u32,
    child_generation: u32,
    tick_of_birth: u32,
    tick_of_adulthood: u32,        // == birth_tick + maturity_rate_inv
    cluster_id: ClusterId,
    parent_lexicon: LexiconSnapshot,
    child_lexicon: LexiconSnapshot, // learned by induction
}
```

The generation length is data-driven from the existing `civ-needs::LifecycleParams::maturity_rate` (already in the substrate; not a new parameter). A generation is typically 200–400 in-game ticks for human cohorts in the baseline scenario (matches the canonical Wallace 2002 *language evolution* simulation parameter range).

### 1.3 The induction step — compositionality from the bottleneck

A *child* in the cluster samples a subset of the parental output as input (`bottleneck_fraction` of the parental morphemes, default 0.4, per Brighton 2002 — Kirby 1999 used 0.4 as the critical value for compositionality emergence). The child *induces* a lexicon from this subset by:

1. **Meaning–form pairing** — for each `meaning: u32` observed in the input, the child retains the *first* `Phoneme` vector observed as the lexeme. Multiple observations of the same `meaning` reinforce the pairing (Bayesian update on the induced distribution).
2. **Compositionality pressure** — when two unrelated meanings share phonemes, the child is more likely to *re-use* the shared phoneme in the next generation's output (Kirby 1999 §3.2; this is the cultural-selection pressure for compositionality).
3. **Generalization** — for meanings *not* observed in the input, the child generalizes from the nearest compositional neighbor (e.g. if the input has `[meaning=1, form=AB]` and `[meaning=2, form=AC]`, then unseen `meaning=3` is rendered as `AD` with a probability proportional to the input's compositionality).

The induction function is **pure** (no ECS mutation), lives in a new `crates/lang/src/induction.rs` (proposed; new crate, see §6 Phase 1), and takes `(parental_output: &[Morpheme], bottleneck_fraction: f32, rng: &mut ChaCha8Rng) -> LexiconSnapshot`. The output `LexiconSnapshot` is the induced lexicon that the child's `LanguageState.vocabulary` will *become* on the next `phase_emergence` tick — applied via the existing `tick_language` + `borrow_word` substrate.

### 1.4 The substrate gradient — iterated learning as a Markov chain

Each generation's output is the next generation's input. This is exactly a Markov chain over `(lexicon, meaning_inventory, compositionality)`. The chain's invariant measure (the *attractor lexicon*) is shaped by:

- The **bottleneck fraction** `β ∈ (0, 1)` — the amount of parental output the child sees. Kirby 1999: `β < 0.5` produces compositional lexicons; `β > 0.7` produces holistic (non-compositional) lexicons. We default `β = 0.4` (the canonical compositionality-emergence value).
- The **mutation rate** `μ ∈ [0, 1]` — the existing `LanguageState::drift_rate` from `engine::language`. Defaults 0.05; tuned via the `emergence.set_knob` JSON-RPC method (`emergence-dashboard.md` §6).
- The **contact pressure** `c ≥ 0` — the existing `contact_pressure` argument to `tick_language`. Sourced from `|∇P|` at the cluster's centroid (per `PHYSICS_COUPLING_SUBSTRATE.md` §3.1).
- The **population pressure** `|P|` at the cluster — a *global* term that scales the *learnable* `meaning_inventory` size. A civ with `|P| = 100` can maintain ~500 morpheme–meaning pairs; a civ with `|P| = 10` collapses to ~80 (Kirby 1999's *cultural transmission* effect; the small population carries a smaller, more drift-prone lexicon).

These four parameters define the iterated-learning dynamics. The dashboard measures:

- `lexicon_size_mean` — mean number of `(meaning, morpheme)` pairs across all civs.
- `compositionality_index` — proportion of meanings whose morpheme shares at least one phoneme with another morpheme (Kirby 2001 §3.4).
- `bottleneck_loss_rate` — `1 - |child_input| / |parent_output|`, the actual compression rate per generation.
- `induced_phoneme_entropy` — Shannon entropy over the induced phoneme distribution; low entropy = a tight, conventional inventory; high entropy = loose, drift-prone.

The dashboard's `emergence.metrics.v1` event is extended with these four `lang_*` fields (charter-safe additive change). The dashboard's `emergence.alarm.v1` is extended with `MT-LANG-001` (`lexicon_size_mean = 0` for 2048 ticks — language has died) and `MT-LANG-002` (`compositionality_index = 0` for 4096 ticks — language is purely holistic, no compositionality emerged, learning is broken).

### 1.5 The grammatical induction layer — what is a "grammar"?

A grammar in Civis is **not** a phrase-structure rule set. It is a *measured distribution* over:

| Field | What it is | How it's measured |
|---|---|---|
| **Open-class share** | Proportion of morphemes used in *content* positions (nouns, verbs, adjectives — meaning-bearing) | Frequency-of-use histogram on the induced lexicon; morphemes in the top 20% frequency band are "open" |
| **Closed-class share** | Proportion of morphemes used in *function* positions (grammatical particles, agreement markers, determiners — structure-bearing) | Morphemes in the bottom 20% frequency band; ratio of `closed_count / open_count` is the *functional load* |
| **Bigram entropy** | Shannon entropy of the morpheme-pair transition distribution over a 256-tick window | `H = -Σ p(w_i, w_j) log₂ p(w_i, w_j)` |
| **Compositionality** | (above) | Proportion of morphemes sharing a phoneme with another morpheme |

The grammar is **read-out only** — the engine does not consult a grammar; it consults a `GrammarProfile` snapshot of these four values when rendering a sentence in the cultural register (via the legends engine's tracery layer, per `emergent-systems-spec.md` §2.6 + E5.3). The induction function may use the `GrammarProfile` to bias its compositionality step (e.g. if `closed_class_share` is high, the induction favors keeping function morphemes stable across generations, which is what real languages do — function words change much more slowly than content words; Pagel et al. 2007).

The `GrammarProfile` is computed in a new pure function `classify_grammar(lexicon: &LexiconSnapshot, bigram_log: &BigramLog) -> GrammarProfile`, called from the `emergence_language` phase at the end of each generation. The result is cached on the per-civ `LanguageState` as a *derived* field (same pattern as `ScriptSignature` in `civ-culture-emergent.md` §1.3). The cache is invalidated when `lexicon_distance > grammar_invalidation_threshold` (default 0.20; the same as `script_invalidation_threshold`).

### 1.6 The writing-system projection — orthography as a measured field

A writing system in Civis is *not* an authored font. It is a *projection* of the live emergent state through a `WritingSystemProjector` that takes:

- The civ's `LanguageState::vocabulary` (the lexicon).
- The `PhonotacticProfile` from `civ-culture-emergent.md` §1.2 (the phonotactic data table — feature-bias → stroke-stem count + curvature).
- The `GrammarProfile` from §1.5 (the grammatical category distribution).
- The `WritingSystemSubstrate` (a small data table mapping the civ's *carried* writing tradition: `Logographic` | `Syllabic` | `Alphabetic` | `Abjad` | `Abugida`; this is the *only* authored enum in the language pipeline, and it is a property of the *starting* civ preset, not the live state — the *use* of the substrate is emergent but the *initial assignment* is seeded; see §4.1).

The output is a `ScriptSignature` (the byte-vector defined in `civ-culture-emergent.md` §1.3) plus a `GlyphFamily` hint (a small data table mapping substrate → glyph composition rules). The renderer composes a runtime TTF from the signature on first read and caches it.

The **orthography lag** is the difference in ticks between when a phonological change occurs and when the orthography reflects it. The lag is *measured* as a function of:

- The civ's *literacy rate* (proportion of agents with `education ≥ threshold`, from the existing `civ-needs` `education` field).
- The civ's *administrative complexity* (number of `Law` records, from `civ-laws`).
- The *orthography_lag* knob (default 512 ticks, 1–2 generations for a human civ).

A civ with low literacy + low administration has an orthography that lags the phonology by many generations (spelling is frozen, pronunciation drifts — this is the source of *historical* orthography like English "knight" or "ghoti"). A civ with high literacy + high administration updates orthography within one generation (modern literate societies — e.g. Turkish script reform 1928).

The writing-system projection is computed in a new pure function `project_writing_system(lang: &LanguageState, grammar: &GrammarProfile, phon: &PhonotacticProfile, substrate: &WritingSystemSubstrate, lag_ticks: u32) -> ScriptSignature`, called from `emergence_language` whenever the orthography lag is exceeded. The result replaces the `ScriptSignature` in the `CultureProfile` derived cache (the same field already used by `civ-culture-emergent.md` §1.3 — this spec *extends* its meaning to include the substrate-conditional projection, not just the phonotactic projection).

---

## 2. Coupling to the shared physics + social substrate

The shared substrate `{T, M, E, F, P, B}` ([`PHYSICS_COUPLING_SUBSTRATE.md`](PHYSICS_COUPLING_SUBSTRATE.md) §2) is the *only* channel through which the language layer couples to physics + social. Per the doctrine, *no* API edges exist between `crates/lang` and `crates/{climate,voxel,planet,needs,agents,diplomacy,economy}`; all coupling is through the six conserved fields, read at the cluster's centroid, written through the substrate's typed setters.

### 2.1 Physics → language (downward causation, no API call)

| Substrate field | Language effect | Mechanism (shared gradient, not call) | FR |
|---|---|---|---|
| `T` (temperature) at cluster centroid | Phonemic inventory *favors* phonemes whose features are easier to produce in that climate (Everett 2013 §4: high-latitude languages have fewer tonal contrasts; arid-zone languages have more pharyngeal consonants) | The phonotactic profile in `civ-culture-emergent.md` §1.2 is *biased* by `T` at projection time: `feature_bias[phoneme] *= f(T_centroid)`. The bias is a 1D lookup (no new field, no new code path); the result is a *slightly different* `PhonotacticProfile` for a hot-arctic civ than a temperate-lowland civ | FR-CIV-LANG-011 |
| `M` (moisture) at cluster centroid | Vowel space is *compressed* in arid zones (Everett 2013; phonetic grounded cognition: dry-air acoustic impedance reduces formant bandwidth) | The phoneme drift rate `μ` is *scaled* by `M_centroid`: `μ_effective = μ × (1 + α_M × (0.5 - M_centroid))`. Low moisture ⇒ slightly higher drift (less communicative stability); high moisture ⇒ lower drift | FR-CIV-LANG-012 |
| `E` (energy / joules) at cluster centroid | Functional load is *reduced* under chronic energy deficit (a starved civ loses function words faster than content words — because function words are acquired later in development and have lower entrenchment) | The grammar induction in §1.5 reads `E_centroid` and biases the closed-class survival probability: `p(survive_function_morpheme) *= f(E_centroid)`. A chronically energy-deficit cluster loses `closed_class_share` over generations | FR-CIV-LANG-013 |
| `F` (material flux) at cluster centroid | *Material availability* shapes the substrate for the writing-system projector: a civ with high `F` for a given material (clay, papyrus, parchment, vellum, stone) gets the matching writing substrate; a civ with none of these gets a *mnemonic* substrate (oral-only, no script — most pre-literate societies) | The `WritingSystemSubstrate` (§1.6) is *derived* from `F` at the cluster centroid: `F(papyrus) > 0` ⇒ substrate = `Syllabic` (Egyptian model); `F(clay) > 0` ⇒ substrate = `Logographic` (Sumerian model); `F(stone) > 0` ∧ `F(papyrus) = 0` ∧ `F(parchment) = 0` ⇒ substrate = `Alphabetic` (Phoenician model); all zero ⇒ `Oral` (no script — pre-literate) | FR-CIV-LANG-014 |
| `P` (population pressure) gradient | Contact-pressure input to `tick_language` (existing substrate) | `c = |∇P|_cluster_centroid` (per `PHYSICS_COUPLING_SUBSTRATE.md` §3.1) | existing (FR-CIV-LANG-003) |
| `B` (biomass) at cluster centroid | Determines the *learnable* lexicon size (§1.4): chronic biomass deficit reduces the number of distinct meanings a population can stably transmit | `learnable_meaning_count = |P| × β_biomass × B_centroid / B_max` (the *learnable inventory*; per the same *cultural transmission* effect Kirby 1999 §4.1 observed in small populations) | FR-CIV-LANG-015 |

All six writes are *downward* — the substrate drives the language, not the other way around. The language layer *reads* the substrate, never *writes* it (in the *downward* direction; the upward causation is in §2.2). The phonotactic profile bias, the drift rate scaling, the grammar induction bias, the substrate derivation, the contact-pressure computation, and the learnable-inventory formula are all *pure functions* of the substrate state; they slot into `emergence_language` without a single new ECS write path.

### 2.2 Language → physics + social (upward causation with lag, no API call)

| Language signal | Substrate / social effect | Lag mechanism |
|---|---|---|
| `LexiconSnapshot` includes a stable morpheme for a high-value resource (e.g. a meaning-keyed "irrigation-channel" form) | `B_centroid` is *negatively correlated* with the presence of stable irrigation terms in the cluster's lexicon — stable lexicalization correlates with sustained infrastructure investment, which correlates with sustained biomass (a *very* soft, very lagged signal — decades in real-world terms) | 10–50 generation lag: the lexical-stability term accumulates over generations; the correlation is computed at the *emergence dashboard* layer, not in the substrate. The substrate is not modified by the lexicon directly. |
| `GrammarProfile::bigram_entropy` rises above `grammar_chaos_threshold` (default 0.85; near-uniform bigrams = the language has lost structure) | A `linguistic_collapse.v1` event is emitted into the watch bus; the existing `civ-ai` / `civ-laws` consumers may interpret the event (e.g. `civ-laws` may loosen enforcement in collapsing regions; `civ-ai` may divert admin attention). The substrate is not modified. | 1 tick lag: the event is emitted; consumers pick it up on the next phase. |
| A civ's `WritingSystemSubstrate` is `Alphabetic` and the literacy rate is high | The civ can now *label* its territory, monuments, and treaties — but the substrate is not modified. The *upward causation* here is on the *legends* layer (per `legends-engine.md`): named entities (settlements, heroes, deities) are more stable in literate civs, which is *measured* in the saga graph as a higher proportion of named nodes. | Generational: a writing system takes 2–3 generations to reach functional literacy. |
| `compositionality_index` falls below `compositionality_collapse_threshold` (default 0.05) | A `compositionality_lost.v1` event is emitted; the existing `civ-ai` consumer may treat this as a "language fragmentation" signal. No substrate write. | 1 tick lag. |
| A civ's `LexiconSnapshot` is *borrowed into* a neighboring civ's lexicon via `borrow_word` (existing `engine::language` substrate) | The contact-edge weight between the two civs in the existing `civ-culture-emergent.md` `phase_emergence` contact-builder rises (a soft signal: lexical exchange implies cultural exchange implies contact). The *magnitude* of the rise is bounded by `borrow_weight_max = 0.1` (per tick) and decays with `borrow_weight_decay = 0.05` per tick. No new edge; the existing contact-edge builder reads the borrow log. | 1–2 tick lag. |

The upward-causation pattern is *identical* to `civ-culture-emergent.md` §2.2 and `civ-economy-emergent-markets.md` §2: the language layer emits *events* and *logs* that the substrate / other layers *measure*; the language layer never *writes* the substrate directly. This preserves the charter's no-API-between-silos rule.

### 2.3 The social-graph coupling — the channel for iterated learning

The iterated-learning pathway runs *through* the existing `SocialGraph` ties:

- **Parental exposure** — a child's input set is the union of morphemes produced by the *strongest ties* (top-`k_parents` by `Tie::familiarity + Tie::kinship`, default `k_parents = 4`) among the parent's generation in the same cluster. The exposure is weighted by tie weight, mirroring the existing `belief_culture_exposure` aggregation in `crates/agents/src/psyche.rs:220-246`.
- **Cross-cluster borrowing** — the existing `borrow_word` in `engine::language.rs:67-71` is the substrate for cross-civ lexical exchange. The new path is the *trigger*: when a cluster's contact-edge weight to a neighbor rises above `borrow_threshold` (default 0.6), the language engine *samples* a small number of the neighbor's morphemes (default `borrow_sample_size = 8`, sampled by frequency × meaning-novelty) and applies `borrow_word` for each.
- **Trust-weighted transmission fidelity** — a child's induction is *noisier* when the parental tie is low-trust (per `Tie::trust`): `induction_noise = (1 - tie.trust) × induction_jitter`. This is the substrate for *epistemic* language change: a child who distrusts a parent is more likely to invent new forms (a small effect; default `induction_jitter = 0.05`).
- **Register selector** — the existing two-register system (cultural vs. formal, per `emergent-systems-spec.md` §1.2) is *implemented* at the language layer: the cultural register uses the *full* lexicon (including low-frequency archaic forms); the formal register uses the *top-frequency* forms only. The register selector is a pure function `(lexicon, register: Register) -> FilteredLexicon` called by the consumer (legends for cultural; diplomacy for formal; same as the existing `emergent-systems-spec.md` §2.6 / §2.8 split).

### 2.4 The emergence-dashboard extension

Per `emergence-dashboard.md` §5, the `emergence.metrics.v1` replay-bus event is extended with the language-specific metrics:

| Metric | How to compute | Target signature (healthy language) | Failure mode |
|---|---|---|---|
| **Lexicon size** | Mean number of `(meaning, morpheme)` pairs per civ at the sample tick | Power-law (a few large lexicons + many small) | All civs at `lexicon_size = 0` (language death); all civs at the same size (no variation, the bottleneck has homogenized) |
| **Compositionality index** | Proportion of meanings whose morpheme shares a phoneme with another morpheme (per §1.4) | Moderate (0.3..0.7) — high enough that the language is learnable; low enough that morphemes are not all homophones | Near 0 (no compositionality; iterated learning is broken or `β > 0.7`); near 1.0 (over-compositionality; the language has become a perfect agglutinating mush) |
| **Bottleneck loss rate** | Mean `1 - |child_input| / |parent_output|` across civs per generation | `[0.3, 0.6]` — within the canonical Kirby range | `> 0.8` (the bottleneck is so tight that no language is being transmitted); `< 0.1` (the bottleneck is loose, no compositionality pressure) |
| **Induced phoneme entropy** | Shannon entropy of the induced phoneme distribution (per §1.4) | `[2.0, 4.0]` bits (3–16 phonemes in effective use) | Near 0 (one-phoneme language; pathological); near `log2(64) = 6` (too many phonemes; nobody can learn the inventory) |
| **Closed-class share** | Per-civ `GrammarProfile::closed_class_share` (§1.5) | `[0.1, 0.3]` (real human languages: 10–30% of unique morphemes are closed-class; per Pagel et al. 2007) | Near 0 (no grammar; the language is purely content-bearing); near 1.0 (everything is function; the language is unlearnable) |
| **Bigram entropy** | Per-civ `GrammarProfile::bigram_entropy` | `[2.0, 4.0]` bits over a 256-tick window | Near 0 (degenerate bigrams; the language has only one phrase); near `log2(|V|)` (random bigrams; no syntax) |
| **Orthography lag (ticks)** | Mean `(orthography_update_tick - phonology_change_tick)` across all phoneme changes that have orthographic consequences | Long-tail (most changes are *unwritten*; a few recent ones catch up over time) | Zero (orthography tracks phonology perfectly; the lag knob is broken); unbounded (orthography is frozen; the script does not reflect the language) |
| **Cross-civ borrow rate** | `borrow_word` events per civ per 1024-tick window | Low and stable; borrowing is a *moment*, not a steady state | Sustained high (continuous borrowing; the civs are merging into one) |
| **MI (lexicon, belief)** | Mutual information between cluster-belief histogram and cluster-lexicon histogram (per `emergence-dashboard.md` §3.5) | Moderate (0.2..0.6) — the lexicon and the belief system are coupled but not deterministic | Near 0 (no coupling; the social-graph channel is broken); near 1.0 (one is a function of the other; the iterated-learning channel is dominating) |
| **MT-LANG-001** alarm | `lexicon_size_mean = 0` for 2048 ticks | (alarm) | language death (the iterated-learning chain has stopped; no new generations are emerging) |
| **MT-LANG-002** alarm | `compositionality_index = 0` for 4096 ticks | (alarm) | iterated learning is broken (the bottleneck is not forcing compositionality; the induction step is not running) |

The `emergence.alarm.v1` event is extended with `MT-LANG-001` and `MT-LANG-002`; the `MT-CUL-001` / `MT-CUL-002` / `MT-CUL-003` culture alarms from `civ-culture-emergent.md` §5 are unchanged.

---

## 3. The English default + native toggle

### 3.1 The two render paths

The Civis renderer maintains **two parallel glyph atlases** for every civ in view:

1. **The English default atlas** — a pre-built system-font atlas (e.g. `IBM Plex Sans` or `Inter` — to be selected by the user in the mod-host UI; not in scope for this spec). This atlas is *always* present; it never changes; it is the *fallback* for any semantic key that has no civ-native entry.
2. **The civ-native atlas** — a runtime-materialized TTF (per FR-CIV-LANG-007) built from the per-civ `ScriptSignature` (§1.6). The atlas is generated lazily on first read of the civ's vocabulary; cached in `civ-watch` keyed by `(cluster_id, ScriptSignature)`.

The *semantic key* layer (per FR-CIV-LANG-001 and `emergent-systems-spec.md` §1.2) is the bridge: every player-facing string is keyed by a `semantic_key: u32` (e.g. `key=42_539 = "Treaty_of_..."`); the *display* is a lookup `lexicon.lookup(meaning = semantic_key) -> Option<Morpheme>`. If `None`, fall back to English (the system atlas). If `Some(morpheme)`, render through the civ-native atlas.

### 3.2 The toggle — read-only, no substrate mutation

The per-civ *view language* toggle is a `civ-watch` HTTP cookie + a `civ-server` WS subscription flag (`view_lang: ViewLang { civ_id: ClusterId, mode: ViewMode }` where `ViewMode = English | Native | Both`). The toggle:

- **Never mutates the live language state.** The toggle is a *render* setting, not a *world* setting.
- **Never affects the simulation.** A sim tick is deterministic; the toggle is a *presentation* concern, like camera position.
- **Defaults to `English`** for the player (the player is human; English is the human-UI language). A player may flip a specific civ to `Native` to see the civ's actual emergent lexicon rendered in the civ's emergent script.
- **`Both` mode** is the *learning* mode: the renderer shows the English text *underneath* the civ-native text in a smaller font, so the player can learn what the civ-native terms mean (a la in-game codex / dictionary). This is the spec's bridge to the *Civis Codex* (per `emergent-systems-spec.md` §1.2 — "narrative, not authored dialogue trees").
- **The toggle is per-civ, not global.** A player may have civ-A in `Native` (so they see the civ-A script) and civ-B in `English` (so they see a translation). This is the same pattern as Civilization's per-civ language / diplomacy screens.

### 3.3 What the toggle *cannot* do

- It cannot *re-translate* the live language into English. The English rendering is the *fallback*; the live lexicon is the *primary*; the toggle controls which is *displayed*, not what the sim holds.
- It cannot *hide* a civ's emergent language. The civ's lexicon is always a *queryable* artifact (the inspector / legends / diplomacy tools read the *lexicon*, not the rendered text).
- It cannot *bias* the iterated-learning dynamics. The dynamics are a function of the substrate + the language engine; the toggle is a render-layer concern, separate from the language engine. (This is enforced by the *no-API-between-render-and-sim* charter rule.)

### 3.4 The render pipeline

```
player input  →  civ-server  →  sim.tick()  →  sim.snapshot()
                                                     │
                                                     ▼
                                          LexiconSnapshot for cluster
                                                     │
                                                     ▼
                                          ScriptSignature cached?
                                                     │
                                       ┌─────────────┴─────────────┐
                                       │                           │
                                  yes, cached                    no
                                       │                           │
                                       ▼                           ▼
                                use cached atlas          materialize TTF from
                                (English or Native)       ScriptSignature via
                                       │                   ab_glyph / cosmic-text
                                       │                           │
                                       └─────────────┬─────────────┘
                                                     │
                                                     ▼
                                          semantic_key → morpheme
                                                     │
                                                     ▼
                                          render glyph for morpheme
                                          (English fallback if missing)
                                                     │
                                                     ▼
                                          composite frame
```

The `materialize_ttf` function (existing per FR-CIV-LANG-007 — this spec does *not* re-implement; the runtime TTF builder is in `crates/lang/src/materialize.rs` per the plan in `emergent-systems-spec.md` §2.4 E5.3) takes a `ScriptSignature` byte-vector and returns a `Vec<u8>` containing a TTF blob that the renderer wraps in a font face. The TTF is the *output* of the projection; the projection is *derived* from the live state; the TTF is *cached*; the cache is *invalidated* when the projection changes.

---

## 4. Criticality knobs — edge of chaos

All knobs concentrate in a new `LanguageEmergenceParams` struct on `EmergenceState` (loaded from scenario RON, same pattern as `civ-003-emergent-lifecycle.md` §3 and `civ-economy-emergent-markets.md` §4). Defaults target weak emergence (Class 4): the iterated-learning chain is *transmitting* but *adapting*, the grammar has *structure* but *flexibility*, the script has *identity* but *evolves*.

| Parameter | Type | Default | Effect on language dynamics | Heat-death direction | Explosion direction |
|---|---|---|---|---|---|
| `bottleneck_fraction` | `f32` | `0.4` (Kirby 1999 canonical) | `β` in §1.4; proportion of parental output the child sees | `> 0.7` (no compositionality pressure; language becomes holistic / unlearnable) | `< 0.1` (over-compression; the language loses information each generation) |
| `language_drift_rate` | `f32` | `0.05` (existing in `language.rs:42`) | `μ` in `tick_language`; per-tick gaussian sigma on phoneme features | `0.0` (language is frozen; no evolution) | `> 0.3` (language is noise; no conventional forms) |
| `contact_split_threshold` | `f32` | `0.35` (existing in `language.rs:43`) | `split_threshold` in `LanguageState`; the distance at which `should_split` fires | `> 1.0` (no language ever splits; one language per world) | `0.0` (every micro-drift is a split; thousands of micro-languages) |
| `creole_threshold` | `f32` | `0.85` (existing in `culture.rs:175,190`) | `language_distance` in `civ-culture-emergent.md` §4 at which the second-pass creolization in `culture.rs:157-164` fires | `> 1.5` (no creolization; languages fragment) | `0.0` (every contact creolizes; all languages merge) |
| `induction_jitter` | `f32` | `0.05` | `μ_induction` in §1.3; per-induction gaussian noise on the child's inferred phoneme features | `0.0` (deterministic induction; languages converge by force) | `> 0.2` (induction is noise; no transmission) |
| `k_parents` | `u32` | `4` | Number of strongest ties that contribute to a child's input set (§2.3) | `1` (single-parent transmission; fragile) | `> 32` (over-diverse input; no convention emerges) |
| `borrow_sample_size` | `u32` | `8` | Number of morphemes sampled from a high-contact neighbor per borrow tick (§2.3) | `0` (no borrowing; isolation hardens) | `> 64` (over-borrowing; the civ loses its identity) |
| `borrow_threshold` | `f32` | `0.6` | Contact-edge weight at which a civ begins borrowing from a neighbor | `> 1.0` (no borrowing; total isolation) | `0.0` (every contact borrows; identity lost) |
| `borrow_weight_max` | `f32` | `0.1` | Maximum contact-edge weight increase per tick from a borrow event (§2.2) | — | `> 0.5` (borrowing dominates the contact graph) |
| `borrow_weight_decay` | `f32` | `0.05` | Per-tick decay on the borrow-induced contact-edge weight | `0.0` (borrows are permanent; the contact graph saturates) | `1.0` (borrows are instantaneous; the contact graph never reflects borrowing) |
| `phonotactic_T_bias` | `f32` | `0.2` | `α_T` in §2.1; the strength of temperature's bias on the phonotactic profile | `0.0` (climate has no effect on phonology) | `> 1.0` (climate dominates phonology; the language is climate-determined) |
| `drift_M_sensitivity` | `f32` | `0.15` | `α_M` in §2.1; the strength of moisture's effect on drift rate | `0.0` (moisture has no effect) | `> 0.5` (drift is weather-determined) |
| `closed_class_E_sensitivity` | `f32` | `0.3` | Strength of energy's effect on closed-class survival (§2.1) | `0.0` (energy has no effect on grammar) | `> 1.0` (energy dominates grammar; starved civs lose structure) |
| `learnable_biomass_coupling` | `f32` | `0.4` | `β_biomass` in §2.1; the strength of biomass's effect on learnable inventory size | `0.0` (biomass has no effect on lexicon) | `> 1.0` (biomass dominates lexicon; starvation collapses the language) |
| `compositionality_collapse_threshold` | `f32` | `0.05` | Threshold below which the `compositionality_lost.v1` event fires (§2.2) | `> 0.5` (false alarms; compositionality drops are not pathological) | `0.0` (never fires; compositionality collapse is undetected) |
| `orthography_lag` | `u32` | `512` | Mean ticks between phonology change and orthography update (§1.6) | `> 8192` (orthography is frozen; the script does not reflect the language) | `0` (orthography tracks phonology instantly; the lag knob is broken) |
| `view_lang_default` | `enum` | `English` | The default `ViewMode` for a new player session (§3.2) | — | — |

All 17 knobs are grouped in one `LanguageEmergenceParams` struct (not scattered across modules) and loaded from the scenario RON config. The emergence dashboard (§2.4) plots a real-time criticality indicator so the designer can see whether the system is heading toward heat-death (one language, one script, no compositionality, no transmission) or explosion (thousands of micro-languages, compositionality churn, script-cache thrash) before adjusting.

---

## 5. Coupling to existing systems — extension sites, not replacements

This spec *extends* the existing substrate; it does not replace it. The extension sites are:

### 5.1 Extension to `crates/engine/src/language.rs`

Add (do not replace):

- `pub fn generate_child_induced_lexicon(parental_output: &[Morpheme], bottleneck: f32, rng: &mut ChaCha8Rng) -> LexiconSnapshot` — the induction step (§1.3).
- `pub fn tick_language_with_induction(lang: &mut LanguageState, parental_output: &[Morpheme], contact_pressure: f32, bottleneck: f32, rng: &mut ChaCha8Rng)` — the new entry point that combines `tick_language` + `generate_child_induced_lexicon` for one generation.
- `pub fn compute_grammar_profile(lexicon: &LexiconSnapshot, bigram_log: &BigramLog) -> GrammarProfile` — the grammar induction (§1.5).
- `pub fn project_writing_system(lang: &LanguageState, grammar: &GrammarProfile, phon: &PhonotacticProfile, substrate: &WritingSystemSubstrate, lag_ticks: u32) -> ScriptSignature` — the writing-system projection (§1.6).
- `pub fn classify_language_register(lexicon: &LexiconSnapshot, register: Register) -> FilteredLexicon` — the register selector (§2.3).

### 5.2 Extension to `crates/agents/src/culture.rs`

The existing `drift_populations` (substrate correct per `civ-culture-emergent.md` §0) is extended with one new arg: a *language* induced-lexicon update after the drift step. The added step is bounded: `μ_induction × (1 - kinship)`; the existing kinship-insulation logic is preserved.

### 5.3 Extension to `crates/agents/src/psyche.rs`

The existing `belief_culture_exposure` (substrate correct per `psyche-social.md` §3.3) is extended with a *language exposure* term: a child whose `Tie::familiarity × |meaning_novelty|` is high (i.e. the child is hearing novel terms from the parent) drifts its beliefs toward the parent's belief centroid faster (the *epistemic* channel of the iterated-learning pathway). The extension is bounded; the existing `belief_sociability_lr` knob is preserved.

### 5.4 Extension to `crates/agents/src/social.rs`

The existing `apply_social_event` (substrate correct per `psyche-social.md` §3.4) is extended with one new event kind: `Interaction::LexicalExchange { meaning: u32, fidelity: f32 }`. The new event is produced by the language engine on a successful `borrow_word` (per §2.3) and consumed by the social graph to add a `lexical_tie` component to the `Tie` (a small `f32` field tracking the cumulative `fidelity` of lexical exchanges between two agents; the field is *derived* from the borrow log, not a new contact channel). The extension is additive; the existing `decay_social_graph` is unchanged.

### 5.5 Extension to `crates/civ-emergence-metrics/src/dashboard.rs`

The `EmergenceDashboard` struct gains a new field, `language: LanguageDashboard`, computed in a new pure function `compute_language_metrics` that calls the language engine's `generate_child_induced_lexicon` and `compute_grammar_profile` on a per-civ sample. The function is bounded; the existing 5-tile dashboard consumers see no change.

### 5.6 Extension to `crates/legends/src/worker.rs`

The existing `RawSimEvent` ingest path (per `emergent-systems-spec.md` §2.6) is extended with three new event kinds: `linguistic_collapse.v1`, `compositionality_lost.v1`, `script_materialized.v1`. The events are produced by the language engine on the corresponding transitions; consumed by the legends worker to tag chronicle / rumor chains with the linguistic state. The extension is additive; the existing `LegendsWorker` and `SagaGraph` are unchanged.

### 5.7 Extension to `crates/legends/src/rumor.rs`

The existing rumor-hop mutation (per `emergent-systems-spec.md` §2.6) is extended with a *register mutation* step: a rumor that hops through a region with `closed_class_share < 0.1` is more likely to drop function morphemes (becoming "telegraphic"); a rumor that hops through a region with `bigram_entropy > 0.8` is more likely to *shuffle* morpheme order (becoming "pidgin"). The extension is bounded; the existing mutation steps are unchanged.

### 5.8 Extension to `crates/audio/src/lib.rs`

The existing `MusicalTradition` projection (per `emergent-systems-spec.md` §2.5 / `audio-direction.md`) is extended with one new input: `GrammarProfile::bigram_entropy`. The intuition: a language with high bigram entropy (loose syntax) tends to *rhythmically simple* music; a language with low bigram entropy (rigid syntax) tends to *rhythmically complex* music (Everett 2013 §7 on language–music coupling). The extension is additive; the existing `material-gated instruments` path is unchanged.

### 5.9 No extension to `crates/{diplomacy, economy, laws, tactics, planet, climate, voxel, needs}`

These crates do not need to change. They consume the new `LanguageDashboard` field via the existing `sim.snapshot` JSON-RPC method (the same field already used by `civ-culture-emergent.md` §1.3 / §2.4); they emit no API calls to the language layer. The coupling is purely through the shared substrate (per §2.1 + §2.2) and the `sim.snapshot` read-out.

---

## 6. Phased implementation plan (DAG)

This is a planner-only spec; file paths, struct names, and function signatures are identified for the implementing agent. The plan *extends* the existing `emergent-systems-spec.md` E5.0-E5.3 WBS and the `civ-culture-emergent.md` Phase 0-7 WBS. No phase here duplicates a phase there; each phase here adds a *new* piece on top of the substrate those specs defined.

### Phase L0 — Prerequisite audit (no new structs)

| Task | File | Depends on | Agent effort |
|---|---|---|---|
| L0-A: Verify `tick_language` is the sole mutation path for `LanguageState` (`language.rs:48-60`); audit the determinism contract (the `blake3` hash is seeded by `tick + drift_rate + split_threshold + contact_pressure + vocabulary bytes`) | `crates/engine/src/language.rs` | — | 2 tool calls |
| L0-B: Audit `crates/agents/src/culture.rs` `drift_populations` and confirm the language-distance / creolization path is unchanged (substrate correct per `civ-culture-emergent.md` §0) | `crates/agents/src/culture.rs` | — | 2 tool calls |
| L0-C: Map which existing signals can feed `contact_pressure` to `tick_language` from the physics substrate: `|∇P|` at the cluster centroid (per `PHYSICS_COUPLING_SUBSTRATE.md` §3.1), `M_centroid`, `E_centroid`, `B_centroid`; the audit identifies the data flows | `crates/agents/src/culture.rs`, `crates/climate/src/lib.rs`, `crates/voxel/src/fluid_ca.rs` | — | 3 tool calls |
| L0-D: Audit `crates/legends/src/worker.rs` and confirm the existing `RawSimEvent` ingest path supports the new `linguistic_collapse.v1` / `compositionality_lost.v1` / `script_materialized.v1` event kinds (additive; new variants) | `crates/legends/src/worker.rs` | — | 2 tool calls |
| L0-E: Audit `crates/civ-emergence-metrics/src/dashboard.rs` to confirm the existing `EmergenceDashboard` struct's serialization format; the new `language: LanguageDashboard` field is an additive change, not a breaking one | `crates/civ-emergence-metrics/src/dashboard.rs` | — | 2 tool calls |
| L0-F: Audit `crates/agents/src/social.rs` to confirm the existing `Tie` struct is `Serialize`/`Deserialize`; the new `lexical_tie: f32` field is additive | `crates/agents/src/social.rs` | — | 1 tool call |

### Phase L1 — The Kirby iterated-learning bridge in `crates/engine/src/language.rs`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L1-A: Add `pub struct LexiconSnapshot { entries: BTreeMap<u32, Morpheme>, bigram_log: BigramLog, generation: u32 }` and `pub struct BigramLog { pairs: BTreeMap<(u32, u32), u32> }` to `crates/engine/src/language.rs` | `crates/engine/src/language.rs` | L0-A | Pure data; `BigramLog` is small (256 most-recent bigrams per civ, ring-buffered) |
| L1-B: Add `pub fn generate_child_induced_lexicon(parental_output: &[Morpheme], bottleneck: f32, rng: &mut ChaCha8Rng) -> LexiconSnapshot` | same | L1-A | Pure fn; the induction step per §1.3. Deterministic. |
| L1-C: Add `pub fn tick_language_with_induction(lang: &mut LanguageState, parental_output: &[Morpheme], contact_pressure: f32, bottleneck: f32, rng: &mut ChaCha8Rng)` | same | L1-B | The new entry point; calls `tick_language` then `generate_child_induced_lexicon`; updates `lang.vocabulary` with the induced snapshot. |
| L1-D: Add `pub fn borrow_with_log(lang: &mut LanguageState, source: &LanguageState, meaning: u32, log: &mut BorrowLog) -> bool` | same | L1-A | Wraps the existing `borrow_word` (substrate correct); logs the borrow event for the §2.3 contact-edge builder. |
| L1-E: Tests: L1-B same input + same seed → same output; L1-C two civs with the same `tick` and same `drift_rate` produce the same induction (function-purity); L1-D a borrow event lands in the `BorrowLog` | `crates/engine/src/language.rs` tests | L1-A..D | Property-based via `proptest` consistent with the existing tests |

### Phase L2 — The grammar induction layer in `crates/engine/src/language.rs`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L2-A: Add `pub struct GrammarProfile { open_class_share: f32, closed_class_share: f32, bigram_entropy: f32, compositionality: f32, tick: u32 }` to `crates/engine/src/language.rs` | `crates/engine/src/language.rs` | L1-A | Small POD struct; derived from `LexiconSnapshot` + `BigramLog` |
| L2-B: Add `pub fn compute_grammar_profile(lexicon: &LexiconSnapshot, bigram_log: &BigramLog, e_centroid: f32, e_sensitivity: f32) -> GrammarProfile` | same | L2-A, L0-C | Pure fn; reads `E_centroid` for the closed-class-survival bias per §2.1. The `closed_class_share` is `closed_count / (open_count + closed_count)` * `f(E_centroid)`. |
| L2-C: Add `pub enum Register { Cultural, Formal }` and `pub fn classify_language_register(lexicon: &LexiconSnapshot, register: Register) -> FilteredLexicon` | same | L2-A | Cultural uses the full lexicon; Formal uses the top-frequency band only (top 70% by frequency). |
| L2-D: Tests: L2-A structural sanity; L2-B at `e_centroid = 1.0` (high energy) `closed_class_share` is in `[0.2, 0.4]` (real-language band); at `e_centroid = 0.0` `closed_class_share` falls below 0.1 (energy-deficit grammar); L2-C Formal filter excludes low-frequency morphemes | `crates/engine/src/language.rs` tests | L2-A..C | Property-based |

### Phase L3 — The writing-system projection in `crates/engine/src/language.rs`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L3-A: Add `pub struct WritingSystemSubstrate { kind: WritingSubstrateKind, material_requirements: BTreeMap<MaterialId, f32> }` and `pub enum WritingSubstrateKind { Logographic, Syllabic, Alphabetic, Abjad, Abugida, Oral }` to `crates/engine/src/language.rs` | `crates/engine/src/language.rs` | L0-A | The `WritingSubstrateKind` is the *only* authored enum in the language pipeline; it is a property of the *starting* civ preset, not the live state (per §1.6) |
| L3-B: Add `pub fn derive_substrate_from_flux(f_centroid: BTreeMap<MaterialId, f32>) -> WritingSystemSubstrate` | same | L3-A, L0-C | Pure fn; maps `F` at the cluster centroid to the substrate per §2.1 (clay → Logographic; papyrus → Syllabic; stone + parchment + none-of-the-above → Alphabetic; all zero → Oral) |
| L3-C: Add `pub fn project_writing_system(lang: &LanguageState, grammar: &GrammarProfile, phon: &PhonotacticProfile, substrate: &WritingSystemSubstrate, lag_ticks: u32) -> ScriptSignature` | same | L2-B, L3-A, L0-E of `civ-culture-emergent.md` | Pure fn; extends the existing `project_script` from `civ-culture-emergent.md` §1.2 with the substrate-conditional projection. The `orthography_lag` knob (L3-D) gates when the projection actually updates. |
| L3-D: Add `pub struct LanguageEmergenceParams { /* all §4 knobs */ }` to `crates/engine/src/language.rs` | same | L3-C | `Default::default()` returns the §4 defaults; `#[serde(default)]` for scenario RON loading. |
| L3-E: Tests: L3-B clay-heavy → Logographic; L3-C the same language + different substrate → different `ScriptSignature`; L3-D default params match §4 table; L3-C at `orthography_lag = 0` the signature tracks phonology every tick; at `orthography_lag = 8192` the signature is frozen | `crates/engine/src/language.rs` tests | L3-A..D | Property-based |

### Phase L4 — The social-graph coupling in `crates/agents/src/social.rs`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L4-A: Add `pub enum Interaction { ..., LexicalExchange { meaning: u32, fidelity: f32 } }` (additive variant) and a `lexical_tie: f32` field to `Tie` | `crates/agents/src/social.rs` | L1-D, L0-F | The `lexical_tie` is updated by `apply_social_event` on `LexicalExchange`; bounded to `[0, 1]`. |
| L4-B: Add `pub fn apply_lexical_exchange(graph: &mut SocialGraph, a: u64, b: u64, meaning: u32, fidelity: f32, tick: u32)` to `crates/agents/src/social.rs` | same | L4-A | Pure fn; the new social event handler. |
| L4-C: Tests: L4-A the enum is `Serialize`/`Deserialize`; L4-B a `LexicalExchange` between two agents with no prior tie creates a new tie with `lexical_tie = fidelity`; repeated exchanges accumulate (saturating at 1.0) | `crates/agents/src/social.rs` tests | L4-A, L4-B | Property-based |

### Phase L5 — The shared-substrate coupling in `crates/agents/src/culture.rs` + `crates/climate/src/lib.rs`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L5-A: Add `pub struct LanguageSubstrate { t_centroid: f32, m_centroid: f32, e_centroid: f32, f_centroid: BTreeMap<MaterialId, f32>, b_centroid: f32, contact_pressure: f32 }` to `crates/agents/src/culture.rs` | `crates/agents/src/culture.rs` | L0-C | Pure data; the projection of the shared substrate onto the cluster's centroid |
| L5-B: Add `pub fn sample_language_substrate(world: &World, cluster_id: ClusterId, cluster_centroid: WorldCoord) -> LanguageSubstrate` to `crates/agents/src/culture.rs` | same | L5-A, L0-C | Pure fn; walks the world, samples the substrate fields at the centroid, returns the snapshot. The `contact_pressure` is `|∇P|` at the centroid (per `PHYSICS_COUPLING_SUBSTRATE.md` §3.1). |
| L5-C: Add `pub fn drift_populations_with_substrate(profiles: &mut [CultureProfile], edges: &[ContactEdge], substrate: &LanguageSubstrate, params: &LanguageEmergenceParams, rng: &mut impl Rng)` to `crates/agents/src/culture.rs` | same | L5-B, L1-C, L3-D | Extends the existing `drift_populations` (substrate correct) with: (a) the `substrate` read, (b) the drift rate scaled by `M_centroid` (per §2.1), (c) the `tick_language_with_induction` call per generation (per L1-C), (d) the `compute_grammar_profile` call per generation (per L2-B), (e) the `derive_substrate_from_flux` call per generation (per L3-B). The original `drift_populations` is preserved for back-compat. |
| L5-D: Wire L5-C into `phase_emergence` after the `drift_populations` call in `emergence_culture` (`emergence.rs:190-193`); the new call is conditional on `params.enable_substrate_coupling == true` so the existing tests can opt out | `crates/engine/src/emergence.rs` | L5-C | One-line integration |
| L5-E: Tests: L5-B the substrate sampling is pure (same world → same substrate); L5-C with `substrate_coupling = false` is a no-op (preserves the existing `culture.rs:222-249` `kinship_resists_external_contact` test); L5-C with `substrate_coupling = true` and `M_centroid = 0.0` (arid) the drift rate is measurably higher than at `M_centroid = 1.0`; the existing integration test in `emergence.rs:699-710` (`culture_phase_drifts_cluster_profiles`) still passes | `crates/agents/src/culture.rs` tests + engine integration | L5-A..D | Property-based |

### Phase L6 — The dashboard + metrics layer in `crates/civ-emergence-metrics/src/dashboard.rs`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L6-A: Add `pub struct LanguageDashboard { /* all §2.4 fields */ }` to `crates/civ-emergence-metrics/src/dashboard.rs` | `crates/civ-emergence-metrics/src/dashboard.rs` | L1-C, L2-B, L0-E | Small POD struct |
| L6-B: Add `pub fn compute_language_metrics(world: &World, sample_window: &SampleWindow, params: &LanguageEmergenceParams) -> LanguageDashboard` to `crates/civ-emergence-metrics/src/dashboard.rs` | same | L6-A, L5-B | Pure fn; calls the language engine's `generate_child_induced_lexuction` and `compute_grammar_profile` on a per-civ sample |
| L6-C: Add `pub language: LanguageDashboard` field to `EmergenceDashboard` (`dashboard.rs:39-54`); add a new `compute` overload that takes the language inputs and routes to L6-B; preserve the existing 5-tile `compute` for back-compat | same | L6-B | Additive: existing 5-tile dashboard consumers see no change; the new `language` field is the 6th tile |
| L6-D: Wire `compute_language_metrics` into the existing `sample_emergence` pass in `crates/engine/src/emergence_metrics.rs`; the call site already walks the world for cluster cultures, so no new ECS iteration | `crates/engine/src/emergence_metrics.rs` | L6-B, L6-C | The new field is added to `EmergenceSample::dashboard` (an additive change to the JSON-RPC `sim.snapshot` result) |
| L6-E: Extend `emergence.metrics.v1` replay-bus event with the §2.4 `lang_*` fields; extend `emergence.alarm.v1` with `MT-LANG-001` / `MT-LANG-002` thresholds | `crates/engine/src/emergence.rs` | L6-C, L6-D | Same wire format, new fields. Charter: no API break. |
| L6-F: Tests: L6-A default params match §4 table; L6-B all 9 metrics are bounded in `[0, 1]` (or the documented unbounded range) for populations of 10, 100, 1000 agents; L6-C the existing `emerg_emerg_001_dashboard_compute_combines_all_five` test still passes; L6-D `sample_emergence` populates the new `language` field without breaking the existing 5 tiles; L6-E replay event round-trips through `bincode` for the new fields | `crates/civ-emergence-metrics/src/dashboard.rs` tests + engine integration | L6-A..E | Property-based |

### Phase L7 — The render-layer toggle + civ-watch cookie in `crates/watch/`

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L7-A: Add `pub struct ViewLang { civ_id: ClusterId, mode: ViewMode }` and `pub enum ViewMode { English, Native, Both }` to `crates/watch/src/session.rs` (or the appropriate session module) | `crates/watch/` | L3-C | The toggle structure |
| L7-B: Add `pub fn view_lang_for(civ_id: ClusterId, session: &Session) -> ViewMode` to `crates/watch/` | same | L7-A | Pure fn; reads the session cookie |
| L7-C: Add a `view_lang` JSON-RPC method to `civ-server` that takes `ViewLang` and updates the session | `crates/server/` | L7-B | The mutator; updates the cookie, not the sim state |
| L7-D: Extend the `sim.snapshot` JSON-RPC result with a per-civ `available_scripts: BTreeMap<ClusterId, ScriptSignature>` so the client knows which scripts to render | `crates/server/` | L3-C | Additive |
| L7-E: Tests: L7-A struct is `Serialize`/`Deserialize`; L7-B a fresh session has `ViewMode::English`; L7-C the JSON-RPC method is idempotent (calling twice with the same value is a no-op); L7-D the snapshot extension preserves the existing fields | `crates/watch/` tests + `crates/server/` tests | L7-A..D | Property-based |

### Phase L8 — The legends / rumor / audio extensions

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L8-A: Add `RawSimEvent` variants `linguistic_collapse.v1`, `compositionality_lost.v1`, `script_materialized.v1` to `crates/legends/src/worker.rs` | `crates/legends/src/worker.rs` | L1-C, L2-B, L3-C, L0-D | Additive event kinds |
| L8-B: Extend `crates/legends/src/rumor.rs::mutate_rumor` with the *register mutation* step (§5.7) | `crates/legends/src/rumor.rs` | L8-A, L2-B | Bounded: the mutation probability is `(0.1 - closed_class_share)` and `(bigram_entropy - 0.5)` respectively |
| L8-C: Extend `crates/audio/src/lib.rs` `MusicalTradition` projection with the `bigram_entropy` input | `crates/audio/src/lib.rs` | L2-B | Additive; the existing `culture_vec + available_materials` projection is unchanged |
| L8-D: Tests: L8-A event ingest round-trips; L8-B a rumor through a `closed_class_share = 0.0` region drops ~50% of function morphemes; L8-C `bigram_entropy = 0.0` → "simple rhythm" projection; `bigram_entropy = 0.8` → "complex rhythm" projection | `crates/legends/` + `crates/audio/` tests | L8-A..C | Property-based |

### Phase L9 — Coupling to `phase_emergence` orchestrator

| Task | Target file | Depends on | Notes |
|---|---|---|---|
| L9-A: The `phase_emergence` orchestrator (`emergence.rs:115-127`) is already `genetics → culture → social → psyche → legends → civ-ai`. The new `emergence_language` sub-phase slots between `emergence_psyche` and `emergence_legends` (per §1.2 — the child induction needs the parental belief state). Verify the call order: language must finish before legends (legends ingests the new `linguistic_collapse.v1` / `compositionality_lost.v1` / `script_materialized.v1` events) | `crates/engine/src/emergence.rs` | L1-C, L2-B, L3-C, L5-C, L6-D, L8-A | No new orchestrator code; just verification that the existing order supports the new work |
| L9-B: The cross-civ borrow contact-edge weight update (§2.2) is a one-line extension to the existing `phase_emergence` contact-builder; the builder already walks the world and the borrow log is in scope | `crates/engine/src/emergence.rs` | L1-D, L5-C | One-line integration: `for borrow in borrow_log.iter() { edges[borrow.civ_pair].weight = (edges[borrow.civ_pair].weight + borrow_weight_max * borrow.fidelity).min(1.0); }` |
| L9-C: Add a CI-level invariant test: assert that NO production code path mutates `LanguageState.vocabulary` outside of `tick_language` + `tick_language_with_induction` + `borrow_word` + `borrow_with_log` (per `civ-culture-emergent.md` P5-E parallel) | `crates/engine/src/language.rs` tests | Phase L1 complete | Closes the loop on the charter: there is no other way for the language to change |

### Phase L10 — Acceptance criteria reachability (validation)

| Task | File | Depends on | Notes |
|---|---|---|---|
| L10-A: Scenario test: "Kirby iterated learning produces compositional lexicons" — start with 2 civs sharing a 100-morpheme lexicon; run for 20 generations with `bottleneck_fraction = 0.4`; assert `compositionality_index` rises above 0.3 (the canonical Kirby 1999 result) | `crates/engine/src/language.rs` integration test | L1-C | Charter AC: compositionality emerges from the bottleneck, not from authoring |
| L10-B: Scenario test: "grammar induction produces closed-class morphemes" — run a 50-generation scenario; assert `closed_class_share` stabilizes in `[0.1, 0.3]` (the real-language band per Pagel et al. 2007) | same | L2-B | Charter AC: grammar emerges from frequency, not from an authored rule set |
| L10-C: Scenario test: "writing systems vary by substrate" — start 3 civs with the same `LanguageState` but different `WritingSystemSubstrate` (Logographic / Syllabic / Alphabetic); after 10 generations, assert the 3 `ScriptSignature`s are distinct | same | L3-C | Charter AC: scripts are derived, not authored |
| L10-D: Scenario test: "physics coupling shifts phonology" — start 2 civs with the same `LanguageState`; one at `T_centroid = 0.0` (cold), one at `T_centroid = 1.0` (hot); after 20 generations, assert the hot civ's phoneme distribution has shifted measurably from the cold civ's (the `phonotactic_T_bias` from §2.1) | same | L5-C | Charter AC: language responds to climate |
| L10-E: Scenario test: "social coupling shifts transmission" — start 2 civs with the same `LanguageState`; one with `k_parents = 1` (single-parent transmission), one with `k_parents = 8` (diverse input); after 20 generations, assert the single-parent civ's lexicon is measurably more idiosyncratic (higher `induced_phoneme_entropy` is fine, but the *phoneme distribution* should differ from the diverse-input civ) | same | L1-C, L5-C | Charter AC: language responds to social structure |
| L10-F: Scenario test: "English default + native toggle does not mutate state" — start a civ, set the session `ViewMode = Native`, run the sim for 4096 ticks, then set `ViewMode = English`; assert the `LanguageState.vocabulary` is *bit-identical* before and after the toggle (proves the toggle is render-only) | same + `crates/server/` integration test | L7-C | Charter AC: render toggles never mutate sim state |
| L10-G: Scenario test: "criticality edge of chaos" — run a 4096-tick scenario with default `LanguageEmergenceParams`; assert `lexicon_size_mean` is in `[10, 1000]`, `compositionality_index` is in `[0.3, 0.7]`, `bigram_entropy` is in `[2.0, 4.0]` (the §2.4 target bands) | same | All phases complete | Performance-gated; runs in CI as a regression |
| L10-H: Scenario test: "criticality knob reachability" — for each knob in §4, set it to its heat-death and explosion extremes; assert the system reaches the corresponding failure mode within 1024 ticks (`compositionality_index = 0` for heat-death, `bigram_entropy = log2(|V|)` for explosion); assert setting it back to default returns the system to the target band within 2048 ticks | same | L10-G | Ensures the knobs are *real* knobs, not constants |

### DAG summary (critical path)

```
L0-* (all parallel) → L1-A..E → L2-A..D (depends on L1-A)
                                  ↓
                                L3-A..E (depends on L1-A, L2-A)
                                  ↓
                                L4-A..C (depends on L1-D)        ┐
                                  ↓                                │ can run in parallel
                                L5-A..E (depends on L1-C, L2-B, L3-D)│
                                  ↓                                │
                                L6-A..F (depends on L1-C, L2-B, L5-C)│
                                  ↓                                │
                                L7-A..E (depends on L3-C)        ┘
                                  ↓
                                L8-A..D (depends on L1-C, L2-B, L3-C, L6-D)
                                  ↓
                                L9-A..C (depends on Phase L5 + L6 + L7 + L8 complete)
                                  ↓
                                L10-A..H (depends on Phase L9 complete)
```

**Critical path to acceptance:** L0 → L1 → L2 → L3 → L5 → L6 → L8 → L9 → L10. L4, L7 can run in parallel with the main path.

---

## 7. Charter (non-negotiable)

1. **No authored languages.** The phoneme inventory, lexicon, grammar, and writing system of every civ are derived from data — Kirby iterated learning + grammar induction + script materialization. No hand-tuned rule sets, no scripted `LanguageState::initial_default` overrides beyond the §4 bootstrap.
2. **Render-only English toggle.** The `ViewMode` cookie is read at render time in `civ-watch` and at RPC time in `civ-server`. It never writes back to `LanguageState` or `LanguageEmergenceParams`. The L10-F test proves this.
3. **No API break.** All wire formats (`sim.snapshot`, `emergence.metrics.v1`, `emergence.alarm.v1`, `RawSimEvent`) gain additive fields only. Existing clients keep working.
4. **No language tuning in the simulation core.** The emergent language engine lives in `crates/engine/src/language.rs`. The borrow layer in `crates/engine/src/language_borrow.rs`. The orchestration in `crates/engine/src/emergence_language.rs`. No language math in the render layer.
5. **Determinism.** `LanguageState` is a pure function of the seed, the world history, and `LanguageEmergenceParams`. `generate_child_induced_lexuction` is deterministic given a fixed `RNG` and fixed `k_parents` parents (no sampling variance). The §1.3 determinism promise is preserved.
6. **Reproducibility.** Every `LanguageState.vocabulary` mutation goes through `tick_language`, `tick_language_with_induction`, `borrow_word`, or `borrow_with_log`. The L9-C invariant test enforces this. There is no other path.
7. **Edge of chaos.** The §4 defaults target the band where compositionality, grammar, and script innovation all coexist. The L10-G regression test enforces this.
8. **Physics and social coupling are real, not decorative.** The §2.1 phonotactic table is read at every drift step; the §2.2 borrow table is read at every contact step. The L10-D and L10-E scenario tests prove the coupling produces measurable differences.

---

## 8. Functional Requirements matrix

| FR ID | Description | Section | Phase |
|---|---|---|---|
| FR-CIV-LANG-011 | Kirby iterated-learning pathway: child learns from a randomly-sampled `k_parents` parents through a bottleneck of `bottleneck_fraction` of the parental lexicon | §2.1 | L1-C |
| FR-CIV-LANG-012 | Phonotactic biasing by `T_centroid`, `M_centroid`, `geom_centroid` (climate + biome coupling) | §2.1 | L5-B |
| FR-CIV-LANG-013 | Grammar induction: closed-class morphemes emerge from frequency thresholding of the induced grammar | §2.2 | L2-B |
| FR-CIV-LANG-014 | Writing-system materialization: `ScriptSignature` derived from `LanguageState.vocabulary + WritingSystemSubstrate` | §2.3 | L3-C |
| FR-CIV-LANG-015 | Cross-civ borrow: `borrow_word` with a fidelity-gated contact-edge weight update | §2.2 | L1-D |
| FR-CIV-LANG-016 | Contact-edge influence on grammar: `closed_class_share` and `bigram_entropy` leak through the contact graph | §2.2 | L2-D |
| FR-CIV-LANG-017 | Cross-civ universal pressure: shared `ConceptId`s acquire shared `MorphemeId`s with a `cross_civ_borrow_weight_max`-bounded rate | §2.2 | L1-D |
| FR-CIV-LANG-018 | Rumor register mutation: function morphemes are dropped and bigram structure degrades per the closed-class and bigram-entropy gates | §2.2, §5.7 | L8-B |
| FR-CIV-LANG-019 | Audio `MusicalTradition` projection reads `bigram_entropy` | §2.2 | L8-C |
| FR-CIV-LANG-020 | Render-only English default + native toggle via `civ-watch` session cookie and `civ-server` JSON-RPC | §3 | L7-A..E |
| FR-CIV-LANG-021 | Dashboard tile for the 9 §2.4 metrics; integrated into `sim.snapshot` | §2.4 | L6-A..F |
| FR-CIV-LANG-022 | Three new replay-bus events: `linguistic_collapse.v1`, `compositionality_lost.v1`, `script_materialized.v1` | §5.7 | L8-A |
| FR-CIV-LANG-023 | Two new emergence alarms: `MT-LANG-001` (linguistic collapse), `MT-LANG-002` (compositionality loss) | §2.4 | L6-E |
| FR-CIV-LANG-024 | Orchestrator ordering: `emergence_language` slots between `emergence_psyche` and `emergence_legends` | §1.2 | L9-A |
| FR-CIV-LANG-025 | Invariant test: NO production code path mutates `LanguageState.vocabulary` outside the four mutator functions | §7 | L9-C |
| FR-CIV-LANG-026 | 8 scenario acceptance tests covering Kirby compositionality, grammar induction, substrate-specific scripts, climate coupling, social coupling, render-toggle idempotence, criticality band, knob reachability | §10 | L10-A..H |

Total: **16 new FR IDs** extending the existing FR-CIV-LANG-001..010 set (which already cover the basic diachronic sound-change engine).

---

## 9. Open questions

1. **Cross-civ universal pressure scope.** §2.2 proposes that *all* shared `ConceptId`s drift toward a shared `MorphemeId`. Should the pressure be limited to a `UniversalPressure::MaxSharedConcepts` cap (e.g., 200)? Without a cap, very dense civilizations with thousands of shared concepts will all converge to one morpheme per concept — this is the degenerate case.
2. **Phonotactic table authority.** §2.1's climate-bias table is hand-curated from linguistic typology. Should it instead be derived from `crates/audio/src/lib.rs` `available_materials` and the `T` / `M` / `geom` centoids? The hand-curated table is faster to ship; the derived table is more in keeping with the "no authored languages" charter.
3. **Script signature rendering.** The §3.3 `ScriptSignature` is a struct of glyph counts. How is it actually rendered to a player? A bitmap font per `ScriptSignature`? A vector glyph path per `MorphemeId`? The render-layer integration is not specified here — that belongs in a separate `fr-script-render.md` spec.
4. **Multiple dialects per civ.** Real languages have dialects. Should a `Cluster` (which currently has one `LanguageState`) instead have a `Vec<LanguageState>` and a per-agent dialect assignment? The §2.2 cross-civ borrow layer could be repurposed as cross-dialect borrow. This is a v2 extension.
5. **Dead languages.** When a `Cluster` is wiped, should its `LanguageState` be archived as a `DeadLanguage` in `crates/legends/`? The legends system already has dead-language hooks. This is a v2 extension.
6. **Language change as `phase_emergence` sub-phase vs. parallel.** §1.2 places language between `emergence_psyche` and `emergence_legends`. Should language instead run in parallel with `emergence_psyche` and synchronize via an event queue? The sequential placement is simpler; the parallel placement allows language and psyche to interact more tightly (e.g., a child hears the parent's belief *and* the parent's language simultaneously).

---

## 10. References

- Kirby, S. (1999). *Function, selection, and innateness: The emergence of language universals*. Oxford University Press.
- Kirby, S., Cornish, H., & Smith, K. (2008). *Cumulative cultural evolution in the laboratory: An experimental approach to the origins of structure in human language*. PNAS 105(31).
- Smith, K., Brighton, H., & Kirby, S. (2003). *Complex systems in language evolution*. Computational Linguistics 29(1).
- Pagel, M., Atkinson, Q. D., & Meade, A. (2007). *Frequency of word-use predicts rates of lexical evolution throughout Indo-European history*. Nature 449.
- Nettle, D. (1999). *Linguistic diversity*. Oxford University Press.
- Atkinson, Q. D. (2011). *Phonemic diversity supports a cultural founder effect model of language expansion from Africa*. Science 332.
- Deutscher, G. (2005). *The Unfolding of Language*. Henry Holt.
- Everett, D. L. (2017). *How Language Began*. Liveright.
- Fenk-Oczlon, G., & Fenk, A. (2008). *Frequency effects on phonological processes: A developmental perspective*. In A. Rawoens, T. Breban, & J. V. L. (Eds.), *Variation, selection, development: Probing the evolutionary model of language change*.
- Labov, W. (1994). *Principles of Linguistic Change, Volume 1: Internal Factors*. Blackwell.
- Trudgill, P. (2011). *Sociolinguistic Typology*. Oxford University Press.
- Thomason, S. G., & Kaufman, T. (1988). *Language Contact, Creolization, and Genetic Linguistics*. University of California Press.
- Hockett, C. F. (1960). *Logical considerations in the study of animal communication*. In W. E. Lanyon & W. N. Tavolga (Eds.), *Animal Sounds and Communication*.
- Sampson, G., Gil, D., & Trudgill, P. (Eds.). (2009). *Language Complexity as an Evolving Variable*. Oxford University Press.
- Perfors, A., & Regier, T. (2006). *Learning to learn: The emergence of structure in iterated learning*. Cognitive Science 30.

Internal references:
- `crates/engine/src/language.rs` (existing diachronic sound-change engine; the substrate this design extends)
- `crates/agents/src/culture.rs` (existing high-level language vector; the substrate this design supersedes for the language field)
- `crates/agents/src/psyche.rs` (existing belief state; the `LanguageState.belief_association` is keyed by these beliefs)
- `crates/audio/src/lib.rs` (existing `MusicalTradition` projection; extended in L8-C)
- `crates/legends/src/rumor.rs` (existing `mutate_rumor`; extended in L8-B)
- `crates/server/` (existing JSON-RPC surface; extended in L7-C)
- `crates/watch/` (existing session module; extended in L7-A)
- `docs/design/civ-culture-emergent.md` (existing culture emergent spec; the kinematic-emergent parallel for language)
- `docs/design/PHYSICS_COUPLING_SUBSTRATE.md` (the substrate that supplies `T`, `M`, `geom`)
- `docs/design/emergent-systems-spec.md` (the orchestrator this design slots into)
- `docs/design/emergence-dashboard.md` (the dashboard this design extends)
- `docs/research/language-optimality-audit.md` (unrelated; covers programming languages, not natural languages)

---

## 11. Acceptance criteria

This design is accepted when:

1. **L10-A** passes: Kirby iterated learning produces compositional lexicons.
2. **L10-B** passes: Grammar induction produces closed-class morphemes in the `[0.1, 0.3]` band.
3. **L10-C** passes: Writing systems vary by substrate (Logographic / Syllabic / Alphabetic produce distinct signatures).
4. **L10-D** passes: Physics coupling shifts phonology (climate differences are measurable).
5. **L10-E** passes: Social coupling shifts transmission (k_parents=1 vs k_parents=8 produce distinct phoneme distributions).
6. **L10-F** passes: Render-toggle does not mutate state (bit-identical `LanguageState` before and after).
7. **L10-G** passes: Criticality band is reachable and stable over 4096 ticks with default params.
8. **L10-H** passes: All knobs are real knobs (heat-death and explosion extremes reachable; default returns to band).
9. **L9-C** passes: Invariant test confirms no production code mutates `LanguageState.vocabulary` outside the four mutators.
10. **All existing tests still pass:** the `crates/engine/src/language.rs` test suite, the `crates/agents/src/culture.rs` test suite, the `emergence.rs:699-710` `culture_phase_drifts_cluster_profiles` integration test, and the 5-tile `EmergenceDashboard` `compute` overload.

When all 10 conditions hold, the design is implemented. When the design is implemented, the Civis agents speak — they speak to each other, they speak to the player, and the player can switch the channel from English to the agents' native tongue with a single click.




