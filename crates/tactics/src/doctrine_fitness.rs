//! Doctrine fitness from engagement outcomes (FR-CIV-TACTICS-023).

use crate::Doctrine;

/// Per-faction combat stats accumulated during the last war-bridge cadence.
// The following 1 requirement tags were removed from FactionEngagementStats.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// This is the site a mid-audit coordination note reported as already removed, asserting that no tag remained in source. That was not accurate when checked: crates/tactics/src/doctrine_fitness.rs:6 still reads '// FR-CIV-WAR-030' directly above 'pub struct FactionEngagementStats', re-verified with git grep and a raw line dump after the note arrived. The removal entry is therefore written, because the tag is present and the claim is false. The same module carries a second FR-CIV-WAR-030 tag at crates/tactics/src/doctrine_fitness.rs:27, above pub fn score_doctrine_fitness, which is likewise still present.
// The two FR-CIV-WAR-030 tags inside this module are the mirror image of the FR-CIV-WAR-021 pair: here the pair agrees outright. score_doctrine_fitness computes precisely the tactical-only signal FR-CIV-WAR-030 asks to extend, so tagging the function with the extension requirement asserts the opposite of what it does, and tagging the stats struct it consumes with the same id is equally unfounded. Both tags overstate coverage of a requirement the spec itself marks as not yet coded.
// Only the FactionEngagementStats entry is in SITES here. The score_doctrine_fitness tag at crates/tactics/src/doctrine_fitness.rs:27 was not in the assigned nine, so it is reported rather than removed; it is a false binding of the same kind and should be added to scope.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-WAR-030: CONTAINER-ONLY. Same requirement as the tag on Doctrine: extend the doctrine fitness signal beyond tactical engagement stats to include operational and strategic outcomes -- net theater objective gain, supply efficiency, own attrition and routs, civilian grievance generated -- so doctrines that win battles but lose the war are selected against (docs/design/warfare.md:124-140, AC-WAR-7 at :178). The spec states the status plainly: 'Doctrine fitness today reads tactical engagement stats only. Extend the fitness signal (spec; not yet coded)' at docs/design/warfare.md:125, and §4.1 at :122 records the existing mechanism as shipped under FR-CIV-TACTICS-023. FactionEngagementStats at crates/tactics/src/doctrine_fitness.rs:8 is the worst possible carrier for this id: its three fields are exactly the tactical inputs the spec names as the starting point to be moved past (engagements_as_shooter, engagements_as_target, voxels_removed), and it has no field for any of the four operational or strategic terms. Its only method, net_pressure at crates/tactics/src/doctrine_fitness.rs:19, subtracts target engagements from shooter engagements -- the definition of tactical-only. git grep -n -i -E 'k_terr|k_supl|k_loss|k_grv|civilian_grievance|own_attrition|memetic|diffusion' -- crates/ returned no hits in the tactics crate, and this module has exactly two public functions, net_pressure and score_doctrine_fitness, with no per-cluster selection and no doctrine-copying path. The test at crates/tactics/tests/fr_fr_civ_war_030.rs asserts only that higher engagement stats raise fitness and that scoring is deterministic, which confirms the pre-extension tactical behavior the requirement says to extend.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct FactionEngagementStats {
    /// Engagements where this faction had the shooter role.
    pub engagements_as_shooter: u32,
    /// Engagements where this faction was targeted.
    pub engagements_as_target: u32,
    /// Voxels removed by this faction's queued damage events.
    pub voxels_removed: u32,
}

impl FactionEngagementStats {
    /// Net engagement pressure (shooter minus target).
    pub fn net_pressure(&self) -> i32 {
        self.engagements_as_shooter as i32 - self.engagements_as_target as i32
    }
}

/// Re-score a doctrine from composition balance plus live engagement stats.
///
/// Deterministic for fixed inputs; used immediately before [`crate::evolve_doctrine`].
// The following 1 requirement tag was removed from score_doctrine_fitness.
// It is not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// Adjudicated as a function, not as a data container, at the coordinator's request. Being a function makes it a more plausible candidate for a behavioral requirement than any struct in this batch, and it is still false: it is the pre-extension fitness function the requirement is written to change, not an implementation of the change.
// All three FR-CIV-WAR-030 sites are now adjudicated and all three are false, so this id disagrees with itself nowhere: crates/tactics/src/lib.rs:106 on Doctrine, crates/tactics/src/doctrine_fitness.rs:6 on FactionEngagementStats, and crates/tactics/src/doctrine_fitness.rs:27 here. The requirement is real and specified, but the extension is unimplemented, so every tag asserting it is overstating coverage. Whoever implements the four extra terms can re-add the tag to this function, which is where it belongs.
// find_decl in _apply_verdicts.py:75-85 matches 'pub fn' as well as struct, enum, const, static, type and trait, and the tag at :27 sits directly above this pub fn with no intervening doc line, so this site is mechanically removable in the same way as the struct sites.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-WAR-030: DATA-SHAPE-ONLY, and in this case the function's own body is the clearest evidence that the tag is false. The requirement is to extend the doctrine fitness signal beyond tactical engagement stats to include operational and strategic outcomes -- net theater objective gain, supply efficiency, own attrition and routs, civilian grievance generated -- so that doctrines which win battles but lose the war are selected against, with per-cluster independent libraries and memetic diffusion across contact networks (docs/design/warfare.md:124-140, AC-WAR-7 at :178). The coordinator's hypothesis that a function might be the genuine binding where the struct is not does not survive reading it. score_doctrine_fitness at crates/tactics/src/doctrine_fitness.rs:28 takes (doctrine: &Doctrine, stats: &FactionEngagementStats) and returns the sum of exactly two terms: composition_balance, the mean of doctrine.unit_composition, and a 'battle' term computed solely from stats.net_pressure(), stats.engagements_as_shooter and stats.voxels_removed (:29-39). Every input is a tactical quantity. The function has no parameter through which an operational or strategic outcome could enter: no objective gain, no supply state, no attrition, no rout count, no grievance term exists on either argument. That is the definition of the pre-extension state the spec describes at docs/design/warfare.md:125, 'Doctrine fitness today reads tactical engagement stats only. Extend the fitness signal (spec; not yet coded)'. Tagging this function with FR-CIV-WAR-030 therefore asserts the exact opposite of what it does: it is the code the requirement says must be extended, and it has not been. Corroborating searches: git grep -n -i -E 'k_terr|k_supl|k_loss|k_grv|civilian_grievance|own_attrition|memetic|diffusion' -- crates/ returns no hit in the tactics crate (the only diffusion matches are unrelated civ-agents tech/wardrobe propagation); the module defines only two public functions, net_pressure (:19) and this one (:28), so there is no second scoring path elsewhere; and git grep -n 'score_doctrine_fitness' -- crates/ shows the only production call sites are crates/engine/src/engine/military_phases.rs:95 and crates/tactics/src/doctrine_evolution.rs:47, both of which pass nothing but a &Doctrine and a &FactionEngagementStats, so no operational or strategic input is available anywhere on the call path. The remaining references are re-exports and tests, including crates/tactics/tests/fr_fr_civ_war_030.rs, which asserts only that higher engagement stats raise fitness and that scoring is deterministic -- precisely the tactical-only behavior, never the extension.
pub fn score_doctrine_fitness(doctrine: &Doctrine, stats: &FactionEngagementStats) -> f32 {
    let composition_sum: u32 = doctrine
        .unit_composition
        .iter()
        .map(|&c| u32::from(c))
        .sum();
    let composition_balance =
        composition_sum as f32 / doctrine.unit_composition.len().max(1) as f32;
    let battle = stats.net_pressure() as f32 * 3.0
        + stats.engagements_as_shooter as f32 * 1.5
        + stats.voxels_removed as f32 * 0.25;
    composition_balance + battle
}
