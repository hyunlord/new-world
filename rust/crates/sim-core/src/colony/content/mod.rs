//! Immutable content catalogue: local validation precedes one explicit process installation.
mod boot;
pub mod defs;
#[cfg(test)]
mod ecology_tests;
mod error;
#[cfg(test)]
mod flora_tests;
pub mod host;
pub mod ids;
mod index;
mod load;
mod load_ecology;
mod load_flora;
mod load_plants;
mod raw_blueprint;
mod raw_item;
mod raw_recipe;
mod resolve_blueprints;
mod resolve_ecology;
mod resolve_items;
mod resolve_recipes;
mod sources;
#[cfg(test)]
mod tests;
mod validate;
mod validate_ecology;

use super::knowledge::{Gates, KnowledgeDefs};
use crate::material::MaterialId;
pub use boot::{bootstrap, installed, installed_or_bundled, BootSelection};
pub use defs::*;
pub use error::{BootError, CatalogAccessError, CatalogError};
pub use ids::{AnimalKind, BlueprintKind, ItemKind, PlantKind, PressureKind, RecipeKind};
use index::Index;
pub use sources::{NamedSource, RawSources};
use std::collections::{BTreeMap, BTreeSet};

/// Fully validated, immutable selected definitions and derived knowledge data.
#[derive(Debug)]
pub struct Catalog {
    flora: FloraCatalog,
    plants: Vec<PlantDef>,
    plant_index: Index,
    animals: Vec<AnimalDef>,
    pressures: Vec<PressureDef>,
    animal_index: Index,
    pressure_index: Index,
    animal_roles: BTreeMap<String, AnimalKind>,
    pressure_roles: BTreeMap<String, PressureKind>,
    pressure_schedule: PressureSchedule,
    items: Vec<ItemDef>,
    blueprints: Vec<BlueprintDef>,
    recipes: Vec<RecipeDef>,
    item_index: Index,
    blueprint_index: Index,
    recipe_index: Index,
    item_roles: BTreeMap<String, ItemKind>,
    blueprint_roles: BTreeMap<String, BlueprintKind>,
    recipe_roles: BTreeMap<String, RecipeKind>,
    material_roles: BTreeMap<String, MaterialId>,
    historical_stone_materials: BTreeSet<MaterialId>,
    materials: BTreeSet<String>,
    sources: BTreeMap<String, Vec<u8>>,
    revision: String,
    knowledge_defs: KnowledgeDefs,
    knowledge_gates: Gates,
}
macro_rules! family_access {
    ($id:ident,$def:ident,$field:ident,$index:ident,$one:ident,$code:ident,$name:ident,$serde:ident,$required:ident,$iter:ident,$ids:ident) => {
        /// Definition in this catalogue; an ID from another local catalogue may be absent.
        pub fn $one(&self, id: $id) -> Option<&$def> {
            self.$code(id.code())
        }
        /// Lookup an explicit sparse stable code.
        pub fn $code(&self, code: u32) -> Option<&$def> {
            self.$index
                .codes
                .get(&code)
                .and_then(|i| self.$field.get(*i))
        }
        /// Resolve canonical content names and declared resolver aliases.
        pub fn $name(&self, name: &str) -> Option<$id> {
            self.$index
                .names
                .get(name)
                .copied()
                .map($id::from_validated)
        }
        /// Resolve unit-serde names and only declared serde aliases.
        pub fn $serde(&self, name: &str) -> Option<$id> {
            self.$index
                .serde_names
                .get(name)
                .copied()
                .map($id::from_validated)
        }
        /// A contextual checked definition lookup.
        pub fn $required(&self, id: $id) -> Result<&$def, CatalogError> {
            self.$one(id).ok_or_else(|| {
                CatalogError::new(stringify!($id), format!("unknown code {}", id.code()))
            })
        }
        /// Iterate definitions in stable code order.
        pub fn $iter(&self) -> impl ExactSizeIterator<Item = &$def> {
            self.$field.iter()
        }
        /// Iterate checked family identities in stable code order.
        pub fn $ids(&self) -> impl ExactSizeIterator<Item = $id> + '_ {
            self.$index.codes.keys().copied().map($id::from_validated)
        }
    };
}
impl Catalog {
    /// Validated flora membership and generation distributions.
    pub fn flora(&self) -> &FloraCatalog {
        &self.flora
    }
    /// Pure local construction; never installs or reads global definitions.
    pub fn from_sources(
        base: RawSources,
        extension: Option<RawSources>,
    ) -> Result<Self, CatalogError> {
        load::build(base, extension)
    }
    family_access!(
        PlantKind,
        PlantDef,
        plants,
        plant_index,
        plant,
        plant_by_code,
        plant_by_name,
        plant_by_serde_name,
        require_plant,
        plants,
        plant_ids
    );
    /// Resolve a species, falling back to the first stable-code member of its category.
    pub fn resolve_plant(&self, name: &str, category: &str) -> Option<PlantKind> {
        self.plant_by_name(name).or_else(|| {
            self.plants
                .iter()
                .find(|d| d.category == category)
                .map(|d| PlantKind::from_validated(d.identity.code))
        })
    }
    family_access!(
        ItemKind,
        ItemDef,
        items,
        item_index,
        item,
        item_by_code,
        item_by_name,
        item_by_serde_name,
        require_item,
        items,
        item_ids
    );
    family_access!(
        BlueprintKind,
        BlueprintDef,
        blueprints,
        blueprint_index,
        blueprint,
        blueprint_by_code,
        blueprint_by_name,
        blueprint_by_serde_name,
        require_blueprint,
        blueprints,
        blueprint_ids
    );
    family_access!(
        RecipeKind,
        RecipeDef,
        recipes,
        recipe_index,
        recipe,
        recipe_by_code,
        recipe_by_name,
        recipe_by_serde_name,
        require_recipe,
        recipes,
        recipe_ids
    );
    family_access!(
        AnimalKind,
        AnimalDef,
        animals,
        animal_index,
        animal,
        animal_by_code,
        animal_by_name,
        animal_by_serde_name,
        require_animal,
        animals,
        animal_ids
    );
    family_access!(
        PressureKind,
        PressureDef,
        pressures,
        pressure_index,
        pressure,
        pressure_by_code,
        pressure_by_name,
        pressure_by_serde_name,
        require_pressure,
        pressures,
        pressure_ids
    );
    /// Default animal selected by a validated data role.
    pub fn animal_role(&self, role: &str) -> Option<AnimalKind> {
        self.animal_roles.get(role).copied()
    }
    /// Default pressure selected by a validated data role.
    pub fn pressure_role(&self, role: &str) -> Option<PressureKind> {
        self.pressure_roles.get(role).copied()
    }
    /// Immutable seasonal scheduling policy.
    pub fn pressure_schedule(&self) -> &PressureSchedule {
        &self.pressure_schedule
    }
    /// Existing recipe slice API backed by the selected catalogue, not a second cache.
    pub fn recipe_definitions(&self) -> &[RecipeDef] {
        &self.recipes
    }
    /// A validated unique item role.
    pub fn item_role(&self, role: &str) -> Option<ItemKind> {
        self.item_roles.get(role).copied()
    }
    /// A validated unique blueprint role.
    pub fn blueprint_role(&self, role: &str) -> Option<BlueprintKind> {
        self.blueprint_roles.get(role).copied()
    }
    /// A validated unique recipe role.
    pub fn recipe_role(&self, role: &str) -> Option<RecipeKind> {
        self.recipe_roles.get(role).copied()
    }
    /// A registered material role, separate from content IDs.
    pub fn material_role(&self, role: &str) -> Option<MaterialId> {
        self.material_roles.get(role).copied()
    }
    /// Exact historical wall projection membership from data.
    pub fn historical_stone_wall_material(&self, id: MaterialId) -> bool {
        self.historical_stone_materials.contains(&id)
    }
    /// Selected content revision, independent of filesystem staging paths.
    pub fn revision(&self) -> &str {
        &self.revision
    }
    /// Frozen selected bytes; family names accept a stem or `.ron` suffix.
    pub fn source_bytes(&self, name: &str) -> Option<&[u8]> {
        self.sources
            .get(sources::normalize(name))
            .map(Vec::as_slice)
    }
    /// Frozen UTF-8 source with source-specific error context.
    pub fn source_text(&self, name: &str) -> Result<&str, CatalogError> {
        sources::text(&self.sources, name)
    }
    /// Definitions derived from the same selected catalogue.
    pub fn knowledge_defs(&self) -> &KnowledgeDefs {
        &self.knowledge_defs
    }
    /// Gate masks derived only after local family resolution.
    pub fn knowledge_gates(&self) -> &Gates {
        &self.knowledge_gates
    }
}
