/// EventBus — collect-then-drain event dispatcher.
///
/// Events are pushed during a tick, then drained at tick end and
/// delivered to registered subscribers. This avoids re-entrant borrows.
///
/// For Phase R-0 the bus is single-threaded. Phase R-2 may add a
/// channel-based async variant for cross-thread delivery.
use crate::events::GameEvent;
use log::debug;

/// A subscriber callback: receives a reference to each event.
pub type Subscriber = Box<dyn FnMut(&GameEvent) + Send + 'static>;

/// In-process event bus.
pub struct EventBus {
    /// Pending events collected during the current tick.
    pending: Vec<GameEvent>,
    /// Registered subscribers, called in registration order.
    subscribers: Vec<Subscriber>,
    /// Running count of events dispatched (for diagnostics).
    total_dispatched: u64,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            pending: Vec::with_capacity(256),
            subscribers: Vec::new(),
            total_dispatched: 0,
        }
    }

    /// Register a subscriber. Called once at initialization.
    pub fn subscribe(&mut self, sub: Subscriber) {
        self.subscribers.push(sub);
    }

    /// Enqueue an event to be dispatched at end of tick.
    pub fn emit(&mut self, event: GameEvent) {
        self.pending.push(event);
    }

    /// Drain pending events and call all subscribers.
    /// Call once per tick after all systems have run.
    ///
    /// Iterates the drain iterator directly — no intermediate allocation.
    pub fn flush(&mut self) {
        let count = self.pending.len();
        // Drain in-place: avoids a second heap allocation every tick.
        // `drain(..)` yields owned values; we need a reference for subscribers,
        // so we collect into a local stack-allocated iteration variable.
        // Use a swap to avoid borrowing self.pending while iterating:
        let mut batch = std::mem::take(&mut self.pending);
        for event in &batch {
            debug!("[EventBus] dispatching: {}", event.name());
            for sub in self.subscribers.iter_mut() {
                sub(event);
            }
        }
        batch.clear();
        self.pending = batch; // return the allocation for reuse
        self.total_dispatched += count as u64;
    }

    /// Total events dispatched since creation (diagnostics).
    pub fn total_dispatched(&self) -> u64 {
        self.total_dispatched
    }

    /// Number of events currently queued (not yet flushed).
    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for EventBus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EventBus")
            .field("pending", &self.pending.len())
            .field("subscribers", &self.subscribers.len())
            .field("total_dispatched", &self.total_dispatched)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::ids::EntityId;
    use crate::events::GameEvent;
    use std::sync::{Arc, Mutex};

    #[test]
    fn flush_delivers_events_in_order() {
        let received: Arc<Mutex<Vec<u64>>> = Arc::new(Mutex::new(Vec::new()));
        let rx = received.clone();

        let mut bus = EventBus::new();
        bus.subscribe(Box::new(move |e| {
            if let GameEvent::TickCompleted { tick } = e {
                rx.lock().unwrap().push(*tick);
            }
        }));

        bus.emit(GameEvent::TickCompleted { tick: 1 });
        bus.emit(GameEvent::TickCompleted { tick: 2 });
        bus.flush();

        let got = received.lock().unwrap().clone();
        assert_eq!(got, vec![1, 2]);
        assert_eq!(bus.pending_count(), 0);
        assert_eq!(bus.total_dispatched(), 2);
    }

    #[test]
    fn flush_empty_is_noop() {
        let mut bus = EventBus::new();
        bus.flush(); // must not panic
        assert_eq!(bus.total_dispatched(), 0);
    }
}
