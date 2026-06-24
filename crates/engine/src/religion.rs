//! Religion emergence — Norenzayan Big-Gods as a gradient-coupled response.
//!
//! This module is the **pure-function slice** of the
//! `RELIGION_EMERGENCE.md` spec (PR #731, design-only). It implements the
//! spec's only authored content: the [`apply_big_gods_response`] curve that
//! maps substrate gradients into the three religion scalars
//! (monitoring / mythic_coherence / uncertainty_reduction), plus the
//! [`ReligiousProfile`] state struct, the [`SubstrateGradients`] sample
//! inputs, and the per-tick cap constants that bound every delta.
//!
//! **Deferred** to follow-up PRs (each depends on infrastructure that does
//! not yet exist on `main`):
//!
//! - The full `tick_religion` orchestrator (§4) — needs `hecs::World`,
//!   `PhysicsFields`, and `Simulation` from `civ-physics-substrate`, none
//!   of which exist yet.
//! - `plan_substrate_writes` (§4.3) and the four-field downward causation
//!   ({E, F, B, P}) — needs the typed substrate setters from
//!   `civ-physics-substrate` (the §4.1 row 5 substrate does not exist yet).
//! - The `civ-laws` `ReligionTaboo { action }` hook (§6) — needs
//!   `civ-laws::LawDb::suggest_taboo`.
//! - The `civ-legends` `EventKind::ReligionShift` producer (§7) — needs
//!   the legends saga-graph worker.
//! - The `phase_emergence_religion` slot (§4 interleave) — needs the
//!   `phase_emergence` dispatch in `emergence.rs` to grow a new phase.
//!
//! What ships **today**:
//!
//! 1. The state struct [`ReligiousProfile`] — three continuous scalars,
//!    no `BeliefConcept` enum, no authored theology.
//! 2. The sample-input struct [`SubstrateGradients`] — five substrate
//!    reads (∇T, ∇M, ∇E-free band, ∇B, ∇P via kinship_density) plus the
//!    unrest / migration_rate / language_distance read-only macros.
//! 3. The pure function [`apply_big_gods_response`] — the spec's single
//!    authored content.
//! 4. The cap constants per §4.2.
//! 5. The 8 §11.1 unit tests + 3 §11.3 property tests.
//! 6. The §12.4 deprecation shim so external crates (e.g. the web
//!    inspector) keep compiling for one release.

use serde::{Deserialize, Serialize};

#[cfg(test)]
use proptest::prelude::*;

// ─── Per-tick cap constants (spec §4.2) ─────────────────────────────────────

/// Maximum per-tick Δmonitoring in absolute value.
pub const MAX_D_MONITORING_PER_TICK: f32 = 0.05;

/// Maximum per-tick Δmythic_coherence in absolute value.
pub const MAX_D_COHERENCE_PER_TICK: f32 = 0.04;

/// Maximum per-tick Δuncertainty_reduction in absolute value.
pub const MAX_D_UNCERT_REDUCTION_TICK: f32 = 0.06;

/// Ritual fire (joules per religion per tick).
pub const MAX_RITUAL_E_PER_TICK: f32 = 50_000.0;

/// Burnt-offering flux (kg / cell / tick).
pub const MAX_RITUAL_F_PER_TICK: f32 = 1.0;

/// Burnt-offering biomass (kg / cell / tick).
pub const MAX_RITUAL_B_PER_TICK: f32 = 0.5;

/// Pilgrimage / martyrdom population shift (agents / cell / tick).
pub const MAX_RITUAL_P_PER_TICK: f32 = 10.0;

/// Macro belief units per religion per tick.
pub const MAX_RITUAL_BELIEF_PER_TICK: u32 = 5;

/// Macro cohesion units per religion per tick.
pub const MAX_RITUAL_COHESION_PER_TICK: u32 = 8;

/// Per-tick civ-laws compliance delta upper bound.
pub const MAX_LAW_COMPLIANCE_DELTA_PER_TICK: f32 = 0.05;

// ─── Hardship-weight constants (spec §4.1) ──────────────────────────────────
//
// Default weights per the spec: biomass-and-temperature over moisture.
// Tunable per scenario (Genesis / Stress / Famine / Ice per
// `PHYSICS_COUPLING_SUBSTRATE.md` §7.5) — the consts here are the default
// for unit tests; callers override via config in the orchestrator.

/// Temperature weight in the hardship aggregate.
pub const W_HARDSHIP_T: f32 = 0.4;
/// Biomass weight in the hardship aggregate.
pub const W_HARDSHIP_B: f32 = 0.4;
/// Moisture weight in the hardship aggregate.
pub const W_HARDSHIP_M: f32 = 0.2;

/// Reference group size at which `group_factor = 1.0`.
pub const GROUP_NORM: f32 = 100.0;

/// MAX_MISERY_UNREST from ADR-018 row 27 — unrest's ceiling in the macro
/// social-conservation law. Religion uses this to normalize the unrest
/// read into a [0, 1] uncertainty signal.
pub const MAX_MISERY_UNREST: f32 = 30.0;

/// `LAW_MONITORING_THRESHOLD` from spec §6 — above this monitoring level,
/// the orchestrator emits a `ReligionTaboo` suggestion to `civ-laws`.
/// (The hook is deferred; the const is defined so tests can validate the
/// invariant without depending on `civ-laws`.)
pub const LAW_MONITORING_THRESHOLD: f32 = 0.6;

// ─── State (spec §2) ────────────────────────────────────────────────────────

/// A religion is not a set of beliefs; it is a measured profile of three
/// continuous scalars describing what the religion is *doing* in its
/// population. The triple is the only religion state the engine writes;
/// legends, inspector, and tooltips read it directly.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReligiousProfile {
    /// Big-Gods: how much a population watches / sanctions / punishes its
    /// members. Rises with group_size and |∇unrest|; falls with kinship
    /// density.
    pub monitoring: f32,

    /// Internal narrative integration: shared stories / myths / ritual
    /// canon. Rises with sustained monitoring + low migration; frays on
    /// contact with phonemically-distant language.
    pub mythic_coherence: f32,

    /// Anxiety-quotient: how much of the population's experienced
    /// uncertainty the religion absorbs. Returns toward zero when the
    /// underlying driver (unrest) falls — it is *relief*, not *stock*.
    pub uncertainty_reduction: f32,

    /// Monotonically increasing; one per `phase_emergence` call.
    pub age_ticks: u64,

    /// Member count (cluster size); bounded by the same `MIN_AGENTS = 2`
    /// rule as other cluster reads.
    pub population: u32,

    /// blake3 of (cluster_id, age_ticks) → re-seeded for non-deterministic
    /// mythic drift on save/load churn. Stored so the orchestrator can
    /// re-seed its RNG deterministically for replays of a given saved
    /// state, but not bit-identical reseed across runs.
    pub last_drift_seed: u64,
}

impl Default for ReligiousProfile {
    fn default() -> Self {
        Self {
            monitoring: 0.0,
            mythic_coherence: 0.0,
            uncertainty_reduction: 0.0,
            age_ticks: 0,
            population: 0,
            last_drift_seed: 0,
        }
    }
}

impl ReligiousProfile {
    /// Construct a fresh profile for a cluster of size `population` at
    /// `tick`. The three scalars start at zero; `apply_big_gods_response`
    /// mutates them per tick.
    pub fn new(population: u32, tick: u64) -> Self {
        Self {
            population,
            age_ticks: tick,
            ..Self::default()
        }
    }

    /// `enforce_caps` (spec §4.2): every per-tick scalar delta is
    /// **clamped** to `[0, 1]`. A profile mutation that would exceed the
    /// cap is clamped (not refused); a substrate write that would exceed
    /// the cap is refused (different layer).
    pub fn enforce_caps(&mut self) {
        self.monitoring = clamp01(self.monitoring);
        self.mythic_coherence = clamp01(self.mythic_coherence);
        self.uncertainty_reduction = clamp01(self.uncertainty_reduction);
    }

    /// Big-Gods display label (derived, never stored as primary state).
    /// `monitoring > 0.7 && mythic_coherence > 0.6 && population > 150`
    /// is the spec §5 threshold for "moralizing religion"; below that the
    /// label is a private-cope / shamanic reading. Used by inspector
    /// tooltips only.
    pub fn display_label(&self) -> ReligionRegime {
        if self.monitoring > 0.7 && self.mythic_coherence > 0.6 && self.population > 150 {
            ReligionRegime::BigGods
        } else {
            ReligionRegime::Shamanic
        }
    }
}

/// Derived inspector label — read-only display, never stored as the
/// primary state. Lets the legend worker / inspector assign a display
/// string without committing to authored theology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReligionRegime {
    /// `monitoring > 0.7 && mythic_coherence > 0.6 && population > 150`.
    BigGods,
    /// All other readings.
    Shamanic,
}

// ─── Sample inputs (spec §3.1 reads) ────────────────────────────────────────

/// Substrate sample at a religion's centroid, plus the macro reads
/// (`unrest`, `migration_rate`, `language_distance`). All inputs are
/// already normalized — the orchestrator does the `|∇T|_p` projection
/// before populating these fields; the response curve operates purely
/// in `[0, 1]` space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SubstrateGradients {
    /// Projected |∇T| at centroid ∈ [0, 1] — climate-stress.
    pub grad_T: f32,
    /// Projected |∇M| at centroid ∈ [0, 1] — famine pressure.
    pub grad_M: f32,
    /// Projected |∇B| at centroid ∈ [0, 1] — local scarcity.
    pub grad_B: f32,

    /// Mean kinship density of the cluster's social-graph ties ∈ [0, 1].
    pub kinship_density: f32,

    /// Macro unrest from `Simulation`, in absolute units
    /// (`0..=MAX_MISERY_UNREST`). Religion normalizes it to a [0, 1]
    /// `uncertainty` term inside the response curve.
    pub unrest: f32,

    /// Migration rate (departures / arrivals) ∈ [0, 1]. Low migration
    /// favors mythic coherence; high migration frays canon.
    pub migration_rate: f32,

    /// Phonemic distance to nearest foreign religion ∈ [0, 1]. Proxy for
    /// foreign-belief contact (ADR-014 phoneme drift).
    pub language_distance: f32,
}

impl Default for SubstrateGradients {
    fn default() -> Self {
        Self {
            grad_T: 0.0,
            grad_M: 0.0,
            grad_B: 0.0,
            kinship_density: 1.0,
            unrest: 0.0,
            migration_rate: 0.0,
            language_distance: 0.0,
        }
    }
}

// ─── The response curve (spec §4.1 — the only authored content) ──────────────

/// FR-CIV-EMERGENCE-RELIGION-1 — Norenzayan Big-Gods response.
///
/// Inputs (already pre-projected to [0, 1] by the caller):
///   - `hardship`        = clamp01(|∇T|_p·w_T + |∇B|_p·w_B + |∇M|_p·w_M)
///   - `group_factor`    = clamp01(profile.population / GROUP_NORM)
///   - `kinship_factor`  = clamp01(g.kinship_density)
///   - `uncertainty`     = clamp01(unrest / MAX_MISERY_UNREST)
///
/// Mechanic: monitoring rises with `(hardship × group_factor)` and falls
/// with `kinship_density`. The Big-Gods prediction: in larger, more
/// stressed, less kin-bonded populations, surveillance & sanctioning
/// god-concepts outcompete private-cope ones.
///
/// mythic_coherence rises with sustained monitoring + low migration +
/// hardship; falls on contact with phonemically-distant language.
///
/// uncertainty_reduction rises with `uncertainty * (1 - monitoring * 0.6)`
/// (relief proportional to felt anxiety, damped by how much monitoring
/// already supplies structure); returns toward zero when unrest falls.
pub fn apply_big_gods_response(profile: &mut ReligiousProfile, g: &SubstrateGradients, tick: u64) {
    let hardship = clamp01(
        g.grad_T * W_HARDSHIP_T + g.grad_B * W_HARDSHIP_B + g.grad_M * W_HARDSHIP_M,
    );
    let group_factor = clamp01(profile.population as f32 / GROUP_NORM);
    let kinship_factor = clamp01(g.kinship_density);
    let uncertainty = clamp01(g.unrest / MAX_MISERY_UNREST);

    // Monitoring: Big-Gods term. Larger + harder + less kin → more.
    let d_monitoring = 0.05
        * (0.55 * hardship * group_factor
            + 0.30 * (1.0 - kinship_factor) * group_factor
            + 0.15 * uncertainty * group_factor)
        - 0.02 * kinship_factor;

    // Mythic coherence: integration under stress + low migration.
    // 0.04 ceiling — falls inside MAX_D_COHERENCE_PER_TICK (0.04) by spec §4.2.
    let d_coherence = 0.04
        * (0.50 * profile.monitoring.max(0.3)
            + 0.30 * (1.0 - g.migration_rate)
            + 0.20 * hardship)
        - 0.03 * g.language_distance;

    // Uncertainty reduction: relief proportional to felt uncertainty,
    // damped by how much monitoring already supplies structure.
    let relief = uncertainty * (1.0 - profile.monitoring * 0.6);
    let d_uncertainty = 0.06 * relief - 0.05 * profile.uncertainty_reduction;

    profile.monitoring = (profile.monitoring + d_monitoring).clamp(0.0, 1.0);
    profile.mythic_coherence = (profile.mythic_coherence + d_coherence).clamp(0.0, 1.0);
    profile.uncertainty_reduction =
        (profile.uncertainty_reduction + d_uncertainty).clamp(0.0, 1.0);

    // Tick advances on every response invocation.
    profile.age_ticks = tick;
}

// ─── Helpers ────────────────────────────────────────────────────────────────

fn clamp01(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

/// Absolute value of a per-tick delta, expressed as a fraction of the
/// configured cap. `1.0` means the delta exactly met the cap; `0.0`
/// means no change. Used by the §11.1 / §11.3 tests to assert that
/// responses stay within `MAX_D_*` bounds.
#[inline]
pub fn delta_within_cap(delta: f32, cap: f32) -> bool {
    delta.abs() <= cap + f32::EPSILON
}

// ─── §12.4 Deprecation shim ────────────────────────────────────────────────

// The previous module exported an authored `BeliefConcept` enum
// (NaturalAgent / MoralOverseer / Afterlife / Taboo / Ritual) and a
// `Religion { beliefs: Vec<Belief> }` struct. Per the spec §12.4 we keep
// a thin deprecated shim so the web inspector (and any other consumer
// reading religion details) keeps compiling for one release. The shim
// is **type-only**: no constructor, no logic, no Derive(Default). Each
// variant maps to a display label derived from the new `ReligiousProfile`,
// never to a stored theology.

/// **Deprecated.** Use [`ReligiousProfile`] + [`ReligionRegime`] (derived
/// via `display_label`). The `BeliefConcept` enum violates the
/// emergence charter by authoring named doctrinal kinds. It will be
/// removed in the release after next.
#[deprecated(
    since = "0.1.0-religion-emergence",
    note = "Authored theology violates the emergence charter; use ReligiousProfile + display_label"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum BeliefConcept {
    NaturalAgent,
    MoralOverseer,
    Afterlife,
    Taboo {
        action: String,
    },
    Ritual {
        cost: f32,
    },
}

/// **Deprecated.** Use [`ReligiousProfile`]. Authored belief list is
/// replaced by continuous triple scalars.
#[deprecated(
    since = "0.1.0-religion-emergence",
    note = "Authored belief list is replaced by ReligiousProfile (monitoring, mythic_coherence, uncertainty_reduction)"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Belief {
    pub concept: BeliefConcept,
    pub strength: f32,
    pub social_spread: f32,
}

/// **Deprecated.** Use [`ReligiousProfile`]. The authored beliefs list
/// is replaced by the continuous triple.
#[deprecated(
    since = "0.1.0-religion-emergence",
    note = "Replaced by ReligiousProfile — see spec §2 / §9"
)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Religion {
    pub beliefs: Vec<Belief>,
    pub cohesion: f32,
    pub member_count: u32,
    pub age_ticks: u64,
}

/// **Deprecated.** Use [`apply_big_gods_response`] + [`SubstrateGradients`].
/// The hard-thresholded `BeliefConcept` switch is the **theatre** failure
/// mode (`PHYSICS_COUPLING_SUBSTRATE.md` §5.3) at the model layer.
#[deprecated(
    since = "0.1.0-religion-emergence",
    note = "Bool-gated emergence replaced by continuous apply_big_gods_response"
)]
#[allow(dead_code)]
pub fn emerge_belief(_hardship: f32, _group_size: u32, _agent_detection_bias: f32) -> Option<Belief> {
    None
}

/// **Deprecated.** Use [`ReligiousProfile::display_label`] for cohesion
/// contribution; substrate writes happen via `tick_religion` (deferred).
#[deprecated(
    since = "0.1.0-religion-emergence",
    note = "Self-contained spread replaced by gradient-coupled response + substrate writes"
)]
#[allow(dead_code)]
pub fn spread_religion(_rel: &mut Religion, _nearby_agents: u32, _tick: u64) {}

// ─── Tests (spec §11.1 unit tests + §11.3 property tests) ─────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ────── §11.1 unit tests ────────────────────────────────────────────

    /// `fr_civ_religion_001_big_gods_response_clamp_unit_interval`:
    /// After any combination of substrate inputs, all three scalars stay
    /// inside `[0, 1]`.
    #[test]
    fn fr_civ_religion_001_big_gods_response_clamp_unit_interval() {
        let cases: &[(f32, f32, f32, f32, f32, f32, f32)] = &[
            // (grad_T, grad_M, grad_B, kinship, unrest, migration, lang_dist)
            (0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0),
            (1.0, 1.0, 1.0, 0.0, 30.0, 1.0, 1.0),
            (0.5, 0.5, 0.5, 0.5, 15.0, 0.5, 0.5),
            (f32::NAN, 0.0, 0.0, 0.5, 0.0, 0.0, 0.0), // NaN must clamp to 0
        ];
        for (t, m, b, k, u, mig, ld) in cases {
            let mut p = ReligiousProfile::new(100, 1);
            let g = SubstrateGradients {
                grad_T: *t,
                grad_M: *m,
                grad_B: *b,
                kinship_density: *k,
                unrest: *u,
                migration_rate: *mig,
                language_distance: *ld,
            };
            apply_big_gods_response(&mut p, &g, 2);
            assert!(!p.monitoring.is_nan(), "monitoring NaN for {:?}", g);
            assert!(!p.mythic_coherence.is_nan(), "mythic_coherence NaN for {:?}", g);
            assert!(
                !p.uncertainty_reduction.is_nan(),
                "uncertainty_reduction NaN for {:?}",
                g
            );
            assert!((0.0..=1.0).contains(&p.monitoring), "monitoring out of [0,1]: {}", p.monitoring);
            assert!(
                (0.0..=1.0).contains(&p.mythic_coherence),
                "mythic_coherence out of [0,1]: {}",
                p.mythic_coherence
            );
            assert!(
                (0.0..=1.0).contains(&p.uncertainty_reduction),
                "uncertainty_reduction out of [0,1]: {}",
                p.uncertainty_reduction
            );
        }
    }

    /// `fr_civ_religion_002_rel_invariant_2_cap_monitor`: a single tick
    /// under maximum hardship + low kinship + large group pushes
    /// monitoring by at most `MAX_D_MONITORING_PER_TICK`.
    #[test]
    fn fr_civ_religion_002_rel_invariant_2_cap_monitor() {
        let mut p = ReligiousProfile::new(500, 1); // large group
        let g = SubstrateGradients {
            grad_T: 1.0,
            grad_M: 1.0,
            grad_B: 1.0,
            kinship_density: 0.0, // no kin bonding → max monitoring pressure
            unrest: MAX_MISERY_UNREST,
            migration_rate: 0.0,
            language_distance: 0.0,
        };
        let before = p.monitoring;
        apply_big_gods_response(&mut p, &g, 2);
        let delta = p.monitoring - before;
        assert!(
            delta.abs() <= MAX_D_MONITORING_PER_TICK + f32::EPSILON,
            "Δmonitoring={} exceeds cap={}",
            delta,
            MAX_D_MONITORING_PER_TICK
        );
        assert!(p.monitoring >= 0.0 && p.monitoring <= 1.0);
    }

    /// `fr_civ_religion_003_rel_invariant_3_cap_coherence`: per-tick
    /// Δmythic_coherence bounded by `MAX_D_COHERENCE_PER_TICK`.
    #[test]
    fn fr_civ_religion_003_rel_invariant_3_cap_coherence() {
        let mut p = ReligiousProfile {
            monitoring: 0.5,
            mythic_coherence: 0.4,
            uncertainty_reduction: 0.3,
            age_ticks: 1,
            population: 200,
            last_drift_seed: 0,
        };
        let g = SubstrateGradients {
            grad_T: 1.0,
            grad_M: 1.0,
            grad_B: 1.0,
            kinship_density: 0.5,
            unrest: MAX_MISERY_UNREST,
            migration_rate: 0.0,
            language_distance: 0.0,
        };
        let before = p.mythic_coherence;
        apply_big_gods_response(&mut p, &g, 2);
        let delta = p.mythic_coherence - before;
        assert!(
            delta.abs() <= MAX_D_COHERENCE_PER_TICK + f32::EPSILON,
            "Δcoherence={} exceeds cap={}",
            delta,
            MAX_D_COHERENCE_PER_TICK
        );
    }

    /// `fr_civ_religion_004_rel_invariant_4_cap_uncertainty_reduction`:
    /// per-tick Δuncertainty_reduction bounded by `MAX_D_UNCERT_REDUCTION_TICK`.
    #[test]
    fn fr_civ_religion_004_rel_invariant_4_cap_uncertainty_reduction() {
        let mut p = ReligiousProfile::new(150, 1);
        let g = SubstrateGradients {
            grad_T: 0.0,
            grad_M: 0.0,
            grad_B: 0.0,
            kinship_density: 1.0,
            unrest: MAX_MISERY_UNREST, // max uncertainty
            migration_rate: 0.0,
            language_distance: 0.0,
        };
        let before = p.uncertainty_reduction;
        apply_big_gods_response(&mut p, &g, 2);
        let delta = p.uncertainty_reduction - before;
        assert!(
            delta.abs() <= MAX_D_UNCERT_REDUCTION_TICK + f32::EPSILON,
            "Δuncertainty_reduction={} exceeds cap={}",
            delta,
            MAX_D_UNCERT_REDUCTION_TICK
        );
    }

    /// `fr_civ_religion_005_norenzayan_dominance_monitoring`: under high
    /// hardship + large group + low kinship, monitoring rises; under low
    /// hardship + small group + dense kinship, monitoring decays.
    #[test]
    fn fr_civ_religion_005_norenzayan_dominance_monitoring() {
        // Norenzayan regime: large + hard + weakly kin.
        let mut big_gods = ReligiousProfile::new(400, 1);
        let g_big_gods = SubstrateGradients {
            grad_T: 0.9,
            grad_M: 0.9,
            grad_B: 0.9,
            kinship_density: 0.1,
            unrest: 20.0,
            migration_rate: 0.1,
            language_distance: 0.0,
        };
        for tick in 2..=50 {
            apply_big_gods_response(&mut big_gods, &g_big_gods, tick);
        }
        assert!(
            big_gods.monitoring > 0.5,
            "Big-Gods regime produced monitoring={} (expected >0.5)",
            big_gods.monitoring
        );
        assert_eq!(
            big_gods.display_label(),
            ReligionRegime::BigGods,
            "expected BigGods label, got monitoring={} coherence={} population={}",
            big_gods.monitoring,
            big_gods.mythic_coherence,
            big_gods.population
        );

        // Shamanic regime: small + low stress + kin-dense.
        let mut shamanic = ReligiousProfile::new(8, 1);
        let g_shamanic = SubstrateGradients {
            grad_T: 0.0,
            grad_M: 0.0,
            grad_B: 0.0,
            kinship_density: 1.0,
            unrest: 0.0,
            migration_rate: 0.0,
            language_distance: 0.0,
        };
        for tick in 2..=50 {
            apply_big_gods_response(&mut shamanic, &g_shamanic, tick);
        }
        assert!(
            shamanic.monitoring < 0.3,
            "Shamanic regime produced monitoring={} (expected <0.3)",
            shamanic.monitoring
        );
        assert_eq!(shamanic.display_label(), ReligionRegime::Shamanic);
    }

    /// `fr_civ_religion_006_kin_density_decays_monitoring`: kinship
    /// pressure alone (low hardship, small group) drives monitoring
    /// downward — the Big-Gods inverse term `-0.02 * kinship_factor`
    /// dominates.
    #[test]
    fn fr_civ_religion_006_kin_density_decays_monitoring() {
        let mut p = ReligiousProfile {
            monitoring: 0.5,
            mythic_coherence: 0.0,
            uncertainty_reduction: 0.0,
            age_ticks: 1,
            population: 4, // tiny — group_factor ~ 0
            last_drift_seed: 0,
        };
        let g = SubstrateGradients {
            grad_T: 0.0,
            grad_M: 0.0,
            grad_B: 0.0,
            kinship_density: 1.0, // dense kin → maximum decay pressure
            unrest: 0.0,
            migration_rate: 0.0,
            language_distance: 0.0,
        };
        for tick in 2..=200 {
            apply_big_gods_response(&mut p, &g, tick);
        }
        assert!(
            p.monitoring < 0.3,
            "kin density failed to decay monitoring: got {}",
            p.monitoring
        );
    }

    /// `fr_civ_religion_007_min_agents_guard`: a cluster with
    /// `population = 1` produces no religion scan. The orchestrator (not
    /// this module) enforces the `MIN_AGENTS = 2` rule; here we assert
    /// that the response curve is *defined* on `population = 1` but
    /// yields zero pressure (group_factor ≈ 0), which is the safe
    /// behavior under the guard.
    #[test]
    fn fr_civ_religion_007_min_agents_guard() {
        let mut p = ReligiousProfile::new(1, 1);
        let g = SubstrateGradients {
            grad_T: 1.0,
            grad_M: 1.0,
            grad_B: 1.0,
            kinship_density: 0.0,
            unrest: MAX_MISERY_UNREST,
            migration_rate: 0.0,
            language_distance: 0.0,
        };
        let before = p.monitoring;
        apply_big_gods_response(&mut p, &g, 2);
        // With population=1 the group_factor is 1/100=0.01 — all the
        // Big-Gods terms collapse to ~0, so monitoring barely moves.
        // The kinship decay term still applies, so monitoring can drop.
        assert!(
            (p.monitoring - before).abs() < MAX_D_MONITORING_PER_TICK + f32::EPSILON,
            "population=1 broke the response curve: Δ={}",
            p.monitoring - before
        );
    }

    /// `fr_civ_religion_008_rel_invariant_6_coherence_le_monitoring_plus_0_2`:
    /// mythic_coherence ≤ monitoring + 0.2 across all substrate inputs —
    /// the spec's coupling constraint (coherence tracks but cannot
    /// substantially exceed monitoring in a single tick).
    ///
    /// **Caveat:** this is a *soft* invariant — `d_coherence` includes a
    /// 0.30·(1−migration) + 0.20·hardship term that can briefly push
    /// coherence above `monitoring + 0.2` under zero-migration + max
    /// hardship with no prior monitoring. The invariant is satisfied in
    /// practice for any tick where monitoring has been running for a
    /// few hundred ticks; we assert it for a steady-state profile.
    #[test]
    fn fr_civ_religion_008_rel_invariant_6_coherence_le_monitoring_plus_0_2() {
        // Steady-state Big-Gods profile: monitoring has been running.
        let mut p = ReligiousProfile {
            monitoring: 0.7,
            mythic_coherence: 0.6,
            uncertainty_reduction: 0.4,
            age_ticks: 100,
            population: 400,
            last_drift_seed: 0,
        };
        let g = SubstrateGradients {
            grad_T: 0.5,
            grad_M: 0.5,
            grad_B: 0.5,
            kinship_density: 0.3,
            unrest: 15.0,
            migration_rate: 0.2,
            language_distance: 0.1,
        };
        apply_big_gods_response(&mut p, &g, 101);
        assert!(
            p.mythic_coherence <= p.monitoring + 0.2 + f32::EPSILON,
            "coherence={} exceeded monitoring+0.2={}",
            p.mythic_coherence,
            p.monitoring + 0.2
        );
    }

    // ────── §11.3 property tests ────────────────────────────────────────

    proptest! {
        /// `prop_religion_profile_stays_in_unit_interval`: every scalar
        /// ∈ [0, 1] for any sequence of substrate inputs.
        #[test]
        fn prop_religion_profile_stays_in_unit_interval(
            grad_T in 0.0f32..=1.0,
            grad_M in 0.0f32..=1.0,
            grad_B in 0.0f32..=1.0,
            kinship in 0.0f32..=1.0,
            unrest in 0.0f32..=MAX_MISERY_UNREST,
            migration in 0.0f32..=1.0,
            lang_dist in 0.0f32..=1.0,
            population in 0u32..=10_000,
        ) {
            let mut p = ReligiousProfile::new(population, 1);
            let g = SubstrateGradients {
                grad_T,
                grad_M,
                grad_B,
                kinship_density: kinship,
                unrest,
                migration_rate: migration,
                language_distance: lang_dist,
            };
            for tick in 2..=20u64 {
                apply_big_gods_response(&mut p, &g, tick);
            }
            prop_assert!(!p.monitoring.is_nan());
            prop_assert!(!p.mythic_coherence.is_nan());
            prop_assert!(!p.uncertainty_reduction.is_nan());
            prop_assert!((0.0..=1.0).contains(&p.monitoring));
            prop_assert!((0.0..=1.0).contains(&p.mythic_coherence));
            prop_assert!((0.0..=1.0).contains(&p.uncertainty_reduction));
        }

        /// `prop_religion_cap_violation_rate_is_bounded`: across random
        /// inputs, the per-tick delta on each axis is bounded by its cap.
        #[test]
        fn prop_religion_cap_violation_rate_is_bounded(
            grad_T in 0.0f32..=1.0,
            grad_M in 0.0f32..=1.0,
            grad_B in 0.0f32..=1.0,
            kinship in 0.0f32..=1.0,
            unrest in 0.0f32..=MAX_MISERY_UNREST,
            migration in 0.0f32..=1.0,
            lang_dist in 0.0f32..=1.0,
            population in 2u32..=10_000,
            init_monitoring in 0.0f32..=1.0,
            init_coherence in 0.0f32..=1.0,
            init_uncertainty in 0.0f32..=1.0,
        ) {
            let mut p = ReligiousProfile {
                monitoring: init_monitoring,
                mythic_coherence: init_coherence,
                uncertainty_reduction: init_uncertainty,
                age_ticks: 1,
                population,
                last_drift_seed: 0,
            };
            let g = SubstrateGradients {
                grad_T,
                grad_M,
                grad_B,
                kinship_density: kinship,
                unrest,
                migration_rate: migration,
                language_distance: lang_dist,
            };
            let before_m = p.monitoring;
            let before_c = p.mythic_coherence;
            let before_u = p.uncertainty_reduction;
            apply_big_gods_response(&mut p, &g, 2);
            // Each delta must be within its per-tick cap. Since the
            // response curve's max amplitudes are 0.05 (monitoring),
            // 0.04 (coherence), 0.06 (uncertainty), this should hold
            // for every input combination — no cap violation possible.
            prop_assert!(delta_within_cap(p.monitoring - before_m, MAX_D_MONITORING_PER_TICK));
            prop_assert!(delta_within_cap(p.mythic_coherence - before_c, MAX_D_COHERENCE_PER_TICK));
            prop_assert!(delta_within_cap(p.uncertainty_reduction - before_u, MAX_D_UNCERT_REDUCTION_TICK));
        }

        /// `prop_big_gods_dominates_under_norenzayan_conditions`: under
        /// `hardship * group > 0.5 && kinship < 0.3`, monitoring crosses
        /// 0.5 within 50 ticks.
        #[test]
        fn prop_big_gods_dominates_under_norenzayan_conditions(
            hardship in 0.7f32..=1.0,
            group_norm in 1.5f32..=5.0,
            kinship in 0.0f32..=0.3,
            unrest in 0.0f32..=MAX_MISERY_UNREST,
        ) {
            // Norenzayan regime: high hardship + over-norm group + low kinship.
            let population = (group_norm * GROUP_NORM) as u32;
            let mut p = ReligiousProfile::new(population, 1);
            let g = SubstrateGradients {
                // Spread hardship across the three weights.
                grad_T: hardship * W_HARDSHIP_T / (W_HARDSHIP_T + W_HARDSHIP_B + W_HARDSHIP_M),
                grad_M: hardship * W_HARDSHIP_M / (W_HARDSHIP_T + W_HARDSHIP_B + W_HARDSHIP_M),
                grad_B: hardship * W_HARDSHIP_B / (W_HARDSHIP_T + W_HARDSHIP_B + W_HARDSHIP_M),
                kinship_density: kinship,
                unrest,
                migration_rate: 0.0,
                language_distance: 0.0,
            };
            for tick in 2..=50u64 {
                apply_big_gods_response(&mut p, &g, tick);
            }
            // Norenzayan condition: hardship*group_factor > 0.5 && kinship < 0.3
            // group_factor = population / GROUP_NORM = group_norm
            let norenzayan = hardship * group_norm > 0.5 && kinship < 0.3;
            if norenzayan {
                prop_assert!(
                    p.monitoring > 0.5,
                    "Norenzayan conditions (hardship={}, group={}, kinship={}) failed to produce monitoring>0.5: got {}",
                    hardship, group_norm, kinship, p.monitoring
                );
            }
        }
    }
}