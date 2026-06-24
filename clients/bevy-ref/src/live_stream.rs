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
    CivilianStateEntry, CivilianStateFrame, DeathEvent3d, DisasterEvent3d, EventFeedMessage3d,
    FacadeStyle, FactionStateFrame, ParcelKind, TechEvent3d, VoxelDeltaFrame, WorldXZ,
};
use civ_voxel::{ChunkId, ChunkView, CubicMesher, LodLevel, MaterialId};

use crate::bevy_render::{apply_chunk_material, mesh_buffer_to_bevy};
use crate::game_ui::civilian_display_name;
use crate::live_ground::{live_ground_y, ChunkVoxelCache};
use crate::{
    agent_color_from_id, agent_scale_multiplier, chunk_distance_from_camera, decode_chunk_id,
    mesh_lod_level, should_render_chunk, DebugRender, LiveEntityKind, SelectedLiveEntity,
    AGENT_MARKER_DEPTH, AGENT_MARKER_HEIGHT, AGENT_MARKER_WIDTH,
};

/// Chunk edge length in voxels (matches kernel).
pub const LIVE_CHUNK_EDGE: usize = 16;
/// Default chunk albedo for streamed meshes.
pub const LIVE_CHUNK_BASE_COLOR: [f32; 3] = [0.72, 0.69, 0.62];

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
        apply_chunk_material(
            &mut material,
            LIVE_CHUNK_BASE_COLOR,
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
    for update in agents.updates {
        let rgb = agent_color_from_id(update.agent_id);
        let scale = agent_scale_multiplier(update.scale);
        let (x, _, z) = agent_world_translation(&update, 0.0);
        if !x.is_finite() || !z.is_finite() || !scale.is_finite() {
            continue;
        }
        let y = live_ground_y(&scene.chunk_voxels, x, z, AGENT_GROUND_Y);
        if !y.is_finite() {
            continue;
        }
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
}
