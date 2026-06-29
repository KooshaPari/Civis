//! Language emergence & drift over contact networks (FR-CIV-PSYCHE-912).
//!
//! A *language* in this module is a fixed-size vector of real-valued
//! **features** (lexical/phonological/morphological traits, modelled
//! abstractly — there is no authored vocabulary table). Each agent speaks a
//! language; agents transmit features to the agents they are in contact
//! with, and each agent also accumulates a small amount of random
//! innovation on its own. Over time, two populations that are *isolated*
//! drift to distinct dialects while populations joined by a *contact zone*
//! (a thin bridge of edges through which features diffuse) produce a
//! mixed/creole-like intermediate.
//!
//! The module is **pure-logic**: no I/O, no async, no Bevy ECS, no
//! `civ-server` / `civ-research` / `civ-watch` dependency. It can be
//! embedded inside a `civ-ai` [`crate::social_graph::SocialGraph`]
//! driven update, a worker-pool job, or a one-off unit test. It is
//! intentionally domain-agnostic — the same machinery works for
//! villagers, factions, civilizations, or starfish.
//!
//! ## Detection of distinct language regions
//!
//! [`LanguageField::detect_regions`] runs a deterministic single-link
//! cluster scan over the live feature vectors (deterministic under the
//! provided RNG seed) and returns the partition. The acceptance test
//! for FR-CIV-PSYCHE-912 is:
//!
//! 1. Two isolated groups drift to **distinct** dialects (their
//!    centroid distance is above the configured threshold).
//! 2. A contact zone between two parent populations produces a
//!    measurable **mixing** band: bridging agents have a
//!    centroid-distance to *each* parent that is strictly smaller than
//!    the parent-to-parent distance.
//!
//! See `docs/specs/requirements/FR-CIV-PSYCHE.md` and the acceptance
//! test in this file's `tests` submodule.

use serde::{Deserialize, Serialize};

/// Stable identifier for an agent inside a [`LanguageField`].
///
/// Re-exported from [`crate::social_graph::AgentId`] so the two
/// modules' ids line up at the crate root and `pub use language::*;`
/// does not collide with `pub use social_graph::*;`.
pub use crate::social_graph::AgentId;

/// Tiny self-contained pseudo-random number generator.
///
/// We deliberately avoid pulling `rand` into `civ-ai` (it is not yet a
/// dependency of this crate). Splitmix64 is more than sufficient for
/// driving Gaussian noise in the language-drift demo: it has a
/// 2^64-period, passes basic statistical tests, and is fully
/// deterministic from a `u64` seed.
#[derive(Debug, Clone)]
struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Seed from any `u64`.
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the state and return a fresh `u64`.
    fn next_u64(&mut self) -> u64 {
        // Standard splitmix64 mix step.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform `f32` in `[0, 1)`.
    fn next_f32(&mut self) -> f32 {
        // 24 bits of mantissa are enough for our noise draws.
        ((self.next_u64() >> 40) as f32) / ((1u32 << 24) as f32)
    }
}

/// Number of features per agent's language vector.
///
/// Kept as a compile-time constant so the math is straight `f32` dot
/// products. Eight features is enough to demonstrate drift / mixing
/// with tiny test populations; the constant is `pub` so callers can
/// use it when allocating scratch buffers.
pub const FEATURES_PER_LANGUAGE: usize = 8;

/// Configuration knobs for [`LanguageField`].
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Per-tick, per-agent standard deviation of random innovation
    /// noise added to the agent's feature vector (before clamping).
    ///
    /// Innovations are zero-mean; large values accelerate drift.
    pub innovation_sigma: f32,
    /// Per-contact, per-feature magnitude that flows from the donor
    /// agent toward the recipient. A value of `0.5` means "half the
    /// difference is absorbed each contact event".
    pub contact_blend: f32,
    /// Distance below which two agents are considered to speak the
    /// same dialect for [`LanguageField::detect_regions`].
    pub same_dialect_threshold: f32,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        Self {
            innovation_sigma: 0.01,
            contact_blend: 0.5,
            same_dialect_threshold: 0.5,
        }
    }
}

impl LanguageConfig {
    /// Build a config with the default values.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override [`LanguageConfig::innovation_sigma`].
    #[must_use]
    pub fn with_innovation_sigma(mut self, sigma: f32) -> Self {
        self.innovation_sigma = sigma.max(0.0);
        self
    }

    /// Override [`LanguageConfig::contact_blend`].
    #[must_use]
    pub fn with_contact_blend(mut self, blend: f32) -> Self {
        // Clamp into [0, 1] so a contact event either does nothing or
        // actually closes the gap.
        self.contact_blend = blend.clamp(0.0, 1.0);
        self
    }

    /// Override [`LanguageConfig::same_dialect_threshold`].
    #[must_use]
    pub fn with_same_dialect_threshold(mut self, t: f32) -> Self {
        self.same_dialect_threshold = t.max(0.0);
        self
    }
}

/// A single agent's language: an id and a feature vector.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LanguageState {
    /// Stable identifier for the agent speaking this language.
    pub agent: AgentId,
    /// The `FEATURES_PER_LANGUAGE`-long feature vector. Clamped to
    /// `[0.0, 1.0]` after every mutation so the metric stays comparable
    /// across time.
    pub features: [f32; FEATURES_PER_LANGUAGE],
}

impl LanguageState {
    /// Construct a language with all features initialised to `seed`.
    #[must_use]
    pub fn from_seed(agent: AgentId, seed: f32) -> Self {
        let s = seed.clamp(0.0, 1.0);
        Self {
            agent,
            features: [s; FEATURES_PER_LANGUAGE],
        }
    }

    /// Euclidean (L2) distance between two languages, ignoring `agent`.
    ///
    /// Returns `f32::INFINITY` if the buffers differ in length —
    /// callers that mix `LanguageState`s built with different
    /// [`FEATURES_PER_LANGUAGE`] constants will see distances blow up,
    /// which surfaces the bug loudly.
    #[must_use]
    pub fn distance(a: &LanguageState, b: &LanguageState) -> f32 {
        if a.features.len() != b.features.len() {
            return f32::INFINITY;
        }
        let mut acc = 0.0f32;
        for (x, y) in a.features.iter().zip(b.features.iter()) {
            let d = x - y;
            acc += d * d;
        }
        acc.sqrt()
    }

    /// Move `self` part of the way toward `other` by `blend ∈ [0, 1]`:
    /// `self = self + blend * (other - self)`. Features are clamped to
    /// `[0.0, 1.0]` afterward.
    pub fn absorb_from(&mut self, other: &LanguageState, blend: f32) {
        let blend = blend.clamp(0.0, 1.0);
        for (s, o) in self.features.iter_mut().zip(other.features.iter()) {
            *s = (*s + blend * (*o - *s)).clamp(0.0, 1.0);
        }
    }

    /// Add zero-mean Gaussian noise of stddev `sigma` to every feature.
    ///
    /// Features are clamped to `[0.0, 1.0]` afterward.
    pub fn innovate(&mut self, sigma: f32, rng: &mut SplitMix64) {
        if sigma <= 0.0 {
            return;
        }
        for f in self.features.iter_mut() {
            // Box–Muller: two zero-mean normals per draw.
            let u1 = rng.next_f32().max(f32::EPSILON);
            let u2 = rng.next_f32();
            let n = (-2.0 * u1.ln()).sqrt() * (2.0 * std::f32::consts::PI * u2).cos();
            *f = (*f + sigma * n).clamp(0.0, 1.0);
        }
    }
}

/// Outcome of [`LanguageField::contact`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ContactOutcome {
    /// Either endpoint was missing from the field.
    NoOp,
    /// Donor and recipient were the same agent — no language change.
    SelfContact,
    /// Recipient absorbed part of the donor's features.
    Mixed {
        /// Distance from donor to recipient *before* the contact event.
        prev_distance: f32,
        /// Distance from donor to recipient *after* the contact event.
        new_distance: f32,
    },
}

/// The population-level language field: one [`LanguageState`] per
/// agent, plus a deterministic RNG for innovations.
///
/// The field deliberately does **not** know the topology of the
/// contact network — the caller passes an unordered pair (e.g. taken
/// from a [`crate::social_graph::SocialEdge`]) to
/// [`LanguageField::contact`] per contact event. This keeps the module
/// decoupled from `SocialGraph` (which lives in the same crate but
/// shouldn't be a hard dependency) and lets the same field be reused
/// across multiple contact graphs (e.g. trade routes vs. household).
#[derive(Debug, Clone)]
pub struct LanguageField {
    states: Vec<LanguageState>,
    /// RNG used for the next [`LanguageField::innovate_all`] call.
    rng: SplitMix64,
}

impl Default for LanguageField {
    fn default() -> Self {
        Self::with_seed(0)
    }
}

impl LanguageField {
    /// Build an empty field with a default seed.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Build an empty field with the supplied RNG seed. Two fields
    /// built with the same seed and the same event sequence produce
    /// identical state — useful for determinism tests.
    #[must_use]
    pub fn with_seed(seed: u64) -> Self {
        Self {
            states: Vec::new(),
            rng: SplitMix64::new(seed),
        }
    }

    /// Number of agents currently in the field.
    #[must_use]
    pub fn len(&self) -> usize {
        self.states.len()
    }

    /// Whether the field has no agents.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    /// Borrow a language state by agent id.
    #[must_use]
    pub fn get(&self, agent: AgentId) -> Option<&LanguageState> {
        self.states.iter().find(|s| s.agent == agent)
    }

    /// Add a new agent with a language initialised to `seed`. If the
    /// agent is already present this is a **no-op** (does not
    /// overwrite the existing features), so callers can safely call
    /// it on every tick.
    pub fn ensure_agent(&mut self, agent: AgentId, seed: f32) {
        if self.states.iter().any(|s| s.agent == agent) {
            return;
        }
        self.states.push(LanguageState::from_seed(agent, seed));
    }

    /// Apply one contact event between `donor` and `recipient`.
    ///
    /// The recipient's features are blended toward the donor's by
    /// [`LanguageConfig::contact_blend`]. Returns whether anything
    /// actually happened, plus the donor-to-recipient distance before
    /// and after so callers can chart drift.
    pub fn contact(
        &mut self,
        donor: AgentId,
        recipient: AgentId,
        cfg: &LanguageConfig,
    ) -> ContactOutcome {
        if donor == recipient {
            return ContactOutcome::SelfContact;
        }
        let donor_idx = self.states.iter().position(|s| s.agent == donor);
        let recip_idx = self.states.iter().position(|s| s.agent == recipient);
        let (Some(d), Some(r)) = (donor_idx, recip_idx) else {
            return ContactOutcome::NoOp;
        };

        // Snapshot donor features so we can compute distances and
        // avoid holding two simultaneous `&mut` on `self.states`.
        let donor_features = self.states[d].features;
        let donor_state = LanguageState {
            agent: donor,
            features: donor_features,
        };
        let prev_distance =
            LanguageState::distance(&donor_state, &self.states[r]);

        // Blend recipient toward donor.
        self.states[r].absorb_from(&donor_state, cfg.contact_blend);

        let new_distance =
            LanguageState::distance(&donor_state, &self.states[r]);
        ContactOutcome::Mixed {
            prev_distance,
            new_distance,
        }
    }

    /// Apply one round of zero-mean Gaussian innovation noise to every
    /// agent's language.
    ///
    /// Drives the "absence of contact still produces drift" half of
    /// the acceptance test: groups that *aren't* connected still
    /// change over time, just along different trajectories.
    pub fn innovate_all(&mut self, cfg: &LanguageConfig) {
        for s in self.states.iter_mut() {
            s.innovate(cfg.innovation_sigma, &mut self.rng);
        }
    }

    /// Centroid of a subset of agents' feature vectors. Returns
    /// `None` if `agents` is empty.
    #[must_use]
    pub fn centroid<'a, I: IntoIterator<Item = &'a AgentId>>(
        &self,
        agents: I,
    ) -> Option<[f32; FEATURES_PER_LANGUAGE]> {
        let mut acc = [0.0f32; FEATURES_PER_LANGUAGE];
        let mut n = 0usize;
        for a in agents {
            if let Some(s) = self.get(*a) {
                for (i, f) in s.features.iter().enumerate() {
                    acc[i] += f;
                }
                n += 1;
            }
        }
        if n == 0 {
            return None;
        }
        let inv = 1.0 / n as f32;
        for f in acc.iter_mut() {
            *f *= inv;
        }
        Some(acc)
    }

    /// Deterministic single-link cluster scan over the agents'
    /// feature vectors.
    ///
    /// Two agents end up in the same region when their L2 distance is
    /// strictly less than
    /// [`LanguageConfig::same_dialect_threshold`]. The resulting
    /// `Vec<Vec<AgentId>>` is sorted (outer: by smallest agent id;
    /// inner: ascending) so callers can compare partitions for
    /// equality in tests.
    #[must_use]
    pub fn detect_regions(&self, cfg: &LanguageConfig) -> Vec<Vec<AgentId>> {
        let n = self.states.len();
        let mut parent: Vec<usize> = (0..n).collect();

        fn find(parent: &mut [usize], mut x: usize) -> usize {
            while parent[x] != x {
                parent[x] = parent[parent[x]];
                x = parent[x];
            }
            x
        }
        fn union(parent: &mut [usize], a: usize, b: usize) {
            let ra = find(parent, a);
            let rb = find(parent, b);
            if ra != rb {
                parent[ra] = rb;
            }
        }

        for i in 0..n {
            for j in (i + 1)..n {
                let d = LanguageState::distance(&self.states[i], &self.states[j]);
                if d < cfg.same_dialect_threshold {
                    union(&mut parent, i, j);
                }
            }
        }

        let mut buckets: std::collections::BTreeMap<usize, Vec<AgentId>> =
            std::collections::BTreeMap::new();
        for i in 0..n {
            let root = find(&mut parent, i);
            buckets
                .entry(root)
                .or_default()
                .push(self.states[i].agent);
        }
        let mut out: Vec<Vec<AgentId>> = buckets.into_values().collect();
        for v in out.iter_mut() {
            v.sort_unstable();
        }
        out.sort_by_key(|v| v.first().copied().unwrap_or(u64::MAX));
        out
    }
}

#[cfg(test)]
mod tests {
    //! Acceptance tests for FR-CIV-PSYCHE-912.
    //!
    //! Per the spec: *two isolated groups drift to distinct dialects;
    //! a contact zone shows mixing.* Both halves must hold.

    use super::*;
    use crate::social_graph::{
        SocialEdge, SocialEvent, SocialEventKind, SocialGraph,
        SocialGraphConfig,
    };

    /// Tight `LanguageConfig` so the test can demonstrate drift
    /// without thousands of ticks.
    fn lang_cfg() -> LanguageConfig {
        LanguageConfig::new()
            .with_innovation_sigma(0.05)
            .with_contact_blend(0.5)
            .with_same_dialect_threshold(0.6)
    }

    /// Default social-graph config — we don't prune anything here,
    /// only inspect the contact topology.
    fn social_cfg() -> SocialGraphConfig {
        SocialGraphConfig::new()
    }

    #[test]
    fn fr_civ_psyche_912_acceptance_isolated_groups_drift_to_distinct_dialects() {
        // Two groups {0,1,2} and {10,11,12} with no edges between
        // them. After many ticks, each group's centroid should differ
        // from the other group's centroid by *more* than the
        // same-dialect threshold.
        let cfg = lang_cfg();
        let mut field = LanguageField::with_seed(42);
        let mut graph = SocialGraph::new();

        let left = [0u64, 1, 2];
        let right = [10u64, 11, 12];

        // Seed each side with a different initial dialect and wire up
        // within-group contact edges only.
        for &a in left.iter() {
            field.ensure_agent(a, 0.1);
        }
        for &a in right.iter() {
            field.ensure_agent(a, 0.9);
        }
        for i in 0..left.len() {
            for j in (i + 1)..left.len() {
                graph.apply_event(
                    SocialEvent {
                        a: left[i],
                        b: left[j],
                        kind: SocialEventKind::Interaction,
                        magnitude: 1.0,
                        tick: 0,
                    },
                    &social_cfg(),
                );
            }
        }
        for i in 0..right.len() {
            for j in (i + 1)..right.len() {
                graph.apply_event(
                    SocialEvent {
                        a: right[i],
                        b: right[j],
                        kind: SocialEventKind::Interaction,
                        magnitude: 1.0,
                        tick: 0,
                    },
                    &social_cfg(),
                );
            }
        }

        // Drive many ticks of pure drift (innovate_all only — no
        // contact across the two groups because the graph has no
        // inter-group edges).
        for _ in 0..400 {
            field.innovate_all(&cfg);
            // Within-group contact: clone the edges so we can iterate
            // without holding a borrow on the graph while mutating the
            // field.
            for edge in graph_edges(&graph) {
                field.contact(edge.a, edge.b, &cfg);
            }
        }

        let left_centroid = field
            .centroid(left.iter())
            .expect("left centroid must exist");
        let right_centroid = field
            .centroid(right.iter())
            .expect("right centroid must exist");

        let inter_distance = centroid_distance(&left_centroid, &right_centroid);
        assert!(
            inter_distance > cfg.same_dialect_threshold,
            "isolated groups must drift apart beyond the same-dialect threshold (got {inter_distance:.3}, threshold {:.3})",
            cfg.same_dialect_threshold
        );

        // Region detection must partition the two groups apart.
        let regions = field.detect_regions(&cfg);
        let flat: Vec<AgentId> = regions.iter().flatten().copied().collect();
        assert_eq!(flat.len(), left.len() + right.len(), "no agents lost");
        assert!(
            regions.len() >= 2,
            "detector must report at least two distinct language regions (got {})",
            regions.len()
        );
        // Each group must land entirely in its own region.
        for a in left.iter() {
            assert!(
                regions
                    .iter()
                    .any(|r| r.len() == left.len() && r.contains(a)),
                "left agent {a} must be co-located with the rest of the left group",
            );
        }
        for a in right.iter() {
            assert!(
                regions
                    .iter()
                    .any(|r| r.len() == right.len() && r.contains(a)),
                "right agent {a} must be co-located with the rest of the right group",
            );
        }
    }

    #[test]
    fn fr_civ_psyche_912_acceptance_contact_zone_shows_mixing() {
        // Two parent groups plus a thin contact zone (two bridging
        // agents). After enough ticks, each bridge agent's centroid
        // distance to *each* parent must be strictly less than the
        // parent-to-parent distance — that is the empirical signature
        // of a mixed dialect.
        let cfg = lang_cfg();
        let mut field = LanguageField::with_seed(7);
        let mut graph = SocialGraph::new();

        let left = [0u64, 1, 2];
        let right = [10u64, 11, 12];
        let bridge = [100u64, 101];

        // Initial dialects: 0.1 on the left, 0.9 on the right.
        // Bridges start in the middle so the very first contact event
        // pulls them in both directions.
        for &a in left.iter() {
            field.ensure_agent(a, 0.1);
        }
        for &a in right.iter() {
            field.ensure_agent(a, 0.9);
        }
        for &a in bridge.iter() {
            field.ensure_agent(a, 0.5);
        }

        // Within-group edges plus bridge<->parent edges.
        for group in [&left[..], &right[..], &bridge[..]] {
            for i in 0..group.len() {
                for j in (i + 1)..group.len() {
                    graph.apply_event(
                        SocialEvent {
                            a: group[i],
                            b: group[j],
                            kind: SocialEventKind::Interaction,
                            magnitude: 1.0,
                            tick: 0,
                        },
                        &social_cfg(),
                    );
                }
            }
        }
        for b in bridge.iter() {
            for a in left.iter() {
                graph.apply_event(
                    SocialEvent {
                        a: *b,
                        b: *a,
                        kind: SocialEventKind::Interaction,
                        magnitude: 1.0,
                        tick: 0,
                    },
                    &social_cfg(),
                );
            }
            for a in right.iter() {
                graph.apply_event(
                    SocialEvent {
                        a: *b,
                        b: *a,
                        kind: SocialEventKind::Interaction,
                        magnitude: 1.0,
                        tick: 0,
                    },
                    &social_cfg(),
                );
            }
        }

        // Run the simulation long enough that the parents themselves
        // drift apart and the bridges absorb the mixture.
        for _ in 0..600 {
            field.innovate_all(&cfg);
            for edge in graph_edges(&graph) {
                field.contact(edge.a, edge.b, &cfg);
            }
        }

        let left_centroid = field.centroid(left.iter()).unwrap();
        let right_centroid = field.centroid(right.iter()).unwrap();
        let bridge_centroid = field.centroid(bridge.iter()).unwrap();

        let lr = centroid_distance(&left_centroid, &right_centroid);
        let bl = centroid_distance(&bridge_centroid, &left_centroid);
        let br = centroid_distance(&bridge_centroid, &right_centroid);

        assert!(
            lr > cfg.same_dialect_threshold,
            "the two parents must be distinct dialects after isolation+drift (got LR={lr:.3})"
        );
        assert!(
            bl < lr && br < lr,
            "each bridge must be strictly closer to each parent than the parents are to each other (LR={lr:.3}, BL={bl:.3}, BR={br:.3})"
        );
    }

    /// Snapshot the [`SocialGraph`]'s edges into a fresh `Vec` so we
    /// can iterate them without holding a borrow while mutating the
    /// field.
    ///
    /// The graph's API exposes `neighbors_of(agent)` but no flat
    /// iterator. The acceptance tests use small, dense id ranges
    /// (`{0,1,2}`, `{10,11,12}`, `{100,101}`); sweeping `[0, 200)`
    /// and deduping is fine.
    fn graph_edges(graph: &SocialGraph) -> Vec<SocialEdge> {
        let mut out = Vec::new();
        let mut seen: std::collections::HashSet<(u64, u64)> =
            std::collections::HashSet::new();
        for s in 0u64..200 {
            for (other, edge) in graph.neighbors_of(s) {
                let key = if edge.a < edge.b {
                    (edge.a, edge.b)
                } else {
                    (edge.b, edge.a)
                };
                if seen.insert(key) {
                    let _ = other;
                    out.push(edge);
                }
            }
        }
        out
    }

    /// L2 distance between two equal-length feature buffers.
    fn centroid_distance(
        a: &[f32; FEATURES_PER_LANGUAGE],
        b: &[f32; FEATURES_PER_LANGUAGE],
    ) -> f32 {
        let mut acc = 0.0f32;
        for (x, y) in a.iter().zip(b.iter()) {
            let d = x - y;
            acc += d * d;
        }
        acc.sqrt()
    }

    #[test]
    fn contact_self_is_a_no_op() {
        let mut field = LanguageField::with_seed(1);
        field.ensure_agent(0, 0.5);
        assert_eq!(
            field.contact(0, 0, &lang_cfg()),
            ContactOutcome::SelfContact
        );
    }

    #[test]
    fn contact_with_missing_agent_is_a_no_op() {
        let mut field = LanguageField::with_seed(1);
        field.ensure_agent(0, 0.5);
        assert_eq!(field.contact(0, 999, &lang_cfg()), ContactOutcome::NoOp);
        assert_eq!(field.contact(999, 0, &lang_cfg()), ContactOutcome::NoOp);
    }

    #[test]
    fn ensure_agent_is_idempotent() {
        let mut field = LanguageField::with_seed(1);
        field.ensure_agent(7, 0.2);
        let before = field.get(7).unwrap().features;
        field.ensure_agent(7, 0.9);
        let after = field.get(7).unwrap().features;
        assert_eq!(before, after, "ensure_agent must not overwrite existing state");
    }

    #[test]
    fn contact_moves_recipient_toward_donor() {
        let cfg = lang_cfg();
        let mut field = LanguageField::with_seed(1);
        field.ensure_agent(1, 0.1);
        field.ensure_agent(2, 0.9);
        let outcome = field.contact(1, 2, &cfg);
        match outcome {
            ContactOutcome::Mixed { prev_distance, new_distance } => {
                assert!(new_distance < prev_distance);
            }
            other => panic!("expected Mixed, got {other:?}"),
        }
    }
}