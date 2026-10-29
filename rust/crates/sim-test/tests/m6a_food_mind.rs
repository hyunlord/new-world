//! Physical food choice through authored policy, target ownership and real effects.
use sim_core::colony::{ItemKind, ItemSpec};
use sim_core::components::{Agent, Hunger, Position, SeekTarget};
use sim_systems::runtime::food::{
    self,
    items::{create_ground, FoodSource},
};
use sim_systems::runtime::mind::{ColonyMindSystem, MindProgram};

const POLICY: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"cooked_novel",band:"need",behaviour:"eat",target:Some("food_cooked_novel"),when:[(fact:"hunger_actionable",op:"has"),(fact:"food_cooked_novel_available",op:"has")],score:(constant:40,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
(id:"cooked",band:"need",behaviour:"eat",target:Some("food_cooked"),when:[(fact:"hunger_actionable",op:"has"),(fact:"food_cooked_available",op:"has")],score:(constant:30,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
(id:"novel",band:"need",behaviour:"eat",target:Some("food_novel"),when:[(fact:"hunger_actionable",op:"has"),(fact:"food_novel_available",op:"has")],score:(constant:20,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
(id:"edible",band:"need",behaviour:"eat",target:Some("food"),when:[(fact:"hunger_actionable",op:"has")],score:(constant:10,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
(id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

fn fixture() -> (sim_engine::SimEngine, hecs::Entity, u64, ColonyMindSystem) {
    let mut engine = sim_test::colony_engine(0);
    let mut agents: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(entity, agent)| (agent.id, entity))
        .collect();
    agents.sort_unstable_by_key(|row| row.0);
    let (id, entity) = agents[0];
    for (_, other) in agents.into_iter().skip(1) {
        engine
            .world
            .despawn(other)
            .expect("isolated person's fixture");
    }
    engine.resources.mind = Default::default();
    engine
        .world
        .get::<&mut Hunger>(entity)
        .expect("hunger")
        .value = 70.0;
    let system = ColonyMindSystem::new(MindProgram::load(POLICY).expect("authored food policy"));
    (engine, entity, id, system)
}

fn place(
    engine: &mut sim_engine::SimEngine,
    origin: (u32, u32),
    item: &str,
    min_distance: u32,
) -> (u32, u32) {
    let tile = (0..engine.resources.tile_grid.height)
        .flat_map(|y| (0..engine.resources.tile_grid.width).map(move |x| (x, y)))
        .filter(|tile| origin.0.abs_diff(tile.0).max(origin.1.abs_diff(tile.1)) >= min_distance)
        .filter(|tile| !engine.resources.colony.ground_items.contains_key(tile))
        .filter(|tile| engine.resources.nav.reachable(origin, *tile))
        .min_by_key(|tile| {
            (
                origin.0.abs_diff(tile.0).max(origin.1.abs_diff(tile.1)),
                tile.1,
                tile.0,
            )
        })
        .expect("reachable unoccupied item tile");
    let day = food::day(&engine.resources);
    create_ground(
        &mut engine.resources,
        tile,
        ItemSpec::raw(ItemKind::from_name(item).expect("fixture item"), None),
        3,
        day,
        FoodSource::Harvest,
    )
    .expect("real item creation");
    tile
}

#[test]
fn cooked_preference_comes_from_data_and_consumption_requires_arrival() {
    let (mut engine, entity, id, mut system) = fixture();
    let position = *engine.world.get::<&Position>(entity).expect("position");
    let origin = (position.x, position.y);
    let raw = place(&mut engine, origin, "berries", 0);
    let cooked = place(&mut engine, origin, "porridge", 3);
    assert_ne!(raw, cooked);
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("select");
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("food target")
            .tile,
        cooked
    );
    assert_eq!(engine.resources.food.tallies.meals, 0);
    *engine.world.get::<&mut Position>(entity).expect("arrival") =
        Position::new(cooked.0, cooked.1);
    engine.resources.current_tick += 1;
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("arrive");
    assert_eq!(engine.resources.food.tallies.meals, 0);
    engine.resources.current_tick += 1;
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("consume");
    assert_eq!(engine.resources.food.tallies.meals, 1);
    assert_eq!(
        engine.resources.food.meals[&id].last_meals[0].item,
        ItemKind::from_name("porridge").expect("porridge")
    );
    assert_eq!(engine.resources.colony.ground_items[&raw].count, 3);
    assert_eq!(engine.resources.colony.ground_items[&cooked].count, 2);
}

#[test]
fn raw_grain_cannot_create_food_availability_or_an_actual_meal() {
    let (mut engine, entity, _, mut system) = fixture();
    let position = *engine.world.get::<&Position>(entity).expect("position");
    place(&mut engine, (position.x, position.y), "wild_grain", 0);
    for _ in 0..4 {
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("tick");
        engine.resources.current_tick += 1;
    }
    assert_eq!(engine.resources.food.tallies.meals, 0);
    assert_eq!(
        engine.world.get::<&Hunger>(entity).expect("hunger").value,
        70.0
    );
}

fn with_nut_history() -> (
    sim_engine::SimEngine,
    hecs::Entity,
    u64,
    ColonyMindSystem,
    (u32, u32),
    (u32, u32),
) {
    let (mut engine, entity, id, system) = fixture();
    let position = *engine.world.get::<&Position>(entity).expect("position");
    let origin = (position.x, position.y);
    let hazelnuts = place(&mut engine, origin, "hazelnuts", 0);
    let roots = place(&mut engine, origin, "roots", 3);
    engine.resources.food.meals.entry(id).or_default().record(
        sim_core::colony::food::MealRecord {
            tick: engine.resources.current_tick,
            item: ItemKind::from_name("hazelnuts").expect("hazelnuts"),
            pieces: 1,
            units: 1,
            cooked: false,
            source_owner: sim_core::colony::food::FoodOwner::Ground(origin),
            born_day: 0,
            storage_lifetime_days: 6,
            source_choice: None,
        },
        &["nut".to_owned()],
    );
    (engine, entity, id, system, hazelnuts, roots)
}

#[test]
fn unseen_groups_use_three_day_history_then_nearest_eligible_food() {
    // Each case starts a new decision: an admitted fresh owner remains pinned
    // during movement, independently of this novelty-window boundary.
    for at_boundary in [0, 1, 2] {
        let (mut engine, entity, id, mut system, hazelnuts, roots) = with_nut_history();
        let window = engine.resources.rules.calendar.days_to_ticks(3);
        engine.resources.current_tick += match at_boundary {
            0 => 0,
            1 => window - 1,
            _ => window,
        };
        let (selected, other, item) = if at_boundary == 2 {
            (hazelnuts, roots, "hazelnuts")
        } else {
            (roots, hazelnuts, "roots")
        };
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("new food decision");
        assert_eq!(
            engine
                .world
                .get::<&SeekTarget>(entity)
                .expect("target")
                .tile,
            selected
        );
        assert_eq!(engine.resources.food.tallies.meals, 0);
        *engine.world.get::<&mut Position>(entity).expect("arrival") =
            Position::new(selected.0, selected.1);
        engine.resources.current_tick += 1;
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("arrive");
        assert_eq!(engine.resources.food.tallies.meals, 0);
        engine.resources.current_tick += 1;
        system
            .try_tick(&mut engine.world, &mut engine.resources)
            .expect("consume");
        assert_eq!(engine.resources.food.tallies.meals, 1);
        assert_eq!(
            engine.resources.food.meals[&id]
                .last_meals
                .last()
                .expect("actual meal")
                .item,
            ItemKind::from_name(item).expect("item")
        );
        assert_eq!(engine.resources.colony.ground_items[&selected].count, 2);
        assert_eq!(engine.resources.colony.ground_items[&other].count, 3);
    }
}

#[test]
fn active_food_destination_survives_group_expiry_until_arrival() {
    let (mut engine, entity, id, mut system, hazelnuts, roots) = with_nut_history();
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("select roots");
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("target")
            .tile,
        roots
    );
    engine.resources.current_tick += engine.resources.rules.calendar.days_to_ticks(3);
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("retain admitted owner");
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("target")
            .tile,
        roots
    );
    assert_eq!(engine.resources.food.tallies.meals, 0);
    *engine.world.get::<&mut Position>(entity).expect("arrival") = Position::new(roots.0, roots.1);
    engine.resources.current_tick += 1;
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("arrive");
    engine.resources.current_tick += 1;
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("consume retained owner");
    assert_eq!(engine.resources.food.tallies.meals, 1);
    assert_eq!(
        engine.resources.food.meals[&id]
            .last_meals
            .last()
            .expect("actual meal")
            .item,
        ItemKind::from_name("roots").expect("roots")
    );
    assert_eq!(engine.resources.colony.ground_items[&roots].count, 2);
    assert_eq!(engine.resources.colony.ground_items[&hazelnuts].count, 3);
}

#[test]
fn authored_score_change_reverses_preference_without_provider_changes() {
    let (mut engine, entity, _, _) = fixture();
    let position = *engine.world.get::<&Position>(entity).expect("position");
    let origin = (position.x, position.y);
    let raw = place(&mut engine, origin, "berries", 0);
    place(&mut engine, origin, "porridge", 3);
    let source = POLICY
        .replace("constant:40", "constant:4")
        .replace("constant:30", "constant:3");
    let mut system =
        ColonyMindSystem::new(MindProgram::load(&source).expect("policy-only score change"));
    system
        .try_tick(&mut engine.world, &mut engine.resources)
        .expect("authored preference");
    assert_eq!(
        engine
            .world
            .get::<&SeekTarget>(entity)
            .expect("target")
            .tile,
        raw
    );
}
