//! Engine-adjacent PBR substrate: triplanar atlas packer + WGSL shader
//! integration layer.
//!
//! Layered on top of the pure-Rust policy substrate in
//! [`crate::material_pbr`]:
//!
//! - [`material_pbr`](../material_pbr/index.html) owns the **policy** — CC0
//!   sourcing (FR-001), LOD render-mode selection (FR-005), canonical-vs-
//!   primitive manifest mode (FR-007), missing-texture policy (FR-008),
//!   channel-map routing (FR-002), triplanar splat blending (FR-003),
//!   triplanar math helper (FR-009), greedy-mesh atlas plan (FR-004), and
//!   color-space policy (FR-006).
//! - This module owns the **placement** — a 2D greedy shelf bin packer
//!   ([`greedy_atlas`]) for triplanar surface textures and an engine-portable
//!   binding shape ([`triplanar_pipeline`]) the Bevy adapter compiles down
//!   to a real `bevy_pbr` material.
//!
//! The WGSL fragment shader for the triplanar surface lives at
//! `../shaders/pbr_triplanar.wgsl` (loaded by the Bevy adapter with
//! `include_str!` — kept out of `src/` so the substrate can compile on
//! toolchains without file-loading).
//!
//! ## FR coverage
//!
//! - FR-CIV-PBR-001..009 — see `material_pbr`.
//! - This module is the Phase-3 slice that hooks the substrate into the GPU
//!   side. It does NOT introduce any new FR; it consumes the substrate.

pub mod greedy_atlas;
pub mod triplanar_pipeline;

/// Canonical path to the WGSL fragment shader. The Bevy adapter reads the
/// shader source with `include_str!` at compile time and feeds it into
/// `bevy::pbr::MaterialDescriptor`. The path is a `pub const` so the
/// adapter does not have to duplicate the convention.
pub const TRIPLANAR_WGSL_PATH: &str = "shaders/pbr_triplanar.wgsl";

#[doc(inline)]
pub use greedy_atlas::{AtlasError, AtlasRect, AtlasTexture, GreedyAtlas, Vec2};
#[doc(inline)]
pub use triplanar_pipeline::{
    spawn_triplanar_chunk, TriplanarAtlasHandle, TriplanarChunkBinding, TriplanarChunkError,
    TriplanarChunkSpec, TriplanarPbrMaterial,
};
