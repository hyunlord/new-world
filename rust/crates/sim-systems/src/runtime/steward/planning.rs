//! Pure integer planners for ordered data rules; all effects are ordinary commands.
use crate::commands::{camp_reaches, check_blueprint};
use crate::runtime::jobs::{bills::maintain_count, hearth::hearth_tiles, items::can_hold_items};
use hecs::World;
use sim_core::colony::{
    knowledge::{self, Knowledge, TechStatus},
    steward::{Action, Rule, Target},
    BillMode, ColonySite, Command,
};
use sim_core::components::{Agent, Child, Life, Position, Temperament};
use sim_engine::SimResources;
use std::{cmp::Reverse, collections::BTreeSet};
type Tile = (u32, u32);
fn distance(a: Tile, b: Tile) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}
fn candidates(res: &SimResources, hearth: Tile) -> Vec<Tile> {
    let mut tiles: Vec<_> = (1..res.tile_grid.height.saturating_sub(1))
        .flat_map(|y| (1..res.tile_grid.width.saturating_sub(1)).map(move |x| (x, y)))
        .collect();
    tiles.sort_by_key(|&(x, y)| (distance((x, y), hearth), y, x));
    tiles
}
fn permitted(res: &SimResources, tile: Tile) -> bool {
    !res.steward
        .vetoed(&Target::Tile(tile.0, tile.1), res.current_tick)
        && camp_reaches(res, tile)
}
/// Evaluate one rule without mutating simulation state.
pub fn plan(world: &World, res: &SimResources, rule: &Rule) -> Vec<Command> {
    let Some(hearth) = hearth_tiles(res).first().copied() else {
        return Vec::new();
    };
    let adults = world.query::<&Agent>().without::<&Child>().iter().count() as u32;
    match &rule.action {
        Action::Stockpile {
            base_cells,
            divisor,
        } => {
            if !res.colony.zones.is_empty() {
                return Vec::new();
            }
            let area = base_cells.saturating_add(adults / divisor);
            let mut side = 1u32;
            while side.saturating_mul(side) < area {
                side += 1;
            }
            for (x, y) in candidates(res, hearth) {
                if x.saturating_add(side) >= res.tile_grid.width
                    || y.saturating_add(side) >= res.tile_grid.height
                {
                    continue;
                }
                if (y..y + side).all(|yy| {
                    (x..x + side).all(|xx| {
                        can_hold_items(res, (xx, yy))
                            && !res.colony.zone_of_tile.contains_key(&(xx, yy))
                            && permitted(res, (xx, yy))
                    })
                }) {
                    return vec![Command::AddStockpile {
                        x0: x,
                        y0: y,
                        x1: x + side - 1,
                        y1: y + side - 1,
                    }];
                }
            }
            Vec::new()
        }
        Action::Wood {
            items,
            days,
            camp_radius,
            batch,
        } => {
            let have: u32 = items
                .iter()
                .map(|&k| maintain_count(world, res, k))
                .sum::<u32>()
                .saturating_add(crate::runtime::jobs::hearth::settlement_wood(res));
            let marked: u32 = res
                .colony
                .trees
                .iter()
                .filter(|(t, _)| res.colony.designations.contains_key(t))
                .map(|(_, t)| u32::from(t.logs))
                .sum();
            let building_need: u32 = world
                .query::<&sim_core::colony::MaterialNeeds>()
                .iter()
                .map(|(_, n)| {
                    n.used()
                        .filter(|l| items.contains(&l.kind))
                        .map(|l| u32::from(l.needed.saturating_sub(l.delivered)))
                        .sum::<u32>()
                })
                .sum();
            let mut need = days
                .saturating_mul(daily_fuel(res))
                .saturating_add(building_need)
                .saturating_sub(have.saturating_add(marked));
            if need == 0 {
                return Vec::new();
            }
            let mut trees: Vec<_> = res
                .colony
                .trees
                .iter()
                .filter(|(t, _)| {
                    distance(**t, hearth) > *camp_radius
                        && !res.colony.designations.contains_key(t)
                        && permitted(res, **t)
                })
                .collect();
            trees.sort_by_key(|(t, _)| (distance(**t, hearth), t.1, t.0));
            let mut tiles = Vec::new();
            for (&tile, tree) in trees.into_iter().take(*batch as usize) {
                if need == 0 {
                    break;
                }
                tiles.push(tile);
                need = need.saturating_sub(u32::from(tree.logs));
            }
            if tiles.is_empty() {
                Vec::new()
            } else {
                vec![Command::DesignateChopTiles { tiles }]
            }
        }
        action @ Action::House { .. } => {
            if let Some(command) = super::completion::plan(res, rule) {
                vec![command]
            } else {
                house(world, res, hearth, action, false).commands
            }
        }
        Action::Bills {
            recipes,
            winter_recipe,
            days,
        } => {
            let reserved_house = sim_core::colony::steward::rules()
                .ok()
                .and_then(|data| {
                    data.rules.iter().find(|rule| {
                        matches!(rule.action, Action::House { .. })
                            && res.steward.scope.permits(rule.scope)
                    })
                })
                .map(|rule| house(world, res, hearth, &rule.action, true).reserved)
                .unwrap_or_default();
            let mut wanted = recipes.clone();
            wanted.push((
                *winter_recipe,
                u16::try_from(u32::from(*days).saturating_mul(winter_daily_fuel(res)))
                    .unwrap_or(u16::MAX),
            ));
            let mut out = super::production::inputs(world, res, hearth, &wanted);
            let mut pending = BTreeSet::new();
            for (recipe, target) in wanted {
                let Some(def) = recipe.recipe() else {
                    continue;
                };
                if !crate::runtime::knowledge::anyone_practises(
                    world,
                    knowledge::gates().build(def.workshop),
                ) || !crate::runtime::knowledge::anyone_practises(
                    world,
                    knowledge::gates().recipe(recipe),
                ) {
                    continue;
                }
                if let Some((&id, shop)) = res
                    .colony
                    .workshops
                    .iter()
                    .find(|(_, s)| def.supports_workshop(s.kind))
                {
                    if shop.bills.iter().any(|b| {
                        b.recipe == recipe
                            && (!b.auto || super::production::player_edited_bill(res, id, b.id))
                    }) || res
                        .steward
                        .vetoed(&Target::Bill(id, recipe.code()), res.current_tick)
                    {
                        continue;
                    }
                    out.push(Command::AddBill {
                        workshop: id,
                        recipe,
                        mode: BillMode::Maintain(target),
                    });
                } else if !world
                    .query::<&ColonySite>()
                    .iter()
                    .any(|(_, s)| def.supports_workshop(s.kind))
                    && pending.insert(def.workshop)
                {
                    let mut locations = candidates(res, hearth);
                    if let Some(colony_rule) = res.rules.colony {
                        let (size, _, _) = def.workshop.plan(&colony_rule);
                        locations.sort_by_key(|&(x, y)| {
                            let work = sim_core::colony::workshop::work_tile_of((x, y), size);
                            let usable = res.colony.zone_of_tile.contains_key(&work)
                                || distance(work, hearth)
                                    <= crate::runtime::jobs::hearth::HEARTH_PILE_REACH;
                            (
                                !usable,
                                !crate::runtime::rooms::roofed(res, work),
                                distance(work, hearth),
                                distance((x, y), hearth),
                                y,
                                x,
                            )
                        });
                    }
                    if let Some((x, y)) = locations.into_iter().find(|&t| {
                        let avoids_house = res.rules.colony.is_some_and(|rule| {
                            let (size, _, _) = def.workshop.plan(&rule);
                            !reserved_house
                                .contains(&sim_core::colony::workshop::work_tile_of(t, size))
                                && (t.1..t.1 + size.1).all(|y| {
                                    (t.0..t.0 + size.0).all(|x| !reserved_house.contains(&(x, y)))
                                })
                        });
                        avoids_house
                            && permitted(res, t)
                            && check_blueprint(res, def.workshop, t.0, t.1).is_ok()
                    }) {
                        // One missing workshop per interval avoids overlapping speculative footprints.
                        out.push(Command::PlaceBlueprint {
                            kind: def.workshop,
                            x,
                            y,
                        });
                        break;
                    }
                }
            }
            out
        }
        Action::Hunt {
            item,
            animal,
            knowledge,
            days,
            radius,
        } => {
            let need: u64 = res
                .settlements
                .keys()
                .map(|&id| crate::runtime::settlement::relations::need_milli(world, res, id))
                .sum();
            let food: u64 = res
                .settlements
                .keys()
                .map(|&id| crate::runtime::settlement::food_economy::stock_in_hand(world, res, id))
                .sum();
            if (food + u64::from(maintain_count(world, res, *item))).saturating_mul(1000)
                >= need.saturating_mul(u64::from(*days))
            {
                return Vec::new();
            }
            let Some(node) = knowledge::id(knowledge) else {
                return Vec::new();
            };
            if !crate::runtime::knowledge::anyone_practises(world, 1u32 << node) {
                return Vec::new();
            }
            let Some(target) = res
                .colony
                .wildlife
                .animals
                .values()
                .filter(|a| a.kind == *animal && a.hp > 0 && !res.colony.wildlife.huntable(a))
                .map(|a| a.home.unwrap_or(a.pos))
                .filter(|&t| permitted(res, t))
                .min_by_key(|&t| (distance(t, hearth), t.1, t.0))
            else {
                return Vec::new();
            };
            vec![Command::AddHuntZone {
                x0: target.0.saturating_sub(*radius),
                y0: target.1.saturating_sub(*radius),
                x1: target
                    .0
                    .saturating_add(*radius)
                    .min(res.tile_grid.width - 1),
                y1: target
                    .1
                    .saturating_add(*radius)
                    .min(res.tile_grid.height - 1),
            }]
        }
        Action::Teach => teach(world, res),
    }
}
#[derive(Default)]
struct HousePlan {
    commands: Vec<Command>,
    reserved: BTreeSet<Tile>,
}
fn house(
    world: &World,
    res: &SimResources,
    hearth: Tile,
    action: &Action,
    reserve_only: bool,
) -> HousePlan {
    let Action::House {
        wall,
        door,
        bed,
        max_beds,
        service_cells,
        ..
    } = *action
    else {
        return HousePlan::default();
    };
    if !res.colony.space_sites.is_empty()
        || (!reserve_only && world.query::<&ColonySite>().iter().next().is_some())
    {
        return HousePlan::default();
    }
    if [wall, door, bed].iter().any(|&kind| {
        !crate::runtime::knowledge::anyone_practises(world, knowledge::gates().build(kind))
    }) {
        return HousePlan::default();
    }
    let covered = res
        .colony
        .beds
        .keys()
        .filter(|tile| {
            res.colony
                .room_of_tile
                .get(tile)
                .and_then(|id| res.colony.rooms.get(id))
                .is_some_and(|room| room.roofed)
        })
        .count() as u32;
    let beds = (world.query::<&Agent>().iter().count() as u32)
        .saturating_sub(covered)
        .min(max_beds);
    if beds == 0 {
        return HousePlan::default();
    }
    let stock: Vec<_> = res.colony.zone_of_tile.keys().copied().collect();
    let area = beds
        .saturating_add(service_cells)
        .saturating_add((stock.len() as u32).min(beds.saturating_add(service_cells)));
    let mut width = 1u32;
    while width.saturating_mul(width) < area {
        width += 1;
    }
    let Some(space) = res.rules.colony.and_then(|rule| rule.space) else {
        return HousePlan::default();
    };
    let minimum_height = area.div_ceil(width).max(2);
    if width.saturating_mul(minimum_height) > space.room_max_tiles {
        return HousePlan::default();
    }
    let actual_people: BTreeSet<_> = world
        .query::<&Position>()
        .iter()
        .map(|(_, pos)| (pos.x, pos.y))
        .collect();
    let people = BTreeSet::new();
    let inside = |tile: Tile, x: u32, y: u32, width: u32, height: u32| {
        tile.0 > x && tile.0 <= x + width && tile.1 > y && tile.1 <= y + height
    };
    // Prefix counts make candidate capacity checks constant-time, independent of envelope area.
    let stride = res.tile_grid.width as usize + 1;
    let mut capacity = vec![0usize; stride * (res.tile_grid.height as usize + 1)];
    for yy in 0..res.tile_grid.height {
        let mut row = 0usize;
        for xx in 0..res.tile_grid.width {
            let tile = (xx, yy);
            row += usize::from(
                !res.colony.zone_of_tile.contains_key(&tile)
                    && permitted(res, tile)
                    && can_hold_items(res, tile)
                    && crate::commands::check_space_after_tree_clear(res, &people, bed, tile, true)
                        .is_ok(),
            );
            let i = (yy as usize + 1) * stride + xx as usize + 1;
            capacity[i] = capacity[i - stride] + row;
        }
    }
    let free_cells = |x: u32, y: u32, width: u32, height: u32| {
        let left = x as usize + 1;
        let top = y as usize + 1;
        let right = left + width as usize;
        let bottom = top + height as usize;
        capacity[bottom * stride + right] + capacity[top * stride + left]
            - capacity[top * stride + right]
            - capacity[bottom * stride + left]
    };
    let mut local_origins = BTreeSet::new();
    let mut origins: BTreeSet<_> = candidates(res, hearth)
        .into_iter()
        .map(|(x, y)| (x, y, width))
        .collect();
    // Adjacent aspect ratios search around service anchors; the original ratio retains its global fallback.
    for candidate_width in [width.saturating_sub(1).max(2), width, width + 1] {
        for anchor in stock.iter().copied().chain(std::iter::once(hearth)) {
            for y in anchor
                .1
                .saturating_sub(space.room_max_tiles / candidate_width)
                .max(1)..anchor.1
            {
                for x in anchor.0.saturating_sub(candidate_width).max(1)..anchor.0 {
                    origins.insert((x, y, candidate_width));
                    local_origins.insert((x, y, candidate_width));
                }
            }
        }
    }
    let mut locations: Vec<_> = origins
        .into_iter()
        .flat_map(|(x, y, width)| {
            let mut heights = Vec::new();
            let minimum_height = area.div_ceil(width).max(2);
            if x + width + 2 >= res.tile_grid.width {
                return heights;
            }
            for height in minimum_height..=space.room_max_tiles / width {
                if y + height + 2 >= res.tile_grid.height {
                    break;
                }
                if free_cells(x, y, width, height) >= (beds + service_cells) as usize {
                    heights.push((x, y, width, height));
                    // Only bounded service-anchor origins explore alternative envelopes.
                    if !local_origins.contains(&(x, y, width)) {
                        break;
                    }
                }
            }
            heights
        })
        .collect();
    locations.sort_by_cached_key(|&(x, y, width, height)| {
        (
            Reverse(
                stock
                    .iter()
                    .filter(|&&tile| inside(tile, x, y, width, height))
                    .count(),
            ),
            !inside(hearth, x, y, width, height),
            width * height,
            distance((x + width / 2, y + height / 2), hearth),
            y,
            x,
            width,
        )
    });
    let camp_radius = sim_core::colony::steward::rules()
        .ok()
        .and_then(|data| {
            data.rules.iter().find_map(|rule| match rule.action {
                Action::Wood { camp_radius, .. } => Some(camp_radius),
                _ => None,
            })
        })
        .unwrap_or(0);
    let can_chop = crate::runtime::knowledge::anyone_practises(
        world,
        knowledge::gates().designation(sim_core::colony::Designation::Chop),
    );
    for (x, y, width, height) in locations {
        if x + width + 2 >= res.tile_grid.width || y + height + 2 >= res.tile_grid.height {
            continue;
        }
        let centre = (x + width.div_ceil(2), y + height.div_ceil(2));
        let preferred_door = if hearth.0.abs_diff(centre.0) > hearth.1.abs_diff(centre.1) {
            (
                if hearth.0 < centre.0 {
                    x
                } else {
                    x + width + 1
                },
                centre.1,
            )
        } else {
            (
                centre.0,
                if hearth.1 < centre.1 {
                    y
                } else {
                    y + height + 1
                },
            )
        };
        // Keep the hearth-facing side, but choose a legal opening in its natural boundary.
        let mut openings: Vec<_> = if preferred_door.0 == x || preferred_door.0 == x + width + 1 {
            (y + 1..=y + height)
                .map(|yy| (preferred_door.0, yy))
                .collect()
        } else {
            (x + 1..=x + width)
                .map(|xx| (xx, preferred_door.1))
                .collect()
        };
        if inside(hearth, x, y, width, height) {
            openings = (x + 1..=x + width)
                .flat_map(|xx| [(xx, y), (xx, y + height + 1)])
                .chain((y + 1..=y + height).flat_map(|yy| [(x, yy), (x + width + 1, yy)]))
                .collect();
            openings.sort_by_key(|&tile| (distance(tile, hearth), tile.1, tile.0));
        } else {
            openings.sort_by_key(|&tile| (distance(tile, preferred_door), tile.1, tile.0));
        }
        let Some(door_tile) = openings.into_iter().find(|&tile| {
            let outside = if tile.0 == x {
                (x - 1, tile.1)
            } else if tile.0 == x + width + 1 {
                (tile.0 + 1, tile.1)
            } else if tile.1 == y {
                (tile.0, y - 1)
            } else {
                (tile.0, tile.1 + 1)
            };
            permitted(res, tile)
                && crate::runtime::nav::passable(res, outside)
                && permitted(res, outside)
                && crate::commands::check_space_after_tree_clear(
                    res,
                    &people,
                    door,
                    tile,
                    res.colony.trees.contains_key(&tile),
                )
                .is_ok()
        }) else {
            continue;
        };
        let mut clearing = Vec::new();
        let mut border = Vec::new();
        let mut free = Vec::new();
        let mut valid = true;
        for yy in y..=y + height + 1 {
            for xx in x..=x + width + 1 {
                let tile = (xx, yy);
                if crate::runtime::rooms::bounds(res, tile) {
                    // Existing terrain/walls already enclose; never build or dig on them.
                    continue;
                }
                let tree = res.colony.trees.contains_key(&tile);
                if tree {
                    if !can_chop
                        || distance(tile, hearth) <= camp_radius
                        || !permitted(res, tile)
                        || res
                            .colony
                            .designations
                            .get(&tile)
                            .is_some_and(|mark| *mark != sim_core::colony::Designation::Chop)
                    {
                        valid = false;
                        break;
                    }
                    clearing.push(tile);
                }
                if xx == x || xx == x + width + 1 || yy == y || yy == y + height + 1 {
                    let kind = if tile == door_tile { door } else { wall };
                    if !permitted(res, tile)
                        || crate::commands::check_space_after_tree_clear(
                            res, &people, kind, tile, tree,
                        )
                        .is_err()
                    {
                        valid = false;
                        break;
                    }
                    border.push(Command::PlaceBlueprint { kind, x: xx, y: yy });
                } else {
                    if res.colony.nodes.contains_key(&tile)
                        || crate::runtime::jobs::items::has_wall(res, tile)
                    {
                        valid = false;
                        break;
                    }
                    // Existing storage and finished structures remain untouched inside the shell.
                    if crate::runtime::jobs::items::under_building(res, tile)
                        || res.colony.beds.contains_key(&tile)
                    {
                        continue;
                    }
                    if !can_hold_items(res, tile) || !permitted(res, tile) {
                        valid = false;
                        break;
                    }
                    if !res.colony.zone_of_tile.contains_key(&tile)
                        && crate::commands::check_space_after_tree_clear(
                            res, &people, bed, tile, tree,
                        )
                        .is_ok()
                    {
                        free.push(tile);
                    }
                }
            }
            if !valid {
                break;
            }
        }
        if !valid || free.len() < (beds + service_cells) as usize {
            continue;
        }
        let boundary: BTreeSet<_> = border
            .iter()
            .filter_map(|command| match command {
                Command::PlaceBlueprint { x, y, .. } => Some((*x, *y)),
                _ => None,
            })
            .collect();
        let local = |tile: Tile| (tile.0 + x, tile.1 + y);
        let rooms = sim_core::rooms::detect(
            width + 2,
            height + 2,
            1,
            width * height,
            |tile| {
                boundary.contains(&local(tile)) || crate::runtime::rooms::bounds(res, local(tile))
            },
            |tile| local(tile) == door_tile,
            |tile| boundary.contains(&local(tile)),
        );
        if rooms.len() != 1 {
            continue;
        }
        let enclosed_stock: Vec<_> = stock
            .iter()
            .copied()
            .filter(|&tile| inside(tile, x, y, width, height))
            .collect();
        let Some(room) = rooms.into_iter().find(|room| {
            room.built
                && !room.doors.is_empty()
                && enclosed_stock
                    .iter()
                    .all(|tile| room.tiles.contains(&(tile.0 - x, tile.1 - y)))
                && free
                    .iter()
                    .filter(|tile| room.tiles.contains(&(tile.0 - x, tile.1 - y)))
                    .count()
                    >= (beds + service_cells) as usize
        }) else {
            continue;
        };
        free.retain(|tile| room.tiles.contains(&(tile.0 - x, tile.1 - y)));
        free.sort_by_key(|&tile| (Reverse(distance(tile, hearth)), tile.1, tile.0));
        let reserved: BTreeSet<_> = boundary
            .iter()
            .copied()
            .chain(free.iter().copied().take(beds as usize))
            .collect();
        if !clearing.is_empty() {
            let tiles = clearing
                .into_iter()
                .filter(|tile| !res.colony.designations.contains_key(tile))
                .collect::<Vec<_>>();
            return HousePlan {
                reserved,
                commands: if tiles.is_empty() {
                    Vec::new()
                } else {
                    vec![Command::DesignateChopTiles { tiles }]
                },
            };
        }
        free.sort_by_key(|&tile| (Reverse(distance(tile, hearth)), tile.1, tile.0));
        let mut commands: Vec<_> = free
            .into_iter()
            .take(beds as usize)
            .map(|(x, y)| Command::PlaceBlueprint { kind: bed, x, y })
            .collect();
        commands.extend(border);
        // Static geometry is stable across transient people. Ordinary placement still waits
        // until every intended bed and boundary tile passes the real person-aware validator.
        if commands.iter().any(|command| match command {
            Command::PlaceBlueprint { kind, x, y } => {
                crate::commands::check_space_after_tree_clear(
                    res,
                    &actual_people,
                    *kind,
                    (*x, *y),
                    false,
                )
                .is_err()
            }
            _ => false,
        }) {
            return HousePlan {
                commands: Vec::new(),
                reserved,
            };
        }
        return HousePlan { commands, reserved };
    }
    HousePlan::default()
}
fn teach(world: &World, res: &SimResources) -> Vec<Command> {
    for (node, state) in res.colony.tech.states.iter().enumerate() {
        if state.status != TechStatus::AtRisk || state.practitioners == 0 {
            continue;
        }
        let Ok(node) = u8::try_from(node) else {
            continue;
        };
        if res.colony.teach_orders.iter().any(|o| o.knowledge == node) {
            continue;
        }
        let teacher = world
            .query::<(&Agent, &Knowledge)>()
            .without::<&Child>()
            .iter()
            .filter(|(_, (_, k))| k.practises(node))
            .map(|(_, (a, _))| a.id)
            .min();
        let Some(teacher) = teacher else {
            continue;
        };
        let learner = world
            .query::<(
                &Agent,
                Option<&Knowledge>,
                Option<&Temperament>,
                Option<&Life>,
            )>()
            .without::<&Child>()
            .iter()
            .filter(|(_, (a, k, _, _))| {
                a.id != teacher
                    && !k.is_some_and(|k| k.practises(node))
                    && !res.colony.teach_orders.iter().any(|o| o.learner == a.id)
                    && !res
                        .steward
                        .vetoed(&Target::Teach(a.id, node), res.current_tick)
            })
            .map(|(_, (a, _, t, l))| {
                (
                    Reverse(t.map_or(0, |t| t.ns)),
                    Reverse(l.map_or(i64::MIN, |l| l.born_tick)),
                    a.id,
                )
            })
            .min();
        if let Some((_, _, learner)) = learner {
            return vec![Command::OrderTeach {
                teacher,
                learner,
                knowledge: node,
            }];
        }
    }
    Vec::new()
}

fn daily_fuel(res: &SimResources) -> u32 {
    use crate::runtime::settlement::firewood::{burn_hours, night_tick_at, winter_burn_per_hour};
    let per_hearth = res.rules.society.firewood.map_or(0, |rule| {
        u32::from(winter_burn_per_hour(
            res,
            &rule,
            night_tick_at(res, &rule, res.current_tick),
        ))
        .saturating_mul(burn_hours(&rule))
    });
    crate::runtime::settlement::fire::effective_burn_per_hour(res, per_hearth)
        .saturating_mul(res.settlements.len().max(1) as u32)
}
fn winter_daily_fuel(res: &SimResources) -> u32 {
    res.rules
        .society
        .firewood
        .map_or(0, |rule| {
            u32::from(rule.burn_per_hour[3])
                .saturating_mul(crate::runtime::settlement::firewood::burn_hours(&rule))
        })
        .max(daily_fuel(res))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn data_only_stockpile_variant_changes_the_actual_ordinary_order() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled content");
        let engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(29),
            sim_core::rules::WorldRules::colony(29),
        );
        let mut rule = sim_core::colony::steward::rules().expect("rules").rules[0].clone();
        let before = plan(&engine.world, &engine.resources, &rule);
        rule.id = 71;
        rule.action = Action::Stockpile {
            base_cells: 64,
            divisor: 100,
        };
        let after = plan(&engine.world, &engine.resources, &rule);
        assert_ne!(before, after);
        let Some(Command::AddStockpile { x0, y0, x1, y1 }) = after.first() else {
            panic!("stockpile candidate");
        };
        assert_eq!((x1 - x0 + 1) * (y1 - y0 + 1), 64);
    }
}

#[cfg(test)]
mod rule_coverage {
    use super::*;
    use sim_core::colony::{
        knowledge::{KnowledgeEntry, Source, TechState},
        wildlife::{Animal, AnimalState},
    };
    use sim_core::components::{Inventory, ResourceKind};
    #[test]
    fn all_six_rules_have_a_positive_and_negative_fixture() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled content");
        for rule in &sim_core::colony::steward::rules().expect("rules").rules {
            let mut engine = crate::worldgen::build_world_with(
                &crate::worldgen::WorldGenConfig::colony(29),
                sim_core::rules::WorldRules::colony(29),
            );
            let mut settlement = sim_core::components::Settlement::new_with_id(999, 0);
            settlement.member_agents = engine
                .world
                .query::<&Agent>()
                .iter()
                .map(|(_, agent)| agent.id)
                .collect();
            engine.resources.settlements.insert(999, settlement);
            let people: Vec<_> = engine
                .world
                .query::<&Agent>()
                .without::<&Child>()
                .iter()
                .map(|(e, a)| (a.id, e))
                .collect();
            let all = Knowledge::from_entries(
                (0..knowledge::defs().nodes.len())
                    .map(|id| KnowledgeEntry::new(id as u8, Source::Oral, 1000, 0))
                    .collect(),
            );
            for &(_, entity) in &people {
                engine
                    .world
                    .insert_one(entity, all.clone())
                    .expect("knowledge");
            }
            match &rule.action {
                Action::Hunt { animal, .. } => {
                    for settlement in engine.resources.settlements.values_mut() {
                        settlement.stockpile.insert(ResourceKind::Food, 0);
                    }
                    for (_, inventory) in engine.world.query::<&mut Inventory>().iter() {
                        let count = inventory.get(ResourceKind::Food);
                        inventory.remove(ResourceKind::Food, count);
                    }
                    let tile = hearth_tiles(&engine.resources)[0];
                    engine.resources.colony.wildlife.animals.insert(
                        999,
                        Animal {
                            id: 999,
                            kind: *animal,
                            hp: 100,
                            state: AnimalState::Wander,
                            pos: tile,
                            pack: None,
                            born_tick: 0,
                            path: Vec::new(),
                            goal: None,
                            home: Some(tile),
                        },
                    );
                }
                Action::Teach => {
                    engine
                        .world
                        .insert_one(people[0].1, Knowledge::default())
                        .expect("learner");
                    engine.resources.colony.tech.states = vec![TechState {
                        status: TechStatus::AtRisk,
                        practitioners: 1,
                        holders: 1,
                        ..Default::default()
                    }];
                }
                _ => {}
            }
            let commands = plan(&engine.world, &engine.resources, rule);
            assert!(!commands.is_empty(), "positive rule {}", rule.id);
            match &rule.action {
                Action::Stockpile { .. } => {
                    for command in commands {
                        assert!(
                            crate::commands::apply_command(
                                &mut engine.world,
                                &mut engine.resources,
                                command
                            )
                            .accepted
                        );
                    }
                }
                Action::Wood { .. } => {
                    for settlement in engine.resources.settlements.values_mut() {
                        settlement.stockpile.insert(ResourceKind::Wood, u32::MAX);
                    }
                }
                Action::House { bed, .. } => {
                    engine.resources.colony.space_sites.insert((1, 1), *bed);
                }
                Action::Bills { .. } => {
                    for &(_, entity) in &people {
                        engine
                            .world
                            .insert_one(entity, Knowledge::default())
                            .expect("no production knowledge");
                    }
                }
                Action::Hunt { .. } => {
                    for settlement in engine.resources.settlements.values_mut() {
                        settlement.stockpile.insert(ResourceKind::Food, u32::MAX);
                    }
                }
                Action::Teach => {
                    engine.resources.colony.tech.states[0].status = TechStatus::Stable;
                }
            }
            assert!(
                plan(&engine.world, &engine.resources, rule).is_empty(),
                "negative rule {}",
                rule.id
            );
        }
    }
}

#[cfg(test)]
mod shared_home_tests {
    use super::*;
    #[test]
    fn compact_shared_geometry_waits_for_people_then_issues_ordinary_orders() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled content");
        for seed in [1, 6, 7] {
            let mut e = crate::worldgen::build_world_with(
                &crate::worldgen::WorldGenConfig::colony(seed),
                sim_core::rules::WorldRules::colony(seed),
            );
            let rules = sim_core::colony::steward::rules().expect("rules");
            for c in plan(&e.world, &e.resources, &rules.rules[0]) {
                assert!(crate::commands::apply_command(&mut e.world, &mut e.resources, c).accepted);
            }
            // Completed-harvest fixture retains natural terrain, nodes and fixed buildings.
            e.resources.colony.trees.clear();
            let hearth = hearth_tiles(&e.resources)[0];
            for (_, p) in e.world.query::<&mut Position>().iter() {
                p.x = hearth.0;
                p.y = hearth.1;
            }
            let start = std::time::Instant::now();
            let ready = house(
                &e.world,
                &e.resources,
                hearth,
                &rules.rules[2].action,
                false,
            );
            println!(
                "D26 planner seed={seed} elapsed_us={}",
                start.elapsed().as_micros()
            );
            let walls: Vec<_> = ready
                .commands
                .iter()
                .filter_map(|c| match c {
                    Command::PlaceBlueprint { kind, x, y } if kind.is_wall() => Some((*x, *y)),
                    _ => None,
                })
                .collect();
            let x0 = walls.iter().map(|t| t.0).min().expect("wall");
            let x1 = walls.iter().map(|t| t.0).max().expect("wall");
            let y0 = walls.iter().map(|t| t.1).min().expect("wall");
            let y1 = walls.iter().map(|t| t.1).max().expect("wall");
            assert_eq!(
                (x1 - x0 - 1) * (y1 - y0 - 1),
                40,
                "compact envelope seed={seed}"
            );
            assert!(e
                .resources
                .colony
                .zone_of_tile
                .keys()
                .chain(std::iter::once(&hearth))
                .all(|&(x, y)| x > x0 && x < x1 && y > y0 && y < y1));
            let person = e
                .world
                .query::<&Position>()
                .iter()
                .next()
                .expect("person")
                .0;
            {
                let mut p = e.world.get::<&mut Position>(person).expect("position");
                p.x = walls[0].0;
                p.y = walls[0].1;
            }
            let wait = house(
                &e.world,
                &e.resources,
                hearth,
                &rules.rules[2].action,
                false,
            );
            assert!(wait.commands.is_empty(), "wait for transient person");
            assert_eq!(wait.reserved, ready.reserved);
            {
                let mut p = e.world.get::<&mut Position>(person).expect("position");
                p.x = hearth.0;
                p.y = hearth.1;
            }
            let cleared = house(
                &e.world,
                &e.resources,
                hearth,
                &rules.rules[2].action,
                false,
            );
            assert_eq!(cleared.reserved, ready.reserved);
            assert!(!cleared.commands.is_empty());
            for c in cleared.commands {
                assert!(crate::commands::apply_command(&mut e.world, &mut e.resources, c).accepted);
            }
        }
    }
    #[test]
    fn tree_preparation_waits_for_ordinary_harvest() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled content");
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(29),
            sim_core::rules::WorldRules::colony(29),
        );
        let rules = sim_core::colony::steward::rules().expect("rules");
        for command in plan(&engine.world, &engine.resources, &rules.rules[0]) {
            assert!(
                crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                    .accepted
            );
        }
        // Isolate preparation from which naturally generated template currently ranks first.
        engine.resources.colony.trees.clear();
        let hearth = hearth_tiles(&engine.resources)[0];
        // Preparation is independent of transient person clearance, covered separately above.
        for (_, position) in engine.world.query::<&mut Position>().iter() {
            position.x = hearth.0;
            position.y = hearth.1;
        }
        let clean = plan(&engine.world, &engine.resources, &rules.rules[2]);
        let tree_tile = clean
            .iter()
            .find_map(|command| match command {
                Command::PlaceBlueprint { x, y, .. } if distance((*x, *y), hearth) > 4 => {
                    Some((*x, *y))
                }
                _ => None,
            })
            .expect("template reaches outside protected camp");
        engine
            .resources
            .colony
            .trees
            .insert(tree_tile, sim_core::colony::Tree { logs: 3 });
        let preparation = plan(&engine.world, &engine.resources, &rules.rules[2]);
        let Some(Command::DesignateChopTiles { tiles }) = preparation.first() else {
            panic!("room preparation must be ordinary tree marks");
        };
        let tiles = tiles.clone();
        assert!(!tiles.is_empty());
        for command in preparation {
            assert!(
                crate::commands::apply_command(
                    &mut engine.world,
                    &mut engine.resources,
                    Command::Steward {
                        rule: rules.rules[2].id,
                        command: Box::new(command)
                    }
                )
                .accepted
            );
        }
        assert!(
            engine.resources.steward.cooldowns[&rules.rules[2].id] > engine.resources.current_tick
        );
        let reserved = house(
            &engine.world,
            &engine.resources,
            hearth,
            &rules.rules[2].action,
            true,
        )
        .reserved;
        assert!(
            !reserved.is_empty(),
            "prep cooldown keeps prospective house cells reserved"
        );
        let production = plan(&engine.world, &engine.resources, &rules.rules[3]);
        for command in production {
            if let Command::PlaceBlueprint { kind, x, y } = command {
                let (size, _, _) = kind.plan(&engine.resources.rules.colony.expect("colony"));
                assert!((y..y + size.1)
                    .all(|yy| (x..x + size.0).all(|xx| !reserved.contains(&(xx, yy)))));
                assert!(!reserved.contains(&sim_core::colony::workshop::work_tile_of((x, y), size)));
            }
        }
        assert!(
            plan(&engine.world, &engine.resources, &rules.rules[2]).is_empty(),
            "wait while designated trees still stand"
        );
        // Completed-harvest fixture: only the designated trees have disappeared.
        for tile in tiles {
            engine.resources.colony.trees.remove(&tile);
            engine.resources.colony.designations.remove(&tile);
        }
        let house = plan(&engine.world, &engine.resources, &rules.rules[2]);
        assert!(!house.is_empty());
        for command in house {
            assert!(matches!(command, Command::PlaceBlueprint { .. }));
            assert!(
                crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                    .accepted
            );
        }
    }
    #[test]
    fn shared_first_home_preserves_storage_and_workshop_outputs_are_usable() {
        sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
            .expect("bundled content");
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(29),
            sim_core::rules::WorldRules::colony(29),
        );
        // Clear, level-ground fixture isolates enclosure preference from site availability.
        engine.resources.colony.trees.clear();
        engine.resources.colony.nodes.clear();
        let hearth = hearth_tiles(&engine.resources)[0];
        for (_, position) in engine.world.query::<&mut Position>().iter() {
            position.x = hearth.0;
            position.y = hearth.1;
        }
        for terrain in &mut engine.resources.tile_grid.terrain_type {
            *terrain = sim_core::TerrainType::Plain;
        }
        let nav = crate::runtime::nav::build(&engine.resources);
        engine.resources.nav.install(nav);
        let rules = sim_core::colony::steward::rules().expect("rules");
        let stock = plan(&engine.world, &engine.resources, &rules.rules[0]);
        for cmd in stock {
            assert!(
                crate::commands::apply_command(&mut engine.world, &mut engine.resources, cmd)
                    .accepted
            );
        }
        let zones = engine.resources.colony.zones.clone();
        let house = plan(&engine.world, &engine.resources, &rules.rules[2]);
        assert!(!house.is_empty());
        let walls: Vec<_> = house
            .iter()
            .filter_map(|cmd| match cmd {
                Command::PlaceBlueprint { kind, x, y } if kind.is_wall() => Some((*x, *y)),
                _ => None,
            })
            .collect();
        let x0 = walls.iter().map(|t| t.0).min().expect("wall");
        let x1 = walls.iter().map(|t| t.0).max().expect("wall");
        let y0 = walls.iter().map(|t| t.1).min().expect("wall");
        let y1 = walls.iter().map(|t| t.1).max().expect("wall");
        assert!(
            engine
                .resources
                .colony
                .zone_of_tile
                .keys()
                .any(|&(x, y)| x > x0 && x < x1 && y > y0 && y < y1),
            "existing storage is enclosed: bounds={x0},{y0}..{x1},{y1}, stock={:?}, hearth={:?}",
            engine
                .resources
                .colony
                .zone_of_tile
                .keys()
                .collect::<Vec<_>>(),
            hearth_tiles(&engine.resources)
        );
        for cmd in house {
            assert!(
                crate::commands::apply_command(&mut engine.world, &mut engine.resources, cmd)
                    .accepted
            );
        }
        assert_eq!(
            zones, engine.resources.colony.zones,
            "no relocation or cancellation"
        );
        let production = plan(&engine.world, &engine.resources, &rules.rules[3]);
        let (&kind, x, y) = production
            .iter()
            .find_map(|cmd| match cmd {
                Command::PlaceBlueprint { kind, x, y } => Some((kind, *x, *y)),
                _ => None,
            })
            .expect("workshop");
        let (size, _, _) = kind.plan(&engine.resources.rules.colony.expect("colony"));
        let work = sim_core::colony::workshop::work_tile_of((x, y), size);
        let hearth = hearth_tiles(&engine.resources)[0];
        assert!(
            engine.resources.colony.zone_of_tile.contains_key(&work)
                || distance(work, hearth) <= crate::runtime::jobs::hearth::HEARTH_PILE_REACH
        );
    }
}
