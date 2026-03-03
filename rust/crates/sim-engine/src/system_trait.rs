/// SimSystem trait — the contract every simulation system must implement.
///
/// Systems are registered with SimEngine and called every `tick_interval` ticks.
/// They receive mutable access to the ECS world and shared simulation resources.
use hecs::World;
use crate::engine::SimResources;

/// Core simulation system trait.
///
/// # Implementation Notes
/// - `tick_interval()` is called once at registration. Changing it at runtime
///   has no effect — restart the engine to change intervals.
/// - Systems must not store references to entities across ticks.
/// - Heavy computation (A*, pathfinding) should use Rayon inside `run()`.
pub trait SimSystem: Send + Sync {
    /// Human-readable name for logging and diagnostics.
    fn name(&self) -> &'static str;

    /// How often this system runs: 1 = every tick, 12 = every 12 ticks (daily), etc.
    /// Return 0 to permanently disable the system — it will never be called.
    fn tick_interval(&self) -> u64;

    /// Run one system tick.
    ///
    /// # Arguments
    /// - `world`: the ECS entity-component store (mutable)
    /// - `resources`: shared non-component data (calendar, map, settlements, RNG, event bus)
    /// - `tick`: current absolute tick number (for determinism and logging)
    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64);

    /// Called once when the system is registered. Use for one-time setup.
    /// Default: no-op.
    fn on_register(&mut self, _world: &mut World, _resources: &mut SimResources) {}

    /// Returns a priority hint: lower = runs first within the same tick.
    /// Default: 100. Systems with the same priority run in registration order.
    fn priority(&self) -> u32 {
        100
    }
}

/// A system entry in the engine's registry.
pub(crate) struct SystemEntry {
    pub system: Box<dyn SimSystem>,
    pub last_run_tick: u64,
}

impl SystemEntry {
    pub fn new(system: Box<dyn SimSystem>) -> Self {
        Self { system, last_run_tick: 0 }
    }

    /// Returns true if this system should run on `current_tick`.
    pub fn should_run(&self, current_tick: u64) -> bool {
        let interval = self.system.tick_interval();
        if interval == 0 {
            return false; // disabled
        }
        current_tick.is_multiple_of(interval)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct NullSystem;
    impl SimSystem for NullSystem {
        fn name(&self) -> &'static str { "null" }
        fn tick_interval(&self) -> u64 { 5 }
        fn run(&mut self, _w: &mut World, _r: &mut SimResources, _t: u64) {}
    }

    #[test]
    fn should_run_correct_interval() {
        let entry = SystemEntry::new(Box::new(NullSystem));
        assert!(entry.should_run(0));
        assert!(entry.should_run(5));
        assert!(entry.should_run(10));
        assert!(!entry.should_run(3));
        assert!(!entry.should_run(7));
    }
}
