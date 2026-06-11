//! Deterministic ring iterators.
//!
//! Used by the streaming layer (and the follow-up slice that wires
//! [`crate::window::WindowPolicy`] into [`crate::stream::StreamingWorld`])
//! to enumerate the chunks in a given ring around an anchor. The order
//! is **stable** so replay produces the same mesh build / load order
//! given the same `(anchor, policy)`.

use phenotype_voxel::ChunkCoord;

/// Iterator over chunks in a given ring at Chebyshev distance
/// `ring` from `anchor`, with a vertical weight `vy_weight` applied
/// to `Δy` (so the ring is a "weighted Chebyshev sphere").
///
/// Order: lexicographic over `(dx, dy, dz)`, so two iterators with the
/// same inputs always produce the same sequence — replay-safe.
pub struct RingIter {
    anchor: ChunkCoord,
    /// Ring radius (weighted Chebyshev distance).
    ring: u32,
    /// Vertical weight (mirrors `WindowPolicy::vy_weight`).
    vy_weight: u8,
    /// Current `dx` (in `-ring..=ring`).
    dx: i32,
    /// Current `dy` (in `-ring..=ring`).
    dy: i32,
    /// Current `dz` (in `-ring..=ring`).
    dz: i32,
    /// Exhausted.
    done: bool,
}

impl RingIter {
    /// Iterate chunks at exactly `ring` from `anchor`, applying
    /// `vy_weight` to `|Δy|`.
    #[must_use]
    pub const fn new(anchor: ChunkCoord, ring: u32, vy_weight: u8) -> Self {
        // Start at the lexicographically-first valid (dx, dy, dz) in
        // [-ring, ring]^3. We pre-compute dx0 = -ring (cast to i32;
        // the unsigned→signed cast is well-defined for `ring` values
        // up to i32::MAX, far above any sane chunk coord).
        let start = -(ring as i32);
        Self {
            anchor,
            ring,
            vy_weight,
            dx: start,
            dy: start,
            dz: start,
            done: false,
        }
    }
}

impl Iterator for RingIter {
    type Item = ChunkCoord;

    fn next(&mut self) -> Option<ChunkCoord> {
        // Weighted Chebyshev predicate (replicates the kernel of
        // `crate::window::ring_distance` so the iterator and the
        // distance function agree on the boundary).
        let w = if self.vy_weight == 0 {
            1u32
        } else {
            self.vy_weight as u32
        };
        while !self.done {
            let dx = self.dx.unsigned_abs();
            let dz = self.dz.unsigned_abs();
            let dy = self.dy.unsigned_abs() * w;
            // Element of THIS ring (not a smaller one): the max of
            // (dx, dz, dy) must equal `self.ring`.
            let m = dx.max(dz).max(dy);
            let item = if m == self.ring {
                Some(ChunkCoord {
                    cx: self.anchor.cx + self.dx,
                    cy: self.anchor.cy + self.dy,
                    cz: self.anchor.cz + self.dz,
                })
            } else {
                None
            };
            // Advance (dx, dy, dz) lexicographically.
            self.dz += 1;
            if self.dz > self.ring as i32 {
                self.dz = -(self.ring as i32);
                self.dy += 1;
                if self.dy > self.ring as i32 {
                    self.dy = -(self.ring as i32);
                    self.dx += 1;
                    if self.dx > self.ring as i32 {
                        self.done = true;
                    }
                }
            }
            if item.is_some() {
                return item;
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn coord(cx: i32, cy: i32, cz: i32) -> ChunkCoord {
        ChunkCoord { cx, cy, cz }
    }

    /// Ring 0 is exactly the anchor chunk, iterated once.
    #[test]
    fn ring_zero_yields_anchor() {
        let anchor = coord(3, -1, 7);
        let items: Vec<ChunkCoord> = RingIter::new(anchor, 0, 2).collect();
        assert_eq!(items, vec![anchor]);
    }

    /// Ring 1 with `vy_weight = 1` is a 3×3×3 cube minus the 1×1×1
    /// interior (so 27 − 1 = 26 chunks).
    #[test]
    fn ring_one_chebyshev_cube_minus_interior() {
        let anchor = coord(0, 0, 0);
        let items: Vec<ChunkCoord> = RingIter::new(anchor, 1, 1).collect();
        assert_eq!(items.len(), 26);
        // Anchor is NOT in ring 1.
        assert!(!items.contains(&anchor));
    }

    /// Ring 1 with `vy_weight = 2` excludes every chunk whose weighted
    /// distance is 2: that's the 4 face-centers on the vertical axis
    /// (0, ±1, 0), all 8 corners (±1, ±1, ±1), and 8 of the 12
    /// edge-centers. The 4 edge-centers on the *horizontal* plane
    /// (±1, 0, ±1) stay in ring 1, giving 4 + 4 = 8 chunks.
    #[test]
    fn ring_one_vertical_weighted_drops_corners() {
        let anchor = coord(0, 0, 0);
        let items: Vec<ChunkCoord> = RingIter::new(anchor, 1, 2).collect();
        assert_eq!(
            items.len(),
            8,
            "with vy_weight=2, ring 1 has 8 chunks (4 horizontal face-centers + 4 horizontal edge-centers)"
        );
        // No chunk with |dy| * 2 + max(|dx|, |dz|) > 1.
        for c in &items {
            let ady = (c.cy - anchor.cy).unsigned_abs();
            let adx = (c.cx - anchor.cx).unsigned_abs();
            let adz = (c.cz - anchor.cz).unsigned_abs();
            assert!(
                ady * 2 + adx.max(adz) <= 1,
                "chunk {c:?} is past ring 1 with vy_weight=2"
            );
        }
    }

    /// Iterating rings 0..=K covers exactly the (2K+1)³ chunk cube
    /// (Chebyshev) without duplicates and without gaps.
    #[test]
    fn rings_cover_full_cube_with_vy_weight_one() {
        let anchor = coord(0, 0, 0);
        let k = 3u32;
        let mut all: Vec<ChunkCoord> = Vec::new();
        for r in 0..=k {
            all.extend(RingIter::new(anchor, r, 1));
        }
        // (2*3+1)^3 = 343 chunks.
        assert_eq!(all.len(), (2 * k + 1).pow(3) as usize);
        // Sorted iteration is identity (no duplicates).
        let mut sorted = all.clone();
        sorted.sort_by_key(|c| (c.cx, c.cy, c.cz));
        let mut unique = sorted.clone();
        unique.dedup();
        assert_eq!(sorted.len(), unique.len(), "no duplicates");
    }

    /// The iterator order is stable (deterministic) for the same inputs.
    #[test]
    fn iterator_order_is_deterministic() {
        let anchor = coord(1, -1, 2);
        let a: Vec<ChunkCoord> = RingIter::new(anchor, 2, 1).collect();
        let b: Vec<ChunkCoord> = RingIter::new(anchor, 2, 1).collect();
        assert_eq!(a, b, "two iterators with the same inputs must match");
    }

    /// Iterating a ring twice and concatenating gives 2× the count,
    /// and reordering one of the runs into the same lex order keeps
    /// the per-ring order stable.
    #[test]
    fn ring_order_is_lexicographic() {
        let anchor = coord(0, 0, 0);
        let items: Vec<ChunkCoord> = RingIter::new(anchor, 1, 1).collect();
        let mut expected: Vec<ChunkCoord> = items
            .iter()
            .copied()
            .map(|c| (c.cx, c.cy, c.cz))
            .map(|(cx, cy, cz)| ChunkCoord { cx, cy, cz })
            .collect();
        expected.sort_by_key(|c| (c.cx, c.cy, c.cz));
        assert_eq!(items, expected);
    }
}
