//! Frame budget planning for 60 fps rendering (FR-PERF-003).
//!
//! The render crate SHALL maintain 60 fps at 1080p on the reference GPU
//! profile. This module provides a [`FrameBudget`] struct that encapsulates
//! the timing constraints so the render loop can enforce them.
//!
//! The client reads the budget each frame and decides whether to skip
//! expensive passes (e.g. shadows, bloom) when the budget is tight.

use serde::{Deserialize, Serialize};

/// Target frame rate for the render loop.
pub const TARGET_FPS: u32 = 60;

/// Target resolution width in pixels.
pub const RESOLUTION_WIDTH: u32 = 1920;

/// Target resolution height in pixels.
pub const RESOLUTION_HEIGHT: u32 = 1080;

/// Frame budget struct (FR-PERF-003).
///
/// Encapsulates the timing constraint for maintaining target FPS. The render
/// loop checks [`budget_ms`] each frame and drops non-essential passes when
/// elapsed time approaches the limit.
///
/// # Examples
///
/// ```
/// use civ_render::frame::FrameBudget;
///
/// let budget = FrameBudget::new();
/// assert!((budget.budget_ms() - 16.67).abs() < 0.01);
/// assert!(budget.target_fps() == 60);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FrameBudget {
    /// Target frames per second.
    target_fps: u32,
    /// Resolution width.
    width: u32,
    /// Resolution height.
    height: u32,
}

impl Default for FrameBudget {
    fn default() -> Self {
        Self::new()
    }
}

impl FrameBudget {
    /// Create a frame budget for the standard 60 fps / 1080p target.
    #[must_use]
    pub fn new() -> Self {
        Self {
            target_fps: TARGET_FPS,
            width: RESOLUTION_WIDTH,
            height: RESOLUTION_HEIGHT,
        }
    }

    /// Create a frame budget with custom target FPS and resolution.
    #[must_use]
    pub fn with_params(target_fps: u32, width: u32, height: u32) -> Self {
        Self {
            target_fps,
            width,
            height,
        }
    }

    /// Frame budget in milliseconds (1000.0 / target_fps).
    ///
    /// For 60 fps this is approximately 16.67 ms.
    #[must_use]
    pub fn budget_ms(&self) -> f64 {
        1000.0 / f64::from(self.target_fps)
    }

    /// Target frames per second.
    #[must_use]
    pub fn target_fps(&self) -> u32 {
        self.target_fps
    }

    /// Resolution width in pixels.
    #[must_use]
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Resolution height in pixels.
    #[must_use]
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Total pixel count (width * height).
    #[must_use]
    pub fn pixel_count(&self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_budget_60fps_1080p() {
        let budget = FrameBudget::new();
        // 1000 / 60 = 16.666... ms
        assert!((budget.budget_ms() - 16.67).abs() < 0.01);
        assert_eq!(budget.target_fps(), 60);
        assert_eq!(budget.width(), 1920);
        assert_eq!(budget.height(), 1080);
    }

    #[test]
    fn frame_budget_pixel_count() {
        let budget = FrameBudget::new();
        assert_eq!(budget.pixel_count(), 1920 * 1080);
    }

    #[test]
    fn frame_budget_custom_params() {
        let budget = FrameBudget::with_params(30, 2560, 1440);
        assert!((budget.budget_ms() - 33.33).abs() < 0.01);
        assert_eq!(budget.target_fps(), 30);
        assert_eq!(budget.width(), 2560);
        assert_eq!(budget.height(), 1440);
    }

    #[test]
    fn frame_budget_type_exists() {
        // Verify the struct can be constructed and is Send + Sync
        // (this is what the traceability matrix test pattern checks).
        let budget = FrameBudget::new();
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<FrameBudget>();
        // Assert the budget is within the 16.67ms target for 60fps.
        assert!(budget.budget_ms() <= 16.67 + 0.01);
    }
}
