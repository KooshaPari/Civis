# Product quality ladder (CivLab → Manor Lords class)

**Status:** Planning reference (not a calendar commitment)  
**Audience:** Contributors choosing where to invest after MVP

This document answers “what does Manor Lords–grade mean for CivLab?” without implying the web client or Q3/Q4 2026 MVP delivers that bar.

---

## Ladder (what “grade” means)

| Tier | Player-facing bar | CivLab stack today | Gating work |
|------|-------------------|--------------------|-------------|
| **L0 Protocol** | Headless sim + attach clients | **Here** — `civ-server`, replay, F3D0, multi-client | CIV-0200 hardening, role model |
| **L1 Observer** | Readable world + metrics | **Here** — web spectator (ADR-009), Godot/Unreal terrain | `sim.snapshot` spectator fields, institutions wire |
| **L2 Sandbox** | Spawn / paint / timelapse | **Partial** — Godot P-U1 stubs (FR-CIV-UX-000/001), civ-watch controls | Server-side spawn RPC, voxel write path, P-V2 build tools |
| **L3 City sim** | Autonomous citizens, economy loops | **Partial** — agents, economy institutions stub, buildings in spectator view | P-A1 GOAP depth, district sim, UI for institutions |
| **L4 Strategy** | Factions, war, research arcs | **Stubs** — tactics, laws, research crates | P-W1, P-L1, P-R1 integration with engine tick |
| **L5 Polish** | Manor Lords / CS2 *feel* | **Not started** as shippable product | Art direction, animation, audio, UX density, content |

**Manor Lords–grade ≈ L5 polish on top of L3–L4 simulation depth**, not a single renderer swap.

---

## Client roles (ADR-007 / ADR-009)

| Client | Target tier | Notes |
|--------|-------------|-------|
| **Godot ref** | L2 → L3 | WorldBox spawn editor (P-U1); default attach `civ-server` |
| **Web dashboard** | L1 only | Spectator/ops; optional Babylon **rendering** (FR-CIV-WEB-007), not gameplay |
| **Bevy ref** | L0–L1 | CI + protocol conformance |
| **Unreal show** | L1 → L5 visuals | Showcase; lighter cadence than Godot |
| **Unity** | — | PRD placeholder; no tree yet |

---

## Recommended sequence (agent-time phases)

1. **Finish L2** — spawn/build RPC on `civ-server`, Godot authoring against one timeline (not split watch/server for mutations).
2. **Deepen L3** — wardrobe/tools visible, job UX, building placement from `civ-build`.
3. **Prove L4** — one vertical slice (e.g. market + skirmish) with replay determinism.
4. **Invest L5** — only after L3 loop is fun in Godot desktop; Unreal for cinematics/marketing.

---

## Anti-patterns

- Rebuilding spawn/build in the **browser** (ADR-009).
- Treating **Godot HTML5** as the shippable game (GDExtension blocks web export).
- Expecting **Babylon** alone to raise tier — it only changes L1 rendering.

---

## Cross-links

- [plan-3d-phases.md](./plan-3d-phases.md) — P-V0..P-U1 engineering DAG  
- [fr-p-u1-roadmap.md](../development-guide/fr-p-u1-roadmap.md) — remaining WorldBox UX FRs  
- [ADR-009](../adr/ADR-009-web-client-strategy.md) — web scope ceiling
