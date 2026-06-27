#![cfg(all(feature = "bevy", feature = "egui"))]

//! Performance HUD overlay (FR-CIV-PERF-001).
//! P key toggles. Shows FPS, frame-ms, sim tick, tick-ms, civilians, factions.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::menus::in_game;

/// Latest snapshot values forwarded from HudState each frame.
#[derive(Resource, Default, Clone)]
pub struct PerfMetrics {
    pub fps: f32,
    pub tick: u64,
    pub civilian_count: usize,
    pub faction_count: usize,
    /// Server-reported tick duration (ms), from sim.perf poll.
    pub tick_ms: f64,
}

#[derive(Resource, Default)]
pub struct PerfHudState {
    pub visible: bool,
}

pub struct PerfHudPlugin;

impl Plugin for PerfHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PerfHudState>()
            .init_resource::<PerfMetrics>()
            .add_systems(Update, (toggle_perf_hud, draw_perf_hud).chain().run_if(in_game));
    }
}

fn toggle_perf_hud(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<PerfHudState>) {
    if keys.just_pressed(KeyCode::KeyP) {
        state.visible = !state.visible;
    }
}

fn fps_color(fps: f32) -> egui::Color32 {
    if fps > 55.0 {
        egui::Color32::from_rgb(80, 220, 120)
    } else if fps > 30.0 {
        egui::Color32::from_rgb(240, 200, 60)
    } else {
        egui::Color32::from_rgb(240, 80, 80)
    }
}

fn draw_perf_hud(
    mut contexts: EguiContexts,
    state: Res<PerfHudState>,
    metrics: Res<PerfMetrics>,
) {
    if !state.visible { return; }

    let fps = metrics.fps;
    let frame_ms = if fps > 0.0 { 1000.0 / fps } else { 0.0 };

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let screen = ctx.screen_rect();

    egui::Area::new(egui::Id::new("perf_hud"))
        .fixed_pos(egui::pos2(screen.max.x - 230.0, 8.0))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 210))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(40, 45, 55)))
                .rounding(egui::Rounding::same(6_u8))
                .inner_margin(egui::Margin::symmetric(10_i8, 6_i8))
                .show(ui, |ui| {
                    ui.set_width(210.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("FPS: {:.1}", fps))
                                .monospace().color(fps_color(fps)).size(12.0),
                        );
                        ui.label(
                            egui::RichText::new(format!("  frame: {:.1}ms", frame_ms))
                                .monospace().color(egui::Color32::from_rgb(160, 170, 180)).size(12.0),
                        );
                    });
                    ui.label(
                        egui::RichText::new(format!(
                            "sim tick: {}  tick_ms: {:.1}ms",
                            metrics.tick, metrics.tick_ms
                        ))
                        .monospace().color(egui::Color32::from_rgb(160, 170, 180)).size(12.0),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "civilians: {}  factions: {}",
                            metrics.civilian_count, metrics.faction_count
                        ))
                        .monospace().color(egui::Color32::from_rgb(160, 170, 180)).size(12.0),
                    );
                });
        });
}
