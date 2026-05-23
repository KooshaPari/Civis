# 2026-05-22 — Civis 3D Extension Kickoff

**Branch:** `feat/civis-3d-foundation`
**PR:** [#296](https://github.com/KooshaPari/Civis/pull/296)
**Plan:** `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md` (owner-approved)

## Scope

Civis is extended (not pivoted) into a 3D high-fidelity WorldBox-class civilization
sandbox. Fuses WorldBox + Dinosaur sim spawn UX, Cities Skylines 2 + Manor Lords
autonomous city growth, Star Wars EAW:FOC + Call to Arms warfare layering, Galimulator
+ Dwarf Fortress depth. Single planet + moon. Eras prehistoric → near-future scifi.

## Decided

- **Renderers:** Bevy (CI + determinism + agent-driven) + Godot 4 (UX iteration) +
  Unreal 5 (visual showcase), all attaching via existing WebSocket protocol. See
  ADR-007.
- **Voxel substrate:** adaptive/hybrid (SVO + dense 16³ leaf chunks). Extracted as a
  new Phenotype-org shared crate `phenotype-voxel`. Bevy adapter ships in that crate;
  Godot + Unreal adapters in their respective clients. See ADR-005.
- **WSM3D coordination:** existing WSM3D voxel work informs the shared crate. WSM3D
  consumes the Rust crate via `ffi-core`/`cbindgen` after extraction.
- **LLM event sourcing:** every LLM output is hash-keyed and cached; Canonical/Hybrid/
  Free modes preserve `.civreplay` bit-identity. See ADR-006.
- **Genetics:** algorithmic only, no LLM in the genetic loop. See ADR-008.
- **Visual bar:** COD / Rust / The Finals tier — PBR + per-engine GI.

## Landed this iteration

- 11 new crate stubs scaffolded under `crates/` (voxel, build, genetics, species,
  agents, diffusion, laws, research, tactics, planet, protocol-3d). Each has a
  `SCHEMA_VERSION` and an FR-CIV-`<NS>`-000 placeholder test.
- Workspace `Cargo.toml` updated.
- `cargo build / test / clippy -D warnings / fmt --check` all green.
- One pre-existing clippy lint fixed in `civ-engine` tests
  (`field-reassign-with-default`, exposed by the Rust 1.95 toolchain bump).
- Local `rustup stable` bumped 1.83 → 1.95 (proptest/blake3 transitive `edition2024`
  requirement).
- Docs addenda (this worklog + PRD-3d-extension + ADRs 005–008 + plan-3d-phases +
  fr-3d-additions).
- Memory entries saved: `project_civis_3d_pivot`, `feedback_codex_harness`.

## Open

- **Modding day-1 vs post-MVP** — RON tech cards loadable from `mods/` from P-L1
  onward? Owner to decide.
- **Unreal client EULA** — ship under Epic's standard EULA, or constrain to a
  shell-only renderer (no game logic) to dodge royalty surface?
- **LLM tick budget** — resolved as async-multi-tick per build.md.
- **WorldBox-style "anything spawnable" boundary** — kingdoms/factions post-hoc
  spawnable (assumed yes) or scenario-init only?

## Next iteration

1. Bootstrap `KooshaPari/phenotype-voxel` repo per
   `~/.claude/plans/civis-3d-scratch/phenotype-voxel-design.md` (P-V0).
2. Wire `crates/voxel` as a path-dep on the new repo (once available locally).
3. Begin FR-CIV-VOXEL-001 / 002 / 003 implementation (P-V1).

## Cross-references

- Plan: `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md`
- Build/progression design: `~/.claude/plans/civis-3d-scratch/build.md`
- Voxel kernel design: `~/.claude/plans/civis-3d-scratch/phenotype-voxel-design.md`
- PR: [#296](https://github.com/KooshaPari/Civis/pull/296)
