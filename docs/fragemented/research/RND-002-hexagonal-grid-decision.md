# RND-002: Hexagonal Grid Library -- hexx vs Manual Axial Implementation

**Status:** RESEARCH COMPLETE
**Date:** 2026-02-21
**Assigned to:** researcher-alpha

---

## Executive Summary

**Recommendation: Use the `hexx` crate (version 0.21.x)** for CivLab's hexagonal grid system.
`hexx` is the most actively maintained Rust hex library, provides integer-based axial
coordinates (`Hex` struct with `i32` fields), includes A* pathfinding, range/ring/spiral
queries, neighbor lookups, coordinate conversions (axial/cube/offset/doubled), and has optional
Bevy integration. The core hex coordinate math is entirely integer (`i32`), meeting our
determinism requirement. The `f32` usage is confined to the `HexLayout` display layer
(hex-to-pixel conversion for rendering), which is non-simulation code. CivLab should use
`hexx` for coordinate math and pathfinding, with a thin wrapper for simulation-specific
concerns (movement cost evaluation, territory boundaries, fog of war). A manual implementation
would require ~1500-2000 LOC to replicate what `hexx` provides, with ongoing maintenance burden
and no community review.

---

## Research Findings

### 1. `hexx` Crate Analysis

**Repository:** [github.com/ManevilleF/hexx](https://github.com/ManevilleF/hexx)
**Version:** 0.21.0 (latest as of 2026-02)
**License:** MIT/Apache-2.0
**Downloads:** ~300k total on crates.io -- well-adopted in the Rust gamedev ecosystem.

#### 1.1 Core Type: `Hex`

```rust
// From hexx source:
pub struct Hex {
    /// The `x` axial coordinate (also called `q` in some references)
    pub x: i32,
    /// The `y` axial coordinate (also called `r` in some references)
    pub y: i32,
}
```

- **Coordinate system:** Axial coordinates. Cubic `z` coordinate is derived: `z = -x - y`.
- **Integer-only core:** All coordinate math (neighbors, distance, range, ring, line) operates
  on `i32`. No floating-point contamination in simulation-critical paths.
- **`Hash` implementation:** `Hex` implements `Hash`, enabling `HashMap \< Hex, TileData>` storage.
- **`Ord` implementation:** `Hex` implements `Ord` (lexicographic on `(x, y)`), enabling
  `BTreeMap` and sorted iteration for determinism.
- **Serde:** Available via `serde` feature flag. Serializes as `{ "x": i32, "y": i32 }`.

#### 1.2 Coordinate Conversions

| Method | Direction | Notes |
|--------|-----------|-------|
| `to_cubic_array() -> [i32; 3]` | Axial -> Cubic | Returns `[x, y, z]` |
| `from_cubic([i32; 3]) -> Self` | Cubic -> Axial | Drops `z` |
| `to_offset_coordinates(mode) -> [i32; 2]` | Axial -> Offset | Supports Odd-R, Even-R, Odd-Q, Even-Q |
| `from_offset_coordinates([i32; 2], mode) -> Self` | Offset -> Axial | |
| `to_doubled_coordinates(mode) -> [i32; 2]` | Axial -> Doubled | |
| `from_doubled_coordinates([i32; 2], mode) -> Self` | Doubled -> Axial | |
| `to_hexmod_coordinates() -> [i32; 2]` | Axial -> HexMod | |
| `from_hexmod_coordinates([i32; 2]) -> Self` | HexMod -> Axial | |

All conversions are integer-only. No `f32` involved.

#### 1.3 Distance and Neighbor Operations

```rust
impl Hex {
    /// Manhattan distance in hex space. Returns i32.
    pub fn distance_to(self, other: Self) -> i32;

    /// Unsigned distance. Returns u32.
    pub fn unsigned_distance_to(self, other: Self) -> u32;

    /// Returns the neighbor in the given EdgeDirection.
    pub fn neighbor(self, direction: EdgeDirection) -> Self;

    /// Returns all 6 neighbors.
    pub fn all_neighbors(self) -> [Self; 6];

    /// Returns the diagonal neighbor in the given VertexDirection.
    pub fn diagonal_neighbor(self, direction: VertexDirection) -> Self;

    /// Returns all 6 diagonal neighbors.
    pub fn all_diagonals(self) -> [Self; 6];

    /// Identifies the EdgeDirection from self to a neighbor.
    pub fn neighbor_direction(self, other: Self) -> Option<EdgeDirection>;
}
```

All operations return `Hex` (i32 pairs) or primitive integers. Fully deterministic.

#### 1.4 Range, Ring, Line, and Spiral Queries

```rust
impl Hex {
    /// All hexes within `range` steps (inclusive). Returns Vec<Hex>.
    pub fn range(self, range: u32) -> Vec<Self>;

    /// Same as range but excludes center.
    pub fn xrange(self, range: u32) -> Vec<Self>;

    /// All hexes on the ring at exactly `radius` distance.
    pub fn ring(self, radius: u32) -> Vec<Self>;

    /// Hexes arranged in spiral rings from center outward.
    pub fn spiral_range(self, range: u32) -> Vec<Self>;

    /// All hexes along the line from self to other (Bresenham-style).
    pub fn line_to(self, other: Self) -> Vec<Self>;

    /// Two-segment rectilinear path.
    pub fn rectiline_to(self, other: Self) -> Vec<Self>;
}
```

**Determinism note:** `range()`, `ring()`, and `spiral_range()` return hexes in a consistent
order (rings iterate clockwise from a fixed starting direction). `line_to()` uses a
deterministic interpolation algorithm. No `f32` in any of these paths.

#### 1.5 A* Pathfinding

```rust
// In hexx::algorithms module:
pub fn a_star(
    start: Hex,
    end: Hex,
    cost_fn: impl FnMut(Hex, Hex) -> Option<f32>,  // NOTE: f32 cost!
) -> Option<Vec<Hex>>;
```

**Critical finding:** The A* implementation uses `f32` for edge costs. This is problematic
for determinism.

**Mitigation options:**
1. **Wrap with integer costs:** Write a thin wrapper that converts our `i64` movement costs
   to `f32` for `hexx`'s A*. Since A* only compares costs (not accumulates with catastrophic
   cancellation), the `f32` precision loss for typical movement costs (1-1000 range) is
   negligible and produces identical paths. However, this violates our strict "no f32 in
   simulation" rule.
2. **Reimplement A* over `hexx` coordinates:** Use `hexx` for coordinate math (neighbors,
   distance heuristic) but implement our own A* with `i64` costs. This is ~50-80 LOC and
   fully deterministic. **Recommended.**
3. **Contribute upstream:** Submit a PR to `hexx` adding a generic cost type parameter.
   Possible but not guaranteed to be accepted, and blocks our timeline.

**Recommendation:** Option 2. Reimplement A* using `hexx::Hex` for neighbor/heuristic
queries, with integer costs.

#### 1.6 Field of View and Field of Movement

```rust
pub fn field_of_view(
    origin: Hex,
    range: u32,
    blocking_fn: impl FnMut(Hex) -> bool,
) -> HashSet<Hex>;

pub fn field_of_movement(
    origin: Hex,
    budget: f32,  // NOTE: f32 budget!
    cost_fn: impl FnMut(Hex, Hex) -> Option<f32>,
) -> HashMap<Hex, f32>;
```

Same `f32` issue as A*. Same mitigation: reimplement FOV/FOM with integer costs using `hexx`
for coordinate math. FOV's `blocking_fn` is bool-returning and fully integer; only the budget
tracking needs replacement.

#### 1.7 HexLayout (Display Layer)

```rust
pub struct HexLayout {
    pub orientation: HexOrientation,  // Flat-top or Pointy-top
    pub origin: Vec2,                 // f32 pixel offset
    pub hex_size: Vec2,              // f32 pixel size
    pub invert_x: bool,
    pub invert_y: bool,
}

impl HexLayout {
    /// Converts hex coordinates to world pixel position.
    pub fn hex_to_world_pos(&self, hex: Hex) -> Vec2;  // Returns f32 Vec2

    /// Converts world pixel position to hex coordinates.
    pub fn world_pos_to_hex(&self, pos: Vec2) -> Hex;  // Accepts f32, returns i32 Hex
}
```

The `HexLayout` is the **only** part of `hexx` that uses `f32`, and it's purely for
rendering/display. The simulation never touches `HexLayout` -- only the client renderer does.
This is safe and expected.

#### 1.8 Storage Collections

`hexx` provides optimized dense storage types:

| Type | Description | Use Case |
|------|-------------|----------|
| `HexagonalMap\<T\>` | Dense hexagonal area storage | Fixed-size hex maps |
| `RombusMap\<T\>` | Dense rhombus-shaped storage | Rectangular regions |
| `HexModMap\<T\>` | HexMod-addressed storage | Wrapping/tiling maps |

These use array-based indexing (faster than `HashMap`) for known map bounds. All integer-
addressed.

#### 1.9 Cargo Features

```toml
[features]
default = []
serde = ["dep:serde"]        # Serde Serialize/Deserialize
bevy = ["dep:bevy_ecs", "dep:bevy_reflect", "dep:bevy_math"]  # Bevy integration
grid = []                     # Face/Vertex/Edge grid types
rayon = ["dep:rayon"]         # Parallel iteration
mesh = ["dep:glam"]           # Mesh generation (rendering)
```

For the simulation crate: use `serde` only. The `bevy` feature is for the client crate.

### 2. Manual Implementation Assessment

If we built hex grid support from scratch:

| Feature | LOC Estimate | Complexity |
|---------|-------------|------------|
| `Hex` struct + basic ops | ~100 | Low |
| Neighbor/direction lookups | ~80 | Low |
| Distance (Manhattan, unsigned) | ~30 | Low |
| Coordinate conversions (cube, offset, doubled) | ~150 | Medium |
| Range/ring/spiral queries | ~200 | Medium |
| Line drawing (Bresenham hex) | ~60 | Medium |
| A* pathfinding (integer) | ~80 | Medium |
| FOV (shadowcasting hex) | ~150 | High |
| Field of movement (Dijkstra) | ~80 | Medium |
| Dense storage collections | ~200 | Medium |
| Serde implementations | ~50 | Low |
| Tests | ~400 | Medium |
| **Total** | **~1580** | -- |

**Maintenance burden:** Every new hex algorithm (wedge queries, hex region intersection,
multi-resolution) would need to be implemented and tested from scratch. `hexx` already has
these and is community-maintained.

**Risk:** Manual implementations of hex math have subtle edge cases (negative coordinates,
boundary conditions, wrap-around). `hexx` has been battle-tested by the Bevy gamedev community.

### 3. Alternative Libraries

| Crate | Status | Notes |
|-------|--------|-------|
| `hexagonal` | Unmaintained (last update 2020) | Incomplete API, no pathfinding |
| `hex2d` | Low activity | Uses `i32` but minimal features |
| `hexing` | New (2024) | Small, missing many features |
| `hexgridspiral` | Niche | Spiral-only, not general purpose |

None are competitive with `hexx` in feature completeness or maintenance.

---

## Decision

**Use `hexx` version 0.21.x** with the following strategy:

1. **Use `hexx::Hex` as the canonical coordinate type** throughout the simulation. All tile
   positions, unit positions, and territory boundaries use `Hex`.
2. **Use `hexx`'s integer operations directly:** neighbors, distance, range, ring, spiral,
   line drawing, coordinate conversions.
3. **Reimplement pathfinding with integer costs:** Write CivLab-specific A*, FOV, and
   field-of-movement algorithms that use `hexx::Hex` for coordinate math but `i64` for cost
   accumulation. This is ~200 LOC total.
4. **Use `HexLayout` only in the client renderer**, never in the simulation crate.
5. **Use `HexagonalMap\<T\>` or `HashMap \< Hex, T>`** for tile data storage, depending on whether
   the map has fixed bounds.

---

## Implementation Contract

### Cargo.toml (simulation crate)

```toml
[dependencies]
hexx = { version = "0.21", default-features = false, features = ["serde"] }
```

### Cargo.toml (client crate)

```toml
[dependencies]
hexx = { version = "0.21", features = ["serde", "bevy", "mesh"] }
```

### Canonical Coordinate Type

```rust
// In crates/engine/src/hex.rs or similar

/// Re-export hexx::Hex as the canonical coordinate type.
/// All simulation code uses this type for tile/entity positions.
pub use hexx::Hex;

/// Re-export direction types.
pub use hexx::{EdgeDirection, VertexDirection};

/// Re-export coordinate conversion modes.
pub use hexx::OffsetHexMode;
```

### Map Storage Pattern

```rust
use hexx::Hex;
use std::collections::HashMap;

/// Tile data for the simulation. Each hex cell has associated data.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct TileData {
    pub terrain: TerrainType,
    pub elevation: i32,          // meters, integer
    pub moisture: i32,           // 0-1000 scale (fixed-point-like)
    pub movement_cost: i64,      // base movement cost to enter this tile
    pub owner: Option<NationId>,
}

/// The hex map: HashMap<Hex, TileData>.
/// For fixed-size maps, consider hexx::HexagonalMap<TileData> for better perf.
pub type HexMap = HashMap<Hex, TileData>;
```

### Integer A* Pathfinding

```rust
use hexx::Hex;
use std::collections::{BinaryHeap, HashMap};
use std::cmp::Reverse;

/// A* pathfinding with integer costs. Uses hexx for coordinate math.
/// Returns None if no path exists.
pub fn a_star_integer(
    start: Hex,
    goal: Hex,
    cost_fn: impl Fn(Hex, Hex) -> Option<i64>,
) -> Option<(Vec<Hex>, i64)> {
    // Priority queue: (cost, hex). Use Reverse for min-heap.
    let mut open: BinaryHeap<Reverse<(i64, Hex)>> = BinaryHeap::new();
    let mut came_from: HashMap<Hex, Hex> = HashMap::new();
    let mut g_score: HashMap<Hex, i64> = HashMap::new();

    g_score.insert(start, 0);
    let h = heuristic(start, goal);
    open.push(Reverse((h, start)));

    while let Some(Reverse((_, current))) = open.pop() {
        if current == goal {
            return Some(reconstruct_path(&came_from, current, &g_score));
        }

        let current_g = g_score[&current];

        // Use hexx's all_neighbors() for neighbor enumeration
        for neighbor in current.all_neighbors() {
            if let Some(edge_cost) = cost_fn(current, neighbor) {
                let tentative_g = current_g + edge_cost;
                if tentative_g < *g_score.get(&neighbor).unwrap_or(&i64::MAX) {
                    came_from.insert(neighbor, current);
                    g_score.insert(neighbor, tentative_g);
                    let f = tentative_g + heuristic(neighbor, goal);
                    open.push(Reverse((f, neighbor)));
                }
            }
        }
    }
    None
}

/// Heuristic: hex Manhattan distance scaled to minimum movement cost.
/// Uses hexx's built-in distance_to (integer).
fn heuristic(a: Hex, b: Hex) -> i64 {
    a.unsigned_distance_to(b) as i64 * MIN_MOVEMENT_COST
}

const MIN_MOVEMENT_COST: i64 = 100; // Minimum possible tile cost

fn reconstruct_path(
    came_from: &HashMap<Hex, Hex>,
    mut current: Hex,
    g_score: &HashMap<Hex, i64>,
) -> (Vec<Hex>, i64) {
    let total_cost = g_score[&current];
    let mut path = vec![current];
    while let Some(&prev) = came_from.get(&current) {
        path.push(prev);
        current = prev;
    }
    path.reverse();
    (path, total_cost)
}
```

### Integer Field of View

```rust
use hexx::Hex;
use std::collections::HashSet;

/// Field of view using hexx coordinate math with boolean blocking.
/// No f32 involved -- purely geometric visibility.
pub fn field_of_view_integer(
    origin: Hex,
    range: u32,
    is_blocking: impl Fn(Hex) -> bool,
) -> HashSet<Hex> {
    let mut visible = HashSet::new();
    visible.insert(origin);

    // Ray-cast from origin to each hex on the outer ring
    for target in origin.ring(range) {
        let line = origin.line_to(target);
        for hex in line {
            if hex == origin {
                continue;
            }
            visible.insert(hex);
            if is_blocking(hex) {
                break; // Stop ray at blocking tile
            }
        }
    }
    visible
}
```

### Viewport Culling (AABB to Hex Range)

```rust
use hexx::{Hex, HexLayout};

/// Convert a screen-space AABB to the set of visible hex coordinates.
/// This is CLIENT-ONLY code (uses f32 via HexLayout).
/// Never call from simulation.
pub fn viewport_to_hex_range(
    layout: &HexLayout,
    viewport_min: glam::Vec2,  // top-left pixel
    viewport_max: glam::Vec2,  // bottom-right pixel
) -> Vec<Hex> {
    // Convert corners to hex coordinates
    let hex_min = layout.world_pos_to_hex(viewport_min);
    let hex_max = layout.world_pos_to_hex(viewport_max);

    // Compute bounding range and collect all hexes in the rectangle
    let range_x = hex_min.x.min(hex_max.x)..=hex_min.x.max(hex_max.x);
    let range_y = hex_min.y.min(hex_max.y)..=hex_min.y.max(hex_max.y);

    let mut result = Vec::new();
    for x in range_x {
        for y in range_y.clone() {
            result.push(Hex::new(x, y));
        }
    }
    result
}
```

### Determinism Guarantee

```rust
/// All hex operations in the simulation crate MUST satisfy:
/// 1. No f32/f64 in any computation
/// 2. All neighbor iterations use hexx's fixed EdgeDirection ordering
/// 3. All range/ring queries return hexes in hexx's documented order
/// 4. Pathfinding uses integer costs only
/// 5. Map iteration uses BTreeMap<Hex, _> or sorted HashMap keys
///
/// The HexLayout type is FORBIDDEN in the simulation crate.
/// It is only used in the client renderer crate.
#[cfg(test)]
mod determinism_tests {
    use super::*;

    #[test]
    fn neighbor_order_is_deterministic() {
        let hex = Hex::new(5, -3);
        let neighbors_a = hex.all_neighbors();
        let neighbors_b = hex.all_neighbors();
        assert_eq!(neighbors_a, neighbors_b);
    }

    #[test]
    fn range_order_is_deterministic() {
        let hex = Hex::new(0, 0);
        let range_a = hex.range(10);
        let range_b = hex.range(10);
        assert_eq!(range_a, range_b);
    }

    #[test]
    fn pathfinding_is_deterministic() {
        let start = Hex::new(0, 0);
        let goal = Hex::new(10, -5);
        let cost_fn = |_from: Hex, _to: Hex| -> Option<i64> { Some(100) };

        let path_a = a_star_integer(start, goal, cost_fn);
        let path_b = a_star_integer(start, goal, cost_fn);
        assert_eq!(path_a, path_b);
    }
}
```

---

## Open Questions Remaining

1. **`hexx` Bevy version compatibility:** `hexx` 0.21 targets a specific Bevy version for
   its `bevy` feature. Verify compatibility with `bevy 0.18` or whatever version the client
   uses. The simulation crate doesn't use the `bevy` feature, so this only affects the client.

2. **Map wrapping / toroidal geometry:** CivLab may want a wrapping world map (cylindrical
   or toroidal). `hexx` doesn't natively support wrapping. Need to implement modular
   coordinate wrapping on top of `Hex`. Estimate: ~100 LOC for cylindrical wrapping.

3. **Multi-resolution hex grids:** `hexx` supports `to_lower_res()` and `to_higher_res()`
   for multi-resolution coordinates. Evaluate if this is useful for CivLab's zoom levels
   or strategic map view. If used, ensure the resolution conversion is deterministic (it
   should be, as it's purely integer math).

4. **Performance of `HashMap \< Hex, TileData>`:** For a 100x100 hex map (10k tiles), `HashMap`
   is fine. For larger maps (1M+ tiles), `HexagonalMap` (dense array) will be significantly
   faster. Profile and decide based on actual map sizes.

5. **A* performance budget:** The integer A* implementation uses `BinaryHeap` which is
   O(n log n). For long paths across large maps (>1000 tiles), consider jump-point search
   or hierarchical pathfinding. This is an optimization concern, not a correctness concern.

---

## References

- [hexx crates.io](https://crates.io/crates/hexx)
- [hexx docs.rs](https://docs.rs/hexx/latest/hexx/)
- [hexx GitHub](https://github.com/ManevilleF/hexx)
- [Hex struct API](https://docs.rs/hexx/latest/hexx/struct.Hex.html)
- [Red Blob Games - Hexagonal Grids](https://www.redblobgames.com/grids/hexagons/) (canonical reference)
- [hexx algorithms module](https://docs.rs/hexx/latest/hexx/algorithms/index.html)
