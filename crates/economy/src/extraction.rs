//! Extraction sites and resource yield for civ-economy.
//!
//! Defines the kinds of extractable resources, extraction sites, and the
//! per-tick extraction tick function. The [`find_extraction_site`] entry
//! point is the only voxel→economy coupling (see CIV-0100 §extraction).

/// Kinds of extractable resources in the world.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResourceKind {
    /// Metallic ore suitable for smelting into metal.
    #[default]
    Ore,
    /// Dimensional-stone blocks for construction.
    Stone,
    /// Raw timber for wood and tool production.
    Wood,
    /// Edible resources gathered or grown.
    Food,
}

/// Material ID range lower bound for ore deposits.
pub const ORE_MATERIAL_MIN: u16 = 10;
/// Material ID range upper bound for ore deposits.
pub const ORE_MATERIAL_MAX: u16 = 19;

/// Material ID range lower bound for stone deposits.
pub const STONE_MATERIAL_MIN: u16 = 1;
/// Material ID range upper bound for stone deposits.
pub const STONE_MATERIAL_MAX: u16 = 9;

/// Material ID range lower bound for wood (tree) deposits.
pub const WOOD_MATERIAL_MIN: u16 = 20;
/// Material ID range upper bound for wood (tree) deposits.
pub const WOOD_MATERIAL_MAX: u16 = 29;

/// Material ID range lower bound for food (farmable) deposits.
pub const FOOD_MATERIAL_MIN: u16 = 30;
/// Material ID range upper bound for food (farmable) deposits.
pub const FOOD_MATERIAL_MAX: u16 = 39;

/// Base extraction yield per worker per tick.
pub const BASE_YIELD_PER_WORKER: i64 = 2;

impl ResourceKind {
    /// Returns the voxel material ID range `(min, max)` for this resource kind.
    ///
    /// Used by the engine to scan voxel density maps and discover
    /// extraction sites. Ranges are non-overlapping and inclusive.
    pub fn material_range(&self) -> (u16, u16) {
        match self {
            ResourceKind::Ore => (ORE_MATERIAL_MIN, ORE_MATERIAL_MAX),
            ResourceKind::Stone => (STONE_MATERIAL_MIN, STONE_MATERIAL_MAX),
            ResourceKind::Wood => (WOOD_MATERIAL_MIN, WOOD_MATERIAL_MAX),
            ResourceKind::Food => (FOOD_MATERIAL_MIN, FOOD_MATERIAL_MAX),
        }
    }

    /// Returns a yield multiplier for this resource kind.
    ///
    /// Ore is denser and yields more per worker; food is lighter.
    pub fn yield_multiplier(&self) -> i64 {
        match self {
            ResourceKind::Ore => 3,
            ResourceKind::Stone => 2,
            ResourceKind::Wood => 2,
            ResourceKind::Food => 1,
        }
    }

    /// Checks whether a voxel material ID falls within this resource kind's band.
    pub fn matches_material(&self, material_id: u16) -> bool {
        let (min, max) = self.material_range();
        material_id >= min && material_id <= max
    }
}

/// A site where resources can be extracted.
///
/// Each site is tied to a settlement that claims it and a resource band
/// determined by the voxel material density at the site's position.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ExtractionSite {
    /// Unique site identifier.
    pub id: u32,
    /// The kind of resource this site yields.
    pub resource_kind: ResourceKind,
    /// The settlement that controls this extraction site.
    pub settlement_id: u64,
}

impl ExtractionSite {
    /// Compute extraction yield for this site given the number of workers.
    ///
    /// Yield = `BASE_YIELD_PER_WORKER × workers × kind.yield_multiplier()`.
    pub fn yield_per_tick(&self, workers: u32) -> i64 {
        BASE_YIELD_PER_WORKER * workers as i64 * self.resource_kind.yield_multiplier()
    }
}

/// An extractor assigned to an extraction site.
///
/// Tracks how many workers are operating at a given site and the site
/// they are assigned to.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Extractor {
    /// Unique extractor identifier.
    pub id: u32,
    /// The extraction site this extractor is assigned to.
    pub site_id: u32,
    /// Number of workers operating this extractor.
    pub workers: u32,
}

/// Discover extraction sites from the voxel world.
///
/// Returns all extraction sites currently active. The caller (typically
/// `civ-engine::Simulation::phase_economy`) iterates the returned sites
/// and calls [`tick_extraction`] on each to advance resource gathering.
///
/// Currently returns no sites; in a full integration this scans the
/// voxel material density map for regions matching the resource-kind
/// material bands defined in [`ResourceKind::material_range`].
pub fn find_extraction_site() -> Vec<ExtractionSite> {
    Vec::new()
}

/// Advance extraction for one tick at the given site.
///
/// Returns the quantity of raw material extracted. The caller maps the
/// site's [`ResourceKind`] to the appropriate engine [`ResourceType`](crate)
/// and deposits the yield into the owning settlement's stocks.
///
/// Default worker count is 1 when no [`Extractor`] is linked.
pub fn tick_extraction(site: &mut ExtractionSite) -> i64 {
    let default_workers = 1u32;
    site.yield_per_tick(default_workers)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_kind_material_ranges_are_non_overlapping() {
        let ranges = [
            ResourceKind::Ore.material_range(),
            ResourceKind::Stone.material_range(),
            ResourceKind::Wood.material_range(),
            ResourceKind::Food.material_range(),
        ];
        for i in 0..ranges.len() {
            for j in (i + 1)..ranges.len() {
                assert!(
                    ranges[i].1 < ranges[j].0 || ranges[j].1 < ranges[i].0,
                    "ranges overlap: {:?} vs {:?}",
                    ranges[i],
                    ranges[j]
                );
            }
        }
    }

    #[test]
    fn matches_material_hits_own_range() {
        assert!(ResourceKind::Ore.matches_material(15));
        assert!(!ResourceKind::Ore.matches_material(5));
        assert!(ResourceKind::Stone.matches_material(3));
        assert!(!ResourceKind::Stone.matches_material(25));
        assert!(ResourceKind::Wood.matches_material(24));
        assert!(ResourceKind::Food.matches_material(35));
    }

    #[test]
    fn yield_per_tick_scales_with_workers() {
        let site = ExtractionSite {
            id: 1,
            resource_kind: ResourceKind::Ore,
            settlement_id: 100,
        };
        // Ore multiplier=3, BASE_YIELD=2 -> per_worker=6
        assert_eq!(site.yield_per_tick(1), 6);
        assert_eq!(site.yield_per_tick(5), 30);
    }

    #[test]
    fn tick_extraction_returns_yield() {
        let mut site = ExtractionSite {
            id: 2,
            resource_kind: ResourceKind::Food,
            settlement_id: 200,
        };
        // Food multiplier=1, BASE_YIELD=2, workers=1 -> 2
        assert_eq!(tick_extraction(&mut site), 2);
    }

    #[test]
    fn find_extraction_site_returns_empty_initially() {
        assert!(find_extraction_site().is_empty());
    }

    #[test]
    fn yield_multiplier_varies_by_kind() {
        assert!(ResourceKind::Ore.yield_multiplier() > ResourceKind::Food.yield_multiplier());
    }
}
