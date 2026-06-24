#![cfg(all(feature = "bevy", feature = "egui"))]

//! Save / Load panel — F5 toggle, 5 slots (slot-1..slot-5), civ-server RPCs.
//!
//! RPC wire names (jsonrpc.rs):
//!   save.slot  (SaveSlot)
//!   save.load  (LoadSlot)  -- NOT "load.slot"
//!   save.list  (SaveList)

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::live_attach::LiveAttachBridge;

/// Panel visibility + last echo line.
#[derive(Resource, Default)]
pub struct SaveLoadPanel {
    pub visible: bool,
    pub last_status: String,
}

/// Registers the save/load panel systems.
pub struct SaveLoadUiPlugin;

impl Plugin for SaveLoadUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveLoadPanel>()
            .add_systems(Update, (toggle_panel, render_panel).chain());
    }
}

fn toggle_panel(keys: Res<ButtonInput<KeyCode>>, mut panel: ResMut<SaveLoadPanel>) {
    if keys.just_pressed(KeyCode::F5) {
        panel.visible = !panel.visible;
    }
}

/// Action decoded from a single egui frame so we can mutate panel after the UI closure.
enum SaveLoadAction {
    Save(u8),
    Load(u8),
    List,
}

fn render_panel(
    mut contexts: EguiContexts,
    mut panel: ResMut<SaveLoadPanel>,
    bridge: Option<Res<LiveAttachBridge>>,
) {
    if !panel.visible {
        return;
    }
    let Some(bridge) = bridge else { return };

    let ctx = contexts.ctx_mut();
    let mut open = panel.visible;
    let mut action: Option<SaveLoadAction> = None;

    egui::Window::new("Save / Load")
        .open(&mut open)
        .resizable(false)
        .min_width(220.0)
        .show(ctx, |ui| {
            ui.label("Slots persisted on the civ-server.");
            ui.separator();

            for slot in 1u8..=5 {
                ui.horizontal(|ui| {
                    ui.label(format!("Slot {slot}"));
                    if ui.button("Save").clicked() {
                        action = Some(SaveLoadAction::Save(slot));
                    }
                    if ui.button("Load").clicked() {
                        action = Some(SaveLoadAction::Load(slot));
                    }
                });
            }

            ui.separator();
            if ui.button("List Saves").clicked() {
                action = Some(SaveLoadAction::List);
            }

            if !panel.last_status.is_empty() {
                ui.separator();
                ui.label(format!("Last: {}", panel.last_status));
            }
        });

    panel.visible = open;

    // Apply RPC after egui closure so panel is no longer borrowed.
    if let Some(act) = action {
        let (json, status) = match act {
            SaveLoadAction::Save(slot) => {
                let name = format!("slot-{slot}");
                (
                    format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"save.slot","params":{{"slot_name":"{name}"}}}}"#, id = 2000 + slot as u32),
                    format!("save.slot → {name}"),
                )
            }
            SaveLoadAction::Load(slot) => {
                let name = format!("slot-{slot}");
                (
                    format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"save.load","params":{{"slot_name":"{name}"}}}}"#, id = 2010 + slot as u32),
                    format!("save.load → {name}"),
                )
            }
            SaveLoadAction::List => (
                r#"{"jsonrpc":"2.0","id":2099,"method":"save.list","params":{}}"#.to_string(),
                "save.list requested".to_string(),
            ),
        };
        bridge.client.send_rpc(json);
        panel.last_status = status;
    }
}
