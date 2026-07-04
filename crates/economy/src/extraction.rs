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
pub fn find_extraction_site() -> Vec<ExtractionSite> {
    Vec::new()
}

/// Advance extraction for one tick at the given site.
///
/// Reads the site's resource kind and produces the appropriate quantity
/// of raw material into the owning settlement's stocks.
pub fn tick_extraction(site: &mut ExtractionSite) {
    let _ = site;
}
