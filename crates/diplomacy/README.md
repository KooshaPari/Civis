# diplomacy

> Diplomacy substrate with pairwise relations, concession state machines, and Zeuthen/Rubinstein bargaining.

## Overview

The `diplomacy` crate provides a sophisticated substrate for modeling international and interpersonal relations. It features a pairwise `Relation` graph, scalar standing, and threshold-crossing events that trigger changes in stance and alliance.

It includes a `ConcessionStateMachine` for handling negotiations, treaty transcripts for logging, and support for emergent diplomacy behaviors. The crate also implements Zeuthen and Rubinstein bargaining models to simulate realistic negotiation outcomes.

All data structures are deterministic and BTreeMap-based, using integer math exclusively to ensure consistency across different platforms and simulation runs.

## Features

- Pairwise Relation graph with scalar standing
- Threshold-crossing events and stances
- Concession state machine for negotiations
- Treaty transcripts and logging
- Alliance formation mechanics
- Zeuthen and Rubinstein bargaining models
- Emergent diplomacy behaviors
- Deterministic BTreeMap-based storage
- Integer-only math

## Usage

```rust
use diplomacy::*;
```

## Architecture

- **DiplomacyState**: Central state container for all diplomatic data.
- **Relation**: Represents the state between two entities.
- **ConcessionStateMachine**: Manages the flow of concessions during negotiations.
- **TreatyTranscript**: Records the details of treaties and agreements.
- **AllianceFormation**: Logic for forming and managing alliances.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.