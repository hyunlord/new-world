//! Player orders (M1, ADR 0012): `apply_command` is the only path that changes
//! designations, zones and blueprints; `submit` also records the command.
//!
//! A command arrives between ticks: the bridge applies it at once and logs it
//! with the engine's next tick (`SimEngine::current_tick`), so a replay that
//! applies every logged command right before that tick's systems reproduces
//! the run exactly, paused or not. Every command is rejected
//! (`RejectReason::NoColony`) without `rules.colony`.
//!
//! Rectangles are inclusive corners in any order, clipped to the map; a
//! rectangle wholly outside the map is `OutOfMap`, and a valid one that
//! changes nothing is `Nothing`.
use hecs::{Entity, World};
use sim_core::colony::{
    BlueprintKind, CancelReason, ColonySite, Command, CommandRecord, CommandResult, Designation,
    ItemKind, Job, JobKind, MaterialNeeds, Occupant, RejectReason, Zone, ZoneKind,
};
use sim_core::components::{
    Agent, AgentId, BuildingBlueprint, ConstructionSite, Position, HUT_BLUEPRINT_ID,
};
use sim_core::TerrainType;
use sim_engine::{SimEngine, SimResources};
use std::collections::{BTreeMap, BTreeSet};

use crate::runtime::jobs::cancel;
use crate::runtime::jobs::items::{put_items, terrain, under_building};

type Tile = (u32, u32);

/// The tiles of the rectangle with corners `(x0, y0)`, `(x1, y1)` that lie on
/// the map, in `(y, x)` order; `None` when none does.
fn rect_tiles(res: &SimResources, x0: u32, y0: u32, x1: u32, y1: u32) -> Option<Vec<Tile>> {
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let (lx, hx) = (x0.min(x1), x0.max(x1).min(w.saturating_sub(1)));
    let (ly, hy) = (y0.min(y1), y0.max(y1).min(h.saturating_sub(1)));
    if lx >= w || ly >= h || w == 0 || h == 0 {
        return None;
    }
    Some(
        (ly..=hy)
            .flat_map(|y| (lx..=hx).map(move |x| (x, y)))
            .collect(),
    )
}

fn counted(count: usize) -> CommandResult {
    if count == 0 {
        CommandResult::rejected(RejectReason::Nothing)
    } else {
        CommandResult::ok(count as u32)
    }
}

/// Apply `cmd` to the world and return what it did (goal 6.3). Does not log;
/// see [`submit`].
pub fn apply_command(world: &mut World, res: &mut SimResources, cmd: Command) -> CommandResult {
    let result = apply_inner(world, res, cmd);
    if !result.accepted {
        res.colony.tallies.commands_rejected += 1;
    }
    result
}

fn apply_inner(world: &mut World, res: &mut SimResources, cmd: Command) -> CommandResult {
    let Some(rule) = res.rules.colony else {
        return CommandResult::rejected(RejectReason::NoColony);
    };
    match cmd {
        Command::DesignateChop { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => counted(designate(res, &tiles)),
        },
        Command::DesignateChopTiles { tiles } => {
            let (w, h) = (res.tile_grid.width, res.tile_grid.height);
            let inside: Vec<Tile> = tiles.into_iter().filter(|t| t.0 < w && t.1 < h).collect();
            if inside.is_empty() {
                return CommandResult::rejected(RejectReason::OutOfMap);
            }
            counted(designate(res, &inside))
        }
        Command::CancelArea { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => counted(cancel_area(world, res, &tiles)),
        },
        Command::AddStockpile { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => counted(add_stockpile(res, &tiles)),
        },
        Command::RemoveZoneArea { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => counted(remove_zone_area(world, res, &tiles)),
        },
        Command::PlaceBlueprint { kind, x, y } => match check_blueprint(res, kind, x, y) {
            Err(reason) => CommandResult::rejected(reason),
            Ok(()) => {
                place_blueprint(world, res, &rule, kind, x, y);
                CommandResult::ok(1)
            }
        },
        Command::SetDrafted { .. } => CommandResult::rejected(RejectReason::NotYet),
    }
}

/// Apply `cmd` before the engine's next tick and append it to the command log.
pub fn submit(engine: &mut SimEngine, cmd: Command) -> CommandResult {
    let tick = engine.current_tick();
    let result = apply_command(&mut engine.world, &mut engine.resources, cmd.clone());
    engine.resources.colony.command_log.push(CommandRecord {
        tick,
        command: cmd,
        result,
    });
    result
}

fn designate(res: &mut SimResources, tiles: &[Tile]) -> usize {
    let mut count = 0;
    for tile in tiles {
        if res.colony.trees.contains_key(tile) && !res.colony.designations.contains_key(tile) {
            res.colony.designations.insert(*tile, Designation::Chop);
            count += 1;
        }
    }
    if count > 0 {
        res.colony.touch(false);
    }
    count
}

/// Cancel, in `AgentId` order, the jobs of every agent whose job satisfies `hit`.
fn cancel_jobs_where(world: &mut World, res: &mut SimResources, hit: impl Fn(&Job) -> bool) {
    let mut victims: Vec<(AgentId, Entity)> = world
        .query::<(&Agent, &Job)>()
        .iter()
        .filter(|(_, (_, job))| hit(job))
        .map(|(entity, (agent, _))| (agent.id, entity))
        .collect();
    victims.sort_by_key(|v| v.0);
    for (_, entity) in victims {
        cancel(world, res, entity, CancelReason::Command);
    }
}

fn cancel_area(world: &mut World, res: &mut SimResources, tiles: &[Tile]) -> usize {
    let area: BTreeSet<Tile> = tiles.iter().copied().collect();
    // Chop marks.
    let marks: Vec<Tile> = res
        .colony
        .designations
        .keys()
        .copied()
        .filter(|t| area.contains(t))
        .collect();
    for tile in &marks {
        res.colony.designations.remove(tile);
    }
    let marked: BTreeSet<Tile> = marks.iter().copied().collect();
    cancel_jobs_where(
        world,
        res,
        |job| matches!(job.kind, JobKind::Chop { tree } if marked.contains(&tree)),
    );
    // Blueprint sites whose footprint touches the area.
    let mut sites: Vec<(Tile, Entity, ColonySite, MaterialNeeds)> = world
        .query::<(&ColonySite, &MaterialNeeds)>()
        .iter()
        .filter(|(_, (site, _))| site.tiles().iter().any(|t| area.contains(t)))
        .map(|(entity, (site, needs))| (site.centre(), entity, *site, *needs))
        .collect();
    sites.sort_by_key(|s| s.0);
    let removed: BTreeSet<Tile> = sites.iter().map(|s| s.0).collect();
    cancel_jobs_where(world, res, |job| match job.kind {
        JobKind::Deliver { site, .. } | JobKind::Build { site } => removed.contains(&site),
        _ => false,
    });
    for (centre, entity, site, needs) in &sites {
        for tile in site.tiles() {
            res.colony.occupied.remove(&tile);
        }
        let _ = world.despawn(*entity);
        if needs.delivered > 0 {
            put_items(res, *centre, needs.kind, needs.delivered);
        }
    }
    if !marks.is_empty() || !sites.is_empty() {
        res.colony.touch(false);
    }
    marks.len() + sites.len()
}

/// Whether `tile` may become a stockpile cell.
fn zoneable(res: &SimResources, tile: Tile) -> bool {
    matches!(terrain(res, tile), Some(t) if t != TerrainType::River && t != TerrainType::Mountain)
        && !under_building(res, tile)
        && !res.colony.zone_of_tile.contains_key(&tile)
}

fn add_stockpile(res: &mut SimResources, tiles: &[Tile]) -> usize {
    let mut fresh: BTreeSet<Tile> = tiles
        .iter()
        .copied()
        .filter(|t| zoneable(res, *t))
        .collect();
    let count = fresh.len();
    // 4-connected groups of new tiles become one new zone each, in (y, x) order of their first tile.
    let mut ordered: Vec<Tile> = fresh.iter().copied().collect();
    ordered.sort_by_key(|t| (t.1, t.0));
    for start in ordered {
        if !fresh.remove(&start) {
            continue;
        }
        let mut group = BTreeSet::from([start]);
        let mut frontier = vec![start];
        while let Some((x, y)) = frontier.pop() {
            let neighbours = [
                (x.wrapping_sub(1), y),
                (x + 1, y),
                (x, y.wrapping_sub(1)),
                (x, y + 1),
            ];
            for n in neighbours {
                if fresh.remove(&n) {
                    group.insert(n);
                    frontier.push(n);
                }
            }
        }
        let id = res.colony.next_zone_id;
        res.colony.next_zone_id += 1;
        for tile in &group {
            res.colony.zone_of_tile.insert(*tile, id);
        }
        res.colony.zones.insert(
            id,
            Zone {
                id,
                kind: ZoneKind::Stockpile,
                tiles: group,
            },
        );
    }
    if count > 0 {
        res.colony.touch(false);
    }
    count
}

fn remove_zone_area(world: &mut World, res: &mut SimResources, tiles: &[Tile]) -> usize {
    let mut removed: BTreeSet<Tile> = BTreeSet::new();
    for tile in tiles {
        if let Some(id) = res.colony.zone_of_tile.remove(tile) {
            removed.insert(*tile);
            if let Some(zone) = res.colony.zones.get_mut(&id) {
                zone.tiles.remove(tile);
                if zone.tiles.is_empty() {
                    res.colony.zones.remove(&id);
                }
            }
        }
    }
    cancel_jobs_where(
        world,
        res,
        |job| matches!(job.kind, JobKind::Haul { to, .. } if removed.contains(&to)),
    );
    if !removed.is_empty() {
        res.colony.touch(false);
    }
    removed.len()
}

/// The footprint of `kind` with its top-left at `(x, y)`, or `OutOfMap`.
fn footprint(
    res: &SimResources,
    kind: BlueprintKind,
    x: u32,
    y: u32,
) -> Result<ColonySite, RejectReason> {
    let size = match kind {
        BlueprintKind::Hut => res.rules.colony.map_or((3, 3), |rule| rule.hut_footprint),
    };
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    if x.checked_add(size.0).is_none_or(|end| end > w)
        || y.checked_add(size.1).is_none_or(|end| end > h)
    {
        return Err(RejectReason::OutOfMap);
    }
    Ok(ColonySite {
        kind,
        top_left: (x, y),
        size,
    })
}

/// Whether a blueprint of `kind` may be placed with its top-left at `(x, y)`
/// (goal 6.3): every footprint tile in bounds (`OutOfMap`), not water
/// (`Water`), not mountain and free of trees, buildings, sites, zones and
/// reservations (`Blocked`).
pub fn check_blueprint(
    res: &SimResources,
    kind: BlueprintKind,
    x: u32,
    y: u32,
) -> Result<(), RejectReason> {
    if res.rules.colony.is_none() {
        return Err(RejectReason::NoColony);
    }
    let site = footprint(res, kind, x, y)?;
    let tiles = site.tiles();
    if tiles
        .iter()
        .any(|t| terrain(res, *t) == Some(TerrainType::River))
    {
        return Err(RejectReason::Water);
    }
    let reserved: BTreeSet<Tile> = res.colony.reservations.keys().map(|k| k.tile()).collect();
    let blocked = tiles.iter().any(|t| {
        terrain(res, *t) == Some(TerrainType::Mountain)
            || res.colony.trees.contains_key(t)
            || under_building(res, *t)
            || res.colony.zone_of_tile.contains_key(t)
            || reserved.contains(t)
    });
    if blocked {
        return Err(RejectReason::Blocked);
    }
    Ok(())
}

fn place_blueprint(
    world: &mut World,
    res: &mut SimResources,
    rule: &sim_core::rules::ColonyRule,
    kind: BlueprintKind,
    x: u32,
    y: u32,
) {
    let Ok(site) = footprint(res, kind, x, y) else {
        return;
    };
    let centre = site.centre();
    let (logs, progress) = match kind {
        BlueprintKind::Hut => (rule.hut_logs, rule.hut_progress),
    };
    for tile in site.tiles() {
        res.colony.occupied.insert(tile, Occupant::Site(centre));
    }
    // Loose logs under the new footprint move out (merge and spill, 7.4); none
    // of them is reserved (a reserved tile blocks the placement).
    let under: BTreeMap<Tile, (ItemKind, u16)> = site
        .tiles()
        .into_iter()
        .filter_map(|t| {
            res.colony
                .ground_items
                .get(&t)
                .map(|s| (t, (s.kind, s.count)))
        })
        .collect();
    for (tile, (item, count)) in under {
        res.colony.ground_items.remove(&tile);
        put_items(res, tile, item, count);
    }
    world.spawn((
        ConstructionSite::new(
            BuildingBlueprint::new(HUT_BLUEPRINT_ID, site.size.0, site.size.1, progress),
            Position::new(centre.0, centre.1),
        ),
        MaterialNeeds {
            kind: ItemKind::Log,
            needed: logs,
            delivered: 0,
        },
        site,
    ));
    res.colony.touch(false);
}
