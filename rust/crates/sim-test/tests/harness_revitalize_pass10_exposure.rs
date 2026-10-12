//! Pass 10 Part 1: exposure — a cold night wounds, by life stage, and health
//! can reach zero (`SocietyRules::exposure`, `game_v16_neutral`).
use std::collections::BTreeSet;
use std::sync::{Arc, Mutex};

use hecs::{Entity, World};
use sim_core::causal::event::{CausalEvent, DeathReason};
use sim_core::components::{
    Agent, AgentId, BodyHealth, Child, Cold, Elder, ExposureClock, Hunger, Position, Settlement,
    Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimEngine, SimResources};
use sim_systems::runtime::settlement::FireSystem;
use sim_systems::runtime::survival::StarvationSystem;
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v15_scene, game_v16_neutral_scene};
use sim_test::worldmodel::digest::state_digest;

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

/// A bare v16_neutral world at `tick`: one settlement at `HOME` with an empty
/// wood store (the fire goes out) and one member per `(id, stage, hp)` row,
/// spawned in the given order with its explicit AgentId, needs met.
fn exposure_fixture(tick: u64, residents: &[(AgentId, Stage, f64)]) -> (SimEngine, Vec<(AgentId, Entity)>) {
    let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::game_v16_neutral();
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
        ));
        match stage {
            Stage::Child => engine.world.insert_one(entity, Child { born_tick: 0 }).expect("child"),
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
                    CausalEvent::AgentDied { id, agent, reason, tick, .. } => Some((*id, *agent, *reason, *tick)),
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The v15 pins of `harness_revitalize_pass10_frozen.rs` (Part 0), seeds 0 and 1:
/// (seed, fingerprint at 40,320, digest at 40,320).
const V15_PINS: [(u64, &str, u64); 2] = [
    (0, "8f1794218ad25438", 0xb370_bddd_b2af_c8bc),
    (1, "00a294390654d3c1", 0xf706_ed0c_84bb_27ae),
];

#[test]
fn harness_exposure_inert_before_v16() {
    let rows: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = V15_PINS
            .iter()
            .map(|&(seed, ..)| {
                scope.spawn(move || {
                    let mut engine = game_v15_scene(seed);
                    for _ in 0..40_320 {
                        engine.tick();
                    }
                    let clocks = engine.world.query::<&ExposureClock>().iter().count();
                    (fingerprint(&engine), state_digest(&engine), clocks, engine.resources.tallies)
                })
            })
            .collect();
        handles.into_iter().map(|h| h.join().expect("seed thread")).collect()
    });
    let mut cold_hours = 0;
    for ((fp, digest, clocks, tallies), (seed, pin_fp, pin_digest)) in rows.into_iter().zip(V15_PINS) {
        assert_eq!((fp.as_str(), digest), (pin_fp, pin_digest), "seed {seed}: v15 differs from Part 0");
        assert_eq!(clocks, 0, "seed {seed}: no ExposureClock without the rule");
        assert_eq!(tallies.deaths_exposure, 0, "seed {seed}");
        cold_hours += tallies.cold_member_hours;
    }
    // v15 has cold nights (seed 0's first winter); they only speed hunger.
    assert!(cold_hours > 0);
    assert_eq!(WorldRules::game_v15().society.exposure, None);
}

#[test]
fn harness_exposure_kills_a_cold_elder() {
    let night = tick_at(21, 1320); // 22:00 of the first winter day, fire out
    let (mut engine, residents) =
        exposure_fixture(night, &[(1, Stage::Elder, 20.0), (2, Stage::Adult, 100.0), (3, Stage::Child, 100.0)]);
    let (elder, adult, child) = (residents[0].1, residents[1].1, residents[2].1);
    fire_at(&mut engine, night);
    assert!(!engine.resources.fires[&1].lit);
    assert!(engine.world.get::<&Cold>(elder).is_ok());
    assert_eq!(hp(&engine, elder), 11.0);
    assert_eq!(hp(&engine, adult), 94.0);
    assert_eq!(hp(&engine, child), 91.0);
    fire_at(&mut engine, night + 60);
    assert_eq!(hp(&engine, elder), 2.0);
    assert!(engine.world.contains(elder));
    fire_at(&mut engine, night + 120); // the third cold hour
    assert!(!engine.world.contains(elder), "an elder at 20 hp dies within three cold hours");
    assert_eq!(engine.resources.tallies.deaths_exposure, 1);
    assert_eq!(engine.resources.tallies.cold_member_hours, 3 + 3 + 3);
    let deaths = deaths_at_home(&engine);
    assert_eq!(deaths.len(), 1);
    assert_eq!((deaths[0].1, deaths[0].2, deaths[0].3), (1, DeathReason::Exposure, night + 120));
    assert!(!engine.resources.settlements[&1].member_agents.contains(&1));
    // Tallied once: later hours wound the others but kill no one else yet.
    fire_at(&mut engine, night + 180);
    assert_eq!(engine.resources.tallies.deaths_exposure, 1);
    assert_eq!(engine.resources.tallies.deaths_starvation + engine.resources.tallies.deaths_old_age, 0);
    assert_eq!((hp(&engine, adult), hp(&engine, child)), (76.0, 64.0));
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
    assert_eq!(hp(&engine, adult), 52.0);
    let last_cold = night + 60 * 7;
    assert_eq!(engine.world.get::<&ExposureClock>(adult).expect("clock").last_cold_tick, last_cold);
    // While Cold: no healing even though both needs are met.
    let mut survival = StarvationSystem::new();
    engine.resources.current_tick = last_cold + 30;
    survival.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(hp(&engine, adult), 52.0);
    // 06:00: the cold window ends, the marker leaves.
    fire_at(&mut engine, last_cold + 60);
    assert!(engine.world.get::<&Cold>(adult).is_err());
    assert_eq!(engine.world.get::<&ExposureClock>(adult).expect("clock").last_cold_tick, last_cold);
    // Not Cold, but inside the block (the marked hour plus two hours): no healing.
    for tick in [last_cold + 60, last_cold + 120, last_cold + 179] {
        engine.resources.current_tick = tick;
        survival.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(hp(&engine, adult), 52.0, "tick {tick}");
    }
    // The block ends at last_cold + 60 + 120: healing resumes at 3 hp an hour.
    engine.resources.current_tick = last_cold + 180;
    survival.tick(&mut engine.world, &mut engine.resources);
    assert_eq!(hp(&engine, adult), 52.05);
    // The same resident without the rule heals at once (v15 behaviour).
    let (mut plain, residents) = exposure_fixture(night, &[(1, Stage::Adult, 52.0)]);
    plain.resources.rules = WorldRules::game_v15();
    let entity = residents[0].1;
    plain.world.insert_one(entity, ExposureClock { last_cold_tick: last_cold }).expect("clock");
    plain.world.insert_one(entity, Cold).expect("cold");
    plain.resources.current_tick = last_cold + 30;
    StarvationSystem::new().tick(&mut plain.world, &mut plain.resources);
    assert_eq!(hp(&plain, entity), 52.05);
}

#[test]
fn harness_exposure_adult_survives_one_night() {
    let evening = tick_at(21, 1080); // 18:00: the burn window opens, no wood
    let (mut engine, residents) =
        exposure_fixture(evening, &[(1, Stage::Adult, 100.0), (2, Stage::Child, 100.0), (3, Stage::Elder, 100.0)]);
    // Every hourly run from 18:00 to 07:00: the cold window holds eight of them.
    for hour in 0..14 {
        fire_at(&mut engine, evening + 60 * hour);
    }
    assert_eq!(engine.resources.tallies.cold_member_hours, 3 * 8);
    let (adult, child, elder) = (residents[0].1, residents[1].1, residents[2].1);
    assert_eq!(hp(&engine, adult), 100.0 - 8.0 * 6.0);
    assert_eq!(hp(&engine, child), 100.0 - 8.0 * 9.0);
    assert_eq!(hp(&engine, elder), 100.0 - 8.0 * 9.0);
    assert_eq!(engine.resources.tallies.deaths_exposure, 0);
    assert!(engine.world.query::<&Cold>().iter().next().is_none());
}

/// Run a full dark night (22:00..05:00) on the fixture `residents` spawned in
/// the given order; return the death log and the fingerprint/digest.
fn night_outcome(residents: &[(AgentId, Stage, f64)]) -> (Vec<Death>, String, u64, u64) {
    let night = tick_at(21, 1320);
    let (mut engine, _) = exposure_fixture(night, residents);
    for hour in 0..8 {
        fire_at(&mut engine, night + 60 * hour);
    }
    (deaths_at_home(&engine), fingerprint(&engine), state_digest(&engine), engine.resources.tallies.deaths_exposure)
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
    assert!(forward.0.len() >= 4, "several exposure deaths: {:?}", forward.0);
    assert!(forward.0.windows(2).any(|w| w[0].3 == w[1].3), "at least two deaths share an hour");
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

/// Before `FireSystem` (priority 147): each agent's hp, life stage and
/// settlement at the start of the tick's fire run.
struct PreFire(Arc<Mutex<Vec<(AgentId, f64, usize)>>>);
impl RuntimeSystem for PreFire {
    fn name(&self) -> &str {
        "PreFireObserver"
    }
    fn priority(&self) -> u32 {
        146
    }
    fn tick_interval(&self) -> u64 {
        60
    }
    fn tick(&mut self, world: &mut World, _resources: &mut SimResources) {
        let rows = world
            .query::<(&Agent, &BodyHealth, Option<&Child>, Option<&Elder>)>()
            .iter()
            .map(|(_, (a, h, child, elder))| (a.id, h.hp, if child.is_some() { 0 } else if elder.is_some() { 2 } else { 1 }))
            .collect();
        *self.0.lock().expect("lock") = rows;
    }
}

/// Agents whose `AgentDied { reason: Exposure }` was booked at `tick`, over every tile.
fn exposure_deaths_at(engine: &SimEngine, tick: u64) -> Vec<AgentId> {
    let grid = &engine.resources.tile_grid;
    let mut ids = Vec::new();
    for tile in 0..grid.width * grid.height {
        if let Some(log) = engine.resources.causal_log.get(tile) {
            for event in log.as_slice() {
                if let CausalEvent::AgentDied { agent, reason: DeathReason::Exposure, tick: t, .. } = event {
                    if *t == tick {
                        ids.push(*agent);
                    }
                }
            }
        }
    }
    ids.sort_unstable();
    ids
}

#[test]
fn harness_exposure_only_from_cold() {
    // Seed 6 of game_v16_neutral loses villagers to exposure in its first
    // (harsh) winter. Every exposure death must be booked on a FireSystem run
    // (a multiple of 60) inside the cold window of a winter night, to a member
    // of a settlement whose fire is out, whose hp before the run was at most
    // that run's damage for its stage.
    let mut engine = game_v16_neutral_scene(6);
    let damage = engine.resources.rules.society.exposure.expect("rule").damage_per_cold_hour;
    let pre = Arc::new(Mutex::new(Vec::new()));
    engine.register_system(Box::new(PreFire(Arc::clone(&pre))));
    let mut seen = 0u64;
    let mut checked = 0usize;
    let end = tick_at(28, 420); // day 29 07:00: the first winter's last night is over
    while engine.current_tick() < end {
        let tick = engine.current_tick();
        let members_before: Vec<(u32, BTreeSet<AgentId>)> = {
            let mut rows: Vec<_> = engine
                .resources
                .settlements
                .values()
                .map(|s| (s.settlement_id, s.member_agents.iter().copied().collect::<BTreeSet<_>>()))
                .collect();
            rows.sort_by_key(|row| row.0);
            rows
        };
        engine.tick();
        let total = engine.resources.tallies.deaths_exposure;
        if total == seen {
            continue;
        }
        assert_eq!(tick % 60, 0, "exposure death off a FireSystem run at tick {tick}");
        let minute = (tick + engine.resources.rules.clock_offset_ticks) % 1440;
        assert!(!(360..1320).contains(&minute), "exposure death outside the cold window (minute {minute})");
        let evening_day = (tick + engine.resources.rules.clock_offset_ticks) / 1440 - u64::from(minute < 360);
        assert_eq!(evening_day % 28 / 7, 3, "the night of evening day {evening_day} is not a winter night");
        let pre_rows = pre.lock().expect("lock").clone();
        let died = exposure_deaths_at(&engine, tick);
        assert_eq!(died.len() as u64, total - seen, "every new exposure death is in this tick's causal log");
        for id in died {
            let (_, hp_before, stage) = *pre_rows.iter().find(|row| row.0 == id).expect("observed before the fire run");
            let (sid, _) = members_before.iter().find(|(_, m)| m.contains(&id)).expect("the dead was a settlement member");
            assert!(!engine.resources.fires[sid].lit, "agent {id} died of exposure beside a lit fire");
            assert!(hp_before <= damage[stage], "agent {id} had {hp_before} hp, more than one cold hour's damage");
            checked += 1;
        }
        seen = total;
    }
    assert!(seen >= 1, "seed 6 should lose someone to exposure in its first winter");
    assert_eq!(checked as u64, seen);
}
