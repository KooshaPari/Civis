//! Live WebSocket attach for `civ-standalone` (server mode parity with `civ-bevy-window`).

use bevy::prelude::*;

use crate::atmosphere::DayNightCycle;
use crate::live_pick::{LivePickPlugin, LiveSelection};
use crate::live_scene::LiveScenePlugin;
use crate::live_stream::{LiveAgentTag, LiveBuildingTag, LiveGraphParcelTag};
use crate::ws_client::{WsClient, WsClientConfig};
use crate::{resolve_live_ws_url, AttachMode, LiveHudSnapshot, WsSpectatorMeta};

#[cfg(feature = "egui")]
use crate::WsConnectionState;

#[cfg(feature = "egui")]
use crate::event_feed::{connection_toast_message, EventFeed, EventKind};

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
                (
                    poll_live_meta,
                    sync_live_hud_connection,
                    sync_live_hud_stats,
                    sync_live_selection,
                ),
            );
        #[cfg(feature = "egui")]
        {
            app.init_resource::<LastConnectionToastState>().add_systems(
                Update,
                (
                    sync_live_connection_toasts,
                    sync_live_game_ui,
                    sync_live_selection_to_inspector,
                    sync_diplomacy_panel_from_scene,
                )
                    .chain(),
            );
        }
    }
}

/// Last WebSocket state used for connection toasts (egui only).
#[cfg(feature = "egui")]
#[derive(Resource, Default)]
struct LastConnectionToastState(Option<WsConnectionState>);

fn sync_live_hud_connection(
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    mut hud: ResMut<LiveHudSnapshot>,
) {
    if *attach != AttachMode::Server {
        return;
    }
    hud.connection = bridge.client.latest_connection_state();
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
    let civilians = crate::live_stream::civilian_hud_count(&scene);
    let factions = crate::live_stream::faction_hud_count(&scene);
    hud.sync_scene_counts(
        scene.chunks.len(),
        scene.agents.len(),
        scene.buildings.len(),
        scene.graph_parcels.len(),
        civilians,
        factions,
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
fn sync_live_connection_toasts(
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    mut feed: ResMut<EventFeed>,
    mut last: ResMut<LastConnectionToastState>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    let state = bridge.client.latest_connection_state();
    if last.0 == Some(state) {
        return;
    }

    if last.0.is_some() || state == WsConnectionState::Connected {
        feed.push(
            EventKind::System,
            connection_toast_message(state).to_string(),
        );
    }
    last.0 = Some(state);
}

#[cfg(all(feature = "bevy", feature = "egui"))]
fn sync_diplomacy_panel_from_scene(
    attach: Res<AttachMode>,
    scene: Res<crate::live_stream::LiveStreamScene>,
    mut diplomacy: ResMut<crate::diplomacy_ui::DiplomacyState>,
) {
    if *attach != AttachMode::Server {
        return;
    }
    if scene.faction_entries.is_empty() {
        return;
    }
    if !scene.is_changed() {
        return;
    }
    let frame = civ_protocol_3d::FactionStateFrame {
        tick: 0,
        factions: scene.faction_entries.clone(),
    };
    let population_by_faction: std::collections::HashMap<u32, u32> = scene.population_by_faction.iter().map(|(&k, &v)| (k, v)).collect();
    crate::live_stream::sync_diplomacy_from_faction_frame(
        &mut diplomacy,
        &frame,
        &population_by_faction,
    );
}

#[cfg(feature = "egui")]
fn sync_live_selection_to_inspector(
    attach: Res<AttachMode>,
    selection: Res<LiveSelection>,
    scene: Res<crate::live_stream::LiveStreamScene>,
    agents: Query<(&LiveAgentTag, &GlobalTransform)>,
    buildings: Query<(&LiveBuildingTag, &GlobalTransform)>,
    graph_parcels: Query<(&LiveGraphParcelTag, &GlobalTransform)>,
    mut details: ResMut<crate::game_ui::SelectedEntityDetails>,
) {
    if *attach != AttachMode::Server || !selection.is_changed() {
        return;
    }

    let Some(selected) = selection.0 else {
        *details = crate::game_ui::SelectedEntityDetails::default();
        return;
    };

    let position = live_entity_world_position(selected, &agents, &buildings, &graph_parcels);

    *details = if selected.kind == crate::LiveEntityKind::Agent {
        scene
            .civilian_entries
            .get(&selected.id)
            .map(|entry| {
                let mut rows = crate::game_ui::inspector_details_from_civilian(entry);
                if let Some(pos) = position {
                    rows.position = crate::game_ui::format_world_position(pos);
                }
                rows
            })
            .unwrap_or_else(|| {
                crate::game_ui::inspector_details_for_live_entity(selected, position)
            })
    } else {
        crate::game_ui::inspector_details_for_live_entity(selected, position)
    };
}

#[cfg(feature = "egui")]
fn live_entity_world_position(
    selected: crate::SelectedLiveEntity,
    agents: &Query<(&LiveAgentTag, &GlobalTransform)>,
    buildings: &Query<(&LiveBuildingTag, &GlobalTransform)>,
    graph_parcels: &Query<(&LiveGraphParcelTag, &GlobalTransform)>,
) -> Option<Vec3> {
    match selected.kind {
        crate::LiveEntityKind::Agent => agents
            .iter()
            .find(|(tag, _)| tag.id == selected.id)
            .map(|(_, transform)| transform.translation()),
        crate::LiveEntityKind::Building => buildings
            .iter()
            .find(|(tag, _)| tag.id == selected.id)
            .map(|(_, transform)| transform.translation()),
        crate::LiveEntityKind::GraphParcel => graph_parcels
            .iter()
            .find(|(tag, _)| tag.id == selected.id)
            .map(|(_, transform)| transform.translation()),
    }
}

#[cfg(feature = "egui")]
fn sync_live_game_ui(
    attach: Res<crate::AttachMode>,
    state: Res<LiveAttachState>,
    hud: Res<LiveHudSnapshot>,
    scene: Res<crate::live_stream::LiveStreamScene>,
    mut snapshot: ResMut<crate::game_ui::GameUiSnapshot>,
) {
    if *attach != crate::AttachMode::Server {
        return;
    }
    let tick = hud.tick.or(state.tick).unwrap_or(0);
    let population = crate::live_stream::civilian_hud_count(&scene) as u64;
    let factions = crate::live_stream::faction_hud_count(&scene) as u32;
    let era = if scene.faction_era > 0 {
        scene.faction_era.to_string()
    } else {
        tick.to_string()
    };
    snapshot.set_sim_state(tick, population, factions, era, 1);
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
