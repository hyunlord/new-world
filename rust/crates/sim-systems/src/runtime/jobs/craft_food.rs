//! Food recipe admission and atomic fresh shelf-input consumption.
use crate::runtime::food::{items, storage};
use hecs::World;
use sim_core::colony::food::{FoodOwner, FoodStackKey};
use sim_core::colony::workshop::Recipe;
use sim_core::colony::Workshop;
use sim_engine::SimResources;

fn local_pot(res: &SimResources, shop: &Workshop) -> bool {
    shop.shelf
        .iter()
        .any(|stack| stack.count > 0 && stack.kind.definition().pot_capacity > 0)
        || res.food.pots.values().any(|pot| {
            pot.tile == shop.work_tile()
                || (pot.tile.0 >= shop.top_left.0
                    && pot.tile.1 >= shop.top_left.1
                    && pot.tile.0 < shop.top_left.0.saturating_add(shop.size.0)
                    && pot.tile.1 < shop.top_left.1.saturating_add(shop.size.1))
        })
}

/// A reusable vessel to bring to the shelf; selected by data capability, never item name.
pub(super) fn pot_need(
    res: &SimResources,
    shop: &Workshop,
    recipe: &Recipe,
    practice: u32,
) -> Option<sim_core::colony::ItemKind> {
    let requirements = &recipe.food_requirements;
    let knows = requirements.knowledge_any.is_empty()
        || requirements
            .knowledge_any
            .iter()
            .any(|code| practice & (1_u32 << code) != 0);
    let needed = requirements.requires_pot || (requirements.pot_alternative && !knows);
    if !needed || local_pot(res, shop) {
        return None;
    }
    res.colony
        .ground_items
        .iter()
        .filter(|(tile, stack)| {
            stack.count > 0
                && stack.kind.definition().pot_capacity > 0
                && !res
                    .colony
                    .reservations
                    .contains_key(&sim_core::colony::ReserveKey::Stack(**tile))
                && (!res.nav.is_built()
                    || res.nav.reaches(res.nav.region_of(shop.work_tile()), **tile))
        })
        .min_by_key(|(tile, stack)| {
            (
                super::items::chebyshev(shop.work_tile(), **tile),
                tile.1,
                tile.0,
                stack.kind,
            )
        })
        .map(|(_, stack)| stack.kind)
}

/// Planning may admit an existing reachable vessel that a supply job must still bring.
pub(crate) fn supplyable(
    res: &SimResources,
    shop: &Workshop,
    recipe: &Recipe,
    practice: u32,
) -> bool {
    requirements(
        res,
        shop,
        recipe,
        practice,
        local_pot(res, shop) || pot_need(res, shop, recipe, practice).is_some(),
    )
}

/// Check authored cooking requirements against this workshop and practitioner.
pub(crate) fn permitted(
    res: &SimResources,
    shop: &Workshop,
    recipe: &Recipe,
    practice: u32,
) -> bool {
    requirements(res, shop, recipe, practice, local_pot(res, shop))
}

fn requirements(
    res: &SimResources,
    shop: &Workshop,
    recipe: &Recipe,
    practice: u32,
    pot: bool,
) -> bool {
    let requirements = &recipe.food_requirements;
    let inside = |tile: (u32, u32)| {
        tile.0 >= shop.top_left.0
            && tile.1 >= shop.top_left.1
            && tile.0 < shop.top_left.0.saturating_add(shop.size.0)
            && tile.1 < shop.top_left.1.saturating_add(shop.size.1)
    };
    if requirements.requires_lit_hearth
        && !res.settlements.iter().any(|(id, settlement)| {
            inside(settlement.formation_tile) && res.fires.get(id).is_some_and(|fire| fire.lit)
        })
    {
        return false;
    }
    if requirements.requires_pot && !pot {
        return false;
    }
    requirements.knowledge_any.is_empty()
        || requirements
            .knowledge_any
            .iter()
            .any(|code| practice & (1_u32 << code) != 0)
        || (requirements.pot_alternative && pot)
}

/// Consume every recipe input only after all food stacks pass freshness checks.
pub(super) fn consume(
    world: &mut World,
    res: &mut SimResources,
    workshop: u32,
    recipe: &Recipe,
) -> Option<Option<sim_core::material::MaterialId>> {
    let shop = res.colony.workshops.get(&workshop)?;
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).ok()?;
    let mut plan = Vec::new();
    let mut material = None;
    for (input, &(kind, count)) in recipe.inputs.iter().enumerate() {
        let mut left = count;
        for stack in shop.shelf.iter().filter(|stack| stack.kind == kind) {
            if left == 0 {
                break;
            }
            let owner = FoodOwner::Shelf {
                workshop,
                key: FoodStackKey::from(stack.spec()),
            };
            if kind.definition().food.is_some()
                && storage::remaining_days(world, res, owner, day).ok()? == 0
            {
                return None;
            }
            if input == 0 && material.is_none() {
                material = stack.material;
            }
            let amount = left.min(stack.count);
            plan.push((owner, amount));
            left -= amount;
        }
        if left > 0 {
            return None;
        }
    }
    for (owner, count) in plan {
        // Admission above sees the same exclusively borrowed physical state.
        let batch = items::take(world, res, owner, count, day).ok()?;
        items::book_craft_input(res, batch);
    }
    Some(material)
}
