#![cfg(all(feature = "bevy", feature = "egui"))]

//! Save / Load panel — F5 toggle, 5 slots (slot-1..slot-5), local session files
//! plus civ-server RPCs when live-attached.
//!
//! RPC wire names (jsonrpc.rs):
//!   save.slot  (SaveSlot)
//!   save.load  (LoadSlot)  -- NOT "load.slot"
//!   save.list  (SaveList)

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};
use civ_engine::Simulation;

use crate::game_ui::GameSpeed;
use crate::live_attach::LiveAttachBridge;
use crate::menus::{GameUiMode, WorldSetupParams};
use crate::session::{self, SessionData, SESSION_FORMAT_VERSION};
use crate::sim_bridge::SimState;
use crate::{AttachMode, LiveHudSnapshot};

#[cfg(feature = "voxel")]
use crate::voxel_sim::{VoxelSimState, WorldBuilt};

/// Panel visibility + last echo line.
#[derive(Resource, Default)]
pub struct SaveLoadPanel {
    pub visible: bool,
    pub last_status: String,
}

#[derive(Resource, Default)]
struct PendingSaveLoadAction(Option<SaveLoadAction>);

/// Registers the save/load panel systems.
pub struct SaveLoadUiPlugin;

impl Plugin for SaveLoadUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SaveLoadPanel>()
            .init_resource::<PendingSaveLoadAction>()
            .add_systems(
                Update,
                (toggle_panel, render_panel, process_save_load_actions).chain(),
            );
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
    mut pending: ResMut<PendingSaveLoadAction>,
) {
    if !panel.visible {
        return;
    }

    let ctx = contexts.ctx_mut();
    let mut open = panel.visible;
    let mut action: Option<SaveLoadAction> = None;

    egui::Window::new("Save / Load")
        .open(&mut open)
        .resizable(false)
        .min_width(220.0)
        .show(ctx, |ui| {
            ui.label("Slots persisted locally and on civ-server when attached.");
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
    if action.is_some() {
        pending.0 = action;
    }
}

fn process_save_load_actions(
    time: Res<Time>,
    attach: Res<AttachMode>,
    mut params: ResMut<WorldSetupParams>,
    mut panel: ResMut<SaveLoadPanel>,
    mut pending: ResMut<PendingSaveLoadAction>,
    mut mode: ResMut<GameUiMode>,
    mut speed: ResMut<GameSpeed>,
    sim: Option<ResMut<SimState>>,
    hud: Option<Res<LiveHudSnapshot>>,
    bridge: Option<Res<LiveAttachBridge>>,
    #[cfg(feature = "voxel")] mut built: Option<ResMut<WorldBuilt>>,
    #[cfg(feature = "voxel")] mut voxel: Option<ResMut<VoxelSimState>>,
) {
    let Some(action) = pending.0.take() else {
        return;
    };

    let stamp_secs = time.elapsed_secs().floor() as u64;

    match action {
        SaveLoadAction::Save(slot) => {
            let world_state = collect_session_data(&params, &attach, sim.as_deref(), hud.as_deref(), stamp_secs);
            match session::save(&world_state, slot) {
                Ok(()) => {
                    let stamp = session::format_stamp_hms(stamp_secs);
                    panel.last_status = format!("Saved slot-{slot} at {stamp}");
                    send_save_rpc(bridge.as_deref(), slot);
                }
                Err(err) => {
                    panel.last_status = format!("Save slot-{slot} failed: {err}");
                }
            }
        }
        SaveLoadAction::Load(slot) => match session::load(slot) {
            Ok(data) => {
                apply_session_data(
                    &data,
                    &mut params,
                    sim,
                    #[cfg(feature = "voxel")]
                    built.as_deref_mut(),
                    #[cfg(feature = "voxel")]
                    voxel.as_deref_mut(),
                );
                *mode = GameUiMode::Playing;
                speed.multiplier = 1.0;
                let stamp = session::format_stamp_hms(data.save_timestamp_unix_ms);
                panel.last_status = format!("Loaded slot-{slot} at {stamp}");
                send_load_rpc(bridge.as_deref(), slot, data.seed);
            }
            Err(err) => {
                panel.last_status = format!("Load slot-{slot} failed: {err}");
            }
        },
        SaveLoadAction::List => {
            let slots = session::list_saved_slots();
            panel.last_status = if slots.is_empty() {
                "No local saves".to_string()
            } else {
                format!("Local saves: {}", format_slot_list(&slots))
            };
            send_list_rpc(bridge.as_deref());
        }
    }
}

fn collect_session_data(
    params: &WorldSetupParams,
    attach: &AttachMode,
    sim: Option<&SimState>,
    hud: Option<&LiveHudSnapshot>,
    stamp_secs: u64,
) -> SessionData {
    let tick = current_tick(attach, sim, hud);
    SessionData {
        version: SESSION_FORMAT_VERSION,
        seed: params.seed,
        tick,
        world_setup: session::WorldSetupParams {
            seed: params.seed,
            world_size: params.world_size,
        },
        save_timestamp_unix_ms: stamp_secs,
    }
}

fn current_tick(attach: &AttachMode, sim: Option<&SimState>, hud: Option<&LiveHudSnapshot>) -> u64 {
    match attach {
        AttachMode::Server => hud.and_then(|h| h.tick).unwrap_or(0),
        AttachMode::Standalone => sim.map(|s| s.0.state.tick).unwrap_or(0),
    }
}

fn apply_session_data(
    data: &SessionData,
    params: &mut WorldSetupParams,
    sim: Option<ResMut<SimState>>,
    #[cfg(feature = "voxel")] built: Option<&mut WorldBuilt>,
    #[cfg(feature = "voxel")] voxel: Option<&mut VoxelSimState>,
) {
    params.seed = data.world_setup.seed;
    params.world_size = data.world_setup.world_size;

    if let Some(mut sim) = sim {
        sim.0 = Simulation::with_seed(data.seed);
        sim.0.state.tick = data.tick;
        sim.0.state.rng_seed = data.seed;
    }

    #[cfg(feature = "voxel")]
    {
        if let Some(voxel) = voxel {
            voxel.tick = data.tick;
        }
        if let Some(built) = built {
            built.0 = false;
        }
    }
}

fn format_slot_list(slots: &[u8]) -> String {
    slots
        .iter()
        .map(|slot| format!("slot-{slot}"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn send_save_rpc(bridge: Option<&LiveAttachBridge>, slot: u8) {
    let Some(bridge) = bridge else {
        return;
    };
    let name = format!("slot-{slot}");
    let json = format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"save.slot","params":{{"slot_name":"{name}"}}}}"#,
        id = 2000 + u32::from(slot)
    );
    bridge.client.send_rpc_raw(json);
}

fn send_load_rpc(bridge: Option<&LiveAttachBridge>, slot: u8, seed: u64) {
    let Some(bridge) = bridge else {
        return;
    };
    let name = format!("slot-{slot}");
    let load_json = format!(
        r#"{{"jsonrpc":"2.0","id":{id},"method":"save.load","params":{{"slot_name":"{name}"}}}}"#,
        id = 2010 + u32::from(slot)
    );
    bridge.client.send_rpc_raw(load_json);
    bridge
        .client
        .send_rpc("sim.reset", serde_json::json!({ "seed": seed }));
}

fn send_list_rpc(bridge: Option<&LiveAttachBridge>) {
    let Some(bridge) = bridge else {
        return;
    };
    bridge.client.send_rpc_raw(
        r#"{"jsonrpc":"2.0","id":2099,"method":"save.list","params":{}}"#.to_string(),
    );
}
