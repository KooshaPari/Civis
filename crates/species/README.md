# species

> Deterministic DNA-to-phenotype expression + multi-species tagging.

## Overview

The `species` crate implements the biological foundation of Civis entities. It maps raw DNA bytes into detailed physical phenotypes (morphology) and behavioral weights. This includes height, color, limb counts, and abstract traits like aggression and curiosity.

It also handles speciation logic, defining how populations differentiate and how environmental pressures influence mortality and adaptation. The system is fully deterministic, ensuring that identical DNA always results in identical entities.

## Features

- DNA-to-Phenotype mapping (Morphology and BehaviorWeights)
- Multi-species tagging and identification
- Niche adaptation logic
- Population pressure mortality calculations
- Deterministic expression algorithms

## Usage

```rust
use species::*;
```

## Architecture

The core logic flows from DNA bytes to `Phenotype` structs, which contain `Morphology` and `BehaviorWeights`. `SpeciesRecord` objects track population-level data, while `Niche` definitions describe environmental roles.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
