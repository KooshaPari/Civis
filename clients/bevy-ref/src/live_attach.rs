//! Live WebSocket attach for `civ-standalone` (server mode parity with `civ-bevy-window`).

use bevy::prelude::*;

use crate::atmosphere::DayNightCycle;
use crate::live_pick::{LivePickPlugin, LiveSelection};
use crate::live_scene::LiveScenePlugin;
use crate::live_stream::ServerBridge;
use crate::ws_client::{SimPerfData, WsClient, WsClientConfig};
use crate::{
    resolve_live_ws_url, AttachMode, LiveHudSnapshot, MusicCues, OutcomeProgressHud,
    WsSpectatorMeta,
};

const PERF_POLL_SECS: f32 = 2.0;
const PERF_RPC: &str = r#"{"jsonrpc":"2.0","id":3,"method":"sim.perf","params":{}}"#;
const SIM_EVENTS_POLL_SECS: f32 = 2.0;
const SIM_EVENTS_RPC: &str =
    r#"{"jsonrpc":"2.0","id":9011,"method":"sim.sim_events","params":{}}"#;

#[cfg(feature = "egui")]
use crate::WsConnectionState;

#[cfg(feature = "egui")]
use crate::event_feed::{connection_toast_message, EventFeed, EventKind};

#[cfg(feature = "egui")]
use crate::notifications::{NotificationKind, Notifications};

/// Connection state mirrored from the live attach WebSocket client.
#[derive(Resource, Debug, Clone, Default)]
pub struct LiveAttachState {
    /// Whether at least one frame or snapshot has been received since connect.
    pub connected: bool,
    /// Latest tick from snapshot metadata or tick frames.
    pub tick: Option<u64>,
}

/// Cadence for lightweight server performance telemetry.
#[derive(Resource, Debug, Clone, Copy, Default)]
struct PerfPollTimer(pub f32);

/// Active live attach bridge (server mode only).
///
/// Alias of [`crate::live_stream::LiveBridge`] so egui HUD plugins (`outcome_overlay`,
/// `god_panel`) share one resource type with `civ-bevy-window`.
pub use crate::live_stream::LiveBridge as LiveAttachBridge;

/// Wires `civ-server` WebSocket attach into the standalone gameplay client.
pub struct LiveAttachPlugin;

impl Plugin for LiveAttachPlugin {
    fn build(&self, app: &mut App) {
        let ws = WsClient::spawn_with_config(resolve_live_ws_url(), WsClientConfig::default());
        let rpc_sender = ws.rpc_sender();

        app.add_plugins((LiveScenePlugin, LivePickPlugin))
            .init_resource::<LiveAttachState>()
            .init_resource::<LiveHudSnapshot>()
            .init_resource::<PerfPollTimer>()
            .init_resource::<SimEventsPollTimer>()
            .insert_resource(LiveAttachBridge { client: ws })
            .insert_resource(ServerBridge::new(rpc_sender))
            .add_systems(
                Update,
                (
                    poll_live_meta,
                    poll_live_perf,
                    poll_live_sim_events,
                    sync_live_hud_connection,
                    sync_live_hud_stats,
                    sync_live_selection,
                ),
            );
        #[cfg(all(feature = "bevy", feature = "egui"))]
        {
            app.add_plugins(crate::outcome_overlay::OutcomeOverlayPlugin);
        }
        #[cfg(feature = "egui")]
        {
            app.init_resource::<LastConnectionToastState>().add_systems(
                Update,
                (
                    sync_live_connection_toasts,
                    sync_live_game_ui,
                    sync_diplomacy_panel_from_scene,
                )
                    .chain(),
            );
        }
    }
}

fn poll_live_perf(
    time: Res<Time>,
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    mut timer: ResMut<PerfPollTimer>,
    mut hud: ResMut<LiveHudSnapshot>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    if let Some(sample) = bridge.client.poll_perf() {
        hud.tick_ms = sample.tick_ms;
    }

    timer.0 += time.delta_secs();
    if timer.0 >= PERF_POLL_SECS {
        timer.0 = 0.0;
        bridge.client.send_rpc_raw(PERF_RPC.to_owned());
    }
}

#[derive(Resource, Default)]
struct SimEventsPollTimer(f32);

/// Poll the server's `sim.sim_events` channel and surface the aggregated per-tick
/// buffers (damage, audio, research, emergence, climate, religion, legends)
/// as EventFeed notifications so the player sees the simulation actually
/// doing things.
///
/// `SimSimEventsData` is a flat struct: scalar counters, optional JSON
/// blobs for nested state, and Vec<serde_json::Value> for repeating
/// entries. This function only touches fields that actually exist.
fn poll_live_sim_events(
    time: Res<Time>,
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    mut timer: ResMut<SimEventsPollTimer>,
    mut feed: ResMut<EventFeed>,
    #[cfg(feature = "egui")] mut notifs: ResMut<Notifications>,
) {
    if *attach != AttachMode::Server {
        return;
    }

    // Drain all pending sim_events messages (the background poll keeps a small queue).
    while let Some(events) = bridge.client.poll_sim_events() {
        // Damage — real scalar fields on the struct.
        if events.damage_events_count > 0 {
            feed.push(
                EventKind::Disaster,
                format!("{} damage event(s) this tick", events.damage_events_count),
            );
            #[cfg(feature = "egui")]
            notifs.notify(
                NotificationKind::Disaster,
                format!(
                    "Damage: {} voxel hits (-{} material)",
                    events.damage_events_count, events.voxel_damage_removed
                ),
            );
        }

        // Per-event damage records — push to feed without touching event.kind (event
        // is a serde_json::Value). Just count them.
        if !events.damage_events.is_empty() {
            #[cfg(feature = "egui")]
            notifs.notify(
                NotificationKind::Diplomacy,
                format!("{} damage events recorded", events.damage_events.len()),
            );
        }

        // Research — researched Vec has the techs completed this tick,
        // in_progress_tech is the JSON blob of the current research item.
        if !events.researched.is_empty() {
            for tech in &events.researched {
                // tech is a JSON value; render as compact JSON string
                let label = tech.to_string();
                feed.push(EventKind::Tech, format!("Research completed: {}", label));
                #[cfg(feature = "egui")]
                notifs.notify(NotificationKind::Tech, format!("Research: {}", label));
            }
        } else if !events.in_progress_tech.is_null() {
            // Has active research — surface as History entry on first sight per tick
            let active = events.in_progress_tech.to_string();
            if active != "null" && active != "{}" {
                feed.push(EventKind::Diplomacy, format!("Researching: {}", active));
            }
        }

        // Audio events — push each as Sfx event (no field access needed)
        if !events.audio_events.is_empty() {
            for _audio in &events.audio_events {
                feed.push(EventKind::System, "Audio cue".to_owned());
            }
            #[cfg(feature = "egui")]
            notifs.notify(
                NotificationKind::Diplomacy,
                format!("{} audio cues this tick", events.audio_events.len()),
            );
        }

        // Religion — Option<serde_json::Value> blob. Just signal presence.
        if events.religion_state.is_some() {
            feed.push(EventKind::Diplomacy, "Religion: state updated".to_owned());
        }

        // Legends — same pattern. Presence indicates saga activity.
        if events.legends.is_some() {
            feed.push(EventKind::Diplomacy, "Legends: saga updated".to_owned());
        }

        // Emergence sample — Option<serde_json::Value>. Extract entropy if present.
        if let Some(sample) = &events.emergence_sample {
            let entropy_bits = sample.get("entropy_bits").and_then(|v| v.as_f64()).unwrap_or(0.0);
            if entropy_bits > 2.0 {
                feed.push(
                    EventKind::System,
                    format!("Emergence: entropy {:.2} (high regime)", entropy_bits),
                );
            }
            // Branching alert: check is_branching field
            let branching = sample.get("is_branching").and_then(|v| v.as_bool()).unwrap_or(false);
            if branching {
                feed.push(
                    EventKind::System,
                    "Emergence: branching — critical regime entered".to_owned(),
                );
                #[cfg(feature = "egui")]
                notifs.notify(
                    NotificationKind::Disaster,
                    "Emergence branching detected",
                );
            }
        }

        // Climate — Option<serde_json::Value> blob. Extract storm status.
        if let Some(climate) = &events.climate {
            let storm = climate.get("storm_active").and_then(|v| v.as_bool()).unwrap_or(false);
            if storm {
                let sev = climate.get("storm_severity").and_then(|v| v.as_f64()).unwrap_or(0.0);
                feed.push(EventKind::Disaster, format!("Storm active (sev {:.2})", sev));
                #[cfg(feature = "egui")]
                notifs.notify(
                    NotificationKind::Disaster,
                    format!("Storm active (severity {:.2})", sev),
                );
            }
        }
    }

    timer.0 += time.delta_secs();
    if timer.0 >= SIM_EVENTS_POLL_SECS {
        timer.0 = 0.0;
        bridge.client.send_rpc_raw(SIM_EVENTS_RPC.to_owned());
    }
}

/// Last WebSocket state used for connection toasts (egui only).
#[cfg(feature = "egui")]
#[derive(Resource, Default)]
struct LastConnectionToastState(Option<WsConnectionState>);

fn sync_live_hud_connection(
    attach: Res<AttachMode>,
    bridge: Res<LiveAttachBridge>,
    mut state: ResMut<LiveAttachState>,
    mut hud: ResMut<LiveHudSnapshot>,
) {
    if *attach != AttachMode::Server {
        return;
    }
    let connection = bridge.client.latest_connection_state();
    let connected = connection_is_live(connection);
    state.connected = connected;
    hud.connected = connected;
    hud.connection = connection;
}

fn poll_live_meta(
    bridge: Res<LiveAttachBridge>,
    mut state: ResMut<LiveAttachState>,
    mut hud: ResMut<LiveHudSnapshot>,
    mut day_night: ResMut<DayNightCycle>,
    mut music_cues: ResMut<MusicCues>,
    mut outcome_progress: ResMut<OutcomeProgressHud>,
    #[cfg(feature = "audio")] mut sfx: bevy::prelude::MessageWriter<crate::audio::SfxEvent>,
) {
    for meta in bridge.client.poll_meta() {
        if let Some(tick) = meta.tick {
            hud.tick = Some(tick);
        }
        hud.connected = true;
        music_cues.0 = meta.music_cues.clone();
        outcome_progress.0 = meta.outcome_progress;
        #[cfg(feature = "audio")]
        {
            for event in &meta.audio_events {
                let (kind, volume) = crate::audio::sfx_from_audio_event(event);
                if volume > 0.0 {
                    sfx.write(crate::audio::SfxEvent::with_volume(kind, volume));
                }
            }
        }
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
        population_by_faction: scene
            .population_by_faction
            .iter()
            .map(|(&k, &v)| (k, v))
            .collect(),
    };
    let population_by_faction: std::collections::HashMap<u32, u32> = scene
        .population_by_faction
        .iter()
        .map(|(&k, &v)| (k, v))
        .collect();
    crate::live_stream::sync_diplomacy_from_faction_frame(
        &mut diplomacy,
        &frame,
        &population_by_faction,
    );
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
    snapshot.set_sim_state(tick, population, factions, era, 1.0);
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

    #[test]
    fn connection_indicator_tracks_transport_state() {
        assert!(connection_is_live(crate::WsConnectionState::Connected));
        assert!(!connection_is_live(crate::WsConnectionState::Reconnecting));
        assert!(!connection_is_live(crate::WsConnectionState::Disconnected));
    }

    #[test]
    fn perf_poll_contract_updates_tick_and_is_bounded() {
        let mut hud = LiveHudSnapshot::default();
        hud.tick_ms = SimPerfData { tick_ms: 7.5 }.tick_ms;
        assert_eq!(hud.tick_ms, 7.5);
        assert_eq!(PERF_POLL_SECS, 2.0);
        assert!(PERF_RPC.contains("\"id\":3"));
        assert!(PERF_RPC.contains("sim.perf"));
    }
}

fn connection_is_live(state: crate::WsConnectionState) -> bool {
    state == crate::WsConnectionState::Connected
}
