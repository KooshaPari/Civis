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

### Capacity-budgeted opt-in

Use `-Budgeted` when the smoke must run under an explicit D:/G: capacity gate. The
budget values and receipt directory are intentionally mandatory; the receipt
directory must already exist under `agents\sandbox`.

```powershell
.\scripts\agent-smoke.ps1 -SkipUnreal -Budgeted `
  -TargetDirectory G:\GameDev-Infra\CargoTargets\Civis-clone `
  -ExpectedGrowthBytes 8000000000 `
  -ReserveBytes 50000000000 `
  -Jobs 2 `
  -ReceiptDirectory C:\Users\koosh\agents\sandbox
```

The budgeted path performs a point-in-time healthy-volume, ordinary-directory,
capacity, and advisory same-target Rust/Cargo-owner preflight, then launches the existing
smoke script as a `pwsh` child. Only that child receives `CARGO_TARGET_DIR` and
`CARGO_BUILD_JOBS`; the caller's environment is unchanged. Receipt JSON and
captured child stdout/stderr are written to the explicit sandbox directory.
The owner scan is command-line based and cannot prove ownership when a build
keeps its target directory only in inherited environment variables.
Readers may inspect the log files while the child runs; output is buffered until
the child stream flushes, so this is not a line-live monitoring guarantee.

This is opt-in and only covers `agent-smoke.ps1` plus the Cargo commands it
starts. Direct `cargo`, `just`, `process-compose`, or other raw launchers still
bypass this gate and must not be described as budgeted.

The script now has a named `playable` block for the terrain gate: `civ-server` WS smoke, `civ-watch` API smoke, and Unreal preflight or full UBT build when requested.

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

`just godot-test` preserves an inherited `CARGO_TARGET_DIR`; when the variable is
unset, it falls back to the recipe-local `target-godot-smoke` directory. The
recipe returns Cargo's exit code so target routing and failures remain visible.

The `playable` block groups the `ws_smoke`, `civ-watch`, and Unreal steps above so terrain playability stays a single fail-fast sequence.

**Note:** Agent smoke does **not** need a separate job test binary. The full `ws_smoke` integration suite already asserts job labels on startup snapshots in `ws_jsonrpc_sim_snapshot_returns_snapshot_fields` (non-null `job`, at least one `"farmer"` pin). Spawn-after-RPC coverage is in `ws_jsonrpc_spawn_civilian_pin_appears_in_snapshot`. No extra flags on `agent-smoke.ps1` are required for job wire shape.

## Web dashboard (separate gate)

Agent smoke does **not** run TypeScript tests. After protocol changes or `web/` edits:

```powershell
cd web && bun test
cd web && bun run build
```

FR closure: [`docs/traceability/fr-web-matrix.md`](../traceability/fr-web-matrix.md).

## Full quality gate (developers)

```bash
just civis-3d-verify
```

Rust-only; add `cd web && bun test` when the dashboard or shared `web/src` helpers change.

## Bevy native and live-attach boundaries

`agent-smoke.ps1` and `just civis-3d-live-smoke` are headless protocol gates.
They do not open a native window or require a running server. For the windowed
standalone startup/clean-exit path, run:

```powershell
just civis-3d-standalone-smoke
```

That recipe builds `civ-standalone` and runs it for a bounded number of update
frames (`CIVIS_SMOKE_FRAMES`, default `5`) on a GPU adapter. It proves startup
and its scripted `AppExit`; it does **not** prove operating-system minimize,
restore, close-button behavior, or live-server gameplay. Treat those as
separate acceptance evidence.

The menu-level live pause/resume acknowledgement is covered without a window by:

```powershell
cargo test -p civ-bevy-ref --features bevy,egui --lib menus::tests::live_pause_and_resume_wait_for_set_speed_acknowledgements
```

## Live client attach

After smoke passes, start services and attach per [`client-attach-matrix.md`](client-attach-matrix.md). The matrix records the current `:3800` server/Bevy versus `:3000` Godot/Unreal/web endpoint split; select one explicit endpoint before running a cross-client session.

## Unreal compile (human or agent with UE installed)

```powershell
.\clients\unreal-show\scripts\build.ps1
```

Requires **Desktop development with C++** (MSVC). VS Community 2026 (VS 18) works; see maturity audit VS section.
