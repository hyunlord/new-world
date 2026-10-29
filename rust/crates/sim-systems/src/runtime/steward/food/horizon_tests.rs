use hecs::World;
use sim_core::colony::food::PlantState;
use sim_core::colony::steward::{Action, FoodPlan};
use sim_core::colony::{Command, ItemKind, ItemSpec};
use sim_core::components::{Agent, Position, Settlement};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;

fn fixture() -> (World, SimResources, FoodPlan) {
    let mut world = World::new();
    let mut res = SimResources::new(16, 16, sim_core::MaterialRegistry::new());
    res.rules = WorldRules::colony(1);
    res.rules.clock_offset_ticks = 0;
    res.ticks_per_day = res.rules.calendar.ticks_per_day;
    res.current_tick = 14 * res.ticks_per_day;
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (8, 8);
    res.settlements.insert(1, settlement);
    world.spawn((Agent { id: 1 }, Position::new(8, 8)));
    assert!(
        crate::commands::apply_command(
            &mut world,
            &mut res,
            Command::AddStockpile {
                x0: 7,
                y0: 7,
                x1: 8,
                y1: 7
            }
        )
        .accepted
    );
    let mut policy = sim_core::colony::steward::rules()
        .expect("rules")
        .rules
        .iter()
        .find_map(|rule| match &rule.action {
            Action::FoodPlan(p) => Some(p.clone()),
            _ => None,
        })
        .expect("food policy");
    policy.winter_keep_days = 6;
    policy.recipes.clear();
    policy.vessel_recipes.clear();
    (world, res, policy)
}

#[test]
fn winter_horizon_counts_expiry_at_end_but_not_the_previous_day() {
    let (world, mut res, policy) = fixture();
    let roots = ItemKind::from_name("roots").expect("roots");
    for (tile, born, count) in [((7, 7), 13, 3), ((8, 7), 14, 5)] {
        crate::runtime::food::items::create_ground(
            &mut res,
            tile,
            ItemSpec::raw(roots, None),
            count,
            born,
            crate::runtime::food::items::FoodSource::Harvest,
        )
        .expect("actual stock");
    }
    let facts = super::summary(&world, &res, &policy);
    assert_eq!(facts.stored_units, 8);
    assert_eq!(
        facts.winter_units, 5,
        "expiry23 does not cover [18,24), expiry24 does"
    );
}

#[test]
fn winter_horizon_harvest_accepts_same_expiry_boundary_as_stored_food() {
    let (world, mut res, policy) = fixture();
    let catalog = sim_core::colony::content::installed_or_bundled();
    let species = catalog.plant_by_name("wild_root").expect("root plant");
    res.food.plants.insert(
        (9, 8),
        PlantState {
            species,
            units_left: 5,
            next_regrow_day: u32::MAX,
        },
    );
    let mut facts = super::summary(&world, &res, &policy);
    facts.usable_units = 1000;
    facts.groups.extend(
        ItemKind::from_name("roots")
            .expect("roots")
            .definition()
            .food
            .as_ref()
            .expect("food")
            .groups
            .iter()
            .cloned(),
    );
    let commands = super::harvest::plan(&world, &res, &policy, &facts);
    assert!(
        commands
            .iter()
            .any(|command| matches!(command, Command::DesignateHarvest { .. })),
        "fresh day14 roots expire24 and must fill the winter-only gap: {commands:?}"
    );
}
