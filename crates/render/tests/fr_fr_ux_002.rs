//! FR-UX-002 — the UI SHALL support RTS-style camera pan, zoom, and unit
//! selection.
//!
//! Matrix check: `render::rts_camera_controls`.

use civ_render::camera::{CameraInput, RtsCamera, SelectableUnit, MAX_ZOOM, MIN_ZOOM};

/// Pan, zoom and rect selection all behave as an RTS camera requires.
#[test]
fn rts_camera_controls() {
    let units = [
        SelectableUnit { id: 7, x: 40.0, y: 40.0 },
        SelectableUnit { id: 8, x: 900.0, y: 900.0 },
    ];
    let mut cam = RtsCamera::new();
    assert_eq!(cam.centre().q, 0);

    // Pan.
    cam.apply(CameraInput::Pan { dx: 12.0, dy: -4.0 }, &units);
    assert_eq!(cam.centre().q, 12);
    assert_eq!(cam.centre().r, -4);

    // Zoom in and out, clamped at both ends.
    cam.apply(CameraInput::Zoom { factor: 2.0 }, &units);
    assert!((cam.zoom() - 2.0).abs() < f32::EPSILON);
    for _ in 0..40 {
        cam.apply(CameraInput::Zoom { factor: 2.0 }, &units);
    }
    assert!((cam.zoom() - MAX_ZOOM).abs() < f32::EPSILON);
    for _ in 0..40 {
        cam.apply(CameraInput::Zoom { factor: 0.5 }, &units);
    }
    assert!((cam.zoom() - MIN_ZOOM).abs() < f32::EPSILON);

    // Unit selection by screen-space rectangle.
    cam.apply(
        CameraInput::SelectRect { x0: 0.0, y0: 0.0, x1: 100.0, y1: 100.0 },
        &units,
    );
    assert_eq!(cam.selection(), &[7]);
    cam.apply(CameraInput::ClearSelection, &units);
    assert!(cam.selection().is_empty());
}
