use civ_voxel::material::{MaterialDef, Phase};
use civ_voxel::MaterialId;
use serde::{Deserialize, Serialize};

/// Mod-authored material definition, anchored to the Civis voxel taxonomy.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialSpec {
    /// Human-readable name.
    pub name: String,
    /// Broad phase classification.
    pub phase: Phase,
    /// Relative density used by simulation and worldgen.
    pub density: u16,
    /// Spread/flow speed.
    pub flow_rate: u8,
    /// Effective viscosity.
    pub viscosity: u16,
    /// Angle of repose for powders.
    pub angle_of_repose: Option<u8>,
    /// RGBA render hint.
    pub color: [u8; 4],
}

/// A custom material plus an optional base material id to inherit from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomMaterial {
    /// Optional base material to use as a template.
    pub base_id: Option<MaterialId>,
    /// The material spec the mod provides.
    pub spec: MaterialSpec,
}

impl CustomMaterial {
    /// Convert into a `civ_voxel::MaterialDef` using the supplied stable id.
    #[must_use]
    pub fn into_material_def(self, id: MaterialId) -> MaterialDef {
        let spec = self.spec;
        MaterialDef {
            id,
            name: Box::leak(spec.name.into_boxed_str()),
            phase: spec.phase,
            density: spec.density,
            flow_rate: spec.flow_rate,
            viscosity: spec.viscosity,
            angle_of_repose: spec.angle_of_repose,
            color: spec.color,
            temperature: 20,
            flammability: 0,
        }
    }
}

/// Registration payload stored by hosts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaterialRegistration {
    /// Stable mod-local identifier.
    pub id: String,
    /// Material definition.
    pub material: CustomMaterial,
}

/// Registry of mod materials.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MaterialCatalog {
    materials: Vec<MaterialRegistration>,
}

impl MaterialCatalog {
    /// Register a new material.
    pub fn register(&mut self, material: CustomMaterial) -> MaterialId {
        let id = MaterialId((self.materials.len() as u16).saturating_add(1_000));
        self.materials.push(MaterialRegistration {
            id: material.spec.name.clone(),
            material,
        });
        id
    }

    /// Get a material by name.
    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<&MaterialRegistration> {
        self.materials.iter().find(|entry| entry.id == name)
    }

    /// All registered materials.
    #[must_use]
    pub fn materials(&self) -> &[MaterialRegistration] {
        &self.materials
    }
}

/// Trait implemented by mods that contribute materials.
pub trait MaterialRegistrar {
    /// Register custom materials into the given catalog.
    fn register_materials(&self, catalog: &mut MaterialCatalog);
}
