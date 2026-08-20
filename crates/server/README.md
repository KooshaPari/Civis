# server

> Civis server library exposing the 3D-extension protocol bridge.

## Overview

The `server` crate implements the core network and persistence layer for the Civis 3D server. It bridges the 3D extension protocol to client connections via JSON-RPC and WebSocket with SSE streaming. It manages the voxel frame construction, autosave loops, and subscription filtering for connected clients.

This crate is the "glue" that connects the simulation logic (voxel, simulation) to the external world. It handles the lifecycle of client sessions, ensuring efficient state transmission and synchronization.

## Features

- JSON-RPC method dispatch for client commands
- WebSocket bridge with Server-Sent Events (SSE) streaming
- Voxel frame builder for efficient delta transmission
- Background autosave loop with configurable intervals
- Subscription filtering for targeted client updates

## Usage

```rust
use server::*;
```

## Architecture

The server operates on a `WsBridgeConfig` that defines the transport settings. `JsonRpcMethod` enums route incoming requests. The `AutosaveContext` manages background persistence, while `VoxelFrameBuilder` constructs efficient binary frames for transmission.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
