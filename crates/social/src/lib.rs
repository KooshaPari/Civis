//! CIV-0106 — Social simulation layer.
//!
//! Provides ideology tracking, citizen stress accumulation, insurgency
//! lifecycle, and health index computation for the civilisation simulation.
//!
//! All arithmetic is integer-saturating. No floats accumulate across calls.

pub mod events;
pub mod health;
pub mod ideology;
pub mod insurgency;
pub mod stress;

pub use events::{Event, EventType};
pub use health::{compute_health, HealthConfig, HealthInputs, HealthResult, MAX_HEALTH_BP};
pub use ideology::IdeologyScore;
pub use insurgency::{InsurgencyConfig, InsurgencyTracker};
pub use stress::{StressAccumulator, StressConfig};
