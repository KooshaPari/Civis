#![cfg(feature = "bevy")]

//! Shared terraform brush controller + tool UX (Cities-Skylines + WorldBox
//! superset) for the Bevy reference client.
//!
//! Per `docs/specs/tool-design-directives.md`, the Terraform category must be a
//! SUPERSET of two design languages, sharing ONE brush param surface:
//!
//! * **Cities-Skylines precise modes** — surveyor-grade height editing with an
//!   adjustable brush **size / strength / falloff / shape**, and modes for
//!   **raise / lower / level-to-height / smooth / slope / flatten**.
//! * **WorldBox god-brushes** — instant, chunky, zero-friction strokes that
//!   **add land / dig ocean / raise mountain / drop biome**.
//!
//! ## Architecture (decoupled, file-local)
//!
//! This module owns *only* the brush UX and the brush math. It does **not** edit
//! `spawn_tools.rs` and does **not** reach into the terrain field or voxel
//! substrate directly. Instead it:
//!
//! 1. Reads the existing [`ActiveTool`]/[`SpawnTool::Terraform`] + the shared
//!    cursor hit ([`CursorMarker`]) and egui pointer gate ([`PointerOverUi`]) —
//!    honouring the same "egui owns the pointer first" rule as the spawn tools.
//! 2. Translates a click/drag under the Terraform tool into one
//!    [`TerraformEditRequest`] message per affected stroke, carrying the brush
//!    footprint (centre, radius, strength, falloff, shape) and the [`BrushOp`].
//!
//! The voxel/terrain applier (owned by the Voxel/Material Lead) drains
//! [`TerraformEditRequest`] and mutates material columns, respecting the
//! gravity/fluid CA after the edit. Until that applier lands, requests are still
//! emitted (and logged) so this UX is fully exercisable today.
//!
//! The same [`BrushSettings`] resource is intended to be reused by the Material
//! paint tool — it is the single brush param surface the directives mandate.

use bevy::prelude::*;

use crate::spawn_tools::{select_action_binding, ActiveTool, CursorMarker, PointerOverUi, SpawnTool};

/// Brush footprint shape on the XZ plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrushShape {
    /// Radial circle (default; CS soft brush, WorldBox round stamp).
    #[default]
    Circle,
    /// Axis-aligned square footprint.
    Square,
    /// Thin 1D diamond — useful for ridgelines / trenches.
    Diamond,
}

impl BrushShape {
    /// All shapes in palette order, for UI iteration.
    pub const ALL: [BrushShape; 3] = [BrushShape::Circle, BrushShape::Square, BrushShape::Diamond];

    /// Player-facing label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            BrushShape::Circle => "Circle",
            BrushShape::Square => "Square",
            BrushShape::Diamond => "Diamond",
        }
    }

    /// Normalised coverage weight in `[0, 1]` for a sample at planar offset
    /// `(dx, dz)` from the brush centre, given `radius`. `0` is outside the
    /// footprint, `1` is dead centre. Falloff is applied separately.
    #[must_use]
    pub fn coverage(self, dx: f32, dz: f32, radius: f32) -> f32 {
        if radius <= f32::EPSILON {
            return 0.0;
        }
        let nx = dx / radius;
        let nz = dz / radius;
        let d = match self {
            BrushShape::Circle => (nx * nx + nz * nz).sqrt(),
            BrushShape::Square => nx.abs().max(nz.abs()),
            BrushShape::Diamond => nx.abs() + nz.abs(),
        };
        (1.0 - d).clamp(0.0, 1.0)
    }
}

/// Falloff curve applied to a shape's normalised coverage weight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrushFalloff {
    /// `w` (CS soft brush — linear edge).
    #[default]
    Linear,
    /// Smoothstep — soft, photographic edges.
    Smooth,
    /// `1` everywhere inside the footprint (WorldBox chunky stamp).
    Hard,
}

impl BrushFalloff {
    /// All falloff curves in palette order, for UI iteration.
    pub const ALL: [BrushFalloff; 3] = [
        BrushFalloff::Linear,
        BrushFalloff::Smooth,
        BrushFalloff::Hard,
    ];

    /// Player-facing label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            BrushFalloff::Linear => "Linear",
            BrushFalloff::Smooth => "Smooth",
            BrushFalloff::Hard => "Hard",
        }
    }

    /// Map a normalised coverage `w` in `[0, 1]` through the falloff curve.
    #[must_use]
    pub fn apply(self, w: f32) -> f32 {
        let w = w.clamp(0.0, 1.0);
        match self {
            BrushFalloff::Linear => w,
            BrushFalloff::Smooth => w * w * (3.0 - 2.0 * w),
            BrushFalloff::Hard => {
                if w > 0.0 {
                    1.0
                } else {
                    0.0
                }
            }
        }
    }
}

/// What a brush stroke does to the terrain/voxel column under it.
///
/// The first family is Cities-Skylines *precise* editing; the second is
/// WorldBox *god-brush* instant edits. Both share the [`BrushSettings`] param
/// surface; god-brushes simply imply a chunky strength/falloff at emit time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrushOp {
    // --- Cities-Skylines precise modes ---
    /// Add height, weighted by brush + falloff.
    #[default]
    Raise,
    /// Subtract height, weighted by brush + falloff.
    Lower,
    /// Drive height toward a picked target ([`BrushSettings::target_height`]).
    LevelToHeight,
    /// Low-pass / relax height toward the local neighbourhood average.
    Smooth,
    /// Ramp height linearly between two picked points (handled as a directional
    /// level by the applier; carries the same footprint).
    Slope,
    /// Flatten to the height sampled at the stroke's first contact point.
    Flatten,

    // --- WorldBox god-brushes (instant, chunky) ---
    /// Instantly add a chunk of land (raise above water + fill).
    AddLand,
    /// Instantly carve below sea level (dig ocean / lake / river).
    DigOcean,
    /// Instantly push up a mountain mass.
    RaiseMountain,
    /// Instantly stamp a biome over the footprint (no height change).
    DropBiome,
}

impl BrushOp {
    /// Precise (Cities-Skylines) modes, in UI order.
    pub const PRECISE: [BrushOp; 6] = [
        BrushOp::Raise,
        BrushOp::Lower,
        BrushOp::LevelToHeight,
        BrushOp::Smooth,
        BrushOp::Slope,
        BrushOp::Flatten,
    ];

    /// God-brush (WorldBox) modes, in UI order.
    pub const GOD: [BrushOp; 4] = [
        BrushOp::AddLand,
        BrushOp::DigOcean,
        BrushOp::RaiseMountain,
        BrushOp::DropBiome,
    ];

    /// Player-facing label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            BrushOp::Raise => "Raise",
            BrushOp::Lower => "Lower",
            BrushOp::LevelToHeight => "Level",
            BrushOp::Smooth => "Smooth",
            BrushOp::Slope => "Slope",
            BrushOp::Flatten => "Flatten",
            BrushOp::AddLand => "Add Land",
            BrushOp::DigOcean => "Dig Ocean",
            BrushOp::RaiseMountain => "Mountain",
            BrushOp::DropBiome => "Biome",
        }
    }

    /// True for the instant WorldBox god-brushes (chunky, zero-friction).
    #[must_use]
    pub fn is_god(self) -> bool {
        matches!(
            self,
            BrushOp::AddLand | BrushOp::DigOcean | BrushOp::RaiseMountain | BrushOp::DropBiome
        )
    }

    /// True if this op edits height (everything except the pure biome stamp).
    #[must_use]
    pub fn edits_height(self) -> bool {
        !matches!(self, BrushOp::DropBiome)
    }
}

/// Whether the brush is in precise (CS) or god (WorldBox) sub-mode. Selecting a
/// mode in either family also flips this so the UI and emit path agree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum BrushFamily {
    /// Cities-Skylines surveyor-grade controls.
    #[default]
    Precise,
    /// WorldBox instant god-brushes.
    God,
}

/// The single shared brush parameter surface (size / strength / falloff / shape
/// / op). Reused by terraform and, by design, the material paint tool.
#[derive(Resource, Debug, Clone, Copy, PartialEq)]
pub struct BrushSettings {
    /// Brush radius in world metres.
    pub radius: f32,
    /// Per-stroke height delta magnitude in metres at full coverage.
    pub strength: f32,
    /// Falloff curve from centre to edge.
    pub falloff: BrushFalloff,
    /// Footprint shape.
    pub shape: BrushShape,
    /// Active operation.
    pub op: BrushOp,
    /// Precise vs god sub-mode.
    pub family: BrushFamily,
    /// Target height for [`BrushOp::LevelToHeight`] (world metres).
    pub target_height: f32,
    /// Continuous paint (apply every frame while held) vs single stamp on press.
    pub continuous: bool,
    /// Biome id stamped by [`BrushOp::DropBiome`].
    pub biome_id: u8,
}

impl Default for BrushSettings {
    fn default() -> Self {
        Self {
            radius: 18.0,
            strength: 4.0,
            falloff: BrushFalloff::Smooth,
            shape: BrushShape::Circle,
            op: BrushOp::Raise,
            family: BrushFamily::Precise,
            target_height: 64.0,
            continuous: true,
            biome_id: 0,
        }
    }
}

impl BrushSettings {
    /// Minimum / maximum brush radius (world metres) exposed by the UI slider.
    pub const RADIUS_RANGE: (f32, f32) = (2.0, 96.0);
    /// Minimum / maximum stroke strength (metres) exposed by the UI slider.
    pub const STRENGTH_RANGE: (f32, f32) = (0.25, 40.0);

    /// Select an op and keep [`Self::family`] consistent with it.
    pub fn select_op(&mut self, op: BrushOp) {
        self.op = op;
        self.family = if op.is_god() {
            BrushFamily::God
        } else {
            BrushFamily::Precise
        };
    }

    /// Effective per-stamp height delta (signed) at full coverage for the
    /// current op. God-brushes apply a chunky multiplier so they feel instant.
    #[must_use]
    pub fn effective_strength(&self) -> f32 {
        match self.op {
            BrushOp::Raise | BrushOp::AddLand => self.strength,
            BrushOp::Lower | BrushOp::DigOcean => -self.strength,
            BrushOp::RaiseMountain => self.strength * 4.0,
            // Level/Smooth/Slope/Flatten resolve their delta against samples in
            // the applier; carry the configured magnitude as a rate cap.
            _ => self.strength,
        }
    }

    /// Per-sample signed height delta at planar offset `(dx, dz)` from centre,
    /// after shape coverage + falloff. Used by the applier and unit tests.
    #[must_use]
    pub fn sample_delta(&self, dx: f32, dz: f32) -> f32 {
        if !self.op.edits_height() {
            return 0.0;
        }
        let cov = self.shape.coverage(dx, dz, self.radius);
        let w = self.falloff.apply(cov);
        w * self.effective_strength()
    }
}

/// A terraform stroke stamp to be applied to the terrain/voxel substrate.
///
/// One message is emitted per stamp (press, or per-frame while held in
/// continuous mode). The applier owns translating this footprint into column
/// edits + CA re-settle.
#[derive(Message, Debug, Clone, Copy, PartialEq)]
pub struct TerraformEditRequest {
    /// World-space centre of the stamp (terrain hit point).
    pub center: Vec3,
    /// Brush radius (world metres).
    pub radius: f32,
    /// Signed full-coverage height delta (metres) for height ops; rate cap for
    /// level/smooth/slope/flatten.
    pub strength: f32,
    /// Falloff curve.
    pub falloff: BrushFalloff,
    /// Footprint shape.
    pub shape: BrushShape,
    /// Operation to apply.
    pub op: BrushOp,
    /// Target height for [`BrushOp::LevelToHeight`].
    pub target_height: f32,
    /// Biome id for [`BrushOp::DropBiome`].
    pub biome_id: u8,
}

/// Per-stroke state so `Flatten` can capture the first-contact height and the
/// stamp can debounce in single-stamp mode.
#[derive(Resource, Debug, Default, Clone, Copy)]
pub struct BrushStroke {
    /// Whether a stroke is currently in progress (left button held).
    pub active: bool,
    /// Height captured at the stroke's first contact (for `Flatten`).
    pub anchor_height: Option<f32>,
}

/// Plugin wiring the brush settings, stroke state, edit message + emit system.
pub struct TerraformBrushPlugin;

impl Plugin for TerraformBrushPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BrushSettings>()
            .init_resource::<BrushStroke>()
            .add_message::<TerraformEditRequest>()
            // Runs after the spawn-tools cursor/pointer systems each frame so the
            // hit + egui gate are current; ordering is via the default Update
            // set (spawn_tools chains its own pipeline independently).
            .add_systems(Update, emit_terraform_edits);

        // The applier that actually mutates the voxel world. Without it every
        // TerraformEditRequest is emitted then dropped (all god/precise brushes
        // were silent no-ops). Runs only when the voxel sim is compiled in.
        #[cfg(feature = "voxel")]
        app.add_systems(Update, apply_terraform_edits.after(emit_terraform_edits));

        #[cfg(feature = "egui")]
        app.add_systems(
            bevy_egui::EguiPrimaryContextPass,
            draw_brush_panel.run_if(crate::menus::in_game),
        );
    }
}

/// Drain [`TerraformEditRequest`] stamps and mutate the live voxel grid so the
/// terraform + god-brushes actually edit the world. Height-raising ops add
/// solid columns, lowering ops carve to air, level/flatten drive the surface to
/// a target. Each write goes through `grid.set`, which marks the chunk dirty so
/// `step_and_remesh` re-meshes it (the change becomes visible next tick).
#[cfg(feature = "voxel")]
fn apply_terraform_edits(
    mut edits: MessageReader<TerraformEditRequest>,
    mut sim: ResMut<crate::voxel_sim::VoxelSimState>,
) {
    use civ_voxel::material::{MaterialRegistry, STONE};
    let registry = MaterialRegistry::standard();
    let solid = registry
        .by_name("Stone")
        .or_else(|| registry.by_name("Dirt"))
        .map_or(STONE, |m| m.id);
    for req in edits.read() {
        apply_one_terraform_edit(&mut sim.grid, req, solid);
    }
}

/// Apply one terraform stamp across its XZ footprint, one column at a time.
#[cfg(feature = "voxel")]
fn apply_one_terraform_edit(
    grid: &mut civ_voxel::fluid_ca::CaGrid,
    req: &TerraformEditRequest,
    solid: civ_voxel::MaterialId,
) {
    let ri = req.radius.ceil() as i64;
    let cx = req.center.x.round() as i64;
    let cz = req.center.z.round() as i64;
    for dz in -ri..=ri {
        for dx in -ri..=ri {
            let w = req
                .falloff
                .apply(req.shape.coverage(dx as f32, dz as f32, req.radius));
            if w <= 0.0 {
                continue;
            }
            let (x, z) = (cx + dx, cz + dz);
            if x < 0 || z < 0 {
                continue;
            }
            edit_column(grid, x as usize, z as usize, w, req, solid);
        }
    }
}

/// Mutate a single voxel column according to the stamp's op + per-column weight.
#[cfg(feature = "voxel")]
fn edit_column(
    grid: &mut civ_voxel::fluid_ca::CaGrid,
    x: usize,
    z: usize,
    weight: f32,
    req: &TerraformEditRequest,
    solid: civ_voxel::MaterialId,
) {
    use civ_voxel::material::AIR;
    if x >= grid.dims[0] || z >= grid.dims[2] {
        return;
    }
    let height = grid.dims[1];
    let surface = surface_y(grid, x, z);
    match req.op {
        BrushOp::Raise | BrushOp::AddLand | BrushOp::RaiseMountain => {
            let add = (req.strength.abs() * weight).round() as usize;
            for y in surface..(surface + add).min(height - 1) {
                grid.set(x, y, z, solid);
            }
        }
        BrushOp::Lower | BrushOp::DigOcean => {
            let cut = (req.strength.abs() * weight).round() as usize;
            for y in surface.saturating_sub(cut)..surface {
                grid.set(x, y, z, AIR);
            }
        }
        BrushOp::LevelToHeight | BrushOp::Flatten | BrushOp::Slope => {
            let target = (req.target_height.round() as usize).min(height - 1);
            level_column(grid, x, z, surface, target, solid);
        }
        BrushOp::Smooth => {
            // Low-pass: nudge this column toward its 4-neighbour surface average
            // so jagged terrain relaxes. Weight scales how far it moves per stamp.
            let avg = neighbour_surface_avg(grid, x, z);
            let target = (surface as f32 + (avg - surface as f32) * weight)
                .round()
                .clamp(0.0, (height - 1) as f32) as usize;
            level_column(grid, x, z, surface, target, solid);
        }
        // DropBiome edits no height; biome painting is deferred for a later pass.
        BrushOp::DropBiome => {}
    }
}

/// Mean surface height of the four axial neighbours (clamped to the grid), used
/// by the Smooth brush as the relaxation target.
#[cfg(feature = "voxel")]
fn neighbour_surface_avg(grid: &civ_voxel::fluid_ca::CaGrid, x: usize, z: usize) -> f32 {
    let xm = x.saturating_sub(1);
    let zm = z.saturating_sub(1);
    let xp = (x + 1).min(grid.dims[0] - 1);
    let zp = (z + 1).min(grid.dims[2] - 1);
    let sum = surface_y(grid, xm, z)
        + surface_y(grid, xp, z)
        + surface_y(grid, x, zm)
        + surface_y(grid, x, zp);
    sum as f32 / 4.0
}

/// First-non-air cell scan from the top: returns the lowest air `y` resting on
/// solid (i.e. the surface). Returns 0 for an all-air column.
#[cfg(feature = "voxel")]
fn surface_y(grid: &civ_voxel::fluid_ca::CaGrid, x: usize, z: usize) -> usize {
    use civ_voxel::material::AIR;
    for y in (0..grid.dims[1]).rev() {
        if grid.get(x, y, z) != AIR {
            return y + 1;
        }
    }
    0
}

/// Drive a column's surface toward `target`: fill with `solid` if below, carve
/// to air if above.
#[cfg(feature = "voxel")]
fn level_column(
    grid: &mut civ_voxel::fluid_ca::CaGrid,
    x: usize,
    z: usize,
    surface: usize,
    target: usize,
    solid: civ_voxel::MaterialId,
) {
    use civ_voxel::material::AIR;
    if target > surface {
        for y in surface..target {
            grid.set(x, y, z, solid);
        }
    } else {
        for y in target..surface {
            grid.set(x, y, z, AIR);
        }
    }
}

/// Translate a click/drag under the Terraform tool into [`TerraformEditRequest`]
/// stamps, honouring the egui pointer gate and the shared cursor hit.
fn emit_terraform_edits(
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    settings: Option<Res<crate::settings_ui::GameSettings>>,
    active: Res<ActiveTool>,
    over_ui: Res<PointerOverUi>,
    marker: Res<CursorMarker>,
    brush: Res<BrushSettings>,
    mut stroke: ResMut<BrushStroke>,
    mut edits: MessageWriter<TerraformEditRequest>,
) {
    // Only the Terraform tool drives the brush; reset stroke state otherwise.
    if active.tool != SpawnTool::Terraform {
        stroke.active = false;
        stroke.anchor_height = None;
        return;
    }

    let select_binding = select_action_binding(settings.as_deref());
    let released = select_binding_just_released(select_binding, &keys, &buttons);
    if released {
        stroke.active = false;
        stroke.anchor_height = None;
        return;
    }

    // Egui owns the pointer first: never stamp through a HUD panel/slider.
    if over_ui.0 {
        return;
    }

    let pressed = select_binding_just_pressed(select_binding, &keys, &buttons);
    let held = select_binding.is_pressed(&keys, &buttons);
    if !pressed && !held {
        return;
    }

    let Some(center) = marker.position else {
        return;
    };

    // Single-stamp ops (or non-continuous mode) fire once on press; continuous
    // ops keep stamping while held. God-brushes are always single chunky stamps.
    let stamp_now = if pressed {
        true
    } else {
        held && brush.continuous && !brush.op.is_god()
    };
    if !stamp_now {
        return;
    }

    if pressed {
        stroke.active = true;
        stroke.anchor_height = Some(center.y);
    }

    // Flatten targets the first-contact height for the whole stroke.
    let target_height = match brush.op {
        BrushOp::Flatten => stroke.anchor_height.unwrap_or(center.y),
        _ => brush.target_height,
    };

    edits.write(TerraformEditRequest {
        center,
        radius: brush.radius,
        strength: brush.effective_strength(),
        falloff: brush.falloff,
        shape: brush.shape,
        op: brush.op,
        target_height,
        biome_id: brush.biome_id,
    });
}

/// egui control surface: brush size / strength / falloff / shape sliders, the
/// precise-vs-god family toggle, and the mode buttons for both families. Only
/// shown while the Terraform tool is active so it does not clutter the HUD.
#[cfg(feature = "egui")]
fn draw_brush_panel(
    mut contexts: bevy_egui::EguiContexts,
    active: Res<ActiveTool>,
    mut brush: ResMut<BrushSettings>,
) {
    use bevy_egui::egui;

    if active.tool != SpawnTool::Terraform {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Terraform Brush")
        .resizable(false)
        .default_width(244.0)
        .anchor(egui::Align2::RIGHT_TOP, egui::vec2(-12.0, 84.0))
        .show(ctx, |ui| {
            family_selector(ui, &mut brush);
            ui.separator();
            mode_buttons(ui, &mut brush);
            ui.separator();
            brush_param_sliders(ui, &mut brush);
        });
}

/// Precise vs god family toggle row.
#[cfg(feature = "egui")]
fn family_selector(ui: &mut bevy_egui::egui::Ui, brush: &mut BrushSettings) {
    ui.horizontal(|ui| {
        if ui
            .selectable_label(brush.family == BrushFamily::Precise, "Precise (CS)")
            .clicked()
            && brush.family != BrushFamily::Precise
        {
            brush.select_op(BrushOp::Raise);
        }
        if ui
            .selectable_label(brush.family == BrushFamily::God, "God (WorldBox)")
            .clicked()
            && brush.family != BrushFamily::God
        {
            brush.select_op(BrushOp::AddLand);
        }
    });
}

/// Mode buttons for the active family (precise or god).
#[cfg(feature = "egui")]
fn mode_buttons(ui: &mut bevy_egui::egui::Ui, brush: &mut BrushSettings) {
    let ops: &[BrushOp] = match brush.family {
        BrushFamily::Precise => &BrushOp::PRECISE,
        BrushFamily::God => &BrushOp::GOD,
    };
    ui.horizontal_wrapped(|ui| {
        for &op in ops {
            if ui.selectable_label(brush.op == op, op.label()).clicked() {
                brush.select_op(op);
            }
        }
    });
}

/// Size / strength / falloff / shape sliders + op-specific extras.
#[cfg(feature = "egui")]
fn brush_param_sliders(ui: &mut bevy_egui::egui::Ui, brush: &mut BrushSettings) {
    use bevy_egui::egui;

    let (r0, r1) = BrushSettings::RADIUS_RANGE;
    let (s0, s1) = BrushSettings::STRENGTH_RANGE;
    ui.add(egui::Slider::new(&mut brush.radius, r0..=r1).text("Size"));
    ui.add(egui::Slider::new(&mut brush.strength, s0..=s1).text("Strength"));

    ui.horizontal(|ui| {
        ui.label("Falloff:");
        for f in BrushFalloff::ALL {
            if ui.selectable_label(brush.falloff == f, f.label()).clicked() {
                brush.falloff = f;
            }
        }
    });
    ui.horizontal(|ui| {
        ui.label("Shape:");
        for s in BrushShape::ALL {
            if ui.selectable_label(brush.shape == s, s.label()).clicked() {
                brush.shape = s;
            }
        }
    });

    match brush.op {
        BrushOp::LevelToHeight => {
            ui.add(egui::Slider::new(&mut brush.target_height, 0.0..=200.0).text("Target height"));
        }
        BrushOp::DropBiome => {
            ui.add(egui::Slider::new(&mut brush.biome_id, 0..=15).text("Biome id"));
        }
        _ => {}
    }

    if !brush.op.is_god() {
        ui.checkbox(&mut brush.continuous, "Continuous paint");
    }
}

fn select_binding_just_pressed(
    binding: crate::settings_ui::KeyBinding,
    keys: &ButtonInput<KeyCode>,
    buttons: &ButtonInput<MouseButton>,
) -> bool {
    match binding {
        crate::settings_ui::KeyBinding::Key(key) => keys.just_pressed(key),
        crate::settings_ui::KeyBinding::Mouse(button) => buttons.just_pressed(button),
    }
}

fn select_binding_just_released(
    binding: crate::settings_ui::KeyBinding,
    keys: &ButtonInput<KeyCode>,
    buttons: &ButtonInput<MouseButton>,
) -> bool {
    match binding {
        crate::settings_ui::KeyBinding::Key(key) => keys.just_released(key),
        crate::settings_ui::KeyBinding::Mouse(button) => buttons.just_released(button),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn circle_coverage_peaks_at_center_and_zero_at_edge() {
        let s = BrushShape::Circle;
        assert!((s.coverage(0.0, 0.0, 10.0) - 1.0).abs() < 1e-6);
        assert!(s.coverage(10.0, 0.0, 10.0).abs() < 1e-6);
        assert!(s.coverage(20.0, 0.0, 10.0).abs() < 1e-6);
    }

    #[test]
    fn square_covers_corners_circle_does_not() {
        // A point at the box corner (r, r) is inside a square footprint of edge
        // half-extent r only along an axis; the diagonal corner is outside.
        assert!(BrushShape::Square.coverage(9.9, 0.0, 10.0) > 0.0);
        assert!(BrushShape::Circle.coverage(8.0, 8.0, 10.0).abs() < 1e-6);
    }

    #[test]
    fn hard_falloff_is_binary() {
        assert_eq!(BrushFalloff::Hard.apply(0.01), 1.0);
        assert_eq!(BrushFalloff::Hard.apply(0.0), 0.0);
    }

    #[test]
    fn smooth_falloff_is_monotonic_between_endpoints() {
        assert!((BrushFalloff::Smooth.apply(0.0)).abs() < 1e-6);
        assert!((BrushFalloff::Smooth.apply(1.0) - 1.0).abs() < 1e-6);
        assert!(BrushFalloff::Smooth.apply(0.25) < BrushFalloff::Smooth.apply(0.75));
    }

    #[test]
    fn select_op_keeps_family_consistent() {
        let mut b = BrushSettings::default();
        b.select_op(BrushOp::DigOcean);
        assert_eq!(b.family, BrushFamily::God);
        b.select_op(BrushOp::Smooth);
        assert_eq!(b.family, BrushFamily::Precise);
    }

    #[test]
    fn raise_and_lower_have_opposite_signs() {
        let mut b = BrushSettings::default();
        b.select_op(BrushOp::Raise);
        assert!(b.effective_strength() > 0.0);
        b.select_op(BrushOp::Lower);
        assert!(b.effective_strength() < 0.0);
    }

    #[test]
    fn mountain_is_chunkier_than_plain_raise() {
        let mut b = BrushSettings::default();
        b.select_op(BrushOp::Raise);
        let raise = b.effective_strength();
        b.select_op(BrushOp::RaiseMountain);
        assert!(b.effective_strength() > raise);
    }

    #[test]
    fn biome_op_does_not_edit_height() {
        let mut b = BrushSettings::default();
        b.select_op(BrushOp::DropBiome);
        assert_eq!(b.sample_delta(0.0, 0.0), 0.0);
        assert!(!BrushOp::DropBiome.edits_height());
    }

    #[test]
    fn sample_delta_falls_off_from_center() {
        let mut b = BrushSettings::default();
        b.select_op(BrushOp::Raise);
        b.falloff = BrushFalloff::Linear;
        b.shape = BrushShape::Circle;
        let center = b.sample_delta(0.0, 0.0);
        let edge = b.sample_delta(b.radius * 0.5, 0.0);
        assert!(center > edge);
        assert!(edge > 0.0);
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn raise_adds_solid_lower_carves_air() {
        use civ_voxel::fluid_ca::CaGrid;
        use civ_voxel::material::{MaterialRegistry, AIR};
        let solid = MaterialRegistry::standard().by_name("Stone").unwrap().id;
        // Flat ground: bottom 8 cells solid, rest air.
        let mut grid = CaGrid::new([8, 32, 8]);
        for z in 0..8 {
            for x in 0..8 {
                for y in 0..8 {
                    grid.set(x, y, z, solid);
                }
            }
        }
        let base = BrushSettings::default();
        let raise = TerraformEditRequest {
            center: Vec3::new(4.0, 8.0, 4.0),
            radius: 3.0,
            strength: 4.0,
            falloff: BrushFalloff::Hard,
            shape: BrushShape::Circle,
            op: BrushOp::Raise,
            target_height: base.target_height,
            biome_id: 0,
        };
        apply_one_terraform_edit(&mut grid, &raise, solid);
        assert_eq!(
            grid.get(4, 8, 4),
            solid,
            "raise must add solid above surface"
        );

        let lower = TerraformEditRequest {
            op: BrushOp::Lower,
            ..raise
        };
        apply_one_terraform_edit(&mut grid, &lower, solid);
        assert_eq!(
            grid.get(4, 8, 4),
            AIR,
            "lower must carve the raised solid back to air"
        );
    }

    #[cfg(feature = "voxel")]
    #[test]
    fn surface_y_finds_top_of_solid() {
        use civ_voxel::fluid_ca::CaGrid;
        use civ_voxel::material::MaterialRegistry;
        let solid = MaterialRegistry::standard().by_name("Stone").unwrap().id;
        let mut grid = CaGrid::new([4, 16, 4]);
        for y in 0..5 {
            grid.set(1, y, 1, solid);
        }
        assert_eq!(
            surface_y(&grid, 1, 1),
            5,
            "surface is one above the top solid cell"
        );
        assert_eq!(surface_y(&grid, 0, 0), 0, "all-air column has surface 0");
    }

    #[test]
    fn god_ops_classified() {
        for op in BrushOp::GOD {
            assert!(op.is_god(), "{op:?} should be a god-brush");
        }
        for op in BrushOp::PRECISE {
            assert!(!op.is_god(), "{op:?} should be precise");
        }
    }
}
