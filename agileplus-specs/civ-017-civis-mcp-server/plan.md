# Plan: Civis MCP Server (civ-017)

## Phased WBS

### Phase 1: Tool proxy (E8.1, E8.2)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M1.1 | Register one MCP tool per JSON-RPC method in `docs/api/jsonrpc-surface.md` | — | Planned |
| M1.2 | Forward tool calls to `civ-server` over the WebSocket surface | M1.1 | Planned |
| M1.3 | Map JSON-RPC errors to MCP tool errors (preserve code, message) | M1.1 | Planned |

### Phase 2: Read-only HTTP tools (E8.3)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M2.1 | Add read-only tools for `/terrain`, `/snapshot`, `sim.snapshot` | M1.2 | Planned |
| M2.2 | Add `--allow-mutations` CLI flag (default deny) | M2.1 | Planned |

### Phase 3: Contract tests (E8.4)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M3.1 | One contract test per tool; assert envelope + response shape | M1.2, M2.1 | Planned |
| M3.2 | Unknown-method error mapping test (-32601) | M1.3 | Planned |

### Phase 4: Transports + config (E8.5, E8.6)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M4.1 | `--transport stdio` (default) and `--transport tcp --bind <addr>` | — | Planned |
| M4.2 | Read `CIVIS_MCP_CIV_SERVER_URL`, `CIVIS_MCP_CIV_WATCH_URL`, `CIVIS_MCP_AUTH_TOKEN` from env | M4.1 | Planned |

### Phase 5: Smoke gate (E8.7)
| Task | Description | Depends On | Status |
|------|-------------|------------|--------|
| M5.1 | `agent-smoke.ps1` civis-mcp health check (start, list, kill) | M3.*, M4.* | Planned |

## DAG Dependencies

```
M1.1 → M1.2, M1.3
M1.2 → M2.1 → M2.2
M1.2, M2.1 → M3.1
M1.3 → M3.2
M4.1 → M4.2
M3.1, M3.2, M4.2 → M5.1
```
