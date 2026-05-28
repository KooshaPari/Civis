#![cfg(all(feature = "bevy", feature = "egui"))]

//! Bevy Egui gameplay HUD for the Civis reference client.
//!
//! This module keeps the HUD state isolated from the renderer binaries. The
//! UI is compile-gated behind the `egui` feature so `standalone.rs` stays
//! untouched.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};

/// Lightweight sim snapshot consumed by the HUD.
#[derive(Resource, Debug, Clone)]
pub struct GameUiSnapshot {
    /// Current simulation tick.
    pub tick: u64,
    /// Total population.
    pub population: u64,
    /// Number of factions.
    pub factions: u32,
    /// Current era label.
    pub era: String,
    /// Current tick speed multiplier.
    pub speed_multiplier: u32,
}

impl Default for GameUiSnapshot {
    fn default() -> Self {
        Self {
            tick: 0,
            population: 0,
            factions: 0,
            era: "0".to_string(),
            speed_multiplier: 1,
        }
    }
}

impl GameUiSnapshot {
    /// Update the snapshot from live sim state.
    pub fn set_sim_state(
        &mut self,
        tick: u64,
        population: u64,
        factions: u32,
        era: impl Into<String>,
        speed_multiplier: u32,
    ) {
        self.tick = tick;
        self.population = population;
        self.factions = factions;
        self.era = era.into();
        self.speed_multiplier = speed_multiplier.max(1);
    }
}

/// Tracks the currently selected entity in the HUD.
#[derive(Resource, Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SelectedEntity {
    /// Selected Bevy entity, if any.
    pub entity: Option<Entity>,
}

/// Display details for the selected entity.
#[derive(Resource, Debug, Clone, Default)]
pub struct SelectedEntityDetails {
    /// Name shown in the right panel.
    pub name: String,
    /// Faction label shown in the right panel.
    pub faction: String,
    /// Health shown in the right panel.
    pub health: String,
    /// Profession shown in the right panel.
    pub profession: String,
    /// World position shown in the right panel.
    pub position: String,
}

/// Tick speed resource used by the HUD controls.
#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct GameSpeed {
    /// Tick speed multiplier. `0` means paused.
    pub multiplier: u32,
}

impl Default for GameSpeed {
    fn default() -> Self {
        Self { multiplier: 1 }
    }
}

impl GameSpeed {
    fn display_label(self) -> String {
        match self.multiplier {
            0 => "Paused".to_string(),
            1 => "1x".to_string(),
            2 => "2x".to_string(),
            3 => "5x".to_string(),
            4 => "10x".to_string(),
            value => format!("{value}x"),
        }
    }
}

/// Plugin that renders the gameplay HUD and binds keyboard speed shortcuts.
pub struct GameUiPlugin;

impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<GameUiSnapshot>()
            .init_resource::<SelectedEntity>()
            .init_resource::<SelectedEntityDetails>()
            .init_resource::<GameSpeed>()
            .add_systems(Update, (handle_speed_shortcuts, draw_game_ui));
    }
}

fn handle_speed_shortcuts(keys: Res<ButtonInput<KeyCode>>, mut speed: ResMut<GameSpeed>) {
    if keys.just_pressed(KeyCode::Space) {
        speed.multiplier = if speed.multiplier == 0 { 1 } else { 0 };
    }
    if keys.just_pressed(KeyCode::Digit1) {
        speed.multiplier = 1;
    }
    if keys.just_pressed(KeyCode::Digit2) {
        speed.multiplier = 2;
    }
    if keys.just_pressed(KeyCode::Digit3) {
        speed.multiplier = 3;
    }
    if keys.just_pressed(KeyCode::Digit4) {
        speed.multiplier = 4;
    }
}

fn draw_game_ui(
    mut contexts: EguiContexts,
    snapshot: Res<GameUiSnapshot>,
    selected: Res<SelectedEntity>,
    details: Res<SelectedEntityDetails>,
    attach_mode: Res<crate::AttachMode>,
    live_attach: Option<Res<crate::live_attach::LiveAttachState>>,
) {
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::TopBottomPanel::top("civis_game_top_bar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            ui.label(format!("Tick #{}", snapshot.tick));
            ui.separator();
            ui.label(format!("Pop: {}", snapshot.population));
            ui.separator();
            ui.label(format!("Factions: {}", snapshot.factions));
            ui.separator();
            ui.label(format!("Era: {}", snapshot.era));
            ui.separator();
            ui.label(format!("Speed: {}", GameSpeed {
                multiplier: snapshot.speed_multiplier,
            }
            .multiplier_display()));
            if *attach_mode == crate::AttachMode::Server {
                ui.separator();
                let status = live_attach
                    .as_ref()
                    .map(|state| {
                        if state.connected {
                            "WS: connected"
                        } else {
                            "WS: connecting"
                        }
                    })
                    .unwrap_or("WS: connecting");
                ui.label(status);
            }
        });
    });

    egui::TopBottomPanel::bottom("civis_game_bottom_bar").show(ctx, |ui| {
        ui.horizontal_wrapped(|ui| {
            for label in [
                "Select",
                "Spawn Civ",
                "Spawn Building",
                "Terraform",
                "Destroy",
                "Weather",
            ] {
                if ui.button(label).clicked() {
                    // Intentionally left as a UI stub for the gameplay wave.
                }
            }
        });
    });

    if selected.entity.is_some() {
        egui::SidePanel::right("civis_game_selected_panel")
            .resizable(true)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Selection");
                ui.separator();
                ui.label(format!("Name: {}", details.name));
                ui.label(format!("Faction: {}", details.faction));
                ui.label(format!("Health: {}", details.health));
                ui.label(format!("Profession: {}", details.profession));
                ui.label(format!("Position: {}", details.position));
            });
    }
}

impl GameSpeed {
    fn multiplier_display(&self) -> String {
        self.display_label()
    }
}
