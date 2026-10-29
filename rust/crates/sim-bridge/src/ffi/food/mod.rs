//! Read-only food and plant projections; commands enter the normal replay log.
mod chronicle;
pub(super) use chronicle::{food_chronicle_rows_to_dict, food_why_to_dict};
mod marshal;
mod person;
mod plants;
mod stacks;
mod summary;
pub(super) use marshal::{person_food_to_dict, plants_to_dict, stacks_to_dict, summary_to_dict};
pub use person::{collect_person_food, PersonFoodView};
pub use plants::{collect_plants, PlantView};
use sim_core::colony::{Command, CommandResult};
use sim_engine::SimEngine;
pub use stacks::{collect_tile_food, FoodStackView};
pub use summary::{collect_food_summary, FoodSummaryView};

/// Submit an area harvest order through the simulation command recorder.
pub fn submit_harvest(engine: &mut SimEngine, rect: (i32, i32, i32, i32)) -> CommandResult {
    let coord = |v: i32| u32::try_from(v.max(0)).unwrap_or(0);
    sim_systems::commands::submit(
        engine,
        Command::DesignateHarvest {
            x0: coord(rect.0),
            y0: coord(rect.1),
            x1: coord(rect.2),
            y1: coord(rect.3),
        },
    )
}
