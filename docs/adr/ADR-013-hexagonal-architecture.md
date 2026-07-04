# ADR-013: Hexagonal Architecture + Wire Protocol

## Status
Accepted

## Context
Civis needs clear separation between domain logic, ports, and adapters to support multiple clients (Bevy, Godot, Unreal, Web, CLI) and multiple transport mechanisms (WebSocket, MCP, CLI).

## Decision
Adopt hexagonal (ports and adapters) architecture:
- Core: engine/economy/agents/emergence crates (pure domain, no I/O)
- Ports: protocol-3d (wire types), server (WebSocket), civis-mcp (MCP tool surface)
- Adapters: bevy-ref (game client), godot-ref (secondary), civis-cli (CLI), web/dashboard (web)

## Wire Protocol
protocol-3d crate defines canonical types: CivilianStateEntry, FactionStateFrame, LiveHudSnapshot, JsonRpcMethod enum.

## Consequences
All new features enter via Core, expose via Port interfaces, implemented by Adapters. No adapter imports from another adapter.
