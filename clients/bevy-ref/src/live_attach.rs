//! Live WebSocket attach for `civ-standalone` (server mode parity with `civ-bevy-window`).

use bevy::prelude::*;

use crate::atmosphere::DayNightCycle;
use crate::live_pick::{LivePickPlugin, LiveSelection};
use crate::live_scene::LiveScenePlugin;
use crate::ws_client::{WsClient, WsClientConfig};
use crate::{resolve_live_ws_url, AttachMode, LiveHudSnapshot, WsSpectatorMeta};

/// Connection state mirrored from the live attach WebSocket client.
#[derive(Resource, Debug, Clone, Default)]
pub struct LiveAttachState {
    /// Whether at least one frame or snapshot has been received since connect.
    pub connected: bool,
    /// Latest tick from snapshot metadata or tick frames.
    pub tick: Option<u64>,
}

/// Active live attach bridge (server mode only).
#[derive(Resource)]
pub struct LiveAttachBridge {
    /// Background reconnecting WebSocket client.
    pub client: WsClient,
}

/// Wires `civ-server` WebSocket attach into the standalone gameplay client.
pub struct LiveAttachPlugin;

impl Plugin for LiveAttachPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((LiveScenePlugin, LivePickPlugin))
            .init_resource::<LiveAttachState>()
            .init_resource::<LiveHudSnapshot>()
            .insert_resource(LiveAttachBridge {
                client: WsClient::spawn_with_config(
                    resolve_live_ws_url(),
                    WsClientConfig::default(),
                ),
            })
            .add_systems(
                Update,
                (poll_live_meta, sync_live_hud_stats, sync_live_selection),
            );
        #[cfg(feature = "egui")]
        app.add_systems(Update, sync_live_game_ui);
    }
}

fn poll_live_meta(
    bridge: Res<LiveAttachBridge>,
    mut state: ResMut<LiveAttachState>,
    mut hud: ResMut<LiveHudSnapshot>,
    mut day_night: ResMut<DayNightCycle>,
) {
    for meta in bridge.client.poll_meta() {
        if let Some(tick) = meta.tick {
            hud.tick = Some(tick);
        }
        hud.connected = true;
        apply_snapshot_meta(&mut state, &mut day_night, meta);
    }
    if let Some(rtt) = bridge.client.latest_rtt_ms() {
        hud.ws_rtt_ms = Some(rtt);
    }
}

fn sync_live_hud_stats(
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    scene: Res<crate::live_stream::LiveStreamScene>,
    mut hud: ResMut<LiveHudSnapshot>,
) {
    if *attach != AttachMode::Server {
        return;
    }
    hud.sync_scene_counts(
        scene.chunks.len(),
        scene.agents.len(),
        scene.buildings.len(),
        scene.graph_parcels.len(),
    );
    if let Some(rtt) = bridge.client.latest_rtt_ms() {
        hud.ws_rtt_ms = Some(rtt);
    }
}

fn sync_live_selection(
    attach: Res<AttachMode>,
    selection: Res<LiveSelection>,
    mut hud: ResMut<LiveHudSnapshot>,
) {
    if *attach != AttachMode::Server {
        return;
    }
    hud.selected_live = selection.0;
}

#[cfg(feature = "egui")]
fn sync_live_game_ui(
    attach: Res<crate::AttachMode>,
    state: Res<LiveAttachState>,
    hud: Res<LiveHudSnapshot>,
    mut snapshot: ResMut<crate::game_ui::GameUiSnapshot>,
) {
    if *attach != crate::AttachMode::Server {
        return;
    }
    let tick = hud.tick.or(state.tick).unwrap_or(0);
    let era = tick.to_string();
    snapshot.set_sim_state(tick, 0, 0, era, 1);
    snapshot.live_hud_overlay = Some(hud.format_overlay());
}

fn apply_snapshot_meta(
    state: &mut LiveAttachState,
    day_night: &mut DayNightCycle,
    meta: WsSpectatorMeta,
) {
    state.connected = true;
    if let Some(tick) = meta.tick {
        state.tick = Some(tick);
    }
    day_night.set_from_is_day(meta.is_day);
}

/// Returns true when the standalone client should attach to `civ-server` instead of in-process sim.
#[must_use]
pub fn is_server_attach_mode(mode: AttachMode) -> bool {
    mode == AttachMode::Server
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_attach_mode_helper() {
        assert!(is_server_attach_mode(AttachMode::Server));
        assert!(!is_server_attach_mode(AttachMode::Standalone));
    }
}
