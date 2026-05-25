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
| `cargo test -p civ-server --test ws_smoke` | JSON-RPC health, snapshot shape, spawn → `civ_pins` |
| `cargo test -p civ-watch` | HTTP terrain/snapshot/control contracts |
| `verify-unreal-ready.ps1` (default) | Target.cs, rust `.lib`, UE path scaffolding |
| `build.ps1` (`-FullUnreal`) | Full rust-shim + CivShowEditor UBT when engine installed |

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
