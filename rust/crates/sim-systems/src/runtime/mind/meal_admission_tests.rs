//! Bundled meal policy cannot mistake reserved workshop ingredients for an edible target.
use super::{ColonyMindSystem, MindProgram};
use crate::runtime::food::items::{self, FoodSource};
use sim_core::colony::food::{FoodOwner, FoodStackKey, PlantState};
use sim_core::colony::{
    content::installed_or_bundled, BlueprintKind, ItemKind, ItemSpec, Job, JobKind, Workshop,
};
use sim_core::components::{
    Agent, AgentState, Hunger, Inventory, Position, Settlement, Sleep, Social, Thirst,
};
use sim_core::rules::WorldRules;
use sim_engine::{RuntimeSystem, SimResources};

fn fixture(water: bool, public_food: bool) -> (hecs::World, SimResources, hecs::Entity) {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 720;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    if water {
        res.water_tiles.insert((2, 2), u8::MAX);
    }
    let mut world = hecs::World::new();
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        AgentState::Idle,
        Hunger::new(60.0, 0.0),
        Thirst::new(0.0, 0.0),
        Sleep::new(0.0, 0.0),
        Social::new(0.0, 0.0),
        Inventory::default(),
    ));
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (2, 2);
    settlement.add_member_agent(7);
    res.settlements.insert(1, settlement);
    res.colony.zones.insert(
        1,
        sim_core::colony::Zone {
            id: 1,
            kind: sim_core::colony::ZoneKind::Stockpile,
            tiles: std::collections::BTreeSet::from([(2, 2)]),
        },
    );
    res.colony.zone_of_tile.insert((2, 2), 1);
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: BlueprintKind::HearthFire,
            top_left: (2, 1),
            size: (1, 1),
            bills: Vec::new(),
            shelf: Vec::new(),
            active: None,
        },
    );
    let roots = ItemSpec::raw(ItemKind::from_name("roots").expect("roots"), None);
    items::create_ground(&mut res, (2, 2), roots, 2, 0, FoodSource::Harvest).expect("food");
    if !public_food {
        items::transfer(
            &mut world,
            &mut res,
            FoodOwner::Ground((2, 2)),
            FoodOwner::Shelf {
                workshop: 1,
                key: FoodStackKey::from(roots),
            },
            2,
            0,
        )
        .expect("shelf");
    }
    res.food.plants.insert(
        (2, 2),
        PlantState {
            species: installed_or_bundled()
                .plant_by_name("wild_root")
                .expect("plant"),
            units_left: 5,
            next_regrow_day: 0,
        },
    );
    (world, res, entity)
}

#[test]
fn shelf_only_reserves_do_not_block_real_harvest_and_meal() {
    for water in [false, true] {
        let (mut world, mut res, entity) = fixture(water, false);
        assert_eq!(
            crate::runtime::food::reserves::units(&world, &res, 1, false),
            2
        );
        assert_eq!(
            sim_core::colony::knowledge::gates()
                .designation(sim_core::colony::Designation::Harvest),
            0,
            "fixture harvest requires no missing practice"
        );
        let mut mind = ColonyMindSystem::new(
            MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
                .expect("bundled policy"),
        );
        mind.try_tick(&mut world, &mut res).expect("selection");
        let policy = MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
            .expect("policy");
        eprintln!(
            "water={water}, actual node={}",
            policy
                .policy
                .node(res.mind.running[&7].node)
                .expect("selected node")
                .name
        );
        assert!(
            world
                .get::<&Job>(entity)
                .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { plant: (2, 2) })),
            "shelf-only reserve must not win a targetless meal, water={water}"
        );
        let mut jobs = crate::runtime::jobs::JobExecuteSystem::new();
        for tick in 721..=840 {
            res.current_tick = tick;
            jobs.tick(&mut world, &mut res);
            mind.try_tick(&mut world, &mut res).expect("progress");
        }
        assert_eq!(
            res.food.tallies.harvested_units, 7,
            "two fixture units plus actual five-unit harvest"
        );
        assert!(
            res.food.tallies.meals > 0,
            "harvest output must reach a real meal"
        );
        assert_eq!(
            res.colony.workshops[&1].shelved(ItemKind::from_name("roots").expect("roots")),
            2,
            "reserved inputs remain reserved"
        );
        crate::runtime::jobs::invariants::check(&world, &res).expect("physical conservation");
    }
}

#[test]
fn publicly_edible_food_still_completes_a_meal() {
    for water in [false, true] {
        let (mut world, mut res, entity) = fixture(water, true);
        let mut mind = ColonyMindSystem::new(
            MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
                .expect("bundled policy"),
        );
        for tick in 720..=724 {
            res.current_tick = tick;
            mind.try_tick(&mut world, &mut res).expect("meal");
        }
        assert!(res.food.tallies.meals > 0);
        assert!(world.get::<&Job>(entity).is_err());
        assert_eq!(res.food.plants[&(2, 2)].units_left, 5);
    }
}

#[test]
fn shelf_only_reserves_do_not_interrupt_a_routine_for_a_targetless_meal() {
    use sim_core::components::{Activity, ActivityKind, SeekTarget, TargetKind};
    for water in [false, true] {
        let (mut world, mut res, entity) = fixture(water, false);
        world
            .insert(
                entity,
                (
                    Activity {
                        kind: ActivityKind::SitByFire,
                        settlement: 1,
                        place: (2, 2),
                        chosen_tick: 710,
                        arrived_tick: Some(710),
                        until_tick: Some(900),
                    },
                    AgentState::Consuming {
                        target: TargetKind::Activity,
                    },
                    SeekTarget::new((2, 2)),
                ),
            )
            .expect("routine");
        let program = MindProgram::load(include_str!("../../../../sim-core/data/colony/mind.ron"))
            .expect("bundled");
        let forbidden = program
            .policy
            .node_id(if water {
                "meal_routine_water_not_critical"
            } else {
                "meal_routine_no_water"
            })
            .expect("node");
        let mut mind = ColonyMindSystem::new(program);
        mind.try_tick(&mut world, &mut res)
            .expect("routine selection");
        assert_ne!(
            res.mind.running[&7].node, forbidden,
            "shelf ingredients are not a real meal target"
        );
        assert!(
            world.get::<&Activity>(entity).is_ok(),
            "unavailable meal must not cancel routine ownership"
        );
        assert_eq!(res.food.tallies.meals, 0);
    }
}
