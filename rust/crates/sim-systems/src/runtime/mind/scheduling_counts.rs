//! Fixed diagnostic counters; no persistent simulation or decision state.
use sim_core::colony::mind::GuardWake;

#[derive(Default, Debug)]
pub(super) struct Counts {
    pub(super) visits: u64,
    pub(super) visits_without_bound_behaviour: u64,
    pub(super) visited_behaviours: Vec<u64>,
    pub(super) unsupported_owner_behaviours: Vec<u64>,
    pub(super) critical_incumbent_behaviours: Vec<u64>,
    pub(super) replay_or_invitation: u64,
    pub(super) missing_history: u64,
    pub(super) mismatched_history: u64,
    pub(super) critical_incumbents: u64,
    pub(super) unsupported_owner: u64,
    pub(super) owner_mismatch: u64,
    pub(super) guard_errors: u64,
    pub(super) reads: u64,
    pub(super) would_continue: u64,
    pub(super) full_reasons: [u64; 8],
    pub(super) continued_behaviours: Vec<u64>,
    pub(super) unsupported_facts: Vec<u64>,
    pub(super) same_node: u64,
    pub(super) different_node: u64,
    pub(super) different_behaviour: u64,
}

pub(super) fn wake_index(reason: GuardWake) -> usize {
    match reason {
        GuardWake::NoRunning => 0,
        GuardWake::Finished => 1,
        GuardWake::Cadence => 2,
        GuardWake::CriticalIncumbent => 3,
        GuardWake::FinishIncumbent => 4,
        GuardWake::IncumbentIneligible => 5,
        GuardWake::UrgentEligible => 6,
        GuardWake::UnsupportedFact => 7,
    }
}
