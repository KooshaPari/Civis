//! Resource extraction helpers for voxel-backed economy sites.

use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};
use glam::IVec3;
use serde::{Deserialize, Serialize};

/// Raw resource categories produced by extraction sites.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ResourceKind {
    /// Ore from mining.
    Ore,
    /// Grain or other farm output.
    Grain,
    /// Stone from quarrying.
    Stone,
    /// Fish from fisheries.
    Fish,
}

/// A voxel-backed extraction site.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ExtractionSite {
    /// Ore-rich deposit suitable for mining.
    Mine {
        /// Estimated ore density in the sampled volume.
        ore_density: f32,
    },
    /// Fertile land suitable for farms.
    Farm {
        /// Estimated fertility in the sampled volume.
        fertility: f32,
    },
    /// Stone-rich deposit suitable for quarrying.
    Quarry {
        /// Estimated stone density in the sampled volume.
        stone_density: f32,
    },
    /// Water-rich region suitable for fishing.
    Fishery {
        /// Estimated fish stock in the sampled volume.
        fish_stock: f32,
    },
}

/// A worker group assigned to an extraction site.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Extractor {
    /// Site being worked.
    pub site: ExtractionSite,
    /// Number of workers assigned to the site.
    pub workers: u32,
    /// Multiplier for the tools used by the workers.
    pub tool_quality: f32,
    /// World position of the site in voxel coordinates.
    pub pos: IVec3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SiteBand {
    Mine,
    Farm,
    Quarry,
    Fishery,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct SiteSamples {
    mine: u32,
    farm: u32,
    quarry: u32,
    fishery: u32,
    occupied: u32,
    total: u32,
}

/// Compute extraction output for a single tick.
///
/// Mines lose output as the site depletes: `0.95^(tick % 100)` is applied only
/// to [`ExtractionSite::Mine`].
#[must_use]
pub fn tick_extraction(ext: &Extractor, tick: u64) -> (ResourceKind, f32) {
    let workers = ext.workers as f32;
    let tool_quality = ext.tool_quality.max(0.0);
    match ext.site {
        ExtractionSite::Mine { ore_density } => {
            let depletion = 0.95_f32.powi((tick % 100) as i32);
            (
                ResourceKind::Ore,
                workers * tool_quality * ore_density.max(0.0) * depletion,
            )
        }
        ExtractionSite::Farm { fertility } => (
            ResourceKind::Grain,
            workers * tool_quality * fertility.max(0.0),
        ),
        ExtractionSite::Quarry { stone_density } => (
            ResourceKind::Stone,
            workers * tool_quality * stone_density.max(0.0),
        ),
        ExtractionSite::Fishery { fish_stock } => (
            ResourceKind::Fish,
            workers * tool_quality * fish_stock.max(0.0),
        ),
    }
}

/// Find the dominant extraction site around `pos` by sampling voxel materials.
///
/// The sampler scans a sphere with the supplied `radius` and classifies the
/// observed materials into broad site bands. Empty samples are ignored, but a
/// dominant air/water band can still produce a fishery when mixed with solids.
#[must_use]
pub fn find_extraction_site(
    world: &VoxelWorld<MaterialId>,
    pos: IVec3,
    radius: u32,
) -> Option<ExtractionSite> {
    let radius = i64::from(radius);
    let center = WorldCoord {
        x: i64::from(pos.x),
        y: i64::from(pos.y),
        z: i64::from(pos.z),
    };
    let mut samples = SiteSamples::default();

    for dz in -radius..=radius {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx * dx + dy * dy + dz * dz > radius * radius {
                    continue;
                }

                samples.total = samples.total.saturating_add(1);
                let material = world.read(WorldCoord {
                    x: center.x + dx,
                    y: center.y + dy,
                    z: center.z + dz,
                });

                if material.0 == 0 {
                    samples.fishery = samples.fishery.saturating_add(1);
                    continue;
                }

                samples.occupied = samples.occupied.saturating_add(1);
                match classify_material(material) {
                    Some(SiteBand::Mine) => samples.mine = samples.mine.saturating_add(1),
                    Some(SiteBand::Farm) => samples.farm = samples.farm.saturating_add(1),
                    Some(SiteBand::Quarry) => samples.quarry = samples.quarry.saturating_add(1),
                    Some(SiteBand::Fishery) => samples.fishery = samples.fishery.saturating_add(1),
                    None => {}
                }
            }
        }
    }

    if samples.occupied == 0 {
        return None;
    }

    let total = samples.total.max(1) as f32;
    let (band, score) = [
        (SiteBand::Mine, samples.mine),
        (SiteBand::Farm, samples.farm),
        (SiteBand::Quarry, samples.quarry),
        (SiteBand::Fishery, samples.fishery),
    ]
    .into_iter()
    .max_by_key(|(_, count)| *count)
    .unwrap();

    if score == 0 {
        return None;
    }

    let density = score as f32 / total;
    Some(match band {
        SiteBand::Mine => ExtractionSite::Mine {
            ore_density: density,
        },
        SiteBand::Farm => ExtractionSite::Farm { fertility: density },
        SiteBand::Quarry => ExtractionSite::Quarry {
            stone_density: density,
        },
        SiteBand::Fishery => ExtractionSite::Fishery {
            fish_stock: density,
        },
    })
}

fn classify_material(material: MaterialId) -> Option<SiteBand> {
    match material.0 {
        1 | 2 => Some(SiteBand::Farm),
        3 | 4 => Some(SiteBand::Quarry),
        5 | 6 => Some(SiteBand::Mine),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_world(material: u16) -> VoxelWorld<MaterialId> {
        let mut world = VoxelWorld::new(1);
        for z in -2..=2 {
            for y in -2..=2 {
                for x in -2..=2 {
                    world.write(WorldCoord { x, y, z }, MaterialId(material));
                }
            }
        }
        world
    }

    #[test]
    fn tick_extraction_scales_output_and_depletes_mines() {
        let mine = Extractor {
            site: ExtractionSite::Mine { ore_density: 0.5 },
            workers: 10,
            tool_quality: 1.2,
            pos: IVec3::new(1, 2, 3),
        };
        let farm = Extractor {
            site: ExtractionSite::Farm { fertility: 0.5 },
            workers: 10,
            tool_quality: 1.2,
            pos: IVec3::new(1, 2, 3),
        };
        let quarry = Extractor {
            site: ExtractionSite::Quarry { stone_density: 0.5 },
            workers: 10,
            tool_quality: 1.2,
            pos: IVec3::new(1, 2, 3),
        };
        let fishery = Extractor {
            site: ExtractionSite::Fishery { fish_stock: 0.5 },
            workers: 10,
            tool_quality: 1.2,
            pos: IVec3::new(1, 2, 3),
        };

        assert_eq!(tick_extraction(&farm, 7), (ResourceKind::Grain, 6.0));
        assert_eq!(tick_extraction(&quarry, 7), (ResourceKind::Stone, 6.0));
        assert_eq!(tick_extraction(&fishery, 7), (ResourceKind::Fish, 6.0));

        let (_, tick0) = tick_extraction(&mine, 0);
        let (_, tick1) = tick_extraction(&mine, 1);
        assert!((tick0 - 6.0).abs() < f32::EPSILON);
        assert!(tick1 < tick0);
        assert_eq!(tick_extraction(&mine, 0).0, ResourceKind::Ore);
    }

    #[test]
    fn find_extraction_site_classifies_supported_material_bands() {
        let cases = [
            (1u16, ExtractionSite::Farm { fertility: 1.0 }),
            (3u16, ExtractionSite::Quarry { stone_density: 1.0 }),
            (5u16, ExtractionSite::Mine { ore_density: 1.0 }),
        ];

        for (material, expected) in cases {
            let world = sample_world(material);
            let site = find_extraction_site(&world, IVec3::ZERO, 2).expect("site");
            assert_eq!(site, expected);
        }
    }

    #[test]
    fn find_extraction_site_prefers_fishery_over_empty_world_none() {
        let empty = VoxelWorld::<MaterialId>::new(1);
        assert_eq!(find_extraction_site(&empty, IVec3::ZERO, 2), None);

        let mut mixed = VoxelWorld::new(1);
        for z in -2..=2 {
            for y in -2..=2 {
                for x in -2..=2 {
                    mixed.write(WorldCoord { x, y, z }, MaterialId(0));
                }
            }
        }
        mixed.write(WorldCoord { x: 0, y: 0, z: 0 }, MaterialId(1));

        let site = find_extraction_site(&mixed, IVec3::ZERO, 2).expect("site");
        assert!(matches!(site, ExtractionSite::Fishery { fish_stock } if fish_stock > 0.0));
    }
}
