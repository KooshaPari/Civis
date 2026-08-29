---
title: Deployment
description: Running civ-server, civ-watch, and Grafana dashboards for a Civis deployment.
---

# Deployment

## Overview

A Civis deployment runs three cooperating processes:

1. **`civ-server`** — the WebSocket JSON-RPC and `Frame3d` broadcast service.
2. **`civ-watch`** — the HTTP control plane for mod lifecycle, uploads, and remote-cache.
3. **The Bevy client** — connects to `civ-server` for live gameplay; optional for headless deployments.

Grafana dashboards under `deploy/grafana/` visualize per-tick metrics, emergence samples, and saga-graph events emitted via the OpenTelemetry pipeline.

This page describes a single-host deployment with `process-compose.yaml` plus a Grafana container. Multi-host deployments should place `civ-server` behind a TCP load balancer (sticky to socket ID) and `civ-watch` behind an HTTP load balancer.

## Process Layout

```text
┌──────────────────────────────────────────────────────────────────┐
│ Host                                                            │
│                                                                  │
│  ┌────────────────┐    ┌────────────────┐    ┌──────────────┐  │
│  │  civ-server    │    │  civ-watch     │    │  grafana     │  │
│  │  :7777 (ws)    │    │  :9090 (http)  │    │  :3000       │  │
│  └────────┬───────┘    └────────┬───────┘    └──────┬───────┘  │
│           │                     │                    │          │
│  ┌────────▼─────────────────────▼────────────────────▼──────┐ │
│  │  Shared volume: saves/, replays/, mods/publish/         │ │
│  └──────────────────────────────────────────────────────────┘ │
└──────────────────────────────────────────────────────────────────┘
```

## Configuration

| Variable | Default | Description |
|----------|---------|-------------|
| `CIVIS_WS_PORT` | `7777` | WebSocket listen port for `civ-server`. |
| `CIVIS_REQUIRE_ROLE` | `0` | When `1`, enables role gating on privileged JSON-RPC methods. |
| `CIV_WATCH_PORT` | `9090` | HTTP control listen port for `civ-watch`. |
| `CIVIS_SAVE_DIR` | `./saves` | Directory for save files. |
| `CIVIS_REPLAY_DIR` | `./replays` | Directory for replay files. |
| `CIVIS_MODS_DIR` | `./mods/publish` | Directory for published mods. |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | (unset) | OTLP collector endpoint for traces and metrics. |
| `RUST_LOG` | `info` | Standard `tracing` filter. |

## process-compose

The included `process-compose.yaml` declares all three processes with health checks and restart policies:

```bash
# Start the full stack
process-compose up

# Tail logs
process-compose logs -f

# Stop
process-compose down
```

Each process declares its own readiness probe:

| Process | Readiness |
|---------|-----------|
| `civ-server` | WebSocket upgrade succeeds on `ws://127.0.0.1:7777/ws` |
| `civ-watch` | `GET http://127.0.0.1:9090/control/mods/catalog` returns 200 |
| `grafana` | `GET http://127.0.0.1:3000/api/health` returns 200 |

## Persistent Volumes

| Path | Purpose | Backup |
|------|---------|--------|
| `./saves` | Save files (`.civsave`) | Daily snapshot to object storage. |
| `./replays` | Replay files (`.json`) | Daily snapshot to object storage. |
| `./mods/publish` | Published mods | Versioned; tagged per release. |
| `./mods/uploads` | Uploaded (unpublished) mods | Optional; can be excluded. |

Save and replay files are JSON-serializable and forward-compatible across minor versions.

## Observability

`crates/observability` ships an OpenTelemetry exporter. When `OTEL_EXPORTER_OTLP_ENDPOINT` is set, the server emits:

- Per-tick metrics: tick duration, entity count, emergence samples.
- Traces: per-phase spans (`physics`, `economy`, `ai`, `governance`, `diplomacy`).
- Logs: structured JSON via `tracing-subscriber`.

Grafana dashboards in `deploy/grafana/` consume the OTLP stream and expose:

- **Tick health** — tick duration histogram, missed-tick counter.
- **Economy** — energy budget, surplus/waste, scarcity multiplier.
- **AI** — citizen welfare distribution, faction goal distribution.
- **Emergence** — emergence samples timeline and heatmap.
- **Diplomacy** — faction relation graph and treaty timeline.

## Reverse Proxy

For external access, terminate TLS at a reverse proxy:

```nginx
server {
  listen 443 ssl;
  server_name civ.example.com;

  location /ws {
    proxy_pass http://127.0.0.1:7777;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
  }

  location /control/ {
    proxy_pass http://127.0.0.1:9090/control/;
  }
}
```

Sticky routing is not required for `civ-watch` but is required for `civ-server` WebSocket subscriptions; configure the load balancer to pin by source IP or to forward the upgrade headers as shown.

## Upgrade Procedure

1. Drain active subscriptions by setting `sim.command` to `noop` for one tick.
2. Stop `civ-server`.
3. Replace the binary.
4. Start `civ-server`; subscriptions re-establish on the next client reconnect.
5. `civ-watch` can be upgraded in-place; mods survive.

`civ-watch` retains mod state across restarts; `civ-server` recovers tick state from the last snapshot if `CIVIS_SAVE_DIR` is configured.

## See Also

- [Architecture](/architecture/) — process layout and crate responsibilities.
- [API](/api/) — JSON-RPC and HTTP control surfaces exposed by the deployed processes.
- [Development](/development/) — building the binaries that get deployed.