use std::collections::{BTreeMap, VecDeque};

use serde::{Deserialize, Serialize};

use crate::config;
use crate::ids::{BuildingId, EntityId, SettlementId};

/// Typed reference to the immediate cause of a simulation change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CauseRef {
    /// System or subsystem key that produced the change.
    pub system: String,
    /// Stable cause kind or event identifier.
    pub kind: String,
    /// Optional source entity.
    pub entity: Option<EntityId>,
    /// Optional source building.
    pub building: Option<BuildingId>,
    /// Optional source settlement.
    pub settlement: Option<SettlementId>,
}

/// One causal event written into the log.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CausalEvent {
    /// Tick at which the causal event occurred.
    pub tick: u64,
    /// Causal reference describing the source.
    pub cause: CauseRef,
    /// Target field or subsystem affected.
    pub effect_key: String,
    /// Human-readable but localized-through-key summary token.
    pub summary_key: String,
    /// Signed scalar magnitude if the effect has one.
    pub magnitude: f64,
}

/// Per-entity causal ring buffer used by future runtime systems.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalLog {
    entries: BTreeMap<EntityId, VecDeque<CausalEvent>>,
    max_per_entity: usize,
}

impl CausalLog {
    /// Creates a new causal log using the configured ring-buffer capacity.
    pub fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            max_per_entity: config::CAUSAL_LOG_MAX_PER_ENTITY,
        }
    }

    /// Appends one causal event to an entity-local ring buffer.
    pub fn push(&mut self, entity: EntityId, event: CausalEvent) {
        let deque = self
            .entries
            .entry(entity)
            .or_insert_with(|| VecDeque::with_capacity(self.max_per_entity));
        if deque.len() >= self.max_per_entity {
            deque.pop_front();
        }
        deque.push_back(event);
    }

    /// Returns recent causal events for one entity, newest first.
    pub fn recent(&self, entity: EntityId, count: usize) -> Vec<&CausalEvent> {
        self.entries
            .get(&entity)
            .map(|deque| deque.iter().rev().take(count).collect())
            .unwrap_or_default()
    }

    /// Clears all causal entries for an entity.
    pub fn clear_entity(&mut self, entity: EntityId) {
        self.entries.remove(&entity);
    }

    /// Returns the total number of causal events across all entities.
    pub fn total_entries(&self) -> usize {
        self.entries.values().map(VecDeque::len).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_event(tick: u64) -> CausalEvent {
        CausalEvent {
            tick,
            cause: CauseRef {
                system: "needs_system".to_string(),
                kind: "hunger_decay".to_string(),
                entity: Some(EntityId(7)),
                building: None,
                settlement: None,
            },
            effect_key: "need_hunger".to_string(),
            summary_key: "CAUSE_HUNGER_DECAY".to_string(),
            magnitude: -0.02,
        }
    }

    #[test]
    fn causal_log_uses_ring_buffer_capacity() {
        let mut log = CausalLog::new();
        let entity = EntityId(7);
        for tick in 0..40 {
            log.push(entity, sample_event(tick));
        }

        let recent = log.recent(entity, 64);
        assert_eq!(recent.len(), config::CAUSAL_LOG_MAX_PER_ENTITY);
        assert_eq!(recent[0].tick, 39);
    }

    #[test]
    fn causal_log_clear_entity_removes_entries() {
        let mut log = CausalLog::new();
        let entity = EntityId(3);
        log.push(entity, sample_event(1));
        log.clear_entity(entity);
        assert!(log.recent(entity, 4).is_empty());
    }
}
