# DINOForge.Bridge.Client

Out-of-process C# client library for communicating with the DINOForge game bridge via named pipes. Use this to query game state, apply runtime overrides, and automate testing without modifying game DLLs.

## Purpose

The Bridge Client allows external tools and tests to:

- **Query game state** — Entity counts, component data, stat values
- **Apply overrides** — Modify unit stats, weapon balance, faction properties at runtime
- **Inject input** — Simulate keyboard/mouse events without game focus
- **Reload packs** — Hot-swap mod content without restarting
- **Capture screenshots** — Analyze game visuals from scripts
- **Automate testing** — Drive end-to-end test scenarios

## Key Features

- **Named pipe transport** — Low-latency local IPC (Windows)
- **Async/await API** — Non-blocking JSON-RPC 2.0 requests
- **Type-safe queries** — Compile-time entity component definitions
- **Automatic reconnection** — Resilience to bridge restarts
- **Logging** — Serilog integration for debugging

## Installation

```bash
dotnet add package DINOForge.Bridge.Client
```

## Quick Start

```csharp
using DINOForge.Bridge.Client;

// Connect to the bridge
var client = new GameClient("dinoforge-game-bridge");
await client.ConnectAsync();

// Query game status
var status = await client.SendAsync(new StatusQuery());
Console.WriteLine($"Entities: {status.EntityCount}");

// Apply a stat override
var request = new OverrideRequest
{
    EntityId = 42,
    ComponentType = "Health",
    Field = "CurrentHealth",
    Value = 100
};
var result = await client.SendAsync(request);
if (result.Success)
    Console.WriteLine("Override applied!");

// Reload packs without restarting
await client.SendAsync(new ReloadPacksRequest());
```

## Common Operations

### Query Entities by Component

```csharp
var query = new EntityQuery { ComponentType = "Unit" };
var entities = await client.QueryAsync(query);
foreach (var entity in entities)
{
    Console.WriteLine($"Entity {entity.Id}: {string.Join(", ", entity.Components)}");
}
```

### Override Unit Health

```csharp
var request = new OverrideRequest
{
    EntityId = 10,
    ComponentType = "Health",
    Field = "CurrentHealth",
    Value = 50
};
await client.SendAsync(request);
```

### Capture Screenshot

```csharp
var screenshot = await client.SendAsync(new ScreenshotRequest { Format = "png" });
await File.WriteAllBytesAsync("game.png", screenshot.Data);
```

### Hot Reload Packs

```csharp
// Recompile your pack, then reload it in the running game without restart
await client.SendAsync(new ReloadPacksRequest());
```

## Framework Support

- **.NET Standard 2.0** — Compatible with .NET Framework 4.6.2+, .NET Core 2.0+, Mono
- **.NET 8.0+** — Full async/await support

## Error Handling

```csharp
try
{
    await client.ConnectAsync(timeout: TimeSpan.FromSeconds(5));
}
catch (TimeoutException)
{
    Console.WriteLine("Bridge not responding — is the game running?");
}
catch (JsonRpcException ex)
{
    Console.WriteLine($"Bridge error: {ex.Message}");
}
```

## Dependencies

- **DINOForge.Bridge.Protocol** — Message contracts
- **Newtonsoft.Json** — Serialization
- **Serilog** — Structured logging

## See Also

- **DINOForge.Bridge.Protocol** — RPC message definitions
- **DINOForge.SDK** — High-level mod API (registries, packs)
- [DINOForge Documentation](https://kooshapari.github.io/Dino/)
- [GitHub Repository](https://github.com/KooshaPari/Dino)
