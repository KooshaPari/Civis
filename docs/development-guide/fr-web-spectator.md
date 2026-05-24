# FR-CIV-WEB — Browser Spectator & Operations Client

**Status:** IN PROGRESS (ADR-009 accepted; FR-CIV-WEB-000..006 landed in `web/`)
**ADR:** [ADR-009-web-client-strategy](../adr/ADR-009-web-client-strategy.md)
**Owner path:** `web/dashboard/`

> The web client is **not** the shippable builder/sandbox game. Gameplay UX lives in
> `clients/godot-ref` (P-U1) and visual polish in `clients/unreal-show`. Web proves protocol
> attachment, replay tooling, and live observation.

---

## Scope

| In scope | Out of scope |
|----------|----------------|
| Connect to `civ-server` WS (`/ws`) or read-only `civ-watch` HTTP | Spawn editor, freehand build, voxel write tools |
| Display `sim.snapshot` metrics | Feature parity with Godot P-U1 |
| Import/export `.civreplay` via HTTP or RPC | Full PBR / Manor Lords visual bar |
| Read-only 3D terrain/agents/buildings from snapshot or `F3D0` | Second gameplay ruleset in TypeScript |

---

## Requirements

| FR ID | Requirement | Acceptance criterion |
|-------|-------------|---------------------|
| **FR-CIV-WEB-000** | Dashboard builds and runs (`vite` dev / production build). | `npm test` in `web/` passes; `web/dashboard` builds without error. |
| **FR-CIV-WEB-001** | Resolve WS URL from env (`CIVIS_WS_URL`, `CIVIS_WS_ADDR`) with documented default. | Unit test `resolveWsUrlFromEnv` passes; default `ws://127.0.0.1:3000/ws`. |
| **FR-CIV-WEB-002** | On connect, call `health` and `sim.snapshot`; surface tick, population, economy fields in UI. | E2E or component test: mock WS returns snapshot JSON; UI shows tick ≥ 0. |
| **FR-CIV-WEB-003** | Read-only 3D view: terrain biomes + building/agent proxies from snapshot (no sim mutation). | Visual regression optional; unit test: snapshot → scene object counts > 0 for fixture data. |
| **FR-CIV-WEB-004** | Operator controls limited to RPC already on server: `sim.command` tick/noop, `sim.set_speed`, `sim.set_policy`, `sim.reset` (with role header when required). | Integration test against `civ-server` smoke harness or mocked RPC; no custom game commands. |
| **FR-CIV-WEB-005** | Replay: trigger `sim.save_replay` / load via `sim.load_replay` or `POST /replay/import`; show success/error. | Roundtrip test: save → load → snapshot tick matches within spec. |
| **FR-CIV-WEB-006** | (Optional P2) Decode `F3D0` binary WS frames for smoother voxel deltas; still read-only. | Unit test: decode sample `F3D0` fixture; no encode/write path. |
| **FR-CIV-WEB-007** | (Optional P2) Babylon.js viewer module replaces raw Three.js **rendering only**; same FR-CIV-WEB-003 data contract. | `?renderer=babylon`; falls back to Three if load fails; `web/src/rendererMode.mjs` tests. |

---

## Non-goals (explicit)

- HTML5 Godot export (see ADR-009 § Alternatives)
- Bevy WASM game client
- PlayCanvas/Babylon as authoritative gameplay layer

---

## Traceability

Add rows to `docs/traceability/fr-3d-matrix.md` or a dedicated `fr-web-matrix.md` when
implementing. Status `planned` until each acceptance test exists.
