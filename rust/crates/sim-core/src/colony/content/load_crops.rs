//! Strict crop definitions; extensions append stable identities.
use super::{
    index::Index, sources, Catalog, CatalogError, CropDef, CropYield, IdentityDef, SowingWindow,
};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CropFile {
    schema_version: u32,
    crops: Vec<CropRaw>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct YieldRaw {
    item: String,
    count: u16,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CropRaw {
    identity: IdentityDef,
    category: String,
    tags: Vec<String>,
    seed_item: String,
    seeds_per_tile: u16,
    yields: Vec<YieldRaw>,
    #[serde(default = "default_frost_sensitive")]
    frost_sensitive: bool,
    sow_windows: Vec<SowingWindow>,
    growth_days: u32,
    till_ticks: u32,
    sow_ticks: u32,
    tend_ticks: u32,
    harvest_ticks: u32,
    untended_yield_permille: u16,
    fertility_cost: u16,
    fallow_recovery_per_day: u16,
    initial_fertility: u16,
    min_fertility: u16,
    knowledge: Option<String>,
}
fn default_frost_sensitive() -> bool {
    true
}
fn parse(map: &BTreeMap<String, Vec<u8>>) -> Result<Vec<CropRaw>, CatalogError> {
    let file: CropFile = ron::from_str(sources::text(map, "crops.ron")?)
        .map_err(|e| CatalogError::new("crops", e.to_string()))?;
    if file.schema_version != 1 {
        return Err(CatalogError::new("crops", "unsupported schema"));
    }
    Ok(file.crops)
}
fn resolve(raw: CropRaw, catalog: &Catalog) -> Result<CropDef, CatalogError> {
    let error = |message| CatalogError::new(&raw.identity.id, message);
    if raw.category.is_empty()
        || raw.tags.iter().any(String::is_empty)
        || raw.tags.iter().collect::<BTreeSet<_>>().len() != raw.tags.len()
        || raw.seeds_per_tile == 0
        || raw.yields.is_empty()
        || raw.growth_days == 0
        || [
            raw.till_ticks,
            raw.sow_ticks,
            raw.tend_ticks,
            raw.harvest_ticks,
        ]
        .contains(&0)
        || raw.untended_yield_permille > 1000
        || raw.initial_fertility > 1000
        || raw.min_fertility > raw.initial_fertility
        || raw.fertility_cost > 1000
        || raw.fallow_recovery_per_day > 1000
        || raw.sow_windows.is_empty()
        || raw
            .sow_windows
            .iter()
            .any(|w| w.season >= 4 || w.start_day >= w.end_day_exclusive || w.end_day_exclusive > 6)
    {
        return Err(error("invalid crop work, soil or sowing window"));
    }
    let seed_item = catalog
        .item_by_name(&raw.seed_item)
        .ok_or_else(|| error("unknown seed item"))?;
    let mut seen = BTreeSet::new();
    let yields = raw
        .yields
        .iter()
        .map(|row| {
            let item = catalog
                .item_by_name(&row.item)
                .ok_or_else(|| error("unknown crop output"))?;
            if row.count == 0 || !seen.insert(item) {
                return Err(error("empty or duplicate crop output"));
            }
            Ok(CropYield {
                item,
                count: row.count,
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let knowledge = raw
        .knowledge
        .as_deref()
        .map(|name| {
            catalog
                .knowledge_defs()
                .id_of(name)
                .ok_or_else(|| error("unknown cultivation knowledge"))
        })
        .transpose()?;
    Ok(CropDef {
        identity: raw.identity,
        category: raw.category,
        tags: raw.tags,
        seed_item,
        seeds_per_tile: raw.seeds_per_tile,
        yields,
        sow_windows: raw.sow_windows,
        frost_sensitive: raw.frost_sensitive,
        growth_days: raw.growth_days,
        till_ticks: raw.till_ticks,
        sow_ticks: raw.sow_ticks,
        tend_ticks: raw.tend_ticks,
        harvest_ticks: raw.harvest_ticks,
        untended_yield_permille: raw.untended_yield_permille,
        fertility_cost: raw.fertility_cost,
        fallow_recovery_per_day: raw.fallow_recovery_per_day,
        initial_fertility: raw.initial_fertility,
        min_fertility: raw.min_fertility,
        knowledge,
    })
}
pub(super) fn load(
    catalog: &mut Catalog,
    base: &BTreeMap<String, Vec<u8>>,
    extension: &BTreeMap<String, Vec<u8>>,
) -> Result<(), CatalogError> {
    let mut rows = parse(base)?;
    if extension.contains_key("crops.ron") {
        rows.extend(parse(extension)?);
    }
    rows.sort_by_key(|row| row.identity.code);
    let index = Index::build("crops", rows.iter().map(|row| &row.identity))?;
    let crops = rows
        .into_iter()
        .map(|row| resolve(row, catalog))
        .collect::<Result<_, _>>()?;
    catalog.crop_index = index;
    catalog.crops = crops;
    Ok(())
}
