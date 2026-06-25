//! Frame-time budget enforcer (P1.5.1).
//!
//! Samples Bevy's `frame_time` diagnostic (same source as [`crate::perf_hud`]) and
//! tracks a rolling 60-frame average against a 30 FPS floor (33.3 ms).

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use std::collections::VecDeque;

/// Target frame budget in milliseconds (30 FPS floor).
pub const FRAME_BUDGET_MS: f32 = 33.3;

/// Rolling window length for budget averaging.
pub const FRAME_BUDGET_WINDOW: usize = 60;

/// Time window used for recovery recovery decisions.
pub const FRAME_BUDGET_RECOVERY_WINDOW_SECS: f64 = 10.0;

/// Minimum seconds between throttled budget warnings.
const WARN_THROTTLE_SECS: f64 = 5.0;

/// Mild recovery threshold: enter reduced-quality mode when exceeded.
const FRAME_BUDGET_RECOVERY_SOFT_THRESHOLD: usize = 5;

/// Severe recovery threshold: stronger reduction and warn when exceeded.
const FRAME_BUDGET_RECOVERY_SEVERE_THRESHOLD: usize = 20;

/// Mild quality damping (distance multiplier for draw distance).
pub const FRAME_BUDGET_RECOVERY_SOFT_DISTANCE_SCALE: f32 = 0.90;

/// Severe quality damping (distance multiplier for draw distance).
pub const FRAME_BUDGET_RECOVERY_SEVERE_DISTANCE_SCALE: f32 = 0.80;

/// Mild quality damping for LOD decision distance.
pub const FRAME_BUDGET_RECOVERY_SOFT_LOD_SCALE: f32 = 1.10;

/// Severe quality damping for LOD decision distance.
pub const FRAME_BUDGET_RECOVERY_SEVERE_LOD_SCALE: f32 = 1.25;

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

/// Current frame-budget recovery state used by LOD and draw-distance consumers.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct FrameBudgetRecovery {
    /// Whether recovery is active.
    pub active: bool,
    /// Draw-distance multiplier; < 1.0 reduces cull distance.
    pub distance_scale: f32,
    /// LOD distance multiplier; > 1.0 increases selected LOD level.
    pub lod_scale: f32,
    /// Whether strong recovery tier is active.
    pub severe: bool,
}

impl Default for FrameBudgetRecovery {
    fn default() -> Self {
        Self {
            active: false,
            distance_scale: 1.0,
            lod_scale: 1.0,
            severe: false,
        }
    }
}

impl FrameBudgetRecovery {
    /// Draw-distance scale for culling in live voxel systems.
    #[must_use]
    pub const fn draw_distance_scale(self) -> f32 {
        self.distance_scale
    }

    /// LOD distance scale for coarser mesh selection.
    #[must_use]
    pub const fn lod_distance_scale(self) -> f32 {
        self.lod_scale
    }
}

#[derive(Resource, Default)]
struct FrameBudgetState {
    window: [f32; FRAME_BUDGET_WINDOW],
    index: usize,
    filled: usize,
    last_warn_at: Option<f64>,
    last_recovery_warn_at: Option<f64>,
    recent_drops: VecDeque<f64>,
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
            .init_resource::<FrameBudgetRecovery>()
            .add_systems(
                PostUpdate,
                enforce_frame_budget.after(FrameTimeDiagnosticsPlugin::diagnostic_system),
            );
    }
}

fn enforce_frame_budget(
    diagnostics: Res<DiagnosticsStore>,
    time: Res<Time>,
    mut metrics: ResMut<FrameBudgetMetrics>,
    mut state: ResMut<FrameBudgetState>,
    mut recovery: ResMut<FrameBudgetRecovery>,
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

    state.window[state.index] = frame_ms;
    state.index = (state.index + 1) % FRAME_BUDGET_WINDOW;
    if state.filled < FRAME_BUDGET_WINDOW {
        state.filled += 1;
    }

    if state.filled < FRAME_BUDGET_WINDOW {
        return;
    }

    let avg_ms = state.window.iter().sum::<f32>() / FRAME_BUDGET_WINDOW as f32;
    let now = time.elapsed_secs_f64();

    while let Some(&drop_at) = state.recent_drops.front() {
        if now - drop_at <= FRAME_BUDGET_RECOVERY_WINDOW_SECS {
            break;
        }
        let _ = state.recent_drops.pop_front();
    }

    if avg_ms <= FRAME_BUDGET_MS {
        let next = recovery_state_for_recent_drops(state.recent_drops.len());
        if next != *recovery {
            if recovery.active {
                info!("frame budget recovery exited (frame stability restored)");
            }
            *recovery = next;
        }
        return;
    }

    metrics.drop_count = metrics.drop_count.saturating_add(1);
    state.recent_drops.push_back(now);
    while let Some(&drop_at) = state.recent_drops.front() {
        if now - drop_at <= FRAME_BUDGET_RECOVERY_WINDOW_SECS {
            break;
        }
        let _ = state.recent_drops.pop_front();
    }

    let next = recovery_state_for_recent_drops(state.recent_drops.len());
    if next != *recovery {
        if next.severe {
            let should_warn = state
                .last_recovery_warn_at
                .is_none_or(|last| now - last >= WARN_THROTTLE_SECS);
            if should_warn {
                warn!(
                    "frame budget recovery (strong): {} drops over {:.1}s, reducing draw distance and increasing LOD coarse bias",
                    state.recent_drops.len(),
                    FRAME_BUDGET_RECOVERY_WINDOW_SECS,
                );
                state.last_recovery_warn_at = Some(now);
            }
        } else {
            info!(
                "frame budget recovery (mild): {} drops over {:.1}s, reducing draw distance and increasing LOD coarse bias",
                state.recent_drops.len(),
                FRAME_BUDGET_RECOVERY_WINDOW_SECS,
            );
        }
        *recovery = next;
    }

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

#[must_use]
const fn recovery_state_for_recent_drops(recent_drops: usize) -> FrameBudgetRecovery {
    if recent_drops > FRAME_BUDGET_RECOVERY_SEVERE_THRESHOLD {
        FrameBudgetRecovery {
            active: true,
            distance_scale: FRAME_BUDGET_RECOVERY_SEVERE_DISTANCE_SCALE,
            lod_scale: FRAME_BUDGET_RECOVERY_SEVERE_LOD_SCALE,
            severe: true,
        }
    } else if recent_drops > FRAME_BUDGET_RECOVERY_SOFT_THRESHOLD {
        FrameBudgetRecovery {
            active: true,
            distance_scale: FRAME_BUDGET_RECOVERY_SOFT_DISTANCE_SCALE,
            lod_scale: FRAME_BUDGET_RECOVERY_SOFT_LOD_SCALE,
            severe: false,
        }
    } else {
        FrameBudgetRecovery::default()
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
    fn recovery_transitions_with_drop_count() {
        assert!(!recovery_state_for_recent_drops(5).active);
        assert!(!recovery_state_for_recent_drops(5).severe);
        let soft = recovery_state_for_recent_drops(6);
        assert!(soft.active);
        assert!(!soft.severe);
        assert!((soft.distance_scale - FRAME_BUDGET_RECOVERY_SOFT_DISTANCE_SCALE).abs() < f32::EPSILON);
        assert!(
            (soft.lod_scale - FRAME_BUDGET_RECOVERY_SOFT_LOD_SCALE).abs() < f32::EPSILON
        );
        assert!(!recovery_state_for_recent_drops(20).severe);
        assert!(!recovery_state_for_recent_drops(20).active);
        let severe = recovery_state_for_recent_drops(21);
        assert!(severe.active);
        assert!(severe.severe);
        assert!((severe.distance_scale - FRAME_BUDGET_RECOVERY_SEVERE_DISTANCE_SCALE).abs() < f32::EPSILON);
        assert!((severe.lod_scale - FRAME_BUDGET_RECOVERY_SEVERE_LOD_SCALE).abs() < f32::EPSILON);
    }
}
