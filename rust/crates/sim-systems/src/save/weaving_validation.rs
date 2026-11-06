//! Validate physical pending-output authority without advancing or repairing saved work.
use sim_engine::SimResources;
use std::collections::BTreeMap;

pub(super) fn validate(res: &SimResources) -> Result<(), String> {
    let mut counts = BTreeMap::<u32, (usize, usize)>::new();
    for batch in &res.processing.pending {
        if batch.workshop == 0
            || batch.workshop >= res.colony.next_workshop_id
            || batch.bill == 0
            || batch.bill >= res.colony.next_bill_id
            || batch.count == 0
            || batch.output.quality > 100
            || batch.tile.0 >= res.tile_grid.width
            || batch.tile.1 >= res.tile_grid.height
            || batch.output.kind.definition().food.is_some()
        {
            return Err("invalid pending nonfood identity, output or tile".into());
        }
        let shop = res.colony.workshops.get(&batch.workshop);
        if shop.is_some_and(|shop| shop.work_tile() != batch.tile) {
            return Err("pending nonfood output is not at its workshop work tile".into());
        }
        // Bills are editable/removable after input consumption; do not reconstruct
        // their past value from a current bill. A destroyed shop is a valid pending
        // loss until JobExecuteSystem removes it, and cannot emit output there.
        let compatible = |recipe: &&sim_core::colony::Recipe| {
            shop.is_none_or(|shop| recipe.supports_workshop(shop.kind))
        };
        let recipes = sim_core::colony::recipes();
        let matched = recipes.iter().filter(compatible).any(|recipe| {
            recipe.process.as_ref().is_some_and(|process| {
                recipe.output == (batch.output.kind, batch.count)
                    && process.requires_river == batch.requires_river
                    && batch.remaining_ticks
                        <= res
                            .rules
                            .calendar
                            .days_to_ticks(u64::from(process.duration_days))
            })
        });
        if !matched {
            return Err("pending nonfood output has no matching authored process".into());
        }
        let capacity = recipes
            .iter()
            .filter(compatible)
            .filter_map(|recipe| {
                recipe
                    .process
                    .as_ref()
                    .map(|process| usize::from(process.capacity))
            })
            .max()
            .unwrap_or(0);
        let entry = counts.entry(batch.workshop).or_insert((0, 0));
        entry.0 += 1;
        entry.1 = entry.1.max(capacity);
    }
    if counts.values().any(|(count, capacity)| count > capacity) {
        return Err("pending nonfood batches exceed authored workshop capacity".into());
    }
    Ok(())
}
