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
