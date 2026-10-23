//! M5.5 Parts 2–3: neighbours, migrants, caravans and eureka (ADR 0031, 0032) — two neighbours a world on
//! different edges, drawn deterministically; waves 1 and 2 in their windows; the population cap holds a wave
//! back; an accepted wave joins with its knowledge (source `Migrant`), apparel, beds and membership; a declined
//! or unanswered wave leaves and its neighbour cools; caravans never come during a major pressure or before day 6;
//! barter follows the 80 % / 120 % rule and keeps the ledger exact through `traded_in` / `traded_out`; an artifact
//! hints at the node that makes it (eureka).

mod colony_support;

use colony_support::{adults, bedroll, open_area, tick_checked};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::knowledge::{self, defs, Era, Knowledge, Source};
use sim_core::colony::neighbours::{VisitState, WaveAnswer};
use sim_core::colony::people::Apparel;
use sim_core::colony::{Command, ItemKind, ItemSpec, RejectReason};
use sim_core::components::{Agent, Lineage};
use sim_core::rules::NeighbourRule;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::invariants::{self, ledger_of};
use sim_systems::runtime::neighbours::{self, trade, NeighbourSystem};
use sim_systems::runtime::story::StorytellerSystem;

const DAY: u64 = 1_440;

fn people(engine: &SimEngine) -> usize {
    engine.world.query::<&Agent>().iter().count()
}

#[test]
fn two_neighbours_on_different_edges_are_drawn_from_the_seed() {
    // The pottery lane's nodes (knowledge.ron), for the "one of them knows the pottery chain" check.
    let pottery = defs()
        .nodes
        .iter()
        .filter(|n| n.lane == sim_core::colony::knowledge::Lane::Pottery)
        .fold(0u32, |m, n| m | knowledge::bit(n.id));
    let rule = NeighbourRule::m5_5().expect("neighbours.ron");
    let neo: Vec<u8> = neighbours::neolithic();
    let mut pottery_worlds = 0;
    for seed in 0..8u64 {
        let list = neighbours::generate(seed, &rule);
        assert_eq!(
            list,
            neighbours::generate(seed, &rule),
            "seed {seed}: deterministic"
        );
        assert_eq!(list.len(), 2);
        assert_ne!(list[0].edge, list[1].edge, "seed {seed}: different edges");
        assert_ne!(list[0].name_key, list[1].name_key);
        for n in &list {
            assert!((2..=5).contains(&n.distance_days), "{n:?}");
            assert!((20..=60).contains(&n.population), "{n:?}");
            assert!((40..=60).contains(&n.attitude), "{n:?}");
            assert_eq!(
                n.known & neighbours::paleolithic_mask(),
                neighbours::paleolithic_mask(),
                "every Paleolithic node"
            );
            let k = neo
                .iter()
                .filter(|id| n.known & knowledge::bit(**id) != 0)
                .count();
            // 2–4 drawn (pot firing replaces the last one when neither drew it, D23).
            assert!((2..=5).contains(&k), "seed {seed}: {k} Neolithic nodes");
        }
        assert!(
            list.iter().any(|n| n.known & pottery != 0),
            "seed {seed}: one of them knows the pottery chain"
        );
        pottery_worlds += 1;
        let waves = neighbours::first_waves(seed, &rule);
        assert!(
            (8..=12).contains(&waves[0].0) && (2..=4).contains(&waves[0].1),
            "{waves:?}"
        );
        assert!(
            (20..=26).contains(&waves[1].0) && (2..=6).contains(&waves[1].1),
            "{waves:?}"
        );
    }
    assert_eq!(pottery_worlds, 8);
}

#[test]
fn wave_one_arrives_in_its_window_and_an_accepted_wave_joins_with_its_knowledge() {
    let mut engine = sim_test::colony_engine(3);
    let rule = neighbours::rule(&engine.resources).expect("the M5.5 colony has neighbours");
    // Beds for everyone who may come (the founders and two waves).
    let at = open_area(&engine, 6, 4);
    for y in 0..4 {
        for x in 0..6 {
            bedroll(&mut engine, (at.0 + x, at.1 + y));
        }
    }
    let planned = neighbours::first_waves(engine.resources.world_seed, &rule);
    while engine.resources.colony.neighbours.waiting_wave().is_none()
        && engine.current_tick() < 13 * DAY
    {
        tick_checked(&mut engine);
    }
    let wave = engine
        .resources
        .colony
        .neighbours
        .waiting_wave()
        .cloned()
        .expect("wave 1 reached the hearth");
    let day = engine.current_tick() / DAY;
    assert_eq!(
        day,
        u64::from(planned[0].0),
        "it arrives the day it sets out (a few hours' walk)"
    );
    assert!((8..=12).contains(&day));
    assert_eq!(wave.people.len(), usize::from(planned[0].1));
    let before = people(&engine);
    let r = submit(&mut engine, Command::AcceptMigrants { wave: wave.id });
    assert!(r.accepted);
    assert_eq!(r.count as usize, wave.people.len());
    assert_eq!(
        people(&engine),
        before + wave.people.len(),
        "every migrant joined"
    );
    let w = engine.resources.colony.neighbours.waves[0].clone();
    assert_eq!(
        (w.state, w.answer),
        (VisitState::Gone, Some(WaveAnswer::Accepted))
    );
    // Each joined person carries their spec: lineage, knowledge from the neighbour (source Migrant), apparel.
    let n = engine
        .resources
        .colony
        .neighbours
        .neighbour(w.neighbour)
        .cloned()
        .expect("neighbour");
    let d = defs();
    let mut practitioners = 0;
    for (id, spec) in w.joined.iter().zip(&w.people) {
        let (e, _) = engine
            .world
            .query::<&Agent>()
            .iter()
            .find(|(_, a)| a.id == *id)
            .map(|(e, a)| (e, a.id))
            .expect("joined person");
        assert_eq!(
            engine.world.get::<&Lineage>(e).map(|l| l.key).ok(),
            Some(spec.key)
        );
        let k = engine
            .world
            .get::<&Knowledge>(e)
            .expect("knowledge")
            .clone();
        for entry in k.entries() {
            assert_eq!(entry.source(), Source::Migrant);
            assert!(
                n.known & knowledge::bit(entry.id()) != 0,
                "only what the neighbour knows"
            );
            let def = d.get(entry.id()).expect("node");
            match def.era {
                Era::Paleolithic => assert!((450..=700).contains(&entry.proficiency)),
                Era::Neolithic => {
                    assert!((420..=650).contains(&entry.proficiency));
                    practitioners += usize::from(entry.proficiency >= def.threshold);
                }
            }
        }
        let paleo = d
            .nodes
            .iter()
            .filter(|node| node.era == Era::Paleolithic && n.known & knowledge::bit(node.id) != 0)
            .count();
        assert!(
            k.entries().len() >= paleo,
            "every Paleolithic node the neighbour knows"
        );
        assert_eq!(
            engine.world.get::<&Apparel>(e).is_ok(),
            spec.wrap,
            "a wrap when the spec says"
        );
        let members: Vec<_> = engine
            .resources
            .settlements
            .values()
            .filter(|s| s.member_agents.contains(id))
            .collect();
        assert_eq!(members.len(), 1, "a member of the colony's settlement");
    }
    // Every Neolithic node a migrant brings lies at 420–650 ‰, at or above its threshold (≤ 400): practitioners.
    let neo_entries: usize = w
        .people
        .iter()
        .flat_map(|p| p.knowledge.iter())
        .filter(|(id, _)| d.get(*id).is_some_and(|n| n.era == Era::Neolithic))
        .count();
    assert_eq!(practitioners, neo_entries);
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::MigrantsJoined));
    // At 20:00 the beds are handed out: the migrants get beds too.
    while engine.current_tick() % DAY != 12 * 60 + 1 {
        tick_checked(&mut engine);
    }
    for id in &w.joined {
        assert!(
            engine.resources.bed_of.contains_key(id),
            "migrant {id} has a bed"
        );
    }
    // The ledger holds with the wraps they brought (created and worn).
    assert!(invariants::check(&engine.world, &engine.resources).is_ok());
    let wraps = ledger_of(&engine.world, &engine.resources, ItemKind::HideWrap);
    assert_eq!(wraps.created, wraps.held + wraps.ground);
}

#[test]
fn a_declined_wave_walks_away_and_its_neighbour_cools() {
    let mut engine = sim_test::colony_engine(5);
    while engine.resources.colony.neighbours.waiting_wave().is_none()
        && engine.current_tick() < 13 * DAY
    {
        tick_checked(&mut engine);
    }
    let w = engine
        .resources
        .colony
        .neighbours
        .waiting_wave()
        .cloned()
        .expect("wave 1 waits");
    let attitude = |e: &SimEngine| {
        e.resources
            .colony
            .neighbours
            .neighbour(w.neighbour)
            .map(|n| n.attitude)
    };
    let before = attitude(&engine).expect("attitude");
    let people_before = people(&engine);
    assert!(submit(&mut engine, Command::DeclineMigrants { wave: w.id }).accepted);
    assert_eq!(attitude(&engine), Some(before - 5), "attitude −5");
    assert_eq!(
        submit(&mut engine, Command::AcceptMigrants { wave: w.id }).reason,
        Some(RejectReason::NoVisitor),
        "a declined wave cannot be taken back"
    );
    for _ in 0..DAY {
        tick_checked(&mut engine);
    }
    let after = &engine.resources.colony.neighbours.waves[0];
    assert_eq!(
        (after.state, after.answer),
        (VisitState::Gone, Some(WaveAnswer::Declined)),
        "walked off the map"
    );
    assert_eq!(people(&engine), people_before, "nobody joined");
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::MigrantsDeclined && e.value >> 24 == 0));
}

#[test]
fn an_unanswered_wave_leaves_after_a_day() {
    let mut engine = sim_test::colony_engine(1);
    while engine.resources.colony.neighbours.waiting_wave().is_none()
        && engine.current_tick() < 13 * DAY
    {
        tick_checked(&mut engine);
    }
    let w = engine
        .resources
        .colony
        .neighbours
        .waiting_wave()
        .cloned()
        .expect("wave 1 waits");
    let VisitState::Waiting { since } = w.state else {
        panic!("waiting")
    };
    let before = engine
        .resources
        .colony
        .neighbours
        .neighbour(w.neighbour)
        .map(|n| n.attitude);
    while engine.current_tick() < since + DAY + 3 {
        tick_checked(&mut engine);
    }
    let after = &engine.resources.colony.neighbours.waves[0];
    assert_eq!(after.answer, Some(WaveAnswer::Unanswered));
    assert_ne!(after.state, VisitState::Waiting { since });
    assert_eq!(
        engine
            .resources
            .colony
            .neighbours
            .neighbour(w.neighbour)
            .map(|n| n.attitude),
        before.map(|a| a - 5)
    );
}

#[test]
fn the_population_cap_holds_a_wave_back() {
    let mut engine = sim_test::colony_engine(2);
    // 40 people: the founders and extra adults.
    while people(&engine) < 40 {
        let e = engine.spawn_agent(5, 5);
        let _ = engine
            .world
            .insert_one(e, sim_core::components::AgentState::Idle);
    }
    let rule = NeighbourRule::m5_5().expect("neighbours.ron");
    let first = neighbours::first_waves(engine.resources.world_seed, &rule)[0].0;
    let mut system = NeighbourSystem;
    for day in 0..=first {
        engine.resources.current_tick = u64::from(day) * DAY;
        system.tick(&mut engine.world, &mut engine.resources);
    }
    let nb = &engine.resources.colony.neighbours;
    assert!(nb.waves.is_empty(), "no wave at 40 people");
    assert_eq!(nb.tallies.capped, 1);
}

/// The storyteller and the neighbours alone, day by day (no one walks or works): the caravan dates.
fn caravan_days(seed: u64, days: u32) -> (Vec<u32>, Vec<(u32, u32)>) {
    let mut engine = sim_test::colony_engine(seed);
    let (mut story, mut nb) = (StorytellerSystem, NeighbourSystem);
    for day in 0..days {
        engine.resources.current_tick = u64::from(day) * DAY;
        story.tick(&mut engine.world, &mut engine.resources);
        nb.tick(&mut engine.world, &mut engine.resources);
    }
    let set_out = engine
        .resources
        .colony
        .neighbours
        .caravans
        .iter()
        .map(|c| c.day)
        .collect();
    let majors = engine
        .resources
        .colony
        .story
        .begun()
        .filter(|p| p.kind.is_major())
        .map(|p| (p.start_day, p.end_day))
        .collect();
    (set_out, majors)
}

#[test]
fn caravans_never_set_out_during_a_major_pressure_or_before_day_six() {
    let mut caravans = 0;
    for seed in 0..8u64 {
        let (set_out, majors) = caravan_days(seed, 48);
        for day in &set_out {
            assert!(*day >= 6, "seed {seed}: a caravan on day {day}");
            assert!(
                !majors.iter().any(|(s, e)| (*s..*e).contains(day)),
                "seed {seed}: a caravan on day {day} during {majors:?}"
            );
        }
        caravans += set_out.len();
    }
    // Observed 6–8 a world over 8 seasons (one date a season, the first spring's before day 6 skipped).
    assert!(caravans >= 8 * 5, "{caravans} caravans on 8 worlds");
}

/// A world with a caravan trading at the hearth (ticking everything until one arrives).
fn trading_world(seed: u64) -> SimEngine {
    let mut engine = sim_test::colony_engine(seed);
    while engine
        .resources
        .colony
        .neighbours
        .trading_caravan()
        .is_none()
        && engine.current_tick() < 20 * DAY
    {
        tick_checked(&mut engine);
    }
    assert!(
        engine
            .resources
            .colony
            .neighbours
            .trading_caravan()
            .is_some(),
        "a caravan came"
    );
    engine
}

#[test]
fn barter_follows_the_balance_rule_and_the_ledger_holds_through_the_hand_over() {
    let mut engine = trading_world(0);
    let c = engine
        .resources
        .colony
        .neighbours
        .trading_caravan()
        .cloned()
        .expect("caravan");
    // A stockpile with 12 flint blades at quality 50 (value 5 each).
    let at = open_area(&engine, 3, 3);
    assert!(
        submit(
            &mut engine,
            Command::AddStockpile {
                x0: at.0,
                y0: at.1,
                x1: at.0 + 2,
                y1: at.1 + 2
            }
        )
        .accepted
    );
    engine
        .resources
        .colony
        .item_ledger
        .create(ItemKind::FlintBlade, 12);
    sim_systems::runtime::jobs::items::put_items(
        &mut engine.resources,
        at,
        ItemSpec::raw(ItemKind::FlintBlade, None),
        12,
    );
    let blade = sim_core::colony::values::value(ItemKind::FlintBlade, 50);
    assert_eq!(blade, 5);
    // The cheapest goods stack: its unit value at 120 % against blades at 80 %.
    let (i, g) = c
        .goods
        .iter()
        .enumerate()
        .min_by_key(|(_, g)| sim_core::colony::values::value(g.kind, g.quality))
        .expect("goods");
    let unit = u64::from(sim_core::colony::values::value(g.kind, g.quality));
    // Blades needed: n × 5 × 800 ≥ unit × 1,200.
    let need = (unit * 1_200).div_ceil(5 * 800) as u16;
    if need > 1 {
        let short = submit(
            &mut engine,
            Command::Trade {
                caravan: c.id,
                give: vec![(at.0, at.1, need - 1)],
                take: vec![(i as u16, 1)],
            },
        );
        assert_eq!(
            short.reason,
            Some(RejectReason::Unbalanced),
            "one blade short is refused"
        );
    }
    let attitude = engine
        .resources
        .colony
        .neighbours
        .neighbour(c.neighbour)
        .map(|n| n.attitude)
        .expect("n");
    let ok = submit(
        &mut engine,
        Command::Trade {
            caravan: c.id,
            give: vec![(at.0, at.1, need)],
            take: vec![(i as u16, 1)],
        },
    );
    assert!(ok.accepted, "{ok:?}");
    let nb = &engine.resources.colony.neighbours;
    assert_eq!(
        nb.neighbour(c.neighbour).map(|n| n.attitude),
        Some((attitude + 3).min(100)),
        "attitude +3"
    );
    assert_eq!(
        nb.trading_caravan()
            .map(|c| c.owed.iter().map(|o| o.2).sum::<u16>()),
        Some(need)
    );
    assert_eq!(engine.resources.colony.item_ledger.traded_in_of(g.kind), 1);
    // Haulers carry the blades to the caravan's spot; the ledger holds every tick.
    let mut handed = false;
    for _ in 0..DAY / 2 {
        tick_checked(&mut engine);
        if engine
            .resources
            .colony
            .item_ledger
            .traded_out_of(ItemKind::FlintBlade)
            == u64::from(need)
        {
            handed = true;
            break;
        }
    }
    assert!(handed, "the blades reached the traders");
    let blades = ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade);
    assert_eq!(blades.traded_out, u64::from(need));
    assert!(blades.balanced());
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::Traded));
}

#[test]
fn an_artifact_no_one_can_make_is_a_clue_and_triples_the_draw() {
    let mut engine = sim_test::colony_engine(4);
    tick_checked(&mut engine);
    let pot_forming = knowledge::id("pot_forming").expect("node");
    // Nobody practises pot forming at the start; no pot, no clue.
    assert!(
        trade::clues(&engine.world, &engine.resources).is_empty()
            || !trade::clues(&engine.world, &engine.resources).contains_key(&pot_forming)
    );
    engine
        .resources
        .colony
        .item_ledger
        .trade_in(ItemKind::ClayPot, 1);
    let at = open_area(&engine, 1, 1);
    sim_systems::runtime::jobs::items::put_items(
        &mut engine.resources,
        at,
        ItemSpec::raw(ItemKind::ClayPot, None),
        1,
    );
    let clues = trade::clues(&engine.world, &engine.resources);
    assert_eq!(
        clues.get(&pot_forming),
        Some(&ItemKind::ClayPot),
        "a clay pot hints at pot forming"
    );
    // Once someone practises it, the pot is no clue for it any more.
    let (_, e) = adults(&engine)[0];
    if let Ok(mut k) = engine.world.get::<&mut Knowledge>(e) {
        k.set(pot_forming, 600, Source::Seeded, 0);
    }
    assert!(!trade::clues(&engine.world, &engine.resources).contains_key(&pot_forming));
    // The draw: a clue wins on draws 0, 1, 2 of `odds`, otherwise only on 0 — three times the chance.
    let rule = engine
        .resources
        .rules
        .colony
        .and_then(|c| c.neighbours)
        .expect("rule");
    let wins = |clue: bool, odds: u64| -> u64 {
        let w = if clue {
            u64::from(rule.eureka_permille)
        } else {
            1000
        };
        (0..odds).filter(|d| d * 1000 < w).count() as u64
    };
    assert_eq!((wins(false, 40), wins(true, 40)), (1, 3));
}

#[test]
fn a_save_with_a_wave_on_its_way_loads_to_the_same_world() {
    let mut a = sim_test::colony_engine(6);
    let mut b = sim_test::colony_engine(6);
    let on_way = |e: &SimEngine| {
        e.resources
            .colony
            .neighbours
            .waves
            .iter()
            .any(|w| w.state == VisitState::Arriving)
    };
    while !on_way(&a) && a.current_tick() < 13 * DAY {
        a.tick();
        b.tick();
    }
    assert!(on_way(&a), "a wave walks in");
    let bytes = sim_systems::save::save(&mut b).expect("save");
    let mut loaded = sim_systems::save::load(&bytes).expect("load");
    for _ in 0..DAY {
        a.tick();
        loaded.tick();
    }
    assert_eq!(
        a.resources.colony.neighbours,
        loaded.resources.colony.neighbours
    );
    assert_eq!(
        sim_test::probe::colony::colony_fingerprint(&a),
        sim_test::probe::colony::colony_fingerprint(&loaded)
    );
    let x = sim_systems::save::save(&mut a).expect("save a");
    let y = sim_systems::save::save(&mut loaded).expect("save loaded");
    assert!(x == y, "full state equal after a day");
}

/// Twelve blades on a fresh stockpile cell of a trading world: `(engine, the caravan's number, the stack's tile)`.
fn blades_for_trade(seed: u64, blades: u16) -> (SimEngine, u32, (u32, u32)) {
    let mut engine = trading_world(seed);
    let caravan = engine
        .resources
        .colony
        .neighbours
        .trading_caravan()
        .map(|c| c.id)
        .expect("a caravan trades");
    let at = open_area(&engine, 3, 3);
    let zoned = submit(
        &mut engine,
        Command::AddStockpile {
            x0: at.0,
            y0: at.1,
            x1: at.0 + 2,
            y1: at.1 + 2,
        },
    );
    assert!(zoned.accepted);
    engine
        .resources
        .colony
        .item_ledger
        .create(ItemKind::FlintBlade, u64::from(blades));
    sim_systems::runtime::jobs::items::put_items(
        &mut engine.resources,
        at,
        ItemSpec::raw(ItemKind::FlintBlade, None),
        blades,
    );
    (engine, caravan, at)
}

fn trade_job(engine: &SimEngine) -> Option<sim_core::colony::Job> {
    engine
        .world
        .query::<&sim_core::colony::Job>()
        .iter()
        .map(|(_, j)| *j)
        .find(|j| matches!(j.kind, sim_core::colony::JobKind::Trade { .. }))
}

fn owed_total(engine: &SimEngine) -> u32 {
    engine
        .resources
        .colony
        .neighbours
        .trading_caravan()
        .map_or(0, |c| c.owed.iter().map(|o| u32::from(o.2)).sum())
}

/// Review F1: items given to a caravan stay owed until a hauler picks them up, so they cannot be given again while
/// they still lie on the stack (the old code dropped them from `owed` when the haul was assigned, and a second gift
/// of 10 of the same 12 blades was accepted).
#[test]
fn given_items_stay_owed_until_picked_up_and_cannot_be_given_twice() {
    let (mut engine, caravan, tile) = blades_for_trade(0, 12);
    let gift = submit(
        &mut engine,
        Command::Trade {
            caravan,
            give: vec![(tile.0, tile.1, 12)],
            take: vec![],
        },
    );
    assert!(gift.accepted, "{gift:?}");
    assert_eq!(owed_total(&engine), 12);
    // A hauler takes the Trade haul; until the pickup the blades are on the stack and all still owed.
    let mut assigned = false;
    for _ in 0..DAY / 2 {
        tick_checked(&mut engine);
        if trade_job(&engine).is_some() {
            assigned = true;
            break;
        }
    }
    assert!(assigned, "a trade haul was assigned");
    let on_stack = engine
        .resources
        .colony
        .ground_items
        .get(&tile)
        .map_or(0, |s| u32::from(s.count));
    assert_eq!(on_stack, 12, "not picked up yet");
    assert_eq!(owed_total(&engine), 12, "owed until picked up");
    let again = submit(
        &mut engine,
        Command::Trade {
            caravan,
            give: vec![(tile.0, tile.1, 1)],
            take: vec![],
        },
    );
    assert!(!again.accepted, "an owed blade cannot be given again");
    assert_eq!(again.reason, Some(RejectReason::Unknown));
    // After the pickup the carried blades leave `owed`; the rest stays owed.
    let mut carried = 0;
    for _ in 0..DAY / 2 {
        tick_checked(&mut engine);
        let c = engine
            .world
            .query::<(&sim_core::colony::Job, &sim_core::colony::Carry)>()
            .iter()
            .find(|(_, (j, _))| matches!(j.kind, sim_core::colony::JobKind::Trade { .. }))
            .map(|(_, (_, c))| u32::from(c.count));
        if let Some(n) = c {
            carried = n;
            break;
        }
    }
    assert!(carried > 0, "a hauler carries blades");
    assert_eq!(owed_total(&engine), 12 - carried);
    let l = ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade);
    assert!(l.traded_out <= 12 && engine.resources.colony.neighbours.tallies.gifts == 1);
}

/// Review F8: an owed entry whose stack now holds another kind does not hide a later entry on the same tile (the old
/// assignment took the first entry by tile, failed its kind check and offered nothing).
#[test]
fn an_owed_entry_of_another_kind_does_not_block_the_tile() {
    let (mut engine, _, tile) = blades_for_trade(0, 6);
    if let Some(c) = engine
        .resources
        .colony
        .neighbours
        .caravans
        .iter_mut()
        .find(|c| matches!(c.state, VisitState::Waiting { .. }))
    {
        c.owed = vec![(tile, ItemKind::Hide, 2), (tile, ItemKind::FlintBlade, 4)];
    }
    let mut job = None;
    for _ in 0..DAY / 2 {
        tick_checked(&mut engine);
        job = trade_job(&engine);
        if job.is_some() {
            break;
        }
    }
    let job = job.expect("the blades are hauled");
    assert_eq!(job.item, ItemKind::FlintBlade);
    assert_eq!(job.amount, 4, "the owed four, not the stack's six");
    // The pickup takes only the blade entry down.
    for _ in 0..DAY / 2 {
        tick_checked(&mut engine);
        if owed_total(&engine) < 6 {
            break;
        }
    }
    let owed = engine
        .resources
        .colony
        .neighbours
        .trading_caravan()
        .map(|c| c.owed.clone())
        .expect("still trading");
    assert_eq!(owed, vec![(tile, ItemKind::Hide, 2)]);
}

/// One `DiscoverySystem` run on colony seed 4 where every person practises clay digging and nobody clay preparation:
/// `(discoverers of clay preparation, eurekas)`. `neighbours`: the rule on (with `eureka_permille`) or off; `pot`: a
/// clay pot lies on the ground (a clue for clay preparation); `clay`: 4 clay stockpiled (its own condition met).
fn clay_prep_discoveries(
    neighbours: Option<u32>,
    pot: bool,
    clay: bool,
    odds: u32,
) -> (usize, u64) {
    let mut engine = sim_test::colony_engine(4);
    if let Some(c) = engine.resources.rules.colony.as_mut() {
        c.neighbours = neighbours.map(|eureka| NeighbourRule {
            eureka_permille: eureka,
            ..NeighbourRule::m5_5().expect("neighbours.ron")
        });
        if let Some(k) = c.knowledge.as_mut() {
            k.discovery_odds = [odds; 3];
        }
    }
    tick_checked(&mut engine);
    let clay_prep = knowledge::id("clay_prep").expect("node");
    let clay_digging = knowledge::id("clay_digging").expect("node");
    let all: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, _)| e)
        .collect();
    for e in all {
        if let Ok(mut k) = engine.world.get::<&mut Knowledge>(e) {
            k.set(clay_digging, 800, Source::Seeded, 0);
            assert!(!k.practises(clay_prep), "nobody practises clay preparation");
        }
    }
    if pot {
        engine
            .resources
            .colony
            .item_ledger
            .create(ItemKind::ClayPot, 1);
        let at = open_area(&engine, 1, 1);
        sim_systems::runtime::jobs::items::put_items(
            &mut engine.resources,
            at,
            ItemSpec::raw(ItemKind::ClayPot, None),
            1,
        );
    }
    if clay {
        let at = open_area(&engine, 2, 2);
        let zoned = submit(
            &mut engine,
            Command::AddStockpile {
                x0: at.0,
                y0: at.1,
                x1: at.0 + 1,
                y1: at.1 + 1,
            },
        );
        assert!(zoned.accepted);
        engine
            .resources
            .colony
            .item_ledger
            .create(ItemKind::Clay, 4);
        sim_systems::runtime::jobs::items::put_items(
            &mut engine.resources,
            at,
            ItemSpec::raw(ItemKind::Clay, None),
            4,
        );
    }
    let stocked =
        sim_systems::runtime::knowledge::discover::stockpiled(&engine.resources, ItemKind::Clay);
    assert_eq!(
        stocked >= 4,
        clay,
        "clay stockpiled only when asked: {stocked}"
    );
    let before = engine.resources.colony.neighbours.tallies.eurekas;
    sim_systems::runtime::knowledge::DiscoverySystem.tick(&mut engine.world, &mut engine.resources);
    let found = engine
        .world
        .query::<&Knowledge>()
        .iter()
        .filter(|(_, k)| k.practises(clay_prep))
        .count();
    (
        found,
        engine.resources.colony.neighbours.tallies.eurekas - before,
    )
}

/// Review F2: eureka acts only under the neighbour rule — with the rule off a held pot neither opens clay
/// preparation's stockpile condition nor widens its draw (the old code opened the condition for every rule).
#[test]
fn eureka_needs_the_neighbour_rule() {
    let (off, off_eurekas) = clay_prep_discoveries(None, true, false, 1);
    assert_eq!((off, off_eurekas), (0, 0), "rule off: the pot is no clue");
    let (on, on_eurekas) = clay_prep_discoveries(Some(3_000), true, false, 1);
    assert!(
        on >= 2,
        "rule on: the pot opens the condition ({on} discoverers)"
    );
    assert_eq!(
        on_eurekas as usize, on,
        "every one of them decided by the clue"
    );
}

/// The ×3 through `DiscoverySystem` itself (DECISIONS D20 re-done after review F2): at odds 1 in 3 a clue wins every
/// draw (draws 0–2 × 1,000 < 3,000), so as many candidates discover as at odds 1 in 1; with `eureka_permille` 1,000
/// (no ×3) only those whose draw is 0 do.
#[test]
fn a_clue_triples_the_discovery_draw() {
    let (sure, _) = clay_prep_discoveries(Some(3_000), true, false, 1);
    let (tripled, _) = clay_prep_discoveries(Some(3_000), true, false, 3);
    let (plain, _) = clay_prep_discoveries(Some(1_000), true, false, 3);
    assert!(sure >= 2, "{sure} candidates");
    assert_eq!(tripled, sure, "×3 at odds 3 wins every draw");
    assert!(plain < sure, "without ×3 fewer win: {plain} of {sure}");
}

/// Review F5: a clue node's discovery counts as a eureka only when the clue decided it — with the node's condition
/// met by its own clay and the draw a sure win (odds 1), nobody needed the pot.
#[test]
fn a_eureka_is_counted_only_when_the_clue_decided_it() {
    let (found, eurekas) = clay_prep_discoveries(Some(3_000), true, true, 1);
    assert!(found >= 2, "{found} discoverers");
    assert_eq!(eurekas, 0, "the clay alone opened it and the draw was sure");
}

/// The first wave of colony seed 3 while it is still walking in.
fn wave_on_its_way(seed: u64) -> (SimEngine, u32) {
    let mut engine = sim_test::colony_engine(seed);
    while !engine
        .resources
        .colony
        .neighbours
        .waves
        .iter()
        .any(|w| w.state == VisitState::Arriving)
        && engine.current_tick() < 13 * DAY
    {
        tick_checked(&mut engine);
    }
    let id = engine
        .resources
        .colony
        .neighbours
        .waves
        .iter()
        .find(|w| w.state == VisitState::Arriving)
        .map(|w| w.id)
        .expect("a wave walks in");
    (engine, id)
}

/// Review F3: a wave accepted while still walking in keeps walking and joins on reaching the hearth (the old code
/// spawned its people where the group stood — 72 tiles from the hearth on seed 3).
#[test]
fn a_wave_accepted_on_its_way_joins_at_the_hearth() {
    let (mut engine, id) = wave_on_its_way(3);
    let size = engine.resources.colony.neighbours.waves[(id - 1) as usize]
        .people
        .len();
    let before = people(&engine);
    let r = submit(&mut engine, Command::AcceptMigrants { wave: id });
    assert!(r.accepted && r.count as usize == size);
    let w = engine.resources.colony.neighbours.waves[(id - 1) as usize].clone();
    assert_eq!(w.state, VisitState::Arriving, "still walking");
    assert_eq!(w.answer, Some(WaveAnswer::Accepted));
    assert_eq!(people(&engine), before, "nobody joined on the way");
    for _ in 0..2 * DAY {
        tick_checked(&mut engine);
        if engine.resources.colony.neighbours.waves[(id - 1) as usize].state == VisitState::Gone {
            break;
        }
    }
    let w = engine.resources.colony.neighbours.waves[(id - 1) as usize].clone();
    assert_eq!(w.state, VisitState::Gone);
    assert_eq!(w.joined.len(), size, "all of them joined");
    assert_eq!(engine.resources.colony.neighbours.tallies.accepted, 1);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let far = engine
        .world
        .query::<(&Agent, &sim_core::components::Position)>()
        .iter()
        .filter(|(_, (a, _))| w.joined.contains(&a.id))
        .map(|(_, (_, p))| p.x.abs_diff(hearth.0).max(p.y.abs_diff(hearth.1)))
        .max()
        .expect("the joined people");
    assert!(far <= 14, "they joined by the hearth: {far} tiles");
}

/// Review F7: with no meeting tile (here: no hearth) a wave still stops, waits and times out instead of freezing.
#[test]
fn visitors_without_a_meeting_tile_still_wait_and_leave() {
    let (mut engine, id) = wave_on_its_way(3);
    engine
        .resources
        .building_kinds
        .retain(|_, k| *k != sim_core::components::BuildingKind::Hearth);
    assert!(sim_systems::worldgen::colony::hearth(&engine.resources).is_none());
    let start = engine.resources.current_tick;
    for k in 1..=(2 * DAY / 3) {
        engine.resources.current_tick = start + 3 * k;
        neighbours::VisitorSystem.tick(&mut engine.world, &mut engine.resources);
    }
    let w = &engine.resources.colony.neighbours.waves[(id - 1) as usize];
    assert_eq!(
        w.answer,
        Some(WaveAnswer::Unanswered),
        "it waited and timed out"
    );
    assert!(matches!(w.state, VisitState::Leaving | VisitState::Gone));
}
