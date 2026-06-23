# AGENTS.md — Civis (3D / agentic line)

Extends Phenotype parent governance. **Authoritative 3D FR matrix:** `docs/traceability/fr-3d-matrix.md`.

## Verify before you claim done

| Gate | Command |
|------|---------|
| Agent smoke (fast) | `.\scripts\agent-smoke.ps1` |
| Agent smoke (full UE) | `.\scripts\agent-smoke.ps1 -FullUnreal` (when `UE_ROOT`/UBT present) |
| Full 3D workspace | `just civis-3d-verify` (catalog + scenario + web + mod-host, then check/test/clippy/fmt) |
| JSON-RPC catalog drift | `just civis-3d-catalog-check` |
| Scenario YAML | `just civis-3d-scenario-check` |
| Example mod WASM | `just civis-3d-mod-wasm` (`wasm32-unknown-unknown`) → `mods/example-policy/mod.wasm` + `mods/example-economic/mod.wasm` |
| Example `.civmod` | `just civis-3d-mod-package` → `example-policy.civmod`; `just civis-3d-mod-package-all` → policy + economic |
| Example mod sign | `just civis-3d-mod-sign` → `scripts/sign-example-mod.ps1` (prints `author_pubkey_hex`) |
| Quality manifest (optional UE) | `scripts/quality/README.md`; `CIVIS_QUALITY_UNREAL=1` + `emit-quality-manifest.ps1` |
| Web dashboard | `cd web && npm test` and `cd web && npm run build` |
| Godot GDExtension | `just godot-test` (`--manifest-path clients/godot-ref/rust/Cargo.toml`) |
| Unreal CivShow | `.\clients\unreal-show\scripts\build.ps1` (needs UE 5.7 + MSVC) |
| Unreal PIE prep | `.\scripts\pie-validation.ps1` (starts backends, WS/terrain smoke, prints PIE checklist) |

See [`docs/guides/agent-smoke.md`](docs/guides/agent-smoke.md) for the playable terrain gate breakdown and the full smoke coverage matrix.

## Attach matrix (do not guess URLs)

See [`docs/guides/client-attach-matrix.md`](docs/guides/client-attach-matrix.md).

Default stack:

- `cargo run -p civ-server` → WS `ws://127.0.0.1:3000/ws?tick_format=binary`
- `cargo run -p civ-watch` → HTTP `http://127.0.0.1:9090` (terrain + dashboard)

## FR / playbook index

| Topic | Doc |
|-------|-----|
| AX/DX/UX maturity gaps | `docs/development-guide/fr-ax-dx-ux-maturity-audit.md` |
| Unreal agent steps | `docs/development-guide/fr-unreal-agent-playbook.md` |
| Godot attach | `docs/development-guide/fr-godot-attach.md` |
| L5 visual pass | `docs/development-guide/fr-l5-visual-pass.md` |
| Modding (spec only) | `docs/specs/CIV-0700-modding-api-spec.md` |
| Scenario YAML | `docs/guides/scenario-yaml.md` |
| Agent smoke | `docs/guides/agent-smoke.md` |
| Web FR matrix (closed) | `docs/traceability/fr-web-matrix.md` |

## Toolchain notes (Unreal)

- Engine: **UE 5.7** (`CivShow.uproject` `EngineAssociation`)
- **VS Community 2026** (VS 18) with **Desktop development with C++** is sufficient; full `build.ps1` (rust-shim + UBT) succeeds on this toolchain. UBT may warn that 14.51 is not the “preferred” 14.44 — build still completes.
- VS 2022 Community without `VC\Tools\MSVC` is **not** enough until the C++ workload is installed.
- Offline preflight: `clients/unreal-show/scripts/verify-unreal-ready.ps1`

## Do not (agents)

- Do not implement full CIV-0700 (capability enforcement, mod store/publish, hot reload) — v3 **partial–good**: manifest + `.civmod` ZIP + `wasmtime` ticks + Ed25519 verify (`just civis-3d-mod-sign`); mod browser on `sim.snapshot` + web **Mods** panel; `mod.loaded.v1` replay-bus JSON; determinism scan unless `mod-dev`.
- Do not assume Quixel/Megascans assets are in git (`Content/Megascans/` is local-only).
- Do not edit non-primary worktrees unless the user asked.
- Do not skip `agent-smoke` or `civis-3d-verify` when changing JSON-RPC or snapshot shapes.

## Parent / local contracts

- **Local:** `CLAUDE.md` (stack, testing, worktrees)
- **Phenotype org:** parent `AGENTS.md` under Phenotype repos
- **AgilePlus:** `cd /repos/AgilePlus && agileplus <command>` before large features

## Maturity status (2026-05-26)

**Mature:** determinism/replay, `civ-server` WS tests (incl. spawn palette), `civ-watch`, web L2 authoring, Godot/Bevy/Unreal server attach, JSON-RPC catalog + `just civis-3d-verify`.

**Partial:** modding v3 — **25+** `civ-mod-host` tests, WASM ticks, `.civmod`, `civlab-sdk`, Ed25519 verify + `just civis-3d-mod-sign` + `just civis-3d-mod-package-all`; example mods on **civ-server** / **civ-watch** + `baseline.yaml`; mod browser (`mods` + `mod_lifecycle` on `sim.snapshot`, web **Mods** panel, Godot **Mods** label); `mod.loaded.v1` replay-bus JSON in replay + watch event feed; F3D0 — Bevy full `Frame3d`, Godot/Unreal **16³ mesh** when dense `voxels`; cross-client minimap click-to-focus (Bevy/Godot/web/Unreal).

**Product-only (not agent blockers):** Quixel/Megascans mesh import — engineering slots in `Content/Megascans/` + [fr-l5-visual-pass.md](docs/development-guide/fr-l5-visual-pass.md); artists import via Bridge.

See `docs/development-guide/fr-ax-dx-ux-maturity-audit.md`.
