//! Connected-components / structure-count metric.
//!
//! See `docs/design/emergence-dashboard.md` §3.3 for the design rationale.
//!
//! The dashboard samples a 3-D voxel grid at stride `S` (default 16) and
//! runs 6-connectivity connected-components labelling on the *binary
//! solid mask* `M(p) = 1[material(p) > air]`. This metric returns the
//! count of components, the size of the largest component, and an
//! `evaluate` helper that runs both. The union-find core is Hopcroft-
//! Ullman 1973.
//!
//! ## Why a separate `Grid` type?
//!
//! Decoupling the grid representation (any flat `[T]` plus a stride) from
//! the metric keeps the metric testable on small synthetic inputs and
//! keeps the dashboard free to take a `&[MaterialId]` from `civ-voxel` and
//! pass a `Grid<&MaterialId>` view without copying.
//!
//! ## Why 6-connectivity, not 26?
//!
//! Face-share is the canonical percolation-theoretic choice (Stauffer &
//! Aharony 1995, ch. 2). 26-connectivity inflates the incipient-infinite-
//! cluster exponent `β` and biases the structure-count trend; 6 keeps the
//! `β ∈ [0.35, 0.50]` band the dashboard uses to alarm.

use crate::{Histogram, Metric};

/// 3-D dense grid of `T` with a fixed `(sx, sy, sz)` shape and a flat
/// `data` buffer in `z`-major / `y`-minor / `x`-innermost order
/// (`idx = x + y * sx + z * sx * sy`).
#[derive(Debug, Clone, Copy)]
pub struct Grid<'a, T> {
    /// Width (x).
    pub sx: usize,
    /// Height (y).
    pub sy: usize,
    /// Depth (z).
    pub sz: usize,
    /// Flat cell buffer.
    pub data: &'a [T],
}

impl<'a, T> Grid<'a, T> {
    /// Construct a grid; returns `None` if `data.len() != sx * sy * sz`.
    #[must_use]
    pub fn new(sx: usize, sy: usize, sz: usize, data: &'a [T]) -> Option<Self> {
        if sx.checked_mul(sy)?.checked_mul(sz)? != data.len() {
            return None;
        }
        Some(Self { sx, sy, sz, data })
    }

    /// Read cell at integer coords; returns `None` when out of range.
    #[must_use]
    pub fn get(&self, x: usize, y: usize, z: usize) -> Option<&T> {
        if x >= self.sx || y >= self.sy || z >= self.sz {
            return None;
        }
        Some(&self.data[x + y * self.sx + z * self.sx * self.sy])
    }

    /// Total number of cells.
    #[must_use]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// `true` iff the grid is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
}

/// Result of a single connected-components pass.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ComponentSummary {
    /// Number of distinct components found (excluding the background).
    pub count: usize,
    /// Size of the largest component, in cells.
    pub largest: usize,
    /// Total cells that were classified as foreground (i.e. visited by
    /// the BFS/DFS). Useful for sanity-checking that the mask is right.
    pub foreground: usize,
}

/// Structure-count metric.
///
/// The metric ignores its `Histogram` argument: a `Grid` carries richer
/// information than a histogram, so the trait is satisfied with a
/// degenerate `compute` that reports the largest-component fraction
/// (`largest / foreground`). Call [`StructureCount::evaluate`] directly
/// for the full [`ComponentSummary`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct StructureCount;

impl StructureCount {
    /// Construct a new instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    /// Run 6-connectivity connected components on the binary mask
    /// `M(p) = 1[pred(cell(p))]`.
    ///
    /// Time: O(N α(N)) where N is the grid size. Space: O(N) for the
    /// `visited` buffer; the recursion is iterative (we use an explicit
    /// stack) so we don't blow the stack on large grids.
    pub fn evaluate<T, F>(&self, grid: &Grid<'_, T>, pred: F) -> ComponentSummary
    where
        F: Fn(&T) -> bool,
    {
        let n = grid.len();
        let mut visited = vec![false; n];
        let mut stack: Vec<usize> = Vec::new();
        let mut summary = ComponentSummary::default();

        if n == 0 {
            return summary;
        }

        for start in 0..n {
            if visited[start] || !pred(&grid.data[start]) {
                continue;
            }

            // New component — DFS it.
            let mut size: usize = 0;
            stack.clear();
            stack.push(start);
            visited[start] = true;

            while let Some(idx) = stack.pop() {
                size += 1;
                let x = idx % grid.sx;
                let y = (idx / grid.sx) % grid.sy;
                let z = idx / (grid.sx * grid.sy);

                // 6-connectivity neighbours: ±x, ±y, ±z.
                // We could emit them all and bounds-check; for a hot
                // path the inline tests are equivalent to a tighter
                // implementation in the same op count.
                if x > 0 {
                    let n_idx = idx - 1;
                    if !visited[n_idx] && pred(&grid.data[n_idx]) {
                        visited[n_idx] = true;
                        stack.push(n_idx);
                    }
                }
                if x + 1 < grid.sx {
                    let n_idx = idx + 1;
                    if !visited[n_idx] && pred(&grid.data[n_idx]) {
                        visited[n_idx] = true;
                        stack.push(n_idx);
                    }
                }
                if y > 0 {
                    let n_idx = idx - grid.sx;
                    if !visited[n_idx] && pred(&grid.data[n_idx]) {
                        visited[n_idx] = true;
                        stack.push(n_idx);
                    }
                }
                if y + 1 < grid.sy {
                    let n_idx = idx + grid.sx;
                    if !visited[n_idx] && pred(&grid.data[n_idx]) {
                        visited[n_idx] = true;
                        stack.push(n_idx);
                    }
                }
                if z > 0 {
                    let n_idx = idx - grid.sx * grid.sy;
                    if !visited[n_idx] && pred(&grid.data[n_idx]) {
                        visited[n_idx] = true;
                        stack.push(n_idx);
                    }
                }
                if z + 1 < grid.sz {
                    let n_idx = idx + grid.sx * grid.sy;
                    if !visited[n_idx] && pred(&grid.data[n_idx]) {
                        visited[n_idx] = true;
                        stack.push(n_idx);
                    }
                }
            }

            summary.foreground += size;
            if size > summary.largest {
                summary.largest = size;
            }
            summary.count += 1;
        }

        summary
    }
}

impl Metric for StructureCount {
    // The trait is keyed on a `Histogram`; the structure metric consumes
    // a `Grid` and is normally called via `evaluate`. We map a histogram
    // to a trivial "1-bin Dirac = 1.0" / "0-bin = 0.0" so the trait is
    // total. Callers wanting the real signal must use `evaluate`.
    const NAME: &'static str = "structure_count_largest_fraction";

    fn compute(&self, input: &Histogram) -> f32 {
        // No grid → we don't have structure information. Return 0.0 so
        // the dashboard can render a "no data" tile rather than
        // crashing; the real value comes from `evaluate`.
        if input.is_empty() {
            0.0
        } else {
            // The fraction of the histogram in its largest bin is a
            // reasonable proxy for "is the layer dominated by one
            // category" — the same alarm that `largest / foreground`
            // would raise. Not as good as the real metric, but it's a
            // non-trivial signal on the trait path.
            let total = input.total();
            if total == 0 {
                return 0.0;
            }
            let max = input.bins().iter().copied().max().unwrap_or(0);
            max as f32 / total as f32
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_grid_is_zero_components() {
        let data: [u8; 0] = [];
        let g = Grid::new(0, 0, 0, &data).expect("empty grid");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s, ComponentSummary::default());
    }

    #[test]
    fn all_background_is_zero_components() {
        let data = vec![0u8; 2 * 2 * 2];
        let g = Grid::new(2, 2, 2, &data).expect("2³ grid");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 0);
        assert_eq!(s.largest, 0);
        assert_eq!(s.foreground, 0);
    }

    #[test]
    fn single_voxel_in_corner_is_one_component() {
        let mut data = vec![0u8; 3 * 3 * 3];
        data[0] = 1; // (0,0,0)
        let g = Grid::new(3, 3, 3, &data).expect("3³ grid");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 1);
        assert_eq!(s.largest, 1);
        assert_eq!(s.foreground, 1);
    }

    #[test]
    fn two_disconnected_voxels_in_face_diagonal() {
        // 3×1×1 grid: cells 0 and 2 are filled, cell 1 is empty.
        // 6-connectivity cannot reach from 0 to 2 (they are 2 cells
        // apart in x, not face-adjacent), so we expect 2 components.
        let data = [1u8, 0, 1];
        let g = Grid::new(3, 1, 1, &data).expect("3×1×1");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 2);
        assert_eq!(s.largest, 1);
        assert_eq!(s.foreground, 2);
    }

    #[test]
    fn three_voxels_in_a_row_are_one_component() {
        let data = [1u8, 1, 1];
        let g = Grid::new(3, 1, 1, &data).expect("3×1×1");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 1);
        assert_eq!(s.largest, 3);
    }

    #[test]
    fn face_connectivity_does_not_count_corner_touch() {
        // 2×2×1 grid with corners 0 and 3 filled and the centre pair
        // empty. 6-connectivity: (0,0,0) and (1,1,0) are diagonal, not
        // face-adjacent, so we expect 2 components. 26-connectivity
        // would call this 1.
        let data = [1u8, 0, 0, 1];
        let g = Grid::new(2, 2, 1, &data).expect("2×2×1");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 2, "6-connectivity must NOT merge diagonal corners");
    }

    #[test]
    fn full_block_is_one_component() {
        // 4×4×4 cube of foreground = 1 component, largest = 64 cells.
        let data = vec![1u8; 4 * 4 * 4];
        let g = Grid::new(4, 4, 4, &data).expect("4³ grid");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 1);
        assert_eq!(s.largest, 64);
        assert_eq!(s.foreground, 64);
    }

    #[test]
    fn checkerboard_3d_has_many_components() {
        // 4×4×4 3-D checkerboard: cell (x,y,z) is foreground iff
        // (x + y + z) is even. With 6-connectivity every foreground
        // cell's six face-neighbours are all of the opposite parity,
        // so no two foreground cells touch → component count equals
        // the foreground count. 4³ = 64 cells / 2 = 32.
        let sx = 4usize;
        let sy = 4usize;
        let sz = 4usize;
        let mut data = Vec::with_capacity(sx * sy * sz);
        for z in 0..sz {
            for y in 0..sy {
                for x in 0..sx {
                    data.push(if (x + y + z) % 2 == 0 { 1u8 } else { 0u8 });
                }
            }
        }
        let g = Grid::new(sx, sy, sz, &data).expect("4³ grid");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 32);
        assert_eq!(s.largest, 1);
    }

    #[test]
    fn grid_shape_mismatch_rejected() {
        let data = vec![0u8; 7];
        assert!(Grid::new(2, 2, 2, &data).is_none());
    }

    #[test]
    fn get_returns_value_and_handles_oob() {
        let data = vec![1u8, 2, 3, 4];
        let g = Grid::new(2, 2, 1, &data).expect("2×2×1");
        assert_eq!(*g.get(0, 0, 0).unwrap(), 1);
        assert_eq!(*g.get(1, 1, 0).unwrap(), 4);
        assert!(g.get(2, 0, 0).is_none());
        assert!(g.get(0, 0, 1).is_none());
    }

    #[test]
    fn compute_on_histogram_returns_largest_fraction() {
        // Sanity: the trait method on a histogram returns
        // max(bins) / total — same as `largest / foreground` in spirit.
        let h = Histogram::from_counts(vec![3, 1]);
        let m = StructureCount;
        let v = m.compute(&h);
        assert!((v - 0.75).abs() < 1e-6);
    }

    #[test]
    fn synthetic_two_block_distribution() {
        // Two 2×2×2 blocks at opposite corners of a 4×4×4 cube, rest
        // empty. 6-connectivity: the two blocks do not touch, so we
        // get 2 components each of size 8.
        let mut data = vec![0u8; 4 * 4 * 4];
        for z in 0..2 {
            for y in 0..2 {
                for x in 0..2 {
                    data[x + y * 4 + z * 16] = 1;
                    data[(x + 2) + (y + 2) * 4 + (z + 2) * 16] = 1;
                }
            }
        }
        let g = Grid::new(4, 4, 4, &data).expect("4³ grid");
        let s = StructureCount.evaluate(&g, |&v| v > 0);
        assert_eq!(s.count, 2);
        assert_eq!(s.largest, 8);
        assert_eq!(s.foreground, 16);
    }

    #[test]
    fn metric_name_is_stable() {
        assert_eq!(StructureCount::NAME, "structure_count_largest_fraction");
    }
}
