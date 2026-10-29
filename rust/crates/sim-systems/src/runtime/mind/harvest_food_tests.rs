//! Autonomous harvesting uses real reservations, work, dated output and meal consumption.
use super::*;
use sim_core::colony::food::PlantState;
use sim_core::colony::{content::installed_or_bundled, Command, Designation, Job, ReserveKey};
use sim_core::components::{Agent, AgentState, Hunger, Inventory, Position};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimResources};

const POLICY: &str = r#"(
 schema_version:1, bands:["critical","duty","need","work","leisure","idle"], nodes:[
 (id:"job_harvest",band:"need",behaviour:"harvest_food",target:Some("edible_plant"),when:[(fact:"job_owned_state",op:"has"),(fact:"job_harvesting",op:"not"),(fact:"hungry",op:"has"),(fact:"food_available",op:"not"),(fact:"edible_plant_available",op:"has")],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult"],tags:[],name_key:"HARVEST"),
 (id:"eat",band:"need",behaviour:"eat",target:Some("food"),when:[(fact:"hungry",op:"has"),(fact:"food_available",op:"has")],score:(constant:10,terms:[]),hysteresis:0,life_stages:["adult"],tags:[],name_key:"EAT"),
 (id:"harvest",band:"need",behaviour:"harvest_food",target:Some("edible_plant"),when:[(fact:"has_job",op:"not"),(fact:"hungry",op:"has"),(fact:"food_available",op:"not"),(fact:"edible_plant_available",op:"has")],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult"],tags:[],name_key:"HARVEST"),
 (id:"job",band:"work",behaviour:"do_job",target:Some("job"),when:[(fact:"job_owned_state",op:"has")],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult"],tags:[],name_key:"JOB"),
 (id:"idle",band:"idle",behaviour:"wander",when:[(fact:"idle",op:"has")],score:(constant:0,terms:[]),hysteresis:0,every_ticks:1,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

fn fixture() -> (hecs::World, SimResources, hecs::Entity) {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    res.food.plants.insert(
        (2, 2),
        PlantState {
            species: installed_or_bundled()
                .plant_by_name("wild_root")
                .expect("root"),
            units_left: 5,
            next_regrow_day: 0,
        },
    );
    let mut world = hecs::World::new();
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(80.0, 0.05),
        Inventory::default(),
    ));
    (world, res, entity)
}

#[test]
fn autonomous_harvest_runs_real_job_before_any_food_or_meal_exists() {
    let (mut world, mut res, entity) = fixture();
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
    let mut jobs = crate::runtime::jobs::JobExecuteSystem::new();
    assert!(!res.steward.enabled);
    assert!(res.colony.designations.is_empty());
    mind.try_tick(&mut world, &mut res)
        .expect("autonomous selection");
    assert_eq!(
        world
            .get::<&sim_core::components::SeekTarget>(entity)
            .expect("actual movement target")
            .tile,
        (2, 2)
    );
    assert!(matches!(
        world.get::<&Job>(entity).expect("real job").kind,
        sim_core::colony::JobKind::Harvest { plant: (2, 2) }
    ));
    assert_eq!(
        res.colony.reservations.get(&ReserveKey::Plant((2, 2))),
        Some(&7)
    );
    assert_eq!(
        res.colony.designations.get(&(2, 2)),
        Some(&Designation::Harvest)
    );
    assert_eq!(res.food.tallies.harvested_units, 0);
    assert_eq!(res.food.tallies.eaten_units, 0);
    jobs.tick(&mut world, &mut res);
    for tick in 1..=80 {
        res.current_tick = tick;
        mind.try_tick(&mut world, &mut res).expect("mind progress");
        jobs.tick(&mut world, &mut res);
    }
    assert_eq!(res.food.tallies.harvested_units, 5);
    assert!(
        res.food.tallies.eaten_units > 0,
        "the physical output was eaten"
    );
    assert_eq!(res.food.plants[&(2, 2)].units_left, 0);
    assert!(!res
        .colony
        .reservations
        .contains_key(&ReserveKey::Plant((2, 2))));
    crate::runtime::jobs::invariants::check(&world, &res).expect("physical conservation");
}

#[test]
fn player_cancel_stops_autonomous_work_and_veto_prevents_recreation() {
    let (mut world, mut res, entity) = fixture();
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
    mind.try_tick(&mut world, &mut res).expect("start");
    let cancelled = crate::commands::apply_command(
        &mut world,
        &mut res,
        Command::CancelArea {
            x0: 2,
            y0: 2,
            x1: 2,
            y1: 2,
        },
    );
    assert!(cancelled.accepted);
    assert!(world.get::<&Job>(entity).is_err());
    res.current_tick += 1;
    mind.try_tick(&mut world, &mut res)
        .expect("cancelled intent");
    assert!(world.get::<&Job>(entity).is_err());
    assert!(!res.colony.designations.contains_key(&(2, 2)));
    assert_eq!(res.food.plants[&(2, 2)].units_left, 5);
    assert_eq!(res.food.tallies.harvested_units, 0);
}

#[test]
fn plant_snapshot_checks_live_depletion_reservation_ripening_and_clock_reset() {
    let (world, mut res, _) = fixture();
    let mut cache = harvest_food::PlantCache::default();
    let index = cache.snapshot(&res);
    let context = context::TickContext::capture(&world, &res).expect("context");
    let person = context.row(7).expect("person");
    assert!(index.nearest(person, &res).is_some());
    res.colony.reservations.insert(ReserveKey::Plant((2, 2)), 8);
    assert!(index.nearest(person, &res).is_none());
    res.colony.reservations.clear();
    res.food.plants.get_mut(&(2, 2)).expect("plant").units_left = 0;
    assert!(index.nearest(person, &res).is_none());
    assert_eq!(
        index.len(),
        0,
        "depleted entries are pruned for later agents"
    );
    assert!(index.nearest(person, &res).is_none());
    res.current_tick = 1;
    let reused = cache.snapshot(&res);
    assert!(std::sync::Arc::ptr_eq(&index, &reused));
    res.current_tick = res.ticks_per_day;
    res.food.plants.get_mut(&(2, 2)).expect("plant").units_left = 5;
    let ripened = cache.snapshot(&res);
    assert!(!std::sync::Arc::ptr_eq(&index, &ripened));
    assert!(
        ripened.nearest(person, &res).is_some(),
        "daily ripening is captured"
    );
    let plant = res.food.plants.remove(&(2, 2)).expect("plant");
    res.food.plants.insert((3, 2), plant);
    res.current_tick = 0;
    let reset = cache.snapshot(&res);
    assert!(!std::sync::Arc::ptr_eq(&index, &reset));
    assert_eq!(
        reset.nearest(person, &res).expect("reset plant").tile,
        (3, 2)
    );
}

#[test]
fn winter_has_no_dormant_plant_candidates_to_scan() {
    let (world, mut res, _) = fixture();
    res.current_tick = 18 * res.ticks_per_day;
    let index = harvest_food::PlantCandidates::capture(&res);
    assert_eq!(index.len(), 0);
    let context = context::TickContext::capture(&world, &res).expect("context");
    assert!(index
        .nearest(context.row(7).expect("person"), &res)
        .is_none());
}

#[test]
fn harvest_start_refuses_breakdown_and_disabled_haul_even_for_custom_policy() {
    for broken in [true, false] {
        let (mut world, mut res, entity) = fixture();
        if broken {
            world
                .insert_one(
                    entity,
                    sim_core::colony::people::Mood {
                        breakdown_until: crate::runtime::mood::today(&res),
                        ..Default::default()
                    },
                )
                .expect("mood");
        } else {
            let mut priorities = sim_core::colony::WorkPriorities::defaults(false);
            priorities.levels[sim_core::colony::WorkType::Haul.code() as usize] = 0;
            world.insert_one(entity, priorities).expect("priorities");
        }
        // This custom policy intentionally omits the authored breakdown guard:
        // execution admission must still refuse jobs and leave plants untouched.
        let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
        mind.try_tick(&mut world, &mut res).expect("selection");
        assert!(world.get::<&Job>(entity).is_err());
        assert!(res.colony.reservations.is_empty());
        assert!(res.colony.designations.is_empty());
        assert_eq!(res.food.plants[&(2, 2)].units_left, 5);
        assert_eq!(res.food.tallies.harvested_units, 0);
    }
}

fn assigned_chop(world: &mut hecs::World, res: &mut SimResources, entity: hecs::Entity) -> Job {
    let job = Job {
        kind: sim_core::colony::JobKind::Chop { tree: (4, 4) },
        stage: sim_core::colony::JobStage::ToSource,
        taken_at: 0,
        amount: 0,
        item: sim_core::colony::ItemKind::role("default_placeholder"),
        material: None,
        fetch: None,
        best_distance: 2,
        progress_tick: 0,
    };
    world.insert_one(entity, job).expect("old job");
    world
        .insert_one(entity, sim_core::components::SeekTarget::new((4, 4)))
        .expect("old target");
    *world.get::<&mut AgentState>(entity).expect("state") = AgentState::Seeking {
        target: sim_core::components::TargetKind::Job,
    };
    res.colony.reservations.insert(ReserveKey::Tree((4, 4)), 7);
    res.colony.designations.insert((4, 4), Designation::Chop);
    job
}

#[test]
fn hungry_work_transfers_once_to_harvest_and_keeps_original_order() {
    let (mut world, mut res, entity) = fixture();
    assigned_chop(&mut world, &mut res, entity);
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
    mind.try_tick(&mut world, &mut res).expect("transfer");
    let harvest = *world.get::<&Job>(entity).expect("harvest job");
    assert_eq!(
        world
            .get::<&sim_core::components::SeekTarget>(entity)
            .expect("replacement target")
            .tile,
        (2, 2)
    );
    assert!(matches!(
        harvest.kind,
        sim_core::colony::JobKind::Harvest { .. }
    ));
    assert_eq!(
        res.colony.designations.get(&(4, 4)),
        Some(&Designation::Chop)
    );
    assert!(!res
        .colony
        .reservations
        .contains_key(&ReserveKey::Tree((4, 4))));
    assert_eq!(
        res.colony.reservations.get(&ReserveKey::Plant((2, 2))),
        Some(&7)
    );
    assert_eq!(
        res.colony
            .tallies
            .cancels
            .get(&sim_core::colony::CancelReason::Needs),
        Some(&1)
    );
    res.current_tick += 1;
    mind.try_tick(&mut world, &mut res)
        .expect("continue harvest");
    assert_eq!(*world.get::<&Job>(entity).expect("same job"), harvest);
    assert_eq!(
        res.colony
            .tallies
            .cancels
            .get(&sim_core::colony::CancelReason::Needs),
        Some(&1)
    );
}

#[test]
fn work_transfer_drops_carried_grain_with_age_or_retains_full_map_owner() {
    use crate::runtime::food::{self, items};
    use sim_core::colony::{food::FoodOwner, ItemKind, ItemSpec, ItemStack};
    for packed in [false, true] {
        let (mut world, mut res, entity) = fixture();
        res.current_tick = res.ticks_per_day;
        let grain = ItemSpec::raw(ItemKind::from_name("wild_grain").expect("grain"), None);
        items::create_ground(&mut res, (2, 2), grain, 2, 0, items::FoodSource::Harvest)
            .expect("grain");
        items::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 2)),
            FoodOwner::Carried(7),
            2,
            1,
        )
        .expect("carry");
        let old_job = assigned_chop(&mut world, &mut res, entity);
        if packed {
            let blocker = ItemSpec::raw(ItemKind::from_name("hide").expect("hide"), None);
            for y in 0..8 {
                for x in 0..8 {
                    res.colony
                        .ground_items
                        .insert((x, y), ItemStack::of(blocker, 1));
                }
            }
        }
        let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
        mind.try_tick(&mut world, &mut res).expect("selection");
        if packed {
            assert_eq!(*world.get::<&Job>(entity).expect("retained job"), old_job);
            assert_eq!(
                world
                    .get::<&sim_core::colony::Carry>(entity)
                    .expect("retained load")
                    .count,
                2
            );
            assert_eq!(res.food.ages[&FoodOwner::Carried(7)].born_day, 0);
            assert_eq!(
                res.colony.reservations.get(&ReserveKey::Tree((4, 4))),
                Some(&7)
            );
            assert_eq!(
                world
                    .get::<&sim_core::components::SeekTarget>(entity)
                    .expect("old target retained")
                    .tile,
                (4, 4)
            );
            assert!(!res.colony.designations.contains_key(&(2, 2)));
        } else {
            assert!(matches!(
                world.get::<&Job>(entity).expect("new job").kind,
                sim_core::colony::JobKind::Harvest { .. }
            ));
            assert!(world.get::<&sim_core::colony::Carry>(entity).is_err());
            assert_eq!(res.colony.ground_items[&(2, 2)].count, 2);
            assert_eq!(res.food.ages[&FoodOwner::Ground((2, 2))].born_day, 0);
            food::validate(&world, &res).expect("food conserved");
        }
    }
}

#[test]
fn autonomous_harvest_walks_to_remote_plant_before_yielding_food() {
    use crate::runtime::agent::{AgentMovementSystem, MovementRng};
    let (mut world, mut res, entity) = fixture();
    *world.get::<&mut Position>(entity).expect("position") = Position::new(6, 6);
    world
        .insert_one(entity, MovementRng::new(7))
        .expect("movement");
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).expect("policy"));
    let mut movement = AgentMovementSystem::new();
    let mut jobs = crate::runtime::jobs::JobExecuteSystem::new();
    mind.try_tick(&mut world, &mut res).expect("start");
    assert_eq!(
        world
            .get::<&sim_core::components::SeekTarget>(entity)
            .expect("movement target survived postpass")
            .tile,
        (2, 2)
    );
    assert_eq!(res.food.tallies.harvested_units, 0);
    for tick in 1..=240 {
        res.current_tick = tick;
        movement.tick(&mut world, &mut res);
        mind.try_tick(&mut world, &mut res).expect("mind");
        jobs.tick(&mut world, &mut res);
        if res.food.tallies.harvested_units > 0 {
            break;
        }
    }
    let pos = world.get::<&Position>(entity).expect("arrived");
    assert!(pos.x.abs_diff(2).max(pos.y.abs_diff(2)) <= 1);
    assert_eq!(res.food.tallies.harvested_units, 5);
    crate::runtime::jobs::invariants::check(&world, &res).expect("conserved after movement/work");
}

#[test]
fn disabled_harvest_admission_keeps_timestamped_observation_from_player_setting() {
    use sim_core::colony::food_evidence::AcquisitionReason;
    let (mut world, mut res, entity) = fixture();
    let command = crate::commands::apply_command(
        &mut world,
        &mut res,
        Command::SetWorkPriority {
            agent: 7,
            work: sim_core::colony::WorkType::Haul,
            level: 0,
        },
    );
    assert!(command.accepted);
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).unwrap());
    mind.try_tick(&mut world, &mut res).unwrap();
    assert!(world.get::<&Job>(entity).is_err());
    assert_eq!(res.food.plants[&(2, 2)].units_left, 5);
    let observed = res
        .food
        .evidence
        .active
        .get(&7)
        .expect("actual denied admission receipt");
    assert_eq!(observed.last.tick, res.current_tick);
    assert_eq!(observed.last.target, Some((2, 2)));
    assert_eq!(observed.last.job_started, None);
    assert_eq!(observed.last.reason, AcquisitionReason::WorkDisabled);
}

#[test]
fn actual_empty_region_query_records_absence_without_creating_food() {
    use sim_core::colony::food_evidence::AcquisitionReason;
    let (mut world, mut res, entity) = fixture();
    res.food.plants.clear();
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).unwrap());
    mind.try_tick(&mut world, &mut res).unwrap();
    assert!(world.get::<&Job>(entity).is_err());
    assert_eq!(res.food.tallies.harvested_units, 0);
    let receipt = &res
        .food
        .evidence
        .active
        .get(&7)
        .expect("observed regional absence")
        .last;
    assert_eq!(receipt.reason, AcquisitionReason::NoRipeInRegion);
    assert_eq!(receipt.target, None);
}

#[test]
fn reserved_ripe_candidate_is_not_mislabeled_as_no_ripe_in_region() {
    let (mut world, mut res, _) = fixture();
    res.colony
        .reservations
        .insert(ReserveKey::Plant((2, 2)), 99);
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).unwrap());
    mind.try_tick(&mut world, &mut res).unwrap();
    assert!(!res.food.evidence.active.contains_key(&7));
    assert_eq!(res.food.plants[&(2, 2)].units_left, 5);
}
