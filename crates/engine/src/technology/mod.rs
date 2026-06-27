//! Emergent technology subsystem (FR-CIV-TECH).
//!
//! Provides:
//! - [`research`] — emergent research-rate calculation.
//! - [`tech_tree`] — tech DAG with prerequisite enforcement.
//! - [`diffusion`] — tech diffusion across trade-route networks.
//! - [`specialization`] — cluster specialization from environment.

pub mod diffusion;
pub mod research;
pub mod specialization;
pub mod tech_tree;

pub use diffusion::diffuse_tech;
pub use research::calculate_research_rate;
pub use specialization::{compute_specialization, ClusterEnvironment, TechSpecialization};
pub use tech_tree::{TechId, TechNode, TechTree};
