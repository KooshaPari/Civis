//! Unit formation offsets (FR-CIV-TACTICS-021).
//!
//! Formations compute **absolute grid positions** for a squad given a list of
//! current unit positions, a desired anchor (e.g. the squad centroid or leader
//! position), and the direction the formation faces.  All output positions are
//! snapped to integer grid coordinates.

/// Cardinal facing direction for a formation.
///
/// `North` = −Y, `South` = +Y, `East` = +X, `West` = −X (grid convention).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Facing {
    /// Formation faces −Y (grid north).
    North,
    /// Formation faces +Y (grid south).
    South,
    /// Formation faces +X (grid east).
    East,
    /// Formation faces −X (grid west).
    West,
}

/// Tactical formation layout for squad offsets on the hex/grid plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormationKind {
    /// Single rank along the axis perpendicular to facing.
    Line,
    /// Single file along the facing axis (depth-first column).
    Column,
    /// V-shaped advance (leader at front, wings trail behind).
    Wedge,
    /// Compact block (nearest square, fills row by row).
    Square,
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Rotate a canonical offset `(dx, dy)` — defined for `Facing::East` (i.e.
/// +X = forward, +Y = right) — into the requested facing direction.
///
/// Canonical frame:
///   +dx = forward (depth into the formation)
///   +dy = right flank
fn rotate_offset(dx: i32, dy: i32, facing: Facing) -> (i32, i32) {
    match facing {
        // East: forward = +X, right = +Y → identity
        Facing::East => (dx, dy),
        // West: forward = −X, right = −Y → negate both
        Facing::West => (-dx, -dy),
        // North: forward = −Y, right = +X
        //   world_x = +dy,  world_y = -dx
        Facing::North => (dy, -dx),
        // South: forward = +Y, right = −X
        //   world_x = -dy,  world_y = +dx
        Facing::South => (-dy, dx),
    }
}

/// Compute the centroid of a set of (x, y) positions (integer-rounded).
///
/// Returns `(0, 0)` for an empty slice.
fn centroid(positions: &[(i32, i32)]) -> (i32, i32) {
    if positions.is_empty() {
        return (0, 0);
    }
    let sum_x: i32 = positions.iter().map(|p| p.0).sum();
    let sum_y: i32 = positions.iter().map(|p| p.1).sum();
    let n = positions.len() as i32;
    (sum_x / n, sum_y / n)
}

// ---------------------------------------------------------------------------
// Core public API
// ---------------------------------------------------------------------------

/// Grid offsets `(dx, dy)` for `slots` units anchored at the leader cell,
/// in the **canonical East-facing frame**.  Positive `dx` = deeper into the
/// formation (forward), positive `dy` = right flank.
///
/// This is the low-level offset generator.  Use [`formation_positions`] for
/// absolute positions with full facing support.
pub fn formation_offsets(kind: FormationKind, slots: usize) -> Vec<(i32, i32)> {
    if slots == 0 {
        return Vec::new();
    }
    match kind {
        FormationKind::Line => {
            // Rank perpendicular to forward (dy axis), centred on leader.
            let center = (slots as i32 - 1) / 2;
            (0..slots).map(|i| (0, i as i32 - center)).collect()
        }
        FormationKind::Column => {
            // File along forward (dx axis), leader at front (dx=0).
            (0..slots).map(|i| (i as i32, 0)).collect()
        }
        FormationKind::Wedge => {
            // Leader at front (dx=0,dy=0); wings extend back and outward.
            let mut out = Vec::with_capacity(slots);
            out.push((0, 0));
            let mut rank = 1i32;
            let mut placed = 1usize;
            while placed < slots {
                // Left wing first, then right wing.
                for &side in &[-1i32, 1i32] {
                    if placed >= slots {
                        break;
                    }
                    // Trailing rank = +dx, lateral = side*rank.
                    out.push((rank, side * rank));
                    placed += 1;
                }
                rank += 1;
            }
            out
        }
        FormationKind::Square => {
            // Compact grid, row-major, centred on (0,0).
            let side = (slots as f64).sqrt().ceil() as i32;
            let col_offset = (side - 1) / 2;
            let row_offset = (side - 1) / 2;
            let mut out = Vec::with_capacity(slots);
            'outer: for row in 0..side {
                for col in 0..side {
                    if out.len() >= slots {
                        break 'outer;
                    }
                    out.push((row - row_offset, col - col_offset));
                }
            }
            out
        }
    }
}

/// Apply facing rotation to a set of canonical offsets.
///
/// Returns a new vector of offsets rotated into the given `facing` direction.
pub fn rotate_offsets(offsets: &[(i32, i32)], facing: Facing) -> Vec<(i32, i32)> {
    offsets
        .iter()
        .map(|&(dx, dy)| rotate_offset(dx, dy, facing))
        .collect()
}

/// Compute **absolute target grid positions** for a squad.
///
/// The formation is anchored at the centroid of `unit_positions`.  Each slot
/// receives a target position by applying the facing-rotated formation offset
/// to that anchor.
///
/// # Arguments
/// * `unit_positions` – current grid positions `(x, y)` of each unit in the
///   squad.  The length of this slice determines `slots`.
/// * `kind`    – desired formation pattern.
/// * `facing`  – direction the formation faces.
///
/// # Returns
/// A `Vec<(i32, i32)>` of the same length as `unit_positions`, where each
/// element is the snapped grid target for the corresponding slot.
pub fn formation_positions(
    unit_positions: &[(i32, i32)],
    kind: FormationKind,
    facing: Facing,
) -> Vec<(i32, i32)> {
    let slots = unit_positions.len();
    if slots == 0 {
        return Vec::new();
    }
    let anchor = centroid(unit_positions);
    let offsets = formation_offsets(kind, slots);
    rotate_offsets(&offsets, facing)
        .into_iter()
        .map(|(dx, dy)| (anchor.0 + dx, anchor.1 + dy))
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ------------------------------------------------------------------
    // Helpers
    // ------------------------------------------------------------------

    /// Build a trivial squad of `n` units all at the origin.
    fn squad_at_origin(n: usize) -> Vec<(i32, i32)> {
        vec![(0, 0); n]
    }

    /// Build a squad with units spread along X (useful for centroid tests).
    fn squad_on_x(n: usize) -> Vec<(i32, i32)> {
        (0..n as i32).map(|x| (x, 0)).collect()
    }

    // ------------------------------------------------------------------
    // formation_offsets — canonical frame (East)
    // ------------------------------------------------------------------

    /// FR-CIV-TACTICS-021 — Line: slots spread on dy axis, centred.
    #[test]
    fn line_offsets_odd_count() {
        let offsets = formation_offsets(FormationKind::Line, 3);
        assert_eq!(offsets, vec![(0, -1), (0, 0), (0, 1)]);
    }

    #[test]
    fn line_offsets_even_count() {
        let offsets = formation_offsets(FormationKind::Line, 4);
        // centre = (4-1)/2 = 1  → slots at dy = -1, 0, 1, 2
        assert_eq!(offsets, vec![(0, -1), (0, 0), (0, 1), (0, 2)]);
    }

    #[test]
    fn line_single_unit() {
        assert_eq!(formation_offsets(FormationKind::Line, 1), vec![(0, 0)]);
    }

    /// Column: units file along dx axis.
    #[test]
    fn column_offsets_odd_count() {
        let offsets = formation_offsets(FormationKind::Column, 3);
        assert_eq!(offsets, vec![(0, 0), (1, 0), (2, 0)]);
    }

    #[test]
    fn column_offsets_even_count() {
        let offsets = formation_offsets(FormationKind::Column, 4);
        assert_eq!(offsets, vec![(0, 0), (1, 0), (2, 0), (3, 0)]);
    }

    #[test]
    fn column_single_unit() {
        assert_eq!(formation_offsets(FormationKind::Column, 1), vec![(0, 0)]);
    }

    /// Wedge: leader at (0,0), wings trail back.
    #[test]
    fn wedge_offsets_single() {
        let offsets = formation_offsets(FormationKind::Wedge, 1);
        assert_eq!(offsets, vec![(0, 0)]);
    }

    #[test]
    fn wedge_offsets_three() {
        let offsets = formation_offsets(FormationKind::Wedge, 3);
        assert_eq!(offsets.len(), 3);
        assert_eq!(offsets[0], (0, 0), "leader at front");
        // Wings: rank=1, sides -1 and +1 → (1,-1) and (1,+1)
        assert!(offsets.contains(&(1, -1)));
        assert!(offsets.contains(&(1, 1)));
    }

    #[test]
    fn wedge_offsets_five() {
        let offsets = formation_offsets(FormationKind::Wedge, 5);
        assert_eq!(offsets.len(), 5);
        assert_eq!(offsets[0], (0, 0));
        // rank 1: (1,-1), (1,1)
        assert!(offsets.contains(&(1, -1)));
        assert!(offsets.contains(&(1, 1)));
        // rank 2: (2,-2), (2,2)
        assert!(offsets.contains(&(2, -2)));
        assert!(offsets.contains(&(2, 2)));
    }

    /// Square: compact grid centred on origin.
    #[test]
    fn square_offsets_four() {
        let offsets = formation_offsets(FormationKind::Square, 4);
        assert_eq!(offsets.len(), 4);
        // side=2, offsets = (0,0),(0,1),(1,0),(1,1) before centre-shift
        // col_offset=0, row_offset=0 → same (no shift for side=2 odd floor)
        // Actually centre = (side-1)/2 = 0, so no shift
        for &o in &offsets {
            assert!(
                o.0 >= -1 && o.0 <= 1 && o.1 >= -1 && o.1 <= 1,
                "offset {:?} out of expected range",
                o
            );
        }
        // No duplicates
        let mut sorted = offsets.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 4, "no duplicate offsets");
    }

    #[test]
    fn square_offsets_nine() {
        let offsets = formation_offsets(FormationKind::Square, 9);
        assert_eq!(offsets.len(), 9);
        let mut sorted = offsets.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 9, "no duplicate offsets");
    }

    #[test]
    fn square_single_unit() {
        assert_eq!(formation_offsets(FormationKind::Square, 1), vec![(0, 0)]);
    }

    /// Empty slots always returns empty vec.
    #[test]
    fn empty_slots_returns_empty() {
        for kind in [
            FormationKind::Line,
            FormationKind::Column,
            FormationKind::Wedge,
            FormationKind::Square,
        ] {
            assert!(formation_offsets(kind, 0).is_empty());
        }
    }

    // ------------------------------------------------------------------
    // rotate_offsets — all four cardinal directions
    // ------------------------------------------------------------------

    #[test]
    fn rotate_east_is_identity() {
        let offsets = vec![(1, 0), (0, 1), (-1, -1)];
        assert_eq!(rotate_offsets(&offsets, Facing::East), offsets);
    }

    #[test]
    fn rotate_west_negates() {
        let offsets = vec![(1, 2)];
        assert_eq!(rotate_offsets(&offsets, Facing::West), vec![(-1, -2)]);
    }

    #[test]
    fn rotate_north() {
        // (dx, dy) → (dy, -dx)
        let offsets = vec![(1, 0)];
        assert_eq!(rotate_offsets(&offsets, Facing::North), vec![(0, -1)]);
    }

    #[test]
    fn rotate_south() {
        // (dx, dy) → (-dy, dx)
        let offsets = vec![(1, 0)];
        assert_eq!(rotate_offsets(&offsets, Facing::South), vec![(0, 1)]);
    }

    /// Rotating all four directions and back must form a cycle.
    #[test]
    fn rotation_cycle_identity() {
        let original = vec![(3, -2)];
        // East → North → West → South → East (four 90-degree CCW rotations)
        let n = rotate_offsets(&original, Facing::North);
        let w = rotate_offsets(&n, Facing::North);
        let s = rotate_offsets(&w, Facing::North);
        let e = rotate_offsets(&s, Facing::North);
        // Each North rotation: (dx,dy) → (dy,-dx)
        // Four applications returns to original.
        assert_eq!(
            e, original,
            "four North rotations should return to original"
        );
    }

    // ------------------------------------------------------------------
    // formation_positions — Line pattern in all four cardinal directions
    // ------------------------------------------------------------------

    fn line_positions(n: usize, facing: Facing) -> Vec<(i32, i32)> {
        formation_positions(&squad_at_origin(n), FormationKind::Line, facing)
    }

    /// Line facing East: units spread along Y axis.
    #[test]
    fn line_facing_east_spreads_on_y() {
        let pos = line_positions(3, Facing::East);
        // East: rotate (0,dy) → (0,dy) — spread on Y
        let mut ys: Vec<i32> = pos.iter().map(|p| p.1).collect();
        ys.sort_unstable();
        assert_eq!(ys, vec![-1, 0, 1]);
        assert!(pos.iter().all(|p| p.0 == 0), "all on x=0");
    }

    /// Line facing North: units spread along X axis.
    #[test]
    fn line_facing_north_spreads_on_x() {
        let pos = line_positions(3, Facing::North);
        // North: rotate (0,dy) → (dy, 0) — spread on X
        let mut xs: Vec<i32> = pos.iter().map(|p| p.0).collect();
        xs.sort_unstable();
        assert_eq!(xs, vec![-1, 0, 1]);
        assert!(pos.iter().all(|p| p.1 == 0), "all on y=0");
    }

    /// Line facing South: units spread along X axis (mirrored).
    #[test]
    fn line_facing_south_spreads_on_x() {
        let pos = line_positions(3, Facing::South);
        let mut xs: Vec<i32> = pos.iter().map(|p| p.0).collect();
        xs.sort_unstable();
        assert_eq!(xs, vec![-1, 0, 1]);
    }

    /// Line facing West: units spread along Y axis (mirrored).
    #[test]
    fn line_facing_west_spreads_on_y() {
        let pos = line_positions(3, Facing::West);
        let mut ys: Vec<i32> = pos.iter().map(|p| p.1).collect();
        ys.sort_unstable();
        assert_eq!(ys, vec![-1, 0, 1]);
        assert!(pos.iter().all(|p| p.0 == 0));
    }

    // ------------------------------------------------------------------
    // formation_positions — Column pattern
    // ------------------------------------------------------------------

    fn column_positions(n: usize, facing: Facing) -> Vec<(i32, i32)> {
        formation_positions(&squad_at_origin(n), FormationKind::Column, facing)
    }

    /// Column facing East: units file along +X.
    #[test]
    fn column_facing_east_files_along_x() {
        let pos = column_positions(3, Facing::East);
        // East: rotate (dx,0) → (dx,0)
        let mut xs: Vec<i32> = pos.iter().map(|p| p.0).collect();
        xs.sort_unstable();
        assert_eq!(xs, vec![0, 1, 2]);
        assert!(pos.iter().all(|p| p.1 == 0));
    }

    /// Column facing North: units file along −Y.
    #[test]
    fn column_facing_north_files_along_neg_y() {
        let pos = column_positions(3, Facing::North);
        // North: rotate (dx,0) → (0,-dx) — file along -Y
        let mut ys: Vec<i32> = pos.iter().map(|p| p.1).collect();
        ys.sort_unstable();
        assert_eq!(ys, vec![-2, -1, 0]);
        assert!(pos.iter().all(|p| p.0 == 0));
    }

    /// Column facing South: units file along +Y.
    #[test]
    fn column_facing_south_files_along_pos_y() {
        let pos = column_positions(3, Facing::South);
        // South: rotate (dx,0) → (0,dx) — file along +Y
        let mut ys: Vec<i32> = pos.iter().map(|p| p.1).collect();
        ys.sort_unstable();
        assert_eq!(ys, vec![0, 1, 2]);
        assert!(pos.iter().all(|p| p.0 == 0));
    }

    /// Column facing West: units file along −X.
    #[test]
    fn column_facing_west_files_along_neg_x() {
        let pos = column_positions(3, Facing::West);
        // West: rotate (dx,0) → (-dx,0)
        let mut xs: Vec<i32> = pos.iter().map(|p| p.0).collect();
        xs.sort_unstable();
        assert_eq!(xs, vec![-2, -1, 0]);
        assert!(pos.iter().all(|p| p.1 == 0));
    }

    // ------------------------------------------------------------------
    // formation_positions — Wedge
    // ------------------------------------------------------------------

    #[test]
    fn wedge_positions_leader_at_anchor_east() {
        let pos = formation_positions(&squad_at_origin(3), FormationKind::Wedge, Facing::East);
        assert!(pos.contains(&(0, 0)), "leader stays at anchor");
    }

    #[test]
    fn wedge_facing_north_leader_at_anchor() {
        let pos = formation_positions(&squad_at_origin(3), FormationKind::Wedge, Facing::North);
        assert!(
            pos.contains(&(0, 0)),
            "leader stays at anchor when facing North"
        );
    }

    // ------------------------------------------------------------------
    // formation_positions — Square
    // ------------------------------------------------------------------

    #[test]
    fn square_positions_no_duplicates() {
        for n in [1, 2, 4, 5, 9] {
            let pos = formation_positions(&squad_at_origin(n), FormationKind::Square, Facing::East);
            assert_eq!(pos.len(), n);
            let mut sorted = pos.clone();
            sorted.sort_unstable();
            sorted.dedup();
            assert_eq!(sorted.len(), n, "no duplicate positions for Square({})", n);
        }
    }

    // ------------------------------------------------------------------
    // Anchor follows centroid of squad
    // ------------------------------------------------------------------

    #[test]
    fn anchor_follows_centroid() {
        // Squad of 3 units at x=0,1,2 → centroid=(1,0)
        let positions = squad_on_x(3);
        let pos = formation_positions(&positions, FormationKind::Line, Facing::East);
        // Line East: centred on (1,0), spread dy=-1,0,+1
        let mut ys: Vec<i32> = pos.iter().map(|p| p.1).collect();
        ys.sort_unstable();
        assert_eq!(ys, vec![-1, 0, 1]);
        assert!(pos.iter().all(|p| p.0 == 1), "all x == centroid x");
    }

    // ------------------------------------------------------------------
    // Odd / even unit count correctness
    // ------------------------------------------------------------------

    #[test]
    fn line_even_count_east_correct_spread() {
        // 4 units: centre = (4-1)/2 = 1 → dy offsets = -1,0,1,2
        let pos = formation_positions(&squad_at_origin(4), FormationKind::Line, Facing::East);
        let mut ys: Vec<i32> = pos.iter().map(|p| p.1).collect();
        ys.sort_unstable();
        assert_eq!(ys, vec![-1, 0, 1, 2]);
    }

    #[test]
    fn wedge_even_count_all_slots_filled() {
        for n in [2, 4, 6] {
            let pos = formation_positions(&squad_at_origin(n), FormationKind::Wedge, Facing::East);
            assert_eq!(pos.len(), n, "wedge slots filled for n={}", n);
        }
    }

    #[test]
    fn column_all_unique_for_large_squad() {
        let pos = formation_positions(&squad_at_origin(8), FormationKind::Column, Facing::North);
        let mut sorted = pos.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 8, "no duplicates in 8-unit column");
    }

    // ------------------------------------------------------------------
    // Slot count == input unit count
    // ------------------------------------------------------------------

    #[test]
    fn output_len_matches_input_len() {
        for n in [0, 1, 3, 7] {
            let units = squad_at_origin(n);
            for kind in [
                FormationKind::Line,
                FormationKind::Column,
                FormationKind::Wedge,
                FormationKind::Square,
            ] {
                for facing in [Facing::North, Facing::South, Facing::East, Facing::West] {
                    let pos = formation_positions(&units, kind, facing);
                    assert_eq!(
                        pos.len(),
                        n,
                        "len mismatch for {:?} {:?} n={}",
                        kind,
                        facing,
                        n
                    );
                }
            }
        }
    }
}
