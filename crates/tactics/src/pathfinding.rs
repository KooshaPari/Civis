//! Deterministic grid pathfinding for the operational layer (FR-CIV-TACTICS-033/036).

use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// Cardinal neighbors in stable order (E, N, W, S) for reproducible BFS expansion.
const NEIGHBORS: [(i32, i32); 4] = [(1, 0), (0, 1), (-1, 0), (0, -1)];

/// One step along a shortest path from `from` toward `to` on an unobstructed grid plane.
///
/// Returns `None` when already at the goal or no path exists within `max_search`.
#[must_use]
pub fn bfs_next_step(from: (i32, i32), to: (i32, i32), max_search: u32) -> Option<(i32, i32)> {
    bfs_next_step_with_blocked(from, to, max_search, &|_x, _y| false)
}

/// BFS next step with optional blocked cells (FR-CIV-TACTICS-036).
///
/// `blocked` is consulted for every neighbor except the start cell `from`.
#[must_use]
pub fn bfs_next_step_with_blocked(
    from: (i32, i32),
    to: (i32, i32),
    max_search: u32,
    blocked: &impl Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    if from == to {
        return None;
    }
    let max_search = max_search.max(1) as i32;
    let mut queue = std::collections::VecDeque::new();
    let mut visited = std::collections::BTreeSet::new();
    let mut parent: std::collections::BTreeMap<(i32, i32), (i32, i32)> =
        std::collections::BTreeMap::new();

    queue.push_back(from);
    visited.insert(from);

    while let Some(current) = queue.pop_front() {
        if current == to {
            break;
        }
        let depth = manhattan(from, current);
        if depth >= max_search {
            continue;
        }
        for (dx, dy) in NEIGHBORS {
            let next = (current.0 + dx, current.1 + dy);
            if visited.contains(&next) || blocked(next.0, next.1) {
                continue;
            }
            visited.insert(next);
            parent.insert(next, current);
            if next == to {
                queue.push_back(next);
                break;
            }
            queue.push_back(next);
        }
    }

    if !visited.contains(&to) {
        return greedy_step(from, to, blocked);
    }

    let mut cursor = to;
    while let Some(&prev) = parent.get(&cursor) {
        if prev == from {
            return Some(cursor);
        }
        cursor = prev;
    }
    None
}

fn manhattan(a: (i32, i32), b: (i32, i32)) -> i32 {
    (a.0 - b.0).abs() + (a.1 - b.1).abs()
}

// ---------------------------------------------------------------------------
// A* full-path search (FR-CIV-TACTICS-037)
// ---------------------------------------------------------------------------

/// A* obstacle-aware full-path search on the cardinal grid (FR-CIV-TACTICS-037).
///
/// Returns the full path from `from` to `goal` as an ordered `Vec` of grid
/// positions **including both endpoints**, or `None` when:
/// * `from == goal` (already at destination), or
/// * no path exists within `max_search` node expansions.
///
/// `blocked` follows the same convention as [`bfs_next_step_with_blocked`]: it is
/// consulted for every candidate cell *except* the start.  The start cell is
/// always treated as passable.
///
/// Uses the Manhattan-distance heuristic; ties are broken deterministically by
/// `(f, g, x, y)` ordering so the same start/goal pair always yields the same
/// path regardless of insertion order.
#[must_use]
pub fn astar_path(from: (i32, i32), goal: (i32, i32), max_search: u32) -> Option<Vec<(i32, i32)>> {
    astar_path_with_blocked(from, goal, max_search, &|_x, _y| false)
}

/// A* full-path search with an obstacle predicate (FR-CIV-TACTICS-037).
///
/// See [`astar_path`] for full documentation.  The `blocked` closure receives
/// the `(x, y)` grid coordinates of each candidate neighbour; returning `true`
/// marks the cell impassable.
#[must_use]
pub fn astar_path_with_blocked(
    from: (i32, i32),
    goal: (i32, i32),
    max_search: u32,
    blocked: &impl Fn(i32, i32) -> bool,
) -> Option<Vec<(i32, i32)>> {
    if from == goal {
        return None;
    }

    let max_nodes = max_search.max(1) as usize;

    // g_score: cost from start to node (cardinal steps, each cost 1).
    let mut g_score: std::collections::BTreeMap<(i32, i32), i32> =
        std::collections::BTreeMap::new();
    // came_from: predecessor map for path reconstruction.
    let mut came_from: std::collections::BTreeMap<(i32, i32), (i32, i32)> =
        std::collections::BTreeMap::new();

    // Open set: (Reverse(f), Reverse(g), x, y) so BinaryHeap gives min-f first;
    // ties broken by max-g (deeper node first), then by coordinate for stability.
    let mut open: BinaryHeap<(Reverse<i32>, Reverse<i32>, i32, i32)> = BinaryHeap::new();

    g_score.insert(from, 0);
    open.push((Reverse(manhattan(from, goal)), Reverse(0), from.0, from.1));

    let mut expansions = 0usize;

    while let Some((_, Reverse(g), cx, cy)) = open.pop() {
        let current = (cx, cy);

        if current == goal {
            return Some(reconstruct_path(&came_from, goal));
        }

        if expansions >= max_nodes {
            break;
        }
        expansions += 1;

        // Skip stale open-set entries (lazy deletion).
        if g_score.get(&current).copied().unwrap_or(i32::MAX) < g {
            continue;
        }

        for (dx, dy) in NEIGHBORS {
            let neighbour = (current.0 + dx, current.1 + dy);
            if neighbour != goal && blocked(neighbour.0, neighbour.1) {
                continue;
            }
            let tentative_g = g + 1;
            if tentative_g < g_score.get(&neighbour).copied().unwrap_or(i32::MAX) {
                g_score.insert(neighbour, tentative_g);
                came_from.insert(neighbour, current);
                let f = tentative_g + manhattan(neighbour, goal);
                open.push((Reverse(f), Reverse(tentative_g), neighbour.0, neighbour.1));
            }
        }
    }

    None
}

/// Reconstruct the ordered path by following `came_from` back to the start.
fn reconstruct_path(
    came_from: &std::collections::BTreeMap<(i32, i32), (i32, i32)>,
    goal: (i32, i32),
) -> Vec<(i32, i32)> {
    let mut path = Vec::new();
    let mut cursor = goal;
    loop {
        path.push(cursor);
        match came_from.get(&cursor) {
            Some(&prev) => cursor = prev,
            None => break,
        }
    }
    path.reverse();
    path
}

fn greedy_step(
    from: (i32, i32),
    to: (i32, i32),
    blocked: &impl Fn(i32, i32) -> bool,
) -> Option<(i32, i32)> {
    let dx = (to.0 - from.0).clamp(-1, 1);
    let dy = (to.1 - from.1).clamp(-1, 1);
    if dx == 0 && dy == 0 {
        return None;
    }
    let next = (from.0 + dx, from.1 + dy);
    if blocked(next.0, next.1) {
        None
    } else {
        Some(next)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bfs_next_step_moves_toward_target() {
        assert_eq!(bfs_next_step((0, 0), (3, 0), 16), Some((1, 0)));
        assert_eq!(bfs_next_step((0, 0), (0, 4), 16), Some((0, 1)));
    }

    #[test]
    fn bfs_next_step_is_deterministic_for_diagonal_targets() {
        let a = bfs_next_step((0, 0), (2, 2), 16);
        let b = bfs_next_step((0, 0), (2, 2), 16);
        assert_eq!(a, b);
        assert!(a.is_some());
    }

    #[test]
    fn bfs_next_step_none_at_goal() {
        assert_eq!(bfs_next_step((1, 1), (1, 1), 8), None);
    }

    #[test]
    fn bfs_avoids_blocked_cell() {
        let blocked = |x: i32, y: i32| x == 1 && y == 0;
        assert_eq!(
            bfs_next_step_with_blocked((0, 0), (3, 0), 16, &blocked),
            Some((0, 1))
        );
    }

    // -----------------------------------------------------------------------
    // FR-CIV-TACTICS-037 — A* full-path search
    // -----------------------------------------------------------------------

    /// Straight path on an open field returns all cells from start to goal.
    #[test]
    fn astar_open_field_straight_path() {
        let path = astar_path((0, 0), (4, 0), 64).expect("open field must find a path");
        // Path must start at origin and end at goal.
        assert_eq!(path.first().copied(), Some((0, 0)));
        assert_eq!(path.last().copied(), Some((4, 0)));
        // Each consecutive step must be a cardinal neighbour.
        for window in path.windows(2) {
            let (ax, ay) = window[0];
            let (bx, by) = window[1];
            assert_eq!((ax - bx).abs() + (ay - by).abs(), 1, "non-cardinal step");
        }
        // Optimal length on an open grid equals Manhattan distance + 1 (inclusive endpoints).
        assert_eq!(path.len(), 5);
    }

    /// Path routes around a wall of blocked cells.
    #[test]
    fn astar_routes_around_obstacle() {
        // Wall blocking x == 1 for y in [0, 1, 2].
        let blocked = |x: i32, y: i32| x == 1 && (0..=2).contains(&y);
        let path = astar_path_with_blocked((0, 0), (3, 0), 128, &blocked)
            .expect("path around obstacle must exist");
        assert_eq!(path.first().copied(), Some((0, 0)));
        assert_eq!(path.last().copied(), Some((3, 0)));
        // No cell in the path may be blocked.
        for &(x, y) in &path {
            assert!(!blocked(x, y), "path stepped through blocked cell ({x},{y})");
        }
        // Path must be connected.
        for window in path.windows(2) {
            let (ax, ay) = window[0];
            let (bx, by) = window[1];
            assert_eq!((ax - bx).abs() + (ay - by).abs(), 1);
        }
    }

    /// Returns `None` when the goal is completely surrounded by blocked cells.
    #[test]
    fn astar_no_path_when_fully_blocked() {
        // Goal at (2,2); surround it on all cardinal sides and seal the only
        // approach corridors with a solid ring.
        let blocked = |x: i32, y: i32| {
            matches!(
                (x, y),
                (2, 1) | (2, 3) | (1, 2) | (3, 2)
            )
        };
        let result = astar_path_with_blocked((0, 0), (2, 2), 256, &blocked);
        assert!(result.is_none(), "expected None for enclosed goal");
    }

    /// Returns `None` when start equals goal.
    #[test]
    fn astar_none_at_goal() {
        assert!(astar_path((5, 5), (5, 5), 32).is_none());
    }

    /// Path is deterministic: same inputs always produce the same result.
    #[test]
    fn astar_is_deterministic() {
        let blocked = |x: i32, y: i32| x == 2 && y < 3;
        let first = astar_path_with_blocked((0, 0), (4, 0), 128, &blocked);
        let second = astar_path_with_blocked((0, 0), (4, 0), 128, &blocked);
        assert_eq!(first, second);
        assert!(first.is_some());
    }

    /// Returns `None` when `max_search` is too small to reach the goal.
    #[test]
    fn astar_respects_max_search_budget() {
        // Goal is 10 steps away; budget of 3 node expansions is insufficient.
        let result = astar_path((0, 0), (10, 0), 3);
        assert!(result.is_none());
    }
}
