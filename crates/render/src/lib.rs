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
//!   Implements `FR-CIV-AUDIO-004` (adaptive score from mood stems).
//! - [`frame`] — Frame budget planning for 60 fps / 1080p target.
//! - [`hex_map`] — Hex grid model and 60 fps draw-list generation.
//! - [`camera`] — RTS camera pan, zoom, and rect selection.
//! - [`timeline`] — Tick-history ring and scrubber rewind.
//! - [`lod`] — Single-frame level-of-detail transitions.
//! - [`state`] — Event-derived UI state (no engine polling).
//! - [`atlas`] — SVG build-time rasterisation and per-LOD atlas packing.
//! - [`gltf`] — glTF 2.0 lazy asset loading.
//!
//! ## Traceability provenance
//!
//! This crate previously advertised implementations of `FR-UX-001..005`,
//! `FR-ASSET-001..004`, `FR-AUD-001..003`, and `FR-PERF-003`. None of those
//! ids is defined by any authoritative spec: their only definition was a
//! table in `docs/traceability/TRACEABILITY_MATRIX.md` that cited three
//! nonexistent spec files. Several of them (`FR-UX-001..005`) collide with
//! unrelated real requirements in `docs/models/civ-sim/USER_SPEC.md`, so the
//! tags were actively false. They have been removed. The surviving tag,
//! `FR-CIV-AUDIO-004`, is defined in `docs/design/audio-direction.md` and
//! is the one audio requirement this crate genuinely implements.
//!
//! See `docs/audits/id-provenance-corrections.md` for the full record.

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
