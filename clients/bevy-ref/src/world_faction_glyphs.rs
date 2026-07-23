#![cfg(all(feature = "bevy", feature = "egui"))]
//! In-world faction emergent-glyph sigils: render each faction's lead glyph as a
//! floating egui overlay at each building's projected screen position.
//!
//! **Approach:** Egui layer overlay with world-to-viewport projection. For each
//! faction with buildings, compute the lead glyph (glyphs_for_language seed → [0]),
//! project a building's center to screen space, and draw that glyph (scaled small,
//! ~20px) as a faction sigil via egui painter. Offscreen glyphs are culled. No mesh
//! overhead; cheap (one glyph per faction per frame).

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use civ_engine::writing::{glyphs_for_language, Glyph};

use crate::live_stream::{LiveBuildingTag, LiveStreamScene};

// ── Systems ───────────────────────────────────────────────────────────────────

/// Draw faction emergent-glyph sigils in-world (egui overlay, world-to-screen projection).
fn draw_world_faction_glyphs(
    mut contexts: EguiContexts,
    scene: Res<LiveStreamScene>,
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    building_q: Query<&GlobalTransform, With<LiveBuildingTag>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Guard: if no factions or no camera, skip
    if scene.faction_entries.is_empty() {
        return;
    }

    let Ok(window) = windows.single() else {
        return;
    };

    let Ok((camera, camera_transform)) = camera_q.single() else {
        return;
    };

    // Create a custom layer for faction glyphs (rendered on top of other egui)
    let painter = ctx.layer_painter(egui::LayerId::new(
        egui::Order::Foreground,
        egui::Id::new("faction_glyphs"),
    ));

    let glyph_size = 20.0;
    let margin = glyph_size + 4.0;

    // Collect building world positions once (wire has no per-building faction_id yet).
    let building_positions: Vec<Vec3> = scene
        .buildings
        .values()
        .filter_map(|&building_entity| {
            building_q
                .get(building_entity)
                .ok()
                .map(|transform| transform.translation())
        })
        .collect();
    if building_positions.is_empty() {
        return;
    }

    // For each faction, pick a building slot + fan offset so sigils do not stack.
    for (faction_index, faction_entry) in scene.faction_entries.iter().enumerate() {
        let building_world_pos = building_positions[faction_index % building_positions.len()];

        // Generate the faction's lead glyph from its seed
        let faction_seed = faction_entry.id as u64;
        let glyphs = glyphs_for_language(faction_seed, 8, 1);
        let Some(lead_glyph) = glyphs.first() else {
            continue;
        };

        // Project building world position to screen (viewport) coordinates using Camera::world_to_viewport
        let screen_pos = match camera.world_to_viewport(camera_transform, building_world_pos) {
            Ok(viewport_pos) => viewport_pos,
            Err(_) => continue, // Skip if off-screen or behind camera
        };

        // Fan offset when multiple factions share the same building anchor.
        let fan = faction_index as f32;
        let screen_pos = bevy::math::Vec2::new(
            screen_pos.x + (fan % 4.0) * (glyph_size + 6.0) - glyph_size,
            screen_pos.y + (fan / 4.0).floor() * (glyph_size + 6.0),
        );

        // Cull if off-screen (add margin for glyph size)
        if screen_pos.x < -margin
            || screen_pos.x > window.physical_width() as f32 + margin
            || screen_pos.y < -margin
            || screen_pos.y > window.physical_height() as f32 + margin
        {
            continue;
        }

        let egui_screen_pos = egui::pos2(screen_pos.x, screen_pos.y);

        // Draw the glyph sigil at the projected position
        draw_glyph_sigil(
            &painter,
            egui_screen_pos,
            glyph_size,
            lead_glyph,
            faction_entry.id,
        );
    }
}

/// Draw a single glyph as a faction sigil at a screen position (egui painter).
fn draw_glyph_sigil(
    painter: &egui::Painter,
    screen_pos: egui::Pos2,
    size_px: f32,
    glyph: &Glyph,
    faction_id: u32,
) {
    if glyph.strokes.is_empty() {
        return;
    }

    // Glyph bounding box for normalization
    let (min_x, min_y, max_x, max_y) = glyph.bounding_box();
    let width = (max_x - min_x).max(0.01);
    let height = (max_y - min_y).max(0.01);

    // Scale to fit the sigil size, maintaining aspect ratio
    let scale_x = size_px / width;
    let scale_y = size_px / height;
    let scale = scale_x.min(scale_y);

    let scaled_width = width * scale;
    let scaled_height = height * scale;

    // Center the glyph at screen_pos
    let offset_x = screen_pos.x - scaled_width / 2.0;
    let offset_y = screen_pos.y - scaled_height / 2.0;

    // Stroke color based on faction (deterministic from faction_id)
    let stroke_color = faction_color_from_id(faction_id);
    let stroke_width = 1.5;

    // Draw each stroke
    for stroke in &glyph.strokes {
        let p0 = egui::pos2(
            offset_x + (stroke.x0 - min_x) * scale,
            offset_y + (stroke.y0 - min_y) * scale,
        );
        let p1 = egui::pos2(
            offset_x + (stroke.x1 - min_x) * scale,
            offset_y + (stroke.y1 - min_y) * scale,
        );

        // Draw straight lines (curvature ignored for MVP)
        painter.line_segment([p0, p1], egui::Stroke::new(stroke_width, stroke_color));
    }

    // Optional: add a small circle marker behind the glyph
    let marker_radius = size_px / 4.0;
    painter.circle_stroke(
        screen_pos,
        marker_radius,
        egui::Stroke::new(
            0.8,
            egui::Color32::from_rgba_unmultiplied(100, 100, 100, 80),
        ),
    );
}

/// Crest-aligned swatch color (matches [`crate::faction_hud`] crest hexes).
fn faction_color_from_id(faction_id: u32) -> egui::Color32 {
    const CREST_COLORS: [egui::Color32; 6] = [
        egui::Color32::from_rgb(0xE8, 0xB8, 0x4B), // gold
        egui::Color32::from_rgb(0x29, 0x80, 0xB9), // blue
        egui::Color32::from_rgb(0x9B, 0x59, 0xB6), // violet
        egui::Color32::from_rgb(0x27, 0xAE, 0x60), // green
        egui::Color32::from_rgb(0xC0, 0x39, 0x2B), // red
        egui::Color32::from_rgb(0x50, 0xC8, 0xF0), // cyan
    ];
    CREST_COLORS[(faction_id as usize) % CREST_COLORS.len()]
}

// ── Plugin ────────────────────────────────────────────────────────────────────

/// Registers the in-world faction glyph rendering system.
pub struct WorldFactionGlyphsPlugin;

impl Plugin for WorldFactionGlyphsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            draw_world_faction_glyphs.run_if(resource_exists::<LiveStreamScene>),
        );
    }
}
