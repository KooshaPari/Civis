# emergence-migration

> Emergent population migration substrate with push/pull mechanics and cultural mixing.

## Overview

The `emergence-migration` crate models the movement of populations within the Civis simulation. It uses a push/pull engine driven by cluster stress (push) and opportunity (pull) factors to determine migration patterns.

The crate handles settlement reshaping, cultural mixing (counter-divergence), and refugee surges triggered by disasters or wars. It ensures that migration is a dynamic and emergent behavior that responds to the changing state of the world.

All migration logic is deterministic, utilizing a seeded `ChaCha8Rng` to ensure reproducible results.

## Features

- Push/pull migration engine
- Cluster stress and opportunity factors
- Settlement reshaping
- Cultural mixing and counter-divergence
- Refugee surges from disasters and wars
- Deterministic logic with seeded RNG
- Emergent population movement

## Usage

```rust
use emergence_migration::*;
```

## Architecture

- **MigrationEngine**: The core engine managing all migration processes.
- **ClusterMigration**: Represents migration between specific clusters.
- **MigrationStress**: Defines push factors (e.g., overcrowding, resource depletion).
- **MigrationOpportunity**: Defines pull factors (e.g., new jobs, safety).
- **SurgeEvent**: Represents sudden, large-scale migration events.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.