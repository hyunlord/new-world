//! Saved ordinary haul continuation and deterministic in-test command schedule replay.
use super::*;
use sim_systems::runtime::agent::AgentMovementSystem;
use sim_systems::runtime::construction::ConstructionSystem;
use sim_systems::runtime::jobs::{BillSystem, JobAssignSystem, JobExecuteSystem};
use sim_systems::save;

fn register_jobs(engine: &mut SimEngine) {
    engine.register_system(Box::new(AgentMovementSystem::new()));
    engine.register_system(Box::new(JobAssignSystem::new()));
    engine.register_system(Box::new(JobExecuteSystem::new()));
    engine.register_system(Box::new(ConstructionSystem::new()));
    engine.register_system(Box::new(BillSystem::new()));
}

fn restore(f: &mut Fixture) -> Fixture {
    let bytes = save::save(&mut f.engine).expect("save genuine haul checkpoint");
    let engine = save::load_with(&bytes, register_jobs).expect("restore same job systems");
    let hunter = engine
        .world
        .query::<&Agent>()
        .iter()
        .find_map(|(entity, agent)| (agent.id == f.id).then_some(entity))
        .expect("restored hunter by stable AgentId");
    Fixture {
        engine,
        hunter,
        id: f.id,
    }
}

fn equal(a: &mut Fixture, b: &mut Fixture) {
    invariants::check(&a.engine.world, &a.engine.resources).expect("original ledger");
    invariants::check(&b.engine.world, &b.engine.resources).expect("restored ledger");
    assert_eq!(job(a), job(b));
    assert_eq!(
        a.engine.resources.colony.reservations,
        b.engine.resources.colony.reservations
    );
    assert_eq!(a.engine.resources.food, b.engine.resources.food);
    assert_eq!(
        a.engine.resources.colony.command_log,
        b.engine.resources.colony.command_log
    );
    assert_eq!(
        save::save(&mut a.engine).expect("original bytes"),
        save::save(&mut b.engine).expect("twin bytes")
    );
}

fn continue_to_food(a: &mut Fixture, b: &mut Fixture) {
    for _ in 0..180 {
        tick_checked(&mut a.engine);
        tick_checked(&mut b.engine);
        assert_eq!(job(a), job(b));
        if job(a).is_none() {
            break;
        }
    }
    assert_released(a);
    assert_released(b);
    assert_eq!(ground(a, meat()), 2);
    assert_eq!(ground(b, meat()), 2);
    assert_eq!(food(a), 0);
    equal(a, b);
    a.engine.register_system(Box::new(AnimalSystem));
    b.engine.register_system(Box::new(AnimalSystem));
    for _ in 0..61 {
        tick_checked(&mut a.engine);
        tick_checked(&mut b.engine);
    }
    assert_eq!(a.engine.resources.colony.wildlife.tallies.meat_eaten, 0);
    assert_tracked_meat(a, 2);
    assert_eq!(a.engine.resources.food.tallies.eaten_units, 0);
    assert_eq!(b.engine.resources.colony.wildlife.tallies.meat_eaten, 0);
    assert_tracked_meat(b, 2);
    assert_eq!(b.engine.resources.food.tallies.eaten_units, 0);
    assert_eq!(ground(a, ItemKind::Hide), 1);
    assert_eq!(ground(b, ItemKind::Hide), 1);
    assert_eq!(food(a), 0);
    assert_eq!(food(b), 0);
    equal(a, b);
}

#[test]
fn saved_hearth_haul_continues_from_source_and_carry_with_identical_bytes() {
    for carrying in [false, true] {
        let mut original = fixture();
        kill(&mut original);
        haul(&original);
        if carrying {
            pickup(&mut original);
        }
        let mut restored = restore(&mut original);
        equal(&mut original, &mut restored);
        continue_to_food(&mut original, &mut restored);
    }
}

#[test]
fn hearth_removal_after_restore_cancels_with_identical_conserved_drop() {
    for carrying in [false, true] {
        let mut original = fixture();
        kill(&mut original);
        haul(&original);
        if carrying {
            pickup(&mut original);
        }
        let mut restored = restore(&mut original);
        remove_hearth(&mut original);
        remove_hearth(&mut restored);
        tick_checked(&mut original.engine);
        tick_checked(&mut restored.engine);
        assert_released(&original);
        assert_released(&restored);
        assert_eq!(ground(&original, meat()), 2);
        assert_eq!(ground(&original, ItemKind::Hide), 1);
        assert_eq!(food(&original), 0);
        assert_eq!(
            original
                .engine
                .resources
                .colony
                .tallies
                .cancels
                .get(&CancelReason::TargetGone),
            Some(&1)
        );
        equal(&mut original, &mut restored);
    }
}

#[test]
fn ordinary_hunt_command_schedule_replay_reproduces_haul_and_physical_storage() {
    let mut first = fixture_at(HEARTH, SOURCE, false);
    let mut replay = fixture_at(HEARTH, SOURCE, false);
    let animal = *first
        .engine
        .resources
        .colony
        .wildlife
        .animals
        .keys()
        .next()
        .expect("animal");
    assert!(replay
        .engine
        .resources
        .colony
        .wildlife
        .animals
        .contains_key(&animal));
    assert_eq!(first.engine.current_tick(), 0);
    assert_eq!(replay.engine.current_tick(), 0);
    let command = Command::DesignateHunt { animal };
    let receipt = submit(&mut first.engine, command.clone());
    let replay_receipt = submit(&mut replay.engine, command);
    assert!(receipt.accepted);
    assert_eq!(receipt, replay_receipt);
    // Replay the same tick-zero ordinary command in independently constructed worlds.
    // This deliberately does not claim CLI command-file parsing/replay coverage.
    kill(&mut first);
    kill(&mut replay);
    assert_eq!(haul(&first), haul(&replay));
    equal(&mut first, &mut replay);
    continue_to_food(&mut first, &mut replay);
}
