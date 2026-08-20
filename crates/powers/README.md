# powers

> Data-driven god-tool registry for the Holocron Deck.

## Overview

The `powers` crate serves as the central data repository for the "Holocron Deck" system. It contains a catalog of 50 distinct god-tool verbs, defining their schemas, classifications, cooldown mechanics, and synergy multipliers. It establishes the contract for substrate write surfaces without implementing business logic.

This separation allows for easy extension of the power system through data definition rather than code changes. The registry provides a structured way to manage and query available powers, their costs, and their interactions.

## Features

- Catalog of 50 god-tool verbs with detailed schemas
- Tab and category classification for UI organization
- Cooldown tracking and management
- Synergy multiplier definitions for power interactions
- Substrate write surface contract definitions

## Usage

```rust
use powers::*;
```

## Architecture

The core of the crate is the `PowerRegistry`, which holds a collection of `PowerDef` structures. Each `PowerDef` contains metadata about a power, including its schema and cooldown parameters (`PowerCooldown`). Synergies between powers are represented by `SynergyEdge` structures.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
