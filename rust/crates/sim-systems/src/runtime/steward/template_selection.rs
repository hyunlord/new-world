//! Shared dwelling choice and exact material commitment accounting for house planners.
use hecs::World;
use sim_core::colony::building_template::{BuildingCategory, BuildingTemplate, PlannedPiece};
use sim_core::colony::content::{installed_or_bundled, Catalog};
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::{ItemKind, RejectReason};
use sim_engine::SimResources;
use std::collections::BTreeMap;
type Tile = (u32, u32);

pub(super) fn costs(
    res: &SimResources,
    plan: &[PlannedPiece],
) -> Result<BTreeMap<ItemKind, u32>, RejectReason> {
    crate::runtime::building_templates::commands::plan_materials(res, plan)
}

#[cfg(test)]
pub(super) fn available(world: &World, res: &SimResources, item: ItemKind) -> u32 {
    crate::runtime::building_templates::materials::available_materials(world, res)
        .get(&item)
        .copied()
        .unwrap_or(0)
}

pub(super) fn ranked<'a>(
    world: &World,
    res: &SimResources,
    catalog: &'a Catalog,
) -> Vec<&'a BuildingTemplate> {
    let available = crate::runtime::building_templates::materials::available_materials(world, res);
    let mut candidates: Vec<_> = catalog
        .building_templates()
        .buildings
        .iter()
        .filter(|template| {
            template.status == ContentStatus::Connected
                && template.category == BuildingCategory::Dwelling
                && crate::runtime::building_templates::commands::known(
                    world,
                    &template.requires_knowledge,
                    catalog,
                )
        })
        .filter(|template| {
            template.expand(catalog, None, 0).is_ok_and(|plan| {
                costs(res, &plan).is_ok_and(|needs| {
                    needs
                        .iter()
                        .all(|(&item, &count)| available.get(&item).copied().unwrap_or(0) >= count)
                })
            })
        })
        .collect();
    candidates.sort_by(|a, b| b.tier.cmp(&a.tier).then_with(|| a.id.cmp(&b.id)));
    candidates
}

pub(super) fn select<'a>(
    world: &World,
    res: &SimResources,
    catalog: &'a Catalog,
) -> Option<&'a BuildingTemplate> {
    ranked(world, res, catalog).into_iter().next()
}

pub(super) fn inside_housing(res: &SimResources, anchor: Tile, size: Tile) -> bool {
    if res.colony.buildings.housing_zones.is_empty() {
        return true;
    }
    let Some(right) = anchor.0.checked_add(size.0) else {
        return false;
    };
    let Some(bottom) = anchor.1.checked_add(size.1) else {
        return false;
    };
    res.colony
        .buildings
        .housing_zones
        .values()
        .any(|zone| (anchor.1..bottom).all(|y| (anchor.0..right).all(|x| zone.contains(&(x, y)))))
}

pub(super) fn pending_logs(
    world: &World,
    res: &SimResources,
    log_items: &[ItemKind],
) -> Option<u32> {
    let catalog = installed_or_bundled();
    let template = select(world, res, catalog)?;
    let plan = template.expand(catalog, None, 0).ok()?;
    Some(
        costs(res, &plan)
            .ok()?
            .iter()
            .filter(|(item, _)| log_items.contains(item))
            .map(|(_, count)| *count)
            .sum(),
    )
}
