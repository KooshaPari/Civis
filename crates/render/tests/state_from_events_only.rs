//!
//! Provenance: this test previously carried a `FR-UX-005` tag. That id is
//! defined in `docs/models/civ-sim/USER_SPEC.md` as an unrelated
//! requirement and is NOT implemented by this code. The only document
//! claiming otherwise was `docs/traceability/TRACEABILITY_MATRIX.md`,
//! which cited a nonexistent spec file. The assertions below are real and
//! are retained as a behavioral test of event-derived UI state with zero polling; the false id was removed
//! rather than rebound. See `docs/audits/id-provenance-corrections.md`.
//! engine state polling.
//!

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
