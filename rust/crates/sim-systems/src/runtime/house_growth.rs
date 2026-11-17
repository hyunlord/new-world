//! HouseGrowthSystem: data-authored dwelling residence and monotone yard stages.
//!
//! Priority: 146 · Interval: day boundary (dynamic calendar, checked each tick).
//! Reads: Agent, Skills, bed_of, physical buildings, stockpiles, selected yards data.
//! Writes: colony.buildings.house_growth, chronicle, overlay_version.
//! Events: HouseStage; accepted steward upgrades call note_upgrade for MaterialWave.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::building_site::{BuildingSite, BuildingStatus};
use sim_core::colony::building_template::BuildingCategory;
use sim_core::colony::content::{installed_or_bundled, Catalog};
use sim_core::colony::{ItemKind, ReserveKey, Skills, ZoneKind};
use sim_core::components::{Agent, BuildingKind, SettlementId};
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeMap;

/// Daily dwelling residence observation; no independent system state.
#[derive(Default)]
pub struct HouseGrowthSystem;

impl RuntimeSystem for HouseGrowthSystem {
    fn name(&self) -> &str {
        "HouseGrowthSystem"
    }
    fn priority(&self) -> u32 {
        146
    }
    // The actual day length is supplied by SimResources, not by the system trait.
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        if res.rules.colony.is_none()
            || res.current_tick == 0
            || res.ticks_per_day == 0
            || !res.current_tick.is_multiple_of(res.ticks_per_day)
        {
            return;
        }
        let catalog = installed_or_bundled();
        if catalog.yards().stages.is_empty() {
            return;
        }
        let residents: BTreeMap<_, _> = world
            .query::<(&Agent, Option<&Skills>)>()
            .iter()
            .map(|(_, (agent, skills))| {
                (
                    agent.id,
                    skills.map_or(0, |skills| {
                        skills
                            .entries
                            .iter()
                            .map(|skill| skill.level)
                            .max()
                            .unwrap_or(0)
                    }),
                )
            })
            .collect();
        let goods = stored_goods(res);
        let mut changes = Vec::new();
        for site in res.colony.buildings.sites.values() {
            if !dwelling(catalog, site) || !super::building_templates::is_intact(res, site) {
                continue;
            }
            let skill = res
                .bed_of
                .iter()
                .filter(|(_, bed)| contains(site, **bed))
                .filter_map(|(agent, _)| residents.get(agent))
                .copied()
                .max();
            let Some(skill) = skill else {
                continue;
            };
            let settlement = owner(res, site);
            let local_goods = settlement
                .and_then(|id| goods.get(&id))
                .copied()
                .unwrap_or(0);
            let progress = res
                .colony
                .buildings
                .house_growth
                .entry(site.id)
                .or_default();
            progress.occupied_days = progress.occupied_days.saturating_add(1);
            let stage = catalog
                .yards()
                .stage_for(progress.occupied_days, local_goods, skill);
            if stage > progress.stage {
                progress.stage = stage;
                changes.push((site.id, stage, settlement));
            }
        }
        for (id, stage, settlement) in changes {
            let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::HouseStage);
            entry.cause = id;
            entry.value = i64::from(stage);
            entry.settlement = settlement;
            super::chronicle_log::record(world, res, entry);
            res.colony.touch(false);
        }
    }
}

/// Whether an authored era has reached the data-defined material-wave threshold.
pub fn era_ready(res: &SimResources, catalog: &Catalog, era: &str) -> bool {
    let Some(era) = catalog
        .knowledge_defs()
        .eras
        .eras
        .iter()
        .find(|row| row.id == era)
    else {
        return false;
    };
    era.order == 0
        || (!catalog.yards().stages.is_empty()
            && res
                .colony
                .tech
                .era_progress
                .get(usize::from(era.order))
                .copied()
                .unwrap_or(0)
                >= catalog.yards().material_wave_permille)
}

/// First safe material-wave upgrade, ordered by site id; execute via an ordinary command.
pub fn upgrade_candidate(world: &World, res: &SimResources) -> Option<u32> {
    if res.rules.colony.is_none() || !res.steward.enabled {
        return None;
    }
    let catalog = installed_or_bundled();
    if catalog.yards().stages.is_empty() {
        return None;
    }
    // A settlement admits at most one dwelling generation at a time, including
    // an unfinished replacement whose beds cannot yet protect an occupied home.
    let mut active = BTreeMap::<SettlementId, usize>::new();
    for site in res
        .colony
        .buildings
        .sites
        .values()
        .filter(|site| dwelling(catalog, site) && site.status == BuildingStatus::Building)
    {
        if let Some(id) = owner(res, site) {
            *active.entry(id).or_default() += 1;
        }
    }
    let available = super::building_templates::materials::available_materials(world, res);
    for site in res.colony.buildings.sites.values() {
        if !dwelling(catalog, site) || !super::building_templates::is_intact(res, site) {
            continue;
        }
        let Some(settlement) = owner(res, site) else {
            continue;
        };
        if active.get(&settlement).copied().unwrap_or(0)
            >= usize::from(catalog.yards().max_simultaneous_upgrades)
        {
            continue;
        }
        let Some(target) = catalog
            .building_templates()
            .get(&site.template)
            .and_then(|template| template.upgrade_to.as_deref())
            .and_then(|id| catalog.building_templates().get(id))
        else {
            continue;
        };
        if !era_ready(res, catalog, &target.era) || !spare_beds(world, res, catalog, site) {
            continue;
        }
        let Ok(needs) =
            super::building_templates::upgrade::material_requirements(world, res, site.id)
        else {
            continue;
        };
        if supplies_ready(res, settlement, &needs, &available) {
            return Some(site.id);
        }
    }
    None
}

fn supplies_ready(
    res: &SimResources,
    settlement: SettlementId,
    needs: &BTreeMap<ItemKind, u32>,
    available: &BTreeMap<ItemKind, u32>,
) -> bool {
    let mut local = BTreeMap::<ItemKind, u32>::new();
    for (tile, stack) in &res.colony.ground_items {
        if tile_owner(res, *tile) == Some(settlement)
            && res
                .colony
                .zone_of_tile
                .get(tile)
                .and_then(|id| res.colony.zones.get(id))
                .is_some_and(|zone| zone.kind == ZoneKind::Stockpile)
            && !res
                .colony
                .reservations
                .contains_key(&ReserveKey::Stack(*tile))
        {
            let count = local.entry(stack.kind).or_default();
            *count = count.saturating_add(u32::from(stack.count));
        }
    }
    needs.iter().all(|(item, count)| {
        local.get(item).copied().unwrap_or(0) >= *count
            && available.get(item).copied().unwrap_or(0) >= *count
    })
}

/// Record only an accepted material-wave command, after its target template was installed.
pub fn note_upgrade(world: &mut World, res: &mut SimResources, id: u32) {
    let catalog = installed_or_bundled();
    let Some(site) = res.colony.buildings.sites.get(&id) else {
        return;
    };
    let Some(era) = catalog
        .building_templates()
        .get(&site.template)
        .and_then(|template| {
            catalog
                .knowledge_defs()
                .eras
                .eras
                .iter()
                .find(|era| era.id == template.era)
        })
    else {
        return;
    };
    let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::MaterialWave);
    entry.cause = id;
    entry.value = i64::from(era.order);
    entry.settlement = owner(res, site);
    super::chronicle_log::record(world, res, entry);
}

fn stored_goods(res: &SimResources) -> BTreeMap<SettlementId, u32> {
    let mut goods = BTreeMap::<SettlementId, u32>::new();
    for (tile, stack) in &res.colony.ground_items {
        if !res
            .colony
            .zone_of_tile
            .get(tile)
            .and_then(|id| res.colony.zones.get(id))
            .is_some_and(|zone| zone.kind == ZoneKind::Stockpile)
        {
            continue;
        }
        if let Some(id) = tile_owner(res, *tile) {
            let count = goods.entry(id).or_default();
            *count = count.saturating_add(u32::from(stack.count));
        }
    }
    goods
}

fn dwelling(catalog: &Catalog, site: &BuildingSite) -> bool {
    catalog
        .building_templates()
        .get(&site.template)
        .is_some_and(|template| template.category == BuildingCategory::Dwelling)
}

fn contains(site: &BuildingSite, tile: (u32, u32)) -> bool {
    tile.0 >= site.anchor.0
        && tile.1 >= site.anchor.1
        && tile.0 < site.anchor.0.saturating_add(site.footprint.0)
        && tile.1 < site.anchor.1.saturating_add(site.footprint.1)
}

fn owner(res: &SimResources, site: &BuildingSite) -> Option<SettlementId> {
    tile_owner(res, site.anchor)
}

fn tile_owner(res: &SimResources, tile: (u32, u32)) -> Option<SettlementId> {
    res.settlements
        .iter()
        .min_by_key(|(id, settlement)| {
            let home = settlement.formation_tile;
            (tile.0.abs_diff(home.0).max(tile.1.abs_diff(home.1)), **id)
        })
        .map(|(id, _)| *id)
}

fn spare_beds(world: &World, res: &SimResources, catalog: &Catalog, site: &BuildingSite) -> bool {
    if !res.bed_of.values().any(|bed| contains(site, *bed)) {
        return true;
    }
    let Some(owner_id) = owner(res, site) else {
        return false;
    };
    let Some(settlement) = res.settlements.get(&owner_id) else {
        return false;
    };
    let people = world
        .query::<&Agent>()
        .iter()
        .filter(|(_, agent)| settlement.member_agents.contains(&agent.id))
        .count();
    let beds: usize = res
        .colony
        .buildings
        .sites
        .values()
        .filter(|other| {
            other.id != site.id
                && owner(res, other) == Some(owner_id)
                && super::building_templates::is_intact(res, other)
        })
        .filter(|other| dwelling(catalog, other))
        .map(|other| {
            let reachable = |bed: (u32, u32)| {
                res.rules.colony.is_none_or(|rule| rule.space.is_none())
                    || res.nav.reachable(settlement.formation_tile, bed)
            };
            let bedrolls = res
                .colony
                .beds
                .keys()
                .filter(|&&bed| {
                    contains(other, bed) && tile_owner(res, bed) == Some(owner_id) && reachable(bed)
                })
                .count();
            let huts = res
                .building_kinds
                .iter()
                .filter(|(tile, kind)| {
                    **kind == BuildingKind::Hut
                        && contains(other, **tile)
                        && res.building_owner.get(tile) == Some(&owner_id)
                        && reachable(**tile)
                })
                .count();
            bedrolls + huts * res.rules.colony.map_or(0, |rule| rule.hut_beds as usize)
        })
        .sum();
    beds >= people
}

#[cfg(test)]
#[path = "house_growth_tests.rs"]
mod tests;
