//! Emergent psyche state for civilian agents.
//!
//! Psyche is not an authored personality taxonomy. It is a compact vector
//! updated from genetics, lived needs, and culture exposure through social
//! ties.

use rand::Rng;
use serde::{Deserialize, Serialize};

use civ_genetics::{sentience::CognitionTraitProfile, Dna};

use crate::{culture, Needs};

/// Shared psyche vector width.
pub const PSYCHE_DIM: usize = 4;

/// Reactivity/sociability/risk/impulsivity temperament.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Temperament {
    /// How strongly mood swings in response to events.
    pub reactivity: f32,
    /// Baseline pull toward social contact.
    pub sociability: f32,
    /// Willingness to tolerate safety risk.
    pub risk_tol: f32,
    /// Distance/effort discount in planning.
    pub impulsivity: f32,
}

impl Temperament {
    /// Neutral baseline temperament.
    #[must_use]
    pub fn neutral() -> Self {
        Self {
            reactivity: 0.5,
            sociability: 0.5,
            risk_tol: 0.5,
            impulsivity: 0.5,
        }
    }
}

/// Fast-moving affect state.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Mood {
    /// Valence from `-1.0` misery to `+1.0` contentment.
    pub valence: f32,
    /// Arousal from calm to agitated.
    pub arousal: f32,
}

impl Mood {
    /// Neutral mood.
    #[must_use]
    pub fn neutral() -> Self {
        Self {
            valence: 0.0,
            arousal: 0.0,
        }
    }
}

/// Compact psyche vector for one agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Psyche {
    /// Stable need-biasing drives.
    pub drives: [f32; PSYCHE_DIM],
    /// Reaction style.
    pub temperament: Temperament,
    /// Current affect.
    pub mood: Mood,
    /// Culture-sampled personal beliefs.
    pub beliefs: [f32; PSYCHE_DIM],
    /// Maturity in `[0, 1]`.
    pub maturity: f32,
}

/// Data-driven genome projection for psyche axes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PsychGenomeProfile {
    /// DNA byte slots for the four drive axes.
    pub drive_slots: [Vec<(usize, f32)>; PSYCHE_DIM],
    /// DNA byte slots for reactivity.
    pub reactivity_slots: Vec<(usize, f32)>,
    /// DNA byte slots for sociability.
    pub sociability_slots: Vec<(usize, f32)>,
    /// DNA byte slots for risk tolerance.
    pub risk_slots: Vec<(usize, f32)>,
    /// DNA byte slots for impulsivity.
    pub impulsivity_slots: Vec<(usize, f32)>,
}

impl PsychGenomeProfile {
    /// A compact default projection that mirrors the sentience trait pattern.
    #[must_use]
    pub fn default_profile() -> Self {
        Self {
            drive_slots: [
                vec![(0, 1.0), (8, 0.5)],
                vec![(1, 1.0), (9, 0.5)],
                vec![(2, 1.0), (10, 0.5)],
                vec![(3, 1.0), (11, 0.5)],
            ],
            reactivity_slots: vec![(12, 1.0), (13, 0.5)],
            sociability_slots: vec![(14, 1.0), (15, 0.5)],
            risk_slots: vec![(16, 1.0), (17, 0.5)],
            impulsivity_slots: vec![(18, 1.0), (19, 0.5)],
        }
    }
}

fn clamp01(value: f32) -> f32 {
    value.clamp(0.0, 1.0)
}

fn clamp11(value: f32) -> f32 {
    value.clamp(-1.0, 1.0)
}

fn score_axis(dna: &Dna, slots: &[(usize, f32)]) -> f32 {
    let profile = CognitionTraitProfile::new("psyche-axis", slots.to_vec());
    civ_genetics::sentience::cognition_score(dna, &profile)
}

/// Default psyche genome projection.
#[must_use]
pub fn psych_genome_profile() -> PsychGenomeProfile {
    PsychGenomeProfile::default_profile()
}

/// Build a psyche vector from DNA and a projection profile.
#[must_use]
pub fn psyche_from_dna(dna: &Dna, profile: &PsychGenomeProfile) -> Psyche {
    Psyche {
        drives: profile
            .drive_slots
            .clone()
            .map(|slots| score_axis(dna, &slots)),
        temperament: Temperament {
            reactivity: score_axis(dna, &profile.reactivity_slots),
            sociability: score_axis(dna, &profile.sociability_slots),
            risk_tol: score_axis(dna, &profile.risk_slots),
            impulsivity: score_axis(dna, &profile.impulsivity_slots),
        },
        mood: Mood::neutral(),
        beliefs: [0.5; PSYCHE_DIM],
        maturity: 0.0,
    }
}

/// Update temperament with a small lived-experience nudge.
///
/// Defensive against non-finite inputs: `maturity`, `recent_mood_variance`,
/// and `recent_social_satisfaction` are sanitized before use. Final writes
/// are reset to neutral (0.5) if they would otherwise be NaN/Inf, so an
/// upstream overflow cannot pin the temperament vector to a poisoned state.
pub fn nudge_temperament(
    temperament: &mut Temperament,
    recent_mood_variance: f32,
    recent_social_satisfaction: f32,
    maturity: f32,
) {
    let maturity = sanitize_finite(maturity, 0.0);
    let mood_var = sanitize_finite(recent_mood_variance, temperament.reactivity);
    let social = sanitize_finite(recent_social_satisfaction, temperament.sociability);
    let plasticity = (1.0 - maturity * 0.8).clamp(0.0, 1.0);
    let lr = 0.002 * plasticity;
    let new_reactivity =
        clamp01(temperament.reactivity + lr * (mood_var - temperament.reactivity));
    let new_sociability =
        clamp01(temperament.sociability + lr * (social - temperament.sociability));
    temperament.reactivity = sanitize_finite(new_reactivity, 0.5);
    temperament.sociability = sanitize_finite(new_sociability, 0.5);
}

/// Replace a non-finite f32 (NaN/±Inf) with `fallback`; finite values pass
/// through unchanged. Used as the final assignment step for any f32 field
/// whose value was produced by arithmetic on the hot path — a single NaN
/// poisons every downstream hash, so we treat non-finite results as a
/// defensive reset to the safe default.
#[inline]
fn sanitize_finite(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        fallback
    }
}

/// Update mood from needs plus a social-event term.
///
/// Defensive against non-finite inputs: if any computed `mood` value is NaN
/// or ±Inf (which can happen with extreme RNG seeds, overflowing event
/// terms, or upstream NaN in `needs` / `temperament`), the result is reset
/// to neutral (0.0) rather than poisoning the psyche vector. Finite,
/// in-range updates behave identically to the pre-guard implementation.
pub fn update_mood(
    mood: &mut Mood,
    needs: &Needs,
    temperament: &Temperament,
    threat_pressure: f32,
    delta_needs: f32,
    event_term: f32,
) {
    // Sanitize inputs so a single non-finite upstream value cannot cascade
    // through the multiplications below. Safe defaults: 0.0 for additive
    // signal terms (neutral perturbation); current mood fields for state
    // values (so the function still moves mood toward a stable state).
    let needs_food = sanitize_finite(needs.food, 0.5);
    let needs_shelter = sanitize_finite(needs.shelter, 0.5);
    let needs_safety = sanitize_finite(needs.safety, 0.5);
    let needs_belonging = sanitize_finite(needs.belonging, 0.5);
    let reactivity = sanitize_finite(temperament.reactivity, 0.5);
    let threat = sanitize_finite(threat_pressure, 0.0);
    let delta = sanitize_finite(delta_needs, 0.0);
    let event = sanitize_finite(event_term, 0.0);

    let need_valence = ((needs_food + needs_shelter + needs_safety + needs_belonging) / 4.0 - 0.5)
        .clamp(-1.0, 1.0);
    let target_val = clamp11(need_valence + 0.25 * event);
    let lr = 0.12 * (0.5 + reactivity);
    let new_valence = clamp11(mood.valence + (target_val - mood.valence) * lr);
    let new_arousal =
        (threat + delta.abs() + 0.25 * event.abs()).clamp(0.0, 1.0);
    // Final guard: never assign a non-finite value to the model. Reset
    // to neutral (0.0) if anything went sideways through the pipeline.
    mood.valence = sanitize_finite(new_valence, 0.0);
    mood.arousal = sanitize_finite(new_arousal, 0.0);
}

/// Blend beliefs toward a culture exposure vector with a small mutational wobble.
///
/// Defensive against a non-finite `sociability` (which would otherwise
/// propagate NaN through `lr` and every output component). The final
/// belief vector is scrubbed so any non-finite component is reset to 0.5
/// (the canonical neutral belief).
pub fn update_beliefs(
    beliefs: &mut [f32; PSYCHE_DIM],
    exposure: [f32; PSYCHE_DIM],
    sociability: f32,
    rng: &mut impl Rng,
) {
    let soc = sanitize_finite(sociability, 0.5).clamp(0.0, 1.0);
    let lr = 0.08 * soc;
    let mut mixed = [0.0; PSYCHE_DIM];
    for i in 0..PSYCHE_DIM {
        let b = sanitize_finite(beliefs[i], 0.5);
        let e = sanitize_finite(exposure[i], 0.5);
        mixed[i] = clamp01(b + (e - b) * lr);
        let jitter = (rng.gen::<f32>() - 0.5) * 0.02;
        mixed[i] = clamp01(mixed[i] + jitter);
    }
    let result = culture::mutate_traits(rng, mixed, 0.01);
    for (slot, value) in beliefs.iter_mut().zip(result.into_iter()) {
        *slot = sanitize_finite(value, 0.5);
    }
}

/// Expose the culture vector sampled through an agent's social ties.
///
/// Defensive: any non-finite component in the output (from a non-finite
/// `weight`, a non-finite `trait[i]`, or a divide-by-zero edge case in the
/// normalization pass) is reset to 0.5 (neutral). Finite outputs are
/// unchanged, so in-range inputs see the pre-guard behavior.
#[must_use]
pub fn belief_culture_exposure(exposures: &[(f32, [f32; PSYCHE_DIM])]) -> [f32; PSYCHE_DIM] {
    let mut out = [0.5; PSYCHE_DIM];
    let mut total = 0.0;
    for (weight, traits) in exposures {
        if !weight.is_finite() || *weight <= 0.0 {
            continue;
        }
        total += *weight;
        for i in 0..PSYCHE_DIM {
            if traits[i].is_finite() {
                out[i] += traits[i] * *weight;
            }
        }
    }
    if total > 0.0 {
        for value in &mut out {
            // `total + 1.0` is always positive (total is a sum of
            // positive finite f32s from the loop above, so total+1.0 is
            // strictly > 0 and finite), making the division safe.
            *value = sanitize_finite(clamp01(*value / (total + 1.0)), 0.5);
        }
    }
    // Final scrub: any output component that is still non-finite (e.g.
    // a NaN that survived all the checks) is reset to 0.5.
    for value in &mut out {
        *value = sanitize_finite(*value, 0.5);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    fn rng(seed: u64) -> rand_chacha::ChaCha8Rng {
        rand_chacha::ChaCha8Rng::seed_from_u64(seed)
    }

    #[test]
    fn genetics_projection_is_bounded_and_sibling_sensitive() {
        let profile = psych_genome_profile();
        let mut a = Dna::zero(64);
        let mut b = Dna::zero(64);
        a.0[0] = 255;
        b.0[0] = 255;
        b.0[1] = 255;
        let pa = psyche_from_dna(&a, &profile);
        let pb = psyche_from_dna(&b, &profile);
        assert!(pa.drives.iter().all(|v| (0.0..=1.0).contains(v)));
        assert!(pb.drives.iter().all(|v| (0.0..=1.0).contains(v)));
        assert_ne!(pa, pb);
    }

    #[test]
    fn mood_tracks_needs_and_reactivity() {
        let needs = Needs {
            food: 0.0,
            shelter: 0.0,
            safety: 0.0,
            belonging: 0.0,
        };
        let mut low = Mood::neutral();
        let mut high = Mood::neutral();
        let low_t = Temperament {
            reactivity: 0.0,
            sociability: 0.5,
            risk_tol: 0.5,
            impulsivity: 0.5,
        };
        let high_t = Temperament {
            reactivity: 1.0,
            ..low_t
        };
        for _ in 0..5 {
            update_mood(&mut low, &needs, &low_t, 0.2, 0.4, 0.0);
            update_mood(&mut high, &needs, &high_t, 0.2, 0.4, 0.0);
        }
        assert!(low.valence < 0.0);
        assert!(high.valence < low.valence);
        assert!(high.arousal >= low.arousal);
    }

    #[test]
    fn beliefs_move_toward_culture_exposure() {
        let mut beliefs = [0.0, 0.0, 0.0, 0.0];
        let exposure = belief_culture_exposure(&[(1.0, [1.0, 0.5, 0.25, 0.75])]);
        update_beliefs(&mut beliefs, exposure, 1.0, &mut rng(7));
        assert!(beliefs.iter().all(|v| (0.0..=1.0).contains(v)));
        assert!(beliefs[0] > 0.0);
    }

    #[test]
    fn temperament_nudges_but_stays_bounded() {
        let mut temperament = Temperament::neutral();
        nudge_temperament(&mut temperament, 1.0, 0.0, 0.2);
        assert!(temperament.reactivity >= 0.0 && temperament.reactivity <= 1.0);
        assert!(temperament.sociability >= 0.0 && temperament.sociability <= 1.0);
    }

    /// L5-116 FR-CIV-NA/INF-GUARD: `update_mood` is a saturating guard on
    /// the psyche hot path. Driving the inputs with adversarial finite +
    /// non-finite values (NaN, ±Inf) MUST keep `mood.valence` in
    /// `[-1.0, 1.0]` and `mood.arousal` in `[0.0, 1.0]`; both must
    /// remain finite. This is the property that prevents a single bad
    /// RNG seed or upstream overflow from poisoning the simulation hash.
    ///
    /// We sweep 1000 random f32 inputs (per the spec) plus the canonical
    /// non-finite corner cases to make sure the saturation holds under
    /// proptest-style adversarial coverage.
    #[test]
    fn update_mood_saturates_on_overflow() {
        use rand::Rng;
        let mut rng = rng(0xDEAD_BEEF_C0FFEE_42u64);

        let corner_event_terms: [f32; 9] = [
            f32::NAN,
            f32::INFINITY,
            f32::NEG_INFINITY,
            f32::MAX,
            f32::MIN,
            0.0,
            1.0,
            -1.0,
            // subnormal edge case
            f32::from_bits(1),
        ];
        let corner_threats: [f32; 5] = [f32::NAN, f32::INFINITY, 0.0, 1.0, -1.0];
        let corner_deltas: [f32; 5] = [f32::NAN, f32::INFINITY, 0.0, 0.5, -0.5];
        let corner_needs_vals: [f32; 5] = [f32::NAN, f32::INFINITY, 0.0, 0.5, 1.5];
        let corner_reactivities: [f32; 5] = [f32::NAN, f32::INFINITY, -1.0, 0.5, 5.0];

        let mut mood = Mood::neutral();
        let mut needs = Needs {
            food: 0.5,
            shelter: 0.5,
            safety: 0.5,
            belonging: 0.5,
        };
        let mut temperament = Temperament::neutral();

        // Step 1: corner cases — guarantees the saturation guards catch
        // every non-finite path explicitly.
        for &event in &corner_event_terms {
            for &threat in &corner_threats {
                for &delta in &corner_deltas {
                    for &food in &corner_needs_vals {
                        for &react in &corner_reactivities {
                            needs.food = food;
                            temperament.reactivity = react;
                            update_mood(
                                &mut mood,
                                &needs,
                                &temperament,
                                threat,
                                delta,
                                event,
                            );
                            assert!(
                                mood.valence.is_finite(),
                                "mood.valence non-finite: event={event} threat={threat} delta={delta} food={food} react={react}"
                            );
                            assert!(
                                mood.arousal.is_finite(),
                                "mood.arousal non-finite: event={event} threat={threat} delta={delta} food={food} react={react}"
                            );
                            assert!(
                                (-1.0..=1.0).contains(&mood.valence),
                                "mood.valence out of [-1,1]: {} (event={event} threat={threat} delta={delta} food={food} react={react})",
                                mood.valence
                            );
                            assert!(
                                (0.0..=1.0).contains(&mood.arousal),
                                "mood.arousal out of [0,1]: {} (event={event} threat={threat} delta={delta} food={food} react={react})",
                                mood.arousal
                            );
                        }
                    }
                }
            }
        }

        // Step 2: 1000 random f32 inputs as specified in the proptest
        // description. We treat the test as a property check: every
        // random input must leave mood finite and in range.
        for _ in 0..1000 {
            let event: f32 = rng.gen();
            let threat: f32 = rng.gen();
            let delta: f32 = rng.gen();
            let food: f32 = rng.gen();
            let react: f32 = rng.gen();
            needs.food = food;
            temperament.reactivity = react;
            update_mood(&mut mood, &needs, &temperament, threat, delta, event);
            assert!(mood.valence.is_finite());
            assert!(mood.arousal.is_finite());
            assert!((-1.0..=1.0).contains(&mood.valence));
            assert!((0.0..=1.0).contains(&mood.arousal));
        }
    }

    /// L5-116 — `nudge_temperament` must also stay bounded + finite under
    /// non-finite inputs. NaN reactivity or NaN maturity must NOT
    /// propagate into the temperament vector.
    #[test]
    fn nudge_temperament_saturates_on_overflow() {
        let mut temperament = Temperament::neutral();
        for (mood_var, social, maturity) in [
            (f32::NAN, f32::NAN, f32::NAN),
            (f32::INFINITY, f32::INFINITY, f32::INFINITY),
            (f32::NEG_INFINITY, 0.0, 1.0),
            (1.0, -1.0, 5.0),
            (-1.0, 1.0, -1.0),
        ] {
            nudge_temperament(&mut temperament, mood_var, social, maturity);
            assert!(temperament.reactivity.is_finite());
            assert!(temperament.sociability.is_finite());
            assert!((0.0..=1.0).contains(&temperament.reactivity));
            assert!((0.0..=1.0).contains(&temperament.sociability));
        }
    }

    /// L5-116 — `belief_culture_exposure` must produce a finite
    /// `[f32; PSYCHE_DIM]` from any (weight, traits) input, even when
    /// individual entries are NaN/Inf.
    #[test]
    fn belief_culture_exposure_saturates_on_overflow() {
        // Empty exposures — output stays at the neutral [0.5; 4].
        let out = belief_culture_exposure(&[]);
        for v in out {
            assert!(v.is_finite());
            assert!((0.0..=1.0).contains(&v));
        }

        // Non-finite weights / traits — should be dropped, output
        // should be finite and in range.
        let exposures = [
            (f32::NAN, [0.0, 0.0, 0.0, 0.0]),
            (f32::INFINITY, [0.5, 0.5, 0.5, 0.5]),
            (1.0, [f32::NAN, f32::INFINITY, 0.0, 1.0]),
            (0.5, [f32::NEG_INFINITY, 0.5, 0.5, 0.5]),
        ];
        let out = belief_culture_exposure(&exposures);
        for v in out {
            assert!(v.is_finite(), "exposure component non-finite: {v}");
            assert!((0.0..=1.0).contains(&v), "exposure out of [0,1]: {v}");
        }
    }
}
