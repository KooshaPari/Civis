//! Per-mod guest scratch memory save/load (CIV-1000 §16.3 stub).

use serde::{Deserialize, Serialize};

/// Schema version for [`ModGuestStateSave`].
pub const MOD_GUEST_STATE_VERSION: u32 = 1;

/// One mod's opaque guest scratch bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModGuestMemoryBlob {
    /// Stable mod id (`manifest.meta.id`).
    pub mod_id: String,
    /// Host-managed scratch bytes (capped by the mod-host guest memory limit).
    pub bytes: Vec<u8>,
}

/// Serializable bundle of all mod guest memories for save/load.
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
#[derive(Debug, thiserror::Error)]
pub enum GuestStateError {
    /// JSON parse/serialize failure.
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// Save file targets a newer schema than this host.
    #[error("unsupported guest state version {0}")]
    UnsupportedVersion(u32),
}

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
