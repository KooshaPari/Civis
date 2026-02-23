# CIV-0200: Multi-Client Protocol Specification

**Version:** 1.0
**Status:** SPECIFICATION
**Date:** 2026-02-21
**Authors:** CIV Protocol & Integration Team

---

## Executive Summary

CivLab's **multi-client protocol** enables game engines (Bevy, Unreal, Unity, Godot), web browsers, and research APIs to attach to a single deterministic simulation core. The protocol is **transport-agnostic** (WebSocket, gRPC, shared memory) but we specify:

1. **JSON-RPC 2.0 over WebSocket** (web clients, debugging, research)
2. **Binary frame format** (game engine clients requiring high frequency)
3. **Client lifecycle** (handshake → subscribe → command → disconnect)
4. **Engine-specific integration patterns** (Bevy plugin, Unreal C++, Unity C#, Web TS)

---

## Design Principles

| Principle | Rationale |
|---|---|
| **Protocol agnostic** | Multiple transport layers can co-exist (WebSocket + gRPC + shared memory) |
| **Deterministic messaging** | Same input message → identical server response (no temporal coupling) |
| **Client-side latency hiding** | Clients predict/interpolate; server broadcasts ground truth |
| **Bandwidth-efficient** | Support both high-volume JSON and compact binary frames |
| **Engine-native types** | Bevy can use bevy_reflect, Unreal uses native UStructs, etc. |
| **Version-stable** | Major version breaks are explicit; minor versions are backwards-compatible |

---

## Layer 1: JSON-RPC 2.0 over WebSocket

### Connection Lifecycle

```
┌─────────────────────────────────────────────────────────────┐
│ 1. TCP CONNECT                                              │
│    Client: TCP connect to 127.0.0.1:9876                   │
│    Server: Accept connection                               │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 2. WEBSOCKET UPGRADE                                        │
│    Client: Upgrade to WebSocket (RFC 6455)                 │
│    Server: Accept upgrade                                  │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 3. HANDSHAKE RPC                                            │
│    Client → Server: sim.handshake(client_id, version)      │
│    Server → Client: {tick, seed, snapshot}                 │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 4. SUBSCRIBE (Optional)                                     │
│    Client → Server: sim.subscribe(filter, framerate)       │
│    Server: Add client to broadcast list                    │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 5. COMMAND/SUBSCRIBE PHASE (Repeating)                      │
│    Client → Server: sim.command(action, params)            │
│    Server → Client: {accepted, tick_applied}               │
│    Server → All: sim.tick_broadcast(snapshot, events)      │
└─────────────────────────────────────────────────────────────┘
                          ↓
┌─────────────────────────────────────────────────────────────┐
│ 6. DISCONNECT                                               │
│    Client: Close WebSocket                                 │
│    Server: Cleanup ClientSession                           │
└─────────────────────────────────────────────────────────────┘
```

### Message Structure

**All WebSocket messages are JSON-RPC 2.0 compliant:**

```json
{
  "jsonrpc": "2.0",
  "id": 12345,
  "method": "method.name",
  "params": { /* method-specific */ }
}
```

**Responses (success):**
```json
{
  "jsonrpc": "2.0",
  "id": 12345,
  "result": { /* method-specific */ }
}
```

**Responses (error):**
```json
{
  "jsonrpc": "2.0",
  "id": 12345,
  "error": {
    "code": -32600,
    "message": "Invalid Request"
  }
}
```

### Method Catalog

#### 1. `sim.handshake` — Establish Client Session

**Purpose:** Client identifies itself; server returns current state + bootstrap data.

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "method": "sim.handshake",
  "params": {
    "client_id": "bevy_renderer_1",
    "client_type": "game",
    "client_version": "1.0.0",
    "protocol_version": "1",
    "platform": "Linux",
    "desired_framerate": 60
  }
}
```

**Response (Success):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": {
    "session_id": "sess_9876543210",
    "tick": 100,
    "sim_speed": 10.0,
    "seed": 54321,
    "snapshot": {
      "header": {
        "tick": 100,
        "sim_time_hours": 166.67,
        "version": "1"
      },
      "world": { /* ... */ },
      "metrics": { /* ... */ }
    },
    "server_time_ms": 1708505400000,
    "capabilities": [
      "binary_frames",
      "snapshot_filtering",
      "command_pipelining"
    ]
  }
}
```

**Response (Error):**
```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32603,
    "message": "Server overloaded",
    "data": { "retry_after_ms": 5000 }
  }
}
```

**Invariants:**
- Server MUST return current tick (not future tick)
- Snapshot MUST be deterministic for same tick
- Session ID is unique per client connection
- Client types: `game`, `research`, `admin`, `logger`

#### 2. `sim.command` — Issue Action to Simulation

**Purpose:** Client requests state change (build, move, trade, declare war, etc.)

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "method": "sim.command",
  "params": {
    "action": "build",
    "subject_entity": 1001,
    "building_type": "farm",
    "target_position": {"x": 10, "y": 20},
    "resources": {
      "grain": 50,
      "labor_hours": 100
    }
  }
}
```

**Supported Actions:**
- `build` — Construct building
- `move` — Move unit or agent
- `trade` — Initiate trade agreement
- `produce` — Set production order
- `harvest` — Gather resources
- `attack` — Declare war/attack entity
- `diplomacy` — Propose treaty, alliance, etc.
- `policy_set` — Change policy parameter (admin only)
- `spawn_entity` — Create new entity (admin/research only)

**Response (Accepted):**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "result": {
    "accepted": true,
    "command_id": "cmd_123456",
    "tick_issued": 100,
    "tick_applied": 101,
    "cost": {
      "grain": 50,
      "labor_hours": 100
    },
    "duration_ticks": 10,
    "estimated_completion_tick": 111
  }
}
```

**Response (Rejected):**
```json
{
  "jsonrpc": "2.0",
  "id": 2,
  "error": {
    "code": -32001,
    "message": "Command rejected",
    "data": {
      "reason": "insufficient_resources",
      "available_grain": 30,
      "required_grain": 50
    }
  }
}
```

**Invariants:**
- Command MUST NOT execute if resources unavailable
- Tick applied MUST be > tick issued
- Conflicting commands from same client are serialized (FIFO)
- Conflicting commands from different clients resolved by priority

#### 3. `sim.snapshot` — Request State at Specific Tick

**Purpose:** Client requests historical or current snapshot (useful for debugging, research).

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "method": "sim.snapshot",
  "params": {
    "tick": 105,
    "include": [
      "world.cells",
      "world.agents",
      "metrics.all"
    ],
    "region": {
      "x_min": 0,
      "y_min": 0,
      "x_max": 100,
      "y_max": 100
    },
    "include_history": false
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 3,
  "result": {
    "snapshot": {
      "header": {
        "tick": 105,
        "sim_time_hours": 175.0,
        "version": "1"
      },
      "world": {
        "cells": [
          {
            "x": 0, "y": 0,
            "terrain": "grassland",
            "fertility": 95,
            "population": 450
          },
          /* ... filtered by region ... */
        ],
        "agents": [ /* ... */ ]
      },
      "metrics": { /* ... */ }
    },
    "retrieved_at_tick": 109
  }
}
```

**Invariants:**
- If `tick` is in past, snapshot MUST be from replay/cache (deterministic)
- If `tick` is current or future, snapshot is current state (may change)
- Region filtering is optional; if omitted, return all cells
- Include filtering is optional; if omitted, return full snapshot

#### 4. `sim.subscribe` — Stream Ticks to Client

**Purpose:** Client subscribes to tick broadcasts (most efficient for real-time game clients).

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "method": "sim.subscribe",
  "params": {
    "filter": [
      "entities.agents",
      "entities.buildings",
      "events.all",
      "metrics.gdp",
      "metrics.happiness"
    ],
    "region": {
      "x_min": 0,
      "y_min": 0,
      "x_max": 100,
      "y_max": 100
    },
    "framerate": 60,
    "use_binary_frames": false
  }
}
```

**Response (Immediate):**
```json
{
  "jsonrpc": "2.0",
  "id": 4,
  "result": {
    "subscribed": true,
    "subscription_id": "sub_7654321",
    "frame_frequency": "every_tick",
    "protocol": "json_rpc"
  }
}
```

**Broadcast Messages (Async, every tick or every N ticks):**
```json
{
  "jsonrpc": "2.0",
  "method": "sim.tick_broadcast",
  "params": {
    "subscription_id": "sub_7654321",
    "tick": 101,
    "snapshot": {
      "header": { "tick": 101, ... },
      "world": {
        "agents": [ /* filtered */ ],
        "buildings": [ /* filtered */ ]
      },
      "metrics": {
        "gdp": 5000000,
        "happiness": 42
      }
    },
    "events": [
      {
        "tick": 101,
        "type": "production.completed",
        "entity": 1001,
        "data": { "output": 150 }
      }
    ]
  }
}
```

**Invariants:**
- Subscription is asynchronous (no response per frame; only async broadcasts)
- Frame frequency depends on client framerate and server load
- Filtering reduces bandwidth (only requested entity types/regions sent)
- Binary frames (see Layer 2) are more bandwidth-efficient for game engines

#### 5. `sim.query` — Run Diagnostic Query on State

**Purpose:** Research/admin clients query simulation state (e.g., "all agents in region X", "economy ledger for institution Y").

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "method": "sim.query",
  "params": {
    "query_type": "agent_in_region",
    "region": { "x_min": 10, "y_min": 20, "x_max": 30, "y_max": 40 },
    "limit": 100
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 5,
  "result": {
    "query_result": [
      {
        "entity_id": 10001,
        "position": {"x": 15, "y": 25},
        "health": 87,
        "mood": {"happiness": 35, "legitimacy": 62}
      }
    ],
    "count": 42,
    "limit_exceeded": false
  }
}
```

**Supported Queries:**
- `agent_in_region` — All agents in bounding box
- `building_by_type` — All buildings of type X
- `institution_ledger` — Financial ledger for institution
- `market_prices` — Price history for good at location
- `event_log` — Events in tick range
- `replay_info` — Metadata about current run

#### 6. `sim.unsubscribe` — Stop Receiving Broadcasts

**Request:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "method": "sim.unsubscribe",
  "params": {
    "subscription_id": "sub_7654321"
  }
}
```

**Response:**
```json
{
  "jsonrpc": "2.0",
  "id": 6,
  "result": { "unsubscribed": true }
}
```

---

## Layer 2: Binary Frame Format (Game Engines)

### Problem & Motivation

JSON-RPC is flexible but verbose. A 60 FPS game client needs ~60 snapshots/sec. JSON overhead grows linearly with world size.

**Solution:** Optional binary frame format (more compact, zero deserialization overhead in C++).

### Frame Structure

```
┌─────────────────────────────────────────────────────────────┐
│ HEADER (16 bytes)                                           │
├─────────────────────────────────────────────────────────────┤
│ [0-3]   Tick number (u32, big-endian)                       │
│ [4-7]   Event count (u32, big-endian)                       │
│ [8-11]  Snapshot size (u32, big-endian)                     │
│ [12-15] Flags (u32, big-endian)                             │
│         Bit 0: Has events                                   │
│         Bit 1: Compression (0=none, 1=zstd)                │
│         Bit 2: Reserved                                     │
│         Bits 3-31: Reserved                                 │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│ SNAPSHOT PAYLOAD                                            │
├─────────────────────────────────────────────────────────────┤
│ [16 ... 16+snapshot_size]                                   │
│ If (flags & 0x2):                                           │
│   zstd-compressed JSON snapshot                             │
│ Else:                                                       │
│   Raw JSON snapshot (as string)                             │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│ EVENT PAYLOAD (if event_count > 0)                          │
├─────────────────────────────────────────────────────────────┤
│ For each event:                                             │
│   [4 bytes] Event size (u32, big-endian)                    │
│   [N bytes] Event JSON                                      │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│ OPTIONAL CHECKSUM (4 bytes)                                 │
├─────────────────────────────────────────────────────────────┤
│ CRC32 of entire frame (big-endian)                          │
│ Only if flags indicate checksum enabled                     │
└─────────────────────────────────────────────────────────────┘
```

### Example: Unreal C++ Integration

```cpp
#include <cstdint>
#include <vector>
#include <zstd.h>

struct TickFrame {
    uint32_t tick;
    uint32_t event_count;
    std::string snapshot_json;
    std::vector<std::string> events;
};

class FrameUnpacker {
public:
    static TickFrame unpack(const std::vector<uint8_t>& raw) {
        TickFrame frame;

        // Parse header
        frame.tick = read_u32_be(raw, 0);
        frame.event_count = read_u32_be(raw, 4);
        uint32_t snapshot_size = read_u32_be(raw, 8);
        uint32_t flags = read_u32_be(raw, 12);

        size_t offset = 16;

        // Parse snapshot
        bool has_compression = (flags & 0x2) != 0;
        if (has_compression) {
            std::vector<uint8_t> compressed(raw.begin() + offset,
                                           raw.begin() + offset + snapshot_size);
            frame.snapshot_json = zstd_decompress(compressed);
        } else {
            frame.snapshot_json = std::string(
                (const char*)(raw.data() + offset),
                snapshot_size
            );
        }
        offset += snapshot_size;

        // Parse events
        for (uint32_t i = 0; i < frame.event_count; i++) {
            uint32_t event_size = read_u32_be(raw, offset);
            offset += 4;
            frame.events.push_back(
                std::string((const char*)(raw.data() + offset), event_size)
            );
            offset += event_size;
        }

        return frame;
    }

private:
    static uint32_t read_u32_be(const std::vector<uint8_t>& data, size_t offset) {
        return ((uint32_t)data[offset] << 24) |
               ((uint32_t)data[offset+1] << 16) |
               ((uint32_t)data[offset+2] << 8) |
               ((uint32_t)data[offset+3]);
    }

    static std::string zstd_decompress(const std::vector<uint8_t>& compressed) {
        unsigned long long frame_size = ZSTD_getFrameContentSize(
            compressed.data(),
            compressed.size()
        );
        std::string output(frame_size, 0);
        ZSTD_decompress(output.data(), frame_size,
                       compressed.data(), compressed.size());
        return output;
    }
};
```

### Using Binary Frames in Subscription

**Request:**
```json
{
  "method": "sim.subscribe",
  "params": {
    "filter": ["entities.agents", "metrics.all"],
    "use_binary_frames": true
  }
}
```

**Response:**
```json
{
  "result": {
    "subscribed": true,
    "protocol": "binary_frame",
    "frame_version": 1
  }
}
```

**Broadcasts:** Raw binary frames on WebSocket (not JSON-RPC wrapped).

---

## Engine-Specific Integration Patterns

### Pattern 1: Bevy (Rust, Client-Side)

**Plugin Architecture:**

```rust
use bevy::prelude::*;
use civlab_protocol::{CivLabClient, Snapshot};

pub struct CivLabPlugin {
    server_url: String,
}

#[derive(Component)]
pub struct CivLabSession {
    client: CivLabClient,
    subscription_id: String,
}

pub fn setup_civlab(mut commands: Commands, plugin: Res<CivLabPlugin>) {
    let client = CivLabClient::new(&plugin.server_url);
    let handshake = client.handshake("bevy_client_1", "game").await.unwrap();

    let subscription = client.subscribe(
        vec!["entities.agents", "entities.buildings"],
        None, // filter
        60,   // framerate
    ).await.unwrap();

    commands.insert_resource(CivLabSession {
        client,
        subscription_id: subscription.id,
    });
}

pub fn tick_system(
    session: Res<CivLabSession>,
    mut query: Query<(&mut Transform, &mut Sprite), With<CivLabEntity>>,
) {
    // Subscribe broadcasts frames to event stream
    // This system reads from the event stream
    while let Ok(frame) = session.client.recv_frame() {
        for agent in &frame.snapshot.world.agents {
            // Update Bevy entity transform from snapshot agent position
            let bevy_entity = /* lookup by civlab entity id */;
            if let Ok((mut transform, _)) = query.get_mut(bevy_entity) {
                transform.translation.x = agent.position.x as f32;
                transform.translation.y = agent.position.y as f32;
            }
        }
    }
}

pub fn command_system(
    session: Res<CivLabSession>,
    input: Res<Input<KeyCode>>,
) {
    if input.just_pressed(KeyCode::B) {
        // Player clicked "Build Farm"
        let cmd = Command {
            action: "build".to_string(),
            params: json!({
                "building_type": "farm",
                "position": {"x": 10, "y": 20}
            }),
        };
        session.client.command(cmd).await.unwrap();
    }
}

pub struct CivLabPlugin;

impl Plugin for CivLabPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_startup_system(setup_civlab)
            .add_system(tick_system)
            .add_system(command_system);
    }
}
```

**Usage:**

```rust
fn main() {
    App::new()
        .add_plugin(CivLabPlugin {
            server_url: "ws://localhost:9876".to_string(),
        })
        // ... other plugins
        .run();
}
```

### Pattern 2: Unreal Engine (C++, Client-Side)

**Plugin Structure:**

```cpp
// CivLabPlugin.h
#pragma once

#include "CoreMinimal.h"
#include "Modules/ModuleManager.h"
#include "WebSocket.h"
#include "TickFrame.h"

class FCivLabPlugin : public IModuleInterface {
public:
    virtual void StartupModule() override;
    virtual void ShutdownModule() override;
};

// CivLabClientSubsystem.h
#pragma once

#include "Subsystems/WorldSubsystem.h"
#include "TickFrame.h"
#include "CivLabClientSubsystem.generated.h"

UCLASS()
class CIVLAB_API UCivLabClientSubsystem : public UWorldSubsystem {
    GENERATED_BODY()

public:
    virtual void Initialize(FSubsystemCollectionBase& Collection) override;
    virtual void Deinitialize() override;

    UFUNCTION(BlueprintCallable)
    void Connect(const FString& ServerURL);

    UFUNCTION(BlueprintCallable)
    void SendCommand(const FString& Action, const FString& Params);

    UPROPERTY(BlueprintReadOnly)
    FTickFrame CurrentSnapshot;

private:
    void OnWebSocketConnected();
    void OnWebSocketMessage(const FString& Msg);
    void OnWebSocketClosed(int32 StatusCode, const FString& Reason);

    TSharedPtr<IWebSocket> WebSocket;
    bool bConnected = false;
};

// CivLabClientSubsystem.cpp
void UCivLabClientSubsystem::Connect(const FString& ServerURL) {
    WebSocket = FWebSocketsModule::Get().CreateWebSocket(ServerURL);
    WebSocket->OnConnected().AddDynamic(this, &UCivLabClientSubsystem::OnWebSocketConnected);
    WebSocket->OnMessage().AddDynamic(this, &UCivLabClientSubsystem::OnWebSocketMessage);
    WebSocket->OnClosed().AddDynamic(this, &UCivLabClientSubsystem::OnWebSocketClosed);
    WebSocket->Connect();
}

void UCivLabClientSubsystem::OnWebSocketConnected() {
    // Send handshake
    FString HandshakeMsg = TEXT(R"({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "sim.handshake",
        "params": {
            "client_id": "unreal_client_1",
            "client_type": "game",
            "client_version": "1.0.0"
        }
    })");
    WebSocket->Send(HandshakeMsg);

    // Subscribe to snapshots
    FString SubscribeMsg = TEXT(R"({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "sim.subscribe",
        "params": {
            "filter": ["entities.agents", "entities.buildings"],
            "use_binary_frames": true
        }
    })");
    WebSocket->Send(SubscribeMsg);
}

void UCivLabClientSubsystem::OnWebSocketMessage(const FString& Msg) {
    if (Msg.Contains(TEXT("tick_broadcast"))) {
        // Parse binary frame
        TArray<uint8> RawData = ConvertStringToUint8Array(Msg);
        CurrentSnapshot = FrameUnpacker::Unpack(RawData);

        // Trigger blueprint event
        OnSnapshotReceived.Broadcast(CurrentSnapshot);
    }
}
```

### Pattern 3: Unity (C#, Client-Side)

```csharp
using UnityEngine;
using UnityEngine.Networking;
using NativeWebSocket;
using Newtonsoft.Json.Linq;

public class CivLabClient : MonoBehaviour {
    private WebSocket webSocket;
    private bool isConnected = false;

    [SerializeField]
    private string serverUrl = "ws://localhost:9876";

    public delegate void OnSnapshotReceived(Snapshot snapshot);
    public event OnSnapshotReceived SnapshotReceived;

    async void Start() {
        await Connect();
    }

    private async System.Threading.Tasks.Task Connect() {
        webSocket = new WebSocket(serverUrl);

        webSocket.OnOpen += OnWebSocketOpen;
        webSocket.OnMessage += OnWebSocketMessage;
        webSocket.OnError += OnWebSocketError;
        webSocket.OnClose += OnWebSocketClose;

        await webSocket.Connect();
    }

    private void OnWebSocketOpen() {
        isConnected = true;
        Debug.Log("Connected to CivLab");

        // Send handshake
        var handshake = new {
            jsonrpc = "2.0",
            id = 1,
            method = "sim.handshake",
            @params = new {
                client_id = "unity_client_1",
                client_type = "game",
                client_version = "1.0.0"
            }
        };
        webSocket.SendText(JsonConvert.SerializeObject(handshake));

        // Subscribe
        var subscribe = new {
            jsonrpc = "2.0",
            id = 2,
            method = "sim.subscribe",
            @params = new {
                filter = new[] { "entities.agents", "entities.buildings" }
            }
        };
        webSocket.SendText(JsonConvert.SerializeObject(subscribe));
    }

    private void OnWebSocketMessage(byte[] data) {
        string message = System.Text.Encoding.UTF8.GetString(data);
        var json = JObject.Parse(message);

        if (json["method"]?.Value<string>() == "sim.tick_broadcast") {
            var snapshot = JsonConvert.DeserializeObject<Snapshot>(
                json["params"]["snapshot"].ToString()
            );
            SnapshotReceived?.Invoke(snapshot);
        }
    }

    private void OnWebSocketError(string errorMsg) {
        Debug.LogError($"WebSocket error: {errorMsg}");
    }

    private void OnWebSocketClose(WebSocketCloseCode code) {
        isConnected = false;
        Debug.Log("Disconnected from CivLab");
    }

    public void SendCommand(string action, JObject @params) {
        if (!isConnected) return;

        var command = new {
            jsonrpc = "2.0",
            id = System.Guid.NewGuid().GetHashCode(),
            method = "sim.command",
            @params = new {
                action,
                @params
            }
        };
        webSocket.SendText(JsonConvert.SerializeObject(command));
    }

    async void OnDestroy() {
        if (webSocket != null) {
            await webSocket.Close();
        }
    }
}
```

### Pattern 4: Web (TypeScript + React/Vue)

```typescript
// civlab-client.ts
import { EventEmitter } from 'events';

export interface Snapshot {
  header: {
    tick: number;
    sim_time_hours: number;
  };
  world: {
    agents: Agent[];
    buildings: Building[];
  };
  metrics: Metrics;
  events: Event[];
}

export interface Command {
  action: string;
  [key: string]: any;
}

export class CivLabClient extends EventEmitter {
  private ws: WebSocket | null = null;
  private nextId = 1;
  private pendingRequests = new Map<number, (result: any) => void>();

  async connect(serverUrl: string): Promise<void> {
    return new Promise((resolve, reject) => {
      try {
        this.ws = new WebSocket(serverUrl);

        this.ws.onopen = () => {
          this.handshake().then(resolve).catch(reject);
        };

        this.ws.onmessage = (event) => {
          const msg = JSON.parse(event.data);
          if (msg.id) {
            const resolve = this.pendingRequests.get(msg.id);
            if (resolve) {
              resolve(msg.result || msg.error);
              this.pendingRequests.delete(msg.id);
            }
          }
          if (msg.method === 'sim.tick_broadcast') {
            this.emit('tick', msg.params);
          }
        };

        this.ws.onerror = reject;
      } catch (error) {
        reject(error);
      }
    });
  }

  private async handshake(): Promise<void> {
    const result = await this.rpc('sim.handshake', {
      client_id: 'web_client_1',
      client_type: 'game',
      client_version: '1.0.0',
    });
    console.log('Handshake successful, tick:', result.tick);
  }

  async subscribe(filter: string[]): Promise<void> {
    await this.rpc('sim.subscribe', {
      filter,
      framerate: 60,
    });
    console.log('Subscribed to:', filter);
  }

  async command(cmd: Command): Promise<any> {
    return this.rpc('sim.command', cmd);
  }

  private rpc(method: string, params: any): Promise<any> {
    return new Promise((resolve) => {
      const id = this.nextId++;
      this.pendingRequests.set(id, resolve);

      const msg = {
        jsonrpc: '2.0',
        id,
        method,
        params,
      };

      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify(msg));
      }
    });
  }

  disconnect(): void {
    if (this.ws) {
      this.ws.close();
      this.ws = null;
    }
  }
}

// React hook
import { useEffect, useState } from 'react';

export function useCivLab(serverUrl: string) {
  const [client] = useState(() => new CivLabClient());
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);

  useEffect(() => {
    client.connect(serverUrl).then(() => {
      client.subscribe(['entities.agents', 'entities.buildings']);
      client.on('tick', (params) => {
        setSnapshot(params.snapshot);
      });
    });

    return () => client.disconnect();
  }, [serverUrl]);

  const sendCommand = (cmd: Command) => client.command(cmd);

  return { snapshot, sendCommand };
}
```

---

## Latency & Frame Sync Protocol

### Problem: Multi-Frame Latency

**Scenario:**
- Client sends command at tick 100
- Server processes at tick 101
- Server broadcasts result at tick 102
- Client receives at tick 103 (local render frame)

**Local timing:** Client is 3 ticks behind server (300 ms at 10 ticks/sec).

### Solution: Predictive Client-Side Extrapolation

**Strategy:**
1. Client predicts next state (same simulation logic as server)
2. Renders predicted state (looks responsive)
3. When server broadcast arrives, blend to authoritative state
4. If mismatch > threshold, hard reset to server state

**Code (Bevy example):**

```rust
pub struct PredictiveClient {
    local_state: State,
    server_state: State,
    local_tick: u64,
    server_tick: u64,
}

impl PredictiveClient {
    pub fn predict_frame(&mut self, dt: f32) {
        // Run local simulation (same logic as server, but ~every frame instead of every 100ms)
        let control = Control::from_local_input(); // Player input
        self.local_state = self.local_state.apply_deterministic_transition(&control);
        self.local_tick += 1;
    }

    pub fn receive_server_broadcast(&mut self, server_snapshot: &Snapshot) {
        // Server is authoritative
        self.server_state = server_snapshot.to_state();
        self.server_tick = server_snapshot.header.tick;

        // Blend local to server (or hard reset if drift > 10 ticks)
        if (self.local_tick as i64 - self.server_tick as i64).abs() > 10 {
            self.local_state = self.server_state.clone();
            self.local_tick = self.server_tick;
        }
    }

    pub fn render_state(&self) -> &State {
        // Render local state (which is ~3 ticks ahead of server, but responsive)
        &self.local_state
    }
}
```

---

## Acceptance Criteria

### FR-CIV-PROTO-001: JSON-RPC 2.0 Compliance
**Spec:** All messages comply with JSON-RPC 2.0 spec (id, jsonrpc, method/result/error).
**Test:** Send malformed message, verify error response is JSON-RPC compliant.
**Status:** Open

### FR-CIV-PROTO-002: WebSocket Transport
**Spec:** Server accepts WebSocket connections on port 9876; upgrade from HTTP.
**Test:** Connect with standard WebSocket client; verify upgrade succeeds.
**Status:** Open

### FR-CIV-PROTO-003: Handshake Protocol
**Spec:** Client sends handshake, receives current tick + seed + snapshot.
**Test:** Send handshake, verify response includes tick > 0 and snapshot.
**Status:** Open

### FR-CIV-PROTO-004: Command Acceptance
**Spec:** Commands accepted if resources sufficient; rejected with reason if not.
**Test:** Send command with insufficient resources; verify rejection with data.reason.
**Status:** Open

### FR-CIV-PROTO-005: Snapshot Filtering
**Spec:** Subscribe with filter; receive only requested entity types/regions.
**Test:** Subscribe with filter ["entities.agents"], verify buildings absent from broadcast.
**Status:** Open

### FR-CIV-PROTO-006: Binary Frame Format
**Spec:** Support binary frames with zstd compression; unpack without errors.
**Test:** Subscribe with use_binary_frames=true; verify frames unpack correctly.
**Status:** Open

### FR-CIV-PROTO-007: Multi-Client Simultaneous
**Spec:** Multiple clients connect simultaneously; commands don't interfere.
**Test:** Connect 5 clients, issue commands in parallel, verify all succeed/fail as expected.
**Status:** Open

### FR-CIV-PROTO-008: Command Ordering (Priority)
**Spec:** Commands ordered by client_priority, then tick_received.
**Test:** Issue conflicting commands from priority 0 and 1 clients; verify priority 0 wins.
**Status:** Open

### FR-CIV-PROTO-009: Subscription Unsubscribe
**Spec:** Client can unsubscribe from broadcasts.
**Test:** Subscribe, receive 5 frames, unsubscribe, verify no more frames.
**Status:** Open

### FR-CIV-PROTO-010: Query API
**Spec:** Research clients can query state (agent_in_region, institution_ledger, etc.).
**Test:** Query agent_in_region, verify results are deterministic and complete.
**Status:** Open

### FR-CIV-PROTO-011: Error Handling
**Spec:** All errors return JSON-RPC error format with code, message, optional data.
**Test:** Send invalid command; verify error response matches spec.
**Status:** Open

### FR-CIV-PROTO-012: Bevy Plugin Integration
**Spec:** Bevy client can connect, subscribe, and render agent positions from snapshots.
**Test:** Run example_bevy_client, verify agents render at correct positions.
**Status:** Open

### FR-CIV-PROTO-013: Unreal C++ Integration
**Spec:** Unreal plugin can unpack binary frames and update AActor transforms.
**Test:** Run example_unreal_client, verify actors move in sync with snapshots.
**Status:** Open

### FR-CIV-PROTO-014: Unity C# Integration
**Spec:** Unity client can connect via WebSocket and render snapshots.
**Test:** Run example_unity_client, verify game objects update from snapshots.
**Status:** Open

### FR-CIV-PROTO-015: Web TypeScript Integration
**Spec:** Web client can connect, subscribe, and render in React/Vue.
**Test:** Run example_web_client in browser, verify agents render and respond to commands.
**Status:** Open

---

## References

- **CIV-0001:** Core Simulation Loop spec
- **RFC 6455:** The WebSocket Protocol
- **JSON-RPC 2.0 Specification:** https://www.jsonrpc.org/specification
- **Zstandard (zstd):** https://github.com/facebook/zstd
- **Bevy:** https://bevyengine.org/
- **Unreal Engine 5:** https://www.unrealengine.com/
- **Unity:** https://unity.com/
- **Godot:** https://godotengine.org/

---

**Version History:**
- v1.0 (2026-02-21): Initial specification. JSON-RPC 2.0, binary frames, multi-engine patterns.
