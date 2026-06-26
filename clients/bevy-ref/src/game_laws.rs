#![cfg(all(feature = "bevy", feature = "egui"))]

//! Lightweight in-game laws panel.
//!
//! The panel is intentionally read-only: laws are foundational simulation
//! constraints in this milestone and cannot be edited here.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use civ_laws::{LawDb, LawKind};
use std::sync::OnceLock;

use crate::ui_theme;

/// Public state shared with the toolbar button in `game_ui`.
#[derive(Resource, Debug)]
pub struct GameLawsOpen(pub bool);

impl Default for GameLawsOpen {
    fn default() -> Self {
        Self(false)
    }
}

/// A single law line for the popup list.
#[derive(Debug, Clone)]
struct LawLine {
    category: String,
    title: String,
    description: String,
}

/// Read-only laws panel root type.
pub struct GameLawsPanel;

/// Plugin that renders the laws popup.
pub struct GameLawsPlugin;

impl GameLawsPanel {
    fn law_line_from_db_law(law: &civ_laws::Law) -> LawLine {
        let category = match law.kind {
            LawKind::Conservation => "Physical",
            LawKind::Material => "Material",
            LawKind::FictionalExtension => "Futurism",
        };
        let inputs = if law.inputs.is_empty() {
            "none".to_string()
        } else {
            law.inputs.join(", ")
        };
        let outputs = if law.outputs.is_empty() {
            "none".to_string()
        } else {
            law.outputs.join(", ")
        };
        let losses = if law.losses.is_empty() {
            "none".to_string()
        } else {
            law.losses.join(", ")
        };
        let deps = if law.dependencies.is_empty() {
            "none".to_string()
        } else {
            law.dependencies.join(", ")
        };

        let description = format!(
            "Era {era_min}. Inputs: {inputs}. Outputs: {outputs}. Losses: {losses}. Depends on: {deps}.",
            era_min = law.era_min,
            inputs = inputs,
            outputs = outputs,
            losses = losses,
            deps = deps,
        );

        LawLine {
            category: category.to_string(),
            title: law.id.clone(),
            description,
        }
    }

    /// Curated fallback laws list when direct `civ-laws` wiring is unavailable.
    fn fallback_laws() -> Vec<LawLine> {
        vec![
            LawLine {
                category: "Physical".to_string(),
                title: "Conservation of Matter".to_string(),
                description: "Mass is neither created nor destroyed by simulation events; terrain edits redistributes existing mass.".to_string(),
            },
            LawLine {
                category: "Physical".to_string(),
                title: "Energy Conservation".to_string(),
                description: "Energy is tracked explicitly and only reduced through consumptive simulation sinks such as movement and production.".to_string(),
            },
            LawLine {
                category: "Genomic".to_string(),
                title: "Population Growth Bound".to_string(),
                description: "Population growth has diminishing returns constrained by food throughput and carrying capacity.".to_string(),
            },
            LawLine {
                category: "Environmental".to_string(),
                title: "Biome Stability".to_string(),
                description: "Biome state (soil/water/temperature) feeds back into settlement productivity and migration pressure.".to_string(),
            },
            LawLine {
                category: "Environmental".to_string(),
                title: "Disaster Envelope".to_string(),
                description: "Disasters are pseudo-random but reproducible within seeded law tables and world state.".to_string(),
            },
        ]
    }

    fn law_lines() -> &'static [LawLine] {
        static LAW_LINES: OnceLock<Vec<LawLine>> = OnceLock::new();
        LAW_LINES
            .get_or_init(|| {
                if let Ok(db) = LawDb::default_canon() {
                    db.laws.iter().map(Self::law_line_from_db_law).collect()
                } else {
                    Self::fallback_laws()
                }
            })
            .as_slice()
    }

    /// Current law source: the canonical `civ-laws` DB when available.
    fn laws() -> &'static [LawLine] {
        Self::law_lines()
    }
}

impl Plugin for GameLawsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameLawsOpen>()
            .add_systems(EguiPrimaryContextPass, draw_game_laws_panel);
    }
}

fn draw_game_laws_panel(mut contexts: EguiContexts, mut open: ResMut<GameLawsOpen>) {
    if !open.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };
    ui_theme::apply_theme(ctx);

    let mut is_open = true;
    egui::Window::new("Game Laws")
        .open(&mut is_open)
        .default_size(egui::vec2(640.0, 430.0))
        .resizable(true)
        .collapsible(false)
        .frame(ui_theme::liquid_glass_frame(egui::Margin::same(12), 12))
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Physical / Genomic / Environmental Laws")
                    .strong()
                    .color(ui_theme::ACCENT),
            );
            ui.add_space(4.0);
            ui.label(
                egui::RichText::new(
                    "These constraints are enforced by the simulation core. They are visible here for clarity, and are not player-editable in this phase except explicit knobs.",
                )
                .small()
                .color(ui_theme::DIM),
            );
            ui.add_space(8.0);
            egui::ScrollArea::vertical().show(ui, |ui| {
                let mut current_category = String::new();
                for law in GameLawsPanel::laws() {
                    if current_category != law.category {
                        current_category = law.category.clone();
                        ui.label(
                            egui::RichText::new(law.category.as_str())
                                .color(ui_theme::ACCENT)
                                .strong(),
                        );
                        ui.add_space(2.0);
                    }
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(law.title.as_str()).strong().color(ui_theme::TEXT));
                        ui.label(
                            egui::RichText::new(law.description.as_str())
                                .small()
                                .color(ui_theme::DIM),
                        );
                    });
                    ui.add_space(6.0);
                }
            });
        });

    if !is_open {
        open.0 = false;
    }
}
