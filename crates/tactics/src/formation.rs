//! Unit formation offsets (FR-CIV-TACTICS-021).

/// Tactical formation layout for squad offsets on the hex/grid plane.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormationKind {
    /// Single-file along +X.
    Line,
    /// V-shaped advance (leader at front).
    Wedge,
    /// Compact block (pairs on X/Y).
    Square,
}

/// Grid offsets `(dx, dy)` for `slots` units anchored at the leader cell.
pub fn formation_offsets(kind: FormationKind, slots: usize) -> Vec<(i32, i32)> {
    if slots == 0 {
        return Vec::new();
    }
    match kind {
        FormationKind::Line => (0..slots)
            .map(|i| {
                let center = (slots as i32).saturating_sub(1) / 2;
                (i as i32 - center, 0)
            })
            .collect(),
        FormationKind::Wedge => {
            let mut out = Vec::with_capacity(slots);
            out.push((0, 0));
            let mut rank = 1i32;
            let mut placed = 1usize;
            while placed < slots {
                for side in [-1i32, 1i32] {
                    if placed >= slots {
                        break;
                    }
                    out.push((side * rank, rank));
                    placed += 1;
                }
                rank += 1;
            }
            out
        }
        FormationKind::Square => {
            let side = (slots as f64).sqrt().ceil() as i32;
            let mut out = Vec::with_capacity(slots);
            for row in 0..side {
                for col in 0..side {
                    if out.len() >= slots {
                        break;
                    }
                    out.push((col, row));
                }
            }
            out
        }
    }
}
