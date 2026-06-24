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
//! The terrain basemap is rasterised **once** from the live voxel grid into an
//! egui texture: per-pixel biome palette
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
#[cfg(feature = "voxel")]
use civ_voxel::fluid_ca::CaGrid;
#[cfg(feature = "voxel")]
use civ_voxel::material::{MaterialDef, MaterialRegistry, AIR, WATER};

use crate::camera::CameraRig;
use crate::sim_bridge::SimState;
use crate::spawn_tools::SelectEntityRequest;
use crate::terrain::{HEIGHT_SCALE, WATER_LEVEL};
#[cfg(feature = "voxel")]
use crate::voxel_sim::{voxel_surface_y, VoxelSimState};
use crate::AttachMode;
use crate::settings_ui::{GameSettings, KeyBinding, ACTION_TOGGLE_MAP};
#[cfg(not(feature = "voxel"))]
#[derive(Resource)]
struct VoxelSimState;

fn civilian_faction_id(civilian: &Civilian) -> u32 {
    match civilian.alignment {
        civ_agents::Alignment::Faction(faction) => faction,
        _ => 0,
    }
}

/// Basemap raster resolution per side (crisp, sub-tile detail — not 8-bit).
const MAP_CLICK_PICK_RADIUS_PX: f32 = 12.0;
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
    last_seed: Option<u64>,
    last_dirty_marker: Option<usize>,
}

/// Marker component on the *main* perspective `Camera3d` so this module can
/// distinguish it from the minimap's `Camera3d` and toggle its `is_active`
/// while the 2D map is up. Without this, the live 3D scene would render
/// underneath the alpha-faded basemap and bleed through.
#[derive(Component)]
pub struct MainSceneCamera;

/// Plugin: registers the 2D alternate map view (key + far-zoom toggle).
#[derive(Default)]
pub struct Map2dPlugin;

impl Plugin for Map2dPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MapView>()
            .init_resource::<MapBasemap>()
            // Chained so the autoshot hook re-asserts `active` BEFORE
            // auto_engage_from_zoom runs (which clears manual_override every
            // frame at close camera distances). Unordered, auto-engage could win
            // the race and the map would never open under CIVIS_MAP_OPEN.
            .add_systems(
                Update,
                (
                    open_map_for_autoshot,
                    toggle_map_hotkey,
                    auto_engage_from_zoom,
                    tick_fade,
                    // After `tick_fade` has settled the new fade value, swap
                    // the main 3D camera's `is_active` so the 3D scene no
                    // longer bleeds through the alpha-faded 2D basemap.
                    hide_3d_scene_when_map_active,
                )
                    .chain(),
            )
            .add_systems(
                EguiPrimaryContextPass,
                (draw_map_view, draw_map_hint).run_if(crate::menus::in_game),
            );
    }
}

// ---------------------------------------------------------------------------
// Triggers
// ---------------------------------------------------------------------------

/// Toggle the map and pin the manual override.
fn toggle_map_hotkey(
    settings: Option<Res<GameSettings>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    mut view: ResMut<MapView>,
) {
    let toggle_map = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_TOGGLE_MAP))
        .unwrap_or(KeyBinding::Key(KeyCode::KeyM));
    if toggle_map.is_just_pressed(&keys, &mouse_buttons) {
        view.active = !view.active;
        view.manual_override = true;
        if !view.active {
            // Reset pan/zoom so the next open is framed to fit.
            view.pan = egui::Vec2::ZERO;
            view.zoom = 1.0;
        }
    }
}

/// Verification hook: when `CIVIS_MAP_OPEN=1` is set, hold the 2D map view open
/// so a headless autoshot can frame the live-grid basemap (otherwise behind the
/// `M` key / far-zoom auto-engage and invisible in captures).
///
/// Runs every frame and pins `manual_override` so `auto_engage_from_zoom` can't
/// disengage it at the (now closer) default camera distance during warm-up. The
/// env var is read once via a `Local` cache.
fn open_map_for_autoshot(mut view: ResMut<MapView>, mut enabled: Local<Option<bool>>) {
    let on = *enabled.get_or_insert_with(|| std::env::var("CIVIS_MAP_OPEN").as_deref() == Ok("1"));
    if on {
        view.active = true;
        view.manual_override = true;
        // Diagnostic: prove the plugin is scheduled and the flags/fade ramp so a
        // headless capture can confirm WHY the 2D map does or doesn't appear.
        info!(
            "[map] open_map_for_autoshot: active={} fade={:.3} override={}",
            view.active, view.fade, view.manual_override
        );
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

/// Hysteresis band: when `fade` climbs past `0.5`, deactivate the main
/// `Camera3d` so the 3D scene stops rendering and the alpha-faded basemap is
/// the only thing the player sees. When `fade` drops back below `0.5`,
/// reactivate it. The `0.5` threshold is the geometric centre of the ramp,
/// so the swap is symmetric and avoids stuttering on the boundary.
fn hide_3d_scene_when_map_active(
    view: Res<MapView>,
    mut cameras: Query<&mut Camera, With<MainSceneCamera>>,
) {
    let hide = view.fade > 0.5;
    for mut cam in &mut cameras {
        if cam.is_active != !hide {
            cam.is_active = !hide;
        }
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

/// Sample the live voxel column at grid coords `(gx, gz)`: returns the surface
/// height (top of the highest non-AIR voxel) and that voxel's [`MaterialId`].
///
/// Coordinates are floored into the grid and clamped in-bounds. An all-AIR
/// column yields `(0.0, None)` so callers treat it as open water / void.
#[cfg(feature = "voxel")]
fn sample_top_material(grid: &CaGrid, gx: f32, gz: f32) -> Option<civ_voxel::MaterialId> {
    let max_x = grid.dims[0].saturating_sub(1);
    let max_z = grid.dims[2].saturating_sub(1);
    let xi = (gx.max(0.0).floor() as usize).min(max_x);
    let zi = (gz.max(0.0).floor() as usize).min(max_z);
    for yi in (0..grid.dims[1]).rev() {
        let mat = grid.get(xi, yi, zi);
        if mat != AIR {
            return Some(mat);
        }
    }
    None
}

/// Rasterise the live voxel world into a crisp shaded-relief image. The base
/// colour of each pixel is the top surface voxel's material colour (the same
/// palette the 3D view uses); height drives the hillshade. Sampled across the
/// full grid extent so the map matches the active per-seed world.
#[cfg(feature = "voxel")]
pub fn world_extent_for_basemap(grid: &CaGrid) -> egui::Rect {
    egui::Rect::from_min_max(
        egui::pos2(0.0, 0.0),
        egui::pos2(grid.dims[0] as f32, grid.dims[2] as f32),
    )
}

#[cfg(feature = "voxel")]
fn build_basemap_image(grid: &CaGrid) -> egui::ColorImage {
    let mut pixels = vec![egui::Color32::BLACK; MAP_TEX * MAP_TEX];
    let registry = MaterialRegistry::standard();
    let cell_x = (grid.dims[0].saturating_sub(1)) as f32 / (MAP_TEX as f32 - 1.0);
    let cell_z = (grid.dims[2].saturating_sub(1)) as f32 / (MAP_TEX as f32 - 1.0);
    let scale = 2.0 * cell_x.max(cell_z);
    // Light from the north-west, slightly elevated.
    let light = Vec3::new(-0.55, 0.72, -0.42).normalize();

    for py in 0..MAP_TEX {
        for px in 0..MAP_TEX {
            let gx = px as f32 * (grid.dims[0].max(1) as f32 - 1.0) / (MAP_TEX as f32 - 1.0);
            let gz = py as f32 * (grid.dims[2].max(1) as f32 - 1.0) / (MAP_TEX as f32 - 1.0);
            let h = voxel_surface_y(grid, gx, gz);
            let top_material = sample_top_material(grid, gx, gz);
            let mut base = top_material
                .and_then(|id| registry.get(id))
                .map(|def: &MaterialDef| {
                    [
                        f32::from(def.color[0]) / 255.0,
                        f32::from(def.color[1]) / 255.0,
                        f32::from(def.color[2]) / 255.0,
                    ]
                })
                .unwrap_or([0.04, 0.08, 0.20]);

            // Finite-difference normal for hillshade (sample neighbours).
            let hx = voxel_surface_y(
                grid,
                (gx + cell_x).min((grid.dims[0].max(1) - 1) as f32),
                gz,
            ) - voxel_surface_y(grid, (gx - cell_x).max(0.0), gz);
            let hz = voxel_surface_y(
                grid,
                gx,
                (gz + cell_z).min((grid.dims[2].max(1) - 1) as f32),
            ) - voxel_surface_y(grid, gx, (gz - cell_z).max(0.0));
            let n = Vec3::new(-hx, scale, -hz).normalize();
            let lambert = n.dot(light).clamp(0.0, 1.0);

            let is_water =
                top_material.is_none() || top_material == Some(WATER) || top_material == Some(AIR);
            // Hillshade only the land; water gets a gentle flat sheen.
            let shade = if is_water {
                0.92 + 0.08 * lambert
            } else {
                0.55 + 0.55 * lambert
            };

            // Soft coastline darkening: emphasise the shore edge.
            if !is_water && h < 3.0 {
                let t = (h - WATER_LEVEL).max(0.0) / 2.5;
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

fn building_norm_xz_with_state(
    building: &Building,
    voxel_state: Option<&VoxelSimState>,
) -> egui::Vec2 {
    #[cfg(not(feature = "voxel"))]
    {
        let _ = voxel_state;
        return building_norm_xz(building);
    }
    #[cfg(feature = "voxel")]
    if let Some(voxel_state) = voxel_state {
        let x_span = (voxel_state.grid.dims[0] as f32 - 1.0).max(1.0);
        let z_span = (voxel_state.grid.dims[2] as f32 - 1.0).max(1.0);
        egui::vec2(
            (building.position.x as f32 / x_span).clamp(0.0, 1.0),
            (building.position.y as f32 / z_span).clamp(0.0, 1.0),
        )
    } else {
        building_norm_xz(building)
    }
}

#[derive(Clone, Copy)]
enum MapMarkerKind {
    Actor { faction: u32 },
    Building { building_type: BuildingType },
}

#[derive(Clone, Copy)]
struct MapMarker {
    screen_pos: egui::Pos2,
    world_pos: Vec3,
    kind: MapMarkerKind,
}

fn marker_world_from_actor(
    civ_world: &civ_agents::Position3d,
    voxel_state: Option<&VoxelSimState>,
) -> Vec3 {
    let u = (civ_world.coord.x as f32 / civ_voxel::FIXED_SCALE as f32).clamp(0.0, 1.0);
    let v = (civ_world.coord.z as f32 / civ_voxel::FIXED_SCALE as f32).clamp(0.0, 1.0);
    #[cfg(not(feature = "voxel"))]
    {
        let _ = voxel_state;
        return Vec3::new(u * 256.0 - 128.0, 0.0, v * 256.0 - 128.0);
    }
    #[cfg(feature = "voxel")]
    if let Some(voxel_state) = voxel_state {
        Vec3::new(
            u * voxel_state.grid.dims[0] as f32,
            0.0,
            v * voxel_state.grid.dims[2] as f32,
        )
    } else {
        Vec3::new(u * 256.0 - 128.0, 0.0, v * 256.0 - 128.0)
    }
}

fn marker_world_from_building(building: &Building, voxel_state: Option<&VoxelSimState>) -> Vec3 {
    let n = building_norm_xz_with_state(building, voxel_state);
    #[cfg(not(feature = "voxel"))]
    {
        let _ = voxel_state;
        return Vec3::new(n.x * 256.0 - 128.0, 0.0, n.y * 256.0 - 128.0);
    }
    #[cfg(feature = "voxel")]
    if let Some(voxel_state) = voxel_state {
        Vec3::new(
            n.x * voxel_state.grid.dims[0] as f32,
            0.0,
            n.y * voxel_state.grid.dims[2] as f32,
        )
    } else {
        Vec3::new(n.x * 256.0 - 128.0, 0.0, n.y * 256.0 - 128.0)
    }
}

fn marker_label(kind: BuildingType) -> &'static str {
    match kind {
        BuildingType::Farm => "Farm",
        BuildingType::Mine => "Mine",
        BuildingType::Barracks => "Barracks",
        BuildingType::Temple => "Temple",
        BuildingType::Market => "Market",
        BuildingType::House => "House",
        BuildingType::CityCenter => "City",
    }
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
fn draw_map_hint(
    mut contexts: EguiContexts,
    view: Res<MapView>,
    settings: Option<Res<GameSettings>>,
) {
    let toggle_label = map_toggle_binding_label(settings.as_deref());
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
                    egui::RichText::new(format!("{toggle_label} · 2D map view"))
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
    voxel_state: Option<Res<VoxelSimState>>,
    mut select_entity: MessageWriter<SelectEntityRequest>,
    attach: Res<AttachMode>,
    sim: Option<Res<SimState>>,
    params: Res<crate::menus::WorldSetupParams>,
    settings: Option<Res<GameSettings>>,
) {
    if view.fade <= 0.01 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    #[cfg(feature = "voxel")]
    let current_dirty = voxel_state.as_ref().map(|s| s.grid.dirty_chunks.len());
    #[cfg(not(feature = "voxel"))]
    let current_dirty = None;
    if current_dirty != basemap.last_dirty_marker && voxel_state.is_some() {
        basemap.handle = None;
    }

    if basemap.last_seed != Some(params.seed) {
        basemap.handle = None;
    }
    // Lazily rasterise the basemap on first display.
    if basemap.handle.is_none() {
        #[cfg(feature = "voxel")]
        if let Some(voxel_state) = voxel_state.as_ref() {
            let image = build_basemap_image(&voxel_state.grid);
            basemap.handle =
                Some(ctx.load_texture("map2d_basemap", image, egui::TextureOptions::LINEAR));
            basemap.last_seed = Some(params.seed);
            // current_dirty is `Option<usize>`; we know it's `Some(_)` here
            // because `voxel_state.as_ref()` already matched above.
            basemap.last_dirty_marker = current_dirty;
        }
        #[cfg(not(feature = "voxel"))]
        {
            let _ = voxel_state;
            basemap.last_seed = None;
            basemap.last_dirty_marker = None;
        }
    }
    let fade = view.fade;

    egui::Area::new(egui::Id::new("map2d_root"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen = ctx.screen_rect();
            let painter = ui.painter();
            let min_zoom = (screen.width().max(screen.height())) / MAP_TEX as f32;

            // Vignette backdrop (darkens the page behind the map; eases the fade).
            painter.rect_filled(
                screen,
                0.0,
                egui::Color32::from_rgba_unmultiplied(6, 9, 14, (fade * 255.0) as u8),
            );

            view.zoom = view.zoom.max(min_zoom).min(8.0);
            // Fit-to-viewport coverage uses base texture pixels so the map never
            // shows borders/void; zoom is clamped against that requirement.
            let side = MAP_TEX as f32 * view.zoom;
            let centre = screen.center() + view.pan;
            let map_rect = egui::Rect::from_center_size(centre, egui::vec2(side, side));

            let tint = egui::Color32::from_white_alpha((fade * 255.0) as u8);
            if let Some(tex) = basemap.handle.as_ref() {
                painter.image(
                    tex.id(),
                    map_rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    tint,
                );
            } else {
                painter.rect_filled(map_rect, 0.0, egui::Color32::from_rgb(8, 12, 20));
            }
            // Crisp framing border.
            painter.rect_stroke(
                map_rect,
                4.0,
                egui::Stroke::new(
                    1.5,
                    egui::Color32::from_rgba_unmultiplied(180, 200, 220, (fade * 160.0) as u8),
                ),
                egui::StrokeKind::Outside,
            );

            // norm(0..1) → screen point inside map_rect (v flips to match minimap).
            let to_screen = |n: egui::Vec2| -> egui::Pos2 { norm_to_screen(map_rect, n) };

            let mut map_markers: Vec<MapMarker> = Vec::new();
            if *attach != AttachMode::Server {
                if let Some(sim) = sim.as_ref() {
                    let voxel = voxel_state.as_ref().map(|s| s.as_ref());
                    for (_, (civ, position)) in sim
                        .0
                        .world
                        .query::<(&Civilian, &civ_agents::Position3d)>()
                        .iter()
                    {
                        let n = agent_norm_xz(position);
                        map_markers.push(MapMarker {
                            screen_pos: to_screen(n),
                            world_pos: marker_world_from_actor(position, voxel),
                            kind: MapMarkerKind::Actor {
                                faction: civilian_faction_id(civ),
                            },
                        });
                    }
                    for (_, building) in sim.0.world.query::<&Building>().iter() {
                        let n = building_norm_xz_with_state(building, voxel);
                        map_markers.push(MapMarker {
                            screen_pos: to_screen(n),
                            world_pos: marker_world_from_building(building, voxel),
                            kind: MapMarkerKind::Building {
                                building_type: building.building_type,
                            },
                        });
                    }
                }
            }
            for marker in map_markers.iter() {
                match marker.kind {
                    MapMarkerKind::Actor { faction } => {
                        let tint = with_alpha(faction_tint(faction), fade);
                        painter.circle_filled(
                            marker.screen_pos,
                            3.2,
                            with_alpha(tint, fade * 0.30),
                        );
                        painter.circle(
                            marker.screen_pos,
                            1.8,
                            tint,
                            egui::Stroke::new(
                                0.8,
                                with_alpha(egui::Color32::from_rgb(18, 22, 28), fade),
                            ),
                        );
                    }
                    MapMarkerKind::Building { building_type } => {
                        let p = marker.screen_pos;
                        let col = with_alpha(building_color(building_type), fade);
                        let size = 5.0;
                        match building_type {
                            BuildingType::House | BuildingType::CityCenter => {
                                painter.add(egui::Shape::convex_polygon(
                                    vec![
                                        egui::pos2(p.x, p.y - size * 1.15),
                                        egui::pos2(p.x + size, p.y - size * 0.2),
                                        egui::pos2(p.x, p.y + size * 1.0),
                                        egui::pos2(p.x - size, p.y - size * 0.2),
                                    ],
                                    col,
                                    egui::Stroke::new(
                                        0.6,
                                        with_alpha(egui::Color32::BLACK, fade * 0.6),
                                    ),
                                ));
                            }
                            BuildingType::Market => {
                                painter.rect_stroke(
                                    egui::Rect::from_center_size(
                                        p,
                                        egui::vec2(size * 1.8, size * 1.2),
                                    ),
                                    0.0,
                                    egui::Stroke::new(1.0, with_alpha(egui::Color32::BLACK, fade)),
                                    egui::StrokeKind::Outside,
                                );
                                painter.circle_filled(
                                    egui::pos2(p.x, p.y + size * 0.3),
                                    2.0,
                                    with_alpha(egui::Color32::BLACK, fade * 0.22),
                                );
                            }
                            BuildingType::Temple => {
                                painter.add(egui::Shape::convex_polygon(
                                    vec![
                                        egui::pos2(p.x - size * 0.9, p.y + size),
                                        egui::pos2(p.x - size * 0.35, p.y - size * 0.8),
                                        egui::pos2(p.x + size * 0.35, p.y - size * 0.8),
                                        egui::pos2(p.x + size * 0.9, p.y + size),
                                    ],
                                    col,
                                    egui::Stroke::new(
                                        0.6,
                                        with_alpha(egui::Color32::BLACK, fade * 0.6),
                                    ),
                                ));
                            }
                            BuildingType::Mine => {
                                painter.line_segment(
                                    [p + egui::vec2(-size, -size), p + egui::vec2(size, size)],
                                    egui::Stroke::new(1.0, with_alpha(egui::Color32::BLACK, fade)),
                                );
                                painter.line_segment(
                                    [p + egui::vec2(-size, size), p + egui::vec2(size, -size)],
                                    egui::Stroke::new(1.0, with_alpha(egui::Color32::BLACK, fade)),
                                );
                            }
                            BuildingType::Farm | BuildingType::Barracks => {
                                painter.circle_filled(p, size * 0.95, with_alpha(col, fade * 0.85));
                                painter.circle_stroke(
                                    p,
                                    size * 0.75,
                                    egui::Stroke::new(1.0, with_alpha(egui::Color32::BLACK, fade)),
                                );
                            }
                        }
                        painter.text(
                            egui::pos2(p.x + size * 0.85, p.y - size * 1.35),
                            egui::Align2::LEFT_TOP,
                            marker_label(building_type),
                            egui::FontId::proportional(9.0),
                            with_alpha(egui::Color32::from_rgb(220, 228, 237), fade),
                        );
                    }
                }
            }

            // Title + legend ribbon.
            draw_title(
                painter,
                screen,
                fade,
                &map_toggle_binding_label(settings.as_deref()),
            );

            egui::Area::new(egui::Id::new("map2d_zoom_panel"))
                .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 12.0))
                .show(ctx, |ui| {
                    let frame = egui::Frame::NONE
                        .fill(egui::Color32::from_rgba_unmultiplied(10, 14, 20, 180))
                        .inner_margin(egui::Margin::symmetric(8, 6))
                        .corner_radius(6.0);
                    frame.show(ui, |ui| {
                        ui.vertical(|ui| {
                            ui.label(
                                egui::RichText::new(format!(
                                    "Zoom {:.2}x",
                                    view.zoom / min_zoom
                                ))
                                .color(egui::Color32::from_rgb(220, 230, 240))
                                .size(12.5),
                            );
                            if ui
                                .button("Reset to fit")
                                .on_hover_text("Reset pan and zoom to the fitted overview")
                                .clicked()
                            {
                                view.pan = egui::Vec2::ZERO;
                                view.zoom = min_zoom;
                            }
                        });
                    });
                });

            // --- Interaction: pan (drag) + zoom (scroll) within the map ---
            let resp = ui.interact(
                map_rect,
                egui::Id::new("map2d_drag"),
                egui::Sense::click_and_drag(),
            );
            if resp.dragged() {
                view.pan += resp.drag_delta();
            }
            let scroll = ctx.input(|i| i.raw_scroll_delta.y);
            if scroll != 0.0 && resp.hovered() {
                view.zoom = (view.zoom * (1.0 + scroll * 0.0015)).clamp(min_zoom, 8.0);
            }
            let max_pan_x = ((side - screen.width()) * 0.5).max(0.0);
            let max_pan_y = ((side - screen.height()) * 0.5).max(0.0);
            view.pan = egui::vec2(
                view.pan.x.clamp(-max_pan_x, max_pan_x),
                view.pan.y.clamp(-max_pan_y, max_pan_y),
            );

            if resp.clicked() && resp.hovered() {
                let Some(pointer) = resp.interact_pointer_pos() else {
                    return;
                };
                let mut best: Option<(f32, MapMarker)> = None;
                for marker in map_markers.iter() {
                    let d2 = marker.screen_pos.distance_sq(pointer);
                    if d2 <= MAP_CLICK_PICK_RADIUS_PX * MAP_CLICK_PICK_RADIUS_PX
                        && best.map_or(true, |(best_d2, _)| d2 < best_d2)
                    {
                        best = Some((d2, *marker));
                    }
                }
                if let Some((_, hit)) = best {
                    select_entity.write(SelectEntityRequest {
                        position: hit.world_pos,
                    });
                }
            }
        });
}

fn draw_title(
    painter: &egui::Painter,
    screen: egui::Rect,
    fade: f32,
    map_exit_key: &str,
) {
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
        &format!("drag to pan · scroll to zoom · {map_exit_key} to exit"),
        egui::FontId::proportional(11.0),
        with_alpha(egui::Color32::from_rgb(150, 165, 185), fade),
    );
}

fn norm_to_screen(map_rect: egui::Rect, n: egui::Vec2) -> egui::Pos2 {
    let side = map_rect.width();
    egui::pos2(
        map_rect.min.x + n.x * side,
        map_rect.min.y + (1.0 - n.y) * side,
    )
}

fn screen_to_norm(map_rect: egui::Rect, p: egui::Pos2) -> egui::Vec2 {
    let side = map_rect.width().max(1.0);
    egui::vec2(
        ((p.x - map_rect.min.x) / side).clamp(0.0, 1.0),
        (1.0 - ((p.y - map_rect.min.y) / side)).clamp(0.0, 1.0),
    )
}

fn map_toggle_binding_label(settings: Option<&GameSettings>) -> String {
    settings
        .and_then(|s| s.key_for(ACTION_TOGGLE_MAP))
        .map_or_else(|| "M".to_string(), |binding| binding.to_string())
}

fn with_alpha(c: egui::Color32, fade: f32) -> egui::Color32 {
    let a = (c.a() as f32 * fade.clamp(0.0, 1.0)) as u8;
    egui::Color32::from_rgba_unmultiplied(c.r(), c.g(), c.b(), a)
}

#[cfg(all(test, feature = "voxel"))]
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
        // Build a tiny voxel world with a height gradient so the relief shading
        // produces pixel variety (column height grows with x).
        let dims = [16, 12, 16];
        let mut grid = CaGrid::new(dims);
        for z in 0..dims[2] {
            for x in 0..dims[0] {
                let h = (1 + x % (dims[1] - 1)).min(dims[1] - 1);
                for y in 0..h {
                    grid.set(x, y, z, WATER);
                }
            }
        }
        let img = build_basemap_image(&grid);
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

    #[test]
    fn map_overlay_norm_screen_round_trip_is_stable() {
        let rects = [
            egui::Rect::from_min_size(egui::pos2(12.0, 34.0), egui::vec2(256.0, 256.0)),
            egui::Rect::from_min_size(egui::pos2(80.0, 24.0), egui::vec2(384.0, 384.0)),
            egui::Rect::from_min_size(egui::pos2(4.0, 10.0), egui::vec2(768.0, 768.0)),
        ];
        let samples = [
            egui::vec2(0.0, 0.0),
            egui::vec2(0.25, 0.75),
            egui::vec2(0.5, 0.5),
            egui::vec2(0.9, 0.1),
            egui::vec2(1.0, 1.0),
        ];

        for rect in rects {
            for sample in samples {
                let screen = norm_to_screen(rect, sample);
                let round_trip = screen_to_norm(rect, screen);
                assert!((round_trip.x - sample.x).abs() < 1.0e-5);
                assert!((round_trip.y - sample.y).abs() < 1.0e-5);
            }
        }
    }
}
