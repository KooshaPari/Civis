//! civ-agents — civilian agent ECS components + LOD tick + per-civilian
//! wardrobe / tools state.
//!
//! Components live in a shared `hecs::World`. This crate ships:
//!
//! - `Civilian` — identity + age + faction
//! - `Wardrobe` — current clothing era + material slot (diffusion-driven)
//! - `Tools` — current tool era + material slot (diffusion-driven)
//! - `Needs` — utility-AI scalar weights (food, shelter, safety, belonging)
//! - `LodTier::{Hot, Warm, Cold}` — simulation fidelity level
//! - `Position3d` — fixed-point world coordinates (composes with `civ-voxel`'s
//!   `WorldCoord`)
//!
//! Diffusion-driven propagation: `propagate_era` consults a tech-adoption
//! S-curve from `civ-diffusion` to decide whether to bump a civilian's
//! `wardrobe.era` or `tools.era` this tick. Per ADR-008 + the always-auto
//! determinism rules, propagation is pure / seeded — no time leaks.
//!
//! See `docs/development-guide/fr-3d-additions.md` for `FR-CIV-AGENTS-*`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use civ_diffusion::{advance as diffusion_advance, DiffusionParams};
use civ_voxel::{MaterialId, WorldCoord};
use hecs::World;
use rand::Rng;
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

/// Schema version. Bumped on breaking changes.
pub const SCHEMA_VERSION: u32 = 0;

/// Civilian identity component.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Civilian {
    /// Stable agent ID.
    pub id: u64,
    /// Faction this civilian belongs to.
    pub faction: u32,
    /// Age in years (game-time).
    pub age: u16,
}

/// Wardrobe state. The `era` is the civilian's currently worn-tech era;
/// `material` is the visible material slot the renderer uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Wardrobe {
    /// Era index of the currently-worn clothing.
    pub era: u16,
    /// Material slot driving renderer colour / texture.
    pub material: MaterialId,
}

/// Tool inventory state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tools {
    /// Era index of the currently-held tools.
    pub era: u16,
    /// Material slot.
    pub material: MaterialId,
}

/// Utility-AI scalar weights ∈ `[0, 1]`. Higher = more pressing need.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Needs {
    /// Food need.
    pub food: f32,
    /// Shelter need.
    pub shelter: f32,
    /// Safety need (combat/disease threat).
    pub safety: f32,
    /// Belonging need (social).
    pub belonging: f32,
}

/// Simulation fidelity tier. Far-from-camera civilians collapse to lower tiers
/// to bound the per-tick cost.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LodTier {
    /// Full fidelity — every tick.
    Hot,
    /// Reduced fidelity — every 4 ticks.
    Warm,
    /// Gestalt — every 16 ticks.
    Cold,
}

/// Fixed-point world position. Composes with `civ-voxel`'s `WorldCoord` at
/// `FIXED_SCALE`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position3d {
    /// `WorldCoord` from `civ-voxel`.
    pub coord: WorldCoord,
}

/// Utility-AI action priority derived from unmet needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NeedAction {
    /// Seek food.
    FindFood,
    /// Seek shelter.
    FindShelter,
    /// Escape danger.
    Flee,
    /// Seek social contact.
    Socialize,
    /// No urgent action.
    Idle,
}

/// Utility weights used when scoring unmet needs.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct UtilityWeights {
    /// Food weight.
    pub food: f32,
    /// Shelter weight.
    pub shelter: f32,
    /// Safety weight.
    pub safety: f32,
    /// Belonging weight.
    pub belonging: f32,
}

/// Spawn one civilian entity in a `hecs::World`.
pub fn spawn_civilian(
    world: &mut World,
    civilian: Civilian,
    position: Position3d,
    wardrobe: Wardrobe,
    tools: Tools,
    needs: Needs,
    lod: LodTier,
) -> hecs::Entity {
    world.spawn((civilian, position, wardrobe, tools, needs, lod))
}

/// Spawn a deterministic batch of civilians with sequential IDs.
pub fn spawn_many(
    world: &mut World,
    count: u32,
    seed_civilian_id: u64,
    faction: u32,
) -> Vec<hecs::Entity> {
    let mut entities = Vec::with_capacity(count as usize);
    for offset in 0..count {
        let civilian = Civilian {
            id: seed_civilian_id + u64::from(offset),
            faction,
            age: 18 + (offset % 50) as u16,
        };
        let position = Position3d {
            coord: WorldCoord { x: 0, y: 0, z: 0 },
        };
        let wardrobe = Wardrobe {
            era: 0,
            material: MaterialId(0),
        };
        let tools = Tools {
            era: 0,
            material: MaterialId(0),
        };
        let needs = Needs {
            food: 0.25,
            shelter: 0.25,
            safety: 0.25,
            belonging: 0.25,
        };
        let lod = match offset % 3 {
            0 => LodTier::Hot,
            1 => LodTier::Warm,
            _ => LodTier::Cold,
        };
        entities.push(spawn_civilian(
            world, civilian, position, wardrobe, tools, needs, lod,
        ));
    }
    entities
}

/// Count civilian entities in the world.
pub fn count_civilians(world: &World) -> usize {
    world.query::<&Civilian>().iter().count()
}

/// Score unmet needs using utility weights.
pub fn score_needs(needs: &Needs, weights: &UtilityWeights) -> f32 {
    needs.food * weights.food
        + needs.shelter * weights.shelter
        + needs.safety * weights.safety
        + needs.belonging * weights.belonging
}

/// Pick the highest-priority unmet need.
pub fn top_action(needs: &Needs, weights: &UtilityWeights) -> NeedAction {
    let scores = [
        (needs.food * weights.food, NeedAction::FindFood),
        (needs.shelter * weights.shelter, NeedAction::FindShelter),
        (needs.safety * weights.safety, NeedAction::Flee),
        (needs.belonging * weights.belonging, NeedAction::Socialize),
    ];
    scores
        .into_iter()
        .max_by(|(a, _), (b, _)| a.total_cmp(b))
        .map(|(_, action)| action)
        .unwrap_or(NeedAction::Idle)
}

/// Return whether a civilian should tick on the current simulation tick.
pub fn should_tick_now(tier: LodTier, current_tick: u64) -> bool {
    match tier {
        LodTier::Hot => true,
        LodTier::Warm => current_tick % 4 == 0,
        LodTier::Cold => current_tick % 16 == 0,
    }
}

/// Drive era propagation for one tick on one civilian's [`Wardrobe`]. The
/// `target_era` is the civilization-wide "current best" era; this tick may or
/// may not promote the civilian based on the diffusion S-curve sample and
/// the RNG draw.
///
/// Returns `true` if the wardrobe era was bumped.
pub fn propagate_wardrobe(
    wardrobe: &mut Wardrobe,
    target_era: u16,
    civ_adoption_fraction: f32,
    params: DiffusionParams,
    rng: &mut ChaCha8Rng,
) -> bool {
    if wardrobe.era >= target_era {
        return false;
    }
    // Per-tick adoption rate as a probability that THIS civilian flips this tick.
    let rate = diffusion_advance(civ_adoption_fraction, params) - civ_adoption_fraction;
    if rng.gen::<f32>() < rate.max(0.0) {
        wardrobe.era += 1;
        return true;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> ChaCha8Rng {
        ChaCha8Rng::seed_from_u64(seed)
    }

    /// FR-CIV-AGENTS-000 — schema version exposed.
    #[test]
    fn schema_version_present() {
        assert_eq!(SCHEMA_VERSION, 0);
    }

    /// FR-CIV-AGENTS-001 — propagate_wardrobe is deterministic under a fixed
    /// seed (replay-safe).
    #[test]
    fn propagate_wardrobe_is_deterministic() {
        let params = DiffusionParams::default();
        let mut w1 = Wardrobe {
            era: 1,
            material: MaterialId(2),
        };
        let mut w2 = w1;
        let mut r1 = rng(42);
        let mut r2 = rng(42);
        for _ in 0..200 {
            propagate_wardrobe(&mut w1, 5, 0.3, params, &mut r1);
            propagate_wardrobe(&mut w2, 5, 0.3, params, &mut r2);
        }
        assert_eq!(w1, w2);
    }

    /// FR-CIV-AGENTS-002 — propagation never goes backwards.
    #[test]
    fn propagation_is_monotone() {
        let params = DiffusionParams::default();
        let mut w = Wardrobe {
            era: 2,
            material: MaterialId(1),
        };
        let mut r = rng(7);
        for _ in 0..1000 {
            propagate_wardrobe(&mut w, 5, 0.5, params, &mut r);
        }
        assert!(w.era >= 2);
        assert!(w.era <= 5);
    }

    /// FR-CIV-AGENTS-003 — civilians at the target era do not propagate further.
    #[test]
    fn at_target_era_is_a_noop() {
        let params = DiffusionParams::default();
        let mut w = Wardrobe {
            era: 5,
            material: MaterialId(1),
        };
        let mut r = rng(1);
        for _ in 0..100 {
            assert!(!propagate_wardrobe(&mut w, 5, 0.99, params, &mut r));
        }
        assert_eq!(w.era, 5);
    }

    /// FR-CIV-AGENTS-010 — LodTier debug / equality holds across copies.
    #[test]
    fn lod_tier_equality_round_trips() {
        for t in [LodTier::Hot, LodTier::Warm, LodTier::Cold] {
            let copy = t;
            assert_eq!(t, copy);
        }
    }

    /// FR-CIV-AGENTS-011 — fixture access: civilians compose with civ-genetics
    /// and civ-species without leaking RNG into the type system.
    #[test]
    fn civilian_composition_smoke() {
        use civ_genetics::{Dna, DnaClass};
        use civ_species::{express, Phenotype};
        let class = DnaClass::default();
        let dna = Dna::zero(class.length);
        let _: Phenotype = express(&dna);
        let _civ = Civilian {
            id: 0,
            faction: 1,
            age: 30,
        };
    }

    /// FR-CIV-AGENTS-020 — spawn_civilian inserts all requested components.
    #[test]
    fn spawn_civilian_inserts_components() {
        let mut world = World::new();
        let civ = Civilian {
            id: 11,
            faction: 7,
            age: 24,
        };
        let pos = Position3d {
            coord: WorldCoord { x: 0, y: 0, z: 0 },
        };
        let wardrobe = Wardrobe {
            era: 2,
            material: MaterialId(3),
        };
        let tools = Tools {
            era: 4,
            material: MaterialId(5),
        };
        let needs = Needs {
            food: 0.1,
            shelter: 0.2,
            safety: 0.3,
            belonging: 0.4,
        };
        let lod = LodTier::Warm;
        let entity = spawn_civilian(&mut world, civ.clone(), pos, wardrobe, tools, needs, lod);

        assert_eq!(&*world.get::<&Civilian>(entity).unwrap(), &civ);
        assert_eq!(&*world.get::<&Position3d>(entity).unwrap(), &pos);
        assert_eq!(&*world.get::<&Wardrobe>(entity).unwrap(), &wardrobe);
        assert_eq!(&*world.get::<&Tools>(entity).unwrap(), &tools);
        assert_eq!(&*world.get::<&Needs>(entity).unwrap(), &needs);
        assert_eq!(&*world.get::<&LodTier>(entity).unwrap(), &lod);
    }

    /// FR-CIV-AGENTS-021 — spawn_many produces sequential IDs.
    #[test]
    fn spawn_many_produces_sequential_ids() {
        let mut world = World::new();
        let entities = spawn_many(&mut world, 4, 100, 9);
        assert_eq!(entities.len(), 4);
        let mut ids = Vec::new();
        for entity in entities {
            let civ = world.get::<&Civilian>(entity).unwrap();
            ids.push(civ.id);
        }
        assert_eq!(ids, vec![100, 101, 102, 103]);
    }

    /// FR-CIV-AGENTS-022 — count_civilians reports the current world total.
    #[test]
    fn count_civilians_reports_correctly() {
        let mut world = World::new();
        assert_eq!(count_civilians(&world), 0);
        spawn_many(&mut world, 3, 1, 2);
        assert_eq!(count_civilians(&world), 3);
    }

    /// FR-CIV-AGENTS-023 — score_needs is a deterministic weighted sum.
    #[test]
    fn score_needs_returns_deterministic_sums() {
        let needs = Needs {
            food: 0.5,
            shelter: 0.25,
            safety: 0.75,
            belonging: 0.125,
        };
        let weights = UtilityWeights {
            food: 2.0,
            shelter: 3.0,
            safety: 4.0,
            belonging: 5.0,
        };
        assert_eq!(
            score_needs(&needs, &weights),
            0.5 * 2.0 + 0.25 * 3.0 + 0.75 * 4.0 + 0.125 * 5.0
        );
    }

    /// FR-CIV-AGENTS-024 — top_action selects the highest-weighted unmet need.
    #[test]
    fn top_action_picks_highest_weighted_unmet_need() {
        let needs = Needs {
            food: 0.1,
            shelter: 0.9,
            safety: 0.2,
            belonging: 0.3,
        };
        let weights = UtilityWeights {
            food: 1.0,
            shelter: 10.0,
            safety: 2.0,
            belonging: 3.0,
        };
        assert_eq!(top_action(&needs, &weights), NeedAction::FindShelter);
    }

    /// FR-CIV-AGENTS-025 — should_tick_now respects LOD modulo cadence.
    #[test]
    fn should_tick_now_respects_lod_modulo() {
        assert!(should_tick_now(LodTier::Hot, 1));
        assert!(should_tick_now(LodTier::Warm, 4));
        assert!(!should_tick_now(LodTier::Warm, 5));
        assert!(should_tick_now(LodTier::Cold, 16));
        assert!(!should_tick_now(LodTier::Cold, 17));
    }
}
