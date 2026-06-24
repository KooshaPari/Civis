#![allow(clippy::result_large_err)] // godot_api generated closures

mod f3d0_mesh;
mod ux;
mod ws_frame;

use civ_engine::{Simulation, SimulationSnapshot};
use civ_voxel::WorldCoord;
use godot::prelude::*;

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

fn height_rgb(height: f32) -> [f32; 3] {
    biome_rgb(biome_from_height(height))
}

fn terrain_seed() -> u64 {
    42
}

fn terrain_noise(x: i32, z: i32) -> f32 {
    let mut v = terrain_seed()
        ^ ((x as i64 as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15))
        ^ ((z as i64 as u64).wrapping_mul(0xC2B2_AE3D_27D4_EB4F));
    v ^= v >> 33;
    v = v.wrapping_mul(0xff51_afd7_ed55_8ccd);
    v ^= v >> 33;
    v = v.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
    v ^= v >> 33;
    (v as f64 / u64::MAX as f64) as f32
}

fn terrain_height_at(sim: &Simulation, x: i32, z: i32) -> f32 {
    let mut top = 0.0f32;
    for y in (0..128).rev() {
        let value = sim
            .voxel()
            .read(WorldCoord {
                x: x as i64,
                y: y as i64,
                z: z as i64,
            })
            .0;
        if value != 0 {
            top = (y as f32 + 1.0) / 128.0;
            break;
        }
    }
    if top > 0.0 {
        return top;
    }

    let nx = x as f32 / 127.0;
    let nz = z as f32 / 127.0;
    let base = 0.36
        + 0.10 * (nx * std::f32::consts::TAU * 2.0).sin()
        + 0.08 * (nz * std::f32::consts::TAU * 1.5).cos()
        + 0.05 * ((nx + nz) * std::f32::consts::TAU * 3.0).sin();
    let jitter = (terrain_noise(x, z) - 0.5) * 0.08;
    (base + jitter).clamp(0.0, 1.0)
}

fn terrain_biome_at(sim: &Simulation, x: i32, z: i32) -> u8 {
    biome_from_height(terrain_height_at(sim, x, z))
}

fn simulation_snapshot_to_dict(snapshot: &SimulationSnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("tick", snapshot.tick as i64);
    dict.set("population", snapshot.population as i64);
    dict.set("voxel_dirty_count", 0i64);
    dict.set("citizen_count", snapshot.citizen_count as i64);
    dict.set("building_count", snapshot.building_count as i64);
    dict.set("military_count", snapshot.military_count as i64);
    dict.set(
        "energy_budget",
        snapshot.energy_budget.to_f64() as f32,
    );
    dict.set("births_this_tick", snapshot.births_this_tick as i64);
    dict.set("deaths_this_tick", snapshot.deaths_this_tick as i64);
    dict.set("damage_events", snapshot.damage_events as i64);

    let mut civ_pins: VarArray = VarArray::new();
    for idx in 0..snapshot.citizen_count.min(32) {
        let mut pin_dict = VarDictionary::new();
        pin_dict.set("idx", idx as i64);
        pin_dict.set("x", (idx as f32 * 0.03125).fract());
        pin_dict.set("y", (idx as f32 * 0.0625).fract());
        pin_dict.set("job", "unemployed");
        let pin_variant = pin_dict.to_variant();
        civ_pins.push(&pin_variant);
    }
    dict.set("civ_pins", civ_pins);

    let mut buildings: VarArray = VarArray::new();
    for idx in 0..snapshot.building_count.min(16) {
        let mut building = VarDictionary::new();
        building.set("id", idx as i64);
        building.set("kind", "Residential");
        building.set("x", (idx as f32 * 0.071).fract());
        building.set("y", (idx as f32 * 0.043).fract());
        let building_variant = building.to_variant();
        buildings.push(&building_variant);
    }
    dict.set("buildings", buildings);

    let mut military_units: VarArray = VarArray::new();
    for idx in 0..snapshot.military_count.min(16) {
        let mut unit = VarDictionary::new();
        unit.set("id", idx as i64);
        unit.set("unit_type", "Soldier");
        unit.set("x", (idx as f32 * 0.057).fract());
        unit.set("y", (idx as f32 * 0.089).fract());
        let unit_variant = unit.to_variant();
        military_units.push(&unit_variant);
    }
    dict.set("military_units", military_units);
    dict.set("is_day", true);

    dict
}

#[derive(GodotClass)]
#[class(base = Node)]
pub struct SimulationHost {
    #[base]
    base: Base<Node>,
    sim: Simulation,
}

#[godot_api]
impl SimulationHost {
    #[func]
    fn biome_color(biome: i32) -> Color {
        rgb_to_color(biome_rgb(biome.clamp(0, 255) as u8))
    }

    #[func]
    fn height_color(height: f32) -> Color {
        rgb_to_color(height_rgb(height))
    }

    #[func]
    fn tick(&mut self) {
        self.sim.tick();
    }

    #[func]
    fn snapshot(&self) -> VarDictionary {
        simulation_snapshot_to_dict(&self.sim.snapshot())
    }

    #[func]
    fn get_terrain(&self) -> VarDictionary {
        let mut heights = PackedFloat32Array::new();
        let mut biomes = PackedByteArray::new();
        for z in 0..128 {
            for x in 0..128 {
                let h = terrain_height_at(&self.sim, x, z);
                heights.push(h);
                biomes.push(terrain_biome_at(&self.sim, x, z));
            }
        }
        let mut dict = VarDictionary::new();
        dict.set("heights", heights);
        dict.set("biomes", biomes);
        dict
    }

    #[func]
    fn era_at_tick(tick: i64, era_length_ticks: i32) -> i32 {
        ux::TimelapseView::at_tick(tick.max(0) as u64, era_length_ticks.max(1) as u32).era as i32
    }

    #[func]
    fn preview_spawn_entity_ids(
        positions: PackedVector2Array,
        factions: PackedInt32Array,
        start_id: i64,
    ) -> PackedInt64Array {
        let n = positions.len().min(factions.len());
        let spawns: Vec<_> = (0..n)
            .map(|i| {
                let pos = positions.get(i).unwrap_or(Vector2::ZERO);
                let faction = factions.get(i).unwrap_or(0);
                ux::spawn_civilian_body(pos.x, pos.y, faction)
            })
            .collect();
        let events = ux::spawn_batch_events(&spawns, start_id.max(0) as u64);
        let mut out = PackedInt64Array::new();
        for event in events {
            out.push(event.entity_id as i64);
        }
        out
    }
}

#[godot_api]
impl INode for SimulationHost {
    fn init(base: Base<Node>) -> Self {
        Self {
            base,
            sim: Simulation::with_seed(42),
        }
    }
}

struct CivisRustExtension;

#[gdextension(entry_symbol = civis_rust_init)]
unsafe impl ExtensionLibrary for CivisRustExtension {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn biome_rgb_covers_all_palette_ids() {
        for biome in 0..=5u8 {
            let rgb = biome_rgb(biome);
            assert!(rgb.iter().all(|c| (0.0..=1.0).contains(c)), "biome {biome}");
        }
        assert_eq!(biome_rgb(99), [0.55, 0.68, 0.45]);
    }
}
