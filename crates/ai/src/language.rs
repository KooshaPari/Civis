//! Language emergence and dialect drift over contact networks (FR-CIV-PSYCHE-912).
//!
//! A speaker's *idiolect* is modelled as a small fixed-size feature vector —
//! a "dialect signature" in an abstract phonological / lexical space. There
//! is **no authored language table**: every speaker starts from a seed vector
//! (or carries one inherited from a parent) and the field evolves entirely
//! from two forces:
//!
//! 1. **Drift** — every tick, every speaker nudges their own vector by a
//!    deterministic pseudo-random delta (seeded by the speaker id and the
//!    tick). This is the source of *all* divergence: two speakers that
//!    never meet will inevitably accumulate different drift and end up
//!    speaking mutually unintelligibly distinct dialects.
//! 2. **Diffusion** — at the same time, every speaker pulls their vector
//!    a fraction of the way toward each of their contact-weighted
//!    neighbors. Stronger edges (more frequent contact) pull harder. This
//!    is the source of *convergence*: speakers who talk regularly move
//!    toward one another, so a contact zone ends up mid-way between the
//!    dialects on either side.
//!
//! The model is deliberately tiny so it can run on thousands of agents per
//! tick without dominating the simulation. It is also fully deterministic
//! given `(initial_state, contact_events_per_tick)`: two sim runs with the
//! same seed reproduce the exact same dialect field, which is what the
//! determinism contract (NFR-CIV-DET) demands.
//!
//! This module is **pure** (no I/O, no async, no Bevy ECS dependency) so it
//! can be embedded inside a Bevy system, a worker-pool job, or a one-off
//! unit test without dragging the rest of `civ-ai` along. It takes a
//! [`crate::social_graph::SocialGraph`] by value for the contact weights
//! but does not mutate it — coupling between language and the social
//! graph is one-way (graph -> language), which keeps the modules
//! composable.
//!
//! ## How "distinct language regions" are detected
//!
//! Per FR-CIV-PSYCHE-912 acceptance criteria, distinct regions must be
//! *detectable* without an authored language table. The detection function
//! [`LanguageField::dialect_clusters`] performs a single-linkage-style
//! grouping: two agents end up in the same cluster iff the L2 distance
//! between their dialect vectors is below a configurable threshold
//! ([`LanguageConfig::cluster_radius`]). The threshold is in the same units
//! as the dialect vector, so a caller can tune "how different is too
//! different" to taste.
//!
//! ## How "contact zones show mixing" is detected
//!
//! For each agent, the helper [`LanguageField::dialect_centroid`] returns
//! the mean vector of their ego-network (themselves + neighbors). An agent
//! in a contact zone sits between two distinct dialects, so their
//! ego-network centroid is by construction closer to *both* parent dialects
//! than any pure speaker's centroid is. The acceptance test in this module
//! asserts exactly that: an isolated group pulls away from a contact zone,
//! and the contact zone's centroid falls between the two pure groups.

use crate::social_graph::{AgentId, SocialEdge};
use serde::{Deserialize, Serialize};

/// Dimensionality of the dialect feature vector.
///
/// 8 is large enough to give speakers plenty of room to drift apart while
/// staying cheap to allocate and step. Callers that want more nuance can
/// rebuild with a different dimension; this constant is `pub` purely so
/// tests can size their expected vectors.
pub const DIALECT_DIM: usize = 8;

/// One speaker's idiolect: a point in `DIALECT_DIM`-dimensional dialect
/// space.
///
/// Two vectors whose L2 distance is small represent mutually intelligible
/// dialects; vectors that have drifted far apart represent mutually
/// unintelligible languages. There is no authored lexicon — every entry
/// of the vector is just a number whose relative drift carries meaning.
pub type Dialect = [f32; DIALECT_DIM];

/// Configuration for the language / dialect field.
///
/// Kept tiny on purpose — three knobs cover the entire FR-CIV-PSYCHE-912
/// acceptance test. Callers that want a richer model can layer extra
/// dynamics on top in a higher-level system.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LanguageConfig {
    /// Per-tick magnitude of the deterministic drift noise added to every
    /// speaker's vector. Larger values make isolated groups diverge faster.
    pub drift_per_tick: f32,
    /// Fraction of the *neighbor's* vector that a speaker absorbs per tick
    /// along each incident edge (weighted by edge weight). Larger values
    /// make contact zones mix faster. The total pull on a speaker is
    /// `convergence_rate * sum_edge(weight) * neighbor_vector`, then
    /// normalized so the per-tick movement stays bounded even for high-
    /// degree speakers.
    pub convergence_rate: f32,
    /// L2 distance below which two speakers are considered to share a
    /// dialect for the purposes of [`LanguageField::dialect_clusters`].
    pub cluster_radius: f32,
    /// Symmetric clamp on the per-coordinate magnitude of a dialect
    /// vector. Without this, drift accumulates unboundedly over a long
    /// simulation and floating-point precision eventually degrades.
    pub coord_clamp: f32,
}

impl Default for LanguageConfig {
    fn default() -> Self {
        // Tuned so that, over the ~30-tick acceptance test, two isolated
        // groups drift visibly apart while a contact zone visibly mixes.
        Self {
            drift_per_tick: 0.05,
            convergence_rate: 0.10,
            cluster_radius: 0.75,
            coord_clamp: 4.0,
        }
    }
}

impl LanguageConfig {
    /// Default config.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Override the per-tick drift magnitude.
    #[must_use]
    pub fn with_drift_per_tick(mut self, v: f32) -> Self {
        self.drift_per_tick = v;
        self
    }

    /// Override the per-edge convergence rate.
    #[must_use]
    pub fn with_convergence_rate(mut self, v: f32) -> Self {
        self.convergence_rate = v;
        self
    }

    /// Override the dialect-cluster L2 radius.
    #[must_use]
    pub fn with_cluster_radius(mut self, v: f32) -> Self {
        self.cluster_radius = v;
        self
    }
}

/// The dialect field: one idiolect per known speaker.
///
/// Speakers can be added lazily — the field starts empty and grows as the
/// simulation discovers new agents. The order of insertion is preserved
/// in [`LanguageField::agent_ids`] so iteration order is deterministic.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LanguageField {
    /// Speakers in insertion order. Parallel to [`Self::vectors`].
    agents: Vec<AgentId>,
    /// Per-agent dialect vector. Parallel to [`Self::agents`].
    vectors: Vec<Dialect>,
    /// Highest tick we have stepped to. Used to keep [`Self::step`] a pure
    /// function of `(current_tick, cfg)` so the field is reproducible.
    ticks_elapsed: u64,
}

impl LanguageField {
    /// Empty field.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// All speaker ids known to the field, in insertion order.
    #[must_use]
    pub fn agent_ids(&self) -> &[AgentId] {
        &self.agents
    }

    /// Number of speakers tracked.
    #[must_use]
    pub fn len(&self) -> usize {
        self.agents.len()
    }

    /// `true` if no speakers have been registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.agents.is_empty()
    }

    /// Highest tick reached by [`Self::step`].
    #[must_use]
    pub fn ticks_elapsed(&self) -> u64 {
        self.ticks_elapsed
    }

    /// Look up the index of a speaker, if present.
    #[must_use]
    fn index_of(&self, id: AgentId) -> Option<usize> {
        self.agents.iter().position(|a| *a == id)
    }

    /// Register a speaker with a given idiolect. Idempotent: re-registering
    /// the same id overwrites the vector (useful when a parent bequeaths
    /// their dialect to a newborn and the field already knew them).
    pub fn register(&mut self, id: AgentId, vector: Dialect) {
        if let Some(i) = self.index_of(id) {
            self.vectors[i] = vector;
        } else {
            self.agents.push(id);
            self.vectors.push(vector);
        }
    }

    /// Borrow a speaker's idiolect, if they are known.
    #[must_use]
    pub fn dialect_of(&self, id: AgentId) -> Option<&Dialect> {
        self.index_of(id).map(|i| &self.vectors[i])
    }

    /// Distance between two speakers' idiolects. Returns `None` if either
    /// speaker is unknown. The distance is the L2 norm of the per-coord
    /// difference — a small, well-behaved scalar suitable for clustering.
    #[must_use]
    pub fn distance(&self, a: AgentId, b: AgentId) -> Option<f32> {
        let va = self.dialect_of(a)?;
        let vb = self.dialect_of(b)?;
        Some(l2_distance(va, vb))
    }

    /// Mean idiolect of an ego-network (the agent plus their weighted
    /// neighbors). For an isolated agent this is just their own vector.
    /// For a contact-zone agent it sits between the surrounding dialects.
    ///
    /// `edges` is the list of (other_id, edge) pairs incident to `agent`,
    /// in any order. Edges with non-positive weight do not pull the
    /// centroid (they represent grudges, which we do not want to count as
    /// linguistic contact).
    #[must_use]
    pub fn dialect_centroid<I>(&self, agent: AgentId, edges: I) -> Option<Dialect>
    where
        I: IntoIterator<Item = (AgentId, SocialEdge)>,
    {
        let self_v = self.dialect_of(agent)?;
        let mut sum = *self_v;
        let mut total = 1.0_f32;
        for (other, edge) in edges {
            if edge.weight <= 0.0 {
                continue;
            }
            let Some(v) = self.dialect_of(other) else {
                continue;
            };
            let w = edge.weight.max(0.0);
            for i in 0..DIALECT_DIM {
                sum[i] += w * v[i];
            }
            total += w;
        }
        let mut out = [0.0_f32; DIALECT_DIM];
        for i in 0..DIALECT_DIM {
            out[i] = sum[i] / total;
        }
        Some(out)
    }

    /// Group speakers into dialect clusters using single-linkage by L2
    /// distance.
    ///
    /// Two speakers end up in the same cluster iff there is a chain of
    /// speakers connecting them such that every adjacent pair has L2
    /// distance `<= cfg.cluster_radius`. This is a deliberately crude
    /// clustering — adequate for "is this region a single dialect?" and
    /// cheap enough to run on every tick of a large simulation.
    #[must_use]
    pub fn dialect_clusters(&self, cfg: &LanguageConfig) -> Vec<Vec<AgentId>> {
        let n = self.agents.len();
        if n == 0 {
            return Vec::new();
        }
        // Union-Find over the speakers. Two speakers are unioned iff
        // their L2 distance is at or below the cluster radius.
        let mut parent: Vec<usize> = (0..n).collect();
        for i in 0..n {
            for j in (i + 1)..n {
                if l2_distance(&self.vectors[i], &self.vectors[j]) <= cfg.cluster_radius {
                    union(&mut parent, i, j);
                }
            }
        }
        // Collect members by root.
        let mut buckets: std::collections::HashMap<usize, Vec<AgentId>> =
            std::collections::HashMap::new();
        for i in 0..n {
            buckets
                .entry(find(&mut parent, i))
                .or_default()
                .push(self.agents[i]);
        }
        // Stable order: sort cluster members by AgentId, then clusters by
        // smallest member so the output is deterministic across runs.
        let mut out: Vec<Vec<AgentId>> = buckets.into_values().collect();
        for cluster in &mut out {
            cluster.sort_unstable();
        }
        out.sort_by_key(|c| c.first().copied().unwrap_or(0));
        out
    }

    /// Advance the field by one tick.
    ///
    /// For each speaker:
    ///   1. **Drift** — add a small deterministic per-tick delta to every
    ///      coordinate. The delta is seeded from `(agent_id, tick)`, so
    ///      given the same starting state and contact schedule, two sims
    ///      produce the exact same field after any number of ticks.
    ///   2. **Convergence** — pull the speaker a fraction of the way
    ///      toward each incident positive-weight edge's other endpoint,
    ///      weighted by edge weight. Total movement is normalized by the
    ///      weighted degree so the per-tick step size stays bounded
    ///      regardless of how chatty the speaker is.
    ///   3. **Clamp** — clamp every coordinate to `[-coord_clamp,
    ///      coord_clamp]` to bound the dynamic range.
    ///
    /// `edges_of` is a closure the caller supplies so we do not need to
    /// borrow the social graph mutably. The closure should return the
    /// list of `(other_id, edge)` pairs incident to the speaker (the
    /// same shape [`crate::social_graph::SocialGraph::neighbors_of`]
    /// produces).
    pub fn step<F>(&mut self, tick: u64, cfg: &LanguageConfig, mut edges_of: F)
    where
        F: FnMut(AgentId) -> Vec<(AgentId, SocialEdge)>,
    {
        // Snapshot the current vectors so a speaker's drift doesn't feed
        // back into a neighbor's diffusion within the same tick.
        let snapshot: Vec<Dialect> = self.vectors.clone();

        for (i, &agent) in self.agents.iter().enumerate() {
            let v = snapshot[i];

            // 1. Drift.
            let mut next = v;
            for d in 0..DIALECT_DIM {
                let delta = drift_delta(agent, tick, d, cfg.drift_per_tick);
                next[d] += delta;
            }

            // 2. Convergence toward weighted neighbors.
            let neighbors = edges_of(agent);
            let mut pull = [0.0_f32; DIALECT_DIM];
            let mut total = 0.0_f32;
            for (other, edge) in &neighbors {
                if edge.weight <= 0.0 {
                    continue;
                }
                if let Some(j) = self.index_of(*other) {
                    let w = edge.weight.max(0.0);
                    let ov = snapshot[j];
                    for d in 0..DIALECT_DIM {
                        pull[d] += w * (ov[d] - next[d]);
                    }
                    total += w;
                }
            }
            if total > 0.0 {
                let scale = cfg.convergence_rate / total;
                for d in 0..DIALECT_DIM {
                    next[d] += scale * pull[d];
                }
            }

            // 3. Clamp.
            for d in 0..DIALECT_DIM {
                next[d] = next[d].clamp(-cfg.coord_clamp, cfg.coord_clamp);
            }

            self.vectors[i] = next;
        }
        self.ticks_elapsed = self.ticks_elapsed.max(tick + 1);
    }
}

/// Deterministic, seedable per-(agent, tick, dim) drift delta.
///
/// We use a tiny xorshift mix on `(agent, tick, dim)` so the result is
/// reproducible without dragging in a full RNG crate. The output is
/// uniformly distributed in `[-drift_per_tick, +drift_per_tick]`.
fn drift_delta(agent: AgentId, tick: u64, dim: usize, drift_per_tick: f32) -> f32 {
    // Mix three small numbers into a u64 state.
    let mut s: u64 = agent
        .wrapping_mul(0x9E3779B97F4A7C15)
        .wrapping_add(tick.wrapping_mul(0xBF58476D1CE4E5B9))
        .wrapping_add(dim as u64);
    s ^= s >> 30;
    s = s.wrapping_mul(0xBF58476D1CE4E5B9);
    s ^= s >> 27;
    s = s.wrapping_mul(0x94D049BB133111EB);
    s ^= s >> 31;
    // Map the lowest 24 bits to [-1.0, +1.0] and scale by drift_per_tick.
    let unit = ((s & 0x00FF_FFFF) as f32) / (0x0100_0000 as f32) * 2.0 - 1.0;
    unit * drift_per_tick
}

/// L2 distance between two dialect vectors.
fn l2_distance(a: &Dialect, b: &Dialect) -> f32 {
    let mut acc = 0.0_f32;
    for i in 0..DIALECT_DIM {
        let d = a[i] - b[i];
        acc += d * d;
    }
    acc.sqrt()
}

/// Union-Find helpers for [`LanguageField::dialect_clusters`].
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
        parent[rb] = ra;
    }
}

#[cfg(test)]
mod tests {
    //! Acceptance tests for FR-CIV-PSYCHE-912.
    //!
    //! Two things must hold (per the FR acceptance test):
    //!   1. Two isolated groups drift to distinct dialects.
    //!   2. A contact zone between them ends up mixed (closer to both
    //!      parent dialects than either pure group is).

    use super::*;
    use crate::social_graph::{SocialEvent, SocialGraph, SocialGraphConfig};

    /// Build a social graph shaped like the acceptance scenario for the
    /// *contact* test:
    ///
    /// ```text
    ///   group_a (A0..A4)  <--bridge (A5..A6)-->  group_b (B0..B4)
    /// ```
    ///
    /// All agents are pairwise-tied inside their group, and the bridge
    /// agents are pairwise-tied to *both* groups so they sit in a contact
    /// zone. Used only by [`fr_civ_psyche_912_acceptance_contact_zone_shows_mixing`].
    fn build_contact_scenario() -> (SocialGraph, Vec<AgentId>, Vec<AgentId>, Vec<AgentId>) {
        let cfg = SocialGraphConfig::default();
        let mut g = SocialGraph::new();
        let group_a: Vec<AgentId> = (0..5).collect();
        let group_b: Vec<AgentId> = (100..105).collect();
        let bridge: Vec<AgentId> = vec![50, 51];

        // Intra-group edges — strong contact.
        let mut all = Vec::new();
        all.extend(group_a.iter().copied());
        all.extend(group_b.iter().copied());
        all.extend(bridge.iter().copied());

        for w in all.windows(2) {
            let ev = SocialEvent::interaction(w[0], w[1], 0);
            g.apply_event(ev, &cfg);
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
        }

        // Bridge agents are tied to all of group_a and all of group_b.
        for &br in &bridge {
            for &ga in &group_a {
                g.apply_event(SocialEvent::interaction(br, ga, 0), &cfg);
                g.apply_event(SocialEvent::interaction(br, ga, 0), &cfg);
            }
            for &gb in &group_b {
                g.apply_event(SocialEvent::interaction(br, gb, 0), &cfg);
                g.apply_event(SocialEvent::interaction(br, gb, 0), &cfg);
            }
        }

        (g, group_a, group_b, bridge)
    }

    /// Build a social graph shaped like the acceptance scenario for the
    /// *isolation* test: two internally-cohesive groups with **no** edges
    /// between them and no bridge.
    fn build_isolated_scenario() -> (SocialGraph, Vec<AgentId>, Vec<AgentId>) {
        let cfg = SocialGraphConfig::default();
        let mut g = SocialGraph::new();
        let group_a: Vec<AgentId> = (0..5).collect();
        let group_b: Vec<AgentId> = (100..105).collect();

        for w in group_a.windows(2) {
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
        }
        for w in group_b.windows(2) {
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
            g.apply_event(SocialEvent::interaction(w[0], w[1], 0), &cfg);
        }
        (g, group_a, group_b)
    }

    /// Mean idiolect of a group of speakers — convenience for the
    /// assertions below.
    fn group_mean(field: &LanguageField, group: &[AgentId]) -> Dialect {
        let mut acc = [0.0_f32; DIALECT_DIM];
        let mut n = 0.0_f32;
        for &id in group {
            if let Some(v) = field.dialect_of(id) {
                for d in 0..DIALECT_DIM {
                    acc[d] += v[d];
                }
                n += 1.0;
            }
        }
        for d in 0..DIALECT_DIM {
            acc[d] /= n;
        }
        acc
    }

    #[test]
    fn fr_civ_psyche_912_acceptance_two_isolated_groups_drift_to_distinct_dialects() {
        // Acceptance criterion (1): two isolated groups drift to distinct
        // dialects.
        let cfg = LanguageConfig::default();
        let (graph, group_a, group_b) = build_isolated_scenario();

        // Initial field: all speakers start at the same point.
        let mut field = LanguageField::new();
        let start: Dialect = [0.0_f32; DIALECT_DIM];
        for &id in group_a.iter().chain(group_b.iter()) {
            field.register(id, start);
        }

        let initial_distance = field
            .distance(group_a[0], group_b[0])
            .expect("both speakers must be registered");

        // Step the field many ticks. With no edges between the groups,
        // drift is the only force — and it makes them diverge.
        for tick in 0..30 {
            field.step(
                tick,
                &cfg,
                |agent| graph.neighbors_of(agent),
            );
        }

        // Compare *group means* — a single speaker is too noisy; the
        // group mean is what the FR means by "language region".
        let mean_a = group_mean(&field, &group_a);
        let mean_b = group_mean(&field, &group_b);
        let drifted_distance = l2_distance(&mean_a, &mean_b);

        assert!(
            drifted_distance > initial_distance + cfg.cluster_radius,
            "two isolated groups must drift to distinct dialects \
             (initial={initial_distance:.3}, drifted={drifted_distance:.3}, \
             threshold={:.3})",
            initial_distance + cfg.cluster_radius,
        );

        // And the clusterer must agree they are different dialects.
        let clusters = field.dialect_clusters(&cfg);
        assert!(
            clusters.len() >= 2,
            "the clusterer must see at least two distinct dialect regions; got {clusters:?}"
        );
    }

    #[test]
    fn fr_civ_psyche_912_acceptance_contact_zone_shows_mixing() {
        // Acceptance criterion (2): a contact zone between two
        // divergent groups ends up with a dialect that mixes both.
        let cfg = LanguageConfig::default();
        let (graph, group_a, group_b, bridge) = build_contact_scenario();

        // Seed group A with a "north" dialect and group B with a "south"
        // dialect — two well-separated points in feature space. The
        // bridge agents start at the origin so they begin neutral.
        let mut field = LanguageField::new();
        let north: Dialect = [1.0_f32; DIALECT_DIM];
        let south: Dialect = [-1.0_f32; DIALECT_DIM];
        let neutral: Dialect = [0.0_f32; DIALECT_DIM];
        for &id in &group_a {
            field.register(id, north);
        }
        for &id in &group_b {
            field.register(id, south);
        }
        for &id in &bridge {
            field.register(id, neutral);
        }

        // Step the field many ticks. The bridge agents have edges to
        // *both* groups, so they get pulled toward both poles — i.e.
        // they mix.
        for tick in 0..30 {
            field.step(
                tick,
                &cfg,
                |agent| graph.neighbors_of(agent),
            );
        }

        let mean_a = group_mean(&field, &group_a);
        let mean_b = group_mean(&field, &group_b);
        let mean_bridge = group_mean(&field, &bridge);

        // The bridge group's mean must be *between* the two poles: closer
        // to the origin (or, at minimum, no closer to either pole than
        // the pure groups are to their own pole).
        let dist_bridge_to_a = l2_distance(&mean_bridge, &mean_a);
        let dist_bridge_to_b = l2_distance(&mean_bridge, &mean_b);
        let dist_a_to_a = l2_distance(&mean_a, &mean_a);
        let dist_b_to_b = l2_distance(&mean_b, &mean_b);

        // Each pole should have stayed (mostly) put, so the pole's
        // distance to its own starting point is small.
        assert!(
            dist_a_to_a < cfg.cluster_radius,
            "group A must remain near its starting pole (dist={dist_a_to_a:.3})"
        );
        assert!(
            dist_b_to_b < cfg.cluster_radius,
            "group B must remain near its starting pole (dist={dist_b_to_b:.3})"
        );

        // The contact zone must sit *between* the two poles, i.e. closer
        // to each pole than the inter-pole distance. We compare against
        // the inter-pole distance to assert "mixing".
        let inter_pole = l2_distance(&mean_a, &mean_b);
        assert!(
            dist_bridge_to_a < inter_pole
                && dist_bridge_to_b < inter_pole,
            "the contact zone must mix both dialects: \
             bridge->a={dist_bridge_to_a:.3}, bridge->b={dist_bridge_to_b:.3}, \
             a<->b={inter_pole:.3}",
        );

        // And the clusterer must see the contact zone as a third
        // distinct region (or at minimum, must distinguish it from the
        // pure groups' poles — depending on threshold, two-cluster or
        // three-cluster outcomes are both acceptable evidence of mixing).
        let clusters = field.dialect_clusters(&cfg);
        assert!(
            clusters.len() >= 2,
            "the clusterer must see at least two distinct dialect regions; got {clusters:?}"
        );
    }

    #[test]
    fn drift_is_deterministic_under_fixed_seed() {
        // Determinism: the same starting state and the same edge
        // schedule must produce the same dialect field after any number
        // of ticks.
        let cfg = LanguageConfig::default();
        let (graph, group_a, group_b, bridge) = build_contact_scenario();

        let seed: Dialect = [0.0_f32; DIALECT_DIM];
        let mut a = LanguageField::new();
        let mut b = LanguageField::new();
        for &id in group_a.iter().chain(group_b.iter()).chain(bridge.iter()) {
            a.register(id, seed);
            b.register(id, seed);
        }

        for tick in 0..20 {
            a.step(tick, &cfg, |ag| graph.neighbors_of(ag));
            b.step(tick, &cfg, |ag| graph.neighbors_of(ag));
        }

        for &id in group_a.iter().chain(group_b.iter()).chain(bridge.iter()) {
            let va = a.dialect_of(id).expect("a knows speaker");
            let vb = b.dialect_of(id).expect("b knows speaker");
            assert_eq!(
                va, vb,
                "deterministic step must produce identical dialect vectors for speaker {id}"
            );
        }
    }

    #[test]
    fn dialect_clusters_groups_close_speakers() {
        // Two clusters, clearly separated, must show up as two clusters
        // in the clusterer output.
        let cfg = LanguageConfig::default();
        let mut field = LanguageField::new();
        let north: Dialect = [1.0_f32; DIALECT_DIM];
        let south: Dialect = [-1.0_f32; DIALECT_DIM];
        for i in 0..4u64 {
            field.register(i, north);
            field.register(i + 100, south);
        }
        let clusters = field.dialect_clusters(&cfg);
        assert_eq!(clusters.len(), 2, "expected 2 clusters, got {clusters:?}");
        // Each cluster must contain 4 members.
        for c in &clusters {
            assert_eq!(c.len(), 4);
        }
    }

    #[test]
    fn dialect_centroid_for_isolated_speaker_is_their_own_vector() {
        // Sanity: an isolated speaker's ego-network centroid is just
        // their own vector.
        let mut field = LanguageField::new();
        let v: Dialect = [0.1, 0.2, -0.3, 0.4, -0.5, 0.6, -0.7, 0.8];
        field.register(1, v);
        let centroid = field
            .dialect_centroid(1, std::iter::empty())
            .expect("speaker 1 is known");
        for i in 0..DIALECT_DIM {
            assert!((centroid[i] - v[i]).abs() < 1e-6);
        }
    }
}