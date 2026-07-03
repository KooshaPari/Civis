/// Snapshot of the per-tick emergence state, populated from
/// [`civ_engine::emergence_metrics::EmergenceSample`] and
/// consumed by the L1 web dashboard, the L2 Bevy minimap, and the
/// `emergence.dashboard` JSON-RPC read.
///
/// All fields already exist on the engine's
/// `EmergenceSample`; the snapshot is a flat, transport-safe DTO
/// (no `Option`s except via the `criticality_*` band) so the
/// dashboard can read each tile as a single JSON number.
pub struct EmergenceSampleSnapshot {
    /// Total live civilian count.
    pub agent_count: u32,
    /// Live faction count.
    pub faction_count: u32,
    /// Normalised Shannon entropy over the material histogram.
    pub resource_entropy: f32,
    /// Connected structure count on the sampled chunk.
    pub structure_count: u32,
    /// Per-capita rate of novel world configurations in the
    /// current `W_nov` window (charter §3.4).
    pub novelty_rate: f32,
    /// Estimated mutual information between the material and
    /// faction layers, normalised to `[0, 1]`.
    pub coupling_strength: f32,
    /// Power-law slope `α` on the cluster-size distribution
    /// (charter §3.4). `0.0` sentinel when fewer than 3 clusters
    /// are present or the fit is non-finite.
    pub power_law_alpha: f32,
    /// Rolling-mean branching ratio `σ̄_W` (charter §3.6).
    pub branching_sigma: f32,
    /// Engine tick the snapshot was captured at.
    pub tick: u64,
}
