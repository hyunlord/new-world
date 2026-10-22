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
                assert_eq!(k.get(id(name)).map(|e| e.source), Some(Source::Seeded));
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
            "oral" => engine.register_system(Box::new(sim_systems::runtime::knowledge::OralTransmissionSystem)),
            "decay" => engine.register_system(Box::new(sim_systems::runtime::knowledge::KnowledgeDecaySystem)),
            "discover" => engine.register_system(Box::new(sim_systems::runtime::knowledge::DiscoverySystem)),
            "aggregate" => engine.register_system(Box::new(sim_systems::runtime::knowledge::TechAggregateSystem)),
            other => panic!("no system {other}"),
        }
    }
    engine
}

fn agent_id(engine: &SimEngine, e: hecs::Entity) -> u64 {
    engine.world.get::<&Agent>(e).map(|a| a.id).unwrap_or(u64::MAX)
}

/// Goal §7 (oral): two people at the evening fire, one knows knapping. On each
/// evening the telling takes when the keyed draw (`STREAM_KNOWLEDGE`, subject =
/// the listener, tick of the telling, slot 0x1_0002) is below 500. The first
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
            let _ = engine.world.remove_one::<sim_systems::runtime::agent::MovementRng>(e);
        }
        know(&mut engine, teller, &["flint_knapping"], 600);
        let key = agent_id(&engine, listener);
        let offset = sim_systems::runtime::knowledge::OralTransmissionSystem
            .tick_offset(&engine.resources);
        // The evening (1-based) on which the draw first falls below 500, if within 8.
        let expected = (0..8u64).find(|e| {
            let tick = offset + e * 1440;
            sim_core::rng::keyed_below(seed, sim_core::rng::STREAM_KNOWLEDGE, key, tick, 0x1_0002, 0, 1000) < 500
        });
        let mut got = None;
        for evening in 0..8u64 {
            while engine.current_tick() <= offset + evening * 1440 {
                tick_checked(&mut engine);
            }
            let k = engine.world.get::<&Knowledge>(listener).map(|k| (*k).clone()).unwrap_or_default();
            if k.practises(id("flint_knapping")) {
                assert_eq!(k.proficiency(id("flint_knapping")), 350, "seed {seed}");
                assert_eq!(k.get(id("flint_knapping")).map(|e| e.source), Some(Source::Oral));
                got = Some(evening);
                break;
            }
        }
        assert_eq!(got, expected, "seed {seed}: learned on evening {got:?}, the draws say {expected:?}");
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
    let r = submit(&mut engine, Command::OrderTeach { teacher: t, learner: l, knowledge: id("doorway") });
    assert!(r.accepted, "{r:?}");
    // Refusals: the teacher must practise it; the learner must not.
    let r = submit(&mut engine, Command::OrderTeach { teacher: l, learner: t, knowledge: id("doorway") });
    assert_eq!(r.reason, Some(sim_core::colony::RejectReason::NoPractitioner));
    let used = run_until(&mut engine, 4 * 1440, |e| e.resources.colony.tallies.teach_done == 1);
    assert!(used < 4 * 1440, "done within 4 days ({used} ticks)");
    let k = engine.world.get::<&Knowledge>(learner).map(|k| (*k).clone()).unwrap_or_default();
    assert!(k.practises(id("doorway")));
    assert_eq!(k.get(id("doorway")).map(|e| e.source), Some(Source::Apprenticed));
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
    assert!(submit(&mut engine, Command::OrderTeach { teacher: t, learner: l, knowledge: id("doorway") }).accepted);
    run_until(&mut engine, 2_000, |e| e.resources.colony.tallies.teach_sessions >= 2);
    let before = engine.world.get::<&Knowledge>(learner).map(|k| k.proficiency(id("doorway"))).unwrap_or(0);
    // Without instruction: round(15 × 700 / 1000) = 11 ‰ a session (DECISIONS D8).
    assert_eq!(before, 22);
    assert!(submit(&mut engine, Command::DebugKill { agent: t }).accepted);
    for _ in 0..3 {
        tick_checked(&mut engine);
    }
    assert!(engine.resources.colony.teach_orders.is_empty());
    assert_eq!(engine.resources.colony.tallies.teach_cancelled, 1);
    assert!(engine.world.get::<&Job>(learner).is_err(), "the learner's lesson ended");
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
    let k = engine.world.get::<&Knowledge>(w).map(|k| (*k).clone()).unwrap_or_default();
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
    drop_items(&mut engine, (21, 20), ItemSpec::raw(ItemKind::FlintBlade, Some(FLINT)), 4);
    assert!(submit(&mut engine, Command::AddBill { workshop: bench, recipe: RecipeKind::HaftAxe, mode: BillMode::Make(2) }).accepted);
    for _ in 0..2_000 {
        tick_checked(&mut engine);
        assert!(!engine.world.get::<&Job>(w).is_ok_and(|j| matches!(j.kind, JobKind::Craft { .. })));
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
            sim_core::components::Temperament { ns: 700, ha: 500, rd: 500, p: 500 },
        );
        know(&mut engine, w, &["hafting", "stone_quarrying"], 600);
        assert!(submit(&mut engine, Command::AddStockpile { x0: 20, y0: 20, x1: 23, y1: 23 }).accepted);
        drop_items(&mut engine, (21, 21), ItemSpec::raw(ItemKind::Stone, None), 12);
        let used = run_until(&mut engine, 60 * 1440, |e| {
            e.resources
                .chronicle
                .entries()
                .iter()
                .any(|x| x.kind == sim_core::chronicle::ChronicleKind::KnowledgeDiscovered && x.value == i64::from(id("stone_grinding")))
        });
        if used < 60 * 1440 {
            found += 1;
            let k = engine.world.get::<&Knowledge>(w).map(|k| (*k).clone()).unwrap_or_default();
            assert_eq!(k.proficiency(id("stone_grinding")), 500);
            assert_eq!(k.get(id("stone_grinding")).map(|e| e.source), Some(Source::Discovered));
            let who = engine.resources.chronicle.entries().iter().find(|x| x.kind == sim_core::chronicle::ChronicleKind::KnowledgeDiscovered).and_then(|x| x.subject);
            assert_eq!(who, Some(agent_id(&engine, w)));
        }
    }
    assert!(found >= 6, "{found}/8 discovered within 60 days");
}
