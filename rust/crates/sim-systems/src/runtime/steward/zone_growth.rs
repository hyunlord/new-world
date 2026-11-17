//! Daily data-driven village layout before ordinary steward requests, including replay.
//! Reads: people, diagnosis facts, navigation, existing layout and physical contents.
//! Writes: building layout tiles and causes, shared zone allocator, Chronicle.
//! Events: ZoneLaidOut. No random draws or item creation.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::building_site::ZoneCause;
use sim_core::colony::zones::{self, Adjacency, ZoneBlocker, ZoneRule, ZoneUse};
use sim_core::components::Agent;
use sim_engine::SimResources;
use std::collections::BTreeSet;
type Tile = (u32, u32);

pub(super) fn maybe_grow_zones(world: &mut World, res: &mut SimResources) {
    if !res.steward.enabled
        || res.rules.colony.is_none()
        || res.ticks_per_day == 0
        || !res.current_tick.is_multiple_of(res.ticks_per_day)
    {
        return;
    }
    let people = u32::try_from(world.query::<&Agent>().iter().count()).unwrap_or(u32::MAX);
    for rule in zones::installed().connected() {
        if !res.steward.scope.permits(rule.scope) {
            continue;
        }
        let requested =
            crate::runtime::diagnosis::settlement_fact(world, res, &rule.trigger).unwrap_or(0);
        // Bootstrap: seed the first layout zone on the initial day before diagnosis has
        // observed demand, so planning actions can place buildings/stockpiles on the
        // same tick.  Storage is gated on StewardScope::All so a Housing-only steward
        // does not open storage eagerly.  Housing bootstraps unconditionally so hut
        // orders land inside the zone from tick 0 regardless of scope.
        use sim_core::colony::steward::StewardScope;
        let bootstrap = (rule.category == ZoneUse::Storage
            && res.steward.scope != StewardScope::Housing)
            || (rule.category == ZoneUse::Housing
                && res.colony.buildings.housing_zones.is_empty());
        if (requested <= 0 && !bootstrap) || has_free_slot(res, rule) {
            continue;
        }
        let count = zone_tiles(res, rule).count();
        if count
            >= usize::try_from(people.div_ceil(rule.people_per_zone).max(1)).unwrap_or(usize::MAX)
        {
            continue;
        }
        if let Some(tiles) = find_zone_site(res, rule) {
            lay_out_zone(world, res, rule, tiles, people);
        }
    }
}

fn zone_tiles<'a>(
    res: &'a SimResources,
    rule: &'a ZoneRule,
) -> impl Iterator<Item = &'a BTreeSet<Tile>> {
    res.colony
        .buildings
        .housing_zones
        .iter()
        .chain(res.colony.buildings.layout_zones.iter())
        .filter(move |(id, _)| {
            res.colony
                .buildings
                .zone_causes
                .get(id)
                .map_or(rule.category == ZoneUse::Housing, |cause| {
                    cause.kind == rule.kind
                })
        })
        .map(|(_, tiles)| tiles)
}

/// Ordinary autonomous placement stays inside its category's available layout.
pub(super) fn allows(res: &SimResources, category: ZoneUse, anchor: Tile, size: Tile) -> bool {
    if !res.steward.enabled
        || !zones::installed()
            .connected()
            .any(|rule| rule.category == category)
    {
        return true;
    }
    zones::installed()
        .connected()
        .filter(|rule| rule.category == category)
        .any(|rule| {
            zone_tiles(res, rule)
                .any(|tiles| rectangle(anchor, size).all(|tile| tiles.contains(&tile)))
        })
}

/// Stockpiles occupy a storehouse interior, leaving its one-tile physical shell legal.
pub(super) fn allows_stockpile(res: &SimResources, anchor: Tile, size: Tile) -> bool {
    if !res.steward.enabled
        || !zones::installed()
            .connected()
            .any(|rule| rule.category == ZoneUse::Storage)
    {
        return true;
    }
    let Some(x) = anchor.0.checked_sub(1) else {
        return false;
    };
    let Some(y) = anchor.1.checked_sub(1) else {
        return false;
    };
    let Some(width) = size.0.checked_add(2) else {
        return false;
    };
    let Some(height) = size.1.checked_add(2) else {
        return false;
    };
    allows(res, ZoneUse::Storage, (x, y), (width, height))
}

/// Connected storage starts with a fitting interior; legacy placement keeps its full request.
pub(super) fn stockpile_sides(res: &SimResources, requested: u32) -> std::ops::RangeInclusive<u32> {
    if !res.steward.enabled
        || !zones::installed()
            .connected()
            .any(|rule| rule.category == ZoneUse::Storage)
    {
        return requested..=requested;
    }
    // Any square and its shell require at least (side + 2)^2 layout tiles.
    // This is only an upper bound: placement still checks the actual geometry.
    let maximum = zones::installed()
        .connected()
        .filter(|rule| rule.category == ZoneUse::Storage)
        .flat_map(|rule| zone_tiles(res, rule))
        .map(|tiles| tiles.len().isqrt().saturating_sub(2))
        .max()
        .unwrap_or(0);
    1..=requested.min(u32::try_from(maximum).unwrap_or(u32::MAX))
}

fn has_free_slot(res: &SimResources, rule: &ZoneRule) -> bool {
    zone_tiles(res, rule).any(|tiles| {
        tiles.iter().any(|&anchor| {
            (rule.category != ZoneUse::Storage || allows_stockpile(res, anchor, rule.slot_size))
                && rectangle(anchor, rule.slot_size).all(|tile| {
                    tiles.contains(&tile)
                        && !blocked(res, rule, tile)
                        && (rule.category != ZoneUse::Storage
                            || (!res.colony.zone_of_tile.contains_key(&tile)
                                && !res.colony.ground_items.contains_key(&tile)))
                })
        })
    })
}

fn rectangle(anchor: Tile, size: Tile) -> impl Iterator<Item = Tile> {
    (anchor.1..anchor.1.saturating_add(size.1))
        .flat_map(move |y| (anchor.0..anchor.0.saturating_add(size.0)).map(move |x| (x, y)))
}

fn blocked(res: &SimResources, rule: &ZoneRule, tile: Tile) -> bool {
    blocked_for_layout(res, rule, tile, false)
}

fn blocked_for_layout(
    res: &SimResources,
    rule: &ZoneRule,
    tile: Tile,
    prepare_trees: bool,
) -> bool {
    !crate::runtime::nav::passable(res, tile)
        || !crate::commands::camp_reaches(res, tile)
        || rule.avoid.iter().any(|blocker| match blocker {
            ZoneBlocker::Trees => !prepare_trees && res.colony.trees.contains_key(&tile),
            ZoneBlocker::Nodes => res.colony.nodes.contains_key(&tile),
            ZoneBlocker::Fields => res.cultivation.fields.contains_key(&tile),
            ZoneBlocker::Structures => {
                res.colony.occupied.contains_key(&tile)
                    || res.colony.space_sites.contains_key(&tile)
                    || res.colony.buildings.sites.values().any(|site| {
                        site.status != sim_core::colony::building_site::BuildingStatus::Cancelled
                            && tile.0 >= site.anchor.0
                            && tile.1 >= site.anchor.1
                            && tile.0 < site.anchor.0.saturating_add(site.footprint.0)
                            && tile.1 < site.anchor.1.saturating_add(site.footprint.1)
                    })
            }
            ZoneBlocker::Outcrops => res.ore.deposits.contains_key(&tile),
        })
}

fn distance(a: Tile, b: Tile) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

fn edge_distance(tile: Tile, anchor: Tile, size: Tile) -> u32 {
    let right = anchor.0.saturating_add(size.0).saturating_sub(1);
    let bottom = anchor.1.saturating_add(size.1).saturating_sub(1);
    distance(
        tile,
        (
            tile.0.clamp(anchor.0, right),
            tile.1.clamp(anchor.1, bottom),
        ),
    )
}

fn find_zone_site(res: &SimResources, rule: &ZoneRule) -> Option<BTreeSet<Tile>> {
    find_zone_site_for_preparation(res, rule, false)
}

/// Ordinary stockpile demand may prepare trees, but only workers can make the layout admissible.
pub(super) fn prepare_stockpile_zone(
    world: &World,
    res: &SimResources,
) -> Vec<sim_core::colony::Command> {
    if !res.steward.enabled {
        return Vec::new();
    }
    let people = u32::try_from(world.query::<&Agent>().iter().count()).unwrap_or(u32::MAX);
    for rule in zones::installed()
        .connected()
        .filter(|rule| rule.category == ZoneUse::Storage)
    {
        if !res.steward.scope.permits(rule.scope)
            || has_free_slot(res, rule)
            || zone_tiles(res, rule).count()
                >= usize::try_from(people.div_ceil(rule.people_per_zone).max(1))
                    .unwrap_or(usize::MAX)
            || find_zone_site(res, rule).is_some()
        {
            continue;
        }
        if let Some(site) = find_zone_site_for_preparation(res, rule, true) {
            let trees: Vec<_> = site
                .into_iter()
                .filter(|tile| {
                    res.colony.trees.contains_key(tile)
                        && !res.colony.designations.contains_key(tile)
                })
                .collect();
            return if trees.is_empty() {
                Vec::new()
            } else {
                vec![sim_core::colony::Command::DesignateChopTiles { tiles: trees }]
            };
        }
    }
    Vec::new()
}

fn find_zone_site_for_preparation(
    res: &SimResources,
    rule: &ZoneRule,
    prepare_trees: bool,
) -> Option<BTreeSet<Tile>> {
    let mut village: BTreeSet<_> = res
        .colony
        .buildings
        .housing_zones
        .values()
        .chain(res.colony.buildings.layout_zones.values())
        .flat_map(|tiles| tiles.iter().copied())
        .chain(res.colony.occupied.keys().copied())
        .collect();
    let hearth = crate::runtime::jobs::hearth::hearth_tiles(res)
        .first()
        .copied()?;
    village.insert(hearth);
    let existing: BTreeSet<_> = res
        .colony
        .buildings
        .housing_zones
        .values()
        .chain(res.colony.buildings.layout_zones.values())
        .flat_map(|tiles| tiles.iter().copied())
        .chain(res.colony.zone_of_tile.keys().copied())
        .collect();
    let mut candidates: Vec<_> = (1..res.tile_grid.height.saturating_sub(rule.size.1))
        .flat_map(|y| (1..res.tile_grid.width.saturating_sub(rule.size.0)).map(move |x| (x, y)))
        .collect();
    candidates.sort_by_key(|&tile| (distance(tile, hearth), tile.1, tile.0));
    candidates.into_iter().find_map(|anchor| {
        let tiles: BTreeSet<_> = rectangle(anchor, rule.size).collect();
        if tiles.iter().any(|&tile| {
            blocked_for_layout(res, rule, tile, prepare_trees)
                || existing.contains(&tile)
                || (prepare_trees
                    && res.colony.trees.contains_key(&tile)
                    && res.steward.vetoed(
                        &sim_core::colony::steward::Target::Tile(tile.0, tile.1),
                        res.current_tick,
                    ))
        }) || existing
            .iter()
            .any(|&tile| edge_distance(tile, anchor, rule.size) <= rule.min_spacing)
        {
            return None;
        }
        let adjacent = match rule.adjacency {
            Adjacency::NextToZoneOrHearth => village.iter().any(|&tile| {
                let gap = edge_distance(tile, anchor, rule.size);
                gap > 0 && gap <= rule.max_gap.saturating_add(1)
            }),
        };
        if !adjacent {
            return None;
        }
        // Each category can bootstrap at the hearth before people have worn a path.
        // Further growth must follow the authored minimum path stage.
        if rule.min_path_stage > 0 && zone_tiles(res, rule).next().is_some() {
            let near_path = rectangle(
                (anchor.0.saturating_sub(1), anchor.1.saturating_sub(1)),
                (rule.size.0 + 2, rule.size.1 + 2),
            )
            .any(|tile| {
                !tiles.contains(&tile)
                    && tile.0 < res.tile_grid.width
                    && tile.1 < res.tile_grid.height
                    && res
                        .colony
                        .path_stage
                        .get((tile.1 * res.tile_grid.width + tile.0) as usize)
                        .is_some_and(|&stage| stage >= rule.min_path_stage)
            });
            if !near_path {
                return None;
            }
        }
        Some(tiles)
    })
}

fn lay_out_zone(
    world: &mut World,
    res: &mut SimResources,
    rule: &ZoneRule,
    tiles: BTreeSet<Tile>,
    people: u32,
) {
    let Some(next) = res.colony.next_zone_id.checked_add(1) else {
        return;
    };
    let zone_id = res.colony.next_zone_id;
    res.colony.next_zone_id = next;
    let slots = match rule.category {
        ZoneUse::Housing => super::planning::covered_beds(res),
        ZoneUse::Storage => res.colony.zone_of_tile.len().try_into().unwrap_or(u32::MAX),
        ZoneUse::Workshop => res.colony.workshops.len().try_into().unwrap_or(u32::MAX),
        ZoneUse::Field => res.cultivation.fields.len().try_into().unwrap_or(u32::MAX),
        ZoneUse::Pen => {
            let catalog = sim_core::colony::content::installed_or_bundled();
            res.colony
                .buildings
                .sites
                .values()
                .filter(|site| {
                    site.status != sim_core::colony::building_site::BuildingStatus::Cancelled
                })
                .filter_map(|site| catalog.building_templates().get(&site.template))
                .fold(0u32, |total, template| {
                    total.saturating_add(u32::from(template.function.livestock_capacity))
                })
        }
    };
    let map = match rule.category {
        ZoneUse::Housing => &mut res.colony.buildings.housing_zones,
        ZoneUse::Storage | ZoneUse::Workshop | ZoneUse::Field | ZoneUse::Pen => {
            &mut res.colony.buildings.layout_zones
        }
    };
    map.insert(zone_id, tiles);
    res.colony.buildings.zone_causes.insert(
        zone_id,
        ZoneCause {
            kind: rule.kind.clone(),
            trigger: rule.trigger.clone(),
            people,
            slots,
        },
    );
    let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::ZoneLaidOut);
    entry.value = (i64::from(people) << 32) | i64::from(slots);
    entry.cause = zone_id;
    entry.settlement = res.settlements.keys().copied().min();
    crate::runtime::chronicle_log::record(world, res, entry);
    res.colony.touch(false);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::building_site::{BuildCause, BuildingSite, BuildingStatus};
    use sim_core::colony::steward::StewardScope;

    fn fixture() -> (World, SimResources) {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled content");
        let mut res = SimResources::new(48, 48, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        res.ticks_per_day = res.rules.calendar.ticks_per_day;
        res.tile_grid
            .terrain_type
            .fill(sim_core::TerrainType::Plain);
        res.colony.path_stage.resize(48 * 48, 0);
        res.steward.enabled = true;
        res.steward.scope = StewardScope::Housing;
        let mut settlement = sim_core::components::Settlement::new_with_id(0, 0);
        settlement.formation_tile = (24, 24);
        let mut world = World::new();
        for id in 0..8 {
            world.spawn((Agent { id },));
            settlement.member_agents.insert(id);
        }
        res.settlements.insert(0, settlement);
        (world, res)
    }

    #[test]
    fn storage_preparation_preserves_physical_guards_vetoes_and_caps() {
        use sim_core::colony::steward::Target;
        use sim_core::colony::{Command, Designation, NodeKind, ResourceNode, Tree};
        let (mut world, mut res) = fixture();
        res.steward.scope = StewardScope::All;
        let rule = zones::installed()
            .connected()
            .find(|rule| rule.category == ZoneUse::Storage)
            .expect("storage");
        assert!(
            prepare_stockpile_zone(&world, &res).is_empty(),
            "a clear site needs no preparation"
        );
        for y in 0..res.tile_grid.height {
            for x in 0..res.tile_grid.width {
                res.colony.trees.insert((x, y), Tree { logs: 1 });
            }
        }
        assert!(find_zone_site(&res, rule).is_none());
        let orders = prepare_stockpile_zone(&world, &res);
        let [Command::DesignateChopTiles { tiles }] = orders.as_slice() else {
            panic!("ordinary tree preparation")
        };
        assert!(!tiles.is_empty());
        assert!(
            res.colony.buildings.layout_zones.is_empty(),
            "preparation never lays out a blocked site"
        );
        for &tile in tiles {
            res.colony.designations.insert(tile, Designation::Chop);
        }
        assert!(
            prepare_stockpile_zone(&world, &res).is_empty(),
            "wait for already designated trees"
        );
        res.colony.designations.clear();
        for &(x, y) in res.colony.trees.keys() {
            res.steward.vetoes.insert(Target::Tile(x, y), 100);
        }
        assert!(prepare_stockpile_zone(&world, &res).is_empty());
        res.steward.vetoes.clear();
        res.steward.enabled = false;
        assert!(prepare_stockpile_zone(&world, &res).is_empty());
        res.steward.enabled = true;
        res.steward.scope = StewardScope::Housing;
        assert!(
            !prepare_stockpile_zone(&world, &res).is_empty(),
            "All-family storage remains shared material supply in Housing scope"
        );
        res.steward.scope = StewardScope::All;
        for &tile in res.colony.trees.keys() {
            res.colony.nodes.insert(
                tile,
                ResourceNode {
                    kind: NodeKind::Boulder,
                    left: 1,
                    material: sim_core::material::MaterialId::from_str_hash("test_blocker"),
                },
            );
        }
        assert!(
            prepare_stockpile_zone(&world, &res).is_empty(),
            "nodes remain mandatory physical blockers"
        );
        res.colony.nodes.clear();
        let site = find_zone_site_for_preparation(&res, rule, true).expect("wooded site");
        lay_out_zone(&mut world, &mut res, rule, site, 8);
        assert!(
            prepare_stockpile_zone(&world, &res).is_empty(),
            "preparation cannot exceed the authored population cap"
        );
    }

    #[test]
    fn seed_five_storage_bootstrap_physically_clears_its_layout() {
        use sim_core::colony::Command;
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled content");
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(5),
            sim_core::rules::WorldRules::colony(5),
        );
        assert!(
            crate::commands::apply_command(
                &mut engine.world,
                &mut engine.resources,
                Command::SetSteward {
                    on: true,
                    scope: StewardScope::All
                }
            )
            .accepted
        );
        while engine.resources.colony.zone_of_tile.is_empty()
            && engine.current_tick() < 6 * engine.resources.ticks_per_day
        {
            engine.tick();
        }
        let prepared: Vec<_> = engine
            .resources
            .colony
            .command_log
            .iter()
            .filter_map(|record| match &record.command {
                Command::Steward { rule: 1, command } => match command.as_ref() {
                    Command::DesignateChopTiles { tiles } => Some(tiles),
                    _ => None,
                },
                _ => None,
            })
            .flatten()
            .copied()
            .collect();
        assert!(
            !prepared.is_empty(),
            "ordinary stockpile demand prepares its site"
        );
        assert!(
            prepared
                .iter()
                .all(|tile| !engine.resources.colony.trees.contains_key(tile)),
            "workers physically cleared the planned trees"
        );
        assert!(
            !engine.resources.colony.zone_of_tile.is_empty(),
            "ordinary seed5 storage bootstraps after preparation"
        );
        assert!(engine
            .resources
            .colony
            .zone_of_tile
            .keys()
            .all(|&tile| allows_stockpile(&engine.resources, tile, (1, 1))));
    }

    #[test]
    fn daily_zone_growth_is_scoped_capped_and_replay_stable() {
        let (mut world, mut res) = fixture();
        res.steward.enabled = false;
        maybe_grow_zones(&mut world, &mut res);
        assert!(res.colony.buildings.housing_zones.is_empty());
        res.steward.enabled = true;
        res.current_tick = 1;
        maybe_grow_zones(&mut world, &mut res);
        assert!(res.colony.buildings.housing_zones.is_empty());
        res.current_tick = 0;
        maybe_grow_zones(&mut world, &mut res);
        assert_eq!(res.colony.buildings.housing_zones.len(), 1);
        assert!(res.colony.buildings.layout_zones.is_empty());
        let (mut replay_world, mut replay) = fixture();
        replay.steward.replay = true;
        maybe_grow_zones(&mut replay_world, &mut replay);
        assert_eq!(res.colony.buildings, replay.colony.buildings);
        let tiles = res
            .colony
            .buildings
            .housing_zones
            .values()
            .next()
            .expect("housing")
            .clone();
        assert_eq!(
            tiles.len(),
            81,
            "four two-bed dwellings fit the eight-person design cap"
        );
        for tile in tiles {
            res.colony
                .occupied
                .insert(tile, sim_core::colony::Occupant::Hut(tile));
        }
        res.colony.path_stage.fill(1);
        res.current_tick = res.ticks_per_day;
        maybe_grow_zones(&mut world, &mut res);
        assert_eq!(
            res.colony.buildings.housing_zones.len(),
            1,
            "population cap"
        );
        world.spawn((
            Agent { id: 8 },
            sim_core::components::Child { born_tick: 0 },
        ));
        res.current_tick += res.ticks_per_day;
        maybe_grow_zones(&mut world, &mut res);
        assert_eq!(res.colony.buildings.housing_zones.len(), 2);
    }

    #[test]
    fn later_zones_follow_paths_and_leave_the_authored_gap() {
        let (mut world, mut res) = fixture();
        let rule = zones::installed()
            .connected()
            .find(|rule| rule.category == ZoneUse::Housing)
            .expect("housing");
        let first = find_zone_site(&res, rule).expect("bootstrap without paths");
        lay_out_zone(&mut world, &mut res, rule, first.clone(), 8);
        assert!(
            find_zone_site(&res, rule).is_none(),
            "later zone requires a path"
        );
        res.colony.path_stage.fill(1);
        let next = find_zone_site(&res, rule).expect("growth along trodden path");
        let nearest = first
            .iter()
            .flat_map(|&a| next.iter().map(move |&b| distance(a, b)))
            .min()
            .expect("tiles");
        assert_eq!(nearest, rule.min_spacing + 1);
        assert!(next.is_disjoint(&first));
    }

    #[test]
    fn physical_guards_include_whole_pen_interior_and_cancelled_sites_release_it() {
        let (_, mut res) = fixture();
        let rule = zones::installed()
            .connected()
            .find(|rule| rule.category == ZoneUse::Workshop)
            .expect("rule");
        let tile = (12, 12);
        assert!(!blocked(&res, rule, tile));
        let index = res.tile_grid.idx(tile.0, tile.1);
        res.tile_grid.terrain_type[index] = sim_core::TerrainType::River;
        assert!(blocked(&res, rule, tile));
        res.tile_grid.terrain_type[index] = sim_core::TerrainType::Plain;
        res.colony
            .trees
            .insert(tile, sim_core::colony::Tree { logs: 1 });
        assert!(blocked(&res, rule, tile));
        res.colony.trees.clear();
        let catalog = sim_core::colony::content::installed_or_bundled();
        res.cultivation.fields.insert(
            tile,
            sim_core::colony::cultivation::FieldTile {
                owner: 0,
                crop: catalog.crop_ids().next().expect("crop"),
                phase: sim_core::colony::cultivation::FieldPhase::Untilled,
                active: true,
                work: sim_core::colony::cultivation::FieldWork::Auto,
                tended: false,
            },
        );
        assert!(blocked(&res, rule, tile));
        res.cultivation.fields.clear();
        res.ore.deposits.insert(
            tile,
            sim_core::colony::ore::OreDeposit {
                kind: catalog.outcrop_ids().next().expect("outcrop"),
                remaining: 1,
                revealed: false,
                cluster: tile,
            },
        );
        assert!(blocked(&res, rule, tile));
        res.ore.deposits.clear();
        res.colony.buildings.sites.insert(
            1,
            BuildingSite {
                id: 1,
                template: "goat_pen".into(),
                station: None,
                anchor: (10, 10),
                rotation: 0,
                footprint: (5, 5),
                pieces: Vec::new(),
                cause: BuildCause::Command,
                source_record: 0,
                placed_tick: 0,
                status: BuildingStatus::Building,
                completion_entry: None,
            },
        );
        assert!(blocked(&res, rule, tile), "empty pen interior is reserved");
        res.colony.buildings.sites.get_mut(&1).expect("site").status = BuildingStatus::Cancelled;
        assert!(!blocked(&res, rule, tile));
    }

    #[test]
    fn storage_layout_space_is_consumed_by_physical_stockpiles() {
        let (mut world, mut res) = fixture();
        let base_rule = zones::installed()
            .connected()
            .find(|rule| rule.category == ZoneUse::Storage)
            .expect("storage");
        for (size, slot_size, capacity) in [
            (base_rule.size, base_rule.slot_size, 16),
            ((3, 3), (1, 1), 1),
        ] {
            res.colony.buildings.layout_zones.clear();
            res.colony.buildings.zone_causes.clear();
            res.colony.zone_of_tile.clear();
            let mut rule = base_rule.clone();
            rule.size = size;
            rule.slot_size = slot_size;
            let tiles = find_zone_site(&res, &rule).expect("storage site");
            lay_out_zone(&mut world, &mut res, &rule, tiles.clone(), 8);
            assert!(has_free_slot(&res, &rule));
            let interior: Vec<_> = tiles
                .iter()
                .copied()
                .filter(|&tile| allows_stockpile(&res, tile, (1, 1)))
                .collect();
            assert_eq!(interior.len(), capacity);
            for tile in interior {
                res.colony.zone_of_tile.insert(tile, 55);
            }
            assert!(
                !has_free_slot(&res, &rule),
                "empty wall border cannot suppress layout expansion"
            );
            assert!(
                tiles.len() > res.colony.zone_of_tile.len(),
                "the shell stays unoccupied"
            );
        }
        assert!(!allows(&res, ZoneUse::Housing, (1, 1), (3, 3)));
    }

    #[test]
    fn missing_workshop_demand_bootstraps_a_real_blueprint_inside_its_layout() {
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(0),
            sim_core::rules::WorldRules::colony(0),
        );
        engine.resources.steward.enabled = true;
        engine.resources.steward.scope = StewardScope::All;
        // Isolate the missing-station prerequisite from the world's seeded station.
        engine.resources.colony.workshops.clear();
        engine
            .resources
            .colony
            .occupied
            .retain(|_, occupant| !matches!(occupant, sim_core::colony::Occupant::Workshop(_)));
        assert!(super::super::planning::workshop_space_needed(
            &engine.world,
            &engine.resources
        ));
        assert!(engine.resources.colony.workshops.is_empty());
        let rules = sim_core::colony::steward::rules().expect("rules");
        assert!(
            !rules
                .rules
                .iter()
                .filter(|rule| matches!(
                    rule.action,
                    sim_core::colony::steward::Action::Bills { .. }
                ))
                .flat_map(|rule| super::super::planning::plan(
                    &engine.world,
                    &engine.resources,
                    rule
                ))
                .any(|command| matches!(command, sim_core::colony::Command::PlaceBlueprint { .. })),
            "no workshop before layout"
        );
        maybe_grow_zones(&mut engine.world, &mut engine.resources);
        let command = rules
            .rules
            .iter()
            .filter(|rule| matches!(rule.action, sim_core::colony::steward::Action::Bills { .. }))
            .flat_map(|rule| super::super::planning::plan(&engine.world, &engine.resources, rule))
            .find(|command| matches!(command, sim_core::colony::Command::PlaceBlueprint { .. }))
            .expect("ordinary missing-workshop blueprint after layout");
        let sim_core::colony::Command::PlaceBlueprint { kind, x, y } = command else {
            unreachable!()
        };
        assert!(allows(
            &engine.resources,
            ZoneUse::Workshop,
            (x, y),
            kind.plan(&engine.resources.rules.colony.expect("colony")).0
        ));
        assert!(
            crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                .accepted
        );
    }

    #[test]
    fn spacing_and_adjacency_measure_empty_gap_from_rectangle() {
        let anchor = (4, 4);
        let size = (3, 3);
        assert_eq!(edge_distance((4, 4), anchor, size), 0);
        assert_eq!(edge_distance((3, 4), anchor, size), 1);
        assert_eq!(edge_distance((2, 4), anchor, size), 2);
        assert_eq!(edge_distance((8, 8), anchor, size), 2);
        assert_eq!(rectangle(anchor, size).count(), 9);
    }
}
