//! Pass 11 Part 1: a cold that can take anyone (exposure v2: hardiness,
//! hunger and per-stage damage) and rain that soaks the woodpile
//! (`game_v17_nofission`).
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
use sim_test::scenes::{game_v16_scene, game_v17_nofission_scene};

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

/// The goal's start values (section 6.2): the mechanism tests below use them
/// explicitly so their numbers do not move with Part 3's tuning.
const START: ExposureRule = ExposureRule {
    damage_per_cold_hour: [8.0, 6.0, 8.0],
    heal_block_hours: 2,
    hunger_weight: 0.5,
    use_hardiness: true,
};

/// v17 rules with the exposure start values.
fn start_rules() -> WorldRules {
    let mut rules = WorldRules::game_v17_nofission(0);
    rules.society.exposure = Some(START);
    rules
}

fn v17_exposure() -> ExposureRule {
    START
}

#[test]
fn harness_hardiness_keyed_and_ranged() {
    for seed in [3u64, 41, 900_017] {
        // Unforked and forked (replicate 3 from tick 1) runs of the same world.
        let mut plain = game_v17_nofission_scene(seed);
        let mut forked = game_v17_nofission_scene(seed);
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
        assert!(
            plain.resources.tallies.births + forked.resources.tallies.births > 0,
            "seed {seed}: births happened"
        );
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
    // The draw ignores the replicate: same key, same value; spread over the range.
    let values: std::collections::BTreeSet<u16> = (0..3_000u64)
        .map(|k| Hardiness::draw(5, k).permille)
        .collect();
    assert!(
        values.contains(&HARDINESS_MIN_PERMILLE) || values.iter().next().is_some_and(|v| *v < 710)
    );
    assert!(values.iter().next_back().is_some_and(|v| *v > 1290));
    // v16 worlds carry none.
    let v16 = game_v16_scene(3);
    assert_eq!(v16.world.query::<&Hardiness>().iter().count(), 0);
}

#[test]
fn harness_exposure_v2_formula() {
    let rule = v17_exposure();
    assert_eq!(rule.damage_per_cold_hour, [8.0, 6.0, 8.0]);
    let night = tick_at(21, 1320); // 22:00 of the first winter day, fire out
                                   // Three hardiness × hunger cases (adults), plus a child and an elder.
    let cases: [Resident; 5] = [
        (1, Stage::Adult, Some(700), 0.0, 100.0),
        (2, Stage::Adult, Some(1000), 75.0, 100.0),
        (3, Stage::Adult, Some(1300), 99.0, 100.0),
        (4, Stage::Child, Some(1100), 60.0, 100.0),
        (5, Stage::Elder, Some(900), 40.0, 100.0),
    ];
    let (mut engine, entities) = fixture(start_rules(), night, &cases);
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
    // Spot values: (700, 0) = 4.2; (1000, 75) = 6 × 1.25 = 7.5; (1300, 99) = 6 × 1.3 × 1.49.
    assert!((100.0 - hp(&engine, entities[0]) - 4.2).abs() < 1e-9);
    assert!((100.0 - hp(&engine, entities[1]) - 7.5).abs() < 1e-9);
    assert!((100.0 - hp(&engine, entities[2]) - 6.0 * 1.3 * 1.49).abs() < 1e-9);
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
    let rules = start_rules();
    // Frail and hungry: hardiness 1300, hunger fixed at 99 (below
    // `Hunger::SATURATION` 100, so no starvation damage), thirst 0.
    let (mut engine, entities) = fixture(
        rules,
        tick_at(21, 1320),
        &[(1, Stage::Adult, Some(1300), 99.0, 100.0)],
    );
    let frail = entities[0];
    let per_hour = 6.0 * 1.3 * (1.0 + 0.5 * 0.98);
    assert!(
        night_and_day(&mut engine, 21, frail),
        "survives the first night"
    );
    // ≈ 93 hp lost on the first full dark night; no healing (hunger > 50).
    let after_first = hp(&engine, frail);
    assert!(
        (100.0 - after_first - 8.0 * per_hour).abs() < 1e-6,
        "lost {}",
        100.0 - after_first
    );
    assert!((92.0..94.0).contains(&(100.0 - after_first)));
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

    // Hardy and fed: hardiness 700, hunger 0 survives two dark nights (4.2 an
    // hour, healed by day).
    let (mut engine, entities) = fixture(
        rules,
        tick_at(21, 1320),
        &[(1, Stage::Adult, Some(700), 0.0, 100.0)],
    );
    assert!(night_and_day(&mut engine, 21, entities[0]));
    assert!(night_and_day(&mut engine, 22, entities[0]));
    assert_eq!(engine.resources.tallies.deaths_exposure, 0);

    // The tuned v17 rule keeps both properties: the frail hungry adult dies
    // within two dark nights, the hardy fed one survives them.
    let tuned = WorldRules::game_v17_nofission(0);
    assert!(tuned.society.exposure.expect("exposure").use_hardiness);
    let (mut engine, entities) = fixture(
        tuned,
        tick_at(21, 1320),
        &[(1, Stage::Adult, Some(1300), 99.0, 100.0)],
    );
    let survived_first = night_and_day(&mut engine, 21, entities[0]);
    assert!(
        !survived_first || !night_and_day(&mut engine, 22, entities[0]),
        "frail adult dies within two nights"
    );
    let (mut engine, entities) = fixture(
        tuned,
        tick_at(21, 1320),
        &[(1, Stage::Adult, Some(700), 0.0, 100.0)],
    );
    assert!(
        night_and_day(&mut engine, 21, entities[0]) && night_and_day(&mut engine, 22, entities[0])
    );
}

#[test]
fn harness_v16_exposure_unchanged() {
    // Under v16's rule the v2 formula is exactly the stage rate, whatever the
    // hardiness and hunger.
    let rule = WorldRules::game_v16(0)
        .society
        .exposure
        .expect("v16 exposure");
    assert_eq!((rule.hunger_weight, rule.use_hardiness), (0.0, false));
    for stage in 0..3 {
        for hardiness in [
            None,
            Some(Hardiness { permille: 700 }),
            Some(Hardiness { permille: 1300 }),
        ] {
            for hunger in [0.0, 60.0, 99.0] {
                assert_eq!(
                    exposure_damage(&rule, stage, hardiness, hunger),
                    rule.damage_per_cold_hour[stage]
                );
            }
        }
    }
    // A v16 fire run: a hungry, frail-tagged adult loses exactly 4 an hour.
    let night = tick_at(21, 1320);
    let (mut engine, entities) = fixture(
        WorldRules::game_v16(0),
        night,
        &[
            (1, Stage::Adult, Some(1300), 99.0, 100.0),
            (2, Stage::Child, None, 0.0, 100.0),
        ],
    );
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(hp(&engine, entities[0]), 96.0);
    assert_eq!(hp(&engine, entities[1]), 90.0);
    // The v16 digests over 28 days are pinned in harness_revitalize_pass11_frozen.rs.
}
