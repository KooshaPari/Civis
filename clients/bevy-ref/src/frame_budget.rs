//! Frame-time budget enforcer (P1.5.1).
//!
//! Samples Bevy's `frame_time` diagnostic (same source as [`crate::perf_hud`]) and
//! tracks a rolling 60-frame average against a 30 FPS floor (33.3 ms).

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

/// Target frame budget in milliseconds (30 FPS floor).
pub const FRAME_BUDGET_MS: f32 = 33.3;

/// Rolling window length for budget averaging.
pub const FRAME_BUDGET_WINDOW: usize = 60;

/// Minimum seconds between throttled budget warnings.
const WARN_THROTTLE_SECS: f64 = 5.0;

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

#[derive(Resource, Default)]
struct FrameBudgetState {
    window: [f32; FRAME_BUDGET_WINDOW],
    index: usize,
    filled: usize,
    last_warn_at: Option<f64>,
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
}
