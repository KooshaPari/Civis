//! `civ-dag` — indefinitely extendable DAG plan + parallel agent dispatch + sponsor-format reporter.
//!
//! The crate is organised as three layers:
//!
//! 1. [`model`] — typed plan, pillars, lanes, nodes, ticks, states
//! 2. [`loader`] — markdown spec → [`model::Plan`]
//! 3. [`wave`] — parallel executor (fan-out by lane, topologically by wave)
//! 4. [`reporter`] — sponsor format (progress bars, DAG tree, agent rows)
//!
//! # Example
//!
//! ```no_run
//! use civ_dag::{loader, wave::WaveExecutor};
//! use std::path::PathBuf;
//! use std::sync::Arc;
//! let src = std::fs::read_to_string("CIVIS_GAME_DAG.md").unwrap();
//! let plan = Arc::new(loader::load_from_markdown(&src, PathBuf::from("CIVIS_GAME_DAG.md")).unwrap());
//! let exec = WaveExecutor::new(plan);
//! ```

pub mod loader;
pub mod model;
pub mod reporter;
pub mod wave;

pub use model::{Layer, Lane, Node, NodeState, Plan, PlanMeta, PlanStatus};
pub use reporter::{DagEvent, NoopReporter, ReportSnapshot, Reporter};
pub use wave::{AgentRunner, NodeReport, RetryPolicy, WaveContext, WaveExecutor};
