//! Per-mod guest scratch memory save/load (CIV-1000 §16.3 stub).

use serde::{Deserialize, Serialize};

/// Schema version for [`ModGuestStateSave`].
// The following 1 requirement tags were removed from MOD_GUEST_STATE_VERSION.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// See the namespace-collision note above.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-MOD-010: COLLIDING ID. modding-platform.md:36 = mod loading pipeline discover..bind; CIV-0700:2428 = action validation and conservation with ModActionRejected. The loader does parse + determinism-scan + signature-verify + register, with no charter-validate, dependency-resolve, ordering, law-merge, or bind stage; ModActionRejected has zero hits. Tagged on a u32 version const and two blob structs (guest_state.rs)
pub const MOD_GUEST_STATE_VERSION: u32 = 1;

/// One mod's opaque guest scratch bytes.
// The following 1 requirement tags were removed from ModGuestMemoryBlob.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// See the namespace-collision note above.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-MOD-010: COLLIDING ID. modding-platform.md:36 = mod loading pipeline discover..bind; CIV-0700:2428 = action validation and conservation with ModActionRejected. The loader does parse + determinism-scan + signature-verify + register, with no charter-validate, dependency-resolve, ordering, law-merge, or bind stage; ModActionRejected has zero hits. Tagged on a u32 version const and two blob structs (guest_state.rs)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModGuestMemoryBlob {
    /// Stable mod id (`manifest.meta.id`).
    pub mod_id: String,
    /// Host-managed scratch bytes (capped by the mod-host guest memory limit).
    pub bytes: Vec<u8>,
}

/// Serializable bundle of all mod guest memories for save/load.
// The following 1 requirement tags were removed from ModGuestStateSave.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// See the namespace-collision note above.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-MOD-010: COLLIDING ID. modding-platform.md:36 = mod loading pipeline discover..bind; CIV-0700:2428 = action validation and conservation with ModActionRejected. The loader does parse + determinism-scan + signature-verify + register, with no charter-validate, dependency-resolve, ordering, law-merge, or bind stage; ModActionRejected has zero hits. Tagged on a u32 version const and two blob structs (guest_state.rs)
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ModGuestStateSave {
    /// Format version for forward-compatible loaders.
    pub version: u32,
    /// Per-mod memory blobs.
    pub memories: Vec<ModGuestMemoryBlob>,
}

impl ModGuestStateSave {
    /// Empty save with the current schema version.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            version: MOD_GUEST_STATE_VERSION,
            memories: Vec::new(),
        }
    }

    /// JSON encode for CIV-1000 persistence stubs.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }

    /// JSON decode; rejects unknown future versions.
    pub fn from_json(json: &str) -> Result<Self, GuestStateError> {
        let save: Self = serde_json::from_str(json).map_err(GuestStateError::Json)?;
        if save.version > MOD_GUEST_STATE_VERSION {
            return Err(GuestStateError::UnsupportedVersion(save.version));
        }
        Ok(save)
    }
}

/// Errors loading guest state blobs.
// The following 1 requirement tags were removed from GuestStateError.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// See the namespace-collision note above.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-MOD-011: COLLIDING ID. modding-platform.md:37 = dependency + version + capability model with semver; CIV-0700:2436 = custom good type registration. ModDependencies.civlab_api is parsed but never compared and ModMeta.api_version is never checked against a host range, so no IncompatibleApiVersion can fire. GuestStateError is a 2-variant enum whose real content is a JSON parse failure (guest_state.rs, lib.rs)
#[derive(Debug, thiserror::Error)]
pub enum GuestStateError {
    /// JSON parse/serialize failure.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Save file targets a newer schema than this host.
    #[error("unsupported guest state version {0}")]
    UnsupportedVersion(u32),
}

// The following 1 requirement tags were removed from ModBrowserEntry.
// They are not discharged by this symbol. The tag named a requirement whose
// behavior lives elsewhere, or a requirement with no implementation at all, so
// leaving the tag here asserted coverage that this declaration does not provide.
// See the namespace-collision note above. FR-CIV-TACTICS-062 is NOT undefined: it is
// defined at docs/traceability/fr-3d-matrix.md:152 as "Mod catalog + runtime install",
// implemented by civ-watch's post_mods_install handler
// (crates/watch/src/mods_api.rs; behaviour tested at api_tests.rs:681). The tag sits
// on a stub struct in the wrong crate and is removed as a mis-binding, not as an
// undefined id.
//
// Removed, with the reason each cannot be discharged here:
//  [unbound] FR-CIV-TACTICS-062: MIS-BOUND, NOT UNDEFINED. Defined at docs/traceability/fr-3d-matrix.md:152 as "Mod catalog + runtime install", discharged by civ-watch's post_mods_install handler (crates/watch/src/mods_api.rs, tested at api_tests.rs:681). ModBrowserEntry is a 7-field stub row in the wrong crate and implements none of it (mod-host/src/guest_state.rs:66)
/// UI / RPC row describing a loaded mod (mod browser stub).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModBrowserEntry {
    /// Stable mod id.
    pub id: String,
    /// Display name from manifest.
    pub name: String,
    /// Semver string.
    pub version: String,
    /// `policy` | `economic` | `event` | `scenario`.
    pub mod_type: String,
    /// Whether `mod.wasm` was loaded.
    pub has_wasm: bool,
    /// Current guest scratch byte length.
    pub guest_memory_len: usize,
    /// Float opcode count from determinism scan (0 when WASM absent).
    pub float_instruction_count: u32,
    /// `action_emit` sites with float-derived args (CIV-0700 §3.5 data-flow trace).
    pub float_contamination_site_count: u32,
}
