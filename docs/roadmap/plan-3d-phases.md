# Civis 3D — Phase Plan (P-V0 → P-U1)

**Status:** IN PROGRESS (additive to `PLAN.md` Phases 0–6)
**Date:** 2026-05-22
**Last updated:** 2026-05-28
**Branch:** `feat/civis-3d-foundation`

> Per Civis `CLAUDE.md` — agent-time only; no calendar weeks.

---

## DAG

```
P-V0 phenotype-voxel kernel (new shared Phenotype-org repo)
  └─> P-V1 voxel foundation (crates/voxel, protocol-3d, all 3 renderers stand up empty)
        └─> P-V2 building substrate (crates/build; procedural + freehand)
              └─> P-A1 civilian agents (crates/agents + diffusion)
                    └─> P-U1 WorldBox UX (clients/godot-ref spawn editor)
P-V0 ──┐
       │
       └─> (parallel) P-G1 genetics + species (crates/genetics + species)
                       └─> P-A1
P-V1 ──┐
       └─> P-W1 tactical warfare (crates/tactics; depends on voxel + agents)
P-L1 physics-law DB (crates/laws)
  └─> P-R1 hybrid research (crates/research; depends on laws + agents)
P-P1 planet + moon (crates/planet) — can run anytime after P-V1
```

Existing PLAN.md Phases 0–6 (foundation → economy → actors → protocol → war → research → polish)
remain prerequisites and run in parallel where their existing DAG permits. The 3D phases
hook into them at the seams (e.g. P-A1 extends the Phase 2 actor system; P-W1 extends
the Phase 4 war system).

## Phases

| Phase | Crate(s) | FR namespace | Depends on | Acceptance | Status |
|---|---|---|---|---|---|
| **P-V0** Phenotype voxel kernel | (new repo `phenotype-voxel`) | n/a | — | New repo green; SVO + dense leaf storage; deterministic dirty queue; `Mesher` trait; Bevy reference adapter. | COMPLETE |
| **P-V1** Voxel foundation | `civ-voxel`, `civ-protocol-3d`, `clients/voxel-bridge` | FR-CIV-VOXEL-* | P-V0 | Adaptive substrate wired into the engine tick; protocol carries voxel deltas; all three reference clients render empty terrain at 60 FPS. | COMPLETE |
| **P-W1** Tactical warfare | `civ-tactics` | FR-CIV-TACTICS-* | P-V1, P-A1 | Voxel-destructible per-soldier combat; doctrine evolution genetic-algo; integration with existing Phase 4 war system. | COMPLETE |
| **P-V2** Building substrate | `civ-build` | FR-CIV-BUILD-* | P-V1 | `BuildingGraph` schema; era grammars (mud-brick → arcology); freehand authoring tools (paint/extrude/loft/mirror/radial/copy-rotate); autonomous demand-driven allocation; all three clients render procedural blocks. | PLANNED |
| **P-G1** Genetics + species | `civ-genetics`, `civ-species` | FR-CIV-GENETICS-*, FR-CIV-SPECIES-* | (independent) | DNA byte-vectors; mutation + recombination + fitness; speciation thresholds; deterministic DNA → phenotype mapping; WorldBox-style spawn API exposed via protocol. | PLANNED |
| **P-A1** Civilian agents | `civ-agents`, `civ-diffusion` | FR-CIV-AGENTS-*, FR-CIV-DIFFUSION-* | P-V2, P-G1 | Utility-AI + GOAP + BT layered tick; LOD-aware sim; per-civilian wardrobe + tools state; Bass/Rogers S-curve adoption; civilians visibly age across eras. | PLANNED |
| **P-L1** Physics-law DB | `civ-laws` | FR-CIV-LAWS-* | (independent) | Versioned RON schema; validator; era unlock graph; futurism extension rules typed as `kind: fictional_extension`. | PLANNED |
| **P-R1** Hybrid research | `civ-research` | FR-CIV-RESEARCH-* | P-L1, P-A1 | LLM proposal pipeline; hash-keyed cache (blake3); Canonical-mode refusal path; replay determinism holds (ADR-006). | PLANNED |
| **P-P1** Planet + moon | `civ-planet` | FR-CIV-PLANET-* | P-V1 | Geology, weather, day/night cycle, moon tides — deterministic. | PLANNED |
| **P-U1** WorldBox UX | `clients/godot-ref` | FR-CIV-UX-* | P-A1, P-V2 | Spawn-anything UI; era timelapse view; drag-place vehicles/buildings/airports/hangars/ports. | PLANNED |

## Per-phase workflow

Each phase ships as a **stacked PR series** on `feat/civis-3d-foundation` (or its child
branches), per Civis `PHENOTYPE_GIT_DELIVERY_PROTOCOL`:

1. **Test-first.** Failing FR-traceable tests land first (`cargo test` reds).
2. **Implementation.** Code lands to turn tests green.
3. **Docs.** README + module docstrings + relevant `docs/` updates.
4. **Quality gates.** `cargo clippy -- -D warnings`, `cargo fmt --check`, `task quality`,
   determinism replay test, FR coverage.
5. **CI.** All required checks green per CI Completeness Policy.

Phase boundaries are PR groups; each PR within a phase is independently mergeable.

## Cross-references

- `docs/roadmap/civis-3d-extension.md` — PRD addendum (feature matrix).
- ADR-005..008.
- `docs/development-guide/fr-3d-additions.md` — FR stubs.
- Plan file: `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md`.
