# FR-CIV-MATURITY — AX / DX / UX / Modding maturity audit

**Date:** 2026-05-25
**Scope:** Civis 3D workspace (`feat/civis-3d-foundation` line) — what is **mature** vs **immature** for agents, developers, players/modders.

Legend: **Mature** = documented + tested + automatable · **Partial** = works but gaps · **Immature** = spec-only or stub · **Missing** = no code path.

---

## Executive summary

| Surface | Maturity | Blocker to “done” |
|---------|----------|-------------------|
| **AX (agents)** | Good | `AGENTS.md` + `agent-smoke.ps1` + attach matrix; optional `-FullUnreal` off default verify |
| **DX (developers)** | Good | `just civis-3d-verify` (catalog + scenario checks), `jsonrpc-surface.md`, `client-attach-matrix.md` |
| **UX (players / L2)** | Partial | Godot/web L2 strong; **F3D0 partial–good** — Bevy full mesh; Godot/Unreal **16³ mesh** when dense `voxels`; Unreal minimap UMG |
| **Modding** | Partial (v3) | Manifest + `.civmod` + WASM policy tick + `ReplayEvent::ModLoaded`; no signing / economic WASM |

**Gates (2026-05-25):** `ws_smoke` 32 tests · `just civis-3d-verify` pass (incl. `check-jsonrpc-catalog.ps1`, `civis-3d-scenario-check`).

**Recommended finish order:** P0 agent contracts → P1 protocol/attach parity → P2 modding MVP hooks → P3 L5 polish.

---

## P0 — Agent experience (AX)

| ID | Gap | Evidence | Finish criterion |
|----|-----|----------|------------------|
| AX-01 | ~~Civis AGENTS.md~~ **Done** | Root `AGENTS.md` (verify table, attach matrix, FR index, do-not) | Keep in sync when RPC/snapshot shapes change |
| AX-02 | ~~Traceability matrix stale~~ **Done** | `TRACEABILITY_MATRIX.md` → `fr-3d-matrix.md` / `fr-web-matrix.md` | Regenerate CIV-01xx rows only when strategic crates land |
| AX-03 | **Unreal build** optional tier | `scripts/quality/README.md`; `unreal_*` gates in emit when UE+UBT; `agent-smoke -FullUnreal` | **Partial** — full UBT not in default smoke/lefthook; set `CIVIS_QUALITY_UNREAL=1` to attest |
| AX-04 | ~~Agent attach smoke~~ **Done** | `agent-smoke.ps1`; `ws_smoke` asserts `civ_pins[].job` | Extend when new snapshot fields land |
| AX-05 | ~~fr-web-spectator open~~ **Done** | `fr-web-matrix.md`; `fr-web-spectator.md` IMPLEMENTED | Reopen only if FR-CIV-WEB acceptance criteria change |
| AX-06 | ~~MSVC gate undocumented~~ **Done** | `AGENTS.md` § Toolchain; playbook + offline scripts | Add MSVC row to product-quality manifest when UE build is mandatory |

---

## P1 — Developer experience (DX)

| ID | Gap | Evidence | Finish criterion |
|----|-----|----------|------------------|
| DX-01 | **Modding API** v3 partial | `wasmtime`, `.civmod`, `civlab-sdk`; 7 `civ-mod-host` tests | Capability API + economic WASM per `fr-modding-roadmap.md` |
| DX-02 | ~~Scenario YAML~~ **Done** | [`scenario-yaml.md`](../guides/scenario-yaml.md); `civis-3d-scenario-check` in verify | Keep keys in sync with `scenario.rs` |
| DX-03 | ~~JSON-RPC catalog split~~ **Done** | [`jsonrpc-surface.md`](../api/jsonrpc-surface.md) + [`scripts/check-jsonrpc-catalog.ps1`](../../scripts/check-jsonrpc-catalog.ps1) in `civis-3d-verify` | Keep doc table in sync when adding `JsonRpcMethod` variants |
| DX-04 | ~~Client attach matrix~~ **Done** | [`docs/guides/client-attach-matrix.md`](../guides/client-attach-matrix.md) | Update when new client or default URL changes |
| DX-05 | ~~Godot GDExtension~~ **Done** | `clients/godot-ref/README.md` § Authoring paths | Rebuild `rust/` DLL after `CivisWsFrame` changes |
| DX-06 | ~~Research crate~~ **Deferred** | [`deferred-crates.md`](../guides/deferred-crates.md) | Wire when research FRs land |
| DX-07 | ~~Domain stubs~~ **Deferred** | [`deferred-crates.md`](../guides/deferred-crates.md) | One vertical slice when product prioritizes |

---

## P2 — User / modder experience (UX) & L2 authoring

| ID | Gap | Evidence | Finish criterion |
|----|-----|----------|------------------|
| UX-01 | **`job` on `civ_pins`** wired from `Citizen` on agent entities | `spectator.rs` reads `Citizen.job`; `attach_citizen_to_agents` | **Mature** — `civ-engine` `civ_pins_include_job_when_citizen_component_present` |
| UX-02 | ~~Cross-client spawn palette~~ **Partial–Good** | [`client-attach-matrix.md`](../guides/client-attach-matrix.md) — all five `kind`s on WS; `ws_smoke` covers civilian + vehicle | Optional `ws_smoke` per `port` / `hangar` |
| UX-03 | **F3D0 voxel stream** partial–good | Bevy: binary `Frame3d`; Godot/Unreal: **16³ procedural mesh** when dense `voxels` (4096), else markers + throttle | Unreal click-to-focus; full-world mesh budget |
| UX-04 | **Minimap conventions** — Bevy/Godot/web; Unreal partial | `ACivMinimapCapture` + `UCivMinimapWidget` in CivShow | Click-to-focus + parity tests when product needs it |
| UX-05 | **spectator_mode default** differs (Godot true, web false) | Confusing for demos | Document in attach matrix; align defaults or query params |
| UX-06 | **Manor Lords L5** incremental only | `fr-l5-visual-pass.md` IN PROGRESS | Close scoped slices; defer art to Quixel |
| UX-07 | **Modder-facing surface** none | No in-game mod browser; no hot reload | Post CIV-0700 MVP |

---

## P3 — Modding contracts (CIV-0700)

| Area | Status | Notes |
|------|--------|-------|
| WASM sandbox | **Partial** | `wasmtime` policy tick in `civ-mod-host`; no full capability API |
| `.civmod` format | **Partial** | ZIP load in `ModHost::load_civmod_archive` |
| `civlab-sdk` guest crate | **Partial** | `crates/civlab-sdk` + `build-example-policy-wasm.ps1` |
| `mods/` directory | **Partial** | `mods/manifest.schema.json`, `mods/example-policy/manifest.toml` |
| Manifest host (`civ-mod-host`) | **Partial** | 7 unit tests; WASM + registry policy phase |
| Engine scenario hook | **Partial** | `register_mod_stubs`; `ReplayEvent::ModLoaded` in replay |
| PolicyMod / EconomicMod hooks | **Partial** | Economy phase `ModHost::tick`; military stub; no economic WASM |
| Lua scenario path | Missing | Spec §12 |
| Mod signing / dev mode | Missing | Spec §14 |

**Minimum modding MVP (v1 — done 2026-05-25):**

1. [x] `mods/manifest.schema.json` + `mods/example-policy/manifest.toml`
2. [x] `crates/mod-host` — load manifest, no-op `tick`, 3 unit tests
3. [x] Engine: `Simulation::register_mod_stubs` from scenario YAML `mods: []`
4. [x] Docs: [`fr-modding-roadmap.md`](fr-modding-roadmap.md)

---

## What is already mature (keep, don’t rewrite)

| Area | Why mature |
|------|------------|
| Determinism + replay | `civreplay`, hash chain, proptest, ws replay roundtrip |
| `civ-server` JSON-RPC | 28+ ws_smoke tests, role gate, spawn/place/damage |
| `civ-watch` HTTP | 16 API tests, terrain/snapshot/control |
| Web L2 authoring | `authoring.ts`, spawnRouting tests, attach config |
| Godot server attach | WS + terrain HTTP, P-U1 sync |
| Bevy CI surface | 41 lib tests, F3D0 parse, minimap math tests |
| Minimap UV contract | `docs/guides/minimap-conventions.md` + web tests |
| Quality local-first CI | lefthook + quality manifest |
| 3D FR traceability | `docs/traceability/fr-3d-matrix.md` implemented rows |

---

## Prioritized backlog (finish & mature)

### Sprint A — Agent contracts (1–2 days)

| # | Item | Status (2026-05-25) |
|---|------|---------------------|
| 1 | Expand root `AGENTS.md` + `docs/guides/agent-smoke.md` | [x] Done |
| 2 | Add `docs/guides/client-attach-matrix.md` | [x] Done |
| 3 | Add `scripts/agent-smoke.ps1` | [x] Done — passes locally |
| 4 | Update `TRACEABILITY_MATRIX.md` header → `fr-3d-matrix.md` | [x] Done |

**Remaining:** AX-03 — optional `unreal_build` in committed manifest on UE machines; default lefthook stays UE-free.

### Sprint B — DX protocol catalog (1 day)

| # | Item | Status (2026-05-25) |
|---|------|---------------------|
| 1 | `docs/api/jsonrpc-surface.md` from `jsonrpc.rs` | [x] Done |
| 2 | Close `fr-web-spectator.md` / `fr-web-matrix.md` | [x] Done |
| 3 | `install-msvc.ps1` + `verify-unreal-ready.ps1` linked | [x] Done — `AGENTS.md`, `agent-smoke` |

**Remaining:** DX-03 CI drift check; DX-02 scenario key docs in verify gate.

### Sprint C — UX parity (2–3 days)

| # | Item | Status (2026-05-25) |
|---|------|---------------------|
| 1 | Wire `job` on `CivPin` | [x] Done |
| 2 | F3D0 throttle doc for Godot | [x] Done — `fr-godot-attach.md` § F3D0; attach matrix notes throttle vs Bevy mesh path |
| 3 | Unreal build + Play checklist | [x] Done — `build.ps1` on VS 2026 + UE 5.7; default smoke = preflight; full UBT via `-FullUnreal` |
| 4 | Godot/Unreal F3D0 mesh | [x] Done — **16³ procedural mesh** when dense `voxels`; else chunk markers |

### Sprint D — Modding MVP (1 week)

| # | Item | Status (2026-05-25) |
|---|------|---------------------|
| 1 | `mods/` + manifest schema | [x] Done |
| 2 | `crates/mod-host` stub + engine hook | [x] Done |
| 3 | CIV-0700 phased roadmap | [x] Done — `fr-modding-roadmap.md` |

**Remaining:** v2 registry, WASM, `.civmod`, policy phase hooks (P3 table).

---

## VS Community & UE 5.7 (toolchain)

**Confirmed 2026-05-25 (KOOSHAPARI-DESK):**

| Item | Result |
|------|--------|
| VS | **Community 2026** (VS 18), workload **Desktop development with C++** |
| UE | **5.7** (`CivShow.uproject` `EngineAssociation`) |
| `build.ps1` | **PASS** — rust-shim + `CivShowEditor` Win64 Development via UBT |
| UBT note | May warn MSVC 14.51 vs “preferred” 14.44; compile still succeeds |
| Agent path | Default `agent-smoke.ps1` = offline preflight only; **full UBT** via `-FullUnreal` or `CIVIS_QUALITY_UNREAL=1` emit |

VS 2022 without `VC\Tools\MSVC` remains insufficient until the C++ workload is installed.

---

## Related docs

- [`IMPLEMENTATION_STATUS.md`](../IMPLEMENTATION_STATUS.md)
- [`fr-3d-matrix.md`](../traceability/fr-3d-matrix.md)
- [`fr-unreal-agent-playbook.md`](fr-unreal-agent-playbook.md)
- [`fr-modding-roadmap.md`](fr-modding-roadmap.md)
- [`CIV-0700-modding-api-spec.md`](../specs/CIV-0700-modding-api-spec.md)
- [`product-quality-ladder.md`](../roadmap/product-quality-ladder.md)
- [`agent-smoke.md`](../guides/agent-smoke.md)
- [`client-attach-matrix.md`](../guides/client-attach-matrix.md)
- [`jsonrpc-surface.md`](../api/jsonrpc-surface.md)

---

## Loop status

**Last verified:** 2026-05-25 (local, `feat/civis-3d-foundation` workspace)

| Gate | Result |
|------|--------|
| `.\scripts\agent-smoke.ps1` | **PASS** — civ-server 32/32 `ws_smoke`, civ-watch, Unreal preflight |
| `.\scripts\agent-smoke.ps1 -FullUnreal` | **PASS** when UE 5.7 + VS 2026 present — full `build.ps1` |
| `just civis-3d-verify` | **PASS** — workspace check/test/clippy/fmt + catalog/scenario/web/mod-host |
| `just civis-3d-mod-wasm` | **Local** — `mods/example-policy/mod.wasm` (gitignored) |
| `just godot-test` | **PASS** — F3D0 mesh + WS decode (13 tests) |
| `just civis-3d-catalog-check` | **PASS** — 14 methods: `jsonrpc.rs` ↔ `jsonrpc-surface.md` |
| `just civis-3d-scenario-check` | **PASS** — 7 `scenario::*` tests |

### Open P0

1. **Modding v2** — registry, WASM, `.civmod`, policy phase hooks — only if the product needs a full mod mesh beyond v1 manifest + `civ-mod-host` stubs.
