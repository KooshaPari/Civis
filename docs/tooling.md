# Civis Tooling

This is the local tooling reference for the Civis 3D line.
It is intentionally command-first: use it to orient yourself, then rely on the
`justfile` and the client READMEs for exact invocation details.

## Overview

Civis is a Rust simulation/game workspace with:

- a Rust engine and server stack (`civ-server`, `civ-watch`, shared crates)
- a TypeScript web dashboard in `web/dashboard`
- multiple engine clients: Bevy, Godot, and Unreal

The authoritative 3D feature matrix lives in
[`docs/traceability/fr-3d-matrix.md`](traceability/fr-3d-matrix.md).

## CLI / `justfile`

Start here:

```powershell
just --list
```

The repo `justfile` is the source of truth for available targets. The most useful
3D-oriented targets today are:

- `just civis-3d-verify`
- `just civis-3d-catalog-check`
- `just civis-3d-scenario-check`
- `just civis-3d-web-check`
- `just civis-3d-mod-check`
- `just civis-3d-mod-wasm`
- `just civis-3d-mod-package`
- `just civis-3d-mod-package-all`
- `just civis-3d-mod-sign`
- `just civis-3d-bevy-smoke`
- `just civis-3d-bevy-window`
- `just civis-3d-standalone`
- `just civis-3d-watch`
- `just civis-3d-watch-build`
- `just godot-test`
- `just infra-up`

If you need a target that is not listed here, check the `justfile` before adding a
new wrapper.

## Process-compose

[`process-compose.yaml`](../process-compose.yaml) orchestrates the local native
infra and sim stack:

- PostgreSQL
- DragonFly
- NATS
- MinIO
- `civ-server`
- a lightweight metrics collector
- a replay validator

The compose file also writes logs under `.process-compose/logs/` and keeps data in
`.process-compose/data/`.

For the watch UI and browser dashboard, use the paired just target:

```powershell
just civis-3d-watch
```

That starts `civ-watch` and `cd web/dashboard && bun run dev` side by side.

## Docker infra

The infra services expose the usual local ports:

- PostgreSQL on `:5432`
- DragonFly on `:6379`
- NATS on `:4222`

`process-compose` prefers native binaries when available and falls back to Docker
only when the native service is missing.

## Web dashboard

The web dashboard lives in `web/dashboard` and runs on Vite:

```powershell
cd web/dashboard
bun run dev
```

That serves the dashboard on `http://127.0.0.1:5173`.

Useful companion commands:

- `bun test`
- `bun run build`
- `bun run typecheck`

## Engine clients

### Bevy

The Bevy reference client is the Rust-first, agent-friendly surface:

```powershell
cargo run -p civ-bevy-ref --features bevy --bin civ-standalone
```

Use `civ-bevy-window` when you want the live windowed attach path.

### Godot

Open `clients/godot-ref/project.godot` in the Godot editor.

When the Rust extension changes:

```powershell
cd clients/godot-ref/rust
cargo build
```

The default attach path is the Godot editor/project plus the running Civis backends
from `civ-server` and `civ-watch`.

### Unreal

Use the Unreal showcase project in `clients/unreal-show`:

```powershell
.\clients\unreal-show\scripts\build.ps1
```

Then open `clients/unreal-show/CivShow.uproject` in the Unreal editor.

## Testing

Use the smallest command that exercises the surface you changed:

- Rust workspace and engine changes: `cargo test`
- Web dashboard changes: `cd web/dashboard && bun test`
- .NET / C# surfaces: `dotnet test` for the relevant solution or project, if one
  exists in the area you are touching

There is not a dedicated repo-wide `.NET` test suite in the current tree, so for the
current Unreal surface the practical validation path is the Unreal build script plus
the repo gates above.

## Dev workflow

The usual iteration loop is:

1. Start the shared infra with `just infra-up` or `process-compose up`.
2. Run the dashboard with `cd web/dashboard && bun run dev`.
3. Start the client you are working on.
4. Use the narrowest test/build command that covers your change.
5. Once the change is stable, run the broader gate:

```powershell
just civis-3d-verify
```

When you are touching only one surface, keep the loop scoped to that surface until
the local build/test is clean. Use the full verification gate before claiming the
change is done.
