//! M5 Part 1: the storyteller and weather pressure (ADR 0027) — the schedule is
//! deterministic and keeps its mercy and gaps, a cold snap doubles the burn and
//! a roofed room with the hearth lit keeps its sleepers warm, a drought slows
//! forage regrowth, illness passes only within a bedroom, and every pressure is
//! chronicled with its beginning, its end and its cost.

mod colony_support;

use colony_support::{adults, bedroll, detect, open_area, walled_box};
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::people::Sick;
use sim_core::colony::story::PressureKind;
use sim_core::components::{AgentState, Cold, Position, ResourceKind, TargetKind};
use sim_core::rules::StoryRule;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::story::{self, simulate_schedule, StorytellerSystem};

const DAYS: u32 = 48;

fn schedule(seed: u64) -> Vec<sim_core::colony::story::Pressure> {
    simulate_schedule(seed, &StoryRule::M5, DAYS, 6)
}

#[test]
fn the_schedule_is_deterministic_and_keeps_its_mercy_and_gaps() {
    let rule = StoryRule::M5;
    for seed in 0..8u64 {
        let plan = schedule(seed);
        assert_eq!(
            plan,
            schedule(seed),
            "seed {seed}: the same seed plans the same"
        );
        assert!(!plan.is_empty(), "seed {seed}: two years bring pressure");
        let majors: Vec<_> = plan.iter().filter(|p| p.kind.is_major()).collect();
        for pair in majors.windows(2) {
            assert!(
                pair[1].start_day >= pair[0].start_day + rule.major_gap_days,
                "seed {seed}: majors {pair:?} closer than 6 days"
            );
        }
        let beasts: Vec<u32> = plan
            .iter()
            .filter(|p| p.kind == PressureKind::Beasts)
            .map(|p| p.start_day)
            .collect();
        for pair in beasts.windows(2) {
            assert!(
                pair[1] >= pair[0] + rule.beasts_gap_days,
                "seed {seed}: beasts {pair:?}"
            );
        }
        for p in &plan {
            let (season, day_of_season) = ((p.start_day / 6) % 4, p.start_day % 6);
            assert!(
                (1..=4).contains(&day_of_season),
                "seed {seed}: {p:?} outside days 2-5"
            );
            match p.kind {
                PressureKind::ColdSnap => {
                    assert_eq!(season, 3, "seed {seed}: a cold snap outside winter {p:?}");
                    assert!(
                        p.start_day >= 20 || p.start_day >= 24,
                        "seed {seed}: first-winter mercy {p:?}"
                    );
                    assert!((2..=4).contains(&(p.end_day - p.start_day)));
                }
                PressureKind::Drought => {
                    assert_eq!(season, 1, "seed {seed}: {p:?}");
                    assert!(
                        p.start_day >= rule.mercy_day
                            && (4..=6).contains(&(p.end_day - p.start_day))
                    );
                }
                PressureKind::Beasts => {
                    assert!(p.start_day >= rule.mercy_day, "seed {seed}: {p:?}");
                    assert!(
                        !(season == 3 && day_of_season >= 4),
                        "seed {seed}: beasts on winter's last days {p:?}"
                    );
                }
                PressureKind::Illness => {
                    assert!(
                        p.start_day >= rule.illness_day && matches!(season, 2 | 3),
                        "seed {seed}: {p:?}"
                    );
                }
            }
        }
    }
}

#[test]
fn every_two_year_world_meets_a_cold_snap_and_beasts() {
    // F1's schedule half, on the calibration and the held-out seeds (registered in advance).
    for seed in (0..8u64).chain(800..832) {
        let plan = schedule(seed);
        let has = |kind| plan.iter().any(|p| p.kind == kind);
        assert!(
            has(PressureKind::ColdSnap) && has(PressureKind::Beasts),
            "seed {seed}: {plan:?}"
        );
    }
}

/// Run only the storyteller, day by day, on a colony world (nothing else ticks).
fn storyteller_days(engine: &mut SimEngine, days: u32) {
    let mut system = StorytellerSystem;
    for day in 0..days {
        engine.resources.current_tick = u64::from(day) * 1_440;
        system.tick(&mut engine.world, &mut engine.resources);
    }
}

#[test]
fn the_live_storyteller_begins_what_the_schedule_plans_and_chronicles_it() {
    let mut engine = sim_test::colony_engine(3);
    storyteller_days(&mut engine, DAYS);
    let story = &engine.resources.colony.story;
    let begun: Vec<(PressureKind, u32)> = story.begun().map(|p| (p.kind, p.start_day)).collect();
    let mut begun_sorted = begun.clone();
    begun_sorted.sort_by_key(|(k, d)| (*d, *k));
    // The colony runs the M5.5 rule (no stacked pressure); nobody falls ill twice here (no bedrooms), so no major waits.
    let planned: Vec<(PressureKind, u32)> =
        simulate_schedule(engine.resources.world_seed, &StoryRule::M5_5, DAYS, 6)
            .iter()
            .map(|p| (p.kind, p.start_day))
            .collect();
    assert_eq!(begun_sorted, planned, "the live run follows the schedule");
    let chronicle = &engine.resources.chronicle;
    let began: Vec<u32> = (0..chronicle.len() as u32)
        .filter(|&i| {
            chronicle
                .entry(i)
                .is_some_and(|e| e.kind == ChronicleKind::PressureBegan)
        })
        .collect();
    assert_eq!(began.len(), begun.len(), "one PressureBegan per pressure");
    let ended: Vec<u32> = (0..chronicle.len() as u32)
        .filter(|&i| {
            chronicle
                .entry(i)
                .is_some_and(|e| e.kind == ChronicleKind::PressureEnded)
        })
        .collect();
    assert!(!ended.is_empty(), "pressures end within two years");
    for i in ended {
        let Some(CauseRef::Entry(start)) = chronicle.cause_of(i) else {
            panic!("PressureEnded {i} has no cause");
        };
        let (b, e) = (
            chronicle.entry(start).expect("begin"),
            chronicle.entry(i).expect("end"),
        );
        assert_eq!(
            (b.kind, b.cause),
            (ChronicleKind::PressureBegan, e.cause),
            "the end points at its own beginning"
        );
        assert!(e.tick >= b.tick);
    }
}

/// The settlement ids of a colony world.
fn settlements(engine: &SimEngine) -> Vec<sim_core::components::SettlementId> {
    let mut ids: Vec<sim_core::components::SettlementId> =
        engine.resources.settlements.keys().copied().collect();
    ids.sort();
    ids
}

/// The first winter night (minute 0) of a colony world.
fn winter_night(engine: &mut SimEngine) -> u64 {
    let rule = engine
        .resources
        .rules
        .society
        .firewood
        .expect("firewood rule");
    (0..64u64)
        .map(|k| 960 + k * 1_440)
        .find(|t| {
            engine.resources.current_tick = *t;
            sim_systems::runtime::settlement::firewood::night_season(&engine.resources, &rule) == 3
        })
        .expect("a winter night within 64 days")
}

#[test]
fn a_cold_snap_doubles_the_night_burn() {
    for snap in [false, true] {
        let mut engine = sim_test::colony_engine(1);
        for _ in 0..10 {
            engine.tick();
        }
        let night = winter_night(&mut engine);
        engine.resources.current_tick = night - 960; // the snap's day (08:00)
        if snap {
            story::force(
                &mut engine.world,
                &mut engine.resources,
                PressureKind::ColdSnap,
                3,
            );
            assert!(engine
                .resources
                .colony
                .story
                .is_active(PressureKind::ColdSnap));
        }
        engine.resources.current_tick = night;
        let sid = settlements(&engine)[0];
        if let Some(s) = engine.resources.settlements.get_mut(&sid) {
            s.withdraw(ResourceKind::Wood, u32::MAX);
            s.store(ResourceKind::Wood, 100);
        }
        let rule = engine
            .resources
            .rules
            .society
            .firewood
            .expect("firewood rule");
        let normal =
            sim_systems::runtime::settlement::firewood::night_rate(&engine.resources, &rule, night);
        assert!(normal > 0, "the hearth burns at midnight");
        sim_systems::runtime::settlement::FireSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        let left = engine.resources.settlements[&sid]
            .stockpile
            .get(&ResourceKind::Wood)
            .copied()
            .unwrap_or(0);
        let expected = if snap { 100 - 2 * normal } else { 100 - normal };
        assert_eq!(
            left,
            expected,
            "snap {snap}: burned {} of {normal} an hour",
            100 - left
        );
    }
}

#[test]
fn in_a_cold_snap_only_a_roof_with_the_hearth_lit_keeps_sleepers_warm() {
    // (snap, fire lit) -> (roofed sleeper cold, outside sleeper cold)
    for (snap, lit, roofed_cold, outside_cold) in [
        (false, false, false, true),
        (true, false, true, true),
        (true, true, false, false),
    ] {
        let mut engine = sim_test::colony_engine(2);
        for _ in 0..10 {
            engine.tick();
        }
        let at = open_area(&engine, 5, 5);
        let inside = walled_box(&mut engine, at, 5, 5, true);
        detect(&mut engine);
        let night = winter_night(&mut engine);
        if snap {
            engine.resources.current_tick = night - 960;
            story::force(
                &mut engine.world,
                &mut engine.resources,
                PressureKind::ColdSnap,
                3,
            );
        }
        let people = adults(&engine);
        let ((_, a), (_, b)) = (people[0], people[1]);
        let asleep = AgentState::Consuming {
            target: TargetKind::Sleep,
        };
        for (e, t) in [(a, inside[4]), (b, (at.0 + 2, at.1 + 6))] {
            let _ = engine.world.insert_one(e, asleep);
            if let Ok(mut p) = engine.world.get::<&mut Position>(e) {
                (p.x, p.y) = t;
            }
        }
        for sid in settlements(&engine) {
            if let Some(s) = engine.resources.settlements.get_mut(&sid) {
                s.withdraw(ResourceKind::Wood, u32::MAX);
                if lit {
                    s.store(ResourceKind::Wood, 100);
                }
            }
        }
        engine.resources.current_tick = night;
        sim_systems::runtime::settlement::FireSystem::new()
            .tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            engine.world.get::<&Cold>(a).is_ok(),
            roofed_cold,
            "snap {snap} lit {lit}: roofed sleeper"
        );
        assert_eq!(
            engine.world.get::<&Cold>(b).is_ok(),
            outside_cold,
            "snap {snap} lit {lit}: outside sleeper"
        );
        if snap && !lit {
            assert!(
                engine
                    .resources
                    .causal_log
                    .iter()
                    .any(|(_, log)| log.as_slice().iter().any(|ev| matches!(
                        ev,
                        sim_core::causal::CausalEvent::PressureCause {
                            kind: 0,
                            effect: 0,
                            ..
                        }
                    ))),
                "a cold person's 'why?' names the snap"
            );
        }
    }
}

#[test]
fn a_drought_slows_forage_regrowth_for_its_days() {
    let regrown = |drought: bool| {
        let mut engine = sim_test::colony_engine(4);
        // Summer of the first year (day 6) at 08:00, every bush emptied.
        engine.resources.current_tick = 6 * 1_440;
        if drought {
            story::force(
                &mut engine.world,
                &mut engine.resources,
                PressureKind::Drought,
                5,
            );
        }
        engine.resources.food_tiles.clear();
        // Caps high enough that no bush fills up in the run.
        for cap in engine.resources.food_source_max.values_mut() {
            *cap = 200;
        }
        let mut regen = sim_systems::runtime::resource_regen::ResourceRegenSystem::new();
        for k in 0..10u64 {
            engine.resources.current_tick = 6 * 1_440 + k * 120;
            regen.tick(&mut engine.world, &mut engine.resources);
        }
        engine
            .resources
            .food_tiles
            .values()
            .map(|v| u64::from(*v))
            .sum::<u64>()
    };
    let (normal, dry) = (regrown(false), regrown(true));
    // Expected from the rules alone: ten pulses at the summer rate p ‰ give
    // floor(10 p / 1000) units a bush; in a drought floor(10 p · 400 / 1000 / 1000).
    let engine = sim_test::colony_engine(4);
    let p = u64::from(
        engine
            .resources
            .rules
            .seasons
            .expect("seasons")
            .regen_permille[1],
    );
    let bushes = engine.resources.food_source_max.len() as u64;
    assert!(bushes > 0 && p > 0);
    assert_eq!(normal, bushes * (10 * p / 1000), "normal regrowth");
    assert_eq!(
        dry,
        bushes * (10 * p * 400 / 1000 / 1000),
        "drought regrowth"
    );
    assert!(dry < normal);
}

#[test]
fn illness_passes_only_to_those_sleeping_in_the_same_bedroom() {
    let mut engine = sim_test::colony_engine(5);
    for _ in 0..10 {
        engine.tick();
    }
    let at = open_area(&engine, 6, 4);
    let inside = walled_box(&mut engine, at, 6, 4, true);
    for t in &inside {
        bedroll(&mut engine, *t);
    }
    engine.resources.colony.rooms_dirty = true;
    detect(&mut engine);
    let room = sim_systems::runtime::rooms::room_at(&engine.resources, inside[0]).expect("a room");
    assert_eq!(
        room.role,
        sim_core::rooms::RoomRole::Bedroom,
        "beds make a bedroom"
    );
    let people = adults(&engine);
    let (roommates, others): (Vec<_>, Vec<_>) =
        people.iter().enumerate().partition(|(i, _)| *i < 4);
    for (i, (id, _)) in roommates.iter().enumerate().map(|(i, (_, p))| (i, *p)) {
        engine.resources.bed_of.insert(*id, inside[i]);
    }
    // The illness strikes the first roommate.
    engine.resources.current_tick = 20 * 1_440;
    let (first, first_e) = *roommates[0].1;
    // A long illness, so it has many nights to pass on (the rule's 3 days are tested by recovery).
    let until = engine.resources.current_tick + 15 * 1_440;
    let _ = engine.world.insert_one(first_e, Sick { until });
    story::force(
        &mut engine.world,
        &mut engine.resources,
        PressureKind::Illness,
        3,
    );
    // `force` makes one more adult sick; keep the test's own carrier only.
    for (_, e) in people.iter().skip(1) {
        let _ = engine.world.remove_one::<Sick>(*e);
    }
    let mut system = StorytellerSystem;
    engine.resources.colony.story.next_check_day = 21;
    for day in 21..40u64 {
        engine.resources.current_tick = day * 1_440;
        system.tick(&mut engine.world, &mut engine.resources);
    }
    let ever_sick: std::collections::BTreeSet<u64> = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .filter(|e| e.kind == ChronicleKind::Sick && e.other.is_some())
        .filter_map(|e| e.subject)
        .collect();
    assert!(
        !ever_sick.is_empty(),
        "in 19 nights at 200 ‰ the illness passed on"
    );
    let mates: std::collections::BTreeSet<u64> = roommates.iter().map(|(_, (id, _))| *id).collect();
    assert!(
        ever_sick.iter().all(|id| mates.contains(id)),
        "only roommates: {ever_sick:?}"
    );
    assert!(
        ever_sick.iter().any(|id| *id != first),
        "it passed to someone else: {ever_sick:?}"
    );
    assert!(others.iter().all(|(_, (id, _))| !ever_sick.contains(id)));
    assert!(
        engine.world.query::<&Sick>().iter().next().is_none(),
        "everyone recovered"
    );
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::Recovered));
}

#[test]
fn an_illness_in_a_big_bedroom_strikes_each_sleeper_once_and_ends() {
    let mut engine = sim_test::colony_engine(5);
    for _ in 0..10 {
        engine.tick();
    }
    let at = open_area(&engine, 6, 4);
    let inside = walled_box(&mut engine, at, 6, 4, true);
    for t in &inside {
        bedroll(&mut engine, *t);
    }
    engine.resources.colony.rooms_dirty = true;
    detect(&mut engine);
    let people = adults(&engine);
    assert!(people.len() >= 8, "a dormitory of {}", people.len());
    for (i, (id, _)) in people.iter().enumerate() {
        engine
            .resources
            .bed_of
            .insert(*id, inside[i % inside.len()]);
    }
    engine.resources.current_tick = 20 * 1_440;
    story::force(
        &mut engine.world,
        &mut engine.resources,
        PressureKind::Illness,
        3,
    );
    let mut system = StorytellerSystem;
    engine.resources.colony.story.next_check_day = 21;
    let mut ended_on = None;
    for day in 21..120u64 {
        engine.resources.current_tick = day * 1_440;
        system.tick(&mut engine.world, &mut engine.resources);
        if ended_on.is_none()
            && !engine
                .resources
                .colony
                .story
                .is_active(PressureKind::Illness)
        {
            ended_on = Some(day);
        }
    }
    let mut onsets: std::collections::BTreeMap<u64, u32> = std::collections::BTreeMap::new();
    let first_end = ended_on.expect("the illness ends");
    for e in engine.resources.chronicle.entries() {
        if e.kind == ChronicleKind::Sick && e.tick < first_end * 1_440 {
            *onsets.entry(e.subject.unwrap_or(u64::MAX)).or_default() += 1;
        }
    }
    assert!(onsets.len() >= 2, "it spread in the dormitory: {onsets:?}");
    assert!(
        onsets.values().all(|n| *n == 1),
        "nobody fell ill twice in one pressure (code review M1): {onsets:?}"
    );
    // Each sleeper at most once, 3 days each: the pressure cannot outlast people x 3 days + the onset day.
    let bound = 21 + 3 * people.len() as u64 + 1;
    assert!(
        first_end <= bound,
        "ended on day {first_end}, bound {bound}"
    );
}

/// M5.5 (ADR 0027 amendment, goal §3.1.1): with `no_stack` no illness begins while a major is active or within two
/// days after one ended, on the calibration seeds over two years; the rule acts (M5 had such illnesses).
#[test]
fn no_illness_begins_during_or_within_two_days_after_a_major() {
    let (mut illnesses, mut m5_overlaps) = (0, 0);
    let overlaps = |plan: &[sim_core::colony::story::Pressure]| {
        plan.iter()
            .filter(|p| p.kind == PressureKind::Illness)
            .filter(|p| {
                plan.iter().any(|m| {
                    m.kind.is_major() && p.start_day >= m.start_day && p.start_day < m.end_day + 2
                })
            })
            .count()
    };
    for seed in 0..8u64 {
        let plan = simulate_schedule(seed, &StoryRule::M5_5, DAYS, 6);
        assert_eq!(overlaps(&plan), 0, "seed {seed}: {plan:?}");
        illnesses += plan
            .iter()
            .filter(|p| p.kind == PressureKind::Illness)
            .count();
        m5_overlaps += overlaps(&schedule(seed));
    }
    // Observed on seeds 0–7 × 48 days: 14 illnesses under M5.5; M5's schedule stacked 15 illnesses on a major.
    assert!(illnesses >= 8, "illness still comes: {illnesses}");
    assert!(
        m5_overlaps >= 8,
        "the M5 schedule stacked pressure, so the rule has work to do: {m5_overlaps}"
    );
}

/// M5.5 (goal §3.1.1): a major planned for a day when two people are sick waits a day at a time; it begins once
/// fewer than two are sick, inside its season.
#[test]
fn a_major_waits_while_two_people_are_sick() {
    let mut engine = sim_test::colony_engine(1);
    // Day 13 (summer, past the mercy day): plan a drought, make two people sick.
    let day = 13u32;
    engine.resources.colony.story.seasons_planned = 3;
    engine.resources.colony.story.next_check_day = day;
    engine.resources.colony.story.planned = vec![sim_core::colony::story::Pressure {
        kind: PressureKind::Drought,
        start_day: day,
        end_day: day + 4,
        began_entry: None,
        deaths_at_start: 0,
        intensity: 0,
        subject: None,
        cost: Default::default(),
    }];
    let sick: Vec<hecs::Entity> = adults(&engine).iter().take(2).map(|(_, e)| *e).collect();
    for e in &sick {
        let _ = engine.world.insert_one(*e, Sick { until: u64::MAX });
    }
    let mut system = StorytellerSystem;
    engine.resources.current_tick = u64::from(day) * 1_440;
    system.tick(&mut engine.world, &mut engine.resources);
    let story = &engine.resources.colony.story;
    assert!(
        !story.is_active(PressureKind::Drought),
        "two sick: the drought waits"
    );
    assert_eq!(
        story.planned.first().map(|p| (p.start_day, p.end_day)),
        Some((day + 1, day + 5))
    );
    // One recovers: the drought begins the next day.
    let _ = engine.world.remove_one::<Sick>(sick[0]);
    engine.resources.current_tick = u64::from(day + 1) * 1_440;
    system.tick(&mut engine.world, &mut engine.resources);
    assert!(
        engine
            .resources
            .colony
            .story
            .is_active(PressureKind::Drought),
        "one sick: it begins"
    );
    // Without the rule (M5) it would have begun on day 13 with two sick.
    let mut m5 = sim_test::colony_engine(1);
    if let Some(c) = m5.resources.rules.colony.as_mut() {
        c.story = Some(StoryRule::M5);
    }
    m5.resources.colony.story = engine.resources.colony.story.clone();
    m5.resources.colony.story.active.clear();
    m5.resources.colony.story.next_check_day = day;
    m5.resources.colony.story.planned = vec![sim_core::colony::story::Pressure {
        kind: PressureKind::Drought,
        start_day: day,
        end_day: day + 4,
        began_entry: None,
        deaths_at_start: 0,
        intensity: 0,
        subject: None,
        cost: Default::default(),
    }];
    for (_, e) in adults(&m5).iter().take(2) {
        let _ = m5.world.insert_one(*e, Sick { until: u64::MAX });
    }
    m5.resources.current_tick = u64::from(day) * 1_440;
    system.tick(&mut m5.world, &mut m5.resources);
    assert!(m5.resources.colony.story.is_active(PressureKind::Drought));
}
