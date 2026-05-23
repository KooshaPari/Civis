//! civ-agents — civilian agent ECS components + LOD tick + per-civilian
//! wardrobe / tools state.
//!
//! Components live in `civ-engine`'s shared `hecs::World`. This crate ships:
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
}
