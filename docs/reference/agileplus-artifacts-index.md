# AgilePlus Artifacts Index — Civis

**Generated**: 2026-05-29
**Branch**: feat/civis-bevy-game
**Spec root**: `agileplus-specs/`
**Format**: AgilePlus native (`meta.json` + `spec.md` + `plan.md` per artifact)

---

## CLI Status

The `agileplus` CLI is **not available** in the system PATH. The CLI binary lives at
`C:/Users/koosh/Dev/AgilePlus/agileplus/` but is not installed or on PATH. Artifacts
were authored directly in AgilePlus native format (three-file pattern: `meta.json`,
`spec.md`, `plan.md`) matching the structure observed in
`C:/Users/koosh/Dev/AgilePlus/kitty-specs/`.

---

## Format Reference

Each artifact follows:

```
agileplus-specs/<slug>/
  meta.json   — machine-readable: spec_id, slug, title, status, epic, fr_ids, priority, target_release
  spec.md     — YAML frontmatter + Problem Statement + FRs + ACs + Status
  plan.md     — Phased WBS + DAG dependency table
```

---

## Artifacts

### civ-001 — Core Simulation Engine

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-001-core-simulation-engine/` |
| **Epic** | E1 |
| **FR IDs** | FR-CORE-001, FR-CORE-002, FR-CORE-003, FR-CORE-004, FR-CORE-005, FR-CORE-006, FR-CORE-007 |
| **Priority** | SHALL |
| **Target Release** | MVP |
| **Status** | active / IN_PROGRESS |
| **Civ Spec Ref** | `docs/specs/CIV-0001-core-simulation-loop.md` |

**Stories**: Fixed-timestep tick loop (E1.1), ECS entity model (E1.2), policy evaluation phase (E1.3), deterministic transition (E1.4), stochastic event phase (E1.5), state serialization (E1.6), multi-client command queue (E1.7), .civreplay export (E1.8), determinism CI gate (E1.9), performance profiling (E1.10).

---

### civ-002 — Economy and Joule System

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-002-economy-joule-system/` |
| **Epic** | E2 |
| **FR IDs** | FR-ECON-001, FR-ECON-002, FR-ECON-003, FR-ECON-004, FR-ECON-005, FR-METRICS-001, FR-METRICS-002, FR-METRICS-003 |
| **Priority** | SHALL |
| **Target Release** | MVP |
| **Status** | active / IN_PROGRESS |
| **Civ Spec Ref** | `docs/specs/CIV-0100-economy-v1.md`, `CIV-0107-joule-economy-system-v1.md` |

**Stories**: Production system (E2.1), inventory management (E2.2), market clearing (E2.3), Joule accounting (E2.4), allocation algorithm (E2.5), taxation (E2.6), budget system (E2.7), legitimacy model (E2.8), property testing (E2.9), stress testing (E2.10). Metrics struct + computation + fixed-point (FR-METRICS-*).

---

### civ-003 — Actor and Citizen Lifecycle

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-003-actor-citizen-lifecycle/` |
| **Epic** | E2 |
| **FR IDs** | FR-CIV-ACTOR-001, FR-CIV-ACTOR-002, FR-CIV-SOCIAL-001, FR-CIV-SOCIAL-002 |
| **Priority** | SHALL |
| **Target Release** | v1 |
| **Status** | active / PLANNED |
| **Civ Spec Ref** | `docs/specs/CIV-0103-institutions-timeseries-citizen-lifecycle-v1.md` |

**Stories**: Citizen lifecycle state machine (P2.1–P2.2), institution system (P2.3–P2.4), social ideology (P2.5–P2.6).

---

### civ-004 — Building Tiers and Production Chains

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-004-building-tiers/` |
| **Epic** | E2 |
| **FR IDs** | FR-ECON-001, FR-CIV-BUILD-001, FR-CIV-BUILD-002, FR-CIV-BUILD-003 |
| **Priority** | SHALL |
| **Target Release** | v1 |
| **Status** | active / PLANNED |
| **Civ Spec Ref** | `docs/specs/CIV-0001-core-simulation-loop.md` |

**Stories**: Building tier enum (B1.1–B1.2), production chain with halt logic (B2.1–B2.3), scenario YAML building schema (B3.1–B3.2).

---

### civ-005 — Climate, Disasters, and Seasons

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-005-climate-disasters-seasons/` |
| **Epic** | E2 |
| **FR IDs** | FR-CIV-CLIMATE-001, FR-CIV-CLIMATE-002, FR-CIV-CLIMATE-003 |
| **Priority** | SHALL |
| **Target Release** | v1 |
| **Status** | active / PLANNED |
| **Civ Spec Ref** | `docs/specs/CIV-0102-climate-followup-v1.md` |

**Stories**: Season calendar (C1.1–C1.3), stochastic disaster events (C2.1–C2.3), disaster effects on production and citizen health (C3.1–C3.3).

---

### civ-006 — Deep Combat System

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-006-deep-combat/` |
| **Epic** | E4 |
| **FR IDs** | FR-CIV-WAR-001, FR-CIV-WAR-002, FR-CIV-WAR-003, FR-CIV-WAR-004 |
| **Priority** | SHALL |
| **Target Release** | v1 |
| **Status** | active / PLANNED |
| **Civ Spec Ref** | `docs/specs/CIV-0105-war-diplomacy-shadow-v1.md` |

**Stories**: Military unit entity (E4.1 / W1.1–W1.2), combat resolution (E4.2 / W2.1–W2.3), casualty handling + territory (E4.3, E4.6 / W3.1–W3.3), battle replay CI test (E4.8 / W4.1).

---

### civ-007 — Diplomacy, Laws, and Government

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-007-diplomacy-laws-government/` |
| **Epic** | E4 |
| **FR IDs** | FR-CIV-DIPLO-001, FR-CIV-DIPLO-002, FR-CIV-DIPLO-003, FR-CIV-GOV-001, FR-CIV-GOV-002 |
| **Priority** | SHALL |
| **Target Release** | v1 |
| **Status** | active / PLANNED |
| **Civ Spec Ref** | `docs/specs/CIV-0105-war-diplomacy-shadow-v1.md` |

**Stories**: Diplomatic FSM 8-state (E4.4–E4.7 / D2.1–D2.3), influence capital (D3.1–D3.2), shadow networks (D4.1–D4.3), government type enum (D1.1), laws RON stubs (D1.2), threshold metrics L₀/E*/C₀ (D5.1).

---

### civ-008 — Genetics and Species Diversity

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-008-genetics-species/` |
| **Epic** | E2 |
| **FR IDs** | FR-CIV-BIO-001, FR-CIV-BIO-002, FR-CIV-BIO-003 |
| **Priority** | SHOULD |
| **Target Release** | v2 |
| **Status** | active / PLANNED |
| **Civ Spec Ref** | `docs/specs/CIV-0103-institutions-timeseries-citizen-lifecycle-v1.md` |

**Stories**: Species type enum + YAML schema (G1.1–G1.2), genetic trait inheritance + mutation (G2.1–G2.3), trait-simulation couplings (strength/intelligence/longevity/disease_resistance) (G3.1–G3.4).

---

### civ-009 — Culture Diffusion and Ideology Spread

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-009-culture-diffusion/` |
| **Epic** | E2 |
| **FR IDs** | FR-CIV-CULT-001, FR-CIV-CULT-002, FR-CIV-CULT-003 |
| **Priority** | SHOULD |
| **Target Release** | v2 |
| **Status** | active / PLANNED |
| **Civ Spec Ref** | `docs/specs/CIV-0106-social-ideology-health-insurgency-v1.md` |

**Stories**: Culture entity (CU1.1–CU1.2), diffusion mechanics via adjacency/contact intensity (CU2.1–CU2.3), ideology convergence with legitimacy feedback (CU3.1–CU3.2).

---

### civ-010 — Multi-Client Protocol

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-010-multi-client-protocol/` |
| **Epic** | E3 |
| **FR IDs** | FR-PROTO-001, FR-PROTO-002, FR-PROTO-003, FR-PROTO-004, FR-PROTO-005, FR-CLIENT-003 |
| **Priority** | SHALL |
| **Target Release** | v1 |
| **Status** | active / IN_PROGRESS |
| **Civ Spec Ref** | `docs/specs/CIV-0200-client-protocol.md` |

**Stories**: RFC 6455 WebSocket server (E3.1 / PR—WS), JSON-RPC 2.0 dispatcher (E3.2), handshake + bootstrap (E3.3 / PR1.1–PR1.2), binary frames (E3.6 / PR2.1–PR2.3), snapshot filtering (E3.5, E3.8 / PR3.1–PR3.3), role authorization (E3.7 / PR4.1–PR4.2), performance gate (E3.10 / PR5.1).

---

### civ-011 — Bevy Primary Client (3D, DX12/DLSS)

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-011-bevy-primary-client/` |
| **Epic** | E7 (new) |
| **FR IDs** | FR-CLIENT-001, FR-CIV-HUD-001, FR-CIV-HUD-002, FR-CIV-HUD-003, FR-CIV-HUD-004, FR-CIV-HUD-005 |
| **Priority** | SHALL |
| **Target Release** | MVP |
| **Status** | active / IN_PROGRESS |
| **Client Path** | `clients/bevy-ref` |
| **Civ Spec Ref** | `docs/specs/CIV-0300-rts-ui-ux-spec.md`, `CIV-0601-3d-asset-transition-and-agentic-gen-spec.md` |

**Stories**: Server connection + tick delta rendering (B1.1–B1.3), 3D voxel world rendering + DLSS (B2.1–B2.3), click-to-build (B3.1–B3.2), tool palette HUD (B4.1), tech tree HUD (B4.2), diplomacy panel HUD (B4.3), event feed HUD (B4.4), menu system (B4.5).

**Notes**: Bevy 0.18 with `default-features = true` mandatory (tonemapping LUTs). DX12 Ultimate + DXR + DLSS on RTX 3090 Ti; Metal fallback on M1. Web client DEPRECATED.

---

### civ-012 — Godot Secondary Client

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-012-godot-secondary-client/` |
| **Epic** | E8 (new) |
| **FR IDs** | FR-CIV-CLIENT-GODOT-001, FR-CIV-CLIENT-GODOT-002 |
| **Priority** | SHOULD |
| **Target Release** | v2 |
| **Status** | active / PLANNED |
| **Client Path** | `clients/godot-ref` (to be created) |
| **Civ Spec Ref** | `docs/specs/CIV-0200-client-protocol.md` |

**Stories**: GDScript WebSocket connection + handshake (GO1.1–GO1.2), 3D scene rendering at strategic zoom (GO2.1–GO2.2), minimal HUD (GO3.1–GO3.2), protocol conformance test vs Bevy (GO4.1).

**Notes**: Supersedes deprecated web/TypeScript client as secondary validation target. Consumes identical wire protocol as Bevy client.

---

### civ-013 — Research API and Scenario System

| Field | Value |
|-------|-------|
| **Path** | `agileplus-specs/civ-013-research-api/` |
| **Epic** | E5 |
| **FR IDs** | FR-API-001, FR-API-002, FR-API-003, FR-API-004, FR-REPLAY-001, FR-REPLAY-002 |
| **Priority** | SHALL |
| **Target Release** | v1 |
| **Status** | active / IN_PROGRESS |
| **Civ Spec Ref** | `docs/specs/CIV-0001-core-simulation-loop.md` |

**Stories**: Scenario YAML + CI schema validation (RA1.1–RA1.2), .civreplay SHA-256 + CI replay gate (RA2.1–RA2.2), Python scenario runner + policy overrides (RA3.1–RA3.3), data export CSV/JSON (RA4.1–RA4.3).

---

## Traceability Matrix

| FR ID | AgilePlus Artifact | Epic | Status |
|-------|--------------------|------|--------|
| FR-CORE-001 | civ-001 | E1 | Partial |
| FR-CORE-002 | civ-001 | E1 | Partial |
| FR-CORE-003 | civ-001 | E1 | Partial |
| FR-CORE-004 | civ-001 | E1 | Planned |
| FR-CORE-005 | civ-001 | E1 | Planned |
| FR-CORE-006 | civ-001 | E1 | Planned |
| FR-CORE-007 | civ-001 | E1 | Planned |
| FR-ECON-001 | civ-002, civ-004 | E2 | Partial |
| FR-ECON-002 | civ-002 | E2 | Planned |
| FR-ECON-003 | civ-002 | E2 | Partial |
| FR-ECON-004 | civ-002 | E2 | Planned |
| FR-ECON-005 | civ-002 | E2 | Planned |
| FR-METRICS-001 | civ-002 | E2 | Partial |
| FR-METRICS-002 | civ-002 | E2 | Partial |
| FR-METRICS-003 | civ-002 | E2 | Planned |
| FR-CIV-ACTOR-001 | civ-003 | E2 | Planned |
| FR-CIV-ACTOR-002 | civ-003 | E2 | Planned |
| FR-CIV-SOCIAL-001 | civ-003 | E2 | Planned |
| FR-CIV-SOCIAL-002 | civ-003 | E2 | Planned |
| FR-CIV-BUILD-001 | civ-004 | E2 | Planned |
| FR-CIV-BUILD-002 | civ-004 | E2 | Partial |
| FR-CIV-BUILD-003 | civ-004 | E2 | Partial |
| FR-CIV-CLIMATE-001 | civ-005 | E2 | Partial |
| FR-CIV-CLIMATE-002 | civ-005 | E2 | Planned |
| FR-CIV-CLIMATE-003 | civ-005 | E2 | Planned |
| FR-CIV-WAR-001 | civ-006 | E4 | Planned |
| FR-CIV-WAR-002 | civ-006 | E4 | Planned |
| FR-CIV-WAR-003 | civ-006 | E4 | Planned |
| FR-CIV-WAR-004 | civ-006 | E4 | Planned |
| FR-CIV-DIPLO-001 | civ-007 | E4 | Planned |
| FR-CIV-DIPLO-002 | civ-007 | E4 | Planned |
| FR-CIV-DIPLO-003 | civ-007 | E4 | Planned |
| FR-CIV-GOV-001 | civ-007 | E4 | Planned |
| FR-CIV-GOV-002 | civ-007 | E4 | Partial |
| FR-CIV-BIO-001 | civ-008 | E2 | Planned |
| FR-CIV-BIO-002 | civ-008 | E2 | Planned |
| FR-CIV-BIO-003 | civ-008 | E2 | Planned |
| FR-CIV-CULT-001 | civ-009 | E2 | Planned |
| FR-CIV-CULT-002 | civ-009 | E2 | Planned |
| FR-CIV-CULT-003 | civ-009 | E2 | Planned |
| FR-PROTO-001 | civ-010 | E3 | Partial |
| FR-PROTO-002 | civ-010 | E3 | Partial |
| FR-PROTO-003 | civ-010 | E3 | Planned |
| FR-PROTO-004 | civ-010 | E3 | Planned |
| FR-PROTO-005 | civ-010 | E3 | Planned |
| FR-CLIENT-003 | civ-010 | E3 | Planned |
| FR-CLIENT-001 | civ-011 | E7 | Partial |
| FR-CIV-HUD-001 | civ-011 | E7 | Planned |
| FR-CIV-HUD-002 | civ-011 | E7 | Planned |
| FR-CIV-HUD-003 | civ-011 | E7 | Planned |
| FR-CIV-HUD-004 | civ-011 | E7 | Planned |
| FR-CIV-HUD-005 | civ-011 | E7 | Planned |
| FR-CIV-CLIENT-GODOT-001 | civ-012 | E8 | Planned |
| FR-CIV-CLIENT-GODOT-002 | civ-012 | E8 | Planned |
| FR-API-001 | civ-013 | E5 | Partial |
| FR-API-002 | civ-013 | E5 | Planned |
| FR-API-003 | civ-013 | E5 | Planned |
| FR-API-004 | civ-013 | E5 | Planned |
| FR-REPLAY-001 | civ-013 | E5 | Partial |
| FR-REPLAY-002 | civ-013 | E5 | Partial |

---

## Epic Coverage Summary

| Epic | Description | AgilePlus Artifacts | Target |
|------|-------------|---------------------|--------|
| E1 | Core Engine | civ-001 | MVP |
| E2 | Economy + Simulation Depth | civ-002, civ-003, civ-004, civ-005, civ-008, civ-009 | MVP/v1/v2 |
| E3 | Multi-Client Protocol | civ-010 | v1 |
| E4 | War + Diplomacy | civ-006, civ-007 | v1 |
| E5 | Research API | civ-013 | v1 |
| E6 | Client Implementations (legacy grouping) | merged into E7/E8 | — |
| **E7** | **Bevy Primary Client (new)** | **civ-011** | **MVP** |
| **E8** | **Godot Secondary Client (new)** | **civ-012** | **v2** |

---

## Deprecated Artifacts

| Artifact | Reason |
|----------|--------|
| FR-CLIENT-002 (Web TypeScript Client) | Deprecated 2026-05-28 pivot; user has zero interest in web; archived |
| Web/TypeScript client (E6.4) | Superseded by Godot as secondary target |

---

## Governance Notes

- AgilePlus CLI: **not usable** (not in PATH; binary at `C:/Users/koosh/Dev/AgilePlus/agileplus/` not installed)
- Artifacts authored directly in AgilePlus native three-file format
- Each spec covers one domain slice; traceability held through `fr_ids` in `meta.json` + `Traces to` in spec body
- FR ID namespace: `FR-CIV-*` new IDs (BIO, CULT, BUILD, CLIMATE, WAR, DIPLO, GOV, HUD) authored alongside existing `FR-CORE-*`, `FR-ECON-*`, `FR-PROTO-*`, `FR-API-*`, `FR-REPLAY-*`, `FR-CLIENT-*` from `FUNCTIONAL_REQUIREMENTS.md`
