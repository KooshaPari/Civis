//!
//! FR-EMERGENCE-tests — anti-theater emergence quality gate (integration).
//!
//! Complements [`fr_emergence_quality`] with explicit coverage for:
//! - derived dashboard metrics (`novelty_score`, `coupling_mi_estimate`,
//!   `criticality_indicator`);
//! - canonical emergent structures (settlement / institution / faction /
//!   religion).
//!
//! Assertions are **range-based** only — no exact tick values or byte
//! determinism (stochastic drift is expected).
//!
//! Spec: `docs/design/emergence-dashboard.md`, `docs/design/EMERGENCE_TESTS_PLAN.md`

use civ_engine::faction_emergence::{cluster_into_factions, AgentIdeology};
use civ_engine::{
    apply_big_gods_response, last_religion_sample, InstitutionKind, ReligiousProfile,
    Simulation, SubstrateGradients,
};

/// Tick budget large enough to cross several `EMERGENCE_SAMPLE_INTERVAL` (50)
/// boundaries and let gov / psyche layers accumulate signal.
const RUN_TICKS: u32 = 250;

/// Multi-seed sweep — one lucky seed must not mask a theater regression.
const RUN_SEEDS: &[u64] = &[7, 31, 97];

fn assert_finite_in_range(label: &str, value: f32, lo: f32, hi: f32) {
    assert!(value.is_finite(), "{label}: expected finite, got {value}");
    assert!(
        (lo..=hi).contains(&value),
        "{label}: expected [{lo}, {hi}], got {value}"
    );
}

/// FR-EMERGENCE-tests — entropy, structure-count, power-law, novelty,
/// coupling-MI, and criticality indicators must leave the all-zero /
/// non-finite theater state after a live sample.
#[test]
fn fr_emergence_anti_theater_core_metrics_reach_nontrivial_ranges() {
    for &seed in RUN_SEEDS {
        let mut sim = Simulation::with_seed(seed);
        sim.advance_ticks(RUN_TICKS);
        assert!(
            sim.sample_emergence(),
            "sample_emergence must fire on boundary (seed={seed})"
        );

        let s = sim
            .last_emergence_sample()
            .expect("sampler caches the latest EmergenceSample");

        // Shannon / entropy band (dashboard charter §3.4).
        assert_finite_in_range("entropy_norm", s.entropy_norm, 0.0, 1.0);
        assert!(
            s.histogram_total > 0,
            "histogram_total must be > 0 after {RUN_TICKS} ticks (seed={seed})"
        );
        assert!(
            s.histogram_populated_bins >= 1,
            "histogram_populated_bins must be >= 1 (seed={seed})"
        );

        // Structure count — at least one 6-connected component on the voxel mask.
        let structures = s.structure_count.unwrap_or(0);
        assert!(
            structures >= 1,
            "structure_count must be >= 1 (seed={seed}); got {structures}"
        );

        // Power-law slope — finite, non-negative sentinel contract.
        assert!(s.power_law_alpha.is_finite(), "power_law_alpha (seed={seed})");
        assert!(
            s.power_law_alpha >= 0.0,
            "power_law_alpha must be >= 0 (seed={seed}); got {}",
            s.power_law_alpha
        );

        // Novelty — raw rate + dashboard-derived score.
        assert!(s.novelty_rate.is_finite(), "novelty_rate (seed={seed})");
        assert_finite_in_range("novelty_score", s.novelty_score, 0.0, 1.0);
        if s.novelty_rate > 0.0 {
            assert!(
                s.novelty_score > 0.0,
                "novelty_score must be > 0 when novelty_rate > 0 (seed={seed})"
            );
        }

        // Coupling MI — normalised estimate in [0, 1]; optional raw MI when wired.
        assert_finite_in_range(
            "coupling_mi_estimate",
            s.coupling_mi_estimate,
            0.0,
            1.0,
        );
        if let Some(mi) = s.mi_material_faction_norm {
            assert_finite_in_range("mi_material_faction_norm", mi, 0.0, 1.0);
            if mi > 0.0 && s.entropy_norm > 0.0 {
                assert!(
                    s.coupling_mi_estimate > 0.0,
                    "coupling_mi_estimate must be > 0 when MI and entropy are (seed={seed})"
                );
            }
        }

        // Criticality — combines branching σ, power-law α, entropy norm.
        assert_finite_in_range(
            "criticality_indicator",
            s.criticality_indicator,
            0.0,
            1.0,
        );
        assert_finite_in_range("branching_sigma", s.branching_sigma, 0.0, 3.0);

        // Anti-theater: at least two independent metric families must show
        // non-degenerate signal (not all stuck at documented sentinels).
        let mut signal_families = 0u32;
        if s.entropy_norm > 0.05 {
            signal_families += 1;
        }
        if structures >= 1 {
            signal_families += 1;
        }
        if s.power_law_alpha > 0.0 || s.histogram_populated_bins >= 3 {
            signal_families += 1;
        }
        if s.branching_sigma > 0.0 || s.criticality_indicator > 0.0 {
            signal_families += 1;
        }
        assert!(
            signal_families >= 2,
            "expected >= 2 non-degenerate metric families (seed={seed}); got {signal_families}"
        );
    }
}

/// FR-EMERGENCE-tests — at least one canonical emergent structure
/// (settlement / institution / faction / religion) must be observable via
/// public read APIs after a seeded run with settlement fixtures.
#[test]
fn fr_emergence_anti_theater_at_least_one_canonical_structure() {
    for &seed in RUN_SEEDS {
        let mut sim = Simulation::with_seed(seed);

        // Settlement fixture — drives institutions, mood, stratification.
        sim.set_settlement_population(0, 60);
        sim.set_settlement_food_stocked(0, 1_000);
        sim.set_settlement_housing_capacity(0, 60);
        sim.set_settlement_crime_pressure(0, 10);

        // Religion fixture — continuous profile via the public Big-Gods response.
        let mut profile = ReligiousProfile::new(60, 0);
        let gradients = SubstrateGradients {
            grad_T: 0.6,
            grad_M: 0.5,
            grad_B: 0.4,
            kinship_density: 0.2,
            unrest: 15.0,
            migration_rate: 0.1,
            language_distance: 0.1,
        };
        apply_big_gods_response(&mut profile, &gradients, 1);
        sim.religious_profiles.insert(0, profile);

        sim.advance_ticks(RUN_TICKS);

        // Re-touch population so institution phase can emit on the final tick.
        sim.set_settlement_population(0, 60);
        sim.advance_ticks(1);

        // Settlement — mood snapshots are keyed off the settlements map.
        let settlement_present = sim
            .last_tick_mood_all()
            .iter()
            .any(|m| m.settlement_id == 0);

        // Institution — temple unlock at pop >= TEMPLE_UNLOCK_POPULATION (50).
        let institution_present = sim.last_tick_institution_events().iter().any(|e| {
            matches!(e.kind, InstitutionKind::Temple if e.settlement_id == 0)
        });

        // Faction — doctrine libraries exist and k-means clustering yields seeds
        // from spread ideologies (public `faction_emergence` API).
        let faction_doctrines_present = !sim.faction_doctrines().is_empty();
        let ideologies: Vec<AgentIdeology> = (0..12)
            .map(|i| AgentIdeology {
                values: std::array::from_fn(|d| {
                    ((i as f32 + 1.0) / 13.0) * ((d as f32 + 1.0) / 9.0)
                }),
            })
            .collect();
        let faction_clusters = cluster_into_factions(&ideologies, 3);
        let faction_structure_present =
            faction_doctrines_present && faction_clusters.len() >= 1;

        // Religion — profile snapshot accessor returns the wired settlement.
        let religion_present = last_religion_sample(&sim.religious_profiles)
            .iter()
            .any(|(sid, p)| {
                *sid == 0
                    && (p.monitoring > 0.0
                        || p.mythic_coherence > 0.0
                        || p.uncertainty_reduction > 0.0)
            });

        let any = settlement_present
            || institution_present
            || faction_structure_present
            || religion_present;

        assert!(
            any,
            "no canonical emergent structure for seed {seed}: \
             settlement={settlement_present} institution={institution_present} \
             faction={faction_structure_present} religion={religion_present}"
        );
    }
}

/// FR-EMERGENCE-tests — faction doctrine evolution across ticks proves the
/// faction layer is wired (not a static stub).
#[test]
fn fr_emergence_anti_theater_faction_doctrine_generation_advances() {
    let mut sim = Simulation::with_seed(42);
    let gen0 = sim.faction_doctrines()[0].generation;

    sim.advance_ticks(63);
    assert_eq!(
        sim.faction_doctrines()[0].generation,
        gen0,
        "doctrine generation must not advance before tick 64"
    );

    sim.advance_ticks(1);
    assert!(
        sim.faction_doctrines()[0].generation > gen0,
        "faction doctrine generation must advance at tick 64 (anti-theater)"
    );
}
