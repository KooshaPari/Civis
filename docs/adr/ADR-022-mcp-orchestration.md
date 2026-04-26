# ADR-022: MCP Orchestration Model

**Date**: 2026-04-04
**Status**: Proposed
**Deciders**: DINOForge Architecture Team

## Context

DINOForge's MCP (Model Context Protocol) server has grown from a simple bridge to a complex orchestration layer. As we add multi-instance support, distributed testing, and agent-driven workflows, we need a formal model for how MCP servers coordinate, discover, and communicate.

Current state:
- Single MCP server per game instance
- HTTP transport on fixed port (8765)
- Direct game bridge integration
- No server-to-server communication

Target state:
- Multiple MCP servers (one per instance)
- Dynamic port allocation
- Server discovery and registry
- Coordinator for multi-instance workflows

## Decision Drivers

- **Scalability**: Support 10+ concurrent instances
- **Discoverability**: Auto-discover available MCP endpoints
- **Coordination**: Synchronize actions across instances
- **Reliability**: Handle server crashes and restarts
- **Extensibility**: Support future transport protocols

## Options Considered

### Option A: Static Configuration

Pre-configured port assignments, manual client configuration.

**Pros**:
- Simple implementation
- No discovery complexity
- Predictable endpoints

**Cons**:
- Manual configuration burden
- Port conflicts if range exhausted
- No dynamic scaling
- Hard-coded limits

### Option B: Registry-Based Discovery (Selected)

Central registry service tracks active MCP servers with metadata.

**Pros**:
- Dynamic server registration
- Rich metadata (instance type, status, capabilities)
- Service health monitoring
- Enables load balancing

**Cons**:
- Additional infrastructure (registry service)
- Single point of failure (mitigated)
- More complex architecture

**Registry Schema**:
```json
{
  "mcp_servers": [
    {
      "id": "dino-instance-0",
      "transport": "http",
      "endpoint": "http://localhost:8765",
      "port": 8765,
      "instance_type": "primary",
      "status": "ready",
      "capabilities": ["game_status", "spawn_unit", "screenshot"],
      "game_state": "main_menu",
      "packs_loaded": ["warfare-starwars", "example-balance"],
      "last_heartbeat": "2026-04-04T12:00:00Z"
    }
  ]
}
```

### Option C: mDNS/Bonjour Discovery

Multicast DNS for automatic service discovery.

**Pros**:
- Zero configuration
- Industry standard (AirDrop, Chromecast)
- Decentralized

**Cons**:
- Limited to local network
- Windows support less robust
- Adds network complexity
- Firewall issues common

### Option D: Message Bus (Redis/RabbitMQ)

Central message broker for all MCP communication.

**Pros**:
- Pub/sub patterns
- Persistent queues
- Scales horizontally

**Cons**:
- Heavyweight for local use
- Additional dependency
- Overkill for single-machine scenarios

## Decision

**Adopt Option B (Registry-Based Discovery)** with a lightweight file-based registry for local development and optional Redis for distributed scenarios.

### Architecture

```
MCP Orchestration Architecture
==============================

Coordinator (McpCoordinator)
├── Registry (IMcpRegistry)
│   ├── FileBasedRegistry (default)
│   │   └── ~/.dinoforge/mcp-registry.json
│   └── RedisRegistry (distributed)
│       └── redis://localhost:6379
├── Discovery (IMcpDiscovery)
│   ├── PortScanner (find available ports)
│   └── HealthChecker (verify endpoints)
└── LoadBalancer (IMcpLoadBalancer)
    ├── RoundRobin (default)
    └── LeastConnections (optional)

Per-Instance MCP Server
├── Transport (IMcpTransport)
│   ├── HttpTransport (current)
│   └── SseTransport (future)
├── GameBridge (IGameBridge)
│   ├── ECS queries
│   ├── Entity manipulation
│   └── State serialization
└── Heartbeat (IMcpHeartbeat)
    └── Every 30s to registry

Client (Claude Code / Companion)
├── RegistryClient
│   └── Fetch available servers
├── ConnectionPool
│   └── Maintain connections to N servers
└── RetryPolicy
    └── Exponential backoff
```

### Registry API

```csharp
public interface IMcpRegistry
{
    // Server registration
    Task RegisterAsync(McpServerInfo server);
    Task DeregisterAsync(string serverId);
    
    // Discovery
    Task<IReadOnlyList<McpServerInfo>> ListAsync(
        ServerFilter? filter = null);
    Task<McpServerInfo?> GetAsync(string serverId);
    
    // Health
    Task UpdateHeartbeatAsync(string serverId);
    Task MarkUnhealthyAsync(string serverId, string reason);
}

public record McpServerInfo
{
    public string Id { get; init; }
    public string Transport { get; init; } // "http", "sse"
    public string Endpoint { get; init; }
    public int Port { get; init; }
    public string InstanceType { get; init; } // "primary", "test", "scenario"
    public string Status { get; init; } // "starting", "ready", "busy", "error"
    public IReadOnlyList<string> Capabilities { get; init; }
    public string? GameState { get; init; }
    public IReadOnlyList<string>? PacksLoaded { get; init; }
    public DateTimeOffset LastHeartbeat { get; init; }
}
```

### Port Allocation Strategy

| Range | Purpose | Allocation |
|-------|---------|------------|
| 8765 | Primary instance | Fixed |
| 8766-8799 | Test instances | Dynamic |
| 8800-8899 | Scenario instances | Dynamic |
| 8900+ | Reserved | Future |

### Heartbeat Protocol

```
MCP Server ──Heartbeat 30s──► Registry
     │                            │
     ◄────Ack/Refresh────────────┘
     
Missing 3 heartbeats → Mark unhealthy
Missing 5 heartbeats → Auto-deregister
```

### Multi-Instance Workflows

| Workflow | Coordinator Role | Example |
|----------|------------------|---------|
| **Parallel Test** | Distribute tests | Run 10 tests across 5 instances |
| **A/B Compare** | Synchronize state | Same scenario, different packs |
| **Load Test** | Scale instances | Spawn 1000 units across 10 instances |
| **Scenario Recording** | Orchestrate sequence | Record gameplay on instance 1, replay on 2-5 |

## Consequences

### Positive

- **Dynamic Scaling**: Add/remove instances without reconfiguration
- **Fault Tolerance**: Handle server crashes gracefully
- **Rich Metadata**: Make informed routing decisions
- **Multi-Machine Ready**: Foundation for distributed testing
- **Observability**: Central view of all MCP endpoints

### Negative

- **Complexity**: More moving parts than static config
- **Registry Dependency**: Must be available for discovery
- **Consistency Challenges**: Distributed state management

### Neutral

- **Migration Path**: Existing single-server setups continue working
- **Configuration**: Can opt into registry or stay static

## Implementation Plan

### Phase 1: Registry Foundation
- [ ] IMcpRegistry interface
- [ ] FileBasedRegistry implementation
- [ ] McpServer registration on startup
- [ ] Heartbeat mechanism

### Phase 2: Discovery & Coordination
- [ ] McpCoordinator service
- [ ] Port allocation algorithm
- [ ] Health checking
- [ ] Client-side discovery

### Phase 3: Advanced Features
- [ ] Load balancing strategies
- [ ] Workflow orchestration
- [ ] Redis registry (distributed)
- [ ] WebSocket transport option

## Related ADRs

- ADR-020: Multi-Instance Concurrency Architecture (foundation)
- ADR-014: Runtime Execution Model (MCP server basis)

## References

- Model Context Protocol: https://modelcontextprotocol.io/
- Service Discovery Patterns: https://microservices.io/patterns/service-discovery.html
- Consul Service Mesh: https://www.consul.io/
- etcd Distributed Store: https://etcd.io/
