//! FR-CIV-ARCH emergence tiers — era grammar, biome style bias, settlement clustering,
//! and era-gated unlock tables (`BuildingGraph` extensions).

use std::collections::BTreeMap;

use civ_voxel::{MaterialId, WorldCoord};

use crate::{
    default_facade_for_era, ArchitectureMode, BuildingGraph, BuildingId, CultureEraWealthVector,
    DemandSignals, FacadeStyle, ParcelKind, TileSetProfile, facade_for_vector,
};

/// Compact biome tag for architectural style lookup (engine maps `BiomeKind` → this).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BiomeStyleTag(pub u8);

impl BiomeStyleTag {
    /// Neutral / unknown biome — no material bias.
    pub const NEUTRAL: Self = Self(0);
    /// Arid biomes — earth-tone palette shift.
    pub const ARID: Self = Self(1);
    /// Forested biomes — timber-forward palette.
    pub const FOREST: Self = Self(2);
    /// Coastal / wetland — lighter stone and reed accents.
    pub const COASTAL: Self = Self(3);
    /// Cold / alpine — dense stone, low window density bias.
    pub const COLD: Self = Self(4);
}

/// Emergent style key: culture + era + wealth + biome (`FR-CIV-ARCH-003`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EmergentStyleKey {
    /// Quantized culture id from trait vector hash.
    pub culture: u16,
    /// Build-era index (0 = mud-brick … 5 = arcology).
    pub era: u16,
    /// Wealth permille proxy before bucket quantization.
    pub wealth: u16,
    /// Biome architectural family.
    pub biome: BiomeStyleTag,
}

impl EmergentStyleKey {
    /// Builds a style key from emergence inputs.
    #[must_use]
    pub const fn new(culture: u16, era: u16, wealth: u16, biome: BiomeStyleTag) -> Self {
        Self {
            culture,
            era,
            wealth,
            biome,
        }
    }

    /// Maps to the tile-set lookup vector (culture XOR biome folds biome into family id).
    #[must_use]
    pub fn to_style_vector(self) -> CultureEraWealthVector {
        let culture = self.culture ^ (u16::from(self.biome.0) * 257);
        CultureEraWealthVector::new(culture, self.era, self.wealth)
    }
}

/// Deterministic culture id from a 4-trait meme vector.
#[must_use]
pub fn culture_id_from_traits(traits: [f32; 4]) -> u16 {
    let mut hash = 0_u32;
    for (i, value) in traits.iter().enumerate() {
        let quantized = (value.clamp(0.0, 1.0) * 255.0).round() as u32;
        hash ^= quantized.wrapping_mul(1_000_003).wrapping_add(i as u32 * 97);
    }
    (hash % 4096) as u16
}

/// Wealth permille from integer resource stocks (wood + metal proxy).
#[must_use]
pub fn wealth_permille_from_stocks(wood_units: i64, metal_units: i64) -> u16 {
    let total = wood_units.saturating_add(metal_units);
    ((total.min(1_000_000) * 1000) / 1_000_000).min(1000) as u16
}

/// Era index for the build grammar (matches `default_facade_for_era` tiers).
#[must_use]
pub const fn era_index_from_pop_tech(population: u64, tech_count: usize) -> u16 {
    if tech_count >= 12 || population >= 50_000 {
        5
    } else if population >= 10_000 || tech_count >= 10 {
        4
    } else if population >= 5_000 || tech_count >= 8 {
        3
    } else if population >= 2_000 || tech_count >= 5 {
        2
    } else if population >= 500 || tech_count >= 2 {
        1
    } else {
        0
    }
}

/// Minimum build-era for a parcel kind (`FR-CIV-ARCH` era-gated unlocks).
#[must_use]
pub const fn parcel_kind_min_era(kind: ParcelKind) -> u16 {
    match kind {
        ParcelKind::Residential => 0,
        ParcelKind::Commercial => 1,
        ParcelKind::Industrial => 2,
        ParcelKind::Civic => 3,
    }
}

/// Returns true when `era` unlocks the parcel kind.
#[must_use]
pub const fn parcel_kind_unlocked(kind: ParcelKind, era: u16) -> bool {
    era >= parcel_kind_min_era(kind)
}

/// Minimum build-era for engine [`BuildingType`] variants (few era-gated types).
///
/// Uses byte-slice comparison so this function can remain `const`.
#[must_use]
pub const fn building_type_min_era(type_tag: &str) -> u16 {
    let b = type_tag.as_bytes();
    // Match on byte slices — stable in const context unlike str matching.
    if const_bytes_eq(b, b"Mine") {
        1
    } else if const_bytes_eq(b, b"Market") {
        2
    } else if const_bytes_eq(b, b"Temple") {
        3
    } else if const_bytes_eq(b, b"Barracks") {
        4
    } else {
        // Farm, House, CityCenter, and all unknown types unlock at era 0.
        0
    }
}

/// Byte-slice equality suitable for use in `const fn`.
const fn const_bytes_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut i = 0;
    while i < a.len() {
        if a[i] != b[i] {
            return false;
        }
        i += 1;
    }
    true
}

/// Returns true when `era` unlocks the named building type.
#[must_use]
pub fn building_type_unlocked(type_tag: &str, era: u16) -> bool {
    era >= building_type_min_era(type_tag)
}

/// Applies biome material bias on top of a resolved facade.
#[must_use]
pub fn apply_biome_facade_bias(mut facade: FacadeStyle, biome: BiomeStyleTag) -> FacadeStyle {
    let material_offset = match biome {
        BiomeStyleTag::ARID => 1,
        BiomeStyleTag::FOREST => 2,
        BiomeStyleTag::COASTAL => 3,
        BiomeStyleTag::COLD => 4,
        _ => 0,
    };
    if material_offset > 0 {
        for mat in &mut facade.materials {
            mat.0 = mat.0.saturating_add(material_offset);
        }
        facade.name = format!("{}-biome{}", facade.name, biome.0);
    }
    facade
}

/// Resolves facade style from emergence key + demand signals.
#[must_use]
pub fn facade_for_emergence(
    key: EmergentStyleKey,
    demand: &DemandSignals,
    tile_sets: &[TileSetProfile],
) -> FacadeStyle {
    let vector = key.to_style_vector();
    let base = facade_for_vector(
        &vector,
        demand,
        tile_sets,
        ArchitectureMode::Canonical,
        None,
    );
    apply_biome_facade_bias(base, key.biome)
}

/// Deterministic cluster centroid from member fixed-point positions.
#[must_use]
pub fn settlement_cluster_centroid(positions: &[(i64, i64, i64)]) -> WorldCoord {
    if positions.is_empty() {
        return WorldCoord { x: 0, y: 0, z: 0 };
    }
    let count = positions.len() as i64;
    let (sx, sy, sz) = positions.iter().fold((0_i64, 0_i64, 0_i64), |acc, (x, y, z)| {
        (acc.0 + x, acc.1 + y, acc.2 + z)
    });
    WorldCoord {
        x: sx / count,
        y: sy / count,
        z: sz / count,
    }
}

/// Deterministic parcel offset within a settlement cluster (layout clustering).
#[must_use]
pub fn clustered_parcel_offset(cluster_id: u64, parcel_index: u32, spacing: u32) -> WorldCoord {
    let spacing = i64::from(spacing.max(4));
    let lane = parcel_index as i64;
    let ring = lane / 8 + 1;
    let slot = (lane % 8) as usize;
    const DELTAS: [(i64, i64); 8] = [
        (1, 0),
        (1, 1),
        (0, 1),
        (-1, 1),
        (-1, 0),
        (-1, -1),
        (0, -1),
        (1, -1),
    ];
    let (dx, dz) = DELTAS[slot];
    let hash = cluster_id
        .wrapping_mul(1_104_824_245)
        .wrapping_add(u64::from(parcel_index) * 1_664_527);
    let jitter = (hash % 3) as i64 - 1;
    WorldCoord {
        x: dx * ring * spacing + jitter,
        y: 0,
        z: dz * ring * spacing + jitter,
    }
}

/// Canonical architecture tile-set registry for emergence-driven selection.
#[must_use]
pub fn default_architecture_tile_sets() -> Vec<TileSetProfile> {
    let mut sets = Vec::new();
    let mut id = 1_u16;
    for culture in 0_u16..4 {
        for era in 0_u16..=5 {
            let facade = default_facade_for_era(era);
            let wealth_bucket = 4_u8;
            sets.push(TileSetProfile {
                id,
                culture,
                era,
                wealth_bucket,
                facade,
                adjacency_weights: BTreeMap::from([(id, 10), (id.saturating_add(1), 5)]),
            });
            id = id.saturating_add(1);
        }
    }
    sets
}

/// Filters demand signals to era-unlocked parcel channels only.
#[must_use]
pub fn era_gated_demand_signals(mut signals: DemandSignals, era: u16) -> DemandSignals {
    if !parcel_kind_unlocked(ParcelKind::Residential, era) {
        signals.residential = 0.0;
    }
    if !parcel_kind_unlocked(ParcelKind::Commercial, era) {
        signals.commercial = 0.0;
    }
    if !parcel_kind_unlocked(ParcelKind::Industrial, era) {
        signals.industrial = 0.0;
    }
    if !parcel_kind_unlocked(ParcelKind::Civic, era) {
        signals.civic = 0.0;
    }
    signals
}

/// L1 distance between two facade-name histograms (for FR-CIV-ARCH-008).
#[must_use]
pub fn facade_histogram_l1(
    left: &BTreeMap<String, u32>,
    right: &BTreeMap<String, u32>,
) -> u32 {
    let keys: BTreeMap<&String, ()> = left
        .keys()
        .chain(right.keys())
        .map(|k| (k, ()))
        .collect();
    keys.keys()
        .map(|name| {
            let a = left.get(*name).copied().unwrap_or(0);
            let b = right.get(*name).copied().unwrap_or(0);
            a.abs_diff(b)
        })
        .sum()
}

/// Settlement-cluster layout extensions on [`BuildingGraph`].
impl BuildingGraph {
    /// Records a parcel under a settlement cluster id.
    pub fn assign_to_cluster(&mut self, cluster_id: u64, building_id: BuildingId) {
        self.settlement_clusters
            .entry(cluster_id)
            .or_default()
            .push(building_id);
    }

    /// Returns parcel ids assigned to a cluster.
    #[must_use]
    pub fn parcels_in_cluster(&self, cluster_id: u64) -> &[BuildingId] {
        self.settlement_clusters
            .get(&cluster_id)
            .map_or(&[], Vec::as_slice)
    }

    /// Number of settlement clusters with at least one parcel.
    #[must_use]
    pub fn settlement_cluster_count(&self) -> usize {
        self.settlement_clusters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// FR-CIV-ARCH — culture trait hash is stable and distinct across profiles.
    #[test]
    fn culture_id_from_traits_is_stable_and_distinct() {
        let a = culture_id_from_traits([0.1, 0.2, 0.3, 0.4]);
        let b = culture_id_from_traits([0.9, 0.1, 0.2, 0.3]);
        assert_eq!(a, culture_id_from_traits([0.1, 0.2, 0.3, 0.4]));
        assert_ne!(a, b);
    }

    /// FR-CIV-ARCH — biome bias changes facade material ids.
    #[test]
    fn biome_bias_changes_facade_materials() {
        let base = default_facade_for_era(2);
        let biased = apply_biome_facade_bias(base.clone(), BiomeStyleTag::FOREST);
        assert_ne!(base.materials, biased.materials);
        assert!(biased.name.contains("biome"));
    }

    /// FR-CIV-ARCH — era gates suppress industrial/civic at low era.
    #[test]
    fn era_gated_demand_zeroes_locked_channels() {
        let signals = DemandSignals {
            residential: 0.9,
            commercial: 0.8,
            industrial: 0.7,
            civic: 0.6,
        };
        let gated = era_gated_demand_signals(signals, 0);
        assert!(gated.residential > 0.0);
        assert_eq!(gated.commercial, 0.0);
        assert_eq!(gated.industrial, 0.0);
        assert_eq!(gated.civic, 0.0);
    }

    /// FR-CIV-ARCH — clustered offsets are deterministic per cluster.
    #[test]
    fn clustered_parcel_offset_is_deterministic() {
        let a = clustered_parcel_offset(42, 3, 16);
        let b = clustered_parcel_offset(42, 3, 16);
        let c = clustered_parcel_offset(99, 3, 16);
        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    /// FR-CIV-ARCH-008 — culture divergence yields measurable histogram L1 distance.
    #[test]
    fn fr_arch_008_histogram_l1_tracks_culture_divergence() {
        let tile_sets = default_architecture_tile_sets();
        let demand = DemandSignals {
            residential: 0.8,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        };
        let key_a = EmergentStyleKey::new(0, 2, 500, BiomeStyleTag::NEUTRAL);
        let key_b = EmergentStyleKey::new(3, 2, 500, BiomeStyleTag::NEUTRAL);
        let mut hist_a = BTreeMap::new();
        let mut hist_b = BTreeMap::new();
        *hist_a
            .entry(facade_for_emergence(key_a, &demand, &tile_sets).name)
            .or_insert(0) += 1;
        *hist_b
            .entry(facade_for_emergence(key_b, &demand, &tile_sets).name)
            .or_insert(0) += 1;
        assert!(facade_histogram_l1(&hist_a, &hist_b) > 0);
    }

    /// FR-CIV-ARCH — building type era gates match spec table.
    #[test]
    fn building_type_era_gates() {
        assert!(building_type_unlocked("Farm", 0));
        assert!(!building_type_unlocked("Temple", 2));
        assert!(building_type_unlocked("Temple", 3));
        assert!(!building_type_unlocked("Barracks", 3));
        assert!(building_type_unlocked("Barracks", 4));
    }

    /// FR-CIV-ARCH — settlement cluster assignment on BuildingGraph.
    #[test]
    fn building_graph_cluster_assignment() {
        let mut graph = BuildingGraph::new();
        graph.assign_to_cluster(7, BuildingId(1));
        graph.assign_to_cluster(7, BuildingId(2));
        assert_eq!(graph.parcels_in_cluster(7).len(), 2);
        assert_eq!(graph.settlement_cluster_count(), 1);
    }
}
