//! FR-CIV-LANG depth: phoneme drift, lexicon growth, cluster divergence, naming.

use civ_agents::culture::{cluster_language_distance, CultureProfile};
use civ_agents::language::{name_from_lexicon, LexemeKind};
use civ_engine::Simulation;

fn run_ticks(sim: &mut Simulation, n: u64) {
    for _ in 0..n {
        sim.tick();
    }
}

#[test]
fn lang_lexicon_grows_after_warmup() {
    let mut sim = Simulation::with_seed(42);
    run_ticks(&mut sim, 300);
    assert!(
        !sim.cluster_lexicons().is_empty(),
        "lexicon must grow for settlement clusters"
    );
    let (cluster_id, profile) = sim
        .cluster_cultures()
        .iter()
        .next()
        .expect("cluster culture");
    let lexicon = sim.cluster_lexicons().get(cluster_id).expect("lexicon");
    let name = name_from_lexicon(lexicon, &profile.phonemes, LexemeKind::Settlement, *cluster_id)
        .expect("settlement name");
    assert!(!name.is_empty());
    assert!(name.chars().next().unwrap().is_uppercase());
}

#[test]
fn lang_emergence_feed_language_regions_after_250_ticks() {
    let mut sim = Simulation::with_seed(17);
    run_ticks(&mut sim, 280);
    let has_region = sim
        .emergence_feed()
        .iter()
        .any(|e| e.kind == "language_region");
    if sim.cluster_cultures().len() >= 2 {
        assert!(
            has_region,
            "expected language_region feed after warmup with >=2 clusters"
        );
    }
}

#[test]
fn lang_cluster_divergence_couples_culture_and_language() {
    let a = CultureProfile::new([0.0, 0.0, 0.0, 0.0]);
    let b = CultureProfile::new([1.0, 1.0, 1.0, 1.0]);
    let dist = cluster_language_distance(&a, &b);
    assert!(dist >= 0.35, "maximally divergent cultures must yield high language distance");
}

#[test]
fn lang_faction_names_coined_from_lexicon() {
    let mut sim = Simulation::with_seed(55);
    run_ticks(&mut sim, 100);
    let faction_id = 1u64;
    let lexicon = sim
        .cluster_lexicons()
        .get(&faction_id)
        .expect("faction lexicon");
    let inv = sim
        .cluster_cultures()
        .values()
        .next()
        .map(|p| p.phonemes.clone())
        .expect("phoneme inventory");
    let name = name_from_lexicon(lexicon, &inv, LexemeKind::Faction, faction_id)
        .expect("faction name");
    assert!(!name.is_empty());
}
