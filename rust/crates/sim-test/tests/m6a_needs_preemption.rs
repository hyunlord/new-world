//! Ordinary seed-5 U-script survival: trace critical thirst without state injection.
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::{Carry, Job};
use sim_core::components::{Agent, AgentState, Hunger, Position, SeekTarget, Thirst};
use sim_systems::commands::submit;

#[test]
fn seed_five_u_script_does_not_let_food_work_delay_drinking_until_death() {
    let mut engine = sim_test::colony_engine(5);
    let person = engine
        .world
        .query::<&Agent>()
        .iter()
        .find_map(|(entity, agent)| (agent.id == 3).then_some(entity))
        .expect("seed-five person 3");
    let mut last_transition = String::new();
    let mut was_critical = false;
    for _ in 0..3 * 1_440 {
        if engine.current_tick() == 360 {
            let commands = sim_test::probe::colony::u_script_commands(&engine);
            assert_eq!(commands.len(), 3, "exact ordinary U-script command set");
            for command in commands {
                let result = submit(&mut engine, command);
                eprintln!("u-script tick360: {result:?}");
                assert!(result.accepted, "ordinary U-script accepted: {result:?}");
            }
        }
        engine.tick();
        let thirst = engine.world.get::<&Thirst>(person).ok().map(|t| t.value);
        let critical = thirst.is_some_and(|value| value > 80.0);
        let transition = format!(
            "state={:?} job={:?} carry={:?} seek={:?} running={:?}",
            engine.world.get::<&AgentState>(person).ok().as_deref(),
            engine.world.get::<&Job>(person).ok().as_deref(),
            engine.world.get::<&Carry>(person).ok().as_deref(),
            engine.world.get::<&SeekTarget>(person).ok().as_deref(),
            engine.resources.mind.running.get(&3),
        );
        if (critical || was_critical || (700..=820).contains(&engine.current_tick()))
            && (transition != last_transition || engine.current_tick().is_multiple_of(60))
        {
            let choice = engine.resources.mind.history.as_ref().and_then(|history| {
                history.latest(3).map(|record| {
                    record
                        .decode_display(history.dictionary())
                        .expect("retained actual mind choice")
                })
            });
            eprintln!(
                "person3 tick={} pos={:?} thirst={thirst:?} hunger={:?} water_here={:?} water_seek={:?} water_ceiling_seek={:?} ceiling_counts={:?} {transition} choice={choice:?}",
                engine.current_tick(),
                engine.world.get::<&Position>(person).ok().as_deref(),
                engine.world.get::<&Hunger>(person).ok().as_deref(),
                engine.world.get::<&Position>(person).ok().and_then(|p| engine.resources.water_tiles.get(&(p.x,p.y)).copied()),
                engine.world.get::<&SeekTarget>(person).ok().and_then(|s| engine.resources.water_tiles.get(&s.tile).copied()),
                engine.world.get::<&SeekTarget>(person).ok().and_then(|s| engine.resources.water_source_max.get(&s.tile).copied()),
                (engine.resources.food_source_max.len(), engine.resources.water_source_max.len(), engine.resources.sleep_source_max.len()),
            );
        }
        if matches!(
            engine.world.get::<&AgentState>(person).ok().as_deref(),
            Some(AgentState::Seeking {
                target: sim_core::components::TargetKind::Water
            })
        ) {
            let target = engine
                .world
                .get::<&SeekTarget>(person)
                .expect("drink target after completed build")
                .tile;
            assert!(engine.resources.water_tiles.get(&target).is_some_and(|amount| *amount > 0),
                "completed build must not leave its non-water target attached: tick={} target={target:?}", engine.current_tick());
        }
        last_transition = transition;
        was_critical = critical;
        let deaths: Vec<_> = engine
            .resources
            .chronicle
            .entries()
            .iter()
            .filter(|entry| entry.kind == ChronicleKind::Died)
            .collect();
        assert!(
            deaths.is_empty(),
            "ordinary seed-five U-script must survive three days; tick={} deaths={deaths:?}",
            engine.current_tick()
        );
    }
}

#[test]
fn physical_food_to_water_handoff_replaces_the_old_seek_target() {
    use sim_core::colony::{ItemKind, ItemSpec};
    use sim_core::components::TargetKind;
    use sim_systems::runtime::food::{
        self,
        items::{create_ground, FoodSource},
    };
    use sim_systems::runtime::mind::{ColonyMindSystem, MindProgram};
    const POLICY: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"drink",band:"critical",behaviour:"drink",target:Some("water"),when:[(fact:"thirst_actionable",op:"has")],score:(constant:20,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"DRINK"),
(id:"eat",band:"need",behaviour:"eat",target:Some("food"),when:[(fact:"hunger_actionable",op:"has")],score:(constant:10,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
(id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    let mut engine = sim_test::colony_engine(5);
    let mut people: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(e, a)| (a.id, e))
        .collect();
    people.sort_unstable_by_key(|row| row.0);
    let (_, person) = people[0];
    for (_, other) in people.into_iter().skip(1) {
        engine
            .world
            .despawn(other)
            .expect("isolated handoff fixture");
    }
    engine.resources.mind = Default::default();
    engine
        .world
        .get::<&mut Hunger>(person)
        .expect("hunger")
        .value = 70.0;
    engine
        .world
        .get::<&mut Thirst>(person)
        .expect("thirst")
        .value = 0.0;
    let pos = *engine.world.get::<&Position>(person).expect("position");
    let tile = (0..engine.resources.tile_grid.height)
        .flat_map(|y| (0..engine.resources.tile_grid.width).map(move |x| (x, y)))
        .filter(|t| pos.x.abs_diff(t.0).max(pos.y.abs_diff(t.1)) >= 3)
        .filter(|t| !engine.resources.water_tiles.contains_key(t))
        .filter(|t| !engine.resources.colony.ground_items.contains_key(t))
        .filter(|t| engine.resources.nav.reachable((pos.x, pos.y), *t))
        .filter(|t| sim_systems::runtime::jobs::items::can_hold_items(&engine.resources, *t))
        .min_by_key(|t| (pos.x.abs_diff(t.0) + pos.y.abs_diff(t.1), t.1, t.0))
        .expect("reachable food tile distinct from water");
    let day = food::day(&engine.resources);
    create_ground(
        &mut engine.resources,
        tile,
        ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None),
        3,
        day,
        FoodSource::Debug,
    )
    .expect("tracked food");
    let mut mind = ColonyMindSystem::new(MindProgram::load(POLICY).expect("handoff policy"));
    mind.try_tick(&mut engine.world, &mut engine.resources)
        .expect("food choice");
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(person)
            .expect("food target")
            .tile,
        tile
    );
    engine.resources.current_tick += 1;
    engine
        .world
        .get::<&mut Thirst>(person)
        .expect("thirst")
        .value = 90.0;
    mind.try_tick(&mut engine.world, &mut engine.resources)
        .expect("drink interruption");
    assert_eq!(
        *engine.world.get::<&AgentState>(person).expect("state"),
        AgentState::Seeking {
            target: TargetKind::Water
        }
    );
    let target = engine
        .world
        .get::<&SeekTarget>(person)
        .expect("water target")
        .tile;
    assert_ne!(
        target, tile,
        "old physical-food target must not survive drink entry"
    );
    assert!(
        engine
            .resources
            .water_tiles
            .get(&target)
            .is_some_and(|count| *count > 0),
        "drink targets actual water"
    );
}
