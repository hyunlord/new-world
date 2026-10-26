//! Opt-in old-arm observations, outside simulation state, RNG and event IDs.
//! Bands are comparison mappings, not values computed by the legacy cascade.
//! Drain after each tick: evaluations include replay; rows are unique person-ticks.
use sim_core::components::{ActivityKind, AgentId, TargetKind};
use std::{cell::RefCell, collections::BTreeMap};

/// Comparison-only band mapping.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Band {
    /// Immediate threat response.
    Critical,
    /// Direct duty ownership.
    Duty,
    /// Basic needs, meals and social need.
    Need,
    /// Work and gathering.
    Work,
    /// Routine activities.
    Leisure,
    /// Idle behaviour, when explicitly selected.
    Idle,
}
/// Inventory disposition of the observed choice, not its event provenance.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Preserved choice semantics.
    Port,
    /// Explicitly replaced policy from the inventory.
    Replace,
    /// Colony-unreachable arm retained for observation.
    Drop,
    /// Insufficient branch evidence to classify.
    Uncovered,
}
/// Explicit branch witness; unknown paths are never inferred from final state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Choice {
    /// Stable observer arm label.
    pub arm: &'static str,
    /// Behaviour selected/owned at this branch, or an explicit unknown/no-choice.
    pub behaviour: &'static str,
    /// Mapped band; None means no selected behaviour or unresolved mapping.
    pub mapped_band: Option<Band>,
    /// Inventory disposition, including an uncovered sentinel.
    pub verdict: Verdict,
    /// Actual routine activity, when available.
    pub activity: Option<ActivityKind>,
    /// Actual lifecycle exit, retaining the dispatched behaviour for this tick.
    pub exit: Option<&'static str>,
}
impl Choice {
    /// Construct a branch witness without touching the simulation.
    pub const fn new(
        arm: &'static str,
        behaviour: &'static str,
        mapped_band: Option<Band>,
        verdict: Verdict,
    ) -> Self {
        Self {
            arm,
            behaviour,
            mapped_band,
            verdict,
            activity: None,
            exit: None,
        }
    }
}
const UNCOVERED: Choice = Choice::new("uncovered", "uncovered", None, Verdict::Uncovered);
/// One actual pass through the old decision loop.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evaluation {
    /// Whether this is an invitation replay evaluation.
    pub replay: bool,
    /// Last explicit choice/continuation branch in this evaluation.
    pub choice: Choice,
}
/// Unique person-tick, retaining evaluations and the post-settlement outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersonTick {
    /// Simulation tick.
    pub tick: u64,
    /// Stable person identifier.
    pub agent: AgentId,
    /// Primary plus every replay evaluation; length is the evaluation denominator.
    pub evaluations: Vec<Evaluation>,
    /// Last replay choice, overridden only by an explicit job restoration branch.
    pub final_choice: Choice,
    /// Actual settlement branch, or None when this person was not held.
    pub settlement: Option<&'static str>,
    /// A held job still awaits its explicit settlement hook.
    pub pending_settlement: bool,
}
#[derive(Default)]
struct Observer {
    enabled: bool,
    rows: BTreeMap<(u64, AgentId), PersonTick>,
    ownership: BTreeMap<AgentId, Choice>,
}
thread_local! { static OBSERVER: RefCell<Observer> = RefCell::new(Observer::default()); }
/// Start a fresh capture on this simulation thread. Call once per world.
pub fn begin() {
    OBSERVER.with(|o| {
        *o.borrow_mut() = Observer {
            enabled: true,
            ..Observer::default()
        }
    });
}
/// Stop capture and discard thread-local history, including continuation ownership.
pub fn end() {
    OBSERVER.with(|o| *o.borrow_mut() = Observer::default());
}
/// Drain sorted unique person-ticks, retaining ownership for subsequent ticks.
/// Call only after the full decision tick (including replay and settlement).
pub fn drain() -> Vec<PersonTick> {
    OBSERVER.with(|o| {
        std::mem::take(&mut o.borrow_mut().rows)
            .into_values()
            .collect()
    })
}
/// Previous explicit old ownership; never inferred from the current final state.
pub fn previous_choice(agent: AgentId) -> Option<Choice> {
    OBSERVER.with(|o| o.borrow().ownership.get(&agent).copied())
}
/// Scope guard emits even when a legacy branch exits with continue.
pub struct PersonGuard {
    tick: u64,
    agent: AgentId,
    replay: bool,
    enabled: bool,
    choice: Choice,
    prior: Option<Choice>,
    pending: bool,
}
/// Begin one real evaluation, not one inferred state sample.
pub fn start_person(tick: u64, agent: AgentId, replay: bool) -> PersonGuard {
    let (enabled, prior) = OBSERVER.with(|o| {
        let o = o.borrow();
        (o.enabled, o.ownership.get(&agent).copied())
    });
    PersonGuard {
        tick,
        agent,
        replay,
        enabled,
        choice: UNCOVERED,
        prior,
        pending: false,
    }
}
impl PersonGuard {
    /// Record lifecycle completion without erasing the behaviour dispatched this tick.
    pub fn exit(&mut self, arm: &'static str) {
        self.choice.exit = Some(arm);
    }
    /// Record an actual selected branch; later explicit branches may supersede it.
    pub fn set(&mut self, choice: Choice) {
        self.choice = choice;
    }
    /// Mark deferred job ownership resolution without predicting its result.
    pub fn held(&mut self) {
        self.pending = true;
    }
    /// Map an explicit selected target. Agent means social only at social branches.
    pub fn target(&mut self, arm: &'static str, target: TargetKind) {
        let (behaviour, band, verdict) = match target {
            TargetKind::Food => ("eat", Some(Band::Need), Verdict::Port),
            TargetKind::Water => ("drink", Some(Band::Need), Verdict::Port),
            TargetKind::Sleep => ("sleep", Some(Band::Need), Verdict::Port),
            TargetKind::ConstructionSite => {
                ("assist_construction", Some(Band::Work), Verdict::Drop)
            }
            TargetKind::Agent(_) => ("socialise", Some(Band::Need), Verdict::Port),
            TargetKind::Job => ("do_job", Some(Band::Work), Verdict::Replace),
            TargetKind::GatherWood => ("firewood", Some(Band::Work), Verdict::Drop),
            TargetKind::GatherFood | TargetKind::Activity => {
                ("uncovered_target", None, Verdict::Uncovered)
            }
        };
        self.set(Choice::new(arm, behaviour, band, verdict));
    }
    /// Preserve measured gather ownership; a warm-start ambiguity stays visible.
    pub fn gather_continuation(&mut self, arm: &'static str) {
        self.set(match self.prior {
            Some(mut choice)
                if choice.exit.is_none() && matches!(choice.behaviour, "forage" | "gather") =>
            {
                choice.arm = arm;
                choice
            }
            _ => Choice::new(arm, "unresolved_gather_owner", None, Verdict::Uncovered),
        });
    }
    /// Record the actual routine kind, rather than guessing from Activity target.
    pub fn routine(&mut self, arm: &'static str, activity: Option<ActivityKind>) {
        let mut choice = Choice::new(arm, "routine", Some(Band::Leisure), Verdict::Port);
        choice.activity = activity;
        if activity.is_none() {
            choice.verdict = Verdict::Uncovered;
        }
        self.set(choice);
    }
}
impl Drop for PersonGuard {
    fn drop(&mut self) {
        if !self.enabled {
            return;
        }
        OBSERVER.with(|o| {
            let mut o = o.borrow_mut();
            let row = o
                .rows
                .entry((self.tick, self.agent))
                .or_insert_with(|| PersonTick {
                    tick: self.tick,
                    agent: self.agent,
                    evaluations: Vec::new(),
                    final_choice: UNCOVERED,
                    settlement: None,
                    pending_settlement: false,
                });
            row.evaluations.push(Evaluation {
                replay: self.replay,
                choice: self.choice,
            });
            row.final_choice = self.choice;
            row.pending_settlement |= self.pending;
            o.ownership.insert(self.agent, self.choice);
        });
    }
}
/// Observe an actual seam branch. Restoration changes final ownership; release
/// or stale-job keeps the recorded selection. Missing evaluation stays missing.
pub fn settle(tick: u64, agent: AgentId, branch: &'static str, restored: bool) {
    OBSERVER.with(|o| {
        let mut o = o.borrow_mut();
        let Some(row) = o.rows.get_mut(&(tick, agent)) else {
            return;
        };
        row.settlement = Some(branch);
        row.pending_settlement = false;
        if restored {
            row.final_choice =
                Choice::new("job_restore", "do_job", Some(Band::Work), Verdict::Replace);
        }
        let choice = row.final_choice;
        o.ownership.insert(agent, choice);
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn replay_and_settlement_keep_unique_denominator_and_actual_evaluations() {
        // Given: two evaluations of one person and one uncovered person.
        begin();
        {
            let mut g = start_person(5, 2, false);
            g.held();
            g.target("natural", TargetKind::Food);
        }
        {
            let mut g = start_person(5, 2, true);
            g.target("replay", TargetKind::Water);
        }
        drop(start_person(5, 1, false));
        // When: actual seam restores the job, then the complete tick is drained.
        settle(5, 2, "restore", true);
        let rows = drain();
        // Then: sorted unique rows retain replay and separate finalized ownership.
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].agent, 1);
        assert_eq!(rows[0].final_choice.verdict, Verdict::Uncovered);
        assert_eq!(rows[1].evaluations.len(), 2);
        assert!(rows[1].evaluations[1].replay);
        assert_eq!(rows[1].evaluations[1].choice.behaviour, "drink");
        assert_eq!(rows[1].final_choice.behaviour, "do_job");
        assert!(!rows[1].pending_settlement);
        end();
    }
    #[test]
    fn disabled_capture_and_unknown_gather_never_invent_choices() {
        // Given: capture disabled.
        end();
        drop(start_person(1, 1, false));
        assert!(drain().is_empty());
        // When: a warm-start gather continuation has no observed origin.
        begin();
        {
            let mut g = start_person(2, 1, false);
            g.gather_continuation("consume");
        }
        // Then: the ambiguity is explicit, not guessed as forage or gather.
        assert_eq!(drain()[0].final_choice.verdict, Verdict::Uncovered);
        end();
    }
    #[test]
    fn gather_origin_survives_continuation_but_not_explicit_completion() {
        begin();
        {
            let mut g = start_person(1, 4, false);
            g.set(Choice::new(
                "forage",
                "forage",
                Some(Band::Work),
                Verdict::Port,
            ));
        }
        drain();
        {
            let mut g = start_person(2, 4, false);
            g.gather_continuation("consume");
            g.exit("gather_pickup_complete");
        }
        let rows = drain();
        assert_eq!(rows[0].final_choice.behaviour, "forage");
        assert_eq!(rows[0].final_choice.exit, Some("gather_pickup_complete"));
        {
            let mut g = start_person(3, 4, false);
            g.gather_continuation("unexpected_gather");
        }
        assert_eq!(drain()[0].final_choice.verdict, Verdict::Uncovered);
        end();
    }
    #[test]
    fn release_and_stale_preserve_last_replay_choice() {
        // Given: a held worker has selected water in a replay evaluation.
        for branch in ["release", "stale_job"] {
            begin();
            {
                let mut g = start_person(3, 9, true);
                g.held();
                g.target("replay", TargetKind::Water);
            }
            // When: the actual seam keeps that selected behaviour.
            settle(3, 9, branch, false);
            let rows = drain();
            // Then: settlement is visible without replacing the choice with do_job.
            assert_eq!(rows[0].final_choice.behaviour, "drink");
            assert_eq!(rows[0].settlement, Some(branch));
            assert!(!rows[0].pending_settlement);
            end();
        }
    }
}
