# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial project scaffolding and governance setup.

## [Unreleased] - Phase 3

### Added
- **FR-CIV-PBR-010 — Triplanar WGSL shader** at `assets/shaders/pbr_triplanar.wgsl`. The shader samples albedo, normal, and metallic-roughness from three orthogonal world-axis projections (X+/X-, Y+/Y-, Z+/Z-) and blends them by squared-and-normalized world-normal weights. An optional detail layer adds a second triplanar set at lower weight for close-up terrain texturing without UV stretching. The shader reuses the existing PBR lighting math from `pbr.wgsl` and exposes `@group(0) @binding(...)` uniforms for every sampler, scalar, and transform.
- **FR-CIV-PBR-011 — Greedy 2D atlas packer** at `crates/voxel/src/atlas/gpu_atlas.rs`. Pure-Rust, `std`-only shelf-bin packing (`shelf height = max(rect heights on shelf)`) that returns a packed `atlas_texture: Vec<u8>` plus a `placements: HashMap<String, Rect>`. Default atlas size is 4096×4096 RGBA; configurable via `GreedyAtlasConfig::new(max_w, max_h)`. Ships with a `pack_to_png(path)` debug helper and unit tests covering 1, 5, and 50 texture cases.

### Changed
- `crates/voxel/src/material_pbr.rs` now exposes `MaterialCatalog::material_count()`, a Phase-3 accessor that returns the union of distinct `MaterialId`s across the triplanar splat plan and the greedy atlas plan. The Bevy adapter uses this to size its scratch bind group before the first frame.
- `crates/voxel/src/lib.rs` declares the new `pub mod atlas` module so the packer is reachable from the crate root.
- Module-level docs in `material_pbr.rs` describe the Phase-3 WGSL + atlas integration and link the new types (`MaterialCatalog`, `crate::atlas::gpu_atlas::GreedyAtlas`).

### Deprecated

### Removed

### Fixed

### Security

[Unreleased]: https://github.com/KooshaPari/Civis/compare/HEAD...HEAD
