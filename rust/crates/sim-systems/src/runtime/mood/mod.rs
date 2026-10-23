//! M5 mood (ADR 0030): the mood value's effect on work — filled in by Part 4.
use hecs::{Entity, World};
use sim_engine::SimResources;

/// The work-speed factor of `entity`'s mood, ‰ (1,000 until mood exists).
pub fn work_permille(_world: &World, _res: &SimResources, _entity: Entity) -> u32 {
    1000
}
