//! Passive nonfood production: one qualifying tick per JobExecuteSystem call.
//! Reads: workshop footprint, river terrain, authored recipe capacity, output space.
//! Writes: saved pending batches, physical ground outputs, craft ledger and losses.
//! Events: first woven output after committed placement; baseline is an empty no-op.
use sim_core::colony::workshop::Recipe;
use sim_core::colony::{Workshop, WorkshopId};
use sim_engine::SimResources;

/// Cardinal river contact beside a dry accessible processing footprint.
pub fn wet(res: &SimResources, tile: (u32, u32)) -> bool {
    if super::items::is_water(res, tile) || !crate::runtime::nav::passable(res, tile) {
        return false;
    }
    let (x, y) = tile;
    [
        x.checked_sub(1).map(|nx| (nx, y)),
        x.checked_add(1).map(|nx| (nx, y)),
        y.checked_sub(1).map(|ny| (x, ny)),
        y.checked_add(1).map(|ny| (x, ny)),
    ]
    .into_iter()
    .flatten()
    .any(|near| super::items::is_water(res, near))
}

/// Processing and ready-but-blocked output both retain their workshop slot.
pub fn pending_count(res: &SimResources, workshop: WorkshopId) -> usize {
    res.processing
        .pending
        .iter()
        .filter(|batch| batch.workshop == workshop)
        .count()
}

/// Whether a new active craft can reserve one passive completion slot.
pub fn permitted(res: &SimResources, shop: &Workshop, recipe: &Recipe) -> bool {
    recipe.process.as_ref().is_none_or(|process| {
        pending_count(res, shop.id) < usize::from(process.capacity)
            && (!process.requires_river || wet(res, shop.top_left))
            && crate::runtime::nav::passable(res, shop.work_tile())
    })
}

/// Advance pending crafts in saved insertion order, without offline elapsed credit.
pub fn advance(res: &mut SimResources) {
    let mut index = 0;
    while index < res.processing.pending.len() {
        let batch = res.processing.pending[index].clone();
        let Some(shop) = res.colony.workshops.get(&batch.workshop) else {
            res.colony.tallies.crafts_lost += 1;
            res.processing.pending.remove(index);
            continue;
        };
        if batch.remaining_ticks > 0 {
            if batch.requires_river && !wet(res, shop.top_left) {
                index += 1;
                continue;
            }
            res.processing.pending[index].remaining_ticks -= 1;
            if res.processing.pending[index].remaining_ticks > 0 {
                index += 1;
                continue;
            }
        }
        if !super::items::try_put_items(res, batch.tile, batch.output, batch.count) {
            index += 1;
            continue;
        }
        res.colony
            .item_ledger
            .craft_end(&[(batch.output.kind, batch.count)]);
        super::craft::note_woven_output(res, batch.output.kind, batch.count, None);
        res.processing.pending.remove(index);
    }
}
