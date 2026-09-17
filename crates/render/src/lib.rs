//! Render substrate for Civis (CIV-0800 audio integration, CIV-0500 frame budget).
//!
//! Pure-Rust core for the audio integration layer and frame budget
//! planning. No GPU or engine dependencies at this layer; the client
//! (`clients/bevy-ref/`) wires these types to actual Kira audio and
//! wgpu rendering.
//!
//! ## Modules
//!
//! - [`audio`] — Kira-backed background music (`KiraMusic`), layered
//!   cross-fades (`MusicLayers`), and reactive SFX dispatch (`SfxTriggerEvent`).
//!   Implements FR-AUD-001, FR-AUD-002, FR-AUD-003.
//! - [`frame`] — Frame budget planning for 60 fps / 1080p target.
//!   Implements FR-PERF-003.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod audio;
pub mod frame;
