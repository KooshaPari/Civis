# CivLab: Code Entity Map

## Engine Crate

| Entity | Path | Maps To |
|--------|------|---------|
| Simulation engine (tick loop, ECS) | `crates/engine/src/engine.rs` | FR-CIV-CORE-001, FR-CIV-CORE-002, FR-CIV-ECON-001 |
| Policy evaluation | `crates/engine/src/policy.rs` | FR-CIV-ECON-004 |
| I/O and event logging | `crates/engine/src/io.rs` | FR-CIV-CORE-004 |
| Metrics export | `crates/engine/src/metrics.rs` | FR-CIV-CORE-005 |
| Engine lib entry | `crates/engine/src/lib.rs` | All FR-CIV-CORE |

## Server Crate

| Entity | Path | Maps To |
|--------|------|---------|
| WebSocket JSON-RPC server | `crates/server/src/main.rs` | FR-CIV-CORE-003, FR-CIV-RES-001 |

## Frontend

| Entity | Path | Maps To |
|--------|------|---------|
| Web client entry | `src/index.html` | FR-CIV-CORE-003 |

## Documentation

| Entity | Path | Maps To |
|--------|------|---------|
| VitePress docs | `docs/` | - |
| Wiki/concepts | `docs/wiki/` | - |
| API docs | `docs/api/` | FR-CIV-CORE-003 |
| Architecture docs | `architecture/` | All ADRs |
