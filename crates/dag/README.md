# civ-dag

**Indefinitely extendable DAG plan + parallel agent dispatch + sponsor-format reporter.**

The `civ-dag` crate provides the infrastructure for defining and executing complex, multi-stage tasks using a Directed Acyclic Graph (DAG). It parses markdown-based plan specifications, organizes them into waves for parallel execution, and provides reporting tools for progress tracking.

## Key Types

- `Plan`: The root structure representing a complete DAG plan.
- `Node`: A single task or step within the plan.
- `WaveExecutor`: Executes nodes in parallel, respecting topological order and lane constraints.
- `Layer` / `Lane`: Organizational structures for grouping and parallelizing tasks.

## Usage Example

```rust
use civ_dag::{loader, wave::WaveExecutor};
use std::path::PathBuf;
use std::sync::Arc;

// Load a plan from a markdown file
let src = std::fs::read_to_string("CIVIS_GAME_DAG.md").unwrap();
let plan = Arc::new(loader::load_from_markdown(&src, PathBuf::from("CIVIS_GAME_DAG.md")).unwrap());

// Create an executor and run the plan
let exec = WaveExecutor::new(plan);
// exec.run_all().await;
```

## Dependencies

- `tokio`: Async runtime for parallel execution.
- `anyhow` / `thiserror`: Error handling.
- `chrono`: Timestamps for task tracking.
