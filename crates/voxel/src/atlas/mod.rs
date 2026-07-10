//! GPU texture atlas packers.
//!
//! This module is the Phase-3 placement companion to the [`crate::material_pbr`]
//! policy substrate. The packer at [`gpu_atlas::GreedyAtlasPacker`] is a
//! pure-Rust, `std`-only 2D shelf-bin packer that the Bevy adapter feeds to
//! the GPU at startup to build the resolution-free material atlas.
//!
//! The module intentionally has no dependency on `bevy`, `wgpu`, or any
//! asset loader — it only deals with bytes and rectangles. The Bevy
//! adapter translates the returned `PackedAtlas::placements: HashMap<String, Rect>`
//! into `wgpu::TextureCopyTextureInfo` source rects at upload time.
//!
//! ## Layout
//!
//! - [`gpu_atlas`] — the packer itself, plus its `Rect`, `PackedAtlas`,
//!   `InputTexture`, `AtlasError`, and `AtlasConfig` types. Unit tests for
//!   1, 5, 50 texture cases live in the same file under
//!   `#[cfg(test)] mod tests`.

pub mod gpu_atlas;

pub use gpu_atlas::{AtlasError, InputTexture, PackedAtlas, Rect};
