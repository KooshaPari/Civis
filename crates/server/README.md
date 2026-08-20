# civ-server

**Server application for the CivLab deterministic simulation engine.**

The `civ-server` crate acts as the primary interface for the Civis simulation, providing a JSON-RPC WebSocket bridge, health endpoints, and 3D protocol frame builders. It handles client connections, manages simulation state (including autosaves), and exposes the simulation to renderers and replay tools.

## Key Types

- `JsonRpcRequest` / `JsonRpcResponse`: Standard JSON-RPC 2.0 messaging types.
- `DispatchContext`: Context provided to JSON-RPC method handlers.
- `AutosaveContext`: Context for managing automated simulation saves.
- `VoxelFrameBuilder`: Converts simulation voxel events into 3D protocol frames.

## Usage Example

The server is typically run as a binary:

```bash
cargo run -p civ-server
```

As a library, you can dispatch requests:

```rust
use civ_server::{DispatchContext, dispatch_request, parse_request};

let raw_json = r#"{"jsonrpc": "2.0", "method": "tick", "id": 1}"#;
let request = parse_request(raw_json).unwrap();
// let response = dispatch_request(request, &context);
```

## Dependencies

The server integrates with almost all other Civis crates:

- [`civ-engine`](../engine): Core simulation engine.
- [`civ-agents`](../agents): Agent systems.
- [`civ-economy`](../economy): Economic simulation.
- [`civ-voxel`](../voxel): Voxel world data.
- [`civ-protocol-3d`](../protocol-3d): 3D streaming protocol.
- [`civ-build`](../build): Building systems.
- [`civ-mod-host`](../mod-host): Modding interface.
- [`civ-save-db`](../save-db): Persistent storage for saves.
- [`civ-observability`](../observability): Logging and tracing.
