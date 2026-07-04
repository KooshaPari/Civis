//! Emergent psyche state for civilian agents.
//!
//! Psyche is not an authored personality taxonomy. It is a compact vector
//! updated from genetics, lived needs, and culture exposure through social
//! ties.

use rand::Rng;
use serde::{Deserialize, Serialize};

use civ_genetics::{sentience::CognitionTraitProfile, Dna};
use civ_needs::{Health as LifeHealth, LifecycleParams};

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

/// Advance maturity from a single tick of lived experience.
pub fn tick_maturity(
    psyche: &mut Psyche,
    health: &LifeHealth,
    critical_fraction: f32,
    params: &LifecycleParams,
) {
    if health.is_dead() {
        return;
    }
    let stress = (critical_fraction.clamp(0.0, 1.0) * params.maturity_stress_penalty)
        .clamp(0.0, 1.0);
    let delta = params.base_maturity_rate * (1.0 - stress);
    psyche.maturity = (psyche.maturity + delta).clamp(0.0, 1.0);
}

/// Update temperament with a small lived-experience nudge.
pub fn nudge_temperament(
    temperament: &mut Temperament,
    recent_mood_variance: f32,
    recent_social_satisfaction: f32,
    maturity: f32,
) {
    let plasticity = (1.0 - maturity * 0.8).clamp(0.0, 1.0);
    let lr = 0.002 * plasticity;
    temperament.reactivity =
        clamp01(temperament.reactivity + lr * (recent_mood_variance - temperament.reactivity));
    temperament.sociability = clamp01(
        temperament.sociability + lr * (recent_social_satisfaction - temperament.sociability),
    );
}

/// Update mood from needs plus a social-event term.
pub fn update_mood(
    mood: &mut Mood,
    needs: &Needs,
    temperament: &Temperament,
    threat_pressure: f32,
    delta_needs: f32,
    event_term: f32,
) {
    let need_valence = ((needs.food + needs.shelter + needs.safety + needs.belonging) / 4.0 - 0.5)
        .clamp(-1.0, 1.0);
    let target_val = clamp11(need_valence + 0.25 * event_term);
    let lr = 0.12 * (0.5 + temperament.reactivity);
    mood.valence = clamp11(mood.valence + (target_val - mood.valence) * lr);
    mood.arousal = (threat_pressure + delta_needs.abs() + 0.25 * event_term.abs()).clamp(0.0, 1.0);
}

/// Blend beliefs toward a culture exposure vector with a small mutational wobble.
pub fn update_beliefs(
    beliefs: &mut [f32; PSYCHE_DIM],
    exposure: [f32; PSYCHE_DIM],
    sociability: f32,
    rng: &mut impl Rng,
) {
    let lr = 0.08 * sociability.clamp(0.0, 1.0);
    let mut mixed = [0.0; PSYCHE_DIM];
    for i in 0..PSYCHE_DIM {
        mixed[i] = clamp01(beliefs[i] + (exposure[i] - beliefs[i]) * lr);
        let jitter = (rng.gen::<f32>() - 0.5) * 0.02;
        mixed[i] = clamp01(mixed[i] + jitter);
    }
    *beliefs = culture::mutate_traits(rng, mixed, 0.01);
}

/// Expose the culture vector sampled through an agent's social ties.
#[must_use]
pub fn belief_culture_exposure(exposures: &[(f32, [f32; PSYCHE_DIM])]) -> [f32; PSYCHE_DIM] {
    let mut out = [0.5; PSYCHE_DIM];
    let mut total = 0.0;
    for (weight, traits) in exposures {
        if *weight <= 0.0 {
            continue;
        }
        total += *weight;
        for i in 0..PSYCHE_DIM {
            out[i] += traits[i] * *weight;
        }
    }
    if total > 0.0 {
        for value in &mut out {
            *value = clamp01(*value / (total + 1.0));
        }
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

    #[test]
    fn maturity_grows_under_low_stress_and_stalls_under_high_stress() {
        let mut psyche = Psyche {
            drives: [0.5; PSYCHE_DIM],
            temperament: Temperament::neutral(),
            mood: Mood::neutral(),
            beliefs: [0.5; PSYCHE_DIM],
            maturity: 0.0,
        };
        let params = LifecycleParams {
            maturity_stress_penalty: 1.0,
            ..LifecycleParams::default()
        };
        let healthy = LifeHealth::default();
        for _ in 0..50 {
            tick_maturity(&mut psyche, &healthy, 0.0, &params);
        }
        let low_stress = psyche.maturity;
        for _ in 0..50 {
            tick_maturity(&mut psyche, &healthy, 1.0, &params);
        }
        let high_stress = psyche.maturity;
        assert!(low_stress > 0.0);
        assert!((high_stress - low_stress).abs() <= f32::EPSILON);
    }
}
