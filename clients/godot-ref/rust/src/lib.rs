#![allow(clippy::result_large_err)] // godot_api generated closures

mod ux;

use godot::prelude::*;
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::Deserialize;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("tokio runtime"))
}

/// Build an HTTP URL under the civ-watch base (e.g. `http://127.0.0.1:9090/terrain`).
fn api_url(base: &str, path: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    let path = path.trim_start_matches('/');
    format!("{base}/{path}")
}

/// Validate the civ-watch HTTP base URL before fetching terrain or control routes.
fn validate_base_url(base: &str) -> Result<(), String> {
    let base = base.trim().trim_end_matches('/');
    if base.is_empty() {
        return Err(
            "base URL is empty — connect to civ-watch at http://127.0.0.1:9090 \
             (civ-server :3000 is WebSocket JSON-RPC, not terrain HTTP)"
                .into(),
        );
    }
    if !base.starts_with("http://") && !base.starts_with("https://") {
        return Err(format!(
            "base URL must start with http:// or https:// (got {base:?})"
        ));
    }
    Ok(())
}

/// Map a normalized height in `[0, 1]` to a biome id for height-based coloring fallback.
fn biome_from_height(height: f32) -> u8 {
    if height < 0.40 {
        2
    } else if height < 0.45 {
        1
    } else if height < 0.60 {
        0
    } else if height < 0.75 {
        4
    } else if height < 0.90 {
        5
    } else {
        3
    }
}

fn rgb_to_color(rgb: [f32; 3]) -> Color {
    Color::from_rgb(rgb[0], rgb[1], rgb[2])
}

/// RGB in `[0, 1]` for a biome id (single source of truth for Godot terrain coloring).
fn biome_rgb(biome: u8) -> [f32; 3] {
    match biome {
        0 => [0.25, 0.45, 0.20],
        1 => [0.84, 0.76, 0.46],
        2 => [0.20, 0.35, 0.48],
        3 => [0.56, 0.55, 0.42],
        4 => [0.14, 0.22, 0.14],
        5 => [0.45, 0.40, 0.30],
        _ => [0.55, 0.68, 0.45],
    }
}

/// Vertex color from normalized terrain height (height → biome → RGB).
fn height_rgb(height: f32) -> [f32; 3] {
    biome_rgb(biome_from_height(height))
}

/// Parse terrain JSON from the civ-watch HTTP API.
fn parse_terrain_json(text: &str) -> Result<TerrainPayload, String> {
    serde_json::from_str(text).map_err(|err| err.to_string())
}

async fn fetch_terrain_payload(client: &Client, url: String) -> TerrainPayload {
    match client.get(url).send().await {
        Ok(response) => match response.text().await {
            Ok(text) => parse_terrain_json(&text).unwrap_or_default(),
            Err(_) => TerrainPayload::default(),
        },
        Err(_) => TerrainPayload::default(),
    }
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct CivisClient {
    #[base]
    base: Base<Node>,
    client: Client,
    base_url: GString,
}

#[godot_api]
impl CivisClient {
    /// Static helper: biome id → vertex color (callable from GDScript as `CivisClient.biome_color(id)`).
    #[func]
    fn biome_color(biome: i32) -> Color {
        rgb_to_color(biome_rgb(biome.clamp(0, 255) as u8))
    }

    /// Static helper: normalized height → vertex color (`CivisClient.height_color(h)`).
    #[func]
    fn height_color(height: f32) -> Color {
        rgb_to_color(height_rgb(height))
    }

    #[func]
    fn connect(&mut self, url: GString) {
        self.base_url = url;
    }

    #[func]
    fn fetch_terrain(&self) -> PackedFloat32Array {
        if validate_base_url(&self.base_url.to_string()).is_err() {
            return PackedFloat32Array::new();
        }
        let url = api_url(&self.base_url.to_string(), "terrain");
        let heights = runtime()
            .block_on(fetch_terrain_payload(&self.client, url))
            .heights;
        let mut array = PackedFloat32Array::new();
        for height in heights {
            array.push(height);
        }
        array
    }

    #[func]
    fn fetch_terrain_biomes(&self) -> PackedByteArray {
        if validate_base_url(&self.base_url.to_string()).is_err() {
            return PackedByteArray::new();
        }
        let url = api_url(&self.base_url.to_string(), "terrain");
        let biomes = runtime()
            .block_on(fetch_terrain_payload(&self.client, url))
            .biomes;
        let mut array = PackedByteArray::new();
        for biome in biomes {
            array.push(biome);
        }
        array
    }

    #[func]
    fn latest_snapshot(&self) -> Dictionary {
        if validate_base_url(&self.base_url.to_string()).is_err() {
            return Dictionary::new();
        }
        let url = api_url(&self.base_url.to_string(), "snapshot");
        let snapshot = runtime().block_on(async {
            match self.client.get(url).send().await {
                Ok(response) => response
                    .json::<Option<SnapshotPayload>>()
                    .await
                    .ok()
                    .flatten(),
                Err(_) => None,
            }
        });
        let mut dict = Dictionary::new();
        if let Some(snapshot) = snapshot {
            dict.set("tick", snapshot.tick);
            dict.set("population", snapshot.population as i64);
            dict.set("voxel_dirty_count", snapshot.voxel_dirty_count as i64);
            let mut civ_pins = Array::new();
            for pin in snapshot.civ_pins {
                let mut pin_dict = Dictionary::new();
                pin_dict.set("idx", pin.idx as i64);
                pin_dict.set("x", pin.x);
                pin_dict.set("y", pin.y);
                civ_pins.push(&pin_dict);
            }
            dict.set("civ_pins", civ_pins);
        }
        dict
    }

    #[func]
    fn post_place_voxel(&self, x: i64, y: i64, z: i64, material: i32) {
        if validate_base_url(&self.base_url.to_string()).is_err() {
            return;
        }
        let url = api_url(&self.base_url.to_string(), "control/place_voxel");
        let body = serde_json::json!({ "x": x, "y": y, "z": z, "material": material });
        let _ = runtime().block_on(async { self.client.post(url).json(&body).send().await });
    }

    #[func]
    fn post_spawn_civilian(&self, x: f32, y: f32, faction: i32) {
        if validate_base_url(&self.base_url.to_string()).is_err() {
            return;
        }
        let url = api_url(&self.base_url.to_string(), "control/spawn_civilian");
        let body = ux::spawn_civilian_body(x, y, faction);
        let _entity = ux::entity_create_from_spawn(&body, body.faction as u64);
        let _ = runtime().block_on(async { self.client.post(url).json(&body).send().await });
    }

    /// Era index at a simulation tick (deterministic timelapse helper for GDScript).
    #[func]
    fn era_at_tick(tick: i64, era_length_ticks: i32) -> i32 {
        ux::TimelapseView::at_tick(tick.max(0) as u64, era_length_ticks.max(1) as u32).era as i32
    }

    /// Preview entity ids for a batch of spawn positions (spawn editor UX).
    #[func]
    fn preview_spawn_entity_ids(
        positions: PackedVector2Array,
        factions: PackedInt32Array,
        start_id: i64,
    ) -> PackedInt64Array {
        let n = positions.len().min(factions.len());
        let spawns: Vec<_> = (0..n)
            .map(|i| {
                let pos = positions.get(i);
                ux::spawn_civilian_body(pos.x, pos.y, factions.get(i))
            })
            .collect();
        let events = ux::spawn_batch_events(&spawns, start_id.max(0) as u64);
        let mut out = PackedInt64Array::new();
        for event in events {
            out.push(event.entity_id as i64);
        }
        out
    }

    /// `POST /control/speed` — pause (0) or 1× / 2× / 4× / 8× (matches civ-watch + civ-server).
    #[func]
    fn post_speed(&self, speed: i32) {
        if validate_base_url(&self.base_url.to_string()).is_err() {
            return;
        }
        let speed = speed.clamp(0, 8) as u8;
        if ux::validate_timelapse_speed(speed).is_err() {
            return;
        }
        let url = api_url(&self.base_url.to_string(), "control/speed");
        let body = serde_json::json!({ "speed": speed });
        let _ = runtime().block_on(async { self.client.post(url).json(&body).send().await });
    }
}

#[derive(Deserialize, Default)]
struct TerrainPayload {
    #[serde(default)]
    heights: Vec<f32>,
    #[serde(default)]
    biomes: Vec<u8>,
}

#[derive(Deserialize)]
struct SnapshotPayload {
    tick: u64,
    population: u64,
    voxel_dirty_count: usize,
    #[serde(default)]
    civ_pins: Vec<CivPinPayload>,
}

#[derive(Deserialize)]
struct CivPinPayload {
    idx: u32,
    x: f32,
    y: f32,
}

#[godot_api]
impl INode for CivisClient {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            client: Client::new(),
            base_url: GString::from("http://127.0.0.1:9090"),
        }
    }
}

struct CivisRustExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CivisRustExtension {}

pub fn civis_rust_init(level: InitLevel) {
    let _ = level;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_url_joins_base_and_path() {
        assert_eq!(
            api_url("http://127.0.0.1:9090", "terrain"),
            "http://127.0.0.1:9090/terrain"
        );
        assert_eq!(
            api_url("http://127.0.0.1:9090/", "control/place_voxel"),
            "http://127.0.0.1:9090/control/place_voxel"
        );
    }

    #[test]
    fn parse_terrain_json_accepts_heights_and_biomes() {
        let payload = parse_terrain_json(r#"{"heights":[1.0,2.5],"biomes":[3,4]}"#).expect("parse");
        assert_eq!(payload.heights, vec![1.0, 2.5]);
        assert_eq!(payload.biomes, vec![3, 4]);
    }

    #[test]
    fn validate_base_url_rejects_empty_and_hints_civ_watch() {
        let err = validate_base_url("").unwrap_err();
        assert!(err.contains("127.0.0.1:9090"));
        assert!(err.contains("civ-server :3000"));
    }

    #[test]
    fn validate_base_url_accepts_civ_watch_http() {
        validate_base_url("http://127.0.0.1:9090").expect("civ-watch url");
        validate_base_url("http://127.0.0.1:9090/").expect("trailing slash");
    }

    #[test]
    fn height_rgb_maps_biome_bands() {
        assert_eq!(height_rgb(0.10), biome_rgb(2)); // deep water
        assert_eq!(height_rgb(0.42), biome_rgb(1)); // sand
        assert_eq!(height_rgb(0.50), biome_rgb(0)); // grass
        assert_eq!(height_rgb(0.70), biome_rgb(4)); // forest
        assert_eq!(height_rgb(0.85), biome_rgb(5)); // stone
        assert_eq!(height_rgb(0.95), biome_rgb(3)); // snow
    }

    #[test]
    fn biome_rgb_covers_all_palette_ids() {
        for biome in 0..=5u8 {
            let rgb = biome_rgb(biome);
            assert!(rgb.iter().all(|c| (0.0..=1.0).contains(c)), "biome {biome}");
        }
        assert_eq!(biome_rgb(99), [0.55, 0.68, 0.45]); // unknown palette entry
    }
}

#[cfg(test)]
mod ux_tests {
    use super::ux::{
        spawn_batch_events, spawn_civilian_body, validate_timelapse_speed, TimelapseView,
        TIMELAPSE_SPEEDS,
    };

    /// FR-CIV-UX-000 — N UI spawns emit N entity-create events.
    #[test]
    fn spawn_emits_entity_events() {
        let n = 7usize;
        let spawns: Vec<_> = (0..n)
            .map(|i| spawn_civilian_body(i as f32 * 2.0, 1.5, i as i32))
            .collect();
        let events = spawn_batch_events(&spawns, 1_000);

        assert_eq!(events.len(), n);
        for (i, event) in events.iter().enumerate() {
            assert_eq!(event.entity_id, 1_000 + i as u64);
            assert_eq!(event.faction, i as u32);
        }
    }

    /// FR-CIV-UX-001 — timelapse rates reach the same tick/era as real-time playback.
    #[test]
    fn timelapse_no_divergence() {
        const ERA_LEN: u32 = 10;
        const TARGET_TICK: u64 = 127;

        let mut realtime = TimelapseView::at_tick(0, ERA_LEN);
        for _ in 0..TARGET_TICK {
            realtime.advance_frame(1, ERA_LEN);
        }
        let baseline = TimelapseView::at_tick(TARGET_TICK, ERA_LEN);
        assert_eq!(realtime, baseline);

        for &speed in &TIMELAPSE_SPEEDS {
            assert!(validate_timelapse_speed(speed).is_ok());
            let mut timelapse = TimelapseView::at_tick(0, ERA_LEN);
            timelapse.advance_to(TARGET_TICK, speed, ERA_LEN);
            assert_eq!(
                timelapse, baseline,
                "speed {speed}x diverged from real-time"
            );
        }
    }
}
