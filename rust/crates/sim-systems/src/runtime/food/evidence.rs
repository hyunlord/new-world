//! Observation-only receipts at real food/job boundaries; no policy decisions.
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::food_evidence::{
    AcquisitionEpisode, AcquisitionReason, AcquisitionReceipt, FlowCounters, FlowInterval,
    OperationEvidence,
};
use sim_engine::SimResources;

/// Retain an actual failed acquisition boundary without changing food or job policy.
pub fn record_failure(res: &mut SimResources, agent: u64, receipt: AcquisitionReceipt) {
    if res.rules.colony.is_none() {
        return;
    }
    let counters = FlowCounters::capture(&res.food.tallies);
    let episode = res
        .food
        .evidence
        .active
        .entry(agent)
        .or_insert(AcquisitionEpisode {
            since_tick: receipt.tick,
            counters,
            last: receipt,
        });
    // Generic cancellation immediately follows a specific completion failure.
    // Keep the more informative receipt for the exact same job at the same tick.
    if episode.last.tick == receipt.tick
        && episode.last.target == receipt.target
        && episode.last.job_started == receipt.job_started
        && matches!(receipt.reason, AcquisitionReason::Cancelled(_))
        && !matches!(episode.last.reason, AcquisitionReason::Cancelled(_))
    {
        return;
    }
    episode.last = receipt;
}

/// Retire stale acquisition observations after success or person removal.
pub fn clear_person(res: &mut SimResources, agent: u64) {
    res.food.evidence.active.remove(&agent);
}

/// Close a daily colony-flow interval, retaining at most four completed rows.
pub fn observe_day(res: &mut SimResources) {
    if res.rules.colony.is_none() {
        return;
    }
    let now = FlowCounters::capture(&res.food.tallies);
    if let Some((tick, counters)) = res.food.evidence.cursor {
        if tick == res.current_tick {
            return;
        }
        if let Some(units) = now.since(counters) {
            let recent = &mut res.food.evidence.recent;
            if recent.len() == 4 {
                recent.pop_front();
            }
            recent.push_back(FlowInterval {
                from_tick: tick,
                to_tick: res.current_tick,
                units,
            });
        }
    }
    res.food.evidence.cursor = Some((res.current_tick, now));
}

/// Append an immutable cause only after observing its actual operation boundary.
pub fn operation(res: &mut SimResources, agent: Option<u64>, payload: OperationEvidence) -> u32 {
    let code = match payload {
        OperationEvidence::Acquisition { .. } => 0,
        OperationEvidence::Intake { .. } => 1,
        OperationEvidence::Stored { .. } => 2,
    };
    let mut entry = ChronicleEntry::new(res.current_tick, ChronicleKind::FoodOperation).cause(code);
    if let Some(agent) = agent {
        entry = entry.subject(agent);
    }
    let index = res.chronicle.push_caused(entry, None);
    res.food.evidence.operations.insert(index, payload);
    index
}

/// Copy a real personal receipt plus explicitly colony-wide flows, then retire it.
pub fn death_parent(res: &mut SimResources, agent: u64) -> Option<u32> {
    let episode = res.food.evidence.active.remove(&agent)?;
    let units = FlowCounters::capture(&res.food.tallies).since(episode.counters)?;
    let tick = res.current_tick;
    Some(operation(
        res,
        Some(agent),
        OperationEvidence::Acquisition {
            receipt: episode.last,
            flow: FlowInterval {
                from_tick: episode.since_tick,
                to_tick: tick,
                units,
            },
        },
    ))
}

#[cfg(test)]
#[path = "evidence_tests.rs"]
mod tests;

/// Observe the same person's need and actual actionable food at daily expiry.
pub(super) fn hungry_at_expiry(
    world: &hecs::World,
    res: &SimResources,
    owner: sim_core::colony::food::FoodOwner,
    choices: &super::choice::FoodChoiceIndex,
) -> Option<sim_core::components::SettlementId> {
    use sim_core::colony::{food::FoodOwner, Carry};
    use sim_core::components::{Agent, AgentState, BodyHealth, Hunger, Position, TargetKind};
    let sid = match owner {
        FoodOwner::Ground(tile) => super::reserves::owner_at(res, tile),
        FoodOwner::Shelf { workshop, .. } => res
            .colony
            .workshops
            .get(&workshop)
            .and_then(|shop| super::reserves::owner_at(res, shop.work_tile())),
        FoodOwner::Pot { id, .. } => res
            .food
            .pots
            .get(&id)
            .and_then(|pot| super::reserves::owner_at(res, pot.tile)),
        FoodOwner::Carried(agent) => res
            .settlements
            .iter()
            .find(|(_, settlement)| settlement.member_agents.contains(&agent))
            .map(|(&id, _)| id),
    }?;
    let settlement = res.settlements.get(&sid)?;
    world
        .query::<(
            &Agent,
            &Position,
            &Hunger,
            &BodyHealth,
            &AgentState,
            Option<&Carry>,
        )>()
        .iter()
        .any(|(_, (agent, pos, hunger, health, state, carry))| {
            settlement.member_agents.contains(&agent.id)
                && !health.is_dead()
                && !matches!(
                    state,
                    AgentState::Consuming {
                        target: TargetKind::Sleep
                    }
                )
                && hunger.value > crate::runtime::agent_actions::needs::HUNGER_THRESHOLD
                && !choices.available_to(res, (pos.x, pos.y), agent.id, carry.copied())
        })
        .then_some(sid)
}
