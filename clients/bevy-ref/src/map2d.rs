#![cfg(all(feature = "bevy", feature = "egui"))]

//! 2D top-down "cartographic" alternate view for the Civis reference client.
//!
//! This is a stylised, higher-than-8-bit strategy map — a WorldBox-adjacent but
//! cleaner look. It is a *render mode*, not a HUD widget: when active it covers
//! the whole screen with a procedurally-shaded relief basemap and crisp vector
//! markers for the live simulation (agents cluster-tinted, buildings as typed
//! icons). The 3D scene is hidden while the map is up.
//!
//! ## Triggers
//! * **`M`** toggles the map on/off (HUD hint shown bottom-left in 3D).
//! * **Auto-engage**: when the orbit camera's [`CameraRig::distance`] crosses
//!   [`AUTO_ENGAGE_DISTANCE`] (zooming far out), the map fades in; zooming back
//!   below [`AUTO_DISENGAGE_DISTANCE`] returns to 3D. A manual `M` toggle pins
//!   the choice so auto-switching does not fight the user until they zoom back
//!   across the hysteresis band.
//!
//! ## Rendering approach — hybrid (procedural raster base + vector overlay)
//! The terrain basemap is rasterised **once** from the deterministic
//! [`crate::terrain`] height field into an egui texture: per-pixel biome palette
//! by elevation/water, Lambertian hillshade from a finite-difference normal, a
//! subtle ordered-dither so flat bands don't posterise, and soft coastline
//! darkening. Live entities are drawn on top with the egui painter (anti-aliased
//! vector circles/diamonds/houses) so they stay crisp at any zoom. A small set
//! of hand-authored SVGs under `assets/ui/map2d/` documents the marker language
//! (rasterise via `Tools/gen-map2d-svg.ps1` + `Tools/rasterize-svg.ps1`).
//!
//! All world data is read from the same sources the 3D view uses
//! ([`SimState`] agents/buildings, [`crate::terrain`] heights) — nothing is
//! fabricated.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use civ_agents::Civilian;
use civ_engine::{Building, BuildingType};

use crate::camera::CameraRig;
use crate::sim_bridge::SimState;
use crate::terrain::{terrain_height, HEIGHT_SCALE, WATER_LEVEL, WORLD_SIZE};
use crate::AttachMode;

/// Basemap raster resolution per side (crisp, sub-tile detail — not 8-bit).
const MAP_TEX: usize = 512;

/// Orbit distance at/above which the 2D map auto-engages (MAX_DISTANCE is 600).
pub const AUTO_ENGAGE_DISTANCE: f32 = 480.0;
/// Orbit distance at/below which the map auto-disengages (hysteresis band).
pub const AUTO_DISENGAGE_DISTANCE: f32 = 430.0;

/// Whole-screen 2D map view state.
#[derive(Resource)]
pub struct MapView {
    /// Currently showing the 2D map.
    pub active: bool,
    /// Smoothed 0..1 fade weight (1.0 = fully in map mode).
    pub fade: f32,
    /// Pan offset in normalised map units (0..1 space), applied as a screen shift.
    pub pan: egui::Vec2,
    /// Zoom multiplier within the 2D view (1.0 = fit-to-screen).
    pub zoom: f32,
    /// True once the user pressed `M`; suppresses auto-engage until they zoom
    /// back across the hysteresis band (so the manual choice is respected).
    manual_override: bool,
}

impl Default for MapView {
    fn default() -> Self {
        Self {
            active: false,
            fade: 0.0,
            pan: egui::Vec2::ZERO,
            zoom: 1.0,
            manual_override: false,
        }
    }
}

/// Cached basemap texture handle (rasterised once, lazily, on first map open).
#[derive(Resource, Default)]
struct MapBasemap {
    handle: Option<egui::TextureHandle>,
}

/// Plugin: registers the 2D alternate map view (key + far-zoom toggle).
#[derive(Default)]
pub struct Map2dPlugin;

impl Plugin for Map2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapView>()
            .init_resource::<MapBasemap>()
            .add_systems(Update, (toggle_map_hotkey, auto_engage_from_zoom, tick_fade))
            .add_systems(EguiPrimaryContextPass, (draw_map_view, draw_map_hint));
    }
}

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

/// `M` toggles the map and pins the manual override.
fn toggle_map_hotkey(keys: Res<ButtonInput<KeyCode>>, mut view: ResMut<MapView>) {
    if keys.just_pressed(KeyCode::KeyM) {
        view.active = !view.active;
        view.manual_override = true;
        if !view.active {
            // Reset pan/zoom so the next open is framed to fit.
            view.pan = egui::Vec2::ZERO;
            view.zoom = 1.0;
        }
    }
}

/// Auto-engage / disengage from the orbit camera distance with hysteresis.
fn auto_engage_from_zoom(rig: Res<CameraRig>, mut view: ResMut<MapView>) {
    let d = rig.distance;
    if !view.active && d >= AUTO_ENGAGE_DISTANCE {
        view.active = true;
        view.manual_override = false;
    } else if view.active && d <= AUTO_DISENGAGE_DISTANCE && !view.manual_override {
        view.active = false;
    }
    // Once the user has zoomed back in past the lower band, clear the manual
    // pin so auto-engage can fire again on the next zoom-out.
    if d <= AUTO_DISENGAGE_DISTANCE {
        view.manual_override = false;
    }
}

/// Smoothly ramp the fade weight toward the active target (≈0.18s transition).
fn tick_fade(time: Res<Time>, mut view: ResMut<MapView>) {
    let target = if view.active { 1.0 } else { 0.0 };
    let rate = 6.0 * time.delta_secs();
    view.fade += (target - view.fade) * rate.min(1.0);
    if (view.fade - target).abs() < 0.002 {
        view.fade = target;
    }
}

// ---------------------------------------------------------------------------
// Procedural basemap raster (built once)
// ---------------------------------------------------------------------------

/// Biome-ish palette by elevation + water, mirroring the 3D `terrain` look but
/// flattened for a clean cartographic read. Returns linear-ish sRGB 0..1.
fn map_palette(h: f32) -> [f32; 3] {
    let sea = WATER_LEVEL;
    if h < sea - 0.05 * HEIGHT_SCALE {
        [0.06, 0.18, 0.42] // deep ocean
    } else if h < sea {
        [0.16, 0.38, 0.70] // shallow ocean
    } else if h < sea + 1.5 {
        [0.84, 0.78, 0.56] // beach
    } else {
        let alt = ((h - sea) / (HEIGHT_SCALE - sea)).clamp(0.0, 1.0);
        if alt < 0.18 {
            [0.36, 0.62, 0.28] // grassland
        } else if alt < 0.40 {
            [0.22, 0.50, 0.24] // forest/lowland
        } else if alt < 0.62 {
            [0.46, 0.55, 0.30] // upland
        } else if alt < 0.82 {
            [0.50, 0.48, 0.45] // rock
        } else {
            [0.95, 0.96, 0.98] // snow
        }
    }
}

/// 4x4 Bayer ordered-dither matrix scaled to ±~1/512 so bands don't posterise.
const BAYER4: [[f32; 4]; 4] = [
    [0.0, 8.0, 2.0, 10.0],
    [12.0, 4.0, 14.0, 6.0],
    [3.0, 11.0, 1.0, 9.0],
    [15.0, 7.0, 13.0, 5.0],
];

/// Rasterise the deterministic terrain field into a crisp shaded-relief image.
fn build_basemap_image() -> egui::ColorImage {
    let mut pixels = vec![egui::Color32::BLACK; MAP_TEX * MAP_TEX];
    let cell = WORLD_SIZE / (MAP_TEX as f32 - 1.0);
    // Light from the north-west, slightly elevated.
    let light = Vec3::new(-0.55, 0.72, -0.42).normalize();

    for py in 0..MAP_TEX {
        for px in 0..MAP_TEX {
            let wx = px as f32 * cell;
            let wz = py as f32 * cell;
            let h = terrain_height(wx, wz);

            // Finite-difference normal for hillshade (sample neighbours).
            let hx = terrain_height((wx + cell).min(WORLD_SIZE), wz)
                - terrain_height((wx - cell).max(0.0), wz);
            let hz = terrain_height(wx, (wz + cell).min(WORLD_SIZE))
                - terrain_height(wx, (wz - cell).max(0.0));
            let n = Vec3::new(-hx, 2.0 * cell, -hz).normalize();
            let lambert = n.dot(light).clamp(0.0, 1.0);

            let mut base = map_palette(h);
            let is_water = h < WATER_LEVEL;
            // Hillshade only the land; water gets a gentle flat sheen.
            let shade = if is_water {
                0.92 + 0.08 * lambert
            } else {
                0.55 + 0.55 * lambert
            };

            // Soft coastline darkening: emphasise the shore edge.
            if !is_water && h < WATER_LEVEL + 2.5 {
                let t = (h - WATER_LEVEL) / 2.5;
                let edge = 0.7 + 0.3 * t;
                base = [base[0] * edge, base[1] * edge, base[2] * edge];
            }

            // Ordered dither in the LSBs to break up flat palette bands.
            let d = (BAYER4[py & 3][px & 3] / 16.0 - 0.5) * (3.0 / 255.0);

            let to_u8 = |c: f32| ((c * shade + d).clamp(0.0, 1.0) * 255.0) as u8;
            pixels[py * MAP_TEX + px] =
                egui::Color32::from_rgb(to_u8(base[0]), to_u8(base[1]), to_u8(base[2]));
        }
    }
    egui::ColorImage {
        size: [MAP_TEX, MAP_TEX],
        source_size: egui::Vec2::new(MAP_TEX as f32, MAP_TEX as f32),
        pixels,
    }
}

// ---------------------------------------------------------------------------
// World → map UV helpers (match the 3D coordinate conventions)
// ---------------------------------------------------------------------------

/// Agent normalised XZ in 0..1 (sim coord / FIXED_SCALE, as in sim_bridge/minimap).
fn agent_norm_xz(position: &civ_agents::Position3d) -> egui::Vec2 {
    let scale = civ_voxel::FIXED_SCALE as f32;
    egui::vec2(
        (position.coord.x as f32 / scale).clamp(0.0, 1.0),
        (position.coord.z as f32 / scale).clamp(0.0, 1.0),
    )
}

/// Building centred-grid position ([-64,63]) → normalised 0..1 XZ.
fn building_norm_xz(building: &Building) -> egui::Vec2 {
    egui::vec2(
        ((building.position.x + 64) as f32 / 127.0).clamp(0.0, 1.0),
        ((building.position.y + 64) as f32 / 127.0).clamp(0.0, 1.0),
    )
}

/// Deterministic cluster-tint for a faction id (stable hue ramp).
fn faction_tint(faction: u32) -> egui::Color32 {
    let hue = (faction as f32 * 0.137).fract();
    let [r, g, b] = hsv_to_rgb(hue, 0.62, 0.95);
    egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn building_color(bt: BuildingType) -> egui::Color32 {
    let [r, g, b] = match bt {
        BuildingType::Farm => [0.55, 0.75, 0.35],
        BuildingType::Mine => [0.52, 0.48, 0.42],
        BuildingType::Barracks => [0.78, 0.32, 0.32],
        BuildingType::Temple => [0.74, 0.62, 0.92],
        BuildingType::Market => [0.92, 0.70, 0.26],
        BuildingType::House => [0.82, 0.62, 0.42],
        BuildingType::CityCenter => [0.40, 0.60, 0.90],
    };
    egui::Color32::from_rgb((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let i = (h * 6.0).floor();
    let f = h * 6.0 - i;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match (i as i32).rem_euclid(6) {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

// ---------------------------------------------------------------------------
// Draw
// ---------------------------------------------------------------------------

/// HUD hint shown in 3D mode (`M`: 2D map). Hidden while the map is up.
fn draw_map_hint(mut contexts: EguiContexts, view: Res<MapView>) {
    if view.fade > 0.02 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    egui::Area::new(egui::Id::new("map2d_hint"))
        .anchor(egui::Align2::LEFT_BOTTOM, egui::vec2(12.0, -12.0))
        .interactable(false)
        .show(ctx, |ui| {
            let frame = egui::Frame::NONE
                .fill(egui::Color32::from_rgba_unmultiplied(10, 14, 20, 180))
                .inner_margin(egui::Margin::symmetric(8, 5))
                .corner_radius(6.0);
            frame.show(ui, |ui| {
                ui.label(
                    egui::RichText::new("M  ·  2D map view")
                        .color(egui::Color32::from_rgb(190, 205, 220))
                        .size(13.0),
                );
            });
        });
}

/// Full-screen 2D map: procedural relief basemap + live vector markers.
fn draw_map_view(
    mut contexts: EguiContexts,
    mut view: ResMut<MapView>,
    mut basemap: ResMut<MapBasemap>,
    attach: Res<AttachMode>,
    sim: Option<Res<SimState>>,
) {
    if view.fade <= 0.01 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    // Lazily rasterise the basemap on first display.
    if basemap.handle.is_none() {
        let image = build_basemap_image();
        basemap.handle = Some(ctx.load_texture("map2d_basemap", image, egui::TextureOptions::LINEAR));
    }
    let tex = basemap.handle.as_ref().unwrap();
    let fade = view.fade;

    egui::Area::new(egui::Id::new("map2d_root"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let painter = ui.painter();

            // Vignette backdrop (darkens the page behind the map; eases the fade).
            painter.rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_unmultiplied(6, 9, 14, (fade * 255.0) as u8),
            );

            // Fit a square map into the screen, centred, with pan + zoom.
            let side = screen.height().min(screen.width()) * 0.96 * view.zoom;
            let centre = screen.center() + view.pan;
            let map_rect = egui::Rect::from_center_size(centre, egui::vec2(side, side));

            let tint = egui::Color32::from_white_alpha((fade * 255.0) as u8);
            painter.image(
                tex.id(),
                map_rect,
                egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                tint,
            );
            // Crisp framing border.
            painter.rect_stroke(
                map_rect,
                4.0,
                egui::Stroke::new(1.5, egui::Color32::from_rgba_unmultiplied(180, 200, 220, (fade * 160.0) as u8)),
                egui::StrokeKind::Outside,
            );

            // norm(0..1) → screen point inside map_rect (v flips to match minimap).
            let to_screen = |n: egui::Vec2| -> egui::Pos2 {
                egui::pos2(
                    map_rect.min.x + n.x * side,
                    map_rect.min.y + (1.0 - n.y) * side,
                )
            };

            // --- Live overlay: buildings then agents (read from SimState) ---
            if *attach != AttachMode::Server {
                if let Some(sim) = sim.as_ref() {
                    draw_buildings(painter, &sim.0, &to_screen, fade);
                    draw_agents(painter, &sim.0, &to_screen, fade);
                }
            }

            // Title + legend ribbon.
            draw_title(painter, screen, fade);

            // --- Interaction: pan (drag) + zoom (scroll) within the map ---
            let resp = ui.interact(map_rect, egui::Id::new("map2d_drag"), egui::Sense::click_and_drag());
            if resp.dragged() {
                view.pan += resp.drag_delta();
            }
            let scroll = ctx.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 && resp.hovered() {
                view.zoom = (view.zoom * (1.0 + scroll * 0.0015)).clamp(0.5, 6.0);
            }
        });
}

fn draw_buildings(
    painter: &egui::Painter,
    sim: &civ_engine::Simulation,
    to_screen: &dyn Fn(egui::Vec2) -> egui::Pos2,
    fade: f32,
) {
    for (_, building) in sim.world.query::<&Building>().iter() {
        let p = to_screen(building_norm_xz(building));
        let col = with_alpha(building_color(building.building_type), fade);
        // Crisp little "house" glyph: filled diamond roof over a square base.
        let r = 4.0;
        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(p.x, p.y - r * 1.3),
                egui::pos2(p.x + r, p.y - r * 0.2),
                egui::pos2(p.x - r, p.y - r * 0.2),
            ],
            col,
            egui::Stroke::new(0.6, with_alpha(egui::Color32::BLACK, fade * 0.6)),
        ));
        painter.rect_filled(
            egui::Rect::from_center_size(egui::pos2(p.x, p.y + r * 0.4), egui::vec2(r * 1.4, r * 1.2)),
            1.0,
            col,
        );
    }
}

fn draw_agents(
    painter: &egui::Painter,
    sim: &civ_engine::Simulation,
    to_screen: &dyn Fn(egui::Vec2) -> egui::Pos2,
    fade: f32,
) {
    for (_, (civ, pos)) in sim.world.query::<(&Civilian, &civ_agents::Position3d)>().iter() {
        let p = to_screen(agent_norm_xz(pos));
        let tint = with_alpha(faction_tint(civ.faction), fade);
        // Soft halo + crisp core for a clean, anti-aliased marker.
        painter.circle_filled(p, 3.2, with_alpha(tint, fade * 0.30));
        painter.circle(
            p,
            1.8,
            tint,
            egui::Stroke::new(0.8, with_alpha(egui::Color32::from_rgb(18, 22, 28), fade)),
        );
    }
}

fn draw_title(painter: &egui::Painter, screen: egui::Rect, fade: f32) {
    let pos = egui::pos2(screen.center().x, screen.top() + 26.0);
    painter.text(
        pos,
        egui::Align2::CENTER_CENTER,
        "CIVIS · WORLD MAP",
        egui::FontId::proportional(20.0),
        with_alpha(egui::Color32::from_rgb(220, 230, 240), fade),
    );
    painter.text(
        egui::pos2(pos.x, pos.y + 18.0),
        egui::Align2::CENTER_CENTER,
        "drag to pan · scroll to zoom · M to exit",
        egui::FontId::proportional(11.0),
        with_alpha(egui::Color32::from_rgb(150, 165, 185), fade),
    );
}

fn with_alpha(c: egui::Color32, fade: f32) -> egui::Color32 {
    let a = (c.a() as f32 * fade.clamp(0.0, 1.0)) as u8;
    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auto_engage_band_is_hysteretic() {
        assert!(AUTO_ENGAGE_DISTANCE > AUTO_DISENGAGE_DISTANCE);
        assert!(AUTO_ENGAGE_DISTANCE <= 600.0, "must be within MAX_DISTANCE");
    }

    #[test]
    fn palette_water_is_blue_land_is_not() {
        let deep = map_palette(WATER_LEVEL - 0.2 * HEIGHT_SCALE);
        assert!(deep[2] > deep[0], "deep ocean should be blue-dominant");
        let land = map_palette(WATER_LEVEL + 0.3 * HEIGHT_SCALE);
        assert!(land[1] >= land[2], "lowland should not be blue-dominant");
    }

    #[test]
    fn basemap_image_is_full_resolution() {
        let img = build_basemap_image();
        assert_eq!(img.size, [MAP_TEX, MAP_TEX]);
        assert_eq!(img.pixels.len(), MAP_TEX * MAP_TEX);
        // Not a flat fill: relief + palette must produce variety.
        let first = img.pixels[0];
        assert!(img.pixels.iter().any(|p| *p != first));
    }

    #[test]
    fn faction_tint_is_deterministic_and_distinct() {
        assert_eq!(faction_tint(1), faction_tint(1));
        assert_ne!(faction_tint(1), faction_tint(2));
    }

    #[test]
    fn building_norm_xz_centres_origin_grid() {
        // Grid (0,0) maps near the middle of the 0..1 map.
        let b_pos = building_norm_xz_from(0, 0);
        assert!((b_pos.x - 0.5).abs() < 0.02);
        assert!((b_pos.y - 0.5).abs() < 0.02);
    }

    fn building_norm_xz_from(x: i32, y: i32) -> egui::Vec2 {
        egui::vec2(
            ((x + 64) as f32 / 127.0).clamp(0.0, 1.0),
            ((y + 64) as f32 / 127.0).clamp(0.0, 1.0),
        )
    }
}
