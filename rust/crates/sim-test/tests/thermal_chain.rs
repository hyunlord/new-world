//! Genuine production scene: ordinary construction and paid fuel make a warm dwelling.
//! Autumn setup gives88 real hours to build and tailor before winter. Starting20cloth
//! is a disclosed Stage2 product; garments are physically tailored and worn.
//! Winter arrives through ordinary ticks, never a clock jump.
//! WORLDSIM_THERMAL_SCENE_OUT optionally exports real milestone saves outside the repo.
use sim_core::colony::knowledge::{self, Knowledge, KnowledgeEntry, Source};
use sim_core::colony::people::Apparel;
use sim_core::colony::{
    BillMode, BlueprintKind, Command, ItemKind, ItemSpec, Job, JobKind, RecipeKind, WorkPriorities,
    WorkType,
};
use sim_core::components::{Agent, Child, Cold, Position, ResourceKind};
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
        "missing {label} at tick{}; thermal={:?}; workshops={:?}; ground={:?}; workforce={:?}",
        e.current_tick(),
        e.resources.thermal,
        e.resources.colony.workshops,
        ItemKind::all()
            .map(|kind| (kind, e.resources.colony.ground_total(kind)))
            .filter(|(_, count)| *count > 0)
            .collect::<Vec<_>>(),
        e.world
            .query::<(&Agent, &Position, Option<&Job>, Option<&WorkPriorities>)>()
            .iter()
            .map(|(_, (agent, pos, job, priorities))| (
                agent.id,
                (pos.x, pos.y),
                job.cloned(),
                priorities.copied()
            ))
            .collect::<Vec<_>>()
    );
}

fn continuity(e: &mut SimEngine) {
    continuity_with_commands(e, &[]);
}

fn continuity_with_commands(e: &mut SimEngine, orders: &[Command]) {
    let bytes = sim_systems::save::save(e).expect("partial thermal save");
    let mut resumed = sim_systems::save::load(&bytes).expect("partial thermal load");
    for order in orders {
        command(e, order.clone());
        command(&mut resumed, order.clone());
    }
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
fn production_scene_prepares_in_autumn_then_tests_winter_heat_and_dwelling_recovery() {
    // Given: full registered game world, autumn day3 08:00 and knowledgeable founders.
    let mut e = sim_test::colony_engine(0);
    e.resources.rules.clock_offset_ticks = 14 * DAY + 480;
    let winter_start = 3 * u64::from(e.resources.rules.season_days()) * e.resources.ticks_per_day;
    let winter_tick = winter_start
        .checked_sub(e.resources.rules.clock_offset_ticks)
        .expect("scene starts before the calendar's winter boundary");
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
    let born_day = u32::try_from(sim_systems::runtime::resource_regen::day_index(
        &e.resources,
    ))
    .expect("actual starting food day");
    for (name, count) in [
        ("log", 120),
        ("stone", 10),
        ("hazelnuts", 200),
        ("linen_cloth", 20),
    ] {
        let kind = ItemKind::from_name(name).expect("raw scene content");
        create_ground(
            &mut e.resources,
            stock,
            ItemSpec::raw(kind, None),
            count,
            born_day,
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
    until(
        &mut e,
        (2 * DAY).min(winter_tick),
        "constructed bench",
        |e| {
            e.resources
                .colony
                .workshops
                .values()
                .any(|s| s.top_left == bench)
        },
    );
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
    until(
        &mut e,
        (2 * DAY).min(winter_tick),
        "constructed enclosure",
        |e| room_at(&e.resources, home).is_some_and(|r| r.roofed),
    );
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
    until(
        &mut e,
        (3 * DAY).min(winter_tick),
        "ordinary packed floors",
        |e| {
            floors.iter().all(|&(x, y)| {
                e.resources.tile_grid.floor_material[e.resources.tile_grid.idx(x, y)].is_some()
            })
        },
    );
    place(&mut e, BlueprintKind::Bedroll, bed);
    until(&mut e, (3 * DAY).min(winter_tick), "actual bed", |e| {
        e.resources.colony.beds.contains_key(&bed)
    });
    // Stage2 product input:20cloth ->10tunics,1200 authored craft ticks.
    // Loom adds4logs/90work; retain the thermal room's existing geometry.
    let loom_kind = BlueprintKind::from_name("loom").expect("loom");
    let mut loom_tiles: Vec<_> = (home.1.saturating_sub(12)..=home.1 + 12)
        .flat_map(|y| (home.0.saturating_sub(12)..=home.0 + 12).map(move |x| (x, y)))
        .collect();
    loom_tiles.sort_by_key(|&(x, y)| (x.abs_diff(home.0).max(y.abs_diff(home.1)), y, x));
    let loom = loom_tiles
        .into_iter()
        .find(|&tile| {
            room_at(&e.resources, tile).is_none()
                && sim_systems::commands::camp_reaches(&e.resources, tile)
                && sim_systems::commands::check_blueprint(&e.resources, loom_kind, tile.0, tile.1)
                    .is_ok()
        })
        .expect("legal loom outside the unchanged thermal room");
    place(&mut e, loom_kind, loom);
    until(&mut e, winter_tick, "ordinary loom construction", |e| {
        e.resources
            .colony
            .workshops
            .values()
            .any(|s| s.top_left == loom)
    });
    let loom_id = e
        .resources
        .colony
        .workshops
        .values()
        .find(|s| s.top_left == loom)
        .expect("built loom")
        .id;
    command(
        &mut e,
        Command::AddStockpile {
            x0: stock.0,
            y0: stock.1,
            x1: stock.0 + 5,
            y1: stock.1 + 5,
        },
    );
    command(
        &mut e,
        Command::AddBill {
            workshop: loom_id,
            recipe: RecipeKind::from_name("make_linen_tunic").expect("tailoring"),
            mode: BillMode::Make(10),
        },
    );
    command(&mut e, Command::SetDressOrder { on: true });
    let tunic = ItemKind::from_name("linen_tunic").expect("tunic");
    until(
        &mut e,
        winter_tick,
        "ten physical tunics and clothed surviving founders",
        |e| {
            assert!(
                e.world.get::<&Agent>(people[0].1).is_ok(),
                "original observer survives preparation"
            );
            e.resources
                .colony
                .item_ledger
                .created
                .get(&tunic)
                .copied()
                .unwrap_or(0)
                == 10
                && people.iter().all(|&(_, entity)| {
                    e.world.get::<&Agent>(entity).is_err()
                        || e.world
                            .get::<&Apparel>(entity)
                            .is_ok_and(|a| a.top.item() == tunic)
                })
        },
    );
    println!(
        "THERMAL_LINEN_PREP tick={} loom={loom:?} created=10 living={}",
        e.current_tick(),
        e.world.query::<&Agent>().iter().count()
    );
    assert!(
        e.resources.current_tick < winter_tick,
        "ordinary house completes before winter"
    );
    checkpoint(&mut e, "00-built-autumn", home);
    let (person, entity) = people[0];
    until(
        &mut e,
        winter_tick + 1,
        "winter arrives through real calendar ticks",
        |e| {
            assert!(
                e.world.get::<&Agent>(entity).is_ok(),
                "original observer survives autumn preparation"
            );
            sim_systems::runtime::resource_regen::season_at(&e.resources)
                .is_some_and(|(season, _)| season == 3)
        },
    );
    command(
        &mut e,
        Command::SetDrafted {
            agent: person,
            drafted: true,
        },
    );
    let outside = (door.0, door.1 + 1);
    command(
        &mut e,
        Command::DraftMove {
            agent: person,
            x: outside.0,
            y: outside.1,
        },
    );
    until(
        &mut e,
        winter_tick + DAY,
        "same observer physically cold outside in winter",
        |e| {
            assert!(
                e.world.get::<&Agent>(entity).is_ok(),
                "original observer remains alive"
            );
            e.world
                .get::<&Position>(entity)
                .is_ok_and(|p| (p.x, p.y) == outside)
                && room_at(&e.resources, outside).is_none()
                && e.world.get::<&Cold>(entity).is_ok()
                && thermal::place_temperature(&e.resources, outside)
                    == thermal::outdoor_temperature(&e.resources)
        },
    );
    checkpoint(&mut e, "01-outside-winter", home);
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
    command(
        &mut e,
        Command::DraftMove {
            agent: person,
            x: bed.0,
            y: bed.1,
        },
    );
    until(&mut e, 6 * DAY, "paid warmth and dwelling upgrade", |e| {
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
    until(&mut e, 6 * DAY, "actual door passage", |e| {
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
    // Suspended bills still allow automatic deadwood gathering. The player also
    // disables Split, leaving existing loads, hauling and survival needs intact.
    let mut split_priorities: Vec<_> = e
        .world
        .query::<(&Agent, Option<&Child>, Option<&WorkPriorities>)>()
        .iter()
        .filter(|(_, (_, child, _))| child.is_none())
        .map(|(entity, (agent, _, priorities))| {
            (
                agent.id,
                entity,
                priorities.map_or(3, |p| p.level(WorkType::Split)),
            )
        })
        .collect();
    split_priorities.sort_by_key(|(id, _, _)| *id);
    for &(agent, _, level) in &split_priorities {
        if level != 0 {
            command(
                &mut e,
                Command::SetWorkPriority {
                    agent,
                    work: WorkType::Split,
                    level: 0,
                },
            );
        }
    }
    let existing_gatherers: std::collections::BTreeSet<_> = e
        .world
        .query::<(&Agent, &Job)>()
        .iter()
        .filter(|(_, (_, job))| matches!(job.kind, JobKind::Gather { .. }))
        .map(|(_, (agent, _))| agent.id)
        .collect();
    let gathers_drained = std::cell::Cell::new(existing_gatherers.is_empty());
    until(
        &mut e,
        7 * DAY,
        "physical fuel exhaustion and cooling",
        |e| {
            for &(_, entity, _) in &split_priorities {
                if e.world.get::<&Agent>(entity).is_ok() {
                    assert_eq!(
                        e.world
                            .get::<&WorkPriorities>(entity)
                            .expect("disabled Split")
                            .level(WorkType::Split),
                        0
                    );
                }
            }
            let mut gathering = 0;
            for (_, (agent, job)) in e.world.query::<(&Agent, &Job)>().iter() {
                if matches!(job.kind, JobKind::Gather { .. }) {
                    assert!(
                        existing_gatherers.contains(&agent.id),
                        "new gatherer after Split disabled"
                    );
                    gathering += 1;
                }
            }
            if gathers_drained.get() {
                assert_eq!(
                    gathering, 0,
                    "gathering restarted after existing jobs drained"
                );
            }
            if gathering == 0 {
                gathers_drained.set(true);
            }
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
    // Resume fuel immediately in both cold-save arms, then compare 120 real ticks.
    let mut refuel_orders = Vec::new();
    for &(agent, entity, level) in &split_priorities {
        if level != 0 && e.world.get::<&Agent>(entity).is_ok() {
            refuel_orders.push(Command::SetWorkPriority {
                agent,
                work: WorkType::Split,
                level,
            });
        }
    }
    refuel_orders.push(Command::SetBillMode {
        workshop: shop,
        bill,
        mode: BillMode::Make(6),
    });
    refuel_orders.push(Command::SetBillSuspended {
        workshop: shop,
        bill,
        suspended: false,
    });
    continuity_with_commands(&mut e, &refuel_orders);
    assert!(
        e.world.get::<&Agent>(entity).is_ok(),
        "observer died during refueled cold continuity at{}: {:?}",
        e.current_tick(),
        e.resources.chronicle.person(person)
    );
    until(&mut e, 8 * DAY, "physical refuel recovery", |e| {
        assert!(
            e.world.get::<&Agent>(entity).is_ok(),
            "observer died during refuel at{}: {:?}; living={}; ground={:?}; workshops={:?}",
            e.current_tick(),
            e.resources.chronicle.person(person),
            e.world.query::<&Agent>().iter().count(),
            ItemKind::all()
                .map(|kind| (kind, e.resources.colony.ground_total(kind)))
                .filter(|(_, count)| *count > 0)
                .collect::<Vec<_>>(),
            e.resources.colony.workshops
        );
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
        if e.resources.current_tick.is_multiple_of(60) && view.paid_heat {
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
