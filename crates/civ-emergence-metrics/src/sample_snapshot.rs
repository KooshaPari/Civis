/// Snapshot of the per-tick emergence state, populated from
/// [`civ_engine::emergence_metrics::EmergenceSample`] and
/// consumed by the L1 web dashboard, the L2 Bevy minimap, and the
/// `emergence.dashboard` JSON-RPC read.
///
/// All fields already exist on the engine's
/// `EmergenceSample`; the snapshot is a flat, transport-safe DTO
/// (no `Option`s except via the `criticality_*` band) so the
/// dashboard can read each tile as a single JSON number.
// The following 1 requirement tags were removed from EmergenceSampleSnapshot.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// A data carrier tagged with an id that no specification ever defined.
//
// Removed, with the reason each cannot be discharged here:
// [unbound] FR-CIV-EMERGENCE-011: The report's Spec file:line column for this row reads "(no spec text; requirement empty)" and no spec under docs/specs/ or agileplus-specs/ defines this id at all, so there is no requirement sentence to satisfy. `EmergenceSampleSnapshot` is a flat, transport-safe DTO mirroring the engine's EmergenceSample fields, so the tag is unjustified in either direction and no implementing symbol exists.
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
