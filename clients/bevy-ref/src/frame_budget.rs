//! Frame-time budget enforcer and quality-recovery helpers for the Bevy client.
//!
//! This module was recovered from history so `god_actions.rs` can keep using the
//! existing cull-distance and quality-mode API without further call-site churn.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

/// Target frame budget in milliseconds (30 FPS floor).
pub const FRAME_BUDGET_MS: f32 = 33.3;
/// Rolling window length for budget averaging.
pub const FRAME_BUDGET_WINDOW: usize = 60;
/// Minimum seconds between throttled budget warnings.
const WARN_THROTTLE_SECS: f64 = 5.0;
/// Rolling window for sustained-drop quality recovery.
pub const DROP_RECOVERY_WINDOW_SECS: f64 = 10.0;
/// Drop delta within the recovery window that triggers reduced quality.
pub const DROP_THRESHOLD_REDUCED: u64 = 5;
/// Drop delta within the recovery window that triggers critical quality.
pub const DROP_THRESHOLD_CRITICAL: u64 = 20;
/// Cull-distance scale in reduced mode.
pub const REDUCED_CULL_SCALE: f32 = 0.9;
/// Additional cull-distance scale in critical mode.
pub const CRITICAL_CULL_SCALE: f32 = 0.9;

/// Runtime GPU quality mode driven by sustained frame drops.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GpuQualityMode {
    /// Full detail.
    #[default]
    Full,
    /// Moderate recovery.
    Reduced,
    /// Strong recovery after sustained drops.
    Critical,
}
impl GpuQualityMode {
    /// Multiplier applied to cull distance.
    #[must_use]
    pub fn cull_distance_scale(self) -> f32 {
        match self {
            Self::Full => 1.0,
            Self::Reduced => REDUCED_CULL_SCALE,
            Self::Critical => REDUCED_CULL_SCALE * CRITICAL_CULL_SCALE,
        }
    }

    /// Multiplier applied to mesh distance before LOD selection.
    #[must_use]
    pub fn lod_distance_scale(self) -> f32 {
        1.0 / self.cull_distance_scale()
    }
}

/// Scale a base cull distance for the active quality mode.
#[must_use]
pub fn scaled_cull_distance(base: f32, mode: GpuQualityMode) -> f32 {
    base * mode.cull_distance_scale()
}

#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct FrameBudgetMetrics {
    pub frame_count: u64,
    pub drop_count: u64,
    pub max_frame_ms: f32,
}

#[derive(Resource)]
struct FrameBudgetState {
    window: [f32; FRAME_BUDGET_WINDOW],
    index: usize,
    filled: usize,
    last_warn_at: Option<f64>,
}

impl Default for FrameBudgetState {
    fn default() -> Self {
        Self { window: [0.0; FRAME_BUDGET_WINDOW], index: 0, filled: 0, last_warn_at: None }
    }
}

#[derive(Resource, Default)]
struct QualityRecoveryState {
    window_start_secs: f64,
    drops_at_window_start: u64,
    last_recovery_warn_at: Option<f64>,
    initialized: bool,
}

/// Registers frame-budget tracking against Bevy diagnostics.
pub struct FrameBudgetPlugin;

impl Plugin for FrameBudgetPlugin {
    fn build(&self, app: &mut App) {
        if !app.is_plugin_added::<FrameTimeDiagnosticsPlugin>() {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default());
        }
        app.init_resource::<FrameBudgetMetrics>()
            .init_resource::<FrameBudgetState>()
            .init_resource::<GpuQualityMode>()
            .init_resource::<QualityRecoveryState>()
            .add_systems(PostUpdate, enforce_frame_budget.after(FrameTimeDiagnosticsPlugin::diagnostic_system));
    }
}

fn enforce_frame_budget(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut metrics: ResMut<FrameBudgetMetrics>,
    mut state: ResMut<FrameBudgetState>,
) {
    let Some(frame_ms) = diagnostics.get(&FrameTimeDiagnosticsPlugin::FRAME_TIME).and_then(|diag| diag.value()).filter(|value| value.is_finite()).map(|value| value as f32) else { return; };
    metrics.frame_count = metrics.frame_count.saturating_add(1);
    metrics.max_frame_ms = metrics.max_frame_ms.max(frame_ms);
    let index = state.index;
    state.window[index] = frame_ms;
    state.index = (index + 1) % FRAME_BUDGET_WINDOW;
    if state.filled < FRAME_BUDGET_WINDOW { state.filled += 1; }
    if state.filled < FRAME_BUDGET_WINDOW { return; }
    let avg_ms = state.window.iter().sum::<f32>() / FRAME_BUDGET_WINDOW as f32;
    if avg_ms <= FRAME_BUDGET_MS { return; }
    metrics.drop_count = metrics.drop_count.saturating_add(1);
    let now = time.elapsed_secs_f64();
    let should_warn = state.last_warn_at.map(|last| now - last >= WARN_THROTTLE_SECS).unwrap_or(true);
    if should_warn {
        warn!("Frame budget exceeded: {avg_ms:.1}ms (target {FRAME_BUDGET_MS})");
        state.last_warn_at = Some(now);
    }
}
