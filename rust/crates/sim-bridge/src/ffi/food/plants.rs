//! Viewport-bounded plant rows with authoritative seasonal phase and harvest depletion.
use sim_core::colony::content::{installed_or_bundled, PlantPhase};
use sim_engine::SimResources;

/// One real plant; phase codes are dormant0, growing1, ripe2, spent3.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlantView {
    /// Authoritative tile.
    pub tile: (u32, u32),
    /// Stable registry code.
    pub kind: u32,
    /// Lifecycle presentation code.
    pub phase: u8,
    /// Actual remaining harvest pieces.
    pub units: u16,
}
/// Collect only columns and rows intersecting the requested inclusive tile rectangle.
pub fn collect_plants(res: &SimResources, rect: (i32, i32, i32, i32)) -> Vec<PlantView> {
    if res.rules.colony.is_none() || rect.2 < 0 || rect.3 < 0 {
        return Vec::new();
    }
    let low = (
        u32::try_from(rect.0.max(0)).unwrap_or(0),
        u32::try_from(rect.1.max(0)).unwrap_or(0),
    );
    let high = (
        u32::try_from(rect.2).unwrap_or(0),
        u32::try_from(rect.3).unwrap_or(0),
    );
    if low.0 > high.0 || low.1 > high.1 {
        return Vec::new();
    }
    let catalog = installed_or_bundled();
    let day = sim_systems::runtime::food::day(res);
    res.food
        .plants
        .range((low.0, 0)..=(high.0, u32::MAX))
        .filter(|(tile, _)| (low.1..=high.1).contains(&tile.1))
        .filter_map(|(&tile, plant)| {
            let def = catalog.plant(plant.species)?;
            let phase =
                match sim_systems::runtime::food::plants::phase(def, &res.rules.calendar, day) {
                    PlantPhase::Dormant => 0,
                    PlantPhase::Growing => 1,
                    PlantPhase::Ripe if plant.units_left > 0 => 2,
                    PlantPhase::Ripe | PlantPhase::Spent => 3,
                };
            Some(PlantView {
                tile,
                kind: plant.species.code(),
                phase,
                units: plant.units_left,
            })
        })
        .collect()
}
