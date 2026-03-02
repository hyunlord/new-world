use hecs::World;
use sim_engine::{SimResources, SimSystem};

/// First production Rust runtime system migrated from GDScript.
///
/// This system mirrors the scheduler slot of `stats_recorder.gd` and
/// executes inside the Rust engine tick loop.
#[derive(Debug, Clone)]
pub struct StatsRecorderSystem {
    priority: u32,
    tick_interval: u64,
}

impl StatsRecorderSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for StatsRecorderSystem {
    fn name(&self) -> &'static str {
        "stats_recorder"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, _tick: u64) {
        // Initial Rust-port baseline: keep this system side-effect free while
        // moving scheduler ownership into Rust. Follow-up phases will port the
        // full history/snapshot behavior from GDScript.
        let _population_count: u32 = world.len();
        let _settlement_count: usize = resources.settlements.len();
        let _queued_events: usize = resources.event_bus.pending_count();
    }
}
