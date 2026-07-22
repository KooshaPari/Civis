//! Frame-time budget enforcer and quality-recovery helpers for the Bevy client.
//!
//! This module was recovered from history so `god_actions.rs` can keep using the
//! existing cull-distance and quality-mode API without further call-site churn.
//!
//! `FRAME_BUDGET_MS` is 33.3 ms (a 30 FPS floor). When sustained over-budget
//! frames accumulate, [`GpuQualityMode`] steps down; it recovers once recent
//! drops age out of [`DROP_RECOVERY_WINDOW_SECS`].

use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use std::collections::VecDeque;

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

/// Alias used by god-tool remesh paths that read the active recovery mode.
pub type FrameBudgetRecovery = GpuQualityMode;

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

/// Scale camera distance before LOD band selection.
#[must_use]
pub fn scaled_mesh_lod_distance(distance: f32, mode: GpuQualityMode) -> f32 {
    distance * mode.lod_distance_scale()
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
    last_recovery_warn_at: Option<f64>,
    recent_drops: VecDeque<f64>,
}

impl Default for FrameBudgetState {
    fn default() -> Self {
        Self {
            window: [0.0; FRAME_BUDGET_WINDOW],
            index: 0,
            filled: 0,
            last_warn_at: None,
            last_recovery_warn_at: None,
            recent_drops: VecDeque::new(),
        }
    }
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
            .add_systems(
                PostUpdate,
                enforce_frame_budget.after(FrameTimeDiagnosticsPlugin::diagnostic_system),
            );
    }
}

pub(crate) fn quality_for_drop_count(drop_count: u64) -> GpuQualityMode {
    if drop_count >= DROP_THRESHOLD_CRITICAL {
        GpuQualityMode::Critical
    } else if drop_count >= DROP_THRESHOLD_REDUCED {
        GpuQualityMode::Reduced
    } else {
        GpuQualityMode::Full
    }
}

fn prune_recent_drops(state: &mut FrameBudgetState, now: f64) {
    while let Some(front) = state.recent_drops.front().copied() {
        if now - front > DROP_RECOVERY_WINDOW_SECS {
            state.recent_drops.pop_front();
        } else {
            break;
        }
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
    let now = time.elapsed_secs_f64();
    let over_budget = avg_ms > FRAME_BUDGET_MS;

    if over_budget {
        metrics.drop_count = metrics.drop_count.saturating_add(1);
        state.recent_drops.push_back(now);
        let should_warn = state
            .last_warn_at
            .map(|last| now - last >= WARN_THROTTLE_SECS)
            .unwrap_or(true);
        if should_warn {
            warn!("Frame budget exceeded: {avg_ms:.1}ms (target {FRAME_BUDGET_MS})");
            state.last_warn_at = Some(now);
        }
    }

    // Always prune aged drops and recompute quality so under-budget frames can
    // upgrade out of Reduced/Critical (hysteresis via DROP_RECOVERY_WINDOW_SECS).
    prune_recent_drops(&mut state, now);
    let new_mode = quality_for_drop_count(state.recent_drops.len() as u64);
    if new_mode != *recovery {
        if new_mode.cull_distance_scale() > recovery.cull_distance_scale() {
            let should_warn = state
                .last_recovery_warn_at
                .map(|last| now - last >= WARN_THROTTLE_SECS)
                .unwrap_or(true);
            if should_warn {
                info!(
                    "GpuQualityMode recovering: {:?} → {:?} (avg {avg_ms:.1}ms, drops {})",
                    *recovery,
                    new_mode,
                    state.recent_drops.len()
                );
                state.last_recovery_warn_at = Some(now);
            }
        }
        *recovery = new_mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_full_at_zero_drops() {
        assert_eq!(quality_for_drop_count(0), GpuQualityMode::Full);
    }

    #[test]
    fn quality_reduced_at_drop_threshold() {
        assert_eq!(
            quality_for_drop_count(DROP_THRESHOLD_REDUCED),
            GpuQualityMode::Reduced
        );
    }

    #[test]
    fn quality_critical_at_drop_threshold() {
        assert_eq!(
            quality_for_drop_count(DROP_THRESHOLD_CRITICAL),
            GpuQualityMode::Critical
        );
    }

    #[test]
    fn cull_distance_scales_by_quality_mode() {
        assert_eq!(GpuQualityMode::Full.cull_distance_scale(), 1.0);
        assert_eq!(GpuQualityMode::Reduced.cull_distance_scale(), REDUCED_CULL_SCALE);
        assert_eq!(
            GpuQualityMode::Critical.cull_distance_scale(),
            REDUCED_CULL_SCALE * CRITICAL_CULL_SCALE
        );
    }

    #[test]
    fn lod_distance_is_inverse_of_cull_scale() {
        for mode in [
            GpuQualityMode::Full,
            GpuQualityMode::Reduced,
            GpuQualityMode::Critical,
        ] {
            assert!(
                (mode.lod_distance_scale() * mode.cull_distance_scale() - 1.0).abs() < f32::EPSILON
            );
        }
    }

    #[test]
    fn scaled_cull_and_lod_helpers_apply_mode_multipliers() {
        let base = 100.0;
        assert_eq!(scaled_cull_distance(base, GpuQualityMode::Full), base);
        assert_eq!(
            scaled_cull_distance(base, GpuQualityMode::Reduced),
            base * REDUCED_CULL_SCALE
        );
        assert_eq!(
            scaled_cull_distance(base, GpuQualityMode::Critical),
            base * REDUCED_CULL_SCALE * CRITICAL_CULL_SCALE
        );

        let distance = 90.0;
        assert_eq!(
            scaled_mesh_lod_distance(distance, GpuQualityMode::Reduced),
            distance / REDUCED_CULL_SCALE
        );
    }
}
