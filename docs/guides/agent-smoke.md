# Agent smoke (AX)

One command to validate the **agent-critical** Rust protocol surface without opening a game client.

## Run

From repo root:

```powershell
.\scripts\agent-smoke.ps1
```

Optional:

```powershell
.\scripts\agent-smoke.ps1 -IncludeBevy    # also run civ-bevy-ref tests
.\scripts\agent-smoke.ps1 -SkipUnreal     # skip all Unreal checks
.\scripts\agent-smoke.ps1 -FullUnreal     # full build.ps1 (UBT) when UE_ROOT/UBT exists; else warn and continue
```

Default (no switches): Rust smokes + `verify-unreal-ready.ps1` offline preflight (no UBT compile).

## What it covers

| Check | Proves |
|-------|--------|
| `cargo test -p civ-server --test ws_smoke` | JSON-RPC health, snapshot shape, spawn → `civ_pins`, **`civ_pins[].job`** (UX-01) |
| `just civis-3d-catalog-check` | `jsonrpc.rs` ↔ `jsonrpc-surface.md` drift |
| `just civis-3d-scenario-check` | `civ-engine` `scenario::*` tests (`-j 1` on Windows — avoids LNK1104 when other cargo builds run) |
| `just civis-3d-mod-check` | `civ-mod-host` + `civlab-sdk` unit tests |
| `just godot-test` | `cargo test --manifest-path clients/godot-ref/rust/Cargo.toml` (F3D0 mesh + WS decode) |
| `cargo test -p civ-watch` | HTTP terrain/snapshot/control contracts |
| `verify-unreal-ready.ps1` (default) | Target.cs, rust `.lib`, UE path scaffolding |
| `build.ps1` (`-FullUnreal`) | Full rust-shim + CivShowEditor UBT when engine installed |

**Note:** Agent smoke does **not** need a separate job test binary. The full `ws_smoke` integration suite already asserts job labels on startup snapshots in `ws_jsonrpc_sim_snapshot_returns_snapshot_fields` (non-null `job`, at least one `"farmer"` pin). Spawn-after-RPC coverage is in `ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot`. No extra flags on `agent-smoke.ps1` are required for job wire shape.

## Web dashboard (separate gate)

Agent smoke does **not** run TypeScript tests. After protocol changes or `web/` edits:

```powershell
cd web && npm test
cd web && npm run build
```

FR closure: [`docs/traceability/fr-web-matrix.md`](../traceability/fr-web-matrix.md).

## Full quality gate (developers)

```bash
just civis-3d-verify
```

Rust-only; add `cd web && npm test` when the dashboard or shared `web/src` helpers change.

## Live client attach

After smoke passes, start services and attach per [`client-attach-matrix.md`](client-attach-matrix.md).

## Unreal compile (human or agent with UE installed)

```powershell
.\clients\unreal-show\scripts\build.ps1
```

Requires **Desktop development with C++** (MSVC). VS Community 2026 (VS 18) works; see maturity audit VS section.
