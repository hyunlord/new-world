//! A1 presentation contracts for routine phases, FSM goals, peers and bed anchors.

mod routine_support;

use hecs::Entity;
use sim_core::colony::visible_action::{
    Phase, VisibleActionKind as Action, VisibleActionRow, VisibleTargetKind as Kind,
};
use sim_core::components::{
    Activity, ActivityKind, Agent, AgentState, BuildingKind, ConstructionSite, Hunger, Inventory,
    Kin, PorterKind, PorterStage, PorterTrip, Position, ResourceKind, SeekTarget, Settlement,
    TargetKind,
};
use sim_core::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::AgentMovementSystem;
use sim_systems::visible_action_rows;

fn row(engine: &SimEngine, entity: Entity) -> VisibleActionRow {
    let id = agent_id(engine, entity);
    visible_action_rows(&engine.world, &engine.resources)
        .into_iter()
        .find(|row| row.agent_id == id)
        .expect("live row")
}

fn agent_id(engine: &SimEngine, entity: Entity) -> u64 {
    engine.world.get::<&Agent>(entity).expect("agent").id
}

fn seek(engine: &mut SimEngine, entity: Entity, target: TargetKind, tile: (u32, u32)) {
    engine
        .world
        .insert(
            entity,
            (AgentState::Seeking { target }, SeekTarget::new(tile)),
        )
        .expect("seek");
}

fn perform(engine: &mut SimEngine, entity: Entity, target: TargetKind) {
    engine
        .world
        .insert_one(entity, AgentState::Consuming { target })
        .expect("perform");
}

#[test]
fn harness_activity_variants_and_legs_project() {
    use routine_support::{activity, decide, fixture, resident};

    let mut world = fixture();
    let adult = resident(&mut world, 7);

    for (kind, expected) in [
        (ActivityKind::TendFire, Action::TendFire),
        (ActivityKind::Craft, Action::RoutineHandwork),
        (ActivityKind::Repair, Action::RoutineHandwork),
        (ActivityKind::HearthCircle, Action::Sit),
        (ActivityKind::Explore, Action::Explore),
        (ActivityKind::Play, Action::Play),
        (ActivityKind::SitByFire, Action::Sit),
    ] {
        activity(&mut world, adult, kind, false);
        let approach = row(&world, adult);
        assert_eq!(
            (approach.action, approach.phase),
            (expected, Phase::Approach)
        );
        assert_eq!(approach.goal, Some((24, 23)));
        assert_eq!(
            (approach.started_at_tick, approach.until_tick),
            (None, None)
        );

        world
            .world
            .insert_one(adult, Position::new(24, 23))
            .expect("arrived");
        activity(&mut world, adult, kind, true);
        let performing = row(&world, adult);
        assert_eq!(
            (performing.action, performing.phase),
            (expected, Phase::Perform)
        );
        assert_eq!(performing.goal, None);
        assert_eq!(performing.started_at_tick, Some(600));
        assert_eq!(performing.until_tick, Some(690));
        assert_eq!(performing.partner, None);
        world
            .world
            .insert_one(adult, Position::new(24, 24))
            .expect("home");
    }

    // Follow an actual chosen activity through movement, arrival and need cancellation.
    let mut world = fixture();
    let child = resident(&mut world, 7);
    world
        .world
        .insert_one(child, sim_core::components::Child { born_tick: 0 })
        .expect("child");
    decide(&mut world);
    let approach = row(&world, child);
    assert_eq!(
        (approach.action, approach.phase),
        (Action::Play, Phase::Approach)
    );
    let place = approach.goal.expect("chosen tile");
    for _ in 0..4 {
        world.resources.current_tick += 1;
        AgentMovementSystem::new().tick(&mut world.world, &mut world.resources);
        decide(&mut world);
    }
    let arrived = row(&world, child);
    let activity = *world
        .world
        .get::<&Activity>(child)
        .expect("running activity");
    assert_eq!(
        (arrived.action, arrived.phase),
        (Action::Play, Phase::Perform)
    );
    assert_eq!(arrived.target.and_then(|target| target.tile), Some(place));
    assert_eq!(arrived.started_at_tick, activity.arrived_tick);
    assert!(arrived.started_at_tick > Some(activity.chosen_tick));
    assert_eq!(arrived.until_tick, activity.until_tick);

    world
        .world
        .insert_one(child, Hunger::new(60.0, 0.0))
        .expect("need");
    world.resources.set_food_tile(30, 30, 5);
    decide(&mut world);
    assert!(world.world.get::<&Activity>(child).is_err());
    assert_eq!(row(&world, child).action, Action::Eat);

    // Cover the other FSM targets directly at the read boundary.
    let pos = (30, 30);
    let peer = resident(&mut world, 19);
    let peer_id = agent_id(&world, peer);
    let child_id = agent_id(&world, child);
    world.resources.social_chase_since.insert(child_id, 0);
    world.resources.water_tiles.insert(pos, 5);
    world.resources.sleep_tiles.insert(pos, 5);
    world.resources.wood_tiles.insert(pos, 5);
    let blueprint = sim_core::components::shelter_blueprint(240);
    world.world.spawn((ConstructionSite::new(
        blueprint,
        Position::new(pos.0, pos.1),
    ),));
    for (kind, action, target) in [
        (TargetKind::Food, Action::Eat, Kind::FoodStore),
        (TargetKind::Water, Action::Drink, Kind::Water),
        (TargetKind::Sleep, Action::Sleep, Kind::Tile),
        (TargetKind::ConstructionSite, Action::Build, Kind::Site),
        (TargetKind::Agent(peer_id), Action::Talk, Kind::Agent),
        (TargetKind::GatherFood, Action::Harvest, Kind::Plant),
        (TargetKind::GatherWood, Action::Carry, Kind::Stack),
    ] {
        seek(&mut world, child, kind, pos);
        let visible = row(&world, child);
        assert_eq!(
            (visible.action, visible.phase),
            (action, Phase::Approach),
            "{kind:?}"
        );
        assert_eq!(visible.goal, Some(pos));
        assert_eq!(visible.target.expect("physical target").kind, target);
    }
    // Job requires its independent job owner. A bare FSM goal cannot invent a pose.
    seek(&mut world, child, TargetKind::Job, pos);
    assert_eq!(row(&world, child).phase, Phase::Idle);
    world
        .world
        .insert_one(child, AgentState::Idle)
        .expect("idle");
    assert_eq!(row(&world, child).goal, None);

    println!(
        "routine_kinds=7/7 fsm_goals=9/9 routine_arrival=1 routine_cancel=1 projection_errors=0"
    );
}

#[test]
fn harness_activity_partners_and_beds_match_owners() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    let bed = (2, 2);
    let first = engine.spawn_agent(bed.0, bed.1);
    let second = engine.spawn_agent(bed.0, bed.1);
    let a = agent_id(&engine, first);
    let b = agent_id(&engine, second);

    engine
        .world
        .insert_one(
            first,
            Kin {
                partner: Some(b),
                ..Kin::default()
            },
        )
        .expect("spouse");
    engine
        .world
        .insert_one(
            second,
            Kin {
                partner: Some(a),
                ..Kin::default()
            },
        )
        .expect("spouse");
    assert_eq!(row(&engine, first).partner, None);
    assert_eq!(row(&engine, second).partner, None);
    perform(&mut engine, first, TargetKind::Agent(b));
    assert_eq!(row(&engine, first).partner, None);
    perform(&mut engine, second, TargetKind::Agent(a));
    assert_eq!(row(&engine, first).partner, Some(b));
    assert_eq!(row(&engine, second).partner, Some(a));

    let mut from = Settlement::new_with_id(1, 0);
    from.formation_tile = bed;
    engine.resources.settlements.insert(1, from);
    let mut to = Settlement::new_with_id(2, 0);
    to.formation_tile = (4, 4);
    engine.resources.settlements.insert(2, to);
    let mut inventory = Inventory::default();
    assert_eq!(inventory.add(ResourceKind::Food, 3), 0);
    engine
        .world
        .insert(
            second,
            (
                PorterTrip {
                    from: 1,
                    to: 2,
                    kind: PorterKind::Food,
                    amount: 3,
                    stage: PorterStage::Outbound,
                    depart_tick: 0,
                },
                inventory,
            ),
        )
        .expect("porter");
    seek(&mut engine, second, TargetKind::Agent(b), (4, 4));
    let porter = row(&engine, second);
    assert_eq!(porter.partner, None);
    assert_eq!(porter.held.expect("inventory cargo").qty, 3);
    assert_eq!(porter.phase, Phase::Transport);
    assert_eq!(row(&engine, first).partner, None);
    assert_eq!(
        engine
            .world
            .get::<&mut Inventory>(second)
            .expect("inventory")
            .remove(ResourceKind::Food, 3),
        3
    );
    assert_eq!(row(&engine, second).held, None);

    engine.world.despawn(second).expect("dead peer");
    assert_eq!(row(&engine, first).partner, None);
    let second = engine.spawn_agent(bed.0, bed.1);
    let b = agent_id(&engine, second);
    engine
        .resources
        .colony
        .beds
        .insert(bed, sim_core::colony::materials::OAK);
    engine.resources.bed_of.extend([(a, bed), (b, bed)]);

    perform(&mut engine, first, TargetKind::Sleep);
    seek(&mut engine, second, TargetKind::Sleep, bed);
    let approaching = row(&engine, second);
    assert_eq!(approaching.phase, Phase::Approach);
    assert_eq!(approaching.target.expect("sleep approach").kind, Kind::Tile);

    perform(&mut engine, second, TargetKind::Sleep);
    assert_eq!(
        row(&engine, first)
            .target
            .expect("assigned physical bed")
            .kind,
        Kind::Bed
    );
    assert_eq!(
        row(&engine, second)
            .target
            .expect("no duplicate anchor")
            .kind,
        Kind::Tile
    );
    engine.resources.colony.beds.remove(&bed);
    engine
        .resources
        .building_kinds
        .insert(bed, BuildingKind::Hearth);
    assert_eq!(
        row(&engine, first).target.expect("camp pose").kind,
        Kind::Tile
    );
    assert_eq!(
        row(&engine, second).target.expect("camp pose").kind,
        Kind::Tile
    );

    engine
        .resources
        .colony
        .beds
        .insert(bed, sim_core::colony::materials::OAK);
    engine
        .world
        .insert_one(first, Position::new(3, 2))
        .expect("different sleep tile");
    let away = row(&engine, first).target.expect("ordinary sleep");
    assert_eq!((away.kind, away.tile), (Kind::Tile, Some((3, 2))));
    println!("fake_partners=0 dead_partners=0 duplicate_bed_anchors=0 camp_bed_anchors=0");
}
