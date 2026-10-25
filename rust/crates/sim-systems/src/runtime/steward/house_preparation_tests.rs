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
                    house_origins(&e.resources, width),
                    expected,
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
