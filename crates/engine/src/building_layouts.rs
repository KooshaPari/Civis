// TODO(cleanup-surgeon): stub module — `EmergentLayout`/`LayoutStrategy` were
// removed by an earlier lane. `lib.rs` still re-exports them. Restore the
// original or rewrite callers.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

/// Infrastructure requirements for a building.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct InfrastructureNeeds {
    pub food_per_tick: i64,
    pub energy_per_tick: i64,
    pub workers_required: u32,
    pub adjacent_requirements: Vec<String>,
}

/// A building layout with footprint and capacity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildingLayout {
    pub building_type: String,
    pub footprint: Vec<(i32, i32)>,
    pub capacity: u32,
    pub infrastructure_needs: InfrastructureNeeds,
    pub efficiency: f32,
}

impl Default for BuildingLayout {
    fn default() -> Self {
        Self {
            building_type: String::new(),
            footprint: Vec::new(),
            capacity: 0,
            infrastructure_needs: InfrastructureNeeds::default(),
            efficiency: 1.0,
        }
    }
}

/// Catalog of standard building layouts.
pub struct LayoutCatalog;

impl LayoutCatalog {
    /// Standard house layout.
    #[must_use]
    pub fn house() -> BuildingLayout {
        BuildingLayout {
            building_type: "house".to_string(),
            footprint: vec![(0, 0), (1, 0), (0, 1), (1, 1)],
            capacity: 4,
            infrastructure_needs: InfrastructureNeeds {
                food_per_tick: 1,
                energy_per_tick: 1,
                workers_required: 0,
                adjacent_requirements: vec!["road".to_string()],
            },
            efficiency: 1.0,
        }
    }

    /// Standard farm layout.
    #[must_use]
    pub fn farm() -> BuildingLayout {
        BuildingLayout {
            building_type: "farm".to_string(),
            footprint: vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)],
            capacity: 10,
            infrastructure_needs: InfrastructureNeeds {
                food_per_tick: 0,
                energy_per_tick: 2,
                workers_required: 3,
                adjacent_requirements: vec!["water".to_string()],
            },
            efficiency: 1.0,
        }
    }

    /// Standard workshop layout.
    #[must_use]
    pub fn workshop() -> BuildingLayout {
        BuildingLayout {
            building_type: "workshop".to_string(),
            footprint: vec![(0, 0), (1, 0), (0, 1)],
            capacity: 6,
            infrastructure_needs: InfrastructureNeeds {
                food_per_tick: 0,
                energy_per_tick: 5,
                workers_required: 4,
                adjacent_requirements: vec!["road".to_string()],
            },
            efficiency: 1.0,
        }
    }

    /// Standard temple layout.
    #[must_use]
    pub fn temple() -> BuildingLayout {
        BuildingLayout {
            building_type: "temple".to_string(),
            footprint: vec![
                (0, 0),
                (1, 0),
                (2, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (0, 2),
                (1, 2),
                (2, 2),
            ],
            capacity: 50,
            infrastructure_needs: InfrastructureNeeds {
                food_per_tick: 0,
                energy_per_tick: 3,
                workers_required: 2,
                adjacent_requirements: vec!["market".to_string()],
            },
            efficiency: 1.0,
        }
    }

    /// Standard market layout.
    #[must_use]
    pub fn market() -> BuildingLayout {
        BuildingLayout {
            building_type: "market".to_string(),
            footprint: vec![(0, 0), (1, 0), (2, 0), (3, 0)],
            capacity: 30,
            infrastructure_needs: InfrastructureNeeds {
                food_per_tick: 2,
                energy_per_tick: 2,
                workers_required: 5,
                adjacent_requirements: vec!["road".to_string()],
            },
            efficiency: 1.0,
        }
    }

    /// Standard barracks layout.
    #[must_use]
    pub fn barracks() -> BuildingLayout {
        BuildingLayout {
            building_type: "barracks".to_string(),
            footprint: vec![(0, 0), (1, 0), (2, 0), (0, 1), (1, 1), (2, 1)],
            capacity: 20,
            infrastructure_needs: InfrastructureNeeds {
                food_per_tick: 3,
                energy_per_tick: 4,
                workers_required: 0,
                adjacent_requirements: vec!["road".to_string()],
            },
            efficiency: 1.0,
        }
    }
}

/// Generate a deterministic building layout from type, size hint, and seed.
#[must_use]
pub fn generate_layout(building_type: &str, size_hint: u32, seed: u64) -> BuildingLayout {
    let mut hasher = DefaultHasher::new();
    building_type.hash(&mut hasher);
    size_hint.hash(&mut hasher);
    seed.hash(&mut hasher);
    let hash = hasher.finish();

    let mut footprint = Vec::new();
    let side = ((size_hint as f64).sqrt().ceil() as i32).max(1);
    for y in 0..side {
        for x in 0..side {
            let point_hash = hash.wrapping_add(y as u64 * 1000 + x as u64);
            if point_hash % 3 != 0 || footprint.len() < 2 {
                footprint.push((x, y));
            }
        }
    }

    let capacity = ((hash % 50) as u32).max(1);
    let efficiency = ((hash % 100) as f32 / 100.0).max(0.1);

    BuildingLayout {
        building_type: building_type.to_string(),
        footprint,
        capacity,
        infrastructure_needs: InfrastructureNeeds::default(),
        efficiency,
    }
}

/// Validate that a layout can be placed at anchor without overlapping occupied cells.
#[must_use]
pub fn validate_placement(
    layout: &BuildingLayout,
    anchor: (i32, i32),
    occupied: &[(i32, i32)],
) -> bool {
    let occupied_set: std::collections::HashSet<(i32, i32)> = occupied.iter().copied().collect();
    for &(dx, dy) in &layout.footprint {
        let cell = (anchor.0 + dx, anchor.1 + dy);
        if occupied_set.contains(&cell) {
            return false;
        }
    }
    true
}

/// Compute efficiency modifier based on nearby building types.
#[must_use]
pub fn compute_efficiency(layout: &BuildingLayout, nearby_types: &[String]) -> f32 {
    let base = layout.efficiency;
    let bonus = nearby_types
        .iter()
        .filter(|t| {
            (layout.building_type == "farm" && **t == "water")
                || (layout.building_type == "temple" && **t == "market")
                || (layout.building_type == "barracks" && **t == "workshop")
        })
        .count() as f32
        * 0.1;
    (base + bonus).min(2.0)
}

/// Advance all building layouts by one tick (efficiency decay).
#[must_use]
pub fn tick_building_layouts(layouts: &[BuildingLayout]) -> Vec<BuildingLayout> {
    layouts
        .iter()
        .map(|l| BuildingLayout {
            efficiency: (l.efficiency - 0.001).max(0.1),
            ..l.clone()
        })
        .collect()
}

#[cfg(test)]
mod building_layouts_tests {
    use super::*;

    #[test]
    fn layout_catalog_house() {
        let h = LayoutCatalog::house();
        assert_eq!(h.building_type, "house");
        assert_eq!(h.capacity, 4);
        assert!(!h.footprint.is_empty());
    }

    #[test]
    fn layout_catalog_farm() {
        let f = LayoutCatalog::farm();
        assert_eq!(f.building_type, "farm");
        assert!(f.capacity > 0);
    }

    #[test]
    fn layout_catalog_workshop() {
        let w = LayoutCatalog::workshop();
        assert_eq!(w.building_type, "workshop");
    }

    #[test]
    fn layout_catalog_temple() {
        let t = LayoutCatalog::temple();
        assert_eq!(t.building_type, "temple");
        assert!(t.capacity >= 50);
    }

    #[test]
    fn layout_catalog_market() {
        let m = LayoutCatalog::market();
        assert_eq!(m.building_type, "market");
    }

    #[test]
    fn layout_catalog_barracks() {
        let b = LayoutCatalog::barracks();
        assert_eq!(b.building_type, "barracks");
    }

    #[test]
    fn generate_layout_deterministic() {
        let a = generate_layout("house", 10, 42);
        let b = generate_layout("house", 10, 42);
        assert_eq!(a.building_type, b.building_type);
        assert_eq!(a.footprint, b.footprint);
        assert_eq!(a.capacity, b.capacity);
    }

    #[test]
    fn generate_layout_different_seeds_diverge() {
        let a = generate_layout("house", 10, 42);
        let b = generate_layout("house", 10, 99);
        assert_ne!(a.capacity, b.capacity);
    }

    #[test]
    fn validate_placement_no_overlap() {
        let layout = LayoutCatalog::house();
        let occupied = vec![(10, 10)];
        assert!(validate_placement(&layout, (5, 5), &occupied));
    }

    #[test]
    fn validate_placement_overlap() {
        let layout = LayoutCatalog::house();
        let occupied = vec![(0, 0)];
        assert!(!validate_placement(&layout, (0, 0), &occupied));
    }

    #[test]
    fn compute_efficiency_bonus() {
        let layout = LayoutCatalog::farm();
        let eff = compute_efficiency(&layout, &["water".to_string()]);
        assert!(eff > layout.efficiency);
    }

    #[test]
    fn compute_efficiency_no_bonus() {
        let layout = LayoutCatalog::farm();
        let eff = compute_efficiency(&layout, &["house".to_string()]);
        assert!((eff - layout.efficiency).abs() < f32::EPSILON);
    }

    #[test]
    fn tick_building_layouts_decays_efficiency() {
        let layouts = vec![LayoutCatalog::house()];
        let ticked = tick_building_layouts(&layouts);
        assert!(ticked[0].efficiency < layouts[0].efficiency);
    }
}
