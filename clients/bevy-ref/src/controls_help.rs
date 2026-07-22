#![cfg(all(feature = "bevy", feature = "egui"))]

//! In-game controls cheat sheet (`?` / Slash). Matches the RTS shell defaults.

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::menus::in_game;
use crate::ui_theme::{DIM, PANEL_FILL, TEXT};

/// Whether the controls cheat sheet is open.
#[derive(Resource, Default)]
pub struct ControlsHelpOpen(pub bool);

/// Registers Slash/`?` toggle + overlay draw.
pub struct ControlsHelpPlugin;

impl Plugin for ControlsHelpPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ControlsHelpOpen>()
            .add_systems(Update, toggle_controls_help.run_if(in_game))
            .add_systems(
                EguiPrimaryContextPass,
                draw_controls_help.run_if(in_game),
            );
    }
}

fn toggle_controls_help(keys: Res<ButtonInput<KeyCode>>, mut open: ResMut<ControlsHelpOpen>) {
    if keys.just_pressed(KeyCode::Slash) {
        open.0 = !open.0;
    }
}

fn draw_controls_help(mut contexts: EguiContexts, open: Res<ControlsHelpOpen>) {
    if !open.0 {
        return;
    }
    let Ok(ctx) = contexts.ctx_mut() else {
        return;
    };

    egui::Window::new("Controls")
        .id(egui::Id::new("controls_help"))
        .collapsible(false)
        .resizable(false)
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .frame(
            egui::Frame::NONE
                .fill(PANEL_FILL)
                .corner_radius(egui::CornerRadius::same(10))
                .inner_margin(egui::Margin::same(16)),
        )
        .show(ctx, |ui| {
            ui.label(
                egui::RichText::new("Shell defaults")
                    .color(TEXT)
                    .strong()
                    .size(16.0),
            );
            ui.add_space(8.0);
            for (keys, action) in [
                ("Space", "Pause / resume"),
                ("Esc", "Close panels (also pause if none open)"),
                ("W A S D", "Pan camera"),
                ("Q / E", "Orbit left / right"),
                ("R / F", "Raise / lower"),
                ("Home", "Reset camera"),
                ("Scroll / = / -", "Zoom"),
                ("1\u{2013}4", "Sim speed"),
                ("Shift+1\u{2013}9", "Tool categories"),
                ("F1", "Faction HUD"),
                ("F7", "Emergence dashboard"),
                ("H", "Replay tutorial"),
                ("?", "This sheet"),
            ] {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(keys).color(TEXT).strong().monospace());
                    ui.label(egui::RichText::new(action).color(DIM));
                });
            }
            ui.add_space(10.0);
            ui.label(
                egui::RichText::new("Esc or ? to close — rebind in Settings → Controls")
                    .color(DIM)
                    .italics()
                    .size(11.0),
            );
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn controls_help_defaults_closed() {
        assert!(!ControlsHelpOpen::default().0);
    }
}
