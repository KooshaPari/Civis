//! Faction Diplomacy panel for the Civis reference client.
//!
//! Provides a dark-glassmorphism overlay (matching `game_ui.rs` palette) that
//! shows all known factions and a symmetric relation matrix. Open / close with
//! `G`. The panel is purely presentational; it reads `DiplomacyState` and
//! writes nothing back to the simulation.
//!
//! # Usage
//! ```no_run
//! app.add_plugins(DiplomacyUiPlugin);
//! // Optionally seed demo data:
//! app.insert_resource(DiplomacyState::demo());
//! ```

use std::collections::HashMap;
use crossbeam_channel::Sender;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use civ_protocol_3d::{FactionStateEntry, FactionStateFrame, Government3d};
use crate::settings_ui::{GameSettings, ACTION_TOGGLE_DIPLOMACY, KeyBinding};

// ---------------------------------------------------------------------------
// Outbound RPC bridge
// ---------------------------------------------------------------------------

/// Bevy resource wrapping a cloned RPC sender so the diplomacy panel can fire
/// JSON-RPC frames without importing the binary-crate `LiveBridge` type.
///
/// Insert from `bevy_window.rs` setup: `commands.insert_resource(DiplomacyBridge::new(bridge.client.rpc_sender()));`
#[derive(Resource)]
pub struct DiplomacyBridge {
    sender: Sender<String>,
}

impl DiplomacyBridge {
    /// Wrap a cloned outbound sender from `WsClient::rpc_sender()`  .
    pub fn new(sender: Sender<String>) -> Self {
        Self { sender }
    }

    /// Enqueue a JSON-RPC text frame (fire-and-forget; drops if disconnected).
    pub fn send_rpc(&self, json: String) {
        let _ = self.sender.send(json);
    }
}

// ---------------------------------------------------------------------------
// Palette (mirrors game_ui.rs)
// ---------------------------------------------------------------------------

/// Dark glass panel fill — identical to `PANEL_FILL` in `game_ui.rs`.
const PANEL_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(17, 20, 31, 235);
/// Chip / cell tint — identical to `CHIP_FILL` in `game_ui.rs`.
const CHIP_FILL: egui::Color32 = egui::Color32::from_rgba_premultiplied(31, 37, 52, 235);
/// Cyan accent — identical to `ACCENT` in `game_ui.rs`.
const ACCENT: egui::Color32 = egui::Color32::from_rgb(80, 200, 240);
/// Dimmed label colour — identical to `DIM` in `game_ui.rs`.
const DIM: egui::Color32 = egui::Color32::from_rgb(150, 158, 178);

// Relation colour stops
const GREEN: egui::Color32 = egui::Color32::from_rgb(100, 210, 120);
const GOLD: egui::Color32 = egui::Color32::from_rgb(240, 200, 90);
const RED: egui::Color32 = egui::Color32::from_rgb(220, 80, 80);

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
#[derive(Resource, Debug, Clone)]
pub struct DiplomacyState {
    /// All known factions.
    pub factions: Vec<DipFaction>,
    /// Square symmetric relation matrix. `relations[i][j]` is A→B stance in
    /// `−100..=100`. Diagonal should be 0 but is ignored in rendering.
    pub relations: Vec<Vec<i8>>,
    /// Whether the panel is currently visible.
    pub open: bool,
    /// Faction pending war-declaration confirmation (two-click guard).
    pub pending_war_target: Option<u32>,
}

impl Default for DiplomacyState {
    fn default() -> Self {
        Self {
            factions: Vec::new(),
            relations: Vec::new(),
            open: false,
            pending_war_target: None,
        }
    }
}

impl DiplomacyState {
    /// Build [`DiplomacyState`] from a live `FactionState` wire frame.
    ///
    /// Faction rows use government labels and deterministic banner colours.
    /// Population comes from `population_by_faction` when present, otherwise a
    /// treasury-scaled stub. Relations are a square neutral matrix (`0`).
    #[must_use]
    pub fn from_faction_frame(
        frame: &FactionStateFrame,
        population_by_faction: &HashMap<u32, u32>,
    ) -> Self {
        diplomacy_state_from_faction_frame(frame, population_by_faction)
    }

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
            pending_war_target: None,
        }
    }
}

/// Maps a `FactionState` wire frame into panel rows and a neutral relation matrix.
#[must_use]
pub fn diplomacy_state_from_faction_frame(
    frame: &FactionStateFrame,
    population_by_faction: &HashMap<u32, u32>,
) -> DiplomacyState {
    let mut entries = frame.factions.clone();
    entries.sort_by_key(|entry| entry.id);
    let factions: Vec<DipFaction> = entries
        .iter()
        .map(|entry| dip_faction_from_entry(entry, population_by_faction))
        .collect::<Vec<_>>();
    let relations = neutral_relations_matrix(factions.len());
    DiplomacyState {
        factions,
        relations,
        open: false,
        pending_war_target: None,
    }
}

/// Symmetric N×N relation matrix with neutral (`0`) off-diagonal cells.
#[must_use]
pub fn neutral_relations_matrix(n: usize) -> Vec<Vec<i8>> {
    (0..n).map(|_| vec![0_i8; n]).collect()
}

/// Display name for a faction row (`"Republic #2"`).
#[must_use]
pub fn faction_display_name(entry: &FactionStateEntry) -> String {
    format!("{} #{}", government_label(&entry.government), entry.id)
}

/// Deterministic sRGB triple for a faction id (matches agent colour hashing).
#[must_use]
pub fn faction_color_from_id(id: u32) -> [f32; 3] {
    crate::agent_color_from_id(u64::from(id))
}

fn dip_faction_from_entry(
    entry: &FactionStateEntry,
    population_by_faction: &HashMap<u32, u32>,
) -> DipFaction {
    DipFaction {
        id: entry.id,
        name: faction_display_name(entry),
        color: faction_color_from_id(entry.id),
        population: population_for_faction(entry, population_by_faction),
    }
}

fn population_for_faction(
    entry: &FactionStateEntry,
    population_by_faction: &HashMap<u32, u32>,
) -> u32 {
    if let Some(count) = population_by_faction.get(&entry.id).copied() {
        return count;
    }
    let amount = entry.treasury.amount;
    if amount.is_finite() && amount > 0.0 {
        return (amount / 10.0).clamp(100.0, 999_999.0) as u32;
    }
    1_000 * (entry.id + 1)
}

fn government_label(government: &Government3d) -> &'static str {
    match government {
        Government3d::Unknown => "Faction",
        Government3d::Monarchy => "Monarchy",
        Government3d::Republic => "Republic",
        Government3d::Theocracy => "Theocracy",
        Government3d::Junta => "Junta",
        Government3d::Council => "Council",
        Government3d::Corporate => "Corporate",
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
            .add_systems(Update, toggle_diplomacy_panel)
            .add_systems(EguiPrimaryContextPass, draw_diplomacy_panel);
    }
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

fn toggle_diplomacy_panel(
    keys: Res<ButtonInput<KeyCode>>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    settings: Option<Res<GameSettings>>,
    mut state: ResMut<DiplomacyState>,
) {
    let toggle_binding = settings
        .as_ref()
        .and_then(|s| s.key_for(ACTION_TOGGLE_DIPLOMACY))
        .unwrap_or(KeyBinding::Key(KeyCode::KeyG));
    if toggle_binding.is_just_pressed(&keys, &mouse_buttons) {
        state.open = !state.open;
    }
}

fn draw_diplomacy_panel(mut contexts: EguiContexts, mut state: ResMut<DiplomacyState>, bridge: Option<Res<DiplomacyBridge>>) {
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
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .inner_margin(egui::Margin::same(14))
                .corner_radius(egui::CornerRadius::same(10)),
        )
        .show(ctx, |ui| {
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
                    faction_list_ui(ui, &mut state, bridge.as_deref());
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
        });

    // Sync close button back to state.
    state.open = open;
}

// ---------------------------------------------------------------------------
// Sub-UI helpers
// ---------------------------------------------------------------------------

/// Renders the faction list: colour swatch + name + population + action buttons.
fn faction_list_ui(ui: &mut egui::Ui, state: &mut DiplomacyState, bridge: Option<&DiplomacyBridge>) {
    for faction in state.factions.clone() {
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
        ui.horizontal(|ui| {
            if ui.small_button("Propose Treaty").clicked() {
                if let Some(b) = bridge {
                    let json = format!(
                        r#"{{"jsonrpc":"2.0","id":10,"method":"sim.diplomacy_action","params":{{"action":"propose_treaty","target_faction":{}}}}"#,
                        faction.id
                    );
                    b.send_rpc(json);
                }
            }
            let war_label = if state.pending_war_target == Some(faction.id) {
                "Confirm War?"
            } else {
                "Declare War"
            };
            if ui.small_button(war_label).clicked() {
                if state.pending_war_target == Some(faction.id) {
                    if let Some(b) = bridge {
                        let json = format!(
                            r#"{{"jsonrpc":"2.0","id":11,"method":"sim.diplomacy_action","params":{{"action":"declare_war","target_faction":{}}}}"#,
                            faction.id
                        );
                        b.send_rpc(json);
                    }
                    state.pending_war_target = None;
                } else {
                    state.pending_war_target = Some(faction.id);
                }
            }
            if ui.small_button("Offer Trade").clicked() {
                if let Some(b) = bridge {
                    let json = format!(
                        r#"{{"jsonrpc":"2.0","id":12,"method":"sim.diplomacy_action","params":{{"action":"offer_trade","target_faction":{},"amount":100}}}"#,
                        faction.id
                    );
                    b.send_rpc(json);
                }
            }
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
        s if s >= -50 => (GOLD.gamma_multiply(0.20), GOLD),
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

    /// FR-CIV-BEVY-034 — faction wire frame maps to diplomacy panel rows + neutral matrix.
    #[test]
    fn diplomacy_state_from_faction_frame_maps_entries() {
        use civ_protocol_3d::{FactionTreasury3d, Government3d};

        let frame = FactionStateFrame {
            tick: 9,
            factions: vec![
                FactionStateEntry {
                    id: 2,
                    era: 1,
                    government: Government3d::Republic,
                    treasury: FactionTreasury3d {
                        amount: 25_000.0,
                        currency: "joules".to_string(),
                    },
                },
                FactionStateEntry {
                    id: 0,
                    era: 1,
                    government: Government3d::Monarchy,
                    treasury: FactionTreasury3d::default(),
                },
            ],
        };
        let mut counts = HashMap::new();
        counts.insert(0, 42);

        let state = diplomacy_state_from_faction_frame(&frame, &counts);
        assert_eq!(state.factions.len(), 2);
        assert_eq!(state.factions[0].id, 0);
        assert_eq!(state.factions[0].name, "Monarchy #0");
        assert_eq!(state.factions[0].population, 42);
        assert_eq!(state.factions[1].id, 2);
        assert_eq!(state.factions[1].name, "Republic #2");
        assert_eq!(state.factions[1].population, 2_500);
        assert_eq!(state.relations, neutral_relations_matrix(2));
        for row in &state.relations {
            assert!(row.iter().all(|cell| *cell == 0));
        }
    }

    #[test]
    fn faction_display_name_uses_government_label() {
        use civ_protocol_3d::FactionStateEntry;

        let entry = FactionStateEntry {
            id: 7,
            government: Government3d::Corporate,
            ..Default::default()
        };
        assert_eq!(faction_display_name(&entry), "Corporate #7");
    }
}
