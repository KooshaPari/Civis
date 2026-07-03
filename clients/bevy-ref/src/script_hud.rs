#![cfg(all(feature = "bevy", feature = "egui"))]
//! Script HUD: renders emergent writing-system glyphs from faction language.
//!
//! Displays a grid of procedurally generated glyphs from each civilization's
//! unique writing system. Glyphs are drawn via egui painter from stroke vectors.
//! Toggle with `F11` or always visible as a compact panel.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use civ_engine::writing::{glyphs_for_language, Stroke};

use crate::live_stream::LiveStreamScene;
use crate::ui_theme::{ACCENT, DIM, PANEL_FILL};

// ── Resources ────────────────────────────────────────────────────────────────

/// HUD open/closed toggle state.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScriptHudOpen(pub bool);

impl Default for ScriptHudOpen {
    fn default() -> Self {
        Self(true)
    }
}

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Registers the script (writing system) HUD panel.
pub struct ScriptHudPlugin;

impl Plugin for ScriptHudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScriptHudOpen>()
            .add_systems(Update, toggle_script_hud)
            .add_systems(EguiPrimaryContextPass, draw_script_hud);
    }
}

// ── Systems ───────────────────────────────────────────────────────────────────

fn toggle_script_hud(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<ScriptHudOpen>) {
    if keys.just_pressed(KeyCode::F11) {
        open.0 = !open.0;
    }
}

fn draw_script_hud(
    mut contexts: EguiContexts,
    open: Res<ScriptHudOpen>,
    scene: Res<LiveStreamScene>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Pick the first faction's seed; if no factions, use a default seed.
    let faction_seed = scene
        .faction_entries
        .first()
        .map(|f| f.id as u64)
        .unwrap_or(42);

    // Generate glyphs for the faction's language: 8 phonemes, 16 glyphs
    let glyphs = glyphs_for_language(faction_seed, 8, 16);

    egui::Window::new("Script")
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-8.0, 8.0))
        .resizable(false)
        .collapsible(false)
        .title_bar(true)
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(10))
                .corner_radius(egui::CornerRadius::same(10)),
        )
        .show(ctx, |ui| {
            ui.set_min_width(240.0);

            // Header: faction writing system
            ui.label(
                egui::RichText::new("Emergent Writing")
                    .color(ACCENT)
                    .strong()
                    .size(12.0),
            );
            ui.add_space(4.0);

            // Grid of glyphs (8 columns, 2 rows for 16 glyphs)
            let cell_size = 24.0;
            let columns = 8;
            let glyph_count = glyphs.len();

            for (idx, glyph) in glyphs.iter().enumerate() {
                // Start a new row every 8 glyphs
                if idx % columns == 0 && idx > 0 {
                    ui.end_row();
                }

                // Allocate space for this glyph cell and draw it
                let (rect, _) = ui.allocate_exact_size(
                    egui::vec2(cell_size, cell_size),
                    egui::Sense::hover(),
                );

                // Draw glyph strokes in the cell
                draw_glyph_in_rect(ui.painter(), rect, glyph);
            }

            ui.add_space(6.0);
            ui.label(
                egui::RichText::new("[F11] to hide")
                    .color(DIM)
                    .small()
                    .italics(),
            );
        });
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Draw a single glyph's strokes into a rect via egui painter.
/// Strokes are normalized to [0, 1]; we scale them to fit the cell with padding.
fn draw_glyph_in_rect(painter: &egui::Painter, rect: egui::Rect, glyph: &civ_engine::writing::Glyph) {
    // Light border around cell
    painter.rect_stroke(
        rect,
        egui::CornerRadius::same(2),
        egui::Stroke::new(0.5, egui::Color32::from_gray(80)),
        egui::StrokeKind::Middle,
    );

    if glyph.strokes.is_empty() {
        return;
    }

    // Get bounding box of glyph strokes and normalize
    let (min_x, min_y, max_x, max_y) = glyph.bounding_box();
    let width = (max_x - min_x).max(0.1); // Avoid division by zero
    let height = (max_y - min_y).max(0.1);

    // Padding inside cell
    let padding = 2.0;
    let cell_width = rect.width() - 2.0 * padding;
    let cell_height = rect.height() - 2.0 * padding;

    let scale_x = cell_width / width;
    let scale_y = cell_height / height;
    let scale = scale_x.min(scale_y); // Maintain aspect ratio

    let cell_center_x = rect.center().x;
    let cell_center_y = rect.center().y;

    let scaled_width = width * scale;
    let scaled_height = height * scale;

    let offset_x = cell_center_x - scaled_width / 2.0;
    let offset_y = cell_center_y - scaled_height / 2.0;

    // Draw each stroke
    let stroke_color = egui::Color32::from_rgb(200, 200, 200);
    let stroke_width = 1.0;

    for stroke in &glyph.strokes {
        // Normalize stroke coordinates
        let x0_norm = (stroke.x0 - min_x) / width;
        let y0_norm = (stroke.y0 - min_y) / height;
        let x1_norm = (stroke.x1 - min_x) / width;
        let y1_norm = (stroke.y1 - min_y) / height;

        // Scale and translate to cell
        let p0 = egui::pos2(
            offset_x + x0_norm * scaled_width,
            offset_y + y0_norm * scaled_height,
        );
        let p1 = egui::pos2(
            offset_x + x1_norm * scaled_width,
            offset_y + y1_norm * scaled_height,
        );

        if stroke.curvature.abs() < 0.01 {
            // Straight line
            painter.line_segment([p0, p1], egui::Stroke::new(stroke_width, stroke_color));
        } else {
            // Approximate arc with 4 segments (simple quadratic bezier approximation)
            draw_curved_stroke(painter, p0, p1, stroke.curvature, stroke_width, stroke_color);
        }
    }
}

/// Draw a curved stroke as a series of line segments (quadratic bezier approximation).
fn draw_curved_stroke(
    painter: &egui::Painter,
    p0: egui::Pos2,
    p1: egui::Pos2,
    curvature: f32,
    stroke_width: f32,
    color: egui::Color32,
) {
    const CURVE_SEGMENTS: usize = 4;
    let mut prev = p0;

    for i in 1..=CURVE_SEGMENTS {
        let t = i as f32 / CURVE_SEGMENTS as f32;

        // Linear interpolation
        let linear_x = p0.x + (p1.x - p0.x) * t;
        let linear_y = p0.y + (p1.y - p0.y) * t;

        // Perpendicular bulge (approximated quadratic bezier)
        let dx = p1.x - p0.x;
        let dy = p1.y - p0.y;
        let mid_t = 0.5;
        let bulge_scale = curvature * (4.0 * mid_t * (1.0 - mid_t)); // max bulge at t=0.5

        let perpendicular_x = -dy * bulge_scale;
        let perpendicular_y = dx * bulge_scale;

        let current = egui::pos2(linear_x + perpendicular_x, linear_y + perpendicular_y);
        painter.line_segment([prev, current], egui::Stroke::new(stroke_width, color));
        prev = current;
    }
}
