//! Integration tests for FR-CIV-EMERGENCE-001 and FR-CIV-EMERGENCE-004.
//!
//! FR-CIV-EMERGENCE-001: Emergence metrics — the five dashboard summary
//! metrics (`cluster_entropy`, `ideology_homophily`, `sentience_fraction`,
//! `psyche_stability`, `diplomacy_tension`) must compute correctly from
//! pre-aggregated inputs.
//!
//! FR-CIV-EMERGENCE-004: Emergence dashboard — the `EmergenceDashboard`
//! struct must be serializable/deserializable for JSON-RPC transport and
//! must combine all five sub-metrics into a single snapshot.

use civ_emergence_metrics::{
    criticality::{criticality_indicator, CriticalityBands, CriticalityInputs},
    dashboard::{
        cluster_entropy, diplomacy_tension, ideology_homophily, psyche_stability,
        sentience_fraction, EmergenceDashboard,
    },
    mutual_information::{mutual_information_bits, mutual_information_normalised, JointHistogram},
    shannon::ShannonEntropy,
    structure::{Grid, StructureCount},
    Histogram, Metric,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn approx(a: f32, b: f32, eps: f32) {
    assert!(
        (a - b).abs() < eps,
        "expected {b} got {a} (|diff|={})",
        (a - b).abs()
    );
}

// ===========================================================================
// FR-CIV-EMERGENCE-001: Emergence metrics
// ===========================================================================

// --- cluster_entropy ---

/// FR-CIV-EMERGENCE-001: cluster_entropy returns 0.0 for an empty world.
#[test]
fn fr_civ_emergence_001_cluster_entropy_empty_is_zero() {
    assert_eq!(cluster_entropy(&[]), 0.0);
}

/// FR-CIV-EMERGENCE-001: cluster_entropy returns 1.0 for perfectly uniform
/// distribution across two clusters (log2(2) = 1, uniform => H_norm = 1.0).
#[test]
fn fr_civ_emergence_001_cluster_entropy_uniform_is_one() {
    approx(cluster_entropy(&[100, 100, 100, 100]), 1.0, 1e-4);
}

/// FR-CIV-EMERGENCE-001: cluster_entropy is permutation-invariant — the
/// metric does not depend on cluster ordering.
#[test]
fn fr_civ_emergence_001_cluster_entropy_permutation_invariant() {
    let a = cluster_entropy(&[10, 30, 50, 20]);
    let b = cluster_entropy(&[50, 10, 20, 30]);
    approx(a, b, 1e-6);
}

// --- ideology_homophily ---

/// FR-CIV-EMERGENCE-001: ideology_homophily returns 0.0 for empty input.
#[test]
fn fr_civ_emergence_001_ideology_homophily_empty_is_zero() {
    assert_eq!(ideology_homophily(&[], 0.2), 0.0);
}

/// FR-CIV-EMERGENCE-001: ideology_homophily returns 1.0 when all agents
/// share the same ideology value (perfect homophily).
#[test]
fn fr_civ_emergence_001_ideology_homophily_unanimous_is_one() {
    let vals = vec![0.5_f32; 100];
    approx(ideology_homophily(&vals, 0.2), 1.0, 1e-6);
}

/// FR-CIV-EMERGENCE-001: ideology_homophily falls back to default bin_width
/// when zero is passed, and still returns a valid result.
#[test]
fn fr_civ_emergence_001_ideology_homophily_zero_bin_width_fallback() {
    let vals = vec![0.3_f32; 10];
    let h = ideology_homophily(&vals, 0.0);
    assert!(h > 0.99, "unanimous ideologies should yield ~1.0, got {h}");
}

// --- sentience_fraction ---

/// FR-CIV-EMERGENCE-001: sentience_fraction returns 0.0 when total is zero.
#[test]
fn fr_civ_emergence_001_sentience_fraction_no_agents_is_zero() {
    assert_eq!(sentience_fraction(0, 0), 0.0);
}

/// FR-CIV-EMERGENCE-001: sentience_fraction returns the correct ratio for
/// partial sentience (50 of 200 agents sentient => 0.25).
#[test]
fn fr_civ_emergence_001_sentience_fraction_partial_ratio() {
    approx(sentience_fraction(50, 200), 0.25, 1e-6);
}

/// FR-CIV-EMERGENCE-001: sentience_fraction clamps to 1.0 when sentient
/// count exceeds total (defensive).
#[test]
fn fr_civ_emergence_001_sentience_fraction_clamps_at_one() {
    approx(sentience_fraction(500, 200), 1.0, 1e-6);
}

// --- psyche_stability ---

/// FR-CIV-EMERGENCE-001: psyche_stability returns 1.0 for a single agent
/// (zero variance => perfect stability).
#[test]
fn fr_civ_emergence_001_psyche_stability_single_agent_is_one() {
    assert_eq!(psyche_stability(&[0.7]), 1.0);
}

/// FR-CIV-EMERGENCE-001: psyche_stability returns 0.0 when mood valences
/// span the full [-1, +1] range with maximum variance.
#[test]
fn fr_civ_emergence_001_psyche_stability_max_spread_is_zero() {
    let v = vec![-1.0_f32, 0.0, 1.0];
    approx(psyche_stability(&v), 0.0, 1e-4);
}

/// FR-CIV-EMERGENCE-001: psyche_stability returns exactly 1.0 for a uniform
/// mood (all agents at the same valence => zero variance).
#[test]
fn fr_civ_emergence_001_psyche_stability_uniform_mood_is_one() {
    let v = vec![0.4_f32; 50];
    approx(psyche_stability(&v), 1.0, 1e-6);
}

// --- diplomacy_tension ---

/// FR-CIV-EMERGENCE-001: diplomacy_tension returns 0.0 for empty input.
#[test]
fn fr_civ_emergence_001_diplomacy_tension_empty_is_zero() {
    assert_eq!(diplomacy_tension(&[]), 0.0);
}

/// FR-CIV-EMERGENCE-001: diplomacy_tension returns 1.0 when all pair scores
/// are at maximum magnitude (all +1.0 or all -1.0).
#[test]
fn fr_civ_emergence_001_diplomacy_tension_all_extreme_is_one() {
    let scores = vec![1.0_f32, -1.0, 1.0, -1.0];
    approx(diplomacy_tension(&scores), 1.0, 1e-6);
}

/// FR-CIV-EMERGENCE-001: diplomacy_tension computes mean absolute value
/// correctly for mixed scores (mean(|0.6|, |-0.2|, |0.4|) = 0.4).
#[test]
fn fr_civ_emergence_001_diplomacy_tension_mixed_mean_abs() {
    let scores = vec![0.6_f32, -0.2, 0.4];
    approx(diplomacy_tension(&scores), 0.4, 1e-4);
}

// --- Shannon entropy (core metric) ---

/// FR-CIV-EMERGENCE-001: ShannonEntropy on a uniform histogram returns
/// log2(N) bits; on a Dirac histogram returns 0.0 bits.
#[test]
fn fr_civ_emergence_001_shannon_uniform_vs_dirac() {
    let uniform = Histogram::uniform(8, 100);
    let dirac = Histogram::dirac(8, 0, 100);
    let e = ShannonEntropy::new();
    approx(e.compute_bits(&uniform), 3.0, 1e-4); // log2(8) = 3
    approx(e.compute_bits(&dirac), 0.0, 1e-4);
}

/// FR-CIV-EMERGENCE-001: Normalised Shannon entropy of a uniform histogram
/// is 1.0 (maximum diversity).
#[test]
fn fr_civ_emergence_001_shannon_normalised_uniform_is_one() {
    let h = Histogram::uniform(16, 50);
    let e = ShannonEntropy::new();
    approx(e.compute_normalised(&h), 1.0, 1e-4);
}

/// FR-CIV-EMERGENCE-001: Metric trait NAME is stable.
#[test]
fn fr_civ_emergence_001_shannon_metric_name() {
    assert_eq!(ShannonEntropy::NAME, "shannon_entropy");
}

// --- Structure count ---

/// FR-CIV-EMERGENCE-001: StructureCount on an all-background grid yields
/// zero components, zero foreground.
#[test]
fn fr_civ_emergence_001_structure_all_background() {
    let data = vec![0u8; 4 * 4 * 4];
    let g = Grid::new(4, 4, 4, &data).expect("4^3 grid");
    let s = StructureCount::new().evaluate(&g, |&v| v > 0);
    assert_eq!(s.count, 0);
    assert_eq!(s.foreground, 0);
}

/// FR-CIV-EMERGENCE-001: StructureCount on a fully-filled grid yields
/// exactly one connected component.
#[test]
fn fr_civ_emergence_001_structure_single_component() {
    let data = vec![1u8; 3 * 3 * 3];
    let g = Grid::new(3, 3, 3, &data).expect("3^3 grid");
    let s = StructureCount::new().evaluate(&g, |&v| v > 0);
    assert_eq!(s.count, 1);
    assert_eq!(s.largest, 27);
}

// --- Criticality indicator ---

/// FR-CIV-EMERGENCE-001: criticality_indicator returns 1.0 when all inputs
/// are exactly at the centre of their operational bands.
#[test]
fn fr_civ_emergence_001_criticality_centre_of_bands_is_one() {
    let inputs = CriticalityInputs {
        branching_sigma: 0.95,  // centre of [0.85, 1.05]
        power_law_alpha: 1.7,   // centre of [1.4, 2.0]
        entropy_norm: 0.75,     // centre of [0.6, 0.9]
    };
    let bands = CriticalityBands::default();
    let result = criticality_indicator(inputs, &bands);
    assert!((result - 1.0).abs() < 0.01, "centre-of-bands => ~1.0, got {result}");
}

/// FR-CIV-EMERGENCE-001: criticality_indicator returns 0.0 when all inputs
/// are far outside their operational bands.
#[test]
fn fr_civ_emergence_001_criticality_far_outside_is_zero() {
    let inputs = CriticalityInputs {
        branching_sigma: 100.0, // way above [0.85, 1.05]
        power_law_alpha: 100.0, // way above [1.4, 2.0]
        entropy_norm: 0.0,      // way below [0.6, 0.9]
    };
    let bands = CriticalityBands::default();
    let result = criticality_indicator(inputs, &bands);
    assert!(result < 0.01, "far-outside => ~0.0, got {result}");
}

// --- Mutual information ---

/// FR-CIV-EMERGENCE-001: mutual_information_bits returns 0.0 for an empty
/// joint histogram.
#[test]
fn fr_civ_emergence_001_mi_empty_histogram_is_zero() {
    let jh = JointHistogram::new(3, 3);
    assert_eq!(mutual_information_bits(&jh), 0.0);
}

/// FR-CIV-EMERGENCE-001: mutual_information_bits returns 0.0 when the two
/// layers are perfectly independent (diagonal is product of marginals).
#[test]
fn fr_civ_emergence_001_mi_independent_layers_is_zero() {
    // Build a 2x2 joint histogram where p(a,b) = p(a)*p(b).
    // Row marginals: [0.5, 0.5], col marginals: [0.5, 0.5].
    // p(0,0) = 0.25, p(0,1) = 0.25, p(1,0) = 0.25, p(1,1) = 0.25.
    let mut jh = JointHistogram::new(2, 2);
    // 100 observations of each cell => 400 total.
    for _ in 0..100 {
        jh.observe(0, 0);
        jh.observe(0, 1);
        jh.observe(1, 0);
        jh.observe(1, 1);
    }
    let mi = mutual_information_bits(&jh);
    assert!(mi.abs() < 0.01, "independent layers => MI ~0.0, got {mi}");
}

/// FR-CIV-EMERGENCE-001: mutual_information_normalised is bounded in [0,1].
#[test]
fn fr_civ_emergence_001_mi_normalised_bounded() {
    let mut jh = JointHistogram::new(4, 4);
    // Perfect diagonal: every observation is (a, a).
    for a in 0..4 {
        for _ in 0..50 {
            jh.observe(a, a);
        }
    }
    let norm = mutual_information_normalised(&jh);
    assert!(
        (0.0..=1.0).contains(&norm),
        "normalised MI must be in [0,1], got {norm}"
    );
    // With perfect correlation the normalised MI should be high.
    assert!(norm > 0.9, "perfect diagonal => norm MI > 0.9, got {norm}");
}

// ===========================================================================
// FR-CIV-EMERGENCE-004: Emergence dashboard
// ===========================================================================

/// FR-CIV-EMERGENCE-004: EmergenceDashboard::compute produces a struct
/// with all five fields populated from the supplied inputs.
#[test]
fn fr_civ_emergence_004_dashboard_combines_all_five_fields() {
    let d = EmergenceDashboard::compute(
        &[50, 50],                          // two equal clusters
        &[0.5, 0.5, 0.5, 0.5, 0.5, 0.5],  // all same ideology
        80,
        100,                                // 80/100 sentient
        &[0.0, 0.0, 0.0, 0.0],             // all same mood
        &[0.9, -0.8, 0.7],                 // mixed diplomacy
    );
    // Two equal clusters => uniform => cluster_entropy = 1.0.
    approx(d.cluster_entropy, 1.0, 1e-4);
    // All same ideology => homophily = 1.0.
    approx(d.ideology_homophily, 1.0, 1e-6);
    // 80/100 => 0.8.
    approx(d.sentience_fraction, 0.8, 1e-6);
    // All same mood => variance = 0 => stability = 1.0.
    approx(d.psyche_stability, 1.0, 1e-6);
    // Mean |0.9| + |-0.8| + |0.7| / 3 = 2.4/3 = 0.8.
    approx(d.diplomacy_tension, 0.8, 1e-4);
}

/// FR-CIV-EMERGENCE-004: empty inputs yield the documented "no data"
/// default where every metric returns 0.0 except psyche_stability (1.0).
#[test]
fn fr_civ_emergence_004_dashboard_empty_inputs_yield_defaults() {
    let d = EmergenceDashboard::compute(&[], &[], 0, 0, &[], &[]);
    assert_eq!(d.cluster_entropy, 0.0);
    assert_eq!(d.ideology_homophily, 0.0);
    assert_eq!(d.sentience_fraction, 0.0);
    assert_eq!(d.psyche_stability, 1.0);
    assert_eq!(d.diplomacy_tension, 0.0);
}

/// FR-CIV-EMERGENCE-004: EmergenceDashboard derives Clone and Debug for
/// transport/replay logging.
#[test]
fn fr_civ_emergence_004_dashboard_clone_and_debug() {
    let d = EmergenceDashboard::compute(
        &[30, 70],
        &[0.1, 0.2, 0.3],
        5,
        10,
        &[-0.5, 0.5],
        &[0.6, -0.3],
    );
    let cloned = d.clone();
    assert_eq!(d, cloned);
    // Debug output should contain the struct name.
    let debug = format!("{d:?}");
    assert!(debug.contains("EmergenceDashboard"), "Debug must include struct name");
}

/// FR-CIV-EMERGENCE-004: EmergenceDashboard default is the all-zeros
/// baseline (except psyche_stability is not part of Default derive —
/// check that Default produces all 0.0).
#[test]
fn fr_civ_emergence_004_dashboard_default_is_all_zeros() {
    let d = EmergenceDashboard::default();
    assert_eq!(d.cluster_entropy, 0.0);
    assert_eq!(d.ideology_homophily, 0.0);
    assert_eq!(d.sentience_fraction, 0.0);
    assert_eq!(d.psyche_stability, 0.0);
    assert_eq!(d.diplomacy_tension, 0.0);
}

/// FR-CIV-EMERGENCE-004: dashboard fields are Copy — assigning one
/// instance to another copies all values.
#[test]
fn fr_civ_emergence_004_dashboard_is_copy() {
    let a = EmergenceDashboard::compute(&[20, 20, 20], &[], 3, 3, &[], &[]);
    let b = a; // Copy
    assert_eq!(a, b);
    approx(a.cluster_entropy, b.cluster_entropy, 1e-10);
}
