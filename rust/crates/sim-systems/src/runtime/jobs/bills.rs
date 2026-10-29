//! BillSystem: picks each workshop's active bill (the first runnable one) and clears shelf items no bill needs.
//!
//! Priority: 123 (`BILL_PRIORITY`). Interval: 60 ticks. Gate: `WorldRules.colony` with `economy`.
//! Reads: `Job`, `Carry`; `SimResources.colony` (workshops and bills, ground items, zones, reservations), `rules.colony`
//! Writes: `SimResources.colony.workshops` (active bill, shelf), ground items (spilled shelf items), `knife_wear` (M5: reset without a shelved blade)
//! Events: none
//!
//! M2 goal 3.5. Workshops are visited in id order and their bills in list
//! order. A bill is **runnable** when it is not suspended, still has work
//! (`Make(n)`: fewer than `n` done; `Maintain(n)`: fewer than `n` items of its
//! output kind in the colony's stock, [`maintain_count`]) and its inputs for one craft
//! are on the shelf or available (unreserved stacks anywhere on the map, in
//! or out of stockpiles). The first runnable bill becomes `active`; the job
//! layer then supplies its inputs (`JobKind::Supply`) and crafts it
//! (`JobKind::Craft`). M4: under the knowledge rule a bill is runnable only
//! while some living adult practises its recipe's gate. Shelf items that no live bill of the workshop uses (a
//! bill removed, suspended or done) are put down at the work tile (merge and
//! spill), so nothing stays stranded on a shelf.
use hecs::World;
use sim_core::colony::workshop::Recipe;
use sim_core::colony::{BillMode, Carry, ItemKind, ItemSpec, Job, JobKind, ReserveKey, Workshop};
use sim_engine::{RuntimeSystem, SimResources};

use super::items::put_items;

/// Priority of [`BillSystem`]: before the job assignment (124).
pub const BILL_PRIORITY: u32 = 123;
/// Interval of [`BillSystem`] in ticks.
pub const BILL_INTERVAL: u64 = 60;

/// See the module header.
#[derive(Debug, Default)]
pub struct BillSystem;

impl BillSystem {
    /// A fresh system.
    pub fn new() -> Self {
        Self
    }
}

/// Unreserved items of `kind` on stockpile cells (the `Maintain` count).
pub fn stockpiled(res: &SimResources, kind: ItemKind) -> u32 {
    let c = &res.colony;
    c.ground_items
        .iter()
        .filter(|(tile, stack)| {
            stack.kind == kind
                && c.zone_of_tile.contains_key(tile)
                && !c.reservations.contains_key(&ReserveKey::Stack(**tile))
        })
        .map(|(_, stack)| u32::from(stack.count))
        .sum()
}

/// The count a `Maintain` bill compares with its target (DECISIONS D12): every
/// item of `kind` on the ground (stocked or waiting for a hauler), in a
/// hauler's hands, and the outputs of crafts in progress that make it.
pub fn maintain_count(world: &World, res: &SimResources, kind: ItemKind) -> u32 {
    let ground = res.colony.ground_total(kind) as u32;
    let mut carried = 0u32;
    let mut making = 0u32;
    for (_, (job, carry)) in world.query::<(&Job, Option<&Carry>)>().iter() {
        match job.kind {
            JobKind::Haul { .. } => {
                carried += carry
                    .filter(|c| c.kind == kind)
                    .map_or(0, |c| u32::from(c.count));
            }
            JobKind::Craft { workshop, bill, .. } => {
                let output = res
                    .colony
                    .workshops
                    .get(&workshop)
                    .and_then(|shop| shop.bill(bill))
                    .and_then(|b| b.recipe.recipe())
                    .map(|r| r.output);
                if let Some((_, n)) = output.filter(|o| o.0 == kind) {
                    making += u32::from(n);
                }
            }
            _ => {}
        }
    }
    let pending: u32 = res
        .food
        .pending_drying
        .iter()
        .filter(|batch| batch.output.kind == kind)
        .map(|batch| u32::from(batch.count))
        .sum();
    if kind.definition().food.is_some() {
        let carried: u32 = world
            .query::<&Carry>()
            .iter()
            .filter(|(_, carry)| carry.kind == kind)
            .map(|(_, carry)| u32::from(carry.count))
            .sum();
        let stored: u32 = res
            .colony
            .workshops
            .values()
            .flat_map(|shop| shop.shelf.iter())
            .chain(res.food.pots.values().flat_map(|pot| pot.contents.iter()))
            .filter(|stack| stack.kind == kind)
            .map(|stack| u32::from(stack.count))
            .sum();
        return ground + carried + stored + making + pending;
    }
    ground + carried + making + pending
}

/// The count a `Maintain` bill compares with its target (M3, ADR 0023): as
/// [`maintain_count`], except that while the colony has a roofed storehouse
/// only the items on the ground under its roof count (in hands and in the
/// making still do), so the colony keeps making until the stock is indoors.
pub fn bill_count(world: &World, res: &SimResources, kind: ItemKind) -> u32 {
    let all = maintain_count(world, res, kind);
    if !crate::runtime::rooms::storehouse_exists(res) {
        return all;
    }
    let outside: u32 = res
        .colony
        .ground_items
        .iter()
        .filter(|(t, s)| s.kind == kind && !crate::runtime::rooms::covered(res, **t))
        .map(|(_, s)| u32::from(s.count))
        .sum();
    all.saturating_sub(outside)
}

/// Unreserved ground items plus pot ingredients eligible for carrying.
pub fn available(res: &SimResources, kind: ItemKind) -> u32 {
    let c = &res.colony;
    c.ground_items
        .iter()
        .filter(|(tile, stack)| {
            stack.kind == kind && !c.reservations.contains_key(&ReserveKey::Stack(**tile))
        })
        .map(|(_, stack)| u32::from(stack.count))
        .sum::<u32>()
        .saturating_add(super::pot_supply::available(res, kind))
}

/// Reuse a shelved tool first, then the first unreserved compatible ground stack.
/// Missing supply retains the declared preferred tool as its outstanding demand.
pub fn selected_tool(res: &SimResources, shop: &Workshop, recipe: &Recipe) -> Option<ItemKind> {
    let tool = recipe.tool.as_ref()?;
    recipe
        .shelf_tool(shop)
        .or_else(|| {
            res.colony
                .ground_items
                .iter()
                .find(|(tile, stack)| {
                    stack.count > 0
                        && recipe.tool_matches(stack.kind)
                        && (!res.nav.is_built()
                            || res.nav.reaches(res.nav.region_of(shop.work_tile()), **tile))
                        && !res
                            .colony
                            .reservations
                            .contains_key(&ReserveKey::Stack(**tile))
                })
                .map(|(_, stack)| stack.kind)
        })
        .or(Some(tool.preferred_item))
}

/// Whether the bill at `index` of `shop` is runnable now (see the module header).
pub fn runnable(world: &World, res: &SimResources, shop: &Workshop, index: usize) -> bool {
    let Some(bill) = shop.bills.get(index) else {
        return false;
    };
    let Some(recipe) = bill.recipe.recipe() else {
        return false;
    };
    if bill.suspended || !recipe.supports_workshop(shop.kind) {
        return false;
    }
    let food_allowed = if crate::runtime::knowledge::rule(res).is_some() {
        world
            .query::<&sim_core::colony::knowledge::Knowledge>()
            .without::<&sim_core::components::Child>()
            .iter()
            .any(|(_, knowledge)| {
                super::craft_food::supplyable(res, shop, recipe, knowledge.practice())
            })
    } else {
        super::craft_food::supplyable(res, shop, recipe, u32::MAX)
    };
    if !food_allowed {
        return false;
    }
    // M4 (ADR 0024): a bill no living adult can craft waits (`NoPractitioner`).
    if crate::runtime::knowledge::rule(res).is_some()
        && !crate::runtime::knowledge::anyone_practises(
            world,
            sim_core::colony::knowledge::gates().recipe(recipe.id),
        )
    {
        return false;
    }
    let more = match bill.mode {
        BillMode::Make(n) => bill.done < u32::from(n),
        BillMode::Maintain(n) => bill_count(world, res, recipe.output.0) < u32::from(n),
    };
    more && recipe
        .needs_with_tool(selected_tool(res, shop, recipe))
        .iter()
        .all(|&(kind, n)| u32::from(shop.shelved(kind)) + available(res, kind) >= u32::from(n))
}

/// Recompute every workshop's active bill and clear unused shelf items.
pub fn schedule(world: &mut World, res: &mut SimResources) {
    let ids: Vec<u32> = res.colony.workshops.keys().copied().collect();
    for id in ids {
        let Some(shop) = res.colony.workshops.get(&id) else {
            continue;
        };
        let active = (0..shop.bills.len())
            .find(|&i| runnable(world, res, shop, i))
            .map(|i| shop.bills[i].id);
        // Shelf items stay while any live bill (not suspended, work left) uses them.
        let used: Vec<ItemKind> = shop
            .bills
            .iter()
            .filter(|b| !b.suspended && b.remaining() != Some(0))
            .filter_map(|b| b.recipe.recipe())
            .flat_map(|r| {
                let mut needs = r.needs();
                needs.extend(
                    shop.shelf
                        .iter()
                        .filter(|s| {
                            r.tool_matches(s.kind)
                                || ((r.food_requirements.requires_pot
                                    || r.food_requirements.pot_alternative)
                                    && s.kind.definition().pot_capacity > 0)
                        })
                        .map(|s| (s.kind, 1)),
                );
                needs.into_iter().map(|(kind, _)| kind)
            })
            .collect();
        let work_tile = shop.work_tile();
        let food_spill: Vec<_> = shop
            .shelf
            .iter()
            .filter(|stack| !used.contains(&stack.kind) && stack.kind.definition().food.is_some())
            .map(|stack| sim_core::colony::food::FoodStackKey::from(stack.spec()))
            .collect();
        let mut spill: Vec<(ItemSpec, u16)> = Vec::new();
        if let Some(shop) = res.colony.workshops.get_mut(&id) {
            shop.active = active;
            shop.shelf.retain(|stack| {
                let keep = used.contains(&stack.kind) || stack.kind.definition().food.is_some();
                if !keep {
                    spill.push((stack.spec(), stack.count));
                }
                keep
            });
        }
        for (spec, n) in spill {
            put_items(res, work_tile, spec, n);
        }
        let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
        for key in food_spill {
            let _ = crate::runtime::food::items::drop_ground(
                world,
                res,
                sim_core::colony::food::FoodOwner::Shelf { workshop: id, key },
                work_tile,
                u16::MAX,
                day,
            );
        }
    }
}

impl RuntimeSystem for BillSystem {
    fn name(&self) -> &str {
        "BillSystem"
    }

    fn priority(&self) -> u32 {
        BILL_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        BILL_INTERVAL
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        if resources.rules.colony.and_then(|r| r.economy).is_none() {
            return;
        }
        schedule(world, resources);
        // Wear follows the selected cutter, and is discarded when that cutter leaves.
        let default_cutter =
            sim_core::colony::content::installed_or_bundled().item_role("preferred_cutter");
        let colony = &mut resources.colony;
        let workshops = &colony.workshops;
        let cutters = &resources.content_state.workshop_cutters;
        colony.knife_wear.retain(|id, _| {
            let cutter = cutters.get(id).copied().or(default_cutter);
            workshops
                .get(id)
                .is_some_and(|shop| cutter.is_some_and(|kind| shop.shelved(kind) > 0))
        });
        resources.content_state.workshop_cutters.retain(|id, kind| {
            workshops
                .get(id)
                .is_some_and(|shop| shop.shelved(*kind) > 0)
        });
    }
}
