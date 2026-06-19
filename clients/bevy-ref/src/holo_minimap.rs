#![cfg(all(feature = "bevy", feature = "egui"))]

//! Holocron **orb** minimap overlay — 2.5D holographic projector chrome.
//!
//! The live terrain texture is rendered by [`crate::minimap`] (Bevy UI +
//! [`MinimapCamera`]). This module paints the holo-cyan rim, scanlines, and
//! corner brackets on [`EguiPrimaryContextPass`] aligned to that panel.
//!
//! TODO(holohud-3d): Replace this 2.5D egui overlay with a tilted 3D mesh
//! quad + custom fresnel/emissive shader sampling the minimap render target.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::menus::in_game;
use crate::minimap::{MINIMAP_INSET, MINIMAP_SIZE};
use crate::ui_holo::{holo_frame, holo_text, scanlines, HoloPhase};
use crate::ui_theme::{DECK_BORDER, HOLO_CYAN, HOLO_DEEP, RADIUS_PANEL};

/// Egui holo projector overlay for the bottom-right minimap panel.
pub struct HoloMinimapPlugin;

impl Plugin for HoloMinimapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            draw_holo_minimap_overlay.run_if(in_game),
        );
    }
}

/// Screen-space rect matching the Bevy UI [`MinimapRoot`] anchor.
fn minimap_content_rect(ctx: &egui::Context) -> egui::Rect {
    let screen = ctx.content_rect();
    egui::Rect::from_min_size(
        egui::pos2(
            screen.right() - MINIMAP_INSET - MINIMAP_SIZE,
            screen.bottom() - MINIMAP_INSET - MINIMAP_SIZE,
        ),
        egui::vec2(MINIMAP_SIZE, MINIMAP_SIZE),
    )
}

fn draw_holo_minimap_overlay(mut contexts: EguiContexts, time: Res<Time>) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    let rect = minimap_content_rect(ctx);
    let phase = HoloPhase::settled(time.elapsed_secs());
    let layer = egui::LayerId::new(egui::Order::Foreground, egui::Id::new("holo_minimap"));
    let painter = ctx.layer_painter(layer);
    let op = phase.opacity();

    painter.rect_filled(
        rect,
        RADIUS_PANEL as f32,
        HOLO_DEEP.gamma_multiply(0.08 * op),
    );
    scanlines(&painter, rect, phase);
    holo_frame(&painter, rect, phase);
    painter.rect_stroke(
        rect,
        RADIUS_PANEL as f32,
        egui::Stroke::new(1.5, HOLO_CYAN.gamma_multiply(0.75 * op)),
        egui::StrokeKind::Inside,
    );
    painter.rect_stroke(
        rect.shrink(2.0),
        (RADIUS_PANEL - 1) as f32,
        egui::Stroke::new(1.0, DECK_BORDER),
        egui::StrokeKind::Inside,
    );
    holo_text(
        &painter,
        rect.left_top() + egui::vec2(8.0, 6.0),
        "HOLOCRON MAP",
        10.0,
        phase,
    );
}
