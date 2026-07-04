# Civis proc-compose services

Run all daemons with `proc-compose up` from this directory.

## Services

| Service | Port | Command | Description |
|---|---|---|---|
| `civ-server` | 8080 | `cargo run -p civ-server` | Game server — JSON-RPC API, game loop |
| `civ-watch` | 9090 | `cargo run -p civ-watch` | Filesystem watcher, live-reload bridge |
| `civis-mcp` | stdio | `cargo run -p civis-mcp` | MCP stdio server for agent tool access |

## Quick start

```bash
# Start all services
proc-compose up

# Start a single service
proc-compose up civ-server

# Tail logs
proc-compose logs -f
```

## Dependencies

- `civ-watch` and `civis-mcp` depend on `civ-server` — proc-compose starts them in order.
- All services require `cargo build` first (or `cargo run` will compile on first start).
- Set `DATA_DIR` env to override default game data path.
