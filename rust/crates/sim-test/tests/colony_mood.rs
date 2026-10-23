//! M5 Part 4: mood, quarrels and the chronicle's causes (ADR 0030) — mood moves
//! to its modifiers' target, a breakdown below 100 refuses jobs for a day and is
//! chronicled with its three largest modifiers, two hungry neighbours on bad
//! terms quarrel over hunger and the "why?" chain resolves through the chronicle,
//! a quarrel turns into a fight only from hostility 0.6, kin never quarrel, and
//! the causes survive a save.

mod colony_support;

use colony_support::adults;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::people::{Bond, Modifier, Mood};
use sim_core::components::{Agent, AgentState, Hunger, Kin, Position, RelationshipKey, TargetKind};
use sim_core::rules::MoodRule;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::mood::{self, why_chain, Link, MoodSystem, QuarrelSystem};

fn place(engine: &mut SimEngine, e: hecs::Entity, t: (u32, u32)) {
    if let Ok(mut p) = engine.world.get::<&mut Position>(e) {
        (p.x, p.y) = t;
    }
}

fn mood_of(engine: &SimEngine, e: hecs::Entity) -> Mood {
    engine.world.get::<&Mood>(e).map(|m| *m).expect("a mood")
}

fn set_mood(engine: &mut SimEngine, e: hecs::Entity, v: i16) {
    if let Ok(mut m) = engine.world.get::<&mut Mood>(e) {
        m.value = v;
    }
}

fn hungry(engine: &mut SimEngine, e: hecs::Entity) {
    if let Ok(mut h) = engine.world.get::<&mut Hunger>(e) {
        h.value = 70.0;
    }
}

/// A colony world whose people all have a mood (one MoodSystem pass), nothing else ticking.
fn mood_world(seed: u64) -> SimEngine {
    let mut engine = sim_test::colony_engine(seed);
    engine.tick();
    engine.resources.current_tick = 60;
    MoodSystem.tick(&mut engine.world, &mut engine.resources);
    engine
}

/// Two adults who are not kin, in `AgentId` order.
fn strangers(engine: &SimEngine) -> ((u64, hecs::Entity), (u64, hecs::Entity)) {
    let all = adults(engine);
    for (i, a) in all.iter().enumerate() {
        for b in all.iter().skip(i + 1) {
            let ka = engine.world.get::<&Kin>(a.1).ok().map(|k| *k);
            let kb = engine.world.get::<&Kin>(b.1).ok().map(|k| *k);
            if !mood::quarrel::kin_close(a.0, ka.as_ref(), b.0, kb.as_ref()) {
                return (*a, *b);
            }
        }
    }
    panic!("no two strangers");
}

#[test]
fn mood_moves_to_its_modifiers_target() {
    let mut engine = mood_world(0);
    let (_, e) = adults(&engine)[0];
    hungry(&mut engine, e);
    let rule = MoodRule::M5;
    for k in 2..60u64 {
        engine.resources.current_tick = 60 * k;
        MoodSystem.tick(&mut engine.world, &mut engine.resources);
    }
    let facts = mood::facts_of(&engine.world, &engine.resources, e);
    let goal = mood::target(&rule, &mood::modifiers(&facts));
    assert!(mood::modifiers(&facts)
        .iter()
        .any(|(m, _)| *m == Modifier::Hungry));
    assert!(goal < 500, "hunger pulls the target down: {goal}");
    assert_eq!(mood_of(&engine, e).value, goal, "58 steps reach the target");
}

#[test]
fn a_breakdown_refuses_jobs_for_a_day_and_names_three_modifiers() {
    let mut engine = mood_world(1);
    let (id, e) = adults(&engine)[0];
    hungry(&mut engine, e);
    if let Ok(mut t) = engine.world.get::<&mut sim_core::components::Thirst>(e) {
        t.value = 70.0;
    }
    if let Ok(mut m) = engine.world.get::<&mut Mood>(e) {
        m.value = 95;
        m.nights = 2 | 4; // slept outside and cold
    }
    engine.resources.current_tick = 120;
    MoodSystem.tick(&mut engine.world, &mut engine.resources);
    let day = mood::today(&engine.resources);
    assert!(mood_of(&engine, e).broken(day), "a breakdown below 100");
    let entry = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .find(|en| en.kind == ChronicleKind::Breakdown && en.subject == Some(id))
        .copied()
        .expect("chronicled");
    assert_eq!(entry.cause, 3, "three modifiers named");
    let codes: Vec<u8> = (0..3)
        .map(|i| ((entry.value >> (8 * i)) & 0xff) as u8)
        .collect();
    // hungry −150 (ha-scaled), cold −120, slept outside −120 / thirsty −100: the three largest.
    assert_eq!(codes[0], Modifier::Hungry.code());
    assert!(
        codes.contains(&Modifier::Cold.code()) && codes.contains(&Modifier::SleptOutside.code())
    );
    // The job layer gives them nothing through the day although work waits.
    let mut engine2 = engine;
    let tiles: Vec<(u32, u32)> = engine2
        .resources
        .colony
        .trees
        .keys()
        .take(30)
        .copied()
        .collect();
    sim_systems::commands::submit(
        &mut engine2,
        sim_core::colony::Command::DesignateChopTiles { tiles },
    );
    let mut others = false;
    for _ in 0..600 {
        engine2.tick();
        assert!(
            engine2.world.get::<&sim_core::colony::Job>(e).is_err(),
            "a broken person takes no job"
        );
        others |= adults(&engine2)
            .iter()
            .any(|(_, o)| engine2.world.get::<&sim_core::colony::Job>(*o).is_ok());
    }
    assert!(others, "others do take jobs");
}

/// Run only the quarrel system until a quarrel is chronicled (or `checks` 30-tick rounds, each
/// five slices six ticks apart, D18); the index of the first quarrel entry made during these
/// checks (earlier ones do not count).
fn quarrel_until(engine: &mut SimEngine, checks: u64) -> Option<u32> {
    let start = engine.resources.current_tick;
    let before = engine.resources.chronicle.len();
    for k in 1..=checks * 5 {
        engine.resources.current_tick = start + 6 * k;
        QuarrelSystem.tick(&mut engine.world, &mut engine.resources);
        let found = engine.resources.chronicle.entries()[before..]
            .iter()
            .position(|e| e.kind == ChronicleKind::Quarrel);
        if let Some(i) = found {
            return Some((before + i) as u32);
        }
    }
    None
}

/// Every other person far away (out of the pair's reach) and both of the pair awake and idle.
fn isolate(engine: &mut SimEngine, a: hecs::Entity, b: hecs::Entity, at: (u32, u32)) {
    let all: Vec<hecs::Entity> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for (i, e) in all.into_iter().enumerate() {
        if e != a && e != b {
            place(engine, e, (5 + 3 * i as u32, 5));
        }
    }
    place(engine, a, at);
    place(engine, b, (at.0 + 1, at.1));
    for e in [a, b] {
        let _ = engine.world.insert_one(e, AgentState::Idle);
    }
}

#[test]
fn hungry_neighbours_on_bad_terms_quarrel_over_hunger_and_the_why_chain_resolves() {
    let mut engine = mood_world(2);
    let ((ia, a), (ib, b)) = strangers(&engine);
    isolate(&mut engine, a, b, (60, 60));
    for e in [a, b] {
        hungry(&mut engine, e);
        set_mood(&mut engine, e, 200);
    }
    engine.resources.colony.bonds.insert(
        (ia.min(ib), ia.max(ib)),
        Bond {
            value: -400,
            day: 0,
            last_quarrel: u32::MAX,
        },
    );
    let before = engine.resources.colony.mood_tallies.quarrels;
    let index = quarrel_until(&mut engine, 400).expect("a quarrel within 400 checks");
    let q = *engine.resources.chronicle.entry(index).expect("entry");
    let mut who = [q.subject, q.other];
    who.sort();
    assert_eq!(
        who,
        [Some(ia.min(ib)), Some(ia.max(ib))],
        "the pair quarrelled"
    );
    assert_eq!(q.cause, 1, "cause 1 + hunger (code 0)");
    // M5.5 (goal §3.1.7): the cause is the initiator's need sample at the quarrel tick, and it resolves.
    let Some(CauseRef::Sample { code, sample }) = engine.resources.chronicle.cause_of(index) else {
        panic!("a hunger quarrel's cause is a need sample");
    };
    assert_eq!(code, Modifier::Hungry.code());
    let s = *engine
        .resources
        .chronicle
        .sample(sample)
        .expect("the sample");
    assert_eq!((s.agent, s.tick), (q.subject.expect("initiator"), q.tick));
    assert!(s.hunger >= 500, "hungry at the quarrel: {}", s.hunger);
    assert_eq!(
        sim_systems::runtime::mood::check_cause(&engine.resources.chronicle, index, 2 * 1_440),
        Ok(())
    );
    let chain = why_chain(&engine.resources.chronicle, index);
    assert_eq!(
        chain[0],
        Link::Entry(index),
        "the chain starts at the quarrel"
    );
    assert_eq!(
        chain[1],
        Link::Modifier(Modifier::Hungry.code()),
        "→ 왜? hunger"
    );
    assert_eq!(engine.resources.colony.mood_tallies.quarrels, before + 1);
    assert!(q.value < -400, "the relation fell: {}", q.value);
    for e in [a, b] {
        assert!(mood_of(&engine, e).value <= 100, "both lost 100");
    }
}

#[test]
fn a_quarrel_comes_to_blows_only_from_hostility_0_6() {
    for (hostility, fights) in [(0.45, 0u64), (0.55, 1)] {
        let mut engine = mood_world(3);
        let ((ia, a), (ib, b)) = strangers(&engine);
        isolate(&mut engine, a, b, (60, 60));
        for e in [a, b] {
            hungry(&mut engine, e);
            set_mood(&mut engine, e, 120);
        }
        engine.resources.colony.bonds.insert(
            (ia.min(ib), ia.max(ib)),
            Bond {
                value: -400,
                day: 0,
                last_quarrel: u32::MAX,
            },
        );
        // The quarrel itself adds 0.1: 0.45 → 0.55 (no fight), 0.55 → 0.65 (a fight).
        engine
            .resources
            .relationships
            .entry(RelationshipKey::new(ia, ib))
            .or_default()
            .hostility = hostility;
        quarrel_until(&mut engine, 400).expect("a quarrel");
        let t = engine.resources.colony.mood_tallies;
        assert_eq!(t.fights, fights, "hostility {hostility} before the quarrel");
        let chronicled = engine
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|e| e.kind == ChronicleKind::Fight)
            .count() as u64;
        assert_eq!(chronicled, fights);
        if fights == 1 {
            let fight = engine
                .resources
                .chronicle
                .entries()
                .iter()
                .rposition(|e| e.kind == ChronicleKind::Fight)
                .expect("fight") as u32;
            assert!(
                matches!(
                    engine.resources.chronicle.cause_of(fight),
                    Some(CauseRef::Entry(_))
                ),
                "a fight's cause is its quarrel"
            );
            // CombatSystem resolves it: one hit of 4.
            let mut combat = sim_systems::runtime::combat::CombatSystem::new();
            combat.tick(&mut engine.world, &mut engine.resources);
            let hp: Vec<f64> = [a, b]
                .iter()
                .map(|e| {
                    engine
                        .world
                        .get::<&sim_core::components::BodyHealth>(*e)
                        .map_or(0.0, |h| h.hp)
                })
                .collect();
            assert!(
                hp.contains(&96.0) && engine.resources.colony.fights.is_empty(),
                "one hit of 4: {hp:?}"
            );
        }
    }
}

#[test]
fn kin_never_quarrel() {
    let mut engine = mood_world(4);
    let ((ia, a), (ib, b)) = strangers(&engine);
    // Make them parent and child.
    let kin = engine.world.get::<&Kin>(b).map(|k| *k).unwrap_or_default();
    let _ = engine.world.insert_one(
        b,
        Kin {
            mother: Some(ia),
            ..kin
        },
    );
    isolate(&mut engine, a, b, (60, 60));
    for e in [a, b] {
        hungry(&mut engine, e);
        set_mood(&mut engine, e, 0);
    }
    engine.resources.colony.bonds.insert(
        (ia.min(ib), ia.max(ib)),
        Bond {
            value: -1000,
            day: 0,
            last_quarrel: u32::MAX,
        },
    );
    assert!(
        quarrel_until(&mut engine, 2_000).is_none(),
        "kin with base 500 never quarrel"
    );
}

#[test]
fn chronicle_causes_survive_a_save() {
    let mut engine = mood_world(2);
    let ((ia, a), (ib, b)) = strangers(&engine);
    isolate(&mut engine, a, b, (60, 60));
    for e in [a, b] {
        hungry(&mut engine, e);
        set_mood(&mut engine, e, 200);
    }
    engine.resources.colony.bonds.insert(
        (ia.min(ib), ia.max(ib)),
        Bond {
            value: -400,
            day: 0,
            last_quarrel: u32::MAX,
        },
    );
    let index = quarrel_until(&mut engine, 400).expect("a quarrel");
    let before = engine.resources.chronicle.cause_of(index);
    let bytes = sim_systems::save::save(&mut engine).expect("save");
    let loaded = sim_systems::save::load(&bytes).expect("load");
    assert_eq!(loaded.resources.chronicle.cause_of(index), before);
    assert_eq!(loaded.resources.colony.bonds, engine.resources.colony.bonds);
    assert_eq!(
        mood_of(&loaded, a),
        mood_of(&engine, a),
        "the mood is saved"
    );
    let _ = TargetKind::Job;
}

/// Quarrel pairs chronicled in 20 quarrel runs of a crowd of 40 on one tile, as gaps in `AgentId` rank.
fn crowd_gaps(cap: u32) -> Vec<usize> {
    use sim_core::rules::WorldRules;
    use sim_systems::worldgen::{build_world_with, WorldGenConfig};
    let mut engine = build_world_with(
        &WorldGenConfig::colony_stress(7, 40, 192),
        WorldRules::colony(7),
    );
    engine.tick();
    engine.resources.current_tick = 60;
    MoodSystem.tick(&mut engine.world, &mut engine.resources);
    if let Some(mood) = engine
        .resources
        .rules
        .colony
        .as_mut()
        .and_then(|c| c.mood.as_mut())
    {
        mood.quarrel_partners_max = cap;
    }
    let mut ids: Vec<(u64, hecs::Entity)> = engine
        .world
        .query::<(&Agent, &Mood)>()
        .iter()
        .map(|(e, (a, _))| (a.id, e))
        .collect();
    ids.sort_unstable();
    assert!(ids.len() >= 30, "a crowd of {}", ids.len());
    for (_, e) in &ids {
        place(&mut engine, *e, (60, 60));
        set_mood(&mut engine, *e, 0);
        let _ = engine.world.insert_one(*e, AgentState::Idle);
    }
    let rank = |id: u64| ids.iter().position(|(i, _)| *i == id).expect("ranked");
    let before = engine.resources.chronicle.len();
    for k in 1..=100u64 {
        engine.resources.current_tick = 60 + 6 * k;
        QuarrelSystem.tick(&mut engine.world, &mut engine.resources);
        for (_, e) in &ids {
            set_mood(&mut engine, *e, 0);
        }
    }
    engine.resources.chronicle.entries()[before..]
        .iter()
        .filter(|e| e.kind == ChronicleKind::Quarrel)
        .map(|e| {
            let (a, b) = (rank(e.subject.expect("a")), rank(e.other.expect("b")));
            a.abs_diff(b)
        })
        .collect()
}

#[test]
fn in_a_crowd_each_person_checks_only_the_nearest_eight_later_ids() {
    // DECISIONS D18: on one tile the nearest are the next ids, so every quarrel spans at most 8 ranks.
    let capped = crowd_gaps(8);
    assert!(
        capped.len() >= 10,
        "quarrels in the crowd: {}",
        capped.len()
    );
    assert!(capped.iter().all(|g| *g <= 8), "gaps {capped:?}");
    // Without the cap some pair is farther apart in rank (the test can fail).
    let open = crowd_gaps(u32::MAX);
    assert!(open.iter().any(|g| *g > 8), "uncapped gaps {open:?}");
}

/// Each person's mood nights by `AgentId`.
fn nights(engine: &SimEngine) -> std::collections::BTreeMap<u64, u8> {
    engine
        .world
        .query::<(&Agent, &Mood)>()
        .iter()
        .map(|(_, (a, m))| (a.id, m.nights))
        .collect()
}

/// Run `engine` from its tick to `end`, counting each person's morning hand-overs by display day: a
/// hand-over moves tonight's facts (bits 3–5) to last night's (bits 0–2) and clears tonight.
fn hand_overs(
    engine: &mut SimEngine,
    end: u64,
    counts: &mut std::collections::BTreeMap<(u64, u64), u32>,
) {
    let offset = engine.resources.rules.clock_offset_ticks;
    let mut before = nights(engine);
    while engine.resources.current_tick < end {
        engine.tick();
        let after = nights(engine);
        let day = (engine.resources.current_tick + offset) / 1_440;
        for (id, new) in &after {
            let Some(old) = before.get(id) else { continue };
            let moved = new & 7 == old >> 3 && new >> 3 == 0;
            if moved && (old >> 3 != 0 || old & 7 != 0) {
                *counts.entry((day, *id)).or_default() += 1;
            }
        }
        before = after;
    }
}

#[test]
fn the_morning_hand_over_runs_once_a_day_across_a_save_at_0830() {
    // MoodSystem runs at minute 7 of each hour; the morning hand-over is the 8 o'clock run (D18).
    let mut continuous = sim_test::colony_engine(3);
    let offset = continuous.resources.rules.clock_offset_ticks;
    let day = continuous.resources.ticks_per_day;
    // 08:30 of display day 2: (tick + offset) % day == 510.
    let save_at = 2 * day + 510 - offset;
    let end = 5 * day + 600 - offset;
    let mut before_save = std::collections::BTreeMap::new();
    hand_overs(&mut continuous, save_at, &mut before_save);
    let bytes = sim_systems::save::save(&mut continuous).expect("save");
    let mut loaded = sim_systems::save::load(&bytes).expect("load");
    let (mut a, mut b) = (before_save.clone(), before_save.clone());
    hand_overs(&mut continuous, end, &mut a);
    hand_overs(&mut loaded, end, &mut b);
    assert_eq!(
        a, b,
        "the loaded copy hands over exactly as the continuous run"
    );
    assert_eq!(nights(&continuous), nights(&loaded));
    // Every hand-over is in the 8 o'clock hour's run: one a person a day, and every person with a mood
    // is handed over on each full day (seed 3: 10 of 10 on days 2, 3 and 4; a second run in the 9
    // o'clock hour makes every count 2 and fails the first assert).
    assert!(a.values().all(|n| *n == 1), "one a day: {a:?}");
    let people = nights(&continuous).len() as u64;
    assert!(people >= 8, "{people} people with a mood");
    for d in 2..=4u64 {
        let on_day = a.keys().filter(|(day, _)| *day == d).count() as u64;
        assert_eq!(on_day, people, "day {d}: {on_day} of {people} handed over");
    }
    // No hand-over after the save on day 2 (08:30 is after the 08:07 run).
    let after_save_day2 = b.keys().filter(|(d, _)| *d == 2).count();
    let before_save_day2 = before_save.keys().filter(|(d, _)| *d == 2).count();
    assert_eq!(
        after_save_day2, before_save_day2,
        "nothing handed over again after the load on day 2"
    );
}

/// Review F4: a cause resolves only when its record fits it — the relation must point at the same pair's earlier
/// quarrel and grief at a death (the old check accepted a quarrel with a third person and a pressure for grief).
#[test]
fn a_quarrels_cause_must_point_at_a_fitting_record() {
    use sim_core::chronicle::{Chronicle, ChronicleEntry};
    use sim_core::colony::people::RELATION_CAUSE;
    let day = 1_440;
    let window = 2 * day;
    let mut c = Chronicle::new();
    let quarrel = |c: &mut Chronicle, tick: u64, who: u64, other: u64, cause: CauseRef| {
        c.push_caused(
            ChronicleEntry::new(tick, ChronicleKind::Quarrel)
                .subject(who)
                .other(Some(other))
                .cause(1),
            Some(cause),
        )
    };
    let none = CauseRef::Modifier {
        code: 0,
        entry: None,
    };
    let with_third = quarrel(&mut c, 9 * day, 1, 3, none);
    let with_two = quarrel(&mut c, 9 * day, 2, 1, none);
    let relation = |entry| CauseRef::Modifier {
        code: RELATION_CAUSE,
        entry: Some(entry),
    };
    let wrong_pair = quarrel(&mut c, 10 * day, 1, 2, relation(with_third));
    let right_pair = quarrel(&mut c, 10 * day, 1, 2, relation(with_two));
    assert_eq!(
        mood::check_cause(&c, wrong_pair, window),
        Err("the record does not fit the cause")
    );
    assert_eq!(mood::check_cause(&c, right_pair, window), Ok(()));
    let began = c.push_caused(
        ChronicleEntry::new(9 * day, ChronicleKind::PressureBegan).cause(0),
        None,
    );
    let grief = quarrel(
        &mut c,
        10 * day,
        1,
        2,
        CauseRef::Modifier {
            code: Modifier::Grief.code(),
            entry: Some(began),
        },
    );
    assert_eq!(
        mood::check_cause(&c, grief, window),
        Err("the record does not fit the cause")
    );
    // Cold (a cold snap, code 0) fits that pressure; sickness (an illness) does not.
    let cold = quarrel(
        &mut c,
        10 * day,
        1,
        2,
        CauseRef::Modifier {
            code: Modifier::Cold.code(),
            entry: Some(began),
        },
    );
    let sick = quarrel(
        &mut c,
        10 * day,
        1,
        2,
        CauseRef::Modifier {
            code: Modifier::Sick.code(),
            entry: Some(began),
        },
    );
    assert_eq!(mood::check_cause(&c, cold, window), Ok(()));
    assert_eq!(
        mood::check_cause(&c, sick, window),
        Err("the record does not fit the cause")
    );
}

/// Review F6: under the M5 mood rule a quarrel's cause is M5's modifier reference, not an M5.5 need sample.
#[test]
fn the_m5_mood_rule_keeps_m5_cause_references() {
    let mut engine = mood_world(2);
    if let Some(c) = engine.resources.rules.colony.as_mut() {
        c.mood = Some(MoodRule::M5);
    }
    let ((ia, a), (ib, b)) = strangers(&engine);
    isolate(&mut engine, a, b, (60, 60));
    for e in [a, b] {
        hungry(&mut engine, e);
        set_mood(&mut engine, e, 200);
    }
    engine.resources.colony.bonds.insert(
        (ia.min(ib), ia.max(ib)),
        Bond {
            value: -400,
            day: 0,
            last_quarrel: u32::MAX,
        },
    );
    let index = quarrel_until(&mut engine, 400).expect("a quarrel within 400 checks");
    let cause = engine.resources.chronicle.cause_of(index);
    assert!(
        matches!(cause, Some(CauseRef::Modifier { code, .. }) if code == Modifier::Hungry.code()),
        "M5's reference: {cause:?}"
    );
    assert!(engine.resources.chronicle.sample(0).is_none(), "no samples");
}
