//! M1.5 calendar, ages and demographics on the colony world (goal 5.5, ADR 0017).
//!
//! `calendar_derivations`, `ages_in_years`, `fertility_rate_matches_rule`,
//! `no_births_without_partner_or_food` and `pregnancy_to_birth_household`.
//! The rate test drives `FertilitySystem` itself at 12:00 of each day over
//! 1,000 woman-years and compares the births with the rule's expectation,
//! computed independently by a day-by-day probability recursion (below). The
//! baseline's bit-identity is the gate's identity step, not a test here.
use hecs::Entity;
use sim_core::components::{
    Agent, AgentId, BodyHealth, Child, Hunger, Kin, Life, Lineage, Pregnant, Settlement, Sex,
    Thirst,
};
use sim_core::rules::{BirthRule, CalendarRule, WorldRules};
use sim_core::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::agent::{FertilitySystem, MaturationSystem};

const DAY: u64 = 1_440;

#[test]
fn calendar_derivations() {
    let colony = WorldRules::colony(0);
    let baseline = WorldRules::baseline(0);
    assert_eq!(colony.calendar.ticks_per_year(), 34_560);
    assert_eq!(baseline.calendar.ticks_per_year(), 40_320);
    assert_eq!(colony.calendar.days_per_year(), 24);
    assert_eq!(colony.season_days(), 6);
    assert_eq!(baseline.season_days(), 7);
    for seed in 0..4 {
        assert!(WorldRules::colony(seed).calendar_consistent());
        assert!(WorldRules::baseline(seed).calendar_consistent());
    }
    // The colony's ages are years of its calendar.
    let aging = colony.society.aging.expect("colony aging");
    assert_eq!(aging.elder_age_ticks, 50 * 34_560);
    assert_eq!(aging.lifespan_mean_ticks, 62 * 34_560);
    assert_eq!(aging.lifespan_spread_ticks, 8 * 34_560);
    assert_eq!(aging.founder_age_min_ticks, 16 * 34_560);
    assert_eq!(aging.founder_age_max_ticks, 40 * 34_560);
    assert_eq!(aging.founder_min_remaining_ticks, 10 * 34_560);
    assert_eq!(
        aging.lifespan_bounds_ticks,
        Some((40 * 34_560, 85 * 34_560))
    );
    let stage = colony.society.life_stage.expect("colony life stage");
    assert_eq!(stage.child_ticks, 15 * 34_560);
    assert_eq!(stage.infant_ticks, Some(3 * 34_560));
    assert!(matches!(colony.society.birth, BirthRule::Fertility { .. }));
    // Baseline keeps its V7 values.
    assert!(matches!(
        baseline.society.birth,
        BirthRule::Provisioned { .. }
    ));
    assert_eq!(
        baseline.society.life_stage.map(|s| s.child_ticks),
        Some(5_760)
    );
    assert_eq!(baseline.calendar, CalendarRule::BASELINE);
}

#[test]
fn ages_in_years() {
    // A child born at tick t is an adult (no `Child`) once 15 colony years have
    // passed; one maturation pass (every 60 ticks) earlier it is still a child.
    let rules = WorldRules::colony(0);
    let child_ticks = rules.society.life_stage.expect("stage").child_ticks;
    let mut world = hecs::World::new();
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    resources.rules = rules;
    let born = 7_200u64;
    let entity = world.spawn((
        Agent { id: 1 },
        Child { born_tick: born },
        Life {
            born_tick: born as i64,
            lifespan_ticks: 62 * 34_560,
        },
        Hunger::new(0.0, 0.03),
        Thirst::new(0.0, 0.048),
    ));
    let mut system = MaturationSystem;
    resources.current_tick = born + child_ticks - 60;
    system.tick(&mut world, &mut resources);
    assert!(
        world.get::<&Child>(entity).is_ok(),
        "still a child at 15 years − 1 hour"
    );
    let child = *world.get::<&Child>(entity).expect("child");
    assert!(!child.is_infant(
        resources.current_tick,
        rules.society.life_stage.and_then(|s| s.infant_ticks)
    ));
    assert!(child.is_infant(born + 3 * 34_560 - 1, Some(3 * 34_560)));
    resources.current_tick = born + child_ticks;
    system.tick(&mut world, &mut resources);
    assert!(world.get::<&Child>(entity).is_err(), "an adult at 15 years");
    let life = *world.get::<&Life>(entity).expect("life");
    assert_eq!(
        rules
            .calendar
            .whole_years(life.age_ticks(born + child_ticks)),
        15
    );
    assert_eq!(
        rules
            .calendar
            .whole_years(life.age_ticks(born + child_ticks - 1)),
        14
    );
}

/// A synthetic colony: `women` women aged 20 (each paired with a man aged
/// 22 when `partners`), all members of settlement 1, with `hunger` and
/// thirst 0. Returns the world, the resources (colony rules) and the women.
fn synthetic(women: u64, partners: bool, hunger: f32) -> (hecs::World, SimResources, Vec<Entity>) {
    let mut world = hecs::World::new();
    let mut resources = SimResources::new(8, 8, MaterialRegistry::new());
    resources.rules = WorldRules::colony(0);
    resources.world_seed = 11;
    let year = resources.rules.calendar.ticks_per_year() as i64;
    let mut settlement = Settlement::new_with_id(1, 0);
    let mut out = Vec::new();
    for i in 0..women {
        let (w, m): (AgentId, AgentId) = (2 * i, 2 * i + 1);
        let woman = world.spawn((
            Agent { id: w },
            Sex { female: true },
            Kin {
                partner: partners.then_some(m),
                ..Kin::default()
            },
            Life {
                born_tick: -20 * year,
                lifespan_ticks: 80 * year as u64,
            },
            Lineage {
                key: 1_000 + w,
                births: 0,
            },
            Hunger::new(hunger, 0.05),
            Thirst::new(0.0, 0.08),
            BodyHealth::new(),
        ));
        world.spawn((
            Agent { id: m },
            Sex { female: false },
            Kin {
                partner: partners.then_some(w),
                ..Kin::default()
            },
            Life {
                born_tick: -22 * year,
                lifespan_ticks: 80 * year as u64,
            },
            Lineage {
                key: 1_000 + m,
                births: 0,
            },
            Hunger::new(hunger, 0.05),
            Thirst::new(0.0, 0.08),
            BodyHealth::new(),
        ));
        settlement.add_member_agent(w);
        settlement.add_member_agent(m);
        out.push(woman);
    }
    resources.settlements.insert(1, settlement);
    (world, resources, out)
}

/// Run `FertilitySystem` at 12:00 of days `0..days`; every queued birth sets
/// the mother's last birth one tick later, as `SettlementSystem` does when it
/// spawns the child on the next tick. Returns the births.
fn run_days(world: &mut hecs::World, resources: &mut SimResources, days: u64) -> u64 {
    let mut system = FertilitySystem;
    let midday = system.tick_offset(resources);
    let mut births = 0;
    for day in 0..days {
        resources.current_tick = day * DAY + midday;
        system.tick(world, resources);
        for birth in std::mem::take(&mut resources.due_births) {
            births += 1;
            for (_, (agent, kin)) in world.query_mut::<(&Agent, &mut Kin)>() {
                if agent.id == birth.mother {
                    kin.last_birth_tick = Some(resources.current_tick + 1);
                }
            }
        }
    }
    births
}

/// The rule's expected births per woman over `days` days, by a day-by-day
/// recursion on probabilities (independent of the system's code): each
/// midday the women due deliver (eligible again 49 days later: the 48
/// postpartum days from the birth one tick after 12:00), then each eligible
/// woman conceives with probability `p` and is due 18 days later.
fn expected_births(p: f64, gestation: usize, postpartum: usize, days: usize) -> f64 {
    let mut due = vec![0.0; days + gestation + 1];
    let mut returning = vec![0.0; days + gestation + postpartum + 2];
    let mut eligible = 1.0;
    let mut births = 0.0;
    for d in 0..days {
        births += due[d];
        returning[d + postpartum + 1] += due[d];
        eligible += returning[d];
        let conceive = eligible * p;
        eligible -= conceive;
        due[d + gestation] += conceive;
    }
    births
}

#[test]
fn fertility_rate_matches_rule() {
    let BirthRule::Fertility {
        conception_per_day_permille,
        gestation_days,
        postpartum_days,
        ..
    } = WorldRules::colony(0).society.birth
    else {
        panic!("the colony births by fertility");
    };
    // 100 women × 240 days = 1,000 woman-years of the 24-day colony year.
    let (women, days) = (100u64, 240u64);
    let (mut world, mut resources, _) = synthetic(women, true, 0.0);
    let births = run_days(&mut world, &mut resources, days);
    let per_woman = expected_births(
        f64::from(conception_per_day_permille) / 1000.0,
        gestation_days as usize,
        postpartum_days as usize,
        days as usize,
    );
    let expected = per_woman * women as f64;
    // The rule's long-run rate is 1 / (18 + 49 + (1 − p) / p) births a day
    // = 0.249 a year at 33 ‰; over 240 days (with the first pregnancies
    // still running) the recursion expects about 230.
    assert!((200.0..260.0).contains(&expected), "expected {expected}");
    let ratio = births as f64 / expected;
    assert!(
        (0.85..=1.15).contains(&ratio),
        "{births} births in 1,000 woman-years against {expected:.1} expected (×{ratio:.3})"
    );
}

#[test]
fn no_births_without_partner_or_food() {
    // Control: paired and fed women do conceive (so the test can fail).
    let (mut world, mut resources, _) = synthetic(20, true, 0.0);
    assert!(run_days(&mut world, &mut resources, 120) > 0);
    // Without partners: no conception at all.
    let (mut world, mut resources, women) = synthetic(20, false, 0.0);
    assert_eq!(run_days(&mut world, &mut resources, 120), 0);
    assert!(women.iter().all(|w| world.get::<&Pregnant>(*w).is_err()));
    // A hungry settlement (mean hunger 80 > 60): no conception.
    let (mut world, mut resources, women) = synthetic(20, true, 80.0);
    assert_eq!(run_days(&mut world, &mut resources, 120), 0);
    assert!(women.iter().all(|w| world.get::<&Pregnant>(*w).is_err()));
    // Too young (15) and too old (45): no conception.
    for age in [15i64, 45] {
        let (mut world, mut resources, women) = synthetic(20, true, 0.0);
        let year = resources.rules.calendar.ticks_per_year() as i64;
        for w in &women {
            if let Ok(mut life) = world.get::<&mut Life>(*w) {
                life.born_tick = -age * year;
            }
        }
        // 20 days keeps the 15-year-olds below 16 for the whole run.
        assert_eq!(run_days(&mut world, &mut resources, 20), 0, "age {age}");
    }
}

#[test]
fn pregnancy_to_birth_household() {
    let mut engine: SimEngine = sim_test::colony_engine(3);
    for _ in 0..30 {
        engine.tick();
    }
    let mut people: Vec<(AgentId, Entity, bool)> = engine
        .world
        .query::<(&Agent, &Sex)>()
        .without::<&Child>()
        .iter()
        .map(|(e, (a, s))| (a.id, e, s.female))
        .collect();
    people.sort_by_key(|p| p.0);
    let (mother_id, mother) = people
        .iter()
        .find(|p| p.2)
        .map(|p| (p.0, p.1))
        .expect("a founder woman");
    let (father_id, father) = people
        .iter()
        .find(|p| !p.2)
        .map(|p| (p.0, p.1))
        .expect("a founder man");
    for (who, partner) in [(mother, father_id), (father, mother_id)] {
        if let Ok(mut kin) = engine.world.get::<&mut Kin>(who) {
            kin.partner = Some(partner);
        }
    }
    let now = engine.current_tick();
    let _ = engine.world.insert_one(
        mother,
        Pregnant {
            conceived_tick: now,
            due_tick: now,
            father: father_id,
        },
    );
    let births_before = engine.resources.tallies.births;
    // The next 12:00 queues the birth; the tick after spawns the child.
    for _ in 0..(2 * DAY) {
        engine.tick();
        if engine.resources.tallies.births > births_before {
            break;
        }
    }
    assert_eq!(engine.resources.tallies.births, births_before + 1);
    assert!(engine.world.get::<&Pregnant>(mother).is_err());
    let offset = engine.resources.rules.clock_offset_ticks;
    let birth_tick = engine.current_tick() - 1;
    assert_eq!(
        (birth_tick + offset) % DAY,
        721,
        "born the tick after 12:00"
    );
    let child: Vec<(AgentId, Kin)> = engine
        .world
        .query::<(&Agent, &Kin, &Child)>()
        .iter()
        .map(|(_, (a, k, _))| (a.id, *k))
        .collect();
    assert_eq!(child.len(), 1);
    let (child_id, kin) = child[0];
    assert_eq!((kin.mother, kin.father), (Some(mother_id), Some(father_id)));
    assert_eq!(
        engine
            .world
            .get::<&Kin>(mother)
            .expect("mother")
            .last_birth_tick,
        Some(birth_tick)
    );
    // The household partition puts the child with its parents.
    let members: std::collections::BTreeSet<AgentId> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .collect();
    let households = sim_systems::runtime::agent::households::households(&engine.world, &members);
    let home = households
        .iter()
        .find(|h| h.contains(&child_id))
        .expect("the child's household");
    assert!(
        home.contains(&mother_id) && home.contains(&father_id),
        "{home:?}"
    );
}
