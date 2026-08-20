# watch

> Local hot-reload sandbox harness for Civis 3D.

## Overview

The `watch` crate provides a local development environment for the Civis 3D simulation. It runs a background simulation loop at roughly 10 Hz and exposes it via an HTTP server. Clients can receive real-time updates via SSE (Server-Sent Events) or query the current state.

It includes a dashboard static build and a set of control endpoints for manipulating the sandbox (spawning units, placing voxels, adjusting speed). This is the primary tool for debugging and iterating on game mechanics locally.

## Features

- Background simulation loop at ~10 Hz
- SSE streaming at `GET /events`
- Current state snapshots at `GET /snapshot`
- Procedural heightmap generation at `GET /terrain`
- Interactive control endpoints (`POST /control/*`)
- Static dashboard serving at `GET /`

## Usage

```rust
use watch::*;
```

## Architecture

The harness is centered around a simulation loop that ticks the game world. The HTTP server exposes the state to clients. `WatchError` defines the error cases for the harness operations. Control commands are routed to specific simulation subsystems.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
