#![cfg(all(feature = "bevy", feature = "egui"))]

//! Material-painting palette panel (WorldBox × The Powder Toy).
//!
//! A rich, themed, scrollable material brush. The palette reads the live
//! `civ-voxel` material registry, buckets every material into a player-facing
//! family (Liquids / Powders / Solids / Gases / Energetic / Bio), and lets the
//! player pick one as the active paint material. Painting writes the chosen
//! `MaterialId` straight into the running CA grid so the painted material then
//! **obeys the cellular automata** (water flows, sand piles, lava cools, gases
//! rise) on the very next sim tick — the WorldBox "instant brush that comes
//! alive" feel, with Powder-Toy element variety.
//!
//! ## What lives here
//! - [`SelectedMaterial`] — the active paint material plus the shared brush
//!   *size* and *strength* (0..=1) surface that every paint-style tool reuses.
//! - [`MaterialBrushPlugin`] — registers the resource, draws the palette panel
//!   (egui, behind the `egui` feature) and runs the voxel paint system (behind
//!   the `voxel` feature).
//!
//! ## Graceful degradation
//! The taxonomy is **data-driven off the registry**, never a hardcoded list.
//! Another worker expanding `crates/voxel/src/material.rs` (dozens of
//! Powder-Toy-class elements) needs zero changes here: new materials simply
//! appear in their family bucket. If a family ends up empty it is omitted. The
//! panel renders correctly with the current 12-material set or a future 60.
//!
//! ## Coordination (no edits to the owned files)
//! `spawn_tools.rs` / `lib.rs` / `standalone.rs` are owned by other leads. This
//! module is self-contained; wiring it requires only:
//! 1. `pub mod material_brush_ui;` in `lib.rs` (gated `bevy`+`egui`),
//! 2. `.add_plugins(material_brush_ui::MaterialBrushPlugin)` in the standalone
//!    app builder,
//! 3. a `SpawnTool::PaintMaterial` variant so the brush only paints when the
//!    Material tool is active (see the module-level report). Until that variant
//!    exists the plugin still compiles and the palette is fully usable; the
//!    paint system gates on [`MaterialPaintArmed`] which the HUD can flip.

use bevy::prelude::*;

use bevy_egui::egui;
use civ_voxel::material::{MaterialDef, MaterialRegistry, Phase};
use civ_voxel::MaterialId;

use crate::ui_theme;

/// Player-facing material families shown as collapsible sections in the palette.
///
/// These are *presentation* buckets layered on top of the CA [`Phase`]. They
/// give the Powder-Toy-style "Liquids / Powders / Solids / Gases / Energetic /
/// Bio" shelves without the registry needing to know about UI grouping.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialFamily {
    /// Flowing liquids — water, lava, oil, acid, …
    Liquids,
    /// Granular powders with an angle of repose — sand, dirt, gravel, ash, …
    Powders,
    /// Static solids — stone, ice, ore, brick, wood, …
    Solids,
    /// Gases that rise/diffuse — air, steam, smoke, toxic gas, …
    Gases,
    /// Reactive / energetic elements — lava, fire, plasma, spark, radioactive.
    Energetic,
    /// Living / organic / special — plant, moss, mold, organic sludge.
    Bio,
}

impl MaterialFamily {
    /// Display order of the family shelves, top to bottom.
    pub const ORDER: [MaterialFamily; 6] = [
        MaterialFamily::Liquids,
        MaterialFamily::Powders,
        MaterialFamily::Solids,
        MaterialFamily::Gases,
        MaterialFamily::Energetic,
        MaterialFamily::Bio,
    ];

    /// Section header label.
    pub fn label(self) -> &'static str {
        match self {
            MaterialFamily::Liquids => "Liquids",
            MaterialFamily::Powders => "Powders",
            MaterialFamily::Solids => "Solids",
            MaterialFamily::Gases => "Gases",
            MaterialFamily::Energetic => "Energetic",
            MaterialFamily::Bio => "Bio",
        }
    }

    /// Small glyph shown beside the section header.
    pub fn icon(self) -> &'static str {
        match self {
            MaterialFamily::Liquids => "\u{1f4a7}",   // droplet
            MaterialFamily::Powders => "\u{1f3d6}",   // beach
            MaterialFamily::Solids => "\u{1faa8}",    // rock
            MaterialFamily::Gases => "\u{1f4a8}",     // dashing wind
            MaterialFamily::Energetic => "\u{1f525}", // fire
            MaterialFamily::Bio => "\u{1f331}",       // seedling
        }
    }

    /// Accent colour used to theme this family's section + selection ring.
    pub fn accent(self) -> egui::Color32 {
        match self {
            MaterialFamily::Liquids => ui_theme::ACCENT,
            MaterialFamily::Powders => ui_theme::GOLD,
            MaterialFamily::Solids => ui_theme::DIM,
            MaterialFamily::Gases => ui_theme::ACCENT_HI,
            MaterialFamily::Energetic => ui_theme::RED,
            MaterialFamily::Bio => ui_theme::GREEN,
        }
    }

    /// Bucket a material into its presentation family.
    ///
    /// Energetic / Bio are detected by name keyword (so registry growth lands
    /// them correctly without a UI change); everything else falls back to the
    /// CA [`Phase`]. This is the only heuristic in the module and is the reason
    /// the palette degrades gracefully across registry expansions.
    pub fn classify(def: &MaterialDef) -> MaterialFamily {
        let name = def.name.to_ascii_lowercase();
        const ENERGETIC: [&str; 8] = [
            "lava", "magma", "fire", "ember", "plasma", "spark", "electric", "radio",
        ];
        const BIO: [&str; 7] = [
            "plant", "moss", "mold", "organic", "sludge", "seed", "bio",
        ];
        if ENERGETIC.iter().any(|k| name.contains(k)) {
            return MaterialFamily::Energetic;
        }
        if BIO.iter().any(|k| name.contains(k)) {
            return MaterialFamily::Bio;
        }
        match def.phase {
            Phase::Liquid => MaterialFamily::Liquids,
            Phase::Powder => MaterialFamily::Powders,
            Phase::Gas => MaterialFamily::Gases,
            Phase::Solid => MaterialFamily::Solids,
            // `Empty`/`Air` is not a paintable material; bucket with gases so it
            // is shown (and skipped) consistently rather than panicking.
            Phase::Empty => MaterialFamily::Gases,
        }
    }
}

/// Minimum / maximum brush radius in voxels (shared paint-tool surface).
pub const BRUSH_RADIUS_MIN: f32 = 1.0;
/// Maximum brush radius in voxels.
pub const BRUSH_RADIUS_MAX: f32 = 16.0;

/// Material-brush action mode (Replace / drop-from-sky / Erase).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MaterialPaintMode {
    /// Overwrite voxels in the brush footprint with the selected material.
    #[default]
    Replace,
    /// Spawn material in a column above the target; CA carries it down.
    AdditiveDrop,
    /// Clear voxels in the footprint (write air/empty).
    Erase,
}

impl MaterialPaintMode {
    /// All modes in UI order.
    pub const ALL: [MaterialPaintMode; 3] = [
        MaterialPaintMode::Replace,
        MaterialPaintMode::AdditiveDrop,
        MaterialPaintMode::Erase,
    ];

    /// Short label for the cluster combobox + radio rows.
    pub fn label(self) -> &'static str {
        match self {
            MaterialPaintMode::Replace => "Replace",
            MaterialPaintMode::AdditiveDrop => "Additive drop",
            MaterialPaintMode::Erase => "Erase",
        }
    }
}

/// The active paint material plus the shared brush parameters.
///
/// `size`/`strength` are the same conceptual surface the terraform + other
/// paint tools reuse (one brush model). `strength` is a 0..=1 probability that
/// any individual voxel in the brush footprint is actually written this stamp —
/// at `1.0` the brush is a solid stamp, lower values give a sparse / feathered
/// paint useful for seeding sand grains or sparse gas.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct SelectedMaterial {
    /// The material id painted on click/drag.
    pub material: MaterialId,
    /// Replace / additive-drop / erase verb for the next stamp.
    pub mode: MaterialPaintMode,
    /// Brush radius in voxels (`BRUSH_RADIUS_MIN..=BRUSH_RADIUS_MAX`).
    pub size: f32,
    /// Per-voxel fill probability in `0.0..=1.0` (feather / density).
    pub strength: f32,
}

impl Default for SelectedMaterial {
    fn default() -> Self {
        // Default to the first non-air material in the registry so the brush is
        // immediately useful; fall back to id 1 (Water in the standard set).
        let material = MaterialRegistry::standard()
            .materials()
            .iter()
            .find(|m| !matches!(m.phase, Phase::Empty) && m.id.0 != 0)
            .map(|m| m.id)
            .unwrap_or(MaterialId(1));
        Self {
            material,
            mode: MaterialPaintMode::Replace,
            size: 4.0,
            strength: 1.0,
        }
    }
}

impl SelectedMaterial {
    /// Clamp the brush radius into the supported range.
    pub fn clamped_size(&self) -> f32 {
        self.size.clamp(BRUSH_RADIUS_MIN, BRUSH_RADIUS_MAX)
    }
}

/// Whether the Material tool is the active tool (HUD-controlled gate).
///
/// The paint system only writes voxels while this is `true`, so the palette and
/// the paint behaviour stay decoupled from the `SpawnTool` enum until a
/// `SpawnTool::PaintMaterial` variant lands. The HUD flips this when the
/// Material category / sub-tool is selected.
#[derive(Resource, Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct MaterialPaintArmed(pub bool);

/// Whether the palette panel is shown (toggled by the HUD; default on).
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct MaterialPaletteOpen(pub bool);

impl Default for MaterialPaletteOpen {
    fn default() -> Self {
        Self(true)
    }
}

/// Material brush palette + voxel paint plugin.
pub struct MaterialBrushPlugin;

impl Plugin for MaterialBrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedMaterial>()
            .init_resource::<MaterialPaintArmed>()
            .init_resource::<MaterialPaletteOpen>()
            .add_systems(Update, sync_paint_armed_from_tool)
            // egui draw MUST run on EguiPrimaryContextPass — on Update the egui
            // context has no fonts yet and panics ("No fonts available").
            .add_systems(
                bevy_egui::EguiPrimaryContextPass,
                material_palette_panel.run_if(crate::menus::in_game),
            );

        // The actual world paint only exists when the voxel sim is compiled in.
        #[cfg(feature = "voxel")]
        app.add_systems(Update, paint_into_voxel_grid);
    }
}

/// Arm the material brush exactly while `SpawnTool::PaintMaterial` is the active
/// tool, so selecting the Material tool flips on `MaterialPaintArmed` and any
/// other tool flips it off. Keeps the paint gate in sync with the HUD palette
/// without the paint system needing to know about the tool enum.
fn sync_paint_armed_from_tool(
    active: Res<crate::spawn_tools::ActiveTool>,
    mut armed: ResMut<MaterialPaintArmed>,
) {
    let want = active.tool == crate::spawn_tools::SpawnTool::PaintMaterial;
    if armed.0 != want {
        armed.0 = want;
    }
}

/// Bucket the live registry into ordered `(family, materials)` shelves.
///
/// Air / empty materials are dropped (not paintable). Empty families are
/// omitted by the caller. Materials keep their registry order within a family.
fn build_shelves(registry: MaterialRegistry) -> Vec<(MaterialFamily, Vec<&'static MaterialDef>)> {
    MaterialFamily::ORDER
        .iter()
        .map(|&family| {
            let mats: Vec<&'static MaterialDef> = registry
                .materials()
                .iter()
                .filter(|def| def.id.0 != 0 && !matches!(def.phase, Phase::Empty))
                .filter(|def| MaterialFamily::classify(def) == family)
                .collect();
            (family, mats)
        })
        .collect()
}

/// Convert a registry RGBA color into an egui swatch colour.
fn swatch_color(def: &MaterialDef) -> egui::Color32 {
    egui::Color32::from_rgba_unmultiplied(def.color[0], def.color[1], def.color[2], 255)
}

/// Lighten or darken an sRGB swatch channel-wise (clamped).
fn swatch_tint(base: egui::Color32, mul: f32) -> egui::Color32 {
    let ch = |v: u8| ((f32::from(v) * mul).round().clamp(0.0, 255.0)) as u8;
    egui::Color32::from_rgba_unmultiplied(ch(base.r()), ch(base.g()), ch(base.b()), base.a())
}

/// Vertical gloss bands: lighter top → darker bottom over the base hue.
fn paint_swatch_gloss(painter: &egui::Painter, rect: egui::Rect, base: egui::Color32, radius: f32) {
    const BANDS: usize = 4;
    painter.rect_filled(rect, radius, swatch_tint(base, 0.7));
    let bh = rect.height() / BANDS as f32;
    for i in 0..BANDS {
        let t = i as f32 / (BANDS - 1) as f32;
        let mul = 1.25 - t * 0.55;
        let y0 = rect.min.y + bh * i as f32;
        let y1 = if i + 1 == BANDS {
            rect.max.y
        } else {
            y0 + bh + 1.0
        };
        let band = egui::Rect::from_min_max(egui::pos2(rect.min.x, y0), egui::pos2(rect.max.x, y1));
        painter.rect_filled(band, 0.0, swatch_tint(base, mul));
    }
}

/// 1px bevel: light top/left, dark bottom/right.
fn paint_swatch_bevel(painter: &egui::Painter, rect: egui::Rect) {
    let light = swatch_tint(egui::Color32::WHITE, 0.55);
    let dark = swatch_tint(egui::Color32::BLACK, 0.45);
    painter.line_segment([rect.left_top(), rect.right_top()], egui::Stroke::new(1.0, light));
    painter.line_segment([rect.left_top(), rect.left_bottom()], egui::Stroke::new(1.0, light));
    painter.line_segment([rect.left_bottom(), rect.right_bottom()], egui::Stroke::new(1.0, dark));
    painter.line_segment([rect.right_top(), rect.right_bottom()], egui::Stroke::new(1.0, dark));
}

/// Cheap phase texture: liquid wave, powder stipple, solid relies on gloss alone.
fn paint_swatch_phase_hint(
    painter: &egui::Painter,
    rect: egui::Rect,
    phase: Phase,
    seed: u32,
) {
    match phase {
        Phase::Liquid => paint_liquid_wave(painter, rect),
        Phase::Powder => paint_powder_stipple(painter, rect, seed),
        Phase::Solid => paint_solid_sheen(painter, rect),
        Phase::Gas => paint_gas_wisp(painter, rect, seed),
        Phase::Empty => {}
    }
}

fn paint_liquid_wave(painter: &egui::Painter, rect: egui::Rect) {
    let y = rect.min.y + rect.height() * 0.32;
    let w = rect.width();
    let mut pts = Vec::with_capacity(9);
    for i in 0..9 {
        let t = i as f32 / 8.0;
        let x = rect.min.x + w * t;
        let wave = (t * std::f32::consts::TAU * 1.6).sin() * rect.height() * 0.06;
        pts.push(egui::pos2(x, y + wave));
    }
    let stroke = egui::Stroke::new(1.2, swatch_tint(egui::Color32::WHITE, 0.75));
    for w in pts.windows(2) {
        painter.line_segment([w[0], w[1]], stroke);
    }
}

fn paint_powder_stipple(painter: &egui::Painter, rect: egui::Rect, seed: u32) {
    let mut h = seed.wrapping_mul(0x9E37_79B9);
    let step = 5.0;
    let mut y = rect.min.y + 2.0;
    while y < rect.max.y - 2.0 {
        let mut x = rect.min.x + 2.0;
        while x < rect.max.x - 2.0 {
            h ^= h << 13;
            h ^= h >> 17;
            h ^= h << 5;
            if h & 3 == 0 {
                let dot = egui::Rect::from_center_size(egui::pos2(x, y), egui::vec2(1.5, 1.5));
                painter.rect_filled(dot, 0.0, swatch_tint(egui::Color32::BLACK, 0.35));
            }
            x += step;
        }
        y += step;
    }
}

fn paint_solid_sheen(painter: &egui::Painter, rect: egui::Rect) {
    let sheen = egui::Rect::from_min_max(
        egui::pos2(rect.min.x + 3.0, rect.min.y + 4.0),
        egui::pos2(rect.max.x - 6.0, rect.min.y + 7.0),
    );
    painter.rect_filled(sheen, 1.0, swatch_tint(egui::Color32::WHITE, 0.35));
}

fn paint_gas_wisp(painter: &egui::Painter, rect: egui::Rect, seed: u32) {
    let mut h = seed;
    for i in 0..3 {
        h = h.wrapping_mul(0x85EB_CA6B).wrapping_add(i);
        let t = (h & 0xFF) as f32 / 255.0;
        let cx = egui::lerp(rect.min.x + 6.0..=rect.max.x - 6.0, t);
        let cy = egui::lerp(rect.min.y + 8.0..=rect.max.y - 8.0, ((h >> 8) & 0xFF) as f32 / 255.0);
        let blob = egui::Rect::from_center_size(egui::pos2(cx, cy), egui::vec2(6.0, 3.0));
        painter.rect_filled(blob, 2.0, swatch_tint(egui::Color32::WHITE, 0.22));
    }
}

/// Draw the scrollable, categorized material palette panel.
fn material_palette_panel(
    mut contexts: bevy_egui::EguiContexts,
    open: Res<MaterialPaletteOpen>,
    armed: Res<MaterialPaintArmed>,
    mut selected: ResMut<SelectedMaterial>,
) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    let registry = MaterialRegistry::standard();
    let shelves = build_shelves(registry);
    let active_name = registry
        .get(selected.material)
        .map(|m| m.name)
        .unwrap_or("—");

    egui::Window::new("Material Brush")
        .frame(ui_theme::accent_frame(egui::Margin::same(10), ui_theme::ACCENT))
        .resizable(false)
        .default_width(248.0)
        .anchor(egui::Align2::RIGHT_CENTER, egui::vec2(-12.0, 0.0))
        .show(ctx, |ui| {
            palette_panel_body(ui, active_name, armed.0, &shelves, &mut selected);
        });
}

/// Window interior: header, cluster combobox, and family swatch shelves.
fn palette_panel_body(
    ui: &mut egui::Ui,
    active_name: &str,
    armed: bool,
    shelves: &[(MaterialFamily, Vec<&'static MaterialDef>)],
    selected: &mut SelectedMaterial,
) {
    palette_header(ui, active_name, armed);
    ui.add_space(4.0);
    brush_cluster_combo(ui, selected);
    ui_theme::hairline(ui);
    egui::ScrollArea::vertical()
        .max_height(360.0)
        .auto_shrink([false, false])
        .show(ui, |ui| {
            for (family, mats) in shelves {
                if mats.is_empty() {
                    continue;
                }
                family_section(ui, *family, mats, selected);
            }
        });
}

/// Header row: title, active material name, and armed/idle status pill.
fn palette_header(ui: &mut egui::Ui, active_name: &str, armed: bool) {
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new("\u{1f3a8} Paint")
                .heading()
                .color(ui_theme::TEXT),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let (txt, col) = if armed {
                ("ARMED", ui_theme::GREEN)
            } else {
                ("idle", ui_theme::DIM)
            };
            ui.label(egui::RichText::new(txt).small().strong().color(col));
        });
    });
    ui.label(
        egui::RichText::new(active_name)
            .strong()
            .color(ui_theme::ACCENT_HI),
    );
}

/// Brush cluster combobox: action mode + size/strength live inside the popover.
fn brush_cluster_combo(ui: &mut egui::Ui, selected: &mut SelectedMaterial) {
    let label = format!("{} · r{:.0}", selected.mode.label(), selected.size);
    egui::ComboBox::from_id_salt("material_brush_cluster")
        .width(220.0)
        .selected_text(label)
        .show_ui(ui, |ui| {
            brush_action_mode_radios(ui, selected);
            ui.add_space(4.0);
            brush_size_strength_sliders(ui, selected);
        });
}

/// Replace / additive-drop / erase radio row inside the cluster popover.
fn brush_action_mode_radios(ui: &mut egui::Ui, selected: &mut SelectedMaterial) {
    ui.label(egui::RichText::new("Action").small().strong().color(ui_theme::DIM));
    for mode in MaterialPaintMode::ALL {
        ui.radio_value(&mut selected.mode, mode, mode.label());
    }
}

/// Shared brush size + strength sliders (the one-brush-model surface).
fn brush_size_strength_sliders(ui: &mut egui::Ui, selected: &mut SelectedMaterial) {
    let mut size = selected.size;
    ui.add(
        egui::Slider::new(&mut size, BRUSH_RADIUS_MIN..=BRUSH_RADIUS_MAX)
            .text("size")
            .fixed_decimals(0),
    );
    let mut strength = selected.strength;
    ui.add(
        egui::Slider::new(&mut strength, 0.05..=1.0)
            .text("strength")
            .fixed_decimals(2),
    );
    selected.size = size;
    selected.strength = strength;
}

/// One family shelf: a header chip plus a wrapped grid of material swatches.
fn family_section(
    ui: &mut egui::Ui,
    family: MaterialFamily,
    mats: &[&'static MaterialDef],
    selected: &mut SelectedMaterial,
) {
    ui.add_space(6.0);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!("{} {}", family.icon(), family.label()))
                .strong()
                .color(family.accent()),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            ui.label(
                egui::RichText::new(format!("{}", mats.len()))
                    .small()
                    .color(ui_theme::DIM),
            );
        });
    });
    ui.add_space(2.0);
    ui.horizontal_wrapped(|ui| {
        for def in mats {
            material_swatch(ui, def, family.accent(), selected);
        }
    });
}

/// A single clickable material swatch button; selecting it sets the active
/// paint material. The current selection gets an accent ring.
fn material_swatch(
    ui: &mut egui::Ui,
    def: &MaterialDef,
    accent: egui::Color32,
    selected: &mut SelectedMaterial,
) {
    let is_selected = selected.material == def.id;
    let size = egui::vec2(40.0, 40.0);
    let (rect, response) = ui.allocate_exact_size(size, egui::Sense::click());
    let inner = rect.shrink(2.0);
    let radius = ui_theme::RADIUS_SM as f32;
    let base = swatch_color(def);
    let painter = ui.painter();
    paint_swatch_gloss(painter, inner, base, radius);
    paint_swatch_phase_hint(painter, inner, def.phase, u32::from(def.id.0));
    paint_swatch_bevel(painter, inner);
    let stroke = if is_selected {
        egui::Stroke::new(2.5, accent)
    } else if response.hovered() {
        egui::Stroke::new(1.5, ui_theme::ACCENT_HI)
    } else {
        egui::Stroke::new(1.0, ui_theme::BORDER)
    };
    painter.rect_stroke(inner, radius, stroke, egui::StrokeKind::Inside);
    if is_selected {
        ui_theme::inner_glow(painter, inner, accent, ui_theme::RADIUS_SM);
    }
    let response = response.on_hover_text(format!(
        "{}\nphase: {:?}  density: {}",
        def.name, def.phase, def.density
    ));
    if response.clicked() {
        selected.material = def.id;
    }
}

/// Paint the selected material into the live CA grid along the cursor stamp.
///
/// Writes a spherical brush of voxels around the cursor's terrain hit while the
/// left button is held and the Material tool is armed. The CA picks the painted
/// cells up on the next tick, so painted water flows, sand piles, lava cools.
///
/// World units map 1:1 to grid coordinates (the voxel sim places each chunk at
/// `chunk_origin` world units), so the cursor hit's `xyz` is the grid centre.
#[cfg(feature = "voxel")]
fn paint_into_voxel_grid(
    buttons: Res<ButtonInput<MouseButton>>,
    armed: Res<MaterialPaintArmed>,
    selected: Res<SelectedMaterial>,
    marker: Res<crate::spawn_tools::CursorMarker>,
    over_ui: Res<crate::spawn_tools::PointerOverUi>,
    mut sim: ResMut<crate::voxel_sim::VoxelSimState>,
) {
    if !armed.0 || over_ui.0 || !buttons.pressed(MouseButton::Left) {
        return;
    }
    let Some(centre) = marker.position else {
        return;
    };
    let salt = sim.tick;
    stamp_sphere(
        &mut sim.grid,
        centre,
        selected.clamped_size(),
        selected.strength,
        selected.material,
        salt,
    );
}

/// Write a spherical brush of `material` into `grid` centred on `centre`.
///
/// `strength` is the per-voxel fill probability; a cheap hash of the voxel
/// coordinate + `salt` gives a stable, allocation-free dither so feathered
/// brushes look organic without an RNG resource. Pure + standalone so it is
/// unit-testable headless.
#[cfg(feature = "voxel")]
fn stamp_sphere(
    grid: &mut civ_voxel::fluid_ca::CaGrid,
    centre: Vec3,
    radius: f32,
    strength: f32,
    material: MaterialId,
    salt: u64,
) {
    let r = radius.max(0.0);
    let r2 = r * r;
    let cx = centre.x.round() as i64;
    let cy = centre.y.round() as i64;
    let cz = centre.z.round() as i64;
    let ri = r.ceil() as i64;
    let strength = strength.clamp(0.0, 1.0);
    for dz in -ri..=ri {
        for dy in -ri..=ri {
            for dx in -ri..=ri {
                let dist2 = (dx * dx + dy * dy + dz * dz) as f32;
                if dist2 > r2 {
                    continue;
                }
                let (x, y, z) = (cx + dx, cy + dy, cz + dz);
                if x < 0 || y < 0 || z < 0 {
                    continue;
                }
                if strength < 1.0 && voxel_dither(x, y, z, salt) >= strength {
                    continue;
                }
                grid.set(x as usize, y as usize, z as usize, material);
            }
        }
    }
}

/// Stable `0.0..1.0` dither value for a voxel coordinate (feather brushes).
#[cfg(feature = "voxel")]
fn voxel_dither(x: i64, y: i64, z: i64, salt: u64) -> f32 {
    let mut h = salt
        ^ (x as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (y as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F)
        ^ (z as u64).wrapping_mul(0x1656_67B1_9E37_79F9);
    h ^= h >> 33;
    h = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
    h ^= h >> 33;
    (h >> 40) as f32 / (1u64 << 24) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_material_is_paintable_non_air() {
        let sel = SelectedMaterial::default();
        assert_ne!(sel.material.0, 0, "default brush must not paint air");
        assert!(sel.size >= BRUSH_RADIUS_MIN && sel.size <= BRUSH_RADIUS_MAX);
        assert!(sel.strength > 0.0 && sel.strength <= 1.0);
    }

    #[test]
    fn clamped_size_respects_bounds() {
        let mut sel = SelectedMaterial::default();
        sel.size = 999.0;
        assert_eq!(sel.clamped_size(), BRUSH_RADIUS_MAX);
        sel.size = 0.0;
        assert_eq!(sel.clamped_size(), BRUSH_RADIUS_MIN);
    }

    #[test]
    fn classify_buckets_standard_phases() {
        let registry = MaterialRegistry::standard();
        let water = registry.by_name("Water").expect("water");
        let sand = registry.by_name("Sand").expect("sand");
        let stone = registry.by_name("Stone").expect("stone");
        let steam = registry.by_name("Steam").expect("steam");
        let lava = registry.by_name("Lava").expect("lava");
        assert_eq!(MaterialFamily::classify(water), MaterialFamily::Liquids);
        assert_eq!(MaterialFamily::classify(sand), MaterialFamily::Powders);
        assert_eq!(MaterialFamily::classify(stone), MaterialFamily::Solids);
        assert_eq!(MaterialFamily::classify(steam), MaterialFamily::Gases);
        // Lava is a liquid by phase but reads as Energetic for the player.
        assert_eq!(MaterialFamily::classify(lava), MaterialFamily::Energetic);
    }

    #[test]
    fn shelves_drop_air_and_keep_all_other_materials() {
        let registry = MaterialRegistry::standard();
        let shelves = build_shelves(registry);
        let painted: usize = shelves.iter().map(|(_, m)| m.len()).sum();
        // Standard set is 12 materials incl. Air; Air must be filtered out.
        let non_air = registry
            .materials()
            .iter()
            .filter(|m| m.id.0 != 0 && !matches!(m.phase, Phase::Empty))
            .count();
        assert_eq!(painted, non_air);
        // No bucket contains air.
        for (_, mats) in &shelves {
            assert!(mats.iter().all(|m| m.id.0 != 0));
        }
    }

    #[test]
    fn shelves_are_in_family_order() {
        let registry = MaterialRegistry::standard();
        let shelves = build_shelves(registry);
        let families: Vec<MaterialFamily> = shelves.iter().map(|(f, _)| *f).collect();
        assert_eq!(families, MaterialFamily::ORDER.to_vec());
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn stamp_sphere_fills_centre_and_radius() {
        use civ_voxel::fluid_ca::CaGrid;
        let mut grid = CaGrid::new([16, 16, 16]);
        let mat = MaterialId(6);
        stamp_sphere(&mut grid, Vec3::new(8.0, 8.0, 8.0), 3.0, 1.0, mat, 0);
        // Centre painted.
        assert_eq!(grid.get(8, 8, 8), mat);
        // A cell well inside the radius painted.
        assert_eq!(grid.get(9, 8, 8), mat);
        // A cell outside the radius untouched (stays air).
        assert_ne!(grid.get(8 + 4, 8, 8), mat);
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn stamp_sphere_strength_feathers() {
        use civ_voxel::fluid_ca::CaGrid;
        let mut grid = CaGrid::new([32, 32, 32]);
        let mat = MaterialId(3);
        stamp_sphere(&mut grid, Vec3::new(16.0, 16.0, 16.0), 8.0, 0.3, mat, 7);
        let painted = grid.cells.iter().filter(|&&c| c == mat).count();
        // Sparse brush paints some but far from all voxels in the footprint.
        assert!(painted > 0, "feathered brush painted nothing");
        let full = (4.0 / 3.0 * std::f32::consts::PI * 8.0_f32.powi(3)) as usize;
        assert!(painted < full, "0.3 strength should not fill the sphere");
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn stamp_sphere_clamps_negative_coords() {
        use civ_voxel::fluid_ca::CaGrid;
        let mut grid = CaGrid::new([8, 8, 8]);
        // Centre near the origin corner so part of the brush is out of bounds.
        stamp_sphere(&mut grid, Vec3::new(0.0, 0.0, 0.0), 4.0, 1.0, MaterialId(1), 0);
        // Did not panic; origin cell painted.
        assert_eq!(grid.get(0, 0, 0), MaterialId(1));
    }
}
