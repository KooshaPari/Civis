//! FR-CIV-RENDER-002 — translucent material pass.
//!
//! Requirement text (docs/guides/voxel-emergent-vision-and-migration.md:152):
//!
//! > "Translucent material pass: liquid and gas cells rendered with
//! > alpha-blended geometry; solid cells rendered opaque first."
//!
//! This crate owns the data the pass consumes: each [`MaterialDef`] carries an
//! RGBA render hint (`color`), and its `phase` classifies the cell. This test
//! asserts the invariants that actually hold over the shipped palette and pins
//! the places where the palette does **not** match a strict reading of the
//! requirement, so the gap is visible rather than assumed away.
//!
//! Replaces a placeholder (`crates/voxel/tests/fr_fr_civ_render_002.rs`) whose
//! entire body was `assert_eq!(c.x, 0); assert!(FIXED_SCALE > 0);` — a check on
//! a coordinate struct that says nothing about transparency.
//!
//! Scope note: the geometry pass itself lives in the Bevy client, which needs a
//! GPU and is not exercised here. What is verified is the classification the
//! pass reads (`phase` + alpha), which is the headless-checkable half.

use civ_voxel::material::{MaterialRegistry, Phase};
use civ_voxel::{AIR, DIRT, GLASS, ICE, LAVA, SAND, STEAM, STONE, WATER};

/// RGBA render hint for `id`.
fn rgba(id: civ_voxel::MaterialId) -> [u8; 4] {
    MaterialRegistry::standard()
        .get(id)
        .expect("material must be in the registry")
        .color
}

/// Alpha channel of the render hint.
fn alpha(id: civ_voxel::MaterialId) -> u8 {
    rgba(id)[3]
}

/// The phase the pass uses to classify a cell.
fn phase(id: civ_voxel::MaterialId) -> Phase {
    MaterialRegistry::standard()
        .get(id)
        .expect("material must be in the registry")
        .phase
}

/// Covers FR-CIV-RENDER-002.
///
/// The three materials the requirement names explicitly: water and steam are
/// blended (alpha < 255), and stone is not (alpha == 255) so it can be drawn
/// opaque first.
#[test]
fn fr_civ_render_002_named_materials_classify_correctly() {
    assert_eq!(phase(WATER), Phase::Liquid, "water is a liquid");
    assert_eq!(phase(STEAM), Phase::Gas, "steam is a gas");
    assert_eq!(phase(STONE), Phase::Solid, "stone is a solid");

    assert!(
        alpha(WATER) < 255,
        "water must be alpha-blended, got alpha {}",
        alpha(WATER)
    );
    assert!(
        alpha(STEAM) < 255,
        "steam must be alpha-blended, got alpha {}",
        alpha(STEAM)
    );
    assert_eq!(
        alpha(STONE),
        255,
        "stone must be opaque so it can be drawn first"
    );
}

/// Covers FR-CIV-RENDER-002.
///
/// Air is the fully transparent case: alpha 0, so empty cells contribute
/// nothing to the blended layer.
#[test]
fn fr_civ_render_002_air_is_fully_transparent() {
    assert_eq!(phase(AIR), Phase::Gas, "air is modelled as a gas");
    assert_eq!(
        alpha(AIR),
        0,
        "air must be fully transparent, otherwise the blend layer is a solid film"
    );
}

/// Covers FR-CIV-RENDER-002.
///
/// The common opaque terrain materials must all be exactly opaque. Mixing
/// partial alpha into terrain would make the opaque-first pass incorrect.
#[test]
fn fr_civ_render_002_terrain_is_strictly_opaque() {
    for id in [STONE, DIRT, SAND, ICE] {
        assert_eq!(
            alpha(id),
            255,
            "terrain material {id:?} must be fully opaque, got alpha {}",
            alpha(id)
        );
    }
}

/// Covers FR-CIV-RENDER-002.
///
/// Observed palette behaviour, recorded deliberately.
///
/// A strict reading of the requirement ("liquid and gas cells rendered
/// alpha-blended; solid cells rendered opaque") does **not** hold across the
/// shipped palette:
///
///   * `LAVA` is a `Liquid` with alpha 255 — an opaque liquid.
///   * `GLASS` and `CRYSTAL` are `Solid` with alpha 190 / 210 — translucent
///     solids.
///
/// Both are physically defensible (molten rock is not see-through; glass is),
/// so the pass cannot simply be "blend every liquid and gas, cull nothing
/// else". This test pins the current classification so a palette edit that
/// changes it is caught, and carries the discrepancy as an explicit TODO rather
/// than pretending the requirement is met verbatim.
#[test]
fn fr_civ_render_002_palette_does_not_split_strictly_on_phase() {
    // Opaque liquids: alpha 255 despite Phase::Liquid.
    assert_eq!(phase(LAVA), Phase::Liquid);
    assert_eq!(
        alpha(LAVA),
        255,
        "TODO(FR-CIV-RENDER-002): lava is an opaque liquid, so the pass cannot \
         key blending on phase alone"
    );

    // Translucent solids: alpha < 255 despite Phase::Solid.
    assert_eq!(phase(GLASS), Phase::Solid);
    assert!(
        alpha(GLASS) < 255,
        "TODO(FR-CIV-RENDER-002): glass is a translucent solid (alpha {}), so it \
         must still reach the blend layer",
        alpha(GLASS)
    );
}

/// Covers FR-CIV-RENDER-002.
///
/// Every material must carry a usable render hint, and alpha must be the
/// deciding channel: the pass reads these bytes, so a malformed or missing hint
/// would silently render as opaque.
#[test]
fn fr_civ_render_002_every_material_has_a_render_hint() {
    let reg = MaterialRegistry::standard();
    let materials = reg.materials();
    assert!(
        !materials.is_empty(),
        "the standard palette must not be empty"
    );

    let mut translucent = 0;
    for def in materials {
        let [r, g, b, a] = def.color;
        // A non-zero RGB with zero alpha would be an invisible opaque material,
        // which is always a palette mistake.
        if a == 0 {
            assert_eq!(
                [r, g, b],
                [0, 0, 0],
                "material {:?} has alpha 0 but non-black RGB; it would be invisible",
                def.id
            );
        }
        if a < 255 {
            translucent += 1;
        }
    }

    assert!(
        translucent > 0,
        "a translucency pass with nothing to blend would be dead code"
    );

    // Air is the only zero-alpha material, so the transparent set is the
    // fully-transparent case plus real blends.
    assert!(
        alpha(WATER) < 255 && alpha(WATER) > 0,
        "water must be a partial blend, not fully transparent"
    );
}
