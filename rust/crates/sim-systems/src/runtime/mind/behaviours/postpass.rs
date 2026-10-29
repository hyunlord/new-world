//! Deferred effect application and target resolution, after all decision/replay rows.
use super::context::*;
use crate::runtime::agent_actions::{
    forage::choose_bush,
    routine::RoutineTick,
    targets::{nearest_gather_tile, nearest_local_food, nearest_resource_tile, GATHER_SCAN_RADIUS},
};
use hecs::{Entity, World};
use sim_core::components::{
    Agent, AgentId, AgentState, Position, SeekTarget, SettlementMigrant, TargetKind,
};
use sim_engine::SimResources;
use std::collections::{BTreeMap, HashMap};
type Retarget = (AgentId, Entity, (u32, u32), (u32, u32));

pub(in crate::runtime) fn finish_tick(
    world: &mut World,
    resources: &mut SimResources,
    effects: &mut EffectState,
    routine: Option<&RoutineTick>,
) {
    super::settle::settle_jobs(world, resources, effects);
    for entry in std::mem::take(&mut effects.quarrel_entries) {
        crate::runtime::chronicle_log::record(world, resources, entry);
    }
    // Discard provisional attaches; resolve the final FSM after replay and job restoration.
    let selected_targets: BTreeMap<_, _> = effects.seek_set.iter().copied().collect();
    effects.seek_set.clear();
    for entity in std::mem::take(&mut effects.seek_clear) {
        let _ = world.remove_one::<SeekTarget>(entity);
    }
    let positions: HashMap<AgentId, (u32, u32)> = world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (a, p))| (a.id, (p.x, p.y)))
        .collect();
    let forage = resources
        .rules
        .agent
        .forage
        .filter(|_| resources.rules.society.stock_target.is_some());
    let mut held = BTreeMap::new();
    let mut retarget: Vec<Retarget> = Vec::new();
    {
        let mut query = world.query::<(
            &Agent,
            &Position,
            &AgentState,
            Option<&SeekTarget>,
            Option<&SettlementMigrant>,
        )>();
        let mut rows: Vec<_> = query.iter().collect();
        if resources.rules.agent.keyed_draws || routine.is_some() {
            rows.sort_by_key(|(_, row)| row.0.id);
        }
        for (entity, (agent, pos, state, seek, migrant)) in rows {
            match state {
                AgentState::Seeking {
                    target: TargetKind::Food,
                } if resources.rules.colony.is_some() => {
                    if let Some(tile) = selected_targets.get(&entity) {
                        effects.seek_set.push((entity, *tile));
                    } else if seek.is_none() {
                        effects.seek_idle.push(entity);
                    }
                }
                AgentState::Seeking {
                    target: TargetKind::Activity,
                } => {
                    if let Some(activity) = routine.and_then(|r| r.activity(agent.id)) {
                        if seek.map(|s| s.tile) != Some(activity.place) {
                            effects.seek_set.push((entity, activity.place));
                        }
                    } else {
                        effects.seek_idle.push(entity);
                    }
                }
                AgentState::Seeking {
                    target: TargetKind::GatherFood,
                } if forage.is_some() => {
                    let tile = if let Some(tile) = effects.forage_targets.get(&entity) {
                        effects.seek_set.push((entity, *tile));
                        Some(*tile)
                    } else if let Some(seek) = seek {
                        if resources.food_tiles.get(&seek.tile).is_some_and(|n| *n > 0) {
                            Some(seek.tile)
                        } else {
                            retarget.push((agent.id, entity, (pos.x, pos.y), seek.tile));
                            None
                        }
                    } else {
                        let tile = nearest_gather_tile(
                            pos,
                            &resources.food_tiles,
                            GATHER_SCAN_RADIUS,
                            effects
                                .local_home
                                .get(&agent.id)
                                .map(|home| (*home, effects.catchment)),
                            effects.gather_avoid(agent.id),
                        );
                        if let Some(tile) = tile {
                            effects.seek_set.push((entity, tile));
                        } else {
                            effects.seek_idle.push(entity);
                        }
                        tile
                    };
                    if let Some(tile) = tile {
                        *held.entry(tile).or_insert(0_u32) += 1;
                    }
                }
                AgentState::Seeking {
                    target:
                        TargetKind::Food
                        | TargetKind::Water
                        | TargetKind::Sleep
                        | TargetKind::GatherFood,
                } => {
                    let bed_changed = resources.rules.agent.night_rest.is_some()
                        && matches!(
                            state,
                            AgentState::Seeking {
                                target: TargetKind::Sleep
                            }
                        )
                        && resources
                            .bed_of
                            .get(&agent.id)
                            .is_some_and(|bed| seek.is_some_and(|s| s.tile != *bed));
                    // A completed job may leave its construction target attached while
                    // the final FSM has already switched to drinking or sleeping.
                    let colony_resource = resources.rules.colony.is_some()
                        && matches!(state.target(), Some(TargetKind::Water | TargetKind::Sleep));
                    let valid = |tile: &(u32, u32)| match state.target() {
                        Some(TargetKind::Water) => {
                            resources.water_tiles.get(tile).is_some_and(|n| *n > 0)
                        }
                        Some(TargetKind::Sleep) => {
                            resources.sleep_tiles.get(tile).is_some_and(|n| *n > 0)
                                && (resources.rules.agent.night_rest.is_none()
                                    || resources
                                        .bed_of
                                        .get(&agent.id)
                                        .is_none_or(|bed| bed == tile))
                        }
                        _ => true,
                    };
                    let selected = colony_resource
                        .then(|| selected_targets.get(&entity).copied())
                        .flatten()
                        .filter(valid);
                    let stale = colony_resource && seek.is_some_and(|s| !valid(&s.tile));
                    if let Some(tile) = selected {
                        effects.seek_set.push((entity, tile));
                    } else if seek.is_none() || bed_changed || stale {
                        if let Some(tile) =
                            resource_target(agent.id, pos, *state, resources, effects)
                                .filter(|tile| !colony_resource || valid(tile))
                        {
                            effects.seek_set.push((entity, tile));
                        } else if colony_resource {
                            effects.seek_idle.push(entity);
                        }
                    }
                }
                AgentState::Seeking {
                    target: TargetKind::Agent(peer),
                } => {
                    let classified = migrant.is_some()
                        || effects.social_seek_entities.contains(&entity)
                        || seek.is_some();
                    let tile = if classified && *peer != agent.id {
                        positions.get(peer).copied()
                    } else {
                        None
                    };
                    match tile {
                        Some(tile) if seek.map(|s| s.tile) != Some(tile) => {
                            effects.seek_set.push((entity, tile))
                        }
                        Some(_) => {}
                        None if seek.is_some() => effects.seek_clear.push(entity),
                        None => {}
                    }
                }
                AgentState::Seeking {
                    target: TargetKind::Job,
                }
                | AgentState::Consuming {
                    target: TargetKind::Job,
                } => {}
                AgentState::Idle
                | AgentState::Seeking {
                    target: TargetKind::ConstructionSite | TargetKind::GatherWood,
                }
                | AgentState::Consuming { .. } => {
                    if seek.is_some() {
                        effects.seek_clear.push(entity);
                    }
                }
            }
        }
    }
    if let Some(rule) = forage {
        retarget.sort_by_key(|row| row.0);
        for (agent, entity, position, old) in retarget {
            let home = effects.members.get(&agent).and_then(|sid| {
                effects.forage_plans.get(sid).map(|p| {
                    (
                        p.home,
                        effects.avoid_of.get(sid).map_or(&[][..], Vec::as_slice),
                    )
                })
            });
            match home.and_then(|(home, avoid)| {
                choose_bush(
                    &rule,
                    home,
                    position,
                    &resources.food_tiles,
                    &held,
                    Some(old),
                    avoid,
                )
            }) {
                Some(tile) => {
                    *held.entry(tile).or_insert(0) += 1;
                    effects.seek_set.push((entity, tile));
                }
                None => effects.seek_idle.push(entity),
            }
        }
    }
    for (entity, tile) in std::mem::take(&mut effects.seek_set) {
        let _ = world.insert_one(entity, SeekTarget { tile });
    }
    for entity in std::mem::take(&mut effects.seek_clear) {
        let _ = world.remove_one::<SeekTarget>(entity);
    }
    for entity in std::mem::take(&mut effects.seek_idle) {
        let _ = world.insert_one(entity, AgentState::Idle);
        let _ = world.remove_one::<SeekTarget>(entity);
    }
}

fn resource_target(
    agent: AgentId,
    pos: &Position,
    state: AgentState,
    resources: &SimResources,
    effects: &EffectState,
) -> Option<(u32, u32)> {
    match state {
        AgentState::Seeking {
            target: TargetKind::Water,
        } => nearest_resource_tile(pos, &resources.water_tiles),
        AgentState::Seeking {
            target: TargetKind::Sleep,
        } => resources
            .bed_of
            .get(&agent)
            .copied()
            .filter(|_| resources.rules.agent.night_rest.is_some())
            .or_else(|| nearest_resource_tile(pos, &resources.sleep_tiles)),
        AgentState::Seeking {
            target: TargetKind::GatherFood,
        } => nearest_gather_tile(
            pos,
            &resources.food_tiles,
            GATHER_SCAN_RADIUS,
            effects
                .local_home
                .get(&agent)
                .map(|home| (*home, effects.catchment)),
            effects.gather_avoid(agent),
        ),
        AgentState::Seeking {
            target: TargetKind::Food,
        } => match effects.local_home.get(&agent) {
            Some(home) => nearest_local_food(pos, &resources.food_tiles, *home, effects.catchment),
            None => nearest_resource_tile(pos, &resources.food_tiles),
        },
        AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } => None,
    }
}
