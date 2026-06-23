# CivLab: FR Implementation Tracker

## Economy (FR-CIV-ECON)

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-CIV-ECON-001 | Ledger double-entry accounting | In Progress | `crates/engine/src/engine.rs` |
| FR-CIV-ECON-002 | Market clearing algorithm | In Progress | `crates/engine/src/engine.rs` |
| FR-CIV-ECON-003 | Joule economy allocator | Planned | `crates/engine/src/engine.rs` |
| FR-CIV-ECON-004 | Policy-driven fiscal control | In Progress | `crates/engine/src/policy.rs` |

## RTS Command Interface (FR-CIV-RTS)

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-CIV-RTS-001 | RTS command dispatch | Planned | `crates/engine/src/engine.rs` |

## Geography (FR-CIV-GEO)

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-CIV-GEO-001 | Terrain and geography | Planned | `crates/engine/src/engine.rs` |

## Actor Lifecycle (FR-CIV-ACT)

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-CIV-ACT-001 | Citizen lifecycle | Planned | `crates/engine/src/engine.rs` |

## War and Diplomacy (FR-CIV-WAR)

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-CIV-WAR-001 | War and diplomacy systems | Planned | - |

## Research/Sandbox API (FR-CIV-RES)

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-CIV-RES-001 | Scenario API | Planned | `crates/server/src/main.rs` |

## Core Engine

| FR ID | Description | Status | Code Location |
|-------|-------------|--------|---------------|
| FR-CIV-CORE-001 | Deterministic tick loop (100ms/tick) | In Progress | `crates/engine/src/engine.rs` |
| FR-CIV-CORE-002 | ECS entity model | In Progress | `crates/engine/src/engine.rs` |
| FR-CIV-CORE-003 | WebSocket JSON-RPC server | In Progress | `crates/server/src/main.rs` |
| FR-CIV-CORE-004 | Event logging and replay | In Progress | `crates/engine/src/io.rs` |
| FR-CIV-CORE-005 | Metrics export | In Progress | `crates/engine/src/metrics.rs` |
