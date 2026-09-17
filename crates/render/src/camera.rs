//! RTS camera controls (FR-UX-002, CIV-0300).
//!
//! The UI SHALL support real-time-strategy style camera pan, zoom, and unit
//! selection. This module models the camera as pure state fed by input events,
//! so behaviour is deterministic and testable without a window.
//!
//! Inputs arrive as [`CameraInput`] values (emitted by the client from raw
//! winit/bevy events); [`RtsCamera::apply`] folds them into the camera state.

use serde::{Deserialize, Serialize};

use crate::hex_map::HexCoord;

/// Zoom limits, in world-units-per-pixel.
pub const MIN_ZOOM: f32 = 0.25;
/// Maximum zoom-in value.
pub const MAX_ZOOM: f32 = 8.0;

/// A single camera input event.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum CameraInput {
    /// Pan by a screen-space delta in pixels.
    Pan {
        /// Horizontal delta in pixels (positive is right).
        dx: f32,
        /// Vertical delta in pixels (positive is down).
        dy: f32,
    },
    /// Zoom by a multiplicative step (e.g. wheel notch).
    Zoom {
        /// Multiplier applied to the current zoom (>1 zooms in).
        factor: f32,
    },
    /// Select every unit inside the given screen-space rectangle.
    SelectRect {
        /// Left edge in pixels.
        x0: f32,
        /// Top edge in pixels.
        y0: f32,
        /// Right edge in pixels.
        x1: f32,
        /// Bottom edge in pixels.
        y1: f32,
    },
    /// Clear the current selection.
    ClearSelection,
}

/// An identifiable selectable unit, positioned in screen space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SelectableUnit {
    /// Stable unit id from the engine.
    pub id: u64,
    /// Screen-space x position in pixels.
    pub x: f32,
    /// Screen-space y position in pixels.
    pub y: f32,
}

/// RTS camera state (FR-UX-002).
///
/// Tracks the world-space centre, zoom level, and current unit selection.
/// All mutation flows through [`apply`](Self::apply) so the same input stream
/// always yields the same camera state.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RtsCamera {
    centre: HexCoord,
    zoom: f32,
    selection: Vec<u64>,
}

impl Default for RtsCamera {
    fn default() -> Self {
        Self::new()
    }
}

impl RtsCamera {
    /// Create a camera centred at the origin with default zoom.
    #[must_use]
    pub fn new() -> Self {
        Self {
            centre: HexCoord::new(0, 0),
            zoom: 1.0,
            selection: Vec::new(),
        }
    }

    /// Current world-space centre.
    #[must_use]
    pub fn centre(&self) -> HexCoord {
        self.centre
    }

    /// Current zoom factor.
    #[must_use]
    pub fn zoom(&self) -> f32 {
        self.zoom
    }

    /// Currently selected unit ids, in selection order.
    #[must_use]
    pub fn selection(&self) -> &[u64] {
        &self.selection
    }

    /// Fold one input event into the camera state.
    ///
    /// Pan deltas are scaled by the inverse of zoom so that panning feels
    /// constant on screen at any zoom level. Zoom is clamped to
    /// `[MIN_ZOOM, MAX_ZOOM]`.
    pub fn apply(&mut self, input: CameraInput, units: &[SelectableUnit]) {
        match input {
            CameraInput::Pan { dx, dy } => {
                let scale = 1.0 / self.zoom;
                self.centre = HexCoord::new(
                    self.centre.q + (dx * scale).round() as i32,
                    self.centre.r + (dy * scale).round() as i32,
                );
            }
            CameraInput::Zoom { factor } => {
                let next = self.zoom * factor;
                self.zoom = next.clamp(MIN_ZOOM, MAX_ZOOM);
            }
            CameraInput::SelectRect { x0, y0, x1, y1 } => {
                let (lo_x, hi_x) = (x0.min(x1), x0.max(x1));
                let (lo_y, hi_y) = (y0.min(y1), y0.max(y1));
                self.selection = units
                    .iter()
                    .filter(|u| u.x >= lo_x && u.x <= hi_x && u.y >= lo_y && u.y <= hi_y)
                    .map(|u| u.id)
                    .collect();
            }
            CameraInput::ClearSelection => self.selection.clear(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pan_scales_with_zoom() {
        let mut cam = RtsCamera::new();
        cam.apply(CameraInput::Zoom { factor: 2.0 }, &[]);
        cam.apply(CameraInput::Pan { dx: 10.0, dy: 0.0 }, &[]);
        // at zoom 2, 10px pan advances 5 hex steps
        assert_eq!(cam.centre().q, 5);
    }

    #[test]
    fn zoom_is_clamped() {
        let mut cam = RtsCamera::new();
        for _ in 0..20 {
            cam.apply(CameraInput::Zoom { factor: 2.0 }, &[]);
        }
        assert!((cam.zoom() - MAX_ZOOM).abs() < f32::EPSILON);
        for _ in 0..40 {
            cam.apply(CameraInput::Zoom { factor: 0.5 }, &[]);
        }
        assert!((cam.zoom() - MIN_ZOOM).abs() < f32::EPSILON);
    }

    #[test]
    fn rect_selection_picks_units_inside() {
        let mut cam = RtsCamera::new();
        let units = [
            SelectableUnit { id: 1, x: 10.0, y: 10.0 },
            SelectableUnit { id: 2, x: 500.0, y: 500.0 },
            SelectableUnit { id: 3, x: 30.0, y: 20.0 },
        ];
        cam.apply(
            CameraInput::SelectRect { x0: 0.0, y0: 0.0, x1: 100.0, y1: 100.0 },
            &units,
        );
        assert_eq!(cam.selection(), &[1, 3]);
        cam.apply(CameraInput::ClearSelection, &units);
        assert!(cam.selection().is_empty());
    }
}
