# civ-dag

Master-DAG substrate for the Civis godgame. Parses `CIVIS_GAME_DAG.md`, plans execution waves, dispatches parallel task agents, and renders sponsor-format progress reports.

## Architecture

The crate is organised into four layers:

| Layer | Module | Purpose |
|---|---|---|
| Model | `model` | Typed plan, pillars, lanes, nodes, ticks, and states |
| Loader | `loader` | Markdown spec → `model::Plan` parser |
| Wave Executor | `wave` | Parallel executor — fan-out by lane, topological ordering by wave |
| Reporter | `reporter` | Sponsor-format output (progress bars, DAG tree, agent rows) |

## Key Types

```rust
use civ_dag::{Plan, Node, NodeState, Lane, Layer};
use civ_dag::wave::{WaveExecutor, RetryPolicy, WaveContext};
use civ_dag::reporter::{Reporter, ReportSnapshot, DagEvent};
```

## Usage

```rust
use civ_dag::{loader, wave::WaveExecutor};
use std::path::PathBuf;
use std::sync::Arc;

// Load a plan from markdown
let src = std::fs::read_to_string("CIVIS_GAME_DAG.md").unwrap();
let plan = Arc::new(
    loader::load_from_markdown(&src, PathBuf::from("CIVIS_GAME_DAG.md")).unwrap()
);

// Execute waves in parallel
let exec = WaveExecutor::new(plan);
```

## Features

- **`default`** — Pure library
- **`cli`** — Activates the `civ-dag` binary entry point (`dagr`)

```bash
# Run the CLI
cargo run -p civ-dag --features cli -- <path-to-dag.md>
```

## Dependencies

| Crate | Role |
|---|---|
| `tokio` | Async runtime for parallel wave execution |
| `serde` / `serde_json` | Plan serialization |
| `chrono` | Timestamps and scheduling |
| `async-trait` | Agent runner trait |
| `anyhow` / `thiserror` | Error handling |
