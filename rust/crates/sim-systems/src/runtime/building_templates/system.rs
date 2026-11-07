//! Runtime scheduling for deterministic aggregate template progress.
//! Priority: 153; interval: 1 tick, colony only, after room recognition.
//! Reads: physical construction sites, completed piece identities, rooms, command log.
//! Writes: building sites, room template associations, stockpile zones, chronicle.
use hecs::World;
use sim_engine::{RuntimeSystem, SimResources};

/// Runs after room detection; absence of template sites is an exact no-op.
#[derive(Debug, Default)]
pub struct BuildingTemplateSystem;
impl RuntimeSystem for BuildingTemplateSystem {
    fn name(&self) -> &str {
        "BuildingTemplateSystem"
    }
    fn priority(&self) -> u32 {
        153
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        super::advance(world, resources);
    }
}
