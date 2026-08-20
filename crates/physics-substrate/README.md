# physics-substrate

> Shared physics-field substrate (Temperature, Mass, Energy, Force, Pressure, Biomass) -- the only public write path between emergent layers.

## Overview

The civ-physics-substrate crate defines the shared physics fields that connect the emergent simulation layers in Civis. It exposes typed field grids for Temperature, Mass, Energy, Force, Pressure, and Biomass. This crate is the sole public write boundary between physics, biology, and society layers.

## Features

- Six core physics fields: T, M, E, F, P, B
- Grid-based field storage with glam vector math
- Typed read and write accessors per field
- Serde serialization for persistence and networking
- Strict ownership model: only substrate methods mutate fields

## Usage

```rust
use civ_physics_substrate::{Substrate, Field};

let mut sub = Substrate::new(width, height);
sub.write(Field::Temperature, x, y, 372.15);
let temp = sub.read(Field::Temperature, x, y);
```

## Architecture

The Substrate struct owns a set of 2D grids, one per field type. Read access is public to all layers. Write access is restricted to methods on Substrate itself, enforcing the single-write-path invariant. Fields are stored as f32 arrays and indexed by (x, y) coordinates. Serialization uses serde for save/load round-trips.

## License

Part of the Civis project.
