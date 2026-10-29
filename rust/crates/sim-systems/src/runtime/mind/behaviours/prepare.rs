//! Real tick preparation. Shadow must use the already-prepared read-only view.
use super::context::*;
use crate::runtime::agent_actions::{forage::ForagePlan, routine::members_of};
use crate::runtime::settlement::food_economy::{catchment_radius, stock_in_hand, stock_target};
use hecs::World;
use sim_core::colony::Job;
use sim_core::components::{
    Agent, AgentId, AgentState, Founding, Lineage, PorterTrip, Position, SeekTarget, TargetKind,
    Temperament,
};
use sim_engine::SimResources;

impl EffectState {
    pub(in crate::runtime) fn prepare(world: &World, resources: &mut SimResources) -> Self {
        let all_members = members_of(resources);
        let mut state = Self {
            members: all_members.clone(),
            catchment: catchment_radius(resources),
            ..Self::default()
        };
        let forage = resources.rules.agent.forage.filter(|_| {
            resources.rules.colony.is_none() && resources.rules.society.stock_target.is_some()
        });
        for (&sid, settlement) in &resources.settlements {
            if forage.is_some() {
                state.forage_plans.insert(
                    sid,
                    ForagePlan {
                        home: settlement.formation_tile,
                        target: stock_target(world, resources, sid).unwrap_or(0.0),
                        in_hand: stock_in_hand(world, resources, sid),
                    },
                );
            }
        }
        if resources.rules.agent.local_food_availability {
            for (_, (agent, founder, porter)) in world
                .query::<(&Agent, Option<&Founding>, Option<&PorterTrip>)>()
                .iter()
            {
                if founder.is_some()
                    || (resources.rules.society.relations.is_some() && porter.is_some())
                {
                    continue;
                }
                if let Some(home) = all_members
                    .get(&agent.id)
                    .and_then(|sid| resources.settlements.get(sid))
                    .map(|s| s.formation_tile)
                {
                    state.local_home.insert(agent.id, home);
                }
            }
        }
        if forage.is_none() && crate::runtime::jobs::hearth::decision_firewood(resources).is_none()
        {
            state.members.clear();
        }
        for (entity, (current, job, seek)) in world
            .query::<(&AgentState, Option<&Job>, Option<&SeekTarget>)>()
            .iter()
        {
            if job.is_some()
                && matches!(
                    current,
                    AgentState::Seeking {
                        target: TargetKind::Job
                    } | AgentState::Consuming {
                        target: TargetKind::Job | TargetKind::ConstructionSite
                    }
                )
            {
                state.previous_jobs.insert(entity, *current);
            }
            if forage.is_some()
                && matches!(
                    current,
                    AgentState::Seeking {
                        target: TargetKind::GatherFood
                    }
                )
            {
                if let Some(seek) = seek {
                    *state.claims.entry(seek.tile).or_default() += 1;
                }
            }
        }
        if resources.rules.society.relations.is_some() {
            for (_, (agent, pos, current, seek, temperament, lineage)) in world
                .query::<(
                    &Agent,
                    &Position,
                    &AgentState,
                    Option<&SeekTarget>,
                    Option<&Temperament>,
                    Option<&Lineage>,
                )>()
                .iter()
            {
                let tile = match current {
                    AgentState::Consuming {
                        target: TargetKind::GatherFood,
                    } => Some((pos.x, pos.y)),
                    AgentState::Seeking {
                        target: TargetKind::GatherFood,
                    } => seek.map(|s| s.tile),
                    AgentState::Idle
                    | AgentState::Seeking { .. }
                    | AgentState::Consuming { .. } => None,
                };
                if let (Some(tile), Some(sid)) = (tile, state.members.get(&agent.id)) {
                    state.forage_spots.entry(tile).or_default().push((
                        agent.id,
                        *sid,
                        lineage.map_or(agent.id, |l| l.key),
                        temperament.map_or(500, |t| t.ns),
                        temperament.map_or(500, |t| t.ha),
                    ));
                }
            }
            for rivals in state.forage_spots.values_mut() {
                rivals.sort_unstable_by_key(|r| r.0);
            }
            for ((a, b), relation) in &resources.settlement_relations.pairs {
                if relation.estranged {
                    if let (Some(left), Some(right)) =
                        (resources.settlements.get(a), resources.settlements.get(b))
                    {
                        state
                            .avoid_of
                            .entry(*a)
                            .or_default()
                            .push(right.formation_tile);
                        state
                            .avoid_of
                            .entry(*b)
                            .or_default()
                            .push(left.formation_tile);
                    }
                }
            }
            let day = resources
                .current_tick
                .saturating_add(resources.rules.clock_offset_ticks)
                / resources.ticks_per_day.max(1);
            if resources.settlement_relations.dispute_day != day {
                resources.settlement_relations.dispute_day = day;
                resources.settlement_relations.disputed.clear();
                resources.settlement_relations.day_counts.clear();
            }
        }
        if resources.rules.agent.social_seek.is_some() {
            resources
                .social_retry_after
                .retain(|_, after| resources.current_tick < *after);
        }
        state
    }

    pub(in crate::runtime) fn gather_avoid(&self, agent: AgentId) -> &[(u32, u32)] {
        if !self.local_home.contains_key(&agent) {
            return &[];
        }
        self.members
            .get(&agent)
            .and_then(|sid| self.avoid_of.get(sid))
            .map_or(&[], Vec::as_slice)
    }
}
