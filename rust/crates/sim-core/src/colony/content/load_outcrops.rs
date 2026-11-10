//! Strict outcrop source loading and locally resolved family references.
use super::{
    index::Index, sources, Catalog, CatalogError, ClimateBand, IdentityDef, ItemArt, OutcropDef,
    SpeciesOrigin,
};
use crate::{colony::knowledge::ContentStatus, material::TerrainType};
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema_version: u32,
    outcrops: Vec<Raw>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Origin {
    climate: Vec<ClimateBand>,
    terrain: Vec<TerrainType>,
    elevation: Option<(u8, u8)>,
    per_world: (u16, u16),
    cluster: (u16, u16),
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    identity: IdentityDef,
    category: String,
    tags: Vec<String>,
    status: ContentStatus,
    debt: Option<String>,
    origin: Origin,
    presence_permille: u16,
    camp_radius: u32,
    units: u16,
    material: String,
    yield_item: String,
    knowledge: String,
    quarry_knowledge: String,
    reveal_radius: u32,
    quarry_radius: u32,
    art: Option<ItemArt>,
    glyph: String,
}
fn parse(map: &BTreeMap<String, Vec<u8>>) -> Result<Vec<Raw>, CatalogError> {
    let file: File = ron::from_str(sources::text(map, "outcrops.ron")?)
        .map_err(|e| CatalogError::new("outcrops", e.to_string()))?;
    if file.schema_version != 1 {
        return Err(CatalogError::new("outcrops", "unsupported schema"));
    }
    Ok(file.outcrops)
}
pub(super) fn load(
    catalog: &mut Catalog,
    base: &BTreeMap<String, Vec<u8>>,
    extension: &BTreeMap<String, Vec<u8>>,
) -> Result<(), CatalogError> {
    let mut rows = parse(base)?;
    if extension.contains_key("outcrops.ron") {
        rows.extend(parse(extension)?);
    }
    rows.sort_by_key(|r| r.identity.code);
    let index = Index::build("outcrops", rows.iter().map(|r| &r.identity))?;
    let mut defs = Vec::new();
    for row in rows {
        let error = |message| CatalogError::new(&row.identity.id, message);
        let origin = SpeciesOrigin {
            climate: row.origin.climate,
            terrain: row.origin.terrain,
            elevation: row.origin.elevation,
        };
        if !origin.valid()
            || row.category.is_empty()
            || row.tags.iter().any(String::is_empty)
            || row.tags.iter().collect::<BTreeSet<_>>().len() != row.tags.len()
            || row.origin.per_world.0 > row.origin.per_world.1
            || row.origin.per_world.1 > 64
            || row.origin.cluster.0 == 0
            || row.origin.cluster.0 > row.origin.cluster.1
            || row.origin.cluster.1 > 9
            || row.units == 0
            || row.presence_permille > 1000
            || row.camp_radius == 0
            || row.reveal_radius == 0
            || row.reveal_radius > row.camp_radius
            || row.quarry_radius == 0
            || row.quarry_radius > row.camp_radius
            || row.glyph.is_empty()
            || row.glyph.chars().count() > 4
            || (row.status == ContentStatus::Inert && row.debt.as_deref().is_none_or(str::is_empty))
        {
            return Err(error("invalid outcrop habitat, counts or discovery policy"));
        }
        let material = super::validate::material(catalog, &row.material)?;
        let yield_item = catalog
            .item_by_name(&row.yield_item)
            .ok_or_else(|| error("unknown ore output"))?;
        let knowledge = catalog
            .knowledge_defs()
            .id_of(&row.knowledge)
            .ok_or_else(|| error("unknown discovery knowledge"))?;
        let quarry_knowledge = catalog
            .knowledge_defs()
            .id_of(&row.quarry_knowledge)
            .ok_or_else(|| error("unknown quarry knowledge"))?;
        let art = row.art.unwrap_or_else(|| {
            let mut art = ItemArt::for_category(&row.category);
            art.tint_material = row.material.clone();
            art
        });
        super::validate::material(catalog, &art.tint_material)?;
        defs.push(OutcropDef {
            identity: row.identity,
            category: row.category,
            tags: row.tags,
            status: row.status,
            debt: row.debt,
            origin,
            per_world: row.origin.per_world,
            cluster: row.origin.cluster,
            presence_permille: row.presence_permille,
            camp_radius: row.camp_radius,
            units: row.units,
            material,
            material_name: row.material,
            yield_item,
            knowledge,
            quarry_knowledge,
            reveal_radius: row.reveal_radius,
            quarry_radius: row.quarry_radius,
            art,
            glyph: row.glyph,
        });
    }
    catalog.outcrop_index = index;
    catalog.outcrops = defs;
    Ok(())
}
