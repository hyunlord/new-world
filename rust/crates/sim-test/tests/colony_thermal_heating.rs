//! Paid daytime room heat uses the existing primary-hearth buffer exactly once.
mod colony_support;
use colony_support::{adults, detect, open_area, walled_box};
use sim_core::components::{BuildingKind, Position, ResourceKind, SettlementId};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::settlement::{firewood, FireSystem};
use sim_systems::runtime::thermal;

fn cold_room() -> (SimEngine, SettlementId, hecs::Entity, (u32, u32)) {
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..10 {
        engine.tick();
    }
    let at = open_area(&engine, 5, 5);
    let tiles = walled_box(&mut engine, at, 5, 5, true);
    detect(&mut engine);
    let tile = tiles[4];
    let sid = *engine.resources.settlements.keys().min().expect("owner");
    engine
        .resources
        .building_kinds
        .insert(tile, BuildingKind::Hearth);
    let owner = engine.resources.settlements.get_mut(&sid).expect("owner");
    owner.formation_tile = tile;
    owner.withdraw(ResourceKind::Wood, u32::MAX);
    owner.store(ResourceKind::Wood, 100);
    let (_, person) = adults(&engine)[0];
    *engine.world.get::<&mut Position>(person).expect("position") = Position::new(tile.0, tile.1);
    engine.resources.rules.clock_offset_ticks = 0;
    engine.resources.current_tick = 21 * 1440 + 719;
    thermal::update_after_rooms(&mut engine.world, &mut engine.resources);
    engine.resources.current_tick = 21 * 1440 + 720;
    let room = engine.resources.colony.room_of_tile[&tile];
    let state = engine.resources.thermal.as_mut().expect("thermal");
    state.rooms.get_mut(&room).expect("room").temperature_d10 = state.rules.cold_threshold_d10 - 1;
    (engine, sid, person, tile)
}

fn wood(engine: &SimEngine, sid: SettlementId) -> u32 {
    engine.resources.settlements[&sid]
        .stockpile
        .get(&ResourceKind::Wood)
        .copied()
        .unwrap_or(0)
}

#[test]
fn occupied_cold_room_pays_one_positive_daytime_charge() {
    let (mut engine, sid, _, _) = cold_room();
    assert!(firewood::room_heat_needed(
        &engine.world,
        &engine.resources,
        sid
    ));
    let rate = firewood::daytime_service_rate(&engine.resources);
    assert!(rate > 0);
    let before = wood(&engine, sid);
    let ledger = engine.resources.colony.item_ledger.clone();
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(before - wood(&engine, sid), rate);
    let receipt = engine
        .resources
        .thermal
        .as_ref()
        .expect("thermal")
        .paid_hearths[&sid];
    assert_eq!((receipt.requested_wood, receipt.debited_wood), (rate, rate));
    assert_eq!(receipt.tick, engine.resources.current_tick);
    assert_eq!(
        engine.resources.colony.item_ledger, ledger,
        "buffer fuel was already booked by existing transfer"
    );
    assert!(engine.resources.fires[&sid].lit);
    let rule = engine.resources.rules.society.firewood.expect("fuel rule");
    assert_eq!(firewood::heating_hours(&engine.resources, &rule), 24);
}

#[test]
fn empty_warm_unroofed_or_nonprimary_room_does_not_request_heat() {
    for control in 0..4 {
        let (mut engine, sid, person, tile) = cold_room();
        let room = engine.resources.colony.room_of_tile[&tile];
        match control {
            0 => {
                *engine.world.get::<&mut Position>(person).expect("position") = Position::new(0, 0);
            }
            1 => {
                let state = engine.resources.thermal.as_mut().expect("thermal");
                state.rooms.get_mut(&room).expect("room").temperature_d10 =
                    state.rules.cold_threshold_d10 + 1000;
            }
            2 => {
                engine
                    .resources
                    .colony
                    .rooms
                    .get_mut(&room)
                    .expect("room")
                    .roofed = false;
            }
            3 => {
                engine
                    .resources
                    .settlements
                    .get_mut(&sid)
                    .expect("owner")
                    .formation_tile = (0, 0);
            }
            _ => unreachable!(),
        }
        assert!(
            !firewood::room_heat_needed(&engine.world, &engine.resources, sid),
            "control {control}"
        );
        let before = wood(&engine, sid);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        assert_eq!(
            wood(&engine, sid),
            before,
            "no daytime heat charge for control {control}"
        );
    }
}

#[test]
fn insufficient_buffer_is_debited_once_but_never_claims_paid_heat() {
    let (mut engine, sid, _, tile) = cold_room();
    engine
        .resources
        .settlements
        .get_mut(&sid)
        .expect("owner")
        .withdraw(ResourceKind::Wood, u32::MAX);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    let receipt = engine
        .resources
        .thermal
        .as_ref()
        .expect("thermal")
        .paid_hearths[&sid];
    assert!(receipt.requested_wood > 0);
    assert_eq!(receipt.debited_wood, 0);
    assert!(!engine.resources.fires[&sid].lit);
    assert!(!thermal::room_paid(
        &engine.resources,
        engine.resources.colony.room_of_tile[&tile]
    ));
}

#[test]
fn simultaneous_cooking_and_room_demand_share_one_payment() {
    use sim_core::colony::{Bill, BillMode, Job, JobKind, JobStage, Workshop};
    let (mut engine, sid, person, tile) = cold_room();
    let recipe = sim_core::colony::workshop::recipes()
        .iter()
        .find(|recipe| recipe.food_requirements.requires_lit_hearth)
        .expect("cooking recipe");
    let workshop = engine
        .resources
        .colony
        .workshops
        .keys()
        .next_back()
        .copied()
        .unwrap_or(0)
        + 1;
    engine.resources.colony.workshops.insert(
        workshop,
        Workshop {
            id: workshop,
            kind: recipe.workshop,
            top_left: tile,
            size: (1, 1),
            bills: vec![Bill {
                id: 1,
                recipe: recipe.id,
                mode: BillMode::Make(1),
                done: 0,
                suspended: false,
                auto: false,
            }],
            shelf: Vec::new(),
            active: Some(1),
        },
    );
    engine
        .world
        .insert_one(
            person,
            Job {
                kind: JobKind::Craft {
                    workshop,
                    bill: 1,
                    at: tile,
                },
                stage: JobStage::Working { ticks_left: 10 },
                taken_at: 719,
                amount: 0,
                item: recipe.output.0,
                material: None,
                fetch: None,
                best_distance: 0,
                progress_tick: 719,
            },
        )
        .expect("active cooking fixture");
    assert!(firewood::room_heat_needed(
        &engine.world,
        &engine.resources,
        sid
    ));
    // Existing cooking admission sees the same physical primary workshop and job.
    let rate = firewood::daytime_service_rate(&engine.resources);
    let before = wood(&engine, sid);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(before - wood(&engine, sid), rate, "one shared fuel service");
    assert_eq!(
        engine
            .resources
            .thermal
            .as_ref()
            .expect("thermal")
            .paid_hearths[&sid]
            .debited_wood,
        rate
    );
}

#[test]
fn baseline_has_no_physical_room_heating_service() {
    let (mut engine, sid, _, _) = cold_room();
    engine.resources.rules.colony = None;
    assert!(firewood::primary_hearth_room(&engine.resources, sid).is_none());
    let rule = engine.resources.rules.society.firewood.expect("fuel rule");
    assert_eq!(
        firewood::heating_hours(&engine.resources, &rule),
        firewood::burn_hours(&rule)
    );
    let before = wood(&engine, sid);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(wood(&engine, sid), before);
}

#[path = "colony_thermal_heating/preview.rs"]
mod preview;
