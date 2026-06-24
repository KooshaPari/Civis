//! civ-bevy-ref library surface.
//!
//! Splits cleanly into two parts:
//!
//! - **Always compiled** — pure converters and helpers that turn the
//!   engine-neutral [`civ_voxel::MeshBuffer`] into engine-native vertex
//!   arrays. Currently this just re-exposes the kernel `MeshBuffer` and adds
//!   small utility shapes.
//! - **`bevy` feature** — the Bevy renderer (`pub mod bevy_render`). Pulls
//!   Bevy 0.18 behind an optional feature set. Off by default so the workspace
//!   build stays fast for CI / agent-driven smoke runs.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "bevy")]
pub mod animation;
#[cfg(feature = "bevy")]
pub mod atmosphere;
#[cfg(feature = "bevy")]
pub mod camera;
#[cfg(feature = "bevy")]
pub mod decorations;
#[cfg(all(feature = "bevy", feature = "models"))]
pub mod animation;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod diplomacy_ui;
pub mod outcome_overlay;
pub mod faction_hud;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod save_load_ui;
#[cfg(all(feature = "bevy", feature = "models"))]
pub mod gltf_models;
pub mod emergence_dashboard;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod event_feed;
#[cfg(all(feature = "bevy", feature = "gi"))]
pub mod lighting_gi;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod game_ui;
#[cfg(all(feature = "bevy", feature = "models"))]
pub mod gltf_models;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod graphics_settings;
pub mod game_laws;
pub mod map2d;
#[cfg(feature = "bevy")]
pub mod gpu_features;
#[cfg(feature = "bevy")]
pub mod info_views;
#[cfg(feature = "bevy")]
pub mod live_attach;
#[cfg(feature = "bevy")]
pub mod live_focus;
#[cfg(feature = "bevy")]
pub mod live_ground;
#[cfg(feature = "bevy")]
pub mod live_minimap;
#[cfg(feature = "bevy")]
pub mod live_pick;
#[cfg(feature = "bevy")]
pub mod live_scene;
#[cfg(feature = "bevy")]
pub mod live_stream;
#[cfg(feature = "pbr-textures")]
pub mod materials;
#[cfg(feature = "bevy")]
pub mod post_fx;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod settings_ui;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod ui_theme;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod menus;
#[cfg(feature = "bevy")]
pub mod minimap;
#[cfg(feature = "bevy")]
pub mod native_backend;
#[cfg(feature = "bevy")]
pub mod native_renderer;
#[cfg(feature = "bevy")]
pub mod sim_bridge;
#[cfg(feature = "bevy")]
pub mod spawn_tools;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod tech_tree_ui;
<<<<<<< HEAD
=======
pub mod civ_history;
>>>>>>> 34495eed48a7965a10f0cb2f2db986adfb380b94
pub mod god_panel;
pub mod tutorial;
pub mod perf_hud;
#[cfg(feature = "bevy")]
pub mod terrain;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod tool_categories;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod ui_cluster;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod ui_holo;
#[cfg(all(feature = "bevy", feature = "egui"))]
pub mod ui_theme;
#[cfg(all(feature = "bevy", feature = "vfx"))]
pub mod vfx;
#[cfg(all(feature = "bevy", feature = "voxel"))]
pub mod ocean;
#[cfg(feature = "voxel")]
pub mod voxel_sim;
#[cfg(feature = "voxel")]
pub mod voxel_smooth_mesher;
/// Camera-driven chunk-streaming sandbox (`StreamingWorld` + `HeightFieldGen`).
/// Gated behind `voxel_stream` so it coexists with the dense `voxel_sim` path.
#[cfg(feature = "voxel_stream")]
pub mod voxel_stream;
#[cfg(feature = "voxel")]
pub mod voxel_triplanar;
#[cfg(feature = "bevy")]
pub mod window_icon;

pub use civ_voxel::{
    ChunkId, CubicMesher, MaterialId, MeshBuffer, MeshVertex, VoxelWorld, WorldCoord,
};
pub use civ_agents::NeedAction;

/// Default orbit azimuth in radians (45° — camera south-east of centre).
pub const DEFAULT_CAMERA_AZIMUTH_RAD: f32 = std::f32::consts::FRAC_PI_4;

/// Default orbit elevation in radians (~35° — slightly above the horizon).
pub const DEFAULT_CAMERA_ELEVATION_RAD: f32 = 0.615_479_7;

/// Default scene centre for a single 16³ chunk centred on the origin grid cell.
pub const DEFAULT_CAMERA_CENTRE: [f32; 3] = [8.0, 8.0, 8.0];

/// Default camera stand-off distance in world units (≈ three chunk edges).
pub const DEFAULT_CAMERA_DISTANCE: f32 = 48.0;

/// Engine-neutral camera placement helper. The actual renderer uses this to
/// position a chase / orbit camera around a voxel scene.
///
/// Orbit controls are not wired yet; drag-to-orbit will reuse these constants.
/// Until then, [`CameraTarget::orbit_position`] frames the first chunk at the origin.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraTarget {
    /// Centre of the scene in world units.
    pub centre: [f32; 3],
    /// Distance from the centre to place the camera.
    pub distance: f32,
    /// Horizontal orbit angle in radians (0 = +Z, increasing toward +X).
    pub azimuth_rad: f32,
    /// Vertical orbit angle in radians (0 = horizon, π/2 = straight down).
    pub elevation_rad: f32,
}

impl Default for CameraTarget {
    fn default() -> Self {
        Self {
            centre: DEFAULT_CAMERA_CENTRE,
            distance: DEFAULT_CAMERA_DISTANCE,
            azimuth_rad: DEFAULT_CAMERA_AZIMUTH_RAD,
            elevation_rad: DEFAULT_CAMERA_ELEVATION_RAD,
        }
    }
}

impl CameraTarget {
    /// Eye position for an orbit camera looking at [`Self::centre`].
    #[must_use]
    pub fn orbit_position(&self) -> [f32; 3] {
        let horiz = self.distance * self.elevation_rad.cos();
        [
            self.centre[0] + horiz * self.azimuth_rad.sin(),
            self.centre[1] + self.distance * self.elevation_rad.sin(),
            self.centre[2] + horiz * self.azimuth_rad.cos(),
        ]
    }
}

/// Debug rendering toggles for the Bevy reference window (`civ-bevy-window`).
#[derive(Debug, Clone, Copy, PartialEq, Default)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct DebugRender {
    /// When true, chunk meshes render with wireframe overlay (Bevy `Wireframe` on native).
    pub wireframe: bool,
}

impl DebugRender {
    /// Flip [`Self::wireframe`] and return the new value.
    pub fn toggle_wireframe(&mut self) -> bool {
        self.wireframe = !self.wireframe;
        self.wireframe
    }
}

/// Alpha for chunk solid fill when wireframe debug mode is active (ghost overlay).
pub const DEBUG_WIREFRAME_OVERLAY_ALPHA: f32 = 0.22;

/// Climate presentation fields from a `sim.snapshot` JSON-RPC response.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct WsSpectatorMeta {
    /// Day/night flag from the simulation climate phase.
    pub is_day: bool,
    /// Latest tick when present on the snapshot payload.
    pub tick: Option<u64>,
}

/// WebSocket session state exposed to live attach HUD and event feed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WsConnectionState {
    /// Active stream to `civ-server`.
    Connected,
    /// Backing off after a disconnect; will retry.
    Reconnecting,
    /// No successful connection yet (initial boot).
    #[default]
    Disconnected,
}

/// Streamed entity kind for viewport pick and HUD labels.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LiveEntityKind {
    /// Streamed agent marker.
    Agent,
    /// Streamed building marker.
    Building,
    /// Streamed building-graph parcel marker.
    GraphParcel,
}

/// A single streamed entity selected in the live viewport.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelectedLiveEntity {
    /// Entity category.
    pub kind: LiveEntityKind,
    /// Server-assigned entity id.
    pub id: u64,
}

/// Format selection for HUD overlay (`sel: agent #N`).
#[must_use]
pub fn format_live_selection(entity: SelectedLiveEntity) -> String {
    let label = match entity.kind {
        LiveEntityKind::Agent => "agent",
        LiveEntityKind::Building => "building",
        LiveEntityKind::GraphParcel => "graph",
    };
    format!("sel: {label} #{}", entity.id)
}

/// Parse `sim.snapshot` JSON-RPC text (not F3D0 tick frames).
#[cfg(any(test, feature = "bevy"))]
#[must_use]
pub fn parse_jsonrpc_snapshot_meta(text: &str) -> Option<WsSpectatorMeta> {
    let value: serde_json::Value = serde_json::from_str(text).ok()?;
    let result = value.get("result")?;
    let is_day = result.get("is_day")?.as_bool()?;
    let tick = result.get("tick").and_then(|v| v.as_u64());
    Some(WsSpectatorMeta { is_day, tick })
}

/// Subset of sim.emergence fields shown in the HUD.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct EmergenceHudData {
    /// Normalised Shannon entropy (`0..=1`).
    pub entropy_norm: f32,
    /// Power-law exponent alpha for cluster-size distribution.
    pub power_law_alpha: f32,
    /// Novel config fingerprints per window per civilian.
    pub novelty_rate: f32,
    /// Normalised mutual information between material and faction distributions.
    pub mi_material_faction_norm: Option<f32>,
    /// 6-connectivity component count from the sampled chunk (None = not yet sampled).
    pub structure_count: Option<u32>,
    /// Charter criticality regime label: "SUBCRITICAL" | "CRITICAL" | "SUPERCRITICAL".
    pub branching_regime: String,
}


/// Outcome data from `sim.outcome` polling (FR-CIV-GAME-001).
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub struct OutcomeHudData {
    pub tag: String,
    pub reason: String,
    pub tick: u64,
}
/// Headless-friendly snapshot for the live attach HUD (FPS / tick / socket / scene stats).
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
#[derive(Debug, Clone, PartialEq, Default)]
pub struct LiveHudSnapshot {
    /// WebSocket session state from the reconnecting client.
    pub connection: WsConnectionState,
    /// Whether live frames or snapshot metadata have been received (stream activity).
    pub connected: bool,
    /// Latest simulation tick from the server, if any frame has arrived yet.
    pub tick: Option<u64>,
    /// Smoothed frames-per-second estimate from the renderer loop.
    pub fps: f32,
    /// Streamed voxel chunks currently in the live scene.
    pub chunk_count: usize,
    /// Streamed agent entities in the live scene.
    pub agent_count: usize,
    /// Streamed building entities in the live scene.
    pub building_count: usize,
    /// Streamed building-graph parcel entities in the live scene.
    pub graph_parcel_count: usize,
    /// Civilians tracked from `Frame3d::CivilianState` wire frames.
    pub civilian_count: usize,
    /// Factions tracked from `Frame3d::FactionState` wire frames.
    pub faction_count: usize,
    /// Max treasury balance across tracked factions (proxy for player wealth, from FactionStateEntry).
    #[serde(default)]
    pub treasury: f32,
    /// Latest `sim.snapshot` round-trip time in milliseconds, when measured.
    pub ws_rtt_ms: Option<f32>,
    /// Chunk under the cursor from minimap click or viewport raycast stub, if any.
    pub focused_chunk: Option<ChunkId>,
    /// Streamed agent/building/graph selection from viewport pick, if any.
    pub selected_live: Option<SelectedLiveEntity>,
    /// Most recent event-feed line for HUD overlay (`civ-bevy-window`, no egui).
    pub last_event: Option<String>,
    /// One-line civilian detail for the current viewport pick (inspector-lite HUD).
    pub pick_detail: Option<String>,
    /// Current sim speed multiplier (0 = paused, 1/2/4/8 = normal/fast/faster/fastest).
    /// Cached emergence metrics from sim.emergence poll (entropy_norm, power_law_alpha, novelty_rate, mi).
    pub emergence: Option<EmergenceHudData>,
    /// Current simulation speed multiplier (0 = paused).
    pub speed_multiplier: u32,
    /// Server-reported last tick wall-clock duration from sim.perf (FR-CIV-PERF-001).
    pub tick_ms: f64,
}

impl LiveHudSnapshot {
    /// Copy streamed entity counts from a live attach scene map.
    pub fn sync_scene_counts(
        &mut self,
        chunks: usize,
        agents: usize,
        buildings: usize,
        graph_parcels: usize,
        civilians: usize,
        factions: usize,
    ) {
        self.chunk_count = chunks;
        self.agent_count = agents;
        self.building_count = buildings;
        self.graph_parcel_count = graph_parcels;
        self.civilian_count = civilians;
        self.faction_count = factions;
    }

    /// Format a single-line overlay string suitable for Bevy UI or CI log checks.
    #[must_use]
    pub fn format_overlay(&self) -> String {
        let status = match self.connection {
            WsConnectionState::Connected => "connected",
            WsConnectionState::Reconnecting => "reconnecting",
            WsConnectionState::Disconnected => "disconnected",
        };
        let tick = self
            .tick
            .map(|value| value.to_string())
            .unwrap_or_else(|| "—".to_string());
        let mut line = format!(
            "FPS: {:.0} | tick: {tick} | {status} | C:{} A:{} B:{} G:{} | P:{} F:{}",
            self.fps,
            self.chunk_count,
            self.agent_count,
            self.building_count,
            self.graph_parcel_count,
            self.civilian_count,
            self.faction_count,
        );
        if let Some(rtt) = self.ws_rtt_ms {
            line.push_str(&format!(" | RTT: {rtt:.0}ms"));
        }
        if let Some(chunk) = self.focused_chunk {
            line.push_str(&format!(" | chunk: {}", chunk.0));
        }
        if let Some(selection) = self.selected_live {
            line.push_str(&format!(" | {}", format_live_selection(selection)));
        }
        if let Some(event) = &self.last_event {
            line.push_str(&format!(" | evt: {event}"));
        }
        if let Some(detail) = &self.pick_detail {
            line.push_str(&format!(" | {detail}"));
        }
        {
            let spd = if self.speed_multiplier == 0 { "PAUSED".to_string() } else { format!("{}x", self.speed_multiplier) };
            line.push_str(&format!(" | spd:{spd}"));
        }
        if let Some(em) = &self.emergence {
            line.push_str(&format!(
                " | ent:{:.2} \u{03b1}:{:.2} nov:{:.3}",
                em.entropy_norm, em.power_law_alpha, em.novelty_rate
            ));
            if let Some(mi) = em.mi_material_faction_norm {
                line.push_str(&format!(" MI:{:.2}", mi));
            }
        }
        line
    }
}

/// Default Civis JSON-RPC WebSocket host.
pub const DEFAULT_WS_HOST: &str = "127.0.0.1";

/// Default Civis JSON-RPC WebSocket port.
pub const DEFAULT_WS_PORT: u16 = 3000;

/// Default Civis JSON-RPC WebSocket path.
pub const DEFAULT_WS_PATH: &str = "/ws";

/// When true, the live attach client skips redundant JSON text tick frames and
/// requests binary-only broadcast when the server honors `tick_format=binary`.
pub const DEFAULT_WS_PREFER_BINARY: bool = true;

/// Build a WebSocket URL for the Civis 3D live attach path (`ws://host:port/path`).
#[must_use]
pub fn live_ws_url(host: &str, port: u16, path: &str) -> String {
    let normalized_path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    format!("ws://{host}:{port}{normalized_path}")
}

/// Default local attach URL used by `civ-bevy-window`.
#[must_use]
pub fn default_live_ws_url() -> String {
    let host = std::env::var("CIV_WS_HOST").unwrap_or_else(|_| DEFAULT_WS_HOST.to_string());
    let port = std::env::var("CIV_SERVER_PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(DEFAULT_WS_PORT);
    let path = std::env::var("CIV_WS_PATH").unwrap_or_else(|_| DEFAULT_WS_PATH.to_string());
    live_ws_url(&host, port, &path)
}

/// Parse `CIVIS_WS_BINARY` (`1`, `true`, `yes`, case-insensitive) or fall back to
/// [`DEFAULT_WS_PREFER_BINARY`].
#[must_use]
pub fn ws_prefer_binary_from_env() -> bool {
    std::env::var("CIVIS_WS_BINARY")
        .map(|value| parse_ws_binary_env_flag(&value))
        .unwrap_or(DEFAULT_WS_PREFER_BINARY)
}

/// Returns true for common truthy env strings used by `CIVIS_WS_BINARY`.
#[must_use]
pub fn parse_ws_binary_env_flag(value: &str) -> bool {
    matches!(
        value.trim(),
        "1" | "true" | "TRUE" | "yes" | "YES" | "on" | "ON"
    )
}

/// Append `tick_format=binary` when absent so `civ-server` may switch to
/// [`TickBroadcastFormat::Binary`] for this connection.
#[must_use]
pub fn with_tick_format_binary_query(url: &str) -> String {
    if url.contains("tick_format=") {
        return url.to_string();
    }
    let separator = if url.contains('?') { '&' } else { '?' };
    format!("{url}{separator}tick_format=binary")
}

/// Resolve the live WebSocket URL from environment variables.
///
/// Precedence:
/// 1. `CIV_WS_URL` — full `ws://…` URL
/// 2. `CIV_SERVER_PORT` plus optional `CIV_WS_HOST` / `CIV_WS_PATH`
/// 3. [`default_live_ws_url`]
///
/// When [`ws_prefer_binary_from_env`] is true, appends `tick_format=binary`.
#[must_use]
pub fn resolve_live_ws_url() -> String {
    let base = resolve_ws_url_from_env();
    if ws_prefer_binary_from_env() {
        with_tick_format_binary_query(&base)
    } else {
        base
    }
}

/// How the Bevy standalone client runs simulation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "bevy", derive(bevy::prelude::Resource))]
pub enum AttachMode {
    /// In-process [`civ_engine::Simulation`] tick (default sandbox).
    #[default]
    Standalone,
    /// Live attach to `civ-server` over WebSocket (`CIVIS_ATTACH=server` or `CIV_WS_URL`).
    Server,
}

/// Resolve attach mode from environment (mirrors web dashboard precedence for server attach).
#[must_use]
pub fn resolve_attach_mode_from_env() -> AttachMode {
    resolve_attach_mode(
        std::env::var("CIVIS_ATTACH").ok().as_deref(),
        std::env::var("CIV_WS_URL").ok().as_deref(),
    )
}

/// Resolve attach mode from explicit env strings (testable without mutating process env).
#[must_use]
pub fn resolve_attach_mode(civis_attach: Option<&str>, civ_ws_url: Option<&str>) -> AttachMode {
    if civis_attach
        .map(|value| value.trim().eq_ignore_ascii_case("server"))
        .unwrap_or(false)
    {
        return AttachMode::Server;
    }
    if civ_ws_url
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
    {
        return AttachMode::Server;
    }
    AttachMode::Standalone
}

/// Resolve attach URL without the binary broadcast hint query parameter.
#[must_use]
pub fn resolve_ws_url_from_env() -> String {
    if let Ok(full) = std::env::var("CIV_WS_URL") {
        let trimmed = full.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }

    default_live_ws_url()
}

/// Decode a packed [`ChunkId`] into chunk grid coordinates (live bridge layout).
#[must_use]
pub fn decode_chunk_id(id: ChunkId) -> (i32, i32, i32) {
    let raw = id.0;
    let mut cx = ((raw >> 40) & 0x00ff_ffff) as i32;
    let mut cy = ((raw >> 16) & 0x00ff_ffff) as i32;
    let mut cz = (raw & 0x0000_ffff) as i32;
    if cx & 0x0080_0000 != 0 {
        cx |= !0x00ff_ffff;
    }
    if cy & 0x0080_0000 != 0 {
        cy |= !0x00ff_ffff;
    }
    if cz & 0x0000_8000 != 0 {
        cz |= !0x0000_ffff;
    }
    (cx, cy, cz)
}

/// Pack chunk grid coordinates into a [`ChunkId`] (inverse of [`decode_chunk_id`]).
#[must_use]
pub fn encode_chunk_id(cx: i32, cy: i32, cz: i32) -> ChunkId {
    let cx_u = cx as u32 as u64;
    let cy_u = cy as u32 as u64;
    let cz_u = cz as u32 as u64;
    ChunkId((cx_u << 40) | (cy_u << 16) | (cz_u & 0xFFFF))
}

/// Map a world-space position to the containing chunk grid cell.
#[must_use]
pub fn chunk_id_at_world_pos(pos: [f32; 3], chunk_edge: f32) -> ChunkId {
    let cx = (pos[0] / chunk_edge).floor() as i32;
    let cy = (pos[1] / chunk_edge).floor() as i32;
    let cz = (pos[2] / chunk_edge).floor() as i32;
    encode_chunk_id(cx, cy, cz)
}

/// Raycast against the procedural terrain surface and return the chunk at the hit.
#[cfg(feature = "bevy")]
#[must_use]
pub fn chunk_raycast_terrain(ray_origin: [f32; 3], ray_dir: [f32; 3], chunk_edge: f32) -> Option<ChunkId> {
    let max_distance = terrain::WORLD_SIZE * 2.0;
    let step = (chunk_edge * 0.5).max(1.0);
    let terrain_half = terrain::WORLD_SIZE * 0.5;

    let sample_surface = |pos: [f32; 3]| {
        terrain::terrain_surface_y(pos[0] + terrain_half, pos[2] + terrain_half)
    };

    let mut prev_t = 0.0_f32;
    let mut prev_pos = ray_origin;
    let mut prev_delta = prev_pos[1] - sample_surface(prev_pos);
    if prev_delta <= 0.0 {
        return Some(chunk_id_at_world_pos(prev_pos, chunk_edge));
    }

    let mut t = step;
    while t <= max_distance {
        let pos = [
            ray_origin[0] + ray_dir[0] * t,
            ray_origin[1] + ray_dir[1] * t,
            ray_origin[2] + ray_dir[2] * t,
        ];
        let delta = pos[1] - sample_surface(pos);
        if delta <= 0.0 {
            let mut lo = prev_t;
            let mut hi = t;
            let mut hit = pos;
            for _ in 0..10 {
                let mid = (lo + hi) * 0.5;
                hit = [
                    ray_origin[0] + ray_dir[0] * mid,
                    ray_origin[1] + ray_dir[1] * mid,
                    ray_origin[2] + ray_dir[2] * mid,
                ];
                let mid_delta = hit[1] - sample_surface(hit);
                if mid_delta > 0.0 {
                    lo = mid;
                } else {
                    hi = mid;
                }
            }
            return Some(chunk_id_at_world_pos(hit, chunk_edge));
        }
        prev_t = t;
        prev_pos = pos;
        prev_delta = delta;
        t += step;
    }

    let _ = (prev_pos, prev_delta);
    None
}

/// Resolve a focused chunk for minimap XZ grid selection against loaded chunk ids.
#[must_use]
pub fn focused_chunk_at_grid(cx: i32, cz: i32, preferred_cy: i32, loaded: &[u64]) -> ChunkId {
    let preferred = encode_chunk_id(cx, preferred_cy, cz);
    if loaded.contains(&preferred.0) {
        return preferred;
    }
    for &raw in loaded {
        let id = ChunkId(raw);
        let (x, _, z) = decode_chunk_id(id);
        if x == cx && z == cz {
            return id;
        }
    }
    preferred
}

/// Voxel chunk edge length in world units (16³ leaf chunks).
pub const VOXEL_CHUNK_EDGE: f32 = 16.0;

/// World-space centre of a chunk grid cell (for distance culling).
#[must_use]
pub fn chunk_world_centre(chunk_id: ChunkId, chunk_edge: f32) -> [f32; 3] {
    let (x, y, z) = decode_chunk_id(chunk_id);
    [
        (x as f32 + 0.5) * chunk_edge,
        (y as f32 + 0.5) * chunk_edge,
        (z as f32 + 0.5) * chunk_edge,
    ]
}

/// Euclidean distance from the camera to a chunk's world-space centre.
#[must_use]
pub fn chunk_distance_from_camera(chunk_id: ChunkId, camera_pos: [f32; 3], chunk_edge: f32) -> f32 {
    let centre = chunk_world_centre(chunk_id, chunk_edge);
    let dx = centre[0] - camera_pos[0];
    let dy = centre[1] - camera_pos[1];
    let dz = centre[2] - camera_pos[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Sphere-distance cull stub: render when chunk centre is within `max_dist`.
#[must_use]
pub fn should_render_chunk(chunk_id: ChunkId, camera_pos: [f32; 3], max_dist: f32) -> bool {
    chunk_distance_from_camera(chunk_id, camera_pos, VOXEL_CHUNK_EDGE) <= max_dist
}

/// Duration of the chunk spawn fade-in in seconds.
pub const CHUNK_FADE_DURATION_SECS: f32 = 0.3;

/// Linear fade alpha for a chunk spawn animation (`0.0` → `1.0` over
/// [`CHUNK_FADE_DURATION_SECS`]).
#[must_use]
pub fn chunk_fade_alpha(elapsed_secs: f32) -> f32 {
    if !elapsed_secs.is_finite() || elapsed_secs <= 0.0 {
        0.0
    } else if elapsed_secs >= CHUNK_FADE_DURATION_SECS {
        1.0
    } else {
        elapsed_secs / CHUNK_FADE_DURATION_SECS
    }
}

/// Returns true once the chunk fade-in has reached full opacity.
#[must_use]
pub fn chunk_fade_complete(elapsed_secs: f32) -> bool {
    elapsed_secs.is_finite() && elapsed_secs >= CHUNK_FADE_DURATION_SECS
}

/// Apply a fade alpha to an opaque sRGB triple (preserves RGB, sets alpha).
#[must_use]
pub fn chunk_fade_color(rgb: [f32; 3], alpha: f32) -> [f32; 4] {
    [rgb[0], rgb[1], rgb[2], alpha.clamp(0.0, 1.0)]
}

/// Default agent marker width in world units (before optional payload scale).
pub const AGENT_MARKER_WIDTH: f32 = 0.8;

/// Default agent marker height in world units (before optional payload scale).
pub const AGENT_MARKER_HEIGHT: f32 = 1.6;

/// Default agent marker depth in world units (before optional payload scale).
pub const AGENT_MARKER_DEPTH: f32 = 0.8;

/// Deterministic sRGB triple in `[0, 1]` for an opaque agent id (hash → hue).
#[must_use]
pub fn agent_color_from_id(agent_id: u64) -> [f32; 3] {
    let hash = splitmix64(agent_id);
    let hue = (hash as f32 / u64::MAX as f32).fract();
    hsv_to_rgb(hue, 0.62, 0.88)
}

/// Short label for an agent marker (`name` when provided, otherwise `#<id>`).
#[must_use]
pub fn agent_label_text(agent_id: u64, name: Option<&str>) -> String {
    if let Some(label) = name.map(str::trim).filter(|value| !value.is_empty()) {
        return label.to_string();
    }
    format!("#{agent_id}")
}

/// Uniform scale multiplier from a payload field (`1.0` when absent or invalid).
#[must_use]
pub fn agent_scale_multiplier(scale: f32) -> f32 {
    if scale.is_finite() && scale > 0.0 {
        scale
    } else {
        1.0
    }
}

fn splitmix64(mut value: u64) -> u64 {
    value = value.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = value;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn hsv_to_rgb(h: f32, s: f32, v: f32) -> [f32; 3] {
    let h = h.fract().max(0.0);
    let i = (h * 6.0).floor() as i32;
    let f = h * 6.0 - i as f32;
    let p = v * (1.0 - s);
    let q = v * (1.0 - f * s);
    let t = v * (1.0 - (1.0 - f) * s);
    match i % 6 {
        0 => [v, t, p],
        1 => [q, v, p],
        2 => [p, v, t],
        3 => [p, q, v],
        4 => [t, p, v],
        _ => [v, p, q],
    }
}

/// Inclusive chunk-grid bounds for the XZ plane: `(min_x, min_z, max_x, max_z)`.
pub type MinimapBounds = (i32, i32, i32, i32);

/// Map a loaded chunk into normalised minimap UV coordinates within `bounds`.
///
/// Returns `[u, v]` in `0.0..=1.0` at the centre of the chunk cell. `(0, 0)` is
/// the top-left of the bounds rectangle; `(1, 1)` is the bottom-right.
#[must_use]
pub fn chunk_to_minimap_uv(chunk_id: ChunkId, bounds: MinimapBounds) -> [f32; 2] {
    let (cx, _cy, cz) = decode_chunk_id(chunk_id);
    world_chunk_grid_to_minimap_uv(cx, cz, bounds)
}

/// Map world XZ (metres) into normalised minimap UV within `bounds`.
#[must_use]
pub fn world_xz_to_minimap_uv(x: f32, z: f32, bounds: MinimapBounds) -> [f32; 2] {
    let cx = (x / VOXEL_CHUNK_EDGE).floor() as i32;
    let cz = (z / VOXEL_CHUNK_EDGE).floor() as i32;
    world_chunk_grid_to_minimap_uv(cx, cz, bounds)
}

fn world_chunk_grid_to_minimap_uv(cx: i32, cz: i32, bounds: MinimapBounds) -> [f32; 2] {
    let (min_x, min_z, max_x, max_z) = bounds;
    let span_x = (max_x - min_x + 1).max(1) as f32;
    let span_z = (max_z - min_z + 1).max(1) as f32;
    [
        (cx - min_x) as f32 / span_x + 0.5 / span_x,
        (cz - min_z) as f32 / span_z + 0.5 / span_z,
    ]
}

/// Inverse of [`chunk_to_minimap_uv`]: map normalised minimap UV back to chunk grid XZ.
///
/// UV `(0, 0)` is the top-left of `bounds`; `(1, 1)` is the bottom-right. Values are
/// clamped to the inclusive chunk grid range.
#[must_use]
pub fn minimap_uv_to_chunk_grid(uv: [f32; 2], bounds: MinimapBounds) -> (i32, i32) {
    let (min_x, min_z, max_x, max_z) = bounds;
    let span_x = (max_x - min_x + 1).max(1) as f32;
    let span_z = (max_z - min_z + 1).max(1) as f32;
    let cx = (uv[0] * span_x + min_x as f32).floor() as i32;
    let cz = (uv[1] * span_z + min_z as f32).floor() as i32;
    (cx.clamp(min_x, max_x), cz.clamp(min_z, max_z))
}

/// Target [`day_factor`](presentation_day_factor_target) when `sim.snapshot` reports night.
pub const PRESENTATION_NIGHT_DAY_FACTOR: f32 = 0.32;

/// Day sky / clear colour (sRGB, matches web `0x87b7e0`).
pub const PRESENTATION_DAY_CLEAR_RGB: [f32; 3] = [0.529, 0.718, 0.878];

/// Night sky / clear colour (sRGB, matches web `0x0a1530`).
pub const PRESENTATION_NIGHT_CLEAR_RGB: [f32; 3] = [0.039, 0.082, 0.188];

/// Day ambient fill brightness (warm).
pub const PRESENTATION_DAY_AMBIENT_BRIGHTNESS: f32 = 0.45;

/// Night ambient fill brightness (dim cool).
pub const PRESENTATION_NIGHT_AMBIENT_BRIGHTNESS: f32 = 0.14;

/// Day ambient colour (warm sRGB).
pub const PRESENTATION_DAY_AMBIENT_RGB: [f32; 3] = [0.96, 0.93, 0.86];

/// Night ambient colour (cool sRGB).
pub const PRESENTATION_NIGHT_AMBIENT_RGB: [f32; 3] = [0.55, 0.68, 0.92];

/// Target presentation blend for directional light, ambient, and clear colour.
#[must_use]
pub fn presentation_day_factor_target(is_day: bool) -> f32 {
    if is_day {
        1.0
    } else {
        PRESENTATION_NIGHT_DAY_FACTOR
    }
}

/// Linear interpolation between two sRGB triples (`t` in `0.0..=1.0`).
#[must_use]
pub fn lerp_rgb(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    let t = t.clamp(0.0, 1.0);
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}

/// Clear / sky colour for a smoothed [`day_factor`](presentation_day_factor_target).
#[must_use]
pub fn presentation_clear_color_rgb(day_factor: f32) -> [f32; 3] {
    lerp_rgb(
        PRESENTATION_NIGHT_CLEAR_RGB,
        PRESENTATION_DAY_CLEAR_RGB,
        day_factor,
    )
}

/// Ambient fill brightness for a smoothed `day_factor`.
#[must_use]
pub fn presentation_ambient_brightness(day_factor: f32) -> f32 {
    PRESENTATION_NIGHT_AMBIENT_BRIGHTNESS
        + (PRESENTATION_DAY_AMBIENT_BRIGHTNESS - PRESENTATION_NIGHT_AMBIENT_BRIGHTNESS) * day_factor
}

/// Ambient fill colour for a smoothed `day_factor`.
#[must_use]
pub fn presentation_ambient_color_rgb(day_factor: f32) -> [f32; 3] {
    lerp_rgb(
        PRESENTATION_NIGHT_AMBIENT_RGB,
        PRESENTATION_DAY_AMBIENT_RGB,
        day_factor,
    )
}

/// Stub LOD selector: 0 = full detail, higher = coarser (max 3).
#[must_use]
pub fn mesh_lod_level(distance: f32) -> u8 {
    if distance < 32.0 {
        0
    } else if distance < 64.0 {
        1
    } else if distance < 128.0 {
        2
    } else {
        3
    }
}

/// Parse a JSON text WebSocket payload into a [`civ_protocol_3d::Frame3d`].
///
/// Mirrors the `ws_client` receive path; kept here so CI can test protocol
/// parsing without a GPU or live server.
#[cfg(any(test, feature = "bevy"))]
pub fn parse_frame3d_json(text: &str) -> Result<civ_protocol_3d::Frame3d, String> {
    serde_json::from_str(text).map_err(|err| err.to_string())
}

/// Parse an F3D0 binary WebSocket payload into a [`civ_protocol_3d::Frame3d`].
///
/// Wraps [`civ_protocol_3d::decode_frame3d_binary`] for the live attach path and
/// CI smoke tests without a GPU or live server.
#[cfg(any(test, feature = "bevy"))]
pub fn parse_frame3d_binary(bytes: &[u8]) -> Result<civ_protocol_3d::Frame3d, String> {
    civ_protocol_3d::decode_frame3d_binary(bytes).map_err(|err| format!("{err:?}"))
}

/// Decode a WebSocket payload the same way as [`ws_client`]: F3D0 binary first,
/// then UTF-8 JSON text fallback.
#[cfg(any(test, feature = "bevy"))]
pub fn parse_ws_payload(payload: &[u8]) -> Result<civ_protocol_3d::Frame3d, String> {
    if civ_protocol_3d::is_frame3d_binary(payload) {
        return parse_frame3d_binary(payload);
    }
    let text = std::str::from_utf8(payload).map_err(|err| err.to_string())?;
    parse_frame3d_json(text)
}

#[cfg(feature = "bevy")]
mod agent_needs {
    use bevy::prelude::{App, Component, Plugin, Query, Res, Resource, Update};
    use civ_agents::{Needs as AgentNeedsData, Position3d as AgentPositionData};
    use civ_voxel::material::{
        ACID, CO2, FIRE, LAVA, MOLD, MOSS, MOLTEN_METAL, PLANT, SALT_WATER, SMOKE, STEAM,
        TOXIC_GAS, WATER, WOOD,
    };
    use civ_voxel::{MaterialId, VoxelWorld, WorldCoord, FIXED_SCALE};

    /// Bevy component wrapper for the agent needs vector.
    #[derive(Component, Debug, Clone, Copy, PartialEq)]
    pub struct AgentNeeds {
        /// Food pressure.
        pub food: f32,
        /// Shelter pressure.
        pub shelter: f32,
        /// Safety pressure.
        pub safety: f32,
        /// Belonging pressure.
        pub belonging: f32,
    }

    impl From<AgentNeedsData> for AgentNeeds {
        fn from(value: AgentNeedsData) -> Self {
            Self {
                food: value.food,
                shelter: value.shelter,
                safety: value.safety,
                belonging: value.belonging,
            }
        }
    }

    impl From<AgentNeeds> for AgentNeedsData {
        fn from(value: AgentNeeds) -> Self {
            Self {
                food: value.food,
                shelter: value.shelter,
                safety: value.safety,
                belonging: value.belonging,
            }
        }
    }

    /// Bevy component wrapper for the agent world position.
    #[derive(Component, Debug, Clone, Copy, PartialEq)]
    pub struct Position {
        /// Fixed-point world coordinate.
        pub coord: WorldCoord,
    }

    impl From<AgentPositionData> for Position {
        fn from(value: AgentPositionData) -> Self {
            Self { coord: value.coord }
        }
    }

    impl From<Position> for AgentPositionData {
        fn from(value: Position) -> Self {
            Self { coord: value.coord }
        }
    }

    /// Bevy resource carrying the voxel world used by the needs tick.
    #[derive(Resource)]
    pub struct VoxelWorldRes {
        /// Backing voxel world.
        pub world: VoxelWorld<MaterialId>,
    }

    impl Default for VoxelWorldRes {
        fn default() -> Self {
            Self {
                world: VoxelWorld::new(FIXED_SCALE),
            }
        }
    }

    const SAMPLE_RADIUS: i64 = 1;
    const FOOD_RELIEF_PER_HIT: f32 = 0.02;
    const FOOD_DECAY_WITHOUT_SIGNAL: f32 = 0.008;
    const SAFETY_PRESSURE_PER_HIT: f32 = 0.03;
    const SAFETY_RELIEF_WITHOUT_SIGNAL: f32 = 0.006;

    fn classify_voxel(material: MaterialId) -> (bool, bool) {
        let food = matches!(material, WATER | SALT_WATER | PLANT | MOSS | MOLD | WOOD);
        let threat = matches!(
            material,
            LAVA | FIRE | ACID | TOXIC_GAS | SMOKE | STEAM | MOLTEN_METAL | CO2
        );
        (food, threat)
    }

    fn nearby_stimuli(world: &VoxelWorld<MaterialId>, center: WorldCoord) -> (u32, u32) {
        let mut food_hits = 0;
        let mut threat_hits = 0;
        'samples: for dx in -SAMPLE_RADIUS..=SAMPLE_RADIUS {
            for dy in -SAMPLE_RADIUS..=SAMPLE_RADIUS {
                for dz in -SAMPLE_RADIUS..=SAMPLE_RADIUS {
                    let sample = WorldCoord {
                        x: center.x + dx * FIXED_SCALE,
                        y: center.y + dy * FIXED_SCALE,
                        z: center.z + dz * FIXED_SCALE,
                    };
                    let (food, threat) = classify_voxel(world.read(sample));
                    if food {
                        food_hits += 1;
                    }
                    if threat {
                        threat_hits += 1;
                    }
                    if food_hits > 0 && threat_hits > 0 {
                        break 'samples;
                    }
                }
            }
        }
        (food_hits, threat_hits)
    }

    /// Tick hunger and safety pressure from nearby voxel-world signals.
    pub fn update_agent_needs(
        mut query: Query<(&mut AgentNeeds, &Position)>,
        world: Res<VoxelWorldRes>,
    ) {
        for (mut needs, position) in &mut query {
            let (food_hits, threat_hits) = nearby_stimuli(&world.world, position.coord);

            if food_hits > 0 {
                needs.food = (needs.food - FOOD_RELIEF_PER_HIT * food_hits as f32).max(0.0);
            } else {
                needs.food = (needs.food + FOOD_DECAY_WITHOUT_SIGNAL).min(1.0);
            }

            if threat_hits > 0 {
                needs.safety =
                    (needs.safety + SAFETY_PRESSURE_PER_HIT * threat_hits as f32).min(1.0);
            } else {
                needs.safety = (needs.safety - SAFETY_RELIEF_WITHOUT_SIGNAL).max(0.0);
            }
        }
    }

    /// Registers the agent-needs tick into the Bevy gameplay loop.
    pub struct AgentNeedsPlugin;

    impl Plugin for AgentNeedsPlugin {
        fn build(&self, app: &mut App) {
            app.init_resource::<VoxelWorldRes>()
                .add_systems(Update, update_agent_needs);
        }
    }
}

#[cfg(feature = "bevy")]
pub use agent_needs::{AgentNeeds, AgentNeedsPlugin, Position, VoxelWorldRes, update_agent_needs};

#[cfg(feature = "bevy")]
pub mod bevy_render;

#[cfg(feature = "bevy")]
/// Live Bevy/WebSocket attach path for the 3D reference client.
pub mod ws_client;

#[cfg(feature = "bevy")]
pub use ws_client::{WsClient, WsClientConfig};

#[cfg(test)]
mod tests {
    use super::*;

    /// Always-on smoke: the public surface compiles + default camera target is
    /// sensible.
    #[test]
    fn default_camera_target_is_sensible() {
        let t = CameraTarget::default();
        assert!(t.distance > 0.0);
        assert_eq!(t.centre, DEFAULT_CAMERA_CENTRE);
        let [x, y, z] = t.orbit_position();
        assert!(y > t.centre[1], "camera should sit above scene centre");
        assert!((x - t.centre[0]).hypot(z - t.centre[2]) > 1.0);
    }

    #[test]
    fn debug_render_wireframe_toggle_flips_state() {
        let mut debug = DebugRender::default();
        assert!(!debug.wireframe);
        assert!(debug.toggle_wireframe());
        assert!(debug.wireframe);
        assert!(!debug.toggle_wireframe());
    }

    #[test]
    fn live_hud_overlay_includes_connection_and_tick() {
        let line = LiveHudSnapshot {
            connection: WsConnectionState::Connected,
            connected: true,
            tick: Some(42),
            fps: 59.7,
            chunk_count: 3,
            agent_count: 5,
            building_count: 2,
            graph_parcel_count: 1,
            civilian_count: 12,
            faction_count: 2,
            ..Default::default()
        }
        .format_overlay();
        assert!(line.contains("60"));
        assert!(line.contains("42"));
        assert!(line.contains("connected"));
        assert!(line.contains("C:3"));
        assert!(line.contains("A:5"));
        assert!(line.contains("B:2"));
        assert!(line.contains("G:1"));
        assert!(line.contains("P:12"));
        assert!(line.contains("F:2"));
    }

    #[test]
    fn live_hud_overlay_includes_ws_rtt_when_present() {
        let line = LiveHudSnapshot {
            connection: WsConnectionState::Connected,
            connected: true,
            tick: Some(1),
            fps: 60.0,
            ws_rtt_ms: Some(12.4),
            ..Default::default()
        }
        .format_overlay();
        assert!(line.contains("RTT: 12ms"));
    }

    #[test]
    fn live_hud_overlay_shows_disconnected_without_tick() {
        let line = LiveHudSnapshot {
            connection: WsConnectionState::Disconnected,
            connected: false,
            tick: None,
            fps: 0.0,
            ..Default::default()
        }
        .format_overlay();
        assert!(line.contains("disconnected"));
        assert!(line.contains('—'));
    }

    #[test]
    fn live_hud_overlay_includes_focused_chunk_id() {
        let line = LiveHudSnapshot {
            connected: true,
            tick: Some(1),
            fps: 60.0,
            focused_chunk: Some(ChunkId(42)),
            ..Default::default()
        }
        .format_overlay();
        assert!(line.contains("chunk: 42"));
    }

    #[test]
    fn format_live_selection_labels_entity_kind() {
        assert_eq!(
            format_live_selection(SelectedLiveEntity {
                kind: LiveEntityKind::Agent,
                id: 7,
            }),
            "sel: agent #7"
        );
        assert_eq!(
            format_live_selection(SelectedLiveEntity {
                kind: LiveEntityKind::Building,
                id: 3,
            }),
            "sel: building #3"
        );
    }

    #[test]
    fn live_hud_overlay_includes_selected_live_entity() {
        let line = LiveHudSnapshot {
            connected: true,
            tick: Some(1),
            fps: 60.0,
            selected_live: Some(SelectedLiveEntity {
                kind: LiveEntityKind::GraphParcel,
                id: 11,
            }),
            ..Default::default()
        }
        .format_overlay();
        assert!(line.contains("sel: graph #11"));
    }

    #[test]
    fn live_hud_overlay_includes_last_event_suffix() {
        let line = LiveHudSnapshot {
            connected: true,
            tick: Some(9),
            fps: 60.0,
            last_event: Some("Entity #3 died (faction 1)".to_string()),
            ..Default::default()
        }
        .format_overlay();
        assert!(line.contains("| evt: Entity #3 died (faction 1)"));
    }

    #[test]
    fn live_hud_overlay_includes_pick_detail_suffix() {
        let line = LiveHudSnapshot {
            connected: true,
            tick: Some(1),
            fps: 60.0,
            pick_detail: Some("Ada | Farmer | 87%".to_string()),
            ..Default::default()
        }
        .format_overlay();
        assert!(line.contains("| Ada | Farmer | 87%"));
    }

    #[test]
    fn parse_jsonrpc_snapshot_meta_reads_is_day_and_tick() {
        let text = r#"{"jsonrpc":"2.0","id":3,"result":{"tick":12,"is_day":false,"population":4}}"#;
        let meta = parse_jsonrpc_snapshot_meta(text).expect("snapshot meta");
        assert!(!meta.is_day);
        assert_eq!(meta.tick, Some(12));
    }

    #[test]
    fn parse_jsonrpc_snapshot_meta_ignores_tick_broadcast() {
        let text = r#"{"tick":5,"voxel_delta":[]}"#;
        assert!(parse_jsonrpc_snapshot_meta(text).is_none());
    }

    #[test]
    fn encode_chunk_id_roundtrips_decode() {
        let id = encode_chunk_id(5, 7, 9);
        assert_eq!(decode_chunk_id(id), (5, 7, 9));
    }

    #[test]
    #[cfg(feature = "bevy")]
    fn chunk_raycast_terrain_hits_terrain_surface() {
        let origin = [0.0, 256.0, 0.0];
        let dir = [0.0, -1.0, 0.0];
        let chunk = chunk_raycast_terrain(origin, dir, VOXEL_CHUNK_EDGE).expect("hit");
        assert_eq!(decode_chunk_id(chunk), (0, 0, 0));
    }

    #[test]
    fn focused_chunk_at_grid_prefers_loaded_y_slice() {
        let raw = encode_chunk_id(2, 3, 4).0;
        let alt = encode_chunk_id(2, 9, 4).0;
        let picked = focused_chunk_at_grid(2, 4, 3, &[alt, raw]);
        assert_eq!(picked, ChunkId(raw));
    }

    /// MeshBuffer re-export is callable.
    #[test]
    fn mesh_buffer_default_is_empty() {
        let m = MeshBuffer::default();
        assert!(m.vertices.is_empty());
        assert!(m.indices.is_empty());
    }

    #[test]
    fn resolve_attach_mode_defaults_to_standalone() {
        assert_eq!(resolve_attach_mode(None, None), AttachMode::Standalone);
        assert_eq!(
            resolve_attach_mode(Some("watch"), None),
            AttachMode::Standalone
        );
    }

    #[test]
    fn resolve_attach_mode_server_from_env_strings() {
        assert_eq!(
            resolve_attach_mode(Some("server"), None),
            AttachMode::Server
        );
        assert_eq!(
            resolve_attach_mode(None, Some("ws://127.0.0.1:3000/ws")),
            AttachMode::Server
        );
        assert_eq!(
            resolve_attach_mode(Some("standalone"), Some("ws://127.0.0.1:3000/ws")),
            AttachMode::Server
        );
    }

    #[test]
    fn default_live_ws_url_matches_builder() {
        assert_eq!(
            default_live_ws_url(),
            live_ws_url(DEFAULT_WS_HOST, DEFAULT_WS_PORT, DEFAULT_WS_PATH)
        );
    }

    #[test]
    fn parse_ws_binary_env_flag_accepts_common_truthy_values() {
        assert!(parse_ws_binary_env_flag("1"));
        assert!(parse_ws_binary_env_flag(" true "));
        assert!(!parse_ws_binary_env_flag("0"));
        assert!(!parse_ws_binary_env_flag("false"));
    }

    #[test]
    fn with_tick_format_binary_query_appends_once() {
        let base = default_live_ws_url();
        assert_eq!(
            with_tick_format_binary_query(&base),
            format!("{base}?tick_format=binary")
        );
        assert_eq!(
            with_tick_format_binary_query(&format!("{base}?role=operator")),
            format!("{base}?role=operator&tick_format=binary")
        );
        assert_eq!(
            with_tick_format_binary_query(&format!("{base}?tick_format=binary")),
            format!("{base}?tick_format=binary")
        );
    }

    #[test]
    fn resolve_ws_url_from_env_parses_addr_and_path() {
        let url = {
            std::env::set_var("CIV_WS_URL", "");
            std::env::set_var("CIV_SERVER_PORT", "8765");
            std::env::set_var("CIV_WS_PATH", "/attach");
            resolve_ws_url_from_env()
        };
        assert_eq!(url, "ws://127.0.0.1:8765/attach");
        std::env::remove_var("CIV_WS_URL");
        std::env::remove_var("CIV_SERVER_PORT");
        std::env::remove_var("CIV_WS_PATH");
    }

    #[test]
    fn decode_chunk_id_unpacks_grid_coords() {
        let raw = (5_i64 << 40) | (7_i64 << 16) | 9_i64;
        assert_eq!(decode_chunk_id(ChunkId(raw as u64)), (5, 7, 9));
    }

    #[test]
    fn should_render_chunk_uses_sphere_distance() {
        let cam = DEFAULT_CAMERA_CENTRE;
        assert!(should_render_chunk(
            ChunkId(0),
            cam,
            DEFAULT_CAMERA_DISTANCE
        ));
        let far = ChunkId((5_i64 << 40) as u64);
        assert!(!should_render_chunk(far, cam, 32.0));
    }

    #[test]
    fn chunk_fade_alpha_ramps_linearly() {
        assert_eq!(chunk_fade_alpha(0.0), 0.0);
        assert!((chunk_fade_alpha(0.15) - 0.5).abs() < f32::EPSILON);
        assert_eq!(chunk_fade_alpha(CHUNK_FADE_DURATION_SECS), 1.0);
        assert_eq!(chunk_fade_alpha(CHUNK_FADE_DURATION_SECS + 1.0), 1.0);
    }

    #[test]
    fn chunk_fade_complete_after_duration() {
        assert!(!chunk_fade_complete(0.0));
        assert!(!chunk_fade_complete(0.29));
        assert!(chunk_fade_complete(CHUNK_FADE_DURATION_SECS));
    }

    #[test]
    fn chunk_fade_color_preserves_rgb() {
        let rgba = chunk_fade_color([0.72, 0.69, 0.62], 0.4);
        assert_eq!(rgba[0], 0.72);
        assert_eq!(rgba[1], 0.69);
        assert_eq!(rgba[2], 0.62);
        assert_eq!(rgba[3], 0.4);
    }

    #[test]
    fn chunk_fade_helpers_ignore_non_finite_elapsed() {
        assert_eq!(chunk_fade_alpha(f32::NAN), 0.0);
        assert!(!chunk_fade_complete(f32::NAN));
        assert_eq!(chunk_fade_alpha(f32::INFINITY), 0.0);
        assert!(!chunk_fade_complete(f32::INFINITY));
    }

    #[test]
    fn agent_color_from_id_is_in_unit_cube_and_distinct() {
        let a = agent_color_from_id(1);
        let b = agent_color_from_id(2);
        for channel in a {
            assert!((0.0..=1.0).contains(&channel));
        }
        assert_ne!(a, b);
        assert_eq!(agent_color_from_id(1), agent_color_from_id(1));
    }

    #[test]
    fn agent_label_text_prefers_name() {
        assert_eq!(agent_label_text(7, Some(" Ada ")), "Ada");
        assert_eq!(agent_label_text(7, None), "#7");
        assert_eq!(agent_label_text(7, Some("   ")), "#7");
    }

    #[test]
    fn agent_scale_multiplier_filters_invalid_values() {
        assert_eq!(agent_scale_multiplier(1.0), 1.0);
        assert_eq!(agent_scale_multiplier(1.5), 1.5);
        assert_eq!(agent_scale_multiplier(0.0), 1.0);
        assert_eq!(agent_scale_multiplier(f32::NAN), 1.0);
    }

    #[test]
    fn chunk_to_minimap_uv_maps_chunk_centre_in_bounds() {
        let bounds = (0, 0, 3, 3);
        let origin = chunk_to_minimap_uv(ChunkId(0), bounds);
        assert!((origin[0] - 0.125).abs() < f32::EPSILON);
        assert!((origin[1] - 0.125).abs() < f32::EPSILON);

        let raw = (3_i64 << 40) | (9_i64 << 16) | 3_i64;
        let corner = chunk_to_minimap_uv(ChunkId(raw as u64), bounds);
        assert!((corner[0] - 0.875).abs() < f32::EPSILON);
        assert!((corner[1] - 0.875).abs() < f32::EPSILON);
    }

    #[test]
    fn chunk_to_minimap_uv_centres_single_chunk_bounds() {
        let raw = (2_i64 << 40) | (4_i64 << 16) | 6_i64;
        let chunk = ChunkId(raw as u64);
        let bounds = (2, 6, 2, 6);
        let uv = chunk_to_minimap_uv(chunk, bounds);
        assert!((uv[0] - 0.5).abs() < f32::EPSILON);
        assert!((uv[1] - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn minimap_uv_to_chunk_grid_inverts_chunk_to_minimap_uv() {
        let bounds = (0, 0, 3, 3);
        let origin = ChunkId(0);
        let uv = chunk_to_minimap_uv(origin, bounds);
        assert_eq!(minimap_uv_to_chunk_grid(uv, bounds), (0, 0));

        let raw = (3_i64 << 40) | (9_i64 << 16) | 3_i64;
        let corner = ChunkId(raw as u64);
        let uv = chunk_to_minimap_uv(corner, bounds);
        assert_eq!(minimap_uv_to_chunk_grid(uv, bounds), (3, 3));
    }

    #[test]
    fn minimap_uv_to_chunk_grid_clamps_to_bounds() {
        let bounds = (1, 2, 4, 5);
        assert_eq!(minimap_uv_to_chunk_grid([0.0, 0.0], bounds), (1, 2));
        assert_eq!(minimap_uv_to_chunk_grid([1.0, 1.0], bounds), (4, 5));
    }

    #[test]
    fn mesh_lod_level_increases_with_distance() {
        assert_eq!(mesh_lod_level(0.0), 0);
        assert_eq!(mesh_lod_level(48.0), 1);
        assert_eq!(mesh_lod_level(200.0), 3);
    }

    #[test]
    fn presentation_day_factor_target_matches_web_night_blend() {
        assert_eq!(presentation_day_factor_target(true), 1.0);
        assert!((presentation_day_factor_target(false) - 0.3).abs() < 0.05);
    }

    #[test]
    fn presentation_clear_color_lerps_day_and_night() {
        let night = presentation_clear_color_rgb(0.0);
        assert_eq!(night, PRESENTATION_NIGHT_CLEAR_RGB);
        let day = presentation_clear_color_rgb(1.0);
        assert_eq!(day, PRESENTATION_DAY_CLEAR_RGB);
        let mid = presentation_clear_color_rgb(0.5);
        assert!(mid[2] > night[2] && mid[2] < day[2]);
    }

    #[test]
    fn presentation_ambient_warm_day_and_cool_night() {
        let day = presentation_ambient_color_rgb(1.0);
        let night = presentation_ambient_color_rgb(0.0);
        assert!(day[0] > night[0], "day ambient should be warmer (more red)");
        assert!(
            night[2] > day[2],
            "night ambient should be cooler (more blue)"
        );
        assert!(presentation_ambient_brightness(1.0) > presentation_ambient_brightness(0.0));
    }

    #[test]
    fn mesh_lod_level_boundary_thresholds() {
        assert_eq!(mesh_lod_level(31.9), 0);
        assert_eq!(mesh_lod_level(32.0), 1);
        assert_eq!(mesh_lod_level(63.9), 1);
        assert_eq!(mesh_lod_level(64.0), 2);
        assert_eq!(mesh_lod_level(127.9), 2);
        assert_eq!(mesh_lod_level(128.0), 3);
    }

    #[test]
    fn chunk_distance_from_camera_at_same_point_is_zero() {
        let cam = DEFAULT_CAMERA_CENTRE;
        let dist = chunk_distance_from_camera(ChunkId(0), cam, VOXEL_CHUNK_EDGE);
        assert!(dist.abs() < f32::EPSILON);
    }

    /// FR-CIV-BEVY-002 — JSON `Frame3d` payloads parse without a live socket.
    #[test]
    fn parse_frame3d_json_accepts_building_diff() {
        use civ_protocol_3d::{BuildingDiffFrame, BuildingProvenance, Frame3d};

        let frame = Frame3d::BuildingDiff(BuildingDiffFrame {
            tick: 9,
            provenance: BuildingProvenance::Procedural,
            buildings: Vec::new(),
            graph: None,
        });
        let json = serde_json::to_string(&frame).expect("serialize");
        let parsed = parse_frame3d_json(&json).expect("parse");
        assert_eq!(parsed.tick(), 9);
    }

    #[test]
    fn is_frame3d_binary_detects_magic() {
        use civ_protocol_3d::{is_frame3d_binary, FRAME3D_BINARY_MAGIC};

        assert!(is_frame3d_binary(FRAME3D_BINARY_MAGIC));
        assert!(!is_frame3d_binary(b"{\"VoxelDelta\""));
    }

    #[test]
    fn parse_ws_payload_accepts_binary_and_text() {
        use civ_protocol_3d::{
            encode_frame3d_binary, BuildingDiffFrame, BuildingProvenance, Frame3d,
        };

        let frame = Frame3d::BuildingDiff(BuildingDiffFrame {
            tick: 3,
            provenance: BuildingProvenance::Procedural,
            buildings: Vec::new(),
            graph: None,
        });
        let json = serde_json::to_string(&frame).expect("serialize");
        let bytes = encode_frame3d_binary(&frame).expect("encode");

        assert_eq!(parse_ws_payload(json.as_bytes()).expect("text"), frame);
        assert_eq!(parse_ws_payload(&bytes).expect("binary"), frame);
    }

    #[test]
    fn parse_ws_payload_prefers_binary_when_magic_present() {
        use civ_protocol_3d::{
            encode_frame3d_binary, BuildingDiffFrame, BuildingProvenance, Frame3d,
        };

        let frame = Frame3d::BuildingDiff(BuildingDiffFrame {
            tick: 11,
            provenance: BuildingProvenance::Procedural,
            buildings: Vec::new(),
            graph: None,
        });
        let bytes = encode_frame3d_binary(&frame).expect("encode");
        assert_eq!(parse_ws_payload(&bytes).expect("binary-first"), frame);
    }

    /// FR-CIV-BEVY-035 — `parse_ws_payload` decodes F3D0 binary for all six `Frame3d` kinds.
    #[test]
    fn parse_ws_payload_decodes_all_frame_kinds() {
        use civ_protocol_3d::{
            encode_frame3d_binary, AgentAppearanceFrame, AgentAppearanceUpdate, BuildingDiffEntry,
            BuildingDiffFrame, BuildingKind3d, BuildingProvenance, CivilianNeeds3d,
            CivilianStateEntry, CivilianStateFrame, EventFeedFrame, EventFeedMessage3d,
            FactionStateEntry, FactionStateFrame, FactionTreasury3d, Frame3d, Government3d,
            TechEvent3d, VoxelChunkDelta, VoxelDeltaFrame, WorldXZ,
        };
        use civ_voxel::{ChunkId, DirtyChunkEvent, MaterialId, WriteSeq};

        let frames = [
            Frame3d::VoxelDelta(VoxelDeltaFrame {
                tick: 1,
                deltas: vec![VoxelChunkDelta {
                    event: DirtyChunkEvent {
                        chunk_id: ChunkId(1),
                        write_seq: WriteSeq(1),
                    },
                    voxels: vec![MaterialId(0); 4],
                }],
            }),
            Frame3d::BuildingDiff(BuildingDiffFrame {
                tick: 2,
                provenance: BuildingProvenance::Procedural,
                buildings: vec![BuildingDiffEntry {
                    id: 1,
                    kind: BuildingKind3d::House,
                    tier: 0,
                    position: WorldXZ { x: 0.0, z: 0.0 },
                }],
                graph: None,
            }),
            Frame3d::AgentAppearance(AgentAppearanceFrame {
                tick: 3,
                updates: vec![AgentAppearanceUpdate {
                    agent_id: 1,
                    era: 0,
                    wardrobe: MaterialId(0),
                    tools: MaterialId(0),
                    scale: 1.0,
                    position: None,
                }],
            }),
            Frame3d::CivilianState(CivilianStateFrame {
                tick: 4,
                civilians: vec![CivilianStateEntry {
                    id: 1,
                    needs: CivilianNeeds3d::default(),
                    profession: String::new(),
                    genome_summary: Default::default(),
                    species: String::new(),
                    health: 1.0,
                }],
            }),
            Frame3d::FactionState(FactionStateFrame {
                tick: 5,
                factions: vec![FactionStateEntry {
                    id: 1,
                    era: 0,
                    government: Government3d::Unknown,
                    treasury: FactionTreasury3d::default(),
                }],
            }),
            Frame3d::EventFeed(EventFeedFrame {
                tick: 6,
                events: vec![EventFeedMessage3d::Tech(TechEvent3d {
                    faction_id: 1,
                    era: 1,
                    tech: "wheel".to_string(),
                })],
            }),
        ];

        for frame in frames {
            let bytes = encode_frame3d_binary(&frame).expect("encode");
            assert_eq!(parse_ws_payload(&bytes).expect("binary decode"), frame);
            let json = serde_json::to_string(&frame).expect("json");
            assert_eq!(
                parse_ws_payload(json.as_bytes()).expect("text decode"),
                frame
            );
        }
    }

    /// FR-CIV-BEVY-003 — F3D0 binary `Frame3d` payloads round-trip without a live socket.
    #[test]
    fn parse_frame3d_binary_roundtrips_building_diff() {
        use civ_protocol_3d::{
            encode_frame3d_binary, BuildingDiffFrame, BuildingProvenance, Frame3d,
        };

        let frame = Frame3d::BuildingDiff(BuildingDiffFrame {
            tick: 9,
            provenance: BuildingProvenance::Procedural,
            buildings: Vec::new(),
            graph: None,
        });
        let bytes = encode_frame3d_binary(&frame).expect("encode");
        let parsed = parse_frame3d_binary(&bytes).expect("parse");
        assert_eq!(parsed, frame);
    }
}
