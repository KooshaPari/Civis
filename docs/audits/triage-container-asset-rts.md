# Triage: container bindings for `crates/engine/src/rts_types.rs` and `crates/asset-pipeline/**`

**Date:** 2026-09-29
**Scope:** the 15 rows of `docs/audits/container-bindings.json` whose `file` is `crates/engine/src/rts_types.rs` (14 distinct ids; the task brief said "13 candidates", the JSON actually holds 15 rows — `FR-CIV-ASSET-003` is tagged twice, at lines 15 and 251). Zero rows exist for `crates/asset-pipeline/**`.
**Method:** requirement text read from `docs/specs/CIV-0600-2d-asset-pipeline-spec.md` §11.1/§11.2 and §14. Judged against the requirement sentence, never the tag or the struct name. No `cargo test` was run. No `.rs` file was modified.

---

## 1. Verdicts

| ID | Verdict | Real implementing symbol (or "none") | Spec file:line | One-line reason |
|---|---|---|---|---|
| FR-CIV-ASSET-001 | CONTAINER-ONLY | none | CIV-0600-2d-asset-pipeline-spec.md:2429 | "SHALL render every `.svg.j2` template for every parameter combination in `asset_parameters.yaml`" is a rendering behavior; `SsConfig { factor, output_width, output_height }` renders nothing, and the repo has 0 `.svg.j2` files, no `asset_parameters.yaml`, no `svg_inject.py`. |
| FR-CIV-ASSET-003 | CONTAINER-ONLY | none | CIV-0600-2d-asset-pipeline-spec.md:2449 | "SHALL apply 4× supersampling … downscale with Lanczos3 … Direct rendering to target size without supersampling is NOT permitted" is a rasterizer behavior; no `resvg` renderer, no Lanczos3, and `SsConfig` is only a `u32` factor field that no code reads. |
| FR-CIV-ASSET-003 (2nd tag) | CONTAINER-ONLY | none | CIV-0600-2d-asset-pipeline-spec.md:2449 | Same requirement, second bare tag on the same `SsConfig` struct (line 251); duplicate claim, same container-only defect. |
| FR-CIV-ASSET-004 | CONTAINER-ONLY | none | CIV-0600-2d-asset-pipeline-spec.md:2459 | "SHALL reject any output sprite with alpha channel coverage below 60%" is a gate; no `rembg_batch.py`, no alpha-coverage computation anywhere in the repo, and `NationColor` holds three hex strings. |
| FR-CIV-ASSET-005 | CONTAINER-ONLY | none | CIV-0600-2d-asset-pipeline-spec.md:2469 | "SHALL preserve nation primary (index 0) and secondary (index 1) as exact forced palette entries" is a quantizer behavior; no `imagequant`/`quantize.rs` exists, and the two `PALETTE_INDEX_*` constants are never consumed by any quantizer. |
| FR-CIV-ASSET-006 | CONTAINER-ONLY | partial, unowned: `civ_render::atlas::next_pow2` (crates/render/src/atlas.rs:191) | CIV-0600-2d-asset-pipeline-spec.md:2479 | "All **output atlas PNGs** SHALL have dimensions that are powers of two" is enforced at pack time; `AtlasConfig` is a dimension record and can guarantee nothing about an output artifact. |
| FR-CIV-ASSET-007 | CONTAINER-ONLY | partial, unowned: `civ_render::atlas::pack_atlas_per_lod` (crates/render/src/atlas.rs:207) | CIV-0600-2d-asset-pipeline-spec.md:2489 | "frame rect fully contained within the atlas dimensions … no overflow, no overlap" is a post-pack validation sweep; `UvRect` is an unused rect type, and the real packer emits `atlas::AtlasEntry`, never `UvRect`. |
| FR-CIV-ASSET-016 | CONTAINER-ONLY | partial, unowned: `civ_engine::rts_types::color_matches` (crates/engine/src/rts_types.rs) | CIV-0600-2d-asset-pipeline-spec.md:2581 | "the nation recoloring **shader** SHALL replace all pixels within TOLERANCE (0.08) of `uBakedPrimary`" is GPU behavior; no nation-recolor shader exists in the repo, and the CPU distance predicate is never called by a renderer. |
| FR-CIV-ASSET-018 | CONTAINER-ONLY | partial, unowned: `SpriteHandle::set_zoom` (crates/engine/src/rts_types.rs) | CIV-0600-2d-asset-pipeline-spec.md:2601 | "when `SpriteManager.setZoomLevel()` is called, **all active** sprite handles SHALL swap textures within the same JavaScript event loop tick" is a manager behavior; there is no `SpriteManager` and no JS runtime, and `set_zoom` mutates one handle. |
| FR-CIV-RTS-RENDER-001 | CONTAINER-ONLY | none | **no spec definition**; only referenced at CIV-0600-2d-asset-pipeline-spec.md:3205 | Id is not defined by any spec. Its JSON `requirement` is the §14 row "SVG Template Rendering \| Stage 1 \| `test_svg_inject.py`" — a traceability cross-reference, not a requirement. The Stage-1 render behavior it aliases is unimplemented. |
| FR-CIV-RTS-RENDER-002 | CONTAINER-ONLY | none | **no spec definition**; only referenced at CIV-0600-2d-asset-pipeline-spec.md:3207 | Id is not defined by any spec. Its `requirement` is the §14 row "4× Supersampling Required \| Stage 2 \| `test_resvg_config.rs`" — an alias for FR-CIV-ASSET-003, a behavior a config struct does not perform. |
| FR-CIV-RTS-RENDER-003 | CONTAINER-ONLY | none | **no spec definition**; only referenced at CIV-0600-2d-asset-pipeline-spec.md:3208 | Id is not defined by any spec. Its `requirement` is the §14 row "Background Removal Quality Gate \| Stage 3 \| `test_rembg_batch.py`" — a gate behavior, tagged on `NationColor`, a nation palette, which is unrelated to background removal under any reading. |
| FR-CIV-RTS-RENDER-004 | CONTAINER-ONLY | partial, unowned: `civ_render::atlas::next_pow2` (crates/render/src/atlas.rs:191) | **no spec definition**; only referenced at CIV-0600-2d-asset-pipeline-spec.md:3210 | Id is not defined by any spec. `AtlasConfig` is a genuine data shape for §7.2 (spec:1366-1381), but the FR is a SHALL over output artifacts enforced by a pack-time assert, which is behavior. Strongest of the six RTS tags; still a false tag. |
| FR-CIV-RTS-RENDER-005 | CONTAINER-ONLY | partial, unowned: `civ_render::atlas::pack_atlas_per_lod` (crates/render/src/atlas.rs:207) | **no spec definition**; only referenced at CIV-0600-2d-asset-pipeline-spec.md:3211 | Id is not defined by any spec. `UvRect::fits_in`/`overlaps` (rts_types.rs) are the right predicates but are called from no packer and no validation gate. |
| FR-CIV-RTS-ZOOM-001 | CONTAINER-ONLY | partial, unowned: `SpriteHandle::set_zoom` (crates/engine/src/rts_types.rs) | **no spec definition**; only referenced at CIV-0600-2d-asset-pipeline-spec.md:3222 | Id is not defined by any spec. `SpriteHandle` is a state record; the FR is a synchronous fan-out swap over all live handles performed by a `SpriteManager` that does not exist. |

**Counts: CONTAINER-ONLY 15. IMPLEMENTED-BY-BEHAVIOR 0. DATA-SHAPE-ONLY 0. NOT-IMPLEMENTED 0.**
**FALSE tags: all 15 rows / 14 distinct ids.**

---

## 2. Why nothing reaches IMPLEMENTED-BY-BEHAVIOR

The six-stage pipeline the spec describes does not exist in this repository. Verified absent by direct filesystem probe:

| Spec stage | Spec reference | Repo state |
|---|---|---|
| 1 `scripts/svg_inject.py` | spec:244, 2431 | absent — `scripts/` has no `.py` at all |
| 2 `src/bin/resvg_batch.rs` | spec:249, 622, 725-760 | absent — no `src/bin` tree; no crate depends on `resvg` |
| 3 `scripts/rembg_batch.py` | spec:245, 2459 | absent — no `rembg` reference in any `.rs` |
| 4 `src/bin/quantize.rs` | spec:250, 2469 | absent — no `imagequant` reference in any `.rs` |
| 5 `src/bin/atlas_pack.rs` | spec:251, 1469-1544 | absent — no `texture_packer` dependency |
| 6 `scripts/gen_manifest.py` | spec:246, 2501 | absent |
| Templates | spec:212-241, 2429 | 0 `.svg.j2` files under `assets/` |
| `asset_parameters.yaml` | spec:2429 | absent |
| `SpriteManager` | spec:1671, 2601 | 0 occurrences in any `.rs`, `.ts`, or `.py` |

`grep -r "lanczos|rembg|imagequant|supersample|uBakedPrimary"` across `crates/**/*.rs` returns hits **only inside `crates/engine/src/rts_types.rs` itself** (the doc comments that narrate the requirements it does not perform). Every match is the claim, never the behavior.

---

## 3. `crates/render/src/atlas.rs` — the specific check the brief asked for

**Does the shelf packer guarantee power-of-two output and non-overlapping frames? Partially, and not for the FRs.**

- **Power-of-two: yes, in memory.** `next_pow2` (atlas.rs:191-197) rounds both dimensions at atlas.rs:247-248, and `pack_atlas_per_lod` allocates `pixels` to that pow2 size. `crates/render/tests/atlas_packed_per_lod.rs` and the in-module test at atlas.rs:352 both assert `is_power_of_two`. This is the closest real behavior in the repo to FR-CIV-ASSET-006.
- **What it does not do:** the FR is about **output atlas PNGs** and requires an `assert_power_of_two` that *panics on non-pow2 input* (spec:1381, 2481). `pack_atlas_per_lod` never takes an atlas size as input, so it cannot reject one — it silently rounds up instead. It also writes no PNG: it returns a `TextureAtlas` with an in-memory `Vec<u8>`. There is no encoder and no file output.
- **Non-overlap: yes, but incidentally and only for small sprites.** The shelf loop (atlas.rs:228-244) advances `x` by `sprite.width` and wraps at `max_row_width = 256`, so frames never overlap. Two consequences: (a) this is a property of the loop, not a validated invariant — nothing asserts it and `UvRect::overlaps` is never called against the output; (b) a sprite wider than 256 px is placed at `x=0` anyway, so a genuinely oversized sprite would not produce a rejection. The spec's 2D interval sweep and pipeline abort (spec:2491) do not exist.
- **It is not the spec's packer.** Spec §7.1 mandates `texture_packer` 0.7 with MaxRects (spec:1341-1357). This is a ~120-line homegrown shelf packer whose own module doc says "The real renderer swaps in `resvg`/`tiny-skia` behind the same signature" (atlas.rs:17).
- **It is a different type family.** The packer emits `atlas::AtlasEntry { x, y, w, h }` (atlas.rs:155-166). `UvRect` in `rts_types.rs` is never constructed, never returned, and never fed to it. The two are entirely disconnected, so no amount of correctness in one covers the other.

Conclusion: `atlas.rs` is genuine unowned behavior worth keeping, but it does not discharge FR-CIV-ASSET-006 or -007, and the struct that was tagged for them discharges neither.

---

## 4. Production consumers

### `crates/asset-pipeline`
- **Container-binding rows:** 0. The brief asked to include it "if any"; there are none.
- **Consumers:** it is a workspace member (`Cargo.toml:42`) but **no other crate's `Cargo.toml` depends on it** (`git grep -l "asset-pipeline" -- "crates/*/Cargo.toml"` returns only itself).
- **State:** `export_svg` (lib.rs) validates two paths then unconditionally returns `Err(ExportError::Encode("scaffold-only stub …")`. The crate's own unit tests assert that failure. It is a scaffold with no production path.
- It carries no `FR-CIV-ASSET-*` tags at all, so it contributes no rows to this triage.

### `crates/engine` (`rts_types` is `pub mod` at `crates/engine/src/lib.rs:100`)
- **Crate consumers:** real. Six crates depend on `civ-engine`: `agents`, `emergence-oracle`, `engine`, `planet`, `server`, `watch`.
- **`rts_types` symbol consumers:** none in production. `NationColor`, `AtlasConfig`, `UvRect`, `SsConfig`, `SpriteHandle`, `ZoomTier`, `color_distance`, `color_matches`, `SHADER_TOLERANCE` are referenced **only** from `crates/engine/src/rts_types.rs` itself and from seven dedicated test files: `fr_fr_civ_rts_render_001..005.rs`, `fr_fr_civ_rts_zoom_001.rs`, `fr_fr_civ_rts_nation_001.rs`, `fr_fr_civ_rts_nation_002.rs`. A repo-wide `findstr` over `crates\*.rs`, `crates\*\src\*.rs`, `clients\*.ts`, `web\*.ts`, `scripts\*.py` returns no other caller.
- So the module is publicly reachable but behaviorally dead: the tagged types exercise nothing in the product.

### `crates/render` (the `atlas.rs` packer)
- **Consumers:** **zero**. `git grep -l "civ-render" -- "crates/*/Cargo.toml"` returns only `crates/render/Cargo.toml` itself. No crate depends on the render substrate.
- `rasterise_at_build` and `pack_atlas_per_lod` are called only from `crates/render/tests/*` and the in-module `#[cfg(test)]` block.

---

## 5. The `FR-CIV-RTS-*` id namespace is itself defective

This is a distinct finding from the container problem and should be raised regardless of `rts_types.rs`.

- `docs/specs/CIV-0300-rts-ui-ux-spec.md` §12.1 defines exactly `FR-CIV-RTS-001` … `FR-CIV-RTS-015` (lines 2004-2018), covering movement, combat, formations, command queuing, supply, construction, damage, fog, diplomacy, siege, espionage, turn structure, XP, faction AI, and client prediction. **`RENDER-*`, `ZOOM-*`, and `NATION-*` do not appear anywhere in that spec.**
- The only occurrence of `FR-CIV-RTS-RENDER-00{1..5}` and `FR-CIV-RTS-ZOOM-001` in the entire repository is the **middle column of CIV-0600's own §14 traceability table** (spec:3205-3224) — i.e. a derived "verification owner" label invented by the very table it points at, naming test files that do not exist.
- The `requirement` field for all six of these rows in `container-bindings.json` is that table row, which is why the field reads as `"Background Removal Quality Gate | FR-CIV-RTS-RENDER-003 | Stage 3 | test_rembg_batch.py | P0"` rather than a requirement sentence. **These six ids cannot be verified against any specification at all**; they are unverifiable by construction, independent of whether the tagged struct implements anything.
- `docs/traceability/fr-civ-rts-render-004/*` compounds this: its intent document claims "Implementing crate: `crates/protocol-3d/src/`", which is a different crate from the one actually tagged (`crates/engine`), and its Definition of Done is unfilled checkboxes. These are generated placeholder documents, not specifications.

**Recommended:** quarantine the whole `FR-CIV-RTS-RENDER-*` / `FR-CIV-RTS-ZOOM-*` / `FR-CIV-RTS-NATION-*` namespace until §14 is reconciled against CIV-0300 §12.1, regardless of the container verdict.

---

## 6. Remediation status observed during this triage

A concurrent agent landed commit `d2246ad2` ("Fix platform-dependent asset-pipeline test; unbind 9 container-only tags") **while this audit was in progress**. It removed 8 of the 9 `FR-CIV-ASSET-*` bare tags from `rts_types.rs` and replaced each with a doc comment quoting the requirement sentence it used to claim.

- `container-bindings.json` rows audited above therefore describe the **pre-remediation** state, which is what the JSON records. That commit, and at least one further concurrent edit, has since shifted line numbers inside `rts_types.rs`, so the "Real implementing symbol" column cites `rts_types.rs` **by symbol name rather than by line** for symbols in that file. Line numbers given for `crates/render/src/atlas.rs` were re-verified after those edits and are current.
- 1 tag remains unaddressed by that commit: the duplicate `FR-CIV-ASSET-003` at pre-remediation line 251 on `SsConfig`, which the commit's own message enumerates but whose second line-number occurrence is easy to miss.
- The commit asserts "The `FR-CIV-RTS-*` tags are untouched: those requirements genuinely are binding and data-shape contracts." This triage does not agree — see §5 and the CONTAINER-ONLY verdicts for all six.
- The commit also added `docs/audits/_detect_container_bindings.py`, which reports 252 requirement tags sitting directly above a `struct`/`enum`/`const` across 57 files. The 15 rows here are a small slice of that set.

No source file was modified by this audit. This markdown file is the only artifact created.
