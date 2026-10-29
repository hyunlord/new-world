//! A real vessel can fulfill a recipe's authored knowledge alternative through supply.
use super::craft_food;
use sim_core::colony::{ItemKind, ItemSpec, ItemStack, RecipeKind, Workshop};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use sim_engine::SimResources;

#[test]
fn missing_knowledge_can_plan_a_vessel_but_execution_waits_for_its_arrival() {
    let mut res = SimResources::new(8, 8, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    let mut recipe = RecipeKind::from_name("porridge")
        .unwrap()
        .recipe()
        .unwrap()
        .clone();
    // Isolate vessel admission from the separately tested actual-fire requirement.
    recipe.food_requirements.requires_lit_hearth = false;
    let mut shop = Workshop {
        id: 1,
        kind: recipe.workshop,
        top_left: (2, 2),
        size: (1, 1),
        bills: Vec::new(),
        shelf: Vec::new(),
        active: None,
    };
    let pot = ItemKind::all()
        .find(|kind| kind.definition().pot_capacity > 0)
        .unwrap();
    assert!(!craft_food::supplyable(&res, &shop, &recipe, 0));
    res.colony
        .ground_items
        .insert((3, 3), ItemStack::of(ItemSpec::raw(pot, None), 1));
    assert_eq!(craft_food::pot_need(&res, &shop, &recipe, 0), Some(pot));
    assert!(craft_food::supplyable(&res, &shop, &recipe, 0));
    assert!(!craft_food::permitted(&res, &shop, &recipe, 0));
    let practiced = 1_u32 << recipe.food_requirements.knowledge_any[0];
    assert_eq!(craft_food::pot_need(&res, &shop, &recipe, practiced), None);
    assert!(craft_food::permitted(&res, &shop, &recipe, practiced));
    shop.shelf
        .push(res.colony.ground_items.remove(&(3, 3)).unwrap());
    assert_eq!(craft_food::pot_need(&res, &shop, &recipe, 0), None);
    assert!(craft_food::permitted(&res, &shop, &recipe, 0));
}
