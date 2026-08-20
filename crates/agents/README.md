# agents

> Civilian agent ECS components with LOD tick scheduling and per-civilian wardrobe/tools state.

## Overview

The `agents` crate provides the entity-component-system foundation for individual civilians in the Civis simulation. Each civilian is represented as a hecs entity carrying components for identity, needs, temperament, social connections, and physical presence in the world.

The crate includes level-of-detail (LOD) tick management so distant civilians update at reduced frequency, keeping simulation cost manageable across large populations. Cluster management groups civilians by geography and culture, enabling efficient batch processing for social and economic interactions.

Culture and language drift are modeled through lexicon propagation, while social graphs capture relationships between civilians. The daily pathfinding loop, diplomacy matrices, and psyche/temperament systems combine to produce believable emergent behavior at the individual level.

## Features

- Civilian entity composition with `hecs::World` integration
- LOD-tiered tick scheduling for performance scalability
- Wardrobe and tools state tracking per civilian
- Social graph and diplomacy matrix
- Culture/language drift via lexicon propagation
- Daily pathfinding and psyche/temperament modeling
- Cluster management for geographic grouping

## Usage

```rust
use agents::*;
```

## Architecture

- **Civilian** — Core component holding identity, needs, and temperament for each agent
- **ClusterId** — Geographic/cultural grouping key for batch processing
- **SocialGraph** — Relationship network between civilians (kinship, trade, rivalry)
- **Psyche** — Temperament, mood, and decision-making weights
- **Lexicon** — Language and cultural knowledge state subject to drift
- **PoiRegistry** — Points-of-interest lookup for pathfinding and daily routines

These types compose into `hecs::World` entities where the simulation loop queries, mutates, and despawns agents each tick.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
