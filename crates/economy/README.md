# economy

> Conservation-complete economy layer with double-entry ledger and allocation engines.

## Overview

The `economy` crate implements a conservation-complete economic simulation layer. It utilizes a double-entry ledger system to ensure that all resources (joules, goods, labor) are accounted for and that conservation invariants are maintained across the system.

It includes allocation engines for distributing resources, district-level production modeling, and market/trade dynamics. The crate also handles institution accounts, tax policies, currency trust, and the management of trade routes and extraction sites.

This crate is designed to be lightweight yet robust, ensuring that the simulation's economic state remains consistent and realistic.

## Features

- Conservation-complete double-entry ledger
- Allocation engines (e.g., Capitalist, Planned)
- District-level production and stocks
- Market and trade route simulation
- Institution accounts and tax policies
- Currency trust and stability modeling
- Joule budget management
- Per-good stock tracking

## Usage

```rust
use economy::*;
```

## Architecture

- **EconomyState**: The central state of the economic system for a simulation tick.
- **AllocationEngine**: Trait and implementations for resource distribution strategies.
- **MultiGoodMarket**: Handles the exchange and pricing of multiple goods.
- **Stocks**: Tracks per-good resource levels.
- **InstitutionLedger**: Manages accounts for specific institutions.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.