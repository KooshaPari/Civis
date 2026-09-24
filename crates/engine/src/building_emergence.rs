//! Building emergence wiring — culture + biome + era style vectors and settlement anchors.
//!
//! NFR-C-05 — `BTreeMap` enforcement: simulation state uses ordered
//! `BTreeMap` exclusively (no `HashMap`) so iteration order is deterministic
//! across runs and replays. This is enforced by the custom clippy lint
//! `sim_hashmap_forbidden` and the pre-commit grep scan.

use std::collections::BTreeMap;

use civ_agents::{ClusterMember, Position3d};
use civ_build::{
    clustered_parcel_offset, culture_id_from_traits, default_architecture_tile_sets,
    era_gated_demand_signals, era_index_from_pop_tech, facade_for_emergence,
    wealth_permille_from_stocks, BiomeStyleTag, EmergentStyleKey,
};
use civ_planet::{BiomeKind, GeologyMap};
use civ_voxel::WorldCoord;
use hecs::World;

use crate::engine::{BuildingType, Resources, Simulation};

/// Maps planet biomes to compact architectural style tags.
#[must_use]
pub fn biome_style_tag(kind: BiomeKind) -> BiomeStyleTag {
    match kind {
        BiomeKind::Desert | BiomeKind::Shrubland | BiomeKind::Steppe | BiomeKind::Savanna => {
            BiomeStyleTag::ARID
        }
        BiomeKind::Forest | BiomeKind::Rainforest | BiomeKind::Taiga | BiomeKind::Grassland => {
            BiomeStyleTag::FOREST
        }
        BiomeKind::Ocean | BiomeKind::Beach | BiomeKind::Wetland | BiomeKind::Mangrove => {
            BiomeStyleTag::COASTAL
        }
        BiomeKind::Tundra | BiomeKind::Glacier | BiomeKind::Alpine | BiomeKind::Mountain => {
            BiomeStyleTag::COLD
        }
        BiomeKind::Plains => BiomeStyleTag::NEUTRAL,
    }
}

/// Dominant multi-member settlement cluster and its centroid anchor.
#[must_use]
pub fn settlement_build_anchor(world: &World) -> (Option<u64>, WorldCoord) {
    let mut cluster_positions: BTreeMap<u64, Vec<(i64, i64, i64)>> = BTreeMap::new();
    for (_, (member, pos)) in world.query::<(&ClusterMember, &Position3d)>().iter() {
        cluster_positions
            .entry(member.cluster.0)
            .or_default()
            .push((pos.coord.x, pos.coord.y, pos.coord.z));
    }

    let best = cluster_positions
        .into_iter()
        .filter(|(_, positions)| positions.len() >= 2)
        .max_by_key(|(_, positions)| positions.len());

    match best {
        Some((cluster_id, positions)) => (
            Some(cluster_id),
            civ_build::settlement_cluster_centroid(&positions),
        ),
        None => (None, WorldCoord { x: 0, y: 0, z: 0 }),
    }
}

/// Culture profile for a settlement cluster (falls back to cluster-id seed).
#[must_use]
pub fn culture_traits_for_cluster(sim: &Simulation, cluster_id: u64) -> [f32; 4] {
    sim.emergence
        .cluster_cultures
        .get(&cluster_id)
        .map(|profile| profile.traits)
        .unwrap_or_else(|| {
            [
                ((cluster_id % 256) as f32) / 255.0,
                (((cluster_id >> 8) % 256) as f32) / 255.0,
                (((cluster_id >> 16) % 256) as f32) / 255.0,
                (((cluster_id >> 24) % 256) as f32) / 255.0,
            ]
        })
}

/// Emergent style key from live simulation state.
#[must_use]
pub fn emergent_style_key_for_sim(
    sim: &Simulation,
    cluster_id: Option<u64>,
    geology: &GeologyMap,
    anchor: &WorldCoord,
) -> EmergentStyleKey {
    let traits = cluster_id
        .map(|id| culture_traits_for_cluster(sim, id))
        .unwrap_or([0.25, 0.25, 0.25, 0.25]);
    let culture = culture_id_from_traits(traits);
    let era = era_index_from_pop_tech(sim.state.population, sim.researched_tech_count());
    let wood = sim.state.resources.wood.to_num::<i64>();
    let metal = sim.state.resources.metal.to_num::<i64>();
    let wealth = wealth_permille_from_stocks(wood, metal);
    let nx = (anchor.x as f32 / civ_voxel::FIXED_SCALE as f32).clamp(0.0, 1.0);
    let nz = (anchor.z as f32 / civ_voxel::FIXED_SCALE as f32).clamp(0.0, 1.0);
    let biome = biome_style_tag(geology.biome_at_normalized(nx, nz));
    EmergentStyleKey::new(culture, era, wealth, biome)
}

/// Returns true when the building type is unlocked at the current build era.
#[must_use]
pub fn building_type_unlocked_at_era(building_type: BuildingType, era: u16) -> bool {
    civ_build::building_type_unlocked(building_type.as_str(), era)
}

impl BuildingType {
    /// Wire-safe label for era-gate tables.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Farm => "Farm",
            Self::Mine => "Mine",
            Self::Barracks => "Barracks",
            Self::Temple => "Temple",
            Self::Market => "Market",
            Self::House => "House",
            Self::CityCenter => "CityCenter",
        }
    }

    /// Minimum build-era required before this type may be placed.
    #[must_use]
    pub fn min_era(self) -> u16 {
        civ_build::building_type_min_era(self.as_str())
    }
}

/// Shared tile-set registry (lazy static via function — deterministic contents).
#[must_use]
pub fn architecture_tile_sets() -> &'static [civ_build::TileSetProfile] {
    use std::sync::OnceLock;
    static REGISTRY: OnceLock<Vec<civ_build::TileSetProfile>> = OnceLock::new();
    REGISTRY.get_or_init(default_architecture_tile_sets)
}

/// Apply emergence facades to newly allocated parcel ids.
pub fn apply_emergence_facades(
    sim: &mut Simulation,
    cluster_id: Option<u64>,
    style: EmergentStyleKey,
    signals: civ_build::DemandSignals,
    allocated: &[civ_build::BuildingId],
) {
    let tile_sets = architecture_tile_sets();
    for (index, id) in allocated.iter().enumerate() {
        let facade = facade_for_emergence(style, &signals, tile_sets);
        sim.building_graph_mut().set_facade(*id, facade);
        if let Some(cluster) = cluster_id {
            sim.building_graph_mut().assign_to_cluster(cluster, *id);
            let offset = clustered_parcel_offset(cluster, index as u32, 16);
            if let Some(parcel) = sim
                .building_graph_mut()
                .parcels
                .iter_mut()
                .find(|p| p.id == *id)
            {
                parcel.origin.x = parcel.origin.x.saturating_add(offset.x);
                parcel.origin.z = parcel.origin.z.saturating_add(offset.z);
            }
        }
    }
}

/// Era-gated demand wrapper used by `phase_construction_sites`.
#[must_use]
pub fn emergence_demand_signals(
    sim: &Simulation,
    raw: civ_build::DemandSignals,
    era: u16,
) -> civ_build::DemandSignals {
    era_gated_demand_signals(raw, era)
}

/// Resource stocks as integer units for material headroom.
#[must_use]
pub fn resource_stock_units(resources: &Resources) -> (i64, i64) {
    (
        resources.wood.to_num::<i64>(),
        resources.metal.to_num::<i64>(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use civ_planet::defaults_earthlike;

    /// FR-CIV-ARCH — biome tags partition Whittaker kinds.
    #[test]
    fn biome_style_tag_maps_desert_to_arid() {
        assert_eq!(biome_style_tag(BiomeKind::Desert), BiomeStyleTag::ARID);
        assert_eq!(biome_style_tag(BiomeKind::Forest), BiomeStyleTag::FOREST);
    }

    /// FR-CIV-ARCH — era-gated building types follow min-era table.
    #[test]
    fn building_type_min_era_gates() {
        assert!(BuildingType::Farm.min_era() == 0);
        assert!(BuildingType::Temple.min_era() == 3);
        assert!(!building_type_unlocked_at_era(BuildingType::Barracks, 2));
        assert!(building_type_unlocked_at_era(BuildingType::Barracks, 4));
    }

    /// FR-CIV-ARCH — style key derives from simulation seed state.
    #[test]
    fn emergent_style_key_uses_population_era() {
        let sim = Simulation::with_seed(42);
        let geology = GeologyMap::seed(&defaults_earthlike().0);
        let anchor = WorldCoord { x: 0, y: 0, z: 0 };
        let key = emergent_style_key_for_sim(&sim, None, &geology, &anchor);
        assert!(key.era <= 5);
    }

    // NFR-C-05 — simulation state uses ordered `BTreeMap` exclusively (no
    // `HashMap`) so iteration order is deterministic across runs and replays.
    #[test]
    fn nfr_c_05_settlement_anchor_deterministic_across_insert_orders() {
        fn two_member_world(reverse_insert: bool) -> World {
            let mut world = World::new();
            let spawn = |world: &mut World, cluster: u64, x: i64, z: i64| {
                let _ = world.spawn((
                    civ_agents::ClusterMember {
                        cluster: civ_agents::ClusterId(cluster),
                    },
                    Position3d {
                        coord: WorldCoord { x, y: 0, z },
                    },
                ));
            };
            // Two clusters tied at 2 members each (both pass the >= 2 filter).
            if reverse_insert {
                spawn(&mut world, 2, 100, 100);
                spawn(&mut world, 2, 200, 200);
                spawn(&mut world, 5, 10, 30);
                spawn(&mut world, 5, 30, 50);
            } else {
                spawn(&mut world, 5, 10, 30);
                spawn(&mut world, 5, 30, 50);
                spawn(&mut world, 2, 100, 100);
                spawn(&mut world, 2, 200, 200);
            }
            // Singleton cluster never qualifies (needs >= 2 members).
            spawn(&mut world, 9, 900, 900);
            world
        }

        let a = two_member_world(false);
        let b = two_member_world(true);
        let (id_a, coord_a) = settlement_build_anchor(&a);
        let (id_b, coord_b) = settlement_build_anchor(&b);

        // Same ordered state => identical anchor regardless of insert order.
        assert_eq!((id_a, coord_a), (id_b, coord_b));
        // Ties resolve deterministically to the larger cluster id (BTreeMap
        // ascending iteration + max_by_key keeps the last maximum).
        assert_eq!(id_a, Some(5));
        // Centroid is the integer mean of the winning cluster's positions.
        assert_eq!(
            coord_a,
            WorldCoord {
                x: 20,
                y: 0,
                z: 40
            }
        );

        // Empty world: no cluster qualifies, anchor falls back to origin.
        let empty = World::new();
        let (none_id, origin) = settlement_build_anchor(&empty);
        assert_eq!(none_id, None);
        assert_eq!(
            origin,
            WorldCoord {
                x: 0,
                y: 0,
                z: 0
            }
        );
    }
}
