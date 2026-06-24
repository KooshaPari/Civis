//! Reactive event→SFX trigger mapping (FR-CIV-AUDIO-005).
//!
//! Audio-direction §1 (Tier 3 — Reactive event SFX) specifies that
//! sim events (`Birth`, `Death`, `Build`, `Tech`, `Battle`, `Disaster`)
//! each map to a per-kind one-shot, with the disaster variant
//! branching by [`SfxKind::for_disaster_label`]. This module owns the
//! **substrate** half of that contract: given an abstract
//! [`SfxTrigger`], produce a batch of [`SfxRequest`]s ready to be
//! pushed into the [`SfxCoalescer`](crate::sfx::SfxCoalescer).
//!
//! The substrate deliberately does not depend on `civ-protocol-3d`'s
//! `EventFeedMessage3d` — that enum carries many wire fields the
//! audio layer does not need. The client (`clients/bevy-ref`) is
//! responsible for converting each `EventFeedMessage3d` into one
//! (or more) [`SfxTrigger`]s and forwarding them to
//! [`trigger_to_sfx_requests`]. This keeps the audio crate free of
//! engine deps and trivially testable with `--lib` only.
//!
//! ## Routing rules
//!
//! - `Birth` / `Death` / `Build` / `Tech` → one request, unit volume.
//! - `Battle { intensity }` → one request, volume = `intensity`
//!   clamped to `[0, 1]` (audio-direction §1 tier 3: "metal clash /
//!   volley; intensity-scaled volume").
//! - `Disaster { kind }` → one request at the per-kind sting
//!   ([`SfxKind::for_disaster_label`]). Volume = `severity` clamped
//!   to `[0, 1]`.
//! - `UiClick` / `UiHover` / `UiConfirm` / `UiCancel` / `UiAlert`
//!   → one request at unit volume. UI palette routing lives in
//!   [`crate::ui_sound`]; this function is the pure plumbing.
//!
//! `None` is never returned — every [`SfxTrigger`] produces at least
//! one request. An empty `intensity` / `severity` short-circuits to a
//! zero-volume request that the coalescer will drop silently,
//! matching the "graceful silence" pillar.

use serde::{Deserialize, Serialize};

use crate::sfx::{SfxKind, SfxRequest};

/// Abstract, substrate-level signal for a reactive one-shot SFX.
///
/// Mirrors the sim-side `EventFeedMessage3d` variants the audio
/// layer cares about (audio-direction §1 tier 3 trigger table).
/// The enum is **wire-stable** — append only.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(tag = "trigger", rename_all = "snake_case")]
pub enum SfxTrigger {
    /// A civilian was born (audio-direction §1: "soft warm gold rise").
    Birth,
    /// A civilian died (audio-direction §1: "low, brief, respectful
    /// — no jump-scare").
    Death,
    /// A building finished construction (audio-direction §1: "wood
    /// / stone thunk + small gold confirm tail").
    Build,
    /// A technology was researched (audio-direction §1: "bright
    /// crystalline cyan chime").
    Tech,
    /// A battle engagement fired; `intensity` in `[0, 1]` scales
    /// the request volume (audio-direction §1: "metal clash /
    /// volley; intensity-scaled volume").
    Battle {
        /// Engagement intensity in `[0, 1]`. 0.0 is suppressed.
        intensity: f32,
    },
    /// A disaster started; `kind` is the per-`DisasterKind` label
    /// and `severity` in `[0, 1]` scales the request volume
    /// (audio-direction §1: per-disaster variants).
    Disaster {
        /// Disaster label (e.g. `"meteor"`, `"flood"`, `"quake"`).
        /// Routed by [`SfxKind::for_disaster_label`].
        kind: &'static str,
        /// Severity in `[0, 1]`. 0.0 is suppressed.
        severity: f32,
    },
    /// UI button click (palette role handled by [`crate::ui_sound`]).
    UiClick,
    /// UI hover / focus.
    UiHover,
    /// UI positive confirm.
    UiConfirm,
    /// UI cancel / dismiss.
    UiCancel,
    /// UI alert / hazard notification (reserved, rarely played).
    UiAlert,
}

impl SfxTrigger {
    /// Map a trigger to its single target [`SfxKind`].
    ///
    /// `Battle` and `Disaster` route to the per-kind sting; `Disaster`
    /// goes through [`SfxKind::for_disaster_label`] so an unknown
    /// `kind` falls back to the umbrella `SfxKind::Disaster` (no
    /// panic, no skipped event).
    #[must_use]
    pub fn kind(self) -> SfxKind {
        match self {
            SfxTrigger::Birth => SfxKind::Birth,
            SfxTrigger::Death => SfxKind::Death,
            SfxTrigger::Build => SfxKind::Build,
            SfxTrigger::Tech => SfxKind::Tech,
            SfxTrigger::Battle { .. } => SfxKind::Battle,
            SfxTrigger::Disaster { kind, .. } => SfxKind::for_disaster_label(kind),
            SfxTrigger::UiClick => SfxKind::UiClick,
            SfxTrigger::UiHover => SfxKind::UiHover,
            SfxTrigger::UiConfirm => SfxKind::UiConfirm,
            SfxTrigger::UiCancel => SfxKind::UiCancel,
            SfxTrigger::UiAlert => SfxKind::UiAlert,
        }
    }

    /// The `volume` field for the [`SfxRequest`]. Clamps to
    /// `[0.0, 1.0]`. `0.0` requests are valid (the coalescer drops
    /// them silently).
    #[must_use]
    pub fn volume(self) -> f32 {
        match self {
            SfxTrigger::Birth
            | SfxTrigger::Death
            | SfxTrigger::Build
            | SfxTrigger::Tech
            | SfxTrigger::UiClick
            | SfxTrigger::UiHover
            | SfxTrigger::UiConfirm
            | SfxTrigger::UiCancel
            | SfxTrigger::UiAlert => 1.0,
            SfxTrigger::Battle { intensity } => intensity.clamp(0.0, 1.0),
            SfxTrigger::Disaster { severity, .. } => severity.clamp(0.0, 1.0),
        }
    }
}

/// Convert one [`SfxTrigger`] to the [`SfxRequest`] batch the
/// [`SfxCoalescer`](crate::sfx::SfxCoalescer) should drain. Always
/// returns exactly one request — the coalescer is responsible for
/// batching across many triggers.
///
/// # Examples
///
/// ```
/// use civ_audio::triggers::{trigger_to_sfx_requests, SfxTrigger};
///
/// // A meteor with severity 0.8 → a single Meteor-tinged SfxRequest
/// // at 0.8 volume.
/// let reqs = trigger_to_sfx_requests(SfxTrigger::Disaster {
///     kind: "meteor",
///     severity: 0.8,
/// });
/// assert_eq!(reqs.len(), 1);
/// assert!((reqs[0].volume - 0.8).abs() < 1e-5);
/// ```
#[must_use]
pub fn trigger_to_sfx_requests(trigger: SfxTrigger) -> Vec<SfxRequest> {
    vec![SfxRequest::with_volume(trigger.kind(), trigger.volume())]
}

/// Convert a batch of [`SfxTrigger`]s to a flat list of
/// [`SfxRequest`]s. Pure; does not touch the coalescer.
#[must_use]
pub fn triggers_to_sfx_requests(triggers: &[SfxTrigger]) -> Vec<SfxRequest> {
    let mut out = Vec::with_capacity(triggers.len());
    for &t in triggers {
        out.push(SfxRequest::with_volume(t.kind(), t.volume()));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn kind_routes_simple_triggers_to_their_target_sfx() {
        assert_eq!(SfxTrigger::Birth.kind(), SfxKind::Birth);
        assert_eq!(SfxTrigger::Death.kind(), SfxKind::Death);
        assert_eq!(SfxTrigger::Build.kind(), SfxKind::Build);
        assert_eq!(SfxTrigger::Tech.kind(), SfxKind::Tech);
        assert_eq!(
            SfxTrigger::Battle { intensity: 1.0 }.kind(),
            SfxKind::Battle
        );
        assert_eq!(SfxTrigger::UiClick.kind(), SfxKind::UiClick);
        assert_eq!(SfxTrigger::UiAlert.kind(), SfxKind::UiAlert);
    }

    #[test]
    fn disaster_kind_routes_through_for_disaster_label() {
        assert_eq!(
            SfxTrigger::Disaster {
                kind: "meteor",
                severity: 1.0
            }
            .kind(),
            SfxKind::Meteor
        );
        assert_eq!(
            SfxTrigger::Disaster {
                kind: "flood",
                severity: 0.0
            }
            .kind(),
            SfxKind::Flood
        );
        assert_eq!(
            SfxTrigger::Disaster {
                kind: "unknown",
                severity: 0.0
            }
            .kind(),
            SfxKind::Disaster
        );
    }

    #[test]
    fn volume_is_unit_for_simple_triggers_and_clamps_for_scaled() {
        assert!((SfxTrigger::Birth.volume() - 1.0).abs() < f32::EPSILON);
        assert!((SfxTrigger::Tech.volume() - 1.0).abs() < f32::EPSILON);
        assert!((SfxTrigger::Battle { intensity: 0.42 }.volume() - 0.42).abs() < f32::EPSILON);
        // Out-of-range clamps to the unit range.
        assert!(SfxTrigger::Battle { intensity: 1.5 }.volume() <= 1.0);
        assert!(SfxTrigger::Battle { intensity: -0.5 }.volume() >= 0.0);
        assert!(
            (SfxTrigger::Disaster {
                kind: "quake",
                severity: 0.5
            }
            .volume()
                - 0.5)
                .abs()
                < f32::EPSILON
        );
    }

    #[test]
    fn trigger_to_sfx_requests_yields_exactly_one_request() {
        let reqs = trigger_to_sfx_requests(SfxTrigger::Birth);
        assert_eq!(reqs.len(), 1);
        assert_eq!(reqs[0].kind, SfxKind::Birth);
        assert!((reqs[0].volume - 1.0).abs() < f32::EPSILON);
    }

    #[test]
    fn batch_conversion_preserves_order() {
        let triggers = [
            SfxTrigger::Birth,
            SfxTrigger::Build,
            SfxTrigger::Tech,
            SfxTrigger::Death,
        ];
        let reqs = triggers_to_sfx_requests(&triggers);
        assert_eq!(reqs.len(), 4);
        assert_eq!(reqs[0].kind, SfxKind::Birth);
        assert_eq!(reqs[3].kind, SfxKind::Death);
    }

    #[test]
    fn empty_batch_is_empty() {
        let reqs = triggers_to_sfx_requests(&[]);
        assert!(reqs.is_empty());
    }

    /// Covers FR-CIV-AUDIO-005 — the trigger→SFX request mapping.
    /// We assert:
    ///   1. A birth storm produces a single Birth request per
    ///      trigger (coalescer handles burst capping; the substrate
    ///      does not).
    ///   2. A battle scales volume with intensity.
    ///   3. Disaster routes by `kind` label and scales by `severity`.
    ///   4. Unknown disaster labels fall back to the umbrella
    ///      `SfxKind::Disaster` rather than panicking.
    #[test]
    fn fr_audio_005_event_triggers_route_to_sfx_requests() {
        // (1) Birth storm — many triggers, each yields a single
        // Birth-kind request at unit volume.
        let birth_storm: Vec<SfxTrigger> = (0..200).map(|_| SfxTrigger::Birth).collect();
        let reqs = triggers_to_sfx_requests(&birth_storm);
        assert_eq!(reqs.len(), 200);
        assert!(reqs.iter().all(|r| r.kind == SfxKind::Birth));
        assert!(reqs.iter().all(|r| (r.volume - 1.0).abs() < 1e-5));

        // (2) Battle scales volume with intensity.
        let battle = trigger_to_sfx_requests(SfxTrigger::Battle { intensity: 0.7 });
        assert_eq!(battle.len(), 1);
        assert_eq!(battle[0].kind, SfxKind::Battle);
        assert!((battle[0].volume - 0.7).abs() < 1e-5);

        // (3) Disaster routes by kind and scales by severity.
        let meteor = trigger_to_sfx_requests(SfxTrigger::Disaster {
            kind: "METEOR",
            severity: 0.5,
        });
        assert_eq!(meteor.len(), 1);
        assert_eq!(meteor[0].kind, SfxKind::Meteor);
        assert!((meteor[0].volume - 0.5).abs() < 1e-5);

        // (4) Unknown disaster label falls back to the umbrella.
        let unknown = trigger_to_sfx_requests(SfxTrigger::Disaster {
            kind: "hailstorm",
            severity: 0.3,
        });
        assert_eq!(unknown.len(), 1);
        assert_eq!(unknown[0].kind, SfxKind::Disaster);
        assert!((unknown[0].volume - 0.3).abs() < 1e-5);
    }
}
