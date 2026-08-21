# emergence-oracle

> Programmatic FR verification for Civis emergence systems.

## Overview

The `emergence-oracle` crate provides a suite of 24 domain-specific oracles for verifying Feature Requirements (FRs) against live simulation state. These oracles cover a wide range of emergence systems including religion, language, economy, genetics, diplomacy, and migration.

Each oracle is designed to validate that a specific feature behaves as expected within the simulation. They return `OracleVerdicts` that indicate success, failure, or specific conditions not being met.

The crate uses an `OracleRegistry` to manage and query these oracles, providing a robust framework for automated testing and verification of complex emergence behaviors.

## Features

- 24 domain-specific verification oracles
- Programmatic FR validation
- OracleVerdict system for clear results
- OracleRegistry for management and querying
- Covers religion, language, economy, genetics, diplomacy, migration, and more
- Live simulation state verification

## Usage

```rust
use emergence_oracle::*;
```

## Architecture

- **FeatureOracle**: Trait defining the interface for all oracles.
- **OracleVerdict**: Enum representing the result of a verification check.
- **OracleRegistry**: Container and manager for all active oracles.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.