//! N5/N6/N8 emergence coupling tests (FR-CIV-TEST-003).
//!
//! - N5  social interaction -> SocialGraph familiarity buildup
//! - N6  needs -> mood (update_mood: sated->positive valence, threat->high arousal, deprived->negative)
//! - N8  beliefs drift toward culture exposure (sociability gates convergence rate)

use civ_agents::psyche::{
    belief_culture_exposure, update_beliefs, update_mood, Mood, Temperament,
};
use civ_agents::social::{apply_social_event, SocialEvent, SocialGraph};
use civ_agents::{Interaction, Needs};
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

// -- N5: social interaction -> familiarity buildup ---------------------------

/// N5 nominal: Cooperated interaction increases familiarity more than Coexisted.
#[test]
fn n5_cooperated_raises_familiarity_more_than_coexisted() {
    let mut graph_coop = SocialGraph::default();
    let mut graph_coex = SocialGraph::default();
    for _ in 0..5 {
        apply_social_event(&mut graph_coop, SocialEvent { a: 1, b: 2, kind: Interaction::Cooperated { benefit: 1.0 }, tick: 0 });
        apply_social_event(&mut graph_coex, SocialEvent { a: 1, b: 2, kind: Interaction::Coexisted, tick: 0 });
    }
    let fam_coop = graph_coop.ties.iter().find(|t| t.other == 2).map(|t| t.familiarity).unwrap_or(0.0);
    let fam_coex = graph_coex.ties.iter().find(|t| t.other == 2).map(|t| t.familiarity).unwrap_or(0.0);
    assert!(fam_coop > fam_coex, "Cooperated must build more familiarity than Coexisted: coop={fam_coop}, coex={fam_coex}");
}

/// N5 boundary: Coexisted events still produce positive familiarity (>0).
#[test]
fn n5_coexisted_builds_nonzero_familiarity() {
    let mut graph = SocialGraph::default();
    for _ in 0..10 {
        apply_social_event(&mut graph, SocialEvent { a: 3, b: 4, kind: Interaction::Coexisted, tick: 0 });
    }
    let fam = graph.ties.iter().find(|t| t.other == 4).map(|t| t.familiarity).unwrap_or(0.0);
    assert!(fam > 0.0, "10 Coexisted events must produce positive familiarity, got {fam}");
}

// -- N6: needs -> mood via update_mood ----------------------------------------

/// N6 nominal: fully sated needs converge mood valence toward positive.
#[test]
fn n6_sated_needs_produce_positive_valence() {
    let mut mood = Mood::neutral();
    let needs = Needs { food: 1.0, shelter: 1.0, safety: 1.0, belonging: 1.0 };
    let temp = Temperament::neutral();
    for _ in 0..20 { update_mood(&mut mood, &needs, &temp, 0.0, 0.0, 0.0); }
    assert!(mood.valence > 0.0, "sated needs must produce positive valence, got {}", mood.valence);
}

/// N6 boundary: high threat_pressure raises arousal > 0.5.
#[test]
fn n6_high_threat_raises_arousal() {
    let mut mood = Mood::neutral();
    let needs = Needs { food: 0.5, shelter: 0.5, safety: 0.0, belonging: 0.5 };
    let temp = Temperament::neutral();
    update_mood(&mut mood, &needs, &temp, 1.0, 0.5, 0.0);
    assert!(mood.arousal > 0.5, "threat_pressure=1.0 must produce arousal > 0.5, got {}", mood.arousal);
}

/// N6 boundary: fully deprived needs produce negative valence after convergence.
#[test]
fn n6_deprived_needs_produce_negative_valence() {
    let mut mood = Mood::neutral();
    let needs = Needs { food: 0.0, shelter: 0.0, safety: 0.0, belonging: 0.0 };
    let temp = Temperament::neutral();
    for _ in 0..20 { update_mood(&mut mood, &needs, &temp, 0.0, 0.5, 0.0); }
    assert!(mood.valence < 0.0, "fully deprived needs must produce negative valence, got {}", mood.valence);
}

// -- N8: beliefs drift toward culture exposure via update_beliefs ------------

/// N8 nominal: high sociability (1.0) converges beliefs toward exposure faster than low (0.0).
#[test]
fn n8_high_sociability_converges_beliefs_faster() {
    let mut rng_h = ChaCha8Rng::seed_from_u64(42);
    let mut rng_l = ChaCha8Rng::seed_from_u64(42);
    let exposure = [1.0_f32; 4];
    let mut beliefs_high = [0.0_f32; 4];
    let mut beliefs_low = [0.0_f32; 4];
    for _ in 0..20 {
        update_beliefs(&mut beliefs_high, exposure, 1.0, &mut rng_h);
        update_beliefs(&mut beliefs_low, exposure, 0.0, &mut rng_l);
    }
    let dist_high: f32 = beliefs_high.iter().zip(&exposure).map(|(b, e)| (b - e).abs()).sum::<f32>() / 4.0;
    let dist_low: f32 = beliefs_low.iter().zip(&exposure).map(|(b, e)| (b - e).abs()).sum::<f32>() / 4.0;
    assert!(dist_high < dist_low, "high sociability must converge faster: dist_high={dist_high}, dist_low={dist_low}");
}

/// N8 boundary: zero sociability causes no directed drift (total shift < 0.5 over 50 steps).
#[test]
fn n8_zero_sociability_no_directed_drift() {
    let mut rng = ChaCha8Rng::seed_from_u64(99);
    let initial = [0.1_f32, 0.2, 0.3, 0.4];
    let mut beliefs = initial;
    for _ in 0..50 { update_beliefs(&mut beliefs, [0.9, 0.8, 0.7, 0.6], 0.0, &mut rng); }
    let drift: f32 = beliefs.iter().zip(&initial).map(|(b, i)| (b - i).abs()).sum();
    assert!(drift < 0.5, "zero sociability must not cause directed drift; total drift={drift}");
}

/// N8 nominal: belief_culture_exposure blends toward dominant (high-weight) culture.
#[test]
fn n8_belief_culture_exposure_weights_dominant_culture() {
    let dominant = [1.0_f32; 4];
    let minor = [0.0_f32; 4];
    let result = belief_culture_exposure(&[(10.0, dominant), (1.0, minor)]);
    let dist_dom: f32 = result.iter().zip(&dominant).map(|(r, d)| (r - d).abs()).sum();
    let dist_min: f32 = result.iter().zip(&minor).map(|(r, m)| (r - m).abs()).sum();
    assert!(dist_dom < dist_min, "dominant culture must win blend: dist_dom={dist_dom}, dist_min={dist_min}");
}