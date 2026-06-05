#![cfg(all(feature = "bevy", feature = "egui"))]

//! Tech tree overlay window for the Civis gameplay HUD.
//!
//! Provides [`TechTreeUiPlugin`] which registers:
//! - [`TechTreeState`]  — resource holding the node catalogue + unlock progress.
//! - [`TechTreeOpen`]   — toggle resource for the overlay window.
//! - `toggle_tech_tree` — Update system mapping **T** to the overlay toggle.
//! - `draw_tech_tree`   — EguiPrimaryContextPass system rendering the tree.
//!
//! Press **T** in-game to open/close the tech tree.  The overlay is purely
//! presentational here: it reads [`TechTreeState`], which other systems mutate
//! as research completes.  EguiPlugin is owned by `GameUiPlugin`; this module
//! never re-registers it.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::ui_theme;

// ---------------------------------------------------------------------------
// Palette — sourced from the shared `ui_theme` so the tech-tree overlay tracks
// the cohesive dark-glass HUD language used across every panel.
// ---------------------------------------------------------------------------

const PANEL_FILL: egui::Color32 = ui_theme::PANEL_FILL;
const CHIP_FILL: egui::Color32 = ui_theme::SURFACE;
const ACCENT: egui::Color32 = ui_theme::ACCENT;
const LOCKED_DIM: egui::Color32 = ui_theme::DIM;
const TEXT_MAIN: egui::Color32 = ui_theme::TEXT;
const GOLD: egui::Color32 = ui_theme::GOLD;

// ---------------------------------------------------------------------------
// Domain types
// ---------------------------------------------------------------------------

/// Research era grouping a column of technologies.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TechEra {
    /// Stone / foundational era.
    Ancient,
    /// Iron / organised era.
    Classical,
    /// Gunpowder / printing era.
    Medieval,
    /// Steam / electricity era.
    Industrial,
}

impl TechEra {
    /// Display label for the era column header.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ancient => "Ancient",
            Self::Classical => "Classical",
            Self::Medieval => "Medieval",
            Self::Industrial => "Industrial",
        }
    }

    /// Column order, ascending (Ancient first).
    pub fn order(&self) -> u8 {
        match self {
            Self::Ancient => 0,
            Self::Classical => 1,
            Self::Medieval => 2,
            Self::Industrial => 3,
        }
    }
}

/// A single technology node in the tree.
#[derive(Clone, Debug)]
pub struct TechNode {
    /// Stable identifier (also used for prerequisite links).
    pub id: &'static str,
    /// Display name shown on the node card.
    pub name: &'static str,
    /// Era column this node lives in.
    pub era: TechEra,
    /// Whether the player has unlocked this technology.
    pub unlocked: bool,
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Catalogue of tech nodes plus unlock progress.
///
/// Populated live from the simulation by [`sync_tech_tree_from_sim`], which
/// reads the emergent era-diffusion state (the highest era any civilian has
/// adopted in their `Wardrobe`/`Tools`, plus the current cohort adoption
/// fraction). The seed catalogue is only a fallback before the first tick.
#[derive(Resource)]
pub struct TechTreeState {
    /// All known technology nodes.
    pub nodes: Vec<TechNode>,
    /// Highest era index any civilian has reached (engine `era` units). Drives
    /// which era columns are unlocked. `0` before the sim has ticked.
    pub current_era: u16,
    /// Live adoption fraction of the cohort at the leading era, in `0.0..=1.0`.
    pub adoption_fraction: f32,
    /// Civilians already at-or-above the leading era (live cohort sample).
    pub at_leading_era: u32,
    /// Total civilians considered in the diffusion cohort this tick.
    pub cohort_total: u32,
    /// Whether the panel has received at least one live sim sample.
    pub live: bool,
}

impl Default for TechTreeState {
    fn default() -> Self {
        Self {
            nodes: default_tech_nodes(),
            current_era: 0,
            adoption_fraction: 0.0,
            at_leading_era: 0,
            cohort_total: 0,
            live: false,
        }
    }
}

impl TechTreeState {
    /// Number of unlocked nodes.
    pub fn unlocked_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.unlocked).count()
    }

    /// Fraction of the tree unlocked, in `0.0..=1.0`.
    pub fn progress(&self) -> f32 {
        if self.nodes.is_empty() {
            return 0.0;
        }
        self.unlocked_count() as f32 / self.nodes.len() as f32
    }

    /// Mark a node unlocked by id, returning true if a node was found.
    pub fn unlock(&mut self, id: &str) -> bool {
        if let Some(node) = self.nodes.iter_mut().find(|n| n.id == id) {
            node.unlocked = true;
            true
        } else {
            false
        }
    }

    /// Reconcile node unlock flags against a live engine era index.
    ///
    /// An era column (and all its nodes) is considered unlocked once the
    /// civilisation's leading era has reached that column's [`TechEra::order`].
    /// This makes the tree reflect the emergent era-diffusion in the sim rather
    /// than a hardcoded unlock list.
    pub fn apply_era(&mut self, current_era: u16) {
        self.current_era = current_era;
        for node in &mut self.nodes {
            node.unlocked = u16::from(node.era.order()) <= current_era;
        }
    }
}

/// Toggle resource for the tech tree overlay window.  Bound to **T**.
#[derive(Resource, Default)]
pub struct TechTreeOpen(pub bool);

/// Default seed catalogue: a small representative tech tree across four eras.
fn default_tech_nodes() -> Vec<TechNode> {
    vec![
        node("pottery", "Pottery", TechEra::Ancient, true),
        node("masonry", "Masonry", TechEra::Ancient, true),
        node("writing", "Writing", TechEra::Ancient, false),
        node("iron_working", "Iron Working", TechEra::Classical, false),
        node("currency", "Currency", TechEra::Classical, false),
        node("mathematics", "Mathematics", TechEra::Classical, false),
        node("gunpowder", "Gunpowder", TechEra::Medieval, false),
        node("printing", "Printing Press", TechEra::Medieval, false),
        node("banking", "Banking", TechEra::Medieval, false),
        node("steam_power", "Steam Power", TechEra::Industrial, false),
        node("electricity", "Electricity", TechEra::Industrial, false),
        node("railroad", "Railroad", TechEra::Industrial, false),
    ]
}

fn node(id: &'static str, name: &'static str, era: TechEra, unlocked: bool) -> TechNode {
    TechNode {
        id,
        name,
        era,
        unlocked,
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Registers the tech tree resources and rendering systems.
///
/// Depends on [`bevy_egui::EguiPlugin`] already being registered by
/// `GameUiPlugin`; this plugin does **not** register it (duplicate
/// registration panics in `bevy_egui`).
pub struct TechTreeUiPlugin;

impl Plugin for TechTreeUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TechTreeState>()
            .init_resource::<TechTreeOpen>()
            .add_systems(Update, (toggle_tech_tree, sync_tech_tree_from_sim))
            .add_systems(
                EguiPrimaryContextPass,
                draw_tech_tree.run_if(crate::menus::in_game),
            );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Map **T** to opening / closing the tech tree overlay.
pub fn toggle_tech_tree(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<TechTreeOpen>) {
    if keys.just_pressed(KeyCode::KeyT) {
        open.0 = !open.0;
    }
}

/// Pull the live tech progression out of the running simulation.
///
/// Reads the emergent era-diffusion state every time the simulation changes:
/// - the **leading era** is the highest `Wardrobe`/`Tools` era any civilian has
///   adopted (the civilisation's research frontier),
/// - the **cohort stats** (`last_cohort_stats`) give the live adoption fraction
///   and how many civilians have reached that frontier this tick.
///
/// Era columns unlock as the frontier advances, so the panel shows real
/// progress instead of the seed placeholder list.
pub fn sync_tech_tree_from_sim(
    sim: Res<crate::sim_bridge::SimState>,
    mut state: ResMut<TechTreeState>,
) {
    if !sim.is_changed() {
        return;
    }
    let sim = &sim.0;

    // Research frontier = highest worn-tech era across the population.
    let mut leading_era: u16 = 0;
    for (_, wardrobe) in sim.world.query::<&civ_agents::Wardrobe>().iter() {
        leading_era = leading_era.max(wardrobe.era);
    }
    for (_, tools) in sim.world.query::<&civ_agents::Tools>().iter() {
        leading_era = leading_era.max(tools.era);
    }

    state.apply_era(leading_era);

    if let Some(stats) = sim.last_cohort_stats() {
        state.adoption_fraction = stats.current_fraction.clamp(0.0, 1.0);
        state.at_leading_era = stats.currently_at_target;
        state.cohort_total = stats.total_civilians;
    }
    state.live = true;
}

/// Draw the tech tree overlay window when [`TechTreeOpen`] is set.
pub fn draw_tech_tree(
    mut contexts: EguiContexts,
    state: Res<TechTreeState>,
    mut open: ResMut<TechTreeOpen>,
) {
    if !open.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ui_theme::apply_theme(ctx);

    let mut window_open = open.0;
    egui::Window::new("⚛ Tech Tree")
        .open(&mut window_open)
        .resizable(true)
        .default_size([640.0, 460.0])
        .frame(ui_theme::frame_e1(egui::Margin::same(14)))
        .show(ctx, |ui| {
            draw_progress_header(ui, &state);
            ui.add_space(8.0);
            egui::ScrollArea::horizontal()
                .auto_shrink([false; 2])
                .show(ui, |ui| {
                    draw_era_columns(ui, &state);
                });
            ui_theme::panel_finish(ui.painter(), ui.min_rect(), ui_theme::RADIUS, false, false);
        });
    open.0 = window_open;
}

// ---------------------------------------------------------------------------
// UI helpers  (each ≤ 40 lines)
// ---------------------------------------------------------------------------

/// Render the unlock-progress header line + bar plus live diffusion stats.
fn draw_progress_header(ui: &mut egui::Ui, state: &TechTreeState) {
    let progress = state.progress();
    let era_label = leading_era_label(state.current_era);
    ui.horizontal(|ui| {
        ui.label(
            egui::RichText::new(format!(
                "{} / {} technologies unlocked",
                state.unlocked_count(),
                state.nodes.len()
            ))
            .color(TEXT_MAIN)
            .size(14.0),
        );
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            let badge = if state.live {
                "● live"
            } else {
                "○ waiting"
            };
            let badge_color = if state.live { ACCENT } else { LOCKED_DIM };
            ui.label(egui::RichText::new(badge).color(badge_color).size(12.0));
        });
    });
    ui.add_space(2.0);
    ui.label(
        egui::RichText::new(format!("Era frontier: {era_label}"))
            .color(GOLD)
            .strong()
            .size(13.0),
    );
    ui.add_space(4.0);
    ui.add(
        egui::ProgressBar::new(progress)
            .desired_width(ui.available_width().min(280.0))
            .fill(ACCENT.gamma_multiply(0.8)),
    );
    if state.cohort_total > 0 {
        ui.add_space(3.0);
        ui.label(
            egui::RichText::new(format!(
                "Adoption: {:.0}%  ({} / {} civilians at frontier)",
                state.adoption_fraction * 100.0,
                state.at_leading_era,
                state.cohort_total
            ))
            .color(LOCKED_DIM)
            .size(12.0),
        );
    }
}

/// Map a leading era index to the nearest named era column label.
fn leading_era_label(era: u16) -> &'static str {
    match era {
        0 => TechEra::Ancient.label(),
        1 => TechEra::Classical.label(),
        2 => TechEra::Medieval.label(),
        _ => TechEra::Industrial.label(),
    }
}

/// Render the four era columns side by side.
fn draw_era_columns(ui: &mut egui::Ui, state: &TechTreeState) {
    let eras = [
        TechEra::Ancient,
        TechEra::Classical,
        TechEra::Medieval,
        TechEra::Industrial,
    ];
    ui.horizontal_top(|ui| {
        for era in eras {
            ui.vertical(|ui| {
                ui.set_min_width(150.0);
                ui.label(
                    egui::RichText::new(era.label())
                        .color(ACCENT)
                        .strong()
                        .size(15.0),
                );
                ui.add_space(6.0);
                let mut nodes: Vec<&TechNode> =
                    state.nodes.iter().filter(|n| n.era == era).collect();
                nodes.sort_by_key(|n| n.name);
                for node in nodes {
                    tech_node_card(ui, node);
                    ui.add_space(6.0);
                }
            });
            ui.add_space(10.0);
        }
    });
}

/// Render a single tech node card; unlocked nodes glow with the accent stroke.
fn tech_node_card(ui: &mut egui::Ui, node: &TechNode) {
    let (stroke_color, text_color, badge) = if node.unlocked {
        (ACCENT, TEXT_MAIN, "✔")
    } else {
        (LOCKED_DIM.gamma_multiply(0.6), LOCKED_DIM, "🔒")
    };

    egui::Frame::NONE
        .fill(CHIP_FILL)
        .stroke(egui::Stroke::new(1.0, stroke_color))
        .corner_radius(egui::CornerRadius::same(7))
        .inner_margin(egui::Margin::symmetric(10, 7))
        .show(ui, |ui| {
            ui.set_min_width(130.0);
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(badge).color(stroke_color).size(13.0));
                ui.label(egui::RichText::new(node.name).color(text_color).size(13.0));
            });
        });
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_state_has_nodes_across_all_eras() {
        let state = TechTreeState::default();
        assert!(!state.nodes.is_empty());
        for era in [
            TechEra::Ancient,
            TechEra::Classical,
            TechEra::Medieval,
            TechEra::Industrial,
        ] {
            assert!(
                state.nodes.iter().any(|n| n.era == era),
                "expected at least one node in era {:?}",
                era.label()
            );
        }
    }

    #[test]
    fn progress_is_fraction_of_unlocked() {
        let state = TechTreeState::default();
        let expected = state.unlocked_count() as f32 / state.nodes.len() as f32;
        assert!((state.progress() - expected).abs() < f32::EPSILON);
        assert!((0.0..=1.0).contains(&state.progress()));
    }

    #[test]
    fn unlock_marks_node_and_increases_progress() {
        let mut state = TechTreeState::default();
        let before = state.unlocked_count();
        assert!(state.unlock("writing"), "writing node should exist");
        assert_eq!(state.unlocked_count(), before + 1);
        assert!(state.nodes.iter().any(|n| n.id == "writing" && n.unlocked));
    }

    #[test]
    fn unlock_unknown_id_returns_false() {
        let mut state = TechTreeState::default();
        assert!(!state.unlock("does_not_exist"));
    }

    #[test]
    fn empty_state_progress_is_zero() {
        let state = TechTreeState {
            nodes: Vec::new(),
            ..TechTreeState::default()
        };
        assert_eq!(state.progress(), 0.0);
        assert_eq!(state.unlocked_count(), 0);
    }

    #[test]
    fn era_order_is_ascending_and_unique() {
        let eras = [
            TechEra::Ancient,
            TechEra::Classical,
            TechEra::Medieval,
            TechEra::Industrial,
        ];
        let mut orders: Vec<u8> = eras.iter().map(|e| e.order()).collect();
        let sorted = {
            let mut s = orders.clone();
            s.sort_unstable();
            s
        };
        assert_eq!(orders, sorted, "era order() must be ascending");
        orders.dedup();
        assert_eq!(orders.len(), eras.len(), "era order() must be unique");
    }
}
