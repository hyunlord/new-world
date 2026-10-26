//! Cascade arms and their memory bias (V7 Phase 8-β, Fix D): the arm enum,
//! the arm priority order, the arm-to-memory tag, the recency-weighted bias
//! delta, the top contributing memory entry and the bias-flip threshold.

use sim_core::causal::EventId;
use sim_core::components::{Memory, MemoryArm, SALIENCE_FLOOR};

use crate::runtime::memory::MAX_RECENCY_TICKS;

/// Cascade arm identifier — the natural drives + non-natural arms whose
/// eligibility is computed independently. Memory is a bias SOURCE, not an arm.
///
/// `pub` so the priority order + per-arm memory tag are inspectable by the
/// harness (Direction-2 slice 2-2 added `Gather`, a positional arm that is
/// never memory-biased and never enters the bias-flip machinery — it is
/// selected directly in the Idle cascade, not via a `CascadeArm` value, so
/// exposing the enum is what keeps its variant a live part of the API).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CascadeArm {
    /// Eat-to-survive drive: `Hunger.value` above `HUNGER_THRESHOLD`. Priority 0.
    Hunger,
    /// Drink-to-survive drive: `Thirst.value` above `THIRST_THRESHOLD`. Priority 1.
    Thirst,
    /// Sleep drive: `Sleep.fatigue` above `FATIGUE_THRESHOLD`. Priority 2.
    Fatigue,
    /// Co-located active construction-site work drive. Priority 3.
    Construction,
    /// Social-interaction drive: `Social.loneliness` above `SOCIAL_THRESHOLD`. Priority 4.
    Social,
    /// Memory-driven negative combat trigger. A non-natural cascade arm
    /// activated by `memory_weight_delta` strictly below
    /// `-BIAS_FLIP_THRESHOLD`. Phase 9-β / P9β-5.
    Combat,
    /// Settlement migration pull. A non-natural cascade arm activated when
    /// the agent is not a member of any settlement AND at least one
    /// settlement with available capacity (`current < SETTLEMENT_MAX_POP`)
    /// exists in the world. The lowest-priority arm — only fires after
    /// every higher-priority arm (Hunger/Thirst/Fatigue/Construction/
    /// Social/Combat) has been ruled out. V7 Phase 10-β / P10β-8.
    Settlement,
    /// Opportunistic ground-food gathering. Direction-2 slice 2-2. A satisfied
    /// agent with inventory room walks to the nearest ground food tile within
    /// [`GATHER_SCAN_RADIUS`] and picks it up into its `Inventory` (carry, NOT
    /// eat). Priority index 6 — below every survival/social/combat arm and
    /// above `Settlement` (which moves to 7). Positional/opportunistic, so it
    /// maps to no [`MemoryArm`] (never memory-biased).
    Gather,
}

/// Linear recency factor: 1.0 at `elapsed == 0`, 0.0 at
/// `elapsed >= MAX_RECENCY_TICKS`.
fn recency_factor(encoded_tick: u64, current_tick: u64) -> f64 {
    let elapsed = current_tick.saturating_sub(encoded_tick);
    if elapsed >= MAX_RECENCY_TICKS {
        0.0
    } else {
        1.0 - (elapsed as f64 / MAX_RECENCY_TICKS as f64)
    }
}

/// The [`MemoryArm`] a cascade arm draws its memory bias from, or `None` for
/// arms that are never memory-biased (`Settlement` is a positional pull arm,
/// so no memory entry is ever tagged with it).
///
/// Mirrors exactly the `(arm, event)` pairs the former `event_id_matches_arm`
/// recognised, so a stored-arm comparison reproduces that lookup's result for
/// any event still resident in the ring — while being a pure function of the
/// entry (no `causal_log` lookup, no ring-eviction dependence). Fix D.
pub fn cascade_arm_memory_tag(arm: CascadeArm) -> Option<MemoryArm> {
    match arm {
        CascadeArm::Hunger => Some(MemoryArm::Hunger),
        CascadeArm::Thirst => Some(MemoryArm::Thirst),
        CascadeArm::Fatigue => Some(MemoryArm::Fatigue),
        CascadeArm::Construction => Some(MemoryArm::Construction),
        CascadeArm::Social => Some(MemoryArm::Social),
        CascadeArm::Combat => Some(MemoryArm::Combat),
        CascadeArm::Settlement => None,
        CascadeArm::Gather => None,
    }
}

/// Compute the cascade-bias weight delta for `arm` from a snapshot of
/// `memory`. Linear product form per plan §2 P8β-MOD-2 step 4:
/// `sum(valence * salience * recency_factor)` over entries that
/// (a) carry the arm's stored [`MemoryArm`] tag, AND (b) have salience
/// strictly above `SALIENCE_FLOOR` (eligibility gate from plan A16).
///
/// Fix D: the arm match reads each entry's encode-time `arm` tag — NOT a
/// runtime `causal_log` lookup — so the delta is a deterministic pure
/// function of `(arm, valence, salience, encoded_tick)`, independent of
/// `event_id` allocation order and ring-buffer eviction. A `Settlement`
/// arm (no memory tag) yields `0.0`.
pub(in crate::runtime) fn memory_weight_delta(
    memory: &Memory,
    arm: CascadeArm,
    current_tick: u64,
) -> f64 {
    let Some(tag) = cascade_arm_memory_tag(arm) else {
        return 0.0;
    };
    memory
        .entries
        .iter()
        .filter(|entry| entry.salience > SALIENCE_FLOOR)
        .filter(|entry| entry.arm == tag)
        .map(|entry| {
            let recency = recency_factor(entry.encoded_tick, current_tick);
            entry.valence * entry.salience * recency
        })
        .sum()
}

/// Find the highest-magnitude contributing entry for `arm` — used as the
/// `recalled_event` field on `MemoryRecalled` emissions. Matches on the
/// stored [`MemoryArm`] tag (Fix D); `None` for a non-memory arm.
pub(in crate::runtime) fn top_contributor_entry(
    memory: &Memory,
    arm: CascadeArm,
    current_tick: u64,
) -> Option<EventId> {
    let tag = cascade_arm_memory_tag(arm)?;
    memory
        .entries
        .iter()
        .filter(|entry| entry.salience > SALIENCE_FLOOR)
        .filter(|entry| entry.arm == tag)
        .max_by(|a, b| {
            let wa = (a.valence * a.salience * recency_factor(a.encoded_tick, current_tick)).abs();
            let wb = (b.valence * b.salience * recency_factor(b.encoded_tick, current_tick)).abs();
            wa.partial_cmp(&wb).unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|e| e.event_id)
}

/// Per-arm natural eligibility score (binary 0.0/1.0 baseline). Memory
/// weight deltas are added on top. The arm with the highest adjusted
/// score wins; ties resolved by the original (Hunger > Thirst > Fatigue >
/// Construction > Social) priority order.
pub fn arm_priority_index(arm: CascadeArm) -> u8 {
    match arm {
        CascadeArm::Hunger => 0,
        CascadeArm::Thirst => 1,
        CascadeArm::Fatigue => 2,
        CascadeArm::Construction => 3,
        CascadeArm::Social => 4,
        CascadeArm::Combat => 5,
        CascadeArm::Gather => 6,
        CascadeArm::Settlement => 7,
    }
}

/// V7 Phase 8-β cascade-bias flip threshold (P8β-MOD-2 step 5 refinement).
///
/// A non-natural-winner arm only flips the cascade when its
/// `memory_weight_delta` STRICTLY exceeds this threshold — i.e. the
/// memory-adjusted score must surpass the natural arm's binary
/// eligibility score (1.0). Below the threshold, the natural winner is
/// preserved and no `MemoryRecalled` event is emitted (plan §A20: bias
/// observed but non-load-bearing).
///
/// Locked at 1.0 (equal to the natural arm's binary eligibility).
/// Only a delta strictly GREATER than 1.0 can override the natural
/// winner, preventing weak positive bias from causing spurious flips.
pub const BIAS_FLIP_THRESHOLD: f64 = 1.0;
