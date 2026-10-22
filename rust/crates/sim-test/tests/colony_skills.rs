//! M2 Part 3: work priorities, skills, quality and tools (goal 3.6–3.7, §7).

mod colony_support;

use colony_support::{drop_items, job_world_m2, plant, run_until, tick_checked, worker};
use sim_core::colony::{
    CancelReason, Command, ItemKind, ItemSpec, Job, JobKind, JobStage, SkillKind, Skills, Tool,
    WorkType,
};
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::cancel::cancel;
use sim_systems::runtime::jobs::invariants::ledger_of;
use sim_test::probe::colony::{adult_of_rank, nearest_trees, stockpile_block};

fn skills(engine: &SimEngine, e: hecs::Entity) -> Skills {
    engine
        .world
        .get::<&Skills>(e)
        .map(|s| *s)
        .unwrap_or_default()
}

fn chop_ticks(engine: &SimEngine, e: hecs::Entity) -> Option<u32> {
    engine
        .world
        .get::<&Job>(e)
        .ok()
        .and_then(|j| match (j.kind, j.stage) {
            (JobKind::Chop { .. }, JobStage::Working { ticks_left }) => Some(ticks_left),
            _ => None,
        })
}

/// One worker, one tree; returns the work ticks of its chop (read right after
/// the tick that started it, before any is counted down) and the engine.
fn timed_chop(with_axe: Option<u8>) -> (u32, SimEngine, hecs::Entity) {
    let mut engine = job_world_m2(40, 40);
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
    if let Some(q) = with_axe {
        let axe = ItemSpec {
            kind: ItemKind::StoneAxe,
            material: None,
            quality: q,
        };
        drop_items(&mut engine, (6, 5), axe, 1);
    }
    plant(&mut engine, (30, 30), 4);
    assert!(
        submit(
            &mut engine,
            Command::DesignateChopTiles {
                tiles: vec![(30, 30)]
            }
        )
        .accepted
    );
    let used = run_until(&mut engine, 2_000, |e| chop_ticks(e, w).is_some());
    assert!(used < 2_000, "the chop started");
    let ticks = chop_ticks(&engine, w).unwrap_or(0);
    (ticks, engine, w)
}

#[test]
fn an_axe_cuts_chop_time_to_between_55_and_65_percent() {
    let (bare, _, _) = timed_chop(None);
    let (axe, engine, w) = timed_chop(Some(36));
    // Level 0: 90 × 1.4 = 126 ticks; an axe of quality 36 (bucket 3) × 0.555 = 70.
    assert_eq!(bare, 126);
    let ratio = f64::from(axe) / f64::from(bare);
    assert!((0.55..=0.65).contains(&ratio), "{axe} / {bare} = {ratio}");
    let tool = engine.world.get::<&Tool>(w).map(|t| *t).ok();
    assert_eq!(
        tool.map(|t| (t.item.kind, t.item.quality)),
        Some((ItemKind::StoneAxe, 36))
    );
    assert_eq!(
        ledger_of(&engine.world, &engine.resources, ItemKind::StoneAxe).held,
        1
    );
}

#[test]
fn xp_grows_only_on_completed_work() {
    let (ticks, mut engine, w) = timed_chop(None);
    assert_eq!(
        skills(&engine, w).get(SkillKind::Chop).xp,
        0,
        "no XP while working"
    );
    run_until(&mut engine, 500, |e| {
        e.resources.colony.tallies.trees_felled == 1
    });
    assert_eq!(
        skills(&engine, w).get(SkillKind::Chop).xp,
        ticks,
        "XP = the work ticks of the finished chop"
    );
    assert_eq!(
        skills(&engine, w).level(SkillKind::Chop),
        2,
        "126 XP is level 2 (90–179)"
    );
    // A second chop cancelled half-way trains nothing.
    plant(&mut engine, (30, 33), 4);
    submit(
        &mut engine,
        Command::DesignateChopTiles {
            tiles: vec![(30, 33)],
        },
    );
    run_until(&mut engine, 2_000, |e| {
        chop_ticks(e, w).is_some_and(|t| t < 40)
    });
    let before = skills(&engine, w).get(SkillKind::Chop).xp;
    cancel(
        &mut engine.world,
        &mut engine.resources,
        w,
        CancelReason::Command,
    );
    tick_checked(&mut engine);
    assert_eq!(skills(&engine, w).get(SkillKind::Chop).xp, before);
}

#[test]
fn a_worn_out_axe_becomes_a_stone() {
    let (_, mut engine, w) = timed_chop(Some(55));
    if let Ok(mut tool) = engine.world.get::<&mut Tool>(w) {
        tool.uses = 199;
    }
    run_until(&mut engine, 500, |e| {
        e.resources.colony.tallies.trees_felled == 1
    });
    let c = &engine.resources.colony;
    assert_eq!((c.tallies.chops_with_axe, c.tallies.axes_worn_out), (1, 1));
    assert!(engine.world.get::<&Tool>(w).is_err());
    let axe = ledger_of(&engine.world, &engine.resources, ItemKind::StoneAxe);
    let stone = ledger_of(&engine.world, &engine.resources, ItemKind::Stone);
    assert_eq!((axe.created, axe.consumed, axe.held), (1, 1, 0));
    assert_eq!((stone.created, stone.ground), (1, 1));
}

/// Goal §7: a person whose Chop priority is 0 never chops in 10 days with 30 trees marked.
#[test]
fn a_person_with_chop_0_never_chops() {
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..360 {
        engine.tick();
    }
    let never = adult_of_rank(&engine, 0).expect("an adult");
    let (x0, y0, x1, y1) = stockpile_block(&engine, 6, 3).expect("stockpile");
    let cmds = vec![
        Command::SetWorkPriority {
            agent: never,
            work: WorkType::Chop,
            level: 0,
        },
        Command::DesignateChopTiles {
            tiles: nearest_trees(&engine, 30),
        },
        Command::AddStockpile { x0, y0, x1, y1 },
    ];
    for cmd in cmds {
        assert!(submit(&mut engine, cmd).accepted);
    }
    for _ in 0..10 * 1_440 {
        tick_checked(&mut engine);
    }
    let c = &engine.resources.colony;
    let chops = |agent: u64| c.tallies.taken_by.get(&(agent, 1)).copied().unwrap_or(0);
    assert_eq!(chops(never), 0, "the Chop = 0 adult took a chop job");
    let others: u64 = c
        .tallies
        .taken_by
        .iter()
        .filter(|((_, k), _)| *k == 1)
        .map(|(_, n)| n)
        .sum();
    assert!(others >= 30, "{others} chop jobs by the others");
    assert!(
        c.tallies.trees_felled >= 25,
        "{} trees felled",
        c.tallies.trees_felled
    );
    // The person still works: hauling or building.
    let work: u64 = c
        .tallies
        .taken_by
        .iter()
        .filter(|((a, _), _)| *a == never)
        .map(|(_, n)| n)
        .sum();
    assert!(work > 0, "the Chop = 0 adult did other work");
}

/// Two idle adults the same distance from one marked tree, both Chop 3 (the
/// default); `skilled` (if any) gets a Chop level of 4. Returns who took the job.
fn chop_taker(skilled: Option<usize>) -> (Vec<hecs::Entity>, Option<hecs::Entity>) {
    let mut engine = job_world_m2(40, 40);
    let people = vec![worker(&mut engine, 10, 20), worker(&mut engine, 30, 20)];
    if let Some(i) = skilled {
        let mut s = Skills::default();
        // 4 levels: XP_TABLE[4] = 15·16 + 15·4 = 300.
        assert_eq!(s.train(SkillKind::Chop, 300), 4);
        let _ = engine.world.insert_one(people[i], s);
    }
    plant(&mut engine, (20, 5), 4);
    assert!(
        submit(
            &mut engine,
            Command::DesignateChopTiles {
                tiles: vec![(20, 5)]
            }
        )
        .accepted
    );
    tick_checked(&mut engine);
    let taker = people.iter().copied().find(|e| {
        engine
            .world
            .get::<&Job>(*e)
            .is_ok_and(|j| matches!(j.kind, JobKind::Chop { .. }))
    });
    (people, taker)
}

#[test]
fn within_a_priority_level_the_more_skilled_person_takes_the_job() {
    // Equal skill: the lower AgentId (the first spawned) takes it.
    let (people, taker) = chop_taker(None);
    assert_eq!(taker, Some(people[0]));
    // The second person has Chop 4: they take it although their AgentId is higher.
    let (people, taker) = chop_taker(Some(1));
    assert_eq!(taker, Some(people[1]));
    // And the first again when the skill is theirs.
    let (people, taker) = chop_taker(Some(0));
    assert_eq!(taker, Some(people[0]));
}
