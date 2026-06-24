# Civis Spec-Map Requirements (gap-fill FR/NFR)

These FR/NFR are derived from [`../feature-matrix.md`](../feature-matrix.md) and fill the **BLIND / INCOMPLETE / UNPOLISHED** gaps the matrix flags. They use new domain codes + high number ranges (900+) to avoid colliding with the existing `FR-CIV-*` catalog (see `docs/development-guide/fr-3d-additions.md` and root `FUNCTIONAL_REQUIREMENTS.md`).

Every requirement traces: **PRD → Feature Matrix → FR/NFR → Acceptance Criteria → test → client/crate**, and tags the **emergent-not-hardcoded** constraint per the [Emergence Charter](../../guides/emergence-charter.md).

| Doc | Domain | Fills (matrix gap) | Primary owner |
|---|---|---|---|
| [FR-CIV-INFOVIEW](./FR-CIV-INFOVIEW.md) | Info-view overlays | §7 BLIND (#1 gap) | UI |
| [FR-CIV-INSPECT](./FR-CIV-INSPECT.md) | Inspect-anything + tooltips | §6/§7 BLIND | UI |
| [FR-CIV-PSYCHE](./FR-CIV-PSYCHE.md) | Psyche, social graph, emergent history | §2 BLIND | Life-Sim |
| [FR-CIV-GODTOOL](./FR-CIV-GODTOOL.md) | God-tool palette + sandbox | §6 INCOMPLETE/BLIND | UI + Voxel-Render |
| [FR-CIV-ROAD](./FR-CIV-ROAD.md) | Roads/desire-paths, vehicles, architecture | §4 BLIND/INCOMPLETE | Infrastructure |
| [FR-CIV-NOTIFY](./FR-CIV-NOTIFY.md) | Notifications, stats, onboarding, hotkeys | §7 BLIND | UI |
| [NFR-CIV-SCALE-PERF](./NFR-CIV-SCALE-PERF.md) | 20mi scale + streaming + 60fps | §1/§8 INCOMPLETE | Voxel-Render + Perf |

**Count:** 7 spec docs; **40 FR** + **8 NFR** = **48 gap-fill requirements**, each with acceptance criteria.

Reference teardowns backing these: see [`../../research/`](../../research/) (cities-skylines, worldbox, empire-at-war-foc, call-to-arms, dwarf-fortress-rimworld, songs-of-syx, manor-lords, civ-oldworld-4x, spore-black-and-white, bevy-ecosystem-reference).
