# legends

> Emergent-history saga-graph engine: a petgraph StableDiGraph of significant entities and events with causal edges.

## Overview

The civ-legends crate builds and maintains a directed acyclic graph of historically significant entities, events, and their causal relationships over the simulation event stream. The graph is queried by the inspector UI and the AI narrator to produce emergent history summaries and saga fragments.

## Features

- Petgraph StableDiGraph of entity and event nodes with causal edges
- Deterministic hashing (Blake3) for node identity and deduplication
- Serde serialization for save/load and network transfer
- Read-only query API for the inspector and narrator
- Tracery-based text generation for saga narration
- Configurable significance thresholds for node creation

## Usage

```rust
use civ_legends::{SagaGraph, Significance};

let mut graph = SagaGraph::new();
graph.record_event(Significance::Major, "The Great Flood".into());
```

## Architecture

Events are captured from the simulation stream and evaluated against significance thresholds. Nodes are inserted into a StableDiGraph with typed edges representing causation, participation, or influence. The query API exposes traversal methods (ancestors, descendants, paths) used by the inspector and narrator subsystems.

## License

Part of the Civis project.
