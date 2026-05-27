//! civ-mod-host — manifest-only mod host stub (CIV-0700 Sprint D / v2).
//!
//! Loads and validates `manifest.toml` (or `mod.toml`) from a mod directory.
//! WASM sandboxing and capability enforcement are future work; v2 adds a
//! [`ModRegistry`] with a log-only policy-phase stub invoked from [`ModHost::tick`].

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod capability;
mod determinism;
mod float_data_flow;
mod guest_state;
mod signature;
mod wasm_guest;

use std::io::Read;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use signature::verify_wasm_signature;
use thiserror::Error;
use capability::ModCapabilitySet;
use wasm_guest::{
    invoke_economy_tick_with_capabilities, invoke_military_tick_with_capabilities,
    invoke_policy_tick_with_capabilities, MOD_WASM_NAME,
};

pub use capability::{
    format_mod_permission_violation_json, ModCapabilitySet as CapabilitySet, ModEnforcementCtx,
    ModStatus, WorldDomain, ACTION_SET_POLICY_PARAM, ACTION_SET_TAX_RATE, ERR_PERMISSION_DENIED,
};
pub use determinism::{
    scan_wasm_determinism, scan_wasm_determinism_report, DeterminismError, DeterminismScanReport,
};
pub use float_data_flow::{scan_float_action_emit_contamination, FloatContaminationSite};
pub use guest_state::{
    GuestStateError, ModBrowserEntry, ModGuestMemoryBlob, ModGuestStateSave,
    MOD_GUEST_STATE_VERSION,
};
pub use signature::{SignatureError, MOD_WASM_SIG_NAME};
pub use wasm_guest::{
    invoke_economy_tick, invoke_military_tick, invoke_policy_tick, HostState, WasmGuestError,
    HOST_CAPABILITY_API_VERSION, HOST_CAPABILITY_IMPORTS, HOST_GUEST_MEMORY_CAP, HOST_IMPORT_MODULE,
    MOD_WASM_NAME as MOD_WASM_FILE,
};

/// Supported mod kinds per CIV-0700 §4.1.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModType {
    /// Policy injection (Phase 3a).
    Policy,
    /// Economic ledger / production hooks.
    Economic,
    /// Scripted world events.
    Event,
    /// Scenario composition.
    Scenario,
}

/// `[mod]` table — required metadata.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModMeta {
    /// Stable id: `[a-z][a-z0-9-]{0,63}`.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Semver string.
    pub version: String,
    /// CivLab API major version (integer, stored as string in TOML).
    pub api_version: String,
    /// One of policy | economic | event | scenario.
    pub mod_type: ModType,
    /// Mod author name or organisation.
    pub author: String,
    /// Short description (max 256 chars).
    pub description: String,
    /// Optional URL to the mod homepage.
    #[serde(default)]
    pub homepage: Option<String>,
    /// SPDX license identifier (e.g. `MIT`).
    #[serde(default)]
    pub license: Option<String>,
    /// Hex-encoded Ed25519 public key for `mod.wasm` signature verification.
    #[serde(default)]
    pub author_pubkey_hex: Option<String>,
}

/// `[dependencies]` table.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModDependencies {
    /// Host API semver range (required).
    #[serde(rename = "civlab-api")]
    pub civlab_api: String,
    /// Optional peer-mod version constraints (`{ "other-mod" = "^1" }`).
    #[serde(default)]
    pub mods: Option<std::collections::BTreeMap<String, String>>,
}

/// `[permissions]` table — all fields optional in file; defaults are false.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct ModPermissions {
    /// Allow reading economy state.
    #[serde(default)]
    pub read_economy: bool,
    /// Allow reading climate state.
    #[serde(default)]
    pub read_climate: bool,
    /// Allow reading military state.
    #[serde(default)]
    pub read_military: bool,
    /// Allow reading diplomacy state.
    #[serde(default)]
    pub read_diplomacy: bool,
    /// Allow reading citizen state.
    #[serde(default)]
    pub read_citizens: bool,
    /// Allow writing policy state.
    #[serde(default)]
    pub write_policy: bool,
    /// Allow writing economy state.
    #[serde(default)]
    pub write_economy: bool,
    /// Allow emitting world events.
    #[serde(default)]
    pub write_events: bool,
    /// Allow modifying scenario configuration.
    #[serde(default)]
    pub write_scenario: bool,
    /// Allow fund-transfer operations.
    #[serde(default)]
    pub transfer_funds: bool,
}

/// `[runtime]` optional overrides.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModRuntime {
    /// Maximum WASM heap in megabytes (host cap: 64).
    pub memory_mb: Option<u32>,
    /// CPU budget in microseconds per tick (host cap: 50).
    pub cpu_us: Option<u32>,
}

/// Parsed manifest (TOML on disk).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ModManifest {
    /// Required `[mod]` metadata table.
    #[serde(rename = "mod")]
    pub meta: ModMeta,
    /// Required `[dependencies]` table.
    pub dependencies: ModDependencies,
    /// Optional `[permissions]` table; defaults all to false.
    #[serde(default)]
    pub permissions: ModPermissions,
    /// Optional `[runtime]` overrides.
    #[serde(default)]
    pub runtime: Option<ModRuntime>,
}

/// Errors while loading or validating a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ManifestError {
    /// Filesystem or IO failure.
    #[error("failed to read manifest at {path}: {message}")]
    Io {
        /// Path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// TOML parse failure.
    #[error("failed to parse manifest at {path}: {message}")]
    Parse {
        /// Path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// Post-parse validation failure (CIV-0700 §4.2 subset).
    #[error("invalid manifest at {path}: {message}")]
    Validation {
        /// Path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
    /// `.civmod` ZIP read or structure failure.
    #[error("failed to read civmod archive at {path}: {message}")]
    Archive {
        /// Archive path attempted.
        path: PathBuf,
        /// Error detail.
        message: String,
    },
}

/// Root manifest path inside a `.civmod` ZIP archive.
pub const CIVMOD_MANIFEST_NAME: &str = "manifest.toml";

/// `mod.loaded.v1` structured lifecycle record (FR-MOD-004).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModLoadedRecord {
    /// Stable mod id from manifest.
    pub mod_id: String,
    /// Display name.
    pub mod_name: String,
    /// Semver string.
    pub version: String,
    /// Simulation tick at load time.
    pub tick: u64,
}

/// `mod.unloaded.v1` structured lifecycle record (FR-MOD-004 partial).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModUnloadedRecord {
    /// Stable mod id from manifest.
    pub mod_id: String,
    /// Display name.
    pub mod_name: String,
    /// Simulation tick at unload time.
    pub tick: u64,
    /// Human-readable unload reason (e.g. `user_request`).
    pub reason: String,
}

/// Loaded mod entry kept by the host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedMod {
    /// Directory or `.civmod` archive path.
    pub root: PathBuf,
    /// Parsed manifest.
    pub manifest: ModManifest,
    /// Runtime capability set compiled from manifest permissions.
    pub capabilities: ModCapabilitySet,
    /// Optional `mod.wasm` bytes when present beside manifest or in archive.
    pub wasm_bytes: Option<Vec<u8>>,
    /// Float opcode count from the last determinism scan (0 when WASM absent).
    pub float_instruction_count: u32,
    /// `action_emit` contamination sites from data-flow scan (0 when WASM absent).
    pub float_contamination_site_count: u32,
}

/// Registry of loaded mod manifests (v2 stub — no WASM guests).
#[derive(Debug, Clone, Default)]
pub struct ModRegistry {
    mods: Vec<LoadedMod>,
}

impl ModRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loaded mods in registration order.
    #[must_use]
    pub fn mods(&self) -> &[LoadedMod] {
        &self.mods
    }

    /// Register a loaded mod entry.
    pub fn register(&mut self, entry: LoadedMod) {
        self.mods.push(entry);
    }

    /// Remove a loaded mod by stable id; returns the removed entry when found.
    pub fn remove_by_id(&mut self, mod_id: &str) -> Option<LoadedMod> {
        let index = self
            .mods
            .iter()
            .position(|entry| entry.manifest.meta.id == mod_id)?;
        Some(self.mods.remove(index))
    }

    /// Military-phase stub (P-W1): one log line per mod with `read_military`.
    ///
    /// Format: `mod:{id}:military_phase:tick={tick}` (WASM callbacks not invoked yet).
    #[must_use]
    pub fn on_military_phase(&self, tick: u64) -> Vec<String> {
        self.mods
            .iter()
            .filter(|m| m.manifest.permissions.read_military)
            .map(|m| format!("mod:{}:military_phase:tick={tick}", m.manifest.meta.id))
            .collect()
    }

    /// Policy-phase stub (Phase 3a): one log line per policy mod with `write_policy`.
    ///
    /// Format: `mod:{id}:policy_phase:tick={tick}` (WASM callbacks not invoked yet).
    #[must_use]
    pub fn on_policy_phase(&self, tick: u64) -> Vec<String> {
        self.mods
            .iter()
            .filter(|m| {
                m.manifest.meta.mod_type == ModType::Policy && m.manifest.permissions.write_policy
            })
            .map(|m| format!("mod:{}:policy_phase:tick={tick}", m.manifest.meta.id))
            .collect()
    }

    /// Economic-phase stub (Phase 3b).
    #[must_use]
    pub fn on_economy_phase(&self, tick: u64) -> Vec<String> {
        self.mods
            .iter()
            .filter(|m| m.manifest.meta.mod_type == ModType::Economic)
            .map(|m| format!("mod:{}:economy_phase:tick={tick}", m.manifest.meta.id))
            .collect()
    }
}

/// In-process mod host (manifest + WASM guest execution).
#[derive(Debug, Clone, Default)]
pub struct ModHost {
    registry: ModRegistry,
    loaded_records: Vec<ModLoadedRecord>,
    /// Source paths for hot reload (mod id → directory or `.civmod` archive).
    reload_roots: std::collections::BTreeMap<String, PathBuf>,
    /// Per-mod guest scratch memory persisted across phase ticks (FR-CIV-TACTICS-052).
    guest_memory_by_mod: std::collections::BTreeMap<String, Vec<u8>>,
    /// Permission violations accumulated during the current policy tick.
    enforcement_by_mod: std::collections::BTreeMap<String, ModEnforcementCtx>,
    /// Runtime mod lifecycle status (CIV-0700 §4.3).
    mod_status_by_id: std::collections::BTreeMap<String, ModStatus>,
}

impl ModHost {
    /// Empty host with no mods registered.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Manifest registry backing this host.
    #[must_use]
    pub fn registry(&self) -> &ModRegistry {
        &self.registry
    }

    /// Mods currently registered.
    #[must_use]
    pub fn mods(&self) -> &[LoadedMod] {
        self.registry.mods()
    }

    /// `mod.loaded.v1` records emitted on successful loads.
    #[must_use]
    pub fn loaded_records(&self) -> &[ModLoadedRecord] {
        &self.loaded_records
    }

    /// Runtime status for a loaded mod (`Active` when unknown).
    #[must_use]
    pub fn mod_status(&self, mod_id: &str) -> ModStatus {
        self.mod_status_by_id
            .get(mod_id)
            .copied()
            .unwrap_or(ModStatus::Active)
    }

    /// Permission violations recorded for the current policy tick.
    #[must_use]
    pub fn enforcement_violations(&self, mod_id: &str) -> u32 {
        self.enforcement_by_mod
            .get(mod_id)
            .map(|ctx| ctx.violations)
            .unwrap_or(0)
    }

    /// Snapshot of a mod's guest scratch memory (empty vec if the mod is unknown).
    #[must_use]
    pub fn guest_memory_snapshot(&self, mod_id: &str) -> Vec<u8> {
        self.guest_memory_by_mod
            .get(mod_id)
            .cloned()
            .unwrap_or_default()
    }

    /// Replace guest scratch memory for a loaded mod (CIV-1000 save/load stub).
    pub fn restore_guest_memory(&mut self, mod_id: &str, bytes: Vec<u8>) {
        let mut trimmed = bytes;
        if trimmed.len() > HOST_GUEST_MEMORY_CAP {
            trimmed.truncate(HOST_GUEST_MEMORY_CAP);
        }
        self.guest_memory_by_mod.insert(mod_id.to_owned(), trimmed);
    }

    /// Export all per-mod guest scratch bytes for save files (CIV-1000 §16.3).
    #[must_use]
    pub fn export_guest_state(&self) -> ModGuestStateSave {
        ModGuestStateSave {
            version: MOD_GUEST_STATE_VERSION,
            memories: self
                .guest_memory_by_mod
                .iter()
                .map(|(mod_id, bytes)| ModGuestMemoryBlob {
                    mod_id: mod_id.clone(),
                    bytes: bytes.clone(),
                })
                .collect(),
        }
    }

    /// Restore guest scratch bytes from a save bundle.
    pub fn import_guest_state(&mut self, save: &ModGuestStateSave) -> Result<(), GuestStateError> {
        if save.version > MOD_GUEST_STATE_VERSION {
            return Err(GuestStateError::UnsupportedVersion(save.version));
        }
        for blob in &save.memories {
            self.restore_guest_memory(&blob.mod_id, blob.bytes.clone());
        }
        Ok(())
    }

    /// Loaded mods for mod-browser UI / `sim.snapshot` wire (FR-CIV-TACTICS-054).
    #[must_use]
    pub fn browser_entries(&self) -> Vec<ModBrowserEntry> {
        self.registry
            .mods()
            .iter()
            .map(|entry| {
                let id = entry.manifest.meta.id.clone();
                ModBrowserEntry {
                    id: id.clone(),
                    name: entry.manifest.meta.name.clone(),
                    version: entry.manifest.meta.version.clone(),
                    mod_type: mod_type_label(entry.manifest.meta.mod_type).to_owned(),
                    has_wasm: entry.wasm_bytes.is_some(),
                    guest_memory_len: self.guest_memory_by_mod.get(&id).map(Vec::len).unwrap_or(0),
                    float_instruction_count: entry.float_instruction_count,
                    float_contamination_site_count: entry.float_contamination_site_count,
                }
            })
            .collect()
    }

    /// `mod.loaded.v1` lifecycle strings (replay / watch consumers).
    #[must_use]
    pub fn loaded_events(&self) -> Vec<String> {
        self.loaded_records
            .iter()
            .map(format_mod_loaded_event)
            .collect()
    }

    /// Load a mod directory or `.civmod` archive (extension selects loader).
    pub fn load_mod_path(&mut self, path: impl AsRef<Path>) -> Result<(), ManifestError> {
        let path = path.as_ref();
        if path.extension().and_then(|e| e.to_str()) == Some("civmod") {
            self.load_civmod_archive(path)
        } else {
            self.load_manifest_dir(path)
        }
    }

    /// Load `manifest.toml` from `mod_dir` and register it.
    pub fn load_manifest_dir(&mut self, mod_dir: impl AsRef<Path>) -> Result<(), ManifestError> {
        let mod_dir = mod_dir.as_ref();
        let manifest_path = mod_dir.join(CIVMOD_MANIFEST_NAME);
        let manifest = load_manifest(&manifest_path)?;
        let wasm_path = mod_dir.join(MOD_WASM_NAME);
        let wasm_bytes = read_optional_file(wasm_path);
        let sig_bytes = read_optional_file(mod_dir.join(MOD_WASM_SIG_NAME));
        if let Some(ref wasm) = wasm_bytes {
            enforce_wasm_determinism(mod_dir, wasm)?;
            enforce_wasm_signature(
                mod_dir,
                &manifest,
                Some(wasm.as_slice()),
                sig_bytes.as_deref(),
            )?;
        }
        let mod_id = manifest.meta.id.clone();
        let root = mod_dir.to_path_buf();
        self.remember_reload_root(&mod_id, root.clone());
        self.push_loaded(&manifest, 0);
        self.guest_memory_by_mod.entry(mod_id).or_default();
        self.registry
            .register(make_loaded_mod(root, manifest, wasm_bytes));
        Ok(())
    }

    /// Load `manifest.toml` from a `.civmod` ZIP archive and register it.
    pub fn load_civmod_archive(
        &mut self,
        archive_path: impl AsRef<Path>,
    ) -> Result<(), ManifestError> {
        let archive_path = archive_path.as_ref().to_path_buf();
        let (manifest, wasm_bytes) = read_civmod_archive(&archive_path)?;
        let mod_id = manifest.meta.id.clone();
        self.remember_reload_root(&mod_id, archive_path.clone());
        self.push_loaded(&manifest, 0);
        self.guest_memory_by_mod.entry(mod_id).or_default();
        self.registry
            .register(make_loaded_mod(archive_path, manifest, wasm_bytes));
        Ok(())
    }

    /// Unload then reload a mod from its remembered source path (hot reload).
    pub fn reload_mod(&mut self, mod_id: &str, tick: u64) -> Result<ModLoadedRecord, String> {
        let root = self
            .registry
            .mods()
            .iter()
            .find(|entry| entry.manifest.meta.id == mod_id)
            .map(|entry| entry.root.clone())
            .or_else(|| self.reload_roots.get(mod_id).cloned())
            .ok_or_else(|| format!("mod not loaded: {mod_id}"))?;

        self.unload_mod(mod_id, "hot_reload", tick)?;
        self.load_mod_path(&root).map_err(|err| err.to_string())?;

        let entry = self
            .registry
            .mods()
            .iter()
            .find(|entry| entry.manifest.meta.id == mod_id)
            .ok_or_else(|| format!("mod reload produced no registry entry: {mod_id}"))?;

        if let Some(record) = self.loaded_records.last_mut() {
            if record.mod_id == mod_id {
                record.tick = tick;
            }
        }

        Ok(ModLoadedRecord {
            mod_id: entry.manifest.meta.id.clone(),
            mod_name: entry.manifest.meta.name.clone(),
            version: entry.manifest.meta.version.clone(),
            tick,
        })
    }

    /// Military-phase hook (P-W1) — manifest stubs + WASM `civlab_military_tick` when loaded.
    #[must_use]
    pub fn military_tick(&mut self, sim_tick: u64) -> Vec<String> {
        let mut lines = self.registry.on_military_phase(sim_tick);
        let mods: Vec<_> = self.registry.mods().to_vec();
        for entry in mods {
            let Some(wasm) = entry.wasm_bytes.as_ref() else {
                continue;
            };
            if !entry.manifest.permissions.read_military {
                continue;
            }
            let mod_id = entry.manifest.meta.id.clone();
            if self.mod_status(&mod_id) == ModStatus::Suspended {
                continue;
            }
            let mem = self
                .guest_memory_by_mod
                .entry(mod_id.clone())
                .or_default();
            let enforcement = self
                .enforcement_by_mod
                .entry(mod_id.clone())
                .or_default();
            match invoke_military_tick_with_capabilities(
                wasm,
                sim_tick,
                mem,
                entry.capabilities.clone(),
                enforcement,
            ) {
                Ok(code) => lines.push(format!(
                    "mod:{mod_id}:wasm_military_tick:tick={sim_tick}:code={code}"
                )),
                Err(err) => lines.push(format_mod_error_event(
                    &mod_id,
                    sim_tick,
                    &err.to_string(),
                )),
            }
            if enforcement.suspended {
                self.mod_status_by_id
                    .insert(mod_id, ModStatus::Suspended);
            }
        }
        lines
    }

    /// Policy-phase hook — stubs + WASM `civlab_policy_tick` when loaded.
    #[must_use]
    pub fn tick(&mut self, sim_tick: u64) -> Vec<String> {
        self.reset_tick_enforcement();
        let mut lines = self.registry.on_policy_phase(sim_tick);
        let mods: Vec<_> = self.registry.mods().to_vec();
        for entry in mods {
            let Some(wasm) = entry.wasm_bytes.as_ref() else {
                continue;
            };
            if entry.manifest.meta.mod_type != ModType::Policy
                || !entry.manifest.permissions.write_policy
            {
                continue;
            }
            let mod_id = entry.manifest.meta.id.clone();
            if self.mod_status(&mod_id) == ModStatus::Suspended {
                continue;
            }
            let mem = self
                .guest_memory_by_mod
                .entry(mod_id.clone())
                .or_default();
            let enforcement = self
                .enforcement_by_mod
                .entry(mod_id.clone())
                .or_default();
            match invoke_policy_tick_with_capabilities(
                wasm,
                sim_tick,
                mem,
                entry.capabilities.clone(),
                enforcement,
            ) {
                Ok(code) => lines.push(format!(
                    "mod:{mod_id}:wasm_policy_tick:tick={sim_tick}:code={code}"
                )),
                Err(err) => lines.push(format_mod_error_event(
                    &mod_id,
                    sim_tick,
                    &err.to_string(),
                )),
            }
            if enforcement.suspended {
                self.mod_status_by_id
                    .insert(mod_id, ModStatus::Suspended);
            }
        }
        lines
    }

    /// Economic-phase hook — stubs + WASM `civlab_economy_tick` when loaded (FR-CIV-TACTICS-046).
    #[must_use]
    pub fn economy_tick(&mut self, sim_tick: u64) -> Vec<String> {
        let mut lines = self.registry.on_economy_phase(sim_tick);
        let mods: Vec<_> = self.registry.mods().to_vec();
        for entry in mods {
            let Some(wasm) = entry.wasm_bytes.as_ref() else {
                continue;
            };
            if entry.manifest.meta.mod_type != ModType::Economic
                || !entry.manifest.permissions.read_economy
            {
                continue;
            }
            let mod_id = entry.manifest.meta.id.clone();
            if self.mod_status(&mod_id) == ModStatus::Suspended {
                continue;
            }
            let mem = self
                .guest_memory_by_mod
                .entry(mod_id.clone())
                .or_default();
            let enforcement = self
                .enforcement_by_mod
                .entry(mod_id.clone())
                .or_default();
            match invoke_economy_tick_with_capabilities(
                wasm,
                sim_tick,
                mem,
                entry.capabilities.clone(),
                enforcement,
            ) {
                Ok(code) => lines.push(format!(
                    "mod:{mod_id}:wasm_economy_tick:tick={sim_tick}:code={code}"
                )),
                Err(err) => lines.push(format_mod_error_event(
                    &mod_id,
                    sim_tick,
                    &err.to_string(),
                )),
            }
            if enforcement.suspended {
                self.mod_status_by_id
                    .insert(mod_id, ModStatus::Suspended);
            }
        }
        lines
    }

    /// Unload a mod by stable id and emit a `mod.unloaded.v1` record.
    pub fn unload_mod(
        &mut self,
        mod_id: &str,
        reason: &str,
        tick: u64,
    ) -> Result<ModUnloadedRecord, String> {
        let removed = self
            .registry
            .remove_by_id(mod_id)
            .ok_or_else(|| format!("mod not loaded: {mod_id}"))?;
        self.guest_memory_by_mod.remove(mod_id);
        self.enforcement_by_mod.remove(mod_id);
        self.mod_status_by_id.remove(mod_id);
        Ok(ModUnloadedRecord {
            mod_id: mod_id.to_owned(),
            mod_name: removed.manifest.meta.name,
            tick,
            reason: reason.to_owned(),
        })
    }

    fn remember_reload_root(&mut self, mod_id: &str, root: PathBuf) {
        self.reload_roots.insert(mod_id.to_owned(), root);
    }

    fn push_loaded(&mut self, manifest: &ModManifest, tick: u64) {
        self.loaded_records.push(ModLoadedRecord {
            mod_id: manifest.meta.id.clone(),
            mod_name: manifest.meta.name.clone(),
            version: manifest.meta.version.clone(),
            tick,
        });
    }

    fn reset_tick_enforcement(&mut self) {
        for ctx in self.enforcement_by_mod.values_mut() {
            *ctx = ModEnforcementCtx::default();
        }
        for status in self.mod_status_by_id.values_mut() {
            if *status == ModStatus::Suspended {
                *status = ModStatus::Active;
            }
        }
    }
}

/// Format a `mod.loaded.v1` lifecycle event (EVENT_TAXONOMY / FR-MOD-004).
#[must_use]
pub fn format_mod_loaded_event(record: &ModLoadedRecord) -> String {
    format!(
        "mod.loaded.v1 mod_id={} mod_name={} version={} tick={}",
        record.mod_id, record.mod_name, record.version, record.tick
    )
}

/// Format `mod.loaded.v1` as JSON for the replay bus (FR-MOD-004 partial).
#[must_use]
pub fn format_mod_loaded_event_json(record: &ModLoadedRecord) -> String {
    serde_json::json!({
        "event": "mod.loaded.v1",
        "mod_id": record.mod_id,
        "mod_name": record.mod_name,
        "version": record.version,
        "tick": record.tick,
    })
    .to_string()
}

/// Format `mod.unloaded.v1` as JSON for the replay bus (FR-MOD-004 partial).
#[must_use]
pub fn format_mod_unloaded_event_json(record: &ModUnloadedRecord) -> String {
    serde_json::json!({
        "event": "mod.unloaded.v1",
        "mod_id": record.mod_id,
        "mod_name": record.mod_name,
        "tick": record.tick,
        "reason": record.reason,
    })
    .to_string()
}

/// Format `mod.error.v1` as JSON for the replay bus (FR-MOD-004 partial stub).
#[must_use]
pub fn format_mod_error_event_json(mod_id: &str, tick: u64, message: &str) -> String {
    serde_json::json!({
        "event": "mod.error.v1",
        "mod_id": mod_id,
        "tick": tick,
        "message": message,
    })
    .to_string()
}

/// Format `mod.error.v1` for host-side guest failures (log-line form).
#[must_use]
pub fn format_mod_error_event(mod_id: &str, tick: u64, message: &str) -> String {
    format!("mod.error.v1 mod_id={mod_id} tick={tick} message={message}")
}

fn read_optional_file(path: PathBuf) -> Option<Vec<u8>> {
    std::fs::read(path).ok()
}

/// Parse and validate manifest TOML from memory.
pub fn parse_manifest(contents: &str, path: &Path) -> Result<ModManifest, ManifestError> {
    let manifest: ModManifest = toml::from_str(contents).map_err(|e| ManifestError::Parse {
        path: path.to_path_buf(),
        message: e.to_string(),
    })?;
    validate_manifest(&manifest, path)?;
    Ok(manifest)
}

/// Read and validate `manifest.toml` at the root of a `.civmod` ZIP archive.
pub fn read_manifest_from_civmod(archive_path: &Path) -> Result<ModManifest, ManifestError> {
    read_civmod_archive(archive_path).map(|(manifest, _)| manifest)
}

/// Read manifest and optional `mod.wasm` from a `.civmod` ZIP archive.
pub fn read_civmod_archive(
    archive_path: &Path,
) -> Result<(ModManifest, Option<Vec<u8>>), ManifestError> {
    let file = std::fs::File::open(archive_path).map_err(|e| ManifestError::Archive {
        path: archive_path.to_path_buf(),
        message: e.to_string(),
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|e| ManifestError::Archive {
        path: archive_path.to_path_buf(),
        message: e.to_string(),
    })?;

    let mut manifest_contents: Option<String> = None;
    let mut wasm_bytes: Option<Vec<u8>> = None;
    let mut wasm_sig: Option<Vec<u8>> = None;
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i).map_err(|e| ManifestError::Archive {
            path: archive_path.to_path_buf(),
            message: e.to_string(),
        })?;
        let name = entry.name().to_string();
        if entry.is_dir() {
            continue;
        }
        if is_unsafe_zip_entry_name(&name) {
            return Err(ManifestError::Archive {
                path: archive_path.to_path_buf(),
                message: format!("unsafe zip entry path: {name}"),
            });
        }
        if name == CIVMOD_MANIFEST_NAME {
            let mut buf = String::new();
            entry
                .read_to_string(&mut buf)
                .map_err(|e| ManifestError::Archive {
                    path: archive_path.to_path_buf(),
                    message: e.to_string(),
                })?;
            manifest_contents = Some(buf);
        } else if name == MOD_WASM_NAME {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| ManifestError::Archive {
                    path: archive_path.to_path_buf(),
                    message: e.to_string(),
                })?;
            wasm_bytes = Some(buf);
        } else if name == MOD_WASM_SIG_NAME {
            let mut buf = Vec::new();
            entry
                .read_to_end(&mut buf)
                .map_err(|e| ManifestError::Archive {
                    path: archive_path.to_path_buf(),
                    message: e.to_string(),
                })?;
            wasm_sig = Some(buf);
        }
    }

    let contents = manifest_contents.ok_or_else(|| ManifestError::Archive {
        path: archive_path.to_path_buf(),
        message: format!("missing root {CIVMOD_MANIFEST_NAME} in archive"),
    })?;

    let manifest = parse_manifest(&contents, archive_path)?;
    if let Some(ref wasm) = wasm_bytes {
        enforce_wasm_determinism(archive_path, wasm)?;
    }
    enforce_wasm_signature(
        archive_path,
        &manifest,
        wasm_bytes.as_deref(),
        wasm_sig.as_deref(),
    )?;
    Ok((manifest, wasm_bytes))
}

fn make_loaded_mod(root: PathBuf, manifest: ModManifest, wasm_bytes: Option<Vec<u8>>) -> LoadedMod {
    let (float_instruction_count, float_contamination_site_count) = wasm_bytes
        .as_ref()
        .map(|wasm| {
            scan_wasm_determinism_report(wasm)
                .map(|report| {
                    (
                        report.float_instruction_count,
                        report.float_contamination_site_count,
                    )
                })
                .unwrap_or((0, 0))
        })
        .unwrap_or((0, 0));
    LoadedMod {
        root,
        capabilities: ModCapabilitySet::from_permissions(&manifest.permissions),
        manifest,
        wasm_bytes,
        float_instruction_count,
        float_contamination_site_count,
    }
}

fn mod_type_label(kind: ModType) -> &'static str {
    match kind {
        ModType::Policy => "policy",
        ModType::Economic => "economic",
        ModType::Event => "event",
        ModType::Scenario => "scenario",
    }
}

fn enforce_wasm_determinism(path: &Path, wasm: &[u8]) -> Result<(), ManifestError> {
    if cfg!(feature = "mod-dev") {
        return Ok(());
    }
    scan_wasm_determinism(wasm).map_err(|e| ManifestError::Validation {
        path: path.to_path_buf(),
        message: format!("determinism scan: {e}"),
    })
}

fn enforce_wasm_signature(
    path: &Path,
    manifest: &ModManifest,
    wasm: Option<&[u8]>,
    sig: Option<&[u8]>,
) -> Result<(), ManifestError> {
    let Some(wasm) = wasm else {
        return Ok(());
    };
    if cfg!(feature = "mod-dev") {
        return Ok(());
    }
    match (sig, manifest.meta.author_pubkey_hex.as_deref()) {
        (None, None) => Ok(()),
        (Some(sig_bytes), Some(pk)) => {
            verify_wasm_signature(wasm, sig_bytes, pk).map_err(|e| ManifestError::Validation {
                path: path.to_path_buf(),
                message: e.to_string(),
            })
        }
        (Some(_), None) => Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: "author_pubkey_hex required when mod.wasm.sig is present".to_owned(),
        }),
        (None, Some(_)) => Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: format!("missing {MOD_WASM_SIG_NAME} for signed mod"),
        }),
    }
}

fn is_unsafe_zip_entry_name(name: &str) -> bool {
    name.contains("..")
        || name.starts_with('/')
        || name.starts_with('\\')
        || name.contains('\\')
        || name.contains('/')
}

/// Load and validate a manifest file from `path`.
pub fn load_manifest(path: impl AsRef<Path>) -> Result<ModManifest, ManifestError> {
    let path = path.as_ref().to_path_buf();
    let contents = std::fs::read_to_string(&path).map_err(|e| ManifestError::Io {
        path: path.clone(),
        message: e.to_string(),
    })?;

    let manifest: ModManifest = toml::from_str(&contents).map_err(|e| ManifestError::Parse {
        path: path.clone(),
        message: e.to_string(),
    })?;

    validate_manifest(&manifest, &path)?;
    Ok(manifest)
}

fn validate_manifest(manifest: &ModManifest, path: &Path) -> Result<(), ManifestError> {
    let id = &manifest.meta.id;
    let valid_id = !id.is_empty()
        && id.as_bytes()[0].is_ascii_lowercase()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-');
    if !valid_id {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: format!("mod.id `{id}` must match [a-z][a-z0-9-]{{0,63}}"),
        });
    }

    if manifest.meta.description.len() > 256 {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: "mod.description must be at most 256 characters".into(),
        });
    }

    if manifest.meta.api_version.parse::<u32>().is_err() {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: format!(
                "mod.api_version `{}` must be a non-negative integer",
                manifest.meta.api_version
            ),
        });
    }

    if manifest.dependencies.civlab_api.trim().is_empty() {
        return Err(ManifestError::Validation {
            path: path.to_path_buf(),
            message: "dependencies.civlab-api must not be empty".into(),
        });
    }

    if let Some(runtime) = &manifest.runtime {
        if let Some(mb) = runtime.memory_mb {
            if mb > 64 {
                return Err(ManifestError::Validation {
                    path: path.to_path_buf(),
                    message: format!("runtime.memory_mb {mb} exceeds host maximum 64"),
                });
            }
        }
        if let Some(us) = runtime.cpu_us {
            if us > 50 {
                return Err(ManifestError::Validation {
                    path: path.to_path_buf(),
                    message: format!("runtime.cpu_us {us} exceeds host maximum 50"),
                });
            }
        }
    }

    Ok(())
}

/// Repo-relative path to `mods/example-policy` from this crate's manifest dir.
#[must_use]
pub fn example_policy_mod_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/example-policy")
}

/// Repo-relative path to `mods/example-economic` from this crate's manifest dir.
#[must_use]
pub fn example_economic_mod_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../mods/example-economic")
}

#[cfg(test)]
mod tests {
    use std::io::Write;

    use super::*;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    const MINIMAL_POLICY_MANIFEST: &str = r#"
[mod]
id = "zip-policy"
name = "ZIP Policy"
version = "0.1.0"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = true
"#;

    #[test]
    fn loads_example_policy_manifest() {
        let dir = example_policy_mod_dir();
        let manifest = load_manifest(dir.join("manifest.toml")).expect("example manifest");

        assert_eq!(manifest.meta.id, "example-policy");
        assert_eq!(manifest.meta.mod_type, ModType::Policy);
        assert!(manifest.permissions.read_economy);
        assert!(manifest.permissions.write_policy);
    }

    #[test]
    fn loads_example_economic_manifest() {
        let dir = example_economic_mod_dir();
        let manifest = load_manifest(dir.join("manifest.toml")).expect("example economic manifest");

        assert_eq!(manifest.meta.id, "example-economic");
        assert_eq!(manifest.meta.mod_type, ModType::Economic);
        assert!(manifest.permissions.read_economy);
        assert!(!manifest.permissions.write_policy);
    }

    #[test]
    fn mod_registry_military_phase_emits_for_read_military() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.toml");
        std::fs::write(
            &path,
            r#"
[mod]
id = "mil-observer"
name = "Military Observer"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
read_military = true
write_policy = false
"#,
        )
        .expect("write");

        let mut host = ModHost::new();
        host.load_manifest_dir(dir.path()).expect("load mod");

        let lines = host.registry().on_military_phase(7);
        assert_eq!(lines, vec!["mod:mil-observer:military_phase:tick=7"]);
        assert_eq!(
            host.military_tick(8),
            vec!["mod:mil-observer:military_phase:tick=8"]
        );
    }

    #[test]
    fn mod_registry_policy_phase_emits_log_lines() {
        let mut host = ModHost::new();
        host.load_manifest_dir(example_policy_mod_dir())
            .expect("load example mod");

        let lines = host.registry().on_policy_phase(42);
        assert_eq!(lines, vec!["mod:example-policy:policy_phase:tick=42"]);

        let via_tick = host.tick(99);
        assert!(via_tick
            .iter()
            .any(|l| l == "mod:example-policy:policy_phase:tick=99"));
    }

    #[test]
    fn wasm_policy_tick_invokes_civlab_export() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 42)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        assert_eq!(invoke_policy_tick(&wasm, 7, &mut mem).expect("invoke"), 42);
    }

    #[test]
    fn wasm_economy_tick_invokes_civlab_export() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_economy_tick") (param i64) (result i32)
                local.get 0
                i32.wrap_i64)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        assert_eq!(invoke_economy_tick(&wasm, 9, &mut mem).expect("invoke"), 9);
    }

    #[test]
    fn mod_guest_state_save_round_trips_json() {
        let mut host = ModHost::new();
        host.restore_guest_memory("demo", vec![9, 8, 7]);
        let json = host.export_guest_state().to_json().expect("json");
        let mut other = ModHost::new();
        let save = ModGuestStateSave::from_json(&json).expect("parse");
        other.import_guest_state(&save).expect("import");
        assert_eq!(other.guest_memory_snapshot("demo"), vec![9, 8, 7]);
    }

    #[test]
    fn mod_host_guest_memory_snapshot_roundtrip() {
        let mut host = ModHost::new();
        host.restore_guest_memory("demo", vec![1, 2, 3]);
        assert_eq!(host.guest_memory_snapshot("demo"), vec![1, 2, 3]);
        assert!(host.guest_memory_snapshot("missing").is_empty());
    }

    #[test]
    fn capability_imports_list_is_complete() {
        assert!(HOST_CAPABILITY_IMPORTS.contains(&"sim_tick"));
        assert!(HOST_CAPABILITY_IMPORTS.contains(&"memory_read"));
        assert!(HOST_CAPABILITY_IMPORTS.contains(&"world_read"));
        assert!(HOST_CAPABILITY_IMPORTS.contains(&"action_emit"));
    }

    #[test]
    fn loaded_mod_carries_capability_set_from_manifest() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("manifest.toml"),
            r#"
[mod]
id = "cap-demo"
name = "Cap Demo"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
read_economy = true
write_policy = true
"#,
        )
        .expect("manifest");
        let mut host = ModHost::new();
        host.load_manifest_dir(dir.path()).expect("load");
        let entry = &host.mods()[0];
        assert!(entry.capabilities.can_read_domain(WorldDomain::Economy));
        assert!(entry.capabilities.can_emit_action(ACTION_SET_TAX_RATE));
        assert!(!entry.capabilities.can_read_domain(WorldDomain::Military));
    }

    #[test]
    fn mod_host_economy_tick_persists_guest_memory() {
        const WAT: &str = r#"
            (module
              (import "civlab" "memory_read" (func $read (param i32) (result i32)))
              (import "civlab" "memory_write" (func $write (param i32 i32)))
              (func (export "civlab_economy_tick") (param i64) (result i32)
                (i32.const 0)
                (call $read)
                (if (result i32)
                  (i32.eqz)
                  (then
                    (i32.const 0)
                    (i32.const 55)
                    (call $write)
                    (i32.const 55))
                  (else
                    (i32.const 0)
                    (call $read))))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("manifest.toml"),
            r#"
[mod]
id = "mem-econ"
name = "Mem Econ"
version = "0.0.1"
api_version = "1"
mod_type = "economic"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
read_economy = true
"#,
        )
        .expect("manifest");
        std::fs::write(dir.path().join(MOD_WASM_NAME), wasm).expect("wasm");

        let mut host = ModHost::new();
        host.load_manifest_dir(dir.path()).expect("load");
        let _ = host.economy_tick(1);
        assert_eq!(
            host.guest_memory_snapshot("mem-econ").first().copied(),
            Some(55)
        );
        let _ = host.economy_tick(2);
        assert_eq!(
            host.guest_memory_snapshot("mem-econ").first().copied(),
            Some(55)
        );
    }

    #[test]
    fn wasm_guest_reads_capability_host_import() {
        const WAT: &str = r#"
            (module
              (import "civlab" "capability_api_version" (func $ver (result i32)))
              (func (export "civlab_policy_tick") (param i64) (result i32)
                (call $ver))
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        assert_eq!(
            invoke_policy_tick(&wasm, 0, &mut mem).expect("invoke"),
            wasm_guest::HOST_CAPABILITY_API_VERSION
        );
    }

    #[test]
    fn wasm_military_tick_invokes_civlab_export() {
        const WAT: &str = r#"
            (module
              (func (export "civlab_military_tick") (param i64) (result i32)
                local.get 0
                i32.wrap_i64)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let mut mem = Vec::new();
        assert_eq!(
            invoke_military_tick(&wasm, 11, &mut mem).expect("invoke"),
            11
        );
    }

    /// When `just civis-3d-mod-wasm` has been run, repo example-policy loads WASM on tick.
    #[test]
    fn example_economic_dir_wasm_ticks_when_built() {
        let dir = example_economic_mod_dir();
        let wasm_path = dir.join(MOD_WASM_NAME);
        if !wasm_path.is_file() {
            return;
        }
        let mut host = ModHost::new();
        host.load_manifest_dir(&dir).expect("example-economic dir");
        let lines = host.economy_tick(3);
        assert!(
            lines.iter().any(|l| l.contains("wasm_economy_tick")),
            "expected wasm_economy_tick after building mod.wasm: {lines:?}"
        );
    }

    #[test]
    fn example_policy_dir_wasm_ticks_when_built() {
        let dir = example_policy_mod_dir();
        let wasm_path = dir.join(MOD_WASM_NAME);
        if !wasm_path.is_file() {
            return;
        }
        let mut host = ModHost::new();
        host.load_manifest_dir(&dir).expect("example-policy dir");
        let lines = host.tick(1);
        assert!(
            lines.iter().any(|l| l.contains("wasm_policy_tick")),
            "expected wasm_policy_tick after building mod.wasm: {lines:?}"
        );
    }

    #[test]
    fn example_policy_civmod_loads_when_packaged() {
        let civmod = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../mods/example-policy/example-policy.civmod");
        if !civmod.is_file() {
            eprintln!(
                "skip example_policy_civmod_loads_when_packaged: run `just civis-3d-mod-package`"
            );
            return;
        }
        let mut host = ModHost::new();
        host.load_mod_path(&civmod)
            .expect("packaged example-policy.civmod");
        assert_eq!(host.mods().len(), 1);
        assert_eq!(host.mods()[0].manifest.meta.id, "example-policy");
    }

    #[test]
    fn load_mod_path_accepts_civmod_extension() {
        let dir = tempfile::tempdir().expect("tempdir");
        let civmod = dir.path().join("policy.civmod");
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 3)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let file = std::fs::File::create(&civmod).expect("create");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file(CIVMOD_MANIFEST_NAME, options)
            .expect("manifest");
        zip.write_all(MINIMAL_POLICY_MANIFEST.as_bytes())
            .expect("write");
        zip.start_file(MOD_WASM_NAME, options).expect("wasm");
        zip.write_all(&wasm).expect("write wasm");
        zip.finish().expect("finish");

        let mut host = ModHost::new();
        host.load_mod_path(&civmod).expect("load_mod_path");
        assert_eq!(host.mods().len(), 1);
    }

    #[test]
    fn load_civmod_with_wasm_invokes_on_tick() {
        let dir = tempfile::tempdir().expect("tempdir");
        let civmod = dir.path().join("policy.civmod");
        const WAT: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 7)
            )
        "#;
        let wasm = wat::parse_str(WAT).expect("wat");
        let file = std::fs::File::create(&civmod).expect("create");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file(CIVMOD_MANIFEST_NAME, options)
            .expect("manifest");
        zip.write_all(MINIMAL_POLICY_MANIFEST.as_bytes())
            .expect("write");
        zip.start_file(MOD_WASM_NAME, options).expect("wasm");
        zip.write_all(&wasm).expect("write wasm");
        zip.finish().expect("finish");

        let mut host = ModHost::new();
        host.load_civmod_archive(&civmod).expect("load");
        let lines = host.tick(1);
        assert!(lines
            .iter()
            .any(|l| l.contains("wasm_policy_tick") && l.contains("code=7")));
    }

    #[test]
    fn policy_phase_skips_mods_without_write_policy() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.toml");
        std::fs::write(
            &path,
            r#"
[mod]
id = "read-only-policy"
name = "x"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = false
"#,
        )
        .expect("write");

        let manifest = load_manifest(&path).expect("manifest");
        let mut registry = ModRegistry::new();
        registry.register(make_loaded_mod(dir.path().to_path_buf(), manifest, None));

        assert!(registry.on_policy_phase(1).is_empty());
    }

    #[test]
    fn signed_mod_rejects_tampered_wasm() {
        use ed25519_dalek::Signer;
        use rand::rngs::OsRng;

        const WAT_SIGNED: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 1)
            )
        "#;
        const WAT_TAMPERED: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 2)
            )
        "#;
        let signed_wasm = wat::parse_str(WAT_SIGNED).expect("signed wat");
        let tampered_wasm = wat::parse_str(WAT_TAMPERED).expect("tampered wat");
        let signing_key = ed25519_dalek::SigningKey::generate(&mut OsRng);
        let signature = signing_key.sign(&signed_wasm);
        let pk_hex: String = signing_key
            .verifying_key()
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();

        let dir = tempfile::tempdir().expect("tempdir");
        let civmod = dir.path().join("signed-policy.civmod");
        let manifest = format!(
            r#"
[mod]
id = "signed-policy"
name = "Signed"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"
author_pubkey_hex = "{pk_hex}"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = true
"#
        );

        let file = std::fs::File::create(&civmod).expect("create");
        let mut zip = ZipWriter::new(file);
        let options = SimpleFileOptions::default();
        zip.start_file(CIVMOD_MANIFEST_NAME, options)
            .expect("manifest");
        zip.write_all(manifest.as_bytes()).expect("write manifest");
        zip.start_file(MOD_WASM_NAME, options).expect("wasm");
        zip.write_all(&tampered_wasm).expect("write wasm");
        zip.start_file(MOD_WASM_SIG_NAME, options).expect("sig");
        zip.write_all(signature.to_bytes().as_slice())
            .expect("write sig");
        zip.finish().expect("finish");

        let err = read_civmod_archive(&civmod).expect_err("tampered wasm must fail verify");
        match err {
            ManifestError::Validation { message, .. } => {
                assert!(
                    message.contains("signature verification failed"),
                    "unexpected message: {message}"
                );
            }
            other => panic!("expected validation error, got {other:?}"),
        }
    }

    #[test]
    fn mod_loaded_event_json_has_required_keys() {
        let record = ModLoadedRecord {
            mod_id: "example-policy".to_owned(),
            mod_name: "Example Policy".to_owned(),
            version: "0.1.0".to_owned(),
            tick: 42,
        };
        let json = format_mod_loaded_event_json(&record);
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["event"], "mod.loaded.v1");
        assert_eq!(v["mod_id"], "example-policy");
        assert_eq!(v["mod_name"], "Example Policy");
        assert_eq!(v["version"], "0.1.0");
        assert_eq!(v["tick"], 42);
    }

    #[test]
    fn mod_error_event_json_has_required_keys() {
        let json = format_mod_error_event_json("demo-mod", 7, "guest trap");
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["event"], "mod.error.v1");
        assert_eq!(v["mod_id"], "demo-mod");
        assert_eq!(v["tick"], 7);
        assert_eq!(v["message"], "guest trap");
    }

    #[test]
    fn rejects_invalid_mod_id() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("manifest.toml");
        std::fs::write(
            &path,
            r#"
[mod]
id = "INVALID"
name = "x"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"
"#,
        )
        .expect("write");

        let err = load_manifest(&path).expect_err("bad id");
        assert!(matches!(err, ManifestError::Validation { .. }));
    }

    #[test]
    fn mod_unloaded_event_json_has_required_keys() {
        let record = ModUnloadedRecord {
            mod_id: "example-policy".to_owned(),
            mod_name: "Example Policy".to_owned(),
            tick: 99,
            reason: "user_request".to_owned(),
        };
        let json = format_mod_unloaded_event_json(&record);
        let v: serde_json::Value = serde_json::from_str(&json).expect("parse json");
        assert_eq!(v["event"], "mod.unloaded.v1");
        assert_eq!(v["mod_id"], "example-policy");
        assert_eq!(v["reason"], "user_request");
    }

    #[test]
    fn unload_mod_removes_from_registry() {
        let mut host = ModHost::new();
        host.load_manifest_dir(example_policy_mod_dir())
            .expect("load example mod");
        assert_eq!(host.mods().len(), 1);
        let record = host
            .unload_mod("example-policy", "user_request", 5)
            .expect("unload");
        assert_eq!(record.mod_id, "example-policy");
        assert!(host.mods().is_empty());
        assert!(host.guest_memory_snapshot("example-policy").is_empty());
    }

    #[test]
    fn reload_mod_rereads_wasm_from_root() {
        const WAT_V1: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 1)
            )
        "#;
        const WAT_V2: &str = r#"
            (module
              (func (export "civlab_policy_tick") (result i32)
                i32.const 2)
            )
        "#;
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            dir.path().join("manifest.toml"),
            r#"
[mod]
id = "reload-demo"
name = "Reload Demo"
version = "0.0.1"
api_version = "1"
mod_type = "policy"
author = "t"
description = "d"

[dependencies]
civlab-api = ">=1.0.0, <2.0.0"

[permissions]
write_policy = true
"#,
        )
        .expect("manifest");
        std::fs::write(dir.path().join(MOD_WASM_NAME), wat::parse_str(WAT_V1).expect("wat"))
            .expect("wasm v1");

        let mut host = ModHost::new();
        host.load_manifest_dir(dir.path()).expect("load");
        let lines_v1 = host.tick(1);
        assert!(lines_v1.iter().any(|line| line.contains("code=1")));

        std::fs::write(dir.path().join(MOD_WASM_NAME), wat::parse_str(WAT_V2).expect("wat"))
            .expect("wasm v2");
        let record = host.reload_mod("reload-demo", 9).expect("reload");
        assert_eq!(record.mod_id, "reload-demo");
        assert_eq!(record.tick, 9);

        let lines_v2 = host.tick(2);
        assert!(lines_v2.iter().any(|line| line.contains("code=2")));
    }
}
