//! Two-pass local family construction, followed by knowledge derivation.
use super::{
    index::Index, raw_blueprint::BlueprintRaw, raw_item::ItemRaw, raw_recipe::RecipeRaw, *,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ItemFile {
    schema_version: u32,
    #[serde(default)]
    roles: BTreeMap<String, String>,
    items: Vec<ItemRaw>,
}
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct BlueprintFile {
    schema_version: u32,
    #[serde(default)]
    roles: BTreeMap<String, String>,
    #[serde(default)]
    material_roles: BTreeMap<String, String>,
    #[serde(default)]
    materials: Vec<String>,
    #[serde(default)]
    historical_stone_wall_materials: Vec<String>,
    blueprints: Vec<BlueprintRaw>,
}
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct RecipeFile {
    schema_version: u32,
    #[serde(default)]
    roles: BTreeMap<String, String>,
    recipes: Vec<RecipeRaw>,
}
fn parse<T: serde::de::DeserializeOwned>(
    map: &BTreeMap<String, Vec<u8>>,
    name: &str,
) -> Result<T, CatalogError> {
    ron::from_str(sources::text(map, name)?).map_err(|e| CatalogError::new(name, e.to_string()))
}
fn schema(version: u32, name: &str) -> Result<(), CatalogError> {
    if version == 1 {
        Ok(())
    } else {
        Err(CatalogError::new(
            name,
            format!("unsupported schema {version}"),
        ))
    }
}
fn append_roles(
    base: &mut BTreeMap<String, String>,
    extra: BTreeMap<String, String>,
) -> Result<(), CatalogError> {
    for (role, value) in extra {
        if role.is_empty() || base.insert(role.clone(), value).is_some() {
            return Err(CatalogError::new(
                "roles",
                format!("duplicate/empty role {role:?}"),
            ));
        }
    }
    Ok(())
}

pub(super) fn build(
    base: RawSources,
    extension: Option<RawSources>,
) -> Result<Catalog, CatalogError> {
    let base = base.into_map()?;
    let extension = extension.unwrap_or_default().into_map()?;
    let revision = sources::revision(&base, &extension);
    let mut items: ItemFile = parse(&base, "items.ron")?;
    let mut blueprints: BlueprintFile = parse(&base, "blueprints.ron")?;
    let mut recipes: RecipeFile = parse(&base, "recipes.ron")?;
    schema(items.schema_version, "items")?;
    schema(blueprints.schema_version, "blueprints")?;
    let base_materials = blueprints.materials.iter().cloned().collect();
    schema(recipes.schema_version, "recipes")?;
    if extension.contains_key("items.ron") {
        let extra: ItemFile = parse(&extension, "items.ron")?;
        schema(extra.schema_version, "items")?;
        if extra
            .items
            .iter()
            .any(|d| d.food.is_some() || d.apparel.is_some())
        {
            return Err(CatalogError::new(
                "items",
                "novel perishable/apparel identity is not supported by saved legacy mechanics",
            ));
        }
        append_roles(&mut items.roles, extra.roles)?;
        items.items.extend(extra.items);
    }
    if extension.contains_key("blueprints.ron") {
        let extra: BlueprintFile = parse(&extension, "blueprints.ron")?;
        schema(extra.schema_version, "blueprints")?;
        if extra
            .blueprints
            .iter()
            .any(|d| d.completion_policy != CompletionPolicy::Authoritative)
        {
            return Err(CatalogError::new(
                "blueprints",
                "extension completion must preserve authoritative identity",
            ));
        }
        append_roles(&mut blueprints.roles, extra.roles)?;
        append_roles(&mut blueprints.material_roles, extra.material_roles)?;
        if !extra.historical_stone_wall_materials.is_empty() {
            return Err(CatalogError::new(
                "blueprints",
                "extension cannot alter historical wall projection",
            ));
        }
        blueprints.materials.extend(extra.materials);
        blueprints.blueprints.extend(extra.blueprints);
    }
    if extension.contains_key("recipes.ron") {
        let extra: RecipeFile = parse(&extension, "recipes.ron")?;
        schema(extra.schema_version, "recipes")?;
        append_roles(&mut recipes.roles, extra.roles)?;
        recipes.recipes.extend(extra.recipes);
    }
    items.items.sort_by_key(|d| d.identity.code);
    blueprints.blueprints.sort_by_key(|d| d.identity.code);
    recipes.recipes.sort_by_key(|d| d.identity.code);
    let item_index = Index::build("items", items.items.iter().map(|d| &d.identity))?;
    let blueprint_index = Index::build(
        "blueprints",
        blueprints.blueprints.iter().map(|d| &d.identity),
    )?;
    let recipe_index = Index::build("recipes", recipes.recipes.iter().map(|d| &d.identity))?;
    let mut selected = base.clone();
    selected.extend(extension.clone());
    if extension.contains_key("knowledge.ron") {
        let merged = super::super::knowledge::merge_sources(
            sources::text(&base, "knowledge.ron")?,
            sources::text(&extension, "knowledge.ron")?,
        )
        .map_err(|e| CatalogError::new("knowledge", e))?;
        selected.insert("knowledge.ron".into(), merged.into_bytes());
    }
    let mut materials = BTreeSet::new();
    for name in blueprints.materials {
        if name.is_empty() || !materials.insert(name.clone()) {
            return Err(CatalogError::new(
                "materials",
                format!("duplicate/empty material {name:?}"),
            ));
        }
    }
    let flora = load_flora::build(&base, &extension, &base_materials, &materials)?;
    let mut catalog = Catalog {
        flora,
        animals: Vec::new(),
        pressures: Vec::new(),
        animal_index: Index::default(),
        pressure_index: Index::default(),
        animal_roles: BTreeMap::new(),
        pressure_roles: BTreeMap::new(),
        pressure_schedule: PressureSchedule::default(),
        items: Vec::new(),
        blueprints: Vec::new(),
        recipes: Vec::new(),
        item_index,
        blueprint_index,
        recipe_index,
        item_roles: BTreeMap::new(),
        blueprint_roles: BTreeMap::new(),
        recipe_roles: BTreeMap::new(),
        material_roles: BTreeMap::new(),
        historical_stone_materials: BTreeSet::new(),
        materials,
        sources: selected,
        revision,
        knowledge_defs: KnowledgeDefs::default(),
        knowledge_gates: Gates::default(),
    };
    catalog.items = items
        .items
        .into_iter()
        .map(|r| resolve_items::resolve(r, &catalog))
        .collect::<Result<_, _>>()?;
    catalog.blueprints = blueprints
        .blueprints
        .into_iter()
        .map(|r| resolve_blueprints::resolve(r, &catalog))
        .collect::<Result<_, _>>()?;
    catalog.recipes = recipes
        .recipes
        .into_iter()
        .map(|r| resolve_recipes::resolve(r, &catalog))
        .collect::<Result<_, _>>()?;
    for (role, name) in items.roles {
        let id = catalog
            .item_by_name(&name)
            .ok_or_else(|| CatalogError::new(&role, "unknown item role target"))?;
        catalog.item_roles.insert(role, id);
    }
    for (role, name) in blueprints.roles {
        let id = catalog
            .blueprint_by_name(&name)
            .ok_or_else(|| CatalogError::new(&role, "unknown blueprint role target"))?;
        catalog.blueprint_roles.insert(role, id);
    }
    for (role, name) in recipes.roles {
        let id = catalog
            .recipe_by_name(&name)
            .ok_or_else(|| CatalogError::new(&role, "unknown recipe role target"))?;
        catalog.recipe_roles.insert(role, id);
    }
    for (role, name) in blueprints.material_roles {
        let id = validate::material(&catalog, &name)?;
        catalog.material_roles.insert(role, id);
    }
    for name in blueprints.historical_stone_wall_materials {
        catalog
            .historical_stone_materials
            .insert(validate::material(&catalog, &name)?);
    }
    load_ecology::load(&mut catalog, &base, &extension)?;
    validate::roles(&catalog)?;
    catalog.knowledge_defs = super::super::knowledge::parse_with_catalog(
        catalog.source_text("knowledge.ron")?,
        &catalog,
    )
    .map_err(|e| CatalogError::new("knowledge", e))?;
    catalog.knowledge_gates =
        super::super::knowledge::gates_with_catalog(&catalog.knowledge_defs, &catalog)
            .map_err(|e| CatalogError::new("knowledge gates", e))?;
    validate::knowledge_refs(&catalog)?;
    Ok(catalog)
}
