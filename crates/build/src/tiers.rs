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
#[must_use]
pub fn building_type_min_era(type_tag: &str) -> u16 {
    match type_tag {
        "Farm" | "House" | "CityCenter" => 0,
        "Mine" => 1,
        "Market" => 2,
        "Temple" => 3,
        "Barracks" => 4,
        _ => 0,
    }
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

    // -----------------------------------------------------------------------
    // FR-CIV-ARCH sub-feature A: style varies by culture + biome + era
    // -----------------------------------------------------------------------

    /// FR-CIV-ARCH-A-001 — different cultures produce different facade styles.
    #[test]
    fn fr_arch_a001_style_varies_by_culture() {
        let tile_sets = default_architecture_tile_sets();
        let demand = DemandSignals {
            residential: 0.8,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        };
        let key_a = EmergentStyleKey::new(0, 2, 500, BiomeStyleTag::NEUTRAL);
        let key_b = EmergentStyleKey::new(2, 2, 500, BiomeStyleTag::NEUTRAL);
        let style_a = facade_for_emergence(key_a, &demand, &tile_sets);
        let style_b = facade_for_emergence(key_b, &demand, &tile_sets);
        // Different cultures must resolve to different style names.
        assert_ne!(
            style_a.name, style_b.name,
            "culture 0 and culture 2 should differ in style"
        );
    }

    /// FR-CIV-ARCH-A-002 — different biomes produce different facade materials.
    #[test]
    fn fr_arch_a002_style_varies_by_biome() {
        let tile_sets = default_architecture_tile_sets();
        let demand = DemandSignals {
            residential: 0.8,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        };
        // Same culture + era, different biome.
        let key_arid = EmergentStyleKey::new(1, 2, 500, BiomeStyleTag::ARID);
        let key_forest = EmergentStyleKey::new(1, 2, 500, BiomeStyleTag::FOREST);
        let style_arid = facade_for_emergence(key_arid, &demand, &tile_sets);
        let style_forest = facade_for_emergence(key_forest, &demand, &tile_sets);
        // Biome bias shifts material ids, so materials must differ.
        assert_ne!(
            style_arid.materials, style_forest.materials,
            "arid and forest biomes should produce different material palettes"
        );
    }

    /// FR-CIV-ARCH-A-003 — higher era yields a later facade style name.
    #[test]
    fn fr_arch_a003_style_varies_by_era() {
        let tile_sets = default_architecture_tile_sets();
        let demand = DemandSignals {
            residential: 0.8,
            commercial: 0.0,
            industrial: 0.0,
            civic: 0.0,
        };
        let key_early = EmergentStyleKey::new(0, 0, 500, BiomeStyleTag::NEUTRAL);
        let key_late = EmergentStyleKey::new(0, 5, 500, BiomeStyleTag::NEUTRAL);
        let style_early = facade_for_emergence(key_early, &demand, &tile_sets);
        let style_late = facade_for_emergence(key_late, &demand, &tile_sets);
        assert_ne!(
            style_early.name, style_late.name,
            "era 0 and era 5 should resolve to different style names"
        );
    }

    // -----------------------------------------------------------------------
    // FR-CIV-ARCH sub-feature B: settlement layout clustering
    // -----------------------------------------------------------------------

    /// FR-CIV-ARCH-B-001 — buildings assigned to a cluster appear in parcels_in_cluster.
    #[test]
    fn fr_arch_b001_layout_cluster_membership() {
        let mut graph = BuildingGraph::new();
        let ids: Vec<BuildingId> = (1..=5).map(BuildingId).collect();
        for &id in &ids {
            graph.assign_to_cluster(42, id);
        }
        let members = graph.parcels_in_cluster(42);
        assert_eq!(members.len(), 5, "all five buildings should be in cluster 42");
        for id in &ids {
            assert!(members.contains(id), "building {id:?} must be in cluster");
        }
    }

    /// FR-CIV-ARCH-B-002 — clustered_parcel_offset spreads buildings around centre.
    #[test]
    fn fr_arch_b002_cluster_offsets_diverge_from_centre() {
        // Eight consecutive slots must not all map to the same offset.
        let offsets: Vec<_> = (0..8)
            .map(|i| clustered_parcel_offset(1, i, 16))
            .collect();
        // Collect unique (x, z) pairs.
        let unique: std::collections::BTreeSet<(i64, i64)> =
            offsets.iter().map(|o| (o.x, o.z)).collect();
        assert!(
            unique.len() > 1,
            "different parcel slots must produce distinct offsets"
        );
    }

    /// FR-CIV-ARCH-B-003 — settlement centroid is within bounding box of members.
    #[test]
    fn fr_arch_b003_cluster_centroid_within_bounds() {
        let positions = vec![(0, 0, 0), (10, 0, 20), (20, 0, 40)];
        let centroid = settlement_cluster_centroid(&positions);
        assert!(centroid.x >= 0 && centroid.x <= 20);
        assert!(centroid.z >= 0 && centroid.z <= 40);
    }

    /// FR-CIV-ARCH-B-004 — multiple clusters are tracked independently.
    #[test]
    fn fr_arch_b004_distinct_clusters_stay_separate() {
        let mut graph = BuildingGraph::new();
        graph.assign_to_cluster(1, BuildingId(10));
        graph.assign_to_cluster(2, BuildingId(20));
        graph.assign_to_cluster(2, BuildingId(21));
        assert_eq!(graph.parcels_in_cluster(1).len(), 1);
        assert_eq!(graph.parcels_in_cluster(2).len(), 2);
        assert_eq!(graph.settlement_cluster_count(), 2);
    }

    // -----------------------------------------------------------------------
    // FR-CIV-ARCH sub-feature C: era-gated building unlocks
    // -----------------------------------------------------------------------

    /// FR-CIV-ARCH-C-001 — era gating suppresses demand signals for locked channels.
    #[test]
    fn fr_arch_c001_era_gate_suppresses_locked_demand_channels() {
        let full = DemandSignals {
            residential: 0.9,
            commercial: 0.9,
            industrial: 0.9,
            civic: 0.9,
        };
        // At era 0: only residential is unlocked.
        let era0 = era_gated_demand_signals(full, 0);
        assert!(era0.residential > 0.0, "residential unlocked at era 0");
        assert_eq!(era0.commercial, 0.0, "commercial locked at era 0");
        assert_eq!(era0.industrial, 0.0, "industrial locked at era 0");
        assert_eq!(era0.civic, 0.0, "civic locked at era 0");

        // At era 2: residential + commercial + industrial unlocked, civic still locked.
        let era2 = era_gated_demand_signals(full, 2);
        assert!(era2.residential > 0.0);
        assert!(era2.commercial > 0.0);
        assert!(era2.industrial > 0.0);
        assert_eq!(era2.civic, 0.0, "civic locked until era 3");

        // At era 3: all channels unlocked.
        let era3 = era_gated_demand_signals(full, 3);
        assert!(era3.civic > 0.0, "civic unlocked at era 3");
    }

    /// FR-CIV-ARCH-C-002 — building_type_unlocked returns false below min_era.
    #[test]
    fn fr_arch_c002_building_type_below_min_era_is_locked() {
        assert!(!building_type_unlocked("Mine", 0), "Mine needs era >= 1");
        assert!(!building_type_unlocked("Market", 1), "Market needs era >= 2");
        assert!(!building_type_unlocked("Temple", 2), "Temple needs era >= 3");
        assert!(!building_type_unlocked("Barracks", 3), "Barracks needs era >= 4");
    }

    /// FR-CIV-ARCH-C-003 — building_type_unlocked returns true at exactly min_era.
    #[test]
    fn fr_arch_c003_building_type_at_min_era_is_unlocked() {
        assert!(building_type_unlocked("Farm", 0));
        assert!(building_type_unlocked("Mine", 1));
        assert!(building_type_unlocked("Market", 2));
        assert!(building_type_unlocked("Temple", 3));
        assert!(building_type_unlocked("Barracks", 4));
    }

    /// FR-CIV-ARCH-C-004 — parcel_kind_unlocked mirrors spec min-era table.
    #[test]
    fn fr_arch_c004_parcel_kind_min_era_spec_table() {
        use crate::ParcelKind;
        assert_eq!(parcel_kind_min_era(ParcelKind::Residential), 0);
        assert_eq!(parcel_kind_min_era(ParcelKind::Commercial), 1);
        assert_eq!(parcel_kind_min_era(ParcelKind::Industrial), 2);
        assert_eq!(parcel_kind_min_era(ParcelKind::Civic), 3);
        assert!(!parcel_kind_unlocked(ParcelKind::Civic, 2));
        assert!(parcel_kind_unlocked(ParcelKind::Civic, 3));
    }

    // -----------------------------------------------------------------------
    // FR-CIV-ARCH determinism: same inputs → same outputs
    // -----------------------------------------------------------------------

    /// FR-CIV-ARCH-D-001 — facade_for_emergence is deterministic.
    #[test]
    fn fr_arch_d001_facade_for_emergence_is_deterministic() {
        let tile_sets = default_architecture_tile_sets();
        let demand = DemandSignals {
            residential: 0.8,
            commercial: 0.6,
            industrial: 0.4,
            civic: 0.2,
        };
        let key = EmergentStyleKey::new(1, 3, 800, BiomeStyleTag::COLD);
        let first = facade_for_emergence(key, &demand, &tile_sets);
        let second = facade_for_emergence(key, &demand, &tile_sets);
        assert_eq!(first, second, "facade_for_emergence must be deterministic");
    }

    /// FR-CIV-ARCH-D-002 — clustered_parcel_offset is deterministic per cluster+index.
    #[test]
    fn fr_arch_d002_clustered_offset_is_deterministic() {
        for cluster in [0_u64, 1, 42, u64::MAX / 2] {
            for idx in 0_u32..16 {
                let a = clustered_parcel_offset(cluster, idx, 8);
                let b = clustered_parcel_offset(cluster, idx, 8);
                assert_eq!(a, b, "offset must be identical for cluster={cluster} idx={idx}");
            }
        }
    }

    /// FR-CIV-ARCH-D-003 — era_index_from_pop_tech is deterministic.
    #[test]
    fn fr_arch_d003_era_index_is_deterministic() {
        for (pop, tech) in [(0_u64, 0_usize), (500, 2), (10_000, 8), (50_000, 12)] {
            let a = era_index_from_pop_tech(pop, tech);
            let b = era_index_from_pop_tech(pop, tech);
            assert_eq!(a, b);
        }
    }

    /// FR-CIV-ARCH-D-004 — culture_id_from_traits is deterministic.
    #[test]
    fn fr_arch_d004_culture_id_is_deterministic() {
        let traits = [0.25_f32, 0.5, 0.75, 1.0];
        assert_eq!(
            culture_id_from_traits(traits),
            culture_id_from_traits(traits)
        );
    }
}
