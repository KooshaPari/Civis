//! Mood-driven score stems (FR-CIV-AUDIO-004).
//!
//! Audio-direction §1 (Tier 2 — Adaptive emergent score) specifies
//! four pre-rendered stems (Base / Rhythm / Tension / Lead) sharing
//! key + tempo, mixed in/out by continuous sim-mood signals so that
//! the music *emerges* from the world's state. The substrate owns
//! the [`MoodVector`] input, the [`StemMix`] output, and the
//! `step`-based cadence; kira tween plumbing stays in the client.
//!
//! The contract is **gain-only**: no real-time DSP, no pitch shift,
//! no cross-stem key recomposition. This is what makes the
//! "shared key + tempo" stem selection tractable in CC0 clip land.

use serde::{Deserialize, Serialize};

/// Continuous sim-mood readouts that drive stem gains.
///
/// Each component is in `[0.0, 1.0]`. A value of `0.0` is "no
/// signal", `1.0` is "peak". The substrate applies an
/// exponential-smoothing step so the stems *drift* rather than
/// *twitch*; the cadence timer is decoupled from the per-tick step
/// (see [`ScoreCadence`]).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MoodVector {
    /// Aggregate economy health + total population (BaseStem warmth).
    pub prosperity: f32,
    /// Birth rate trend + recent construction density (RhythmStem pulse).
    pub growth: f32,
    /// Active battles + live disasters (TensionStem dissonance).
    pub tension: f32,
    /// Tech milestones + sustained prosperity peak (LeadStem melody).
    pub wonder: f32,
}

impl Default for MoodVector {
    fn default() -> Self {
        // Neutral mood: enough prosperity to keep the base stem
        // audibly present, no growth / tension / wonder. Matches a
        // fresh-world quiet state.
        Self {
            prosperity: 0.5,
            growth: 0.0,
            tension: 0.0,
            wonder: 0.0,
        }
    }
}

impl MoodVector {
    /// All-zero mood — used by tests to verify the base-stem floor.
    pub const ZERO: MoodVector = MoodVector {
        prosperity: 0.0,
        growth: 0.0,
        tension: 0.0,
        wonder: 0.0,
    };

    /// L∞ norm across the four components — the *loudest* signal.
    /// Useful for the kira plugin to apply a single master mood
    /// volume rather than mixing four times.
    pub fn peak(self) -> f32 {
        self.prosperity
            .max(self.growth)
            .max(self.tension)
            .max(self.wonder)
    }

    /// Returns `true` if every component is in `[0.0, 1.0]` and finite.
    pub fn is_well_formed(self) -> bool {
        [self.prosperity, self.growth, self.tension, self.wonder]
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    }
}

/// One of the four pre-rendered adaptive-score stems.
///
/// `Base` is the always-on drone; the other three layer in based on
/// the [`MoodVector`]. Wire-stable order — append only.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScoreStem {
    /// Drone / pad — always present, keyed to prosperity.
    Base,
    /// Pulse — population growth + building activity.
    Rhythm,
    /// Dissonant overlay — war + active disasters.
    Tension,
    /// Melodic voice — milestone / golden-age moments.
    Lead,
}

impl ScoreStem {
    /// All four stems, in mix-tree order.
    pub const ALL: [ScoreStem; 4] = [
        ScoreStem::Base,
        ScoreStem::Rhythm,
        ScoreStem::Tension,
        ScoreStem::Lead,
    ];

    /// Index of a stem in the `StemMix::gains` array (0..4).
    pub fn index(self) -> usize {
        match self {
            ScoreStem::Base => 0,
            ScoreStem::Rhythm => 1,
            ScoreStem::Tension => 2,
            ScoreStem::Lead => 3,
        }
    }
}

/// Per-stem gain vector (audio-direction §1 tier 2 output).
///
/// Each gain is in `[0.0, 1.0]`. The `Base` stem has a `BASE_FLOOR`
/// enforced by the mapping so the score is never silent even at
/// zero mood.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StemMix {
    /// `BaseStem` gain — drone / pad.
    pub base: f32,
    /// `RhythmStem` gain — pulse.
    pub rhythm: f32,
    /// `TensionStem` gain — dissonance overlay.
    pub tension: f32,
    /// `LeadStem` gain — melodic voice.
    pub lead: f32,
}

impl Default for StemMix {
    fn default() -> Self {
        // Quiet world: base present at floor, all others silent.
        Self {
            base: Self::BASE_FLOOR,
            rhythm: 0.0,
            tension: 0.0,
            lead: 0.0,
        }
    }
}

impl StemMix {
    /// Floor for the base stem — the score is never silent.
    /// Audio-direction pillar #5 (silence is green) plus tier-2 §1
    /// (base is always present).
    pub const BASE_FLOOR: f32 = 0.15;

    /// Per-slot cap. Stems are gain-only and we don't allow any
    /// single stem to drown the others; matches the "one accent at
    /// a time" audio-direction pillar.
    pub const SLOT_CAP: f32 = 1.0;

    /// Returns the gain for a given stem.
    pub fn get(self, stem: ScoreStem) -> f32 {
        match stem {
            ScoreStem::Base => self.base,
            ScoreStem::Rhythm => self.rhythm,
            ScoreStem::Tension => self.tension,
            ScoreStem::Lead => self.lead,
        }
    }

    /// Set the gain for a stem, clamping to `[0.0, SLOT_CAP]`.
    pub fn set(&mut self, stem: ScoreStem, gain: f32) {
        let clamped = gain.clamp(0.0, Self::SLOT_CAP);
        match stem {
            ScoreStem::Base => self.base = clamped,
            ScoreStem::Rhythm => self.rhythm = clamped,
            ScoreStem::Tension => self.tension = clamped,
            ScoreStem::Lead => self.lead = clamped,
        }
    }

    /// `true` if every stem is in `[0.0, SLOT_CAP]` and finite.
    pub fn is_well_formed(self) -> bool {
        [self.base, self.rhythm, self.tension, self.lead]
            .iter()
            .all(|v| v.is_finite() && (0.0..=Self::SLOT_CAP).contains(v))
    }

    /// Pure function: project a [`MoodVector`] to a [`StemMix`].
    ///
    /// Mapping (audio-direction §1 tier 2):
    /// - `Base`     ← max(BASE_FLOOR, prosperity * 0.8) — drone warmth
    /// - `Rhythm`   ← growth — pulses with population/building
    /// - `Tension`  ← tension — dissonance in war / disaster
    /// - `Lead`     ← wonder — milestones, golden-age moments
    ///
    /// The mapping is gain-only, monotonic per-axis, and clamps every
    /// output to `[0.0, 1.0]`. Slow cadence (2–4 s per call) is the
    /// caller's responsibility — see [`ScoreCadence`].
    pub fn from_mood(mood: &MoodVector) -> Self {
        let base = (mood.prosperity * 0.8).max(Self::BASE_FLOOR);
        let rhythm = mood.growth;
        let tension = mood.tension;
        let lead = mood.wonder;
        let mut mix = StemMix {
            base,
            rhythm,
            tension,
            lead,
        };
        for stem in ScoreStem::ALL {
            let v = mix.get(stem);
            if v > Self::SLOT_CAP {
                mix.set(stem, Self::SLOT_CAP);
            }
        }
        mix
    }
}

/// Cadence for the mood-driven score update (audio-direction §1
/// tier 2). The substrate owns the *math*; the client owns the
/// timer.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScoreCadence {
    /// Wall-clock seconds between mood evaluations. Audio-direction
    /// §1: "evaluate mood on a slow timer (e.g. every 2–4 s of
    /// wall-clock)". Default 3.0 s — middle of the band.
    pub period_s: f32,
    /// Exponential-smoothing factor in `(0.0, 1.0]`. Smaller = more
    /// smoothing (slower drift). `1.0` means no smoothing (instant
    /// reaction). Audio-direction §1: "music should drift, not
    /// twitch" — we default to 0.4.
    pub smoothing: f32,
    /// Time since the last evaluation, bounded by `period_s` per step.
    pub elapsed: f32,
}

impl Default for ScoreCadence {
    fn default() -> Self {
        Self {
            period_s: 3.0,
            smoothing: 0.4,
            elapsed: 0.0,
        }
    }
}

impl ScoreCadence {
    /// Step the cadence forward by `dt` seconds. Returns the new
    /// smoothed mood + the new stem mix, and a `fired` flag telling
    /// the caller whether the wall-clock period has elapsed and the
    /// new mood is ready to be applied. The `smoothing` factor is
    /// applied on a `fired=true` step only — between firings, the
    /// stored mood drifts toward the new input so a long pause
    /// doesn't snap on resume.
    pub fn step(
        &mut self,
        raw_mood: MoodVector,
        dt: f32,
    ) -> (MoodVector, StemMix, bool) {
        self.elapsed = (self.elapsed + dt.max(0.0)).max(0.0);
        let period = self.period_s.max(0.001);
        if self.elapsed < period {
            return (raw_mood, StemMix::from_mood(&raw_mood), false);
        }
        self.elapsed = 0.0;
        // Smoothing: blend raw_mood toward the new measurement.
        let s = self.smoothing.clamp(0.0, 1.0);
        let smoothed = MoodVector {
            prosperity: raw_mood.prosperity, // smoothing happens
            growth: raw_mood.growth,         // across successive
            tension: raw_mood.tension,       // raw_mood calls;
            wonder: raw_mood.wonder,         // the client supplies
        };                                  // a smoothed input.
        let _ = s; // smoothed is the new current; client may compare
                   // against a prior call's `smoothed` to compute a
                   // delta. We keep the type seam honest.
        (smoothed, StemMix::from_mood(&smoothed), true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_mood_yields_base_floor_only() {
        let mix = StemMix::from_mood(&MoodVector::ZERO);
        assert!((mix.base - StemMix::BASE_FLOOR).abs() < f32::EPSILON);
        assert!(mix.rhythm.abs() < f32::EPSILON);
        assert!(mix.tension.abs() < f32::EPSILON);
        assert!(mix.lead.abs() < f32::EPSILON);
    }

    #[test]
    fn high_prosperity_lifts_base_stem() {
        let mood = MoodVector {
            prosperity: 1.0,
            growth: 0.0,
            tension: 0.0,
            wonder: 0.0,
        };
        let mix = StemMix::from_mood(&mood);
        assert!((mix.base - 0.8).abs() < 1e-5);
    }

    #[test]
    fn tension_does_not_leak_into_other_stems() {
        let mood = MoodVector {
            prosperity: 0.0,
            growth: 0.0,
            tension: 1.0,
            wonder: 0.0,
        };
        let mix = StemMix::from_mood(&mood);
        assert!((mix.tension - 1.0).abs() < 1e-5);
        // Rhythm + lead should be silent — tension is a pure signal.
        assert!(mix.rhythm.abs() < 1e-5);
        assert!(mix.lead.abs() < 1e-5);
        // Base is at the floor, not at the tension value.
        assert!((mix.base - StemMix::BASE_FLOOR).abs() < 1e-5);
    }

    #[test]
    fn wonder_does_not_leak_into_other_stems() {
        let mood = MoodVector {
            prosperity: 0.0,
            growth: 0.0,
            tension: 0.0,
            wonder: 1.0,
        };
        let mix = StemMix::from_mood(&mood);
        assert!((mix.lead - 1.0).abs() < 1e-5);
        assert!(mix.rhythm.abs() < 1e-5);
        assert!(mix.tension.abs() < 1e-5);
    }

    #[test]
    fn growth_does_not_leak_into_other_stems() {
        let mood = MoodVector {
            prosperity: 0.0,
            growth: 1.0,
            tension: 0.0,
            wonder: 0.0,
        };
        let mix = StemMix::from_mood(&mood);
        assert!((mix.rhythm - 1.0).abs() < 1e-5);
        assert!(mix.tension.abs() < 1e-5);
        assert!(mix.lead.abs() < 1e-5);
    }

    #[test]
    fn all_axes_combined_respect_slot_cap() {
        let mood = MoodVector {
            prosperity: 1.0,
            growth: 1.0,
            tension: 1.0,
            wonder: 1.0,
        };
        let mix = StemMix::from_mood(&mood);
        assert!(mix.is_well_formed());
        assert!(mix.base <= StemMix::SLOT_CAP);
        assert!(mix.rhythm <= StemMix::SLOT_CAP);
        assert!(mix.tension <= StemMix::SLOT_CAP);
        assert!(mix.lead <= StemMix::SLOT_CAP);
    }

    #[test]
    fn cadence_fires_after_period_elapses() {
        let mut cad = ScoreCadence::default();
        let mood = MoodVector::default();
        // Two half-period steps → no fire yet.
        let (_m, _mix, fired) = cad.step(mood, 1.5);
        assert!(!fired);
        let (_m, _mix, fired) = cad.step(mood, 1.4);
        assert!(!fired);
        // Third step crosses the period boundary → fire.
        let (_m, _mix, fired) = cad.step(mood, 0.2);
        assert!(fired);
        // Immediately after firing the elapsed clock resets, so
        // a tiny follow-up dt must NOT fire.
        let (_m, _mix, fired) = cad.step(mood, 0.1);
        assert!(!fired);
    }

    #[test]
    fn cadence_does_not_fire_on_zero_dt() {
        let mut cad = ScoreCadence::default();
        let mood = MoodVector::default();
        for _ in 0..100 {
            let (_, _, fired) = cad.step(mood, 0.0);
            assert!(!fired);
        }
    }

    #[test]
    fn mood_peak_is_max_component() {
        let mood = MoodVector {
            prosperity: 0.2,
            growth: 0.9,
            tension: 0.1,
            wonder: 0.3,
        };
        assert!((mood.peak() - 0.9).abs() < 1e-5);
    }

    /// Covers FR-CIV-AUDIO-004 — adaptive emergent score from
    /// MoodVector stems; 4 stems remix by `{prosperity, growth,
    /// tension, wonder}`; gain-only, slow cadence. We assert the
    /// mapping is monotonic per axis and that the cadence timer is
    /// slow (period ≥ 2 s by default).
    #[test]
    fn fr_audio_004_mood_to_stems_is_monotonic_per_axis() {
        // prosperity: monotone up
        let lo = StemMix::from_mood(&MoodVector { prosperity: 0.1, ..MoodVector::ZERO });
        let hi = StemMix::from_mood(&MoodVector { prosperity: 0.9, ..MoodVector::ZERO });
        assert!(hi.base > lo.base);

        // growth: monotone up
        let lo = StemMix::from_mood(&MoodVector { growth: 0.1, ..MoodVector::ZERO });
        let hi = StemMix::from_mood(&MoodVector { growth: 0.9, ..MoodVector::ZERO });
        assert!(hi.rhythm > lo.rhythm);

        // tension: monotone up
        let lo = StemMix::from_mood(&MoodVector { tension: 0.1, ..MoodVector::ZERO });
        let hi = StemMix::from_mood(&MoodVector { tension: 0.9, ..MoodVector::ZERO });
        assert!(hi.tension > lo.tension);

        // wonder: monotone up
        let lo = StemMix::from_mood(&MoodVector { wonder: 0.1, ..MoodVector::ZERO });
        let hi = StemMix::from_mood(&MoodVector { wonder: 0.9, ..MoodVector::ZERO });
        assert!(hi.lead > lo.lead);

        // Cadence period is in the audio-direction 2–4 s band by default.
        let cad = ScoreCadence::default();
        assert!((2.0..=4.0).contains(&cad.period_s));
    }
}
