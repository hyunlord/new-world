//! Free physical inputs after existing work commitments, shared by steward and UI.
use hecs::World;
use sim_core::colony::building_site::{BuildingStatus, PiecePhase};
use sim_core::colony::{ColonySite, ItemKind, MaterialNeeds, ReserveKey};
use sim_engine::SimResources;
use std::collections::BTreeMap;

/// Unreserved ground inputs, less unfunded active needs and not-yet-active pieces.
/// Reserved source stacks and carried cargo are excluded. Deliver.amount already
/// funds matching outstanding needs, so those promises are not subtracted twice.
/// No future craft output, marked-tree yield, or authored default cost is credited.
pub fn available_materials(world: &World, res: &SimResources) -> BTreeMap<ItemKind, u32> {
    let mut free = BTreeMap::<ItemKind, u32>::new();
    for (tile, stack) in &res.colony.ground_items {
        if !res
            .colony
            .reservations
            .contains_key(&ReserveKey::Stack(*tile))
        {
            let count = free.entry(stack.kind).or_default();
            *count = count.saturating_add(u32::from(stack.count));
        }
    }
    let transit = crate::runtime::jobs::assign::in_transit(world);
    for (_, (needs, site)) in world
        .query::<(&MaterialNeeds, Option<&ColonySite>)>()
        .iter()
    {
        // A need may contain multiple lines of the same item/material. Aggregate
        // first, then deduct the delivery promise once per physical site/item.
        let mut outstanding = BTreeMap::<ItemKind, u32>::new();
        for need in needs.used() {
            let count = outstanding.entry(need.kind).or_default();
            *count = count.saturating_add(u32::from(need.needed.saturating_sub(need.delivered)));
        }
        for (item, count) in outstanding {
            let promised = site
                .and_then(|site| transit.get(&(site.centre(), item)))
                .copied()
                .map_or(0, u32::from);
            let stock = free.entry(item).or_default();
            *stock = stock.saturating_sub(count.saturating_sub(promised));
        }
    }
    if let Some(rule) = res.rules.colony {
        for site in res
            .colony
            .buildings
            .sites
            .values()
            .filter(|site| site.status == BuildingStatus::Building)
        {
            for piece in site
                .pieces
                .iter()
                .filter(|piece| matches!(piece.phase, PiecePhase::Pending | PiecePhase::Replacing))
            {
                for need in piece.kind.plan(&rule).1.used() {
                    let stock = free.entry(need.kind).or_default();
                    *stock = stock.saturating_sub(u32::from(need.needed));
                }
            }
        }
    }
    free
}
