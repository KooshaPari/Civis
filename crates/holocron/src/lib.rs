//! Holocron Keycap UI — verb registry substrate.
//!
//! The `holocron` crate is the **catalog layer** for godgame verbs. It does not
//! fire verbs itself — it enumerates, classifies, and queries them. Firing
//! stays in the substrate (MCP server, JSON-RPC, egui panels) so this crate
//! has zero runtime coupling to the simulation.
//!
//! # Module map
//!
//! - [`descriptor`] — `VerbDescriptor` schema (id, name, group, aliases, ...).
//! - [`group`] — `VerbGroup` enum (civic / economic / divine / debug).
//! - [`provenance`] — `Provenance` enum (Mcp / JsonRpc / Egui / Holocron).
//! - [`risk`] — `RiskTier` enum (Low / Medium / High / Catastrophic).
//! - [`rank`] — context-aware ranking algorithm.
//! - [`registry`] — `VerbRegistry` static catalog with lookup + fuzzy search.
//! - [`verbs`] — auto-enumerated catalog of all godgame verbs (52 verbs).
//! - [`bridge`] — bridge between the MCP server's tool list and the registry.
//! - [`inspect`] — FR-CIV-INSPECT-900 inspect-target resolution: pure-logic
//!   data + query backing layer that resolves a world pick
//!   (voxel/agent/settlement/structure/vehicle) to a `Summary`. No Bevy
//!   rendering; the renderer hands it `WorldPick` values and gets a
//!   `Summary` back.
//!
//! # Usage
//!
//! ```
//! use holocron::{VerbRegistry, default_registry};
//!
//! let registry = default_registry();
//! assert!(registry.len() >= 50);
//! assert!(registry.lookup("civ_bless_faction").is_some());
//! ```

#![forbid(unsafe_code)]
#![deny(rust_2018_idioms)]

pub mod bridge;
pub mod descriptor;
pub mod group;
pub mod inspect;
pub mod provenance;
pub mod rank;
pub mod registry;
pub mod risk;
pub mod verbs;

pub use descriptor::VerbDescriptor;
pub use group::VerbGroup;
pub use inspect::*;
pub use provenance::Provenance;
pub use rank::{rank_verbs, RankedVerb, SimContext};
pub use registry::{VerbRegistry, VerbSearchResult};
pub use risk::RiskTier;

/// Build the default registry by enumerating the static verb catalog.
///
/// This is the canonical entry point used by:
/// - the egui HolocronPanel (crates/hud/src/key_palette.rs),
/// - the CommandKOverlay (Phase 3), and
/// - the context-aware ranker (Phase 4).
pub fn default_registry() -> VerbRegistry {
    VerbRegistry::from_catalog(verbs::STATIC_CATALOG)
}
