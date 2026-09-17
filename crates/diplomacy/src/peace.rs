//! FR-DIPL-002 — Peace treaty system.
//!
//! Peace SHALL be negotiated via signed treaty, producing
//! `diplomacy.peace.signed.v1` events.
//!
//! All state is deterministic integer math. No RNG, no floating-point.

use serde::{Deserialize, Serialize};

use crate::{Pair, PolityId};

// ---------------------------------------------------------------------------
// Peace Treaty
// ---------------------------------------------------------------------------

/// A signed peace treaty between two polities.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PeaceTreaty {
    /// Unique treaty identifier.
    pub id: u64,
    /// The polities that signed the treaty.
    pub parties: (PolityId, PolityId),
    /// Tick at which the treaty was signed.
    pub signed_at_tick: u64,
    /// Duration of the ceasefire in ticks.
    pub ceasefire_duration: u32,
    /// Reparations amount (resource units, can be negative if both owe).
    pub reparations_amount: i32,
    /// Whether territorial concessions are part of the treaty.
    pub territorial_concessions: bool,
}

/// Events emitted by the peace treaty system.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PeaceEvent {
    /// A peace treaty was signed.
    PeaceSigned {
        /// The treaty that was signed.
        treaty: PeaceTreaty,
        /// Simulation tick.
        tick: u64,
    },
    /// A peace treaty expired.
    PeaceExpired {
        /// The pair of parties.
        pair: Pair,
        /// Simulation tick.
        tick: u64,
    },
}

// ---------------------------------------------------------------------------
// Peace Treaty Manager
// ---------------------------------------------------------------------------

/// Manages peace treaties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeaceTreatyManager {
    /// Next available treaty ID.
    next_id: u64,
    /// Active peace treaties.
    treaties: Vec<PeaceTreaty>,
    /// Pending events.
    pending_events: Vec<PeaceEvent>,
}

impl Default for PeaceTreatyManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PeaceTreatyManager {
    /// Create a new empty manager.
    pub fn new() -> Self {
        Self {
            next_id: 1,
            treaties: Vec::new(),
            pending_events: Vec::new(),
        }
    }

    /// Sign a peace treaty. Returns the [`PeaceEvent::PeaceSigned`] event.
    ///
    /// FR-DIPL-002: peace SHALL be negotiated via signed treaty.
    pub fn sign_treaty(
        &mut self,
        party_a: PolityId,
        party_b: PolityId,
        ceasefire_duration: u32,
        reparations_amount: i32,
        territorial_concessions: bool,
        tick: u64,
    ) -> Result<PeaceEvent, PeaceError> {
        if party_a == party_b {
            return Err(PeaceError::SamePolity);
        }

        let id = self.next_id;
        self.next_id += 1;

        let treaty = PeaceTreaty {
            id,
            parties: (party_a, party_b),
            signed_at_tick: tick,
            ceasefire_duration,
            reparations_amount,
            territorial_concessions,
        };

        let event = PeaceEvent::PeaceSigned {
            treaty: treaty.clone(),
            tick,
        };
        self.treaties.push(treaty);
        self.pending_events.push(event.clone());
        Ok(event)
    }

    /// Check for expired treaties and emit events. Returns expired pairs.
    pub fn check_expirations(&mut self, current_tick: u64) -> Vec<Pair> {
        let mut expired = Vec::new();
        for treaty in &mut self.treaties {
            if treaty.ceasefire_duration > 0 {
                let expires_at = treaty
                    .signed_at_tick
                    .saturating_add(treaty.ceasefire_duration as u64);
                if current_tick >= expires_at {
                    let pair = Pair::new(treaty.parties.0, treaty.parties.1);
                    if !expired.contains(&pair) {
                        expired.push(pair);
                        self.pending_events.push(PeaceEvent::PeaceExpired {
                            pair,
                            tick: current_tick,
                        });
                    }
                }
            }
        }
        expired
    }

    /// Check if there is an active peace treaty between two polities.
    pub fn has_peace(&self, a: PolityId, b: PolityId) -> bool {
        let pair = Pair::new(a, b);
        self.treaties.iter().any(|t| {
            Pair::new(t.parties.0, t.parties.1) == pair && t.ceasefire_duration > 0
        })
    }

    /// Get all treaties involving a polity.
    pub fn treaties_for(&self, polity: PolityId) -> Vec<&PeaceTreaty> {
        self.treaties
            .iter()
            .filter(|t| t.parties.0 == polity || t.parties.1 == polity)
            .collect()
    }

    /// Drain pending events.
    pub fn drain_events(&mut self) -> Vec<PeaceEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors during peace treaty operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeaceError {
    /// Cannot sign a treaty with yourself.
    SamePolity,
}

impl std::fmt::Display for PeaceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SamePolity => write!(f, "cannot sign a peace treaty with yourself"),
        }
    }
}

impl std::error::Error for PeaceError {}
