//! Priority 122, interval 1: react to approaching predators before civilian flight.
//! Reads: people, health, weapons, wildlife and defence policy.
//! Writes: ordinary draft commands, owned defender ids and quiet-window state.
//! Events: StewardDefence.
use crate::runtime::jobs::items::chebyshev;
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::people::{DraftOrder, Drafted};
use sim_core::colony::steward::{rules, Action};
use sim_core::colony::wildlife::Fighter;
use sim_core::colony::{Command, Tool};
use sim_core::components::{Agent, BodyHealth, Child, Position};
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeSet;

/// Identify the data-defined fast family without assuming a particular rule id.
pub fn is_rule(id: u16) -> bool {
    rules().is_ok_and(|data| {
        data.rules
            .iter()
            .any(|r| r.id == id && matches!(r.action, Action::Defence { .. }))
    })
}
/// Source envelopes replay in their original runtime phase.
pub fn source(command: &Command) -> bool {
    matches!(command, Command::Steward { rule, .. } if is_rule(*rule))
}
fn issue(world: &mut World, res: &mut SimResources, rule: u16, command: Command) {
    super::issue(
        world,
        res,
        Command::Steward {
            rule,
            command: Box::new(command),
        },
    );
}

/// Fast defence is independent of the slow basic-order planning cadence.
#[derive(Default)]
pub struct StewardDefenceSystem;
impl RuntimeSystem for StewardDefenceSystem {
    fn name(&self) -> &str {
        "StewardDefenceSystem"
    }
    fn priority(&self) -> u32 {
        122
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(_) = crate::runtime::wildlife::rule(res) else {
            return;
        };
        let Ok(data) = rules() else {
            return;
        };
        let Some(rule) = data
            .rules
            .iter()
            .find(|r| matches!(r.action, Action::Defence { .. }))
        else {
            return;
        };
        let Action::Defence {
            max_defenders,
            min_hp,
            release_tiles,
            release_ticks,
        } = rule.action
        else {
            return;
        };
        let Some(hearth) = crate::worldgen::colony::hearth(res) else {
            return;
        };
        let tick = res.current_tick;
        let drafted: BTreeSet<_> = world
            .query::<(&Agent, &Drafted)>()
            .iter()
            .map(|(_, (a, _))| a.id)
            .collect();
        res.steward
            .defence
            .owned_ids
            .retain(|id| drafted.contains(id));
        res.steward
            .defence
            .manual_until
            .retain(|_, until| tick < *until);
        let threats = crate::runtime::wildlife::flee::threats(res);
        let pack = threats
            .iter()
            .filter_map(|(id, _)| res.colony.wildlife.animals.get(id).and_then(|a| a.pack))
            .min()
            .or_else(|| threats.first().map(|(id, _)| *id));
        let near = threats
            .iter()
            .any(|(_, pos)| chebyshev(*pos, hearth) <= release_tiles);
        if near && pack.is_some() && pack == res.steward.defence.retired_pack {
            res.steward.defence.responded_pack = None;
        }
        if near || pack != res.steward.defence.retired_pack {
            res.steward.defence.retired_pack = None;
        }
        let owned = !res.steward.defence.owned_ids.is_empty();
        if near || !owned {
            res.steward.defence.quiet_since = None;
        } else {
            res.steward.defence.quiet_since.get_or_insert(tick);
        }
        let quiet = res
            .steward
            .defence
            .quiet_since
            .is_some_and(|start| tick.saturating_sub(start) >= release_ticks);
        let release =
            !res.steward.enabled || !res.steward.defence.enabled || threats.is_empty() || quiet;
        if quiet {
            res.steward.defence.retired_pack = pack;
        }
        if !res.steward.enabled || !res.steward.defence.enabled || threats.is_empty() {
            res.steward.defence.responded_pack = None;
        }
        let respond = !release
            && pack.is_some()
            && pack != res.steward.defence.retired_pack
            && pack != res.steward.defence.responded_pack;
        if respond {
            res.steward.defence.responded_pack = pack;
        }
        let before = res.steward.defence.owned_ids.clone();
        if res.steward.replay {
            let pending = std::mem::take(&mut res.steward.pending);
            for (at, command) in pending {
                if at <= tick && source(&command) {
                    super::issue(world, res, command);
                } else {
                    res.steward.pending.push((at, command));
                }
            }
        } else if release {
            for id in before.iter().copied() {
                issue(
                    world,
                    res,
                    rule.id,
                    Command::SetDrafted {
                        agent: id,
                        drafted: false,
                    },
                );
            }
        } else if pack.is_none() || res.steward.defence.retired_pack != pack {
            if respond {
                for _ in 0..max_defenders {
                    if res.steward.defence.owned_ids.len() >= usize::from(max_defenders) {
                        break;
                    }
                    let mut candidates = Vec::new();
                    for (_, (a, _, hp, tool, child, draft)) in world
                        .query::<(
                            &Agent,
                            &Position,
                            &BodyHealth,
                            Option<&Tool>,
                            Option<&Child>,
                            Option<&Drafted>,
                        )>()
                        .iter()
                    {
                        if child.is_some()
                            || draft.is_some()
                            || !hp.hp.is_finite()
                            || hp.hp < f64::from(min_hp)
                            || res.steward.defence.manual_until.contains_key(&a.id)
                        {
                            continue;
                        }
                        let held = tool.and_then(|t| {
                            t.item
                                .kind
                                .definition()
                                .weapon
                                .as_ref()
                                .and_then(|w| w.draft_rank)
                        });
                        let proposed = if held.is_some() {
                            Some(std::collections::BTreeMap::new())
                        } else {
                            super::arming::allocation(world, res, a.id)
                        };
                        if let Some(proposed) = proposed {
                            candidates.push((
                                super::arming::team_priority(world, res, a.id, &proposed, hearth),
                                a.id,
                            ));
                        }
                    }
                    let Some((_, agent)) = candidates.into_iter().min() else {
                        break;
                    };
                    issue(
                        world,
                        res,
                        rule.id,
                        Command::SetDrafted {
                            agent,
                            drafted: true,
                        },
                    );
                }
            }
            let mut rows: Vec<_> = world
                .query::<(&Agent, &Position, &Drafted)>()
                .iter()
                .filter(|(_, (a, _, _))| res.steward.defence.owned_ids.contains(&a.id))
                .map(|(_, (a, p, d))| (a.id, (p.x, p.y), d.order))
                .collect();
            rows.sort_by_key(|r| r.0);
            for (agent, pos, order) in rows {
                if let Some((animal, _)) = threats
                    .iter()
                    .min_by_key(|(id, at)| (chebyshev(pos, *at), *id))
                {
                    if order != DraftOrder::Attack(Fighter::Animal(*animal)) {
                        issue(
                            world,
                            res,
                            rule.id,
                            Command::DraftAttack {
                                agent,
                                animal: *animal,
                            },
                        );
                    }
                }
            }
        }
        let count = res.steward.defence.owned_ids.difference(&before).count() as u16;
        if count > 0 {
            res.steward.defence.last_count = count;
            res.steward.last_rule = Some(rule.id);
            res.steward.last_tick = tick;
            let mut entry = ChronicleEntry::new(tick, ChronicleKind::StewardDefence);
            entry.cause = u32::from(rule.id);
            entry.value = i64::from(count);
            res.chronicle.push(entry);
        }
        if res.steward.defence.owned_ids.is_empty() {
            res.steward.defence.quiet_since = None;
        } else if !near {
            res.steward.defence.quiet_since.get_or_insert(tick);
        }
    }
}

/// Apply the same batch allocation to live and replay source commands, retaining
/// ordinary DraftSystem movement and physical pickup for every assigned weapon.
pub fn assign_arming(world: &mut World, res: &SimResources, agent: u64) {
    let Some(assigned) = super::arming::allocation(world, res, agent) else {
        return;
    };
    let people: Vec<_> = world
        .query::<&Agent>()
        .iter()
        .filter_map(|(entity, a)| assigned.get(&a.id).map(|tile| (entity, *tile)))
        .collect();
    for (entity, tile) in people {
        if let Ok(mut draft) = world.get::<&mut Drafted>(entity) {
            draft.arming = Some(tile);
        }
    }
}
