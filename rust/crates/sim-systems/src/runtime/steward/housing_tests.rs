//! Physical housing regressions: natural enclosure does not replace constructed walls.
use super::*;
use sim_core::rooms::{Room, RoomRole};

fn fixture() -> (sim_engine::SimEngine, Action, Tile) {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled content");
    let mut engine = crate::worldgen::build_world_with(
        &crate::worldgen::WorldGenConfig::colony(4),
        sim_core::rules::WorldRules::colony(4),
    );
    let rules = sim_core::colony::steward::rules().expect("rules");
    for command in plan(&engine.world, &engine.resources, &rules.rules[0]) {
        assert!(
            crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                .accepted
        );
    }
    engine.resources.colony.trees.clear();
    engine.resources.content_state.tree_materials.clear();
    let hearth = hearth_tiles(&engine.resources)[0];
    for (_, position) in engine.world.query::<&mut Position>().iter() {
        position.x = hearth.0;
        position.y = hearth.1;
    }
    let mut action = rules.rules[2].action.clone();
    if let Action::House {
        hearth_refuge_width,
        ..
    } = &mut action
    {
        *hearth_refuge_width = None;
    }
    (engine, action, hearth)
}

#[test]
fn natural_enclosure_beds_do_not_clear_physical_housing_deficit() {
    let (mut engine, action, hearth) = fixture();
    assert!(
        !house(&engine.world, &engine.resources, hearth, &action, false)
            .reserved
            .is_empty()
    );
    let count = engine.world.query::<&Agent>().iter().count() as u32;
    let tiles: BTreeSet<_> = (2..2 + count).map(|x| (x, 2)).collect();
    for &tile in &tiles {
        engine
            .resources
            .colony
            .beds
            .insert(tile, sim_core::colony::materials::OAK);
        engine.resources.colony.room_of_tile.insert(tile, 999);
    }
    engine.resources.colony.rooms.insert(
        999,
        Room {
            id: 999,
            tiles,
            doors: BTreeSet::from([(2, 1)]),
            roofed: true,
            role: RoomRole::Bedroom,
            dug: false,
        },
    );
    assert!(
        !house(&engine.world, &engine.resources, hearth, &action, false)
            .reserved
            .is_empty(),
        "roofed beds without a physical perimeter must leave housing demand"
    );
}

#[test]
fn planned_house_has_complete_constructed_cardinal_boundary() {
    let (engine, action, hearth) = fixture();
    let ready = house(&engine.world, &engine.resources, hearth, &action, false);
    assert!(!ready.commands.is_empty(), "fixture must produce a house");
    let Action::House {
        wall, door, bed, ..
    } = action
    else {
        panic!("house")
    };
    let mut boundary = BTreeSet::new();
    let mut doors = BTreeSet::new();
    let mut beds = BTreeSet::new();
    for command in ready.commands {
        if let Command::PlaceBlueprint { kind, x, y } = command {
            if kind == wall || kind == door {
                boundary.insert((x, y));
            }
            if kind == door {
                doors.insert((x, y));
            }
            if kind == bed {
                beds.insert((x, y));
            }
        }
    }
    let grid = &engine.resources.tile_grid;
    let rooms = sim_core::rooms::detect(
        grid.width,
        grid.height,
        1,
        256,
        |tile| boundary.contains(&tile) || crate::runtime::rooms::bounds(&engine.resources, tile),
        |tile| doors.contains(&tile),
        |tile| boundary.contains(&tile),
    );
    let room = rooms
        .iter()
        .find(|room| beds.is_subset(&room.tiles))
        .expect("planned bedroom");
    assert!(!beds.is_empty());
    for &(x, y) in &room.tiles {
        for tile in [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)] {
            if !room.tiles.contains(&tile) {
                assert!(
                    boundary.contains(&tile)
                        || grid.wall_material[grid.idx(tile.0, tile.1)].is_some(),
                    "natural boundary {tile:?} cannot substitute for a built wall"
                );
            }
        }
    }
}

// Derive the expected roof from emitted construction, independently of planner reservations.
fn planned_roof(res: &SimResources, action: &Action, commands: &[Command]) -> BTreeSet<Tile> {
    let Action::House { wall, door, .. } = action else {
        panic!("house action")
    };
    let mut boundary = BTreeSet::new();
    let mut doors = BTreeSet::new();
    for command in commands {
        if let Command::PlaceBlueprint { kind, x, y } = command {
            if kind == wall || kind == door {
                boundary.insert((*x, *y));
            }
            if kind == door {
                doors.insert((*x, *y));
            }
        }
    }
    assert!(!boundary.is_empty());
    let rooms = sim_core::rooms::detect(
        res.tile_grid.width,
        res.tile_grid.height,
        1,
        256,
        |tile| boundary.contains(&tile) || crate::runtime::rooms::bounds(res, tile),
        |tile| doors.contains(&tile),
        |tile| boundary.contains(&tile),
    );
    let mut covered = boundary;
    for room in rooms
        .into_iter()
        .filter(|room| room.built && !room.doors.is_empty())
    {
        covered.extend(room.tiles);
    }
    covered
}

#[test]
fn cultivation_avoids_future_house_and_accepted_site_work_tiles() {
    let (mut engine, _, hearth) = fixture();
    let action = sim_core::colony::steward::rules().expect("rules").rules[2]
        .action
        .clone();
    engine.resources.food.plants.clear();
    let proposed = house(&engine.world, &engine.resources, hearth, &action, false);
    let reserved = planned_roof(&engine.resources, &action, &proposed.commands);
    assert!(!reserved.is_empty());
    let pieces = super::reserved_house_tiles(&engine.world, &engine.resources, hearth);
    let empty_interior: BTreeSet<_> = reserved
        .difference(&pieces)
        .copied()
        .filter(|tile| crate::runtime::cultivation::commands::zoneable(&engine.resources, *tile))
        .collect();
    assert!(
        !empty_interior.is_empty(),
        "expected roof must contain eligible soil beyond reserved pieces"
    );
    let field_reservation =
        super::reserved_field_house_tiles(&engine.world, &engine.resources, hearth);
    assert!(
        reserved.is_subset(&field_reservation),
        "field reservation must include independently detected complete future roof"
    );
    assert!(empty_interior.is_subset(&field_reservation));
    let rules = sim_core::colony::steward::rules().expect("rules");
    let policy = rules
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(policy) => Some(policy),
            _ => None,
        })
        .expect("food policy");
    let fields = |world: &hecs::World, res: &sim_engine::SimResources| {
        crate::runtime::steward::food::plan(world, res, policy)
            .into_iter()
            .filter_map(|command| {
                if let Command::ZoneField { min, .. } = command {
                    Some(min)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    };
    let initial = fields(&engine.world, &engine.resources);
    assert!(
        !initial.is_empty(),
        "unrelated outdoor ground remains available"
    );
    assert!(initial.iter().all(|tile| !reserved.contains(tile)));
    let occupied = initial[0];
    engine.world.spawn((ColonySite {
        kind: sim_core::colony::BlueprintKind::WoodBench,
        top_left: occupied,
        size: (1, 1),
    },));
    let work = initial[1];
    engine
        .resources
        .colony
        .site_work_tiles
        .insert(occupied, work);
    let remaining = fields(&engine.world, &engine.resources);
    assert!(!remaining.is_empty());
    assert!(remaining
        .iter()
        .all(|tile| !reserved.contains(tile) && *tile != occupied && *tile != work));
}

#[test]
fn later_house_envelope_preserves_active_and_erased_established_fields() {
    use sim_core::colony::cultivation::{FieldPhase, FieldTile, FieldWork};
    for (active, phase) in [
        (true, FieldPhase::Untilled),
        (false, FieldPhase::Sown { planted_day: 0 }),
    ] {
        let (mut engine, action, hearth) = fixture();
        let before = house(&engine.world, &engine.resources, hearth, &action, true);
        let tile = *before
            .reserved
            .iter()
            .next()
            .expect("initial house footprint");
        let crop = sim_core::colony::content::installed_or_bundled()
            .crop_by_name("emmer")
            .expect("crop");
        engine.resources.cultivation.fields.insert(
            tile,
            FieldTile {
                owner: 1,
                crop,
                phase,
                active,
                work: FieldWork::Auto,
                tended: false,
            },
        );
        let after = house(&engine.world, &engine.resources, hearth, &action, true);
        assert!(
            !after.reserved.is_empty(),
            "another house site remains possible"
        );
        assert!(
            !after.reserved.contains(&tile),
            "actual next planned envelope must avoid retained field land"
        );
        assert_eq!(engine.resources.cultivation.fields[&tile].phase, phase);
    }
}

#[test]
fn cultivation_waits_for_accepted_house_roof_without_changing_existing_fields() {
    let (mut engine, action, hearth) = fixture();
    engine.resources.food.plants.clear();
    let proposed = house(&engine.world, &engine.resources, hearth, &action, false);
    let roof = planned_roof(&engine.resources, &action, &proposed.commands);
    let pieces: BTreeSet<_> = proposed
        .commands
        .iter()
        .filter_map(|command| {
            if let Command::PlaceBlueprint { x, y, .. } = command {
                Some((*x, *y))
            } else {
                None
            }
        })
        .collect();
    assert!(
        roof.iter().any(|tile| !pieces.contains(tile)
            && crate::runtime::cultivation::commands::zoneable(&engine.resources, *tile)),
        "fixture needs empty future roof interior, not merely wall or bed cells"
    );
    for command in proposed.commands {
        assert!(
            crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                .accepted
        );
    }
    assert!(!engine.resources.colony.space_sites.is_empty());
    assert!(
        roof.iter().any(|tile| !pieces.contains(tile)
            && crate::runtime::cultivation::commands::zoneable(&engine.resources, *tile)),
        "accepted construction still leaves physically open interior"
    );
    let rules = sim_core::colony::steward::rules().expect("rules");
    let policy = rules
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(policy) => Some(policy),
            _ => None,
        })
        .expect("food policy");
    let crop = policy.cultivation.as_ref().expect("cultivation").crop;
    let outside = (1..engine.resources.tile_grid.height - 1)
        .flat_map(|y| (1..engine.resources.tile_grid.width - 1).map(move |x| (x, y)))
        .find(|tile| {
            !roof.contains(tile)
                && crate::runtime::cultivation::commands::zoneable(&engine.resources, *tile)
        })
        .expect("unrelated outdoor field");
    engine.resources.cultivation.fields.insert(
        outside,
        sim_core::colony::cultivation::FieldTile {
            owner: 1,
            crop,
            phase: sim_core::colony::cultivation::FieldPhase::Sown { planted_day: 0 },
            active: false,
            work: sim_core::colony::cultivation::FieldWork::Paused,
            tended: true,
        },
    );
    let before = engine.resources.cultivation.fields.clone();
    let commands = crate::runtime::steward::food::plan(&engine.world, &engine.resources, policy);
    assert!(
        !commands
            .iter()
            .any(|command| matches!(command, Command::ZoneField { .. })),
        "pending house pieces must prevent new fields inside their future roof"
    );
    assert_eq!(
        engine.resources.cultivation.fields, before,
        "existing erased established player crop must be preserved"
    );
}

#[test]
fn later_house_does_not_indirectly_roof_outdoor_fields() {
    use sim_core::colony::cultivation::{FieldPhase, FieldTile, FieldWork};
    let (mut engine, action, hearth) = fixture();
    engine
        .resources
        .tile_grid
        .terrain_type
        .fill(sim_core::TerrainType::Plain);
    engine.resources.colony.nodes.clear();
    let initial = house(&engine.world, &engine.resources, hearth, &action, false);
    let Action::House { wall, door, .. } = &action else {
        panic!("house action")
    };
    let boundary = |commands: &[Command]| {
        commands
            .iter()
            .filter_map(|command| match command {
                Command::PlaceBlueprint { kind, x, y } if kind == wall || kind == door => {
                    Some((*x, *y))
                }
                _ => None,
            })
            .collect::<BTreeSet<_>>()
    };
    let original_boundary = boundary(&initial.commands);
    let right = original_boundary
        .iter()
        .map(|tile| tile.0)
        .max()
        .expect("house right edge");
    let top = original_boundary
        .iter()
        .map(|tile| tile.1)
        .min()
        .expect("house top edge");
    assert!(right + 4 < engine.resources.tile_grid.width);
    assert!(original_boundary.contains(&(right, top + 3)));
    let fields: BTreeSet<_> = (top + 1..=top + 2)
        .flat_map(|y| (right + 1..=right + 2).map(move |x| (x, y)))
        .collect();
    // The fourth side is still open: only the prospective house closes this pocket.
    for tile in (right + 1..=right + 3)
        .flat_map(|x| [(x, top), (x, top + 3)])
        .chain((top + 1..=top + 2).map(|y| (right + 3, y)))
    {
        let index = engine.resources.tile_grid.idx(tile.0, tile.1);
        engine.resources.tile_grid.wall_material[index] = Some(sim_core::colony::materials::OAK);
    }
    let roof = |res: &SimResources, proposed: &BTreeSet<Tile>| {
        let space = res
            .rules
            .colony
            .and_then(|rule| rule.space)
            .expect("space rules");
        sim_core::rooms::detect(
            res.tile_grid.width,
            res.tile_grid.height,
            space.room_min_tiles,
            space.room_max_tiles,
            |tile| proposed.contains(&tile) || crate::runtime::rooms::bounds(res, tile),
            |_| false,
            |tile| {
                proposed.contains(&tile)
                    || res.tile_grid.wall_material[res.tile_grid.idx(tile.0, tile.1)].is_some()
            },
        )
        .into_iter()
        .filter(|room| room.built)
        .flat_map(|room| room.tiles)
        .collect::<BTreeSet<_>>()
    };
    assert!(
        fields.is_disjoint(&roof(&engine.resources, &BTreeSet::new())),
        "fields initially outdoors"
    );
    assert!(
        fields.is_subset(&roof(&engine.resources, &original_boundary)),
        "original house closes a separate doorless field room"
    );
    let crop = sim_core::colony::content::installed_or_bundled()
        .crop_by_name("emmer")
        .expect("crop");
    for &tile in &fields {
        engine.resources.cultivation.fields.insert(
            tile,
            FieldTile {
                owner: 1,
                crop,
                phase: FieldPhase::Fallow,
                active: true,
                work: FieldWork::Auto,
                tended: false,
            },
        );
    }
    let after = house(&engine.world, &engine.resources, hearth, &action, false);
    let after_boundary = boundary(&after.commands);
    assert!(!after_boundary.is_empty(), "another house remains possible");
    assert!(
        fields.is_disjoint(&roof(&engine.resources, &after_boundary)),
        "house must not indirectly roof occupied outdoor fields"
    );
    for tile in fields {
        assert!(engine.resources.cultivation.fields[&tile].active);
        assert_eq!(
            engine.resources.cultivation.fields[&tile].phase,
            FieldPhase::Fallow
        );
    }
}

#[test]
fn indirect_field_roof_uses_room_size_and_open_boundary_semantics() {
    let (mut engine, _, _) = fixture();
    engine
        .resources
        .tile_grid
        .terrain_type
        .fill(sim_core::TerrainType::Plain);
    engine.resources.tile_grid.wall_material.fill(None);
    for tile in (9..=13)
        .flat_map(|x| [(x, 9), (x, 13)])
        .chain((10..=12).map(|y| (9, y)))
    {
        let index = engine.resources.tile_grid.idx(tile.0, tile.1);
        engine.resources.tile_grid.wall_material[index] = Some(sim_core::colony::materials::OAK);
    }
    let boundary = BTreeSet::from([(13, 10), (13, 11), (13, 12)]);
    let fields = [(10, 10)];
    assert!(roofs_outdoor_fields(
        &engine.resources,
        &boundary,
        &fields,
        9
    ));
    assert!(!roofs_outdoor_fields(
        &engine.resources,
        &boundary,
        &fields,
        8
    ));
    assert!(!roofs_outdoor_fields(
        &engine.resources,
        &BTreeSet::new(),
        &fields,
        120
    ));
    // A missing existing wall leaves a path to the map edge, not a roof.
    let opening = engine.resources.tile_grid.idx(9, 10);
    engine.resources.tile_grid.wall_material[opening] = None;
    assert!(!roofs_outdoor_fields(
        &engine.resources,
        &boundary,
        &fields,
        120
    ));
    // A final one-cell pocket is below room_min, but an earlier partial shell
    // can roof it together with neighboring land before this divider is built.
    for tile in [(19, 20), (20, 19), (20, 21)] {
        let index = engine.resources.tile_grid.idx(tile.0, tile.1);
        engine.resources.tile_grid.wall_material[index] = Some(sim_core::colony::materials::OAK);
    }
    let space = engine
        .resources
        .rules
        .colony
        .and_then(|rule| rule.space)
        .expect("space rules");
    assert!(space.room_min_tiles > 1);
    assert!(roofs_outdoor_fields(
        &engine.resources,
        &BTreeSet::from([(21, 20)]),
        &[(20, 20)],
        space.room_max_tiles,
    ));
}

#[test]
fn covered_beds_elsewhere_do_not_suppress_primary_hearth_refuge() {
    let (mut engine, mut action, hearth) = fixture();
    let count = engine.world.query::<&Agent>().iter().count() as u32;
    let tiles: BTreeSet<_> = (2..2 + count).map(|x| (x, 2)).collect();
    assert!(!tiles.contains(&hearth));
    engine.resources.colony.beds.clear();
    for &tile in &tiles {
        engine
            .resources
            .colony
            .beds
            .insert(tile, sim_core::colony::materials::OAK);
        engine.resources.colony.room_of_tile.insert(tile, 999);
    }
    for tile in (1..=count + 2)
        .flat_map(|x| [(x, 1), (x, 3)])
        .chain([(1, 2), (count + 2, 2)])
    {
        let i = engine.resources.tile_grid.idx(tile.0, tile.1);
        engine.resources.tile_grid.wall_material[i] = Some(sim_core::colony::materials::OAK);
    }
    let door = engine.resources.tile_grid.idx(1, 2);
    engine.resources.tile_grid.set_door(door, true);
    engine.resources.colony.rooms.insert(
        999,
        Room {
            id: 999,
            tiles,
            doors: [(1, 2)].into_iter().collect(),
            roofed: true,
            role: RoomRole::Bedroom,
            dug: false,
        },
    );
    assert!(
        house(&engine.world, &engine.resources, hearth, &action, false)
            .commands
            .is_empty()
    );
    let Action::House {
        hearth_refuge_width,
        bed,
        ..
    } = &mut action
    else {
        unreachable!()
    };
    *hearth_refuge_width = Some(3);
    let bed = *bed;
    let ready = house(&engine.world, &engine.resources, hearth, &action, false);
    assert!(!ready.commands.is_empty());
    let Action::House { door, .. } = action else {
        unreachable!()
    };
    let boundary: BTreeSet<_> = ready
        .commands
        .iter()
        .filter_map(|command| match command {
            Command::PlaceBlueprint { x, y, .. } => Some((*x, *y)),
            _ => None,
        })
        .collect();
    let doors: BTreeSet<_> = ready
        .commands
        .iter()
        .filter_map(|command| match command {
            Command::PlaceBlueprint { kind, x, y } if *kind == door => Some((*x, *y)),
            _ => None,
        })
        .collect();
    let detected = sim_core::rooms::detect(
        engine.resources.tile_grid.width,
        engine.resources.tile_grid.height,
        2,
        120,
        |tile| boundary.contains(&tile) || crate::runtime::rooms::bounds(&engine.resources, tile),
        |tile| doors.contains(&tile),
        |tile| boundary.contains(&tile),
    );
    let refuge = detected
        .iter()
        .find(|room| room.tiles.contains(&hearth))
        .expect("actual primary component");
    assert!(refuge.tiles.len() >= 9 && refuge.built && !refuge.doors.is_empty());
    assert!(
        refuge
            .tiles
            .iter()
            .any(|&(x, y)| [(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)]
                .into_iter()
                .any(|tile| !boundary.contains(&tile)
                    && crate::runtime::rooms::bounds(&engine.resources, tile))),
        "fixture retains a natural boundary instead of deleting terrain"
    );
    eprintln!("primary-hearth refuge regression: {} actual interior tiles, {} ordinary construction orders", refuge.tiles.len(), ready.commands.len());
    let terrain = engine.resources.tile_grid.terrain_type.clone();
    let stock = engine.resources.colony.zone_of_tile.clone();
    let buildings = engine.resources.building_kinds.clone();
    assert!(planned_roof(&engine.resources, &action, &ready.commands).contains(&hearth));
    assert!(!ready.commands.iter().any(|command| matches!(command,
        Command::PlaceBlueprint { kind, .. } if *kind == bed)));
    for command in ready.commands {
        assert!(
            crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                .accepted
        );
    }
    assert_eq!(engine.resources.tile_grid.terrain_type, terrain);
    assert_eq!(engine.resources.colony.zone_of_tile, stock);
    assert_eq!(engine.resources.building_kinds, buildings);
}

#[test]
fn primary_hearth_refuge_requires_existing_construction_knowledge() {
    let (engine, mut action, hearth) = fixture();
    let Action::House {
        hearth_refuge_width,
        ..
    } = &mut action
    else {
        unreachable!()
    };
    *hearth_refuge_width = Some(3);
    for (_, knowledge) in engine.world.query::<&mut Knowledge>().iter() {
        *knowledge = Default::default();
    }
    assert!(
        house(&engine.world, &engine.resources, hearth, &action, false)
            .commands
            .is_empty()
    );
}

#[test]
fn untouched_initial_world_can_plan_primary_hearth_refuge() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("content");
    let rules = sim_core::colony::steward::rules().expect("rules");
    let mut blocked = Vec::new();
    for seed in [0, 4] {
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(seed),
            sim_core::rules::WorldRules::colony(seed),
        );
        assert!(
            crate::commands::submit(
                &mut engine,
                Command::SetSteward {
                    on: true,
                    scope: sim_core::colony::steward::StewardScope::All,
                }
            )
            .accepted
        );
        for command in plan(&engine.world, &engine.resources, &rules.rules[0]) {
            assert!(
                crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                    .accepted
            );
        }
        let trees: BTreeSet<_> = engine.resources.colony.trees.keys().copied().collect();
        let nodes: BTreeSet<_> = engine.resources.colony.nodes.keys().copied().collect();
        let stock = engine.resources.colony.zone_of_tile.clone();
        let hearth = hearth_tiles(&engine.resources)[0];
        let pending = house(
            &engine.world,
            &engine.resources,
            hearth,
            &rules.rules[2].action,
            false,
        );
        let commands = plan(&engine.world, &engine.resources, &rules.rules[2]);
        let waiting = commands.is_empty();
        eprintln!(
            "untouched plan seed={seed} reserved={} covered={}",
            pending.reserved.len(),
            pending.covered.len()
        );
        eprintln!(
            "untouched refuge seed={seed} trees={} nodes={} commands={commands:?}",
            trees.len(),
            nodes.len()
        );
        for command in commands {
            assert!(
                crate::commands::apply_command(&mut engine.world, &mut engine.resources, command)
                    .accepted
            );
        }
        assert_eq!(
            engine
                .resources
                .colony
                .trees
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            trees
        );
        assert_eq!(
            engine
                .resources
                .colony
                .nodes
                .keys()
                .copied()
                .collect::<BTreeSet<_>>(),
            nodes
        );
        assert_eq!(engine.resources.colony.zone_of_tile, stock);
        if waiting {
            if pending.reserved.is_empty() {
                blocked.push(seed);
                continue;
            }
            // A person can occupy the chosen wall at tick zero. Preserve that
            // geometry and let actual movement and the ordinary steward run.
            for _ in 0..=360 {
                engine.tick();
                if engine
                    .resources
                    .steward
                    .owned
                    .iter()
                    .any(|order| order.rule == rules.rules[2].id)
                {
                    break;
                }
            }
            let issued = engine
                .resources
                .steward
                .owned
                .iter()
                .any(|order| order.rule == rules.rules[2].id);
            eprintln!(
                "untouched delayed work seed={seed} tick={} accepted={issued}",
                engine.current_tick()
            );
            if !issued {
                blocked.push(seed);
            }
        }
    }
    assert!(
        blocked.is_empty(),
        "initial refuge cannot issue ordinary work: {blocked:?}"
    );
}
