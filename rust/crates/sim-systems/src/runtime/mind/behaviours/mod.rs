//! Registered effect callbacks. The coordinator resolves a winning node and target first.
mod context;
mod food;
mod gather;
mod harvest_food;
mod needs;
mod owned;
mod postpass;
mod prepare;
mod routine;
mod settle;
mod social;

use super::context::TargetCandidate;
pub(in crate::runtime) use context::*;
pub(in crate::runtime) use postpass::finish_tick;
use sim_core::colony::mind::BehaviourSpec;
use sim_core::components::{Activity, ActivityKind, AgentState, TargetKind};

/// Factual lifecycle identity used only to adopt a legacy active state after policy selection.
#[derive(Clone, Copy)]
pub(in crate::runtime) enum LifecycleBinding {
    Idle,
    Resource(TargetKind),
    Social,
    Rest,
    Routine(ActivityKind),
    External,
}
impl LifecycleBinding {
    fn matches(self, state: AgentState, activity: Option<Activity>, target: ActionTarget) -> bool {
        let activity_matches = |kind| {
            matches!(
                state,
                AgentState::Seeking {
                    target: TargetKind::Activity
                } | AgentState::Consuming {
                    target: TargetKind::Activity
                }
            ) && activity.is_some_and(|active| active.kind == kind)
        };
        match self {
            Self::Idle => state == AgentState::Idle,
            Self::Resource(kind) => {
                matches!(state, AgentState::Seeking { target } | AgentState::Consuming { target } if target == kind)
            }
            Self::Social => match (state, target) {
                (
                    AgentState::Seeking {
                        target: TargetKind::Agent(current),
                    }
                    | AgentState::Consuming {
                        target: TargetKind::Agent(current),
                    },
                    ActionTarget::Agent { agent, .. },
                ) => current == agent,
                // A missing peer ends through the existing lifecycle's cooldown
                // path; it does not make an active social FSM a new entry.
                (
                    AgentState::Seeking {
                        target: TargetKind::Agent(_),
                    }
                    | AgentState::Consuming {
                        target: TargetKind::Agent(_),
                    },
                    ActionTarget::None,
                ) => true,
                _ => false,
            },
            Self::Rest => match target {
                ActionTarget::None => state == AgentState::Idle,
                ActionTarget::Tile(_) => activity_matches(ActivityKind::SitByFire),
                ActionTarget::Agent { .. }
                | ActionTarget::Routine { .. }
                | ActionTarget::Food { .. } => false,
            },
            Self::Routine(kind) => activity_matches(kind),
            Self::External => true,
        }
    }
}

pub(in crate::runtime) type Effect = for<'a> fn(&mut EffectContext<'a>) -> EffectResult;
pub(in crate::runtime) struct BehaviourEntry {
    pub name: &'static str,
    pub targets: &'static [&'static str],
    pub activity_kind: Option<ActivityKind>,
    /// Another runtime system owns the current FSM and seek target.
    pub externally_owned: bool,
    pub lifecycle: LifecycleBinding,
    pub start: Effect,
    pub continue_running: Effect,
    pub finish: Effect,
}
impl BehaviourEntry {
    pub(in crate::runtime) fn scheduled_continuation(&self) -> bool {
        self.diagnostic_continuation()
            || matches!(self.lifecycle, LifecycleBinding::Social)
            || (matches!(self.lifecycle, LifecycleBinding::Idle) && self.targets.is_empty())
    }

    pub(in crate::runtime) fn diagnostic_continuation(&self) -> bool {
        matches!(
            self.lifecycle,
            LifecycleBinding::Resource(
                TargetKind::Food | TargetKind::Water | TargetKind::Sleep | TargetKind::GatherFood
            ) | LifecycleBinding::Routine(_)
        )
    }

    pub(in crate::runtime) fn adopts(
        &self,
        state: AgentState,
        activity: Option<Activity>,
        target: ActionTarget,
    ) -> bool {
        self.lifecycle.matches(state, activity, target)
    }

    pub(in crate::runtime) fn action_target(
        &self,
        candidate: Option<TargetCandidate>,
    ) -> Result<ActionTarget, EffectError> {
        Ok(match candidate {
            None => ActionTarget::None,
            Some(target) if target.food.is_some() => ActionTarget::Food {
                tile: target.tile,
                owner: target.food.ok_or(EffectError::WrongTarget)?,
            },
            Some(target) => match (self.activity_kind, target.agent) {
                (Some(kind), None) => ActionTarget::Routine {
                    kind,
                    tile: target.tile,
                },
                (None, Some(agent)) => ActionTarget::Agent {
                    agent,
                    tile: target.tile,
                },
                (None, None) => ActionTarget::Tile(target.tile),
                (Some(_), Some(_)) => return Err(EffectError::WrongTarget),
            },
        })
    }
}

macro_rules! entry {
    (owned, $name:literal, $targets:expr, $start:path, $continue:path, $finish:path) => {
        BehaviourEntry {
            name: $name,
            targets: $targets,
            activity_kind: None,
            externally_owned: true,
            lifecycle: LifecycleBinding::External,
            start: $start,
            continue_running: $continue,
            finish: $finish,
        }
    };
    ($name:literal, $targets:expr, $start:path, $continue:path, $finish:path, $kind:path) => {
        BehaviourEntry {
            name: $name,
            targets: $targets,
            activity_kind: Some($kind),
            externally_owned: false,
            lifecycle: LifecycleBinding::Routine($kind),
            start: $start,
            continue_running: $continue,
            finish: $finish,
        }
    };
    ($binding:expr; $name:literal, $targets:expr, $start:path, $continue:path, $finish:path) => {
        BehaviourEntry {
            name: $name,
            targets: $targets,
            activity_kind: None,
            externally_owned: false,
            lifecycle: $binding,
            start: $start,
            continue_running: $continue,
            finish: $finish,
        }
    };
}
pub(in crate::runtime) static REGISTRY: &[BehaviourEntry] = &[
    entry!(LifecycleBinding::Resource(TargetKind::Job);
        "harvest_food",
        &["edible_plant"],
        harvest_food::start,
        harvest_food::continue_running,
        owned::owned_finish
    ),
    entry!(LifecycleBinding::Resource(TargetKind::Food);
        "eat",
        &["food", "food_cooked", "food_novel", "food_cooked_novel"],
        needs::eat_start,
        needs::eat_continue,
        needs::finish
    ),
    entry!(LifecycleBinding::Resource(TargetKind::Water);
        "drink",
        &["water"],
        needs::drink_start,
        needs::drink_continue,
        needs::finish
    ),
    entry!(LifecycleBinding::Resource(TargetKind::Sleep);
        "sleep",
        &["sleep"],
        needs::sleep_start,
        needs::sleep_continue,
        needs::finish
    ),
    entry!(
        owned,
        "flee",
        &["refuge"],
        owned::flee,
        owned::flee,
        owned::owned_finish
    ),
    entry!(
        owned,
        "fight_drafted",
        &["draft_target"],
        owned::fight,
        owned::fight,
        owned::owned_finish
    ),
    entry!(
        owned,
        "do_job",
        &["job"],
        owned::job,
        owned::job,
        owned::job_finish
    ),
    entry!(LifecycleBinding::Resource(TargetKind::Food);
        "hearth_meal",
        &["hearth", "food", "food_cooked", "food_novel", "food_cooked_novel"],
        needs::eat_start,
        needs::eat_continue,
        needs::finish
    ),
    entry!(LifecycleBinding::Social;
        "socialise",
        &["social_peer", "social_memory_peer", "social_current_peer"],
        social::start,
        social::continue_running,
        social::finish
    ),
    entry!(LifecycleBinding::Rest;
        "rest",
        &["hearth"],
        owned::rest_start,
        owned::rest_continue,
        owned::rest_finish
    ),
    entry!(LifecycleBinding::Idle;
        "routine_no_place",
        &[],
        routine::no_place,
        routine::no_place,
        needs::finish
    ),
    entry!(LifecycleBinding::Idle;
        "wander",
        &[],
        owned::idle_start,
        owned::idle_continue,
        needs::finish
    ),
    entry!(LifecycleBinding::Resource(TargetKind::GatherFood);
        "forage",
        &["forage_bush"],
        gather::forage_start,
        gather::continue_running,
        gather::finish
    ),
    entry!(LifecycleBinding::Resource(TargetKind::GatherFood);
        "gather",
        &["gather_food"],
        gather::start,
        gather::continue_running,
        gather::finish
    ),
    entry!(
        "routine_tend_fire",
        &["routine_tend_fire"],
        routine::start,
        routine::continue_running,
        routine::finish,
        ActivityKind::TendFire
    ),
    entry!(
        "routine_craft",
        &["routine_craft"],
        routine::start,
        routine::continue_running,
        routine::finish,
        ActivityKind::Craft
    ),
    entry!(
        "routine_repair",
        &["routine_repair"],
        routine::start,
        routine::continue_running,
        routine::finish,
        ActivityKind::Repair
    ),
    entry!(
        "routine_hearth_circle",
        &["routine_hearth_circle"],
        routine::start,
        routine::continue_running,
        routine::finish,
        ActivityKind::HearthCircle
    ),
    entry!(
        "routine_explore",
        &["routine_explore"],
        routine::start,
        routine::continue_running,
        routine::finish,
        ActivityKind::Explore
    ),
    entry!(
        "routine_play",
        &["routine_play"],
        routine::start,
        routine::continue_running,
        routine::finish,
        ActivityKind::Play
    ),
    entry!(
        "routine_sit_by_fire",
        &["routine_sit_by_fire"],
        routine::start,
        routine::continue_running,
        routine::finish,
        ActivityKind::SitByFire
    ),
];

pub(in crate::runtime) fn registry_schema() -> Vec<BehaviourSpec> {
    REGISTRY
        .iter()
        .map(|entry| BehaviourSpec {
            name: entry.name.to_owned(),
            target_bindings: entry
                .targets
                .iter()
                .map(|name| (*name).to_owned())
                .collect(),
        })
        .collect()
}
/// Resolve names at policy load; hot-path callers retain the entry's index.
pub(in crate::runtime) fn lookup(name: &str) -> Option<&'static BehaviourEntry> {
    REGISTRY.iter().find(|entry| entry.name == name)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod postpass_tests;

#[cfg(test)]
mod gather_tests;
