//! FR-CIV-VOXEL-023 / 024 / 025 — fluid, gas, and heat-transfer cellular
//! automata.
//!
//! Requirement text (docs/guides/voxel-emergent-vision-and-migration.md:126-128):
//!
//! - FR-CIV-VOXEL-023: "Fluid CA: liquid material flows laterally when
//!   vertically blocked; pressure propagates via fixed-point depth accumulation."
//! - FR-CIV-VOXEL-024: "Gas CA: gas material rises, disperses into adjacent
//!   empty cells; density field decays over ticks."
//! - FR-CIV-VOXEL-025: "Heat transfer CA: temperature field propagates between
//!   adjacent cells; ignition threshold triggers material phase change."
//!
//! These replace placeholder tests (`crates/voxel/tests/fr_fr_civ_voxel_023.rs`
//! and siblings) whose entire body asserted
//! `assert_eq!(c.x, 0); assert!(FIXED_SCALE > 0);` — a check on a coordinate
//! struct that verified nothing about any of the three requirements, and was
//! byte-identical between the two files.
//!
//! Assertions are deliberately expressed over *phases* (via the material
//! registry) rather than hard-coded ids where a phase change is possible, so a
//! legitimate liquid->gas or gas->liquid transition does not fail the test for
//! the wrong reason.

use civ_voxel::material::Phase;
use civ_voxel::{fluid_ca, AIR, SAND, STEAM, STONE, WATER};

/// Standard registry, used for phase lookups.
fn reg() -> civ_voxel::material::MaterialRegistry {
    civ_voxel::material::MaterialRegistry::standard()
}

/// Count cells holding `id`.
///
/// Iterates `grid.dims` (cell extents). Note `chunk_counts()` is NOT this: it
/// returns chunk counts (`dims / 16`, rounded up), so using it would scan a
/// single cell for any grid smaller than 16 cubed.
fn count(grid: &fluid_ca::CaGrid, id: civ_voxel::MaterialId) -> usize {
    let [dx, dy, dz] = grid.dims;
    let mut n = 0;
    for x in 0..dx {
        for y in 0..dy {
            for z in 0..dz {
                if grid.get(x, y, z) == id {
                    n += 1;
                }
            }
        }
    }
    n
}

/// Count non-air cells whose material is the given phase.
///
/// `AIR` is itself `Phase::Gas`, so a raw phase count over a grid mostly
/// reports empty space. Excluding air is what makes a gas-phase assertion
/// meaningful.
fn count_non_air_phase(grid: &fluid_ca::CaGrid, want: Phase) -> usize {
    let r = reg();
    let [dx, dy, dz] = grid.dims;
    let mut n = 0;
    for x in 0..dx {
        for y in 0..dy {
            for z in 0..dz {
                let m = grid.get(x, y, z);
                if m != AIR && r.get(m).is_some_and(|d| d.phase == want) {
                    n += 1;
                }
            }
        }
    }
    n
}

/// A `dims` grid of air with a stone floor at y=0.
fn floored(dims: [usize; 3]) -> fluid_ca::CaGrid {
    let mut g = fluid_ca::CaGrid::new(dims);
    for x in 0..dims[0] {
        for z in 0..dims[2] {
            g.set(x, 0, z, STONE);
        }
    }
    g
}

// ---------------------------------------------------------------------------
// FR-CIV-VOXEL-023 — fluid CA
// ---------------------------------------------------------------------------

/// Covers FR-CIV-VOXEL-023.
///
/// "Liquid material flows laterally when vertically blocked." Water sitting on
/// a stone floor cannot fall, so it must spread sideways.
#[test]
fn fr_civ_voxel_023_liquid_spreads_laterally_when_vertically_blocked() {
    let mut g = floored([5, 4, 1]);
    g.set(2, 1, 0, WATER);
    assert_eq!(count(&g, WATER), 1, "precondition: exactly one water cell");

    let r = reg();
    for _ in 0..8 {
        fluid_ca::step(&mut g, r);
    }

    // Liquid is conserved by lateral motion.
    assert_eq!(
        count(&g, WATER),
        1,
        "liquid must be conserved while spreading, not destroyed"
    );
    // And it must have left its origin column.
    let spread = (0..5).any(|x| g.get(x, 1, 0) == WATER && x != 2);
    assert!(
        spread,
        "vertically-blocked liquid must flow laterally; after 8 steps the water \
         is still only in its origin column"
    );
}

/// Covers FR-CIV-VOXEL-023.
///
/// Liquid must fall when it has room, before spreading sideways.
#[test]
fn fr_civ_voxel_023_liquid_falls_when_unsupported() {
    let mut g = floored([1, 6, 1]);
    g.set(0, 5, 0, WATER);

    let r = reg();
    for _ in 0..10 {
        fluid_ca::step(&mut g, r);
    }

    assert_eq!(count(&g, WATER), 1, "liquid conserved while falling");
    assert_eq!(
        g.get(0, 1, 0),
        WATER,
        "unsupported liquid must settle onto the floor at y=1"
    );
    assert_eq!(g.get(0, 5, 0), AIR, "source cell must be vacated");
}

/// Covers FR-CIV-VOXEL-023.
///
/// Ambient water must stay liquid. A fresh grid defaults every cell to
/// temperature 0, which is exactly water's `freeze_point`; if `set` preserved
/// that instead of using the material's declared initial temperature, the water
/// would flash-freeze to ice on the first step.
#[test]
fn fr_civ_voxel_023_ambient_water_stays_liquid() {
    let mut g = fluid_ca::CaGrid::new([3, 3, 1]);
    g.set(1, 1, 0, WATER);

    let r = reg();
    for _ in 0..5 {
        fluid_ca::step(&mut g, r);
    }

    assert_eq!(
        count(&g, WATER),
        1,
        "ambient water must remain a liquid across several ticks"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-VOXEL-024 — gas CA
// ---------------------------------------------------------------------------

/// Covers FR-CIV-VOXEL-024.
///
/// "Gas material rises." A gas cell seeded low in an open column must vacate
/// its starting cell and end up higher.
#[test]
fn fr_civ_voxel_024_gas_rises() {
    let mut g = fluid_ca::CaGrid::new([1, 8, 1]);
    g.set(0, 1, 0, STEAM);
    let start = g.get(0, 1, 0);

    let r = reg();
    for _ in 0..6 {
        fluid_ca::step(&mut g, r);
    }

    assert_eq!(
        g.get(0, 1, 0),
        AIR,
        "gas must vacate its starting cell when it has room to rise"
    );
    // The gas is somewhere above the origin (whatever id it settled into).
    let above = (2..8).any(|y| g.get(0, y, 0) != AIR);
    assert!(
        above,
        "gas must rise above its origin's row; column above is still empty \
         (seeded {start:?})"
    );
}

/// Covers FR-CIV-VOXEL-024.
///
/// Gas must remain somewhere in an enclosed column: it rises and disperses but
/// is not destroyed.
#[test]
fn fr_civ_voxel_024_gas_is_conserved_while_rising() {
    let mut g = fluid_ca::CaGrid::new([1, 8, 1]);
    g.set(0, 1, 0, STEAM);

    let r = reg();
    for _ in 0..6 {
        fluid_ca::step(&mut g, r);
    }

    let non_air = (0..8).filter(|&y| g.get(0, y, 0) != AIR).count();
    assert_eq!(
        non_air, 1,
        "the single gas cell must persist somewhere in the closed column"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-VOXEL-025 — heat transfer CA
// ---------------------------------------------------------------------------

/// Covers FR-CIV-VOXEL-025.
///
/// "Temperature field propagates between adjacent cells." A hot cell beside
/// cold ones must raise their temperature above the starting value.
#[test]
fn fr_civ_voxel_025_temperature_propagates_to_adjacent_cells() {
    let mut g = fluid_ca::CaGrid::new([3, 1, 1]);
    g.set_with_temp(1, 0, 0, STONE, 900);

    let cold_left = g.get_temp(0, 0, 0);
    let cold_right = g.get_temp(2, 0, 0);
    assert!(cold_left < 900 && cold_right < 900, "precondition: cold neighbours");

    let r = reg();
    for _ in 0..10 {
        fluid_ca::step(&mut g, r);
    }

    let left = g.get_temp(0, 0, 0);
    let right = g.get_temp(2, 0, 0);
    assert!(
        left > cold_left || right > cold_right,
        "heat must propagate into at least one adjacent cell; neighbours went \
         from ({cold_left}, {cold_right}) to ({left}, {right})"
    );
}

/// Covers FR-CIV-VOXEL-025.
///
/// Conduction is a transfer, not a free increase: the hot source cell must not
/// gain heat from colder neighbours.
#[test]
fn fr_civ_voxel_025_hot_cell_does_not_gain_heat_from_cold_neighbours() {
    let mut g = fluid_ca::CaGrid::new([3, 1, 1]);
    g.set_with_temp(1, 0, 0, STONE, 900);
    let before = g.get_temp(1, 0, 0);

    let r = reg();
    for _ in 0..10 {
        fluid_ca::step(&mut g, r);
    }

    let after = g.get_temp(1, 0, 0);
    assert!(
        after <= before,
        "the source cell must not gain heat from colder neighbours; went from \
         {before} to {after}"
    );
}

// ---------------------------------------------------------------------------
// FR-CIV-VOXEL-030 — deterministic world gen
// ---------------------------------------------------------------------------

/// Covers FR-CIV-VOXEL-030.
///
/// "World gen strata ... generated deterministically from seed". Same seed must
/// reproduce the world exactly; a different seed must produce a different one.
#[test]
fn fr_civ_voxel_030_worldgen_is_deterministic_and_seed_dependent() {
    use civ_voxel::worldgen::generate;

    let dims = [8usize, 8, 8];
    let a = generate(dims, 0xABCD_1234);
    let b = generate(dims, 0xABCD_1234);
    assert_eq!(a, b, "same seed must produce an identical voxel world");

    let c = generate(dims, 0x9999_0001);
    assert_ne!(
        a, c,
        "different seeds must produce different worlds, otherwise the seed is ignored"
    );

    assert_eq!(a.dims, dims, "generated world must match requested dims");
    assert_eq!(
        a.cells.len(),
        dims[0] * dims[1] * dims[2],
        "generated world must be densely populated"
    );
}

/// Covers FR-CIV-VOXEL-030.
///
/// "Voxel world valid after gen": no air at the world floor, and every cell
/// holds a material the registry knows.
#[test]
fn fr_civ_voxel_030_generated_world_is_valid() {
    use civ_voxel::worldgen::generate;

    let dims = [8usize, 8, 8];
    let world = generate(dims, 0x0BAD_F00D);
    let r = reg();

    let [dx, dy, dz] = world.dims;
    assert!(dy > 0, "world must have vertical extent");

    for z in 0..dz {
        for x in 0..dx {
            // Row-major `x + y*dx + z*dx*dy`; y=0 is the world floor.
            let floor = world.cells[x + z * dx * dy];
            assert_ne!(
                floor,
                AIR,
                "column ({x},0,{z}) has air at the world floor; world invalid after gen"
            );
        }
    }

    for (i, &cell) in world.cells.iter().enumerate() {
        assert!(
            r.get(cell).is_some(),
            "cell {i} holds MaterialId({}) which is not in the material registry",
            cell.0
        );
    }
}

// ---------------------------------------------------------------------------
// FR-CIV-VOXEL-030 support — powder gravity
// ---------------------------------------------------------------------------

/// Powder must settle onto support rather than hovering.
///
/// Supports FR-CIV-VOXEL-030's "valid after gen" premise together with the
/// material palette contract (FR-CIV-VOXEL-021) that SAND is a falling powder.
#[test]
fn fr_civ_voxel_030_powder_settles_onto_support() {
    let mut g = floored([1, 6, 1]);
    g.set(0, 5, 0, SAND);

    let r = reg();
    for _ in 0..12 {
        fluid_ca::step(&mut g, r);
    }

    assert_eq!(count(&g, SAND), 1, "powder conserved while settling");
    assert_eq!(
        g.get(0, 1, 0),
        SAND,
        "sand must come to rest on the stone floor at y=1"
    );
    assert_eq!(g.get(0, 5, 0), AIR, "sand must vacate its starting cell");
}

/// A gas cell must not count as solid matter: the gas phase classification the
/// other tests rely on must actually distinguish gas from liquid/solid.
///
/// Note `AIR` is itself `Phase::Gas`, so a grid of air already reports the gas
/// phase for every empty cell. This test therefore counts *non-air* gas cells,
/// which is what "STEAM is a gas" actually means here.
#[test]
fn fr_civ_voxel_024_gas_phase_is_classified_as_gas() {
    let r = reg();
    assert_eq!(
        r.get(STEAM).expect("STEAM in registry").phase,
        Phase::Gas,
        "STEAM must be classified as a gas phase"
    );
    assert_eq!(
        r.get(WATER).expect("WATER in registry").phase,
        Phase::Liquid,
        "WATER must be classified as a liquid phase, not gas"
    );

    let mut g = fluid_ca::CaGrid::new([1, 3, 1]);
    g.set(0, 1, 0, STEAM);
    assert_eq!(
        count_non_air_phase(&g, Phase::Gas),
        1,
        "the single STEAM cell must count as a gas"
    );
}
