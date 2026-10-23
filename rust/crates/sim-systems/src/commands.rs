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
use sim_core::colony::skills::LOWEST_PRIORITY;
use sim_core::colony::workshop::work_tile_of;
use sim_core::colony::{
    Bill, BillId, BlueprintKind, CancelReason, ColonySite, Command, CommandRecord, CommandResult,
    Designation, ItemSpec, Job, JobKind, MaterialNeeds, Occupant, RecipeKind, RejectReason,
    WorkPriorities, WorkType, Workshop, WorkshopId, Zone, ZoneKind,
};
use sim_core::components::{
    Agent, AgentId, BuildingBlueprint, Child, ConstructionSite, Elder, Position, HUT_BLUEPRINT_ID,
};
use sim_core::TerrainType;
use sim_engine::{SimEngine, SimResources};
use std::collections::{BTreeMap, BTreeSet};

use crate::runtime::jobs::cancel;
use crate::runtime::jobs::items::{can_hold_items, has_wall, put_items, terrain, under_building};

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

/// The result of a mark command that marked `count` targets and skipped
/// `unreachable` ones no one from the camp can walk to (M4 goal 3.6).
fn marked((count, unreachable): (usize, usize)) -> CommandResult {
    if count == 0 && unreachable > 0 {
        CommandResult::rejected(RejectReason::Unreachable)
    } else {
        counted(count)
    }
}

/// Whether someone at the camp can walk to `tile` (M4 goal 3.6): the hearth's
/// nav region reaches it (an impassable tile from a passable neighbour).
/// `true` without a built nav map or a hearth (M1/M2 rules keep their marks).
pub fn camp_reaches(res: &SimResources, tile: Tile) -> bool {
    if res.rules.colony.and_then(|r| r.space).is_none() || !res.nav.is_built() {
        return true;
    }
    let Some(hearth) = crate::worldgen::colony::hearth(res) else {
        return true;
    };
    let region = res.nav.region_of(hearth);
    region == 0 || res.nav.reaches(region, tile)
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
            Some(tiles) => marked(designate(res, &tiles)),
        },
        Command::DesignateChopTiles { tiles } => {
            let (w, h) = (res.tile_grid.width, res.tile_grid.height);
            let inside: Vec<Tile> = tiles.into_iter().filter(|t| t.0 < w && t.1 < h).collect();
            if inside.is_empty() {
                return CommandResult::rejected(RejectReason::OutOfMap);
            }
            marked(designate(res, &inside))
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
        Command::PlaceBlueprint { kind, x, y } if kind.is_space() => {
            place_space(world, res, &rule, kind, vec![(x, y)])
        }
        Command::PlaceLine { kind, .. } | Command::PlaceRect { kind, .. } if !kind.is_space() => {
            CommandResult::rejected(RejectReason::Blocked)
        }
        Command::PlaceLine {
            kind,
            x0,
            y0,
            x1,
            y1,
        } => {
            // Both ends on the map, or nothing: an off-map end would make the
            // line as long as the coordinates (code review m-5).
            let (w, h) = (res.tile_grid.width, res.tile_grid.height);
            if x0.max(x1) >= w || y0.max(y1) >= h {
                return CommandResult::rejected(RejectReason::OutOfMap);
            }
            let tiles = line_tiles((x0, y0), (x1, y1));
            place_space(world, res, &rule, kind, tiles)
        }
        Command::PlaceRect {
            kind,
            x0,
            y0,
            x1,
            y1,
            hollow,
        } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => {
                let (lx, hx) = (x0.min(x1), x0.max(x1));
                let (ly, hy) = (y0.min(y1), y0.max(y1));
                let tiles = tiles
                    .into_iter()
                    .filter(|t| !hollow || t.0 == lx || t.0 == hx || t.1 == ly || t.1 == hy)
                    .collect();
                place_space(world, res, &rule, kind, tiles)
            }
        },
        Command::DesignateDig { .. } | Command::DesignateDeconstruct { .. }
            if rule.space.is_none() =>
        {
            CommandResult::rejected(RejectReason::NotYet)
        }
        Command::DesignateDig { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => designate_space(res, &tiles, Designation::Dig),
        },
        Command::DesignateDeconstruct { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => designate_space(res, &tiles, Designation::Deconstruct),
        },
        Command::SetRoomRole { .. } if rule.space.is_none() => {
            CommandResult::rejected(RejectReason::NotYet)
        }
        Command::SetRoomRole { room, role } => {
            if !res.colony.rooms.contains_key(&room) {
                return CommandResult::rejected(RejectReason::Unknown);
            }
            let before = res.colony.room_overrides.get(&room).copied();
            if before == role {
                return CommandResult::rejected(RejectReason::Nothing);
            }
            match role {
                Some(r) => res.colony.room_overrides.insert(room, r),
                None => res.colony.room_overrides.remove(&room),
            };
            crate::runtime::rooms::assign_roles(world, res);
            res.colony.touch(false);
            CommandResult::ok(1)
        }
        Command::PlaceBlueprint { kind, x, y } => match check_blueprint(res, kind, x, y) {
            Err(reason) => CommandResult::rejected(reason),
            Ok(()) => {
                place_blueprint(world, res, &rule, kind, x, y);
                CommandResult::ok(1)
            }
        },
        Command::DesignateQuarry { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => marked(designate_quarry(res, &tiles)),
        },
        Command::AddBill {
            workshop,
            recipe,
            mode,
        } => {
            // The auto-bills a player's firewood bill replaces (M2 review m1).
            let replaced: Vec<BillId> = res
                .colony
                .workshops
                .get(&workshop)
                .filter(|_| recipe == RecipeKind::SplitFirewood)
                .map_or(Vec::new(), |shop| {
                    shop.bills.iter().filter(|b| b.auto).map(|b| b.id).collect()
                });
            let result = bill_command(world, res, workshop, |shop, next| {
                let fits = recipe.recipe().is_some_and(|r| r.workshop == shop.kind);
                if !fits {
                    return Err(RejectReason::WrongWorkshop);
                }
                // A player's firewood bill replaces the hearth's auto-bill; player
                // bills go before any auto-bill (M2 goal 3.8, DECISIONS D16).
                if recipe == RecipeKind::SplitFirewood {
                    shop.bills.retain(|b| !b.auto);
                }
                let at = shop
                    .bills
                    .iter()
                    .position(|b| b.auto)
                    .unwrap_or(shop.bills.len());
                shop.bills.insert(
                    at,
                    Bill {
                        id: next,
                        recipe,
                        mode,
                        done: 0,
                        suspended: false,
                        auto: false,
                    },
                );
                Ok(true)
            });
            if result.accepted && !replaced.is_empty() {
                // Crafts and supplies of the replaced auto-bill go on under the
                // new bill (same recipe), so no started craft loses its log.
                let new_bill = res.colony.next_bill_id.saturating_sub(1);
                let mut q = world.query::<&mut Job>();
                for (_, job) in q.iter() {
                    if let JobKind::Craft {
                        workshop: w, bill, ..
                    } = &mut job.kind
                    {
                        if *w == workshop && replaced.contains(bill) {
                            *bill = new_bill;
                        }
                    }
                }
            }
            result
        }
        Command::RemoveBill { workshop, bill } => {
            let result = bill_command(world, res, workshop, |shop, _| {
                let before = shop.bills.len();
                shop.bills.retain(|b| b.id != bill);
                if shop.active == Some(bill) {
                    shop.active = None;
                }
                if before == shop.bills.len() {
                    return Err(RejectReason::Unknown);
                }
                Ok(false)
            });
            if result.accepted {
                // Its crafts and the supplies of this workshop stop; the next
                // BillSystem run picks another bill and puts unused shelf items down.
                cancel_jobs_where(world, res, |job| match job.kind {
                    JobKind::Craft { bill: b, .. } => b == bill,
                    JobKind::Supply { workshop: w, .. } => w == workshop,
                    _ => false,
                });
                crate::runtime::jobs::bills::schedule(world, res);
            }
            result
        }
        Command::SetBillMode {
            workshop,
            bill,
            mode,
        } => bill_command(world, res, workshop, |shop, _| {
            let found = shop
                .bills
                .iter_mut()
                .find(|b| b.id == bill)
                .ok_or(RejectReason::Unknown)?;
            found.mode = mode;
            found.done = 0;
            Ok(false)
        }),
        Command::MoveBill { workshop, bill, up } => {
            bill_command(world, res, workshop, |shop, _| {
                let i = shop
                    .bills
                    .iter()
                    .position(|b| b.id == bill)
                    .ok_or(RejectReason::Unknown)?;
                let j = if up { i.checked_sub(1) } else { Some(i + 1) };
                match j.filter(|j| *j < shop.bills.len()) {
                    Some(j) => {
                        shop.bills.swap(i, j);
                        Ok(false)
                    }
                    None => Err(RejectReason::Nothing),
                }
            })
        }
        Command::SetBillSuspended {
            workshop,
            bill,
            suspended,
        } => bill_command(world, res, workshop, |shop, _| {
            let found = shop
                .bills
                .iter_mut()
                .find(|b| b.id == bill)
                .ok_or(RejectReason::Unknown)?;
            if found.suspended == suspended {
                return Err(RejectReason::Nothing);
            }
            found.suspended = suspended;
            Ok(false)
        }),
        // Priorities exist only with the economy (M2 review m6).
        Command::SetWorkPriority { .. } if rule.economy.is_none() => {
            CommandResult::rejected(RejectReason::NotYet)
        }
        Command::SetWorkPriority { agent, work, level } => {
            set_work_priority(world, agent, work, level)
        }
        // M5 (ADR 0028): drafting and hunting.
        Command::SetDrafted { .. }
        | Command::DraftMove { .. }
        | Command::DraftAttack { .. }
        | Command::DraftAttackPerson { .. }
        | Command::DesignateHunt { .. }
        | Command::AddHuntZone { .. }
        | Command::CancelHunt { .. }
        | Command::SetDressOrder { .. }
            if rule.wildlife.is_none() =>
        {
            CommandResult::rejected(RejectReason::NotYet)
        }
        Command::SetDrafted { agent, drafted } => set_drafted(world, res, agent, drafted),
        Command::DraftMove { agent, x, y } => {
            if x >= res.tile_grid.width || y >= res.tile_grid.height {
                return CommandResult::rejected(RejectReason::OutOfMap);
            }
            draft_order(
                world,
                agent,
                sim_core::colony::people::DraftOrder::MoveTo((x, y)),
            )
        }
        Command::DraftAttack { agent, animal } => {
            if !res.colony.wildlife.animals.contains_key(&animal) {
                return CommandResult::rejected(RejectReason::Unknown);
            }
            draft_order(
                world,
                agent,
                sim_core::colony::people::DraftOrder::Attack(
                    sim_core::colony::wildlife::Fighter::Animal(animal),
                ),
            )
        }
        Command::DraftAttackPerson { agent, target } => {
            if agent == target || person(world, target).is_none() {
                return CommandResult::rejected(RejectReason::Unknown);
            }
            draft_order(
                world,
                agent,
                sim_core::colony::people::DraftOrder::Attack(
                    sim_core::colony::wildlife::Fighter::Person(target),
                ),
            )
        }
        Command::DesignateHunt { animal } => {
            let w = &mut res.colony.wildlife;
            match w.animals.get(&animal) {
                Some(a) if a.kind == sim_core::colony::wildlife::AnimalKind::Deer => {
                    if w.marks.insert(animal) {
                        w.version += 1;
                        CommandResult::ok(1)
                    } else {
                        CommandResult::rejected(RejectReason::Nothing)
                    }
                }
                Some(_) => CommandResult::rejected(RejectReason::Blocked),
                None => CommandResult::rejected(RejectReason::Unknown),
            }
        }
        Command::AddHuntZone { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(_) => {
                let zone = (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1));
                let w = &mut res.colony.wildlife;
                if w.zones.contains(&zone) {
                    CommandResult::rejected(RejectReason::Nothing)
                } else {
                    w.zones.push(zone);
                    w.version += 1;
                    CommandResult::ok(1)
                }
            }
        },
        Command::CancelHunt { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(_) => {
                let (ax, ay, bx, by) = (x0.min(x1), y0.min(y1), x0.max(x1), y0.max(y1));
                let w = &mut res.colony.wildlife;
                let before = w.zones.len() + w.marks.len();
                w.zones
                    .retain(|&(zx0, zy0, zx1, zy1)| zx1 < ax || zx0 > bx || zy1 < ay || zy0 > by);
                let inside: Vec<u32> = w
                    .animals
                    .values()
                    .filter(|a| (ax..=bx).contains(&a.pos.0) && (ay..=by).contains(&a.pos.1))
                    .map(|a| a.id)
                    .collect();
                for id in inside {
                    w.marks.remove(&id);
                }
                let removed = before - (w.zones.len() + w.marks.len());
                w.version += 1;
                counted(removed)
            }
        },
        // M5.5 (ADR 0031, 0032): neighbours' visitors.
        Command::AcceptMigrants { .. }
        | Command::DeclineMigrants { .. }
        | Command::Trade { .. }
            if rule.neighbours.is_none() =>
        {
            CommandResult::rejected(RejectReason::NotYet)
        }
        Command::AcceptMigrants { wave } => {
            match crate::runtime::neighbours::join::accept(world, res, wave) {
                Some(n) => CommandResult::ok(n),
                None => CommandResult::rejected(RejectReason::NoVisitor),
            }
        }
        Command::DeclineMigrants { wave } => {
            if crate::runtime::neighbours::join::decline(res, wave) {
                CommandResult::ok(1)
            } else {
                CommandResult::rejected(RejectReason::NoVisitor)
            }
        }
        Command::Trade {
            caravan,
            ref give,
            ref take,
        } => {
            let give: Vec<((u32, u32), u16)> = give.iter().map(|&(x, y, n)| ((x, y), n)).collect();
            use crate::runtime::neighbours::trade::{trade, TradeRefusal};
            let practice = crate::runtime::neighbours::trade::colony_practice(world);
            match trade(res, practice, caravan, &give, take) {
                Ok(n) => CommandResult::ok(n),
                Err(TradeRefusal::NoCaravan) => CommandResult::rejected(RejectReason::NoVisitor),
                Err(TradeRefusal::BadItems) => CommandResult::rejected(RejectReason::Unknown),
                Err(TradeRefusal::Empty) => CommandResult::rejected(RejectReason::Nothing),
                Err(TradeRefusal::Balance) => CommandResult::rejected(RejectReason::Unbalanced),
            }
        }
        Command::SetDressOrder { on } => {
            if res.colony.dress_off != on {
                return CommandResult::rejected(RejectReason::Nothing);
            }
            res.colony.dress_off = !on;
            CommandResult::ok(1)
        }
        Command::DesignateClay { .. }
        | Command::OrderTeach { .. }
        | Command::CancelTeach { .. }
            if rule.knowledge.is_none() =>
        {
            CommandResult::rejected(RejectReason::NotYet)
        }
        Command::DesignateClay { x0, y0, x1, y1 } => match rect_tiles(res, x0, y0, x1, y1) {
            None => CommandResult::rejected(RejectReason::OutOfMap),
            Some(tiles) => marked(designate_clay(res, &tiles)),
        },
        Command::OrderTeach {
            teacher,
            learner,
            knowledge,
        } => crate::runtime::jobs::teach::order(world, res, teacher, learner, knowledge),
        Command::CancelTeach { order } => {
            crate::runtime::jobs::teach::cancel_order(world, res, order)
        }
        Command::DebugKill { agent } => debug_kill(world, res, agent),
        Command::DebugGive { kind, count, x, y } => {
            if x >= res.tile_grid.width || y >= res.tile_grid.height {
                return CommandResult::rejected(RejectReason::OutOfMap);
            }
            if count == 0 {
                return CommandResult::rejected(RejectReason::Nothing);
            }
            res.colony.item_ledger.create(kind, u64::from(count));
            crate::runtime::jobs::items::put_items(
                res,
                (x, y),
                sim_core::colony::ItemSpec::raw(kind, None),
                count,
            );
            res.colony.touch(false);
            CommandResult::ok(u32::from(count))
        }
    }
}

/// The living person `agent`: its entity and whether it is a child.
fn person(world: &World, agent: AgentId) -> Option<(hecs::Entity, bool)> {
    world
        .query::<(&Agent, Option<&sim_core::components::Child>)>()
        .iter()
        .find(|(_, (a, _))| a.id == agent)
        .map(|(e, (_, child))| (e, child.is_some()))
}

/// `SetDrafted` (M5): draft or release `agent`.
fn set_drafted(
    world: &mut World,
    res: &mut SimResources,
    agent: AgentId,
    drafted: bool,
) -> CommandResult {
    let Some((entity, child)) = person(world, agent) else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    let now = world
        .get::<&sim_core::colony::people::Drafted>(entity)
        .is_ok();
    if now == drafted {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    if drafted {
        if child {
            return CommandResult::rejected(RejectReason::TooYoung);
        }
        crate::runtime::wildlife::draft::draft(world, res, entity, agent);
    } else {
        crate::runtime::wildlife::draft::undraft(world, res, entity, agent, 1);
    }
    CommandResult::ok(1)
}

/// A drafted person's new order (M5).
fn draft_order(
    world: &mut World,
    agent: AgentId,
    order: sim_core::colony::people::DraftOrder,
) -> CommandResult {
    let Some((entity, _)) = person(world, agent) else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    if crate::runtime::wildlife::draft::order(world, entity, order) {
        CommandResult::ok(1)
    } else {
        CommandResult::rejected(RejectReason::NotDrafted)
    }
}

/// `DebugKill` (M4 F10; tests and the probe only, never the game bridge):
/// the person dies at once (recorded as old age) through the despawn path.
fn debug_kill(world: &mut World, res: &mut SimResources, agent: AgentId) -> CommandResult {
    let Some((entity, pos)) = world
        .query::<(&Agent, &Position)>()
        .iter()
        .find(|(_, (a, _))| a.id == agent)
        .map(|(e, (_, p))| (e, (p.x, p.y)))
    else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    let tick = res.current_tick;
    crate::runtime::survival::despawn_agent(
        world,
        res,
        entity,
        agent,
        pos,
        sim_core::causal::DeathReason::OldAge,
        tick,
    );
    CommandResult::ok(1)
}

/// Apply `edit` to `workshop` (`Unknown` when there is none) and reschedule
/// its bills. `edit` gets the next bill id and returns whether it used it.
fn bill_command(
    world: &World,
    res: &mut SimResources,
    workshop: WorkshopId,
    edit: impl FnOnce(&mut Workshop, BillId) -> Result<bool, RejectReason>,
) -> CommandResult {
    let next = res.colony.next_bill_id.max(1);
    let Some(shop) = res.colony.workshops.get_mut(&workshop) else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    match edit(shop, next) {
        Err(reason) => CommandResult::rejected(reason),
        Ok(used) => {
            if used {
                res.colony.next_bill_id = next + 1;
            }
            crate::runtime::jobs::bills::schedule(world, res);
            res.colony.touch(false);
            CommandResult::ok(1)
        }
    }
}

/// `SetWorkPriority` (M2 goal 3.6): insert or update the person's
/// `WorkPriorities` (a structural insert between ticks). Children cannot work
/// (`Blocked`); a level above 4 is `OutOfMap` (out of range).
fn set_work_priority(
    world: &mut World,
    agent: AgentId,
    work: WorkType,
    level: u8,
) -> CommandResult {
    if level > LOWEST_PRIORITY {
        return CommandResult::rejected(RejectReason::OutOfMap);
    }
    let Some((entity, child, elder)) = world
        .query::<(&Agent, Option<&Child>, Option<&Elder>)>()
        .iter()
        .find(|(_, (a, _, _))| a.id == agent)
        .map(|(e, (_, c, el))| (e, c.is_some(), el.is_some()))
    else {
        return CommandResult::rejected(RejectReason::Unknown);
    };
    if child {
        return CommandResult::rejected(RejectReason::Blocked);
    }
    let mut priorities = world
        .get::<&WorkPriorities>(entity)
        .map_or(WorkPriorities::defaults(elder), |p| *p);
    if priorities.level(work) == level {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    priorities.levels[work.code() as usize] = level;
    let _ = world.insert_one(entity, priorities);
    CommandResult::ok(1)
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

fn designate(res: &mut SimResources, tiles: &[Tile]) -> (usize, usize) {
    let (mut count, mut far) = (0, 0);
    for tile in tiles {
        if res.colony.trees.contains_key(tile) && !res.colony.designations.contains_key(tile) {
            if !camp_reaches(res, *tile) {
                far += 1;
                continue;
            }
            res.colony.designations.insert(*tile, Designation::Chop);
            count += 1;
        }
    }
    if count > 0 {
        res.colony.touch(false);
    }
    (count, far)
}

/// Mark every unmarked resource node among `tiles` for quarrying (M2).
fn designate_quarry(res: &mut SimResources, tiles: &[Tile]) -> (usize, usize) {
    let (mut count, mut far) = (0, 0);
    for tile in tiles {
        let quarriable = res
            .colony
            .nodes
            .get(tile)
            .is_some_and(|n| n.kind != sim_core::colony::NodeKind::ClayBank);
        if quarriable && !res.colony.designations.contains_key(tile) {
            if !camp_reaches(res, *tile) {
                far += 1;
                continue;
            }
            res.colony.designations.insert(*tile, Designation::Quarry);
            count += 1;
        }
    }
    if count > 0 {
        res.colony.touch(false);
    }
    (count, far)
}

/// Mark every unmarked clay bank among `tiles` for digging clay (M4 goal 3.5).
fn designate_clay(res: &mut SimResources, tiles: &[Tile]) -> (usize, usize) {
    let (mut count, mut far) = (0, 0);
    for tile in tiles {
        let bank = res
            .colony
            .nodes
            .get(tile)
            .is_some_and(|n| n.kind == sim_core::colony::NodeKind::ClayBank);
        if bank && !res.colony.designations.contains_key(tile) {
            if !camp_reaches(res, *tile) {
                far += 1;
                continue;
            }
            res.colony.designations.insert(*tile, Designation::ClayPit);
            count += 1;
        }
    }
    if count > 0 {
        res.colony.touch(false);
    }
    (count, far)
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
    cancel_jobs_where(world, res, |job| match job.kind {
        JobKind::Chop { tree } => marked.contains(&tree),
        JobKind::Quarry { node } => marked.contains(&node),
        JobKind::Dig { tile } | JobKind::Deconstruct { tile } => marked.contains(&tile),
        _ => false,
    });
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
        JobKind::Clear { from, .. } => removed.contains(&from),
        _ => false,
    });
    for (centre, entity, site, needs) in &sites {
        for tile in site.tiles() {
            res.colony.occupied.remove(&tile);
        }
        res.colony.site_work_tiles.remove(centre);
        let _ = world.despawn(*entity);
        // M3: a space site's tile is walkable again.
        if res.colony.space_sites.remove(centre).is_some() {
            crate::runtime::nav::refresh(res, *centre);
        }
        for line in needs.used() {
            if line.delivered > 0 {
                let spec = ItemSpec::raw(line.kind, line.material);
                put_items(res, *centre, spec, line.delivered);
            }
        }
    }
    if !marks.is_empty() || !sites.is_empty() {
        res.colony.touch(false);
    }
    marks.len() + sites.len()
}

/// Whether `tile` may become a stockpile cell (M3: only where items may lie,
/// so never on undug hill, a wall or a door).
fn zoneable(res: &SimResources, tile: Tile) -> bool {
    can_hold_items(res, tile) && !res.colony.zone_of_tile.contains_key(&tile)
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
    let size = match res.rules.colony {
        Some(rule) => kind.plan(&rule).0,
        None => (3, 3),
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
    let Some(rule) = res.rules.colony else {
        return Err(RejectReason::NoColony);
    };
    // Workshops exist only with the economy (M2 review m4).
    if kind.is_workshop() && rule.economy.is_none() {
        return Err(RejectReason::NotYet);
    }
    // M4: the hearth fire is made by the worldgen; the pottery bench and the
    // grinding stone only exist under the knowledge rule.
    if !kind.placeable()
        || (matches!(
            kind,
            BlueprintKind::PotteryBench | BlueprintKind::GrindingStone
        ) && rule.knowledge.is_none())
    {
        return Err(RejectReason::NotYet);
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
            || !crate::runtime::nav::passable(res, *t)
            || has_wall(res, *t)
            || res.colony.trees.contains_key(t)
            || res.colony.nodes.contains_key(t)
            || res.colony.beds.contains_key(t)
            || under_building(res, *t)
            || res.colony.zone_of_tile.contains_key(t)
            || reserved.contains(t)
    });
    // M2: no footprint tile may cover a workshop's work tile, and a
    // workshop's own work tile must be on the map, dry, flat and free of buildings.
    let work_tiles: BTreeSet<Tile> = res
        .colony
        .workshops
        .values()
        .map(|shop| shop.work_tile())
        .chain(res.colony.site_work_tiles.values().copied())
        .collect();
    if blocked || tiles.iter().any(|t| work_tiles.contains(t)) {
        return Err(RejectReason::Blocked);
    }
    if kind.is_workshop() {
        let work = work_tile_of(site.top_left, site.size);
        if work.1 >= res.tile_grid.height {
            return Err(RejectReason::OutOfMap);
        }
        match terrain(res, work) {
            Some(TerrainType::River) => return Err(RejectReason::Water),
            Some(TerrainType::Mountain) | None => return Err(RejectReason::Blocked),
            _ => {}
        }
        if under_building(res, work)
            || !crate::runtime::nav::passable(res, work)
            || has_wall(res, work)
        {
            return Err(RejectReason::Blocked);
        }
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
    let (_, needs, progress) = kind.plan(rule);
    for tile in site.tiles() {
        res.colony.occupied.insert(tile, Occupant::Site(centre));
    }
    if kind.is_workshop() {
        res.colony
            .site_work_tiles
            .insert(centre, work_tile_of(site.top_left, site.size));
    }
    // Loose logs under the new footprint move out (merge and spill, 7.4); none
    // of them is reserved (a reserved tile blocks the placement).
    let under: BTreeMap<Tile, (ItemSpec, u16)> = site
        .tiles()
        .into_iter()
        .filter_map(|t| {
            res.colony
                .ground_items
                .get(&t)
                .map(|s| (t, (s.spec(), s.count)))
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
        needs,
        site,
    ));
    res.colony.touch(false);
}

/// The tiles of the Bresenham line from `a` to `b`, `a` first (M3 `PlaceLine`).
pub fn line_tiles(a: Tile, b: Tile) -> Vec<Tile> {
    let (mut x, mut y) = (i64::from(a.0), i64::from(a.1));
    let (x1, y1) = (i64::from(b.0), i64::from(b.1));
    let dx = (x1 - x).abs();
    let dy = -(y1 - y).abs();
    let sx = if x < x1 { 1 } else { -1 };
    let sy = if y < y1 { 1 } else { -1 };
    let mut err = dx + dy;
    let mut out = Vec::new();
    loop {
        out.push((x as u32, y as u32));
        if x == x1 && y == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
    out
}

/// Whether a 1×1 space blueprint of `kind` may go on `tile` (M3, ADR 0022):
/// on the map (`OutOfMap`), dry (`Water`), not undug hill (`Hill`), not
/// mountain, nothing built there that the kind would replace (`Built`), no
/// tree, resource node, building, site, workshop work tile or reservation
/// (`Blocked`); walls, doors and bedrolls also not on a stockpile cell; a wall
/// not where a person stands (`Person`). Items on the tile do not refuse it:
/// a hauler carries them off first.
pub fn check_space(
    res: &SimResources,
    people: &BTreeSet<Tile>,
    kind: BlueprintKind,
    tile: Tile,
) -> Result<(), RejectReason> {
    let grid = &res.tile_grid;
    if tile.0 >= grid.width || tile.1 >= grid.height {
        return Err(RejectReason::OutOfMap);
    }
    let i = grid.idx(tile.0, tile.1);
    match grid.terrain_type[i] {
        TerrainType::River => return Err(RejectReason::Water),
        TerrainType::Mountain => return Err(RejectReason::Blocked),
        TerrainType::Hill if !grid.is_dug(i) => return Err(RejectReason::Hill),
        _ => {}
    }
    let walled = grid.wall_material[i].is_some();
    let built = match kind {
        BlueprintKind::PackedFloor => walled || grid.floor_material[i].is_some(),
        BlueprintKind::Bedroll => walled || res.colony.beds.contains_key(&tile),
        _ => walled || res.colony.beds.contains_key(&tile),
    };
    if built {
        return Err(RejectReason::Built);
    }
    let work_tile = res
        .colony
        .workshops
        .values()
        .any(|shop| shop.work_tile() == tile)
        || res.colony.site_work_tiles.values().any(|t| *t == tile);
    // A tile being dug or taken apart is busy; a reserved stack or cell is not
    // (a site does not stop a haul in flight: the stack is cleared, the drop spills).
    let reserved = res
        .colony
        .reservations
        .contains_key(&sim_core::colony::ReserveKey::Work(tile));
    if res.colony.trees.contains_key(&tile)
        || res.colony.nodes.contains_key(&tile)
        || under_building(res, tile)
        || work_tile
        || reserved
        || (kind != BlueprintKind::PackedFloor && res.colony.zone_of_tile.contains_key(&tile))
    {
        return Err(RejectReason::Blocked);
    }
    if kind.is_wall() && people.contains(&tile) {
        return Err(RejectReason::Person);
    }
    Ok(())
}

/// Every agent's tile.
fn people_tiles(world: &World) -> BTreeSet<Tile> {
    world
        .query::<&Position>()
        .iter()
        .map(|(_, p)| (p.x, p.y))
        .collect()
}

/// Place a 1×1 space site on every tile of `tiles` that passes
/// [`check_space`] (M3; `NotYet` without the space rule). The count is the
/// sites placed; with none, the first tile's reason.
fn place_space(
    world: &mut World,
    res: &mut SimResources,
    rule: &sim_core::rules::ColonyRule,
    kind: BlueprintKind,
    tiles: Vec<Tile>,
) -> CommandResult {
    if rule.space.is_none() || rule.economy.is_none() {
        return CommandResult::rejected(RejectReason::NotYet);
    }
    let people = people_tiles(world);
    let (_, needs, progress) = kind.plan(rule);
    let mut placed = 0u32;
    let mut first: Option<RejectReason> = None;
    let mut seen: BTreeSet<Tile> = BTreeSet::new();
    for tile in tiles {
        if !seen.insert(tile) {
            continue;
        }
        if let Err(reason) = check_space(res, &people, kind, tile) {
            first.get_or_insert(reason);
            continue;
        }
        res.colony.occupied.insert(tile, Occupant::Site(tile));
        res.colony.space_sites.insert(tile, kind);
        world.spawn((
            ConstructionSite::new(
                BuildingBlueprint::new(HUT_BLUEPRINT_ID, 1, 1, progress),
                Position::new(tile.0, tile.1),
            ),
            needs,
            ColonySite {
                kind,
                top_left: tile,
                size: (1, 1),
            },
        ));
        crate::runtime::nav::refresh(res, tile);
        placed += 1;
    }
    if placed == 0 {
        return CommandResult::rejected(first.unwrap_or(RejectReason::Nothing));
    }
    res.colony.touch(false);
    CommandResult::ok(placed)
}

/// Mark `tiles` for digging (undug hill, not on the map edge) or for
/// deconstruction (something built), skipping marked tiles (M3).
fn designate_space(res: &mut SimResources, tiles: &[Tile], mark: Designation) -> CommandResult {
    let mut count = 0usize;
    let mut first: Option<RejectReason> = None;
    let mut far = 0usize;
    for tile in tiles {
        if res.colony.designations.contains_key(tile) {
            continue;
        }
        let ok = match mark {
            Designation::Dig => {
                let edge = tile.0 == 0
                    || tile.1 == 0
                    || tile.0 + 1 >= res.tile_grid.width
                    || tile.1 + 1 >= res.tile_grid.height;
                if edge && terrain(res, *tile) == Some(TerrainType::Hill) {
                    first.get_or_insert(RejectReason::Edge);
                }
                crate::runtime::jobs::space::diggable(res, *tile)
            }
            _ => crate::runtime::jobs::space::built_at(res, *tile).is_some(),
        };
        if ok && mark == Designation::Dig && !camp_reaches(res, *tile) {
            far += 1;
            continue;
        }
        if ok {
            res.colony.designations.insert(*tile, mark);
            count += 1;
        }
    }
    if count == 0 && far > 0 {
        return CommandResult::rejected(RejectReason::Unreachable);
    }
    if count == 0 {
        return CommandResult::rejected(first.unwrap_or(RejectReason::Nothing));
    }
    res.colony.touch(false);
    CommandResult::ok(count as u32)
}
