# civ-institutions

> Civic institutions (Temple, Garrison) for the Civis simulation — population-gated, faction-aware social constructs.

## Overview

The `civ-institutions` crate models civic institutions that emerge and evolve within settlements as population grows. Temples, Garrisons, and other institution kinds spawn automatically when a settlement crosses specific population thresholds, and upgrade through tiers as the population continues to expand.

Each institution is affiliated with a faction, and high population can trigger faction splitting events where a single institution divides into competing branches. Legitimacy tracking measures community acceptance, influencing whether an institution thrives, is challenged, or collapses.

Events are modeled as one-shot semantics — each institution upgrade, faction split, or legitimacy shift fires exactly once, allowing downstream systems to react without replay concerns.

## Features

- Population-gated institution spawning and tier upgrades
- Multiple institution kinds (Temple, Garrison, and extensible)
- Faction affiliation and splitting logic
- Legitimacy tracking with community acceptance modeling
- One-shot event semantics for clean downstream reactions
- Per-settlement institution management

## Usage

```rust
use civ_institutions::*;
```

## Architecture

- **InstitutionKind** — Enum of supported institution types (Temple, Garrison, etc.)
- **Faction** — Political/social group affiliated with one or more institutions
- **FactionSplitEvent** — One-shot event emitted when a faction divides due to population pressure
- **InstitutionLegitimacy** — Tracks community acceptance and stability of an institution

Institutions query settlement population each tick, emit upgrade and split events as thresholds are crossed, and feed legitimacy data into the broader social simulation.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
