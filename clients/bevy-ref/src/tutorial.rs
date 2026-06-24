#![cfg(all(feature = "bevy", feature = "egui"))]

//! 6-step tutorial hint system (FR-CIV-CLIENT-011).
//! Shown bottom-centre during InGame. Space/click advances; H replays.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::menus::in_game;

const HINTS: &[&str] = &[
    "Welcome to Civis! Your civilization is emerging. Watch the minimap for faction spread. [M] cycles map modes.",
    "Press [F] to see your faction's stats - population, treasury, and government type.",
    "Events appear in the feed [N]. Disasters and diplomacy shape your world.",
    "Use [T] to research technologies. Each unlock accelerates your civilization.",
    "Open [D] to manage diplomacy - propose treaties or declare war.",
    "Press [?] anytime for all controls. Good luck!",
];

#[derive(Resource)]
pub struct TutorialState {
    pub enabled: bool,
    pub step: u8,
    pub acknowledged: bool,
}

impl Default for TutorialState {
    fn default() -> Self {
        Self { enabled: true, step: 0, acknowledged: false }
    }
}

pub struct TutorialPlugin;

impl Plugin for TutorialPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TutorialState>()
            .add_systems(Update, (handle_tutorial_keys, draw_tutorial_hint).chain().run_if(in_game));
    }
}

fn handle_tutorial_keys(
    keys: Res<ButtonInput<KeyCode>>,
    mut state: ResMut<TutorialState>,
) {
    if keys.just_pressed(KeyCode::KeyH) {
        state.enabled = true;
        state.step = 0;
        state.acknowledged = false;
        return;
    }
    if !state.enabled { return; }
    if keys.just_pressed(KeyCode::Space) {
        advance(&mut state);
    }
}

fn advance(state: &mut TutorialState) {
    if state.step as usize + 1 >= HINTS.len() {
        state.enabled = false;
    } else {
        state.step += 1;
        state.acknowledged = false;
    }
}

fn draw_tutorial_hint(
    mut contexts: EguiContexts,
    mut state: ResMut<TutorialState>,
) {
    if !state.enabled { return; }

    let hint = HINTS[state.step as usize];
    let step = state.step;
    let total = HINTS.len() as u8;

    let ctx = contexts.ctx_mut();
    let screen = ctx.screen_rect();

    let mut clicked = false;
    egui::Area::new(egui::Id::new("tutorial_hint"))
        .fixed_pos(egui::pos2(screen.center().x - 280.0, screen.max.y - 110.0))
        .show(ctx, |ui| {
            egui::Frame::none()
                .fill(egui::Color32::from_rgba_premultiplied(9, 10, 12, 230))
                .stroke(egui::Stroke::new(1.0, egui::Color32::from_rgb(126, 186, 181)))
                .rounding(egui::Rounding::same(8.0))
                .inner_margin(egui::Margin::symmetric(16.0, 10.0))
                .show(ui, |ui| {
                    ui.set_width(560.0);
                    ui.label(
                        egui::RichText::new(hint)
                            .color(egui::Color32::from_rgb(220, 230, 230))
                            .size(13.0),
                    );
                    ui.add_space(6.0);
                    ui.horizontal(|ui| {
                        ui.label(
                            egui::RichText::new(format!("{}/{}", step + 1, total))
                                .color(egui::Color32::from_rgb(126, 186, 181))
                                .size(11.0),
                        );
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            let label = if step + 1 >= total { "Got it" } else { "Next" };
                            if ui.small_button(label).clicked() {
                                clicked = true;
                            }
                            ui.label(
                                egui::RichText::new("Space or click to advance")
                                    .color(egui::Color32::from_rgb(100, 110, 120))
                                    .size(10.0),
                            );
                        });
                    });
                });
        });

    if clicked {
        advance(&mut state);
    }
}