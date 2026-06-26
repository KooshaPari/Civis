//! FR-CIV-WARFARE-002 — Factions evolve combat doctrine from real battle outcomes.

use rand_chacha::ChaCha8Rng;

use crate::{
    evolve_doctrine, score_doctrine_fitness, CombatEngagement, DoctrineLibrary,
    FactionEngagementStats,
};

/// Accumulate per-faction engagement statistics from a slice of combat engagements.
pub fn accumulate_faction_stats(
    faction_id: u32,
    engagements: &[CombatEngagement],
) -> FactionEngagementStats {
    let mut stats = FactionEngagementStats {
        engagements_as_shooter: 0,
        engagements_as_target: 0,
        voxels_removed: 0,
    };
    for e in engagements {
        if e.shooter_faction == faction_id {
            stats.engagements_as_shooter += 1;
            // Approximate voxels removed from the blast radius
            stats.voxels_removed += u32::from(e.damage.radius_voxels);
        }
        if e.target_faction == faction_id {
            stats.engagements_as_target += 1;
        }
    }
    stats
}

/// Re-score and evolve a faction's doctrine library using real battle outcomes.
///
/// 1. Accumulates engagement stats for `faction_id` from `engagements`.
/// 2. Re-scores every doctrine candidate using those stats.
/// 3. Runs one GA generation via [`evolve_doctrine`].
pub fn evolve_doctrine_from_battle(
    library: &mut DoctrineLibrary,
    faction_id: u32,
    engagements: &[CombatEngagement],
    rng: &mut ChaCha8Rng,
    mutation_rate: f32,
) {
    let stats = accumulate_faction_stats(faction_id, engagements);
    for doctrine in &mut library.current {
        doctrine.score = score_doctrine_fitness(doctrine, &stats);
    }
    evolve_doctrine(library, rng, mutation_rate);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Doctrine, DoctrineLibrary, DamageEvent};
    use civ_voxel::WorldCoord;
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    fn dummy_engagement(shooter: u32, target: u32, radius: u8) -> CombatEngagement {
        CombatEngagement {
            shooter_id: shooter as u64,
            target_id: target as u64,
            shooter_faction: shooter,
            target_faction: target,
            damage: DamageEvent {
                center: WorldCoord { x: 0, y: 0, z: 0 },
                radius_voxels: radius,
                energy: 100,
            },
        }
    }

    #[test]
    fn accumulate_stats_counts_shooter_and_target() {
        let engagements = vec![
            dummy_engagement(1, 2, 3),
            dummy_engagement(1, 2, 5),
            dummy_engagement(2, 1, 2),
        ];
        let stats = accumulate_faction_stats(1, &engagements);
        assert_eq!(stats.engagements_as_shooter, 2);
        assert_eq!(stats.engagements_as_target, 1);
        assert_eq!(stats.voxels_removed, 3 + 5);
    }

    #[test]
    fn accumulate_stats_empty_engagements() {
        let stats = accumulate_faction_stats(1, &[]);
        assert_eq!(stats.engagements_as_shooter, 0);
        assert_eq!(stats.engagements_as_target, 0);
        assert_eq!(stats.voxels_removed, 0);
    }

    #[test]
    fn evolve_doctrine_from_battle_bumps_generation() {
        let mut library = DoctrineLibrary {
            current: vec![
                Doctrine { id: 1, unit_composition: vec![4, 4, 4], score: 0.0 },
                Doctrine { id: 2, unit_composition: vec![1, 8, 1], score: 0.0 },
            ],
            generation: 0,
        };
        let engagements = vec![dummy_engagement(7, 8, 3)];
        let mut r = rng(99);
        evolve_doctrine_from_battle(&mut library, 7, &engagements, &mut r, 0.1);
        assert_eq!(library.generation, 1);
    }

    #[test]
    fn evolve_doctrine_from_battle_deterministic() {
        let make_lib = || DoctrineLibrary {
            current: vec![
                Doctrine { id: 1, unit_composition: vec![2, 2, 2], score: 0.0 },
                Doctrine { id: 2, unit_composition: vec![5, 1, 3], score: 0.0 },
            ],
            generation: 0,
        };
        let engagements = vec![dummy_engagement(1, 2, 4)];
        let mut lib1 = make_lib();
        let mut lib2 = make_lib();
        evolve_doctrine_from_battle(&mut lib1, 1, &engagements, &mut rng(7), 0.2);
        evolve_doctrine_from_battle(&mut lib2, 1, &engagements, &mut rng(7), 0.2);
        assert_eq!(lib1, lib2);
    }
}
