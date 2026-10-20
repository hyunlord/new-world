//! Pass 10 Part 1: exposure — a cold night wounds, by life stage, and health
//! can reach zero (`SocietyRules::exposure`, on the baseline rules: exposure
//! v2 with hardiness and the hunger weight).
use hecs::Entity;
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::{
    Agent, AgentId, BodyHealth, Child, Cold, Elder, ExposureClock, Hardiness, Hunger, Position,
    Settlement, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::settlement::FireSystem;
use sim_systems::runtime::survival::StarvationSystem;
use sim_test::probe::fingerprint;

/// World seed of the fixture rules (`WorldRules::baseline`).
const SEED: u64 = 7;

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

/// A bare world under the baseline rules at `tick`: one settlement at `HOME`
/// with an empty wood store (the fire goes out) and one member per
/// `(id, stage, hp)` row, spawned in the given order with its explicit
/// AgentId, needs met (hunger factor 1) and hardiness 1000 permille
/// (hardiness factor 1), so a cold hour costs exactly the rule's damage.
fn exposure_fixture(
    tick: u64,
    residents: &[(AgentId, Stage, f64)],
) -> (SimEngine, Vec<(AgentId, Entity)>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::baseline(SEED);
    engine.resources.world_seed = SEED;
    engine.resources.current_tick = tick;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = HOME;
    let mut entities = Vec::new();
    for &(id, stage, hp) in residents {
        let entity = engine.world.spawn((
            Position::new(HOME.0, HOME.1),
            Agent { id },
            Hunger::new(0.0, 0.0),
            Thirst::new(0.0, 0.0),
            BodyHealth { hp, max_hp: 100.0 },
            Hardiness { permille: 1000 },
        ));
        match stage {
            Stage::Child => engine
                .world
                .insert_one(entity, Child { born_tick: 0 })
                .expect("child"),
            Stage::Elder => engine.world.insert_one(entity, Elder).expect("elder"),
            Stage::Adult => {}
        }
        settlement.add_member_agent(id);
        entities.push((id, entity));
    }
    engine.resources.settlements.insert(1, settlement);
    (engine, entities)
}

/// Run `FireSystem` once at `tick`.
fn fire_at(engine: &mut SimEngine, tick: u64) {
    engine.resources.current_tick = tick;
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
}

fn hp(engine: &SimEngine, entity: Entity) -> f64 {
    engine.world.get::<&BodyHealth>(entity).expect("health").hp
}

/// One booked death: (event id, agent, reason, tick).
type Death = (u64, AgentId, DeathReason, u64);

/// Every `AgentDied` in the causal log at `HOME`, in push order: (event id, agent, reason, tick).
fn deaths_at_home(engine: &SimEngine) -> Vec<Death> {
    let tile = HOME.1 * engine.resources.tile_grid.width + HOME.0;
    engine
        .resources
        .causal_log
        .get(tile)
        .map(|log| {
            log.as_slice()
                .iter()
                .filter_map(|event| match event {
                    CausalEvent::AgentDied {
                        id,
                        agent,
                        reason,
                        tick,
                        ..
                    } => Some((*id, *agent, *reason, *tick)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The rule's hourly damage by stage: (child, adult, elder).
fn damage(engine: &SimEngine) -> (f64, f64, f64) {
    let d = engine
        .resources
        .rules
        .society
        .exposure
        .expect("exposure rule")
        .damage_per_cold_hour;
    (d[0], d[1], d[2])
}

/// An elder with `start` hp and `hardiness` permille beside a fresh adult and
/// child: the fire is out from 22:00; returns the cold hour (1-based) in which
/// the elder died.
fn elder_death_hour(start: f64, hardiness: u16) -> u64 {
    let night = tick_at(21, 1320); // 22:00 of the first winter day, fire out
    let (mut engine, residents) = exposure_fixture(
        night,
        &[
            (1, Stage::Elder, start),
            (2, Stage::Adult, 100.0),
            (3, Stage::Child, 100.0),
        ],
    );
    let (elder, adult, child) = (residents[0].1, residents[1].1, residents[2].1);
    engine
        .world
        .insert_one(
            elder,
            Hardiness {
                permille: hardiness,
            },
        )
        .expect("elder hardiness");
    let (dc, da, de) = damage(&engine);
    let de = de * f64::from(hardiness) / 1000.0;
    for hour in 1..=8u64 {
        fire_at(&mut engine, night + 60 * (hour - 1));
        assert!(!engine.resources.fires[&1].lit);
        assert_eq!(hp(&engine, adult), 100.0 - da * hour as f64);
        assert_eq!(hp(&engine, child), 100.0 - dc * hour as f64);
        if !engine.world.contains(elder) {
            assert_eq!(engine.resources.tallies.deaths_exposure, 1);
            assert_eq!(engine.resources.tallies.cold_member_hours, 3 * hour);
            let deaths = deaths_at_home(&engine);
            assert_eq!(deaths.len(), 1);
            assert_eq!(
                (deaths[0].1, deaths[0].2, deaths[0].3),
                (1, DeathReason::Exposure, night + 60 * (hour - 1))
            );
            assert!(!engine.resources.settlements[&1].member_agents.contains(&1));
            // Tallied once: the next hour wounds the others but kills no one else.
            fire_at(&mut engine, night + 60 * hour);
            assert_eq!(engine.resources.tallies.deaths_exposure, 1);
            assert_eq!(
                engine.resources.tallies.deaths_starvation
                    + engine.resources.tallies.deaths_old_age,
                0
            );
            return hour;
        }
        assert!(engine.world.get::<&Cold>(elder).is_ok());
        assert!((hp(&engine, elder) - (start - de * hour as f64)).abs() < 1e-9);
    }
    panic!("the elder survived the night");
}

#[test]
fn harness_exposure_kills_a_cold_elder() {
    // Baseline rule ([7.5, 5, 5] an hour): an elder at three hours' damage
    // dies in the third cold hour, half a point more lasts into the fourth.
    let rule = WorldRules::baseline(SEED).society.exposure.expect("rule");
    let tuned = rule.damage_per_cold_hour[2];
    assert_eq!(tuned, 5.0);
    assert!(rule.use_hardiness);
    assert_eq!(elder_death_hour(3.0 * tuned, 1000), 3);
    assert_eq!(elder_death_hour(3.0 * tuned + 0.5, 1000), 4);
    // Hardiness scales the damage: at 1300 permille (the frailest draw) the
    // same 15.5 hp lasts only into the third hour (6.5 an hour: 15.5 - 13 =
    // 2.5 > 0 after two), at 700 (the hardiest) into the fifth (3.5 an hour).
    assert_eq!(elder_death_hour(3.0 * tuned + 0.5, 1300), 3);
    assert_eq!(elder_death_hour(3.0 * tuned + 0.5, 700), 5);
}

#[test]
fn harness_exposure_blocks_healing() {
    let night = tick_at(21, 1320);
    let (mut engine, residents) = exposure_fixture(night, &[(1, Stage::Adult, 100.0)]);
    let adult = residents[0].1;
    // A whole dark night: eight cold hours, 22:00 .. 05:00.
    for hour in 0..8 {
        fire_at(&mut engine, night + 60 * hour);
    }
    let (_, da, _) = damage(&engine);
    let after_night = 100.0 - 8.0 * da;
    assert_eq!(hp(&engine, adult), after_night);
    let last_cold = night + 60 * 7;
    assert_eq!(
        engine
            .world
            .get::<&ExposureClock>(adult)
            .expect("clock")
            .last_cold_tick,
        last_cold
    );
    // While Cold: no healing even though both needs are met.
    let mut survival = StarvationSystem::new();
    engine.resources.current_tick = last_cold + 30;
    survival.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(hp(&engine, adult), after_night);
    // 06:00: the cold window ends, the marker leaves.
    fire_at(&mut engine, last_cold + 60);
    assert!(engine.world.get::<&Cold>(adult).is_err());
    assert_eq!(
        engine
            .world
            .get::<&ExposureClock>(adult)
            .expect("clock")
            .last_cold_tick,
        last_cold
    );
    // Not Cold, but inside the block (the marked hour plus two hours): no healing.
    for tick in [last_cold + 60, last_cold + 120, last_cold + 179] {
        engine.resources.current_tick = tick;
        survival.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(hp(&engine, adult), after_night, "tick {tick}");
    }
    // The block ends at last_cold + 60 + 60 x heal_block_hours (2): healing
    // resumes at 3 hp an hour.
    assert_eq!(
        engine
            .resources
            .rules
            .society
            .exposure
            .expect("rule")
            .heal_block_hours,
        2
    );
    engine.resources.current_tick = last_cold + 180;
    survival.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(hp(&engine, adult), after_night + 0.05);
    // A resident whose last cold hour lies a day back is outside every
    // block and heals on the first run.
    let (mut plain, residents) = exposure_fixture(night, &[(1, Stage::Adult, 52.0)]);
    let entity = residents[0].1;
    plain
        .world
        .insert_one(
            entity,
            ExposureClock {
                last_cold_tick: last_cold - 1440,
            },
        )
        .expect("clock");
    plain.resources.current_tick = last_cold + 30;
    StarvationSystem::new().tick(&mut plain.world, &mut plain.resources);
    assert_eq!(hp(&plain, entity), 52.05);
}

/// Run a full dark night (22:00..05:00) on the fixture `residents` spawned in
/// the given order; return the death log and the fingerprint.
fn night_outcome(residents: &[(AgentId, Stage, f64)]) -> (Vec<Death>, String, u64) {
    let night = tick_at(21, 1320);
    let (mut engine, _) = exposure_fixture(night, residents);
    for hour in 0..8 {
        fire_at(&mut engine, night + 60 * hour);
    }
    (
        deaths_at_home(&engine),
        fingerprint(&engine),
        engine.resources.tallies.deaths_exposure,
    )
}

#[test]
fn harness_exposure_order_independent() {
    // Several residents die in the same hour (same tick): deaths are booked,
    // event ids issued and agents despawned in AgentId order whatever the
    // spawn (hecs storage) order.
    let rows = [
        (4, Stage::Elder, 30.0),
        (2, Stage::Child, 25.0),
        (7, Stage::Adult, 20.0),
        (1, Stage::Elder, 26.0),
        (5, Stage::Adult, 100.0),
        (3, Stage::Child, 60.0),
        (6, Stage::Elder, 9.0),
    ];
    let forward = night_outcome(&rows);
    let mut reversed = rows;
    reversed.reverse();
    let backward = night_outcome(&reversed);
    let mut by_stage = rows;
    by_stage.sort_by_key(|row| (row.1 as u8, std::cmp::Reverse(row.0)));
    let grouped = night_outcome(&by_stage);
    assert!(
        forward.0.len() >= 4,
        "several exposure deaths: {:?}",
        forward.0
    );
    assert!(
        forward.0.windows(2).any(|w| w[0].3 == w[1].3),
        "at least two deaths share an hour"
    );
    for outcome in [&backward, &grouped] {
        assert_eq!(outcome, &forward);
    }
    // Within an hour the ids are ascending.
    for w in forward.0.windows(2) {
        if w[0].3 == w[1].3 {
            assert!(w[0].1 < w[1].1);
        }
    }
}
