//! One-time conversion of old colony food ownership into the format13 sidecar.
mod physical;
mod porters;
mod reserves;

use hecs::World;
use sim_engine::SimResources;

/// Convert a decoded pre-food colony; failed conversion rejects the entire load.
/// Baseline resources remain byte-compatible and receive no food state.
pub fn migrate_legacy(world: &mut World, res: &mut SimResources) -> Result<(), String> {
    if res.rules.colony.is_none() {
        return Ok(());
    }
    if res.food != Default::default() {
        return Err("legacy food conversion requires an empty sidecar".into());
    }
    let day = super::day(res);
    physical::attach(world, res)?;
    reserves::convert(world, res, day)?;
    porters::retire(world, res)?;
    crate::worldgen::plants::populate(res)?;
    res.food_tiles.clear();
    res.food_source_max.clear();
    res.food.last_daily_day = Some(day);
    res.food.touch();
    super::validate(world, res)
}

/// Convert food already withdrawn from a legacy settlement into a pending store.
/// This must not withdraw or count the parent's remaining stockpile again.
pub(crate) fn place_pending(
    res: &mut SimResources,
    tile: (u32, u32),
    count: u32,
) -> Result<(), String> {
    let kind = sim_core::colony::content::installed_or_bundled()
        .item_role("legacy_food")
        .ok_or("missing legacy_food migration role")?;
    if kind.definition().food.as_ref().map(|food| food.units) != Some(1) {
        return Err("legacy_food must preserve exactly one unit per piece".into());
    }
    reserves::place(
        res,
        sim_core::colony::ItemSpec::raw(kind, None),
        (tile, count, super::day(res)),
    )
}
