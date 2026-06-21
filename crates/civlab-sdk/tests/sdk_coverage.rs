//! External coverage tests for crates/civlab-sdk (FR-CIV-TEST-013).
//!
//! Covers: BuildingCatalog (register/by_id/buildings),
//!         MaterialCatalog (register/by_name/materials/into_material_def).
use civlab_sdk::{
    building::{BuildingBlueprint, BuildingCatalog, BuildingKind, BuildingRegistration},
    material::{CustomMaterial, MaterialCatalog, MaterialSpec},
};
use civlab_sdk::Phase;

fn sample_blueprint(id: &str) -> BuildingBlueprint {
    BuildingBlueprint {
        id: id.to_string(),
        name: id.to_string(),
        kind: BuildingKind::Residential,
        preferred_materials: vec![],
        era_min: 1,
    }
}

fn sample_material(name: &str) -> CustomMaterial {
    CustomMaterial {
        base_id: None,
        spec: MaterialSpec {
            name: name.to_string(),
            phase: Phase::Solid,
            density: 100,
            flow_rate: 0,
            viscosity: 0,
            angle_of_repose: None,
            color: [128, 64, 32, 255],
        },
    }
}

#[test]
fn building_catalog_register_and_by_id() {
    let mut cat = BuildingCatalog::default();
    assert_eq!(cat.buildings().len(), 0);

    cat.register(BuildingRegistration {
        blueprint: sample_blueprint("hut"),
    });
    assert_eq!(cat.buildings().len(), 1);

    let found = cat.by_id("hut").expect("hut should be registered");
    assert_eq!(found.blueprint.id, "hut");
    assert!(cat.by_id("nonexistent").is_none());
}

#[test]
fn material_catalog_register_by_name_and_into_def() {
    let mut cat = MaterialCatalog::default();
    assert_eq!(cat.materials().len(), 0);

    let mat_id = cat.register(sample_material("obsidian"));
    assert_eq!(cat.materials().len(), 1);

    let found = cat.by_name("obsidian").expect("obsidian should be registered");
    assert_eq!(found.id, "obsidian");
    assert!(cat.by_name("granite").is_none());

    // into_material_def converts with supplied stable id
    let def = sample_material("granite").into_material_def(mat_id);
    assert_eq!(def.id, mat_id);
    assert_eq!(def.density, 100);
    assert_eq!(def.color, [128, 64, 32, 255]);
}