# civ-server

Server application for the Civis deterministic simulation engine. Exposes the simulation over **JSON-RPC via WebSocket** with OpenTelemetry observability and Prometheus metrics.

## Features

- WebSocket bridge for real-time client connections
- Save-game persistence via `civ-save-db`
- OpenTelemetry distributed tracing (OTLP export)
- Prometheus metrics on port 9090 (default)
- Mod host integration through `civ-mod-host`
- 3D protocol support via `civ-protocol-3d`

## Configuration

| Environment Variable | Default | Description |
|---|---|---|
| `CIV_SERVER_PORT` | `3000` | Listen port for the WebSocket server |
| `CIVIS_WS_MAX_CLIENTS` | — | Maximum concurrent WebSocket clients |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | — | OTLP collector endpoint for traces |

## Usage

```bash
# Start with defaults
cargo run -p civ-server

# Custom port
CIV_SERVER_PORT=8080 cargo run -p civ-server
```

## Architecture

```
Client(s) ──WS──▶ civ-server ──▶ civ-engine (Simulation)
                     │
                     ├── civ-save-db   (persistence)
                     ├── civ-mod-host  (mod loading)
                     ├── civ-economy   (economy state)
                     └── observability (tracing + metrics)
```

## Dependencies

| Crate | Role |
|---|---|
| `civ-engine` | Core simulation |
| `civ-agents` | Agent subsystem |
| `civ-economy` | Economy layer |
| `civ-voxel` | 3D terrain |
| `civ-protocol-3d` | 3D client protocol |
| `civ-build` | Construction systems |
| `civ-mod-host` | Mod loading |
| `civ-save-db` | Save/load persistence |
| `civ-observability` | Tracing + metrics setup |
| `civ-emergence-metrics` | Emergent-behaviour metrics |
