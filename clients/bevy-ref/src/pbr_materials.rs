//! Per-material-type PBR `StandardMaterial` look-up table for the Civis Bevy client.
//!
//! Provides [`MaterialType`] — a typed enum covering all terrain/actor material
//! classes — and [`material_for`] which converts a `MaterialType` into a
//! `StandardMaterial` with physically-calibrated PBR values.
//!
//! ## Feature gate
//!
//! The Bevy-dependent portion (`material_for`, `MaterialTypePlugin`) is gated
//! behind `#[cfg(feature = "bevy")]`. The [`MaterialType`] enum and its
//! constant accessors are always compiled so test modules and doc-tests can use
//! them without a Bevy dep.
//!
//! ## Calibration notes
//!
//! All values are tuned against Bevy 0.18 PBR (Cook-Torrance GGX, F0 from
//! `reflectance`, illuminance in lux from `DirectionalLight`).
//!
//! | Material       | base_color            | perceptual_roughness | metallic | reflectance |
//! |----------------|-----------------------|----------------------|----------|-------------|
//! | Water          | deep-blue semi-opaque | 0.04                 | 0.0      | 0.75        |
//! | Stone          | mid-grey              | 0.90                 | 0.0      | 0.04        |
//! | Vegetation     | mid-green             | 0.95                 | 0.0      | 0.04        |
//! | Metal          | silver-grey           | 0.40                 | 0.80     | 0.50        |
//! | Sand           | sandy-tan             | 0.82                 | 0.0      | 0.04        |
//! | Lava           | deep-orange (emissive)| 0.85                 | 0.0      | 0.04        |
//! | Snow           | near-white            | 0.10                 | 0.0      | 0.12        |
//! | Dirt           | brown                 | 0.88                 | 0.0      | 0.04        |
//! | Wood           | warm-brown            | 0.72                 | 0.0      | 0.04        |

// ── MaterialType ─────────────────────────────────────────────────────────────

/// Typed material class for terrain tiles, actor primitives, and buildings.
///
/// Variants are ordered by approximate visual frequency in a temperate Civis
/// world so a simple `as usize` index gives a useful hot-path ordering for
/// texture atlas slots.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MaterialType {
    /// Open water (ocean, lake, river) — high reflectance, translucent.
    Water,
    /// Bare rock / cliff face — very rough, non-metallic.
    Stone,
    /// Grass / leaves / any dense vegetation canopy — matte green.
    Vegetation,
    /// Structural metal (building frames, tools) — high metallic.
    Metal,
    /// Dry sand or desert ground.
    Sand,
    /// Lava / magma — emissive orange-red for bloom.
    Lava,
    /// Packed snow / ice field — low roughness, slight blue tint.
    Snow,
    /// Topsoil / packed earth.
    Dirt,
    /// Timber / wooden structures.
    Wood,
}

impl MaterialType {
    /// All variants in canonical visual-frequency order.
    pub const ALL: [MaterialType; 9] = [
        MaterialType::Water,
        MaterialType::Stone,
        MaterialType::Vegetation,
        MaterialType::Metal,
        MaterialType::Sand,
        MaterialType::Lava,
        MaterialType::Snow,
        MaterialType::Dirt,
        MaterialType::Wood,
    ];

    /// Stable index in `0..9`, matching [`MaterialType::ALL`] ordering.
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            MaterialType::Water => 0,
            MaterialType::Stone => 1,
            MaterialType::Vegetation => 2,
            MaterialType::Metal => 3,
            MaterialType::Sand => 4,
            MaterialType::Lava => 5,
            MaterialType::Snow => 6,
            MaterialType::Dirt => 7,
            MaterialType::Wood => 8,
        }
    }

    /// Base sRGB colour for this material when no texture is available.
    /// Returns `[r, g, b]` in `0.0..=1.0`.
    #[must_use]
    pub const fn base_color_srgb(self) -> [f32; 3] {
        match self {
            MaterialType::Water => [0.06, 0.22, 0.55],
            MaterialType::Stone => [0.50, 0.50, 0.52],
            MaterialType::Vegetation => [0.18, 0.52, 0.18],
            MaterialType::Metal => [0.72, 0.72, 0.75],
            MaterialType::Sand => [0.86, 0.78, 0.52],
            MaterialType::Lava => [0.70, 0.18, 0.04],
            MaterialType::Snow => [0.94, 0.96, 0.98],
            MaterialType::Dirt => [0.45, 0.32, 0.18],
            MaterialType::Wood => [0.55, 0.38, 0.22],
        }
    }

    /// GGX perceptual roughness in `0.0..=1.0`. Lower = mirror-like.
    #[must_use]
    pub const fn perceptual_roughness(self) -> f32 {
        match self {
            MaterialType::Water => 0.04,
            MaterialType::Stone => 0.90,
            MaterialType::Vegetation => 0.95,
            MaterialType::Metal => 0.40,
            MaterialType::Sand => 0.82,
            MaterialType::Lava => 0.85,
            MaterialType::Snow => 0.10,
            MaterialType::Dirt => 0.88,
            MaterialType::Wood => 0.72,
        }
    }

    /// Metallic factor in `0.0..=1.0`. Non-metals should be `0.0`.
    #[must_use]
    pub const fn metallic(self) -> f32 {
        match self {
            MaterialType::Metal => 0.80,
            _ => 0.0,
        }
    }

    /// Fresnel reflectance at normal incidence (F0 proxy in Bevy's PBR).
    /// Default dielectric is `0.04`; water uses ~`0.02` linear (encoded 0.75
    /// via Bevy's remapping so the Fresnel rim is vivid).
    #[must_use]
    pub const fn reflectance(self) -> f32 {
        match self {
            MaterialType::Water => 0.75,
            MaterialType::Snow => 0.18,
            MaterialType::Metal => 0.50,
            _ => 0.04,
        }
    }

    /// Emissive linear-RGB multiplier. Non-zero only for lava.
    /// Values above `1.0` are HDR and drive bloom when `Bloom` is active.
    #[must_use]
    pub const fn emissive_linear_rgb(self) -> [f32; 3] {
        match self {
            MaterialType::Lava => [6.0, 1.2, 0.05],
            _ => [0.0, 0.0, 0.0],
        }
    }
}

// ── Bevy-dependent helpers ────────────────────────────────────────────────────

#[cfg(feature = "bevy")]
mod bevy_impl {
    use super::MaterialType;
    use bevy::prelude::*;

    /// Build a `StandardMaterial` for the given [`MaterialType`] with calibrated
    /// PBR values.  No textures are loaded — call the `pbr-textures` feature path
    /// in `materials.rs` when texture assets are available.
    ///
    /// Water uses `AlphaMode::Blend` with `base_color.alpha = 0.82`.
    #[must_use]
    pub fn material_for(mat: MaterialType) -> StandardMaterial {
        let [r, g, b] = mat.base_color_srgb();
        let [er, eg, eb] = mat.emissive_linear_rgb();

        let base_color = if mat == MaterialType::Water {
            Color::srgba(r, g, b, 0.82)
        } else {
            Color::srgb(r, g, b)
        };

        let alpha_mode = if mat == MaterialType::Water {
            AlphaMode::Blend
        } else {
            AlphaMode::Opaque
        };

        StandardMaterial {
            base_color,
            perceptual_roughness: mat.perceptual_roughness(),
            metallic: mat.metallic(),
            reflectance: mat.reflectance(),
            emissive: LinearRgba::rgb(er, eg, eb),
            alpha_mode,
            ..Default::default()
        }
    }

    /// Bevy plugin that exposes `material_for` to the app via `Assets<StandardMaterial>`.
    ///
    /// Registers no systems; use this as a dependency signal that the PBR
    /// material table is available. Call [`material_for`] directly to get a
    /// `StandardMaterial` value and add it to `Assets<StandardMaterial>`.
    pub struct PbrMaterialsPlugin;

    impl Plugin for PbrMaterialsPlugin {
        fn build(&self, _app: &mut App) {
            // No systems needed — the look-up is purely functional.
            // Downstream systems call `material_for` and add to `Assets`.
        }
    }
}

#[cfg(feature = "bevy")]
pub use bevy_impl::{material_for, PbrMaterialsPlugin};

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_len_matches_count() {
        assert_eq!(MaterialType::ALL.len(), 9);
    }

    #[test]
    fn index_matches_all_ordering() {
        for (i, mat) in MaterialType::ALL.iter().copied().enumerate() {
            assert_eq!(mat.index(), i, "{mat:?} index drift");
        }
    }

    #[test]
    fn base_color_in_unit_cube() {
        for mat in MaterialType::ALL {
            let [r, g, b] = mat.base_color_srgb();
            assert!(
                (0.0..=1.0).contains(&r) && (0.0..=1.0).contains(&g) && (0.0..=1.0).contains(&b),
                "{mat:?} base_color out of gamut: [{r}, {g}, {b}]"
            );
        }
    }

    #[test]
    fn roughness_in_unit_range() {
        for mat in MaterialType::ALL {
            let r = mat.perceptual_roughness();
            assert!(
                (0.0..=1.0).contains(&r),
                "{mat:?} roughness out of range: {r}"
            );
        }
    }

    #[test]
    fn metallic_only_nonzero_for_metal() {
        for mat in MaterialType::ALL {
            if mat == MaterialType::Metal {
                assert!(mat.metallic() > 0.0, "Metal should have metallic > 0");
            } else {
                assert_eq!(mat.metallic(), 0.0, "{mat:?} should have metallic == 0");
            }
        }
    }

    #[test]
    fn lava_has_emissive_glow() {
        let [r, g, b] = MaterialType::Lava.emissive_linear_rgb();
        assert!(r > 1.0, "lava red emissive should be HDR (> 1.0), got {r}");
        assert!(g > 0.0, "lava green emissive should be positive, got {g}");
        let _ = b; // small blue tint (~0.05) is expected
    }

    #[test]
    fn non_lava_has_no_emissive() {
        for mat in MaterialType::ALL {
            if mat == MaterialType::Lava {
                continue;
            }
            let [r, g, b] = mat.emissive_linear_rgb();
            assert_eq!(r, 0.0, "{mat:?} should have zero red emissive");
            assert_eq!(g, 0.0, "{mat:?} should have zero green emissive");
            assert_eq!(b, 0.0, "{mat:?} should have zero blue emissive");
        }
    }

    #[test]
    fn water_has_high_reflectance() {
        assert!(
            MaterialType::Water.reflectance() > 0.5,
            "water reflectance should be > 0.5 for vivid Fresnel rim"
        );
    }

    #[test]
    fn water_has_very_low_roughness() {
        assert!(
            MaterialType::Water.perceptual_roughness() < 0.1,
            "water should be nearly mirror-smooth"
        );
    }

    #[test]
    fn snow_has_low_roughness() {
        assert!(
            MaterialType::Snow.perceptual_roughness() < 0.2,
            "snow/ice should be smooth"
        );
    }

    #[test]
    fn stone_and_vegetation_are_rough_matte() {
        assert!(MaterialType::Stone.perceptual_roughness() > 0.8);
        assert!(MaterialType::Vegetation.perceptual_roughness() > 0.8);
        assert_eq!(MaterialType::Stone.metallic(), 0.0);
        assert_eq!(MaterialType::Vegetation.metallic(), 0.0);
    }
}
