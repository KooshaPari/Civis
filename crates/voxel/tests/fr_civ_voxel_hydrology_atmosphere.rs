//! FR-CIV-VOXEL-031 / 032 — world-gen hydrology and atmosphere.
//!
//! Requirement text (docs/guides/voxel-emergent-vision-and-migration.md:130-131):
//!
//! - FR-CIV-VOXEL-031: "World gen hydrology: water-filled basin cells generated
//!   from elevation + permeability of strata."
//! - FR-CIV-VOXEL-032: "World gen atmosphere: gas-pocket cells seeded in
//!   underground cavities; composition from RON material table."
//!
//! Replaces placeholders (`crates/voxel/tests/fr_fr_civ_voxel_031.rs` and
//! `_032.rs`) whose entire body was `assert_eq!(c.x, 0); assert!(FIXED_SCALE > 0);`
//! — a check on a coordinate struct, identical in both files, verifying neither
//! requirement.
//!
//! ## Status of the two requirements
//!
//! **031 (hydrology) is implemented.** `worldgen::generate` fills water where
//! `y > surface && y <= sea_level(dims)`, so water sits in depressions below sea
//! level and never floats over elevated terrain. The oracles below assert that
//! contract directly.
//!
//! **032 (atmosphere) is NOT implemented.** There is no gas-pocket or cavity
//! generation anywhere in `crates/voxel/src/worldgen.rs` (only `ore_pocket` for
//! ORE), and no `materials.ron` material table exists anywhere in the
//! repository despite being named by the requirement. Rather than invent
//! behaviour to assert, the 032 test pins the current absence as a falsifiable
//! regression guard: when someone implements gas pockets, it fails and must be
//! rewritten into a real positive oracle.

use civ_voxel::material::CO2;
use civ_voxel::worldgen::{generate, sea_level, surface_height, GenWorld};
use civ_voxel::{MaterialId, AIR, METHANE, SMOKE, TOXIC_GAS, WATER};

/// Materials that represent an *atmosphere* gas pocket (i.e. not the empty cell).
const GAS_POCKET_MATERIALS: [MaterialId; 4] = [METHANE, CO2, TOXIC_GAS, SMOKE];

/// Linear index into `GenWorld::cells` (`x + y*dx + z*dx*dy`, Y up).
fn index(world: &GenWorld, x: usize, y: usize, z: usize) -> usize {
    let [dx, _dy, _dz] = world.dims;
    x + y * dx + z * dx * world.dims[1]
}

/// Count cells equal to `mat`.
fn count(world: &GenWorld, mat: MaterialId) -> usize {
    world.cells.iter().filter(|c| **c == mat).count()
}

/// A world large enough to have interior relief.
fn world(seed: u64) -> GenWorld {
    generate([16, 16, 16], seed)
}

// ---------------------------------------------------------------------------
// FR-CIV-VOXEL-031 — hydrology
// ---------------------------------------------------------------------------

/// Covers FR-CIV-VOXEL-031.
///
/// The core hydrology contract: every water cell lies strictly above the
/// terrain surface of its own column and at or below sea level. That is what
/// makes water a *basin* fill rather than a slab floating over hills.
#[test]
fn fr_civ_voxel_031_water_occupies_only_sub_sea_depressions() {
    let w = world(0x0310_5EED);
    let dims = w.dims;
    let sea = sea_level(dims);
    let [dx, dy, dz] = dims;

    let mut water_cells = 0usize;
    for z in 0..dz {
        for x in 0..dx {
            let surface = surface_height(dims, 0x0310_5EED, x, z);
            for y in 0..dy {
                if w.cells[index(&w, x, y, z)] != WATER {
                    continue;
                }
                water_cells += 1;
                assert!(
                    y > surface,
                    "water at ({x},{y},{z}) is at or below its column surface \
                     ({surface}); water must sit above the terrain, not inside it"
                );
                assert!(
                    y <= sea,
                    "water at ({x},{y},{z}) is above sea level ({sea}); water \
                     must not float over elevated terrain"
                );
            }
        }
    }

    assert!(
        water_cells > 0,
        "hydrology must place water somewhere in a 16^3 coastal world, else \
         there is no basin fill to verify"
    );
}

/// Covers FR-CIV-VOXEL-031.
///
/// Columns that rise above sea level must contain no water at all — the
/// flat-blue-slab regression. An elevated column has nothing to fill.
#[test]
fn fr_civ_voxel_031_above_sea_columns_hold_no_water() {
    let seed = 0x0310_5EED;
    let w = world(seed);
    let dims = w.dims;
    let sea = sea_level(dims);
    let [dx, dy, dz] = dims;

    let mut elevated_columns = 0usize;
    for z in 0..dz {
        for x in 0..dx {
            let surface = surface_height(dims, seed, x, z);
            if surface < sea {
                continue; // a depression: water is expected here
            }
            elevated_columns += 1;
            for y in 0..dy {
                assert_ne!(
                    w.cells[index(&w, x, y, z)],
                    WATER,
                    "column ({x},{z}) rises to {surface} (sea = {sea}) yet holds \
                     water at y={y}; that is the flat-slab regression"
                );
            }
        }
    }

    assert!(
        elevated_columns > 0,
        "premise: a 16^3 world should have at least one column at or above sea level"
    );
}

/// Covers FR-CIV-VOXEL-031.
///
/// Hydrology is generated `from elevation ... deterministically from seed`
/// (FR-CIV-VOXEL-030 shares the seed contract). The same seed must produce the
/// same water layout, and different seeds a different one.
#[test]
fn fr_civ_voxel_031_hydrology_is_seed_deterministic() {
    let a = world(0x0310_AAAA);
    let b = world(0x0310_AAAA);
    let c = world(0x0310_BBBB);

    assert_eq!(
        a.cells, b.cells,
        "the same seed must produce an identical world, water included"
    );
    assert!(
        count(&a, WATER) > 0 && count(&c, WATER) > 0,
        "both seeds must generate water for the comparison to be meaningful"
    );
    assert_ne!(
        a.cells, c.cells,
        "a different seed must produce a different world"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-VOXEL-032 — atmosphere (NOT IMPLEMENTED)
// ---------------------------------------------------------------------------

/// Covers FR-CIV-VOXEL-032.
///
/// **This requirement is unimplemented and this test records that.**
///
/// The requirement asks for "gas-pocket cells seeded in underground cavities;
/// composition from RON material table". Neither exists:
///
///   * `crates/voxel/src/worldgen.rs` generates no gas of any kind — the only
///     pocket generator is `ore_pocket`, which produces `ORE`. There is no
///     cavity/cavern carving and no call to `METHANE`, `CO2`, `TOXIC_GAS`, or
///     `SMOKE`.
///   * No `materials.ron` file exists anywhere in the repository, so there is no
///     RON material table for a composition to come from.
///
/// Asserting a gas pocket exists would fail, and asserting invented behaviour
/// would be worse than useless. So this asserts the current absence: it is
/// falsifiable and will fail the moment atmosphere generation lands, which is
/// the signal to replace it with a real positive oracle.
#[test]
fn fr_civ_voxel_032_gas_pockets_are_not_yet_generated() {
    // TODO(FR-CIV-VOXEL-032): implement gas-pocket seeding in worldgen and
    // invert this assertion. Until then the requirement is NOT met, and the
    // audit counting it as COVERED rests on the placeholder file only.
    for seed in [0x0320_0001u64, 0x0320_0002, 0x0320_0003] {
        let w = world(seed);
        for gas in GAS_POCKET_MATERIALS {
            let n = count(&w, gas);
            assert_eq!(
                n,
                0,
                "worldgen emitted {n} cells of {gas:?} for seed {seed:#x}. \
                 FR-CIV-VOXEL-032 appears to have been implemented: replace this \
                 test with a real positive oracle for gas-pocket seeding"
            );
        }
    }
}

/// Covers FR-CIV-VOXEL-032.
///
/// Supporting evidence for the gap above: the only underground void content the
/// generator produces is `ORE`, and the stratum directly above bedrock is
/// solid. If gas pockets were being seeded into cavities, this world would not
/// be uniformly solid below its surface.
#[test]
fn fr_civ_voxel_032_underground_is_solid_apart_from_ore() {
    use civ_voxel::{BEDROCK, DIRT, ORE, PLANT, STONE, WATER};

    let w = world(0x0320_9EED);
    let dims = w.dims;
    let [dx, dy, dz] = dims;

    // Materials legitimately found below the surface line.
    let solid_or_ore = [STONE, ORE, BEDROCK, DIRT];

    for z in 0..dz {
        for x in 0..dx {
            let surface = surface_height(dims, 0x0320_9EED, x, z);
            // Everything strictly below the surface must be solid rock, ore,
            // soil, or bedrock. AIR or a gas here would mean a carved cavity.
            for y in 0..surface.saturating_sub(2) {
                let m = w.cells[index(&w, x, y, z)];
                assert!(
                    solid_or_ore.contains(&m),
                    "sub-surface cell ({x},{y},{z}) is {m:?}, not solid rock/ore \
                     (AIR here would mean a gas cavity was carved). See \
                     FR-CIV-VOXEL-032"
                );
            }
        }
    }

    // Sanity: the world does contain material, and the surface materials are not
    // mistaken for the below-surface set.
    assert!(count(&w, STONE) > 0, "world must contain stone");
    for surface_only in [WATER, PLANT, AIR] {
        assert!(
            !solid_or_ore.contains(&surface_only),
            "the below-surface allowlist must not accept {surface_only:?}"
        );
    }
}
