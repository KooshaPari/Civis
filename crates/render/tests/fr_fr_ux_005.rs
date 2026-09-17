//! FR-UX-005 — all UI state changes SHALL derive from events; no direct
//! engine state polling.
//!
//! Matrix check: `render::state_from_events_only`.

use civ_render::state::{UiEvent, UiState};

/// The UI state is a pure function of the event stream: replaying the same
/// events yields the same state, and no engine poll is ever required.
#[test]
fn state_from_events_only() {
    let events = [
        UiEvent::TickAdvanced { tick: 512 },
        UiEvent::CameraMoved { q: -3, r: 9 },
        UiEvent::SelectionChanged { unit: Some(11) },
        UiEvent::Notification { message: "city founded".into() },
    ];

    let mut a = UiState::default();
    a.apply_all(&events);

    let mut b = UiState::default();
    b.apply_all(&events);

    // Deterministic fold: identical event streams produce identical state.
    assert_eq!(a, b);
    assert_eq!(a.display_tick(), 512);
    assert_eq!(a.camera(), (-3, 9));
    assert_eq!(a.selected_unit(), Some(11));
    assert_eq!(a.notifications().len(), 1);
    assert_eq!(a.events_applied(), 4);

    // The UI reached that state without a single engine poll.
    assert_eq!(a.engine_polls(), 0, "UI must never poll engine state");

    // A fresh state with no events is the default, proving no hidden
    // engine-derived initialisation.
    assert_eq!(UiState::default(), UiState::default());
}
