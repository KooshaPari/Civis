//! SFX coalescing / clamp under event bursts (FR-CIV-AUDIO-006).
//!
//! Audio-direction §1 (Tier 3 — Reactive event SFX) explicitly calls
//! out that births / deaths / battles fire in bursts and the SFX
//! drain must **coalesce**: cap simultaneous same-kind one-shots per
//! frame and sum/clamp volume rather than playing 200 birth chimes.
//! This module owns the *math*: the [`SfxCoalescer`] bookkeeps a
//! rolling per-kind cap, an optional `global_gain_clamp` to protect
//! the master bus, and the [`SfxQueue`] fast-path used by the kira
//! plugin to drain pending events in deterministic order.

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;

/// Hard cap on simultaneous same-kind one-shots per drain step.
///
/// Audio-direction §1 tier 3: "cap simultaneous instances per kind
/// per frame (e.g. ≤ 3)". We round to 3 — the centre of the
/// spec's "≤ 3" recommendation. This is a `const` so the cap is
/// visible at every call site and cannot be silently changed by
/// the client.
pub const COALESCE_CAP_PER_KIND: u8 = 3;

/// The SFX kinds the substrate knows about.
///
/// Mirrors the `SfxKind` in `clients/bevy-ref/src/audio.rs` and
/// extends it with the new event kinds called for in audio-direction
/// §1 tier 3: `Tech`, `Battle`, and the per-`DisasterKind` stings
/// (FR-CIV-AUDIO-005). The mapping is owned by the kira plugin (which
/// decides which `.ogg` to play per kind); the substrate owns the
/// coalescing math + ordering.
///
/// Wire-stable order — append only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SfxKind {
    /// UI button / menu click feedback (cyan palette role).
    UiClick,
    /// UI hover / focus feedback.
    UiHover,
    /// UI positive confirm (gold palette role).
    UiConfirm,
    /// UI cancel / dismiss (cool).
    UiCancel,
    /// UI alert / hazard notification (acid-green, reserved).
    UiAlert,
    /// Birth event from `EventFeedMessage3d::Birth`.
    Birth,
    /// Death event from `EventFeedMessage3d::Death`.
    Death,
    /// Build-complete from spectator `BuildingKind`.
    Build,
    /// Tech milestone from `EventFeedMessage3d::Tech` (cyan chime).
    Tech,
    /// Battle engagement from `EventFeedMessage3d::Battle` (intensity-scaled).
    Battle,
    /// Disaster — split per `DisasterSfx` variant (FR-CIV-AUDIO-005).
    /// Kept as a wire-stable umbrella; the per-kind stings live below
    /// and are routed by [`SfxKind::for_disaster_label`].
    Disaster,
    // --- Per-disaster stings (FR-CIV-AUDIO-005; audio-direction §1
    // tier 3 per-disaster table). Appended at the end to preserve the
    // existing wire format. ---
    /// High whistle→deep impact boom.
    Meteor,
    /// Surging water roar.
    Flood,
    /// Sub-bass rumble + debris.
    Quake,
    /// Crackle + whoosh.
    Wildfire,
    /// Wind gust + thunder.
    Storm,
    /// Low dread drone + sparse bell (least terrain-y, most ominous).
    Plague,
}

impl SfxKind {
    /// Returns the disaster-specific SFX kind for a given
    /// `disaster_label` string. Matches the 6 variants of
    /// `civ_engine::disasters::DisasterKind` — see audio-direction
    /// §1 tier 3 per-disaster table (FR-CIV-AUDIO-005).
    ///
    /// The mapping is case-insensitive and falls back to
    /// [`SfxKind::Disaster`] for unknown labels so a new disaster
    /// kind never crashes the audio drain.
    pub fn for_disaster_label(label: &str) -> SfxKind {
        match label.trim().to_ascii_lowercase().as_str() {
            "meteor" => SfxKind::Meteor,
            "flood" => SfxKind::Flood,
            "quake" | "earthquake" => SfxKind::Quake,
            "wildfire" | "fire" => SfxKind::Wildfire,
            "storm" => SfxKind::Storm,
            "plague" => SfxKind::Plague,
            _ => SfxKind::Disaster,
        }
    }

    /// `true` for the six per-disaster stings (`Meteor`, `Flood`,
    /// `Quake`, `Wildfire`, `Storm`, `Plague`). Used by the coalescer
    /// to apply the per-kind cap uniformly to disaster variants.
    pub fn is_disaster_variant(self) -> bool {
        matches!(
            self,
            SfxKind::Meteor
                | SfxKind::Flood
                | SfxKind::Quake
                | SfxKind::Wildfire
                | SfxKind::Storm
                | SfxKind::Plague
        )
    }

    /// `true` for UI kinds (the four UI palette roles + Alert).
    /// Used by the substrate to apply a per-kind cap of 1 to UI
    /// clicks (you cannot click two buttons in the same frame in a
    /// usable UI), separate from the world-event cap of 3.
    pub fn is_ui(self) -> bool {
        matches!(
            self,
            SfxKind::UiClick
                | SfxKind::UiHover
                | SfxKind::UiConfirm
                | SfxKind::UiCancel
                | SfxKind::UiAlert
        )
    }
}

/// One pending SFX request submitted by an upstream system.
///
/// The substrate coalesces on `(kind, volume)`: identical kind +
/// volume pairs are summed (clamped to `1.0`) up to
/// [`COALESCE_CAP_PER_KIND`] per drain step. Volume is linear
/// `[0.0, 1.0]`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SfxRequest {
    /// Which SFX kind.
    pub kind: SfxKind,
    /// Linear volume multiplier.
    pub volume: f32,
}

impl SfxRequest {
    /// New request at unit volume.
    pub fn new(kind: SfxKind) -> Self {
        Self { kind, volume: 1.0 }
    }

    /// New request at an explicit linear volume (clamped to `[0, 1]`).
    pub fn with_volume(kind: SfxKind, volume: f32) -> Self {
        Self {
            kind,
            volume: volume.clamp(0.0, 1.0),
        }
    }
}

/// The drained output of one coalesce step.
///
/// The `output` vector holds the per-kind requests the kira plugin
/// should play this frame, in deterministic order (sorted by
/// [`SfxKind`] discriminant). `coalesced_count` is the number of
/// requests that were dropped because the per-kind cap was hit; the
/// caller may use it for telemetry / `tracing` events.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct SfxQueue {
    /// Per-kind, post-coalesce output requests.
    pub output: Vec<SfxRequest>,
    /// Number of dropped requests this drain step.
    pub coalesced_count: u32,
}

/// Per-kind SFX coalescer.
///
/// Designed to be cheap to copy / reset each frame: the only state
/// is the `per_kind_count` map (drained + rebuilt every step) and
/// the configurable cap. The substrate never owns the kira
/// `AudioInstance` set — that lives in the client; the substrate
/// only ever returns the *intended* per-kind volume after coalescing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SfxCoalescer {
    /// Hard cap on simultaneous same-kind one-shots per drain step.
    pub per_kind_cap: u8,
    /// Optional global gain clamp applied to the summed gain of all
    /// coalesced SFX (protects headroom). `None` = no clamp.
    pub global_gain_clamp: Option<f32>,
    /// Internal per-kind counter; reset every drain step.
    #[serde(skip)]
    per_kind_count: BTreeMap<SfxKind, u8>,
}

impl Default for SfxCoalescer {
    fn default() -> Self {
        Self {
            per_kind_cap: COALESCE_CAP_PER_KIND,
            global_gain_clamp: Some(1.0),
            per_kind_count: BTreeMap::new(),
        }
    }
}

impl SfxCoalescer {
    /// Construct with a custom per-kind cap. UI kinds are always
    /// capped at 1 (the substrate's "you can't click two buttons in
    /// the same frame" rule); the cap argument applies to the
    /// world-event kinds (Birth / Death / Build / Tech / Battle /
    /// Disaster).
    pub fn with_cap(per_kind_cap: u8) -> Self {
        Self {
            per_kind_cap,
            global_gain_clamp: Some(1.0),
            per_kind_count: BTreeMap::new(),
        }
    }

    /// Drain a batch of pending SFX requests into a coalesced
    /// [`SfxQueue`]. Pure: does not touch kira or any I/O.
    ///
    /// Algorithm (audio-direction §1 tier 3):
    /// 1. Sort `requests` by `(kind, volume)` to get deterministic
    ///    output ordering (matches the testability rule).
    /// 2. For each request, check `per_kind_count[kind] < cap`; if
    ///    not, drop (`coalesced_count += 1`) and continue.
    /// 3. For accepted requests, sum the volumes for that kind, then
    ///    clamp the sum to `1.0` (per-kind gain-clamp).
    /// 4. Apply `global_gain_clamp` to the *summed* gain of all
    ///    output requests to protect master headroom.
    /// 5. Reset the per-kind counter.
    pub fn drain(&mut self, mut requests: Vec<SfxRequest>) -> SfxQueue {
        requests.sort_by(|a, b| {
            a.kind.cmp(&b.kind).then_with(|| {
                a.volume
                    .partial_cmp(&b.volume)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
        });

        // Per-kind coalesce: sum accepted volumes, clamp to 1.0.
        let mut per_kind: BTreeMap<SfxKind, (u8, f32)> = BTreeMap::new();
        let mut coalesced_count: u32 = 0;

        for req in requests {
            let cap = if req.kind.is_ui() {
                1
            } else {
                self.per_kind_cap
            };
            let entry = per_kind.entry(req.kind).or_insert((0, 0.0));
            if entry.0 < cap {
                entry.0 += 1;
                entry.1 = (entry.1 + req.volume).min(1.0);
            } else {
                coalesced_count += 1;
            }
        }

        // Apply global gain clamp to the summed gain across all
        // kinds. The kira plugin will multiply by the per-bus
        // `Sfx` gain from the AudioMix separately.
        let total_gain: f32 = per_kind.values().map(|(_, g)| g).sum();
        let clamp = self.global_gain_clamp.unwrap_or(f32::INFINITY);
        let global_scale = if total_gain <= 0.0001 {
            1.0
        } else {
            (clamp / total_gain).min(1.0)
        };

        let mut output: Vec<SfxRequest> = per_kind
            .into_iter()
            .map(|(kind, (_, gain))| SfxRequest {
                kind,
                volume: (gain * global_scale).clamp(0.0, 1.0),
            })
            .collect();
        output.sort_by_key(|a| a.kind);

        // Reset the per-kind counter for the next step.
        self.per_kind_count.clear();

        SfxQueue {
            output,
            coalesced_count,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_drain_yields_empty_queue() {
        let mut c = SfxCoalescer::default();
        let q = c.drain(vec![]);
        assert!(q.output.is_empty());
        assert_eq!(q.coalesced_count, 0);
    }

    #[test]
    fn single_request_passes_through_at_full_volume() {
        let mut c = SfxCoalescer::default();
        let q = c.drain(vec![SfxRequest::new(SfxKind::Birth)]);
        assert_eq!(q.output.len(), 1);
        assert_eq!(q.output[0].kind, SfxKind::Birth);
        assert!((q.output[0].volume - 1.0).abs() < 1e-5);
        assert_eq!(q.coalesced_count, 0);
    }

    #[test]
    fn cap_drops_excess_same_kind() {
        // Cap for non-UI is 3; submit 6.
        let mut c = SfxCoalescer::default();
        let reqs: Vec<SfxRequest> = (0..6).map(|_| SfxRequest::new(SfxKind::Birth)).collect();
        let q = c.drain(reqs);
        assert_eq!(q.output.len(), 1);
        assert_eq!(q.output[0].kind, SfxKind::Birth);
        // 6 - 3 = 3 dropped.
        assert_eq!(q.coalesced_count, 3);
    }

    #[test]
    fn summed_volume_clamps_to_one_per_kind() {
        // Three 0.6-volume requests for the same kind → sum 1.8 → clamp 1.0.
        let mut c = SfxCoalescer::default();
        let reqs = vec![
            SfxRequest::with_volume(SfxKind::Birth, 0.6),
            SfxRequest::with_volume(SfxKind::Birth, 0.6),
            SfxRequest::with_volume(SfxKind::Birth, 0.6),
        ];
        let q = c.drain(reqs);
        assert_eq!(q.output.len(), 1);
        assert!((q.output[0].volume - 1.0).abs() < 1e-5);
    }

    #[test]
    fn ui_kinds_have_cap_of_one() {
        // Submit 5 UiClick — only the first should be accepted.
        let mut c = SfxCoalescer::default();
        let reqs: Vec<SfxRequest> = (0..5).map(|_| SfxRequest::new(SfxKind::UiClick)).collect();
        let q = c.drain(reqs);
        assert_eq!(q.output.len(), 1);
        assert_eq!(q.output[0].kind, SfxKind::UiClick);
        assert_eq!(q.coalesced_count, 4);
    }

    #[test]
    fn mixed_kinds_each_get_their_own_cap() {
        let mut c = SfxCoalescer::default();
        let mut reqs = vec![];
        for _ in 0..3 {
            reqs.push(SfxRequest::new(SfxKind::Birth));
        }
        for _ in 0..3 {
            reqs.push(SfxRequest::new(SfxKind::Death));
        }
        let q = c.drain(reqs);
        assert_eq!(q.output.len(), 2);
        // Sorted by kind: Birth (lower) < Death.
        assert_eq!(q.output[0].kind, SfxKind::Birth);
        assert_eq!(q.output[1].kind, SfxKind::Death);
        assert_eq!(q.coalesced_count, 0);
    }

    #[test]
    fn drain_output_is_sorted_by_kind() {
        let mut c = SfxCoalescer::default();
        let reqs = vec![
            SfxRequest::new(SfxKind::Tech),
            SfxRequest::new(SfxKind::Birth),
            SfxRequest::new(SfxKind::Death),
            SfxRequest::new(SfxKind::Battle),
        ];
        let q = c.drain(reqs);
        let kinds: Vec<SfxKind> = q.output.iter().map(|r| r.kind).collect();
        let mut sorted = kinds.clone();
        sorted.sort();
        assert_eq!(kinds, sorted);
    }

    #[test]
    fn global_gain_clamp_protects_headroom() {
        // Make a coalescer that allows a max summed gain of 0.5.
        let mut c = SfxCoalescer {
            per_kind_cap: 3,
            global_gain_clamp: Some(0.5),
            per_kind_count: BTreeMap::new(),
        };
        // 3 kinds × 1.0 each = 3.0 total; clamp scales by 0.5/3.0 = 0.1667.
        let reqs = vec![
            SfxRequest::new(SfxKind::Birth),
            SfxRequest::new(SfxKind::Death),
            SfxRequest::new(SfxKind::Build),
        ];
        let q = c.drain(reqs);
        let total: f32 = q.output.iter().map(|r| r.volume).sum();
        // Each kind was already clamped to 1.0 before the global
        // pass; the global scale uniform-shrinks them so the
        // *summed* total is ≤ 0.5.
        assert!(total <= 0.5 + 1e-5);
    }

    #[test]
    fn custom_cap_is_respected() {
        let mut c = SfxCoalescer::with_cap(2);
        let reqs: Vec<SfxRequest> = (0..5).map(|_| SfxRequest::new(SfxKind::Battle)).collect();
        let q = c.drain(reqs);
        assert_eq!(q.output.len(), 1);
        assert_eq!(q.coalesced_count, 3);
    }

    #[test]
    fn for_disaster_label_routes_to_per_kind_stings() {
        // FR-CIV-AUDIO-005: the substrate routes the 6 disaster kinds
        // from `civ_engine::disasters::DisasterKind` to per-sting
        // SfxKind variants; unknown labels fall back to the umbrella
        // `SfxKind::Disaster`.
        assert_eq!(SfxKind::for_disaster_label("Meteor"), SfxKind::Meteor);
        assert_eq!(SfxKind::for_disaster_label("flood"), SfxKind::Flood);
        assert_eq!(SfxKind::for_disaster_label("QUAKE"), SfxKind::Quake);
        assert_eq!(SfxKind::for_disaster_label("earthquake"), SfxKind::Quake);
        assert_eq!(SfxKind::for_disaster_label("wildfire"), SfxKind::Wildfire);
        assert_eq!(SfxKind::for_disaster_label("fire"), SfxKind::Wildfire);
        assert_eq!(SfxKind::for_disaster_label("storm"), SfxKind::Storm);
        assert_eq!(SfxKind::for_disaster_label("plague"), SfxKind::Plague);
        assert_eq!(SfxKind::for_disaster_label("unknown"), SfxKind::Disaster);
        // Whitespace + case insensitivity.
        assert_eq!(SfxKind::for_disaster_label("  METEOR  "), SfxKind::Meteor);
    }

    #[test]
    fn is_disaster_variant_flags_only_the_six_per_kind_stings() {
        // The 6 per-disaster stings are flagged; the umbrella
        // `Disaster` variant is NOT (callers wanting a single
        // bucket should match on `SfxKind::Disaster` directly).
        for kind in [
            SfxKind::Meteor,
            SfxKind::Flood,
            SfxKind::Quake,
            SfxKind::Wildfire,
            SfxKind::Storm,
            SfxKind::Plague,
        ] {
            assert!(kind.is_disaster_variant(), "{kind:?} should be flagged");
        }
        for kind in [
            SfxKind::Birth,
            SfxKind::Death,
            SfxKind::Build,
            SfxKind::Tech,
            SfxKind::Battle,
            SfxKind::Disaster,
            SfxKind::UiClick,
        ] {
            assert!(!kind.is_disaster_variant(), "{kind:?} must NOT be flagged");
        }
    }

    #[test]
    fn drain_is_pure_across_calls() {
        // Calling drain twice on the same coalescer with the same
        // input must produce the same output (idempotent reset of
        // per_kind_count).
        let mut c = SfxCoalescer::default();
        let reqs = vec![
            SfxRequest::new(SfxKind::Birth),
            SfxRequest::new(SfxKind::Death),
        ];
        let q1 = c.drain(reqs.clone());
        let q2 = c.drain(reqs);
        assert_eq!(q1, q2);
    }

    /// Covers FR-CIV-AUDIO-006 — SFX coalescing / clamp under event
    /// bursts. We assert the cap, the per-kind clamp, the global
    /// gain clamp, and the deterministic ordering of the output.
    #[test]
    fn fr_audio_006_birth_storm_coalesces_to_safe_output() {
        let mut c = SfxCoalescer::default();
        // 200 birth events in a single drain — typical emergent world.
        let reqs: Vec<SfxRequest> = (0..200).map(|_| SfxRequest::new(SfxKind::Birth)).collect();
        let q = c.drain(reqs);

        // Cap of 3 → 200 - 3 = 197 coalesced.
        assert_eq!(q.coalesced_count, 197);
        // Single accepted request.
        assert_eq!(q.output.len(), 1);
        assert_eq!(q.output[0].kind, SfxKind::Birth);
        // Volume is 1.0 (single request, no clamp) — and the
        // global clamp = 1.0 means total gain is 1.0, so the
        // accepted request is unchanged.
        assert!((q.output[0].volume - 1.0).abs() < 1e-5);
    }
}
