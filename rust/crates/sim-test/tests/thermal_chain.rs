//! Genuine production scene: ordinary construction and paid fuel make a warm dwelling.
//! Explicit winter start, knowledge and raw supplies are scenario setup, not discovery.
//! WORLDSIM_THERMAL_SCENE_OUT optionally exports real milestone saves outside the repo.
use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::{BillMode, BlueprintKind, Command, ItemKind, ItemSpec, RecipeKind};
use sim_core::components::{Agent, Cold, Position, ResourceKind};
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::food::items::{create_ground, FoodSource};
use sim_systems::runtime::jobs::invariants;
use sim_systems::runtime::{rooms::room_at, thermal};

const DAY: u64 = 1440;
type Tile = (u32, u32);

fn command(e: &mut SimEngine, command: Command) {
    let result = submit(e, command.clone());
    assert!(
        result.accepted,
        "rejected scene order {command:?}: {result:?}"
    );
}

fn place(e: &mut SimEngine, kind: BlueprintKind, tile: Tile) {
    if kind.is_space()
        && sim_systems::commands::check_space(
            &e.resources,
            &std::collections::BTreeSet::new(),
            kind,
            tile,
        )
        .is_err()
    {
        println!("SITE_BLOCK tile={tile:?} kind={kind:?} tree={} node={:?} occupied={:?} building={:?} work={:?}",
            e.resources.colony.trees.contains_key(&tile), e.resources.colony.nodes.get(&tile),
            e.resources.colony.occupied.get(&tile), e.resources.building_kinds.get(&tile),
            e.resources.colony.workshops.values().filter(|s| s.work_tile()==tile).collect::<Vec<_>>());
    }

    command(
        e,
        Command::PlaceBlueprint {
            kind,
            x: tile.0,
            y: tile.1,
        },
    );
}

fn checkpoint(e: &mut SimEngine, name: &str, home: Tile) {
    invariants::check(&e.world, &e.resources).expect("physical ledger");
    println!(
        "THERMAL_STEP {name} tick={} outdoor={:?} place={:?} rooms={:?} doors={:?}",
        e.current_tick(),
        thermal::outdoor_temperature(&e.resources),
        thermal::place_temperature(&e.resources, home),
        e.resources.thermal.as_ref().map(|t| &t.rooms),
        e.resources.thermal.as_ref().map(|t| &t.doors)
    );
    if let Some(directory) = std::env::var_os("WORLDSIM_THERMAL_SCENE_OUT") {
        let directory = std::path::PathBuf::from(directory);
        assert!(directory.is_absolute() && directory.is_dir());
        let bytes = sim_systems::save::save(e).expect("genuine scene save");
        use std::io::Write;
        std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(directory.join(format!("{name}.wsave")))
            .expect("new external scene")
            .write_all(&bytes)
            .expect("scene bytes");
    }
}

fn until(e: &mut SimEngine, deadline: u64, label: &str, ready: impl Fn(&SimEngine) -> bool) {
    while e.current_tick() < deadline && !ready(e) {
        e.tick();
    }
    assert!(
        ready(e),
        "missing {label} at tick{}; thermal={:?}; workshops={:?}",
        e.current_tick(),
        e.resources.thermal,
        e.resources.colony.workshops
    );
}

fn continuity(e: &mut SimEngine) {
    let bytes = sim_systems::save::save(e).expect("partial thermal save");
    let mut resumed = sim_systems::save::load(&bytes).expect("partial thermal load");
    for _ in 0..120 {
        e.tick();
        resumed.tick();
    }
    assert_eq!(
        sim_systems::save::save(e).expect("continued"),
        sim_systems::save::save(&mut resumed).expect("resumed"),
        "partial door/hour/deficiency history resumes byte-exactly"
    );
}

#[test]
fn production_scene_builds_heats_crosses_cools_and_refuels_a_dwelling() {
    // Given: full registered game world, explicit winter and knowledgeable founders.
    let mut e = sim_test::colony_engine(0);
    e.resources.rules.clock_offset_ticks = 18 * DAY + 480;
    let entries: Vec<_> = knowledge::defs()
        .nodes
        .iter()
        .enumerate()
        .map(|(i, _)| KnowledgeEntry::new(i as u8, Source::Seeded, 1000, 0))
        .collect();
    let mut people: Vec<_> = e
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, a)| (a.id, entity))
        .collect();
    people.sort_by_key(|&(id, _)| id);
    for &(_, entity) in &people {
        e.world
            .insert_one(entity, Knowledge::from_entries(entries.clone()))
            .expect("starting knowledge");
    }
    command(
        &mut e,
        Command::SetSteward {
            on: false,
            scope: Default::default(),
        },
    );
    command(&mut e, Command::SetDressOrder { on: false });
    until(&mut e, DAY, "actual settlement", |e| {
        !e.resources.settlements.is_empty()
    });
    let home = e
        .resources
        .settlements
        .values()
        .min_by_key(|s| s.settlement_id)
        .expect("primary settlement")
        .formation_tile;
    assert!(thermal::outdoor_temperature(&e.resources).is_some_and(|t| t < 50));
    assert!(
        e.world.query::<&Cold>().iter().count() > 0,
        "actual outdoor Cold"
    );
    checkpoint(&mut e, "00-outside", home);
    let stock = (home.0 + 4, home.1 - 7);
    command(
        &mut e,
        Command::AddStockpile {
            x0: stock.0,
            y0: stock.1,
            x1: stock.0 + 3,
            y1: stock.1 + 3,
        },
    );
    for (name, count) in [("log", 120), ("stone", 10), ("hazelnuts", 200)] {
        let kind = ItemKind::from_name(name).expect("raw scene content");
        create_ground(
            &mut e.resources,
            stock,
            ItemSpec::raw(kind, None),
            count,
            18,
            FoodSource::Debug,
        )
        .expect("accounted raw stores");
    }
    // Clear the building perimeter by actual player movement; walls cannot cover people.
    for &(agent, _) in &people {
        command(
            &mut e,
            Command::SetDrafted {
                agent,
                drafted: true,
            },
        );
        command(
            &mut e,
            Command::DraftMove {
                agent,
                x: stock.0,
                y: stock.1,
            },
        );
    }
    until(
        &mut e,
        DAY,
        "founders physically clear the building site",
        |e| {
            people.iter().all(|&(_, entity)| {
                e.world
                    .get::<&Position>(entity)
                    .is_ok_and(|p| (p.x, p.y) == stock)
            })
        },
    );
    for &(agent, _) in &people {
        command(
            &mut e,
            Command::SetDrafted {
                agent,
                drafted: false,
            },
        );
    }
    let mut shapes: Vec<_> = (5..=9_u32)
        .flat_map(|h| (5..=9_u32).map(move |w| (w, h)))
        .collect();
    shapes.sort_by_key(|&(w, h)| (w * h, h, w));
    let mut selected = None;
    'geometry: for (width, height) in shapes {
        for dy in 1..height - 1 {
            for dx in 1..width - 1 {
                let (left, top) = (home.0 - dx, home.1 - dy);
                let (right, bottom) = (left + width - 1, top + height - 1);
                let legal = (top..=bottom).all(|y| {
                    (left..=right).all(|x| {
                        if x != left && x != right && y != top && y != bottom {
                            return true;
                        }
                        let kind = if (x, y) == (home.0, bottom) {
                            BlueprintKind::Door
                        } else {
                            BlueprintKind::LogWall
                        };
                        sim_systems::commands::check_space(
                            &e.resources,
                            &std::collections::BTreeSet::new(),
                            kind,
                            (x, y),
                        )
                        .is_ok()
                    })
                });
                if legal {
                    selected = Some(((left, top), (right, bottom)));
                    break 'geometry;
                }
            }
        }
    }
    let (origin, end) = selected
        .expect("smallest legal enclosure around the actual hearth and existing structures");
    let door = (home.0, end.1);
    let bed = (origin.1 + 1..end.1)
        .flat_map(|y| (origin.0 + 1..end.0).map(move |x| (x, y)))
        .find(|&tile| {
            sim_systems::commands::check_space(
                &e.resources,
                &std::collections::BTreeSet::new(),
                BlueprintKind::Bedroll,
                tile,
            )
            .is_ok()
        })
        .expect("buildable bed within the legal shell");
    println!(
        "THERMAL_GEOMETRY home={home:?} origin={origin:?} end={end:?} door={door:?} bed={bed:?}"
    );
    // When: normal construction orders create an enclosure around the actual hearth.
    for y in origin.1..=end.1 {
        for x in origin.0..=end.0 {
            if x == origin.0 || x == end.0 || y == origin.1 || y == end.1 {
                place(
                    &mut e,
                    if (x, y) == door {
                        BlueprintKind::Door
                    } else {
                        BlueprintKind::LogWall
                    },
                    (x, y),
                );
            }
        }
    }
    let bench = (home.0 + 4, home.1);
    place(
        &mut e,
        BlueprintKind::from_name("wood_bench").expect("bench"),
        bench,
    );
    until(&mut e, 2 * DAY, "constructed bench", |e| {
        e.resources
            .colony
            .workshops
            .values()
            .any(|s| s.top_left == bench)
    });
    let shop = e
        .resources
        .colony
        .workshops
        .values()
        .find(|s| s.top_left == bench)
        .expect("bench")
        .id;
    // Auto-bills are suspended through their public command before the bounded scene bill.
    let old: Vec<_> = e.resources.colony.workshops[&shop]
        .bills
        .iter()
        .map(|b| b.id)
        .collect();
    for bill in old {
        command(
            &mut e,
            Command::SetBillSuspended {
                workshop: shop,
                bill,
                suspended: true,
            },
        );
    }
    until(&mut e, 2 * DAY, "constructed enclosure", |e| {
        room_at(&e.resources, home).is_some_and(|r| r.roofed)
    });
    let floors: Vec<_> = room_at(&e.resources, home)
        .expect("built room")
        .tiles
        .iter()
        .copied()
        .filter(|&tile| {
            sim_systems::commands::check_space(
                &e.resources,
                &std::collections::BTreeSet::new(),
                BlueprintKind::PackedFloor,
                tile,
            )
            .is_ok()
        })
        .collect();
    assert!(floors.contains(&bed), "bed has a player-floorable tile");
    for &tile in &floors {
        place(&mut e, BlueprintKind::PackedFloor, tile);
    }
    until(&mut e, 3 * DAY, "ordinary packed floors", |e| {
        floors.iter().all(|&(x, y)| {
            e.resources.tile_grid.floor_material[e.resources.tile_grid.idx(x, y)].is_some()
        })
    });
    place(&mut e, BlueprintKind::Bedroll, bed);
    until(&mut e, 3 * DAY, "actual bed", |e| {
        e.resources.colony.beds.contains_key(&bed)
    });
    checkpoint(&mut e, "01-built", home);
    command(
        &mut e,
        Command::AddBill {
            workshop: shop,
            recipe: RecipeKind::from_name("split_firewood").expect("fuel recipe"),
            mode: BillMode::Make(6),
        },
    );
    let bill = e.resources.colony.workshops[&shop]
        .bills
        .last()
        .expect("scene fuel bill")
        .id;
    let (person, entity) = people[0];
    command(
        &mut e,
        Command::SetDrafted {
            agent: person,
            drafted: true,
        },
    );
    command(
        &mut e,
        Command::DraftMove {
            agent: person,
            x: bed.0,
            y: bed.1,
        },
    );
    until(&mut e, 4 * DAY, "paid warmth and dwelling upgrade", |e| {
        thermal::dwelling_view(&e.world, &e.resources, bed).is_some_and(|v| {
            v.floor_coverage_permille == 1000
                && v.paid_heat
                && v.warm_occupants
                && v.stage.is_some_and(|s| s >= 3)
        }) && e.world.get::<&Cold>(entity).is_err()
            && e.world
                .get::<&Position>(entity)
                .is_ok_and(|p| (p.x, p.y) == bed)
    });
    checkpoint(&mut e, "02-warmed", home);
    let warm = thermal::place_temperature(&e.resources, home).expect("warm room");
    assert!(e
        .resources
        .thermal
        .as_ref()
        .expect("thermal")
        .paid_hearths
        .values()
        .any(|p| p.requested_wood > 0 && p.debited_wood == p.requested_wood));
    command(
        &mut e,
        Command::DraftMove {
            agent: person,
            x: door.0,
            y: door.1 + 1,
        },
    );
    until(&mut e, 4 * DAY, "actual door passage", |e| {
        e.resources
            .thermal
            .as_ref()
            .expect("thermal")
            .doors
            .get(&door)
            .is_some_and(|d| d.open_until_tick > e.resources.current_tick)
    });
    checkpoint(&mut e, "03-door", home);
    continuity(&mut e);
    command(
        &mut e,
        Command::DraftMove {
            agent: person,
            x: bed.0,
            y: bed.1,
        },
    );
    command(
        &mut e,
        Command::SetBillSuspended {
            workshop: shop,
            bill,
            suspended: true,
        },
    );
    until(
        &mut e,
        5 * DAY,
        "physical fuel exhaustion and cooling",
        |e| {
            assert!(
                e.world.get::<&Agent>(entity).is_ok(),
                "scene observer died tick{} record={:?}; living={}; people={:?}",
                e.current_tick(),
                e.resources.chronicle.person(person),
                e.world.query::<&Agent>().iter().count(),
                e.resources.chronicle.people().collect::<Vec<_>>()
            );
            if e.current_tick() % 60 == 1 {
                println!(
                    "COOL_DIAG tick={} wood={:?} dwelling={:?} cold={} person={:?} alive={}",
                    e.current_tick(),
                    e.resources
                        .settlements
                        .values()
                        .map(|s| s.stockpile.get(&ResourceKind::Wood))
                        .collect::<Vec<_>>(),
                    thermal::dwelling_view(&e.world, &e.resources, bed),
                    e.world.get::<&Cold>(entity).is_ok(),
                    e.world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)),
                    e.world.get::<&Agent>(entity).is_ok()
                );
            }
            e.resources
                .settlements
                .values()
                .all(|s| s.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0) == 0)
                && thermal::dwelling_view(&e.world, &e.resources, bed)
                    .is_some_and(|v| !v.paid_heat && v.reason == "fuel")
                && thermal::place_temperature(&e.resources, home)
                    .is_some_and(|t| t < warm && t < 50)
                && e.world.get::<&Cold>(entity).is_ok()
        },
    );
    checkpoint(&mut e, "04-cold", home);
    continuity(&mut e);
    command(
        &mut e,
        Command::SetBillMode {
            workshop: shop,
            bill,
            mode: BillMode::Make(6),
        },
    );
    command(
        &mut e,
        Command::SetBillSuspended {
            workshop: shop,
            bill,
            suspended: false,
        },
    );
    until(&mut e, 6 * DAY, "physical refuel recovery", |e| {
        thermal::dwelling_view(&e.world, &e.resources, bed)
            .is_some_and(|v| v.paid_heat && v.warm_occupants)
            && e.world.get::<&Cold>(entity).is_err()
    });
    checkpoint(&mut e, "05-refueled", home);
    command(
        &mut e,
        Command::SetBillMode {
            workshop: shop,
            bill,
            mode: BillMode::Maintain(24),
        },
    );
    command(
        &mut e,
        Command::SetDrafted {
            agent: person,
            drafted: false,
        },
    );
    let stable_deadline = e.current_tick() + DAY;
    until(
        &mut e,
        stable_deadline,
        "stable earned dwelling before floor removal",
        |e| {
            assert!(
                e.world.get::<&Agent>(entity).is_ok(),
                "same observer survives normal needs"
            );
            thermal::dwelling_view(&e.world, &e.resources, bed).is_some_and(|v| {
                v.stage.is_some_and(|stage| stage >= 3)
                    && v.candidate == v.stage
                    && v.deficiency_deadline.is_none()
                    && v.paid_heat
                    && v.warm_occupants
            })
        },
    );
    let earned = thermal::dwelling_view(&e.world, &e.resources, bed)
        .expect("dwelling")
        .stage;
    let room_id = room_at(&e.resources, bed).expect("physical room").id;
    let improvements_before = e
        .resources
        .chronicle
        .entries()
        .iter()
        .filter(|event| {
            event.kind == sim_core::chronicle::ChronicleKind::DwellingImproved
                && event.value == i64::from(room_id)
        })
        .count();
    let removed = floors
        .iter()
        .copied()
        .find(|&tile| {
            tile != bed
                && sim_systems::runtime::jobs::space::built_at(&e.resources, tile)
                    == Some(BlueprintKind::PackedFloor)
        })
        .expect("eligible floor with no bed or structure above it");
    command(
        &mut e,
        Command::DesignateDeconstruct {
            x0: removed.0,
            y0: removed.1,
            x1: removed.0,
            y1: removed.1,
        },
    );
    let removal_deadline = e.current_tick() + DAY;
    let removed_index = e.resources.tile_grid.idx(removed.0, removed.1);
    while e.resources.tile_grid.floor_material[removed_index].is_some() {
        assert!(
            e.current_tick() < removal_deadline,
            "ordinary floor deconstruction completes"
        );
        assert!(
            e.world.get::<&Agent>(entity).is_ok(),
            "same observer survives before floor removal"
        );
        let view =
            thermal::dwelling_view(&e.world, &e.resources, bed).expect("dwelling before removal");
        assert!(view.roofed && e.resources.colony.beds.contains_key(&bed));
        assert_eq!(view.floor_coverage_permille, 1000);
        assert_eq!(
            view.stage, earned,
            "earned stage stable before actual floor removal"
        );
        assert_eq!(
            view.candidate, earned,
            "no earlier fuel, occupancy or food deficiency"
        );
        assert_eq!(
            view.deficiency_deadline, None,
            "no mixed-cause pending downgrade before removal"
        );
        assert_eq!(
            e.resources.thermal.as_ref().expect("thermal").dwellings[&bed].deficient_since_tick,
            None,
            "deconstruction cannot inherit an existing timer"
        );
        e.tick();
    }
    let removed_tick = e.resources.current_tick;
    let view = thermal::dwelling_view(&e.world, &e.resources, bed).expect("dwelling after removal");
    assert_eq!(view.reason, "floor");
    assert_eq!(view.candidate, Some(1));
    assert_eq!(
        view.stage, earned,
        "floor removal starts a delay, not an immediate downgrade"
    );
    let state = e.resources.thermal.as_ref().expect("thermal");
    let since = state.dwellings[&bed]
        .deficient_since_tick
        .expect("actual deficiency began");
    assert_eq!(
        since, removed_tick,
        "the newly started timer matches the first actual floor disappearance tick"
    );
    println!("THERMAL_FLOOR_REMOVED tick={removed_tick} since={since} prior_deficiency=none");
    let delay = state
        .rules
        .stages
        .iter()
        .find(|stage| Some(stage.id) == earned)
        .expect("earned authored stage")
        .downgrade_delay_ticks;
    let deadline = since + delay;
    assert!(
        e.resources.current_tick < deadline,
        "downgrade is actually delayed"
    );
    // Save a real partially elapsed deficiency; never edit its timer or earned stage.
    e.tick();
    checkpoint(&mut e, "06-floor-pending", home);
    let partial = sim_systems::save::save(&mut e).expect("partial deficiency save");
    let mut resumed = sim_systems::save::load(&partial).expect("partial deficiency load");
    let mut paid_hours = 0;
    let mut min_fuel = u32::MAX;
    while e.resources.current_tick < deadline {
        let fuel = e
            .resources
            .settlements
            .values()
            .map(|s| s.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0))
            .sum::<u32>()
            + sim_systems::runtime::jobs::hearth::feedable_firewood(&e.resources);
        min_fuel = min_fuel.min(fuel);
        assert!(
            fuel > 0,
            "ordinary sustained production keeps physical fuel available"
        );
        assert!(
            e.world.get::<&Agent>(entity).is_ok(),
            "same observer survives floor deficiency"
        );
        assert!(e.resources.colony.beds.contains_key(&bed));
        assert!(e.resources.tile_grid.floor_material
            [e.resources.tile_grid.idx(removed.0, removed.1)]
        .is_none());
        let view = thermal::dwelling_view(&e.world, &e.resources, bed).expect("physical dwelling");
        assert!(view.roofed && view.floor_coverage_permille < 1000);
        assert_eq!(view.reason, "floor");
        assert_eq!(view.candidate, Some(1));
        assert_eq!(
            view.stage, earned,
            "earned stage persists before authored deadline"
        );
        assert_eq!(view.deficiency_deadline, Some(deadline));
        if e.resources.current_tick % 60 == 0 && view.paid_heat {
            paid_hours += 1;
        }
        e.tick();
        resumed.tick();
    }
    let downgraded =
        thermal::dwelling_view(&e.world, &e.resources, bed).expect("downgraded dwelling");
    assert_eq!(
        downgraded.stage,
        Some(1),
        "actual downgrade at authored deadline"
    );
    assert_eq!(downgraded.deficiency_deadline, None);
    assert!(
        e.world.get::<&Agent>(entity).is_ok(),
        "observer alive at downgrade"
    );
    assert_eq!(
        sim_systems::save::save(&mut e).expect("continued deficiency"),
        sim_systems::save::save(&mut resumed).expect("resumed deficiency")
    );
    println!(
        "THERMAL_DOWNGRADE since={since} delay={delay} deadline={deadline} paid_hours={paid_hours} min_fuel={min_fuel}"
    );
    checkpoint(&mut e, "07-downgraded", home);
    place(&mut e, BlueprintKind::PackedFloor, removed);
    let restore_deadline = e.current_tick() + DAY;
    until(
        &mut e,
        restore_deadline,
        "ordinary floor rebuilding and actual stage recovery",
        |e| {
            assert!(
                e.world.get::<&Agent>(entity).is_ok(),
                "same observer survives recovery"
            );
            thermal::dwelling_view(&e.world, &e.resources, bed).is_some_and(|v| {
                v.stage.is_some_and(|stage| stage >= 3)
                    && v.floor_coverage_permille == 1000
                    && v.deficiency_deadline.is_none()
                    && v.paid_heat
                    && v.warm_occupants
            })
        },
    );
    assert!(
        e.resources
            .chronicle
            .entries()
            .iter()
            .filter(
                |event| event.kind == sim_core::chronicle::ChronicleKind::DwellingImproved
                    && event.value == i64::from(room_id)
            )
            .count()
            > improvements_before,
        "restoring this dwelling emits a new genuine improvement chronicle"
    );
    checkpoint(&mut e, "08-restored", home);

    assert!(
        e.resources
            .chronicle
            .entries()
            .iter()
            .any(|event| event.kind == sim_core::chronicle::ChronicleKind::DwellingImproved),
        "genuine house chronicle"
    );
}
