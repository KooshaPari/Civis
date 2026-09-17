//! Render substrate for Civis (CIV-0800 audio integration, CIV-0500 frame
//! budget, CIV-0300 UI, CIV-0600 assets).
//!
//! Pure-Rust core for the audio integration layer, frame budget planning, the
//! event-driven hex-map UI, and the 2D/3D asset pipeline. No GPU or engine
//! dependencies live at this layer; the client (`clients/bevy-ref/`) wires
//! these types to actual Kira audio and wgpu rendering.
//!
//! ## Modules
//!
//! - [`audio`] — Kira-backed background music (`KiraMusic`), layered
//!   cross-fades (`MusicLayers`), and reactive SFX dispatch (`SfxTriggerEvent`).
//!   Implements FR-AUD-001, FR-AUD-002, FR-AUD-003.
//! - [`frame`] — Frame budget planning for 60 fps / 1080p target.
//!   Implements FR-PERF-003.
//! - [`hex_map`] — Hex grid model and 60 fps draw-list generation.
//!   Implements FR-UX-001.
//! - [`camera`] — RTS camera pan, zoom, and rect selection.
//!   Implements FR-UX-002.
//! - [`timeline`] — Tick-history ring and scrubber rewind.
//!   Implements FR-UX-003.
//! - [`lod`] — Single-frame level-of-detail transitions.
//!   Implements FR-UX-004.
//! - [`state`] — Event-derived UI state (no engine polling).
//!   Implements FR-UX-005.
//! - [`atlas`] — SVG build-time rasterisation and per-LOD atlas packing.
//!   Implements FR-ASSET-001, FR-ASSET-002, FR-ASSET-003.
//! - [`gltf`] — glTF 2.0 lazy asset loading.
//!   Implements FR-ASSET-004.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod atlas;
pub mod audio;
pub mod camera;
pub mod frame;
pub mod gltf;
pub mod hex_map;
pub mod lod;
pub mod state;
pub mod timeline;
