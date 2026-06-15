use civ_build::ParcelKind;
use civ_voxel::MaterialId;
use serde::{Deserialize, Serialize};

/// Hexagonal building kind used by mods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuildingKind {
    /// Housing.
    Residential,
    /// Commerce or services.
    Commercial,
    /// Manufacturing / extraction.
    Industrial,
    /// Civic / public.
    Civic,
}

/// Declarative building blueprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingBlueprint {
    /// Stable mod-local id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Building kind.
    pub kind: BuildingKind,
    /// Material preferences expressed in `civ_voxel` ids.
    pub preferred_materials: Vec<MaterialId>,
    /// Earliest era this building can appear.
    pub era_min: u16,
}

impl From<BuildingKind> for ParcelKind {
    fn from(value: BuildingKind) -> Self {
        match value {
            BuildingKind::Residential => Self::Residential,
            BuildingKind::Commercial => Self::Commercial,
            BuildingKind::Industrial => Self::Industrial,
            BuildingKind::Civic => Self::Civic,
        }
    }
}

/// Registration wrapper for a blueprint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BuildingRegistration {
    /// Blueprint definition.
    pub blueprint: BuildingBlueprint,
}

/// Registry of mod buildings.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildingCatalog {
    buildings: Vec<BuildingRegistration>,
}

impl BuildingCatalog {
    /// Register a building blueprint.
    pub fn register(&mut self, building: BuildingRegistration) {
        self.buildings.push(building);
    }

    /// Find a building by id.
    #[must_use]
    pub fn by_id(&self, id: &str) -> Option<&BuildingRegistration> {
        self.buildings.iter().find(|entry| entry.blueprint.id == id)
    }

    /// All registered blueprints.
    #[must_use]
    pub fn buildings(&self) -> &[BuildingRegistration] {
        &self.buildings
    }
}

/// Recipe definition for crafted outputs and unlocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeDefinition {
    /// Stable mod-local id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Input materials and counts.
    pub inputs: Vec<(MaterialId, u32)>,
    /// Output materials and counts.
    pub outputs: Vec<(MaterialId, u32)>,
    /// Optional building gate.
    pub building: Option<String>,
}

/// Registration wrapper for a recipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecipeRegistration {
    /// Recipe definition.
    pub recipe: RecipeDefinition,
}

/// Registry of mod recipes.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RecipeCatalog {
    recipes: Vec<RecipeRegistration>,
}

impl RecipeCatalog {
    /// Register a recipe.
    pub fn register(&mut self, recipe: RecipeRegistration) {
        self.recipes.push(recipe);
    }

    /// Find a recipe by id.
    #[must_use]
    pub fn by_id(&self, id: &str) -> Option<&RecipeRegistration> {
        self.recipes.iter().find(|entry| entry.recipe.id == id)
    }

    /// All registered recipes.
    #[must_use]
    pub fn recipes(&self) -> &[RecipeRegistration] {
        &self.recipes
    }
}

/// Trait implemented by mods that contribute buildings.
pub trait BuildingRegistrar {
    /// Register custom buildings into the supplied catalog.
    fn register_buildings(&self, catalog: &mut BuildingCatalog);
}

/// Trait implemented by mods that contribute recipes.
pub trait RecipeRegistrar {
    /// Register custom recipes into the supplied catalog.
    fn register_recipes(&self, catalog: &mut RecipeCatalog);
}

#[cfg(test)]
mod tests {
    //! Unit tests for the building/recipe catalogs and registrar traits.
    //!
    //! Traces:
    //! - CIV-0700 (modding API): mods contribute blueprints + recipes via
    //!   `BuildingRegistrar` / `RecipeRegistrar` against a `BuildingCatalog` /
    //!   `RecipeCatalog`.
    //! - SDK `SCHEMA_VERSION` is `0.1.0` (see `lib.rs`).
    //!
    //! Targets the public API surface in this file:
    //! - `BuildingCatalog::{register, by_id, buildings}`
    //! - `BuildingKind -> ParcelKind` `From` conversion
    //! - `RecipeCatalog::{register, by_id, recipes}`
    //! - `BuildingRegistrar` / `RecipeRegistrar` trait dispatch

    use super::*;
    use civ_voxel::MaterialId;

    fn sample_blueprint(id: &str, kind: BuildingKind, era_min: u16) -> BuildingBlueprint {
        BuildingBlueprint {
            id: id.to_owned(),
            name: format!("{id} display"),
            kind,
            preferred_materials: vec![MaterialId(1), MaterialId(2)],
            era_min,
        }
    }

    fn sample_recipe(id: &str, building: Option<&str>) -> RecipeDefinition {
        RecipeDefinition {
            id: id.to_owned(),
            name: format!("{id} recipe"),
            inputs: vec![(MaterialId(1), 2)],
            outputs: vec![(MaterialId(7), 1)],
            building: building.map(str::to_owned),
        }
    }

    /// Two registrations of the same id are both retained; `by_id` returns
    /// the first match (catalog does not enforce uniqueness — modders may
    /// shadow).
    #[test]
    fn building_catalog_register_and_lookup_roundtrip() {
        let mut catalog = BuildingCatalog::default();
        catalog.register(BuildingRegistration {
            blueprint: sample_blueprint("cottage", BuildingKind::Residential, 0),
        });
        catalog.register(BuildingRegistration {
            blueprint: sample_blueprint("forge", BuildingKind::Industrial, 1),
        });

        assert_eq!(catalog.buildings().len(), 2);
        let cottage = catalog
            .by_id("cottage")
            .expect("cottage must be retrievable by id");
        assert_eq!(cottage.blueprint.kind, BuildingKind::Residential);
        assert_eq!(cottage.blueprint.era_min, 0);
        assert_eq!(cottage.blueprint.preferred_materials, vec![MaterialId(1), MaterialId(2)]);

        let forge = catalog.by_id("forge").expect("forge must be retrievable by id");
        assert_eq!(forge.blueprint.kind, BuildingKind::Industrial);

        // Unknown id returns None.
        assert!(catalog.by_id("missing").is_none());
    }

    /// Default catalog is empty and `Default` round-trips through `PartialEq`.
    #[test]
    fn building_catalog_default_is_empty() {
        let catalog = BuildingCatalog::default();
        assert!(catalog.buildings().is_empty());
        assert!(catalog.by_id("anything").is_none());
    }

    /// `BuildingKind` maps 1:1 onto `ParcelKind` for engine parcel routing.
    #[test]
    fn building_kind_maps_to_parcel_kind() {
        assert_eq!(ParcelKind::from(BuildingKind::Residential), ParcelKind::Residential);
        assert_eq!(ParcelKind::from(BuildingKind::Commercial), ParcelKind::Commercial);
        assert_eq!(ParcelKind::from(BuildingKind::Industrial), ParcelKind::Industrial);
        assert_eq!(ParcelKind::from(BuildingKind::Civic), ParcelKind::Civic);
    }

    /// Recipes round-trip through the catalog and the recipe's `building`
    /// field is preserved as a free-form gate string (validated by the
    /// engine, not the catalog).
    #[test]
    fn recipe_catalog_register_and_lookup_roundtrip() {
        let mut catalog = RecipeCatalog::default();
        catalog.register(RecipeRegistration {
            recipe: sample_recipe("plank", Some("carpenter")),
        });
        catalog.register(RecipeRegistration {
            recipe: sample_recipe("ore", None),
        });

        assert_eq!(catalog.recipes().len(), 2);
        let plank = catalog
            .by_id("plank")
            .expect("plank recipe must be retrievable by id");
        assert_eq!(plank.recipe.outputs, vec![(MaterialId(7), 1)]);
        assert_eq!(plank.recipe.building.as_deref(), Some("carpenter"));

        let ore = catalog.by_id("ore").expect("ore recipe must be retrievable by id");
        assert!(ore.recipe.building.is_none());
        assert!(catalog.by_id("nope").is_none());
    }

    /// Registrar traits route through the same `register` API; a mod that
    /// implements both traits can populate both catalogs.
    struct MarbleMod;

    impl BuildingRegistrar for MarbleMod {
        fn register_buildings(&self, catalog: &mut BuildingCatalog) {
            catalog.register(BuildingRegistration {
                blueprint: sample_blueprint("marble-cottage", BuildingKind::Residential, 2),
            });
            catalog.register(BuildingRegistration {
                blueprint: sample_blueprint("marble-temple", BuildingKind::Civic, 3),
            });
        }
    }

    impl RecipeRegistrar for MarbleMod {
        fn register_recipes(&self, catalog: &mut RecipeCatalog) {
            catalog.register(RecipeRegistration {
                recipe: sample_recipe("marble-block", Some("marble-cottage")),
            });
        }
    }

    /// Exercising the trait surface end-to-end: registrar dispatches to the
    /// same `register` API used by direct callers.
    #[test]
    fn registrar_traits_populate_catalogs() {
        let mod_ = MarbleMod;
        let mut buildings = BuildingCatalog::default();
        let mut recipes = RecipeCatalog::default();

        mod_.register_buildings(&mut buildings);
        mod_.register_recipes(&mut recipes);

        assert_eq!(buildings.buildings().len(), 2);
        assert!(buildings.by_id("marble-cottage").is_some());
        assert!(buildings.by_id("marble-temple").is_some());
        assert_eq!(
            recipes
                .by_id("marble-block")
                .expect("marble-block recipe")
                .recipe
                .building
                .as_deref(),
            Some("marble-cottage"),
        );
    }

    /// `BuildingRegistration` / `RecipeRegistration` survive a serde round-trip
    /// (modders serialize manifests + runtime catalogs across the host
    /// boundary).
    #[test]
    fn registrations_serde_roundtrip() {
        let bp = sample_blueprint("a", BuildingKind::Commercial, 4);
        let reg = BuildingRegistration { blueprint: bp.clone() };
        let json = serde_json::to_string(&reg).expect("serialize");
        let back: BuildingRegistration = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back, reg);
        assert_eq!(back.blueprint, bp);

        let recipe = sample_recipe("b", Some("a"));
        let rreg = RecipeRegistration { recipe };
        let rjson = serde_json::to_string(&rreg).expect("serialize recipe");
        let rback: RecipeRegistration = serde_json::from_str(&rjson).expect("deserialize recipe");
        assert_eq!(rback, rreg);
    }
}
