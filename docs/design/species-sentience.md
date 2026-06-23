# Species & Sentience Emergence Spec

**Status:** Design (Planner) — no code herein, only requirements, models, and acceptance criteria.
**Owner crates:** `crates/laws`, `crates/planet`, `crates/genetics`, `crates/species`, plus consumers `civ-agents` (behavior), the legends/inspector UI.
**Charter binding:** [emergence-charter.md](../guides/emergence-charter.md). Hardcode **only** physical/environmental/genomic laws. Life, the path to sentience, culture, language, and civilization **emerge**; the sentient form **need not be humanoid**.
**Requirement family:** `FR-CIV-SPECIES-*`, extending `FR-CIV-GENETICS-*`. New IDs allocated in non-colliding bands (existing code/tests occupy `…-000`..`…-011`):

| Band | Topic |
|------|-------|
| `FR-CIV-SPECIES-1xx` | Abiogenesis (life from material + energy) |
| `FR-CIV-SPECIES-2xx` | DNA → phenotype expression (morphology + behavior + body/mind plan) |
| `FR-CIV-SPECIES-3xx` | Speciation (Hamming-distance lineage divergence) |
| `FR-CIV-SPECIES-4xx` | Cognition accumulation & the sentience threshold + unlock ladder |
| `FR-CIV-SPECIES-5xx` | Inspector surfacing (genome / species / cognition views) |

This spec is the bridge from the existing primitives (`Dna`, `DnaClass`, `express → Phenotype`, `speciation_distance`/`should_speciate`, `CognitionTraitProfile`/`SentienceThreshold`/`evaluate_sentience`) to a full **abiogenesis → species → sentience** emergence pipeline. It specifies *rules*, never outcomes.

> **Determinism note.** Per the charter correction (2026-05-29), bit-identical replay is **not** required. The existing crate doc-comments still cite ADR-008 "pure deterministic algorithm / `ChaCha8Rng` replay"; treat that as *legacy wording*. Real randomness/floats are welcome wherever they enrich emergence. The only invariant this spec keeps is **referential transparency of `express`** (same `Dna` ⇒ same `Phenotype`) because it is a pure mapping, not a simulation step — that is a convenience, not a determinism mandate.

---

## 0. Pipeline at a glance

```
material voxels + energy gradient   (Layer-0 laws: crates/laws, crates/planet)
          │   abiogenesis rule (§1) — emergence, not spawn-table
          ▼
   Protocell  ─────────►  Dna (seed genome, crates/genetics)
          │   express() (§2) — pure DNA→Phenotype
          ▼
   Phenotype { Morphology (body plan), BehaviorWeights (mind plan) }
          │   reproduction + mutate/recombine, generations pass
          ▼
   Lineage drift ──► Hamming distance (§3) ──► Species record on threshold cross
          │   cognition traits accumulate in the genome
          ▼
   cognition_score (§4) crosses SentienceThreshold ──► Sentient lineage
          │   unlock ladder: tool-use → culture → language → civilization
          ▼
   Inspector (§5) shows genome bytes, species lineage, cognition gauge, unlocks
```

Each arrow is a **law**; each box is **emergent state**. No box is an authored enum.

---

## 1. Abiogenesis — life from material + energy

**Goal:** life is not seeded from a creature table. It **emerges** where Layer-0 conditions (material composition + energy gradient + solvent + time) cross an *abiogenesis suitability* threshold, exactly as physics/chemistry permit. The first replicator is a `Dna` byte-vector attached to a voxel-local **protocell**.

### 1.1 The abiogenesis suitability field

Define a scalar **abiogenesis suitability** `A ∈ [0,1]` computed per candidate voxel cell from quantities the substrate already simulates (`crates/laws` materials/energy, `crates/planet` climate/hydrology). It is a product of necessity factors (any factor at 0 ⇒ `A=0`, modeling hard chemical gates):

| Factor | Source law | Rationale |
|--------|-----------|-----------|
| **Solvent presence** | fluid CA: liquid (e.g. water) volume fraction in/adjacent to cell | Reactions need a medium; powders/gases alone gate to 0. |
| **Building-block availability** | materials DB: concentration of "organic-capable" element tags (the charter forbids a `life: bool` flag — instead tag *materials* with reactivity properties) | Replicators need feedstock. |
| **Energy gradient** | thermal/chemical-potential gradient across the cell (NOT just heat — a *gradient*, i.e. usable free energy) | Life is a dissipative structure; it needs a flux to feed on. |
| **Goldilocks band** | temperature & pressure within the material's liquid-solvent window (from phase rules) | Too hot/cold/crushing ⇒ no stable chemistry. |
| **Stability dwell** | how long the above co-occurred (accumulated, decays when conditions break) | Abiogenesis is rare and slow; rewards persistence, not instant flips. |

`A` is the (weighted geometric) combination of these, normalized to `[0,1]`. **Weights and bands are data** in a `laws`/`planet` config table, mod-friendly — never hardcoded constants in logic.

### 1.2 The emergence event

- Per simulation tick (LOD-tiered — full near camera, statistical far away), each active cell accrues a tiny **abiogenesis probability** monotonically increasing in `A` and in dwell time. Real randomness (`thread_rng`/floats) is acceptable here — variety is the point.
- On firing, a **protocell** is instantiated at that voxel with a **seed genome**: a fresh `Dna` whose length is its `DnaClass.length` and whose bytes are biased by *local conditions* (not pure noise) — e.g. the dominant-energy axis nudges metabolism-related byte ranges, ambient hue nudges `body_color_hue`. This makes first life *fit its cradle* without scripting the creature.
- The protocell's `DnaClass` is chosen by which **substrate archetype** the cradle matches (aqueous-carbonal, mineral/silicate-thermal, cryo-solvent, …). Archetypes are **data rows** describing genome length, mutation rate, and speciation threshold — they are NOT species; they are the chemistry family the genome operates within.

### 1.3 Requirements

- **FR-CIV-SPECIES-100** — There is **no creature spawn table**. The only authored inputs to first life are (a) material/energy/phase **laws** and (b) the abiogenesis suitability **factor config**. Removing all `DnaClass`/archetype rows still yields a valid (lifeless) world; adding rows is pure data.
- **FR-CIV-SPECIES-101** — `A` is `0` whenever any necessity factor is `0` (no solvent ⇒ no life; no energy gradient ⇒ no life), and strictly increases as factors and dwell improve, holding others fixed.
- **FR-CIV-SPECIES-102** — Abiogenesis fires probabilistically; over many ticks in a suitable cradle at least one protocell emerges, and in an unsuitable cell (`A=0`) none ever do.
- **FR-CIV-SPECIES-103** — A newborn protocell's seed genome is **biased by local conditions** (measurably correlated with the cradle's energy axis / ambient properties), not drawn from a uniform distribution.
- **FR-CIV-SPECIES-104** — The chosen `DnaClass`/substrate archetype is a function of the cradle's chemistry only; the same chemistry deterministically selects the same archetype, while different chemistries may select different ones (enables non-carbon, non-humanoid lineages).
- **FR-CIV-SPECIES-105** — Abiogenesis is **rare and dwell-gated**: a transient suitable flicker (below the dwell requirement) does not produce life; sustained suitability does.

---

## 2. DNA → phenotype expression (body + mind plan)

The pure mapping already exists (`species::express`). This section **extends the byte layout** so a genome encodes a *divergent body and mind plan*, not a parameterized humanoid. The existing 9-byte layout (`height_cm`, `body_color_hue`, `leg_count`, `arm_count`, `eye_count`, `aggression`, `curiosity`, `sociability`, `intelligence`) is the **legacy core**; this spec reserves further byte bands so non-humanoid plans are first-class.

### 2.1 Genome regions (data-driven, per `DnaClass`)

Rather than a fixed offset table baked into one `express`, the body/mind plan is a set of **named gene regions** declared by the `DnaClass` (byte range → trait). This makes a 4-eyed, 6-legged, radially-symmetric photosynthetic sessile organism as expressible as a biped — purely by which regions the class declares and how the genome fills them.

Recommended region taxonomy (all **data**, extensible by mods):

- **Morphology / body plan:** size, segment count, symmetry (bilateral / radial / asymmetric — *emergent from a symmetry gene*, not an enum the designer picks), limb count & kind (leg/arm/wing/fin/none), sensory organ count & kind (eye/antenna/chemoreceptor), integument (skin/scale/chitin/bark), color.
- **Metabolism:** energy source weighting (photo- / chemo- / hetero-trophic), solvent dependence, thermal tolerance band — these **must echo the abiogenesis cradle** so a lineage stays consistent with its chemistry until mutation/selection moves it.
- **Behavior / mind plan (consumed by `civ-agents`):** the legacy `aggression / curiosity / sociability / intelligence`, plus reserved cognition-substrate bytes (§4): memory capacity, signal/communication propensity, tool-affinity, social-coordination, abstraction. These are the **raw material** of the sentience threshold.

### 2.2 Requirements

- **FR-CIV-SPECIES-200** — `express` is a **total, pure** function of `(Dna, DnaClass)`: same inputs ⇒ same `Phenotype`; short/empty genomes zero-fill without panic. (Preserves existing `…-001`/`…-002`/`…-009`.)
- **FR-CIV-SPECIES-201** — The byte→trait layout is **declared by `DnaClass`** (named gene regions), not a single hardcoded offset constant. Adding a region is data; existing genomes remain expressible (unfilled regions zero-fill).
- **FR-CIV-SPECIES-202** — A genome can express a **non-humanoid plan** (e.g. `leg_count ≠ 2`, `arm_count = 0`, radial symmetry, sessile) with no special-casing — the renderer/agent layer consumes whatever the regions say.
- **FR-CIV-SPECIES-203** — Metabolism traits expressed at abiogenesis are **consistent with the cradle archetype** (FR-CIV-SPECIES-104); selection may later shift them, but a fresh lineage is not random with respect to its chemistry.
- **FR-CIV-SPECIES-204** — A single-byte mutation affects **only its region's** trait(s), leaving all others equal (extends existing `…-008`).
- **FR-CIV-SPECIES-205** — Behavior/cognition bytes feed both `civ-agents` (drives) and the cognition score (§4) from the **same genome bytes** — no parallel hidden stat.

---

## 3. Speciation — Hamming-distance lineage divergence

The primitive exists (`speciation_distance`, `should_speciate`, `Species{ id, dna_class, founder_centroid }`). This section specifies the **lineage bookkeeping** that turns drift into named species and a genealogy the inspector and legends engine can read.

### 3.1 Model

- A **lineage** is a chain of reproduction events. Each individual carries its `Dna`, its `DnaClass`, and a `species_id` inherited from its parent(s).
- On each birth, compare the child's `Dna` against its **species' reference centroid** (the founder centroid, or a maintained running centroid for the species) via `speciation_distance`. If it exceeds `DnaClass.speciation_threshold` (`should_speciate`), a **new `Species` is issued**: stable `id`, the child's genome as `founder_centroid`, and a recorded **parent species** link (genealogy edge).
- Cross-class comparison is **undefined** (`speciation_distance` panics on length mismatch by contract). Speciation only ever compares within a `DnaClass`; different archetypes are already different species trees by construction.
- Speciation is **reproductive-isolation-flavored but not gameplay-blocking**: distance only *records* divergence; whether two close lineages still interbreed is a separate emergent matter (contact + behavior), out of scope here but the genealogy must support it.

### 3.2 Requirements

- **FR-CIV-SPECIES-300** — A new `Species` is issued **iff** a child's Hamming distance to its species reference exceeds the class `speciation_threshold` (extends existing `…-010`); below threshold, the child stays in the parent species.
- **FR-CIV-SPECIES-301** — `speciation_distance` is symmetric and normalized to `[0,1]` (preserves existing `…-011`).
- **FR-CIV-SPECIES-302** — Each `Species` record links to its **parent species** (genealogy is a DAG/tree), enabling the inspector and legends engine to render an evolutionary tree. Founder species have no parent.
- **FR-CIV-SPECIES-303** — Speciation never compares across `DnaClass` boundaries; archetypes form disjoint species forests.
- **FR-CIV-SPECIES-304** — Species issuance is **stable & idempotent per birth event**: re-evaluating the same birth does not mint duplicate species; IDs are monotonic and never reused.

---

## 4. Cognition accumulation & the sentience threshold

This is the heart of the spec: **how a lineage may cross from non-sentient to sentient, what it takes, and what crossing unlocks.** The primitive exists (`CognitionTraitProfile`, `cognition_score`, `SentienceThreshold`, `evaluate_sentience`, `SentienceEvent`). Sentience is a **threshold a lineage may cross**, never a given — and the sentient form need not be human.

### 4.1 What accumulates (the cognition substrate)

Cognition is a **weighted blend of specific genome bytes** (`CognitionTraitProfile.trait_weights`), drawn from the mind-plan region (§2.1). The traits that matter — each a genome byte slot, accumulating via mutation + selection across generations:

| Cognitive trait | What it represents | Drives which unlock |
|-----------------|--------------------|--------------------|
| **Memory capacity** | retain/recall past states beyond the immediate | prerequisite for learning |
| **Tool-affinity** | manipulate environment objects as means to ends | tool-use rung |
| **Social-coordination** | act jointly with conspecifics | culture rung |
| **Signal/communication** | emit + interpret learned signals | language rung |
| **Abstraction** | represent things not present (symbols, plans, causes) | civilization rung |

`cognition_score(dna, profile) ∈ [0,1]` is the normalized weighted average of these byte slots. **Profiles are per-lineage/per-class data**, so different body plans can reach sentience by *different cognitive routes* (a social-signaling swarm mind vs. a solitary tool-using manipulator) — there is no single "human" path.

### 4.2 The threshold and the unlock ladder

A lineage is **sentient** when `cognition_score ≥ SentienceThreshold.minimum_cognition` (`evaluate_sentience` ⇒ `SentienceEvent.crossed`). But sentience is the *top* of a **gated ladder**: each rung is its own sub-threshold over the relevant trait(s), and a rung unlocks an emergent capability that becomes available to the agent/culture layers. Crossing is a **lineage-level latch** — once a lineage crosses a rung it is recorded as a transition (for legends/feed), and individuals of that lineage may exhibit the unlocked capability subject to their own genome and context.

**The ladder (each rung gated on accumulated traits; lower rungs are prerequisites):**

1. **Learning** — gated on *memory capacity*. Unlocks: individual learning, habit/memory in `civ-agents` (behavior shaped by experience, not only genome).
2. **Tool-use** — gated on *tool-affinity* + learning. Unlocks: agents treat environment objects/materials as instruments (the charter's emergent tools/machines path begins here).
3. **Culture** — gated on *social-coordination* + tool-use. Unlocks: socially transmitted behavior — norms/techniques that diffuse over the kinship/contact network (feeds the charter's culture layer), heritable *non-genetically*.
4. **Language** — gated on *signal/communication* + culture. Unlocks: learned shared signals that drift into dialects/creoles on contact (feeds the charter's language layer). This is the conventional "sentience" mark, but it is a **rung, not a magic gate**.
5. **Civilization** — gated on *abstraction* + language. Unlocks: symbolic representation enabling polities, markets, architecture/engineering, legends — i.e. the higher emergent layers the charter lists. A lineage at this rung is the substrate from which states/economies/cities can self-organize.

Each rung's gate is **data** (a threshold + which traits it reads). The *full* `SentienceThreshold` corresponds to the language/civilization rungs; lower rungs use lower sub-thresholds over narrower trait subsets. This keeps the charter's promise: tool-use → culture → language → civilization is an **emergent progression**, each step earned by accumulated genomic cognition, never handed out.

### 4.3 Divergent minds (non-humanoid sentience)

- Because rungs read *traits*, not a species name, a **radial sessile organism**, a **distributed swarm**, or a **solitary manipulator** can each reach the same rung through whichever traits its lineage accumulated. There is no humanoid prerequisite — `leg_count`/`arm_count` are irrelevant to the cognition score.
- A lineage may cross some rungs and never others (e.g. high tool-affinity + low communication ⇒ tool-using but never linguistic), producing genuinely alien civilizations or perpetual "almost" lineages.

### 4.4 Requirements

- **FR-CIV-SPECIES-400** — `cognition_score` reads **only** declared cognition byte slots and is normalized to `[0,1]` for non-negative weights (preserves existing sentience-module tests).
- **FR-CIV-SPECIES-401** — Sentience is a **threshold latch**: a lineage is sentient iff its representative cognition score ≥ `minimum_cognition`; below it the lineage remains non-sentient, and the latch is recorded as a one-time transition event (`SentienceEvent`-flavored) for the legends/feed.
- **FR-CIV-SPECIES-402** — Cognition **accumulates via the genome**: the score can only rise across generations by mutation/recombination/selection on the cognition byte slots, never by direct assignment — there is no `is_sentient` flag set by fiat.
- **FR-CIV-SPECIES-403** — The unlock ladder is **strictly ordered and gated**: a lineage cannot register *culture* without *tool-use*, nor *language* without *culture*, nor *civilization* without *language*. Each rung gate is data (threshold + trait subset).
- **FR-CIV-SPECIES-404** — Each rung, when crossed, **unlocks an emergent capability** consumed by downstream layers (learning→agents, tool-use→tools, culture→cultural transmission, language→language drift, civilization→polity/market/architecture). No rung directly scripts the outcome; it gates availability.
- **FR-CIV-SPECIES-405** — Rungs are reachable by **multiple cognitive routes / body plans**: two lineages with disjoint dominant traits but sufficient blended cognition both qualify; humanoid morphology is never a prerequisite.
- **FR-CIV-SPECIES-406** — A lineage may cross lower rungs and **stall** below higher ones indefinitely (partial cognition), and may regress if selection erodes the relevant traits (the latch may be authored as either sticky or reversible — this is a tuning decision, but regression must be *representable*).

---

## 5. Inspector surfacing

The player must be able to select any creature and **see what it is and how it came to be**. The inspector reads existing/derived state only — it is a view, not a new authority.

### 5.1 Views

- **Genome view** — raw `Dna` bytes, grouped by the `DnaClass`'s named gene regions (§2.1) with human-readable labels; highlights which bytes are cognition slots.
- **Species view** — the creature's `Species` (id + name), its position in the **species genealogy tree** (§3.2: parents/siblings/descendants), Hamming distance to sibling species, and its `DnaClass`/substrate archetype (its chemistry origin).
- **Phenotype view** — the expressed `Morphology` (body plan: limbs, eyes, symmetry, size, color, integument) and `BehaviorWeights` (mind plan), i.e. the output of `express`.
- **Cognition view** — the `cognition_score` as a gauge against `SentienceThreshold`, **which rungs the lineage has crossed** (learning / tool-use / culture / language / civilization) shown as a lit ladder, and the contributing cognitive traits with their per-trait values.
- **Origin/lineage view** — abiogenesis cradle archetype, founder centroid, and a link into the **legends engine** timeline (first life, each speciation, each rung crossing as historical events).

### 5.2 Requirements

- **FR-CIV-SPECIES-500** — Selecting a creature shows its raw genome **labeled by gene region** (not an opaque byte blob), including which bytes drive cognition.
- **FR-CIV-SPECIES-501** — The inspector shows the creature's **species and its genealogy** (parent/child species via FR-CIV-SPECIES-302), and its substrate archetype/chemistry origin.
- **FR-CIV-SPECIES-502** — The inspector shows the **expressed phenotype** (body plan + mind plan) consistent with `express` for that genome.
- **FR-CIV-SPECIES-503** — The inspector shows a **cognition gauge** vs. the sentience threshold and the **lit/unlit unlock ladder** (which rungs the lineage has crossed), with per-trait cognition contributions.
- **FR-CIV-SPECIES-504** — Speciation events and rung crossings for a creature's lineage are **queryable as historical events** (legends-engine integration), so "how this mind came to be" is reconstructable.
- **FR-CIV-SPECIES-505** — All inspector views are **read-only projections** of genome/species/cognition state; the inspector introduces no authored creature data and no hidden stats not present in the genome.

---

## 6. Cross-crate boundaries (where each rule lives)

| Concern | Crate / layer | Notes |
|---------|--------------|-------|
| Material/energy/phase laws, abiogenesis suitability factors | `crates/laws`, `crates/planet` | Layer-0; abiogenesis `A` config is data here. |
| `Dna`, `DnaClass`/archetypes, mutate/recombine, speciation distance, cognition score, sentience threshold | `crates/genetics` | Primitives already present; extend with gene-region declarations + rung gates as **data**. |
| `express` (DNA→Phenotype), gene-region taxonomy, body/mind plan | `crates/species` | Extend layout to be `DnaClass`-declared (FR-CIV-SPECIES-201). |
| Abiogenesis emergence tick, lineage bookkeeping, rung latching | simulation/agents layer (`civ-agents` + a worldgen/abiogenesis system) | Consumes the primitives; emits `Species`/`SentienceEvent`/rung transitions. |
| Behavior, learning, cultural transmission, language drift | `civ-agents` + culture/language layers | Downstream consumers of rung unlocks (out of scope to implement here). |
| Inspector + legends timeline | UI / legends-engine | Read-only projections (§5). |

---

## 7. Open design questions (for follow-on planners, not blockers)

1. **Centroid maintenance** — does a species track a running centroid or stay pinned to the founder? (Affects speciation cadence; running centroid resists runaway splitting.)
2. **Rung latch stickiness** — are crossed rungs permanent for a lineage, or can sustained trait erosion revoke them (collapse)? FR-CIV-SPECIES-406 requires regression be *representable*; default policy TBD.
3. **Interbreeding vs. speciation** — speciation records divergence; the *reproductive compatibility* curve (can two near species still mate?) is a separate emergent rule to be specified with the agents/contact layer.
4. **Abiogenesis density caps** — LOD/statistical handling far from camera so suitable-but-unobserved regions still birth life plausibly without per-voxel full sim cost.

---

## 8. Acceptance summary

A correct implementation satisfies: life appears only where Layer-0 conditions permit (no spawn table); genomes express divergent, possibly non-humanoid body/mind plans purely from data; lineages split into a genealogy via Hamming distance; cognition accumulates *in the genome* and a lineage may cross a gated ladder (learning→tool-use→culture→language→civilization), each rung unlocking an emergent downstream capability and never humanoid-gated; and the inspector lets a player read any creature's genome, species lineage, phenotype, and cognition/unlock state as pure projections.
