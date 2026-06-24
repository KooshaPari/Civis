# Content Seeds — Canonical Starter Races & the Divergence Dial

> **Status:** Design (Planner) — rationale + schema + starter-pack inventory for the **2-layer content model**. **No implementation code** in this document; code lives in [`crates/genetics/src/seeds.rs`](../../crates/genetics/src/seeds.rs), [`crates/species/src/lib.rs`](../../crates/species/src/lib.rs), [`crates/engine/src/scenario.rs`](../../crates/engine/src/scenario.rs), and [`scenarios/canonical_seeds.ron`](../../scenarios/canonical_seeds.ron).
> **Governing canon:** [`emergent-systems-spec.md §1.2`](emergent-systems-spec.md) (two-layer content model), [`emergence-charter.md`](../guides/emergence-charter.md) (emergence-default principle), [`species-sentience.md`](species-sentience.md) (abiogenesis → sentience pipeline downstream of seeds).
> **FR families:** `FR-CIV-GENETICS-*`, `FR-CIV-SPECIES-*`, `FR-CONTENT-MODEL`, `FR-CONTENT-SEEDMIX`, `FR-CONTENT-STARTCOND`.
> **Owner crates:** `civ-genetics` (substrate + seed schema + helpers), `civ-species` (DNA → phenotype), `civ-planet` (biome affinity resolution), `civ-engine` (scenario loading, biome-to-seed selection, spawn helpers).

---

## 1. Why this document exists

The 2-layer content model (`emergent-systems-spec.md §1.2`) is the contract: **canonical named seeds + raw-organism primitive + `0..1` divergence dial** sit atop an **algorithmic DNA substrate**, and **emergence is the default everywhere**. Today that contract is **implemented in code** (`crates/genetics/src/seeds.rs`) but **not yet documented in one place** — the schema lives in `serde` derive structs, the divergence semantics in a module doc-comment, and the starter races in a single RON file with no rationale.

This doc consolidates the **content-model design** so that:

- A scenario designer can author new seeds without reading Rust.
- An engineer extending the engine knows exactly where the **canonical layer ends and emergence begins**.
- A reviewer can audit that no authored content **hardcodes outcomes** downstream of the spawn moment.

It is the **single source of truth for seed schema semantics**. If code and doc disagree, **the doc is the design intent** and the bug lives in the code.

---

## 2. The 2-layer content model, restated

```
┌──────────────────────────────────────────────────────────────────────────┐
│ LAYER 1 — CANONICAL SEEDS (authored content)                              │
│                                                                          │
│   • Named starter races (Ardani, Velthari, Grundak, human_baseline,      │
│     deep_one, + future). Hand-curated 64-byte genome, biome affinity,    │
│     divergence dial.                                                     │
│   • Raw-organism primitive (`raw_organism`): the substrate's             │
│     "no preference" baseline. Divergence 1.0 = full drift.               │
│   • `0..1` divergence dial per seed: how fast this preset erodes         │
│     into the substrate after spawn.                                      │
│                                                                          │
│   Purpose: presets over primitives. NOT rails. NOT outcomes.             │
└──────────────────────────────────────────────────────────────────┬───────┘
                                                                   │ spawn
                                                                   ▼
┌──────────────────────────────────────────────────────────────────────────┐
│ LAYER 0 — ALGORITHMIC SUBSTRATE (no authored content)                     │
│                                                                          │
│   • `Dna` byte-vector; `DnaClass { length, mutation_rate,                │
│     speciation_threshold }`                                              │
│   • `mutate` (per-byte point mutation, class-rate)                       │
│   • `recombine` (uniform crossover, deterministic)                       │
│   • `speciation_distance` / `should_speciate` (Hamming-distance)         │
│   • `fitness` (cosine similarity vs environment vector)                  │
│   • `express()` in `civ-species`: pure DNA → Phenotype mapping           │
│                                                                          │
│   Purpose: laws. Always running. Always drifting. Always speciationing.    │
└──────────────────────────────────────────────────────────────────────────┘
                                                                   │
                                                                   ▼
              emergence: culture, language, religion, architecture,
              economy, factions, diplomacy, legends — all measured
              patterns over the substrate, never outcome tables.
```

**Three guarantees this layering makes:**

1. **No canonical seed can hardcode a civilizational outcome.** The seed only nudges the initial genome distribution. From `spawn_genome` onward, the substrate is in charge.
2. **`raw_organism` is the universal escape hatch.** Any agent whose scenario doesn't reference a named seed (or whose biome has no match) falls back to fully random drift over the substrate.
3. **The divergence dial is a knob, not a switch.** It never *prevents* emergence; it only *paces* the rate at which the canonical preset is forgotten.

---

## 3. Seed schema (canonical)

The canonical seed schema is the Rust `SeedDefinition` in `crates/genetics/src/seeds.rs:54-72`. This section is the **design-side** view of the same shape; field semantics live here, not in the Rust doc-comments.

### 3.1 `SeedDefinition` fields

| Field | Type | Design role |
|-------|------|-------------|
| `id` | `String` | Stable identifier; matches the `id` referenced from scenario YAML and the JSON-RPC surface. Examples: `raw_organism`, `ardani`, `human_baseline`, `deep_one`. **Never rename once shipped** — saves reference ids by string. |
| `display_name` | `String` | Human-readable label for UI / debug / inspectors. Localisation-safe (English-default). |
| `dna_length` | `usize` | Length in bytes of the genome. Must equal `genome.len()` after load (validated; loader rejects mismatch). |
| `genome` | `Vec<u8>` | Base genome bytes. The **raw-organism primitive** uses `[0, 1, 2, …]` — a deliberately uniform baseline so `Dna::random` (used in the no-seed fallback) is statistically distinguishable from it. Named races use hand-curated patterns whose Hamming distance from `raw_organism` is large enough for `speciation_distance` to latch on. |
| `divergence` | `f32` | The **`0..1` dial** — see §4. **Finite-required, range-validated** at load. Non-finite, negative, or `> 1.0` values are rejected with `SeedError::InvalidDivergence`. |
| `spawn_biome_affinity` | `Vec<String>` | Soft biome affinity **labels** (free-form strings, NOT an enum reference — keeps `civ-genetics` decoupled from `civ-planet::BiomeKind`). The engine resolves these labels to concrete biomes via `BiomeKind::matches_affinity` (`crates/planet/src/geology.rs`). Empty = no preference → fallback to active seed at spawn. |
| `notes` | `Option<String>` | Free-form author commentary. Round-trips through RON (unicode-safe, escaping tested). |

### 3.2 `SeedSet` container

```
SeedSet {
    version: u32,           // schema version, currently 1
    seeds:  Vec<SeedDefinition>,
}
```

Loaded via `SeedLibrary::from_ron_str` (deserialise → validate each seed → reject duplicates). The version field is a forward-compatibility hook: future schema breaks must bump it and provide a migration path.

### 3.3 `SeedLibrary` (runtime)

```
SeedLibrary = HashMap<SeedId, SeedDefinition>
```

The library is the **only spawn-time entry point** for genome seeding. It supports:

- `from_ron_str(src)` — parse + validate + index
- `insert(seed)` — single-seed insert with duplicate rejection
- `get(id)` — lookup
- `iter()` — full enumeration (used by `select_seed_for_position` for biome matching)
- `retain(predicate)` — used by the engine to drop conflicting ids before re-registering a fresh set (replace semantics, not merge-on-conflict)

### 3.4 Design choices, defended

- **Why `Vec<u8>` and not a custom `Genome` newtype?** The substrate uses `Vec<u8>` (`Dna(Vec<u8>)`) so the seed shape matches byte-for-byte. No conversion layer = no off-by-one bugs.
- **Why labels, not enums, for biome affinity?** `civ-genetics` is a leaf crate — it must not depend on `civ-planet`. Labels travel as strings and are resolved at spawn time by the engine.
- **Why RON, not JSON?** RON is self-describing Rust, supports typed `Vec<Option<String>>` cleanly, and the `ron` crate is already a dependency. JSON would also work (the loader accepts any `serde::de::DeserializeOwned` shape), but RON is the convention.
- **Why a single `version` field, not a full SemVer on each seed?** The seed schema is intentionally minimal and additive (new optional fields are fine). Breaking changes happen at the `SeedSet` level; seeds themselves are mostly stable once shipped.

---

## 4. The `0..1` divergence dial

The divergence dial is the **single most important knob** in the content model. It controls how fast a canonical preset erodes into the substrate after spawn.

### 4.1 Semantics (per-tick)

Given a `DnaClass { mutation_rate: r }` and a seed with `divergence: d ∈ [0, 1]`:

```
effective_mutation_rate(class, d) = clamp(d, 0.0, 1.0) × class.mutation_rate
```

The function `mutate_with_divergence(dna, rng, class, d)` (in `seeds.rs:243-260`) is the canonical call site:

| `divergence` | Effective rate | Behaviour |
|--------------|----------------|-----------|
| `0.0` | `0.0` | Genome is **clamped** to the seed — `mutate` becomes a no-op. A `0.0` race is a "fixed-form" race (cosmetic only; sub-population variation must come from recombination, not point mutation). |
| `0.0 < d < 1.0` | `d × class.mutation_rate` | **Linear blend** between clamp and free drift. The race drifts at a fraction of the class rate. |
| `1.0` | `class.mutation_rate` | **Free drift** — the seed genome is a single starting position in the genome space; mutation proceeds at full class rate. |
| Non-finite | falls back to `class.mutation_rate` | `effective_mutation_rate` returns the class rate when `d` is NaN or ±∞ (defensive — the loader rejects non-finite values, but the math function is safe regardless). |
| Out of `[0, 1]` | **rejected at load** | `SeedError::InvalidDivergence`. Clamping happens in the math layer; the validator does not tolerate out-of-range. |

The function is a thin wrapper over the per-byte mutation loop (`seeds.rs:253-258`). Per-tick behaviour is **bit-deterministic** under a fixed RNG seed — this is a tested invariant (`tests/divergence_dial_intermediate_scales_rate` in `seeds.rs:666-712`).

### 4.2 Semantics (spawn-time)

The spawn-time entry point is `spawn_genome_with_divergence(rng, class, seed, divergence)`:

```
dna = seed.genome.clone()       // start from the canonical bytes
mutate_with_divergence(dna, rng, class, divergence)   // apply the dial
```

There is a **second** spawn helper, `seed_with_divergence(base, divergence, rng)` (`seeds.rs:412-433`), that operates differently: it linearly **lerps each byte** between the base value and a fresh random target. This is the **byte-interpolation spawn** used by the legacy `NamedSeed::Ardani/Velthari/Grundak` archetype path (see §5.1).

Both helpers are present and **deliberately distinct** because they answer different questions:

| Helper | Question | Use when |
|--------|----------|----------|
| `mutate_with_divergence` | "How fast does this genome drift from here on?" | Per-tick drift; spawn-time drift when the seed and per-tick logic should share a knob. |
| `seed_with_divergence` | "How far from the seed should this single individual start?" | One-shot population generation; visual spread on first frame; the archetype helper path. |

A scenario can use either or both. `divergence_override` in scenario YAML routes through `spawn_genome_with_divergence` (the engine spawn path).

### 4.3 Scenario-level override

```
Scenario {
    seeds:               Vec<String>,   // RON file paths to load
    active_seed:         Option<String>,// id of the fallback seed
    divergence_override: Option<f32>,   // 0..1 — overrides seed.divergence at spawn
    starting_conditions: {
        seed_mix:        Vec<SeedWeight>// weighted archetype distribution per spawn
    }
}
```

The override is validated by `Scenario::validate` (`scenario.rs:319-329`) to be in `[0, 1]` and finite. When `None`, the seed's own `divergence` field is used. The override exists because **a designer running a "high-drift" experiment should not have to fork the canonical seed pack**.

### 4.4 What the dial is NOT

- **It is not a speciation rate.** Speciation comes from `DnaClass.speciation_threshold` (a property of the substrate, not the seed). The divergence dial only paces how fast the genome moves; whether that movement crosses the speciation threshold is a substrate decision.
- **It is not a phenotype lock.** Even at `divergence = 0.0`, recombination across parents in the same named-race population will produce byte-level variation — visible in the phenotype. Pure-clamp means "no point mutation", not "all bodies identical".
- **It is not a culture lock.** Culture, language, religion, and architecture emerge from the substrate and the environment, not from the seed. A `0.0`-divergence race can still develop a thousand unique cultures.
- **It is not a per-spawn knob for designers in shipped content.** Scenario designers tune `divergence` once, when authoring. In-session tuning belongs to god-tools, not to scenario YAML.

---

## 5. Named starter races & organisms (canonical pack)

The canonical starter pack is the **default content shipped with the engine**. It is intentionally small — every named race should be a **distinct starting position** in genome space, not a re-skin of another.

### 5.1 Three archetype races (`crates/genetics/src/seeds.rs:311-322`)

These are the **code-level archetypes** — fixed-content constants with hand-curated 64-byte genomes. They are **not loaded from a RON file**; they live in the source so a `NamedSeed::Ardani` value is stable across engine versions regardless of what scenario RON is loaded.

| `NamedSeed` | Id | Biome affinity | `divergence` | Genome pattern | Design intent |
|-------------|-----|----------------|--------------|----------------|---------------|
| **`Ardani`** | `ardani` | `Desert`, `Savanna` | `0.15` | `i * 3 + 37` (wrapping u8) | Arid-world endurance caste. Low drift, structured social hierarchy encoded in leading bytes (high-intensity ramp), heat-tolerance signatures in the tail. Predisposes to **high-aggression, high-endurance** phenotypes. |
| **`Velthari`** | `velthari` | `DeepForest`, `Rainforest` | `0.35` | alternating `i*11+71` / `i*17-13` | Deep-forest symbiotes. Moderate drift, **high plasticity** (high genome variance). Empathic-resonance markers in upper bytes. Predisposes to **high-curiosity, high-sociability** phenotypes. |
| **`Grundak`** | `grundak` | `Cave`, `Underground` | `0.05` | `i * 2 + 128` (wrapping u8) | Subterranean lithomorphs. Very low drift, dense mid-range mineral-affinity encoding. Predisposes to **low-aggression, high-intelligence, low-sociability** phenotypes. |

The genome patterns are deliberately **distinguishable** (the `test_named_seeds_differ` invariant in `seeds.rs:776-792` verifies pairwise distinctness) and **distinguishable from `raw_organism`** (the `[0..63]` ramp). The `divergence` values reflect **lore intent**: Ardani are conservative (low drift → preserves caste identity), Velthari are adaptive (high drift → culture innovation), Grundak are stable (very low drift → slow lineage change).

**Spawn path:** the engine's `choose_named_seed` (`crates/engine/src/engine.rs`, covered by `choose_named_seed_empty_is_round_robin` and `choose_named_seed_weighted_distribution` tests) picks an archetype per spawn index. When `seed_mix` is empty, it round-robins `Ardani → Velthari → Grundak`. When `seed_mix` is non-empty, it samples via `WeightedIndex`. The picked archetype is then run through `seed_with_divergence` for byte interpolation (the **legacy archetype helper**), or via `spawn_genome_with_divergence` if `divergence_override` is set.

### 5.2 The raw-organism primitive (`crates/genetics/src/seeds.rs:439-455`)

```
id: "raw_organism"
dna_length: 64
genome: [0, 1, 2, 3, …, 63]
divergence: 1.0
spawn_biome_affinity: []
```

This is the **substrate's "no preference" baseline**. Its genome is the identity ramp `[0..63]` — the *most uniform, least informative* possible byte-vector. Its divergence is `1.0` — full free drift from spawn. Its biome affinity is empty — no preference, will always fall through to `active_seed` if biome matching fails.

**Design rationale:** keeping `raw_organism` distinct from `Dna::random` is essential for diagnostics. If `spawn_genome` ever produces a genome *identical* to `raw_organism.genome`, that means the no-seed path was taken *and* the RNG happened to land on the identity — vanishingly rare but testable. The contrast lets the emergence inspector show "raw substrate" vs "drifted from canonical seed" cleanly.

### 5.3 The canonical RON pack (`scenarios/canonical_seeds.ron`)

The file ships three named seeds, each loaded by `register_seed_file`:

| Id | `divergence` | Biome affinity | Genome pattern |
|----|--------------|----------------|----------------|
| `raw_organism` | `1.0` | `[]` | `[0..63]` |
| `human_baseline` | `0.1` | `["TemperateForest"]` | `i * 7 + 13` (wrapping u8) |
| `deep_one` | `0.4` | `["Ocean", "Tidepool"]` | `i * 31 + 5` (wrapping u8) |

`human_baseline` and `deep_one` are **RON-loaded seeds** — they live in `scenarios/canonical_seeds.ron` rather than in code because they are **scenario-level content** (a designer could swap them out per scenario). The three `NamedSeed` archetypes (§5.1) are **code-level content** (their genome bytes must be stable across engine versions for tests and historical replays).

The distinction is deliberate:

- **Archetypes** (`Ardani`, `Velthari`, `Grundak`) are the **substrate's vocabulary of races** — the enum is referenced by `seed_mix` and by god-tool presets. They cannot be removed without breaking compatibility.
- **RON seeds** (`raw_organism`, `human_baseline`, `deep_one`, future) are **scenario content** — they can be added, replaced, or omitted per scenario without breaking the substrate.

### 5.4 Currently designed seed count

| Source | Seeds | Count |
|--------|-------|-------|
| `crates/genetics/src/seeds.rs` archetype enum | Ardani, Velthari, Grundak | 3 |
| `crates/genetics/src/seeds.rs` raw-organism primitive | raw_organism | 1 |
| `scenarios/canonical_seeds.ron` | raw_organism, human_baseline, deep_one | 3 (one duplicate of the raw-organism primitive, intentional — the RON file is the scenario-pack view) |
| **Distinct seeds designed** | (raw_organism + 3 archetypes + human_baseline + deep_one) | **6** |

The duplicate `raw_organism` between code and RON is **intentional**: the RON pack is self-contained for scenarios that don't link to the genetics crate's compiled-in defaults; the code primitive ensures the engine *always* has a valid fallback even when no RON is loaded.

**Designed-seed total: 6** — `raw_organism`, `ardani`, `velthari`, `grundak`, `human_baseline`, `deep_one`.

---

## 6. How seeds feed emergence (and where they stop)

This section is the **anti-hardcoding audit trail**. For every place seeds touch the simulation, this section names what the seed controls and — critically — what it does **not** control.

### 6.1 The only places a seed influences runtime state

| Touchpoint | File / function | What the seed controls | What is **out** of scope |
|------------|----------------|------------------------|--------------------------|
| **Spawn-time genome** | `seeds.rs::spawn_genome_with_divergence` | The initial `Dna` byte-vector at agent birth | Phenotype interpretation (`express()` in `civ-species` is substrate-level, seed-agnostic) |
| **Per-tick drift** | `seeds.rs::mutate_with_divergence` | The rate at which a genome moves per tick | The direction (substrate + class mutation_rate govern; seed only scales the magnitude) |
| **Biome-to-seed resolution** | `crates/engine/src/emergence.rs::select_seed_for_position` | Which seed id a given spawn position uses (based on `BiomeKind::matches_affinity`) | The geography itself (geology map owns terrain) |
| **Faction archetype distribution** | `crates/engine/src/engine.rs::choose_named_seed` (via `ScenarioStartingConditions::seed_mix`) | Which `NamedSeed` archetype is sampled per spawn index | Faction creation, alignment, lore |
| **Scenario active fallback** | `crates/engine/src/scenario.rs::Scenario::active_seed` | Which seed id is used when no biome match | Whole-scenario policies, mods, taxation |

### 6.2 What is **never** controlled by a seed

The following systems consume the genome (and therefore *inherit* the canonical-seed nudges) but **do not consult any seed directly**. They are pure substrate + emergence layers, and their outputs are **measured patterns over the genome distribution**, never scripted outcomes:

| System | Crate | Why it's seed-independent |
|--------|-------|---------------------------|
| Phenotype expression (morphology + behaviour weights) | `civ-species` | Pure `Dna → Phenotype` mapping (9-byte layout). The seed nudges the input; the function is seed-agnostic. |
| Speciation (`should_speciate`, `Species` records) | `civ-genetics` | Class-threshold-based; seeds only affect the genome bytes, not the threshold logic. |
| Culture drift, language change | `civ-agents::culture` | Operates on `TraitVector` (culture-internal), not on DNA. The connection to DNA is one-way and indirect (via `psyche`). |
| Psyche (`psyche_from_dna`, `nudge_temperament`) | `civ-agents::psyche` | Reads DNA bytes, but the mapping is in `civ-agents`, not in `civ-genetics`. The seed does not pre-bake a psyche. |
| Cognition / sentience threshold | `civ-genetics::sentience` | Trait-accumulation logic, not a seed field. |
| Architecture | `crates/build` | DNA → ward-drobe / tool era is one input among many; form grammar is in the build crate. |
| Economy / market / institution | `crates/economy` | No seed-aware code paths; population is just `u64`. |
| Religion, legends, factions, diplomacy | `crates/engine/src/{legends,faction_emergence,…}` | Emerge from agent state and history; no seed-driven scripting. |
| Voxel CA physics, climate, geology | `crates/voxel`, `crates/planet`, `crates/climate` | Hardcoded physics laws; not even aware of seeds. |

**The audit question to ask of any new feature:** *Does this feature read `SeedDefinition` directly, or only the genome?* If it reads the seed, it must justify that read against this list. If it only reads the genome, it is in the clear.

### 6.3 The guard rails (testable invariants)

The current code enforces several invariants that protect emergence from being hardcoded by seeds:

| Invariant | Source | What it guarantees |
|-----------|--------|--------------------|
| `raw_organism_primitive_is_valid` | `seeds.rs:512-519` | The raw-organism seed round-trips and validates; the engine always has a fallback. |
| `divergence_dial_zero_means_no_drift_over_generations` | `seeds.rs:626-640` | A `0.0`-divergence seed is truly clamped (10 000 ticks of mutation yield zero byte changes). |
| `divergence_dial_one_means_free_drift` | `seeds.rs:642-663` | A `1.0`-divergence seed drifts at the full class rate. |
| `divergence_dial_intermediate_scales_rate` | `seeds.rs:665-712` | Mid-range divergence produces strictly fewer byte-flips than full divergence over the same number of ticks (rate-monotonicity). |
| `test_named_seeds_differ` | `seeds.rs:776-792` | The three archetype races have pairwise-distinct genomes — none is a re-skin. |
| `choose_named_seed_empty_is_round_robin` | `engine.rs:5406-5423` | With no `seed_mix`, the default is bit-identical across runs. |
| `choose_named_seed_weighted_distribution` | `engine.rs:5426-5463` | A `seed_mix` produces a distribution whose fractions are within tolerance of the weights. |
| `register_seed_set_merges_and_replaces_ids` | `emergence.rs:840-870` | Re-registering seeds does not leave stale ids. |
| `set_active_seed_updates_or_rejects_unknown` | `emergence.rs:872-891` | An unknown seed id is rejected, never silently swallowed. |
| `register_seed_file_loads_fixture_and_reports_missing` | `emergence.rs:893-914` | Missing RON files emit `seed_load_failed` feed events instead of panicking. |
| `scenario_divergence_override_parses_and_validates` | `scenario.rs:715-768` | `divergence_override` is range-checked at parse time. |
| `scenario_seed_mix_*` | `scenario.rs:915-1020` | Weight values must be finite and `> 0`. |

**A future reviewer adding a new seed-affecting code path must add a test from this invariant list** — they are the **anti-regression tests for emergence** in the seed subsystem.

---

## 7. Authoring guide (for scenario designers)

### 7.1 Adding a new seed to the canonical RON pack

1. Open `scenarios/canonical_seeds.ron`.
2. Append a new `SeedDefinition(...)` entry. Required fields:
   - `id` — lowercase snake_case, never reused
   - `display_name` — English label for UI
   - `dna_length` — must match the byte count of `genome`
   - `genome` — 64 bytes recommended (matches the default `DnaClass.length`)
   - `divergence` — start at `0.1` if unsure (low drift = conservative)
   - `spawn_biome_affinity` — list of biome label strings (see `crates/planet/src/geology.rs` for canonical labels)
3. Optional: `notes` — author commentary
4. Run `just civis-3d-scenario-check` to validate the file
5. Run `just civis-3d-verify` to confirm the smoke tests still pass

### 7.2 Adding a new archetype race

Archetypes are **code-level constants** — adding one is a Rust change, not a content change:

1. Add a variant to `pub enum NamedSeed` in `crates/genetics/src/seeds.rs:311`
2. Add a match arm to `archetype_dna` (`seeds.rs:331`) — provide a 64-byte pattern
3. Add a match arm to `archetype_seed` (`seeds.rs:365`) — provide id, name, biomes, divergence, notes
4. Add a `test_*_validates` entry in the test module (`seeds.rs:796-802`)
5. **Do not** remove or reorder existing variants — saves reference them by ordinal

### 7.3 Tuning the divergence dial

| Use case | Recommended divergence | Why |
|----------|------------------------|-----|
| "Fixed-form" species (all bodies look alike) | `0.0` | Pure clamp; no point mutation. Recombination still varies offspring. |
| "Caste-locked" species (slow cultural drift) | `0.05` – `0.15` | Slow genome drift → slower speciation → longer-lived races. (Ardani = `0.15`; Grundak = `0.05`.) |
| "Adaptive" species (fast innovation, fast speciation) | `0.30` – `0.50` | Genome moves at 30–50% of class rate; speciation events within a few hundred ticks. (Velthari = `0.35`.) |
| "Loose starting point, full emergence" | `0.7` – `1.0` | Seed is a one-shot nudge; substrate is in charge from spawn onward. (`raw_organism` = `1.0`.) |
| God-tool / debug experiments | `divergence_override` per scenario | Don't fork the canonical pack — use the override. |

### 7.4 Anti-patterns

- **Don't set `divergence = 0.0` and call it a "race".** A clamped race is a *form*, not a people. Add at least one bi-affinity label and a divergence `> 0.0` if you want cultural emergence.
- **Don't pack every biome affinity into one seed.** A seed with all biomes is just `raw_organism` with extra steps.
- **Don't author the genome to encode culture directly.** Bytes 0–4 are morphology, bytes 5–8 are behaviour weights (see `species-sentience.md §1.2` for the full layout). Other bytes are substrate-internal — assume they're under spec authority of the substrate crate, not content.
- **Don't bypass the dial.** "I want full drift but no speciation" is not a seed concern — it's a `DnaClass` concern. Tune the substrate, not the seed.

---

## 8. Open questions / future work

These are tracked here so the doc doesn't pretend the model is closed:

| Question | Where it lands |
|----------|----------------|
| Should archetype races (Ardani/Velthari/Grundak) graduate from code-level to RON-level once we have a content-update story? | TBD — current decision is "no, code-level" for stability. Re-evaluate after CIV-0700 mod-store ships. |
| Can a seed express *soft* constraints (e.g. "prefer legs ≥ 2") without hardcoding? | Open. Current option is to encode the constraint in the genome bytes and let `express()` project it; this works for morphology but not for emergent behaviour weights. |
| Do we need a `traits` field on `SeedDefinition` to label race-archetype intent (e.g. `["caste", "predator", "symbiote"]`)? | Open. Would let the emergence inspector summarise without genome-byte archaeology. |
| Should `DnaClass` allow `mutation_rate` overrides per-seed? | **No.** That re-introduces the hardcoding the substrate is designed to prevent. The divergence dial already provides per-seed pacing; layering a per-seed rate on top would make the substrate non-uniform. |

---

## 9. References

- [`crates/genetics/src/seeds.rs`](../../crates/genetics/src/seeds.rs) — code substrate
- [`crates/genetics/src/lib.rs`](../../crates/genetics/src/lib.rs) — `Dna`, `DnaClass`, mutation/recombination/speciation
- [`crates/species/src/lib.rs`](../../crates/species/src/lib.rs) — `express()` (DNA → Phenotype)
- [`crates/engine/src/scenario.rs`](../../crates/engine/src/scenario.rs) — scenario YAML, `SeedWeight`, `divergence_override`, `seed_mix`
- [`crates/engine/src/emergence.rs`](../../crates/engine/src/emergence.rs) — `select_seed_for_position`, `Simulation::seed_library`, `register_seed_*`, `set_active_seed`
- [`crates/engine/src/engine.rs`](../../crates/engine/src/engine.rs) — `choose_named_seed` (weighted / round-robin helper)
- [`crates/planet/src/geology.rs`](../../crates/planet/src/geology.rs) — `BiomeKind::matches_affinity`
- [`scenarios/canonical_seeds.ron`](../../scenarios/canonical_seeds.ron) — canonical RON content pack
- [`scenarios/baseline.yaml`](../../scenarios/baseline.yaml), [`scenarios/presets/*.yaml`](../../scenarios/presets) — scenario presets
- [`docs/design/emergent-systems-spec.md §1.2`](emergent-systems-spec.md) — two-layer content model (the charter this doc implements)
- [`docs/design/species-sentience.md`](species-sentience.md) — abiogenesis → sentience pipeline (downstream of seeds)
- [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — emergence-default principle
- [`docs/traceability/fr-3d-matrix.md`](../traceability/fr-3d-matrix.md) — FR-CIV-GENETICS-* and FR-CIV-SPECIES-* coverage
