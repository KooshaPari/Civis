//! FR-DIPL-001 — War declaration system.
//!
//! Civilizations SHALL be able to declare war, producing
//! `diplomacy.war.declared.v1` events.
//!
//! All state is deterministic integer math over `BTreeMap`-backed
//! collections. No RNG, no floating-point, no wall-clock.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

use crate::{Pair, PolityId};

// ---------------------------------------------------------------------------
// Events
// ---------------------------------------------------------------------------

/// Events emitted by the war declaration system.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum WarEvent {
    /// A war was declared between two polities.
    WarDeclared {
        /// The aggressor polity.
        aggressor: PolityId,
        /// The defender polity.
        defender: PolityId,
        /// Simulation tick.
        tick: u64,
    },
    /// A war ended (peace signed or war expired).
    WarEnded {
        /// The polity pair.
        pair: Pair,
        /// Whether the war ended via peace treaty or exhaustion.
        via_treaty: bool,
        /// Simulation tick.
        tick: u64,
    },
}

// ---------------------------------------------------------------------------
// War state
// ---------------------------------------------------------------------------

/// Active war between two polities.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct War {
    /// The polity pair involved in the war.
    pub pair: Pair,
    /// Tick the war was declared.
    pub declared_at_tick: u64,
    /// Cumulative casualties for each side (lo, hi).
    pub casualties: (u32, u32),
}

impl War {
    /// Duration of the war in ticks from a given current tick.
    pub fn duration_ticks(&self, current_tick: u64) -> u64 {
        current_tick.saturating_sub(self.declared_at_tick)
    }

    /// Total casualties across both sides.
    pub fn total_casualties(&self) -> u32 {
        self.casualties.0.saturating_add(self.casualties.1)
    }
}

// ---------------------------------------------------------------------------
// WarDeclarationManager
// ---------------------------------------------------------------------------

/// Manages war declarations and active wars.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WarDeclarationManager {
    /// Active wars, keyed by canonical pair.
    active_wars: BTreeMap<Pair, War>,
    /// Pending events.
    pending_events: Vec<WarEvent>,
}

impl Default for WarDeclarationManager {
    fn default() -> Self {
        Self::new()
    }
}

impl WarDeclarationManager {
    /// Create a new empty manager.
    pub fn new() -> Self {
        Self {
            active_wars: BTreeMap::new(),
            pending_events: Vec::new(),
        }
    }

    /// Declare war between two polities. Returns the [`WarEvent::WarDeclared`]
    /// event on success, or an error if a war is already active.
    ///
    /// This is the core of FR-DIPL-001: producing a `diplomacy.war.declared.v1`
    /// event.
    pub fn declare_war(
        &mut self,
        aggressor: PolityId,
        defender: PolityId,
        tick: u64,
    ) -> Result<WarEvent, WarError> {
        if aggressor == defender {
            return Err(WarError::CannotDeclareWarOnSelf);
        }
        let pair = Pair::new(aggressor, defender);
        if self.active_wars.contains_key(&pair) {
            return Err(WarError::WarAlreadyActive(pair));
        }

        let war = War {
            pair,
            declared_at_tick: tick,
            casualties: (0, 0),
        };
        self.active_wars.insert(pair, war);

        let event = WarEvent::WarDeclared {
            aggressor,
            defender,
            tick,
        };
        self.pending_events.push(event.clone());
        Ok(event)
    }

    /// End a war. Returns the [`WarEvent::WarEnded`] event.
    pub fn end_war(
        &mut self,
        pair: Pair,
        via_treaty: bool,
        tick: u64,
    ) -> Result<WarEvent, WarError> {
        if self.active_wars.remove(&pair).is_none() {
            return Err(WarError::NoActiveWar(pair));
        }
        let event = WarEvent::WarEnded {
            pair,
            via_treaty,
            tick,
        };
        self.pending_events.push(event.clone());
        Ok(event)
    }

    /// Record casualties for an active war side.
    pub fn record_casualties(&mut self, pair: &Pair, casualties: u32) -> Result<(), WarError> {
        let war = self.active_wars.get_mut(pair).ok_or(WarError::NoActiveWar(*pair))?;
        // lo side casualties
        if pair.lo != pair.hi {
            war.casualties.0 = war.casualties.0.saturating_add(casualties);
        }
        Ok(())
    }

    /// Check if a war is active between two polities.
    pub fn is_at_war(&self, a: PolityId, b: PolityId) -> bool {
        self.active_wars.contains_key(&Pair::new(a, b))
    }

    /// Get the war between two polities.
    pub fn get_war(&self, a: PolityId, b: PolityId) -> Option<&War> {
        self.active_wars.get(&Pair::new(a, b))
    }

    /// Number of active wars.
    pub fn active_war_count(&self) -> usize {
        self.active_wars.len()
    }

    /// Drain pending events.
    pub fn drain_events(&mut self) -> Vec<WarEvent> {
        std::mem::take(&mut self.pending_events)
    }
}

// ---------------------------------------------------------------------------
// Errors
// ---------------------------------------------------------------------------

/// Errors during war declaration operations.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WarError {
    /// Cannot declare war on yourself.
    CannotDeclareWarOnSelf,
    /// A war is already active between these polities.
    WarAlreadyActive(Pair),
    /// No active war found between these polities.
    NoActiveWar(Pair),
}

impl std::fmt::Display for WarError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CannotDeclareWarOnSelf => write!(f, "cannot declare war on yourself"),
            Self::WarAlreadyActive(p) => write!(f, "war already active between {p:?}"),
            Self::NoActiveWar(p) => write!(f, "no active war between {p:?}"),
        }
    }
}

impl std::error::Error for WarError {}
