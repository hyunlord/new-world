//! Gather and forage lifecycle effects after authored policy selection.
use super::context::*;

use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::components::{
    Agent, AgentState, Inventory, ResourceKind, SettlementRelations, TargetKind, Temperament,
    INVENTORY_CAPACITY,
};
use sim_core::rng::{keyed_below, STREAM_RELATIONS};
use sim_engine::RESOURCE_SOURCE_INFINITE;

pub(super) fn finish(c: &mut EffectContext<'_>) -> EffectResult {
    if c.authored_completion
        && c.finish_reason == FinishReason::Interrupted
        && c.state()?
            == (AgentState::Seeking {
                target: TargetKind::GatherFood,
            })
    {
        c.resources.tallies.gather_preempted += 1;
    }
    super::needs::finish(c)
}

pub(super) fn start(c: &mut EffectContext<'_>) -> EffectResult {
    let tile = c.tile_target()?;
    c.set_state(AgentState::Seeking {
        target: TargetKind::GatherFood,
    })?;
    c.attach(tile);
    Ok(EffectStatus::Running)
}
pub(super) fn forage_start(c: &mut EffectContext<'_>) -> EffectResult {
    let tile = c.tile_target()?;
    let result = start(c)?;
    *c.effects.claims.entry(tile).or_default() += 1;
    c.effects.claims_version += 1;
    c.effects.forage_targets.insert(c.entity, tile);
    c.effects.dirty.insert(EffectDomain::Claims);
    c.resources.tallies.forage_trips += 1;
    Ok(result)
}
pub(super) fn continue_running(c: &mut EffectContext<'_>) -> EffectResult {
    if c.interrupted {
        return super::needs::finish(c);
    }
    match c.state()? {
        AgentState::Seeking {
            target: TargetKind::GatherFood,
        } => {
            let key = (c.position.x, c.position.y);
            if c.resources.food_tiles.get(&key).is_some_and(|n| *n > 0) {
                c.set_state(AgentState::Consuming {
                    target: TargetKind::GatherFood,
                })?;
                c.clear();
            }
            Ok(EffectStatus::Running)
        }
        AgentState::Consuming {
            target: TargetKind::GatherFood,
        } => consume(c),
        AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } => {
            Ok(EffectStatus::Finished)
        }
    }
}

fn consume(c: &mut EffectContext<'_>) -> EffectResult {
    c.effects.dirty.insert(EffectDomain::FoodSources);
    c.effects.dirty.insert(EffectDomain::Needs);
    let resources = &mut *c.resources;
    let tick = resources.current_tick;
    let key = (c.position.x, c.position.y);
    let agent = c
        .world
        .get::<&Agent>(c.entity)
        .map_err(|_| EffectError::MissingOwner)?;
    let mut state_guard = c
        .world
        .get::<&mut AgentState>(c.entity)
        .map_err(|_| EffectError::MissingState)?;
    let state = &mut *state_guard;
    let mut inventory_opt = c.world.get::<&mut Inventory>(c.entity).ok();
    let temperament_guard = c.world.get::<&Temperament>(c.entity).ok();
    let temperament = temperament_guard.as_deref();
    let relations_rule = resources.rules.society.relations;
    c.effects.dirty.insert(EffectDomain::Relations);
    let member_of = &c.effects.members;
    let forage_spots = &c.effects.forage_spots;
    let quarrelled = &mut c.effects.quarrelled;
    let quarrel_entries = &mut c.effects.quarrel_entries;
    let food_version = &mut c.effects.food_version;
    let want = INVENTORY_CAPACITY
        .saturating_sub(inventory_opt.as_ref().map(|inv| inv.total()).unwrap_or(0));
    // Pass 20 (`game_v25`): a forage dispute. The rival is the
    // lowest-AgentId forager of another village at (or heading
    // to) this bush in the start-of-tick snapshot; each pair of
    // foragers once per day. A dispute costs the villages'
    // relation `dispute_loss`; a keyed draw makes it a quarrel
    // (another `quarrel_loss`): this pickup does not happen, nor
    // the rival's this tick.
    if let Some(rule) = relations_rule {
        if quarrelled.contains(&agent.id) {
            *state = AgentState::Idle;
            return Ok(EffectStatus::Finished);
        }
        let food_here = resources
            .food_tiles
            .get(&key)
            .is_some_and(|count| *count > 0);
        let mine = member_of.get(&agent.id).copied();
        let rival = mine.filter(|_| want > 0 && food_here).and_then(|mine| {
            forage_spots
                .get(&key)
                .and_then(|rows| rows.iter().find(|row| row.1 != mine && row.0 != agent.id))
                .map(|row| (mine, *row))
        });
        if let Some((mine, (rival_id, theirs, _rival_key, rival_ns, rival_ha))) = rival {
            let pair = SettlementRelations::key(mine, theirs);
            let foragers = (agent.id.min(rival_id), agent.id.max(rival_id));
            let replicate = resources.noise_replicate_for(tick);
            let relations = &mut resources.settlement_relations;
            if relations.pairs.contains_key(&pair) && relations.disputed.insert(foragers) {
                resources.tallies.forage_disputes += 1;
                // Pass 21: the pair's daily caps (0 = no cap; the
                // ledger is kept only under a cap). A dispute past
                // the dispute cap costs nothing; past the quarrel
                // cap no draw is made (a keyed draw consumes no
                // state, so skipping it is deterministic).
                let capped = rule.dispute_cap_per_pair_day > 0 || rule.quarrel_cap_per_pair_day > 0;
                let (costed_today, quarrels_today) = if capped {
                    relations.day_counts.get(&pair).copied().unwrap_or((0, 0))
                } else {
                    (0, 0)
                };
                let costed = rule.dispute_cap_per_pair_day == 0
                    || costed_today < rule.dispute_cap_per_pair_day;
                let may_quarrel = rule.quarrel_cap_per_pair_day == 0
                    || quarrels_today < rule.quarrel_cap_per_pair_day;
                let (ns, ha) = temperament.map_or((500, 500), |t| (t.ns, t.ha));
                let p =
                    ((i64::from(ns) + i64::from(rival_ns) - i64::from(ha) - i64::from(rival_ha)
                        + 1000)
                        / 4)
                    .clamp(0, 500);
                let quarrel = may_quarrel && {
                    let draw = keyed_below(
                        resources.world_seed,
                        STREAM_RELATIONS ^ c.node_random_key,
                        agent.id,
                        tick,
                        0,
                        replicate,
                        1000,
                    );
                    (draw as i64) < p
                };
                if !costed {
                    resources.tallies.disputes_uncosted += 1;
                }
                if !may_quarrel {
                    resources.tallies.quarrels_capped += 1;
                }
                if capped {
                    let counts = relations.day_counts.entry(pair).or_insert((0, 0));
                    counts.0 += u32::from(costed);
                    counts.1 += u32::from(quarrel);
                }
                let logged = relations
                    .dispute_days
                    .entry((relations.dispute_day, pair.0, pair.1))
                    .or_insert((0, 0));
                logged.0 += 1;
                logged.1 += u32::from(quarrel);
                let mut after = 0;
                let mut estranged = false;
                if let Some(relation) = relations.pairs.get_mut(&pair) {
                    if costed {
                        relation.add(-rule.dispute_loss);
                    }
                    if quarrel {
                        relation.add(-rule.quarrel_loss);
                    }
                    after = relation.value;
                    estranged = relation.estranged;
                }
                if quarrel {
                    resources.tallies.quarrels += 1;
                    resources.tallies.quarrels_while_estranged += u64::from(estranged);
                    quarrelled.insert(rival_id);
                    quarrel_entries.push(
                        ChronicleEntry::new(tick, ChronicleKind::Quarrel)
                            .subject(agent.id)
                            .other(Some(rival_id))
                            .settlement(Some(mine))
                            .settlement2(Some(theirs))
                            .value(i64::from(after)),
                    );
                    *state = AgentState::Idle;
                    return Ok(EffectStatus::Finished);
                }
            }
        }
    }
    if want > 0 {
        if let Some(counter) = resources.food_tiles.get_mut(&key) {
            *food_version += 1;
            if *counter == RESOURCE_SOURCE_INFINITE {
                // Infinite source: take freely, no decrement.
                if let Some(inv) = inventory_opt.as_mut() {
                    inv.add(ResourceKind::Food, want);
                }
            } else {
                let take = (*counter as u32).min(want);
                if take > 0 {
                    if let Some(inv) = inventory_opt.as_mut() {
                        let overflow = inv.add(ResourceKind::Food, take);
                        let actually = take - overflow; // overflow == 0 (want <= room)
                        *counter = counter.saturating_sub(actually as u8);
                        resources.tallies.food_harvested += u64::from(actually);
                        if *counter == 0 {
                            resources.food_tiles.remove(&key);
                        }
                    }
                }
            }
        }
    }
    *state = AgentState::Idle;
    Ok(EffectStatus::Finished)
}
