# civlab-sdk

> Modder-facing, engine-agnostic SDK for Civis mods with material, building, and recipe registration.

## Overview

The `civlab-sdk` crate provides the public API for Civis mod developers. It uses a hexagonal architecture to keep the mod API decoupled from any specific engine, renderer, or simulation backend. Mods register materials, buildings, and recipes through trait-based catalogs, and hook into simulation events without importing engine internals.

The SDK discovers and loads mod manifests from a `mods/` folder at startup, validating each manifest against the versioned mod schema before enabling the mod's registrations and hooks. This ensures compatibility and prevents runtime crashes from malformed mods.

Simulation event hooks cover the full lifecycle — births, deaths, technology unlocks, institution changes, and more — allowing mods to react to and influence the simulation through a stable, versioned interface.

## Features

- Hexagonal API design — engine-agnostic, backend-independent
- `MaterialCatalog` for registering new materials with properties
- `BuildingCatalog` for registering custom building types and tiers
- `RecipeCatalog` for registering production and crafting recipes
- Simulation event hooks (births, deaths, tech changes, institution events)
- Manifest loading and validation from `mods/` folder
- Versioned mod compatibility checking

## Usage

```rust
use civlab_sdk::*;
```

## Architecture

- **MaterialCatalog** — Registry for mod-defined materials with physical and economic properties
- **BuildingCatalog** — Registry for mod-defined building types, tiers, and construction costs
- **RecipeCatalog** — Registry for production, crafting, and conversion recipes
- **ModRegistry** — Top-level orchestrator that discovers, validates, and enables loaded mods
- **ModManifest** — Parsed and validated mod descriptor (name, version, dependencies, registrations)

Mods interact exclusively through these catalog and registry types, with event hooks providing read/write access to simulation state without requiring direct engine knowledge.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
