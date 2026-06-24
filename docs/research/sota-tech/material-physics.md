# SOTA Physics / Material Simulation → Civis voxel fluid-CA upgrade path

**Scope:** the upgrade path for **our voxel material-fluid CA** (the Layer-0 substrate of the [emergence charter](../../guides/emergence-charter.md)) toward AAA material sim — from today's CPU cellular automata, through GPU CA, to particle methods (MPM/FLIP/SPH) — plus the rigid-body layer for debris. Companion: [gfx.md](./gfx.md) (rendering the result), [engine-parity README](../engine-parity/README.md) (Avian/Rapier/Jolt plug list — referenced not duplicated).

## Civis baseline
`crates/voxel/src/material.rs` + `boundary.rs`: voxel material-fluid CA — liquids flow + find level, powders pile at angle-of-repose, gases rise, solids hold; world-boundary walls; mass-conserving. CPU-side, on the SVO + dense-leaf-chunk substrate (charter Layer-0). This is a **Noita/Powder-Toy-class CA** already. The charter (determinism-NOT-required as of 2026-05-29) frees us to use floats/GPU. Goal: keep the CA's emergent richness, scale it (GPU), and selectively upgrade the *most-visible* materials toward AAA particle sim.

---

## Tier 0 — Harden the CA we have (Noita + Powder-Toy patterns) `[adopt-now]`

### Noita's falling-sand pipeline (GDC "Exploring the Tech and Design of Noita")
The reference for *shippable* large CA worlds. Three load-bearing tricks we should match if not already present:
1. **Dirty-rect chunks.** World split into 64×64 chunks; each keeps a **dirty rectangle** of the sub-region that changed. Static pixels are skipped entirely — a sleeping chunk costs ~nothing. This is *the* scaling trick; our SVO chunks should carry per-chunk dirty bounds and a sleep flag. Maps directly onto our "active working set in memory, stream the rest" charter scale plan.
2. **Update-order discipline.** Falling sand must update **bottom-up** (and alternate scan direction L/R per frame) or you get directional bias / column artifacts. 3D adds the gravity axis; pick a consistent sweep + checkerboard/double-buffer to avoid one cell being processed twice per tick.
3. **Multi-threading by chunk** with a 4-pass checkerboard so neighboring chunks never mutate the same border simultaneously (Noita's threading model). Rust/`rayon` over non-adjacent chunk sets.

### Powder-Toy's element/reaction model `[adopt-now, design ref]`
The reference for *material richness*. Powder Toy ships ~258 elements over **two ambient fields: temperature + pressure**, every element pair having explicit contact reactions + phase transitions driven by temp/pressure. This is exactly our charter's `crates/laws` (reaction/phase rules, energy costs, conservation). **Adopt the model shape**: materials as data rows with `{state, density, melt/boil points, reaction table, thermal/pressure response}`; reactions are data, not code — emergent chemistry from a table. Don't author 258; let the *rule table* generate variety (charter: "model the rule, not the outcome"). Add **temperature + pressure as first-class CA fields** if not present — they unlock fire, steam, ice, explosions, structural stress for free.

**Tier-0 verdict:** ensure dirty-rect sleeping chunks, correct 3D update-order, chunk-parallel `rayon`, and a temp+pressure-driven data-table reaction model. This is pure win, no new deps, and is the foundation everything else builds on.

## Tier 1 — GPU cellular automata `[adopt-next]`
Move the CA stencil to **wgpu compute shaders** (Bevy 0.18). CA is embarrassingly parallel per cell; a compute pass over dense-leaf chunks gives 1–2 orders of magnitude more simulated voxels at framerate. Pattern: ping-pong 3D textures / storage buffers per active chunk, dispatch only dirty chunks (carry the dirty flag to the GPU), read back only what gameplay needs (mostly nothing — render straight from the GPU buffer, see gfx.md voxel rendering). Determinism-not-required removes the float-order objection. **This is the single highest-leverage scale upgrade** for the material substrate. Keep CPU CA for far-LOD / gameplay-authoritative regions; GPU CA for the near, visible, high-detail field.

## Tier 2 — Particle methods for premium materials `[experimental → adopt-next selectively]`
The CA gives *plausible* bulk behavior; particle methods give *AAA* behavior for the materials the camera lingers on (rivers, mud, snow, sand dunes, lava). Don't replace the CA — **couple** a particle sim to it in a bounded region.

- **MPM / MLS-MPM** (Material Point Method) `[experimental]` — the Disney "Frozen" snow method; the modern AAA granular/elastoplastic standard. **MLS-MPM** (Hu et al.) is the real-time-friendly variant: one method covers fluid + sand + snow + mud + elastic + their *mixing* in a unified grid-particle solver. GPU MLS-MPM runs millions of particles in real time (Taichi + CUDA references; multi-GPU debris-fluid-structure work, 2025). **Best fit for Civis** because one solver gives the whole material charter (snow/sand/mud/lava all at once) and couples naturally to our voxel grid (MPM *is* grid+particles). No mature pure-Rust crate yet — path is wrap a CUDA/`wgpu` MLS-MPM kernel or port the ~few-hundred-line MLS-MPM core to a Bevy compute shader (the algorithm is famously compact). Tag experimental now; strong adopt-next bet.
- **FLIP / PIC / APIC** `[experimental]` — splash-y liquid (the "Houdini water" look). PIC=stable/dissipative, FLIP=energetic/noisy, APIC/ASFLIP=the modern stable-yet-lively blend. A FLIP solver in a bounded volume gives premium river/waterfall/ocean-edge visuals over the CA water. Couple at the CA boundary (CA supplies/absorbs mass).
- **SPH** (Smoothed-Particle Hydrodynamics) `[avoid for bulk]` — meshless fluid; popular but harder to scale to world-size + couple to a grid than MPM/FLIP. Use only for small isolated effects; **prefer MPM/FLIP** for the Civis grid-coupled case.

**Coupling pattern:** CA is the world-wide cheap substrate; near the camera / for hero materials, spawn an MLS-MPM (or FLIP) region that reads CA boundary conditions and writes results back as voxel density — the player sees AAA mud/snow/water locally, CA everywhere else. Matches LOD-tiering.

## Tier 3 — Rigid bodies + destruction `[adopt-now, already plugged]`
Covered in engine-parity (don't duplicate): **`Avian`** (pure-Rust, ECS-native — preferred) or **`bevy_rapier`** for rigid debris; **Jolt-C** via FFI only if profiling demands it. Civis destruction = **voxel CA fracture → spawn rigid debris** (chunks break off the CA field into Avian bodies, settle, optionally re-fuse into CA). This is the Chaos-destruction analog without porting Chaos. Add structural-stress propagation in the CA (Powder-Toy pressure field + our `laws` stress) to decide *when* solids fail → hand fragments to Avian.

---

## Recommended material-sim upgrade path (CA → GPU → MPM)
```
NOW   Tier 0  Harden CPU CA: dirty-rect sleeping chunks + 3D update-order + rayon
              chunk-parallel + temp/pressure fields + data-table reactions (Powder-Toy model)
NEXT  Tier 1  Port the CA stencil to wgpu compute (GPU CA), dispatch only dirty chunks,
              render from GPU buffers. CPU CA stays for far-LOD/authoritative.
NEXT  Tier 3  Avian rigid debris + CA structural-stress fracture (destruction loop).
LATER Tier 2  Bounded MLS-MPM (snow/sand/mud/lava unified) + optional FLIP (hero water)
              regions coupled to CA at boundaries, near-camera only. Experimental bet.
```

## Verdict
- **CA stays the substrate** (charter Layer-0) — it is already Noita/Powder-Toy-class; harden it (dirty-rect sleep, 3D order, rayon, temp+pressure, data-table reactions) `[adopt-now]`.
- **GPU CA is the top scale lever** `[adopt-next]` — wgpu compute, dirty-chunk dispatch; determinism-not-required clears the path.
- **MLS-MPM is the premium-material bet** `[experimental → adopt-next]` — one solver for snow/sand/mud/lava/elastic, grid-coupled, bounded near-camera; port the compact core to a compute shader or wrap a CUDA/wgpu kernel (no mature Rust crate yet).
- **Rigid/destruction via Avian** `[adopt-now]` + CA structural-stress fracture; no Chaos port.

## Sources
- [Noita: a Game Based on Falling Sand Simulation (80.lv)](https://80.lv/articles/noita-a-game-based-on-falling-sand-simulation)
- [SandFall Intro — Noita-style CA implementation (Okko)](https://blog.okkohakola.com/SandFall/SandFallIntro)
- [The Powder Toy — 258 elements, temperature+pressure fields](https://powdertoy.co.uk/)
- [CK-MPM: A Compact-Kernel Material Point Method (arXiv 2412.10399)](https://arxiv.org/html/2412.10399)
- [Validating High-Performance Multi-GPU MPM for Debris-Fluid-Structure Interaction (2025, Wiley)](https://onlinelibrary.wiley.com/doi/10.1002/nme.70210)
- [MLS-MPM 3D fluid/elastic/snow (Taichi)](https://github.com/CzzzzH/MLS-MPM) · [Real-time GPU MPM (USPTO)](https://image-ppubs.uspto.gov/dirsearch-public/print/downloadPdf/11966999)
- Avian / bevy_rapier — crates.io (see engine-parity README)
