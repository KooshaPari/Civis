use godot::prelude::*;
use once_cell::sync::OnceCell;
use reqwest::Client;
use serde::Deserialize;
use tokio::runtime::Runtime;

static RUNTIME: OnceCell<Runtime> = OnceCell::new();

fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("tokio runtime"))
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
    #[func]
    fn connect(&mut self, url: GString) {
        self.base_url = url;
    }

    #[func]
    fn fetch_terrain(&self) -> PackedFloat32Array {
        let url = format!("{}/terrain", self.base_url.to_string());
        let heights = runtime().block_on(async {
            match self.client.get(url).send().await {
                Ok(response) => match response.json::<TerrainPayload>().await {
                    Ok(payload) => payload.heights,
                    Err(_) => Vec::new(),
                },
                Err(_) => Vec::new(),
            }
        });
        let mut array = PackedFloat32Array::new();
        for height in heights {
            array.push(height);
        }
        array
    }

    #[func]
    fn fetch_terrain_biomes(&self) -> PackedByteArray {
        let url = format!("{}/terrain", self.base_url.to_string());
        let biomes = runtime().block_on(async {
            match self.client.get(url).send().await {
                Ok(response) => match response.json::<TerrainPayload>().await {
                    Ok(payload) => payload.biomes,
                    Err(_) => Vec::new(),
                },
                Err(_) => Vec::new(),
            }
        });
        let mut array = PackedByteArray::new();
        for biome in biomes {
            array.push(biome);
        }
        array
    }

    #[func]
    fn latest_snapshot(&self) -> Dictionary {
        let url = format!("{}/snapshot", self.base_url.to_string());
        let snapshot = runtime().block_on(async {
            match self.client.get(url).send().await {
                Ok(response) => response.json::<Option<SnapshotPayload>>().await.ok().flatten(),
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
        let url = format!("{}/control/place_voxel", self.base_url.to_string());
        let body = serde_json::json!({ "x": x, "y": y, "z": z, "material": material });
        let _ = runtime().block_on(async {
            self.client.post(url).json(&body).send().await
        });
    }

    #[func]
    fn post_spawn_civilian(&self, x: f32, y: f32, faction: i32) {
        let url = format!("{}/control/spawn_civilian", self.base_url.to_string());
        let body = serde_json::json!({ "x": x, "y": y, "faction": faction });
        let _ = runtime().block_on(async {
            self.client.post(url).json(&body).send().await
        });
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
