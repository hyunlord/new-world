//! Frozen pre-optimization housing oracle and exact preparation equivalence checks.
use super::*;

fn reference_house(
    world: &World,
    res: &SimResources,
    hearth: Tile,
    action: &Action,
    reserve_only: bool,
) -> HousePlan {
    let mut profile =
        super::super::house_profile::HouseProfile::new(res.current_tick, reserve_only);
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
                .is_some_and(|room| super::super::housing::constructed(res, room))
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
    profile.phase(1);
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
    profile.phase(2);
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
    profile.phase(3);
    profile.candidates(locations.len());
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
    profile.phase(4);
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
        profile.visit();
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
                    // Reuse constructed walls, but never substitute terrain for a house wall.
                    if res.tile_grid.wall_material[res.tile_grid.idx(xx, yy)].is_none() {
                        valid = false;
                        break;
                    }
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
                covered: BTreeSet::new(),
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
                covered: BTreeSet::new(),
            };
        }
        return HousePlan {
            commands,
            reserved,
            covered: BTreeSet::new(),
        };
    }
    HousePlan::default()
}

fn reference_capacity(res: &SimResources, bed: sim_core::colony::BlueprintKind) -> Vec<usize> {
    let people = BTreeSet::new();
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
    capacity
}

fn engine(seed: u64) -> sim_engine::SimEngine {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(seed),
        sim_core::rules::WorldRules::colony(seed),
    )
}

#[test]
fn house_preparation_matches_every_capacity_cell_origin_and_ranking_key() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let data = sim_core::colony::steward::rules().expect("rules");
    let rule = data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::House { .. }))
        .expect("house");
    let Action::House { bed, .. } = rule.action else {
        panic!("house")
    };
    for seed in [0, 1, 6, 7] {
        let mut e = engine(seed);
        e.resources.nav = sim_core::nav::NavState::default();
        let hearth = hearth_tiles(&e.resources)[0];
        for phase in 0..6 {
            if phase == 1 {
                crate::runtime::nav::ensure(&mut e.resources);
            }
            if phase == 2 {
                let stock = data
                    .rules
                    .iter()
                    .find(|r| matches!(r.action, Action::Stockpile { .. }))
                    .expect("stock");
                for command in plan(&e.world, &e.resources, stock) {
                    assert!(
                        crate::commands::apply_command(&mut e.world, &mut e.resources, command)
                            .accepted
                    );
                }
                e.resources
                    .steward
                    .vetoes
                    .insert(Target::Tile(hearth.0 + 2, hearth.1 + 2), 100);
                let now = e.resources.current_tick;
                e.resources
                    .steward
                    .vetoes
                    .insert(Target::Tile(hearth.0 + 3, hearth.1 + 2), now);
                e.resources
                    .steward
                    .vetoes
                    .insert(Target::Teach(0, 0), now + 1);
                e.resources
                    .steward
                    .vetoes
                    .insert(Target::Tile(u32::MAX, u32::MAX), now + 1);
                let (&tile, &zone) = e
                    .resources
                    .colony
                    .zone_of_tile
                    .iter()
                    .next()
                    .expect("stock tile");
                e.resources
                    .colony
                    .zone_of_tile
                    .insert((u32::MAX, u32::MAX), zone);
                e.resources
                    .steward
                    .vetoes
                    .insert(Target::Tile(tile.0, tile.1), now + 1);
                let work = e
                    .resources
                    .colony
                    .workshops
                    .values()
                    .next()
                    .expect("workshop")
                    .work_tile();
                e.resources.colony.zone_of_tile.insert(work, zone);
                e.resources
                    .steward
                    .vetoes
                    .insert(Target::Tile(work.0, work.1), now + 1);
            }
            if phase == 3 {
                let camp = crate::worldgen::colony::hearth(&e.resources).expect("camp");
                e.resources.nav.set_cost(camp, sim_core::nav::IMPASSABLE);
                assert_eq!(e.resources.nav.region_of(camp), 0);
            }
            if phase == 4 {
                e.resources.building_kinds.clear();
            }
            if phase == 5 {
                e.resources.rules.colony.as_mut().expect("colony").space = None;
            }
            let (capacity, stock_capacity) = house_capacity(&e.resources, bed);
            assert_eq!(
                capacity,
                reference_capacity(&e.resources, bed),
                "capacity seed={seed} phase={phase}"
            );
            let stride = e.resources.tile_grid.width as usize + 1;
            let stock: Vec<_> = e.resources.colony.zone_of_tile.keys().copied().collect();
            for width in [4, 5, 6] {
                let expected: BTreeSet<_> = candidates(&e.resources, hearth)
                    .into_iter()
                    .map(|(x, y)| (x, y, width))
                    .collect();
                assert_eq!(
                    house_origins(&e.resources, width, &BTreeSet::new()).collect::<Vec<_>>(),
                    expected
                        .iter()
                        .map(|&origin| (origin, false))
                        .collect::<Vec<_>>(),
                    "origins seed={seed} phase={phase}"
                );
                for (x, y, _) in expected {
                    for height in [4, 7, 12] {
                        if x + width + 2 >= e.resources.tile_grid.width
                            || y + height + 2 >= e.resources.tile_grid.height
                        {
                            continue;
                        }
                        let count = stock
                            .iter()
                            .filter(|&&(xx, yy)| {
                                xx > x && xx <= x + width && yy > y && yy <= y + height
                            })
                            .count();
                        let expected_key = (
                            Reverse(count),
                            !(hearth.0 > x
                                && hearth.0 <= x + width
                                && hearth.1 > y
                                && hearth.1 <= y + height),
                            width * height,
                            distance((x + width / 2, y + height / 2), hearth),
                            y,
                            x,
                            width,
                        );
                        assert_eq!(
                            house_rank(&stock_capacity, stride, hearth, (x, y, width, height)),
                            expected_key,
                            "rank seed={seed} phase={phase} at={x},{y}"
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn optimized_house_commands_and_reservations_match_frozen_reference() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let data = sim_core::colony::steward::rules().expect("rules");
    let rule = data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::House { .. }))
        .expect("house");
    let stock = data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::Stockpile { .. }))
        .expect("stock");
    for seed in [0, 1, 6, 7] {
        let mut e = engine(seed);
        e.resources.nav = sim_core::nav::NavState::default();
        let hearth = hearth_tiles(&e.resources)[0];
        for phase in 0..3 {
            if phase == 1 {
                crate::runtime::nav::ensure(&mut e.resources);
                for command in plan(&e.world, &e.resources, stock) {
                    assert!(
                        crate::commands::apply_command(&mut e.world, &mut e.resources, command)
                            .accepted
                    );
                }
            }
            if phase == 2 {
                e.resources
                    .steward
                    .vetoes
                    .insert(Target::Tile(hearth.0 + 2, hearth.1 + 2), 100);
            }
            for reserve_only in [false, true] {
                let old =
                    reference_house(&e.world, &e.resources, hearth, &rule.action, reserve_only);
                let new = house(&e.world, &e.resources, hearth, &rule.action, reserve_only);
                assert_eq!(
                    new.commands, old.commands,
                    "commands seed={seed} phase={phase} reserve={reserve_only}"
                );
                assert_eq!(
                    new.reserved, old.reserved,
                    "reserved seed={seed} phase={phase} reserve={reserve_only}"
                );
            }
        }
    }
}
#[test]
fn prepared_capacity_preserves_preview_with_adversarial_tile_contents() {
    use sim_core::colony::materials::OAK;
    use sim_core::colony::{
        BlueprintKind, NodeKind, Occupant, ReserveKey, ResourceNode, Tree, Workshop,
    };
    use sim_core::components::BuildingKind;
    use sim_core::TerrainType;
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let mut e = sim_engine::SimEngine::new(32, 32, sim_core::MaterialRegistry::new());
    e.resources.rules = sim_core::rules::WorldRules::colony(0);
    e.resources.colony.nodes.insert(
        (5, 5),
        ResourceNode {
            kind: NodeKind::Boulder,
            left: 5,
            material: OAK,
        },
    );
    e.resources.colony.nodes.insert(
        (u32::MAX, 5),
        ResourceNode {
            kind: NodeKind::Boulder,
            left: 5,
            material: OAK,
        },
    );
    e.resources.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: BlueprintKind::WoodBench,
            top_left: (5, 4),
            size: (2, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    e.resources.colony.site_work_tiles.insert((1, 1), (7, 5));
    e.resources
        .colony
        .reservations
        .insert(ReserveKey::Work((8, 5)), 1);
    e.resources.colony.trees.insert((9, 5), Tree { logs: 3 });
    e.resources.colony.zone_of_tile.insert((10, 5), 1);
    e.resources.steward.vetoes.insert(Target::Tile(11, 5), 100);
    e.resources.colony.beds.insert((13, 5), OAK);
    e.resources
        .colony
        .occupied
        .insert((21, 5), Occupant::Site((21, 5)));
    e.resources
        .building_kinds
        .insert((22, 5), BuildingKind::Hearth);
    e.resources
        .colony
        .space_sites
        .insert((23, 5), BlueprintKind::LogWall);
    e.resources
        .colony
        .space_sites
        .insert((24, 5), BlueprintKind::Door);
    for key in [
        ReserveKey::Tree((3, 6)),
        ReserveKey::Stack((4, 6)),
        ReserveKey::Cell((5, 6)),
        ReserveKey::Site((6, 6)),
        ReserveKey::Node((7, 6)),
        ReserveKey::Bench((8, 6)),
        ReserveKey::Wood((9, 6)),
        ReserveKey::Animal(3),
    ] {
        e.resources.colony.reservations.insert(key, 1);
    }
    let grid = &mut e.resources.tile_grid;
    let index = |x: usize| 5usize * 32 + x;
    grid.floor_material[index(12)] = Some(OAK);
    for x in [14, 15] {
        grid.wall_material[index(x)] = Some(OAK);
    }
    grid.set_door(index(15), true);
    for x in [16, 17] {
        grid.terrain_type[index(x)] = TerrainType::River;
    }
    grid.set_ford(index(17), true);
    grid.terrain_type[index(18)] = TerrainType::Mountain;
    for x in [19, 20] {
        grid.terrain_type[index(x)] = TerrainType::Hill;
    }
    grid.dug[index(20)] = true;
    for kind in [
        BlueprintKind::Bedroll,
        BlueprintKind::PackedFloor,
        BlueprintKind::LogWall,
    ] {
        let (actual, _) = house_capacity(&e.resources, kind);
        assert_eq!(
            actual,
            reference_capacity(&e.resources, kind),
            "kind={kind:?}"
        );
        let at = |x: u32, y: u32| rectangle_count(&actual, 33, x - 1, y - 1, 1, 1);
        for x in 3..=9 {
            assert_eq!(at(x, 6), 1, "non-Work reserve at {x}");
        }
        assert_eq!(at(8, 5), 0, "Work reservation blocks");
        assert_eq!(at(9, 5), 1, "tree clearance preview ignores tree");
        assert_eq!(at(20, 5), 1, "dug hill is available");
        assert_eq!(at(24, 5), 1, "a planned door has no built wall yet");
        assert_eq!(at(12, 5), usize::from(kind != BlueprintKind::PackedFloor));
        assert_eq!(at(13, 5), usize::from(kind == BlueprintKind::PackedFloor));
    }
}

#[test]
fn house_order_preserves_full_stable_sequence_and_search_prefixes() {
    let (width, height) = (32usize, 24usize);
    let stride = width + 1;
    for stock_pattern in 0..3 {
        let mut stock = vec![0usize; stride * (height + 1)];
        for y in 0..height {
            let mut row = 0;
            for x in 0..width {
                row += usize::from(match stock_pattern {
                    0 => false,
                    1 => (x + y) % 3 == 0,
                    _ => x > 10 && x < 20 && y > 8 && y < 15,
                });
                let i = (y + 1) * stride + x + 1;
                stock[i] = stock[i - stride] + row;
            }
        }
        let mut all = Vec::new();
        for y in 0u32..16 {
            for x in 0u32..24 {
                for w in 4..=6 {
                    for h in 4..=6 {
                        all.push((x, y, w, h));
                    }
                }
            }
        }
        let duplicate = (7, 8, 4, 6);
        for locations in [
            vec![],
            vec![duplicate],
            vec![duplicate, (4, 5, 6, 4), duplicate],
            all,
        ] {
            for home in [(0, 0), (16, 12), (31, 23)] {
                let mut expected = locations.clone();
                expected.sort_by_cached_key(|&at| house_rank(&stock, stride, home, at));
                assert_eq!(
                    house_locations(locations.clone(), &stock, stride, home).collect::<Vec<_>>(),
                    expected
                );
                for accepted in [0, 1, 2, locations.len() / 2, locations.len()] {
                    let mut visited = Vec::new();
                    let selected =
                        house_locations(locations.clone(), &stock, stride, home).find(|&at| {
                            visited.push(at);
                            visited.len() - 1 == accepted
                        });
                    assert_eq!(visited, expected[..(accepted + 1).min(expected.len())]);
                    assert_eq!(selected, expected.get(accepted).copied());
                }
            }
        }
    }
}

#[test]
fn house_order_preserves_occupied_reservations_and_independent_nav_stats() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let data = sim_core::colony::steward::rules().expect("rules");
    let action = &data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::House { .. }))
        .expect("house")
        .action;
    let stock = data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::Stockpile { .. }))
        .expect("stock");
    for occupied in [false, true] {
        for reserve_only in [false, true] {
            let run = |optimized| {
                let mut e = engine(4);
                for command in plan(&e.world, &e.resources, stock) {
                    assert!(
                        crate::commands::apply_command(&mut e.world, &mut e.resources, command)
                            .accepted
                    );
                }
                e.resources.colony.trees.clear();
                e.resources.content_state.tree_materials.clear();
                let hearth = hearth_tiles(&e.resources)[0];
                for (_, position) in e.world.query::<&mut Position>().iter() {
                    position.x = hearth.0;
                    position.y = hearth.1;
                }
                let ready = reference_house(&e.world, &e.resources, hearth, action, false);
                assert!(!ready.commands.is_empty(), "ready fixture");
                assert!(!ready.reserved.is_empty());
                if occupied {
                    let tile = ready
                        .commands
                        .iter()
                        .find_map(|c| match c {
                            Command::PlaceBlueprint { kind, x, y } if kind.is_wall() => {
                                Some((*kind, (*x, *y)))
                            }
                            _ => None,
                        })
                        .expect("wall blueprint in chosen footprint");
                    assert_eq!(
                        crate::commands::check_space_after_tree_clear(
                            &e.resources,
                            &BTreeSet::from([tile.1]),
                            tile.0,
                            tile.1,
                            false,
                        ),
                        Err(sim_core::colony::RejectReason::Person)
                    );
                    let tile = tile.1;
                    for (_, position) in e.world.query::<&mut Position>().iter() {
                        position.x = tile.0;
                        position.y = tile.1;
                    }
                }
                let map = crate::runtime::nav::build(&e.resources);
                e.resources.nav.install(map);
                *e.resources.nav.stats.borrow_mut() = sim_core::nav::NavStats::default();
                let before = *e.resources.nav.stats.borrow();
                let result = if optimized {
                    house(&e.world, &e.resources, hearth, action, reserve_only)
                } else {
                    reference_house(&e.world, &e.resources, hearth, action, reserve_only)
                };
                if occupied {
                    assert!(
                        result.commands.is_empty(),
                        "occupied house must keep its reservation without commands"
                    );
                    assert_eq!(
                        result.reserved, ready.reserved,
                        "do not select a later footprint"
                    );
                }
                let after = *e.resources.nav.stats.borrow();
                (result.commands, result.reserved, before, after)
            };
            assert_eq!(
                run(true),
                run(false),
                "occupied={occupied} reserve={reserve_only}"
            );
        }
    }
}

#[test]
fn capped_wood_counts_logs_in_transit_and_on_shelves_once() {
    use sim_core::colony::{Carry, ItemSpec, ItemStack, Job, JobKind, JobStage, Workshop};
    for owner in 0..5 {
        let mut e = engine(29);
        let data = sim_core::colony::steward::rules().expect("rules");
        let wood = data
            .rules
            .iter()
            .find(|r| matches!(r.action, Action::Wood { .. }))
            .expect("wood");
        let Action::Wood { items, .. } = &wood.action else {
            unreachable!()
        };
        let log = items[0];
        let hearth = hearth_tiles(&e.resources)[0];
        e.resources.colony.ground_items.clear();
        assert!(
            !plan(&e.world, &e.resources, wood).is_empty(),
            "shortage must still request wood"
        );
        let (target, _, _, _, _) = wood_budget_observation(&e.world, &e.resources).expect("target");
        let count = u16::try_from(target).expect("small fixture");
        let spec = ItemSpec::raw(log, None);
        match owner {
            0 => {
                e.resources
                    .colony
                    .ground_items
                    .insert(hearth, ItemStack::of(spec, count));
            }
            1..=3 => {
                let kind = match owner {
                    1 => JobKind::Haul {
                        from: hearth,
                        to: hearth,
                    },
                    2 => JobKind::Deliver {
                        from: hearth,
                        site: hearth,
                    },
                    _ => JobKind::Supply {
                        from: hearth,
                        workshop: 100,
                        to: hearth,
                    },
                };
                e.world.spawn((
                    Carry::of(spec, count),
                    Job {
                        kind,
                        stage: JobStage::ToDest,
                        taken_at: 0,
                        amount: count,
                        item: log,
                        material: None,
                        fetch: None,
                        best_distance: 0,
                        progress_tick: 0,
                    },
                ));
            }
            _ => {
                e.resources.colony.workshops.insert(
                    100,
                    Workshop {
                        id: 100,
                        kind: sim_core::colony::BlueprintKind::WoodBench,
                        top_left: hearth,
                        size: (2, 1),
                        bills: vec![],
                        shelf: vec![ItemStack::of(spec, count)],
                        active: None,
                    },
                );
            }
        }
        assert_eq!(
            wood_budget_observation(&e.world, &e.resources)
                .expect("target")
                .0,
            target
        );
        assert!(
            plan(&e.world, &e.resources, wood).is_empty(),
            "owner={owner}: physical logs already cover the cap"
        );
        assert_eq!(
            wood_supply(&e.world, &e.resources, items, true),
            (target, 0),
            "owner={owner}"
        );
        assert_eq!(
            wood_supply(&e.world, &e.resources, items, false).0,
            items
                .iter()
                .map(|&kind| maintain_count(&e.world, &e.resources, kind))
                .sum::<u32>()
                + crate::runtime::jobs::hearth::settlement_wood(&e.resources),
            "uncapped legacy accounting remains unchanged"
        );
    }
}

#[test]
fn capped_house_skips_new_clearing_when_valid_tree_free_site_exists() {
    let mut e = engine(29);
    let data = sim_core::colony::steward::rules().expect("rules");
    let home = data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::House { .. }))
        .expect("house");
    let wood = data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::Wood { .. }))
        .expect("wood");
    let stock = data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::Stockpile { .. }))
        .expect("stock");
    for command in plan(&e.world, &e.resources, stock) {
        assert!(crate::commands::apply_command(&mut e.world, &mut e.resources, command).accepted);
    }
    e.resources.colony.trees.clear();
    e.resources.content_state.tree_materials.clear();
    e.resources.steward.enabled = true;
    e.resources.steward.extended = true;
    let hearth = hearth_tiles(&e.resources)[0];
    for (_, position) in e.world.query::<&mut Position>().iter() {
        position.x = hearth.0;
        position.y = hearth.1;
    }
    let Action::Wood {
        items,
        camp_radius,
        cap_log_margin,
        ..
    } = &wood.action
    else {
        unreachable!()
    };
    assert!(cap_log_margin.is_some());
    let clean = reference_house(&e.world, &e.resources, hearth, &home.action, false);
    let mut witness = None;
    for command in &clean.commands {
        let Command::PlaceBlueprint { x, y, .. } = command else {
            continue;
        };
        let tile = (*x, *y);
        if distance(tile, hearth) <= *camp_radius {
            continue;
        }
        let previous = e
            .resources
            .steward
            .vetoes
            .insert(Target::Tile(tile.0, tile.1), e.resources.current_tick + 100);
        let alternative = reference_house(&e.world, &e.resources, hearth, &home.action, false);
        e.resources
            .steward
            .vetoes
            .remove(&Target::Tile(tile.0, tile.1));
        if let Some(until) = previous {
            e.resources
                .steward
                .vetoes
                .insert(Target::Tile(tile.0, tile.1), until);
        }
        if !alternative.commands.is_empty() && !alternative.reserved.contains(&tile) {
            witness = Some((tile, alternative));
            break;
        }
    }
    let (tree_tile, alternative) = witness.expect("real later tree-free candidate");
    assert!(!clean.reserved.contains(&hearth) && !alternative.reserved.contains(&hearth));
    e.resources.colony.ground_items.insert(
        hearth,
        sim_core::colony::ItemStack::of(sim_core::colony::ItemSpec::raw(items[0], None), 1000),
    );
    e.resources
        .colony
        .trees
        .insert(tree_tile, sim_core::colony::Tree { logs: 3 });
    e.resources.steward.vetoes.insert(
        Target::Tile(tree_tile.0, tree_tile.1),
        e.resources.current_tick + 100,
    );
    let witness_after_tree = reference_house(&e.world, &e.resources, hearth, &home.action, false);
    assert!(!witness_after_tree.commands.is_empty());
    assert!(witness_after_tree
        .commands
        .iter()
        .all(|c| matches!(c, Command::PlaceBlueprint { .. })));
    e.resources
        .steward
        .vetoes
        .remove(&Target::Tile(tree_tile.0, tree_tile.1));
    let (target, _, _, _, _) = wood_budget_observation(&e.world, &e.resources).expect("cap budget");
    assert!(target <= maintain_count(&e.world, &e.resources, items[0]));
    assert!(!plan(&e.world, &e.resources, wood)
        .iter()
        .any(|c| matches!(c, Command::DesignateChopTiles { .. })));
    let old = reference_house(&e.world, &e.resources, hearth, &home.action, false);
    assert!(
        old.commands.iter().any(
            |c| matches!(c, Command::DesignateChopTiles { tiles } if tiles.contains(&tree_tile))
        ),
        "old candidate should clear {tree_tile:?}"
    );
    let selected = house(&e.world, &e.resources, hearth, &home.action, false);
    assert!(
        !selected.commands.is_empty(),
        "tree-free candidate remains available"
    );
    assert!(
        selected
            .commands
            .iter()
            .all(|c| matches!(c, Command::PlaceBlueprint { .. })),
        "capped house must skip new clearing: {:?}",
        selected.commands
    );
    assert!(!selected.covered.contains(&tree_tile));
    let preview = house(&e.world, &e.resources, hearth, &home.action, true);
    assert_eq!(preview.reserved, selected.reserved);
    assert_eq!(preview.covered, selected.covered);
    // The cap boundary is independent of the intentional commute ranking change.
    let original_tree_tile = tree_tile;
    e.resources.colony.trees.remove(&tree_tile);
    let clean = house(&e.world, &e.resources, hearth, &home.action, false);
    let tree_tile = clean
        .commands
        .iter()
        .find_map(|command| match command {
            Command::PlaceBlueprint { kind, x, y }
                if kind.is_wall() && distance((*x, *y), hearth) > *camp_radius =>
            {
                Some((*x, *y))
            }
            _ => None,
        })
        .expect("selected new wall outside autonomous clearing radius");
    e.resources
        .colony
        .trees
        .insert(tree_tile, sim_core::colony::Tree { logs: 3 });
    let uncapped = reference_house(&e.world, &e.resources, hearth, &home.action, false);
    // One physical tree reaches the boundary exactly; one additional log rejects it.
    let cost: u32 = clean
        .commands
        .iter()
        .filter_map(|command| match command {
            Command::PlaceBlueprint { kind, .. } => Some(
                kind.definition()
                    .needs
                    .iter()
                    .filter(|need| items.contains(&need.item))
                    .map(|need| u32::from(need.count))
                    .sum::<u32>(),
            ),
            _ => None,
        })
        .sum();
    let (_, building, _, margin, fuel) =
        wood_budget_observation(&e.world, &e.resources).expect("budget");
    let candidate_target = building.max(cost) + margin + fuel;
    let (have, marked) = wood_supply(&e.world, &e.resources, items, true);
    assert_eq!(marked, 0);
    let others = have - 1000;
    let stock = u16::try_from(
        candidate_target
            .checked_sub(others + 3)
            .expect("positive headroom"),
    )
    .expect("stock count");
    e.resources
        .colony
        .ground_items
        .get_mut(&hearth)
        .expect("logs")
        .count = stock;
    let (_, _, _, _, after_fuel) = wood_budget_observation(&e.world, &e.resources).expect("budget");
    assert_eq!(
        after_fuel, fuel,
        "stock change must not change executable fuel inputs"
    );
    let exact = house(&e.world, &e.resources, hearth, &home.action, false);
    assert!(exact
        .commands
        .iter()
        .any(|c| matches!(c, Command::DesignateChopTiles { tiles } if tiles.contains(&tree_tile))));
    e.resources
        .colony
        .trees
        .get_mut(&tree_tile)
        .expect("tree")
        .logs = 4;
    let excess = house(&e.world, &e.resources, hearth, &home.action, false);
    assert!(
        !excess.covered.contains(&tree_tile),
        "one log beyond headroom must choose another site"
    );
    e.resources
        .colony
        .ground_items
        .get_mut(&hearth)
        .expect("logs")
        .count = 1000;
    e.resources
        .colony
        .designations
        .insert(tree_tile, sim_core::colony::Designation::Chop);
    let waiting = house(&e.world, &e.resources, hearth, &home.action, false);
    assert_eq!(
        waiting.reserved, clean.reserved,
        "already accepted clearance adds no yield twice"
    );
    assert!(waiting.commands.is_empty());
    e.resources.colony.designations.remove(&tree_tile);
    e.resources.steward.enabled = false;
    assert_eq!(
        house(&e.world, &e.resources, hearth, &home.action, false).reserved,
        uncapped.reserved,
        "inactive cap preserves original candidate choice"
    );
    e.resources.steward.enabled = true;
    // Restore the original tree-free plan's obstacle state after the cap subcase.
    e.resources.colony.trees.remove(&tree_tile);
    e.resources
        .colony
        .trees
        .insert(original_tree_tile, sim_core::colony::Tree { logs: 3 });
    for command in selected.commands {
        assert!(crate::commands::apply_command(&mut e.world, &mut e.resources, command).accepted);
    }
}

#[test]
fn capped_seed5_house_preserves_post_shell_access() {
    let mut e = engine(5);
    for command in [
        Command::SetSteward {
            on: true,
            scope: sim_core::colony::steward::StewardScope::All,
        },
        Command::SetStewardDefence { on: true },
    ] {
        assert!(crate::commands::submit(&mut e, command).accepted);
    }
    let hearth = hearth_tiles(&e.resources)[0];
    e.tick();
    let sites = crate::runtime::jobs::items::colony_sites(&e.world);
    let data = sim_core::colony::steward::rules().expect("rules");
    let (bed, door) = data
        .rules
        .iter()
        .find_map(|rule| match rule.action {
            Action::House { bed, door, .. } => Some((bed, door)),
            _ => None,
        })
        .expect("house rule");
    assert!(sites.values().any(|site| site.site.kind == bed));
    assert!(sites.values().any(|site| site.site.kind == door));
    let region = e.resources.nav.region_of(hearth);
    assert_ne!(region, 0, "accepted shell must preserve the camp tile");
    let inaccessible: Vec<_> = sites
        .values()
        .filter(|site| {
            !e.resources.nav.reaches(region, site.site.top_left)
                || e.resources.nav.path(hearth, site.site.top_left).is_none()
        })
        .map(|site| (site.site.kind, site.site.top_left))
        .collect();
    assert!(
        inaccessible.is_empty(),
        "accepted seed5 shell isolates construction from hearth {hearth:?}: {inaccessible:?}"
    );
}

#[test]
fn capped_seed5_house_avoids_long_camp_detour() {
    let mut e = engine(5);
    assert!(
        crate::commands::submit(
            &mut e,
            Command::SetSteward {
                on: true,
                scope: sim_core::colony::steward::StewardScope::All,
            }
        )
        .accepted
    );
    let hearth = hearth_tiles(&e.resources)[0];
    e.tick();
    let bed = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find_map(|rule| match rule.action {
            Action::House { bed, .. } => Some(bed),
            _ => None,
        })
        .expect("home bed");
    let beds: Vec<_> = crate::runtime::jobs::items::colony_sites(&e.world)
        .values()
        .filter(|site| site.site.kind == bed)
        .map(|site| site.site.top_left)
        .collect();
    assert_eq!(beds.len(), 10);
    for tile in beds {
        let path = e.resources.nav.path(hearth, tile).expect("reachable bed");
        let mut previous = hearth;
        let cost: u32 = path
            .iter()
            .map(|&next| {
                let cost = e.resources.nav.map().step_cost(previous, next);
                previous = next;
                cost
            })
            .sum();
        // The retained first-fit diagnostic costs at least 962; an affordable
        // candidate within 230 further visits costs at most 526 for every bed.
        assert!(
            cost < 962,
            "bed {tile:?} retains the long camp detour: {cost}"
        );
    }
}

#[test]
fn capped_house_commute_choice_keeps_occupied_reservations() {
    let mut e = engine(5);
    assert!(
        crate::commands::submit(
            &mut e,
            Command::SetSteward {
                on: true,
                scope: sim_core::colony::steward::StewardScope::All,
            }
        )
        .accepted
    );
    crate::runtime::nav::ensure(&mut e.resources);
    let data = sim_core::colony::steward::rules().expect("rules");
    for rule in data
        .rules
        .iter()
        .filter(|r| matches!(r.action, Action::Stockpile { .. } | Action::Wood { .. }))
    {
        for command in plan(&e.world, &e.resources, rule) {
            assert!(
                crate::commands::apply_command(&mut e.world, &mut e.resources, command).accepted
            );
        }
    }
    let action = &data
        .rules
        .iter()
        .find(|r| matches!(r.action, Action::House { .. }))
        .expect("home")
        .action;
    let hearth = hearth_tiles(&e.resources)[0];
    let before = *e.resources.nav.stats.borrow();
    let ready = house(&e.world, &e.resources, hearth, action, false);
    let repeat = house(&e.world, &e.resources, hearth, action, false);
    assert_eq!(ready.commands, repeat.commands);
    assert_eq!(ready.reserved, repeat.reserved);
    let wall = ready
        .commands
        .iter()
        .find_map(|command| match command {
            Command::PlaceBlueprint { kind, x, y } if kind.is_wall() => Some((*x, *y)),
            _ => None,
        })
        .expect("ready home wall");
    for (_, position) in e.world.query::<&mut Position>().iter() {
        position.x = wall.0;
        position.y = wall.1;
    }
    for reserve_only in [false, true] {
        let blocked = house(&e.world, &e.resources, hearth, action, reserve_only);
        assert!(blocked.commands.is_empty());
        assert_eq!(blocked.reserved, ready.reserved);
        assert_eq!(blocked.covered, ready.covered);
    }
    assert_eq!(*e.resources.nav.stats.borrow(), before);
}

#[test]
fn merged_house_origins_preserve_full_sorted_union() {
    let e = engine(0);
    for width in [2, 4, 6] {
        for entries in [
            vec![],
            vec![(1, 1, width), (1, 1, width), (2, 2, width)],
            vec![(1, 1, width - 1), (1, 1, width + 1), (2, 1, width)],
            vec![
                (0, 0, 0),
                (u32::MAX, 0, width),
                (0, u32::MAX, width),
                (u32::MAX, u32::MAX, u32::MAX),
            ],
        ] {
            let local: BTreeSet<_> = entries.into_iter().collect();
            let expected: BTreeSet<_> = (1..e.resources.tile_grid.width.saturating_sub(1))
                .flat_map(|x| {
                    (1..e.resources.tile_grid.height.saturating_sub(1)).map(move |y| (x, y, width))
                })
                .chain(local.iter().copied())
                .collect();
            assert_eq!(
                house_origins(&e.resources, width, &local).collect::<Vec<_>>(),
                expected
                    .into_iter()
                    .map(|origin| (origin, local.contains(&origin)))
                    .collect::<Vec<_>>()
            );
        }
    }
}

#[test]
fn item_capacity_predicate_matches_original_expression_on_every_cell() {
    use crate::runtime::jobs::items::{has_wall, terrain};
    use sim_core::{MaterialId, TerrainType};
    for seed in [0, 6] {
        let mut e = engine(seed);
        crate::runtime::nav::ensure(&mut e.resources);
        // Deliberately leave the cached navigation map stale after these mutations.
        for (x, terrain_type) in [
            (1, TerrainType::River),
            (2, TerrainType::Mountain),
            (3, TerrainType::Hill),
            (4, TerrainType::Hill),
        ] {
            let i = e.resources.tile_grid.idx(x, 1);
            e.resources.tile_grid.terrain_type[i] = terrain_type;
            e.resources.tile_grid.set_ford(i, x == 1);
            e.resources.tile_grid.set_dug(i, x == 4);
        }
        for x in [5, 6] {
            let i = e.resources.tile_grid.idx(x, 1);
            e.resources.tile_grid.terrain_type[i] = TerrainType::Plain;
            e.resources.tile_grid.wall_material[i] = Some(MaterialId::from_str_hash("oak"));
            e.resources.tile_grid.set_door(i, x == 6);
        }
        for baseline in [false, true] {
            if baseline {
                e.resources.rules.colony = None;
            }
            let res = &e.resources;
            for tile in (0..res.tile_grid.width)
                .flat_map(|x| (0..res.tile_grid.height).map(move |y| (x, y)))
                .chain([(u32::MAX, 0), (0, u32::MAX), (u32::MAX, u32::MAX)])
            {
                let expected = matches!(terrain(res, tile), Some(t) if t != TerrainType::River && t != TerrainType::Mountain)
                    && !res.building_kinds.contains_key(&tile)
                    && !res.colony.occupied.contains_key(&tile)
                    && (res.rules.colony.is_none()
                        || (crate::runtime::nav::passable(res, tile) && !has_wall(res, tile)));
                assert_eq!(
                    can_hold_items(res, tile),
                    expected,
                    "seed={seed} baseline={baseline} tile={tile:?}"
                );
            }
        }
    }
}
