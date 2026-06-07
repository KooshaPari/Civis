#![cfg(all(feature = "bevy", feature = "egui"))]

//! Faction Diplomacy panel for the Civis reference client.
//!
//! Provides a dark-glassmorphism overlay (matching `game_ui.rs` palette) that
//! shows all known factions and a symmetric relation matrix. Open / close with
//! `G`. The panel is purely presentational; it reads `DiplomacyState` and
//! writes nothing back to the simulation.
//!
//! # Usage
//! ```no_run
//! # use bevy::prelude::*;
//! # use civ_bevy_ref::diplomacy_ui::{DiplomacyState, DiplomacyUiPlugin};
//! let mut app = App::new();
//! app.add_plugins(DiplomacyUiPlugin);
//! // Optionally seed demo data:
//! app.insert_resource(DiplomacyState::demo());
//! ```

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::ui_theme;

// ---------------------------------------------------------------------------
// Palette — sourced from the shared `ui_theme` dark-glass language.
// ---------------------------------------------------------------------------

/// Chip / cell tint.
const CHIP_FILL: egui::Color32 = ui_theme::SURFACE;
/// Cyan accent.
const ACCENT: egui::Color32 = ui_theme::ACCENT;
/// Dimmed label colour.
const DIM: egui::Color32 = ui_theme::DIM;

// Relation colour stops
const GREEN: egui::Color32 = ui_theme::GREEN;
const GOLD: egui::Color32 = ui_theme::GOLD;
const RED: egui::Color32 = ui_theme::RED;

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A single faction entry displayed in the Diplomacy panel.
#[derive(Debug, Clone)]
pub struct DipFaction {
    /// Unique faction identifier.
    pub id: u32,
    /// Display name.
    pub name: String,
    /// Banner colour in linear `[r, g, b]` (0.0–1.0).
    pub color: [f32; 3],
    /// Current population.
    pub population: u32,
}

impl DipFaction {
    fn new(id: u32, name: impl Into<String>, color: [f32; 3], population: u32) -> Self {
        Self {
            id,
            name: name.into(),
            color,
            population,
        }
    }

    /// Convert the stored linear `[r, g, b]` to `egui::Color32` (gamma 2.2 approx).
    fn egui_color(&self) -> egui::Color32 {
        let to_u8 = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
        egui::Color32::from_rgb(
            to_u8(self.color[0]),
            to_u8(self.color[1]),
            to_u8(self.color[2]),
        )
    }
}

/// Primary resource for the Diplomacy panel.
#[derive(Resource, Debug, Clone, Default)]
pub struct DiplomacyState {
    /// All known factions.
    pub factions: Vec<DipFaction>,
    /// Square symmetric relation matrix. `relations[i][j]` is A→B stance in
    /// `−100..=100`. Diagonal should be 0 but is ignored in rendering.
    pub relations: Vec<Vec<i8>>,
    /// Whether the panel is currently visible.
    pub open: bool,
    /// Whether at least one live sim sample has populated this state.
    pub live: bool,
    /// Tick of the most recently ingested diplomacy event (dedup guard).
    last_event_tick: u64,
}

impl DiplomacyState {
    /// Build a 4-faction demo suitable for screenshots and unit tests.
    pub fn demo() -> Self {
        let factions = vec![
            DipFaction::new(0, "Red Kingdom", [0.85, 0.20, 0.20], 12_400),
            DipFaction::new(1, "Blue Republic", [0.20, 0.45, 0.90], 9_800),
            DipFaction::new(2, "Green Clans", [0.20, 0.75, 0.30], 7_250),
            DipFaction::new(3, "Yellow Guild", [0.90, 0.80, 0.10], 5_100),
        ];
        #[rustfmt::skip]
        let relations = vec![
            vec![ 0,  60, -80,  30],  // Red   → {self, Blue, Green, Yellow}
            vec![ 60,  0,  20, -55],  // Blue  → {Red, self, Green, Yellow}
            vec![-80, 20,   0,  10],  // Green → {Red, Blue, self, Yellow}
            vec![ 30,-55,  10,   0],  // Yellow→ {Red, Blue, Green, self}
        ];
        Self {
            factions,
            relations,
            open: true,
            live: false,
            last_event_tick: 0,
        }
    }

    /// Ensure the relation matrix is square and sized to the faction count,
    /// preserving existing accumulated stances. New cells default to neutral.
    fn resize_matrix(&mut self) {
        let n = self.factions.len();
        self.relations.resize(n, Vec::new());
        for row in &mut self.relations {
            row.resize(n, 0);
        }
    }

    /// Find a faction row index by its sim id.
    fn index_of(&self, id: u32) -> Option<usize> {
        self.factions.iter().position(|f| f.id == id)
    }

    /// Accumulate a single emergent diplomacy outcome into the symmetric
    /// relation matrix. Trade agreements warm the relation, conflicts cool it;
    /// values saturate within the `i8` stance range.
    fn accumulate(&mut self, a: u32, b: u32, delta: i8) {
        let (Some(i), Some(j)) = (self.index_of(a), self.index_of(b)) else {
            return;
        };
        let bump = |v: i8| v.saturating_add(delta).clamp(-100, 100);
        self.relations[i][j] = bump(self.relations[i][j]);
        self.relations[j][i] = bump(self.relations[j][i]);
    }
}

// ---------------------------------------------------------------------------
// Plugin
// ---------------------------------------------------------------------------

/// Plugin that registers `DiplomacyState` and wires the toggle + draw systems.
///
/// Does **not** re-add `EguiPlugin` — that is `GameUiPlugin`'s responsibility.
pub struct DiplomacyUiPlugin;

impl Plugin for DiplomacyUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DiplomacyState>()
            .add_systems(Update, (toggle_diplomacy_panel, sync_diplomacy_from_sim))
            .add_systems(
                EguiPrimaryContextPass,
                draw_diplomacy_panel.run_if(crate::menus::in_game),
            );
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn toggle_diplomacy_panel(keys: Res<ButtonInput<KeyCode>>, mut state: ResMut<DiplomacyState>) {
    if keys.just_pressed(KeyCode::KeyG) {
        state.open = !state.open;
    }
}

/// Faction-banner hue for a sim faction id (matches `sim_bridge::faction_color`).
fn faction_banner(id: u32) -> [f32; 3] {
    let hue = (id as f32 * 85.0) % 360.0;
    let c = Color::hsla(hue, 0.6, 0.5, 1.0).to_srgba();
    [c.red, c.green, c.blue]
}

/// Pull emergent inter-faction relations out of the running simulation.
///
/// The simulation exposes its factions (`sim.state.factions`) and a rolling
/// list of emergent [`civ_engine::DiplomacyEvent`]s via `snapshot()`. This
/// system rebuilds the faction roster (name + treasury-derived size + banner
/// colour) and folds each new trade/conflict outcome into an accumulated,
/// symmetric stance matrix. It degrades gracefully: with <2 factions the panel
/// simply shows whatever roster exists and an empty grid.
pub fn sync_diplomacy_from_sim(
    sim: Res<crate::sim_bridge::SimState>,
    mut state: ResMut<DiplomacyState>,
) {
    if !sim.is_changed() {
        return;
    }
    let world_state = &sim.0.state;

    // Rebuild the roster from the sim's faction registry (id-ordered for a
    // stable matrix layout). Population stands in via treasury magnitude.
    let mut ids: Vec<u32> = world_state.factions.keys().copied().collect();
    ids.sort_unstable();
    let factions: Vec<DipFaction> = ids
        .iter()
        .map(|&id| {
            let name = world_state
                .factions
                .get(&id)
                .cloned()
                .unwrap_or_else(|| format!("Faction {id}"));
            let treasury = world_state
                .faction_treasury
                .get(&id)
                .map(|t| t.to_f64().max(0.0) as u32)
                .unwrap_or(0);
            DipFaction::new(id, name, faction_banner(id), treasury)
        })
        .collect();

    let roster_changed = factions.len() != state.factions.len()
        || factions
            .iter()
            .zip(state.factions.iter())
            .any(|(a, b)| a.id != b.id);
    state.factions = factions;
    if roster_changed {
        // Faction set changed — reset accumulated stances to a clean matrix.
        state.relations.clear();
    }
    state.resize_matrix();

    // Fold in any emergent diplomacy outcomes newer than the last we ingested.
    let snap = sim.0.snapshot();
    for ev in &snap.diplomacy_events {
        if ev.tick <= state.last_event_tick {
            continue;
        }
        let delta: i8 = match ev.kind {
            civ_engine::DiplomacyKind::TradeAgreement => 18,
            civ_engine::DiplomacyKind::Peace => 8,
            civ_engine::DiplomacyKind::Conflict => -22,
        };
        state.accumulate(ev.faction_a, ev.faction_b, delta);
        state.last_event_tick = state.last_event_tick.max(ev.tick);
    }

    state.live = true;
}

fn draw_diplomacy_panel(mut contexts: EguiContexts, mut state: ResMut<DiplomacyState>) {
    let Ok(ctx) = contexts.ctx_mut() else { return };

    if !state.open {
        return;
    }

    let mut open = state.open;

    egui::Window::new("\u{1f91d} Diplomacy")
        .open(&mut open)
        .default_size(egui::vec2(520.0, 380.0))
        .resizable(true)
        .collapsible(false)
        .frame(ui_theme::liquid_glass_frame(
            egui::Margin::same(14),
            ui_theme::RADIUS_PANEL,
        ))
        .show(ctx, |ui| {
            // Live-data status badge.
            ui.horizontal(|ui| {
                let (badge, color) = if state.live {
                    ("● live — emergent inter-faction relations", ACCENT)
                } else {
                    ("○ waiting for simulation…", DIM)
                };
                ui.label(egui::RichText::new(badge).color(color).size(12.0));
            });
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                // Left column: faction list
                ui.vertical(|ui| {
                    ui.set_min_width(170.0);
                    ui.label(
                        egui::RichText::new("Factions")
                            .color(ACCENT)
                            .strong()
                            .size(16.0),
                    );
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                    faction_list_ui(ui, &state.factions);
                });

                ui.add_space(12.0);
                ui.separator();
                ui.add_space(12.0);

                // Right column: relation grid
                ui.vertical(|ui| {
                    ui.label(
                        egui::RichText::new("Relations")
                            .color(ACCENT)
                            .strong()
                            .size(16.0),
                    );
                    ui.add_space(4.0);
                    ui.separator();
                    ui.add_space(4.0);
                    relation_grid_ui(ui, &state.factions, &state.relations);
                });
            });
            ui_theme::liquid_glass_finish(ui.painter(), ui.min_rect(), ui_theme::RADIUS_PANEL);
        });

    // Sync close button back to state.
    state.open = open;
}

// ---------------------------------------------------------------------------
// Sub-UI helpers
// ---------------------------------------------------------------------------

/// Renders the faction list: colour swatch + name + population.
fn faction_list_ui(ui: &mut egui::Ui, factions: &[DipFaction]) {
    for faction in factions {
        ui.horizontal(|ui| {
            color_swatch(ui, faction.egui_color());
            ui.label(egui::RichText::new(&faction.name).strong());
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(
                    egui::RichText::new(format_pop(faction.population))
                        .color(DIM)
                        .small(),
                );
            });
        });
        ui.add_space(2.0);
    }
}

/// Renders the N×N relation grid with colour-coded cells and hover tooltips.
fn relation_grid_ui(ui: &mut egui::Ui, factions: &[DipFaction], relations: &[Vec<i8>]) {
    let n = factions.len();
    if n == 0 {
        ui.label(egui::RichText::new("No factions.").color(DIM).small());
        return;
    }

    let cell_size = egui::vec2(44.0, 28.0);

    // Header row: column labels (abbreviated names).
    ui.horizontal(|ui| {
        ui.add_space(64.0); // offset for row labels
        for col in factions {
            let abbrev = col.name.chars().next().unwrap_or('?').to_string();
            ui.allocate_ui(cell_size, |ui| {
                ui.centered_and_justified(|ui| {
                    ui.label(
                        egui::RichText::new(abbrev)
                            .color(col.egui_color())
                            .strong()
                            .small(),
                    );
                });
            });
        }
    });

    // Data rows.
    for (i, row_faction) in factions.iter().enumerate() {
        ui.horizontal(|ui| {
            // Row label: abbreviated name coloured by faction.
            ui.allocate_ui(egui::vec2(64.0, cell_size.y), |ui| {
                ui.centered_and_justified(|ui| {
                    let abbrev = row_faction.name.chars().next().unwrap_or('?').to_string();
                    ui.label(
                        egui::RichText::new(abbrev)
                            .color(row_faction.egui_color())
                            .strong()
                            .small(),
                    );
                });
            });

            for (j, col_faction) in factions.iter().enumerate() {
                if i == j {
                    // Diagonal: self — render a dim dash.
                    ui.allocate_ui(cell_size, |ui| {
                        ui.centered_and_justified(|ui| {
                            ui.label(egui::RichText::new("—").color(DIM).small());
                        });
                    });
                    continue;
                }

                let stance = relations
                    .get(i)
                    .and_then(|row| row.get(j))
                    .copied()
                    .unwrap_or(0);

                let (fill, text_color) = stance_colors(stance);
                let label = stance.to_string();
                let tooltip = format!(
                    "{} \u{2192} {}: {}",
                    row_faction.name,
                    col_faction.name,
                    stance_label(stance)
                );

                let resp = egui::Frame::NONE
                    .fill(fill)
                    .corner_radius(egui::CornerRadius::same(4))
                    .inner_margin(egui::Margin::symmetric(4, 2))
                    .show(ui, |ui| {
                        ui.allocate_ui(cell_size, |ui| {
                            ui.centered_and_justified(|ui| {
                                ui.label(
                                    egui::RichText::new(&label)
                                        .color(text_color)
                                        .small()
                                        .strong(),
                                );
                            });
                        });
                    })
                    .response;

                resp.on_hover_text(tooltip);
            }
        });
    }
}

/// A small coloured square swatch.
fn color_swatch(ui: &mut egui::Ui, color: egui::Color32) {
    let (rect, _) = ui.allocate_exact_size(egui::vec2(14.0, 14.0), egui::Sense::hover());
    ui.painter()
        .rect_filled(rect, egui::CornerRadius::same(3), color);
}

// ---------------------------------------------------------------------------
// Pure helpers
// ---------------------------------------------------------------------------

/// Returns a human-readable stance label for a numeric stance value.
///
/// | Range        | Label    |
/// |--------------|----------|
/// | > 50         | Allied   |
/// | 1..=50       | Friendly |
/// | 0            | Neutral  |
/// | −50..=−1     | Tense    |
/// | < −50        | At War   |
pub fn stance_label(stance: i8) -> &'static str {
    match stance {
        s if s > 50 => "Allied",
        s if s > 0 => "Friendly",
        0 => "Neutral",
        // -50 is the last "Tense" stance; -51 and below is "At War".
        s if s >= -50 => "Tense",
        _ => "At War",
    }
}

/// Returns `(background_fill, text_color)` for a given stance value.
fn stance_colors(stance: i8) -> (egui::Color32, egui::Color32) {
    match stance {
        s if s > 50 => (GREEN.gamma_multiply(0.25), GREEN),
        s if s > 0 => (GREEN.gamma_multiply(0.12), GREEN),
        0 => (CHIP_FILL, DIM),
        s if s > -50 => (GOLD.gamma_multiply(0.20), GOLD),
        _ => (RED.gamma_multiply(0.25), RED),
    }
}

/// Format a population count compactly (`12 400` → `12.4k`).
fn format_pop(pop: u32) -> String {
    if pop >= 1_000 {
        format!("{:.1}k", pop as f32 / 1_000.0)
    } else {
        pop.to_string()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stance_label_boundaries() {
        assert_eq!(stance_label(51), "Allied");
        assert_eq!(stance_label(50), "Friendly");
        assert_eq!(stance_label(1), "Friendly");
        assert_eq!(stance_label(0), "Neutral");
        assert_eq!(stance_label(-1), "Tense");
        assert_eq!(stance_label(-50), "Tense");
        assert_eq!(stance_label(-51), "At War");
        assert_eq!(stance_label(i8::MAX), "Allied");
        assert_eq!(stance_label(i8::MIN), "At War");
    }

    #[test]
    fn demo_matrix_access() {
        let state = DiplomacyState::demo();
        assert_eq!(state.factions.len(), 4);
        // Matrix must be square.
        assert_eq!(state.relations.len(), state.factions.len());
        for row in &state.relations {
            assert_eq!(row.len(), state.factions.len());
        }
        // Diagonal should be 0.
        for i in 0..state.factions.len() {
            assert_eq!(state.relations[i][i], 0, "diagonal[{i}] != 0");
        }
    }

    #[test]
    fn demo_symmetric_spot_checks() {
        let state = DiplomacyState::demo();
        // Red↔Blue: 60 / 60 (friendly both ways)
        assert_eq!(state.relations[0][1], 60);
        assert_eq!(state.relations[1][0], 60);
        // Red↔Green: −80 / −80 (at war both ways)
        assert_eq!(state.relations[0][2], -80);
        assert_eq!(state.relations[2][0], -80);
    }

    #[test]
    fn format_pop_thresholds() {
        assert_eq!(format_pop(999), "999");
        assert_eq!(format_pop(1_000), "1.0k");
        assert_eq!(format_pop(12_400), "12.4k");
    }

    #[test]
    fn stance_colors_returns_green_for_positive() {
        let (_, text) = stance_colors(75);
        assert_eq!(text, GREEN);
        let (_, text_war) = stance_colors(-99);
        assert_eq!(text_war, RED);
    }
}
