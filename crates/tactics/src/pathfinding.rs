//! Deterministic grid pathfinding for the operational layer (FR-CIV-TACTICS-033/036).

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
}
