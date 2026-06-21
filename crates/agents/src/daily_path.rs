//! Daily path / POI utility AI for civ-agents.
//!
//! This module implements a deterministic, greedy POI selector for the most
//! pressing need and a simple fixed-point movement step toward the chosen
//! target. A fuller pathfinding system can replace the stepping logic later
//! behind the same `path_step` signature.

use civ_needs::{NeedKind, Needs as LifeNeeds};
use civ_voxel::{WorldCoord, FIXED_SCALE};
use serde::{Deserialize, Serialize};

use crate::Position3d;

/// Point-of-interest categories used by utility AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PoiKind {
    /// Food source mapped to [`NeedKind::Food`].
    FoodSource,
    /// Water source mapped to [`NeedKind::Water`].
    WaterSource,
    /// Shelter mapped to [`NeedKind::Rest`].
    Shelter,
    /// Safe area mapped to [`NeedKind::Safety`].
    SafeZone,
    /// Social location mapped to [`NeedKind::Social`].
    SocialHub,
    /// Clinic mapped to [`NeedKind::Health`].
    Clinic,
}

/// A point of interest agents can target to satisfy a need.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Poi {
    /// Stable POI identifier.
    pub id: u64,
    /// POI category.
    pub kind: PoiKind,
    /// Fixed-point world position.
    pub pos: Position3d,
    /// Soft capacity used by higher-level systems.
    pub capacity: u32,
}

/// Vec-backed POI registry with deterministic iteration order.
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct PoiRegistry {
    pois: Vec<Poi>,
}

impl PoiRegistry {
    /// Add a POI to the registry.
    pub fn add(&mut self, poi: Poi) {
        self.pois.push(poi);
    }

    /// Check whether the registry currently contains any POIs.
    pub fn is_empty(&self) -> bool {
        self.pois.is_empty()
    }

    /// Iterate over registered POIs in insertion order.
    pub fn iter(&self) -> impl Iterator<Item = &Poi> {
        self.pois.iter()
    }

    /// Find the nearest POI of a given kind using squared fixed-point distance.
    #[must_use]
    pub fn nearest_of_kind(&self, kind: PoiKind, from: &Position3d) -> Option<&Poi> {
        self.pois
            .iter()
            .filter(|poi| poi.kind == kind)
            .min_by(|a, b| {
                let da = dist_sq(from, &a.pos);
                let db = dist_sq(from, &b.pos);
                da.cmp(&db).then_with(|| a.id.cmp(&b.id))
            })
    }
}

/// Map a life-need to the corresponding POI kind.
#[must_use]
pub fn poi_kind_for_need(kind: NeedKind) -> PoiKind {
    match kind {
        NeedKind::Food => PoiKind::FoodSource,
        NeedKind::Water => PoiKind::WaterSource,
        NeedKind::Rest => PoiKind::Shelter,
        NeedKind::Safety => PoiKind::SafeZone,
        NeedKind::Social => PoiKind::SocialHub,
        NeedKind::Health => PoiKind::Clinic,
    }
}

/// Map a POI kind back to the life-need it serves.
#[must_use]
pub fn need_for_poi_kind(kind: PoiKind) -> NeedKind {
    match kind {
        PoiKind::FoodSource => NeedKind::Food,
        PoiKind::WaterSource => NeedKind::Water,
        PoiKind::Shelter => NeedKind::Rest,
        PoiKind::SafeZone => NeedKind::Safety,
        PoiKind::SocialHub => NeedKind::Social,
        PoiKind::Clinic => NeedKind::Health,
    }
}

fn dist_sq(a: &Position3d, b: &Position3d) -> i128 {
    let dx = a.coord.x as i128 - b.coord.x as i128;
    let dy = a.coord.y as i128 - b.coord.y as i128;
    let dz = a.coord.z as i128 - b.coord.z as i128;
    dx * dx + dy * dy + dz * dz
}

fn need_pressure(needs: &LifeNeeds, kind: NeedKind) -> f32 {
    1.0 - needs.get(kind)
}

/// Weight of the distance tiebreak in [`score_poi`].
///
/// Need pressure ranges over `[0, 1]`; the distance term is a normalized linear
/// distance (also roughly `[0, 1]` across the playable map) scaled by this small
/// weight so that the *most-pressing need always dominates* target selection and
/// distance only breaks ties between comparably-pressing needs. This is what
/// makes a consistent eat→rest→socialize daily routine emerge rather than agents
/// myopically chasing whatever POI is nearest.
pub const DISTANCE_WEIGHT: f32 = 0.05;

/// Score a POI for the given needs snapshot and distance.
///
/// Higher scores are better. The score is the pressure of the need served by the
/// POI minus a small, deterministic distance tiebreak (see [`DISTANCE_WEIGHT`]).
#[must_use]
pub fn score_poi(needs: &LifeNeeds, poi: &Poi, dist_sq: i128) -> f32 {
    let pressure = need_pressure(needs, need_for_poi_kind(poi.kind));
    // Linear normalized distance: sqrt(dist_sq) / FIXED_SCALE, in i128 then f32.
    let dist = (dist_sq as f64).sqrt() as f32 / FIXED_SCALE as f32;
    pressure - DISTANCE_WEIGHT * dist
}

/// Pick the best POI across all categories for the current need pressures.
///
/// This selects the nearest POI of each kind, scores each candidate, and
/// returns the highest-scoring target. Ties are broken deterministically by
/// lower POI id.
#[must_use]
pub fn pick_target<'a>(
    needs: &LifeNeeds,
    registry: &'a PoiRegistry,
    from: &Position3d,
) -> Option<&'a Poi> {
    let mut best: Option<(&Poi, f32)> = None;
    for kind in [
        PoiKind::FoodSource,
        PoiKind::WaterSource,
        PoiKind::Shelter,
        PoiKind::SafeZone,
        PoiKind::SocialHub,
        PoiKind::Clinic,
    ] {
        let poi = match registry.nearest_of_kind(kind, from) {
            Some(poi) => poi,
            None => continue,
        };
        let score = score_poi(needs, poi, dist_sq(from, &poi.pos));
        match best {
            None => best = Some((poi, score)),
            Some((best_poi, best_score)) => {
                if score > best_score || (score == best_score && poi.id < best_poi.id) {
                    best = Some((poi, score));
                }
            }
        }
    }
    best.map(|(poi, _)| poi)
}

/// One deterministic greedy step toward a target position.
///
/// This is a fixed-point approximation of flow-field / A* movement and can be
/// replaced later behind this signature by a fuller path planner.
#[must_use]
pub fn path_step(from: &Position3d, to: &Position3d, speed_fp: i64) -> Position3d {
    if speed_fp <= 0 {
        return *from;
    }

    let dx = to.coord.x - from.coord.x;
    let dy = to.coord.y - from.coord.y;
    let dz = to.coord.z - from.coord.z;
    if dx == 0 && dy == 0 && dz == 0 {
        return *from;
    }

    let max_component = dx.abs().max(dy.abs()).max(dz.abs()).max(1);
    let step = speed_fp.min(max_component);

    let scale = |delta: i64| -> i64 {
        if delta == 0 {
            0
        } else {
            let signed = delta as i128 * step as i128 / max_component as i128;
            signed as i64
        }
    };

    let mut next = Position3d {
        coord: WorldCoord {
            x: (from.coord.x as i128 + scale(dx) as i128).clamp(i64::MIN as i128, i64::MAX as i128)
                as i64,
            y: (from.coord.y as i128 + scale(dy) as i128).clamp(i64::MIN as i128, i64::MAX as i128)
                as i64,
            z: (from.coord.z as i128 + scale(dz) as i128).clamp(i64::MIN as i128, i64::MAX as i128)
                as i64,
        },
    };

    if (dx > 0 && next.coord.x > to.coord.x) || (dx < 0 && next.coord.x < to.coord.x) {
        next.coord.x = to.coord.x;
    }
    if (dy > 0 && next.coord.y > to.coord.y) || (dy < 0 && next.coord.y < to.coord.y) {
        next.coord.y = to.coord.y;
    }
    if (dz > 0 && next.coord.z > to.coord.z) || (dz < 0 && next.coord.z < to.coord.z) {
        next.coord.z = to.coord.z;
    }

    next
}

/// Daily objective chosen by utility AI.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DailyGoal {
    /// Target POI identifier.
    pub target_poi: u64,
    /// POI kind being pursued.
    pub kind: PoiKind,
}

/// Coarse activity state for civilian life simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Activity {
    /// Stay put when needs are comfortable and no local motion is needed.
    Idle,
    /// Local deterministic wandering around a seed/home/current anchor.
    Wander,
    /// Purposeful trip to a POI that satisfies a pressing need.
    SeekNeed,
}

/// Minimum pressure that makes a need worth actively seeking.
pub const PRESSURE_THRESHOLD: f32 = 0.35;

/// Comfort threshold above which the agent prefers idle/wander behavior.
pub const COMFORT_THRESHOLD: f32 = 0.7;

/// Local wander radius measured in fixed-point units.
pub const WANDER_RADIUS: i64 = (0.06 * FIXED_SCALE as f32) as i64;

/// Deterministically choose a coarse activity state from the current needs.
#[must_use]
pub fn choose_activity(needs: &LifeNeeds, has_poi: bool) -> Activity {
    let pressure = [
        NeedKind::Food,
        NeedKind::Water,
        NeedKind::Rest,
        NeedKind::Safety,
        NeedKind::Social,
        NeedKind::Health,
    ]
    .into_iter()
    .map(|kind| need_pressure(needs, kind))
    .fold(0.0_f32, f32::max);

    if pressure <= (1.0 - COMFORT_THRESHOLD) {
        return Activity::Idle;
    }
    if pressure >= PRESSURE_THRESHOLD && has_poi {
        Activity::SeekNeed
    } else {
        Activity::Wander
    }
}

/// Build a deterministic local wander anchor from a seed and current position.
#[must_use]
pub fn wander_anchor(from: &Position3d, seed: u64, tick: u64) -> Position3d {
    let mix =
        seed ^ tick.rotate_left(17) ^ (from.coord.x as u64).rotate_left(7) ^ (from.coord.z as u64);
    let offset = |shift: u32| -> i64 {
        let bits = ((mix >> shift) & 0x3f) as i64;
        bits - 31
    };
    Position3d {
        coord: WorldCoord {
            x: from.coord.x + offset(0) * WANDER_RADIUS / 31,
            y: from.coord.y,
            z: from.coord.z + offset(6) * WANDER_RADIUS / 31,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn pos(x: i64, y: i64, z: i64) -> Position3d {
        Position3d {
            coord: WorldCoord { x, y, z },
        }
    }

    /// Covers FR-CIV-LIFE-010 — nearest_of_kind returns the closest matching POI.
    #[test]
    fn nearest_of_kind_picks_closest() {
        let mut registry = PoiRegistry::default();
        registry.add(Poi {
            id: 2,
            kind: PoiKind::WaterSource,
            pos: pos(10 * FIXED_SCALE, 0, 0),
            capacity: 1,
        });
        registry.add(Poi {
            id: 1,
            kind: PoiKind::WaterSource,
            pos: pos(3 * FIXED_SCALE, 0, 0),
            capacity: 1,
        });

        let nearest = registry
            .nearest_of_kind(PoiKind::WaterSource, &pos(0, 0, 0))
            .expect("water poi");
        assert_eq!(nearest.id, 1);
    }

    /// Covers FR-CIV-LIFE-011 — pick_target chooses the highest-pressure need.
    #[test]
    fn pick_target_chooses_highest_pressure_need() {
        let needs = LifeNeeds {
            food: 0.9,
            water: 0.1,
            rest: 0.8,
            safety: 0.7,
            social: 0.95,
            health: 0.9,
        };
        let mut registry = PoiRegistry::default();
        registry.add(Poi {
            id: 10,
            kind: PoiKind::FoodSource,
            pos: pos(FIXED_SCALE, 0, 0),
            capacity: 1,
        });
        registry.add(Poi {
            id: 20,
            kind: PoiKind::WaterSource,
            pos: pos(2 * FIXED_SCALE, 0, 0),
            capacity: 1,
        });
        registry.add(Poi {
            id: 30,
            kind: PoiKind::Shelter,
            pos: pos(3 * FIXED_SCALE, 0, 0),
            capacity: 1,
        });

        let target = pick_target(&needs, &registry, &pos(0, 0, 0)).expect("target");
        assert_eq!(target.kind, PoiKind::WaterSource);
        assert_eq!(target.id, 20);
    }

    /// Covers FR-CIV-LIFE-012 — path_step moves toward the target and never overshoots.
    #[test]
    fn path_step_moves_toward_target_without_overshoot() {
        let from = pos(0, 0, 0);
        let to = pos(10 * FIXED_SCALE, 0, 0);
        let step = path_step(&from, &to, 3 * FIXED_SCALE);
        assert!(step.coord.x > from.coord.x);
        assert!(step.coord.x <= to.coord.x);
        let final_step = path_step(&pos(9 * FIXED_SCALE, 0, 0), &to, 3 * FIXED_SCALE);
        assert_eq!(final_step.coord.x, to.coord.x);
    }

    /// Covers FR-CIV-LIFE-013 — scoring is deterministic for the same inputs.
    #[test]
    fn scoring_is_deterministic() {
        let needs = LifeNeeds {
            food: 0.4,
            water: 0.2,
            rest: 0.8,
            safety: 0.7,
            social: 0.6,
            health: 0.9,
        };
        let poi = Poi {
            id: 7,
            kind: PoiKind::WaterSource,
            pos: pos(2 * FIXED_SCALE, 0, 0),
            capacity: 1,
        };
        let d = dist_sq(&pos(0, 0, 0), &poi.pos);
        assert_eq!(
            score_poi(&needs, &poi, d).to_bits(),
            score_poi(&needs, &poi, d).to_bits()
        );
    }

    /// Covers FR-CIV-LIFE-014 — empty registry yields no target.
    #[test]
    fn empty_registry_returns_none() {
        let needs = LifeNeeds::sated();
        let registry = PoiRegistry::default();
        assert!(pick_target(&needs, &registry, &pos(0, 0, 0)).is_none());
    }

    /// Covers FR-CIV-LIFE-014a — `PoiRegistry::is_empty` tracks inserted POIs.
    #[test]
    fn poi_registry_is_empty_tracks_insertions() {
        let mut registry = PoiRegistry::default();
        assert!(registry.is_empty());

        registry.add(Poi {
            id: 1,
            kind: PoiKind::FoodSource,
            pos: pos(0, 0, 0),
            capacity: 1,
        });

        assert!(!registry.is_empty());
    }

    /// Covers FR-CIV-LIFE-015 — satisfied needs prefer idle/wander over seek.
    #[test]
    fn satisfied_needs_do_not_seek() {
        let needs = LifeNeeds::sated();
        assert_eq!(choose_activity(&needs, false), Activity::Idle);
        assert_eq!(choose_activity(&needs, true), Activity::Idle);
    }

    /// Covers FR-CIV-LIFE-016 — wander anchors remain local and deterministic.
    #[test]
    fn wander_anchor_stays_local() {
        let from = pos(100 * FIXED_SCALE, 0, -50 * FIXED_SCALE);
        let a = wander_anchor(&from, 42, 7);
        let b = wander_anchor(&from, 42, 7);
        assert_eq!(a, b);
        let dx = (a.coord.x - from.coord.x).abs();
        let dz = (a.coord.z - from.coord.z).abs();
        assert!(dx <= WANDER_RADIUS);
        assert!(dz <= WANDER_RADIUS);
    }
}
