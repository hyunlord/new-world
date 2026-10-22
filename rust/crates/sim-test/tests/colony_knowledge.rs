//! M4: knowledge (ADR 0024–0026) — the founders' seeding, gating, the quality
//! cap, the channels, forgetting, discovery, the aggregate and the era.

mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, run_until, tick_checked, worker};
use sim_core::colony::knowledge::{self, bit, defs, Knowledge, Source};
use sim_core::colony::materials::FLINT;
use sim_core::colony::{
    BillMode, BlueprintKind, Command, ItemKind, ItemSpec, Job, JobKind, RecipeKind, SkillKind,
    Skills,
};
use sim_core::components::{Agent, Child};
use sim_core::rules::ColonyRule;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::commands::submit;
use std::collections::BTreeMap;

fn id(name: &str) -> u8 {
    knowledge::id(name).unwrap_or_else(|| panic!("no node {name}"))
}

/// [`job_world_m2`] under the M4 colony rule (gating on; the knowledge
/// systems are not registered, so nothing spreads or fades by itself).
fn job_world_m4(w: u32, h: u32) -> SimEngine {
    let mut engine = job_world_m2(w, h);
    engine.resources.rules.colony = Some(ColonyRule::M4);
    engine
}

/// Give `e` the nodes `names` at `p` ‰.
fn know(engine: &mut SimEngine, e: hecs::Entity, names: &[&str], p: u16) {
    let mut k = engine
        .world
        .get::<&Knowledge>(e)
        .map(|k| (*k).clone())
        .unwrap_or_default();
    for n in names {
        k.set(id(n), p, Source::Seeded, 0);
    }
    let _ = engine.world.insert_one(e, k);
}

/// Goal 3.1: "seeded (n)" = n of the 10 founders at 500–700 ‰, "seeded" =
/// every founder at 600 ‰, everything else (all Neolithic but timber walls) no
/// one; holders of a node hold its prerequisites (DECISIONS D9). Seeds 0–7.
#[test]
fn founders_are_seeded_by_the_goal_table_on_seeds_0_to_7() {
    // The goal's table, written out independently of knowledge.ron.
    let all = [
        "fire_keeping",
        "foraging",
        "water_finding",
        "hut_building",
        "storytelling",
    ];
    let some: BTreeMap<&str, usize> = [
        ("flint_knapping", 3),
        ("hafting", 2),
        ("firewood_splitting", 4),
        ("timber_wall", 2),
        ("doorway", 1),
        ("packed_floor", 2),
        ("bedding", 3),
        ("stone_quarrying", 3),
        ("digging", 2),
        ("instruction", 2),
        ("tracking", 2),
        ("hide_scraping", 1),
    ]
    .into_iter()
    .collect();
    for seed in 0..8u64 {
        let engine = sim_test::colony_engine(seed);
        let people: Vec<(bool, Knowledge)> = engine
            .world
            .query::<(&Agent, Option<&Child>, Option<&Knowledge>)>()
            .iter()
            .map(|(_, (_, c, k))| {
                (
                    c.is_none(),
                    k.cloned().expect("every founder has Knowledge"),
                )
            })
            .collect();
        assert_eq!(people.len(), 10, "seed {seed}");
        let holders = |name: &str| -> Vec<&Knowledge> {
            people
                .iter()
                .map(|p| &p.1)
                .filter(|k| k.get(id(name)).is_some())
                .collect()
        };
        for name in all {
            let h = holders(name);
            assert_eq!(h.len(), 10, "seed {seed}: {name}");
            assert!(h.iter().all(|k| k.proficiency(id(name)) == 600));
        }
        for (name, n) in &some {
            let h = holders(name);
            assert_eq!(h.len(), *n, "seed {seed}: {name}");
            for k in &h {
                let p = k.proficiency(id(name));
                assert!((500..=700).contains(&p), "seed {seed}: {name} at {p}");
                assert_eq!(k.get(id(name)).map(|e| e.source()), Some(Source::Seeded));
            }
        }
        for node in &defs().nodes {
            if !all.contains(&node.name.as_str()) && !some.contains_key(node.name.as_str()) {
                assert!(
                    holders(&node.name).is_empty(),
                    "seed {seed}: {} is held",
                    node.name
                );
            }
            for k in holders(&node.name) {
                for r in &node.prereqs {
                    assert!(
                        k.practises(*r),
                        "seed {seed}: a {} holder lacks a prerequisite",
                        node.name
                    );
                }
            }
        }
        // Two technologies with exactly one holder from day 1 (the at-risk warning is real).
        let single = defs()
            .nodes
            .iter()
            .filter(|n| holders(&n.name).len() == 1)
            .count();
        assert!(single >= 2, "seed {seed}: {single}");
    }
}

/// Goal §6: a non-practitioner is never given a gated job. Nobody knows
/// hafting: the haft bill never runs while the split bill does; once one
/// person learns hafting, axes are made, by that person only.
#[test]
fn a_non_practitioner_is_never_given_a_gated_job() {
    let mut engine = job_world_m4(40, 40);
    let a = worker(&mut engine, 3, 3);
    let b = worker(&mut engine, 5, 3);
    let c = worker(&mut engine, 7, 3);
    for e in [a, b, c] {
        know(&mut engine, e, &["firewood_splitting"], 600);
    }
    know(&mut engine, a, &["flint_knapping"], 600);
    let bench = build_workshop(&mut engine, BlueprintKind::WoodBench, (12, 12), (4, 6));
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: 20,
                y0: 20,
                x1: 25,
                y1: 25
            }
        )
        .accepted
    );
    drop_items(
        &mut engine,
        (22, 22),
        ItemSpec::raw(ItemKind::Log, None),
        10,
    );
    drop_items(
        &mut engine,
        (23, 22),
        ItemSpec::raw(ItemKind::FlintBlade, Some(FLINT)),
        5,
    );
    for (recipe, mode) in [
        (RecipeKind::HaftAxe, BillMode::Make(3)),
        (RecipeKind::SplitFirewood, BillMode::Make(2)),
    ] {
        let r = submit(
            &mut engine,
            Command::AddBill {
                workshop: bench,
                recipe,
                mode,
            },
        );
        assert!(r.accepted, "{r:?}");
    }
    let hafter_of = |engine: &SimEngine| -> Vec<hecs::Entity> {
        engine
            .world
            .query::<&Job>()
            .iter()
            .filter(|(_, j)| {
                matches!(j.kind, JobKind::Craft { bill, .. }
                    if engine.resources.colony.workshops[&bench].bill(bill).map(|b| b.recipe) == Some(RecipeKind::HaftAxe))
            })
            .map(|(e, _)| e)
            .collect()
    };
    for _ in 0..3_000 {
        tick_checked(&mut engine);
        assert!(
            hafter_of(&engine).is_empty(),
            "a haft craft without a hafter"
        );
    }
    let shop = &engine.resources.colony.workshops[&bench];
    assert_eq!(
        shop.bills
            .iter()
            .find(|b| b.recipe == RecipeKind::SplitFirewood)
            .map(|b| b.done),
        Some(2)
    );
    assert_eq!(
        shop.bills
            .iter()
            .find(|b| b.recipe == RecipeKind::HaftAxe)
            .map(|b| b.done),
        Some(0)
    );
    // Now b learns hafting: axes are made, every haft craft by b.
    know(&mut engine, b, &["hafting"], 600);
    let mut seen = 0;
    for _ in 0..6_000 {
        tick_checked(&mut engine);
        for e in hafter_of(&engine) {
            assert_eq!(e, b, "only the hafter hafts");
            seen += 1;
        }
        if engine.resources.colony.workshops[&bench]
            .bills
            .iter()
            .any(|b| b.recipe == RecipeKind::HaftAxe && b.done == 3)
        {
            break;
        }
    }
    assert!(seen > 0, "b hafted");
    let done = engine.resources.colony.workshops[&bench]
        .bills
        .iter()
        .find(|b| b.recipe == RecipeKind::HaftAxe)
        .map(|b| b.done);
    assert_eq!(done, Some(3));
    // Each craft stamped hafting used and added +2 ‰ (600 + 3 × 2).
    let k = engine
        .world
        .get::<&Knowledge>(b)
        .map(|k| k.proficiency(id("hafting")))
        .ok();
    assert_eq!(k, Some(606));
}

/// Goal 3.2 and F10 (M4 DECISIONS D22): a gated site is built by its
/// practitioners only. In the colony, after the only door maker dies, an adult
/// standing idle on a delivered door site (the V7 construction arm used to work
/// any co-located site) leaves it at progress 0; once that adult practises the
/// doorway, the door gets built.
#[test]
fn a_door_site_is_never_built_by_a_passer_by_without_the_doorway() {
    let mut engine = sim_test::colony_engine(5);
    tick_checked(&mut engine);
    let holder = engine.resources.colony.tech.states[usize::from(id("doorway"))].names[0];
    assert!(submit(&mut engine, Command::DebugKill { agent: holder }).accepted);
    let camp = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let door = (0..8u32)
        .flat_map(|d| {
            [
                (camp.0 + 2 + d, camp.1 + 3),
                (camp.0.saturating_sub(2 + d), camp.1 + 3),
            ]
        })
        .find(|t| {
            sim_systems::commands::check_space(
                &engine.resources,
                &Default::default(),
                BlueprintKind::Door,
                *t,
            )
            .is_ok()
        })
        .expect("a door spot near the camp");
    let r = submit(
        &mut engine,
        Command::PlaceBlueprint {
            kind: BlueprintKind::Door,
            x: door.0,
            y: door.1,
        },
    );
    assert!(r.accepted, "{r:?}");
    let site = |engine: &SimEngine| {
        sim_systems::runtime::jobs::items::colony_sites(&engine.world)
            .get(&door)
            .map(|v| (v.needs.complete(), v.progress))
    };
    let (x0, y0, x1, y1) =
        sim_test::probe::colony::stockpile_block(&engine, 4, 3).expect("a stockpile spot");
    assert!(submit(&mut engine, Command::AddStockpile { x0, y0, x1, y1 }).accepted);
    drop_items(
        &mut engine,
        (x0, y0),
        ItemSpec::raw(ItemKind::Log, None),
        10,
    );
    // Hauling is not gated: the logs arrive.
    let used = run_until(&mut engine, 4_000, |e| site(e).is_some_and(|s| s.0));
    assert!(used < 4_000, "the door's logs were delivered");
    // The lowest-id living adult stands on it whenever it holds no job.
    let a = engine
        .world
        .query::<&Agent>()
        .without::<&Child>()
        .iter()
        .min_by_key(|(_, ag)| ag.id)
        .map(|(e, _)| e)
        .expect("an adult");
    let mut stood = 0;
    for _ in 0..720 {
        if engine.world.get::<&Job>(a).is_err() {
            if let Ok(mut p) = engine.world.get::<&mut sim_core::components::Position>(a) {
                (p.x, p.y) = door;
                stood += 1;
            }
        }
        tick_checked(&mut engine);
        assert_eq!(
            site(&engine),
            Some((true, 0)),
            "nobody but a door maker works it"
        );
    }
    // Observed 720 of 720 ticks without a job on seed 5; the arm had its chance.
    assert!(stood > 100, "stood {stood}");
    know(&mut engine, a, &["doorway"], 600);
    let used = run_until(&mut engine, 4_000, |e| site(e).is_none());
    assert!(used < 4_000, "the door maker built it");
    let grid = &engine.resources.tile_grid;
    assert!(grid.is_door(grid.idx(door.0, door.1)));
}

/// Goal 3.2: craft quality = min(30 + 3·level + bonus, QUAL_CAP[p / 100]).
/// A level-20 knapper at 350 ‰ never makes a blade above 45; at 1,000 ‰ the
/// same knapper makes blades above it.
#[test]
fn the_quality_cap_follows_proficiency() {
    for (p, cap) in [(350u16, Some(45u8)), (1000, None)] {
        let mut engine = job_world_m4(32, 32);
        let w = worker(&mut engine, 3, 3);
        know(&mut engine, w, &["flint_knapping"], 1000);
        let shop = build_workshop(&mut engine, BlueprintKind::KnappingSpot, (12, 12), (4, 4));
        know(&mut engine, w, &["flint_knapping"], p);
        let mut skills = Skills::default();
        skills.train(SkillKind::Knap, 1_000_000);
        let _ = engine.world.insert_one(w, skills);
        drop_items(
            &mut engine,
            (20, 20),
            ItemSpec::raw(ItemKind::Flint, Some(FLINT)),
            6,
        );
        assert!(
            submit(
                &mut engine,
                Command::AddBill {
                    workshop: shop,
                    recipe: RecipeKind::KnapBlade,
                    mode: BillMode::Make(5)
                }
            )
            .accepted
        );
        let used = run_until(&mut engine, 6_000, |e| {
            e.resources.colony.tallies.crafts_done >= 5
        });
        assert!(used < 6_000, "5 blades at {p} ‰");
        let qualities: Vec<u8> = engine
            .resources
            .colony
            .ground_items
            .values()
            .filter(|s| s.kind == ItemKind::FlintBlade)
            .map(|s| s.quality)
            .collect();
        assert!(!qualities.is_empty());
        match cap {
            Some(c) => assert!(qualities.iter().all(|q| *q <= c), "{p} ‰: {qualities:?}"),
            None => assert!(qualities.iter().any(|q| *q > 45), "{p} ‰: {qualities:?}"),
        }
        let _ = bit(0);
    }
}

/// A small knowledge world: [`job_world_m4`] with a hearth at (10, 10) (the
/// colony's camp hearth, so the evening telling has a place) and the given
/// knowledge systems registered.
fn knowledge_world(seed: u64, systems: &[&str]) -> SimEngine {
    let mut engine = job_world_m4(32, 32);
    engine.resources.world_seed = seed;
    engine
        .resources
        .building_kinds
        .insert((10, 10), sim_core::components::BuildingKind::Hearth);
    for name in systems {
        match *name {
            "oral" => engine.register_system(Box::new(
                sim_systems::runtime::knowledge::OralTransmissionSystem,
            )),
            "decay" => engine.register_system(Box::new(
                sim_systems::runtime::knowledge::KnowledgeDecaySystem,
            )),
            "discover" => {
                engine.register_system(Box::new(sim_systems::runtime::knowledge::DiscoverySystem))
            }
            "aggregate" => engine.register_system(Box::new(
                sim_systems::runtime::knowledge::TechAggregateSystem,
            )),
            other => panic!("no system {other}"),
        }
    }
    engine
}

fn agent_id(engine: &SimEngine, e: hecs::Entity) -> u64 {
    engine
        .world
        .get::<&Agent>(e)
        .map(|a| a.id)
        .unwrap_or(u64::MAX)
}

/// Goal §7 (oral): two people at the evening fire, one knows knapping. On each
/// evening the telling takes when the keyed draw (`STREAM_KNOWLEDGE`, subject =
/// the listener, tick of the telling, slot 0x1_0100 + the node id) is below 500. The first
/// evening it takes is computed here from the draws alone; the listener then
/// holds knapping at 350 ‰ from talk. Seeds 0–7: the outcome matches the draws
/// on every seed, and at least 5 of 8 learned by evening 8.
#[test]
fn talk_at_the_fire_teaches_knapping_at_350_with_the_keyed_chance() {
    let mut learned_by_8 = 0;
    for seed in 0..8u64 {
        let mut engine = knowledge_world(seed, &["oral"]);
        let teller = worker(&mut engine, 11, 10);
        let listener = worker(&mut engine, 9, 10);
        // Both stay at the fire (no idle wander without a movement generator).
        for e in [teller, listener] {
            let _ = engine
                .world
                .remove_one::<sim_systems::runtime::agent::MovementRng>(e);
        }
        know(&mut engine, teller, &["flint_knapping"], 600);
        let key = agent_id(&engine, listener);
        let offset =
            sim_systems::runtime::knowledge::OralTransmissionSystem.tick_offset(&engine.resources);
        // The evening (1-based) on which the draw first falls below 500, if within 8.
        let expected = (0..8u64).find(|e| {
            let tick = offset + e * 1440;
            sim_core::rng::keyed_below(
                seed,
                sim_core::rng::STREAM_KNOWLEDGE,
                key,
                tick,
                0x1_0100 + u32::from(id("flint_knapping")),
                0,
                1000,
            ) < 500
        });
        let mut got = None;
        for evening in 0..8u64 {
            while engine.current_tick() <= offset + evening * 1440 {
                tick_checked(&mut engine);
            }
            let k = engine
                .world
                .get::<&Knowledge>(listener)
                .map(|k| (*k).clone())
                .unwrap_or_default();
            if k.practises(id("flint_knapping")) {
                assert_eq!(k.proficiency(id("flint_knapping")), 350, "seed {seed}");
                assert_eq!(
                    k.get(id("flint_knapping")).map(|e| e.source()),
                    Some(Source::Oral)
                );
                got = Some(evening);
                break;
            }
        }
        assert_eq!(
            got, expected,
            "seed {seed}: learned on evening {got:?}, the draws say {expected:?}"
        );
        if got.is_some() {
            learned_by_8 += 1;
        }
    }
    assert!(learned_by_8 >= 5, "{learned_by_8}/8 learned by evening 8");
}

/// Goal §7 (apprenticeship): `OrderTeach` doorway from a holder with
/// instruction completes within 4 days of sessions; the learner is then a
/// practitioner (`Source::Apprenticed`) and the chronicle logs it.
#[test]
fn an_apprenticeship_makes_a_door_maker_within_four_days() {
    let mut engine = knowledge_world(3, &[]);
    let teacher = worker(&mut engine, 12, 12);
    let learner = worker(&mut engine, 6, 6);
    know(&mut engine, teacher, &["doorway", "instruction"], 650);
    let (t, l) = (agent_id(&engine, teacher), agent_id(&engine, learner));
    let r = submit(
        &mut engine,
        Command::OrderTeach {
            teacher: t,
            learner: l,
            knowledge: id("doorway"),
        },
    );
    assert!(r.accepted, "{r:?}");
    // Refusals: the teacher must practise it; the learner must not.
    let r = submit(
        &mut engine,
        Command::OrderTeach {
            teacher: l,
            learner: t,
            knowledge: id("doorway"),
        },
    );
    assert_eq!(
        r.reason,
        Some(sim_core::colony::RejectReason::NoPractitioner)
    );
    let used = run_until(&mut engine, 4 * 1440, |e| {
        e.resources.colony.tallies.teach_done == 1
    });
    assert!(used < 4 * 1440, "done within 4 days ({used} ticks)");
    let k = engine
        .world
        .get::<&Knowledge>(learner)
        .map(|k| (*k).clone())
        .unwrap_or_default();
    assert!(k.practises(id("doorway")));
    assert_eq!(
        k.get(id("doorway")).map(|e| e.source()),
        Some(Source::Apprenticed)
    );
    // 200 ‰ at round(15 × 950 / 1000) = 14 ‰ a session: 15 sessions.
    assert_eq!(engine.resources.colony.tallies.teach_sessions, 15);
    let entry = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .find(|e| e.kind == sim_core::chronicle::ChronicleKind::KnowledgeLearned)
        .map(|e| (e.subject, e.other, e.cause, e.value));
    assert_eq!(entry, Some((Some(l), Some(t), 4, i64::from(id("doorway")))));
    assert!(engine.resources.colony.teach_orders.is_empty());
    assert!(engine.resources.colony.reservations.is_empty());
}

/// Goal §7: an apprenticeship order is cancelled when the teacher dies, and
/// the learner's job ends with it.
#[test]
fn an_apprenticeship_ends_when_the_teacher_dies() {
    let mut engine = knowledge_world(4, &[]);
    let teacher = worker(&mut engine, 12, 12);
    let learner = worker(&mut engine, 6, 6);
    know(&mut engine, teacher, &["doorway"], 650);
    let (t, l) = (agent_id(&engine, teacher), agent_id(&engine, learner));
    assert!(
        submit(
            &mut engine,
            Command::OrderTeach {
                teacher: t,
                learner: l,
                knowledge: id("doorway")
            }
        )
        .accepted
    );
    run_until(&mut engine, 2_000, |e| {
        e.resources.colony.tallies.teach_sessions >= 2
    });
    let before = engine
        .world
        .get::<&Knowledge>(learner)
        .map(|k| k.proficiency(id("doorway")))
        .unwrap_or(0);
    // Without instruction: round(15 × 700 / 1000) = 11 ‰ a session (DECISIONS D8).
    assert_eq!(before, 22);
    assert!(submit(&mut engine, Command::DebugKill { agent: t }).accepted);
    for _ in 0..3 {
        tick_checked(&mut engine);
    }
    assert!(engine.resources.colony.teach_orders.is_empty());
    assert_eq!(engine.resources.colony.tallies.teach_cancelled, 1);
    assert!(
        engine.world.get::<&Job>(learner).is_err(),
        "the learner's lesson ended"
    );
}

/// Goal §7 (forgetting): a seeded 600 ‰ skill unused for 20 days is below
/// 300 ‰, and its holder is no longer offered the gated work.
#[test]
fn an_unused_skill_fades_below_its_threshold_in_20_days_and_gates_again() {
    let mut engine = knowledge_world(5, &["decay"]);
    let w = worker(&mut engine, 5, 5);
    know(&mut engine, w, &["hafting", "firewood_splitting"], 600);
    for _ in 0..20 * 1440 {
        tick_checked(&mut engine);
    }
    let k = engine
        .world
        .get::<&Knowledge>(w)
        .map(|k| (*k).clone())
        .unwrap_or_default();
    let p = k.proficiency(id("hafting"));
    // 6 days of grace, then 1 ‰ an hour (24 a day) from day 7 to 19: 600 − 13 × 24 = 288.
    assert_eq!(p, 288);
    assert!(!k.practises(id("hafting")));
    // Firewood splitting fell the same way to 288 ‰ but its threshold is 200: only hafting is forgotten.
    assert_eq!(k.proficiency(id("firewood_splitting")), 288);
    assert_eq!(engine.resources.colony.tallies.forgotten_seeded, 1);
    // A wood bench with a haft bill and materials: the worker is never given the craft.
    know(&mut engine, w, &["firewood_splitting"], 600);
    let bench = build_workshop(&mut engine, BlueprintKind::WoodBench, (12, 12), (4, 6));
    drop_items(&mut engine, (20, 20), ItemSpec::raw(ItemKind::Log, None), 4);
    drop_items(
        &mut engine,
        (21, 20),
        ItemSpec::raw(ItemKind::FlintBlade, Some(FLINT)),
        4,
    );
    assert!(
        submit(
            &mut engine,
            Command::AddBill {
                workshop: bench,
                recipe: RecipeKind::HaftAxe,
                mode: BillMode::Make(2)
            }
        )
        .accepted
    );
    for _ in 0..2_000 {
        tick_checked(&mut engine);
        assert!(!engine
            .world
            .get::<&Job>(w)
            .is_ok_and(|j| matches!(j.kind, JobKind::Craft { .. })));
    }
    assert_eq!(engine.resources.colony.tallies.crafts_done, 0);
}

/// Goal §7 (discovery): with 12 stone in a stockpile and a curious adult
/// (novelty seeking 700 ‰) who practises hafting and quarrying, stone grinding
/// is discovered within 60 days on at least 6 of 8 seeds, and the chronicle
/// names the discoverer.
#[test]
fn a_curious_hafter_with_stone_discovers_stone_grinding() {
    let mut found = 0;
    for seed in 0..8u64 {
        let mut engine = knowledge_world(seed, &["discover"]);
        let w = worker(&mut engine, 5, 5);
        let _ = engine.world.insert_one(
            w,
            sim_core::components::Temperament {
                ns: 700,
                ha: 500,
                rd: 500,
                p: 500,
            },
        );
        know(&mut engine, w, &["hafting", "stone_quarrying"], 600);
        assert!(
            submit(
                &mut engine,
                Command::AddStockpile {
                    x0: 20,
                    y0: 20,
                    x1: 23,
                    y1: 23
                }
            )
            .accepted
        );
        drop_items(
            &mut engine,
            (21, 21),
            ItemSpec::raw(ItemKind::Stone, None),
            12,
        );
        let used = run_until(&mut engine, 60 * 1440, |e| {
            e.resources.chronicle.entries().iter().any(|x| {
                x.kind == sim_core::chronicle::ChronicleKind::KnowledgeDiscovered
                    && x.value == i64::from(id("stone_grinding"))
            })
        });
        if used < 60 * 1440 {
            found += 1;
            let k = engine
                .world
                .get::<&Knowledge>(w)
                .map(|k| (*k).clone())
                .unwrap_or_default();
            assert_eq!(k.proficiency(id("stone_grinding")), 500);
            assert_eq!(
                k.get(id("stone_grinding")).map(|e| e.source()),
                Some(Source::Discovered)
            );
            let who = engine
                .resources
                .chronicle
                .entries()
                .iter()
                .find(|x| x.kind == sim_core::chronicle::ChronicleKind::KnowledgeDiscovered)
                .and_then(|x| x.subject);
            assert_eq!(who, Some(agent_id(&engine, w)));
        }
    }
    assert!(found >= 6, "{found}/8 discovered within 60 days");
}

/// The aggregate's state of node `name` in `engine` (status code, practitioners, holders).
fn tech(engine: &SimEngine, name: &str) -> (u8, u16, u16) {
    engine
        .resources
        .colony
        .tech
        .states
        .get(usize::from(id(name)))
        .map_or((255, 0, 0), |s| {
            (s.status.code(), s.practitioners, s.holders)
        })
}

/// Goal §8: the aggregate of a seeded colony at tick 0 — the doorway and hide
/// scraping at risk (one holder each), knapping stable (3 practitioners), the
/// badge 2, and the era progress = timber walls' 2 of 10 adults × weight 2 / 14
/// = 28 ‰ (DECISIONS D12: the table seeds a Neolithic marker).
#[test]
fn the_aggregate_of_a_fresh_colony_matches_its_seeding() {
    for seed in [0u64, 3, 6] {
        let mut engine = sim_test::colony_engine(seed);
        tick_checked(&mut engine);
        assert_eq!(tech(&engine, "doorway"), (1, 1, 1), "seed {seed}");
        assert_eq!(tech(&engine, "hide_scraping"), (1, 1, 1), "seed {seed}");
        assert_eq!(tech(&engine, "flint_knapping"), (3, 3, 3), "seed {seed}");
        assert_eq!(tech(&engine, "pot_firing"), (0, 0, 0), "seed {seed}");
        let t = &engine.resources.colony.tech;
        assert_eq!(t.at_risk(), 2, "seed {seed}");
        assert_eq!(t.adults, 10);
        assert_eq!(t.progress_permille, (2 * 200 / 14) as u16);
    }
}

/// Goal §8 / F10: killing the only doorway holder (test-only `DebugKill`)
/// before anyone was told makes the doorway lost within one aggregate interval
/// (chronicle `KnowledgeLost` naming the holder), and a door blueprint then
/// waits for a practitioner (the bridge's waits name the doorway).
#[test]
fn losing_the_only_door_maker_loses_doors() {
    for seed in [1u64, 2] {
        let mut engine = sim_test::colony_engine(seed);
        tick_checked(&mut engine);
        let holder = engine.resources.colony.tech.states[usize::from(id("doorway"))].names[0];
        assert!(submit(&mut engine, Command::DebugKill { agent: holder }).accepted);
        let killed_at = engine.current_tick();
        let used = run_until(&mut engine, 240, |e| {
            e.resources
                .chronicle
                .entries()
                .iter()
                .any(|x| x.kind == sim_core::chronicle::ChronicleKind::KnowledgeLost)
        });
        assert!(used <= 240, "seed {seed}: lost within one interval");
        let lost: Vec<_> = engine
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|x| {
                x.kind == sim_core::chronicle::ChronicleKind::KnowledgeLost && x.tick >= killed_at
            })
            .map(|x| (x.value, x.subject))
            .collect();
        assert!(
            lost.contains(&(i64::from(id("doorway")), Some(holder))),
            "seed {seed}: {lost:?}"
        );
        assert_eq!(tech(&engine, "doorway").0, 4, "lost");
        let camp = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
        let spot = (0..8u32)
            .flat_map(|d| {
                [
                    (camp.0 + 2 + d, camp.1 + 3),
                    (camp.0.saturating_sub(2 + d), camp.1 + 3),
                ]
            })
            .find(|t| {
                sim_systems::commands::check_space(
                    &engine.resources,
                    &Default::default(),
                    BlueprintKind::Door,
                    *t,
                )
                .is_ok()
            })
            .expect("a door spot near the camp");
        let r = submit(
            &mut engine,
            Command::PlaceBlueprint {
                kind: BlueprintKind::Door,
                x: spot.0,
                y: spot.1,
            },
        );
        assert!(r.accepted, "{r:?}");
        let waits = sim_bridge::ffi::collect_knowledge_waits(&engine.world, &engine.resources);
        assert!(
            waits.iter().any(|w| w.kind == 1
                && (w.a, w.b) == (i64::from(spot.0), i64::from(spot.1))
                && w.node == id("doorway")),
            "seed {seed}: {waits:?}"
        );
    }
}

/// Goal §8: era progress rises when people practise the markers, and its
/// ceiling with every adult practising every present marker is
/// (3 + 3 + 2 + 1) × 1000 / 14 = 642 ‰ (farming and weaving absent).
#[test]
fn era_progress_follows_marker_practitioners_and_tops_out_at_642() {
    let mut engine = sim_test::colony_engine(2);
    tick_checked(&mut engine);
    let start = engine.resources.colony.tech.progress_permille;
    let adults: Vec<hecs::Entity> = engine
        .world
        .query::<&Agent>()
        .without::<&Child>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    let now = engine.current_tick();
    for e in adults.iter().take(4) {
        let mut k = engine
            .world
            .get::<&Knowledge>(*e)
            .map(|k| (*k).clone())
            .unwrap_or_default();
        for n in ["pot_firing", "stone_grinding"] {
            k.set(id(n), 600, Source::Discovered, now);
        }
        let _ = engine.world.insert_one(*e, k);
    }
    run_until(&mut engine, 240, |e| {
        e.resources.colony.tech.states[usize::from(id("pot_firing"))].practitioners == 4
    });
    let mid = engine.resources.colony.tech.progress_permille;
    // Timber walls' 2 of 10 (weight 2) and 4 of 10 adults on two weight-3 markers:
    // ⌊(2 × 200 + 3 × 400 + 3 × 400) / 14⌋ = 200 (the start was ⌊400 / 14⌋ = 28).
    assert_eq!((start, mid), (28, 200));
    let now = engine.current_tick();
    for e in &adults {
        let mut k = engine
            .world
            .get::<&Knowledge>(*e)
            .map(|k| (*k).clone())
            .unwrap_or_default();
        for n in [
            "pot_firing",
            "stone_grinding",
            "timber_wall",
            "stone_walling",
        ] {
            k.set(id(n), 600, Source::Discovered, now);
        }
        let _ = engine.world.insert_one(*e, k);
    }
    run_until(&mut engine, 240, |e| {
        e.resources.colony.tech.progress_permille > mid
    });
    let top = engine.resources.colony.tech.progress_permille;
    assert_eq!(top, 642);
    assert_eq!(
        knowledge::era_label(top),
        1,
        "신석기 전환기: 신석기 시대 is out of reach in M4"
    );
}

/// Goal 3.5 / §8: at least 3 clay banks lie within 24 tiles of the camp on at
/// least 30 of 32 seeds; each holds 18 clay.
#[test]
fn clay_banks_lie_near_the_camp_on_30_of_32_seeds() {
    let mut good = 0;
    let mut counts = Vec::new();
    for seed in 0..32u64 {
        let engine = sim_test::colony_engine(seed);
        let r = &engine.resources;
        let camp = sim_systems::worldgen::colony::hearth(r).expect("hearth");
        let near = r
            .colony
            .nodes
            .iter()
            .filter(|(t, n)| {
                n.kind == sim_core::colony::NodeKind::ClayBank
                    && t.0.abs_diff(camp.0).max(t.1.abs_diff(camp.1)) <= 24
            })
            .count();
        assert!(r
            .colony
            .nodes
            .values()
            .filter(|n| n.kind == sim_core::colony::NodeKind::ClayBank)
            .all(|n| n.left == 18));
        counts.push(near);
        if near >= 3 {
            good += 1;
        }
    }
    assert!(
        good >= 30,
        "{good}/32 seeds with 3 banks within 24: {counts:?}"
    );
}

/// Goal 3.5 / §8: the pottery chain in a scripted colony — clay dug from a
/// marked bank, two pots formed at a pottery bench, fired at the hearth with
/// firewood — with the ledger exact every tick for clay, raw pots and pots.
#[test]
fn the_pottery_chain_runs_from_a_clay_pit_to_a_fired_pot() {
    let mut engine = job_world_m4(40, 40);
    engine
        .resources
        .building_kinds
        .insert((10, 10), sim_core::components::BuildingKind::Hearth);
    let potters: Vec<hecs::Entity> = [(5, 5), (7, 5), (9, 5)]
        .iter()
        .map(|p| worker(&mut engine, p.0, p.1))
        .collect();
    for e in &potters {
        know(
            &mut engine,
            *e,
            &["clay_digging", "pot_forming", "pot_firing", "fire_keeping"],
            600,
        );
    }
    engine.resources.colony.nodes.insert(
        (30, 8),
        sim_core::colony::ResourceNode {
            kind: sim_core::colony::NodeKind::ClayBank,
            left: 18,
            material: sim_core::colony::materials::CLAY,
        },
    );
    // The hearth fire workshop the colony worldgen makes (DECISIONS D15).
    let hearth_id = engine.resources.colony.next_workshop_id.max(1);
    engine.resources.colony.next_workshop_id = hearth_id + 1;
    engine.resources.colony.workshops.insert(
        hearth_id,
        sim_core::colony::Workshop {
            id: hearth_id,
            kind: BlueprintKind::HearthFire,
            top_left: (10, 10),
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: 20,
                y0: 20,
                x1: 25,
                y1: 25
            }
        )
        .accepted
    );
    drop_items(
        &mut engine,
        (22, 22),
        ItemSpec::raw(ItemKind::Firewood, Some(sim_core::colony::materials::OAK)),
        10,
    );
    let bench = build_workshop(&mut engine, BlueprintKind::PotteryBench, (14, 14), (4, 8));
    let r = submit(
        &mut engine,
        Command::DesignateClay {
            x0: 30,
            y0: 8,
            x1: 30,
            y1: 8,
        },
    );
    assert_eq!(r.count, 1, "{r:?}");
    for (shop, recipe) in [
        (bench, RecipeKind::FormPot),
        (hearth_id, RecipeKind::FirePot),
    ] {
        let r = submit(
            &mut engine,
            Command::AddBill {
                workshop: shop,
                recipe,
                mode: BillMode::Make(2),
            },
        );
        assert!(r.accepted, "{r:?}");
    }
    let used = run_until(&mut engine, 12_000, |e| {
        e.resources.colony.item_ledger.crafted_of(ItemKind::ClayPot) >= 2
    });
    assert!(used < 12_000, "two pots fired");
    let l = |k: ItemKind| {
        sim_systems::runtime::jobs::invariants::ledger_of(&engine.world, &engine.resources, k)
    };
    let (clay, raw, pot) = (l(ItemKind::Clay), l(ItemKind::RawPot), l(ItemKind::ClayPot));
    assert!(clay.created >= 4 && clay.created % 3 == 0, "{clay:?}");
    assert_eq!(
        engine
            .resources
            .colony
            .item_ledger
            .craft_inputs
            .get(&ItemKind::Clay),
        Some(&4)
    );
    assert_eq!((raw.created, raw.consumed), (2, 2));
    assert_eq!(pot.created, 2);
    assert!(clay.balanced() && raw.balanced() && pot.balanced());
    assert!(engine.resources.colony.tallies.clay_batches >= 2);
}

/// Goal 3.5: a ground-stone axe lasts 120 chops under M4 and is then used up.
#[test]
fn a_ground_axe_breaks_after_120_chops() {
    let mut engine = job_world_m4(40, 40);
    let w = worker(&mut engine, 5, 5);
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: 6,
                y0: 5,
                x1: 7,
                y1: 6
            }
        )
        .accepted
    );
    drop_items(
        &mut engine,
        (6, 5),
        ItemSpec {
            kind: ItemKind::GroundAxe,
            material: None,
            quality: 50,
        },
        1,
    );
    engine
        .resources
        .colony
        .trees
        .insert((30, 30), sim_core::colony::Tree { logs: 4 });
    assert!(
        submit(
            &mut engine,
            Command::DesignateChopTiles {
                tiles: vec![(30, 30)]
            }
        )
        .accepted
    );
    run_until(&mut engine, 2_000, |e| {
        e.world.get::<&sim_core::colony::Tool>(w).is_ok()
    });
    if let Ok(mut tool) = engine.world.get::<&mut sim_core::colony::Tool>(w) {
        assert_eq!(tool.item.kind, ItemKind::GroundAxe);
        tool.uses = 119;
    }
    run_until(&mut engine, 2_000, |e| {
        e.resources.colony.tallies.trees_felled == 1
    });
    assert_eq!(engine.resources.colony.tallies.tools_broken, 1);
    let broke = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .find(|e| e.kind == sim_core::chronicle::ChronicleKind::ToolBroke)
        .map(|e| (e.cause, e.value));
    assert_eq!(broke, Some((ItemKind::GroundAxe.code(), 120)));
}
