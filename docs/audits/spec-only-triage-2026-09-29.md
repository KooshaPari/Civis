# SPEC-ONLY Triage — Adjudicated Ledger (205 IDs)

**Date:** 2026-09-29  
**Scope:** every ID previously classified `SPEC-ONLY` in the FR/NFR coverage audit  
**Method:** nine family-preserving batches, each adjudicated against primary source text, then merged under a validator that rejects missing, duplicate, unknown, or evidence-free verdicts

## Verdict distribution

| Verdict | Count | Meaning |
|---|---:|---|
| `REAL-GAP` | 121 | Real, authored requirement with a machine-checkable acceptance criterion and no satisfying code |
| `IMPLEMENTED-ELSEWHERE` | 14 | Requirement is satisfied by real code, but the ID is not bound to it |
| `NAMESPACE-COLLISION` | 7 | Two live IDs claim the same subject; a human must choose the canonical one |
| `MISFILED-TEMPLATE` | 23 | Real obligation filed into an auto-generated empty `SPEC-TEMPLATE` stub |
| `MISFILED-REPORTING` | 13 | ID exists only as a row in a rollup/index, never as a requirement |
| `MISFILED-DESIGN` | 12 | ID appears only as a design cross-reference, with no per-ID acceptance criterion |
| `SYNTHETIC` | 15 | Auto-numbered by a batch expansion; never authored as a requirement |
| **Total** | **205** | |

Confidence: **178** high, **26** medium, **1** low

## Adjudication standard

One rule was applied to all 205 IDs:

> An ID is **REAL-GAP** when an authored source defines it with a machine-checkable acceptance criterion **and** no implementing code satisfies it. The source's folder (`docs/design/` vs `docs/specs/`) is **not** a demotion criterion.

This rule was arrived at after a coordinator spot-check of `docs/design/tech-engineering.md`, which is a design doc with **zero** `SHALL`/`MUST` keywords yet carries seven acceptance-criteria sections and 47 `AC-*` lines, and supports 16 REAL-GAPs. Two agents had demoted `docs/design/species-sentience.md` and `docs/design/psyche-social.md` on the weaker ground of "design doc, no RFC-2119 modals". Applying that test to one design family and not another was an inconsistency, so **45 IDs were promoted** back to REAL-GAP.

The converse also holds and was verified: the GEO family stays demoted because its only appearance is in `CIV-0300` section 12.2, a table explicitly titled *FR Traceability* that maps IDs **to** UI components. `CIV-0300` contains zero `SHALL`, `MUST`, `SHOULD`, `MAY` or `REQUIRED` across its 2,600+ lines. A back-reference is not a definition.

## Batch results

| Batch | Theme | IDs |
|---|---|---:|
| `b1_species_design` | Species & sentience | 29 |
| `b2_tech_design` | Tech & era progression | 19 |
| `b3_psyche_synth` | Psyche, social & mods | 18 |
| `b4_emergence_reporting` | Emergence & reporting | 15 |
| `b5_asset_social_econ` | Assets, social & economy | 15 |
| `b6_geo_web_audio` | Geo, web & audio | 18 |
| `b7_ux_actionable` | UX / accessibility | 22 |
| `b8_nfr_all` | Non-functional | 50 |
| `b9_save_and_misc` | Save/load & misc | 19 |

## Findings that change how the audit must be read

### 1. `FUNCTIONAL_REQUIREMENTS.md` is invisible to the coverage scanner

This file sits at the **repo root**, outside `docs/`, and carries **177** `SHALL` statements including `FR-CIV-SPECIES-012..017` and `FR-CIV-PSYCHE-004/007/008`. Two agents independently reported that their batches' "synthetic" and "design-only" labels were wrong for this reason. Any scanner that walks only `docs/` will keep misclassifying this family.

Its status line reads **Draft**, tracing to `PRD.md` v1.0 which is **APPROVED** — while `PRD.md` itself contains **0** `SHALL`/`MUST`. Authority order is therefore not self-evident and needs a human ruling.

### 2. The detector counts dead substrate as coverage

`crates/hud/src/accessibility.rs` defines `PaletteMode`, `HighContrastTheme`, `KeybindRegistry` and `palette_pair_distinguishable`, and tags them `FR-CIV-ACCESS-010`, `FR-CIV-ACCESS-020`, `NFR-CIV-ACC-001..004`. Coordinator-verified: every reference to those symbols is inside that one file (definition, re-export in `lib.rs:11`, and its own `#[cfg(test)]` assertions). **No client or renderer consumes them.** The mandated `Monochrome` palette has **zero** code hits anywhere. Tag-on-definition is being read as implemented.

### 3. A test suite named by the spec does not exist

All 12 `FR-CIV-ASSET-*` requirements name a concrete test. `git ls-files tests/` returns exactly one file: `tests/game-e2e/README.md`. All seven test trees in spec §12.1 are absent. Each of the 12 carries a MISSING-TEST note.

### 4. The 155-ID emergence block is range-manufactured

`FR-CIV-EMERGENCE-100..254` is contiguous, split into 15 per-crate slices whose widths sum to the stated 155. The defining rows existed at `1d9854d5` and were **deleted in `f313ec5c`** as mislabelled, pointing at crates such as `civ-linguabridge` that do not exist. The only surviving artifact is a row-counting index at `docs/traceability/emergent-systems-tracelinks.md:150-164`.

`tracelinks.md:167` claims the 11-systems × 30-couplings matrix "promotes each of these 158 dormant IDs to covered". That claim is **false in this tree** — and it is the recorded reason these IDs were deferred rather than triaged, so it is likely load-bearing for other deferrals too.

### 5. Requirements scan only Rust, so YAML/YAML-adjacent work reads as absent

`FR-CIV-BEVY-021` is implemented and **self-tagged** in `.github/workflows/civis-3d-live-smoke.yml:10`. The scanner missed it because it is not Rust. The doc is stale in the other direction: it says *path-filtered CI*, but the workflow trigger is `workflow_dispatch:` only, with an explicit comment that it "never triggers automatically".

### 6. `era.rs` actively contradicts `tech-engineering.md`

`FR-CIV-TECH-017` requires that adoption collapse can lower `era_label` and emit a regression. `crates/engine/src/era.rs:16` states eras are "strictly ordered so advances (**never regressions**)", repeated at `:529`. Separately `FR-CIV-TECH-005` requires that **no global tech-level field exists**; `crates/engine/src/tech.rs:20` declares `pub tech_level: u32` and `era.rs:52` gates production on it. These are not omissions — the code asserts the opposite invariant.

### 7. The `civlab` batch-analysis product does not exist

`USER_SPEC.md` section 4 specifies run lists, run headers, branch-from-tick, multi-run comparison and export bundles. Repo-wide, `RunRecord` has 0 hits, `scenario_label` has 0 **code** hits, and the `civlab` binary does not exist — only `crates/civlab-sdk`, a mod SDK with no run management. All 22 UX IDs depend on that run model, so they are product-scope gaps rather than missing widgets. If `civlab` is out of scope, those 22 should be deferred rather than filed as gaps.

## Open items requiring a human decision

1. **Is `FUNCTIONAL_REQUIREMENTS.md` authoritative?** It is Draft, it traces to an APPROVED `PRD.md` that has no `SHALL` of its own, and it is invisible to the scanner. This single ruling changes the classification of at least 20 IDs.
2. **Do the 155 `FR-CIV-EMERGENCE-*` IDs exist at all?** Recommended: retire the counting index and correct the false coverage claim at `tracelinks.md:167`.
3. **Are `docs/design/*` catalogues authoritative?** This audit answered *yes, per-ID acceptance criteria count*, promoting 45 IDs. If the intent was that only `docs/specs/` binds, those 45 revert to MISFILED-DESIGN.
4. **7 namespace collisions** need a canonical owner each: `FR-CIV-SOCIAL-001` vs `-INSTITUTIONS`, `FR-CIV-SOCIAL-002` vs `-IDEOLOGY`, `FR-CIV-ECON-002-JOULE` vs `FR-CIV-ECON-002`, `FR-CIV-GODOT-UX-000` vs `FR-CIV-UX-000`, `FR-CIV-RESEARCH-004-REPLAY` vs `FR-CIV-RESEARCH-004`, `FR-CIV-0700` vs spec `CIV-0700`, `NFR-C-02` vs the `NFR-CIV-*` scheme.
5. **Two source claims are factually stale and actively cause misclassification:** `civ-021-recovered-requirements/spec.md:225` asserts "crates/social does not exist" (it is a workspace member), and `docs/traceability/fr-web-matrix.md:27` marks `FR-CIV-WEB-004` implemented on `set_speed` alone while `sim.set_policy` and `sim.reset` are absent.

## Detector fix applied 2026-10-01

Finding 2 above (the detector counts dead substrate as coverage) was traced to a
concrete line: `_gather_ids.py` promoted any ID reference found inside a
`#[cfg(test)]` block in a `src/*.rs` file into `in_code`. `classify()` in
`gen-fr-audit.py` then saw `has_code and has_test` and reported `COVERED`, so a
self-assertion counted as an implementation.

The gatherer now records a `self_test_only` flag, and a new `SELF-TEST-ONLY`
status keeps those rows out of `COVERED`. **220 IDs moved**, so `COVERED` drops
from 1057 to 837.

**What this fix does NOT fix, stated plainly.** It only catches IDs whose code
evidence is *entirely* self-test. The `crates/hud` accessibility case in
finding 2 is still reported `COVERED`, because those IDs also carry a reference
on their definition line (`accessibility.rs:15`, `:40`, `:225`) and the
`lib.rs:11` re-export. Separating "implemented but unconsumed" from "implemented
and wired up" requires call-graph analysis, which this scanner does not attempt.

A regression test was written asserting those four accessibility IDs are not
`COVERED`. **It fails**, because they genuinely are. It was reverted rather than
weakened, and the defect is recorded here instead. Closing it needs either
call-graph analysis or a manual consumer annotation on symbols like
`PaletteMode`.

### Fix 2: the audit was citing itself as evidence

A second, separate defect surfaced while chasing fix 1. `scripts/traceability/**`
holds the audit tooling itself, and it was **not** in `SELF_REF_DIRS`. A literal
`FR-CIV-ACCESS-010` inside a test fixture or a docstring there was recorded as a
`code`/`test` reference, so the tooling counted as proof that the requirement was
implemented. **24 IDs** carried such a reference.

Two had tooling references as their *only* evidence and moved
`COVERED -> TEST-NO-CODE-REF`:

| ID | Real evidence after the fix |
|---|---|
| `FR-CIV-3D` | tests only (`crates/engine/tests/fr_fr_civ_3d_00*.rs`), no source file carries the ID |
| `NFR-CIV-PERF-001` | tests only (`crates/engine/tests/fr_engine_hash_lod_perf_tests.rs`) |

Net: `COVERED` 837 → 835, `TEST-NO-CODE-REF` 154 → 156. Nothing else moved.

The exclusion loses no real evidence: that directory is Python and shell only, no
product code.

### Hypothesis tested and rejected: "no in-workspace consumer"

The obvious way to catch the `crates/hud` dead substrate is a dependency signal:
`civ-hud` is a workspace member that **no** crate declares as a dependency. That
looked like a clean 115-ID signal, and it is wrong.

An earlier hand-rolled Cargo parser was also wrong in the opposite direction (it
reported 50 of 51 packages as unconsumed, because this workspace has no
`[workspace.dependencies]` table, so deps are declared per-crate). The ground
truth came from `cargo metadata`.

Checked against `cargo metadata`, 16 packages have no in-workspace consumer. But a
single-pass text scan shows most of them are consumed from outside the crate graph:
`civ-social` 29 hits, `civ-watch` 85, `emergence-oracle` 102, `civ-traffic` 239,
`asset-pipeline` 270, `civis-mcp` 83, `civlab-sdk` 37. Flagging all 115 IDs on
this basis would have produced mass false positives.

`civ-hud` is the one case with genuinely no consumer anywhere: its only external
mentions are 7 files, all documentation or audit scripts
(`docs/traceability/fr-civ-hud-00*/`), and `civ_hud` appears in code only inside
`crates/hud` itself. So the `crates/hud` accessibility family is real dead
substrate, confirmed by hand, but **not** by a generalizable detector. Treating
`civ-social` or `civ-watch` the same way would be wrong.

Closing the general case needs call-graph analysis at symbol level, which this
scanner does not attempt.

### Test verification

Each fix was confirmed to actually detect its defect: reverting the
`SELF_REF_DIRS` entry makes 2 of the new tests fail
(`test_gather_excludes_the_audit_tooling_directory`,
`test_is_self_ref_rejects_tooling_paths`). Suite is 38 passing, up from 34.

---

## Inventory-invisible IDs (recorded separately, NOT added to the ledger)

A source-vs-inventory scan found 156 source-tagged IDs absent from `docs/audits/_id_inventory_v3.json`. Two leaf-shaped examples were verified by the coordinator and are real work that the ledger cannot currently see:

- `FR-PHYS-substrate-000..007` — real code and tests
- `FR-CIV-SPECIATION` — real code and two passing tests at `crates/species/src/speciation.rs:126-209`

These are **not** counted in the 205 above. They are a separate reconciliation task: either the inventory generator must scan the repo root and non-`docs/` sources, or these IDs must be registered.

---

## Appendix: full ledger

### `REAL-GAP` (121)

| ID | Batch | Conf | Source | Evidence |
|---|---|---|---|---|
| `FR-CIV-ASSET-002` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2437-2443` | FR-CIV-ASSET-002 - resvg Rasterization Correctness > The resvg renderer SHALL produce pixel-identical output for identical SVG inputs across all supported CI platforms (Linux x86_64, macOS a |
| `FR-CIV-ASSET-008` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2497-2503` | FR-CIV-ASSET-008 - Manifest Completeness > The `asset_manifest.json` SHALL contain an entry for every sprite packed into every atlas. No sprite that exists in an atlas JSON SHALL be absent f |
| `FR-CIV-ASSET-009` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2507-2513` | FR-CIV-ASSET-009 - Manifest JSON Schema Conformance > The `asset_manifest.json` SHALL conform to the JSON Schema defined in `assets/schemas/asset_manifest_v1.json`. Validation SHALL run as t |
| `FR-CIV-ASSET-010` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2517-2523` | FR-CIV-ASSET-010 - Build-Time Reproducibility (Content Hash Stability) > Running the full pipeline twice with identical inputs ... SHALL produce identical `content_hash` values for every ass |
| `FR-CIV-ASSET-011` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2529-2535` | FR-CIV-ASSET-011 - Build Performance: Render Time > The full baseline sprite render batch (all 102 sprites) SHALL complete in under 30 seconds on a 4-core x86_64 Linux CI runner (Ubuntu 22.0 |
| `FR-CIV-ASSET-012` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2539-2545` | FR-CIV-ASSET-012 - Atlas Load Time > All three atlas files (terrain, buildings, citizens) SHALL load and be available for texture lookup within 500 ms of `SpriteManager.init()` being called  |
| `FR-CIV-ASSET-013` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2549-2555` | FR-CIV-ASSET-013 - VRAM Budget > The total VRAM consumed by all three loaded atlas textures SHALL NOT exceed 20 MB. This is measured as the sum of `width x height x 4` bytes for each atlas P |
| `FR-CIV-ASSET-014` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2559-2565` | FR-CIV-ASSET-014 - No Runtime SVG Parsing > The web bundle (Vite output in `web/dist/`) SHALL NOT contain `resvg`, `svg.js`, or any SVG parser/renderer library. Background removal, palette q |
| `FR-CIV-ASSET-015` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2569-2575` | FR-CIV-ASSET-015 - Atlas Cache-Control Headers > Atlas files served from `web/public/atlases/` SHALL be served with `Cache-Control: public, max-age=31536000, immutable` HTTP headers. The `as |
| `FR-CIV-ASSET-017` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2589-2595` | FR-CIV-ASSET-017 - Sprite Pool Pre-Warm > The `SpritePool` SHALL pre-allocate exactly 256 `Sprite` instances during `SpriteManager.init()`. No Sprite objects SHALL be created after initializ |
| `FR-CIV-ASSET-019` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2609-2615` | FR-CIV-ASSET-019 - SDXL Seed Determinism > When the SDXL enhancement pass is enabled, running the pipeline twice with the same `asset_parameters.yaml` and same `pipeline_hash` SHALL produce  |
| `FR-CIV-ASSET-020` | b5 | high | `docs/specs/CIV-0600-2d-asset-pipeline-spec.md:2619-2625` | FR-CIV-ASSET-020 - Template Validation Pre-Commit > All `.svg.j2` template files MUST pass SVG validity, layer structure, font reference, and variable coverage checks before being committed. |
| `FR-CIV-AUDIO-009` | b6 | high | `docs/design/audio-direction.md:302` | \| **FR-CIV-AUDIO-009** \| CC0-only sourcing with committed provenance manifest \| every shipped clip is CC0/PD and listed in `assets/audio/CREDITS.md` with source + license \| |
| `FR-CIV-AUDIO-010` | b6 | high | `docs/design/audio-direction.md:303` | \| **FR-CIV-AUDIO-010** \| Graceful silence invariant preserved \| any missing clip warns-and-silences; app stays green/playable with zero audio files present \| |
| `FR-CIV-AUDIO-011` | b6 | high | `docs/design/audio-direction.md:304` | \| **FR-CIV-AUDIO-011** \| Mix + cadence tunables exposed as resources, not consts \| `AudioMix` (bus gains) + sampler/mood cadences live in editable resources for tuning/testing \| |
| `FR-CIV-AUDIO-012` | b6 | high | `docs/design/audio-direction.md:305` | \| **FR-CIV-AUDIO-012** \| (Phase 2) Optional positional SFX \| world-positioned events carry an optional coord; pan/attenuate by camera distance as an additive upgrade \| |
| `FR-CIV-EMERGENCE-RELIGION-2` | b2 | high | `docs/design/RELIGION_EMERGENCE.md:456` | /// FR-CIV-EMERGENCE-RELIGION-2 — religion → law compliance hook. fn update_law_compliance(world: &hecs::World, religions: &[ReligiousProfile]) {     ... if profile.monitoring < LAW_MONITORI |
| `FR-CIV-PSYCHE-011` | b3 | high | `docs/design/psyche-social.md:191` | ### 4.1 Utility-AI / daily path (`daily_path.rs`) — **FR-CIV-PSYCHE-011** |
| `FR-CIV-PSYCHE-021` | b3 | high | `docs/design/psyche-social.md:209` | ### 4.4 Culture feedback (`culture.rs`) — **FR-CIV-PSYCHE-021** |
| `FR-CIV-PSYCHE-024` | b3 | high | `docs/design/psyche-social.md:233` | ### 6.1 Agent inspector — "Mind" panel (**FR-CIV-PSYCHE-024**) |
| `FR-CIV-PSYCHE-032` | b3 | high | `docs/design/psyche-social.md:203` | ### 4.2 Cluster membership (`cluster.rs`) — **FR-CIV-PSYCHE-032** |
| `FR-CIV-PSYCHE-033` | b3 | high | `docs/design/psyche-social.md:206` | ### 4.3 Diplomacy (`diplomacy.rs`) — **FR-CIV-PSYCHE-033** |
| `FR-CIV-PSYCHE-034` | b3 | high | `docs/design/psyche-social.md:241` | ### 6.2 Relationships panel + graph (**FR-CIV-PSYCHE-034**) |
| `FR-CIV-PSYCHE-035` | b3 | high | `docs/design/psyche-social.md:245` | - **Social ties** overlay (Gizmo): draw strong edges between nearby agents — friendship networks become visible terrain. **FR-CIV-PSYCHE-035**. |
| `FR-CIV-PSYCHE-036` | b3 | high | `docs/design/psyche-social.md:246` | - **Mood / contentment** overlay (EntityTint): tint agents by mood valence — see a happy district vs a miserable one. **FR-CIV-PSYCHE-036**. |
| `FR-CIV-PSYCHE-037` | b3 | high | `docs/design/psyche-social.md:247` | - **Belief field** overlay (LatticeRecolor over belief centroid): cultural/ideological regions and their boundaries, derived from per-agent beliefs (the "where do worldviews split" map). **F |
| `FR-CIV-SPECIES-012` | b9 | high | `FUNCTIONAL_REQUIREMENTS.md:452` | - **FR-CIV-SPECIES-012** — Accessory child entities SHALL attach when gene thresholds cross configured gates (canonical presets provide default thresholds). |
| `FR-CIV-SPECIES-013` | b9 | high | `FUNCTIONAL_REQUIREMENTS.md:453` | - **FR-CIV-SPECIES-013** — Canonical mode SHALL load named race seeds (human + fantasy exemplars) as preset genome + base mesh + divergence dial 0..1, not scripted outcome paths. |
| `FR-CIV-SPECIES-014` | b9 | high | `FUNCTIONAL_REQUIREMENTS.md:454` | - **FR-CIV-SPECIES-014** — Primitive mode SHALL expose `spawn_organism { genome, cradle_state, … }` with no species enum table. |
| `FR-CIV-SPECIES-015` | b9 | medium | `FUNCTIONAL_REQUIREMENTS.md:455` | - **FR-CIV-SPECIES-015** — Seed base meshes SHALL ship as MakeHuman CC0 exports with offline Mixamo rig; SMPL basis remains an optional upgrade path only. |
| `FR-CIV-SPECIES-016` | b9 | high | `FUNCTIONAL_REQUIREMENTS.md:456` | - **FR-CIV-SPECIES-016** — Clients (Bevy first) SHALL display morphed agents at 60 FPS for ≥100 visible instances on reference hardware. |
| `FR-CIV-SPECIES-017` | b9 | high | `FUNCTIONAL_REQUIREMENTS.md:457` | - **FR-CIV-SPECIES-017** — Same `Dna` input SHALL yield identical morph weights across render clients (pure `express` mapping; sim RNG elsewhere allowed). |
| `FR-CIV-SPECIES-100` | b1 | high | `docs/design/species-sentience.md:73` | - **FR-CIV-SPECIES-100** — There is **no creature spawn table**. The only authored inputs to first life are (a) material/energy/phase **laws** and (b) the abiogenesis suitability **factor co |
| `FR-CIV-SPECIES-101` | b1 | high | `docs/design/species-sentience.md:74` | - **FR-CIV-SPECIES-101** — `A` is `0` whenever any necessity factor is `0` (no solvent ⇒ no life; no energy gradient ⇒ no life), and strictly increases as factors and dwell improve, holding  |
| `FR-CIV-SPECIES-102` | b1 | high | `docs/design/species-sentience.md:75` | - **FR-CIV-SPECIES-102** — Abiogenesis fires probabilistically; over many ticks in a suitable cradle at least one protocell emerges, and in an unsuitable cell (`A=0`) none ever do. |
| `FR-CIV-SPECIES-103` | b1 | high | `docs/design/species-sentience.md:76` | - **FR-CIV-SPECIES-103** — A newborn protocell's seed genome is **biased by local conditions** (measurably correlated with the cradle's energy axis / ambient properties), not drawn from a un |
| `FR-CIV-SPECIES-104` | b1 | high | `docs/design/species-sentience.md:77` | - **FR-CIV-SPECIES-104** — The chosen `DnaClass`/substrate archetype is a function of the cradle's chemistry only; the same chemistry deterministically selects the same archetype, while diff |
| `FR-CIV-SPECIES-105` | b1 | high | `docs/design/species-sentience.md:78` | - **FR-CIV-SPECIES-105** — Abiogenesis is **rare and dwell-gated**: a transient suitable flicker (below the dwell requirement) does not produce life; sustained suitability does. |
| `FR-CIV-SPECIES-201` | b1 | high | `docs/design/species-sentience.md:99` | - **FR-CIV-SPECIES-201** — The byte→trait layout is **declared by `DnaClass`** (named gene regions), not a single hardcoded offset constant. Adding a region is data; existing genomes remain  |
| `FR-CIV-SPECIES-202` | b1 | high | `docs/design/species-sentience.md:100` | - **FR-CIV-SPECIES-202** — A genome can express a **non-humanoid plan** (e.g. `leg_count ≠ 2`, `arm_count = 0`, radial symmetry, sessile) with no special-casing — the renderer/agent layer co |
| `FR-CIV-SPECIES-203` | b1 | high | `docs/design/species-sentience.md:101` | - **FR-CIV-SPECIES-203** — Metabolism traits expressed at abiogenesis are **consistent with the cradle archetype** (FR-CIV-SPECIES-104); selection may later shift them, but a fresh lineage i |
| `FR-CIV-SPECIES-204` | b1 | high | `docs/design/species-sentience.md:102` | - **FR-CIV-SPECIES-204** — A single-byte mutation affects **only its region's** trait(s), leaving all others equal (extends existing `…-008`). |
| `FR-CIV-SPECIES-205` | b1 | high | `docs/design/species-sentience.md:103` | - **FR-CIV-SPECIES-205** — Behavior/cognition bytes feed both `civ-agents` (drives) and the cognition score (§4) from the **same genome bytes** — no parallel hidden stat. |
| `FR-CIV-SPECIES-300` | b1 | high | `docs/design/species-sentience.md:120` | - **FR-CIV-SPECIES-300** — A new `Species` is issued **iff** a child's Hamming distance to its species reference exceeds the class `speciation_threshold` (extends existing `…-010`); below th |
| `FR-CIV-SPECIES-301` | b1 | high | `docs/design/species-sentience.md:121` | - **FR-CIV-SPECIES-301** — `speciation_distance` is symmetric and normalized to `[0,1]` (preserves existing `…-011`). |
| `FR-CIV-SPECIES-302` | b1 | high | `docs/design/species-sentience.md:122` | - **FR-CIV-SPECIES-302** — Each `Species` record links to its **parent species** (genealogy is a DAG/tree), enabling the inspector and legends engine to render an evolutionary tree. Founder  |
| `FR-CIV-SPECIES-303` | b1 | high | `docs/design/species-sentience.md:123` | - **FR-CIV-SPECIES-303** — Speciation never compares across `DnaClass` boundaries; archetypes form disjoint species forests. |
| `FR-CIV-SPECIES-304` | b1 | high | `docs/design/species-sentience.md:124` | - **FR-CIV-SPECIES-304** — Species issuance is **stable & idempotent per birth event**: re-evaluating the same birth does not mint duplicate species; IDs are monotonic and never reused. |
| `FR-CIV-SPECIES-400` | b1 | high | `docs/design/species-sentience.md:167` | - **FR-CIV-SPECIES-400** — `cognition_score` reads **only** declared cognition byte slots and is normalized to `[0,1]` for non-negative weights (preserves existing sentience-module tests). |
| `FR-CIV-SPECIES-401` | b1 | high | `docs/design/species-sentience.md:168` | - **FR-CIV-SPECIES-401** — Sentience is a **threshold latch**: a lineage is sentient iff its representative cognition score ≥ `minimum_cognition`; below it the lineage remains non-sentient,  |
| `FR-CIV-SPECIES-402` | b1 | high | `docs/design/species-sentience.md:169` | - **FR-CIV-SPECIES-402** — Cognition **accumulates via the genome**: the score can only rise across generations by mutation/recombination/selection on the cognition byte slots, never by dire |
| `FR-CIV-SPECIES-403` | b1 | high | `docs/design/species-sentience.md:170` | - **FR-CIV-SPECIES-403** — The unlock ladder is **strictly ordered and gated**: a lineage cannot register *culture* without *tool-use*, nor *language* without *culture*, nor *civilization* w |
| `FR-CIV-SPECIES-404` | b1 | high | `docs/design/species-sentience.md:171` | - **FR-CIV-SPECIES-404** — Each rung, when crossed, **unlocks an emergent capability** consumed by downstream layers (learning→agents, tool-use→tools, culture→cultural transmission, language |
| `FR-CIV-SPECIES-405` | b1 | high | `docs/design/species-sentience.md:172` | - **FR-CIV-SPECIES-405** — Rungs are reachable by **multiple cognitive routes / body plans**: two lineages with disjoint dominant traits but sufficient blended cognition both qualify; humano |
| `FR-CIV-SPECIES-406` | b1 | high | `docs/design/species-sentience.md:173` | - **FR-CIV-SPECIES-406** — A lineage may cross lower rungs and **stall** below higher ones indefinitely (partial cognition), and may regress if selection erodes the relevant traits (the latc |
| `FR-CIV-SPECIES-500` | b1 | high | `docs/design/species-sentience.md:191` | - **FR-CIV-SPECIES-500** — Selecting a creature shows its raw genome **labeled by gene region** (not an opaque byte blob), including which bytes drive cognition. |
| `FR-CIV-SPECIES-501` | b1 | high | `docs/design/species-sentience.md:192` | - **FR-CIV-SPECIES-501** — The inspector shows the creature's **species and its genealogy** (parent/child species via FR-CIV-SPECIES-302), and its substrate archetype/chemistry origin. |
| `FR-CIV-SPECIES-502` | b1 | high | `docs/design/species-sentience.md:193` | - **FR-CIV-SPECIES-502** — The inspector shows the **expressed phenotype** (body plan + mind plan) consistent with `express` for that genome. |
| `FR-CIV-SPECIES-503` | b1 | high | `docs/design/species-sentience.md:194` | - **FR-CIV-SPECIES-503** — The inspector shows a **cognition gauge** vs. the sentience threshold and the **lit/unlit unlock ladder** (which rungs the lineage has crossed), with per-trait cog |
| `FR-CIV-SPECIES-504` | b1 | high | `docs/design/species-sentience.md:195` | - **FR-CIV-SPECIES-504** — Speciation events and rung crossings for a creature's lineage are **queryable as historical events** (legends-engine integration), so "how this mind came to be" is |
| `FR-CIV-SPECIES-505` | b1 | high | `docs/design/species-sentience.md:196` | - **FR-CIV-SPECIES-505** — All inspector views are **read-only projections** of genome/species/cognition state; the inspector introduces no authored creature data and no hidden stats not pre |
| `FR-CIV-TECH-001` | b2 | high | `docs/design/tech-engineering.md:225` | **FR-CIV-TECH-001** \| Invention readiness is the multiplicative product of need, resource, and knowledge factors; any zero factor blocks invention. \| `readiness` unit tests \| AC-E1, AC-E5 |
| `FR-CIV-TECH-003` | b2 | medium | `docs/design/tech-engineering.md:227` | **FR-CIV-TECH-003** \| Invention requires `knowledge_fraction == 1.0` (all prerequisites known) for from-scratch invention. \| prerequisite-gate test \| AC-E2 |
| `FR-CIV-TECH-004` | b2 | high | `docs/design/tech-engineering.md:228` | **FR-CIV-TECH-004** \| `cognitive_capacity` modulates invention hazard (pre-sentient ⇒ ~0). \| hazard-scaling test \| AC-E4 |
| `FR-CIV-TECH-005` | b2 | high | `docs/design/tech-engineering.md:229` | **FR-CIV-TECH-005** \| Knowledge is per-population and partial; no global tech-level field exists. \| architecture/test \| AC-K1, AC-K2 |
| `FR-CIV-TECH-006` | b2 | high | `docs/design/tech-engineering.md:230` | **FR-CIV-TECH-006** \| Knowledge loss is representable: extinguishing the last holder + severing contact removes a technique from a region's frontier. \| knowledge-loss test \| AC-K3, AC-A4 |
| `FR-CIV-TECH-010` | b2 | medium | `docs/design/tech-engineering.md:234` | **FR-CIV-TECH-010** \| No sim tick blocks on an invention/proposal request (async, bounded queue, backpressure). \| concurrency test \| AC-P4 |
| `FR-CIV-TECH-011` | b2 | medium | `docs/design/tech-engineering.md:235` | **FR-CIV-TECH-011** \| Per-`(pop,technique)` adoption advances via `civ-diffusion`; monotone non-decreasing, saturates ≤1.0. \| diffusion-integration test \| AC-D1 |
| `FR-CIV-TECH-012` | b2 | high | `docs/design/tech-engineering.md:236` | **FR-CIV-TECH-012** \| `DiffusionParams` are modulated by emergent substrate (need ↑p; density/openness ↑q; conservatism ↓q). \| param-modulation test \| AC-D2 |
| `FR-CIV-TECH-014` | b2 | high | `docs/design/tech-engineering.md:238` | **FR-CIV-TECH-014** \| Knowledge can precede capability: a population holds a technique with `f≈0` until trade/extraction supplies inputs. \| knowledge-before-capability test \| AC-D4 |
| `FR-CIV-TECH-015` | b2 | high | `docs/design/tech-engineering.md:239` | **FR-CIV-TECH-015** \| `era_label(pop)` is a percentile read-off over adopted techniques' era-rank; no capability is gated on it. \| era-derivation test \| AC-A1, AC-A2 |
| `FR-CIV-TECH-016` | b2 | medium | `docs/design/tech-engineering.md:240` | **FR-CIV-TECH-016** \| Multiple eras coexist across contacting populations (uneven aging). \| multi-era world test \| AC-A3 |
| `FR-CIV-TECH-017` | b2 | high | `docs/design/tech-engineering.md:241` | **FR-CIV-TECH-017** \| Adoption collapse lowers `era_label`; regression is emitted to legends. \| regression test \| AC-A4 |
| `FR-CIV-TECH-018` | b2 | high | `docs/design/tech-engineering.md:242` | **FR-CIV-TECH-018** \| Era transitions (boundary crossings) are emitted to the event log, never used to unlock. \| era-event test \| AC-A1, AC-A2 |
| `FR-CIV-TECH-019` | b2 | high | `docs/design/tech-engineering.md:243` | **FR-CIV-TECH-019** \| An artifact build requires its enabling technique to be known *and* adopted (`f` ≥ floor). \| build-gate test \| AC-G1, AC-G4 |
| `FR-CIV-TECH-020` | b2 | high | `docs/design/tech-engineering.md:244` | **FR-CIV-TECH-020** \| A build consumes declared inputs from `Stocks`; unavailable inputs reject the build (mass-conserving). \| build-economy test \| AC-G2, AC-G5 |
| `FR-CIV-WEB-004` | b6 | medium | `docs/development-guide/fr-web-spectator.md:33` | \| **FR-CIV-WEB-004** \| Operator controls limited to RPC already on server: `sim.command` tick/noop, `sim.set_speed`, `sim.set_policy`, `sim.reset` (with role header when required). \| Inte |
| `FR-SAVE-008` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2807` | \| FR-SAVE-008 \| The ChaCha20Rng state SHALL be serialized to 20 u32 words plus a sub-block word count and SHALL be exactly restored on load, guaranteeing RNG stream continuity. \| MUST \|  |
| `FR-SAVE-011` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2810` | \| FR-SAVE-011 \| WASM mods that implement the `ModStateSave` trait SHALL have their state included in every save. On load, known mods SHALL have their state restored before tick N+1 execute |
| `FR-SAVE-012` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2811` | \| FR-SAVE-012 \| Unknown mod IDs present in a save but not loaded in the current engine instance SHALL produce a warning log entry and be skipped without failing the load operation. \| MUST |
| `FR-SAVE-013` | b9 | medium | `docs/specs/CIV-1000-save-load-persistence-spec.md:2812` | \| FR-SAVE-013 \| The save format version SHALL be stored in the binary header. The engine SHALL apply migration functions to bring saves from versions N-2 through N-1 to the current format  |
| `FR-SAVE-016` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2815` | \| FR-SAVE-016 \| QuickSave latency SHALL be at most 50ms for a scenario with 1,000 citizens, measured as wall-clock time from save invocation to ring push completion. \| MUST \| §12.1 \| |
| `FR-SAVE-017` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2816` | \| FR-SAVE-017 \| SlotSave latency SHALL be at most 500ms for a scenario with 1,000 citizens, including serialization, zstd compression, and DB metadata write. \| MUST \| §12.1 \| |
| `FR-SAVE-018` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2817` | \| FR-SAVE-018 \| Load latency SHALL be at most 1,000ms for a scenario with 1,000 citizens, including decompression, deserialization, migration (if needed), and World restoration. \| MUST \| |
| `FR-SAVE-019` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2818` | \| FR-SAVE-019 \| Save integrity verification (hash check without full deserialization) SHALL complete in at most 200ms for any save size. \| MUST \| §6.6, §12.1 \| |
| `FR-UX-006` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:928` | FR-UX-006: Run list must display run ID, scenario label, seed, status, and tick count. The run list panel must show these fields for each run in a scannable table format. Status is one of: Q |
| `FR-UX-007` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:931` | FR-UX-007: Run header must be persistent during timeline inspection. When inspecting a run, a persistent header bar must display: run ID, scenario label, version, seed, and total tick count. |
| `FR-UX-008` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:934` | FR-UX-008: Comparison panel must display regime labels prominently. When the user assigns regime labels to runs in a comparison, those labels must appear in chart legends, axis annotations,  |
| `FR-UX-009` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:939` | FR-UX-009: Low-confidence states must be surfaced with a visible warning indicator. If any metric enters a low-confidence state ... the metric chart must display an amber warning icon and a  |
| `FR-UX-010` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:942` | FR-UX-010: Determinism violations must trigger an immediate error state. If the engine detects a determinism violation (tick hash mismatch on replay), the affected run must be marked with a  |
| `FR-UX-011` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:945` | FR-UX-011: Unstable simulation states must halt the run and surface an error. If the engine halts a run due to an unstable state (NaN, Inf, or degenerate metric value), the run status must s |
| `FR-UX-012` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:948` | FR-UX-012: Mod-active runs must display a persistent mod badge. Any run where mods are active must display a "Mods Active" badge in the run header. The badge must list the active mods with t |
| `FR-UX-013` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:951` | FR-UX-013: Configuration schema violations must block scenario commit. If the scenario config contains any schema violation, the "Commit Version" action must be disabled. The UI must display |
| `FR-UX-014` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:956` | FR-UX-014: Branch origin must be persistently visible during branch run inspection. When inspecting a branch run, the timeline must display a vertical marker at the branch tick labeled with  |
| `FR-UX-015` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:959` | FR-UX-015: Branch tree must be navigable from a visual run tree panel. A run tree panel must show the parent-child relationships between runs as an expandable tree. Trunk runs are shown at t |
| `FR-UX-016` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:962` | FR-UX-016: Rollback to a prior branch point must be one action. The user must be able to create a new branch from any tick in any completed run without navigating away from the current view. |
| `FR-UX-017` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:965` | FR-UX-017: Destructive actions must require explicit confirmation with consequences stated. Any action that would delete or overwrite simulation data must present a confirmation dialog that  |
| `FR-UX-018` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:970` | FR-UX-018: All interactive elements must meet WCAG 2.1 AA contrast requirements. Text and interactive elements must have a minimum contrast ratio of 4.5:1 against their background. Large tex |
| `FR-UX-019` | b7 | medium | `docs/models/civ-sim/USER_SPEC.md:973` | FR-UX-019: All interactive elements must be keyboard-accessible. Every action available via mouse must be accessible via keyboard. Tab order must be logical ... Focus indicators must be visi |
| `FR-UX-020` | b7 | medium | `docs/models/civ-sim/USER_SPEC.md:976` | FR-UX-020: Screen reader support for simulation state panels. All metric panels, event log entries, and timeline annotations must have appropriate ARIA labels and roles. Dynamic content upda |
| `FR-UX-021` | b7 | medium | `docs/models/civ-sim/USER_SPEC.md:979` | FR-UX-021: Color-blind safe palette required for all metric visualizations. All charts and hex map coloring must use a color-blind safe palette. Default palette must be distinguishable under |
| `FR-UX-022` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:982` | FR-UX-022: Font sizes must be user-adjustable. Base font size must be adjustable from 12px to 24px in the application settings. All layout must accommodate the full range without horizontal  |
| `FR-UX-023` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:987` | FR-UX-023: Timeline scrubber response must be under 200ms. From the moment the user releases the scrubber handle to the moment all visible views (hex map, metric charts, event log) reflect t |
| `FR-UX-024` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:990` | FR-UX-024: Hex map pan and zoom must achieve 60fps. Pan and zoom interactions on the hex map must render at 60 frames per second on target hardware (defined as: laptop with integrated GPU, 1 |
| `FR-UX-025` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:993` | FR-UX-025: Run list must load in under 500ms for up to 500 runs. The run list panel must be populated within 500ms of navigation to the runs view, for workspaces containing up to 500 runs. P |
| `FR-UX-026` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:996` | FR-UX-026: Export generation must not block the UI. Export bundle generation must run asynchronously. The UI must remain interactive during export. A progress indicator must show export stag |
| `FR-UX-027` | b7 | high | `docs/models/civ-sim/USER_SPEC.md:999` | FR-UX-027: Comparison panel must handle up to 8 simultaneous runs. The comparison panel must support up to 8 runs simultaneously without exceeding 500ms render time for metric chart updates. |
| `NFR-CIV-AI-002` | b8 | high | `docs/design/civ-ai-crate.md:49` | \| **NFR-CIV-AI-002** \| Local-first/OSS/free default; cloud is explicit opt-in only. \| SS0.1 \| Default config selects `LocalSlmProvider`; cloud gated off by default. \| |
| `NFR-CIV-PERF-003` | b8 | high | `docs/reference/non-functional-requirements.md:55` | The engine simulation tick SHALL complete all phases (policy, deterministic transition, stochastic event, market clearing, allocation) within a 10 ms wall-clock budget at a load of 200 civil |
| `NFR-CIV-PERF-005` | b8 | high | `docs/reference/non-functional-requirements.md:86` | Procedural terrain mesh generation for a 256x256 chunk SHALL complete within 50 ms on the host CPU thread, permitting background streaming without hitching the main render thread. |
| `NFR-CIV-PERF-008` | b8 | high | `docs/guides/voxel-emergent-vision-and-migration.md:171` | \| New \| NFR-CIV-PERF-008 \| CA physics update for a 128^3 active region SHALL complete within 20 ms at P99 on the RTX 3090 Ti host CPU. \| |
| `NFR-CIV-PERF-900` | b8 | high | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md:15` | Target >=60fps on the reference desktop (Ryzen/RTX 3090 Ti, DX12) at the active working set; >=30fps floor under heavy load. |
| `NFR-CIV-PERF-901` | b8 | high | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md:16` | Rendering SHALL use GPU instancing / indirect draw for massive agent + voxel counts. |
| `NFR-CIV-PERF-902` | b8 | high | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md:17` | Streaming + meshing SHALL be off the critical render thread (async dirty-queue drain). \| No frame hitch >X ms on chunk load/mesh; dirty queue drained on worker threads. |
| `NFR-CIV-REL-001` | b8 | high | `docs/reference/non-functional-requirements.md:236` | The engine and server processes SHALL NOT panic during normal simulation operation. ... never an unhandled `unwrap()` or `expect()` on runtime data. |
| `NFR-CIV-SCALE-003` | b8 | high | `docs/reference/non-functional-requirements.md:220` | The WebSocket server SHALL handle at least 10 simultaneous client connections without degradation in tick processing latency or client delta delivery latency. |
| `NFR-CIV-SCALE-004` | b8 | high | `docs/guides/voxel-emergent-vision-and-migration.md:172` | \| New \| NFR-CIV-SCALE-004 \| The voxel world SHALL support a minimum of 1,024^3 cells in the SVO structure without exceeding the NFR-CIV-PERF-006 memory ceiling. \| |
| `NFR-CIV-SCALE-900` | b8 | high | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md:11` | The world SHALL support a ~20mi x 20mi (~32km x 32km) extent at a 1-4 m base voxel via SVO + dense 16^3 leaf chunks. |
| `NFR-CIV-SCALE-902` | b8 | high | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md:13` | Disk is the primary bound: the on-disk format SHALL be compact (LOD pyramids + compressed chunks). |
| `NFR-CIV-SCALE-910` | b8 | high | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md:14` | Agent simulation SHALL be LOD-tiered: full per-agent (Hot) near camera/active areas, statistical/aggregate (Cold) far away. \| `LodTier::{Hot,Warm,Cold}` drives fidelity |
| `NFR-CIV-SCALE-920` | b8 | high | `docs/specs/requirements/NFR-CIV-SCALE-PERF.md:18` | All LOD/streaming transitions SHALL preserve the determinism contract (NFR-CIV-DET). \| Same seed + same camera path -> bit-identical sim state regardless of which chunks were resident when. |

### `IMPLEMENTED-ELSEWHERE` (14)

| ID | Batch | Conf | Source | Evidence |
|---|---|---|---|---|
| `FR-CIV-BEVY-021` | b3 | medium | `docs/traceability/fr-3d-matrix.md:186` | \| FR-CIV-BEVY-021 \| P-W1 item 46: path-filtered CI invokes the headless live-attach smoke recipe. \| `.github/workflows/civis-3d-live-smoke.yml`, `justfile` \| `just civis-3d-live-smoke` \ |
| `FR-CIV-PSYCHE-004` | b3 | medium | `FUNCTIONAL_REQUIREMENTS.md:510` | - **FR-CIV-PSYCHE-004** — Historian embellishment rates SHALL be a measurable function of psyche traits (testable monotonicity on openness/conscientiousness). |
| `FR-CIV-PSYCHE-007` | b3 | medium | `FUNCTIONAL_REQUIREMENTS.md:513` | - **FR-CIV-PSYCHE-007** — No LLM call SHALL compute needs, mood, or utility scores. |
| `FR-CIV-PSYCHE-020` | b3 | medium | `docs/design/psyche-social.md:153` | Beliefs are the agent's personal sample of the culture it is *exposed to through its strongest ties*, not the cluster average (**FR-CIV-PSYCHE-020**). |
| `FR-CIV-PSYCHE-030` | b3 | high | `docs/design/psyche-social.md:162` | Per `SocialEvent` apply a bounded delta then renormalise (**FR-CIV-PSYCHE-030**): |
| `FR-CIV-PSYCHE-031` | b3 | high | `docs/design/psyche-social.md:174` | **Decay each Warm/Cold tick for ties not seen recently** (**FR-CIV-PSYCHE-031**): |
| `FR-CIV-PSYCHE-040` | b3 | high | `docs/design/psyche-social.md:290` | \| FR-CIV-PSYCHE-040 \| `relation_label` derives relation kind at query time (no stored taxonomy) \| |
| `FR-CIV-TECH-013` | b2 | medium | `docs/design/tech-engineering.md:237` | **FR-CIV-TECH-013** \| A technique spreads across a contact edge to a neighbour but not to an isolated population. \| cross-population test \| AC-D3, AC-D5 |
| `FR-CIV-TECH-021` | b2 | low | `docs/design/tech-engineering.md:245` | **FR-CIV-TECH-021** \| Agent-built and user-placed structures of the same type share identical data tags (author-agnostic). \| tag-parity test \| AC-G3 |
| `FR-CIV-WEB-000` | b6 | medium | `docs/development-guide/fr-web-spectator.md:29` | \| **FR-CIV-WEB-000** \| Dashboard builds and runs (`vite` dev / production build). \| `npm test` in `web/` passes; `web/dashboard` builds without error. \| |
| `FR-CIV-WEB-001` | b6 | high | `docs/development-guide/fr-web-spectator.md:30` | \| **FR-CIV-WEB-001** \| Resolve WS URL from env (`CIVIS_WS_URL`, `CIVIS_WS_ADDR`) with documented default. \| Unit test `resolveWsUrlFromEnv` passes; default `ws://127.0.0.1:3000/ws`. \| |
| `FR-CIV-WEB-005` | b6 | medium | `docs/development-guide/fr-web-spectator.md:34` | \| **FR-CIV-WEB-005** \| Replay: trigger `sim.save_replay` / load via `sim.load_replay` or `POST /replay/import`; show success/error. \| Roundtrip test: save → load → snapshot tick matches w |
| `FR-SAVE-006` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2805` | \| FR-SAVE-006 \| The save file format SHALL include a BLAKE3 integrity hash covering all serialized state, computed at save time and verified at load time before any deserialization begins. |
| `FR-SAVE-009` | b9 | high | `docs/specs/CIV-1000-save-load-persistence-spec.md:2808` | \| FR-SAVE-009 \| The BLAKE3 hash chain tail SHALL be serialized and restored on load, enabling the chain to continue unbroken from the saved tick. \| MUST \| §3.3, §7.1 \| |

### `NAMESPACE-COLLISION` (7)

| ID | Batch | Conf | Source | Evidence |
|---|---|---|---|---|
| `FR-CIV-0700` | b3 | medium | `docs/specs/CIV-0700-modding-api-spec.md:3` | **Spec ID:** CIV-0700 |
| `FR-CIV-ECON-002-JOULE` | b5 | medium | `agileplus-specs/civ-021-recovered-requirements/spec.md:70-74` | - [ ] **FR-CIV-ECON-002-JOULE** — Joule allocator with energy conservation in `crates/economy/src/joule.rs`. Source: `docs/guides/COPILOT_L3_AGENTS.md:92,474`. RENAME-candidate: collapse to  |
| `FR-CIV-EMERGENCE-RELIGION-1` | b2 | high | `docs/design/RELIGION_EMERGENCE.md:243` | /// FR-CIV-EMERGENCE-RELIGION-1 — Norenzayan Big-Gods response. ///   hardship = clamp01(\|∇T\|_p · w_T + \|∇B\|_p · w_B + \|∇M\|_p · w_M) ∈ [0, 1];  group_size = cluster.member_count;  unce |
| `FR-CIV-GODOT-UX-000` | b9 | medium | `docs/development-guide/fr-godot-attach.md:13` | \| FR-CIV-GODOT-UX-000 \| N spawns → N entity events \| `ux::spawn_emits_entity_events` (Rust unit test) \| |
| `FR-CIV-RESEARCH-004-REPLAY` | b9 | medium | `PLAN.md:239` | \| P5.8 \| Replay format spec & validation \| ... `copilot -p "Implement FR-CIV-RESEARCH-004-REPLAY: Replay format. Create docs/REPLAY_FORMAT.md and implement crates/engine/src/replay.rs. Sp |
| `FR-CIV-SOCIAL-001` | b5 | medium | `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:26` | - [ ] **FR-CIV-SOCIAL-001**: Institution system — `Institution { policies, members, budget, approval_rating }`; methods: `add_member()`, `remove_member()`, `update_policy()`; policies stored |
| `FR-CIV-SOCIAL-002` | b5 | medium | `agileplus-specs/civ-003-actor-citizen-lifecycle/spec.md:27` | - [ ] **FR-CIV-SOCIAL-002**: Ideology field on Citizen — `ideology: Fixed` in range [-1, +1] (libertarian to authoritarian); `ideology_shift()` driven by institution policy drift; bounded ea |

### `MISFILED-TEMPLATE` (23)

| ID | Batch | Conf | Source | Evidence |
|---|---|---|---|---|
| `NFR-C-02` | b8 | medium | `docs/models/civ-sim/TECHNICAL_SPEC.md:2042` | \| NFR-C-02 \| Cross-platform hash match \| Same hash on x86_64 Linux, ARM64 macOS, WASM \| Matrix CI build + determinism test on all three targets \| CI matrix: GitHub Actions with `ubuntu- |
| `NFR-CIV-MAINT-005` | b8 | high | `docs/reference/non-functional-requirements.md:517` | Import relationships between workspace crates SHALL conform to the dependency graph defined in `tach.toml`; no crate SHALL import from a crate that is not declared as its dependency. |
| `NFR-CIV-MAINT-006` | b8 | high | `docs/reference/non-functional-requirements.md:531` | No `#[allow(...)]` or `#[cfg_attr(..., allow(...))]` attribute SHALL be committed without an inline comment on the same or preceding line explaining why the suppression is necessary |
| `NFR-CIV-PERF-004` | b8 | high | `docs/reference/non-functional-requirements.md:69` | The engine tick budget SHALL scale sub-linearly with agent count such that the P99 tick duration at 10,000 agents does not exceed 80 ms |
| `NFR-CIV-PERF-006` | b8 | high | `docs/reference/non-functional-requirements.md:100` | The combined engine + Bevy client process SHALL consume no more than 4 GB of resident RAM at steady state with 10,000 entities loaded on a 2,048x2,048 cell map. |
| `NFR-CIV-PERF-007` | b8 | high | `docs/reference/non-functional-requirements.md:114` | The simulation server SHALL NOT consume more than 10 Mbps aggregate outbound bandwidth when serving 10 simultaneous game clients receiving binary delta frames at 60 ticks per second. |
| `NFR-CIV-REL-002` | b8 | high | `docs/reference/non-functional-requirements.md:250` | If a required dependency (database, NATS broker, MinIO, simulation scenario file) is unavailable at startup, the process SHALL emit a structured preflight failure listing each missing depend |
| `NFR-CIV-REL-003` | b8 | high | `docs/reference/non-functional-requirements.md:264` | A running simulation SHALL checkpoint its full world state to a durable .civreplay file at least every 60 seconds of wall-clock time, independently of whether any client is connected. |
| `NFR-CIV-SEC-002` | b8 | high | `docs/reference/non-functional-requirements.md:308` | No secret values (API keys, database passwords, signing keys, Firepass/Kimi credentials) SHALL appear in source files, committed configuration files, or log output. |
| `NFR-CIV-SEC-003` | b8 | high | `docs/reference/non-functional-requirements.md:322` | The engine and client processes SHALL NOT initiate outbound network connections to external hosts (beyond the configured simulation server address) without an explicit opt-in configuration f |
| `NFR-CIV-SEC-004` | b8 | high | `docs/reference/non-functional-requirements.md:336` | The codebase SHALL maintain zero high-severity or critical-severity security findings as reported by `bandit` (Python), `semgrep` (Rust/Python), and `gitleaks` (secrets) on every CI run. |
| `NFR-O-01` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2088` | \| NFR-O-01 \| Prometheus metric coverage \| 100% of tick phases have latency histograms \| Count `HistogramVec` labels vs `PhaseId` enum variants \| |
| `NFR-O-02` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2089` | \| NFR-O-02 \| Structured log completeness \| Every error has structured fields: `tick`, `phase`, `entity_id`, `error` \| Log schema validation in CI \| |
| `NFR-O-03` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2090` | \| NFR-O-03 \| Trace propagation \| Every WebSocket command is traceable from client receipt to tick application \| `tracing::span` with `trace_id` propagated through command -> phase \| |
| `NFR-O-04` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2091` | \| NFR-O-04 \| Metrics cardinality \| Total Prometheus time series count < 10,000 \| Prometheus cardinality API \| |
| `NFR-O-05` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2092` | \| NFR-O-05 \| Dashboard coverage \| All NFR metrics visible in Grafana dashboard \| Manual dashboard review \| |
| `NFR-O-06` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2093` | \| NFR-O-06 \| Alert coverage \| Each p99 latency target has a Prometheus alerting rule \| CI: validate `ops/prometheus/alerts.yml` with `promtool check rules` \| |
| `NFR-R-01` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2077` | \| NFR-R-01 \| Snapshot persistence interval \| Snapshot written to PostgreSQL every 100 ticks \| `civlab_snapshots_written_total` counter \| |
| `NFR-R-02` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2078` | \| NFR-R-02 \| Recovery point objective (RPO) \| On crash, resume from last persisted snapshot (< 100 ticks lost) \| Kill server mid-run, restart, verify tick counter \| |
| `NFR-R-03` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2079` | \| NFR-R-03 \| Recovery time objective (RTO) \| Server restart + state load < 30 seconds \| Health check endpoint `/health` transitions from `starting` to `ready` \| |
| `NFR-R-04` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2080` | \| NFR-R-04 \| Event log durability \| Event log flushed to disk before acknowledgement \| `fsync` on event log append (O_DSYNC) \| |
| `NFR-R-05` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2081` | \| NFR-R-05 \| Client reconnect \| Client can reconnect and receive current snapshot within 2 seconds \| Integration test: disconnect client, reconnect, measure time to first snapshot \| |
| `NFR-R-06` | b8 | high | `docs/models/civ-sim/TECHNICAL_SPEC.md:2082` | \| NFR-R-06 \| Simulation panic isolation \| Panic in one tick phase does not kill the server process \| Inject panic via test endpoint, verify server continues \| |

### `MISFILED-REPORTING` (13)

| ID | Batch | Conf | Source | Evidence |
|---|---|---|---|---|
| `NFR-CIV-001` | b8 | high | `docs/traceability/nfr-matrix.md:52` | \| `NFR-CIV-001` \| performance \| engine \| planned \| engine \| tbd \| Reserved - populated as each NFR is given a row \| |
| `NFR-CIV-002` | b8 | high | `docs/traceability/nfr-matrix.md:53` | \| `NFR-CIV-002` \| performance \| engine \| planned \| engine \| tbd \| Reserved \| |
| `NFR-CIV-003` | b8 | high | `docs/traceability/nfr-matrix.md:54` | \| `NFR-CIV-003` \| reliability \| engine \| planned \| engine \| tbd \| Reserved \| |
| `NFR-CIV-004` | b8 | high | `docs/traceability/nfr-matrix.md:55` | \| `NFR-CIV-004` \| reliability \| supervision \| planned \| substrate \| tbd \| Reserved \| |
| `NFR-CIV-005` | b8 | high | `docs/traceability/nfr-matrix.md:56` | \| `NFR-CIV-005` \| security \| auth \| planned \| substrate \| tbd \| Reserved \| |
| `NFR-CIV-006` | b8 | high | `docs/traceability/nfr-matrix.md:57` | \| `NFR-CIV-006` \| security \| audit \| planned \| substrate \| tbd \| Reserved \| |
| `NFR-CIV-007` | b8 | high | `docs/traceability/nfr-matrix.md:58` | \| `NFR-CIV-007` \| maintainability \| substrate \| planned \| substrate \| tbd \| Reserved \| |
| `NFR-CIV-008` | b8 | high | `docs/traceability/nfr-matrix.md:59` | \| `NFR-CIV-008` \| portability \| driver \| planned \| substrate \| tbd \| Reserved \| |
| `NFR-CIV-009` | b8 | high | `docs/traceability/nfr-matrix.md:60` | \| `NFR-CIV-009` \| scalability \| wave \| planned \| substrate \| tbd \| Reserved \| |
| `NFR-CIV-010` | b8 | high | `docs/traceability/nfr-matrix.md:61` | \| `NFR-CIV-010` \| observability \| trace \| planned \| phenoobs \| tbd \| Reserved \| |
| `NFR-CIV-011` | b8 | high | `docs/traceability/nfr-matrix.md:62` | \| `NFR-CIV-011` \| safety \| faction \| planned \| engine \| tbd \| Reserved \| |
| `NFR-CIV-012` | b8 | high | `docs/traceability/nfr-matrix.md:63` | \| `NFR-CIV-012` \| interoperability \| bridge \| planned \| substrate \| tbd \| Reserved \| |
| `NFR-CIV-013` | b8 | high | `docs/traceability/nfr-matrix.md:64` | \| `NFR-CIV-013..034` \| various \| various \| planned \| various \| tbd \| Reserved (22 rows remaining; populated as each NFR is given a row) \| |

### `MISFILED-DESIGN` (12)

| ID | Batch | Conf | Source | Evidence |
|---|---|---|---|---|
| `FR-CIV-GEO-001` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2024` | \| FR-CIV-GEO-001 \| Terrain Types & Properties \| Map Rendering (Section 3); Tile Layer Stack (Section 3.2); terrain atlas sprites \| Each terrain type has distinct sprite; movement costs s |
| `FR-CIV-GEO-002` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2025` | \| FR-CIV-GEO-002 \| Map Generation & Biome Systems \| Map viewport at Zoom 1 (biome territory blocs); Zoom 2 (biome tile textures); Climate Overlay (Section 5.3) \| Biome visual distinct fr |
| `FR-CIV-GEO-003` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2026` | \| FR-CIV-GEO-003 \| District & Region Subdivision \| District Panel in Zoom 2 left sidebar; breadcrumb navigation (Nation → Region → District → Citizen); drill-down click \| Zoom transition |
| `FR-CIV-GEO-004` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2027` | \| FR-CIV-GEO-004 \| Neighbor Queries & Pathfinding \| Path Visualization (Section 6.4); A* path preview; hex coordinate system (Section 3.1) \| Path preview generated on move command issue; |
| `FR-CIV-GEO-005` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2028` | \| FR-CIV-GEO-005 \| Resource Distribution & Renewal \| Resource Bar (Section 4.2) with delta indicators; District Panel resource bars; Economy Overlay choropleth \| Delta indicators show pe |
| `FR-CIV-GEO-006` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2029` | \| FR-CIV-GEO-006 \| Climate Events & Modulation \| Climate Overlay (Section 5.3); Alert Feed climate events; Resource Bar CO₂ counter \| Drought → food delta warning in Resource Bar; climat |
| `FR-CIV-GEO-007` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2030` | \| FR-CIV-GEO-007 \| District Connectivity & Trade Routes \| Economy Overlay trade route arrows (Section 5.1); District Panel trade routes button \| Trade routes visible in Economy Overlay;  |
| `FR-CIV-GEO-008` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2031` | \| FR-CIV-GEO-008 \| Population Density & Urban Growth \| District Panel population density display; Social Overlay migration flows (Section 5.4) \| High-density districts shown with urbaniz |
| `FR-CIV-GEO-009` | b6 | high | `docs/specs/CIV-0300-rts-ui-ux-spec.md:2032` | \| FR-CIV-GEO-009 \| Disaster Zones & Recovery \| Alert Feed disaster events; Map: disaster zone red overlay polygon; District Panel recovery progress bar \| Recovery progress shown as perce |
| `FR-CIV-GEO-010` | b6 | medium | `docs/specs/CIV-0101-two-zoom-lod-v1.md:1580` | # Section 9: FR-CIV-GEO-010 Traceability  This section explicitly maps the CIV-0101 spec to FR-CIV-GEO-010 acceptance criteria.  \| FR-CIV-GEO-010 Criterion \| Fulfilled By \| Section \| |
| `FR-CIV-PSYCHE-008` | b3 | medium | `FUNCTIONAL_REQUIREMENTS.md:514` | - **FR-CIV-PSYCHE-008** — Canonical mode MAY seed OCEAN priors per culture exemplar; primitive mode draws from configured distributions only. |
| `FR-CIV-WAR-020` | b9 | high | `docs/design/warfare.md:105` | ### 3.2 RTS + direct control — `FR-CIV-WAR-020` Two co-existing control modes over the *same* tactical state (CtA continuity-of-agency): **RTS / command mode:** player issues squad-level ord |

### `SYNTHETIC` (15)

| ID | Batch | Conf | Source | Evidence |
|---|---|---|---|---|
| `FR-CIV-EMERGENCE-100` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:150` | \| civ-linguabridge \| FR-CIV-EMERGENCE-100..110 \| 11 rows \| |
| `FR-CIV-EMERGENCE-111` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:151` | \| civ-factions \| FR-CIV-EMERGENCE-111..118 \| 8 rows \| |
| `FR-CIV-EMERGENCE-119` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:152` | \| civ-religion \| FR-CIV-EMERGENCE-119..123 \| 5 rows \| |
| `FR-CIV-EMERGENCE-124` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:153` | \| civ-market \| FR-CIV-EMERGENCE-124..131 \| 8 rows \| |
| `FR-CIV-EMERGENCE-132` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:154` | \| civ-urban \| FR-CIV-EMERGENCE-132..140 \| 9 rows \| |
| `FR-CIV-EMERGENCE-141` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:155` | \| civ-climate \| FR-CIV-EMERGENCE-141..143 \| 3 rows \| |
| `FR-CIV-EMERGENCE-144` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:156` | \| civ-econ \| FR-CIV-EMERGENCE-144..150 \| 7 rows \| |
| `FR-CIV-EMERGENCE-151` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:157` | \| civ-demographics \| FR-CIV-EMERGENCE-151..167 \| 17 rows \| |
| `FR-CIV-EMERGENCE-168` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:158` | \| civ-psyche \| FR-CIV-EMERGENCE-168..197 \| 30 rows \| |
| `FR-CIV-EMERGENCE-198` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:159` | \| civ-legends \| FR-CIV-EMERGENCE-198..220 \| 23 rows \| |
| `FR-CIV-EMERGENCE-221` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:160` | \| civ-ai \| FR-CIV-EMERGENCE-221..235 \| 15 rows \| |
| `FR-CIV-EMERGENCE-236` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:161` | \| civ-culture \| FR-CIV-EMERGENCE-236..238 \| 3 rows \| |
| `FR-CIV-EMERGENCE-239` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:162` | \| civ-social \| FR-CIV-EMERGENCE-239..240 \| 2 rows \| |
| `FR-CIV-EMERGENCE-241` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:163` | \| civ-diplomacy \| FR-CIV-EMERGENCE-241..248 \| 8 rows \| |
| `FR-CIV-EMERGENCE-249` | b4 | high | `docs/traceability/emergent-systems-tracelinks.md:164` | \| civ-laws \| FR-CIV-EMERGENCE-249..254 \| 6 rows \| |
