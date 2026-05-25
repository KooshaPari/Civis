# FR-CIV-MATURITY — AX / DX / UX / Modding maturity audit

**Date:** 2026-05-25
**Scope:** Civis 3D workspace (`feat/civis-3d-foundation` line) — what is **mature** vs **immature** for agents, developers, players/modders.

Legend: **Mature** = documented + tested + automatable · **Partial** = works but gaps · **Immature** = spec-only or stub · **Missing** = no code path.

---

## Executive summary

| Surface | Maturity | Blocker to “done” |
|---------|----------|-------------------|
| **AX (agents)** | Partial–Good | `AGENTS.md` + `agent-smoke.ps1` + attach matrix in place; Unreal still not in `civis-3d-verify` / lefthook |
| **DX (developers)** | Good | `just civis-3d-verify`, `jsonrpc-surface.md`, `client-attach-matrix.md`; workspace gate flaky on Windows file locks + `civ-bevy-ref` bin |
| **UX (players / L2)** | Partial | Godot P-U1 best; web L2 good; F3D0 live on Bevy only; Unreal minimap out of scope |
| **Modding** | Partial (v1 manifest) | `civ-mod-host` + `mods/` + scenario `mods: []`; WASM / `.civmod` / phase hooks still spec-only |

**Recommended finish order:** P0 agent contracts → P1 protocol/attach parity → P2 modding MVP hooks → P3 L5 polish.

---

## P0 — Agent experience (AX)

| ID | Gap | Evidence | Finish criterion |
|----|-----|----------|------------------|
| AX-01 | ~~Civis AGENTS.md~~ **Done** | Root `AGENTS.md` (verify table, attach matrix, FR index, do-not) | Keep in sync when RPC/snapshot shapes change |
| AX-02 | ~~Traceability matrix stale~~ **Done** | `TRACEABILITY_MATRIX.md` → `fr-3d-matrix.md` / `fr-web-matrix.md` | Regenerate CIV-01xx rows only when strategic crates land |
| AX-03 | **Unreal build** optional tier | `scripts/quality/README.md`; `unreal_*` gates in emit when UE+UBT; `agent-smoke -FullUnreal` | **Partial** — full UBT not in default smoke/lefthook; set `CIVIS_QUALITY_UNREAL=1` to attest |
| AX-04 | ~~No agent attach smoke~~ **Done** | `scripts/agent-smoke.ps1` + `docs/guides/agent-smoke.md` | Extend smoke to assert `civ_pins[].job` when spawn sets job |
| AX-05 | ~~fr-web-spectator open~~ **Done** | `fr-web-matrix.md`; `fr-web-spectator.md` IMPLEMENTED | Reopen only if FR-CIV-WEB acceptance criteria change |
| AX-06 | ~~MSVC gate undocumented~~ **Done** | `AGENTS.md` § Toolchain; playbook + offline scripts | Add MSVC row to product-quality manifest when UE build is mandatory |

---

## P1 — Developer experience (DX)

| ID | Gap | Evidence | Finish criterion |
|----|-----|----------|------------------|
| DX-01 | **Modding API** v1 manifest only | `crates/mod-host`, `mods/`, `fr-modding-roadmap.md`; no WASM | v2: registry + policy stub; CI drift check on manifest schema |
| DX-02 | **Scenario YAML** partial | `scenarios/baseline.yaml` has `mods: []`; `Scenario::validate` | Document all keys; fail `civis-3d-verify` on invalid YAML |
| DX-03 | ~~JSON-RPC catalog split~~ **Done** | [`docs/api/jsonrpc-surface.md`](../api/jsonrpc-surface.md) — 14 methods, `ws_smoke` links | Optional CI check against `JsonRpcMethod` enum |
| DX-04 | ~~Client attach matrix~~ **Done** | [`docs/guides/client-attach-matrix.md`](../guides/client-attach-matrix.md) | Update when new client or default URL changes |
| DX-05 | **Godot GDExtension** path vs scripts-only | `civis-godot-rust` + `scripts/` | README “authoring path” for server vs watch |
| DX-06 | **Research crate** ADR-006 stubs only | `crates/research` | Mark “not on critical path” or wire validator into scenario load |
| DX-07 | **Many domain crates** schema stubs only | genetics, laws, species, diffusion `*_stub` tests | Either implement one vertical slice or mark `deferred` in matrix |

---

## P2 — User / modder experience (UX) & L2 authoring

| ID | Gap | Evidence | Finish criterion |
|----|-----|----------|------------------|
| UX-01 | **`job` on `civ_pins`** wired from `Citizen` on agent entities | `spectator.rs` reads `Citizen.job`; `attach_citizen_to_agents` | **Mature** — `civ-engine` `civ_pins_include_job_when_citizen_component_present` |
| UX-02 | **Cross-client spawn palette** parity | `client-attach-matrix.md` spawn table; Unreal HTTP+WS | WS test per kind (extend `ws_smoke`) |
| UX-03 | **F3D0 voxel stream** not in Unreal/Godot live path | Planned in README | Bevy binary-first done; extend one client |
| UX-04 | **Minimap conventions** — Bevy/Godot/web; Unreal none | `minimap-conventions.md`, `client-attach-matrix.md` UX-04 | **Documented** — Unreal out of scope until implemented |
| UX-05 | **spectator_mode default** differs (Godot true, web false) | Confusing for demos | Document in attach matrix; align defaults or query params |
| UX-06 | **Manor Lords L5** incremental only | `fr-l5-visual-pass.md` IN PROGRESS | Close scoped slices; defer art to Quixel |
| UX-07 | **Modder-facing surface** none | No in-game mod browser; no hot reload | Post CIV-0700 MVP |

---

## P3 — Modding contracts (CIV-0700)

| Area | Status | Notes |
|------|--------|-------|
| WASM sandbox | Missing | Spec §3 — no `wasmtime` / host in workspace |
| `.civmod` format | Missing | Spec §11 |
| `civlab-sdk` guest crate | Missing | Spec §9 |
| `mods/` directory | **Partial** | `mods/manifest.schema.json`, `mods/example-policy/manifest.toml` |
| Manifest host (`civ-mod-host`) | **Partial** | `crates/mod-host` — load/validate, no-op `tick`, 3 unit tests |
| Engine scenario hook | **Partial** | `Scenario.mods`, `Simulation::register_mod_stubs` |
| PolicyMod / EconomicMod hooks | Missing | No phase hooks in `engine.rs` |
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
| 3 | Unreal build + Play checklist | [~] Partial — `build.ps1` on VS 2026 + UE 5.7; default smoke = preflight; full UBT via `-FullUnreal` |

**Remaining:** UX-03 F3D0 **voxel mesh** on Godot/Unreal live path; extend `ws_smoke` per spawn `kind`.

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
| `.\scripts\agent-smoke.ps1` | **PASS** — civ-server + civ-watch; default Unreal offline preflight (`verify-unreal-ready`) |
| `.\scripts\agent-smoke.ps1 -FullUnreal` | **PASS** when UE 5.7 + VS 2026 present — full `build.ps1` (not `-SkipUe`) |
| `just civis-3d-verify` | Stop running `civ-watch` if `civ-watch.exe` file-lock fails |
| `civ-bevy-ref` autobins | **Fixed** — `autobins = false` (no spurious `terrain` bin) |

### Open P0 (agent / verify)

1. **AX-03** — **Partial:** optional `unreal_*` quality gates + `-FullUnreal`; not in default lefthook/`civis-3d-verify`.
2. **Live F3D0** — Godot/Unreal voxel stream still snapshot-only (documented).
