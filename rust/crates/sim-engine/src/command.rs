use crate::engine::SimResources;
use crate::events::GameEvent;
/// Command — deferred world mutation queue.
///
/// Systems must not mutate the ECS world or emit events in ways that would
/// cause re-entrant borrows. Instead, they push `Command` values to the
/// `CommandQueue` during their `run()` call. After all systems have run
/// (but before the event bus flushes), the engine can drain the queue and
/// apply each command.
///
/// # Phase R-0 scope
/// Only the most common commands are implemented here. Richer variants
/// (InsertComponent, RemoveComponent) require type-erased bundles and are
/// deferred to Phase R-1 when concrete systems need them.
use hecs::{Entity, World};
use log::debug;

// ── Command ───────────────────────────────────────────────────────────────────

/// A deferred mutation to the ECS world or simulation resources.
#[derive(Debug)]
pub enum Command {
    /// Remove an entity and all its components from the world.
    ///
    /// Safe to emit even if the entity is already dead — `despawn` is
    /// idempotent when the entity does not exist.
    DespawnEntity { entity: Entity },

    /// Enqueue an event into the event bus.
    ///
    /// Events pushed here are delivered on the *same* tick's flush pass,
    /// indistinguishable from events emitted directly via `resources.event_bus.emit()`.
    EmitEvent { event: GameEvent },
}

// ── CommandQueue ──────────────────────────────────────────────────────────────

/// A lightweight FIFO queue for deferred world mutations.
///
/// Systems receive `&mut CommandQueue` through which they push commands.
/// The engine calls `flush()` once after all systems have run.
///
/// # Example
/// ```ignore
/// fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
///     // Do NOT call world.despawn() here — use the command queue instead.
///     for (entity, health) in world.query::<&Health>().iter() {
///         if health.current <= 0.0 {
///             self.commands.push(Command::DespawnEntity { entity });
///         }
///     }
/// }
/// ```
#[derive(Debug, Default)]
pub struct CommandQueue {
    pending: Vec<Command>,
}

impl CommandQueue {
    pub fn new() -> Self {
        Self {
            pending: Vec::with_capacity(64),
        }
    }

    /// Push a command to be applied at the end of the tick.
    pub fn push(&mut self, cmd: Command) {
        self.pending.push(cmd);
    }

    /// Convenience: push a despawn.
    pub fn despawn(&mut self, entity: Entity) {
        self.push(Command::DespawnEntity { entity });
    }

    /// Convenience: push an event emission.
    pub fn emit(&mut self, event: GameEvent) {
        self.push(Command::EmitEvent { event });
    }

    /// Number of pending commands.
    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    /// Drain and apply all pending commands against the world + resources.
    ///
    /// Called by the engine after all systems have run for a given tick.
    pub fn flush(&mut self, world: &mut World, resources: &mut SimResources) {
        let count = self.pending.len();
        // Drain in-place — Command is owned so no reference gymnastics needed.
        let mut batch = std::mem::take(&mut self.pending);
        for cmd in batch.drain(..) {
            match cmd {
                Command::DespawnEntity { entity } => {
                    debug!("[CommandQueue] despawn {:?}", entity);
                    let _ = world.despawn(entity); // ignore if already gone
                }
                Command::EmitEvent { event } => {
                    debug!("[CommandQueue] emit {}", event.name());
                    resources.event_bus.emit(event);
                }
            }
        }
        self.pending = batch; // return allocation for reuse next tick
        if count > 0 {
            debug!("[CommandQueue] flushed {} commands", count);
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::SimResources;
    use crate::events::GameEvent;
    use sim_core::config::GameConfig;
    use sim_core::ids::EntityId;
    use sim_core::{GameCalendar, WorldMap};

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let cal = GameCalendar::new(&config);
        let map = WorldMap::new(4, 4, 0);
        SimResources::new(cal, map, 0)
    }

    #[test]
    fn despawn_command_removes_entity() {
        let mut world = World::new();
        let mut resources = make_resources();
        let entity = world.spawn(());
        assert!(world.contains(entity));

        let mut queue = CommandQueue::new();
        queue.despawn(entity);
        queue.flush(&mut world, &mut resources);

        assert!(!world.contains(entity));
        assert!(queue.is_empty());
    }

    #[test]
    fn despawn_nonexistent_is_safe() {
        let mut world = World::new();
        let mut resources = make_resources();
        let entity = world.spawn(());
        world.despawn(entity).unwrap(); // already gone

        let mut queue = CommandQueue::new();
        queue.despawn(entity); // should not panic
        queue.flush(&mut world, &mut resources); // no panic
    }

    #[test]
    fn emit_command_queues_event() {
        let mut world = World::new();
        let mut resources = make_resources();

        let mut queue = CommandQueue::new();
        queue.emit(GameEvent::EntitySpawned {
            entity_id: EntityId(42),
        });
        queue.flush(&mut world, &mut resources);

        // Event is in the bus, pending flush
        assert_eq!(resources.event_bus.pending_count(), 1);
    }

    #[test]
    fn flush_empty_queue_is_noop() {
        let mut world = World::new();
        let mut resources = make_resources();
        let mut queue = CommandQueue::new();
        queue.flush(&mut world, &mut resources); // must not panic
        assert_eq!(resources.event_bus.pending_count(), 0);
    }
}
