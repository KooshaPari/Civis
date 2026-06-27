//! Frame-time budget enforcer (P1.5.1) with sustained-drop LOD recovery (P1.5.2).
//!
//! Samples Bevy's `frame_time` diagnostic (same source as [`crate::perf_hud`]) and
//! tracks a rolling 60-frame average against a 30 FPS floor (33.3 ms). When frame
//! drops accumulate within a ~10 s window, [`GpuQualityMode`] signals coarser chunk
//! LOD and shorter cull distance for downstream render systems.

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

/// Target frame budget in milliseconds (30 FPS floor).
pub const FRAME_BUDGET_MS: f32 = 33.3;

/// Rolling window length for budget averaging.
pub const FRAME_BUDGET_WINDOW: usize = 60;

/// Minimum seconds between throttled budget warnings.
const WARN_THROTTLE_SECS: f64 = 5.0;

/// Rolling window for sustained-drop quality recovery (P1.5.2).
pub const DROP_RECOVERY_WINDOW_SECS: f64 = 10.0;

/// Drop delta within [`DROP_RECOVERY_WINDOW_SECS`] that triggers reduced quality.
pub const DROP_THRESHOLD_REDUCED: u64 = 5;

/// Drop delta within [`DROP_RECOVERY_WINDOW_SECS`] that triggers critical quality.
pub const DROP_THRESHOLD_CRITICAL: u64 = 20;

/// Cull-distance scale in [`GpuQualityMode::Reduced`] (~10% closer).
pub const REDUCED_CULL_SCALE: f32 = 0.9;

/// Additional cull-distance scale in [`GpuQualityMode::Critical`].
pub const CRITICAL_CULL_SCALE: f32 = 0.9;

/// Runtime GPU quality mode driven by sustained frame drops (P1.5.2).
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GpuQualityMode {
    /// Full detail — no recovery scaling.
    #[default]
    Full,
    /// Moderate recovery (~10% LOD/cull reduction).
    Reduced,
    /// Strong recovery after sustained drops.
    Critical,
}

impl GpuQualityMode {
    /// Multiplier applied to cull distance (`1.0` = unchanged).
    #[must_use]
    pub fn cull_distance_scale(self) -> f32 {
        match self {
            Self::Full => 1.0,
            Self::Reduced => REDUCED_CULL_SCALE,
            Self::Critical => REDUCED_CULL_SCALE * CRITICAL_CULL_SCALE,
        }
    }

    /// Multiplier applied to mesh distance before LOD selection (`>1` = coarser).
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

/// Inflate world distance before LOD band selection for the active quality mode.
#[must_use]
pub fn scaled_mesh_lod_distance(distance: f32, mode: GpuQualityMode) -> f32 {
    distance * mode.lod_distance_scale()
}

/// Map sustained drop count in the recovery window to a quality mode.
#[must_use]
pub fn evaluate_quality_recovery(drops_in_window: u64) -> GpuQualityMode {
    if drops_in_window > DROP_THRESHOLD_CRITICAL {
        GpuQualityMode::Critical
    } else if drops_in_window > DROP_THRESHOLD_REDUCED {
        GpuQualityMode::Reduced
    } else {
        GpuQualityMode::Full
    }
}

/// Rolling frame-budget metrics exposed to HUD and profiling (P1.5.1).
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq)]
pub struct FrameBudgetMetrics {
    /// Frames observed since startup.
    pub frame_count: u64,
    /// Frames where the rolling average exceeded [`FRAME_BUDGET_MS`].
    pub drop_count: u64,
    /// Worst single-frame time (ms) recorded.
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
        Self {
            window: [0.0; FRAME_BUDGET_WINDOW],
            index: 0,
            filled: 0,
            last_warn_at: None,
        }
    }
}

#[derive(Resource, Default)]
struct QualityRecoveryState {
    window_start_secs: f64,
    drops_at_window_start: u64,
    last_recovery_warn_at: Option<f64>,
    initialized: bool,
}

/// Registers frame-budget tracking against Bevy `frame_time` diagnostics.
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
            .add_systems(
                PostUpdate,
                (
                    enforce_frame_budget,
                    quality_recovery_system,
                )
                    .chain()
                    .after(FrameTimeDiagnosticsPlugin::diagnostic_system),
            );
    }
}

fn enforce_frame_budget(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut metrics: ResMut<FrameBudgetMetrics>,
    mut state: ResMut<FrameBudgetState>,
) {
    let Some(frame_ms) = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FRAME_TIME)
        .and_then(|diag| diag.value())
        .filter(|value| value.is_finite())
        .map(|value| value as f32)
    else {
        return;
    };

    metrics.frame_count = metrics.frame_count.saturating_add(1);
    metrics.max_frame_ms = metrics.max_frame_ms.max(frame_ms);

    let index = state.index;
    state.window[index] = frame_ms;
    state.index = (index + 1) % FRAME_BUDGET_WINDOW;
    if state.filled < FRAME_BUDGET_WINDOW {
        state.filled += 1;
    }

    if state.filled < FRAME_BUDGET_WINDOW {
        return;
    }

    let avg_ms = state.window.iter().sum::<f32>() / FRAME_BUDGET_WINDOW as f32;
    if avg_ms <= FRAME_BUDGET_MS {
        return;
    }

    metrics.drop_count = metrics.drop_count.saturating_add(1);

    let now = time.elapsed_secs_f64();
    let should_warn = state
        .last_warn_at
        .map(|last| now - last >= WARN_THROTTLE_SECS)
        .unwrap_or(true);
    if should_warn {
        warn!(
            "Frame budget exceeded: {avg_ms:.1}ms (target {FRAME_BUDGET_MS})"
        );
        state.last_warn_at = Some(now);
    }
}

fn quality_recovery_system(
    time: Res<Time>,
    metrics: Res<FrameBudgetMetrics>,
    mut mode: ResMut<GpuQualityMode>,
    mut state: ResMut<QualityRecoveryState>,
) {
    let now = time.elapsed_secs_f64();
    if !state.initialized {
        state.window_start_secs = now;
        state.drops_at_window_start = metrics.drop_count;
        state.initialized = true;
    }

    if now - state.window_start_secs >= DROP_RECOVERY_WINDOW_SECS {
        state.window_start_secs = now;
        state.drops_at_window_start = metrics.drop_count;
    }

    let drops_in_window = metrics
        .drop_count
        .saturating_sub(state.drops_at_window_start);
    let new_mode = evaluate_quality_recovery(drops_in_window);
    if new_mode != *mode {
        info!(
            "GPU quality recovery: {:?} -> {:?} ({drops_in_window} drops in {:.0}s window, cull_scale={:.2})",
            *mode,
            new_mode,
            DROP_RECOVERY_WINDOW_SECS,
            new_mode.cull_distance_scale(),
        );
        *mode = new_mode;
    }

    if new_mode == GpuQualityMode::Critical {
        let should_warn = state
            .last_recovery_warn_at
            .map(|last| now - last >= WARN_THROTTLE_SECS)
            .unwrap_or(true);
        if should_warn {
            warn!(
                "Sustained frame drops: {drops_in_window} in {:.0}s window — critical GPU quality reduction active",
                DROP_RECOVERY_WINDOW_SECS,
            );
            state.last_recovery_warn_at = Some(now);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_constants_match_30fps_floor() {
        assert!((FRAME_BUDGET_MS - 33.3).abs() < f32::EPSILON);
        assert_eq!(FRAME_BUDGET_WINDOW, 60);
    }

    #[test]
    fn rolling_average_flags_over_budget_window() {
        let samples = [40.0_f32; FRAME_BUDGET_WINDOW];
        let avg = samples.iter().sum::<f32>() / FRAME_BUDGET_WINDOW as f32;
        assert!(avg > FRAME_BUDGET_MS);
    }

    #[test]
    fn quality_recovery_threshold_transitions() {
        assert_eq!(evaluate_quality_recovery(0), GpuQualityMode::Full);
        assert_eq!(evaluate_quality_recovery(5), GpuQualityMode::Full);
        assert_eq!(evaluate_quality_recovery(6), GpuQualityMode::Reduced);
        assert_eq!(evaluate_quality_recovery(20), GpuQualityMode::Reduced);
        assert_eq!(evaluate_quality_recovery(21), GpuQualityMode::Critical);
    }

    #[test]
    fn reduced_mode_scales_cull_and_lod_by_about_ten_percent() {
        let base = 200.0;
        let reduced = scaled_cull_distance(base, GpuQualityMode::Reduced);
        assert!((reduced - 180.0).abs() < f32::EPSILON);
        let lod_dist = scaled_mesh_lod_distance(32.0, GpuQualityMode::Reduced);
        assert!(lod_dist > 32.0);
    }
}
