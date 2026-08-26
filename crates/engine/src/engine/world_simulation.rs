//! World simulation tick orchestration: the main tick loop that coordinates
//! all subsystems. Extracted from `engine.rs` in decomposition pass 2
//! (Civis Engine Decomposition).

use crate::engine::Simulation;

/// Ordered phase identifiers executed once per [`Simulation::tick`].
///
/// CIV-0001 partial — engine-side deterministic transition. Server command
/// intake and client broadcast are outside this crate. Keep in sync with
/// the calls in [`Simulation::tick`].
pub(crate) const PHASE_ORDER: &[&str] = &[
    "production",
    "citizen_lifecycle",
    "military",
    "policy",
    "economy",
    "planet",
    "disasters",
    "diplomacy",
    "faction_decisions",
    "tactics",
    "voxel",
    "compact",
    "buildings",
    "life",
    "daily_path",
    "cluster",
    "research",
    "tech",
    "belief",
    "unrest",
    "cohesion",
    "social_mood",
    "economic_focus_pre",
    "stratification",
    "institutions",
    "economic_focus",
    "emergence",
    "tutorial",
    "psyche_behavior",
    "culture",
    "language",
    "sentience",
    "species",
    "diffusion",
    "audio",
    "victory_check",
];

// ---------------------------------------------------------------------------
// Simulation methods
// ---------------------------------------------------------------------------

impl Simulation {
    /// Advance the simulation `n` ticks. Convenience wrapper for tests +
    /// scenario runners so they can compress `n` ticks of `phase_*` work
    /// into a single call. Calls [`Self::tick`] `n` times.
    pub fn advance_ticks(&mut self, n: u32) {
        for _ in 0..n {
            self.tick();
        }
    }
}
