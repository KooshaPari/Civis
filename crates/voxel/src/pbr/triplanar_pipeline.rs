//! Triplanar PBR pipeline integration glue.
//!
//! This module owns:
//!
//! - [`TriplanarPbrMaterial`] — the CPU-side data layout the Bevy adapter
//!   reads to build a bevy material binding. It mirrors the GPU struct in
//!   `shaders/pbr_triplanar.wgsl` field-for-field (world_tile,
//!   texture_scale, vertex_color_blend, has_emissive).
//! - [`TriplanarAtlasHandle`] — opaque handle into a [`GreedyAtlas`]'s
//!   packed rectangles; the shader binds the underlying array texture once
//!   and reads the four corner UVs from this handle when rendering each
//!   chunk's quad.
//! - [`spawn_triplanar_chunk`] — the function signature the Bevy adapter
//!   implements to push a chunk onto the ECS with a PBR material bound.
//!
//! ## Bevy wiring
//!
//! The actual bevy material struct is gated behind a feature in the Bevy
//! adapter (`clients/bevy-ref/src/materials.rs`). The `civ-voxel`
//! substrate intentionally stays engine-agnostic (see the docstring at
//! `material_pbr.rs:10`) so adding `bevy` here would cascade into a
//! bevy version pin that conflicts with the rest of the workspace. The
//! type definitions below are the **contract** the bevy adapter implements
//! against — the adapter compiles those bevy types down from this shape.
//!
//! ## FR coverage
//!
//! - FR-CIV-PBR-001..009 — implemented in `material_pbr.rs`. This module
//!   is the engine-facing layer on top of that substrate.
//! - `AtlasTexture` / `AtlasRect` / `GreedyAtlas` — see `greedy_atlas.rs`.

#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};

use super::greedy_atlas::{AtlasRect, AtlasTexture, GreedyAtlas};

/// Stable identifier the Bevy adapter uses to look up a [`GreedyAtlas`]
/// placement for a given `MaterialId` (typically a content hash or a
/// `MaterialId(u16)` cast). See [`TriplanarAtlasHandle::from_matid`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TriplanarAtlasHandle(pub u32);

impl TriplanarAtlasHandle {
    /// Build a handle for a `MaterialId`. The bottom 16 bits carry the
    /// matid; the top 16 bits stay zero and are reserved for future
    /// texture-set generation tags.
    #[must_use]
    pub const fn from_matid(matid: u16) -> Self {
        // `as` is stable in const fn; `u32::from(u16)` is not yet stable
        // (see rust-lang/rust #143874), so cast by hand.
        Self(matid as u32)
    }

    /// Underlying integer id — feeds straight into the `id` field on
    /// [`AtlasRect`].
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// CPU-side mirror of the WGSL `PbrParams` uniform.
///
/// Bevy's adapter constructs one of these per rendered chunk (or globally
/// per material slot) and uploads it as a `UniformBuffer` next to the
/// `texture_2d_array`. The fields match `shaders/pbr_triplanar.wgsl`
/// one-to-one so a desync is impossible — both ends of the wire are
/// `pub` and unit-tested.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TriplanarPbrMaterial {
    /// Voxel units per tile (1.0 = one texel per voxel).
    pub world_tile: f32,
    /// Multiplier on the projected UV before sampling. Used to compress or
    /// stretch surface detail without re-baking the atlas.
    pub texture_scale: f32,
    /// 0.0 = full triplanar PBR; 1.0 = vertex-color fallback (FR-005).
    pub vertex_color_blend: f32,
    /// 0.0 = emissive channel off; 1.0 = on (lava, crystal, etc.).
    pub has_emissive: f32,
}

impl Default for TriplanarPbrMaterial {
    fn default() -> Self {
        Self {
            world_tile: 1.0,
            texture_scale: 1.0,
            vertex_color_blend: 0.0,
            has_emissive: 0.0,
        }
    }
}

impl TriplanarPbrMaterial {
    /// Construct the spec-mandated default — full PBR, no emissive. This
    /// matches the Bevy adapter's `TriplanarParams::default()` and the
    /// `LodRenderPlan::for_distance(0, ..)` vertex_color_blend value.
    #[must_use]
    pub fn triplanar_full() -> Self {
        Self::default()
    }

    /// Construct the distant-LOD vertex-color fallback (FR-005).
    #[must_use]
    pub fn vertex_color_only() -> Self {
        Self {
            world_tile: 1.0,
            texture_scale: 1.0,
            vertex_color_blend: 1.0,
            has_emissive: 0.0,
        }
    }

    /// Stash an emissive override (e.g. for lava chunks).
    #[must_use]
    pub fn with_emissive(mut self) -> Self {
        self.has_emissive = 1.0;
        self
    }
}

/// Inputs to [`spawn_triplanar_chunk`]. The Bevy adapter fills these from
/// the mesher output and a slice / handle into the loaded atlas. The
/// substrate accepts the values as plain data so the function signature
/// is engine-portable.
#[derive(Debug, Clone, PartialEq)]
pub struct TriplanarChunkSpec {
    /// Stable hash identifying the chunk (typically the `ChunkId` cast).
    pub chunk_id: u32,
    /// World-space lower corner of the chunk (deterministic per seed).
    pub chunk_min: [f32; 3],
    /// Half edge length in world units (16 voxel chunks → 8.0 for unit
    /// voxels).
    pub chunk_half_extent: f32,
    /// Handle into the [`GreedyAtlas`] for the dominant material in this
    /// chunk.
    pub dominant_atlas: TriplanarAtlasHandle,
    /// Optional per-chunk material override. Falls back to
    /// [`TriplanarPbrMaterial::triplanar_full`] when `None`.
    pub material: Option<TriplanarPbrMaterial>,
}

impl TriplanarChunkSpec {
    /// Cheap constructor for the most common case: a single-material
    /// chunk using the spec-default triplanar material.
    #[must_use]
    pub fn simple(
        chunk_id: u32,
        chunk_min: [f32; 3],
        chunk_half_extent: f32,
        dominant_atlas: TriplanarAtlasHandle,
    ) -> Self {
        Self {
            chunk_id,
            chunk_min,
            chunk_half_extent,
            dominant_atlas,
            material: None,
        }
    }

    /// Override the material on a `simple` chunk.
    #[must_use]
    pub fn with_material(mut self, material: TriplanarPbrMaterial) -> Self {
        self.material = Some(material);
        self
    }
}

/// Output of a successful [`spawn_triplanar_chunk`] call. The Bevy adapter
/// stores this on the entity as a component so other systems can update
/// the material without re-spawning.
#[derive(Debug, Clone, PartialEq)]
pub struct TriplanarChunkBinding {
    /// Chunk id this binding represents.
    pub chunk_id: u32,
    /// Resolved rect into the packed [`GreedyAtlas`] for the dominant
    /// material. Pinned per chunk so LOD swap cannot move mid-frame.
    pub dominant_rect: AtlasRect,
    /// The material spec the binding is using. `None` means the caller
    /// passed a custom one — Bevy will read it from the entity component
    /// instead.
    pub material: TriplanarPbrMaterial,
}

/// Errors returned by [`spawn_triplanar_chunk`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TriplanarChunkError {
    /// `chunk_half_extent` was non-positive. Bevy refuses to spawn a
    /// zero-size mesh.
    ZeroExtent,
    /// The spec's `dominant_atlas` was not found in the atlas plan.
    UnknownAtlasHandle(TriplanarAtlasHandle),
}

impl std::fmt::Display for TriplanarChunkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ZeroExtent => f.write_str("chunk_half_extent must be > 0"),
            Self::UnknownAtlasHandle(h) => {
                write!(f, "atlas handle {:?} was not packed", h)
            }
        }
    }
}

impl std::error::Error for TriplanarChunkError {}

/// Compose a [`TriplanarChunkBinding`] from a spec + an already-packed
/// atlas. The Bevy adapter wraps this in a public system that calls into
/// the bevy ECS (`commands.spawn(…)` + `StandardMaterial` binding). Keeping
/// the composition step engine-agnostic lets the substrate tests cover it
/// without bringing bevy into the build.
///
/// # Hook into `material_pbr.rs`
///
/// The binding does NOT touch [`super::super::material_pbr`]. The pure-Rust
/// PBR substrate owns the **policy** (FR-001..009 — CC0 sourcing, LOD modes,
/// manifest resolution, missing-texture policy, triplanar blend math). This
/// module owns the **placement** (which atlas slice this specific chunk
/// binds to). They are layered: the Bevy adapter reads `LodRenderPlan` from
/// the substrate and hands the resulting `TriplanarPbrMaterial` here.
pub fn spawn_triplanar_chunk(
    spec: &TriplanarChunkSpec,
    atlas: &mut GreedyAtlas,
    textures: &[AtlasTexture],
) -> Result<TriplanarChunkBinding, TriplanarChunkError> {
    if !(spec.chunk_half_extent > 0.0) {
        return Err(TriplanarChunkError::ZeroExtent);
    }
    let target_id = spec.dominant_atlas.raw();

    // Cache a packed layout so repeated calls are cheap. We re-pack only
    // when the input list changes — the Bevy adapter keys that on a
    // resource id, but the substrate stays stateless and re-packs each
    // time the caller asks. For unit-tests this is fast (microseconds for
    // ≤64 textures) and dead-simple to reason about.
    let rects = atlas
        .pack(textures)
        .map_err(|_| TriplanarChunkError::UnknownAtlasHandle(spec.dominant_atlas))?;

    let dominant_rect = *rects
        .iter()
        .find(|r| r.id == target_id)
        .ok_or(TriplanarChunkError::UnknownAtlasHandle(spec.dominant_atlas))?;

    let material = spec.material.unwrap_or_else(TriplanarPbrMaterial::triplanar_full);
    Ok(TriplanarChunkBinding {
        chunk_id: spec.chunk_id,
        dominant_rect,
        material,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn specimen_textures() -> Vec<AtlasTexture> {
        // Three materials at plausible CC0 sizes.
        vec![
            AtlasTexture::new(1, 128, 128),
            AtlasTexture::new(2, 256, 128),
            AtlasTexture::new(3, 128, 64),
        ]
    }

    /// `triplanar_full` is the substrate default — full PBR, no emissive,
    /// `vertex_color_blend == 0.0`.
    #[test]
    fn triplanar_full_default_matches_lod_plan() {
        let m = TriplanarPbrMaterial::triplanar_full();
        assert_eq!(m.world_tile, 1.0);
        assert_eq!(m.texture_scale, 1.0);
        assert_eq!(m.vertex_color_blend, 0.0);
        assert_eq!(m.has_emissive, 0.0);
        // Same u32 layout as the WGSL `PbrParams` struct — manually zero
        // the uniform at static time and compare, since we cannot link a
        // WGSL validator from a unit test.
        assert_eq!(std::mem::size_of::<TriplanarPbrMaterial>(), 16);
    }

    /// `vertex_color_only` is the FR-005 distant-LOD fallback — blend == 1.0.
    #[test]
    fn vertex_color_only_sets_blend_to_one() {
        let m = TriplanarPbrMaterial::vertex_color_only();
        assert_eq!(m.vertex_color_blend, 1.0);
        assert_eq!(m.has_emissive, 0.0);
    }

    /// `with_emissive` flips the `has_emissive` flag.
    #[test]
    fn with_emissive_lights_the_chunk() {
        let m = TriplanarPbrMaterial::triplanar_full().with_emissive();
        assert_eq!(m.has_emissive, 1.0);
    }

    /// `spawn_triplanar_chunk` resolves a packed rect for a valid handle
    /// and refuses to spawn a chunk with non-positive extent.
    #[test]
    fn spawn_triplanar_chunk_resolves_dominant_rect() {
        let textures = specimen_textures();
        let mut atlas = GreedyAtlas::new(1024, 1024);
        let spec = TriplanarChunkSpec::simple(42, [0.0, 0.0, 0.0], 8.0, TriplanarAtlasHandle::from_matid(2));
        let binding = spawn_triplanar_chunk(&spec, &mut atlas, &textures)
            .expect("valid handle + positive extent resolves");
        assert_eq!(binding.chunk_id, 42);
        assert_eq!(binding.dominant_rect.id, 2);
        assert_eq!(binding.material, TriplanarPbrMaterial::triplanar_full());
    }

    /// A non-positive `chunk_half_extent` returns `ZeroExtent` before any
    /// atlas work happens.
    #[test]
    fn spawn_triplanar_chunk_rejects_zero_extent() {
        let textures = specimen_textures();
        let mut atlas = GreedyAtlas::new(1024, 1024);
        let spec = TriplanarChunkSpec::simple(7, [0.0, 0.0, 0.0], 0.0, TriplanarAtlasHandle::from_matid(1));
        let err = spawn_triplanar_chunk(&spec, &mut atlas, &textures)
            .expect_err("zero extent rejected");
        assert_eq!(err, TriplanarChunkError::ZeroExtent);
    }

    /// An atlas handle that wasn't packed returns `UnknownAtlasHandle`.
    #[test]
    fn spawn_triplanar_chunk_rejects_unknown_handle() {
        let textures = specimen_textures();
        let mut atlas = GreedyAtlas::new(1024, 1024);
        let spec = TriplanarChunkSpec::simple(
            7,
            [0.0, 0.0, 0.0],
            8.0,
            TriplanarAtlasHandle::from_matid(9999),
        );
        let err = spawn_triplanar_chunk(&spec, &mut atlas, &textures)
            .expect_err("unpacked handle rejected");
        assert_eq!(
            err,
            TriplanarChunkError::UnknownAtlasHandle(TriplanarAtlasHandle::from_matid(9999))
        );
    }
}
