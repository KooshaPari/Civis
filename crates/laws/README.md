# laws
> Versioned physics-law database for the Civis simulation engine.

## Overview

The `laws` crate defines the versioned database of physical and social laws that govern the Civis simulation world. Each law is an immutable, semantically versioned entry that describes a rule of nature or society, from gravity to taxation.

Laws are stored as typed structs that serialize to both Rust and scripting languages. The database supports snapshot-and-diff workflows, enabling the simulation engine to evolve laws across game eras without breaking saved states.

The crate provides query APIs for law lookup by ID, tag, or version range. It also includes a migration framework that can transform old law definitions into current schemas when loading legacy save files.

## Features

- Semantically versioned law definitions
- Typed law structs with serde support
- Snapshot and diff for cross-era evolution
- Query by ID, tag, category, or version range
- Save-file migration framework
- Compile-time law schema validation
- Extensible via the mod-host WASM interface

## Usage

```rust
use laws::{LawDb, Law};

let db = LawDb::load("./laws")?;
let gravity: &Law = db.get("physics.gravity")?;
println!("Version: {}", gravity.version());
```

## Architecture

Each law is a struct implementing the `Law` trait. The `LawDb` holds a B-tree of versioned entries indexed by dotted ID. Migrations are defined as functions from `(LawV1) -> LawV2` and are executed automatically during deserialization.

## License

Part of the Civis project (https://github.com/KooshaPari/Civis).
