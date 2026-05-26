# DINOForge.Bridge.Protocol

JSON-RPC 2.0 protocol definitions and message types for communicating with the DINOForge game bridge that runs inside Diplomacy is Not an Option (DINO).

## Purpose

This package provides the foundational message contracts for out-of-process game bridge communication. It defines:

- **IGameBridge** — Interface for bridge implementations
- **JSON-RPC 2.0 messages** — Request/response/notification types
- **Entity queries** — Type-safe component queries
- **Serialization contracts** — Newtonsoft.Json attributes and models

## Usage

```csharp
using DINOForge.Bridge.Protocol;

// Message types for JSON-RPC communication
var request = new JsonRpcRequest { Method = "status", Params = null };
var response = JsonSerializer.Deserialize<JsonRpcResponse>(responseJson);
```

## Key Types

- `IGameBridge` — Core game bridge interface
- `JsonRpcRequest` / `JsonRpcResponse` / `JsonRpcNotification` — RPC message envelopes
- `EntityQuery` — Component-based entity queries
- `GameComponent` — Component metadata
- `GameEntity` — Entity data transfer object

## Installation

```bash
dotnet add package DINOForge.Bridge.Protocol
```

## Framework Support

- **.NET Standard 2.0** — For maximum compatibility with existing projects
- **.NET 8.0+** — Fully supported with latest language features

## See Also

- **DINOForge.Bridge.Client** — Out-of-process client implementation
- [DINOForge Documentation](https://kooshapari.github.io/Dino/)
- [GitHub Repository](https://github.com/KooshaPari/Dino)
