//! Experimental staged adapter around the semantic state manifest.
//!
//! This deliberately does not replace CivSaveBundle yet. It proves ordering:
//! semantic compatibility is validated before semantic state is applied, and
//! orphan guest-memory namespaces are surfaced rather than silently accepted.

use crate::semantic_state_manifest::SemanticStateManifest;
use crate::Simulation;
use std::fs;
use std::path::{Path, PathBuf};

pub const SEMANTIC_STATE_FILE: &str = "semantic-state.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticLoadDisposition {
    Compatible,
    MissingRequiredSemanticState,
    UnsupportedSemanticSchema,
    OrphanGuestMemory(Vec<String>),
}

pub struct SemanticSaveAdapter;

impl SemanticSaveAdapter {
    pub fn write_component(dir: impl AsRef<Path>, sim: &Simulation) -> Result<PathBuf, String> {
        let dir = dir.as_ref();
        fs::create_dir_all(dir).map_err(|e| format!("create semantic save dir: {e}"))?;
        let manifest = SemanticStateManifest::capture(sim)?;
        manifest.validate_supported()?;
        let path = dir.join(SEMANTIC_STATE_FILE);
        let bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| format!("serialize semantic state: {e}"))?;
        fs::write(&path, bytes).map_err(|e| format!("write semantic state: {e}"))?;
        Ok(path)
    }

    pub fn read_component(dir: impl AsRef<Path>) -> Result<SemanticStateManifest, SemanticLoadDisposition> {
        let path = dir.as_ref().join(SEMANTIC_STATE_FILE);
        if !path.is_file() {
            return Err(SemanticLoadDisposition::MissingRequiredSemanticState);
        }
        let bytes = fs::read(&path)
            .map_err(|_| SemanticLoadDisposition::MissingRequiredSemanticState)?;
        let manifest: SemanticStateManifest = serde_json::from_slice(&bytes)
            .map_err(|_| SemanticLoadDisposition::UnsupportedSemanticSchema)?;
        manifest
            .validate_supported()
            .map_err(|_| SemanticLoadDisposition::UnsupportedSemanticSchema)?;
        Ok(manifest)
    }

    pub fn validate_against_resolved_mods(
        manifest: &SemanticStateManifest,
        sim_with_resolved_mods: &Simulation,
    ) -> Result<(), SemanticLoadDisposition> {
        let orphan = manifest.orphan_guest_memory_ids(sim_with_resolved_mods);
        if orphan.is_empty() {
            Ok(())
        } else {
            Err(SemanticLoadDisposition::OrphanGuestMemory(orphan))
        }
    }

    pub fn apply_after_compatibility(
        manifest: &SemanticStateManifest,
        sim: &mut Simulation,
    ) -> Result<(), String> {
        manifest.apply_to(sim)
    }
}


/// Opt-in bridge for vNext experiments. This is intentionally separate from
/// CivSaveBundle::save_dir/load_dir so the current production format remains a
/// comparison baseline until migration/fault gates close.
pub struct SemanticBundleBridge;

impl SemanticBundleBridge {
    pub fn save_opt_in(
        dir: impl AsRef<Path>,
        sim: &Simulation,
    ) -> Result<(), String> {
        let dir = dir.as_ref();
        crate::save_bundle::CivSaveBundle::save_dir(dir, sim)?;
        SemanticSaveAdapter::write_component(dir, sim)?;
        Ok(())
    }

    /// Loads the legacy/current bundle, validates the required semantic
    /// component, checks compatibility against the caller-provided resolved
    /// mod environment, then applies semantic state. The caller must construct
    /// resolved_mod_environment from the accepted scenario/profile before
    /// invoking this bridge.
    pub fn load_opt_in(
        dir: impl AsRef<Path>,
        resolved_mod_environment: &Simulation,
    ) -> Result<Simulation, String> {
        let dir = dir.as_ref();
        let manifest = SemanticSaveAdapter::read_component(dir)
            .map_err(|e| format!("semantic component rejected: {e:?}"))?;
        SemanticSaveAdapter::validate_against_resolved_mods(&manifest, resolved_mod_environment)
            .map_err(|e| format!("semantic compatibility rejected: {e:?}"))?;

        let mut loaded = crate::save_bundle::CivSaveBundle::load_dir(dir)?;
        SemanticSaveAdapter::apply_after_compatibility(&manifest, &mut loaded)?;
        Ok(loaded)
    }
}
