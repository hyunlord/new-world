//! The M1 job layer (goal section 7, ADR 0013): every piece of job logic.
//!
//! - [`bills::BillSystem`] (123, every 60 ticks) picks each workshop's active bill (M2).
//! - [`assign::JobAssignSystem`] (124) hands idle or leisured adults a job by
//!   work priority and type (M2 goal 3.6), with reservations.
//! - [`execute::JobExecuteSystem`] (127) walks each job through its stages.
//! - [`cancel`] ends jobs and releases reservations; [`items`] puts items down
//!   (merge and spill) and indexes colony sites; [`craft`] starts and finishes
//!   crafts; [`invariants`] checks the per-kind ledger, the reservations and
//!   the stacks; [`space`] finishes, takes apart and digs built space (M3);
//!   [`teach`] runs apprenticeship orders as paired Teach / Learn jobs (M4).
//!
//! Data flow: command → designation / zone / site (`crate::commands`) →
//! assign (reserve) → execute (move logs, ledger) → `ConstructionSystem`
//! (hut progress and completion). Every system returns at once without
//! `rules.colony`, so the baseline world never runs any of this.
pub mod assign;
pub mod bills;
pub mod cancel;
pub mod craft;
mod craft_food;
pub mod execute;
mod food_transfers;
pub mod harvest;
pub mod hearth;
pub mod hunt;
pub mod invariants;
pub mod items;
mod pot_supply;
pub mod space;
pub mod teach;

#[cfg(test)]
mod harvest_encoding_tests;

#[cfg(test)]
mod food_job_tests;

pub use assign::{JobAssignSystem, JOB_ASSIGN_PRIORITY};
pub use bills::{BillSystem, BILL_PRIORITY};
pub use cancel::{cancel, finish, on_death, release_job};
pub(crate) use craft_food::supplyable as food_recipe_supplyable;
pub use execute::{JobExecuteSystem, JOB_EXECUTE_PRIORITY};
pub use hearth::{HearthFeedSystem, HEARTH_FEED_PRIORITY};

#[cfg(test)]
pub(crate) use legacy_hold_oracle::{hold_for_decision, holders};

// Keep the pre-switch ownership oracle for frozen mind regression tests only.
// This has no production entry point and contains no selection/yield policy.
#[cfg(test)]
mod legacy_hold_oracle {
    use hecs::{Entity, World};
    use sim_core::colony::Job;
    use sim_core::components::{AgentState, TargetKind};
    use sim_engine::SimResources;
    use std::collections::BTreeSet;

    pub(crate) struct JobHolders(BTreeSet<Entity>);

    pub(crate) fn holders(world: &World, _resources: &SimResources) -> JobHolders {
        JobHolders(
            world
                .query::<&Job>()
                .iter()
                .map(|(entity, _)| entity)
                .collect(),
        )
    }

    pub(crate) fn hold_for_decision(
        entity: Entity,
        state: AgentState,
        holders: &JobHolders,
    ) -> bool {
        match state {
            AgentState::Seeking {
                target: TargetKind::Job,
            }
            | AgentState::Consuming {
                target: TargetKind::Job,
            } => true,
            AgentState::Consuming {
                target: TargetKind::ConstructionSite,
            } => holders.0.contains(&entity),
            _ => false,
        }
    }
}

#[cfg(test)]
mod food_admission_tests;

#[cfg(test)]
mod pot_supply_tests;

#[cfg(test)]
mod pot_ingredient_tests;
