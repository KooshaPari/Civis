//! Shared `Frame3d` entity sync for live attach clients (`live_scene`, `bevy_window`).

use std::collections::{HashMap, HashSet};

#[cfg(feature = "egui")]
use crate::event_feed::{EventFeed, EventKind};
use bevy::pbr::wireframe::{Wireframe, WireframeColor};
use bevy::pbr::MeshMaterial3d;
use bevy::prelude::*;
use bevy::sprite::Text2d;
use bevy::text::{TextColor, TextFont};
use civ_protocol_3d::{
    agent_world_translation, map_build_provenance, AgentAppearanceFrame, BattleEvent3d,
    BirthEvent3d, BuildingDiffFrame, BuildingGraph, BuildingKind3d, BuildingProvenance,
    CivilianStateEntry, CivilianStateFrame, ClimateFrame, DeathEvent3d, DisasterEvent3d,
    EventFeedMessage3d, FacadeStyle, FactionStateFrame, ParcelKind, TechEvent3d, VoxelDeltaFrame,
    WorldXZ,
};
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId};

use crate::bevy_render::{apply_chunk_material, mesh_buffer_to_bevy};
use crate::game_ui::civilian_display_name;
use crate::live_ground::{live_ground_y, ChunkVoxelCache};
use crate::ws_client::WsClient;
use crate::{
    agent_color_from_id, agent_scale_multiplier, chunk_distance_from_camera, decode_chunk_id,
    mesh_lod_level, should_render_chunk, DebugRender, LiveEntityKind, SelectedLiveEntity,
    AGENT_MARKER_DEPTH, AGENT_MARKER_HEIGHT, AGENT_MARKER_WIDTH,
};

/// Bevy resource wrapping a [`WsClient`] so egui plugins can send JSON-RPC calls
/// to `civ-server` without taking a direct dependency on the transport layer.
/// The `civ-bevy-window` binary inserts a concrete instance at startup; tests
/// and headless binaries may substitute a fake.
#[derive(Resource, Clone)]
pub struct LiveBridge {
    /// The shared WebSocket client used for live JSON-RPC + frame traffic.
    pub client: WsClient,
}

/// Chunk edge length in voxels (matches kernel).
pub const LIVE_CHUNK_EDGE: usize = 16;
/// Default chunk albedo for streamed meshes.
pub const LIVE_CHUNK_BASE_COLOR: [f32; 3] = [0.72, 0.69, 0.62];

/// Per-region weather kind observed in the latest `Frame3d::Climate`.
/// Tracked as a flat `u8` tag mirroring the well-known set in `civ_planet::WeatherKind`
/// (Clear=0, Cloudy=1, Rain=2, Storm=3, Snow=4, Sandstorm=5, Fog=6) so we don't
/// pull in a transitive dep just to label the client sky tint (FR-CLIENT-render).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WeatherKindSnapshot(pub u8);

/// Flat copy of `civ_planet::Climate` for the renderer's sky/lighting pass.
/// Only the four phase floats + tick + tide offset are needed downstream;
/// carrying a `ClimateFrame` here would force the client crate to depend on
/// `civ-planet` just for the type alias (FR-CLIENT-render).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClimateSnapshot {
    /// Server tick this climate frame belongs to.
    pub tick: u64,
    /// Sun position on the 0..1 daily cycle (0 = midnight, 0.5 = noon).
    pub day_phase: f32,
    /// Position on the 0..1 yearly cycle (used for seasonal tint blending).
    pub year_phase: f32,
    /// Moon phase in 0..1 (e.g. for night sky ambient).
    pub moon_phase: f32,
    /// Tide offset, useful for shoreline shoreline vertex animation hooks.
    pub tide_offset: f32,
}

impl ClimateSnapshot {
    /// `true` when the snapshot's `day_phase` is in the daylight band
    /// (FR-CLIENT-render). Used by the Bevy presentation system to flip
    /// `ScenePresentation.is_day` as the planet rotates.
    ///
    /// Day band is `[0.20, 0.80]`; outside that (and for non-finite
    /// phases) the snapshot reads as night. `rem_euclid(1.0)` wraps
    /// out-of-range phases so the answer stays meaningful for
    /// server-side overflow.
    #[must_use]
    pub fn is_day_band(&self) -> bool {
        if !self.day_phase.is_finite() {
            return false;
        }
        let p = self.day_phase.rem_euclid(1.0);
        (0.20..0.80).contains(&p)
    }
}

/// Stable biome-to-RGB tint lookup (FR-CLIENT-render).
/// Maps a small set of `MaterialId` tags to a per-chunk base albedo so that
/// terrain streaming visibly tracks the underlying material composition
/// (e.g. a chunk dominated by `SAND` reads warm tan, a chunk dominated by
/// `SNOW` reads cool white) without making the renderer aware of the full
/// material registry.
///
/// The mapping is intentionally tiny and only covers the visually
/// distinctive materials the client cares about; anything not listed
/// resolves to [`LIVE_CHUNK_BASE_COLOR`].
#[must_use]
pub fn material_tint(material: MaterialId) -> [f32; 3] {
    use civ_voxel::{
        ASH, BEDROCK, BRICK, CLAY, COAL, CRYSTAL, DIRT, FIRE, GLASS, GRANITE, GRAVEL, ICE, MOSS,
        MUD, ORE, PACKED_DIRT, PLANT, SALT, SAND, SMOKE, SNOW, STONE, WATER, WOOD,
    };
    let id = material.0;
    match id {
        x if x == SAND.0 => [0.92, 0.83, 0.58],
        x if x == SNOW.0 => [0.95, 0.97, 1.00],
        x if x == ICE.0 => [0.78, 0.88, 0.95],
        x if x == DIRT.0 => [0.55, 0.42, 0.30],
        x if x == PACKED_DIRT.0 => [0.50, 0.38, 0.27],
        x if x == CLAY.0 => [0.72, 0.50, 0.40],
        x if x == MUD.0 => [0.42, 0.30, 0.20],
        x if x == GRAVEL.0 => [0.62, 0.58, 0.54],
        x if x == STONE.0 => [0.55, 0.55, 0.55],
        x if x == GRANITE.0 => [0.60, 0.58, 0.56],
        x if x == BEDROCK.0 => [0.32, 0.30, 0.30],
        x if x == BRICK.0 => [0.68, 0.30, 0.22],
        x if x == WOOD.0 => [0.55, 0.36, 0.18],
        x if x == COAL.0 => [0.18, 0.18, 0.20],
        x if x == ORE.0 => [0.78, 0.62, 0.28],
        x if x == CRYSTAL.0 => [0.70, 0.85, 0.95],
        x if x == GLASS.0 => [0.80, 0.88, 0.92],
        x if x == WATER.0 => [0.30, 0.50, 0.78],
        x if x == PLANT.0 => [0.30, 0.55, 0.22],
        x if x == MOSS.0 => [0.34, 0.52, 0.24],
        x if x == SALT.0 => [0.92, 0.90, 0.82],
        x if x == ASH.0 => [0.55, 0.52, 0.50],
        x if x == FIRE.0 => [0.95, 0.45, 0.10],
        x if x == SMOKE.0 => [0.45, 0.42, 0.42],
        _ => LIVE_CHUNK_BASE_COLOR,
    }
}

/// Computes a stable biome tint for a chunk by counting the most
/// common non-air material id in the voxel payload (FR-CLIENT-render).
///
/// `voxels` is expected to be the flat
/// `LIVE_CHUNK_EDGE^3` slice produced by [`CubicMesher`]. Air and the
/// non-renderable fluid/material ids are filtered out so that a chunk
/// whose bulk is empty space falls back to [`LIVE_CHUNK_BASE_COLOR`]
/// instead of "air" (which is invisible anyway).
///
/// Returns `None` when the chunk has no renderable material — callers
/// should fall back to [`LIVE_CHUNK_BASE_COLOR`].
#[must_use]
pub fn chunk_biome_tint(voxels: &[MaterialId]) -> Option<[f32; 3]> {
    use civ_voxel::{AIR, METHANE, PLASMA, SMOKE, STEAM, TOXIC_GAS};
    use std::collections::HashMap;
    let mut counts: HashMap<u8, u32> = HashMap::new();
    for v in voxels {
        if v.0 == AIR.0
            || v.0 == SMOKE.0
            || v.0 == STEAM.0
            || v.0 == METHANE.0
            || v.0 == TOXIC_GAS.0
            || v.0 == PLASMA.0
        {
            continue;
        }
        *counts.entry(v.0).or_insert(0) += 1;
    }
    let (id, _) = counts.into_iter().max_by_key(|(_, c)| *c)?;
    Some(material_tint(MaterialId(id)))
}

/// Per-agent distance LOD scale (FR-CLIENT-render). The wire agent scale
/// is further attenuated by a smooth function of camera distance so that
/// far-away agents visually compress into the terrain instead of
/// punching through it. Returns at least `min_scale` and at most
/// `max_scale` (defaults match the existing `agent_scale_multiplier`
/// envelope so HUD spacing is unaffected).
#[must_use]
pub fn agent_distance_lod(
    wire_scale: f32,
    distance: f32,
    min_scale: f32,
    max_scale: f32,
) -> f32 {
    if !wire_scale.is_finite() || !distance.is_finite() {
        return min_scale;
    }
    // `1 / (1 + d*k)` falls off gently: ~1.0 at d=0, ~0.5 at d=10, ~0.25 at d=30.
    let falloff = 1.0 / (1.0 + distance.max(0.0) * 0.05);
    let s = wire_scale * falloff;
    s.clamp(min_scale, max_scale)
}

/// Vertical offset for streamed agent markers above ground.
pub const AGENT_GROUND_Y: f32 = 0.8;
/// Vertical offset for streamed building markers above ground.
pub const BUILDING_GROUND_Y: f32 = 1.25;
/// Font size for optional agent name labels.
pub const AGENT_LABEL_FONT_SIZE: f32 = 10.0;
/// Local Y offset for agent name labels.
pub const AGENT_LABEL_Y_OFFSET: f32 = 1.05;

/// Distance culling parameters for streamed chunk meshes.
#[derive(Clone, Copy, Debug)]
pub struct StreamCulling {
    /// Camera position in world space.
    pub eye: [f32; 3],
    /// Maximum render distance in world units.
    pub max_distance: f32,
}

/// Marker for a streamed voxel chunk entity.
#[derive(Component)]
pub struct LiveChunkTag {
    /// Chunk id from the live bridge.
    pub id: ChunkId,
}

/// Fade-in state for newly spawned chunk meshes.
#[derive(Component)]
pub struct LiveChunkFade {
    /// Elapsed fade time in seconds.
    pub elapsed: f32,
    /// Base RGB before fade overlay.
    pub base_rgb: [f32; 3],
}

impl LiveChunkFade {
    /// New fade starting at zero elapsed time.
    #[must_use]
    pub fn new() -> Self {
        Self {
            elapsed: 0.0,
            base_rgb: LIVE_CHUNK_BASE_COLOR,
        }
    }
}

/// Marker for a streamed agent entity.
#[derive(Component)]
pub struct LiveAgentTag {
    /// Server agent id.
    pub id: u64,
}

/// Child label entity for [`LiveAgentTag`].
#[derive(Component)]
pub struct LiveAgentLabel;

/// Marker for a streamed building diff entry.
#[derive(Component)]
pub struct LiveBuildingTag {
    /// Building id from the diff frame.
    pub id: u64,
}

/// Marker for a streamed building-graph parcel.
#[derive(Component)]
pub struct LiveGraphParcelTag {
    /// Parcel id (raw bits).
    pub id: u64,
}

/// Entity maps and voxel cache shared by live attach renderers.
#[derive(Resource)]
pub struct LiveStreamScene {
    pub chunks: HashMap<u64, Entity>,
    pub chunk_voxels: ChunkVoxelCache,
    pub agents: HashMap<u64, Entity>,
    pub buildings: HashMap<u64, Entity>,
    pub graph_parcels: HashMap<u64, Entity>,
    pub agent_materials: HashMap<u64, Handle<StandardMaterial>>,
    pub building_materials: HashMap<u64, Handle<StandardMaterial>>,
    pub graph_parcel_materials: HashMap<u64, Handle<StandardMaterial>>,
    pub building_provenance: BuildingProvenance,
    /// Civilian entity ids from the latest `Frame3d::CivilianState` (HUD counts).
    pub civilian_ids: HashSet<u64>,
    /// Latest civilian payloads keyed by stable civilian id (inspector + HUD).
    pub civilian_entries: HashMap<u64, CivilianStateEntry>,
    /// Faction ids from the latest `Frame3d::FactionState` (HUD counts).
    pub factions: HashSet<u32>,
    /// Latest faction payloads (diplomacy panel + HUD era).
    pub faction_entries: Vec<civ_protocol_3d::FactionStateEntry>,
    /// Max era from the latest faction state frame (HUD era chip).
    pub faction_era: u16,
    /// Civilian count per faction id from the latest FactionState frame (FR-CIV-PROTO-001).
    pub population_by_faction: std::collections::BTreeMap<u32, u32>,
    /// Latest climate snapshot from `Frame3d::Climate` (FR-CLIENT-render).
    /// Driven by the planetary sim and read by client-side lighting/sky systems.
    /// Storing a flat copy (instead of [`civ_protocol_3d::ClimateFrame`])
    /// keeps `civ-bevy-ref` free of a `civ-planet` dep — the renderer only
    /// needs the four phase floats + tide offset for sky/sun blending.
    pub latest_climate: Option<ClimateSnapshot>,
    /// Last tick at which a climate frame was observed (matches
    /// `latest_climate.as_ref().map(|f| f.tick)`; tracked explicitly so the
    /// day/night HUD + skybox blend systems don't need to clone the frame).
    pub latest_climate_tick: Option<u64>,
    /// Last observed per-region weather grid (FR-CLIENT-render). Kept as the
    /// raw `WeatherCell` list so the renderer can drive per-region particle
    /// FX / sky tinting without re-asking the server.
    #[cfg_attr(not(feature = "bevy"), allow(dead_code))]
    pub latest_weather_kinds: Vec<WeatherKindSnapshot>,
}

impl Default for LiveStreamScene {
    fn default() -> Self {
        Self {
            chunks: HashMap::default(),
            chunk_voxels: ChunkVoxelCache::default(),
            agents: HashMap::default(),
            buildings: HashMap::default(),
            graph_parcels: HashMap::default(),
            agent_materials: HashMap::default(),
            building_materials: HashMap::default(),
            graph_parcel_materials: HashMap::default(),
            // BuildingProvenance does not derive Default in the wire crate
            // (intentional — every diff carries an explicit tag). The renderer
            // simply tracks the last-observed provenance, starting Procedural.
            building_provenance: BuildingProvenance::Procedural,
            civilian_ids: HashSet::default(),
            civilian_entries: HashMap::default(),
            factions: HashSet::default(),
            faction_entries: Vec::new(),
            faction_era: 0,
            population_by_faction: std::collections::BTreeMap::new(),
            latest_climate: None,
            latest_climate_tick: None,
            latest_weather_kinds: Vec::new(),
        }
    }
}

/// Replaces civilian HUD tracking from a server `CivilianState` snapshot (may truncate at 256).
pub fn apply_civilian_state_frame(scene: &mut LiveStreamScene, frame: CivilianStateFrame) {
    scene.civilian_ids.clear();
    scene.civilian_entries.clear();
    for entry in frame.civilians {
        scene.civilian_ids.insert(entry.id);
        scene.civilian_entries.insert(entry.id, entry);
    }
}

/// Replaces faction HUD tracking from a server `FactionState` snapshot.
pub fn apply_faction_state_frame(scene: &mut LiveStreamScene, frame: FactionStateFrame) {
    scene.factions.clear();
    scene.faction_era = frame
        .factions
        .iter()
        .map(|entry| entry.era)
        .max()
        .unwrap_or(0);
    scene.faction_entries = frame.factions;
    scene.faction_entries.sort_by_key(|entry| entry.id);
    scene
        .factions
        .extend(scene.faction_entries.iter().map(|entry| entry.id));
    scene.population_by_faction = frame.population_by_faction;
}

/// Records a `Frame3d::Climate` snapshot into [`LiveStreamScene`]
/// (FR-CLIENT-render). Flattens the wire `ClimateFrame` into a
/// [`ClimateSnapshot`] + per-region `WeatherKind` list so the renderer can
/// drive sky/sun/ambient tinting without touching `civ-planet` directly.
///
/// Stale frames (older than the last-seen tick) are dropped — climate frames
/// arrive at a much lower cadence than voxel deltas, so this is the only
/// ordering check we need to prevent the day/night HUD from flickering
/// backwards on retransmits.
pub fn apply_climate_frame(scene: &mut LiveStreamScene, frame: ClimateFrame) {
    let new_tick = frame.climate.tick;
    if let Some(prev) = scene.latest_climate_tick {
        if new_tick < prev {
            return;
        }
    }
    let snapshot = ClimateSnapshot {
        tick: frame.climate.tick,
        day_phase: frame.climate.day_phase,
        year_phase: frame.climate.year_phase,
        moon_phase: frame.climate.moon_phase,
        tide_offset: frame.climate.tide_offset,
    };
    let weather = frame
        .weather
        .iter()
        .map(|cell| WeatherKindSnapshot(cell.kind as u8))
        .collect();
    scene.latest_climate = Some(snapshot);
    scene.latest_climate_tick = Some(snapshot.tick);
    scene.latest_weather_kinds = weather;
}

/// Convenience getter used by client render systems (FR-CLIENT-render).
/// Returns the most recent [`ClimateSnapshot`] together with the per-region
/// weather tags, if any frame has been observed.
#[must_use]
pub fn latest_climate(
    scene: &LiveStreamScene,
) -> Option<(ClimateSnapshot, &[WeatherKindSnapshot])> {
    scene
        .latest_climate
        .map(|snap| (snap, scene.latest_weather_kinds.as_slice()))
}

/// Maps a `FactionState` wire frame into [`DiplomacyState`] for the egui panel.
///
/// Relation cells are neutral (`0`) until a diplomacy simulation feeds stance data.
#[cfg(all(feature = "bevy", feature = "egui"))]
pub fn sync_diplomacy_from_faction_frame(
    diplomacy: &mut crate::diplomacy_ui::DiplomacyState,
    frame: &FactionStateFrame,
    population_by_faction: &HashMap<u32, u32>,
) {
    let open = diplomacy.open;
    *diplomacy =
        crate::diplomacy_ui::diplomacy_state_from_faction_frame(frame, population_by_faction);
    diplomacy.open = open;
}

/// HUD population count from the latest civilian state snapshot.
#[must_use]
pub fn civilian_hud_count(scene: &LiveStreamScene) -> usize {
    scene.civilian_ids.len()
}

/// HUD faction count from the latest faction state snapshot.
#[must_use]
pub fn faction_hud_count(scene: &LiveStreamScene) -> usize {
    scene.factions.len()
}

/// Copies civilian/faction HUD counts from the latest state frames into [`LiveHudSnapshot`].
pub fn sync_live_hud_civilian_faction_counts(
    hud: &mut crate::LiveHudSnapshot,
    scene: &LiveStreamScene,
) {
    hud.civilian_count = civilian_hud_count(scene);
    hud.faction_count = faction_hud_count(scene);
}

/// Resolves a live viewport pick id to the latest [`CivilianStateEntry`].
///
/// Primary lookup uses stable civilian ids in [`LiveStreamScene::civilian_entries`].
/// Fallback: when the picked id is a streamed agent key in [`LiveStreamScene::agents`]
/// but differs from the stable civilian id (legacy servers used ECS entity ids in
/// `AgentAppearanceUpdate::agent_id` while `CivilianStateEntry::id` is
/// `AgentCivilian::id`; item 63 aligns server agent ids), return the sole civilian
/// wire entry when exactly one agent marker and one civilian payload are tracked.
#[must_use]
pub fn resolve_civilian_for_live_pick(
    picked_id: u64,
    scene: &LiveStreamScene,
) -> Option<&CivilianStateEntry> {
    if let Some(entry) = scene.civilian_entries.get(&picked_id) {
        return Some(entry);
    }
    if scene.agents.contains_key(&picked_id)
        && scene.agents.len() == 1
        && scene.civilian_entries.len() == 1
    {
        return scene.civilian_entries.values().next();
    }
    None
}

/// One-line pick detail for HUD overlay (name, profession, health% when wire data exists).
#[must_use]
pub fn format_live_pick_hud_line(
    selection: Option<SelectedLiveEntity>,
    scene: &LiveStreamScene,
) -> Option<String> {
    let selected = selection?;
    match selected.kind {
        LiveEntityKind::Agent => resolve_civilian_for_live_pick(selected.id, scene)
            .map(format_civilian_pick_hud_line)
            .or_else(|| Some(format!("Agent #{}", selected.id))),
        LiveEntityKind::Building => Some(format!("Building #{}", selected.id)),
        LiveEntityKind::GraphParcel => Some(format!("Parcel #{}", selected.id)),
    }
}

fn format_civilian_pick_hud_line(entry: &CivilianStateEntry) -> String {
    let name = if entry.genome_summary.summary.is_empty() {
        format!("Civilian #{}", entry.id)
    } else {
        entry.genome_summary.summary.clone()
    };
    let profession = if entry.profession.is_empty() {
        "—".to_string()
    } else {
        entry.profession.clone()
    };
    let health = format!("{:.0}%", entry.health.clamp(0.0, 1.0) * 100.0);
    format!("{name} | {profession} | {health}")
}

/// Maps one wire event-feed message to an egui toast category and human-readable text.
#[cfg(feature = "egui")]
#[must_use]
pub fn map_event_feed_message(msg: &EventFeedMessage3d) -> (EventKind, String) {
    match msg {
        EventFeedMessage3d::Birth(ev) => (EventKind::Birth, format_birth_event(ev)),
        EventFeedMessage3d::Death(ev) => (EventKind::Death, format_death_event(ev)),
        EventFeedMessage3d::Tech(ev) => (EventKind::Tech, format_tech_event(ev)),
        EventFeedMessage3d::Battle(ev) => (EventKind::Battle, format_battle_event(ev)),
        EventFeedMessage3d::Disaster(ev) => (EventKind::Disaster, format_disaster_event(ev)),
    }
}

/// Pushes all messages from an `EventFeed` frame into the HUD toast feed.
#[cfg(feature = "egui")]
pub fn apply_event_feed_frame(feed: &mut EventFeed, frame: civ_protocol_3d::EventFeedFrame) {
    for msg in frame.events {
        let (kind, text) = map_event_feed_message(&msg);
        feed.push(kind, text);
    }
}

/// Maximum characters for [`crate::LiveHudSnapshot::last_event`] overlay text.
pub const LIVE_HUD_EVENT_MAX_CHARS: usize = 80;

/// Truncates event-feed HUD text to [`LIVE_HUD_EVENT_MAX_CHARS`] (Unicode-safe).
#[must_use]
pub fn truncate_event_feed_hud_line(text: &str) -> String {
    let char_count = text.chars().count();
    if char_count <= LIVE_HUD_EVENT_MAX_CHARS {
        return text.to_string();
    }
    let mut truncated: String = text
        .chars()
        .take(LIVE_HUD_EVENT_MAX_CHARS.saturating_sub(1))
        .collect();
    truncated.push('…');
    truncated
}

/// Updates [`crate::LiveHudSnapshot::last_event`] from the latest wire event-feed frame.
///
/// Keeps the last one or two formatted messages (joined with ` · `) for the HUD line.
pub fn push_event_feed_to_hud_summary(
    hud: &mut crate::LiveHudSnapshot,
    frame: &civ_protocol_3d::EventFeedFrame,
) {
    if frame.events.is_empty() {
        return;
    }
    let formatted: Vec<String> = frame.events.iter().map(format_event_feed_message).collect();
    let summary = match formatted.len() {
        1 => formatted[0].clone(),
        n => formatted[n.saturating_sub(2)..].join(" · "),
    };
    hud.last_event = Some(truncate_event_feed_hud_line(&summary));
}

/// Human-readable text for logging (`civ-bevy-window`, no egui).
#[must_use]
pub fn format_event_feed_message(msg: &EventFeedMessage3d) -> String {
    match msg {
        EventFeedMessage3d::Birth(ev) => format_birth_event(ev),
        EventFeedMessage3d::Death(ev) => format_death_event(ev),
        EventFeedMessage3d::Tech(ev) => format_tech_event(ev),
        EventFeedMessage3d::Battle(ev) => format_battle_event(ev),
        EventFeedMessage3d::Disaster(ev) => format_disaster_event(ev),
    }
}

fn format_birth_event(ev: &BirthEvent3d) -> String {
    let who = if ev.species.is_empty() {
        format!("Entity #{}", ev.entity_id)
    } else {
        ev.species.clone()
    };
    let mut text = format!("{who} was born");
    if ev.faction_id != 0 {
        text.push_str(&format!(" (faction {})", ev.faction_id));
    }
    if let Some(pos) = &ev.position {
        text.push_str(&format!(" at ({:.0}, {:.0})", pos.x, pos.z));
    }
    text
}

fn format_death_event(ev: &DeathEvent3d) -> String {
    if !ev.cause.is_empty() {
        let mut text = ev.cause.clone();
        if ev.entity_id != 0 {
            text.push_str(&format!(" (entity #{})", ev.entity_id));
        }
        return text;
    }
    let mut text = format!("Entity #{} died", ev.entity_id);
    if ev.faction_id != 0 {
        text.push_str(&format!(" (faction {})", ev.faction_id));
    }
    if let Some(pos) = &ev.position {
        text.push_str(&format!(" at ({:.0}, {:.0})", pos.x, pos.z));
    }
    text
}

fn format_tech_event(ev: &TechEvent3d) -> String {
    if !ev.tech.is_empty() {
        let mut text = ev.tech.clone();
        if ev.faction_id != 0 {
            text = format!("Faction {} unlocked {text}", ev.faction_id);
        } else {
            text = format!("Unlocked {text}");
        }
        if ev.era != 0 {
            text.push_str(&format!(" (era {})", ev.era));
        }
        return text;
    }
    if ev.faction_id != 0 {
        format!("Faction {} reached era {}", ev.faction_id, ev.era)
    } else {
        format!("Technology unlocked (era {})", ev.era)
    }
}

fn format_battle_event(ev: &BattleEvent3d) -> String {
    let sides = match (ev.attacker_faction, ev.defender_faction) {
        (0, 0) => String::new(),
        (a, 0) => format!("faction {a}"),
        (0, d) => format!("faction {d}"),
        (a, d) => format!("faction {a} vs faction {d}"),
    };
    let mut text = if sides.is_empty() {
        "Battle".to_string()
    } else {
        format!("Battle: {sides}")
    };
    if !ev.outcome.is_empty() {
        text.push_str(&format!(" — {}", ev.outcome.replace('_', " ")));
    }
    if let Some(pos) = &ev.position {
        text.push_str(&format!(" at ({:.0}, {:.0})", pos.x, pos.z));
    }
    text
}

fn format_disaster_event(ev: &DisasterEvent3d) -> String {
    let label = if ev.disaster_kind.is_empty() {
        "Disaster".to_string()
    } else {
        ev.disaster_kind.replace('_', " ")
    };
    let mut text = if ev.severity > 0.0 {
        format!("{label} (severity {:.0}%)", ev.severity * 100.0)
    } else {
        label
    };
    if let Some(pos) = &ev.position {
        text.push_str(&format!(" at ({:.0}, {:.0})", pos.x, pos.z));
    }
    text
}

/// Shared marker meshes for agents and buildings.
#[derive(Resource, Clone)]
pub struct LiveStreamMeshes {
    pub agent_mesh: Handle<Mesh>,
    pub building_mesh: Handle<Mesh>,
}

/// Whether to spawn floating agent id labels.
#[derive(Clone, Copy, Default)]
pub struct AgentLabelConfig {
    pub enabled: bool,
}

impl AgentLabelConfig {
    /// Labels enabled (standalone / window default).
    #[must_use]
    pub const fn enabled() -> Self {
        Self { enabled: true }
    }
}

/// Builds default agent/building marker meshes.
#[must_use]
pub fn default_stream_meshes(meshes: &mut Assets<Mesh>) -> LiveStreamMeshes {
    LiveStreamMeshes {
        agent_mesh: meshes.add(Cuboid::new(
            AGENT_MARKER_WIDTH,
            AGENT_MARKER_HEIGHT,
            AGENT_MARKER_DEPTH,
        )),
        building_mesh: meshes.add(Cuboid::new(2.0, 2.5, 2.0)),
    }
}

/// Default wireframe-off debug state for the live window.
#[must_use]
pub fn default_stream_wireframe() -> DebugRender {
    DebugRender::default()
}

fn chunk_transform(id: ChunkId) -> Transform {
    let (x, y, z) = decode_chunk_id(id);
    Transform::from_xyz(
        x as f32 * LIVE_CHUNK_EDGE as f32,
        y as f32 * LIVE_CHUNK_EDGE as f32,
        z as f32 * LIVE_CHUNK_EDGE as f32,
    )
}

/// Applies a voxel delta frame (caches voxels, meshes in-range chunks).
pub fn apply_voxel_delta_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    mesh_assets: &mut Assets<Mesh>,
    material_assets: &mut Assets<StandardMaterial>,
    culling: StreamCulling,
    debug: &DebugRender,
    delta: VoxelDeltaFrame,
    wireframe_line_color: Option<Color>,
) {
    for chunk in delta.deltas {
        let chunk_id = chunk.event.chunk_id;
        if chunk.voxels.len() == LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE {
            scene.chunk_voxels.insert(chunk_id, chunk.voxels.clone());
        }

        if !should_render_chunk(chunk_id, culling.eye, culling.max_distance) {
            if let Some(entity) = scene.chunks.remove(&chunk_id.0) {
                commands.entity(entity).despawn();
            }
            continue;
        }

        if chunk.voxels.len() != LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE {
            continue;
        }

        let chunk_view = ChunkView {
            id: chunk.event.chunk_id,
            voxels: &chunk.voxels,
        };
        let distance =
            chunk_distance_from_camera(chunk.event.chunk_id, culling.eye, LIVE_CHUNK_EDGE as f32);
        let lod = LodLevel(mesh_lod_level(distance));
        let Ok(mesh_buffer) = CubicMesher::mesh_cubic(chunk_view, lod) else {
            continue;
        };
        let mesh = mesh_assets.add(mesh_buffer_to_bevy(&mesh_buffer));
        let mut material = StandardMaterial {
            perceptual_roughness: 0.85,
            metallic: 0.0,
            ..default()
        };
        // FR-CLIENT-render: pick a biome-derived base color from the chunk's
        // voxel composition so the streamed mesh visibly tracks the
        // underlying material (sand reads tan, snow reads cool, etc.).
        // Falls back to `LIVE_CHUNK_BASE_COLOR` for empty / unmapped chunks.
        let base_rgb =
            chunk_biome_tint(&chunk.voxels).unwrap_or(LIVE_CHUNK_BASE_COLOR);
        apply_chunk_material(
            &mut material,
            base_rgb,
            debug.wireframe,
            Some(0.0),
        );
        let material_handle = material_assets.add(material);
        let transform = chunk_transform(chunk.event.chunk_id);

        let entity = *scene
            .chunks
            .entry(chunk.event.chunk_id.0)
            .or_insert_with(|| {
                commands
                    .spawn((
                        LiveChunkTag {
                            id: chunk.event.chunk_id,
                        },
                        Transform::default(),
                    ))
                    .id()
            });
        commands.entity(entity).insert((
            Mesh3d(mesh),
            MeshMaterial3d(material_handle),
            transform,
            LiveChunkFade::new(),
        ));

        if let Some(color) = wireframe_line_color {
            commands
                .entity(entity)
                .insert((Wireframe, WireframeColor { color }));
        } else {
            commands
                .entity(entity)
                .remove::<Wireframe>()
                .remove::<WireframeColor>();
        }
    }
}

/// Applies an agent appearance frame to the live stream scene.
pub fn apply_agent_appearance_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    agents: AgentAppearanceFrame,
) {
    apply_agent_appearance_frame_with_labels(
        commands,
        scene,
        materials,
        meshes,
        agents,
        AgentLabelConfig::enabled(),
    );
}

/// Applies an agent appearance frame with optional name labels.
pub fn apply_agent_appearance_frame_with_labels(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    agents: AgentAppearanceFrame,
    labels: AgentLabelConfig,
) {
    apply_agent_appearance_frame_with_labels_and_eye(
        commands,
        scene,
        materials,
        meshes,
        agents,
        labels,
        None,
    );
}

/// Applies an agent appearance frame with optional name labels and a
/// camera `eye` position (FR-CLIENT-render).
///
/// When `camera_eye` is `Some`, the per-agent render scale is further
/// attenuated by [`agent_distance_lod`] so far-away agents visually
/// compress into the terrain instead of punching through it. `None`
/// preserves the original (uniform) marker scale so existing callers
/// (and the live_scene helper) keep their current behavior.
pub fn apply_agent_appearance_frame_with_labels_and_eye(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    agents: AgentAppearanceFrame,
    labels: AgentLabelConfig,
    camera_eye: Option<[f32; 3]>,
) {
    for update in agents.updates {
        let rgb = agent_color_from_id(update.agent_id);
        let wire_scale = agent_scale_multiplier(update.scale);
        let (x, _, z) = agent_world_translation(&update, 0.0);
        if !x.is_finite() || !z.is_finite() || !wire_scale.is_finite() {
            continue;
        }
        let y = live_ground_y(&scene.chunk_voxels, x, z, AGENT_GROUND_Y);
        if !y.is_finite() {
            continue;
        }
        // FR-CLIENT-render: distance-LOD the agent marker scale when we know
        // the camera position. Clamp envelope matches the existing
        // `agent_scale_multiplier` (i.e. no upper bound, lower bound 0.05 so
        // agents never disappear entirely).
        let scale = match camera_eye {
            Some(eye) => {
                let dx = x - eye[0];
                let dz = z - eye[2];
                let d = (dx * dx + dz * dz).sqrt();
                agent_distance_lod(wire_scale, d, 0.05, f32::MAX)
            }
            None => wire_scale,
        };
        let transform = Transform::from_xyz(x, y, z).with_scale(Vec3::splat(scale));

        let material_handle = scene
            .agent_materials
            .entry(update.agent_id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color: Color::srgb(rgb[0], rgb[1], rgb[2]),
                    perceptual_roughness: 0.7,
                    metallic: 0.0,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = Color::srgb(rgb[0], rgb[1], rgb[2]);
        }

        let entity = *scene.agents.entry(update.agent_id).or_insert_with(|| {
            let entity = commands
                .spawn(LiveAgentTag {
                    id: update.agent_id,
                })
                .id();
            if labels.enabled {
                let label = scene
                    .civilian_entries
                    .get(&update.agent_id)
                    .map(civilian_display_name)
                    .unwrap_or_else(|| format!("#{}", update.agent_id));
                commands.entity(entity).with_children(|parent| {
                    parent.spawn((
                        Text2d::new(label),
                        TextFont::from_font_size(AGENT_LABEL_FONT_SIZE),
                        TextColor(Color::srgba(0.95, 0.96, 0.98, 0.92)),
                        Transform::from_xyz(0.0, AGENT_LABEL_Y_OFFSET, 0.0),
                        LiveAgentLabel,
                    ));
                });
            }
            entity
        });

        commands.entity(entity).insert((
            Mesh3d(meshes.agent_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

/// Refreshes floating agent labels from the latest civilian snapshot.
#[cfg(feature = "bevy")]
pub fn sync_agent_labels_from_civilians(
    scene: Res<LiveStreamScene>,
    agents: Query<(&LiveAgentTag, &Children)>,
    mut labels: Query<&mut Text2d, With<LiveAgentLabel>>,
) {
    for (agent, children) in &agents {
        let label = scene
            .civilian_entries
            .get(&agent.id)
            .map(civilian_display_name)
            .unwrap_or_else(|| format!("#{}", agent.id));
        for &child in children.iter() {
            let Ok(mut text) = labels.get_mut(child) else {
                continue;
            };
            *text = Text2d::new(label.clone());
            break;
        }
    }
}

/// Applies a building diff frame (optional graph + building entries).
pub fn apply_building_diff_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    frame: BuildingDiffFrame,
) {
    let BuildingDiffFrame {
        provenance,
        buildings,
        graph,
        ..
    } = frame;

    scene.building_provenance = provenance;

    if let Some(graph) = graph {
        apply_building_graph_frame(commands, scene, materials, meshes, graph);
    }

    if buildings.is_empty() {
        return;
    }

    let incoming: HashSet<u64> = buildings.iter().map(|entry| entry.id).collect();
    for (id, entity) in scene.buildings.clone() {
        if !incoming.contains(&id) {
            commands.entity(entity).despawn();
            scene.buildings.remove(&id);
            scene.building_materials.remove(&id);
        }
    }

    for entry in buildings {
        let (base_color, emissive, roughness) = building_material_style(entry.kind, provenance);
        if !entry.position.x.is_finite() || !entry.position.z.is_finite() {
            continue;
        }
        let material_handle = scene
            .building_materials
            .entry(entry.id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color,
                    emissive: emissive.into(),
                    perceptual_roughness: roughness,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = base_color;
            material.emissive = emissive.into();
            material.perceptual_roughness = roughness;
        }

        let ground = live_ground_y(
            &scene.chunk_voxels,
            entry.position.x,
            entry.position.z,
            BUILDING_GROUND_Y,
        );
        if !ground.is_finite() {
            continue;
        }
        let transform = Transform::from_xyz(entry.position.x, ground, entry.position.z);
        let entity = *scene
            .buildings
            .entry(entry.id)
            .or_insert_with(|| commands.spawn(LiveBuildingTag { id: entry.id }).id());
        commands.entity(entity).insert((
            Mesh3d(meshes.building_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

/// Applies a building graph snapshot (parcels + facades).
pub fn apply_building_graph_frame(
    commands: &mut Commands,
    scene: &mut LiveStreamScene,
    materials: &mut Assets<StandardMaterial>,
    meshes: &LiveStreamMeshes,
    graph: BuildingGraph,
) {
    let incoming: HashSet<u64> = graph.parcels.iter().map(|parcel| parcel.id.0).collect();
    for (id, entity) in scene.graph_parcels.clone() {
        if !incoming.contains(&id) {
            commands.entity(entity).despawn();
            scene.graph_parcels.remove(&id);
            scene.graph_parcel_materials.remove(&id);
        }
    }

    let scale = civ_voxel::FIXED_SCALE as f32;
    for parcel in graph.parcels {
        let id = parcel.id.0;
        let provenance = graph
            .provenance
            .get(&parcel.id)
            .copied()
            .map(map_build_provenance)
            .unwrap_or(scene.building_provenance);
        let kind = parcel_kind_to_building_kind(parcel.kind);
        let mut base_color = parcel_kind_color(parcel.kind);
        if let Some(facade) = graph.facades.get(&parcel.id) {
            base_color = facade_material_color(facade);
        }
        let (styled_base, emissive, roughness) = building_material_style(kind, provenance);
        let base_color = blend_facade_base(base_color, styled_base);

        let material_handle = scene
            .graph_parcel_materials
            .entry(id)
            .or_insert_with(|| {
                materials.add(StandardMaterial {
                    base_color,
                    emissive: emissive.into(),
                    perceptual_roughness: roughness,
                    ..default()
                })
            })
            .clone();
        if let Some(material) = materials.get_mut(&material_handle) {
            material.base_color = base_color;
            material.emissive = emissive.into();
            material.perceptual_roughness = roughness;
        }

        let anchor = WorldXZ::from_fixed_coord(&parcel.origin);
        let ground = live_ground_y(&scene.chunk_voxels, anchor.x, anchor.z, 0.0);
        let height = parcel.size[1] as f32 / scale;
        let width = parcel.size[0] as f32 / scale;
        let depth = parcel.size[2] as f32 / scale;
        let transform = Transform::from_xyz(anchor.x, ground + height * 0.5, anchor.z)
            .with_scale(Vec3::new(width.max(0.5), height.max(0.5), depth.max(0.5)));

        let entity = *scene
            .graph_parcels
            .entry(id)
            .or_insert_with(|| commands.spawn(LiveGraphParcelTag { id }).id());
        commands.entity(entity).insert((
            Mesh3d(meshes.building_mesh.clone()),
            MeshMaterial3d(material_handle),
            transform,
        ));
    }
}

/// Minimap dot tint for building provenance.
#[must_use]
pub fn building_minimap_dot_color(provenance: BuildingProvenance) -> Color {
    match provenance {
        BuildingProvenance::Procedural => Color::srgba(0.92, 0.90, 0.86, 1.0),
        BuildingProvenance::Freehand => Color::srgba(0.98, 0.62, 0.28, 1.0),
    }
}

fn parcel_kind_to_building_kind(kind: ParcelKind) -> BuildingKind3d {
    match kind {
        ParcelKind::Residential => BuildingKind3d::House,
        ParcelKind::Commercial => BuildingKind3d::Market,
        ParcelKind::Industrial => BuildingKind3d::Mine,
        ParcelKind::Civic => BuildingKind3d::CityCenter,
    }
}

fn parcel_kind_color(kind: ParcelKind) -> Color {
    building_kind_color(parcel_kind_to_building_kind(kind))
}

fn facade_material_color(facade: &FacadeStyle) -> Color {
    let material = facade.materials.first().copied().unwrap_or(MaterialId(0));
    material_id_color(material)
}

fn material_id_color(material: MaterialId) -> Color {
    let palette = [
        (0.62, 0.52, 0.44),
        (0.55, 0.42, 0.32),
        (0.58, 0.58, 0.60),
        (0.72, 0.48, 0.36),
        (0.68, 0.70, 0.74),
        (0.42, 0.58, 0.72),
        (0.78, 0.80, 0.84),
    ];
    let idx = material.0 as usize % palette.len();
    let (r, g, b) = palette[idx];
    Color::srgb(r, g, b)
}

fn blend_facade_base(facade: Color, styled: Color) -> Color {
    let f = facade.to_srgba();
    let s = styled.to_srgba();
    Color::srgb(
        f.red * 0.65 + s.red * 0.35,
        f.green * 0.65 + s.green * 0.35,
        f.blue * 0.65 + s.blue * 0.35,
    )
}

fn building_kind_color(kind: BuildingKind3d) -> Color {
    match kind {
        BuildingKind3d::Farm => Color::srgb(0.55, 0.75, 0.35),
        BuildingKind3d::Mine => Color::srgb(0.52, 0.48, 0.42),
        BuildingKind3d::Barracks => Color::srgb(0.72, 0.34, 0.34),
        BuildingKind3d::Temple => Color::srgb(0.72, 0.62, 0.88),
        BuildingKind3d::Market => Color::srgb(0.88, 0.67, 0.25),
        BuildingKind3d::House => Color::srgb(0.79, 0.59, 0.40),
        BuildingKind3d::CityCenter => Color::srgb(0.38, 0.58, 0.86),
    }
}

fn building_material_style(
    kind: BuildingKind3d,
    provenance: BuildingProvenance,
) -> (Color, Color, f32) {
    let base = building_kind_color(kind);
    match provenance {
        BuildingProvenance::Procedural => (base, Color::BLACK, 0.92),
        BuildingProvenance::Freehand => {
            let emissive = Color::srgb(
                base.to_srgba().red * 0.55,
                base.to_srgba().green * 0.55,
                base.to_srgba().blue * 0.55,
            );
            (base, emissive, 0.55)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encode_chunk_id;
    use crate::live_ground::{live_ground_y, live_voxel_surface_y, ChunkVoxelCache};
    use civ_protocol_3d::{DirtyChunkEvent, VoxelChunkDelta, VoxelDeltaFrame, WriteSeq};
    use civ_voxel::material::WATER;
    use civ_voxel::MaterialId;

    const CHUNK_VOXELS: usize = LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE;

    fn voxel_index(ix: usize, iy: usize, iz: usize) -> usize {
        ix + iy * LIVE_CHUNK_EDGE + iz * LIVE_CHUNK_EDGE * LIVE_CHUNK_EDGE
    }

    fn solid_chunk_voxels() -> Vec<MaterialId> {
        let mut voxels = vec![MaterialId(0); CHUNK_VOXELS];
        voxels[voxel_index(4, 3, 5)] = WATER;
        voxels
    }

    fn color_rgb(c: Color) -> [f32; 3] {
        let s = c.to_srgba();
        [s.red, s.green, s.blue]
    }

    #[test]
    fn building_minimap_dot_color_matches_provenance() {
        let procedural = building_minimap_dot_color(BuildingProvenance::Procedural);
        let freehand = building_minimap_dot_color(BuildingProvenance::Freehand);
        assert!(procedural.alpha() > 0.99);
        assert!(freehand.alpha() > 0.99);
        assert_ne!(color_rgb(procedural), color_rgb(freehand));
    }

    #[test]
    fn parcel_kind_to_building_kind_maps_zoning() {
        assert_eq!(
            parcel_kind_to_building_kind(ParcelKind::Residential),
            BuildingKind3d::House
        );
        assert_eq!(
            parcel_kind_to_building_kind(ParcelKind::Commercial),
            BuildingKind3d::Market
        );
        assert_eq!(
            parcel_kind_to_building_kind(ParcelKind::Industrial),
            BuildingKind3d::Mine
        );
        assert_eq!(
            parcel_kind_to_building_kind(ParcelKind::Civic),
            BuildingKind3d::CityCenter
        );
    }

    #[test]
    fn building_kind_color_is_distinct_per_kind() {
        let kinds = [
            BuildingKind3d::Farm,
            BuildingKind3d::Mine,
            BuildingKind3d::Barracks,
            BuildingKind3d::Temple,
            BuildingKind3d::Market,
            BuildingKind3d::House,
            BuildingKind3d::CityCenter,
        ];
        let colors: Vec<_> = kinds
            .iter()
            .map(|k| color_rgb(building_kind_color(*k)))
            .collect();
        for (i, left) in colors.iter().enumerate() {
            for (j, right) in colors.iter().enumerate() {
                if i != j {
                    assert_ne!(left, right, "kinds {i} and {j} share a color");
                }
            }
        }
    }

    #[test]
    fn parcel_kind_color_uses_building_kind_palette() {
        let parcel = color_rgb(parcel_kind_color(ParcelKind::Residential));
        let house = color_rgb(building_kind_color(BuildingKind3d::House));
        assert_eq!(parcel, house);
    }

    #[test]
    fn building_material_style_freehand_adds_emissive() {
        let (base, emissive, roughness) =
            building_material_style(BuildingKind3d::Market, BuildingProvenance::Freehand);
        let (proc_base, proc_emissive, proc_roughness) =
            building_material_style(BuildingKind3d::Market, BuildingProvenance::Procedural);
        assert_eq!(color_rgb(base), color_rgb(proc_base));
        assert_ne!(color_rgb(emissive), color_rgb(proc_emissive));
        assert!(roughness < proc_roughness);
    }

    #[test]
    fn blend_facade_base_mixes_toward_styled_base() {
        let facade = Color::srgb(1.0, 0.0, 0.0);
        let styled = Color::srgb(0.0, 0.0, 1.0);
        let blended = blend_facade_base(facade, styled);
        let b = color_rgb(blended);
        assert!(b[0] > 0.4 && b[0] < 0.8);
        assert!(b[2] > 0.1 && b[2] < 0.5);
    }

    #[test]
    fn material_id_color_is_stable_for_same_id() {
        let a = color_rgb(material_id_color(MaterialId(3)));
        let b = color_rgb(material_id_color(MaterialId(3)));
        assert_eq!(a, b);
    }

    #[test]
    fn stream_ground_y_uses_voxel_surface_plus_agent_offset() {
        let mut cache = ChunkVoxelCache::new();
        cache.insert(encode_chunk_id(0, 0, 0), solid_chunk_voxels());
        let surface = live_voxel_surface_y(&cache, 4.5, 5.5).expect("surface");
        let agent_y = live_ground_y(&cache, 4.5, 5.5, AGENT_GROUND_Y);
        let building_y = live_ground_y(&cache, 4.5, 5.5, BUILDING_GROUND_Y);
        assert!((agent_y - (surface + AGENT_GROUND_Y)).abs() < f32::EPSILON);
        assert!((building_y - (surface + BUILDING_GROUND_Y)).abs() < f32::EPSILON);
    }

    /// FR-CIV-BEVY-016 — live_stream applies voxel deltas and updates chunk scene state for attach smoke.
    /// FR-CIV-BEVY-025 — scene update test coverage is part of attach smoke expectations.
    #[test]
    fn apply_voxel_delta_frame_spawns_chunk_in_scene() {
        use bevy::prelude::*;

        let mut world = World::new();
        let mut mesh_assets = Assets::<Mesh>::default();
        let mut material_assets = Assets::<StandardMaterial>::default();

        let chunk_id = encode_chunk_id(0, 0, 0);
        let delta = VoxelDeltaFrame {
            tick: 1,
            deltas: vec![VoxelChunkDelta {
                event: DirtyChunkEvent {
                    chunk_id,
                    write_seq: WriteSeq(1),
                },
                voxels: solid_chunk_voxels(),
            }],
        };
        let culling = StreamCulling {
            eye: [8.0, 8.0, 8.0],
            max_distance: 512.0,
        };

        let mut scene = LiveStreamScene::default();
        let mut commands = world.commands();
        apply_voxel_delta_frame(
            &mut commands,
            &mut scene,
            &mut mesh_assets,
            &mut material_assets,
            culling,
            &DebugRender::default(),
            delta,
            None,
        );

        assert_eq!(scene.chunks.len(), 1);
        assert!(scene.chunk_voxels.chunks().contains_key(&chunk_id.0));
    }

    #[test]
    fn format_event_feed_message_birth_with_species() {
        use civ_protocol_3d::{BirthEvent3d, EventFeedMessage3d, WorldXZ};

        let text = format_event_feed_message(&EventFeedMessage3d::Birth(BirthEvent3d {
            entity_id: 9,
            faction_id: 2,
            species: "human".to_string(),
            position: Some(WorldXZ { x: 10.0, z: 12.0 }),
        }));
        assert!(text.contains("human"));
        assert!(text.contains("faction 2"));
        assert!(text.contains("(10"));
    }

    #[test]
    fn format_event_feed_message_death_uses_cause() {
        use civ_protocol_3d::{DeathEvent3d, EventFeedMessage3d};

        let text = format_event_feed_message(&EventFeedMessage3d::Death(DeathEvent3d {
            entity_id: 3,
            cause: "battle".to_string(),
            ..Default::default()
        }));
        assert_eq!(text, "battle (entity #3)");
    }

    #[test]
    fn format_event_feed_message_tech_unlock() {
        use civ_protocol_3d::{EventFeedMessage3d, TechEvent3d};

        let text = format_event_feed_message(&EventFeedMessage3d::Tech(TechEvent3d {
            faction_id: 5,
            era: 2,
            tech: "agriculture".to_string(),
        }));
        assert!(text.contains("Faction 5"));
        assert!(text.contains("agriculture"));
    }

    #[test]
    fn format_event_feed_message_battle_outcome() {
        use civ_protocol_3d::{BattleEvent3d, EventFeedMessage3d};

        let text = format_event_feed_message(&EventFeedMessage3d::Battle(BattleEvent3d {
            attacker_faction: 1,
            defender_faction: 2,
            outcome: "attacker_won".to_string(),
            ..Default::default()
        }));
        assert!(text.contains("faction 1 vs faction 2"));
        assert!(text.contains("attacker won"));
    }

    #[test]
    fn format_event_feed_message_disaster_kind() {
        use civ_protocol_3d::{DisasterEvent3d, EventFeedMessage3d};

        let text = format_event_feed_message(&EventFeedMessage3d::Disaster(DisasterEvent3d {
            disaster_kind: "flood".to_string(),
            severity: 0.5,
            ..Default::default()
        }));
        assert!(text.contains("flood"));
        assert!(text.contains("50%"));
    }

    #[cfg(feature = "egui")]
    #[test]
    fn map_event_feed_message_assigns_kinds() {
        use crate::event_feed::EventKind;
        use civ_protocol_3d::{BirthEvent3d, EventFeedMessage3d};

        let (kind, _) = map_event_feed_message(&EventFeedMessage3d::Birth(BirthEvent3d::default()));
        assert_eq!(kind, EventKind::Birth);
    }

    #[cfg(feature = "egui")]
    #[test]
    fn apply_event_feed_frame_pushes_all_events() {
        use crate::event_feed::EventFeed;
        use civ_protocol_3d::{DeathEvent3d, EventFeedFrame, EventFeedMessage3d, TechEvent3d};

        let mut feed = EventFeed::default();
        apply_event_feed_frame(
            &mut feed,
            EventFeedFrame {
                tick: 1,
                events: vec![
                    EventFeedMessage3d::Death(DeathEvent3d {
                        entity_id: 1,
                        ..Default::default()
                    }),
                    EventFeedMessage3d::Tech(TechEvent3d {
                        tech: "iron".to_string(),
                        ..Default::default()
                    }),
                ],
            },
        );
        assert_eq!(feed.events.len(), 2);
    }

    #[test]
    fn push_event_feed_to_hud_summary_stores_last_messages() {
        use civ_protocol_3d::{DeathEvent3d, EventFeedFrame, EventFeedMessage3d, TechEvent3d};

        let mut hud = crate::LiveHudSnapshot::default();
        push_event_feed_to_hud_summary(
            &mut hud,
            &EventFeedFrame {
                tick: 4,
                events: vec![
                    EventFeedMessage3d::Death(DeathEvent3d {
                        entity_id: 1,
                        ..Default::default()
                    }),
                    EventFeedMessage3d::Tech(TechEvent3d {
                        tech: "agriculture".into(),
                        faction_id: 2,
                        ..Default::default()
                    }),
                ],
            },
        );
        let line = hud.last_event.expect("last_event");
        assert!(line.contains("Entity #1 died"));
        assert!(line.contains("agriculture"));
        assert!(line.contains('·'));
    }

    #[test]
    fn truncate_event_feed_hud_line_caps_length() {
        let long = "x".repeat(LIVE_HUD_EVENT_MAX_CHARS + 20);
        let truncated = truncate_event_feed_hud_line(&long);
        assert!(truncated.chars().count() <= LIVE_HUD_EVENT_MAX_CHARS);
        assert!(truncated.ends_with('…'));
    }

    #[test]
    fn resolve_civilian_for_live_pick_by_stable_id() {
        use civ_protocol_3d::{CivilianNeeds3d, GenomeSummary3d};

        let mut scene = LiveStreamScene::default();
        scene.civilian_entries.insert(
            100,
            CivilianStateEntry {
                id: 100,
                needs: CivilianNeeds3d::default(),
                profession: "smith".to_string(),
                genome_summary: GenomeSummary3d::default(),
                species: String::new(),
                health: 1.0,
            },
        );

        let entry = resolve_civilian_for_live_pick(100, &scene).expect("stable id");
        assert_eq!(entry.id, 100);
        assert_eq!(entry.profession, "smith");
    }

    #[test]
    fn resolve_civilian_for_live_pick_fallback_when_agent_key_differs() {
        use civ_protocol_3d::{CivilianNeeds3d, GenomeSummary3d};

        let mut scene = LiveStreamScene::default();
        scene.agents.insert(999, Entity::PLACEHOLDER);
        scene.civilian_entries.insert(
            42,
            CivilianStateEntry {
                id: 42,
                needs: CivilianNeeds3d::default(),
                profession: "farmer".to_string(),
                genome_summary: GenomeSummary3d {
                    summary: "Ada".to_string(),
                    ..Default::default()
                },
                species: String::new(),
                health: 0.5,
            },
        );

        let entry = resolve_civilian_for_live_pick(999, &scene).expect("1:1 fallback");
        assert_eq!(entry.id, 42);
        assert_eq!(entry.profession, "farmer");
    }

    #[test]
    fn resolve_civilian_for_live_pick_none_when_multiple_agents() {
        use civ_protocol_3d::{CivilianNeeds3d, GenomeSummary3d};

        let mut scene = LiveStreamScene::default();
        scene.agents.insert(1, Entity::PLACEHOLDER);
        scene.agents.insert(2, Entity::PLACEHOLDER);
        scene.civilian_entries.insert(
            10,
            CivilianStateEntry {
                id: 10,
                needs: CivilianNeeds3d::default(),
                profession: String::new(),
                genome_summary: GenomeSummary3d::default(),
                species: String::new(),
                health: 1.0,
            },
        );
        scene.civilian_entries.insert(
            20,
            CivilianStateEntry {
                id: 20,
                needs: CivilianNeeds3d::default(),
                profession: String::new(),
                genome_summary: GenomeSummary3d::default(),
                species: String::new(),
                health: 1.0,
            },
        );

        assert!(resolve_civilian_for_live_pick(1, &scene).is_none());
    }

    #[test]
    fn format_live_pick_hud_line_civilian_entry_shows_name_profession_health() {
        use civ_protocol_3d::{CivilianNeeds3d, GenomeSummary3d};

        let mut scene = LiveStreamScene::default();
        scene.civilian_entries.insert(
            7,
            CivilianStateEntry {
                id: 7,
                needs: CivilianNeeds3d::default(),
                profession: "Farmer".to_string(),
                genome_summary: GenomeSummary3d {
                    summary: "Ada".to_string(),
                    ..Default::default()
                },
                species: String::new(),
                health: 0.87,
            },
        );

        let line = format_live_pick_hud_line(
            Some(SelectedLiveEntity {
                kind: LiveEntityKind::Agent,
                id: 7,
            }),
            &scene,
        )
        .expect("pick line");
        assert_eq!(line, "Ada | Farmer | 87%");
    }

    #[test]
    fn format_live_pick_hud_line_agent_without_civilian_entry_uses_id_stub() {
        let scene = LiveStreamScene::default();
        let line = format_live_pick_hud_line(
            Some(SelectedLiveEntity {
                kind: LiveEntityKind::Agent,
                id: 42,
            }),
            &scene,
        )
        .expect("pick line");
        assert_eq!(line, "Agent #42");
    }

    #[test]
    fn format_live_pick_hud_line_uses_resolver_fallback_for_legacy_agent_key() {
        use civ_protocol_3d::{CivilianNeeds3d, GenomeSummary3d};

        let mut scene = LiveStreamScene::default();
        scene.agents.insert(999, Entity::PLACEHOLDER);
        scene.civilian_entries.insert(
            42,
            CivilianStateEntry {
                id: 42,
                needs: CivilianNeeds3d::default(),
                profession: "Farmer".to_string(),
                genome_summary: GenomeSummary3d {
                    summary: "Ada".to_string(),
                    ..Default::default()
                },
                species: String::new(),
                health: 0.87,
            },
        );

        let line = format_live_pick_hud_line(
            Some(SelectedLiveEntity {
                kind: LiveEntityKind::Agent,
                id: 999,
            }),
            &scene,
        )
        .expect("pick line");
        assert_eq!(line, "Ada | Farmer | 87%");
    }

    #[test]
    fn format_live_pick_hud_line_none_when_unselected() {
        let scene = LiveStreamScene::default();
        assert!(format_live_pick_hud_line(None, &scene).is_none());
    }

    #[test]
    fn format_live_pick_hud_line_building_and_parcel_stubs() {
        let scene = LiveStreamScene::default();
        assert_eq!(
            format_live_pick_hud_line(
                Some(SelectedLiveEntity {
                    kind: LiveEntityKind::Building,
                    id: 3,
                }),
                &scene,
            )
            .as_deref(),
            Some("Building #3")
        );
        assert_eq!(
            format_live_pick_hud_line(
                Some(SelectedLiveEntity {
                    kind: LiveEntityKind::GraphParcel,
                    id: 11,
                }),
                &scene,
            )
            .as_deref(),
            Some("Parcel #11")
        );
    }

    #[test]
    fn sync_live_hud_civilian_faction_counts_reads_scene_state() {
        let mut scene = LiveStreamScene::default();
        scene.civilian_ids.insert(1);
        scene.civilian_ids.insert(2);
        scene.factions.insert(7);

        let mut hud = crate::LiveHudSnapshot::default();
        sync_live_hud_civilian_faction_counts(&mut hud, &scene);

        assert_eq!(hud.civilian_count, 2);
        assert_eq!(hud.faction_count, 1);
    }

    #[test]
    fn apply_civilian_state_frame_replaces_tracked_ids() {
        use civ_protocol_3d::{
            CivilianNeeds3d, CivilianStateEntry, CivilianStateFrame, GenomeSummary3d,
        };

        let mut scene = LiveStreamScene::default();
        scene.civilian_ids.insert(99);

        let entry = |id: u64| CivilianStateEntry {
            id,
            needs: CivilianNeeds3d::default(),
            profession: String::new(),
            genome_summary: GenomeSummary3d::default(),
            species: String::new(),
            health: 1.0,
        };

        apply_civilian_state_frame(
            &mut scene,
            CivilianStateFrame {
                tick: 7,
                civilians: vec![entry(1), entry(2)],
            },
        );

        assert_eq!(civilian_hud_count(&scene), 2);
        assert!(scene.civilian_ids.contains(&1));
        assert!(scene.civilian_ids.contains(&2));
        assert!(!scene.civilian_ids.contains(&99));
        let entry = scene.civilian_entries.get(&1).expect("civilian 1");
        assert_eq!(entry.id, 1);
        assert_eq!(entry.profession, String::new());
    }

    #[test]
    fn apply_faction_state_frame_replaces_tracked_ids_and_era() {
        use civ_protocol_3d::{
            FactionStateEntry, FactionStateFrame, FactionTreasury3d, Government3d,
        };

        let mut scene = LiveStreamScene::default();
        scene.factions.insert(9);

        let entry = |id: u32, era: u16| FactionStateEntry {
            id,
            era,
            government: Government3d::Republic,
            treasury: FactionTreasury3d::default(),
        };

        apply_faction_state_frame(
            &mut scene,
            FactionStateFrame {
                tick: 3,
                factions: vec![entry(0, 2), entry(4, 5)],
            },
        );

        assert_eq!(faction_hud_count(&scene), 2);
        assert_eq!(scene.faction_era, 5);
        assert_eq!(scene.faction_entries.len(), 2);
        assert_eq!(scene.faction_entries[0].id, 0);
        assert_eq!(scene.faction_entries[1].id, 4);
        assert!(scene.factions.contains(&0));
        assert!(scene.factions.contains(&4));
        assert!(!scene.factions.contains(&9));
    }

    #[cfg(all(feature = "bevy", feature = "egui"))]
    #[test]
    fn sync_diplomacy_from_faction_frame_preserves_open_flag() {
        use civ_protocol_3d::{
            FactionStateEntry, FactionStateFrame, FactionTreasury3d, Government3d,
        };

        let frame = FactionStateFrame {
            tick: 1,
            factions: vec![FactionStateEntry {
                id: 1,
                era: 0,
                government: Government3d::Junta,
                treasury: FactionTreasury3d::default(),
            }],
        };
        let mut diplomacy = crate::diplomacy_ui::DiplomacyState {
            open: true,
            ..Default::default()
        };
        sync_diplomacy_from_faction_frame(&mut diplomacy, &frame, &HashMap::new());
        assert!(diplomacy.open);
        assert_eq!(diplomacy.factions.len(), 1);
        assert_eq!(diplomacy.factions[0].name, "Junta #1");
    }

    // ---- FR-CLIENT-render ----------------------------------------------------

    #[test]
    fn apply_climate_frame_records_snapshot_and_drops_stale() {
        use civ_planet::{Climate, SeasonKind, WeatherCell, WeatherKind};
        let mut scene = LiveStreamScene::default();
        let frame1 = ClimateFrame {
            tick: 10,
            climate: Climate {
                tick: 10,
                day_phase: 0.25,
                year_phase: 0.5,
                moon_phase: 0.1,
                tide_offset: 0.0,
            },
            weather: vec![WeatherCell {
                region_id: 0,
                latitude_fp: 0,
                season: SeasonKind::Spring,
                kind: WeatherKind::Clear,
                temp_c_fp: 20,
                precip_mm_fp: 0,
                storm_intensity_fp: 0,
            }],
        };
        apply_climate_frame(&mut scene, frame1);
        let (snap, weather) = latest_climate(&scene).expect("climate recorded");
        assert_eq!(snap.tick, 10);
        assert_eq!(snap.day_phase, 0.25);
        assert_eq!(weather.len(), 1);

        // Stale frame (older tick) must be a no-op.
        let stale = ClimateFrame {
            tick: 5,
            climate: Climate {
                tick: 5,
                day_phase: 0.99,
                year_phase: 0.99,
                moon_phase: 0.99,
                tide_offset: 0.0,
            },
            weather: vec![],
        };
        apply_climate_frame(&mut scene, stale);
        let (snap, _) = latest_climate(&scene).expect("still recorded");
        assert_eq!(snap.tick, 10, "stale frame must not overwrite");
        assert_eq!(snap.day_phase, 0.25);
    }

    #[test]
    fn chunk_biome_tint_picks_dominant_solid_material() {
        use civ_voxel::{GRANITE, SAND, SNOW};
        let mut voxels = vec![MaterialId(0); CHUNK_VOXELS];
        // Sand fills ~80% of the chunk.
        for v in voxels.iter_mut().take(CHUNK_VOXELS * 4 / 5) {
            *v = SAND;
        }
        // Granite flecks.
        voxels[10] = GRANITE;
        voxels[20] = GRANITE;
        // Snow top crust.
        for x in 0..LIVE_CHUNK_EDGE {
            for z in 0..LIVE_CHUNK_EDGE {
                voxels[voxel_index(x, LIVE_CHUNK_EDGE - 1, z)] = SNOW;
            }
        }
        let tint = chunk_biome_tint(&voxels).expect("has renderable material");
        // Sand tint is the dominant material by count.
        let sand = material_tint(SAND);
        assert_eq!(tint, sand);

        // All-air chunk returns None so callers fall back to base color.
        let air = vec![MaterialId(0); CHUNK_VOXELS];
        assert!(chunk_biome_tint(&air).is_none());
    }

    #[test]
    fn agent_distance_lod_falls_off_gently_and_clamps() {
        // At distance 0 the wire scale passes through (modulo the gentle
        // 1/(1 + d*0.05) envelope, ~1.0).
        let near = agent_distance_lod(1.0, 0.0, 0.05, f32::MAX);
        assert!((near - 1.0).abs() < 1e-3, "near scale ~1.0, got {near}");

        // At distance 20 the falloff is ~0.5.
        let mid = agent_distance_lod(1.0, 20.0, 0.05, f32::MAX);
        assert!(mid > 0.4 && mid < 0.6, "mid scale ~0.5, got {mid}");

        // Lower bound clamps very-far agents to min_scale.
        let far = agent_distance_lod(1.0, 1_000.0, 0.05, f32::MAX);
        assert!((far - 0.05).abs() < 1e-4, "far clamps to min, got {far}");

        // Non-finite inputs degrade to min_scale.
        assert_eq!(agent_distance_lod(f32::NAN, 1.0, 0.05, f32::MAX), 0.05);
        assert_eq!(agent_distance_lod(1.0, f32::NAN, 0.05, f32::MAX), 0.05);
    }
}
