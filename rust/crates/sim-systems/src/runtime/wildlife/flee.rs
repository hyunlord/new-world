//! FleeSystem: undrafted people near a hunting wolf drop their work and run for a roofed room, else the hearth, else away from the wolf.
//!
//! Priority: 123 (`FLEE_PRIORITY`). Interval: 3 ticks. Gate: `WildlifeRule::flee`.
//! Reads: `Agent`, `Position`, `Drafted`, `Fleeing`, `Hunger`, `Thirst`, `Job`; `SimResources.colony` (wildlife, rooms), `nav`, `building_kinds` (the hearth), `rules`
//! Writes: `Fleeing` (inserted, renewed, removed), `AgentState`, `SeekTarget`, `Job` and `Activity` (a flight cancels them); `SimResources.colony` (reservations and ground items through the cancel, wildlife tallies `flights`), `spatial.agents`
//! Events: none
//!
//! M5.5 (ADR 0028 amendment; goal §3.1.2). Every 3 ticks, for the wolves that
//! hunt or bite (`Hunt`, `Attack`; a retreating pack is no threat), the people
//! within `flee_tiles` of one (the agents' spatial index, chunk rings around
//! each wolf; at most 4 wolves) are handled in `AgentId` order:
//! - a drafted person, or one at a critical hunger or thirst (the draft's
//!   `critical_preemption`), is left alone;
//! - someone already fleeing has the flight renewed (`until` = now +
//!   `flee_ticks`); on reaching the destination they hold there;
//! - someone standing in a roofed room is safe (wolves look only outside) and
//!   stays;
//! - anyone else begins a flight: the job is cancelled (`CancelReason::Needs`,
//!   the flight is a survival choice), and the person walks (`Seeking { Job }`
//!   with a `SeekTarget`, as a drafted person walks) to the nearest tile of a
//!   roofed room they can reach, else to the hearth, else to the tile
//!   `flee_tiles` steps away from the wolf.
//!
//! A flight whose `until` has passed ends: `Fleeing` and the `SeekTarget` are
//! removed and the person is `Idle`. The decision seam (`jobs::seam::holders`)
//! counts fleeing people among those that may not yield, so the decision system
//! skips them (`decision/system.rs` is not changed). A wolf next to a fleeing
//! person bites as before; the person's blows back are `flee_defence_permille`
//! of the weapon's (`hunt::defence_damage`).
use hecs::{Entity, World};
use sim_core::colony::people::{Drafted, Fleeing};
use sim_core::colony::wildlife::AnimalState;
use sim_core::colony::CancelReason;
use sim_core::components::{
    Activity, Agent, AgentId, AgentState, Hunger, Position, SeekTarget, TargetKind, Thirst,
};
use sim_core::rules::WildlifeRule;
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::jobs::items::chebyshev;

/// Priority of [`FleeSystem`] (before the decision, 125).
pub const FLEE_PRIORITY: u32 = 123;

const WALK: AgentState = AgentState::Seeking {
    target: TargetKind::Job,
};
const HOLD: AgentState = AgentState::Consuming {
    target: TargetKind::Job,
};

/// The rule when flight is on.
fn rule(res: &SimResources) -> Option<WildlifeRule> {
    super::rule(res).filter(|r| r.flee)
}

/// The wolves that threaten: hunting or biting (id, tile), in id order.
pub fn threats(res: &SimResources) -> Vec<(u32, (u32, u32))> {
    res.colony
        .wildlife
        .wolves()
        .filter(|a| matches!(a.state, AnimalState::Hunt | AnimalState::Attack { .. }))
        .map(|a| (a.id, a.pos))
        .collect()
}

/// Where a person at `pos` runs from a wolf at `wolf`: the nearest reachable tile of a roofed room
/// (`(distance, y, x)`), else the hearth, else `reach` steps away from the wolf.
pub fn refuge(res: &SimResources, pos: (u32, u32), wolf: (u32, u32), reach: u32) -> (u32, u32) {
    let region = res.nav.region_of(pos);
    let room = res
        .colony
        .rooms
        .values()
        .filter(|r| r.roofed)
        .flat_map(|r| r.tiles.iter().copied())
        .filter(|t| res.nav.passable(*t) && res.nav.reaches(region, *t))
        .min_by_key(|t| (chebyshev(pos, *t), t.1, t.0));
    if let Some(t) = room {
        return t;
    }
    if let Some(h) = crate::worldgen::colony::hearth(res) {
        return h;
    }
    let mut at = pos;
    for _ in 0..reach {
        match super::step_rel(res, at, wolf, false, |t| res.nav.passable(t)) {
            Some(next) => at = next,
            None => break,
        }
    }
    at
}

/// Whether someone at `pos` has reached `to` (on it, or next to it when it cannot be stood on: the hearth).
fn arrived(res: &SimResources, pos: (u32, u32), to: (u32, u32)) -> bool {
    pos == to || (!res.nav.passable(to) && chebyshev(pos, to) <= 1)
}

/// End `entity`'s flight: it is `Idle` again.
pub fn end_flight(world: &mut World, entity: Entity) {
    if world.remove_one::<Fleeing>(entity).is_ok() {
        let _ = world.remove_one::<SeekTarget>(entity);
        if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
            *state = AgentState::Idle;
        }
    }
}

/// One person near a wolf this run: id, entity, tile, the nearest wolf `(id, tile)`.
type Near = (AgentId, Entity, (u32, u32), (u32, (u32, u32)));

/// See the module header.
#[derive(Debug, Default)]
pub struct FleeSystem;

impl RuntimeSystem for FleeSystem {
    fn name(&self) -> &str {
        "FleeSystem"
    }

    fn priority(&self) -> u32 {
        FLEE_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        3
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = rule(res) else {
            return;
        };
        let now = res.current_tick;
        let wolves = threats(res);
        let mut near: Vec<Near> = Vec::new();
        if !wolves.is_empty() {
            let positions = || -> Vec<((u32, u32), AgentId)> {
                world
                    .query::<(&Agent, &Position)>()
                    .iter()
                    .map(|(_, (a, p))| ((p.x, p.y), a.id))
                    .collect()
            };
            let index = res.spatial.agents_at(now, positions);
            let chunks = rule.flee_tiles / sim_core::spatial::CHUNK + 1;
            let mut nearest: std::collections::BTreeMap<AgentId, (u32, u32, (u32, u32))> =
                std::collections::BTreeMap::new();
            for &(wid, wpos) in &wolves {
                for r in 0..=chunks {
                    for &(tile, id) in index.ring(wpos, r) {
                        let d = chebyshev(tile, wpos);
                        if d > rule.flee_tiles {
                            continue;
                        }
                        let e = nearest.entry(id).or_insert((d, wid, wpos));
                        if (d, wid) < (e.0, e.1) {
                            *e = (d, wid, wpos);
                        }
                    }
                }
            }
            if !nearest.is_empty() {
                let mut entity_of: std::collections::BTreeMap<AgentId, (Entity, (u32, u32))> =
                    world
                        .query::<(&Agent, &Position)>()
                        .iter()
                        .filter(|(_, (a, _))| nearest.contains_key(&a.id))
                        .map(|(e, (a, p))| (a.id, (e, (p.x, p.y))))
                        .collect();
                for (id, (_, wid, wpos)) in nearest {
                    if let Some((e, pos)) = entity_of.remove(&id) {
                        near.push((id, e, pos, (wid, wpos)));
                    }
                }
            }
        }
        let critical = res
            .rules
            .agent
            .critical_preemption
            .unwrap_or(crate::runtime::decision::needs::CRITICAL_NEED);
        for (_, entity, pos, (wid, wpos)) in near {
            if world.get::<&Drafted>(entity).is_ok() {
                continue;
            }
            let need = world
                .get::<&Hunger>(entity)
                .map_or(0.0, |h| f64::from(h.value))
                .max(world.get::<&Thirst>(entity).map_or(0.0, |t| t.value));
            if need >= critical {
                continue;
            }
            if let Ok(flight) = world.get::<&Fleeing>(entity).map(|f| *f) {
                if let Ok(mut f) = world.get::<&mut Fleeing>(entity) {
                    f.until = now + rule.flee_ticks;
                }
                if arrived(res, pos, flight.to) {
                    let _ = world.remove_one::<SeekTarget>(entity);
                    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                        *state = HOLD;
                    }
                }
                continue;
            }
            if crate::runtime::rooms::roofed(res, pos) {
                continue;
            }
            if world.get::<&sim_core::colony::Job>(entity).is_ok() {
                crate::runtime::jobs::cancel(world, res, entity, CancelReason::Needs);
            }
            let _ = world.remove_one::<Activity>(entity);
            let to = refuge(res, pos, wpos, rule.flee_tiles);
            let _ = world.insert_one(
                entity,
                Fleeing {
                    until: now + rule.flee_ticks,
                    from: wid,
                    to,
                    since: now,
                },
            );
            if arrived(res, pos, to) {
                let _ = world.remove_one::<SeekTarget>(entity);
                if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                    *state = HOLD;
                }
            } else {
                let _ = world.insert_one(entity, SeekTarget::new(to));
                if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
                    *state = WALK;
                }
            }
            res.colony.wildlife.tallies.flights += 1;
        }
        // Flights not renewed end (in `AgentId` order: structural changes after the query).
        let mut over: Vec<(AgentId, Entity)> = world
            .query::<(&Agent, &Fleeing)>()
            .iter()
            .filter(|(_, (_, f))| f.until <= now)
            .map(|(e, (a, _))| (a.id, e))
            .collect();
        over.sort_by_key(|row| row.0);
        for (_, e) in over {
            end_flight(world, e);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata() {
        let s = FleeSystem;
        assert_eq!(
            (s.name(), s.priority(), s.tick_interval()),
            ("FleeSystem", 123, 3)
        );
    }
}
