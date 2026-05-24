# Web Spectator Traceability Matrix

**ADR:** [ADR-009-web-client-strategy](../adr/ADR-009-web-client-strategy.md)  
**FR source:** [fr-web-spectator.md](../development-guide/fr-web-spectator.md)

| FR ID | Status | Test / evidence |
|-------|--------|-----------------|
| FR-CIV-WEB-000 | implemented | `web/package.json` `npm run build`; `web/dashboard` vite build |
| FR-CIV-WEB-001 | implemented | `web/tests/wsUrl.test.mjs` |
| FR-CIV-WEB-002 | implemented | `useCivisAttach.ts` — `health` + `sim.snapshot` on WS open |
| FR-CIV-WEB-003 | implemented | `web/tests/snapshotView.test.mjs`; `scene3d.tsx` read-only |
| FR-CIV-WEB-004 | implemented | `bottom_bar.tsx` — `sim.set_speed`, `sim.command` tick |
| FR-CIV-WEB-005 | implemented | `bottom_bar.tsx` — `/replay/export`, `/replay/import` |
| FR-CIV-WEB-006 | implemented | `web/tests/frame3d.test.mjs`; `useCivisAttach` binary handler |
| FR-CIV-WEB-007 | partial | `?renderer=babylon`, `babylon_scene.tsx` + Three fallback |
