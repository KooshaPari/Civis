# Civis Dev Environment

The local development stack is centered on `process-compose.yaml` and the `just` targets in the repo root.

## Recommended paths

- `just dev` starts the full local stack.
- `just dev-stop` stops it.
- `just play` runs the Bevy desktop client.
- `just build-all` builds the Rust, Godot, and Unreal client surfaces.
- `just test-all` runs the workspace and client test coverage.
- `just quality` runs lint, audit, and format checks.

## Service selection

`process-compose.yaml` uses a small launcher script to select the best available runtime for each service:

- Native first when a local binary exists.
- Windows + WSL2 when the stack is running inside a WSL2-enabled environment.
- macOS via OrbStack when native binaries are not present.
- Linux via Podman first, Docker last.

The stack covers:

- Postgres on `CIV_PG_PORT` or `5432`
- Dragonfly/Redis on `CIV_REDIS_PORT` or `6379`
- NATS on `CIV_NATS_PORT` or `4222`
- MinIO on `CIV_MINIO_PORT` or `9000`
- `civ-watch`, which depends on Postgres and Dragonfly

## Devcontainer

`.devcontainer/devcontainer.json` installs the Rust toolchain, `just`, `process-compose`, Bun, and common Bevy build dependencies so the repo can be opened directly in a container and used with the same commands as a native checkout.

## CI parity

The GitHub Actions workflow mirrors the local surface by running the same Rust, web, and client checks that developers run through `just`.
