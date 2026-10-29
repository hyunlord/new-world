//! Strict species loading, cross-family references and extension append semantics.
use super::{
    index::Index, sources, Catalog, CatalogError, IdentityDef, PlantDef, PlantPhase, PlantPlacement,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlantFile {
    schema_version: u32,
    plants: Vec<PlantRaw>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PlantRaw {
    identity: IdentityDef,
    category: String,
    tags: Vec<String>,
    placement: PlantPlacement,
    phases: Vec<PlantPhase>,
    ripe_start_day: u32,
    ripe_end_day: Option<u32>,
    #[serde(rename = "yield")]
    output: YieldRaw,
    harvest_ticks: u32,
    knowledge: Option<String>,
    knowledge_speed_permille: u32,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct YieldRaw {
    item: String,
    units: u16,
    regrow_days: u32,
}
fn parse(map: &BTreeMap<String, Vec<u8>>) -> Result<Vec<PlantRaw>, CatalogError> {
    let file: PlantFile = ron::from_str(sources::text(map, "plants.ron")?)
        .map_err(|e| CatalogError::new("plants", e.to_string()))?;
    if file.schema_version != 1 {
        return Err(CatalogError::new("plants", "unsupported schema"));
    }
    Ok(file.plants)
}
fn resolve(raw: PlantRaw, catalog: &Catalog) -> Result<PlantDef, CatalogError> {
    let context = &raw.identity.id;
    let error = |message| CatalogError::new(context, message);
    if raw.category.is_empty()
        || raw.tags.iter().any(String::is_empty)
        || raw.tags.iter().collect::<BTreeSet<_>>().len() != raw.tags.len()
        || raw.placement.terrains.is_empty()
        || raw.placement.weight == 0
        || raw.placement.density_permille > 1000
        || raw.placement.elevation.0 > raw.placement.elevation.1
        || raw.output.units == 0
        || raw.harvest_ticks == 0
        || raw.knowledge_speed_permille == 0
        || raw.phases.len() != 4
        || !raw.phases.contains(&PlantPhase::Ripe)
        || raw
            .ripe_end_day
            .is_some_and(|end| end <= raw.ripe_start_day)
    {
        return Err(error(
            "invalid species distribution, yield or seasonal window",
        ));
    }
    for (i, terrain) in raw.placement.terrains.iter().enumerate() {
        if raw.placement.terrains[..i].contains(terrain) {
            return Err(error("duplicate placement terrain"));
        }
    }
    let yield_item = catalog
        .item_by_name(&raw.output.item)
        .ok_or_else(|| error("unknown yield item"))?;
    let knowledge = raw
        .knowledge
        .as_deref()
        .map(|name| {
            catalog
                .knowledge_defs()
                .id_of(name)
                .ok_or_else(|| error("unknown harvest knowledge"))
        })
        .transpose()?;
    let phases: [PlantPhase; 4] = raw
        .phases
        .try_into()
        .map_err(|_| error("exactly four seasonal phases required"))?;
    Ok(PlantDef {
        identity: raw.identity,
        category: raw.category,
        tags: raw.tags,
        placement: raw.placement,
        phases,
        ripe_start_day: raw.ripe_start_day,
        ripe_end_day: raw.ripe_end_day,
        yield_item,
        yield_count: raw.output.units,
        regrow_days: raw.output.regrow_days,
        harvest_ticks: raw.harvest_ticks,
        knowledge,
        knowledge_speed_permille: raw.knowledge_speed_permille,
    })
}
pub(super) fn load(
    catalog: &mut Catalog,
    base: &BTreeMap<String, Vec<u8>>,
    extension: &BTreeMap<String, Vec<u8>>,
) -> Result<(), CatalogError> {
    let mut rows = parse(base)?;
    if extension.contains_key("plants.ron") {
        rows.extend(parse(extension)?);
    }
    rows.sort_by_key(|r| r.identity.code);
    let index = Index::build("plants", rows.iter().map(|r| &r.identity))?;
    let mut weight_sum = 0u64;
    let mut density_sum = 0u64;
    for row in &rows {
        weight_sum = weight_sum
            .checked_add(u64::from(row.placement.weight))
            .ok_or_else(|| CatalogError::new("plants", "total weight overflow"))?;
        density_sum = density_sum
            .checked_add(
                u64::from(row.placement.weight) * u64::from(row.placement.density_permille),
            )
            .ok_or_else(|| CatalogError::new("plants", "weighted density overflow"))?;
    }
    let plants = rows
        .into_iter()
        .map(|r| resolve(r, catalog))
        .collect::<Result<_, _>>()?;
    catalog.plant_index = index;
    catalog.plants = plants;
    Ok(())
}
