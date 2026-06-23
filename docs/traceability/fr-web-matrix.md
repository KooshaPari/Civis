# Web Spectator Traceability Matrix

**Status:** Closed — all `FR-CIV-WEB-000`..`008` implemented (2026-05-25).
**ADR:** [ADR-009-web-client-strategy](../adr/ADR-009-web-client-strategy.md)
**FR source:** [fr-web-spectator.md](../development-guide/fr-web-spectator.md)
**Owner path:** `web/dashboard/` (shared helpers in `web/src/`)

## Verify

From repo root:

```powershell
cd web && npm test          # 93 unit tests — web/tests/*.test.mjs
cd web && npm run build     # dashboard production build (vite)
```

`just civis-3d-verify` does **not** run web tests; use the commands above in PRs that touch `web/`.

---

| FR ID | Status | Source (primary) | Test / evidence |
|-------|--------|------------------|-----------------|
| FR-CIV-WEB-000 | implemented | `web/dashboard/` (`vite`, `tsc`) | `web/package.json` — `npm test`; `npm run build` → `dashboard` build |
| FR-CIV-WEB-001 | implemented | `web/src/wsUrl.mjs` | `web/tests/wsUrl.test.mjs`, `wsUrlQuery.test.mjs` — `resolveWsUrlFromEnv`, `?ws=` override, default `ws://127.0.0.1:3000/ws` |
| FR-CIV-WEB-002 | implemented | `web/dashboard/src/hooks/useCivisAttach.ts`, `web/src/civRpc.mjs` | `web/tests/civRpc.test.mjs` (`health`, `sim.snapshot`); `connectionStatus.test.mjs` (`buildHealthProbe`); `dashboardConnectionStatus.test.mjs`; `mergeSnapshot.test.mjs` |
| FR-CIV-WEB-003 | implemented | `web/dashboard/src/scene3d.tsx`, `scene_view.tsx` | `web/tests/snapshotView.test.mjs` (`sceneEntityCounts`); read-only scene path (no sim mutation in view layer) |
| FR-CIV-WEB-004 | implemented | `web/dashboard/src/bottom_bar.tsx`, `scene3d.tsx` | `web/tests/civRpc.test.mjs` (RPC envelope); UI: `sim.set_speed`, `sim.command` tick — integration via `civ-server` `ws_smoke` when changing server RPC |
| FR-CIV-WEB-005 | implemented | `web/dashboard/src/bottom_bar.tsx` | HTTP `/replay/export`, `/replay/import` on watch attach; no dedicated roundtrip unit test (manual or server harness) |
| FR-CIV-WEB-006 | implemented | `web/src/frame3d.mjs`, `useCivisAttach.ts` binary handler | `web/tests/frame3d.test.mjs` — decode `F3D0`, `parseWsPayload`, voxel chunk ids; read-only decode |
| FR-CIV-WEB-007 | implemented | `web/dashboard/src/babylon_scene.tsx`, `scene_view.tsx` | `web/tests/rendererMode.test.mjs` — `?renderer=babylon`, `CIVIS_RENDERER`, Three fallback |
| FR-CIV-WEB-008 | implemented | `web/dashboard/src/lib/authoring.ts`, `web/src/authoringMode.mjs` | `web/tests/authoringMode.test.mjs` (`?spectator=1`, `?authoring=0`); `spawnRouting.test.mjs`; `watchAttach.test.mjs` (control routes) |

---

*Last updated: 2026-05-25. Reopen a row only when acceptance criteria in `fr-web-spectator.md` change or a test is removed.*
