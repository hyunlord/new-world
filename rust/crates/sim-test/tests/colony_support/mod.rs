//! Shared fixtures for the M1 colony tests (`colony_*.rs`).
//!
//! `job_world` is a small all-Plain map running only movement, the two job
//! systems and `ConstructionSystem` under the colony rule (no needs, no
//! decision), so a job's mechanics can be followed tick by tick. Tests of the
//! live behaviour (needs pre-empting a job, the hut's beds, viability) start
//! from `sim_test::colony_engine(seed)` instead.
#![allow(dead_code)]

use hecs::Entity;
use sim_core::colony::{Carry, ItemKind, ItemStack, Job, Tree};
use sim_core::components::{AgentState, Position};
use sim_core::rules::{ColonyRule, WorldRules};
use sim_core::MaterialRegistry;
use sim_engine::SimEngine;
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::construction::ConstructionSystem;
use sim_systems::runtime::jobs::invariants;
use sim_systems::runtime::jobs::{JobAssignSystem, JobExecuteSystem};

/// The M1 colony rule.
pub const RULE: ColonyRule = ColonyRule::M1;

/// A `w × h` Plain map with movement, jobs and construction only; agents
/// step every tick.
pub fn job_world(w: u32, h: u32) -> SimEngine {
    let mut engine = SimEngine::new(w, h, MaterialRegistry::new());
    engine.resources.rules = WorldRules {
        colony: Some(RULE),
        ..WorldRules::default()
    };
    engine.resources.rules.agent.move_interval_ticks = 1;
    engine.resources.colony.next_zone_id = 1;
    engine.register_system(Box::new(AgentMovementSystem::new()));
    engine.register_system(Box::new(JobAssignSystem::new()));
    engine.register_system(Box::new(JobExecuteSystem::new()));
    engine.register_system(Box::new(ConstructionSystem::new()));
    engine
}

/// Spawn an idle adult at `(x, y)`.
pub fn worker(engine: &mut SimEngine, x: u32, y: u32) -> Entity {
    let entity = engine.spawn_agent(x, y);
    let _ = engine.world.insert(
        entity,
        (MovementRng::new(7 + u64::from(x)), AgentState::Idle),
    );
    entity
}

/// Plant a tree with `logs` logs at `tile`.
pub fn plant(engine: &mut SimEngine, tile: (u32, u32), logs: u8) {
    engine.resources.colony.trees.insert(tile, Tree { logs });
}

/// Put `count` logs at `tile` as if a tree had yielded them (ledger created).
pub fn drop_logs(engine: &mut SimEngine, tile: (u32, u32), count: u16) {
    engine
        .resources
        .colony
        .item_ledger
        .create(ItemKind::Log, u64::from(count));
    engine.resources.colony.ground_items.insert(
        tile,
        ItemStack::of(sim_core::colony::ItemSpec::raw(ItemKind::Log, None), count),
    );
}

/// Tick once and assert the 7.6 invariants.
pub fn tick_checked(engine: &mut SimEngine) {
    engine.tick();
    if let Err(problem) = invariants::check(&engine.world, &engine.resources) {
        panic!("tick {}: {problem}", engine.current_tick());
    }
}

/// Tick until `done` holds (checking invariants every tick) or `limit` ticks pass;
/// returns the ticks used.
pub fn run_until(
    engine: &mut SimEngine,
    limit: u64,
    mut done: impl FnMut(&SimEngine) -> bool,
) -> u64 {
    for used in 0..limit {
        if done(engine) {
            return used;
        }
        tick_checked(engine);
    }
    limit
}

/// Logs on the ground.
pub fn ground_logs(engine: &SimEngine) -> u64 {
    engine.resources.colony.ground_total(ItemKind::Log)
}

/// The job `entity` holds, if any.
pub fn job_of(engine: &SimEngine, entity: Entity) -> Option<Job> {
    engine.world.get::<&Job>(entity).ok().map(|j| *j)
}

/// What `entity` carries, if anything.
pub fn carry_of(engine: &SimEngine, entity: Entity) -> Option<Carry> {
    engine.world.get::<&Carry>(entity).ok().map(|c| *c)
}

/// `entity`'s tile.
pub fn tile_of(engine: &SimEngine, entity: Entity) -> (u32, u32) {
    engine
        .world
        .get::<&Position>(entity)
        .map(|p| (p.x, p.y))
        .unwrap_or((u32::MAX, u32::MAX))
}

/// `entity`'s state.
pub fn state_of(engine: &SimEngine, entity: Entity) -> AgentState {
    engine
        .world
        .get::<&AgentState>(entity)
        .map(|s| *s)
        .unwrap_or_default()
}

/// The M2 colony rule (the economy on).
pub const RULE_M2: ColonyRule = ColonyRule::M2;

/// [`job_world`] under the M2 rule, with `BillSystem` registered too.
pub fn job_world_m2(w: u32, h: u32) -> SimEngine {
    let mut engine = job_world(w, h);
    engine.resources.rules.colony = Some(RULE_M2);
    engine.resources.colony.materials = sim_core::colony::ColonyMaterials::load();
    engine.register_system(Box::new(sim_systems::runtime::jobs::BillSystem::new()));
    engine
}

/// Put `count` items of `spec` at `tile` as if they had been made (ledger created).
pub fn drop_items(
    engine: &mut SimEngine,
    tile: (u32, u32),
    spec: sim_core::colony::ItemSpec,
    count: u16,
) {
    engine
        .resources
        .colony
        .item_ledger
        .create(spec.kind, u64::from(count));
    sim_systems::runtime::jobs::items::put_items(&mut engine.resources, tile, spec, count);
}

/// Place a blueprint of `kind` at `top_left`, drop its materials at `pile`
/// and run until it is a finished workshop; returns its id.
pub fn build_workshop(
    engine: &mut SimEngine,
    kind: sim_core::colony::BlueprintKind,
    top_left: (u32, u32),
    pile: (u32, u32),
) -> u32 {
    let placed = sim_systems::commands::submit(
        engine,
        sim_core::colony::Command::PlaceBlueprint {
            kind,
            x: top_left.0,
            y: top_left.1,
        },
    );
    assert!(placed.accepted, "{placed:?}");
    let (_, needs, _) = kind.plan(&RULE_M2);
    for line in needs.used() {
        let spec = sim_core::colony::ItemSpec::raw(line.kind, None);
        drop_items(engine, pile, spec, line.needed);
    }
    let before = engine.resources.colony.workshops.len();
    run_until(engine, 3_000, |e| {
        e.resources.colony.workshops.len() > before
    });
    let shop = engine
        .resources
        .colony
        .workshops
        .values()
        .find(|s| s.top_left == top_left)
        .map(|s| s.id);
    shop.expect("the workshop was built within 3,000 ticks")
}
