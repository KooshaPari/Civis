//! Live WebSocket attach for `civ-standalone` (server mode parity with `civ-bevy-window`).

use bevy::prelude::*;

use crate::atmosphere::DayNightCycle;
use crate::live_scene::LiveScenePlugin;
use crate::ws_client::{WsClient, WsClientConfig};
use crate::{resolve_live_ws_url, AttachMode, WsSpectatorMeta};

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
        app.add_plugins(LiveScenePlugin)
            .init_resource::<LiveAttachState>()
            .insert_resource(LiveAttachBridge {
                client: WsClient::spawn_with_config(
                    resolve_live_ws_url(),
                    WsClientConfig::default(),
                ),
            })
            .add_systems(Update, poll_live_meta);
        #[cfg(feature = "egui")]
        app.add_systems(Update, sync_live_game_ui);
    }
}

fn poll_live_meta(
    bridge: Res<LiveAttachBridge>,
    mut state: ResMut<LiveAttachState>,
    mut day_night: ResMut<DayNightCycle>,
) {
    for meta in bridge.client.poll_meta() {
        apply_snapshot_meta(&mut state, &mut day_night, meta);
    }
}

#[cfg(feature = "egui")]
fn sync_live_game_ui(
    attach: Res<crate::AttachMode>,
    state: Res<LiveAttachState>,
    mut snapshot: ResMut<crate::game_ui::GameUiSnapshot>,
) {
    if *attach != crate::AttachMode::Server {
        return;
    }
    let tick = state.tick.unwrap_or(0);
    let era = tick.to_string();
    snapshot.set_sim_state(tick, 0, 0, era, 1);
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
