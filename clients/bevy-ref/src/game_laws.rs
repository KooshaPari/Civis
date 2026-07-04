#![cfg(all(feature = "bevy", feature = "egui"))]

//! Lightweight in-game laws panel.
//!
//! The panel is intentionally read-only: laws are foundational simulation
//! constraints in this milestone and cannot be edited here.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::ui_theme;

/// Public state shared with the toolbar button in `game_ui`.
#[derive(Resource, Debug, Default)]
pub struct GameLawsOpen(pub bool);

/// A single law line for the popup list.
#[derive(Debug, Clone, Copy)]
struct LawLine {
    category: &'static str,
    title: &'static str,
    description: &'static str,
}

/// Read-only laws panel root type.
pub struct GameLawsPanel;

/// Plugin that renders the laws popup.
pub struct GameLawsPlugin;

impl GameLawsPanel {
    /// Curated fallback laws list when direct `civ-laws` wiring is unavailable.
    fn fallback_laws() -> &'static [LawLine] {
        const FALLBACK_LAWS: &[LawLine] = &[
            LawLine {
                category: "Physical",
                title: "Conservation of Matter",
                description: "Mass is neither created nor destroyed by simulation events; terrain edits redistributes existing mass.",
            },
            LawLine {
                category: "Physical",
                title: "Energy Conservation",
                description: "Energy is tracked explicitly and only reduced through consumptive simulation sinks such as movement and production.",
            },
            LawLine {
                category: "Genomic",
                title: "Population Growth Bound",
                description: "Population growth has diminishing returns constrained by food throughput and carrying capacity.",
            },
            LawLine {
                category: "Environmental",
                title: "Biome Stability",
                description: "Biome state (soil/water/temperature) feeds back into settlement productivity and migration pressure.",
            },
            LawLine {
                category: "Environmental",
                title: "Disaster Envelope",
                description: "Disasters are pseudo-random but reproducible within seeded law tables and world state.",
            },
        ];
        FALLBACK_LAWS
    }

    /// Current law source: static fallback first, API-backed later.
    fn laws() -> &'static [LawLine] {
        // TODO: wire from crates/laws public API (`LawDb`, `Law`) when dependency is
        // added for this target.
        Self::fallback_laws()
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
                let mut current_category = "";
                for law in GameLawsPanel::laws() {
                    if current_category != law.category {
                        current_category = law.category;
                        ui.label(
                            egui::RichText::new(law.category)
                                .color(ui_theme::ACCENT)
                                .strong(),
                        );
                        ui.add_space(2.0);
                    }
                    ui.group(|ui| {
                        ui.label(egui::RichText::new(law.title).strong().color(ui_theme::TEXT));
                        ui.label(
                            egui::RichText::new(law.description)
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
