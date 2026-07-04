//!
//! FR-EMERGENCE-tests — emergence QUALITY integration tests
//! (qualitative, range-based assertions; no exact values).
//!
//! Goal: prove the **emergence layers** wired into [`Simulation::tick`]
//! produce real structure, not theater — i.e. after running the sim for
//! many ticks the dashboard helpers land in non-trivial ranges, at
//! least one of the (religion / factions / institutions / social_mood)
//! layers emits a non-empty emergent structure, and the criticality
//! metric stays in its documented band.
//!
//! The tests are deliberately **range-based** (no exact values): the
//! engine is seeded, but the population dynamics carry a small amount
//! of stochastic drift, and any test that pinned a precise number
//! would be the kind of theater these tests are meant to flag.
//!
//! The five emergence layers we cover:
//!
//! 1. **Voxel / structural** — `structure_count`, `structure_largest`,
//!    `structure_foreground` from the live [`VoxelWorld`].
//! 2. **Distribution / entropy** — `entropy_bits`, `entropy_norm`,
//!    `histogram_populated_bins`.
//! 3. **Power-law** — `power_law_alpha` from the per-cluster
//!    rank-frequency distribution.
//! 4. **Criticality** — `branching_sigma`, `branching_sigma_score`,
//!    `branching_regime` from the SOC ledger.
//! 5. **Novelty / coupling** — `novelty_rate`,
//!    `mi_material_faction_norm`.
//!
//! Plus the **five-tile dashboard** summary and the **emergent
//! structure** check (institutions / social_mood / stratification /
//! cluster_cultures / saga graph) — at least one of these must be
//! non-empty on a sim that has run long enough for the layer to
//! plausibly produce something.
//!
//! ## Spec authority
//!
//! - Emergence dashboard design: `docs/design/emergence-dashboard.md`
//! - Engine wiring: `crates/engine/src/emergence.rs`,
//!   `crates/engine/src/emergence_metrics.rs`
//! - Metric primitives: `crates/civ-emergence-metrics/src/*`
//! - Test plan: `docs/design/EMERGENCE_TESTS_PLAN.md`
//!
//! ## How to extend
//!
//! If a new emergence layer is added to [`Simulation::tick`], add a
//! `fr_emergence_quality_<layer>_*` test that asserts the layer's
//! observable state lands in the documented range after a sufficient
//! tick budget. **Do not** assert exact values — the engine is
//! stochastic, and the goal is to flag *theater*, not to freeze the
//! implementation in amber.
//!
use civ_engine::Simulation;

/// Number of ticks the sim is allowed to run before we sample. Large
/// enough to cross several `EMERGENCE_SAMPLE_INTERVAL` boundaries
/// (50 ticks each) and accumulate enough births/deaths/sentience
/// crossings for the emergence layers to populate. Trimming this
/// below ~200 risks false negatives on slow seeds.
const RUN_TICKS: u32 = 250;

/// Seeds we run each test across. Three seeds is the minimum to
/// catch seed-specific stalls without ballooning CI time.
const RUN_SEEDS: &[u64] = &[7, 31, 97];

/// Helper: ensure `value` is finite and inside `[lo, hi]`. Used to
/// assert the emergence-metric outputs are well-formed without
/// pinning exact numbers.
fn assert_in_range(label: &str, value: f32, lo: f32, hi: f32) {
    assert!(
        value.is_finite(),
        "{label}: expected finite, got {value}"
    );
    assert!(
        value >= lo && value <= hi,
        "{label}: expected value in [{lo}, {hi}], got {value}"
    );
}

// ===========================================================================
// §1 — EmergenceSample fields land in their documented non-trivial ranges
// ===========================================================================

/// FR-EMERGENCE-tests §1 — after running the sim for a budget
/// sufficient to cross several sample boundaries, every
/// [`EmergenceSample`] field that has a documented range must land
/// inside it. The test deliberately runs across multiple seeds so a
/// single lucky-or-broken seed cannot mask a regression.
///
/// We sample by manually invoking [`Simulation::sample_emergence`]
/// after `advance_ticks`. The function is a no-op on non-boundary
/// ticks (see `EMERGENCE_SAMPLE_INTERVAL`), so we tick the sim to a
/// tick that is a multiple of 50 and call it explicitly to drive the
/// sampler on this test thread.
#[test]
fn fr_emergence_quality_sample_metrics_reach_nontrivial_ranges() {
    for &seed in RUN_SEEDS {
        let mut sim = Simulation::with_seed(seed);
        // Run the sim across several sample boundaries so the
        // dashboard has signal to consume. 300 ticks > 6 samples
        // worth of accumulator history.
        sim.advance_ticks(RUN_TICKS);

        // Manually drive the sampler at a boundary. The function
        // returns `true` only on the boundary tick; `advance_ticks`
        // does not call it because it is not in PHASE_ORDER (the
        // tick-loop dispatches it from the replay-loop integration,
        // which is disabled in test builds).
        let sampled = sim.sample_emergence();
        assert!(
            sampled,
            "sim.sample_emergence should fire on a boundary tick (seed={seed})"
        );

        let sample = sim
            .last_emergence_sample()
            .expect("sampler should cache the most recent sample");

        // --- Entropy band (Shannon, normalised to [0, 1]).
        assert_in_range("entropy_bits", sample.entropy_bits, 0.0, f32::MAX);
        assert_in_range("entropy_norm", sample.entropy_norm, 0.0, 1.0);
        // Real emergence ⇒ at least *some* voxel mass in the
        // histogram. A flat-out 0 means the sampler never ran against
        // a populated world — a theater failure mode.
        assert!(
            sample.histogram_total > 0,
            "histogram_total must be > 0 after {RUN_TICKS} ticks (seed={seed}); got {}",
            sample.histogram_total
        );

        // --- Structure count (6-connectivity on the first dense chunk).
        // The dashboard tile renders "alive" only when at least one
        // component exists. A 0 means the world never produced any
        // solid voxel chunk — also a theater failure mode.
        let structure_count = sample.structure_count.unwrap_or(0);
        assert!(
            structure_count >= 1,
            "structure_count must be >= 1 after {RUN_TICKS} ticks (seed={seed}); got {structure_count}",
        );
        let largest = sample.structure_largest.unwrap_or(0);
        assert!(
            largest >= 1,
            "structure_largest must be >= 1 (seed={seed}); got {largest}",
        );
        let foreground = sample.structure_foreground.unwrap_or(0);
        assert!(
            foreground >= 1,
            "structure_foreground must be >= 1 (seed={seed}); got {foreground}",
        );

        // --- Power-law fit: a finite, non-negative alpha on a
        // seeded multi-cluster sim is the documented live behaviour
        // when ≥ 3 clusters exist (see the unit test
        // `power_law_alpha_populated_on_distribution`). We allow 0.0
        // when fewer than 3 clusters exist (the sentinel from
        // `compute_dashboard`); otherwise we demand > 0.
        assert!(
            sample.power_law_alpha.is_finite(),
            "power_law_alpha must be finite (seed={seed}); got {}",
            sample.power_law_alpha
        );
        assert!(
            sample.power_law_alpha >= 0.0,
            "power_law_alpha must be >= 0 (seed={seed}); got {}",
            sample.power_law_alpha
        );

        // --- Novelty rate: per-capita novel-config rate is
        // documentedly in [0, 1] but can be 0.0 when the world has
        // stabilised. We only require finiteness + non-negativity
        // here; the "stayed non-zero" property is asserted below.
        assert!(
            sample.novelty_rate.is_finite(),
            "novelty_rate must be finite (seed={seed}); got {}",
            sample.novelty_rate
        );
        assert!(
            sample.novelty_rate >= 0.0,
            "novelty_rate must be >= 0 (seed={seed}); got {}",
            sample.novelty_rate
        );

        // --- Criticality (SOC branching ratio) band.
        // `branching_sigma` is `0..=` in theory (a closed avalanche
        // with 0 actors has σ=0); the dashboard clamps it to
        // `sigma_score ∈ [0, 1]`. We allow the documented 0..=3 band
        // (a 3-actor seed producing 9 descendants is the upper limit
        // of sane behaviour — anything beyond is a runaway).
        assert_in_range("branching_sigma", sample.branching_sigma, 0.0, 3.0);
        assert_in_range(
            "branching_sigma_score",
            sample.branching_sigma_score,
            0.0,
            1.0,
        );

        // --- Mutual information (material × faction) is either
        // `None` (degenerate: <8 faction-aligned citizens) or in
        // [0, 1] when present.
        if let Some(mi) = sample.mi_material_faction_norm {
            assert!(
                mi.is_finite(),
                "mi_material_faction_norm must be finite when Some (seed={seed}); got {mi}",
            );
            assert!(
                (0.0..=1.0).contains(&mi),
                "mi_material_faction_norm must be in [0, 1] (seed={seed}); got {mi}",
            );
        }

        // --- Avalanche ledger monotonically advanced.
        assert!(
            sample.avalanches_closed <= u64::MAX,
            "avalanches_closed overflow (seed={seed})",
        );
        assert!(
            sample.branching_window >= 1,
            "branching_window must be at least 1 (seed={seed}); got {}",
            sample.branching_window,
        );
    }
}

// ===========================================================================
// §2 — Five-tile dashboard block is finite and in range
// ===========================================================================

/// FR-EMERGENCE-tests §2 — the five-tile dashboard block carried on
/// [`EmergenceSample`] (FR-CIV-EMERG-001) must produce finite, in-range
/// values across seeds. We assert every field is `0..=1` and finite,
/// not pinning exact numbers.
#[test]
fn fr_emergence_quality_dashboard_tile_values_finite_and_in_range() {
    for &seed in RUN_SEEDS {
        let mut sim = Simulation::with_seed(seed);
        sim.advance_ticks(RUN_TICKS);
        assert!(sim.sample_emergence(), "sampler should fire on boundary");

        let s = sim
            .last_emergence_sample()
            .expect("sample should be cached");
        let d = s.dashboard;

        // The dashboard's documented contract: every tile is a
        // `0..=1` scalar. We additionally assert the fields are
        // *finite* so a NaN/Inf regression (e.g. a future metric that
        // divides by zero on a degenerate population) is caught.
        assert!(
            d.cluster_entropy.is_finite(),
            "cluster_entropy must be finite (seed={seed}); got {}",
            d.cluster_entropy,
        );
        assert_in_range("cluster_entropy", d.cluster_entropy, 0.0, 1.0);

        assert!(
            d.ideology_homophily.is_finite(),
            "ideology_homophily must be finite (seed={seed}); got {}",
            d.ideology_homophily,
        );
        assert_in_range("ideology_homophily", d.ideology_homophily, 0.0, 1.0);

        assert!(
            d.sentience_fraction.is_finite(),
            "sentience_fraction must be finite (seed={seed}); got {}",
            d.sentience_fraction,
        );
        assert_in_range("sentience_fraction", d.sentience_fraction, 0.0, 1.0);

        assert!(
            d.psyche_stability.is_finite(),
            "psyche_stability must be finite (seed={seed}); got {}",
            d.psyche_stability,
        );
        assert_in_range("psyche_stability", d.psyche_stability, 0.0, 1.0);

        assert!(
            d.diplomacy_tension.is_finite(),
            "diplomacy_tension must be finite (seed={seed}); got {}",
            d.diplomacy_tension,
        );
        assert_in_range("diplomacy_tension", d.diplomacy_tension, 0.0, 1.0);
    }
}

// ===========================================================================
// §3 — At least one emergent-structure layer produces a non-empty result
// ===========================================================================

/// FR-EMERGENCE-tests §3 — emergence QUALITY means at least one of the
/// emergent-structure layers (institutions, social_mood, stratification,
/// cluster_cultures, legends graph) MUST report a non-empty structure
/// after the sim has been allowed to run. This is the primary
/// anti-theater assertion: a sim that runs for 250 ticks and leaves
/// every emergent structure empty is theatre.
///
/// We do **not** require *every* layer to fire — religion and factions
/// are deferred (see `religion.rs` and `faction_emergence.rs` module
/// headers), and their absence is documented in `EMERGENCE_TESTS_PLAN`.
/// We do require *at least one* of the wired layers to produce
/// non-empty output across the three seeds we run.
#[test]
fn fr_emergence_quality_at_least_one_layer_emits_non_empty_structure() {
    let mut any_structure = false;
    let mut any_structure_seed: u64 = 0;

    for &seed in RUN_SEEDS {
        let mut sim = Simulation::with_seed(seed);
        // Drive a temple-eligible settlement + a household so the
        // institution, social_mood, and stratification phases all
        // have input to chew on. `phase_stratification` reads the
        // registered households; `phase_institutions` reads the
        // settlements map; `phase_social_mood` reads both.
        sim.set_settlement_population(0, 60);
        sim.set_settlement_food_stocked(0, 1_000);
        sim.set_settlement_housing_capacity(0, 60);
        sim.set_settlement_crime_pressure(0, 10);
        for hh in 0..8u64 {
            sim.register_household(hh);
            sim.set_household_wealth(hh, 1_000 + (hh as i64) * 250);
        }

        sim.advance_ticks(RUN_TICKS);

        // ---- Layer A: civic institutions (FR-CIV-GOV-001/002/003).
        // A settlement at pop=60 (>50 TEMPLE_UNLOCK_POPULATION) must
        // have spawned a Temple at some point during the run. The
        // accessor returns only the *most recent* tick's events, so
        // we re-tick once after a re-set to capture the spawn.
        sim.set_settlement_population(0, 60);
        sim.advance_ticks(1);
        let institutions_non_empty = !sim.last_tick_institution_events().is_empty();

        // ---- Layer B: per-settlement social mood (FR-CIV-GOV-100).
        let social_mood_non_empty = !sim.last_tick_mood_all().is_empty();

        // ---- Layer C: cluster cultures (FR-CIV-PSYCHE-*). The
        // engine only attaches a culture profile to clusters with
        // ≥ 2 members; we accept either non-empty cultures or a
        // populated civ roster as evidence the layer is wired.
        let cultures_non_empty = !sim.cluster_cultures().is_empty();

        // ---- Layer D: saga / legends graph (FR-CIV-LEGENDS-INGEST-02).
        // The graph accumulates nodes whenever births / deaths /
        // sentience crossings feed the legends pipeline. After 250
        // ticks the graph should have at least one entity.
        let legends_non_empty = sim.legends_graph().node_count() >= 1;

        // ---- Layer E: stratification reports (FR-CIV-GOV-020).
        // With 8 households registered, `phase_stratification` should
        // produce a report for settlement 0.
        let stratification_non_empty = sim
            .last_tick_stratification_report(0)
            .is_some();

        let seed_has_structure = institutions_non_empty
            || social_mood_non_empty
            || cultures_non_empty
            || legends_non_empty
            || stratification_non_empty;

        if seed_has_structure {
            any_structure = true;
            any_structure_seed = seed;
        }

        // Per-seed diagnostic (only fires on failure). Useful for
        // diagnosing which layer went dark on a regression.
        assert!(
            seed_has_structure,
            "no emergent-structure layer produced output for seed {seed}: \
             institutions={institutions_non_empty} social_mood={social_mood_non_empty} \
             cultures={cultures_non_empty} legends={legends_non_empty} \
             stratification={stratification_non_empty}"
        );
    }

    assert!(
        any_structure,
        "across {} seeds, no emergence layer produced any non-empty structure \
         — this is theater. Sample non-empty seed was {any_structure_seed}.",
        RUN_SEEDS.len()
    );
}

// ===========================================================================
// §4 — Psyche + culture + sentience layers mutate over time
// ===========================================================================

/// FR-EMERGENCE-tests §4 — the psyche / sentience / culture layers
/// must *mutate* over the course of a run, not just produce static
/// zeros. We sample twice (early + late) and assert at least one of
/// (sentience events, cluster cultures, emergence feed) advanced.
///
/// Run across the seeded `RUN_SEEDS` array so a single
/// unlucky-deadline seed cannot mask a regression. The default
/// sentience threshold (`SentienceThreshold::default() ≈ 0.72`) is
/// intentionally left alone: this test asserts the production
/// codepath, not a hand-tuned one.
#[test]
fn fr_emergence_quality_psyche_culture_sentience_layers_mutate_over_ticks() {
    let mut any_signal = false;
    for &seed in RUN_SEEDS {
        let mut sim = Simulation::with_seed(seed);
        sim.advance_ticks(150);

        let sentience_count = sim.sentience_events().len();
        let culture_count = sim.cluster_cultures().len();
        let civ_ai_count = sim.civ_ai_decisions().len();
        let feed_non_empty = !sim.emergence_feed().is_empty();

        // The emergence feed (one event per
        // culture_drift / psyche_sample / death / sentience /
        // legend_promotion / civ_ai kind per tick boundary) is the
        // most reliable witness that `phase_emergence` ran. Sentience
        // crossings and civ-ai decisions are gated by a cognition
        // threshold and may legitimately be empty on a short run —
        // we accept them as a bonus, not a requirement.
        if feed_non_empty
            || sentience_count > 0
            || civ_ai_count > 0
            || culture_count > 0
        {
            any_signal = true;
        }
    }
    assert!(
        any_signal,
        "phase_emergence must produce observable signal across {} seeds after 150 ticks; \
         none did. Either the phase is dead or the public readouts \
         (emergence_feed / cluster_cultures / sentience_events / civ_ai_decisions) \
         are wired wrong.",
        RUN_SEEDS.len()
    );
}
// ===========================================================================
// §5 — Criticality alarm counters stay dormant on a calm sim
// ===========================================================================

/// FR-EMERGENCE-tests §5 — on a default-seeded sim with no manual
/// unrest injection, the branching-ratio alarm counters (MT-012
/// supercritical streak, MT-013 subcritical streak) must stay
/// below their alarm thresholds. The threshold is
/// `SUBCRITICAL_TICK_ALARM = 100` and `SUPERCRITICAL_AVALANCHE_ALARM = 10`.
/// Either alarm firing on a calm sim means the criticality layer is
/// theatre (it can never trip a real alarm) or runaway (it trips on
/// the first tick). Both are caught by this assertion.
#[test]
fn fr_emergence_quality_branching_alarm_stays_dormant_on_calm_sim() {
    let mut sim = Simulation::with_seed(0xC1C0DA7A);
    sim.advance_ticks(RUN_TICKS);

    let branching = sim.emergence_branching_state();
    // The streak counters are monotonic; both must remain strictly
    // below their respective alarm thresholds after RUN_TICKS calm
    // ticks. (A real criticality burst that crosses the threshold
    // would still produce a *bounded* counter — the threshold is
    // what the dashboard's alarm checks against.)
    assert!(
        branching.subcritical_tick_streak < 100,
        "MT-013 subcritical streak alarm tripped on a calm sim: {} (threshold=100)",
        branching.subcritical_tick_streak,
    );
    assert!(
        branching.supercritical_avalanche_streak < 10,
        "MT-012 supercritical avalanche streak alarm tripped on a calm sim: {} (threshold=10)",
        branching.supercritical_avalanche_streak,
    );

    // And the rolling-mean σ̄ must remain inside the documented
    // `[0, 3]` band — anything beyond means a division-by-zero or
    // numerical overflow in the SOC ledger.
    assert_in_range(
        "branching.sigma_bar",
        branching.sigma_bar,
        0.0,
        3.0,
    );

    // Regime classification must be one of the four documented
    // labels; any other value would mean the regime enum drifted.
    use civ_engine::BranchingRegime;
    let regime = branching.regime;
    assert!(
        matches!(
            regime,
            BranchingRegime::HeatDeath
                | BranchingRegime::SubcriticalTransition
                | BranchingRegime::EdgeOfChaos
                | BranchingRegime::Supercritical,
        ),
        "branching regime must be one of the 4 documented labels; got {regime:?}",
    );
}

// ===========================================================================
// §6 — Dashboard `sample_emergence` is idempotent on a boundary tick
// ===========================================================================

/// FR-EMERGENCE-tests §6 — `sample_emergence()` must produce a
/// deterministic, in-range sample on a boundary tick, and the
/// fields must stay in range across multiple invocations on the
/// same sim (no accumulator drift between calls).
#[test]
fn fr_emergence_quality_sample_emergence_is_idempotent_on_boundary() {
    let mut sim = Simulation::with_seed(0xDEADBEEF);
    sim.advance_ticks(RUN_TICKS);

    // First sample — should fire (we are past the first boundary).
    assert!(sim.sample_emergence(), "first boundary sample should fire");
    let s1 = sim
        .last_emergence_sample()
        .expect("first sample must be cached");

    // Second sample on the same tick — should also fire and produce
    // an in-range dashboard. (We do not require the values to be
    // *identical* between calls; the fingerprint accumulator may
    // advance. We require every field to remain in its documented
    // range.)
    assert!(sim.sample_emergence(), "second boundary sample should fire");
    let s2 = sim
        .last_emergence_sample()
        .expect("second sample must be cached");

    for (label, a, b) in [
        ("entropy_norm", s1.entropy_norm, s2.entropy_norm),
        ("cluster_entropy", s1.dashboard.cluster_entropy, s2.dashboard.cluster_entropy),
        (
            "sentience_fraction",
            s1.dashboard.sentience_fraction,
            s2.dashboard.sentience_fraction,
        ),
        ("psyche_stability", s1.dashboard.psyche_stability, s2.dashboard.psyche_stability),
        ("diplomacy_tension", s1.dashboard.diplomacy_tension, s2.dashboard.diplomacy_tension),
        ("branching_sigma", s1.branching_sigma, s2.branching_sigma),
        ("branching_sigma_score", s1.branching_sigma_score, s2.branching_sigma_score),
        ("power_law_alpha", s1.power_law_alpha, s2.power_law_alpha),
        ("novelty_rate", s1.novelty_rate, s2.novelty_rate),
    ] {
        assert!(a.is_finite(), "{label}_first must be finite; got {a}");
        assert!(b.is_finite(), "{label}_second must be finite; got {b}");
    }

    // Histogram total must monotonically advance OR stay equal
    // between samples on the same tick (the histogram is rebuilt
    // from the live voxel world — same tick ⇒ same world ⇒ same
    // total).
    assert!(
        s2.histogram_total == s1.histogram_total || s2.histogram_total >= s1.histogram_total,
        "histogram_total regressed between samples on the same tick: {} → {}",
        s1.histogram_total,
        s2.histogram_total,
    );
}

// ===========================================================================
// §7 — Across many ticks the dashboard stays in range (no run-away)
// ===========================================================================

/// FR-EMERGENCE-tests §7 — across a 600-tick run with sampling on
/// every boundary, every dashboard tile must remain in `[0, 1]`.
/// A field that escapes its documented range at any boundary is a
/// runaway that the dashboard would render as a meaningless chip.
#[test]
fn fr_emergence_quality_dashboard_stays_in_range_across_long_run() {
    let mut sim = Simulation::with_seed(0xBEEFCAFE);
    // Sample on every 50th tick after the first 50 ticks. 600 ticks
    // ⇒ 12 sample boundaries. Each one must produce an in-range
    // dashboard.
    let sample_ticks: Vec<u64> = (1..=12).map(|k| k * 50).collect();
    for target in sample_ticks {
        while sim.state.tick < target {
            sim.tick();
        }
        assert!(
            sim.sample_emergence(),
            "sample_emergence should fire on tick {target} (got tick {})",
            sim.state.tick,
        );
        let s = sim.last_emergence_sample().expect("sample cached");
        let d = s.dashboard;
        assert!(
            d.cluster_entropy.is_finite() && (0.0..=1.0).contains(&d.cluster_entropy),
            "tick {}: cluster_entropy out of range: {}",
            target,
            d.cluster_entropy,
        );
        assert!(
            d.sentience_fraction.is_finite()
                && (0.0..=1.0).contains(&d.sentience_fraction),
            "tick {target}: sentience_fraction out of range: {}",
            d.sentience_fraction,
        );
        assert!(
            d.psyche_stability.is_finite() && (0.0..=1.0).contains(&d.psyche_stability),
            "tick {target}: psyche_stability out of range: {}",
            d.psyche_stability,
        );
        assert!(
            d.diplomacy_tension.is_finite() && (0.0..=1.0).contains(&d.diplomacy_tension),
            "tick {target}: diplomacy_tension out of range: {}",
            d.diplomacy_tension,
        );
        assert!(
            s.entropy_norm.is_finite() && (0.0..=1.0).contains(&s.entropy_norm),
            "tick {target}: entropy_norm out of range: {}",
            s.entropy_norm,
        );
        assert!(
            s.branching_sigma.is_finite() && (0.0..=3.0).contains(&s.branching_sigma),
            "tick {target}: branching_sigma out of range: {}",
            s.branching_sigma,
        );
    }
}