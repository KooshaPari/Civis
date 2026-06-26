#![cfg(all(feature = "bevy", feature = "egui"))]

//! Save / Load panel — F5 toggle, 5 slots (slot-1..slot-5), civ-server RPCs.
//!
//! RPC wire names (jsonrpc.rs):
//!   save.slot  (SaveSlot)
//!   save.load  (LoadSlot)  -- NOT "load.slot"
//!   save.list  (SaveList)

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use std::collections::HashMap;

use crate::live_attach::LiveAttachBridge;

/// Panel visibility + last echo line + cached slot metadata.
#[derive(Resource, Default)]
pub struct SaveLoadPanel {
    pub visible: bool,
    pub last_status: String,
    /// Cached save-list entries (populated by `List`).
    pub slots: Vec<SlotInfo>,
    /// Timestamp of the last `List` request for expiry.
    pub last_list_ms: f64,
}

/// Metadata for a single save slot returned by `save.list`.
#[derive(Debug, Clone, Default)]
pub struct SlotInfo {
    pub name: String,
    pub tick: Option<u64>,
    pub world_age: Option<String>,
    pub map_size: Option<String>,
    pub population: Option<u64>,
    pub size_bytes: Option<u64>,
    pub settled: Option<String>,
    pub religion: Option<String>,
}

fn fmt_opt(val: &Option<impl std::fmt::Display>, fallback: &str) -> String {
    val.as_ref().map(|v| v.to_string()).unwrap_or_else(|| fallback.to_string())
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
    Delete(u8),
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

                // Show metadata if cached
                if let Some(info) = panel.slots.iter().find(|s| s.name == format!("slot-{slot}")) {
                    ui.vertical(|ui| {
                        ui.label(format!("  tick: {} | pop: {} | age: {}",
                            fmt_opt(&info.tick, "-"),
                            fmt_opt(&info.population, "-"),
                            fmt_opt(&info.world_age, "-")));
                        ui.label(format!("  map: {} | size: {} | settlements: {}",
                            fmt_opt(&info.map_size, "-"),
                            info.size_bytes.map(|b| {
                                if b < 1024 { format!("{b} B") }
                                else if b < 1024*1024 { format!("{:.1} KB", b as f64 / 1024.0) }
                                else { format!("{:.1} MB", b as f64 / (1024.0*1024.0)) }
                            }).unwrap_or_else(|| "-".to_string()),
                            fmt_opt(&info.settled, "-")));
                    });
                }
            }

            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("List Saves").clicked() {
                    action = Some(SaveLoadAction::List);
                }
                if ui.button("Refresh").clicked() {
                    action = Some(SaveLoadAction::List);
                }
            });

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
            SaveLoadAction::List | SaveLoadAction::Refresh => (
                r#"{"jsonrpc":"2.0","id":2099,"method":"save.list","params":{}}"#.to_string(),
                "save.list requested".to_string(),
            ),
            SaveLoadAction::Delete(slot) => {
                let name = format!("slot-{slot}");
                (
                    format!(r#"{{"jsonrpc":"2.0","id":{id},"method":"save.delete","params":{{"slot_name":"{name}"}}}}"#, id = 2020 + slot as u32),
                    format!("save.delete → {name}"),
                )
            }
        };
        bridge.client.send_rpc_raw(json);
        panel.last_status = status;
    }
}
