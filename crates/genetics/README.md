# genetics

> Algorithmic DNA, mutation, recombination, fitness evaluation, and speciation.

## Overview

The `genetics` crate implements a pure algorithmic system for DNA, mutation, recombination, fitness evaluation, and speciation. It is designed to operate without any LLM involvement, ensuring high performance and determinism.

All randomness is managed via a caller-provided `ChaCha8Rng`, allowing for bit-identical replay of genetic evolution. The crate includes a seed library, disease resistance evolution, sentience traits, and complex trait inheritance mechanics.

This allows for the emergent evolution of populations within the Civis simulation, with traits passing down through generations and species diverging based on fitness and environmental pressures.

## Features

- Algorithmic DNA and trait modeling
- Mutation and recombination mechanics
- Fitness evaluation and selection
- Speciation thresholds and divergence
- Seeded RNG for deterministic replay
- Disease resistance evolution
- Sentience traits and inheritance
- Seed library for diverse starting points

## Usage

```rust
use genetics::*;
```

## Architecture

- **Dna**: The core data structure representing genetic information.
- **DnaClass**: Classification of DNA types or groups.
- **Species**: Represents a group of individuals with shared genetic traits.
- **SeedLibrary**: A collection of starting genetic profiles.
- **TraitVector**: Vector of traits associated with an individual or species.
- **DiseaseResistance**: Specific traits related to resisting diseases.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.