# protocol-3d

> WebSocket binary protocol extensions for the Civis 3D layer.

## Overview

The `protocol-3d` crate defines the binary wire formats and extension protocols used for the Civis 3D streaming layer. It handles complex state updates including voxel deltas, building diffs, and agent appearance changes. It provides a length-prefixed binary envelope with versioning gates to ensure client compatibility.

The protocol is optimized for high-frequency updates, offering optional zstd compression for tick bundles. This crate is essential for any client or server implementing the high-performance 3D streaming interface.

## Features

- Binary frame types for voxel deltas and building diffs
- Agent appearance update packets
- Length-prefixed binary envelope format
- Protocol versioning gate for compatibility checks
- Optional zstd compression for tick bundles

## Usage

```rust
use protocol_3d::*;
```

## Architecture

The crate defines `Frame3dBundle` as the primary container for 3D data streams. It utilizes `BuildingProvenance` to track construction history and `WorldXZ` for coordinate positioning. `DirtyChunkEvent` objects are used to signal specific regions of the world that have changed.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
