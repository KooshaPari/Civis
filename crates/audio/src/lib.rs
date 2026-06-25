//! CIV-0800 audio substrate (pure-Rust core, no engine deps).
//!
//! Implements the spec-only AUDIO cluster flagged BUILD-NEXT in
//! `docs/audits/spec-only-triage-batch1.md` (FRs `FR-CIV-AUDIO-001..008`)
//! and `docs/design/audio-direction.md` §1–§3. The crate deliberately
//! avoids any audio engine (Bevy, kira, wgpu) so its math is testable
//! with `--lib` only — the Bevy client (`clients/bevy-ref/src/audio.rs`)
//! is the engine-bound consumer that wires the substrate to kira.
//!
//! ## What lives here
//!
//! - **Four-tier bus mix** ([`AudioMix`]) — Ambient / Score / Sfx / Ui
//!   under a single Master bus, each independently mutable, persisted as
//!   `serde` for settings round-trip. (FR-CIV-AUDIO-001)
//! - **Biome-driven ambient beds** ([`BiomeFootprint`] + [`AmbientBlend`]) —
//!   the camera footprint → normalised bed-weight vector → cross-fade
//!   targets, deterministic for a given footprint + cadence. (FR-CIV-AUDIO-002)
//! - **Weather / diurnal / seasonal modulation** of the bed weights
//!   (rain, snow, storm, day, winter). (FR-CIV-AUDIO-003)
//! - **Mood-driven score stems** ([`MoodVector`] + [`ScoreStem`]) — 4 stems
//!   remix on a continuous `{prosperity, growth, tension, wonder}` readout
//!   of sim aggregates + events, gain-only, no real-time DSP. (FR-CIV-AUDIO-004)
//! - **Reactive event SFX routing** ([`triggers`]) — sim event →
//!   `SfxRequest` batch, the substrate half of the contract. (FR-CIV-AUDIO-005)
//! - **SFX coalescing / clamp** ([`SfxCoalescer`]) — caps concurrent
//!   same-kind one-shots per frame, clamps summed gain to protect
//!   headroom and the "one accent at a time" pillar. (FR-CIV-AUDIO-006)
//! - **UI sound language** ([`ui_sound`]) — palette role mapping
//!   (cyan / gold / cool / acid) + per-tick UI sound budget. (FR-CIV-AUDIO-007)
//! - **Bus ducking** ([`ducking`]) — 150–400 ms tweens on disaster /
//!   milestone / ui-modal / combat-swell, layered with loudest-wins
//!   semantics. (FR-CIV-AUDIO-008)
//!
//! ## What does NOT live here
//!
//! - The kira / `bevy_kira_audio` plugin itself (it stays in
//!   `clients/bevy-ref/src/audio.rs` behind the `audio` feature).
//! - The CC0 sourcing plan / `assets/audio/CREDITS.md` (spec only — see
//!   `docs/design/audio-direction.md` §4).
//! - Graceful-silence invariant tests with 0 files (engine concern;
//!   the math here never touches the filesystem).
//!
//! ## Determinism
//!
//! Every public function is pure and `Copy`/`Clone`-friendly. Footprint
//! weights, mood readings, and SFX coalescing caps depend only on their
//! explicit inputs — no RNG, no `Instant::now`, no I/O. This is the
//! hard prerequisite for replay-bound audio test coverage.

#![deny(missing_docs)]
#![deny(unsafe_code)]

pub mod ambient;
pub mod bus;
pub mod ducking;
pub mod mix;
pub mod mood;
pub mod sfx;
pub mod triggers;
pub mod ui_sound;

pub use ambient::{AmbientBed, AmbientBlend, BedWeights, BiomeFootprint};
pub use bus::{BusId, BusLevels};
pub use mix::{AudioMix, AudioMixPreset, MIX_SCHEMA_VERSION};
pub use mood::{derive_music_cue, MusicCue, MoodVector, ScoreCadence, ScoreStem, StemMix};
pub use sfx::{SfxCoalescer, SfxKind, SfxQueue, SfxRequest, COALESCE_CAP_PER_KIND};

/// Marker version of this crate's public schema.
///
/// Bumped on breaking changes to any public type. Bumping is a no-op for
/// additive changes; tracked in the engine save format so a future audio
/// migration can detect older snapshots.
pub const SCHEMA_VERSION: &str = "0.1.0-audio-substrate";
