# civ-emergence-metrics

> Weak-emergence and criticality metrics for the Emergence Dashboard — pure math, no I/O.

## Overview

The `civ-emergence-metrics` crate provides the statistical and information-theoretic primitives that power the Emergence Dashboard in Civis. It measures whether macro-level patterns (cities, trade routes, cultural boundaries) are genuinely emergent from micro-level agent behavior, or merely imposed by design.

All types implement the `Metric` trait and operate as pure functions over simulation state snapshots. There is no I/O, no async, and no engine dependency — making the metrics trivially testable, composable, and reproducible.

The crate includes Shannon entropy for categorical state analysis, connected component counting for structural detection, branching ratios, criticality indicators, mutual information between scales, and power-law fitting for distribution analysis.

## Features

- Shannon entropy computation for categorical state distributions
- Connected component counting via 6-connectivity flood fill
- Branching ratio metrics for process tree analysis
- Criticality indicators (near phase-transition detection)
- Mutual information between macro and micro scales
- Power-law distribution fitting and goodness-of-state analysis
- `Metric` trait for composable, type-safe metric pipelines

## Usage

```rust
use civ_emergence_metrics::*;
```

## Architecture

- **ShannonEntropy** — Computes information entropy over categorical state maps
- **StructureCount** — Counts 6-connectivity connected components in voxel grids
- **CriticalityInputs** — Bundles inputs for multi-variate criticality analysis
- **EmergenceDashboard** — Top-level aggregator composing all metrics into a unified dashboard view

All metrics are implemented as structs satisfying the `Metric` trait, with `compute` methods that take simulation snapshots and return scalar or structured results.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.
