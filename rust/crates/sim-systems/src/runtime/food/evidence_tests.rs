//! Observation bounds and chronology do not alter physical food decisions.
use super::*;
use sim_core::colony::food::FoodOwner;
use sim_core::colony::{CancelReason, ItemKind, ItemSpec};
use sim_core::components::{Agent, Hunger};
use sim_core::rules::WorldRules;

fn scene() -> SimResources {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res
}
fn failure(tick: u64, reason: AcquisitionReason) -> AcquisitionReceipt {
    AcquisitionReceipt {
        tick,
        target: Some((2, 2)),
        job_started: Some(0),
        reason,
    }
}
#[test]
fn cancellation_does_not_erase_specific_same_job_failure() {
    let mut res = scene();
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::OutputPlacementFull),
    );
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::Cancelled(CancelReason::TargetGone)),
    );
    assert_eq!(
        res.food.evidence.active[&7].last.reason,
        AcquisitionReason::OutputPlacementFull
    );
    res.current_tick = 2;
    res.food.tallies.harvested_units = 4;
    res.food.tallies.eaten_units = 2;
    let index = death_parent(&mut res, 7).unwrap();
    let OperationEvidence::Acquisition { flow, receipt } = res.food.evidence.operations[&index]
    else {
        panic!("receipt")
    };
    assert_eq!((flow.from_tick, flow.to_tick), (0, 2));
    assert_eq!(flow.units.0[0], 4);
    assert_eq!(flow.units.0[3], 2);
    assert_eq!(receipt.reason, AcquisitionReason::OutputPlacementFull);
    assert!(!res.food.evidence.active.contains_key(&7));
}
#[test]
fn successful_meal_invalidates_old_failed_acquisition() {
    let mut res = scene();
    let mut world = hecs::World::new();
    let entity = world.spawn((Agent { id: 7 }, Hunger::new(90.0, 0.05)));
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::TargetUnavailable),
    );
    let item = ItemKind::from_name("roots").unwrap();
    crate::runtime::food::items::create_ground(
        &mut res,
        (2, 2),
        ItemSpec::raw(item, None),
        1,
        0,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .unwrap();
    crate::runtime::food::meals::consume(
        &mut world,
        &mut res,
        entity,
        FoodOwner::Ground((2, 2)),
        0,
    )
    .unwrap();
    assert!(death_parent(&mut res, 7).is_none());
}
#[test]
fn daily_flow_history_is_bounded_and_duplicate_boundary_is_inert() {
    let mut res = scene();
    observe_day(&mut res);
    for tick in 1..=7 {
        res.current_tick = tick;
        res.food.tallies.harvested_units += 3;
        res.food.tallies.spoiled_units += 1;
        observe_day(&mut res);
        observe_day(&mut res);
    }
    assert_eq!(res.food.evidence.recent.len(), 4);
    for row in &res.food.evidence.recent {
        assert_eq!(row.to_tick - row.from_tick, 1);
        assert_eq!(row.units.0[0], 3);
        assert_eq!(row.units.0[4], 1);
    }
}
#[test]
fn baseline_observers_are_inert() {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    record_failure(
        &mut res,
        7,
        failure(0, AcquisitionReason::OutputPlacementFull),
    );
    observe_day(&mut res);
    assert_eq!(res.food.evidence, Default::default());
}

fn expiry_scene() -> (hecs::World, SimResources, hecs::Entity) {
    use sim_core::components::{AgentState, BodyHealth, Position, Settlement};
    let mut res = scene();
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (1, 1);
    settlement.member_agents.insert(7);
    res.settlements.insert(1, settlement);
    let mut world = hecs::World::new();
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(1, 1),
        Hunger::new(80.0, 0.05),
        BodyHealth::new(),
        AgentState::Idle,
    ));
    (world, res, entity)
}
fn ground(res: &mut SimResources, tile: (u32, u32), name: &str, born: u32) {
    crate::runtime::food::items::create_ground(
        res,
        tile,
        ItemSpec::raw(ItemKind::from_name(name).unwrap(), None),
        2,
        born,
        crate::runtime::food::items::FoodSource::Debug,
    )
    .unwrap();
}
#[test]
fn expiry_requires_the_same_living_awake_hungry_member_without_edible_food() {
    use sim_core::components::{AgentState, BodyHealth, TargetKind};
    for scenario in [
        "hungry",
        "asleep",
        "threshold",
        "dead",
        "other_settlement",
        "fed",
        "grain",
        "unreachable",
        "different_people",
    ] {
        // Given: two expired roots and a precise personal/availability control.
        let (mut world, mut res, person) = expiry_scene();
        ground(&mut res, (2, 2), "roots", 0);
        res.current_tick = 30 * 1440;
        match scenario {
            "asleep" => {
                *world.get::<&mut AgentState>(person).unwrap() = AgentState::Consuming {
                    target: TargetKind::Sleep,
                }
            }
            "threshold" => {
                world.get::<&mut Hunger>(person).unwrap().value =
                    crate::runtime::agent_actions::needs::HUNGER_THRESHOLD
            }
            "dead" => world.get::<&mut BodyHealth>(person).unwrap().hp = 0.0,
            "other_settlement" => {
                res.settlements.get_mut(&1).unwrap().member_agents.clear();
                let mut other = sim_core::components::Settlement::new_with_id(2, 0);
                other.formation_tile = (7, 7);
                other.member_agents.insert(7);
                res.settlements.insert(2, other);
            }
            "different_people" => {
                *world.get::<&mut AgentState>(person).unwrap() = AgentState::Consuming {
                    target: TargetKind::Sleep,
                };
                world.spawn((
                    Agent { id: 8 },
                    sim_core::components::Position::new(1, 1),
                    Hunger::new(10.0, 0.05),
                    BodyHealth::new(),
                    AgentState::Idle,
                ));
                res.settlements.get_mut(&1).unwrap().member_agents.insert(8);
            }
            "fed" => ground(&mut res, (3, 3), "roots", 30),
            "grain" => ground(&mut res, (3, 3), "wild_grain", 30),
            "unreachable" => {
                for y in 0..8 {
                    res.nav.set_cost((4, y), 0);
                }
                ground(&mut res, (6, 3), "roots", 30);
            }
            _ => {}
        }
        // When: the real daily expiry removes the old stack.
        assert_eq!(
            crate::runtime::food::storage::spoil_day(&mut world, &mut res, 30).unwrap(),
            2
        );
        // Then: inedible/unreachable stock cannot feed this person; all other controls exclude.
        let expected = matches!(scenario, "hungry" | "grain" | "unreachable");
        assert_eq!(
            res.cultivation.expiry_hunger_days.get(&(30, 1)).copied(),
            expected.then_some(2),
            "{scenario}"
        );
        assert_eq!(res.food.tallies.spoiled_units, 2);
    }
}
#[test]
fn forced_migration_expiry_is_not_a_daily_expiry_observation() {
    let (mut world, mut res, _) = expiry_scene();
    ground(&mut res, (2, 2), "roots", 0);
    crate::runtime::food::storage::expire_owner(&mut world, &mut res, FoodOwner::Ground((2, 2)))
        .unwrap();
    assert!(res.cultivation.expiry_hunger_days.is_empty());
}

#[test]
fn expiry_day_validation_rejects_future_or_empty_observations() {
    let mut res = scene();
    for (key, units) in [((1, 1), 2), ((0, 1), 0), ((0, 0), 2)] {
        res.cultivation.expiry_hunger_days.clear();
        res.cultivation.expiry_hunger_days.insert(key, units);
        assert!(
            crate::runtime::food::evidence_validation::validate(&Default::default(), &res).is_err()
        );
    }
    res.cultivation.expiry_hunger_days.clear();
    res.cultivation.expiry_hunger_days.insert((0, 1), 2);
    assert!(crate::runtime::food::evidence_validation::validate(&Default::default(), &res).is_ok());
}
