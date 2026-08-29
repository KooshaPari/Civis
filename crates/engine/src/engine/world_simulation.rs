//! World simulation tick orchestration: the main tick loop that coordinates
//! all subsystems. Extracted from `engine.rs` in decomposition pass 2
//! (Civis Engine Decomposition).

use crate::engine::Simulation;

/// Ordered phase identifiers executed once per [`Simulation::tick`].
///
/// CIV-0001 partial — engine-side deterministic transition. Server command
/// intake and client broadcast are outside this crate. Keep in sync with
/// the calls in [`Simulation::tick`].
///
/// FR-CIV-phasewire (audit): this list gained the six new top-level phase
/// entries (`religion`, `language`, `psyche`, `buildings`, `history`,
/// `writing`) plus the renamed legacy phases (`construction_sites`,
/// `language_drift`). The old entries `buildings` (now parcel allocation,
/// not layouts) and `language` (now the top-level module tick) were renamed
/// to free the names for the new wiring.
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
    // FR-CIV-phasewire: was "buildings", renamed to free the name for the
    // new top-level `phase_buildings` (which drives the building_layouts
    // module).
    "construction_sites",
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
    // FR-CIV-phasewire: was "language", renamed to free the name for the
    // new top-level `phase_language` (which drives language::tick_language_system).
    "language_drift",
    "sentience",
    "species",
    "diffusion",
    // Legacy aliases — preserved so old test fixtures remain valid.
    "writing_apply",
    "building_layouts",
    "history_archive",
    // FR-CIV-phasewire: six new top-level phases that wire the expanded
    // modules' exported `tick_*` fns into the engine tick loop.
    "religion",
    "language",
    "psyche",
    "buildings",
    "history",
    "writing",
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
