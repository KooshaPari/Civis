# dag

> Indefinitely extendable DAG plan with parallel agent dispatch and sponsor-format reporter.

## Overview

The `dag` crate provides the infrastructure for defining and executing complex, multi-stage tasks using a Directed Acyclic Graph (DAG). It parses markdown-based plan specifications, organizes them into waves for parallel execution, and provides reporting tools for progress tracking.

It is structured into three main layers: the model layer defines the core data structures (typed plan/nodes/ticks/states); the loader layer converts markdown specs into executable Plan objects; and the wave layer manages the parallel execution fan-out by lane. Additionally, a reporter layer provides progress bars and DAG tree visualizations.

## Features

- Indefinitely extendable DAG-based planning
- Parallel agent dispatch with lane-based fan-out
- Markdown spec to executable Plan loading
- Progress bar and DAG tree reporting
- Typed plan, node, and tick system
- Deterministic execution state management

## Usage

```rust
use dag::*;
```

## Architecture

- **Plan**: The root structure representing a complete DAG plan.
- **Node**: A single task or step within the plan, including its dependencies.
- **WaveExecutor**: Manages the parallel execution of tasks, dispatching agents based on lane availability.
- **AgentRunner**: Executes the logic associated with individual nodes.
- **ReportSnapshot**: Captures the current state of execution for reporting and visualization.

## License

Part of the [Civis](https://github.com/KooshaPari/Civis) project.