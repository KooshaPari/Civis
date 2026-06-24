# ADR-007: Three Reference 3D Clients (Bevy + Godot + Unreal) in Parallel

**Date:** 2026-05-22
**Status:** PROPOSED
**Author:** Civis 3D Extension

---

## Context

ADR (multi-client protocol — existing PRD) already commits CivLab to an engine-agnostic
WebSocket JSON-RPC + binary protocol with Bevy/Unreal/Unity/Godot/Web clients on the
roadmap. The 3D extension raises three new pressures:

1. **Visual bar: COD / Rust / The Finals tier** — PBR materials, runtime GI, Chaos-like
   destruction. Only Unreal 5 hits this off-the-shelf.
2. **Agent-drivability** — codex / claude / CI must be able to launch, replay,
   screenshot. Bevy (Rust, headless mode, in-process attach) is best; Unreal is hostile.
3. **UX iteration velocity** — WorldBox-style spawn editor needs fast prototyping.
   Godot 4's editor + scene tree is fastest by a wide margin.

No single engine wins on all three axes. Going single-engine forces a sub-optimal trade.

## Decision

Maintain **three reference 3D clients in parallel**, all attaching to the same
WebSocket protocol, all consuming the shared `crates/voxel` / `crates/build` /
`crates/protocol-3d` substrate:

- **`clients/bevy-ref` — Bevy (Rust).** Daily-driver for CI, deterministic replay
  verification, screenshot regression, agent-driven workflows. Visual quality below
  Unreal but improving (`bevy_pbr`, `bevy_solari` for RT GI in 0.15+).
- **`clients/godot-ref` — Godot 4 + GDExtension/Rust.** UX iteration surface.
  Spawn-anything UI, era timelapse, drag-place vehicles/buildings/airports. Voxel via
  `Zylann/godot_voxel` (Voxel Tools).
- **`clients/unreal-show` — Unreal 5.** Visual showcase + shipping client. Lumen +
  Nanite + Chaos. Lighter iteration cadence than the other two.

A shared Rust adapter at `clients/voxel-bridge` translates the deterministic dirty-queue
events from `phenotype-voxel` into each engine's mesh format.

## Consequences

- **No single engine constraint hits all three pressures.** Each axis has a primary
  client.
- **Shared substrate prevents logic divergence.** Sim core lives in Rust; engines are
  renderers and input transports only.
- **Maintenance is 3× on the renderer surface.** Mitigated by: (a) shared voxel-bridge
  and protocol, (b) lighter Unreal cadence, (c) Bevy gating CI so determinism
  regressions surface there first.
- **Unreal EULA exposure** — open question (see PRD addendum). If Unreal is held to
  shell-only (no game logic), royalty surface is minimised.

## Alternatives Considered

- **Single-engine, pick Bevy.** Loses the visual bar. Rejected because the owner
  explicitly raised the bar to COD / Rust tier.
- **Single-engine, pick Unreal.** Loses agent-drivability + UX iteration speed. Rejected
  because daily development would become painful.
- **Two-engine, drop Godot.** Defensible. Considered. Godot was kept because it owns
  the WorldBox-style spawn UX better than the other two and adds little to the build
  burden (GDExtension reuses the Rust core verbatim).
- **Two-engine, drop Unreal.** Defensible if the visual bar is later relaxed; revisit
  after first playable.

## Cross-references

- ADR-005 (adaptive voxel substrate) — defines the kernel all three consume.
- `docs/roadmap/civis-3d-extension.md` — feature matrix tying renderers to phases.
- Plan: `~/.claude/plans/weve-spent-a-lot-toasty-reddy.md` § Engine + renderer pick.
