use civ_tactics::astar_path_with_blocked;
use civ_voxel::{MaterialId, VoxelWorld, WorldCoord};
use glam::IVec3;

/// Coarse path plan used by the daily movement system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DailyPath {
    /// Ordered route positions, including the start position.
    pub waypoints: Vec<IVec3>,
    /// Index of the most recently reached waypoint.
    pub current_idx: usize,
    /// True once the route has been fully consumed.
    pub completed: bool,
}

fn to_grid(pos: IVec3) -> (i32, i32) {
    (pos.x, pos.z)
}

fn lift(grid: (i32, i32), y: i32) -> IVec3 {
    IVec3::new(grid.0, y, grid.1)
}

fn blocked_at_height(world: &VoxelWorld<MaterialId>, x: i32, y: i32, z: i32) -> bool {
    world
        .read(WorldCoord {
            x: i64::from(x),
            y: i64::from(y),
            z: i64::from(z),
        })
        .0
        != 0
}

fn append_segment(
    route: &mut Vec<IVec3>,
    start: IVec3,
    goal: IVec3,
    world: &VoxelWorld<MaterialId>,
) -> bool {
    if start == goal {
        if route.last().copied() != Some(goal) {
            route.push(goal);
        }
        return true;
    }

    let start_grid = to_grid(start);
    let goal_grid = to_grid(goal);

    let raw = if start_grid == goal_grid {
        let mut vertical = Vec::new();
        let mut cursor = start;
        vertical.push(cursor);
        while cursor.y != goal.y {
            cursor.y += (goal.y - cursor.y).signum();
            vertical.push(cursor);
        }
        Some(vertical)
    } else {
        let blocked = |x: i32, z: i32| blocked_at_height(world, x, start.y, z);
        astar_path_with_blocked(start_grid, goal_grid, u32::MAX, &blocked).map(|points| {
            points
                .into_iter()
                .map(|p| lift(p, start.y))
                .collect::<Vec<_>>()
        })
    };

    let Some(mut segment) = raw else {
        return false;
    };

    if segment.first().copied() != Some(start) {
        segment.insert(0, start);
    }
    if segment.last().copied() != Some(goal) {
        segment.push(goal);
    }

    for point in segment.into_iter().skip(1) {
        if route.last().copied() != Some(point) {
            route.push(point);
        }
    }

    true
}

/// Plan a deterministic daily path across one or more goals.
#[must_use]
pub fn plan_daily_path(start: IVec3, goals: &[IVec3], world: &VoxelWorld<MaterialId>) -> DailyPath {
    let mut waypoints = vec![start];
    let mut cursor = start;
    let mut failed = false;

    for goal in goals {
        if cursor == *goal {
            continue;
        }
        if !append_segment(&mut waypoints, cursor, *goal, world) {
            failed = true;
            break;
        }
        cursor = *goal;
    }

    DailyPath {
        waypoints,
        current_idx: 0,
        completed: !failed && waypoints.len() <= 1,
    }
}

fn sync_current_index(path: &mut DailyPath, current_pos: IVec3) {
    if path.waypoints.is_empty() {
        path.current_idx = 0;
        path.completed = true;
        return;
    }

    if let Some(index) = path
        .waypoints
        .iter()
        .enumerate()
        .skip(path.current_idx)
        .find_map(|(index, &point)| (point == current_pos).then_some(index))
    {
        path.current_idx = index;
    }
}

/// Advance the path by consuming up to `step_size.floor()` waypoints.
#[must_use]
pub fn advance_path(path: &mut DailyPath, current_pos: IVec3, step_size: f32) -> Option<IVec3> {
    if path.completed || path.waypoints.len() < 2 || !step_size.is_finite() || step_size <= 0.0 {
        if path.waypoints.len() <= 1 {
            path.completed = true;
        }
        return None;
    }

    sync_current_index(path, current_pos);

    if path.current_idx + 1 >= path.waypoints.len() {
        path.completed = true;
        return None;
    }

    let mut steps = step_size.floor() as usize;
    if steps == 0 {
        return None;
    }

    let mut next_pos = current_pos;
    while steps > 0 && path.current_idx + 1 < path.waypoints.len() {
        path.current_idx += 1;
        next_pos = path.waypoints[path.current_idx];
        steps -= 1;
    }

    if path.current_idx + 1 >= path.waypoints.len() {
        path.completed = true;
    }

    Some(next_pos)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn world() -> VoxelWorld<MaterialId> {
        VoxelWorld::new(1)
    }

    fn solid(world: &mut VoxelWorld<MaterialId>, x: i32, y: i32, z: i32) {
        world.write(
            WorldCoord {
                x: i64::from(x),
                y: i64::from(y),
                z: i64::from(z),
            },
            MaterialId(1),
        );
    }

    #[test]
    fn plan_daily_path_chains_goals_and_routes_around_blocker() {
        let mut world = world();
        solid(&mut world, 1, 0, 0);

        let path = plan_daily_path(
            IVec3::new(0, 0, 0),
            &[IVec3::new(3, 0, 0), IVec3::new(3, 0, 2)],
            &world,
        );

        assert_eq!(path.waypoints.first().copied(), Some(IVec3::new(0, 0, 0)));
        assert_eq!(path.waypoints.last().copied(), Some(IVec3::new(3, 0, 2)));
        assert!(path.waypoints.iter().all(|p| *p != IVec3::new(1, 0, 0)));
        assert!(path.waypoints.iter().any(|p| *p == IVec3::new(0, 0, 1)));
        assert!(!path.completed);
    }

    #[test]
    fn plan_daily_path_supports_vertical_only_goals() {
        let world = world();
        let path = plan_daily_path(IVec3::new(2, 0, 2), &[IVec3::new(2, 3, 2)], &world);

        assert_eq!(
            path.waypoints,
            vec![
                IVec3::new(2, 0, 2),
                IVec3::new(2, 1, 2),
                IVec3::new(2, 2, 2),
                IVec3::new(2, 3, 2),
            ]
        );
        assert!(!path.completed);
    }

    #[test]
    fn advance_path_consumes_waypoints_and_marks_completion() {
        let world = world();
        let mut path = plan_daily_path(IVec3::new(0, 0, 0), &[IVec3::new(2, 0, 0)], &world);

        assert_eq!(
            advance_path(&mut path, IVec3::new(0, 0, 0), 2.0),
            Some(IVec3::new(2, 0, 0))
        );
        assert_eq!(path.current_idx, 2);
        assert!(path.completed);
        assert_eq!(advance_path(&mut path, IVec3::new(2, 0, 0), 1.0), None);
    }
}
