# holocron

> Holocron Keycap UI verb registry substrate.

## Overview

The `holocron` crate serves as the catalog layer for godgame verbs in the Civis UI. It enumerates, classifies, and queries the available verbs that players can use to interact with the simulation.

It includes 52 verbs with a descriptor schema, group classification, provenance tracking, and risk tiers. The registry provides context-aware ranking and fuzzy search capabilities to help players find the right action quickly.

Additionally, the crate includes tools for voxel and agent inspection summaries, providing detailed information about specific entities within the simulation world.

## Features

- 52 godgame verbs with descriptor schema
- Group classification for organization
- Provenance tracking and risk tiers
- Context-aware ranking and fuzzy search
- Voxel and agent inspector summaries
- Holocron Keycap UI integration
- Queryable verb registry

## Usage

```rust
use holocron::*;
```

## Architecture

- **VerbRegistry**: The central registry for all available verbs.
- **VerbDescriptor**: Detailed information about a specific verb.
- **VerbGroup**: Classification of verbs into logical groups.
- **RiskTier**: Defines the risk level associated with a verb.
- **Provenance**: Tracks the origin and history of a verb.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.