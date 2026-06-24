# Voxel-Emergent Vision and Migration Plan

**Version:** 1.0
**Status:** AUTHORITATIVE DESIGN — supersedes heightmap/faction assumptions in PRD.md v1.0, SPEC.md, and PLAN.md where they conflict.
**Date:** 2026-05-29
**Scope:** Full paradigm statement + gap analysis + emergence model + phased migration WBS with DAG

---

## 1. Vision: Voxel-Material-Fluid + Emergent Life

### 1.1 The Paradigm

Civis is a **volumetric material simulation** in which every cell of the world is a voxel holding a **material state** — solid rock, granular sand, liquid water, gas, plasma, organic tissue, or composite — subject to physics rules (gravity, fluid flow, pressure, heat transfer, reaction) that run deterministically as a cellular automaton. Life, intelligence, and society are **emergent consequences** of material+energy conditions, not pre-loaded entities. The primary gameplay mode is being a god who manipulates materials; the primary observation mode is watching emergence unfold.

The closest references are **Noita** (per-pixel material CA with heat/pressure/chemical reactions), **Powder Toy** (open-source cellular automaton of ~120 materials), and **WorldBox** (sandbox god-game where conditions drive emergence). Civis extends that lineage with:

- Full 3D volumetric voxels (no billboards, no flat water, no 2D cross-sections)
- Deterministic replay at simulation scale via the existing ChaCha20Rng + fixed-point substrate
- ECS-layer agents that bootstrap **from** material chemistry rather than being pre-spawned
- Emergent sentience, emergent species clusters, and emergent social groupings

### 1.2 Alignment with Seed Plans

A careful read of the existing specs and code shows the foundation **already points toward this vision** in three ways:

**Genetics and speciation are fully algorithmic.** `crates/genetics` implements DNA mutation, recombination, Hamming-distance speciation, and cosine-similarity fitness against an environment vector. `crates/species` implements deterministic DNA→Phenotype expression covering morphology (leg count, eye count, body hue) and behavioural weights (aggression, curiosity, sociability, intelligence). These were designed without assuming human forms; the schema is genuinely species-agnostic.

**The voxel substrate exists and is in progress.** `crates/voxel` re-exports `phenotype-voxel` (SVO + dense 16³ leaf chunks, deterministic dirty queue, fixed-point world coords, `MaterialId` palette, pluggable `Mesher` trait). FR-CIV-VOXEL-001 through FR-CIV-VOXEL-010 are all marked `implemented`. The kernel is architecturally sound for a material CA.

**Culture and ideology diffusion are explicitly modelled as continuous fields,** not discrete factions. `crates/diffusion` implements a Bass-Rogers S-curve; `agileplus-specs/civ-009-culture-diffusion` models cultural spread as a continuous `culture_affinity` field per agent, not as a faction membership flag. The spec already says "ideology convergence" not "faction assignment."

**Where the seed plans diverge:** The PRD.md (v1.0, 2026-02-21) and early PLAN.md describe a heightmap terrain (`TerrainMap`), a flat water representation, a hardcoded `(Human, Dwarf, Elf, Goblin, ...)` species enum (civ-008 spec FR-CIV-BIO-001), and a `Civilian { faction: u32 }` field that pre-assigns each agent to a named group (civ-agents lib). The SPEC.md `WorldState` also includes a `terrain: TerrainMap` (2D heightmap) rather than a volumetric voxel world. These are the paradigm mismatches. They are addressed in the migration plan below.

---

## 2. Current vs Target

| Dimension | Current Build (What Exists) | Target Vision (What the Paradigm Needs) | Gap |
|---|---|---|---|
| **World representation** | `TerrainMap` (2D heightmap + scalar elevation field per cell) | `VoxelWorld<MaterialId>` — every xyz cell is a material; full volumetric 3D | Heightmap must be replaced by voxel world gen; no shared code path |
| **Water** | Flat water: a float threshold on the heightmap; rendered as a plane | Fluid material cells governed by CA flow rules (pressure, viscosity, density) | Requires CA physics layer in voxel substrate |
| **World renderer** | No Bevy client yet (clients/bevy-ref stub) | Bevy chunk streamer consuming `MeshBuffer` from `CubicMesher` / greedy mesher; no billboards | Renderer must be built from voxel chunks, not from heightmap meshes |
| **Sentient agents** | `Civilian { id, faction: u32, age }` pre-spawned at simulation start with fixed faction membership | Agents bootstrap from material CA when energy+chemistry conditions cross emergence thresholds | Life-emergence system does not exist; agent spawn is currently manual |
| **Species** | `Phenotype` from `crates/species` is architecture-correct but civ-008 spec (FR-CIV-BIO-001) names a hardcoded enum `(Human, Dwarf, Elf, Goblin)` | Species are phenotype clusters defined by Hamming-distance centroid in DNA space; no hardcoded taxonomy | The spec-level enum must be removed; `civ-genetics::Species` record (already algorithmic) is the correct substrate |
| **Factions** | `faction: u32` on `Civilian` is a pre-assigned integer ID; no mechanism for group formation | Social groups form from shared territory, kinship graph, cultural affinity, and emergent resource pressure | Faction field must be deprecated; emergent grouping system needed |
| **Cultural spread** | `civ-diffusion` S-curve exists; `civ-009` spec models `culture_affinity` correctly | Culture diffusion over adjacency graph driven by material-world contact (shared biome, trade flow through voxel channels) | Adjacency graph must be grounded in voxel topology, not abstract nations |
| **Physics / material rules** | No CA physics; voxel substrate has only storage + meshing | Per-tick material CA: gravity, fluid flow, gas expansion, heat transfer, chemical reaction table | Largest missing system; lives in a new `crates/material-ca` crate |
| **God tools / player interaction** | No player-facing tools yet | Material brush: paint/erase cells; heat/cool regions; spawn conditions; observe emergence | UI layer needed; `docs/guides/ui-design-system.md` has WorldBox-style bottom bar as reference |
| **Determinism scope** | ChaCha20Rng + fixed-point enforced for engine tick | CA must also be deterministic: integer arithmetic only, no floats in material state, neighbourhood order canonicalised | NFR-CIV-DET-003 scope must be extended to CA paths |

---

## 3. Emergence Model

### 3.1 Principles

Life, species, and social structure must **arise from simulation state**, not be pre-loaded. Three emergence tiers are proposed:

**Tier 1 — Chemical/Material Emergence (abiogenesis analog):** Certain material combinations in the CA produce self-replicating patterns when energy gradients (heat differential, chemical gradient) exceed a threshold. These are not biologically modelled cells — they are CA patterns that exhibit replication and differential survival. Once a replicating pattern stabilises in a region, the engine tags it as a proto-life event and assigns an ECS entity.

**Tier 2 — Phenotypic Emergence (speciation):** Each proto-life entity carries a `Dna` (from `crates/genetics`). The environment vector fed to `civ_genetics::fitness()` is derived from the local material context: temperature, ambient material composition, elevation, available nutrients. Selection pressure is continuous: agents in hostile material configurations accumulate damage and die faster; agents whose DNA fitness against the local environment vector exceeds a survival threshold reproduce. `civ_genetics::should_speciate()` fires when Hamming distance between two lineages crosses `DnaClass::speciation_threshold`, emitting a `civ_genetics::Species` record. **No hardcoded taxonomy exists.** The `(Human, Dwarf, Elf, Goblin)` enum from civ-008 FR-CIV-BIO-001 is **removed**. Species are discovered at runtime by the speciation algorithm.

**Tier 3 — Social Emergence (group formation):** Social groups form from overlapping graphs, not from pre-assigned IDs. Three mechanisms:

- **Kinship graph:** Parent–offspring links create clans. When a kinship cluster's shared territory becomes non-trivially large, the engine can tag it as a candidate group.
- **Cultural affinity clusters:** The `culture_affinity` continuous field from `civ-009` already models ideological proximity. When a set of agents' pairwise affinities exceed `CONVERGENCE_THRESHOLD`, they form a social cluster. `civ-diffusion` drives this.
- **Resource pressure territories:** Material CA creates resource concentrations (ore veins, water sources, fertile soil). Agents that collectively defend a resource zone develop in-group/out-group signalling. This emerges from the CA spatial distribution, not from scripted borders.

The deprecated `faction: u32` field on `Civilian` is **removed** and replaced with:
- `kinship_cluster: Option<ClusterId>` — set by the kinship graph system
- `culture_cluster: Option<ClusterId>` — set by the affinity clustering system
- `territory_cluster: Option<ClusterId>` — set by the spatial resource system

`ClusterId` is a stable runtime-assigned integer with no semantic taxonomy. Player-visible faction names, flags, and colours are **rendered annotations** computed from cluster membership, not hardcoded identities.

### 3.2 Refactoring the Fixed Species Enum and Faction Concept

**Species:** Remove `FR-CIV-BIO-001`'s `(Human, Dwarf, Elf, Goblin, ...)` enum entirely from civ-008. Replace with:
- `SpeciesRecord { id: u64, dna_class: String, founder_centroid: Dna, discovered_tick: u64, name: Option<String> }` — the `Species` type already in `crates/genetics` extended with `discovered_tick`.
- `SpeciesRegistry: BTreeMap<u64, SpeciesRecord>` maintained by the engine; updated whenever `should_speciate()` fires.
- Phenotype rendering uses the expressed `Morphology` fields (leg_count, body_color_hue, etc.) directly; the renderer maps these to procedural geometry, not to pre-made humanoid meshes.

**Factions:** Remove `faction: u32` from `Civilian`. Introduce a lightweight `SocialCluster` ECS component holding a `ClusterId` and a `ClusterKind` (Kinship | Cultural | Territorial). The agent carries `Vec<ClusterId>` memberships. Cluster formation, merge, and split events are logged to the event stream (deterministic, replayable). The `civ-diffusion` S-curve already provides the mathematical substrate for cultural cluster formation; it only needs to be wired to spatial voxel adjacency rather than to abstract nation adjacency.

---

## 4. Migration Plan — Phased WBS and DAG

### 4.1 Phase Table

| Phase | ID | Title | Core Deliverable | Depends On | FR/NFR IDs |
|---|---|---|---|---|---|
| 1 | P-VM-1 | Voxel material substrate | Material CA engine: per-tick physics update for gravity, fluid flow, gas, heat, reactions; `MaterialDef` palette in RON; deterministic neighbourhood scan | FR-CIV-VOXEL-* (existing, all implemented); new substrate below | FR-CIV-VOXEL-020 (CA physics rules), FR-CIV-VOXEL-021 (material palette RON), FR-CIV-VOXEL-022 (CA determinism under fixed-point) |
| 2 | P-VM-2 | Procedural voxel world generation | Replace `TerrainMap` heightmap with a voxel world gen pipeline: strata (bedrock, soil, ore), hydrology (water-filled basins), atmosphere (gas pockets); seeded, deterministic | P-VM-1 (voxel world must exist before gen populates it) | FR-CIV-VOXEL-030 (strata gen), FR-CIV-VOXEL-031 (hydrology), FR-CIV-VOXEL-032 (atmospheric gas), NFR-CIV-DET-001, NFR-CIV-DET-003 |
| 3 | P-VM-3 | Bevy voxel chunk renderer | Bevy desktop client consuming `MeshBuffer` from `CubicMesher`/`GreedyMesher`; chunk streaming (NFR-CIV-SCALE-002); transparent/translucent material pass for liquids/gases; no heightmap or flat-water path | P-VM-1 (mesh buffers must exist), P-VM-2 (world gen feeds renderer) | FR-CIV-VOXEL-010 (mesher, existing), FR-CIV-RENDER-001 (chunk streaming), FR-CIV-RENDER-002 (material transparency pass), NFR-CIV-PERF-001, NFR-CIV-PERF-005, NFR-CIV-SCALE-002 |
| 4 | P-VM-4 | Life emergence on voxel substrate | Abiogenesis threshold system: scan for material CA patterns that exceed replication criteria; bootstrap ECS agent from pattern; attach `Dna::random()` from local environment vector; wire survival → reproduction loop; `SpeciesRegistry` driven by `should_speciate()` | P-VM-2 (world gen produces chemistry gradients), P-VM-1 (CA runs per tick) | FR-CIV-EMERGENCE-001 (abiogenesis thresholds), FR-CIV-EMERGENCE-002 (agent bootstrap from CA), FR-CIV-EMERGENCE-003 (environment-vector fitness), FR-CIV-EMERGENCE-004 (speciation registry), remove FR-CIV-BIO-001 hardcoded enum |
| 5 | P-VM-5 | Emergent social grouping | Kinship graph system; cultural affinity clustering via `civ-diffusion` wired to voxel adjacency; territorial resource clustering; replace `faction: u32` on `Civilian` with `Vec<ClusterId>`; social cluster event logging | P-VM-4 (agents must exist with DNA + phenotype), P-VM-2 (material topology drives territory) | FR-CIV-EMERGENCE-010 (kinship clusters), FR-CIV-EMERGENCE-011 (cultural clusters via diffusion), FR-CIV-EMERGENCE-012 (territorial clusters), FR-CIV-EMERGENCE-013 (cluster event log), deprecate `Civilian::faction` |
| 6 | P-VM-6 | God-tool UI for material interaction | Bevy egui material brush tool (paint, erase, heat, cool); condition observer overlay (temperature, chemistry, life density); god-view zoom/pan; god command → engine CA write path | P-VM-3 (renderer must exist), P-VM-4 (life must be observable) | FR-CIV-UI-001 (material brush), FR-CIV-UI-002 (condition overlay), FR-CIV-UI-003 (emergence notification), NFR-CIV-ACC-002 through NFR-CIV-ACC-004, NFR-CIV-PERF-001 |

### 4.2 DAG of Phase Dependencies

```
P-VM-1 (Voxel CA Physics)
  └─> P-VM-2 (Procedural World Gen)
        ├─> P-VM-3 (Bevy Renderer)          [can run in parallel with P-VM-4]
        │     └─> P-VM-6 (God Tools UI)
        └─> P-VM-4 (Life Emergence)
              └─> P-VM-5 (Social Emergence)
                    └─> P-VM-6 (God Tools UI)
```

P-VM-3 and P-VM-4 may run in parallel once P-VM-2 is complete. P-VM-6 requires both P-VM-3 (to render) and P-VM-4/5 (to have something worth observing).

### 4.3 New FR IDs (Proposed)

The following FR identifiers are introduced by this migration plan. They do not yet exist in `FUNCTIONAL_REQUIREMENTS.md` or `docs/traceability/fr-3d-matrix.md` and must be added as the corresponding phases are implemented.

**FR-CIV-VOXEL-020 through FR-CIV-VOXEL-032** — extend the existing `FR-CIV-VOXEL-*` series for CA physics and world gen:

| Proposed FR ID | Summary |
|---|---|
| FR-CIV-VOXEL-020 | Material CA executes per-tick gravity update: unsupported solid/powder cells fall one voxel per tick toward -Y. |
| FR-CIV-VOXEL-021 | Material palette defined in RON (`crates/material-ca/data/materials.ron`); palettes round-trip without loss. |
| FR-CIV-VOXEL-022 | CA neighbourhood scan is deterministic: scan order canonical (BTreeMap key order); no floats in material state transitions. |
| FR-CIV-VOXEL-023 | Fluid CA: liquid material flows laterally when vertically blocked; pressure propagates via fixed-point depth accumulation. |
| FR-CIV-VOXEL-024 | Gas CA: gas material rises, disperses into adjacent empty cells; density field decays over ticks. |
| FR-CIV-VOXEL-025 | Heat transfer CA: temperature field propagates between adjacent cells; ignition threshold triggers material phase change. |
| FR-CIV-VOXEL-030 | World gen strata: bedrock, soil, ore layers generated deterministically from seed; voxel world valid after gen. |
| FR-CIV-VOXEL-031 | World gen hydrology: water-filled basin cells generated from elevation + permeability of strata. |
| FR-CIV-VOXEL-032 | World gen atmosphere: gas-pocket cells seeded in underground cavities; composition from RON material table. |

**FR-CIV-EMERGENCE-001 through FR-CIV-EMERGENCE-013** — new series for life and social emergence:

| Proposed FR ID | Summary |
|---|---|
| FR-CIV-EMERGENCE-001 | Abiogenesis threshold: engine scans CA state each tick; when a configurable set of material conditions co-occur in a region, a proto-life event is emitted. |
| FR-CIV-EMERGENCE-002 | Agent bootstrap: a proto-life event spawns one ECS entity with `Dna::random(rng)` seeded from local material hash; agent carries `Position3d`, `Needs`, `LodTier`. |
| FR-CIV-EMERGENCE-003 | Environment-vector fitness: each agent's fitness is computed per-tick as cosine similarity of its `Dna` against a local environment vector derived from adjacent material cells. |
| FR-CIV-EMERGENCE-004 | Speciation registry: when `should_speciate()` fires for any two lineages, a `SpeciesRecord` is created, logged to the event stream, and inserted into `SpeciesRegistry`. |
| FR-CIV-EMERGENCE-005 | Reproduction: agents whose fitness exceeds `REPRODUCTION_THRESHOLD` produce offspring via `recombine()` + `mutate()`; offspring inherit parents' material region as home zone. |
| FR-CIV-EMERGENCE-006 | Agent death: agents whose cumulative `Needs` deprivation exceeds `DEATH_THRESHOLD` are despawned; event logged. |
| FR-CIV-EMERGENCE-010 | Kinship cluster: parent–offspring links form a kinship graph; connected components with size ≥ `MIN_CLAN_SIZE` are tagged as a `SocialCluster` of kind `Kinship`. |
| FR-CIV-EMERGENCE-011 | Cultural cluster: `civ-diffusion` S-curve drives `culture_affinity` propagation over voxel adjacency graph; agents whose mutual affinity exceeds threshold are grouped into a `SocialCluster` of kind `Cultural`. |
| FR-CIV-EMERGENCE-012 | Territorial cluster: agents that repeatedly occupy cells within a common resource zone (determined by material CA spatial analysis) are grouped into a `SocialCluster` of kind `Territorial`. |
| FR-CIV-EMERGENCE-013 | Cluster event log: all cluster creation, merge, and split events are logged to the event stream with tick number, cluster kind, and member count; replayable deterministically. |

**FR-CIV-RENDER-001 through FR-CIV-RENDER-002:**

| Proposed FR ID | Summary |
|---|---|
| FR-CIV-RENDER-001 | Bevy chunk streamer loads and unloads chunks within a 3-chunk camera radius; no render-thread stalls (NFR-CIV-SCALE-002). |
| FR-CIV-RENDER-002 | Translucent material pass: liquid and gas cells rendered with alpha-blended geometry; solid cells rendered opaque first. |

**FR-CIV-UI-001 through FR-CIV-UI-003:**

| Proposed FR ID | Summary |
|---|---|
| FR-CIV-UI-001 | Material brush tool: player selects material from palette and paints or erases cells in a configurable radius; writes go through CA command queue (deterministic). |
| FR-CIV-UI-002 | Condition overlay: toggleable heatmap overlays for temperature, material density, life density, and cluster membership rendered as transparent screen-space passes. |
| FR-CIV-UI-003 | Emergence notification: HUD toasts when a speciation event, cluster formation, or proto-life event fires; dismissible; linked to event log. |

### 4.4 NFR Extensions Required

The following existing NFRs need their scope updated or new NFRs added to cover the CA paradigm:

| Action | NFR ID | Change |
|---|---|---|
| Extend scope | NFR-CIV-DET-003 | Add `crates/material-ca` state-mutation paths to the float-prohibition and fixed-point enforcement scope. |
| Extend scope | NFR-CIV-PERF-003 | Re-baseline tick budget benchmarks to include CA physics update phase (target: CA update for a 64³ active region ≤ 5 ms at P99 on reference hardware). |
| New | NFR-CIV-PERF-008 | CA physics update for a 128³ active region SHALL complete within 20 ms at P99 on the RTX 3090 Ti host CPU. |
| New | NFR-CIV-SCALE-004 | The voxel world SHALL support a minimum of 1,024³ cells in the SVO structure without exceeding the NFR-CIV-PERF-006 memory ceiling. |
| Retire | NFR-CIV-ACC-001 | Colorblind-safe *faction* palette is replaced by colorblind-safe *material* palette; rename and re-scope accordingly. |

---

## 5. Risks

### R1 — Determinism of CA at Scale (High Impact, Medium Probability)

**Risk:** Cellular automaton rules that use neighbourhood scans are inherently order-sensitive. If the scan order of active cells is not canonical (e.g., using a `HashMap` rather than a `BTreeMap` for active-cell tracking), two runs from the same seed can diverge because HashMap iteration order is non-deterministic in Rust.

**Constraint:** FR-CIV-VOXEL-022 mandates canonical scan order (BTreeMap key order). The CA dirty-chunk queue in `phenotype-voxel` already uses `WriteSeq`-ordered drain (`FR-CIV-VOXEL-002` is implemented and passes). Material state transitions must use only integer arithmetic consistent with NFR-CIV-DET-003. The existing CI gate `tests/determinism_replay_same_platform` (NFR-CIV-DET-001) must be extended to cover CA worlds.

**Mitigation:** Extend `determinism_replay_same_platform` to run a 50-tick CA world; add cross-platform matrix job for CA state hashes (NFR-CIV-DET-002 scope extension).

### R2 — Performance: CA at Simulation Scale (High Impact, High Probability)

**Risk:** A naive CA that updates all voxels each tick cannot scale to a 1,024³ world. NFR-CIV-PERF-003 and NFR-CIV-PERF-004 are currently defined for the ECS agent tick, not for CA physics. A per-tick full-world scan is O(n³) which is clearly infeasible.

**Constraint:** NFR-CIV-PERF-008 (proposed) sets a 20 ms budget for a 128³ active region. The architecture must use **active-cell tracking**: only cells that changed last tick (already supported by `DirtyChunkEvent` in phenotype-voxel) and their neighbours are re-evaluated. This reduces average per-tick cost to O(active surface area), not O(volume).

**Mitigation:** Implement material CA on top of the existing `drain_dirty()` + `DirtyChunkEvent` queue. P-VM-1 must benchmark active-cell throughput before P-VM-2 world gen is attempted. If active-cell CA still exceeds budget, fall back to chunk-level CA (coarser update with fluid interpolation between frames).

### R3 — Keeping the Current Build Playable During Migration (Medium Impact, Medium Probability)

**Risk:** The migration replaces the core world representation (`TerrainMap` → `VoxelWorld`). If done in a single cut-over, the build will be broken for a long period, blocking other development streams (economy, protocol, research API) that do not need the voxel substrate.

**Mitigation strategy:** Use **parallel world representations** during P-VM-1 and P-VM-2. The engine's `WorldState` currently holds `terrain: TerrainMap`. Add `voxel_world: Option<VoxelWorld<MaterialId>>` alongside it. Phases that do not yet use voxels continue to run against `TerrainMap`. The `option` becomes `Some(...)` once P-VM-2 is complete and passes its own CI gate. `TerrainMap` is deprecated (annotated with upstream issue reference) and removed only after P-VM-3 renderer is green.

This preserves the Phase 0–3 economy, protocol, and research-API work streams (PLAN.md existing phases) and avoids a big-bang cut-over.

### R4 — Species Hardcoding Regression (Low Impact, Low Probability)

**Risk:** The civ-008 spec (FR-CIV-BIO-001) explicitly names a `(Human, Dwarf, Elf, Goblin, ...)` enum. An implementer following the spec without reading this document could re-introduce a hardcoded taxonomy.

**Mitigation:** The spec must be updated in `agileplus-specs/civ-008-genetics-species/spec.md` to strike FR-CIV-BIO-001 and replace it with a reference to FR-CIV-EMERGENCE-004 (species registry) and the algorithmic `Species` record from `crates/genetics`. This is a documentation-only change with no code impact.

### R5 — Faction Field Removal Breaking In-Progress Agent Work (Low Impact, Low Probability)

**Risk:** `Civilian::faction: u32` is referenced in `crates/agents` spawn functions (`spawn_civilian_at`, `spawn_child_near`, `spawn_many`). Removing it breaks callers.

**Mitigation:** P-VM-5 (emergent social grouping) is the phase that removes the field. By P-VM-5, the agent spawn API is already being reworked to fit the life-emergence bootstrap model (P-VM-4). The removal is therefore a natural consequence of P-VM-4/5 implementation order, not an out-of-band breaking change. Annotate `faction: u32` with `#[deprecated(since = "0.0.0", note = "replaced by emergent SocialCluster; see FR-CIV-EMERGENCE-010")]` during P-VM-4 and physically remove it in P-VM-5.

---

## Cross-Project Reuse Opportunities

| Candidate | Current Location | Target Shared Module | Impacted Repos | Notes |
|---|---|---|---|---|
| `MaterialDef` palette + CA rule table | New `crates/material-ca` | `phenotype-voxel` kernel (upstream PR) | Civis, any future phenotype-org 3D repo | Material definitions are engine-agnostic; belong in the shared voxel kernel |
| `SocialCluster` formation algorithm | New `crates/social-emergence` (Civis) | `phenotype-org/social-substrate` (proposed) | Civis, future phenotype-org society sims | Kinship + cultural cluster algorithms are reusable across any agent simulation |
| `SpeciesRegistry` (runtime discovery) | `crates/genetics` (extend) | Already in phenotype-org/civis-game | Any fork consuming `civ-genetics` | `Species` record already exists; registry wrapper is the only addition |

---

**Document History**

| Version | Date | Change |
|---|---|---|
| 1.0 | 2026-05-29 | Initial authoritative vision document reconciling voxel-emergent paradigm with existing seed plans. |
