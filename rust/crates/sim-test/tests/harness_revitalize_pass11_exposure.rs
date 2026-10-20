//! Pass 11 Part 1: a cold that can take anyone (exposure v2: hardiness,
//! hunger and per-stage damage).
//!
//! V8-0: rebased from `game_v17_nofission` (damage [9, 5, 6] an hour) and the
//! goal's start values ([8, 6, 8]) on `WorldRules::baseline` (damage [7.5, 5,
//! 5] an hour, hunger weight 0.5, hardiness on); the numbers below are the
//! baseline's.
use hecs::Entity;
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::hardiness::{HARDINESS_MAX_PERMILLE, HARDINESS_MIN_PERMILLE};
use sim_core::components::{
    Agent, AgentId, BodyHealth, Child, Elder, Hardiness, Hunger, Lineage, Position, Settlement,
    Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rng::NoiseFork;
use sim_core::rules::{ExposureRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::settlement::fire::exposure_damage;
use sim_systems::runtime::settlement::FireSystem;
use sim_systems::runtime::survival::StarvationSystem;
use sim_test::baseline_engine;

const HOME: (u32, u32) = (20, 20);

/// Tick at which the display clock (480-tick offset) reads `minute` on the
/// zero-based display day `day` (day index 21 = the first winter day).
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// Life stage of a fixture resident.
#[derive(Clone, Copy, PartialEq, Debug)]
enum Stage {
    Child,
    Adult,
    Elder,
}

/// One fixture resident: id, stage, hardiness (None = no component), hunger, hp.
type Resident = (AgentId, Stage, Option<u16>, f32, f64);

/// A bare world under `rules` at `tick`: one settlement at `HOME` with an
/// empty wood store (the fire goes out) and the given residents, needs fixed
/// (growth rate 0), thirst 0.
fn fixture(rules: WorldRules, tick: u64, residents: &[Resident]) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.current_tick = tick;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    let mut entities = Vec::new();
    for &(id, stage, hardiness, hunger, hp) in residents {
        let entity = engine.world.spawn((
            Position::new(HOME.0, HOME.1),
            Agent { id },
            Hunger::new(hunger, 0.0),
            Thirst::new(0.0, 0.0),
            BodyHealth { hp, max_hp: 100.0 },
        ));
        match stage {
            Stage::Child => engine
                .world
                .insert_one(entity, Child { born_tick: 0 })
                .expect("child"),
            Stage::Elder => engine.world.insert_one(entity, Elder).expect("elder"),
            Stage::Adult => {}
        }
        if let Some(permille) = hardiness {
            engine
                .world
                .insert_one(entity, Hardiness { permille })
                .expect("hardiness");
        }
        settlement.add_member_agent(id);
        entities.push(entity);
    }
    engine.resources.settlements.insert(1, settlement);
    (engine, entities)
}

fn hp(engine: &SimEngine, entity: Entity) -> f64 {
    engine.world.get::<&BodyHealth>(entity).expect("health").hp
}

/// World seed of the fixture rules; the exposure rule does not depend on it.
const SEED: u64 = 0;

fn rules() -> WorldRules {
    WorldRules::baseline(SEED)
}

fn baseline_exposure() -> ExposureRule {
    rules().society.exposure.expect("baseline exposure rule")
}

#[test]
fn harness_hardiness_keyed_and_ranged() {
    let mut total_births = 0;
    for seed in [3u64, 41, 900_017] {
        // Unforked and forked (replicate 3 from tick 1) runs of the same world.
        let mut plain = baseline_engine(seed);
        let mut forked = baseline_engine(seed);
        forked.resources.noise_fork = Some(NoiseFork {
            tick: 1,
            replicate: 3,
        });
        let founders = |engine: &SimEngine| {
            let mut rows: Vec<(u64, u16)> = engine
                .world
                .query::<(&Lineage, &Hardiness)>()
                .iter()
                .map(|(_, (lineage, h))| (lineage.key, h.permille))
                .collect();
            rows.sort_unstable();
            rows
        };
        let at_start = founders(&plain);
        assert_eq!(
            at_start.len(),
            plain.world.query::<&Agent>().iter().count(),
            "every founder carries hardiness"
        );
        assert_eq!(at_start, founders(&forked));
        for _ in 0..6 * 1440 {
            plain.tick();
            forked.tick();
        }
        // Newborns must be checked too: measured births 1 / 11 / 5 (both runs)
        // on seeds 3 / 41 / 900017 over these six days.
        let births = plain.resources.tallies.births;
        assert!(
            births >= 1 && forked.resources.tallies.births >= 1,
            "seed {seed}: births happened in both runs"
        );
        total_births += births;
        for engine in [&plain, &forked] {
            let mut count = 0;
            for (_, (agent, lineage, hardiness)) in engine
                .world
                .query::<(&Agent, &Lineage, &Hardiness)>()
                .iter()
            {
                // A pure function of (seed, lineage key): independent of the
                // replicate, of the AgentId and of the order of births.
                assert_eq!(
                    *hardiness,
                    Hardiness::draw(seed, lineage.key),
                    "seed {seed} agent {}",
                    agent.id
                );
                assert!(
                    (HARDINESS_MIN_PERMILLE..=HARDINESS_MAX_PERMILLE).contains(&hardiness.permille)
                );
                count += 1;
            }
            assert_eq!(
                count,
                engine.world.query::<&Agent>().iter().count(),
                "every resident carries hardiness"
            );
        }
    }
    assert!(total_births >= 10, "{total_births} births (measured 17)");
    // The draw ignores the replicate: same key, same value; spread over the range.
    let values: std::collections::BTreeSet<u16> = (0..3_000u64)
        .map(|k| Hardiness::draw(5, k).permille)
        .collect();
    assert!(
        values.contains(&HARDINESS_MIN_PERMILLE) || values.iter().next().is_some_and(|v| *v < 710)
    );
    assert!(values.iter().next_back().is_some_and(|v| *v > 1290));
}

#[test]
fn harness_exposure_v2_formula() {
    let rule = baseline_exposure();
    assert_eq!(rule.damage_per_cold_hour, [7.5, 5.0, 5.0]);
    assert_eq!((rule.hunger_weight, rule.use_hardiness), (0.5, true));
    // 22:00 of the first winter day, fire out. Three hardiness x hunger cases
    // (adults), plus a child and an elder.
    let night = tick_at(21, 1320);
    let cases: [Resident; 5] = [
        (1, Stage::Adult, Some(700), 0.0, 100.0),
        (2, Stage::Adult, Some(1000), 75.0, 100.0),
        (3, Stage::Adult, Some(1300), 99.0, 100.0),
        (4, Stage::Child, Some(1100), 60.0, 100.0),
        (5, Stage::Elder, Some(900), 40.0, 100.0),
    ];
    let (mut engine, entities) = fixture(rules(), night, &cases);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    for (&(id, stage, hardiness, hunger, start), entity) in cases.iter().zip(&entities) {
        let index = match stage {
            Stage::Child => 0,
            Stage::Adult => 1,
            Stage::Elder => 2,
        };
        let factor = f64::from(hardiness.expect("hardiness")) / 1000.0;
        let excess = ((f64::from(hunger) - 50.0) / 50.0).clamp(0.0, 1.0);
        let expected =
            rule.damage_per_cold_hour[index] * factor * (1.0 + rule.hunger_weight * excess);
        let lost = start - hp(&engine, *entity);
        assert!(
            (lost - expected).abs() < 1e-9,
            "agent {id}: lost {lost}, formula {expected}"
        );
        let helper = exposure_damage(
            &rule,
            index,
            hardiness.map(|permille| Hardiness { permille }),
            f64::from(hunger),
        );
        assert!((helper - expected).abs() < 1e-12);
    }
    // Spot values: (700, 0) = 5 x 0.7 = 3.5; (1000, 75) = 5 x 1.25 = 6.25;
    // (1300, 99) = 5 x 1.3 x 1.49 = 9.685; child (1100, 60) = 7.5 x 1.1 x 1.1
    // = 9.075; elder (900, 40) = 5 x 0.9 = 4.5.
    assert!((100.0 - hp(&engine, entities[0]) - 3.5).abs() < 1e-9);
    assert!((100.0 - hp(&engine, entities[1]) - 6.25).abs() < 1e-9);
    assert!((100.0 - hp(&engine, entities[2]) - 5.0 * 1.3 * 1.49).abs() < 1e-9);
    assert!((100.0 - hp(&engine, entities[3]) - 7.5 * 1.1 * 1.1).abs() < 1e-9);
    assert!((100.0 - hp(&engine, entities[4]) - 4.5).abs() < 1e-9);
}

/// Run one full dark night (22:00..06:00, eight hourly `FireSystem` runs)
/// starting on display day `day`, and `StarvationSystem` every tick from the
/// first cold hour to the next night's 22:00 run (healing as the rules allow).
/// Returns whether `entity` is still alive after the night.
fn night_and_day(engine: &mut SimEngine, day: u64, entity: Entity) -> bool {
    let start = tick_at(day, 1320);
    let mut fire = FireSystem::new();
    let mut starvation = StarvationSystem::new();
    for tick in start..start + 1440 {
        engine.resources.current_tick = tick;
        if (tick - start).is_multiple_of(60) && tick < start + 8 * 60 {
            fire.tick(&mut engine.world, &mut engine.resources);
        }
        if !engine.world.contains(entity) {
            return false;
        }
        starvation.tick(&mut engine.world, &mut engine.resources);
    }
    engine.world.contains(entity)
}

#[test]
fn harness_exposure_adult_can_die() {
    let rules = rules();
    // Frail and hungry: hardiness 1300, hunger fixed at 99 (below
    // `Hunger::SATURATION` 100, so no starvation damage), thirst 0.
    let (mut engine, entities) = fixture(
        rules,
        tick_at(21, 1320),
        &[(1, Stage::Adult, Some(1300), 99.0, 100.0)],
    );
    let frail = entities[0];
    let per_hour = 5.0 * 1.3 * (1.0 + 0.5 * 0.98);
    assert!(
        night_and_day(&mut engine, 21, frail),
        "survives the first night"
    );
    // 8 x 9.685 = 77.48 hp lost on the first full dark night; no healing
    // (hunger > 50), so the 22.5 left last under three hours of the next.
    let after_first = hp(&engine, frail);
    assert!(
        (100.0 - after_first - 8.0 * per_hour).abs() < 1e-6,
        "lost {}",
        100.0 - after_first
    );
    assert!((77.0..78.0).contains(&(100.0 - after_first)));
    assert!(
        !night_and_day(&mut engine, 22, frail),
        "dies on the second night"
    );
    assert_eq!(engine.resources.tallies.deaths_exposure, 1);
    assert_eq!(
        engine.resources.tallies.deaths_starvation + engine.resources.tallies.deaths_dehydration,
        0
    );
    let width = engine.resources.tile_grid.width;
    let deaths: Vec<_> = engine
        .resources
        .causal_log
        .get(HOME.1 * width + HOME.0)
        .map(|log| {
            log.as_slice()
                .iter()
                .filter_map(|event| match event {
                    CausalEvent::AgentDied { agent, reason, .. } => Some((*agent, *reason)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    assert_eq!(deaths, vec![(1, DeathReason::Exposure)]);

    // Hardy and fed: hardiness 700, hunger 0 survives two dark nights (3.5 an
    // hour, 28 a night).
    let (mut engine, entities) = fixture(
        rules,
        tick_at(21, 1320),
        &[(1, Stage::Adult, Some(700), 0.0, 100.0)],
    );
    assert!(night_and_day(&mut engine, 21, entities[0]));
    assert!(night_and_day(&mut engine, 22, entities[0]));
    // Measured 44.0 = 100 - 2 x 28: no more than the two nights' damage.
    assert!(
        hp(&engine, entities[0]) >= 100.0 - 2.0 * 28.0 - 1e-9,
        "hp {}",
        hp(&engine, entities[0])
    );
    assert_eq!(engine.resources.tallies.deaths_exposure, 0);
}
