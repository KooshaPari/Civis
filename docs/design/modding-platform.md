# Civis Modding Platform — Design Spec (civlab-sdk Expansion)

> **Status:** Design spec (docs-only, 2026-05-30). Owner: Design R&D Lead.
> **Stance:** PLANNER — this document is specs, architecture, acceptance criteria, schemas, and brief
> pseudocode only. It contains **no implementation code**; it equips engineer/codex agents to build.
> **Governing constraint:** [`docs/guides/emergence-charter.md`](../guides/emergence-charter.md) — only
> physical/environmental/genomic **laws** are authored; everything else EMERGES. **Mods extend the
> SUBSTRATE (laws / materials / genome primitives / biome rules / content grammars), never the
> outcomes.** A mod adds new *rules and primitives*; life, society, economy, polity, language,
> architecture still emerge from those rules. A mod that tries to hardcode an outcome
> (a scripted faction, a fixed market, a pre-built city) is **charter-violating and rejected at load**.
> **Why this matters:** per [`docs/research/competitive-benchmark.md`](../research/competitive-benchmark.md),
> a deep, safe, hot-reloadable mod platform that lets the community extend the *laws of the world* (not
> just reskin it) is a **primary differentiator** vs WorldBox/CS2/Civ-style mod ecosystems that only
> expose surface content.
> **Grounded in existing code:** expands `crates/civlab-sdk` (manifest.rs, material.rs, building.rs,
> events.rs, registry.rs) and `crates/laws` (RON law DB + validator). Do NOT duplicate those; this spec
> *extends* their trait/registry surface.

---

## 0. Requirements index (FR-CIV-MOD-*)

| FR | Title | §  |
|----|-------|----|
| FR-CIV-MOD-000 | Mod manifest schema (RON primary / JSON parallel), versioned | §2 |
| FR-CIV-MOD-001 | Moddable surface taxonomy — what a mod may and may not touch | §1 |
| FR-CIV-MOD-002 | Material + reaction registration (extends `material.rs` + `laws`) | §3.1 |
| FR-CIV-MOD-003 | Building / recipe / structure **grammar** registration | §3.2 |
| FR-CIV-MOD-004 | Law / physics-constant extension (extends `crates/laws`) | §3.3 |
| FR-CIV-MOD-005 | Species / genome primitive registration | §3.4 |
| FR-CIV-MOD-006 | Biome / climate rule registration | §3.5 |
| FR-CIV-MOD-007 | Event hooks (read-only observers + bounded reactors) | §3.6 |
| FR-CIV-MOD-008 | UI / overlay registration | §3.7 |
| FR-CIV-MOD-009 | Charter validator — reject hardcoded-outcome mods | §4 |
| FR-CIV-MOD-010 | Mod loading pipeline (discover → parse → validate → resolve → bind) | §5 |
| FR-CIV-MOD-011 | Dependency + version + capability model (semver) | §5.2 |
| FR-CIV-MOD-012 | Load ordering (topological + priority + deterministic tie-break) | §5.3 |
| FR-CIV-MOD-013 | Sandboxing + capability grants + resource limits | §6 |
| FR-CIV-MOD-014 | Hot-reload of mods (data-tier live, code-tier staged) | §7 |
| FR-CIV-MOD-015 | Stable mod API surface (trait + ABI contract, semver'd) | §8 |
| FR-CIV-MOD-016 | Conflict detection + resolution (ID collisions, law contradictions) | §9 |
| FR-CIV-MOD-017 | Sharing format — Workshop-style bundle = seed + diff | §10 |
| FR-CIV-MOD-018 | Sample mod walkthrough (end-to-end) | §11 |
| FR-CIV-MOD-019 | Mod test harness + lint (`civis mod validate`) | §12 |
| FR-CIV-MOD-020 | Save-game / mod compatibility + migration | §13 |

Each FR carries acceptance criteria inline. All trace to the AgilePlus epic **MOD-PLATFORM** and to the
charter; the charter clause `Mods extend laws, the rest emerges` is the parent NFR.

---

## 1. The moddable surface (FR-CIV-MOD-001)

The single rule, restated as a load-time gate: **a mod may add or parameterize a Layer-0 rule or a
content *grammar*; a mod may NOT author a Layer-1 outcome.**

### 1.1 Moddable (Layer-0 substrate + grammars)

| Surface | What a mod registers | Backing crate | Emergence preserved because… |
|---------|----------------------|---------------|------------------------------|
| **Materials** | new `MaterialSpec` (density, phase, flow, viscosity, repose, color, thermal/electrical props) | `civlab-sdk::material` + `civ-voxel` | it is a physical property table; behavior still falls out of the CA |
| **Reactions / phase rules** | reagent→product transitions w/ energy + threshold (temp/pressure/catalyst) | `laws` (`LawKind::Material`) | reaction is a *rule*; whether/where it fires emerges from local conditions |
| **Laws / physics constants** | new `Law` entries + bounded override of named constants (gravity scale, atmospheric composition, fictional-physics extensions) | `crates/laws` | constants parameterize the substrate; all dynamics still emerge |
| **Building / recipe grammars** | `BuildingBlueprint`, `RecipeDefinition`, **structure grammars** (parts, joints, material affinities, era gates) | `civlab-sdk::building` + `civ-build` | grammar = a *vocabulary* agents draw on; *what* gets built emerges from needs+resources |
| **Species / genome primitives** | new genome **loci**, trait→phenotype mappings, mutation operators, metabolic pathways, speciation-threshold params | `civ-genetics` (via SDK re-export) | primitives = the alphabet; species/sentience still emerge via fitness + Hamming speciation |
| **Biome / climate rules** | biome classification predicates (temp×moisture×altitude→biome), hydrology params, insolation curves, flora/fauna seed-rules | `civ-planet` (via SDK) | rules drive worldgen + ecology; specific ecosystems emerge |
| **Event hooks** | read-only observers + **bounded reactors** (may nudge substrate inputs, never set outcomes) | `civlab-sdk::events` | see §3.6 — reactors get a constrained verb set, not arbitrary world writes |
| **UI / overlays** | data overlays, inspector panels, legend lenses, debug HUDs (read sim state, render) | new `civlab-sdk::ui` | UI is presentation only; cannot write sim state |
| **Localization / naming grammars** | phoneme sets, morphology rules feeding the *emergent* language system | `civ-culture` (via SDK) | supplies generative rules; actual languages drift emergently |
| **Assets** | meshes, textures, audio, icons bound to registered IDs | asset manifest | cosmetic; no sim semantics |

### 1.2 NOT moddable (Layer-1 outcomes — rejected at load, FR-CIV-MOD-009)

- A fixed **faction/polity** (`faction: u32`, scripted nation, pre-set borders) — polities emerge.
- A scripted **market** (fixed prices, hardcoded trade routes) — markets emerge.
- A pre-built **city / settlement layout** placed as canon (user may *place* buildings in-session; a mod
  may not ship a settlement as a rule).
- A hardcoded **ideology / religion / culture** instance (mods supply *generative grammars*, not instances).
- A scripted **historical event / quest** with predetermined outcome — history is the measured record
  (see [`legends-engine.md`](legends-engine.md)), not an authored script.
- Direct **agent-behavior overrides** that bypass psyche/drives (mods tune drive *parameters*, not decisions).

The validator (§4) classifies every manifest entry as substrate/grammar (allow) or instance/outcome
(reject) using a typed allow-list, not free-form heuristics.

---

## 2. Mod manifest format (FR-CIV-MOD-000)

**RON is primary** (matches `crates/laws` convention + is the org default for mod-friendly data);
**JSON is parallel** (already supported by `manifest.rs` `parse_manifest`). The existing `ModManifest`
(`metadata`, `materials`, `buildings`, `recipes`, `events`) is the v0 nucleus; this spec extends it to
v1 with the additional surfaces. The loader keeps dual-format parsing already in `parse_manifest`.

### 2.1 Manifest layout on disk

```
mods/<mod-id>/
  manifest.ron            # the manifest (or manifest.json)
  laws/*.ron              # law-DB fragments (LawDb shape, schema-versioned)
  materials/*.ron         # MaterialSpec tables
  grammars/*.ron          # building / recipe / structure grammars
  genome/*.ron            # genome loci + trait maps
  biomes/*.ron            # biome predicates + climate params
  assets/                 # meshes/textures/audio/icons
  scripts/<entrypoint>.wasm   # optional sandboxed code module (§6)
  overlays/*.ron          # UI overlay declarations
  CHANGELOG.md            # author-supplied
```

### 2.2 Manifest schema (RON, v1) — fields

The manifest is **declarative metadata + capability requests + file references**. Heavy data lives in
the side files above (keeps the manifest reviewable and diffable for §10 sharing).

| Block | Field | Type | Notes |
|-------|-------|------|-------|
| `mod` | `id` | string (kebab, namespaced `author.name`) | stable; collision-checked |
| | `name`, `version` (semver), `author`, `description` | string | `version` MUST be semver (FR-CIV-MOD-011) |
| | `sdk` | semver-req | required SDK API range, e.g. `"^1.2"` (§8) |
| | `entrypoint` | optional string | wasm module if code-tier (§6) |
| `requires` | list of `{ id, version (semver-req) }` | dependency edges (§5.2) |
| `conflicts` | list of `{ id, reason }` | hard incompatibilities (§9) |
| `load_after` / `load_before` | list of mod ids | soft ordering hints (§5.3) |
| `priority` | i16 (default 0) | ordering tie-break; higher = later (override-wins) |
| `capabilities` | list of `Capability` | requested sandbox grants (§6.2); load fails if not granted |
| `provides` | `{ materials, laws, grammars, genome, biomes, overlays, events }` | **file globs**, not inline data |
| `compat` | `{ min_save_schema, max_save_schema }` | save-game gate (§13) |

`events` retains the existing `SimulationEventFilter` enum (`birth`/`death`/`tech`), extended in §3.6.

### 2.3 Canonical example (RON)

```ron
(
  mod: (
    id: "civlab.marble",
    name: "Marble & Masonry",
    version: "1.2.0",
    author: "CivLab",
    description: "Adds marble, a quarrying reaction, and a masonry building grammar.",
    sdk: "^1.0",
    entrypoint: None,
  ),
  requires: [ ( id: "civis.core", version: "^1.0" ) ],
  conflicts: [],
  load_after: [ "civis.core" ],
  priority: 0,
  capabilities: [ RegisterMaterials, RegisterReactions, RegisterGrammars, ObserveEvents ],
  provides: (
    materials: [ "materials/*.ron" ],
    laws:      [ "laws/*.ron" ],
    grammars:  [ "grammars/*.ron" ],
    genome:    [],
    biomes:    [],
    overlays:  [],
    events:    [ Birth, Tech ],
  ),
  compat: ( min_save_schema: 1, max_save_schema: 1 ),
)
```

**AC (FR-CIV-MOD-000):** both `manifest.ron` and an equivalent `manifest.json` parse to the identical
in-memory `ModManifest`; round-trip RON↔struct is lossless; an unknown top-level key is a hard parse
error (no silent drop — per the project loud-failure stance); missing required field → named error
listing the field and file path.

---

## 3. The registration surfaces (trait + data)

Every surface follows the existing hexagonal pattern in `civlab-sdk`: **mod implements a `*Registrar`
trait that writes into a host-owned `*Catalog`**; the host binds catalogs into the engine. This spec
adds catalogs/registrars for the new surfaces and groups them under the existing `ModRegistry`
(`registry.rs`), which gains `laws`, `genome`, `biomes`, `reactions`, and `overlays` fields.

### 3.1 Materials + reactions (FR-CIV-MOD-002)

- **Materials:** keep `MaterialRegistrar` / `MaterialCatalog` / `CustomMaterial` / `MaterialSpec`
  unchanged; extend `MaterialSpec` with optional `thermal_conductivity`, `specific_heat`,
  `electrical_conductivity`, `melt_point`, `boil_point`, `ignition_point` (all `Option`, default-skip so
  existing v0 manifests still parse).
- **Reactions:** new `ReactionRegistrar`/`ReactionCatalog`. A `ReactionDef` = `{ id, reagents:
  Vec<(MaterialId, u32)>, products: Vec<(MaterialId, u32)>, energy_delta, threshold:
  ReactionThreshold }` where `ReactionThreshold` carries optional `min_temp`, `min_pressure`,
  `catalyst: Option<MaterialId>`. Reactions are emitted to `crates/laws` as `LawKind::Material` entries
  so the law validator (mass/energy conservation, dependency closure) gates them.
- **AC:** a reaction whose products violate mass conservation (Σ product mass ≠ Σ reagent mass within the
  law DB's tolerance) is **rejected** by `LawDb::validate`, surfaced as a named per-mod error.

### 3.2 Building / recipe / structure grammars (FR-CIV-MOD-003)

- Keep `BuildingRegistrar`/`RecipeRegistrar` and their catalogs.
- Add **structure grammars**: a `StructureGrammar` = `{ id, parts: Vec<PartRule>, joints:
  Vec<JointRule>, material_affinities, era_gate }`. A `PartRule` describes a placeable component
  (wall/floor/roof/support) with material constraints and structural-stress tags consumed by the
  physics substrate. The grammar is a *vocabulary*; the `civ-build` self-organizing placer + the user
  draw from it. Mods do NOT place instances.
- **AC:** registering a grammar makes its parts available to the emergent builder and the user palette;
  a manifest that ships a concrete placed structure (coordinates + canon flag) is rejected by §4.

### 3.3 Laws / physics constants (FR-CIV-MOD-004)

- Mods ship `laws/*.ron` fragments in the existing `LawDb` shape. The loader **merges** all fragments +
  core into one `LawDb`, then runs `LawDb::validate` over the union (duplicate-id, missing-dependency,
  fictional-extension-underspecification checks already exist).
- **Constant overrides:** a bounded, named set (`gravity_scale`, `atmosphere_o2_fraction`,
  `tick_seconds`, etc.) declared in a `constants` block; each has a host-defined `[min,max]` clamp.
  Out-of-range → rejected with the clamp range in the error.
- **AC:** two mods adding contradictory law constants → conflict (§9), not last-writer-wins silently;
  the validator runs on the *merged* DB so cross-mod dependency references resolve or fail loudly.

### 3.4 Species / genome primitives (FR-CIV-MOD-005)

- New `GenomeRegistrar`/`GenomeCatalog`. A mod registers: **loci** (named byte-ranges in the DNA vector
  with semantic tags), **trait→phenotype maps** (locus value → physical/metabolic effect bounded by
  laws), **mutation operators** (point/indel/recombination rates per locus), **metabolic pathways**
  (input materials → energy, gated by reactions §3.1), and **speciation-threshold params** (Hamming
  distance cutoffs).
- Charter guard: a mod supplies the **genome alphabet + fitness-relevant mappings**, NOT a finished
  humanoid species. Abiogenesis and speciation remain emergent.
- **AC:** a registered locus that overlaps an existing locus's byte-range without an explicit `extends`
  is a conflict; a trait map producing a phenotype that violates a conservation law is rejected.

### 3.5 Biome / climate rules (FR-CIV-MOD-006)

- New `BiomeRegistrar`/`BiomeCatalog`. A `BiomeRule` = a predicate over `(temperature, moisture,
  altitude, latitude, ...)` → biome class, plus seed-rules for flora/fauna spawn *probabilities* (not
  placements) and hydrology/insolation parameter tweaks (clamped like §3.3).
- **AC:** biome predicates must **partition** the climate space the host declares (no gap, no overlap
  after priority resolution); a gap/overlap is reported as a named warning→error per host policy.

### 3.6 Event hooks (FR-CIV-MOD-007)

- Keep `SimulationEventHook` (`on_birth`/`on_death`/`on_tech`) and `SimulationEventFilter`. Extend the
  event taxonomy: `Birth, Death, Tech, Migration, Reaction, StructureBuilt, BiomeShift, SpeciationEvent`
  (all opt-in via `provides.events`).
- **Two tiers of hook:**
  - **Observer** (default, no capability beyond `ObserveEvents`): receives `&Event`, may read sim state,
    may emit overlay data / legend annotations. **Cannot write sim state.**
  - **Reactor** (requires `ReactSubstrate` capability): may call a **bounded verb set**
    (`nudge_input(material, amount, region)`, `emit_seed(genome, region)`, `set_constant_tick(...)`
    within clamps) — i.e. it perturbs *substrate inputs*, never sets *outcomes*. All reactor calls are
    rate-limited and budgeted (§6.3) and logged to the legend/event feed.
- **AC:** an observer that attempts a substrate write fails the capability check loudly; a reactor that
  exceeds its per-tick budget is throttled with a named warning, not silently dropped.

### 3.7 UI / overlays (FR-CIV-MOD-008)

- New `civlab-sdk::ui` module: `OverlayRegistrar`/`OverlayCatalog`. An `OverlayDef` declares a data
  source (a read query over sim state), a render kind (heatmap / iso-lines / markers / inspector-panel /
  legend-lens), and a binding to the inspector "inspect-anything" surface.
- Overlays are **read-only**; they run on the render thread, sandboxed from sim writes.
- **AC:** an overlay cannot mutate sim state (compile-time: it only receives read handles); a missing
  data source is a named load error.

---

## 4. Charter validator (FR-CIV-MOD-009)

A dedicated pass run **after parse, before bind**. It classifies each manifest/file entry against a typed
allow-list and rejects Layer-1 outcomes.

Pseudocode (illustrative only):

```
for entry in manifest.all_entries():
    class = classify(entry.kind)            # Substrate | Grammar | Instance | Outcome
    if class in {Instance, Outcome}:
        errors.push(CharterViolation { mod, entry, class, hint })
    if entry.kind == Law and entry overrides a hardcoded-only constant outside clamp:
        errors.push(ClampViolation { ... })
fail_loudly_if(!errors.is_empty())
```

`classify` is a static match on the registration enum (materials/reactions/laws/grammars/genome/biomes/
overlays = allow; placed-structure / fixed-faction / scripted-event / fixed-price-table / agent-decision-
override = reject). No free-text heuristics — the *type* of the entry decides.

**AC:** the validator rejects a manifest containing any Instance/Outcome entry with a message naming the
mod, the offending entry, and the charter clause; a pure-substrate mod passes; the rejection is a hard
load failure (no partial load).

---

## 5. Loading pipeline (FR-CIV-MOD-010..012)

### 5.1 Stages (DAG)

```
discover → parse → charter-validate → dependency-resolve → order → law-merge+validate
        → conflict-check → capability-grant → bind(catalogs) → activate
```

Each stage is a gate: failure stops the pipeline and reports **every** failing mod (collect-all, not
fail-fast-on-first), listing items semicolon-separated per the loud-failure stance.

### 5.2 Dependency + version model (FR-CIV-MOD-011)

- Each mod declares `requires: [{id, semver-req}]` and `sdk: semver-req`.
- Resolver builds a dependency graph; a missing/incompatible dependency is a named error
  (`mod 'civlab.marble' requires 'civis.core ^1.0', found '0.9.2'`).
- SDK compatibility checked against `SCHEMA_VERSION` (§8) using semver: a mod requesting `^1.0` loads on
  SDK `1.x`, rejected on `2.0`.

### 5.3 Load ordering (FR-CIV-MOD-012)

- **Topological sort** over `requires` + `load_after`/`load_before` edges.
- **Tie-break** within a topo layer: `priority` ascending (lower binds first, higher overrides), then
  mod `id` lexicographic — a stable, reproducible order (determinism not *required* per charter, but a
  stable mod order is a usability win and is cheap).
- A cycle in the ordering graph is a named error listing the cycle members.

**AC:** given mods A(requires B), B, C(priority 10), D(priority -5) → order is `D, B, A, C` (or any valid
topo order with the stated tie-break); a `load_after` cycle is rejected.

---

## 6. Sandboxing + capabilities (FR-CIV-MOD-013)

### 6.1 Two tiers of mod

- **Data-tier mods** (no `entrypoint`): pure declarative RON/JSON + assets. **No code executes** — the
  safest, hot-reloadable tier (§7). The vast majority of mods (materials, reactions, laws, grammars,
  biomes, genome tables, overlays-by-declaration) are data-tier.
- **Code-tier mods** (`entrypoint: *.wasm`): run inside a **WASM sandbox** (e.g. `wasmtime`,
  capability-gated, no ambient host access). Used only for reactors (§3.6) and computed overlays.
  Native dynamic libraries are **NOT** a supported distribution format (security); the `cdylib`
  crate-type stays for first-party/dev only.

### 6.2 Capability model

A mod requests capabilities in its manifest; the host grants a subset per policy. **No capability ⇒ the
corresponding registrar trait is never invoked** (deny-by-default). Capabilities:

`RegisterMaterials, RegisterReactions, RegisterLaws, RegisterGrammars, RegisterGenome, RegisterBiomes,
RegisterOverlays, ObserveEvents, ReactSubstrate, ReadAsset`.

`ReactSubstrate` is the only capability that can influence the running sim, and only via the bounded
verb set (§3.6).

### 6.3 Resource limits

Per code-tier mod, enforced by the WASM host: memory ceiling, per-tick CPU fuel budget, per-tick reactor
verb budget, and a wall-clock timeout per hook invocation. Exceeding a limit → the mod is **quarantined**
(deactivated, named error in the mod console), never silently degraded. Data-tier mods have only a parse
time/size limit.

**AC:** a WASM mod that loops forever is killed at the fuel/timeout limit and quarantined; a data-tier
mod cannot escalate to substrate writes (no `ReactSubstrate` without `entrypoint` + grant).

---

## 7. Hot-reload (FR-CIV-MOD-014)

- **Data-tier (live hot-reload):** a file watcher over `mods/` detects changes to `*.ron`/assets; on
  change the loader re-runs parse→validate→merge for the affected mod and **swaps the catalog entries**
  via copy-on-write. Materials/reactions/laws/grammars/biomes/overlays reload without restart. The sim
  reads catalogs through a versioned handle, so an in-flight tick sees a consistent snapshot.
- **Law constants** that affect already-instantiated state (e.g. gravity) apply at the next tick
  boundary; the change is logged to the event feed.
- **Code-tier (staged reload):** a changed WASM module is recompiled and **staged**, then swapped at a
  tick boundary with state hand-off via a `migrate(old_state) -> new_state` hook the mod may implement;
  if absent, the mod re-initializes. Reactor budgets reset on swap.
- **Conflict on reload** (e.g. an edit introduces an ID collision or law contradiction) → the reload is
  **rejected and the prior version stays active**, with a named error; the sim never enters a
  half-loaded state.

**AC:** editing a material's color in a data-tier mod reflects in-world within one watch cycle without
restart; introducing a duplicate material ID on reload keeps the old catalog and reports the collision.

---

## 8. Stable mod API surface (FR-CIV-MOD-015)

The SDK exposes a **semver'd public surface** (`SCHEMA_VERSION`, currently `0.1.0` → target `1.0.0` at
platform GA). The stable contract is:

- **Traits (the API):** `MaterialRegistrar, ReactionRegistrar, BuildingRegistrar, RecipeRegistrar,
  LawRegistrar, GenomeRegistrar, BiomeRegistrar, OverlayRegistrar, SimulationEventHook` (+ reactor verb
  trait). These signatures are frozen within a major version.
- **Data types (the schema):** `ModManifest`, `MaterialSpec`, `ReactionDef`, `Law`/`LawDb`,
  `BuildingBlueprint`, `RecipeDefinition`, `StructureGrammar`, `GenomeLocus`, `BiomeRule`, `OverlayDef`,
  `Capability`. Additive fields use `#[serde(default)]` so minor versions stay backward-compatible
  (as `entrypoint` already does).
- **WASM ABI (code-tier):** a thin, versioned host-import table (the bounded verbs + read queries),
  declared in a `.wit` interface so code-tier mods compile against a stable contract.
- **Versioning policy:** breaking trait/ABI change → major bump → old mods rejected with a clear
  "requires SDK ^N, host provides M" message. Additive → minor. Doc/fix → patch.

**AC:** a mod built against SDK `1.0` loads on host SDK `1.4`; the same mod is cleanly rejected (named
error, not crash) on host SDK `2.0`; adding an optional `MaterialSpec` field does not break existing
manifests.

---

## 9. Conflict detection + resolution (FR-CIV-MOD-016)

Conflict classes and resolution:

| Conflict | Detection | Resolution |
|----------|-----------|------------|
| **ID collision** (material/building/recipe/law/locus/biome id reused) | namespaced ids + post-merge scan | hard error unless one mod declares `extends: <id>` (explicit override) |
| **Explicit `conflicts`** | manifest `conflicts` list | both present → hard error naming both |
| **Law contradiction** | `LawDb::validate` on merged DB + constant-clamp + conservation checks | hard error; no silent last-writer-wins |
| **Constant override clash** | two mods set same constant to different values | resolved by `priority` (higher wins) **only if** both within clamp; logged; if `priority` equal → hard error |
| **Biome partition gap/overlap** | partition check (§3.5) | priority-resolved overlap allowed; gap = error |
| **Capability denied** | grant policy | mod load fails (deny-by-default) |

All conflicts collect-all and report together. Namespaced ids (`author.name:entity`) make collisions
rare by construction.

**AC:** two mods registering material id `marble` without `extends` → reported collision naming both
mods; one using `extends:"civlab.marble:marble"` → override accepted and recorded in the merge log.

---

## 10. Sharing format — seed + diff (FR-CIV-MOD-017)

Steam-Workshop-style sharing. A shared artifact is a **`.civmod` bundle** = a content-addressed,
signed tarball:

```
<mod-id>-<version>.civmod
  manifest.ron
  laws/ materials/ grammars/ genome/ biomes/ overlays/ assets/ scripts/
  LICENSE  CHANGELOG.md
  .civmod-lock           # resolved dependency ids+versions+hashes
  .civmod-sig            # author signature over the content hash
```

### 10.1 Two share modes

- **Full mod bundle** (above): the complete mod, for distribution/install.
- **World share = seed + diff** (the Workshop "share my world" path): a saved world is shared as
  **`(worldgen_seed, mod-set lock, player-diff)`** rather than a full snapshot where possible. The
  `player-diff` captures user-authored deltas (placed buildings, terrain edits) atop the
  seed+mod-deterministic-*enough* base. Because determinism is **not guaranteed** (charter §"Determinism
  is NOT a requirement"), the share also embeds a **full snapshot fallback**; the recipient regenerates
  from seed+mods+diff for a *recognizable* world and falls back to the snapshot if regeneration
  diverges beyond a tolerance. This keeps shares small (seed+diff) while staying correct (snapshot
  fallback).

### 10.2 Registry / index

- A local, OSS, free index (`mods.index.ron`) lists installed mods with hashes; an optional self-hosted
  HTTP catalog mirrors the Workshop role without a paid service (per project OSS-first stance). No paid
  SaaS dependency.

**AC:** a `.civmod` installs by extracting into `mods/<id>/` and passing the full §5 pipeline; a world
share opens to a *recognizable* world via seed+mods+diff, and falls back to the embedded snapshot when
regeneration diverges; signature/hash mismatch blocks install with a named error.

---

## 11. Sample mod walkthrough (FR-CIV-MOD-018) — "Marble & Masonry"

A complete, charter-clean, **data-tier** mod (no code). End-to-end author flow:

1. **Scaffold:** `civis mod new civlab.marble` creates the `mods/civlab.marble/` skeleton (manifest +
   empty side-file dirs).
2. **Author the manifest** (the RON in §2.3): declares materials/reactions/grammars capabilities,
   `requires civis.core ^1.0`, `events: [Birth, Tech]`.
3. **Add a material** — `materials/marble.ron`: a `MaterialSpec` for Marble (solid, density 2700, stone
   color, melt_point set). Behavior (piling, structural use) emerges from the CA + grammar.
4. **Add a reaction** — `laws/quarrying.ron`: a `LawKind::Material` entry `limestone + heat → marble`
   (mass-balanced; validator checks conservation).
5. **Add a grammar** — `grammars/masonry.ron`: a `StructureGrammar` with wall/floor/support parts that
   have a material affinity for Marble and an `era_gate` of 2. This adds *vocabulary*; the emergent
   builder and the user palette gain marble masonry — **no settlement is placed**.
6. **Observe events** — declares `Birth`/`Tech` observers that annotate the legend feed (read-only) when
   a settlement first builds in marble. No sim writes.
7. **Validate:** `civis mod validate civlab.marble` runs charter-validate + law-merge + conflict-check;
   reports green.
8. **Run hot:** drop the folder into a running game's `mods/`; the watcher live-loads it (§7); marble
   appears in the palette and quarrying reactions begin firing where limestone meets heat — *emergently*.
9. **Share:** `civis mod pack civlab.marble` produces `civlab.marble-1.2.0.civmod` (signed, locked) for
   the Workshop index.

**What stays emergent:** whether anyone *builds* in marble, where quarries appear, which cultures prize
masonry — all emerge from needs, resources, and the substrate. The mod only added the *possibility*.

---

## 12. Mod test harness + lint (FR-CIV-MOD-019)

- `civis mod validate <id>`: runs the full §5 pipeline in dry-run (no bind/activate) and reports
  parse/charter/law/conflict/capability results — the author's pre-flight, mirroring CI.
- A **mod-lint** ruleset: namespaced-id enforcement, semver presence, `#[serde(default)]` additive
  checks for custom types, unused-capability warnings, missing-asset references, charter-class lint.
- Reuses `LawDb::validate` and the existing manifest loader; adds catalog-merge dry-run. xUnit-style
  (Rust test) fixtures per surface; these are the platform's own tests, not author-facing.

**AC:** `civis mod validate` exits non-zero with a collected error list on any failure; a clean mod exits
zero; the harness covers each surface (material/reaction/law/grammar/genome/biome/overlay/event).

---

## 13. Save-game / mod compatibility (FR-CIV-MOD-020)

- A save records its **active mod-set** (ids+versions+content-hashes) and a `save_schema` version.
- On load, the host compares the save's mod-set to the installed set: missing mods → named error +
  option to install from index; version drift within semver-minor → load with a migration log;
  major drift → blocked unless the mod ships a `migrate` hook.
- The manifest `compat.{min,max}_save_schema` gates which saves a mod version may open.

**AC:** loading a save whose mods are absent reports each missing `id@version`; a minor-version drift
loads with a recorded migration; a save_schema outside `compat` range is blocked with the range named.

---

## 14. Phased WBS + DAG (build order)

| Phase | Task ID | Deliverable | Depends On |
|-------|---------|-------------|------------|
| P1 Schema | T1 | Extend `ModManifest` to v1 (capabilities/requires/provides/compat) | — |
| P1 | T2 | Extend `MaterialSpec` (thermal/electrical, optional) | — |
| P2 Surfaces | T3 | `ReactionRegistrar/Catalog` + law-emit | T1,T2 |
| P2 | T4 | `LawRegistrar` + merge-into-`LawDb` + constant-clamp | T1 |
| P2 | T5 | `StructureGrammar` + registrar | T1 |
| P2 | T6 | `GenomeRegistrar`, `BiomeRegistrar`, `OverlayRegistrar` | T1 |
| P2 | T7 | Event taxonomy + observer/reactor tiers | T1 |
| P3 Pipeline | T8 | Charter validator (classify) | T3–T7 |
| P3 | T9 | Dependency resolve + semver + ordering | T1 |
| P3 | T10 | Conflict detection (merge scan + law-validate) | T3,T4,T8 |
| P3 | T11 | Capability grant + WASM sandbox + limits | T7 |
| P4 Live | T12 | Hot-reload (data live, code staged) | T8,T10,T11 |
| P4 | T13 | `.civmod` pack/sign + seed+diff world share | T9 |
| P5 Tooling | T14 | `civis mod` CLI (new/validate/pack) + mod-lint | T8–T13 |
| P5 | T15 | Save-game mod-set compat + migration | T9,T13 |

**Cross-Project Reuse Opportunities:** the manifest/loader/validator/capability/WASM-sandbox machinery is
a generic *plugin platform* candidate for the Phenotype org (DINOForge, WSM3D mod loaders). Propose
extracting the host-agnostic core (manifest model + dependency/order resolver + capability + WASM host)
into a shared `phenotype-mod-platform` crate; civlab-sdk would consume it and add Civis-specific
catalogs. Confirm destination with the user before any cross-repo extraction.

---

## 15. Acceptance summary

A mod platform is **charter-aligned** iff: every moddable entry is substrate/grammar (§1.1), every
Layer-1 outcome is rejected (§4), the loader fails loudly and collect-all (§5), capabilities are
deny-by-default with bounded substrate verbs (§6), data-tier mods hot-reload live (§7), the API is
semver-stable (§8), conflicts never resolve silently (§9), and shares are seed+diff with snapshot
fallback (§10). The "Marble & Masonry" walkthrough (§11) exercises all of these on a real, code-free mod.
