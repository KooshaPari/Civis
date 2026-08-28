// TODO(cleanup-surgeon): stub module — `EmergentLayout`/`LayoutStrategy` were
// removed by an earlier lane. `lib.rs` still re-exports them. Restore the
// original or rewrite callers.

use serde::{Deserialize, Serialize};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

// ---------------------------------------------------------------------------
// Existing types and functions (preserved exactly)
// ---------------------------------------------------------------------------

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

// ---------------------------------------------------------------------------
// Feature 1 — Architectural Style Registry
// ---------------------------------------------------------------------------

/// Architectural style associated with a civilisation era.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArchitecturalStyle {
    Ancient,
    Medieval,
    Industrial,
    Modern,
    Futuristic,
}

/// Properties that describe a given architectural style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleProperties {
    pub style: ArchitecturalStyle,
    pub max_height: u32,
    pub material_preference: Vec<String>,
    pub ornamentation_level: f32,
    pub durability: f32,
}

/// Look up the [`StyleProperties`] for a specific [`ArchitecturalStyle`].
#[must_use]
pub fn style_properties(style: ArchitecturalStyle) -> StyleProperties {
    match style {
        ArchitecturalStyle::Ancient => StyleProperties {
            style: ArchitecturalStyle::Ancient,
            max_height: 3,
            material_preference: vec!["stone".to_string(), "wood".to_string(), "clay".to_string()],
            ornamentation_level: 0.2,
            durability: 0.6,
        },
        ArchitecturalStyle::Medieval => StyleProperties {
            style: ArchitecturalStyle::Medieval,
            max_height: 5,
            material_preference: vec![
                "stone".to_string(),
                "timber".to_string(),
                "thatch".to_string(),
            ],
            ornamentation_level: 0.4,
            durability: 0.7,
        },
        ArchitecturalStyle::Industrial => StyleProperties {
            style: ArchitecturalStyle::Industrial,
            max_height: 10,
            material_preference: vec!["brick".to_string(), "iron".to_string(), "glass".to_string()],
            ornamentation_level: 0.3,
            durability: 0.8,
        },
        ArchitecturalStyle::Modern => StyleProperties {
            style: ArchitecturalStyle::Modern,
            max_height: 50,
            material_preference: vec![
                "concrete".to_string(),
                "steel".to_string(),
                "glass".to_string(),
            ],
            ornamentation_level: 0.5,
            durability: 0.9,
        },
        ArchitecturalStyle::Futuristic => StyleProperties {
            style: ArchitecturalStyle::Futuristic,
            max_height: 200,
            material_preference: vec![
                "composite".to_string(),
                "carbon".to_string(),
                "nano-alloy".to_string(),
            ],
            ornamentation_level: 0.8,
            durability: 0.95,
        },
    }
}

/// Map a human-readable era name to an [`ArchitecturalStyle`].
///
/// Matching is case-insensitive and best-effort — unknown eras default to
/// [`ArchitecturalStyle::Ancient`].
#[must_use]
pub fn style_for_era(era_name: &str) -> ArchitecturalStyle {
    let lower = era_name.to_lowercase();
    if lower.contains("future") || lower.contains("sci-fi") || lower.contains("space") {
        ArchitecturalStyle::Futuristic
    } else if lower.contains("modern") || lower.contains("contemporary") || lower.contains("now") {
        ArchitecturalStyle::Modern
    } else if lower.contains("industr") || lower.contains("victorian") || lower.contains("steam") {
        ArchitecturalStyle::Industrial
    } else if lower.contains("medieval") || lower.contains("feudal") || lower.contains("middle") {
        ArchitecturalStyle::Medieval
    } else {
        ArchitecturalStyle::Ancient
    }
}

// ---------------------------------------------------------------------------
// Feature 2 — Floor Plan Generator
// ---------------------------------------------------------------------------

/// Classification of rooms within a building.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RoomType {
    Chamber,
    Corridor,
    Courtyard,
    Workshop,
    Storage,
    Chapel,
}

/// A single room in a floor plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub room_type: RoomType,
    pub width: u32,
    pub height: u32,
    pub position: (i32, i32),
}

/// A complete floor plan consisting of rooms and summary metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FloorPlan {
    pub rooms: Vec<Room>,
    pub total_area: u32,
    pub corridor_length: u32,
}

/// Seeded pseudo-random number generator helper — lightweight linear-congruential
/// generator used solely inside the floor-plan generator to stay fully deterministic
/// without pulling in additional dependencies.
struct Lcg {
    state: u64,
}

impl Lcg {
    fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1);
        self.state
    }

    fn range_u32(&mut self, lo: u32, hi: u32) -> u32 {
        if lo >= hi {
            return lo;
        }
        lo + (self.next_u64() % (hi - lo + 1) as u64) as u32
    }
}

/// Map a [`RoomType`] to a default base area used by the floor-plan generator.
fn room_base_area(rt: RoomType) -> u32 {
    match rt {
        RoomType::Chamber => 16,
        RoomType::Corridor => 6,
        RoomType::Courtyard => 36,
        RoomType::Workshop => 20,
        RoomType::Storage => 12,
        RoomType::Chapel => 25,
    }
}

/// Generate a deterministic floor plan from a seed, target room count, and
/// architectural style.
#[must_use]
pub fn generate_floor_plan(seed: u64, room_count: u32, style: ArchitecturalStyle) -> FloorPlan {
    let mut rng = Lcg { state: seed };
    let style_props = style_properties(style);

    let room_types = [
        RoomType::Chamber,
        RoomType::Corridor,
        RoomType::Courtyard,
        RoomType::Workshop,
        RoomType::Storage,
        RoomType::Chapel,
    ];

    let mut rooms = Vec::new();
    let mut cursor_x: i32 = 0;
    let mut corridor_total: u32 = 0;

    for i in 0..room_count.max(1) {
        let idx = (rng.next_u64() as usize) % room_types.len();
        let rt = room_types[idx];

        // Scale base area by style's max height as a proxy for ambition.
        let base_area = room_base_area(rt);
        let scale_factor = 1.0 + (style_props.max_height as f32 / 100.0);
        let area = (base_area as f32 * scale_factor) as u32;

        let width = (area as f32).sqrt().ceil() as u32;
        let height = if width > 0 { area / width } else { 1 };
        let height = height.max(1);

        let position = (cursor_x, (i * 2) as i32);
        cursor_x += width as i32 + 1;

        if rt == RoomType::Corridor {
            corridor_total += width;
        }

        rooms.push(Room {
            room_type: rt,
            width,
            height,
            position,
        });
    }

    let total_area: u32 = rooms.iter().map(|r| r.width * r.height).sum();

    FloorPlan {
        rooms,
        total_area,
        corridor_length: corridor_total,
    }
}

/// Compute the efficiency of a floor plan as a ratio of usable area to total
/// area. Corridors count as half-usable; courtyards are fully usable.
#[must_use]
pub fn floor_plan_efficiency(plan: &FloorPlan) -> f32 {
    if plan.total_area == 0 {
        return 0.0;
    }

    let usable: f32 = plan
        .rooms
        .iter()
        .map(|r| {
            let area = (r.width * r.height) as f32;
            match r.room_type {
                RoomType::Corridor => area * 0.5,
                _ => area,
            }
        })
        .sum();

    (usable / plan.total_area as f32).min(1.0)
}

// ---------------------------------------------------------------------------
// Feature 3 — Material Palette per Style
// ---------------------------------------------------------------------------

/// Materials assigned to a building based on its architectural style.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialPalette {
    pub style: ArchitecturalStyle,
    pub primary_material: String,
    pub secondary_material: String,
    pub roof_material: String,
    pub cost_factor: f32,
}

/// Return the canonical [`MaterialPalette`] for a given [`ArchitecturalStyle`].
#[must_use]
pub fn palette_for_style(style: ArchitecturalStyle) -> MaterialPalette {
    match style {
        ArchitecturalStyle::Ancient => MaterialPalette {
            style,
            primary_material: "stone".to_string(),
            secondary_material: "wood".to_string(),
            roof_material: "thatch".to_string(),
            cost_factor: 0.5,
        },
        ArchitecturalStyle::Medieval => MaterialPalette {
            style,
            primary_material: "stone".to_string(),
            secondary_material: "timber".to_string(),
            roof_material: "slate".to_string(),
            cost_factor: 0.7,
        },
        ArchitecturalStyle::Industrial => MaterialPalette {
            style,
            primary_material: "brick".to_string(),
            secondary_material: "iron".to_string(),
            roof_material: "corrugated-iron".to_string(),
            cost_factor: 1.0,
        },
        ArchitecturalStyle::Modern => MaterialPalette {
            style,
            primary_material: "concrete".to_string(),
            secondary_material: "steel".to_string(),
            roof_material: "membrane".to_string(),
            cost_factor: 1.5,
        },
        ArchitecturalStyle::Futuristic => MaterialPalette {
            style,
            primary_material: "composite".to_string(),
            secondary_material: "carbon-fiber".to_string(),
            roof_material: "nano-coating".to_string(),
            cost_factor: 3.0,
        },
    }
}

/// Compute a material compatibility score between two material names.
///
/// Returns a value in `[0.0, 1.0]` where `1.0` means identical or perfectly
/// compatible and `0.0` means completely unrelated. The comparison is
/// case-insensitive and based on known material family groupings.
#[must_use]
pub fn material_compatibility(mat_a: &str, mat_b: &str) -> f32 {
    let a = mat_a.to_lowercase();
    let b = mat_b.to_lowercase();

    if a == b {
        return 1.0;
    }

    // Define material families: indices in the outer vec represent the family.
    let families: Vec<Vec<&str>> = vec![
        vec!["stone", "slate", "granite", "limestone", "marble", "basalt"],
        vec!["wood", "timber", "lumber"],
        vec!["brick", "clay", "terracotta"],
        vec!["iron", "steel", "carbon-steel"],
        vec!["glass", "crystal", "plexiglass"],
        vec!["concrete", "cement", "mortar"],
        vec!["thatch", "straw", "reed"],
        vec![
            "composite",
            "carbon-fiber",
            "nano-alloy",
            "nano-coating",
            "membrane",
        ],
        vec!["corrugated-iron", "tin", "zinc", "aluminum"],
    ];

    for family in &families {
        let in_a = family.iter().any(|m| *m == a.as_str());
        let in_b = family.iter().any(|m| *m == b.as_str());
        if in_a && in_b {
            return 0.8;
        }
    }

    // Partial score for mixed families that are historically compatible.
    let cross_compat: [([(&str); 4], f32); 4] = [
        (["stone", "wood", "timber", "wood"], 0.5),
        (["brick", "iron", "steel", "iron"], 0.4),
        (["concrete", "steel", "iron", "steel"], 0.6),
        (["stone", "slate", "thatch", "straw"], 0.3),
    ];

    for (group, score) in &cross_compat {
        let in_a = group.iter().any(|m| *m == a.as_str());
        let in_b = group.iter().any(|m| *m == b.as_str());
        if in_a && in_b {
            return *score;
        }
    }

    0.1
}

// ---------------------------------------------------------------------------
// Feature 4 — Structural Integrity Check
// ---------------------------------------------------------------------------

/// A single load-bearing or decorative structural element within a building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralElement {
    pub position: (i32, i32),
    pub load_capacity: f32,
    pub current_load: f32,
    pub element_type: String,
}

/// Compute the safety factor for every element. A safety factor of `1.0`
/// means the element is at exactly its rated capacity; values above `1.0`
/// indicate headroom, values below `1.0` indicate overloading.
///
/// Returns a `Vec` of `(index, safety_factor)` for every element in the
/// input slice.
#[must_use]
pub fn check_integrity(elements: &[StructuralElement]) -> Vec<(usize, f32)> {
    elements
        .iter()
        .enumerate()
        .map(|(i, el)| {
            let safety = if el.load_capacity <= 0.0 {
                0.0
            } else {
                (el.load_capacity - el.current_load) / el.load_capacity
            };
            (i, safety)
        })
        .collect()
}

/// Average safety factor across all structural elements.
///
/// Returns `0.0` for an empty slice.
#[must_use]
pub fn overall_integrity(elements: &[StructuralElement]) -> f32 {
    if elements.is_empty() {
        return 0.0;
    }
    let results = check_integrity(elements);
    let sum: f32 = results.iter().map(|(_, sf)| *sf).sum();
    sum / results.len() as f32
}

// ---------------------------------------------------------------------------
// Feature 5 — Expansion Mechanics
// ---------------------------------------------------------------------------

/// Direction in which a building can be expanded.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExpansionDirection {
    North,
    South,
    East,
    West,
    Up,
}

/// A plan describing how to expand an existing building.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpansionPlan {
    pub direction: ExpansionDirection,
    pub rooms_to_add: u32,
    pub style: ArchitecturalStyle,
    pub cost_multiplier: f32,
}

/// Compute the base cost of executing an expansion plan.
///
/// The cost is derived from the number of rooms, the style's cost factor,
/// the direction multiplier, and the plan's cost multiplier.
#[must_use]
pub fn expansion_cost(plan: &ExpansionPlan) -> f32 {
    let style_factor = palette_for_style(plan.style).cost_factor;
    let dir_multiplier = match plan.direction {
        ExpansionDirection::Up => 2.5,
        ExpansionDirection::North | ExpansionDirection::South => 1.0,
        ExpansionDirection::East | ExpansionDirection::West => 1.2,
    };
    let base_per_room = 100.0;
    base_per_room * plan.rooms_to_add as f32 * style_factor * dir_multiplier * plan.cost_multiplier
}

/// Produce a new [`BuildingLayout`] that incorporates rooms from an expansion
/// plan. The expansion is deterministic — the same inputs always produce the
/// same output.
#[must_use]
pub fn plan_expansion(layout: &BuildingLayout, plan: &ExpansionPlan, seed: u64) -> BuildingLayout {
    let mut rng = Lcg { state: seed };
    let style_props = style_properties(plan.style);

    // Determine offset for new footprint cells.
    let (off_x, off_y) = match plan.direction {
        ExpansionDirection::North => (0, -1),
        ExpansionDirection::South => {
            let max_y = layout.footprint.iter().map(|(_, y)| *y).max().unwrap_or(0);
            (0, max_y + 1)
        }
        ExpansionDirection::East => {
            let max_x = layout.footprint.iter().map(|(x, _)| *x).max().unwrap_or(0);
            (max_x + 1, 0)
        }
        ExpansionDirection::West => (-1, 0),
        ExpansionDirection::Up => (0, 0), // vertical — no horizontal shift
    };

    let mut new_footprint = layout.footprint.clone();
    let mut new_capacity = layout.capacity;

    for i in 0..plan.rooms_to_add {
        let room_side = (style_props.max_height as f32).sqrt().ceil() as i32 + 1;
        let base_x = off_x + (i as i32 * (room_side + 1));
        let base_y = off_y;

        for dy in 0..room_side {
            for dx in 0..room_side {
                let point_hash = rng.next_u64();
                if point_hash % 3 != 0 || new_footprint.len() < 4 {
                    let cell = (base_x + dx, base_y + dy);
                    // Avoid duplicates.
                    if !new_footprint.contains(&cell) {
                        new_footprint.push(cell);
                    }
                }
            }
        }

        new_capacity = new_capacity.saturating_add(((rng.next_u64() % 10) as u32).max(1));
    }

    // Efficiency remains unchanged from the original layout.

    BuildingLayout {
        building_type: layout.building_type.clone(),
        footprint: new_footprint,
        capacity: new_capacity,
        infrastructure_needs: layout.infrastructure_needs.clone(),
        efficiency: layout.efficiency,
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod building_layouts_tests {
    use super::*;

    // --- Existing tests (preserved) ---

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

    // --- New tests: Architectural Styles ---

    #[test]
    fn style_properties_all_five_styles() {
        let styles = [
            ArchitecturalStyle::Ancient,
            ArchitecturalStyle::Medieval,
            ArchitecturalStyle::Industrial,
            ArchitecturalStyle::Modern,
            ArchitecturalStyle::Futuristic,
        ];
        for s in styles {
            let props = style_properties(s);
            assert_eq!(props.style, s);
            assert!(props.max_height > 0);
            assert!(!props.material_preference.is_empty());
            assert!(props.ornamentation_level >= 0.0 && props.ornamentation_level <= 1.0);
            assert!(props.durability > 0.0 && props.durability <= 1.0);
        }
    }

    #[test]
    fn style_properties_height_ordering() {
        let ancient = style_properties(ArchitecturalStyle::Ancient);
        let future = style_properties(ArchitecturalStyle::Futuristic);
        assert!(future.max_height > ancient.max_height);
    }

    #[test]
    fn style_for_era_mapping() {
        assert_eq!(style_for_era("feudal age"), ArchitecturalStyle::Medieval);
        assert_eq!(style_for_era("steam era"), ArchitecturalStyle::Industrial);
        assert_eq!(style_for_era("modern world"), ArchitecturalStyle::Modern);
        assert_eq!(
            style_for_era("space exploration"),
            ArchitecturalStyle::Futuristic
        );
        assert_eq!(style_for_era("bronze age"), ArchitecturalStyle::Ancient);
    }

    #[test]
    fn style_for_era_case_insensitive() {
        assert_eq!(style_for_era("MEDIEVAL"), ArchitecturalStyle::Medieval);
        assert_eq!(style_for_era("Modern"), ArchitecturalStyle::Modern);
    }

    // --- New tests: Floor Plan ---

    #[test]
    fn floor_plan_deterministic() {
        let a = generate_floor_plan(123, 5, ArchitecturalStyle::Medieval);
        let b = generate_floor_plan(123, 5, ArchitecturalStyle::Medieval);
        assert_eq!(a.rooms.len(), b.rooms.len());
        assert_eq!(a.total_area, b.total_area);
        assert_eq!(a.corridor_length, b.corridor_length);
    }

    #[test]
    fn floor_plan_room_count_minimum_one() {
        let plan = generate_floor_plan(1, 0, ArchitecturalStyle::Ancient);
        assert!(plan.rooms.len() >= 1);
    }

    #[test]
    fn floor_plan_efficiency_range() {
        let plan = generate_floor_plan(42, 10, ArchitecturalStyle::Modern);
        let eff = floor_plan_efficiency(&plan);
        assert!((0.0..=1.0).contains(&eff));
    }

    #[test]
    fn floor_plan_efficiency_no_rooms_is_zero() {
        let plan = FloorPlan {
            rooms: vec![],
            total_area: 0,
            corridor_length: 0,
        };
        assert!((floor_plan_efficiency(&plan) - 0.0).abs() < f32::EPSILON);
    }

    // --- New tests: Material Palette ---

    #[test]
    fn palette_for_style_all_styles() {
        let styles = [
            ArchitecturalStyle::Ancient,
            ArchitecturalStyle::Medieval,
            ArchitecturalStyle::Industrial,
            ArchitecturalStyle::Modern,
            ArchitecturalStyle::Futuristic,
        ];
        for s in styles {
            let pal = palette_for_style(s);
            assert_eq!(pal.style, s);
            assert!(!pal.primary_material.is_empty());
            assert!(!pal.secondary_material.is_empty());
            assert!(!pal.roof_material.is_empty());
            assert!(pal.cost_factor > 0.0);
        }
    }

    #[test]
    fn material_compatibility_identical() {
        assert!((material_compatibility("stone", "stone") - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn material_compatibility_same_family() {
        let score = material_compatibility("stone", "granite");
        assert!(score >= 0.7); // same family = 0.8
    }

    #[test]
    fn material_compatibility_different_families() {
        let score = material_compatibility("thatch", "carbon-fiber");
        assert!(score < 0.5);
    }

    #[test]
    fn material_compatibility_case_insensitive() {
        let score = material_compatibility("STONE", "Granite");
        assert!(score >= 0.7);
    }

    // --- New tests: Structural Integrity ---

    #[test]
    fn integrity_safe_elements() {
        let elements = vec![
            StructuralElement {
                position: (0, 0),
                load_capacity: 100.0,
                current_load: 50.0,
                element_type: "wall".to_string(),
            },
            StructuralElement {
                position: (1, 0),
                load_capacity: 200.0,
                current_load: 80.0,
                element_type: "pillar".to_string(),
            },
        ];
        let results = check_integrity(&elements);
        assert_eq!(results.len(), 2);
        assert!(results[0].1 > 0.0); // safety factor > 0
        assert!(results[1].1 > 0.0);

        let overall = overall_integrity(&elements);
        assert!(overall > 0.0);
    }

    #[test]
    fn integrity_overloaded_element() {
        let elements = vec![StructuralElement {
            position: (0, 0),
            load_capacity: 50.0,
            current_load: 100.0,
            element_type: "wall".to_string(),
        }];
        let results = check_integrity(&elements);
        // Safety factor should be negative (overloaded).
        assert!(results[0].1 < 0.0);
    }

    #[test]
    fn integrity_zero_capacity_element() {
        let elements = vec![StructuralElement {
            position: (0, 0),
            load_capacity: 0.0,
            current_load: 0.0,
            element_type: "decorative".to_string(),
        }];
        let results = check_integrity(&elements);
        assert!((results[0].1 - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn integrity_empty_elements() {
        let elements: Vec<StructuralElement> = vec![];
        assert!((overall_integrity(&elements) - 0.0).abs() < f32::EPSILON);
    }

    // --- New tests: Expansion Mechanics ---

    #[test]
    fn expansion_cost_positive() {
        let plan = ExpansionPlan {
            direction: ExpansionDirection::East,
            rooms_to_add: 3,
            style: ArchitecturalStyle::Medieval,
            cost_multiplier: 1.0,
        };
        assert!(expansion_cost(&plan) > 0.0);
    }

    #[test]
    fn expansion_cost_up_direction_is_most_expensive() {
        let base = ExpansionPlan {
            direction: ExpansionDirection::East,
            rooms_to_add: 1,
            style: ArchitecturalStyle::Modern,
            cost_multiplier: 1.0,
        };
        let up = ExpansionPlan {
            direction: ExpansionDirection::Up,
            rooms_to_add: 1,
            style: ArchitecturalStyle::Modern,
            cost_multiplier: 1.0,
        };
        assert!(expansion_cost(&up) > expansion_cost(&base));
    }

    #[test]
    fn plan_expansion_grows_footprint() {
        let layout = LayoutCatalog::house();
        let plan = ExpansionPlan {
            direction: ExpansionDirection::South,
            rooms_to_add: 2,
            style: ArchitecturalStyle::Medieval,
            cost_multiplier: 1.0,
        };
        let expanded = plan_expansion(&layout, &plan, 42);
        assert!(expanded.footprint.len() > layout.footprint.len());
        assert!(expanded.capacity >= layout.capacity);
    }

    #[test]
    fn plan_expansion_preserves_type() {
        let layout = LayoutCatalog::barracks();
        let plan = ExpansionPlan {
            direction: ExpansionDirection::East,
            rooms_to_add: 1,
            style: ArchitecturalStyle::Industrial,
            cost_multiplier: 1.2,
        };
        let expanded = plan_expansion(&layout, &plan, 7);
        assert_eq!(expanded.building_type, "barracks");
    }

    #[test]
    fn plan_expansion_deterministic() {
        let layout = LayoutCatalog::temple();
        let plan = ExpansionPlan {
            direction: ExpansionDirection::North,
            rooms_to_add: 2,
            style: ArchitecturalStyle::Ancient,
            cost_multiplier: 1.0,
        };
        let a = plan_expansion(&layout, &plan, 99);
        let b = plan_expansion(&layout, &plan, 99);
        assert_eq!(a.footprint, b.footprint);
        assert_eq!(a.capacity, b.capacity);
    }
}
