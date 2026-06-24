//! civlab-sdk - modder-facing, engine-agnostic SDK for Civis mods.
//!
//! The crate exposes:
//! - material registration backed by `civ-voxel` taxonomy types
//! - building and recipe registration traits
//! - simulation event hooks for births, deaths, and tech changes
//! - manifest loading from a `mods/` folder in JSON or RON
//!
//! The API is intentionally hexagonal: consumers implement the traits in this
//! crate, while hosts provide adapters that call into engine internals.

#![forbid(unsafe_code)]
#![allow(missing_docs)]

pub mod building;
pub mod events;
pub mod manifest;
pub mod material;
pub mod registry;

pub use building::{
    BuildingBlueprint, BuildingCatalog, BuildingKind, BuildingRegistrar, BuildingRegistration,
    RecipeCatalog, RecipeDefinition, RecipeRegistrar, RecipeRegistration,
};
pub use events::{BirthEvent, DeathEvent, SimulationEvent, SimulationEventHook, TechEvent};
pub use manifest::{
    load_manifest_file, load_manifests_from_dir, ManifestError, ModManifest, ModManifestFormat,
    ModMetadata,
};
pub use material::{
    CustomMaterial, MaterialCatalog, MaterialRegistrar, MaterialRegistration, MaterialSpec,
};
pub use registry::ModRegistry;

pub use civ_agents::{Civilian, LodTier, Position3d, Tools, Wardrobe};
pub use civ_build::{BuildingId, ParcelKind};
pub use civ_voxel::material::{MaterialDef, MaterialRegistry, Phase};
pub use civ_voxel::MaterialId;

/// Schema version for the SDK public surface.
pub const SCHEMA_VERSION: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::building::{BuildingCatalog, RecipeCatalog};
    use crate::events::SimulationEventHook;
    use crate::material::MaterialCatalog;
    use tempfile::tempdir;

    struct ExampleMod;

    impl MaterialRegistrar for ExampleMod {
        fn register_materials(&self, catalog: &mut MaterialCatalog) {
            catalog.register(CustomMaterial {
                spec: MaterialSpec {
                    name: "Marble".to_owned(),
                    phase: Phase::Solid,
                    density: 2_700,
                    flow_rate: 0,
                    viscosity: 0,
                    angle_of_repose: None,
                    color: [240, 240, 244, 255],
                },
                base_id: None,
            });
        }
    }

    impl BuildingRegistrar for ExampleMod {
        fn register_buildings(&self, catalog: &mut BuildingCatalog) {
            catalog.register(BuildingRegistration {
                blueprint: BuildingBlueprint {
                    id: "marble-cottage".to_owned(),
                    name: "Marble Cottage".to_owned(),
                    kind: BuildingKind::Residential,
                    preferred_materials: vec![MaterialId(12)],
                    era_min: 2,
                },
            });
        }
    }

    impl RecipeRegistrar for ExampleMod {
        fn register_recipes(&self, catalog: &mut RecipeCatalog) {
            catalog.register(RecipeRegistration {
                recipe: RecipeDefinition {
                    id: "marble-block".to_owned(),
                    name: "Marble Block".to_owned(),
                    inputs: vec![(MaterialId(12), 4)],
                    outputs: vec![(MaterialId(12), 1)],
                    building: Some("marble-cottage".to_owned()),
                },
            });
        }
    }

    impl SimulationEventHook for ExampleMod {
        fn on_birth(&mut self, event: &BirthEvent) {
            let _ = event;
        }

        fn on_death(&mut self, event: &DeathEvent) {
            let _ = event;
        }

        fn on_tech(&mut self, event: &TechEvent) {
            let _ = event;
        }
    }

    #[test]
    fn schema_version_is_stable() {
        assert_eq!(SCHEMA_VERSION, "0.1.0");
    }

    #[test]
    fn example_mod_registers_materials_buildings_and_recipes() {
        let mod_ = ExampleMod;

        let mut materials = MaterialCatalog::default();
        mod_.register_materials(&mut materials);
        let marble = materials.by_name("Marble").expect("marble");
        assert_eq!(marble.material.spec.phase, Phase::Solid);

        let mut buildings = BuildingCatalog::default();
        mod_.register_buildings(&mut buildings);
        assert_eq!(
            buildings.by_id("marble-cottage").unwrap().blueprint.era_min,
            2
        );

        let mut recipes = RecipeCatalog::default();
        mod_.register_recipes(&mut recipes);
        assert_eq!(
            recipes.by_id("marble-block").unwrap().recipe.inputs.len(),
            1
        );
    }

    #[test]
    fn manifest_loader_supports_json_and_ron() {
        let dir = tempdir().expect("tempdir");
        let mods = dir.path().join("mods");
        std::fs::create_dir(&mods).expect("mods");
        std::fs::write(
            mods.join("manifest.json"),
            r#"{
              "mod": { "id":"marble", "name":"Marble Mod", "version":"1.2.3", "author":"CivLab", "description":"adds marble", "entrypoint":"marble.wasm" },
              "materials": [{"name":"Marble","phase":"Solid","density":2700,"flow_rate":0,"viscosity":0,"angle_of_repose":null,"color":[240,240,244,255]}]
            }"#,
        )
        .expect("json");
        std::fs::write(
            mods.join("manifest.ron"),
            r#"(mod:(id:"stone",name:"Stone Mod",version:"0.1.0",author:"CivLab",description:"adds stone",entrypoint:Some("stone.wasm")),buildings:[],recipes:[],events:[])"#,
        )
        .expect("ron");

        let manifests = load_manifests_from_dir(&mods).expect("load");
        assert_eq!(manifests.len(), 2);
    }
}
