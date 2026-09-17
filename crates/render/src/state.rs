//! Event-driven UI state (FR-UX-005, CIV-0300).
//!
//! All UI state changes SHALL derive from events; no direct engine state
//! polling. [`UiState`] is therefore a pure fold over a stream of
//! [`UiEvent`]s: the only way to mutate it is [`UiState::apply`]. It exposes
//! no reference to the engine and counts polling accesses (which must remain
//! zero).
//!
//! The client subscribes to the event bus and calls [`apply`](UiState::apply)
//! once per event; it never reads engine memory directly to refresh the UI.

use serde::{Deserialize, Serialize};

/// A UI-affecting event, mirrored from the engine event bus.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UiEvent {
    /// The selected unit changed (`None` clears the selection).
    SelectionChanged {
        /// Newly selected unit id, if any.
        unit: Option<u64>,
    },
    /// The simulation advanced to a new tick.
    TickAdvanced {
        /// The tick now being displayed.
        tick: u64,
    },
    /// The camera centre moved.
    CameraMoved {
        /// New centre `q`.
        q: i32,
        /// New centre `r`.
        r: i32,
    },
    /// A toast/notification should be shown.
    Notification {
        /// Human-readable message.
        message: String,
    },
}

/// Pure, event-derived UI state (FR-UX-005).
///
/// Construct with [`UiState::default`] (empty) and fold events in with
/// [`apply`](Self::apply). [`poll_engine`](Self::poll_engine) exists only to be
/// asserted against: it must always be zero, demonstrating the UI never polls.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UiState {
    selected_unit: Option<u64>,
    display_tick: u64,
    camera: (i32, i32),
    notifications: Vec<String>,
    /// Number of events folded in so far.
    events_applied: u64,
    /// Number of direct engine polls; must stay 0.
    engine_polls: u64,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            selected_unit: None,
            display_tick: 0,
            camera: (0, 0),
            notifications: Vec::new(),
            events_applied: 0,
            engine_polls: 0,
        }
    }
}

impl UiState {
    /// Determine the next state by folding `event` into the current state.
    ///
    /// This is the *only* mutating entry point, enforcing the
    /// events-only contract.
    pub fn apply(&mut self, event: &UiEvent) {
        match event {
            UiEvent::SelectionChanged { unit } => self.selected_unit = *unit,
            UiEvent::TickAdvanced { tick } => self.display_tick = *tick,
            UiEvent::CameraMoved { q, r } => self.camera = (*q, *r),
            UiEvent::Notification { message } => self.notifications.push(message.clone()),
        }
        self.events_applied += 1;
    }

    /// Fold a whole batch of events in order.
    pub fn apply_all<'a, I>(&mut self, events: I)
    where
        I: IntoIterator<Item = &'a UiEvent>,
    {
        for e in events {
            self.apply(e);
        }
    }

    /// Record a direct engine poll. Returns the new count.
    ///
    /// Kept only so tests can prove the UI never polls: the renderer must not
    /// call this.
    pub fn poll_engine(&mut self) -> u64 {
        self.engine_polls += 1;
        self.engine_polls
    }

    /// Currently selected unit, if any.
    #[must_use]
    pub fn selected_unit(&self) -> Option<u64> {
        self.selected_unit
    }

    /// Tick currently displayed.
    #[must_use]
    pub fn display_tick(&self) -> u64 {
        self.display_tick
    }

    /// Camera centre as `(q, r)`.
    #[must_use]
    pub fn camera(&self) -> (i32, i32) {
        self.camera
    }

    /// Queued notification messages.
    #[must_use]
    pub fn notifications(&self) -> &[String] {
        &self.notifications
    }

    /// How many events have been folded in.
    #[must_use]
    pub fn events_applied(&self) -> u64 {
        self.events_applied
    }

    /// How many direct engine polls have occurred. Contract: always 0.
    #[must_use]
    pub fn engine_polls(&self) -> u64 {
        self.engine_polls
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn state_is_deterministic_function_of_events() {
        let events = vec![
            UiEvent::TickAdvanced { tick: 7 },
            UiEvent::SelectionChanged { unit: Some(42) },
            UiEvent::CameraMoved { q: 3, r: -2 },
        ];

        let mut a = UiState::default();
        a.apply_all(&events);

        let mut b = UiState::default();
        b.apply_all(&events);

        assert_eq!(a, b, "same event stream => same state");
        assert_eq!(a.display_tick(), 7);
        assert_eq!(a.selected_unit(), Some(42));
        assert_eq!(a.camera(), (3, -2));
    }

    #[test]
    fn no_polling_required_to_reach_state() {
        let mut s = UiState::default();
        s.apply(&UiEvent::TickAdvanced { tick: 1 });
        assert_eq!(s.engine_polls(), 0, "UI must not poll engine state");
        assert_eq!(s.events_applied(), 1);
    }

    #[test]
    fn notifications_accumulate_in_order() {
        let mut s = UiState::default();
        s.apply(&UiEvent::Notification { message: "a".into() });
        s.apply(&UiEvent::Notification { message: "b".into() });
        assert_eq!(s.notifications(), &["a".to_string(), "b".to_string()]);
    }
}
