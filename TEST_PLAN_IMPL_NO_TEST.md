# Test Plan — IMPL-NO-TEST Gaps (Rust Workspace)

**Source:** `FR_TRACE_SNAPSHOT.txt` (2026-06-16) — 2 IDs with spec + `crates/` impl, no test reference in `crates/`.

**Scope:** Plan only. No source edits, no `cargo`, no commit.

---

## 1. FR-CIV-PROTO3D

| Field | Value |
|-------|-------|
| **Spec** | `docs/traceability/fr-3d-matrix.md` (Protocol 3D section); epic in `docs/development-guide/fr-3d-additions.md:102` |
| **Epic requirement** | `civ-protocol-3d` crate ships wire-format types, F3D0 binary envelope, and AgentStream parallel envelope for 3D client attach |
| **Impl site** | `crates/protocol-3d/src/lib.rs` (module root `:23`; primary API below) |
| **Key functions** | `encode_frame3d_binary`, `decode_frame3d_binary`, `encode_frame3d_binary_from_json`, `is_frame3d_binary`, `Frame3d::tick`, `encode_agent_stream_binary`, `decode_agent_stream_binary`, `quantize_axis`, `dequantize_axis`, `map_build_provenance`, `agent_world_translation`, `WorldXZ::from_fixed_coord` |
| **Gap reason** | Child IDs `FR-CIV-PROTO3D-000`…`-017` have `/// Covers` tests; parent string `FR-CIV-PROTO3D` never appears in any `crates/` test. `Climate` variant and `is_frame3d_binary` / `map_build_provenance` have no dedicated coverage. |

### Proposed unit test

**Location:** `crates/protocol-3d/src/lib.rs` — `mod tests`  
**Name:** `fr_civ_proto3d_epic_all_frame3d_variants_f3d0_roundtrip`  
**Tag:** `/// Covers FR-CIV-PROTO3D — epic: all Frame3d variants losslessly round-trip F3D0 binary envelope.`

| Phase | Sketch |
|-------|--------|
| **Arrange** | Build one minimal instance of each `Frame3d` variant: `VoxelDelta`, `BuildingDiff`, `AgentAppearance`, `CivilianState`, `FactionState`, `EventFeed`, `Climate` (use `civ_planet::Climate::default()` + empty `weather`). Set a shared `tick: 99`. |
| **Act** | For each variant: `bytes = encode_frame3d_binary(&frame)?`; assert `is_frame3d_binary(&bytes)`; `back = decode_frame3d_binary(&bytes)?`. |
| **Assert** | `back == frame` for all seven variants; `bytes[0..4] == FRAME3D_BINARY_MAGIC`; kind byte at index 4 matches variant (`Climate` → `6`); `frame.tick() == 99`. Optionally assert `map_build_provenance(civ_build::BuildingProvenance::Procedural) == BuildingProvenance::Procedural`. |

**Traceability effect:** Moves parent `FR-CIV-PROTO3D` from `IMPL-NO-TEST` → `COVERED` when scanner matches `FR-CIV-PROTO3D` in test comment.

---

## 2. FR-CIV-WEB-003

| Field | Value |
|-------|-------|
| **Spec** | `docs/traceability/fr-web-matrix.md:26`; acceptance in `docs/development-guide/fr-web-spectator.md:32` |
| **Requirement** | Read-only 3D spectator view: terrain biomes + building/agent proxies from snapshot; no sim mutation |
| **Impl site** | `crates/engine/src/spectator.rs` |
| **Key functions** | `Simulation::spectator_view` (`:99`), `civ_pins` (`:119`), `factions_for_tick` (`:148`), `buildings_for_factions` (`:168`), `wrap01` (`:114`) |
| **Gap reason** | Module doc references `FR-CIV-WEB-003` but no `crates/` test cites the ID. Existing tests (`spectator_view_has_pins_after_startup`, etc.) lack `/// Covers FR-CIV-WEB-003`. Web coverage (`web/tests/snapshotView.test.mjs`) is outside the Rust traceability scan. |

### Proposed unit test

**Location:** `crates/engine/src/spectator.rs` — `mod tests`  
**Name:** `fr_civ_web_003_spectator_view_read_only_and_deterministic`  
**Tag:** `/// Covers FR-CIV-WEB-003 — read-only spectator payload is deterministic and serde-round-trips.`

| Phase | Sketch |
|-------|--------|
| **Arrange** | `let mut sim = Simulation::with_seed(42)`; record `tick_before = sim.state.tick`, `pop_before = sim.snapshot().population`. |
| **Act** | `let view_a = sim.spectator_view()`; `let view_b = sim.spectator_view()`; `let json = serde_json::to_string(&view_a)?`; `let view_json: SpectatorView = serde_json::from_str(&json)?`. |
| **Assert** | `!view_a.civ_pins.is_empty() && !view_a.factions.is_empty() && !view_a.buildings.is_empty()`; `view_a == view_b` (deterministic for same seed/tick); `view_a == view_json` (wire contract for `sim.snapshot` / JSON-RPC); `sim.state.tick == tick_before && sim.snapshot().population == pop_before` (read-only — no mutation); `view_a.civ_pins.windows(2).all(|w| w[0].idx <= w[1].idx)` and `view_a.civ_pins.len() <= 256` (pin ordering/cap from `civ_pins`). |

**Traceability effect:** Moves `FR-CIV-WEB-003` from `IMPL-NO-TEST` → `COVERED` in Rust workspace scan.

---

## Dependency graph (execution order)

```text
FR-CIV-PROTO3D test  ─┐
                       ├── independent; safe to implement in parallel
FR-CIV-WEB-003 test   ─┘
```

## Verification (after implementation — not run in this lane)

```bash
cargo test -p civ-protocol-3d fr_civ_proto3d_epic
cargo test -p civ-engine fr_civ_web_003
```

Re-scan with the same grep/classify method used for `FR_TRACE_SNAPSHOT.txt` to confirm both IDs report `COVERED`.
