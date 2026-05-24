# ADR-009: Web Client Strategy — Spectator-First (Not a Fourth Game Engine)

**Date:** 2026-05-24
**Status:** ACCEPTED
**Author:** Civis Architecture
**Supersedes:** Implicit “web as v1 game client” reading of PRD E6.4

---

## Context

CivLab is a **headless Rust simulation** with engine-agnostic attach via WebSocket JSON-RPC and
`F3D0` binary frames ([ADR-007](./ADR-007-three-renderers.md)). Three **game** reference
clients are already committed:

| Client | Role |
|--------|------|
| `clients/godot-ref` | WorldBox-style spawn UX, fast iteration (P-U1) |
| `clients/bevy-ref` | CI, determinism, screenshot regression, agent workflows |
| `clients/unreal-show` | Visual showcase (Lumen / Nanite / Chaos) |

`web/dashboard` today is a **Three.js telemetry viewport** (planes, boxes, orbit camera).
It is not—and should not be mistaken for—the shippable builder/sandbox game.

Four options were evaluated for long-term web strategy:

| Option | Summary |
|--------|---------|
| **A** | Godot HTML5 export — one UX codebase |
| **B** | Babylon.js / PlayCanvas — full browser game client |
| **C** | Web = spectator / ops / replay only; game ships in Godot + Unreal (+ Bevy ref) |
| **D** | Bevy → WASM on canvas — Rust-aligned browser game |

## Decision

**Adopt C as the canonical long-term web strategy.**

The browser surface is a **spectator and operations client**, not a fourth game engine:

- Live metrics, policy controls, timelapse, multi-client observation
- `.civreplay` import/export, hash-chain verification UI
- Optional **read-only** 3D view fed by `sim.snapshot` + `F3D0` streams (quality may improve
  with Babylon.js **only as a renderer**, not as a second gameplay codebase)

**Primary playable game** remains **Godot (desktop)** first, then **Unreal** for visual
milestone; **Bevy** remains engineering reference, not retail SKU.

### Optional later (explicitly secondary)

| When | What | Not a replacement for |
|------|------|------------------------|
| Godot P-U1 playable on desktop | Revisit **A** only if GDExtension-free web build exists | C |
| Spectator 3D quality insufficient | **B subset** — Babylon viewer module, no placement/spawn UX | Full B game client |
| Never default | **D** full Bevy WASM game | Godot UX path |

## Rationale

### Why C wins (robustness)

1. **Aligns with ADR-007** — Three game renderers already imply 3× maintenance. A fourth
   full game stack in TypeScript duplicates Godot’s WorldBox UX investment.
2. **Architecture fit** — The sim is headless; web’s natural strength is dashboards,
   replay analysis, streaming, and “attach many viewers to one timeline”—already in the PRD
   multi-client diagram.
3. **Determinism & CI** — Gameplay correctness is proven on Bevy headless + `.civreplay`;
   web does not need to re-implement placement, GOAP, or voxel authoring.
4. **Product honesty** — Manor Lords / CS2 / WorldBox **feel** requires native clients, art,
   input, and audio—not a better box shader in Three.js.

### Why not A (Godot HTML5) as default

- `clients/godot-ref` uses **GDExtension (Rust)**. Godot 4 **web export does not support
  GDExtension** the same way as desktop; the current client cannot ship to HTML5 without a
  **separate** GDScript-only or HTTP-only web target.
- WASM download size, audio/threading, and export QA add cost without beating desktop Godot
  for the first playable loop.

### Why not B (Babylon / PlayCanvas) as full game

- Rebuilds spawn editor, building tools, and camera UX already owned by Godot (P-U1).
- Highest long-term **duplication risk** and divergent gameplay bugs vs Godot/Unreal.

### Why not D (Bevy WASM)

- Bevy on WASM is improving but still **immature** for a full sandbox (load time, mobile,
  tooling). Duplicates `bevy-ref` while Godot is the designated UX engine.
- Reasonable for **embedded debug panels**, not the canonical game SKU.

## Consequences

- **PRD v1 “Web client”** means **reference spectator + protocol conformance**, not feature
  parity with Godot.
- **`web/dashboard`** may upgrade rendering (e.g. Babylon viewer) under **FR-CIV-WEB-***
  without gaining building/spawn mechanics.
- **Acceptance tests** for web focus on: WS connect, snapshot fields, replay roundtrip UI,
  read-only `F3D0` decode/display—not “60 FPS builder loop.”
- **Godot desktop** is the gating client for P-U1 / FR-CIV-UX-*.

## Alternatives considered

| Alternative | Rejected because |
|-------------|------------------|
| A default | GDExtension blocks current godot-ref on web |
| B default | Fourth game codebase |
| D default | Immature + duplicates Bevy without UX win |

## Cross-references

- [ADR-007](./ADR-007-three-renderers.md) — Bevy / Godot / Unreal split
- `docs/roadmap/plan-3d-phases.md` — P-U1 Godot UX
- `docs/development-guide/fr-web-spectator.md` — FR stubs (spectator acceptance)
- `docs/traceability/fr-3d-matrix.md` — UX rows remain Godot-owned
