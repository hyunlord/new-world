use std::collections::VecDeque;

/// A simulation event recorded for narrative analysis.
#[derive(Clone, Debug)]
pub struct SimEvent {
    /// Absolute simulation tick when the event was recorded.
    pub tick: u64,
    /// Typed event category used by the story sifter.
    pub event_type: SimEventType,
    /// Raw primary actor entity ID.
    pub actor: u32,
    /// Optional raw target entity ID.
    pub target: Option<u32>,
    /// Searchable coarse-grained tags.
    pub tags: Vec<String>,
    /// Short machine-oriented cause descriptor.
    pub cause: String,
    /// Magnitude or score associated with the event.
    pub value: f64,
}

/// Typed simulation event categories persisted in the narrative event store.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SimEventType {
    /// A need dropped below the critical threshold.
    NeedCritical,
    /// A need recovered above the satisfied threshold.
    NeedSatisfied,
    /// Dominant emotion changed.
    EmotionShift,
    /// Mood bucket changed.
    MoodChanged,
    /// Stress gas stage increased.
    StressEscalated,
    /// A mental break started.
    MentalBreakStart,
    /// A mental break ended.
    MentalBreakEnd,
    /// A new relationship edge meaningfully formed.
    RelationshipFormed,
    /// A previously meaningful relationship broke down.
    RelationshipBroken,
    /// A social conflict occurred.
    SocialConflict,
    /// A social cooperation event occurred.
    SocialCooperation,
    /// The agent changed to a different action.
    ActionChanged,
    /// A non-idle action completed.
    TaskCompleted,
    /// A birth occurred.
    Birth,
    /// A death occurred.
    Death,
    /// An age stage transition occurred.
    AgeTransition,
    /// A first-ever milestone occurred.
    FirstOccurrence,
    /// Fallback custom event type.
    Custom(String),
}

/// A fixed-capacity ring buffer of recent simulation events.
#[derive(Clone, Debug)]
pub struct EventStore {
    events: VecDeque<SimEvent>,
    capacity: usize,
}

impl EventStore {
    /// Creates an empty event store with the given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity: capacity.max(1),
        }
    }

    /// Pushes an event, evicting the oldest event when the store is full.
    pub fn push(&mut self, event: SimEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Returns the newest `n` events in reverse chronological order.
    pub fn recent(&self, n: usize) -> impl Iterator<Item = &SimEvent> {
        self.events.iter().rev().take(n)
    }

    /// Returns all events whose tick is greater than or equal to `tick`.
    pub fn since_tick(&self, tick: u64) -> impl Iterator<Item = &SimEvent> {
        self.events.iter().filter(move |event| event.tick >= tick)
    }

    /// Returns all events for one actor since `since_tick`.
    pub fn by_actor(&self, actor: u32, since_tick: u64) -> Vec<&SimEvent> {
        self.events
            .iter()
            .filter(|event| event.actor == actor && event.tick >= since_tick)
            .collect()
    }

    /// Returns all events of a given type since `since_tick`.
    pub fn by_type(&self, event_type: &SimEventType, since_tick: u64) -> Vec<&SimEvent> {
        self.events
            .iter()
            .filter(|event| &event.event_type == event_type && event.tick >= since_tick)
            .collect()
    }

    /// Returns an iterator across the whole store in chronological order.
    pub fn iter(&self) -> impl Iterator<Item = &SimEvent> {
        self.events.iter()
    }

    /// Returns the number of stored events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Returns true when the store contains no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{EventStore, SimEvent, SimEventType};

    fn make_event(tick: u64, actor: u32, event_type: SimEventType) -> SimEvent {
        SimEvent {
            tick,
            event_type,
            actor,
            target: None,
            tags: vec!["test".to_string()],
            cause: "test".to_string(),
            value: tick as f64,
        }
    }

    #[test]
    fn event_store_capacity_evicts_oldest() {
        let mut store = EventStore::new(100);
        for tick in 0..200 {
            store.push(make_event(tick, 1, SimEventType::ActionChanged));
        }
        assert_eq!(store.len(), 100);
        let oldest_tick = store
            .iter()
            .next()
            .expect("store should retain recent events")
            .tick;
        assert_eq!(oldest_tick, 100);
    }

    #[test]
    fn by_type_and_actor_filters_respect_tick_cutoff() {
        let mut store = EventStore::new(8);
        store.push(make_event(1, 10, SimEventType::NeedCritical));
        store.push(make_event(2, 10, SimEventType::NeedSatisfied));
        store.push(make_event(3, 20, SimEventType::NeedCritical));

        let by_actor = store.by_actor(10, 2);
        assert_eq!(by_actor.len(), 1);
        assert_eq!(by_actor[0].tick, 2);

        let by_type = store.by_type(&SimEventType::NeedCritical, 0);
        assert_eq!(by_type.len(), 2);
    }
}
