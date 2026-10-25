//! DraftSystem: walks drafted people to their orders, starts their attacks, arms them, and undrafts anyone at a critical need.
//!
//! Priority: 121 (`DRAFT_PRIORITY`). Interval: 1 tick. Gate: `ColonyRule::wildlife`.
//! Reads: `Agent`, `Position`, `Drafted`, `Tool`, `Hunger`, `Thirst`; `SimResources.colony` (wildlife, fights, ground items), `nav`, `rules`
//! Writes: `Drafted` (order, arming; removed at a critical need), `AgentState`, `SeekTarget`, `Tool` (picked up); `SimResources.colony` (fights, reservations, ground items, wildlife tallies, story cost), `chronicle`
//! Events: chronicle `Drafted` (cause 2: undrafted itself at a critical need)
//!
//! M5 (ADR 0028). A drafted person holds no job and the decision system skips
//! them (the job seam counts them among the holders that may not yield, so
//! the decision's cascade never runs for them; `decision/system.rs` is not
//! changed). Needs still tick. Each tick, in `AgentId` order: a critical hunger
//! or thirst undrafts the person (they go eat or drink; chronicle `Drafted`
//! cause 2, the UI's toast); an arming stop is walked to and its weapon picked
//! up; then the order — hold (`Consuming { Job }`), move to a tile
//! (`Seeking { Job }` with a `SeekTarget` until there), or attack an animal or a
//! person (walk next to it, then a fight `Person → target` with the held
//! weapon's damage, resolved by `CombatSystem`). While a Beasts pressure is
//! active every drafted person-tick is part of its cost (work lost).
use hecs::{Entity, World};
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::people::{DraftOrder, Drafted};
use sim_core::colony::story::PressureKind;
use sim_core::colony::wildlife::{Fight, Fighter};
use sim_core::colony::{CancelReason, Tool};
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Position, SeekTarget, TargetKind, Thirst,
};
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::jobs::items::chebyshev;

/// Priority of [`DraftSystem`].
pub const DRAFT_PRIORITY: u32 = 121;

const HOLD: AgentState = AgentState::Consuming {
    target: TargetKind::Job,
};
const WALK: AgentState = AgentState::Seeking {
    target: TargetKind::Job,
};

fn set_state(world: &mut World, entity: Entity, next: AgentState) {
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = next;
    }
}

fn seek(world: &mut World, entity: Entity, tile: (u32, u32)) {
    let same = world
        .get::<&SeekTarget>(entity)
        .is_ok_and(|s| s.tile == tile);
    if !same {
        let _ = world.insert_one(entity, SeekTarget::new(tile));
    }
    set_state(world, entity, WALK);
}

fn hold(world: &mut World, entity: Entity) {
    let _ = world.remove_one::<SeekTarget>(entity);
    set_state(world, entity, HOLD);
}

/// Whether `entity` holds a weapon (a spear or an axe).
pub fn armed(world: &World, entity: Entity) -> bool {
    world.get::<&Tool>(entity).is_ok_and(|t| {
        t.item
            .kind
            .definition()
            .weapon
            .as_ref()
            .is_some_and(|w| w.draft_rank.is_some())
    })
}

/// Draft `entity` (`id`): its job is cancelled, it holds where it stands, and an
/// unarmed person first fetches the nearest spear or axe within `arm_reach`
/// (not reserved: a person without a job holds no reservation; if someone else
/// takes it first, the drafted person goes on unarmed).
pub fn draft(world: &mut World, res: &mut SimResources, entity: Entity, id: AgentId) {
    let Some(rule) = super::rule(res) else {
        return;
    };
    if world.get::<&Drafted>(entity).is_ok() {
        return;
    }
    crate::runtime::jobs::cancel(world, res, entity, CancelReason::Command);
    let _ = world.remove_one::<sim_core::components::Activity>(entity);
    // M5.5: a drafted person stops running and takes orders.
    super::flee::end_flight(world, entity);
    let pos = world
        .get::<&Position>(entity)
        .map_or((0, 0), |p| (p.x, p.y));
    // Unarmed (no tool, or a tool that is no weapon, such as a knife; review m3) → fetch one.
    let arming = (!armed(world, entity))
        .then(|| {
            crate::runtime::jobs::hunt::nearest_stack(
                res,
                pos,
                rule.arm_reach,
                &crate::runtime::jobs::hunt::draft_weapons(),
                false,
            )
        })
        .flatten();
    let _ = world.insert_one(
        entity,
        Drafted {
            since: res.current_tick,
            order: DraftOrder::Hold,
            arming,
        },
    );
    hold(world, entity);
    res.colony.wildlife.tallies.drafts += 1;
    res.chronicle.push(
        ChronicleEntry::new(res.current_tick, ChronicleKind::Drafted)
            .subject(id)
            .cause(0),
    );
}

/// Release `entity` (`id`) from the draft (`cause` 1 the player, 2 a critical need):
/// its attacks stop, its arming stop is released, and it is `Idle`.
pub fn undraft(world: &mut World, res: &mut SimResources, entity: Entity, id: AgentId, cause: u8) {
    let Ok(d) = world.remove_one::<Drafted>(entity) else {
        return;
    };
    let _ = d;
    super::fight::drop_fights_of_attacker(res, Fighter::Person(id));
    let _ = world.remove_one::<SeekTarget>(entity);
    set_state(world, entity, AgentState::Idle);
    if cause == 2 {
        res.colony.wildlife.tallies.auto_undrafts += 1;
    }
    res.chronicle.push(
        ChronicleEntry::new(res.current_tick, ChronicleKind::Drafted)
            .subject(id)
            .cause(u32::from(cause)),
    );
}

/// Set a drafted person's order (no-op for anyone not drafted); `true` when set.
pub fn order(world: &mut World, entity: Entity, next: DraftOrder) -> bool {
    match world.get::<&mut Drafted>(entity) {
        Ok(mut d) => {
            d.order = next;
            true
        }
        Err(_) => false,
    }
}

/// Where fighter `f` is.
pub fn where_is(world: &World, res: &SimResources, f: Fighter) -> Option<(u32, u32)> {
    match f {
        Fighter::Person(id) => world
            .query::<(&Agent, &Position)>()
            .iter()
            .find(|(_, (a, _))| a.id == id)
            .map(|(_, (_, p))| (p.x, p.y)),
        Fighter::Animal(id) => res.colony.wildlife.animals.get(&id).map(|a| a.pos),
    }
}

/// One drafted person this tick: id, entity, draft, tile, the higher of hunger and thirst.
type DraftRow = (AgentId, Entity, Drafted, (u32, u32), f64);

/// See the module header.
#[derive(Debug, Default)]
pub struct DraftSystem;

impl RuntimeSystem for DraftSystem {
    fn name(&self) -> &str {
        "DraftSystem"
    }

    fn priority(&self) -> u32 {
        DRAFT_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = super::rule(res) else {
            return;
        };
        let mut rows: Vec<DraftRow> = world
            .query::<(
                &Agent,
                &Drafted,
                &Position,
                Option<&Hunger>,
                Option<&Thirst>,
            )>()
            .iter()
            .map(|(e, (a, d, p, h, t))| {
                let need = f64::from(h.map_or(0.0, |h| h.value)).max(t.map_or(0.0, |t| t.value));
                (a.id, e, *d, (p.x, p.y), need)
            })
            .collect();
        if rows.is_empty() {
            return;
        }
        rows.sort_by_key(|row| row.0);
        let critical = res
            .rules
            .agent
            .critical_preemption
            .unwrap_or(crate::runtime::decision::needs::CRITICAL_NEED);
        let beasts = res.colony.story.is_active(PressureKind::Beasts);
        for (id, entity, d, pos, need) in rows {
            if need >= critical {
                undraft(world, res, entity, id, 2);
                continue;
            }
            if beasts {
                crate::runtime::story::add_cost(res, PressureKind::Beasts, 0, 1);
            }
            if let Some(tile) = d.arming {
                if res.nav.arrived(pos, tile) || pos == tile {
                    crate::runtime::jobs::hunt::pick_up_tool(
                        world,
                        res,
                        entity,
                        tile,
                        pos,
                        None,
                        &crate::runtime::jobs::hunt::draft_weapons(),
                    );
                    if let Ok(mut held) = world.get::<&mut Drafted>(entity) {
                        held.arming = None;
                    }
                } else if res.colony.ground_items.get(&tile).is_some_and(|s| {
                    s.kind
                        .definition()
                        .weapon
                        .as_ref()
                        .is_some_and(|w| w.draft_rank.is_some())
                }) {
                    seek(world, entity, tile);
                    continue;
                } else if let Ok(mut held) = world.get::<&mut Drafted>(entity) {
                    held.arming = None;
                }
            }
            match d.order {
                DraftOrder::Hold => hold(world, entity),
                DraftOrder::MoveTo(tile) => {
                    if pos == tile || res.nav.arrived(pos, tile) {
                        hold(world, entity);
                    } else {
                        seek(world, entity, tile);
                    }
                }
                DraftOrder::Attack(target) => {
                    let Some(at) = where_is(world, res, target) else {
                        order(world, entity, DraftOrder::Hold);
                        super::fight::drop_fights_of_attacker(res, Fighter::Person(id));
                        hold(world, entity);
                        continue;
                    };
                    let me = Fighter::Person(id);
                    res.colony
                        .fights
                        .retain(|(a, t), _| *a != me || *t == target);
                    if chebyshev(pos, at) <= 1 {
                        hold(world, entity);
                        let damage =
                            crate::runtime::jobs::hunt::weapon_damage(world, &rule, entity);
                        res.colony.fights.entry((me, target)).or_insert(Fight {
                            progress: 0,
                            ticks: rule.hit_ticks,
                            damage,
                            hits_left: 0,
                            quarrel: None,
                        });
                    } else {
                        seek(world, entity, at);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata() {
        let s = DraftSystem;
        assert_eq!(
            (s.name(), s.priority(), s.tick_interval()),
            ("DraftSystem", 121, 1)
        );
    }
}
