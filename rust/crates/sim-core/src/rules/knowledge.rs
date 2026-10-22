//! M4's knowledge rule (goal 3.2–3.4, ADR 0024–0026; `ColonyRule::knowledge`).
//! Every number here is a calibration candidate (goal §10, M4 DECISIONS).
use serde::{Deserialize, Serialize};

/// Tunables of knowledge: the channels, forgetting, discovery and the
/// aggregate. `None` in the M1–M3 colony rules; every knowledge system returns
/// at once without it, and the job layer then gates nothing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KnowledgeRule {
    /// Clock minute of the evening telling (goal 3.3: at the hearth gathering).
    pub oral_minute: u32,
    /// Who is gathered: within this Chebyshev distance of the hearth, or in the hall.
    pub gather_radius: u32,
    /// Proficiency ‰ a listener gains (goal 3.3: 350).
    pub oral_gain: u16,
    /// Chance ‰ a telling takes (goal 3.3: 500).
    pub oral_chance_permille: u32,
    /// Extra chance ‰ when the teller knows storytelling (goal 3.3: +10 points = 100).
    pub storytelling_bonus_permille: u32,
    /// Youngest listener or learner, in years (goal 3.3: 10).
    pub learner_min_years: u64,
    /// Days an entry may lie unused before it fades (goal 3.3: 6).
    pub unused_days: u64,
    /// Ticks between forgetting steps (goal 3.3: 60).
    pub decay_interval: u64,
    /// ‰ lost per step by source (`Source::code` order: seeded, discovered, oral, observed, apprenticed).
    pub decay: [u16; 5],
    /// Floor ‰ by source (goal 3.3: seeded and discovered 100, others 0).
    pub floor: [u16; 5],
    /// ‰ a completed gated job adds (goal 3.2: 2).
    pub use_gain: u16,
    /// Ticks between discovery checks (goal 3.3: 240).
    pub discovery_interval: u64,
    /// Lowest novelty seeking ‰ of a discoverer (goal 3.3: 600).
    pub discovery_ns: u16,
    /// One in this many checks succeeds, by difficulty 1–3 (goal 3.3: 40, 90, 200).
    pub discovery_odds: [u32; 3],
    /// Proficiency ‰ of a discovery (goal 3.3: 500).
    pub discovery_gain: u16,
    /// Ticks of one teaching session (goal 3.3: 60).
    pub session_ticks: u32,
    /// ‰ a session gives before fidelity (goal 3.3: 15).
    pub session_gain: u16,
    /// Fidelity ‰ of a teacher who knows instruction (goal 3.3: 950).
    pub fidelity: u32,
    /// Fidelity ‰ of a teacher without instruction (M4 DECISIONS D8).
    pub fidelity_untrained: u32,
    /// Fidelity bonus ‰ of a teacher at `master_at` or above (goal 3.3: × 1,100 ‰).
    pub master_permille: u32,
    /// Proficiency ‰ of a master teacher (goal 3.3: 800).
    pub master_at: u16,
    /// Most sessions a day per order (goal 3.3: 4).
    pub sessions_per_day: u8,
    /// Ticks between observation checks (goal 3.3: 30).
    pub observation_interval: u64,
    /// ‰ a watcher gains per check (goal 3.3: 5).
    pub observation_gain: u16,
    /// Chebyshev distance of a watcher from the practitioner (goal 3.3: 3).
    pub observation_radius: u32,
    /// Ticks between aggregates (goal 3.4: 240).
    pub aggregate_interval: u64,
    /// Days within which a holder counts as a practitioner (goal 3.4: 6).
    pub practice_days: u64,
    /// Practitioners for a stable node (goal 3.4: 3).
    pub stable_at: u16,
}

impl KnowledgeRule {
    /// The M4 values (goal 3.2–3.4; DECISIONS for the ones the goal leaves open).
    pub const M4: Self = Self {
        oral_minute: 1230,
        gather_radius: 12,
        oral_gain: 350,
        oral_chance_permille: 500,
        storytelling_bonus_permille: 100,
        learner_min_years: 10,
        unused_days: 6,
        decay_interval: 60,
        decay: [1, 1, 2, 2, 1],
        floor: [100, 100, 0, 0, 0],
        use_gain: 2,
        discovery_interval: 240,
        discovery_ns: 600,
        discovery_odds: [40, 90, 200],
        discovery_gain: 500,
        session_ticks: 60,
        session_gain: 15,
        fidelity: 950,
        fidelity_untrained: 700,
        master_permille: 1100,
        master_at: 800,
        sessions_per_day: 4,
        observation_interval: 30,
        observation_gain: 5,
        observation_radius: 3,
        aggregate_interval: 240,
        practice_days: 6,
        stable_at: 3,
    };

    /// The forgetting step ‰ of an entry from `source`.
    pub const fn decay_of(&self, source: crate::colony::knowledge::Source) -> u16 {
        self.decay[source.code() as usize]
    }

    /// The forgetting floor ‰ of an entry from `source`.
    pub const fn floor_of(&self, source: crate::colony::knowledge::Source) -> u16 {
        self.floor[source.code() as usize]
    }
}
