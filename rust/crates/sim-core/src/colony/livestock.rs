//! Ordered livestock ownership, biological age and immutable event facts.
//! Mechanics live in sim-systems; this sidecar remains empty in baseline worlds.
use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::building_site::{BuildingSiteId, Tile};
use super::content::AnimalKind;
use super::knowledge::KnowledgeId;
use super::wildlife::AnimalId;
use super::{ItemKind, ItemSpec};
use crate::components::Sex;

/// Monotonic livestock identity, independent of people and wild-animal IDs.
pub type LivestockId = u32;
/// Index of the corresponding immutable Chronicle entry.
pub type LivestockWitnessId = u32;

/// One owned animal; geometry is derived from its physical pen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Livestock {
    /// Identity, equal to the owning map key.
    pub id: LivestockId,
    /// Selected content species.
    pub species: AnimalKind,
    /// Biological whole-day age at `last_age_tick`.
    pub age_days: u32,
    /// Biological day phase, retained across capture and save/load.
    pub last_age_tick: u64,
    /// Immutable sex from the keyed demographics draw.
    pub sex: Sex,
    /// Consecutive daily feeding failures.
    pub hunger_days: u32,
    /// Persistent physical pen provenance.
    pub pen_site: BuildingSiteId,
    /// A ready product remains available until physically committed.
    pub milk_ready: bool,
    /// Display-day index of the last committed milk output.
    pub last_milk_day: u32,
    /// Display-day index of creation or the last committed birth.
    pub last_bred_day: u32,
    /// Creation day; feeding begins at a later real daily boundary.
    pub created_day: u32,
    /// Captured/Born Chronicle entry and witness identity.
    pub creation_entry: LivestockWitnessId,
}

/// One physical cell's consumed-forage cooldown, shared across all pens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GrassState {
    /// First display day on which this unit is available again.
    pub next_regrow_day: u32,
}

/// Completed keyed attempts and their retry boundary for one wild animal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaptureAttempt {
    /// Actual completed rolls; cancelled travel does not increment this.
    pub completed_attempts: u32,
    /// First permitted display day after a failed roll.
    pub not_before_day: u32,
}

/// Accepted command provenance, including an optional real steward wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivestockOrderCause {
    /// Index in the accepted outer command log.
    pub source_record: u32,
    /// Exact steward rule when the source command is wrapped.
    pub steward_rule: Option<u16>,
}

/// The actual committing practitioner's admission facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeWitness {
    /// Selected knowledge identity.
    pub knowledge: KnowledgeId,
    /// Actual proficiency at commit.
    pub proficiency: u16,
    /// Policy threshold checked at commit.
    pub required: u16,
}

/// One exact physically produced item batch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivestockYield {
    /// Material, quality and kind of the real output.
    pub item: ItemSpec,
    /// Number of physical units.
    pub count: u16,
}

/// Mechanical natural-death causes; species remain data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivestockDeathReason {
    /// The authored consecutive hunger limit was reached.
    Starvation,
    /// The authored maximum biological age was reached.
    OldAge,
}

/// Immutable facts of a committed livestock event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LivestockEvent {
    /// A wild animal became owned after a real capture job.
    Captured {
        /// Original wild identity, retained after its removal.
        wild: AnimalId,
        /// One-based committed keyed roll ordinal.
        attempt: u32,
        /// Committing worker's knowledge facts.
        knowledge: KnowledgeWitness,
        /// Physical tool consumed, absent only for a no-tool policy.
        tool: Option<ItemSpec>,
        /// Accepted command and optional steward wrapper.
        origin: LivestockOrderCause,
    },
    /// A real offspring, with both historical parents retained.
    Born {
        /// Mother's livestock identity.
        mother: LivestockId,
        /// Father's livestock identity.
        father: LivestockId,
    },
    /// A ready recurring product was placed physically; the historic wire tag is retained.
    Milk {
        /// Committing worker's knowledge facts.
        knowledge: KnowledgeWitness,
        /// Exact physical output.
        product: LivestockYield,
    },
    /// An ordered slaughter committed all physical products.
    Slaughter {
        /// Accepted command and optional steward wrapper.
        origin: LivestockOrderCause,
        /// Exact output batches in authored product order.
        products: Vec<LivestockYield>,
    },
    /// Natural death, without a fabricated worker or command.
    Died {
        /// Actual mechanical cause.
        reason: LivestockDeathReason,
    },
}

/// Typed facts paired with exactly one Chronicle entry of the same index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivestockEventWitness {
    /// Subject livestock identity, never an AgentId.
    pub livestock: LivestockId,
    /// Subject species at the committed event.
    pub species: AnimalKind,
    /// Physical pen provenance at the event.
    pub pen_site: BuildingSiteId,
    /// Mechanically committed event facts.
    pub event: LivestockEvent,
}

/// Ordered livestock sidecar, with a rebuilt nonserialized product index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LivestockState {
    /// Living owned animals in stable identity order.
    pub animals: BTreeMap<LivestockId, Livestock>,
    /// Next identity; zero is invalid and MAX is exhausted.
    pub next_id: LivestockId,
    /// Last completed daily transition, absent before the first.
    pub last_advanced_day: Option<u32>,
    /// Consumed physical forage-cell cooldowns.
    pub grass: BTreeMap<Tile, GrassState>,
    /// Failed completed capture attempts and cooldowns.
    pub capture_attempts: BTreeMap<AnimalId, CaptureAttempt>,
    /// Accepted pending capture intents.
    pub capture_orders: BTreeMap<AnimalId, LivestockOrderCause>,
    /// Accepted pending slaughter intents.
    pub slaughter_orders: BTreeMap<LivestockId, LivestockOrderCause>,
    /// First captured identity, pinned even after removal.
    pub first_capture: Option<LivestockId>,
    /// First born identity, pinned even after removal.
    pub first_kid_id: Option<LivestockId>,
    /// Immutable facts, keyed by their Chronicle entry index.
    pub witnesses: BTreeMap<LivestockWitnessId, LivestockEventWitness>,
    /// Derived last wool-yield days, rebuilt from immutable witnesses after loading.
    #[serde(skip)]
    pub wool_last_day: BTreeMap<LivestockId, u32>,
    /// Physical capture tools consumed as losses on worker death.
    pub lost_capture_tools: BTreeMap<ItemKind, u64>,
    /// Mutation version used by read-side caches.
    pub version: u64,
}

impl Default for LivestockState {
    fn default() -> Self {
        Self {
            animals: BTreeMap::new(),
            next_id: 1,
            last_advanced_day: None,
            grass: BTreeMap::new(),
            capture_attempts: BTreeMap::new(),
            capture_orders: BTreeMap::new(),
            slaughter_orders: BTreeMap::new(),
            first_capture: None,
            first_kid_id: None,
            witnesses: BTreeMap::new(),
            wool_last_day: BTreeMap::new(),
            lost_capture_tools: BTreeMap::new(),
            version: 0,
        }
    }
}

impl LivestockState {
    /// Pure allocation preflight; the caller commits the returned next counter.
    pub fn checked_next_id(&self) -> Option<(LivestockId, LivestockId)> {
        let id = self.next_id;
        if id == 0
            || id == u32::MAX
            || self.animals.keys().any(|issued| *issued >= id)
            || self
                .witnesses
                .values()
                .any(|witness| witness.livestock >= id)
        {
            return None;
        }
        Some((id, id.checked_add(1)?))
    }

    /// Whether a completed capture roll can begin at this display day.
    pub fn capture_ready(&self, wild: AnimalId, day: u32) -> bool {
        self.next_capture_attempt(wild, day).is_some()
    }

    /// Pure one-based attempt preflight; exhausted ordinals never wrap.
    pub fn next_capture_attempt(&self, wild: AnimalId, day: u32) -> Option<u32> {
        match self.capture_attempts.get(&wild) {
            Some(attempt) if day >= attempt.not_before_day => {
                attempt.completed_attempts.checked_add(1)
            }
            Some(_) => None,
            None => Some(1),
        }
    }

    /// Mark a real state mutation without wrapping the cache version.
    pub fn touch(&mut self) {
        self.version = self.version.saturating_add(1);
    }
}

/// Project biological age without mutating the whole-day phase anchor.
pub fn current_age(animal: &Livestock, now_tick: u64, ticks_per_day: u64) -> u32 {
    let days = now_tick.saturating_sub(animal.last_age_tick) / ticks_per_day.max(1);
    animal
        .age_days
        .saturating_add(days.min(u64::from(u32::MAX)) as u32)
}

/// Commit only elapsed whole days, retaining the partial biological day.
pub fn advance_age(animal: &mut Livestock, now_tick: u64, ticks_per_day: u64) {
    let day_ticks = ticks_per_day.max(1);
    let days = now_tick.saturating_sub(animal.last_age_tick) / day_ticks;
    animal.age_days = current_age(animal, now_tick, day_ticks);
    animal.last_age_tick = animal.last_age_tick.saturating_add(days * day_ticks);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn allocation_and_attempt_ordinals_never_wrap_or_mutate_on_preflight() {
        let mut state = LivestockState::default();
        assert_eq!(state.checked_next_id(), Some((1, 2)));
        assert_eq!(state.next_id, 1);
        state.next_id = u32::MAX - 1;
        assert_eq!(state.checked_next_id(), Some((u32::MAX - 1, u32::MAX)));
        state.next_id = u32::MAX;
        assert_eq!(state.checked_next_id(), None);
        state.next_id = 0;
        assert_eq!(state.checked_next_id(), None);
        assert_eq!(state.next_capture_attempt(7, 0), Some(1));
        state.capture_attempts.insert(
            7,
            CaptureAttempt {
                completed_attempts: 3,
                not_before_day: 5,
            },
        );
        assert!(!state.capture_ready(7, 4));
        assert_eq!(state.next_capture_attempt(7, 5), Some(4));
        assert_eq!(state.capture_attempts[&7].completed_attempts, 3);
        state
            .capture_attempts
            .get_mut(&7)
            .unwrap()
            .completed_attempts = u32::MAX;
        assert!(!state.capture_ready(7, u32::MAX));
    }

    #[test]
    fn age_commit_retains_fractional_day_and_saturates() {
        let mut animal = Livestock {
            id: 1,
            species: AnimalKind::Deer,
            age_days: 29,
            last_age_tick: 17,
            sex: Sex { female: true },
            hunger_days: 0,
            pen_site: 1,
            milk_ready: false,
            last_milk_day: 0,
            last_bred_day: 0,
            created_day: 0,
            creation_entry: 0,
        };
        assert_eq!(current_age(&animal, 116, 100), 29);
        assert_eq!(current_age(&animal, 117, 100), 30);
        advance_age(&mut animal, 268, 100);
        assert_eq!((animal.age_days, animal.last_age_tick), (31, 217));
        advance_age(&mut animal, 316, 100);
        assert_eq!((animal.age_days, animal.last_age_tick), (31, 217));
        advance_age(&mut animal, 317, 100);
        assert_eq!((animal.age_days, animal.last_age_tick), (32, 317));
        assert_eq!(current_age(&animal, 0, 100), 32);
        assert_eq!(current_age(&animal, u64::MAX, 0), u32::MAX);
    }
}
