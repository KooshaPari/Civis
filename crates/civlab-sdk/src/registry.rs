use crate::building::{BuildingCatalog, RecipeCatalog};
use crate::manifest::ModManifest;
use crate::material::MaterialCatalog;

/// Registry that groups all mod-facing catalogs.
#[derive(Debug, Default)]
pub struct ModRegistry {
    /// Material catalog.
    pub materials: MaterialCatalog,
    /// Building catalog.
    pub buildings: BuildingCatalog,
    /// Recipe catalog.
    pub recipes: RecipeCatalog,
    manifests: Vec<ModManifest>,
}

impl ModRegistry {
    /// Create an empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Loaded manifests in registration order.
    #[must_use]
    pub fn manifests(&self) -> &[ModManifest] {
        &self.manifests
    }

    /// Add a manifest.
    pub fn register_manifest(&mut self, manifest: ModManifest) {
        self.manifests.push(manifest);
    }
}
