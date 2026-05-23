# Civis 3D Extension — PRD Addendum

**Status:** PROPOSED (additive to `PRD.md`)
**Date:** 2026-05-22
**Branch:** `feat/civis-3d-foundation`
**Owner:** Civis Engineering

> This addendum **extends** `PRD.md` (CivLab v1/v2/v3 scope) into a high-fidelity 3D
> WorldBox-class sandbox. It does not replace any existing requirement; every existing
> FR, ADR, and epic remains in force.

---

## Executive Summary

CivLab is being extended into a **3D high-fidelity civilization sandbox** that fuses
WorldBox + Dinosaur sim spawn UX, Cities Skylines 2 + Manor Lords autonomous citizen-driven
city growth, Star Wars EAW:FOC + Call to Arms warfare layering, and Galimulator + Dwarf
Fortress depth — all anchored on the existing CivLab deterministic Rust spine.

Scope is bounded to **a single planet + moon**, **eras prehistoric → near-future scifi**,
and a **hybrid deterministic + LLM-driven progression model** that preserves bit-identical
replay (ADR-004) under all modes.

## Vision Delta

Where the existing `PRD.md` positions CivLab against Victoria 3 / Dwarf Fortress / CK3 /
Factorio (political-economy depth, deterministic replay, multi-client protocol), this
addendum adds:

- **Voxel-destructible terrain + structures** (Teardown/Noita lineage, smoothed via
  marching-cubes/dual-contouring shaders so debris reads as broken-realistic, not cubic).
- **Multi-species + algorithmic genetics** (WorldBox lineage; DNA as byte vectors, mutation,
  recombination, fitness, speciation thresholds; no LLM in the genetic loop).
- **Deep warfare** across four layers — strategic, operational, tactical voxel-destructible,
  and doctrine (genetic-algo unit-composition evolution).
- **Procedural voxel building with two front-ends** — autonomous demand-driven city growth
  and user freehand authoring — both resolving into a shared `BuildingGraph` schema.
- **Hybrid deterministic + LLM research** — LLM proposes side-tech branches when in-game
  research teams stall; every output is hash-keyed and cached so replay stays bit-identical.
- **Per-civilian wardrobe / tools state** so technology diffuses visibly through society
  via a Bass/Rogers S-curve (`crates/diffusion`) rather than snap-upgrading.

## Target Users (additions)

| User type | Motivation | Example use case |
|---|---|---|
| **Sandbox player** | WorldBox-style emergent play | "Spawn three species on the same continent and watch speciation drift" |
| **Modder** | Add tech / species / facade grammars | "Drop a `mods/futurism/` folder of RON tech cards and run a Free-mode game" |
| **Streamer** | Demo emergent civilizational stories | "Twitch poll which faction the LLM should propose a new doctrine for" |

These do not displace the existing PRD audiences (game developer / designer / researcher /
educator) — they extend them.

## Feature Matrix Additions

Phases ride on the existing PLAN.md Phases 0–6 and on the new `plan-3d-phases.md` (P-V0
through P-U1 — voxel kernel through WorldBox UX). See that file for the DAG.

| Feature | Crate(s) | Phase |
|---|---|---|
| Adaptive voxel substrate (SVO + dense 16³ leaves) | `civ-voxel`, shared `phenotype-voxel` | P-V0 → P-V1 |
| Procedural voxel buildings + freehand authoring | `civ-build` | P-V2 |
| Algorithmic DNA + speciation | `civ-genetics`, `civ-species` | P-G1 |
| Per-civilian agents w/ wardrobe + tools state | `civ-agents`, `civ-diffusion` | P-A1 |
| Physics-law DB (versioned RON) | `civ-laws` | P-L1 |
| LLM R&D pipeline (replay-safe cache) | `civ-research` | P-R1 |
| Tactical voxel-destructible combat + doctrine evolution | `civ-tactics` | P-W1 |
| Single planet + moon, geology/weather/tides | `civ-planet` | P-P1 |
| WorldBox-style spawn UX | `clients/godot-ref` | P-U1 |
| 3D reference rendering (3 clients in parallel) | `clients/{bevy-ref,godot-ref,unreal-show}` | P-V1 onward |

## Non-Functional Additions

- **Replay-safety with LLM:** every LLM output is recorded with
  `{seed, prompt_hash, model_id, model_version, input_snapshot_hash, output_hash}` and
  cached. Canonical mode refuses to advance when the cache miss; Hybrid/Free modes accept
  cache hits and gate live calls. See ADR-006.
- **Determinism with voxels:** all voxel writes go through a write-seq-ordered dirty queue
  so chunk-mesh rebuild order is bit-identical across machines. See ADR-005.
- **Visual target:** PBR materials per voxel facet; per-engine GI (Bevy solari / baked,
  Godot SDFGI, Unreal Lumen). Reference visual bar is COD / Rust / The Finals tier.
- **Scope cap:** single planet + moon. No interstellar. Eras prehistoric → near-future
  scifi only; alt-physics is permitted (Free mode) but must satisfy the law-DB extension
  rules. See ADR-006.

## Open Questions

- **LLM tick budget:** confirmed async-multi-tick in the design (build.md). Resolved.
- **Modding day-1 vs post-MVP:** Owner to decide. Default proposal: RON tech cards
  loadable from `mods/` from P-L1 onward.
- **Unreal client EULA:** ship under standard Epic EULA, or constrain to a shell-only
  renderer (no game logic) to dodge royalty surface? Owner decision.

## Cross-references

- Plan: `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md` (approved 2026-05-22)
- Build/progression design: `~/.claude/plans/civis-3d-scratch/build.md`
- Voxel-kernel design: `~/.claude/plans/civis-3d-scratch/phenotype-voxel-design.md`
- ADR-005 (adaptive voxel), ADR-006 (LLM event sourcing), ADR-007 (three renderers),
  ADR-008 (algorithmic genetics)
- Phased plan: `docs/roadmap/plan-3d-phases.md`
- FR additions: `docs/development-guide/fr-3d-additions.md`
