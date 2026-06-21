//! civlab-sdk integration coverage tests (FR-CIV-TEST-013).
//!
//! Targets uncovered pub fns: MaterialCatalog::materials / into_material_def,
//! BuildingCatalog::buildings, RecipeCatalog::recipes, ModRegistry,
//! load_manifest_file error paths, BuildingKind->ParcelKind, policy module.

use civlab_sdk::building::{BuildingBlueprint, BuildingCatalog, BuildingKind, BuildingRegistration, RecipeCatalog, RecipeDefinition, RecipeRegistration};
use civlab_sdk::material::{CustomMaterial, MaterialCatalog, MaterialSpec};
use civlab_sdk::manifest::{load_manifest_file, load_manifests_from_dir, ManifestError};
use civlab_sdk::registry::ModRegistry;
use civlab_sdk::policy::{
    ACTION_SET_TAX_RATE, ACTION_SET_POLICY_PARAM, ACTION_SET_SUBSIDY_RATE,
    ACTION_TRANSFER_FUNDS, ACTION_TRIGGER_EVENT,
    WorldDomain, PolicyAction, PolicyContext, EconomySnapshot, ClimateSnapshot,
};
use civlab_sdk::{MaterialId, Phase, SCHEMA_VERSION};
use tempfile::tempdir;
use civ_build::ParcelKind;

// ---------------------------------------------------------------------------
// MaterialCatalog: materials() slice + into_material_def()
// ---------------------------------------------------------------------------

#[test]
fn material_catalog_materials_slice_matches_register_count() {
    let mut cat = MaterialCatalog::default();
    assert_eq!(cat.materials().len(), 0, "empty catalog has no materials");

    let spec = MaterialSpec {
        name: "Granite".to_owned(),
        phase: Phase::Solid,
        density: 2_600,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: None,
        color: [200, 180, 170, 255],
    };
    cat.register(CustomMaterial { spec, base_id: None });
    assert_eq!(cat.materials().len(), 1);
    assert_eq!(cat.materials()[0].material.spec.name, "Granite");
}

#[test]
fn custom_material_into_material_def_copies_spec_fields() {
    let spec = MaterialSpec {
        name: "Basalt".to_owned(),
        phase: Phase::Solid,
        density: 3_000,
        flow_rate: 0,
        viscosity: 0,
        angle_of_repose: Some(45),
        color: [50, 50, 50, 255],
    };
    let mat = CustomMaterial { spec: spec.clone(), base_id: None };
    let def = mat.into_material_def(MaterialId(1001));
    assert_eq!(def.id, MaterialId(1001));
    assert_eq!(def.phase, Phase::Solid);
    assert_eq!(def.density, 3_000);
    assert_eq!(def.angle_of_repose, Some(45));
    assert_eq!(def.color, [50, 50, 50, 255]);
}

#[test]
fn material_catalog_by_name_returns_none_for_unknown() {
    let cat = MaterialCatalog::default();
    assert!(cat.by_name("Unobtainium").is_none());
}

// ---------------------------------------------------------------------------
// BuildingCatalog: buildings() slice
// ---------------------------------------------------------------------------

#[test]
fn building_catalog_buildings_slice_and_missing_id() {
    let mut cat = BuildingCatalog::default();
    assert_eq!(cat.buildings().len(), 0);

    cat.register(BuildingRegistration {
        blueprint: BuildingBlueprint {
            id: "watchtower".to_owned(),
            name: "Watch Tower".to_owned(),
            kind: BuildingKind::Civic,
            preferred_materials: vec![],
            era_min: 0,
        },
    });
    assert_eq!(cat.buildings().len(), 1);
    assert_eq!(cat.buildings()[0].blueprint.id, "watchtower");
    assert!(cat.by_id("nonexistent").is_none());
}

// ---------------------------------------------------------------------------
// RecipeCatalog: recipes() slice
// ---------------------------------------------------------------------------

#[test]
fn recipe_catalog_recipes_slice_and_missing_id() {
    let mut cat = RecipeCatalog::default();
    assert_eq!(cat.recipes().len(), 0);

    cat.register(RecipeRegistration {
        recipe: RecipeDefinition {
            id: "clay-brick".to_owned(),
            name: "Clay Brick".to_owned(),
            inputs: vec![(MaterialId(100), 2)],
            outputs: vec![(MaterialId(101), 4)],
            building: None,
        },
    });
    assert_eq!(cat.recipes().len(), 1);
    assert!(cat.by_id("nonexistent").is_none());
}

// ---------------------------------------------------------------------------
// ModRegistry
// ---------------------------------------------------------------------------

#[test]
fn mod_registry_new_is_empty_and_register_manifest_works() {
    use civlab_sdk::manifest::{ModManifest, ModMetadata};

    let mut reg = ModRegistry::new();
    assert_eq!(reg.manifests().len(), 0);

    let manifest = ModManifest {
        metadata: ModMetadata {
            id: "test-mod".to_owned(),
            name: "Test Mod".to_owned(),
            version: "0.1.0".to_owned(),
            author: "CivLab".to_owned(),
            description: "Test".to_owned(),
            entrypoint: None,
        },
        materials: vec![],
        buildings: vec![],
        recipes: vec![],
        events: vec![],
    };
    reg.register_manifest(manifest);
    assert_eq!(reg.manifests().len(), 1);
    assert_eq!(reg.manifests()[0].metadata.id, "test-mod");
}

// ---------------------------------------------------------------------------
// load_manifest_file error paths
// ---------------------------------------------------------------------------

#[test]
fn load_manifest_file_missing_path_returns_io_error() {
    let result = load_manifest_file("/nonexistent/path/manifest.json");
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ManifestError::Io { .. }));
}

#[test]
fn load_manifests_from_dir_empty_dir_returns_not_found() {
    let dir = tempdir().expect("tempdir");
    let result = load_manifests_from_dir(dir.path());
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ManifestError::NotFound { .. }));
}

#[test]
fn load_manifest_file_invalid_json_returns_parse_error() {
    let dir = tempdir().expect("tempdir");
    let f = dir.path().join("manifest.json");
    std::fs::write(&f, "{ this is not valid json }").expect("write");
    let result = load_manifest_file(&f);
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), ManifestError::Parse { .. }));
}

// ---------------------------------------------------------------------------
// BuildingKind -> ParcelKind conversion
// ---------------------------------------------------------------------------

#[test]
fn building_kind_into_parcel_kind_all_variants() {
    assert_eq!(ParcelKind::from(BuildingKind::Residential), ParcelKind::Residential);
    assert_eq!(ParcelKind::from(BuildingKind::Commercial), ParcelKind::Commercial);
    assert_eq!(ParcelKind::from(BuildingKind::Industrial), ParcelKind::Industrial);
    assert_eq!(ParcelKind::from(BuildingKind::Civic), ParcelKind::Civic);
}

// ---------------------------------------------------------------------------
// policy module: WorldDomain, PolicyAction, PolicyContext
// ---------------------------------------------------------------------------

#[test]
fn world_domain_from_i32_covers_all_variants_and_unknown() {
    use civlab_sdk::policy::WorldDomain;
    assert_eq!(WorldDomain::from_i32(0), Some(WorldDomain::Economy));
    assert_eq!(WorldDomain::from_i32(1), Some(WorldDomain::Climate));
    assert_eq!(WorldDomain::from_i32(2), Some(WorldDomain::Military));
    assert_eq!(WorldDomain::from_i32(3), Some(WorldDomain::Diplomacy));
    assert_eq!(WorldDomain::from_i32(4), Some(WorldDomain::Citizens));
    assert_eq!(WorldDomain::from_i32(-1), None);
    assert_eq!(WorldDomain::from_i32(100), None);
}

#[test]
fn policy_action_type_discriminants_stable() {
    assert_eq!(ACTION_SET_TAX_RATE, 1);
    assert_eq!(ACTION_SET_POLICY_PARAM, 2);
    assert_eq!(ACTION_SET_SUBSIDY_RATE, 3);
    assert_eq!(ACTION_TRANSFER_FUNDS, 4);
    assert_eq!(ACTION_TRIGGER_EVENT, 5);

    assert_eq!(
        PolicyAction::SetTaxRate { rate_permille: 250 }.action_type(),
        ACTION_SET_TAX_RATE
    );
    assert_eq!(
        PolicyAction::SetPolicyParam { key_hash: 99, value: 42 }.action_type(),
        ACTION_SET_POLICY_PARAM
    );
}

#[test]
fn policy_context_default_has_all_none_snapshots() {
    let ctx = PolicyContext::default();
    assert_eq!(ctx.tick, 0);
    assert!(ctx.economy.is_none());
    assert!(ctx.climate.is_none());
    assert!(ctx.military.is_none());
    assert!(ctx.diplomacy.is_none());
    assert!(ctx.citizens.is_none());
}

#[test]
fn policy_context_with_snapshots_returns_correct_values() {
    let ctx = PolicyContext {
        tick: 100,
        economy: Some(EconomySnapshot { treasury_millijoules: 1_000_000 }),
        climate: Some(ClimateSnapshot { co2_ppm_milliunits: 420_000 }),
        military: None,
        diplomacy: None,
        citizens: None,
    };
    assert_eq!(ctx.tick, 100);
    assert_eq!(ctx.economy.unwrap().treasury_millijoules, 1_000_000);
    assert_eq!(ctx.climate.unwrap().co2_ppm_milliunits, 420_000);
}

// ---------------------------------------------------------------------------
// SCHEMA_VERSION stability (belt-and-suspenders)
// ---------------------------------------------------------------------------

#[test]
fn schema_version_constant_is_0_1_0() {
    assert_eq!(SCHEMA_VERSION, "0.1.0");
}