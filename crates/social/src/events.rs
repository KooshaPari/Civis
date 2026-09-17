//! Social events (FR-SOCI-004, FR-SOCI-006).
//!
//! All social events share a common `Event` type with a tick stamp.
//! Events are emitted by subsystems (insurgency, health) and consumed
//! by the engine event bus.

use serde::{Deserialize, Serialize};

/// Types of events produced by the social subsystem.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EventType {
    /// Aggregate stress crossed the insurgency start threshold.
    InsurgencyStarted,
    /// Aggregate stress dropped below the insurgency end threshold.
    InsurgencyEnded,
    /// Health index fell below the crisis threshold.
    HealthCrisis,
}

/// A social event with a tick stamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Event {
    /// Which kind of event occurred.
    pub event_type: EventType,
    /// Simulation tick when the event fired.
    pub tick: u64,
}
