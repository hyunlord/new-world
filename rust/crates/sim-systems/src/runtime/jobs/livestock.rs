//! Ordinary capture, recurring livestock yield and explicit slaughter admission.
//! Reads: selected tameable data, live animals, knowledge, pens and navigation.
//! Writes: ordinary jobs/reservations and accepted livestock command intents.
use hecs::{Entity, World};
use sim_core::colony::content::{AnimalMechanic, RecurringProduct, TameableDef};
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::knowledge::{bit, defs, Knowledge};
use sim_core::colony::livestock::{current_age, LivestockId, LivestockOrderCause};
use sim_core::colony::livestock_demographics::wild_demographics;
use sim_core::colony::{
    CancelReason, Command, CommandResult, ItemKind, Job, JobKind, JobStage, RejectReason,
    ReserveKey, WorkPriorities,
};
use sim_core::components::{Agent, AgentId, AgentState, Child, Position, SeekTarget, TargetKind};
use sim_engine::SimResources;
use std::collections::BTreeSet;

use super::items::chebyshev;
use crate::runtime::livestock::pens::{has_capture_slot, nearest_pen_with_room, pen_layout};
mod commit;
mod execute;
pub use execute::step;

type Tile = (u32, u32);
type JobFields = (JobKind, Option<Tile>, ItemKind, u16, Tile);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Read-only admitted wild source, physical pen and optional real tool stack.
pub struct CaptureOffer {
    /// Wild animal identity.
    pub wild: u32,
    /// Completed pen identity.
    pub pen: u32,
    /// Reachable physical gate.
    pub gate: Tile,
    /// Reserved only when the ordinary job starts fetching.
    pub from_tool: Option<Tile>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Read-only recurring-product target; serialized job kind remains compatible.
pub struct MilkOffer {
    /// Domesticated animal identity.
    pub animal: LivestockId,
    /// Reachable physical gate.
    pub gate: Tile,
    /// Selected authored recurring product.
    pub item: ItemKind,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Read-only target of an accepted slaughter command.
pub struct SlaughterOffer {
    /// Domesticated animal identity.
    pub animal: LivestockId,
    /// Reachable physical gate.
    pub gate: Tile,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Transient admission result; durable ownership uses the existing Job component.
pub enum Offer {
    /// Convert a real wild source.
    Capture(CaptureOffer),
    /// Harvest one ready milk or wool product.
    Milk(MilkOffer),
    /// Execute an explicit slaughter order.
    Slaughter(SlaughterOffer),
}

pub(super) fn day(res: &SimResources) -> u32 {
    u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX)
}
pub(super) fn knowledge_mask(name: &str) -> Option<sim_core::colony::knowledge::KnowledgeMask> {
    Some(bit(defs().id_of(name)?))
}
fn practises(practice: sim_core::colony::knowledge::KnowledgeMask, name: &str) -> bool {
    knowledge_mask(name).is_some_and(|m| m & practice == m)
}

/// Pure source admission, shared by ordinary jobs and a steward before its command.
pub fn capture_candidate(
    res: &SimResources,
    practice: sim_core::colony::knowledge::KnowledgeMask,
    region: u32,
    pos: Tile,
    radius: u32,
    require_order: bool,
) -> Option<CaptureOffer> {
    res.rules.colony?;
    res.colony
        .wildlife
        .animals
        .values()
        .filter_map(|a| {
            let def = a.kind.definition();
            let policy = def.tameable.as_ref()?;
            if a.hp == 0
                || def.status != ContentStatus::Connected
                || def.mechanic != AnimalMechanic::Grazer
                || !practises(practice, &policy.capture_knowledge)
                || (require_order != res.livestock.capture_orders.contains_key(&a.id))
                || res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Animal(a.id))
                || chebyshev(pos, a.pos) > radius.min(policy.capture_range_tiles)
                || !res.nav.reaches(region, a.pos)
                || !res.livestock.capture_ready(a.id, day(res))
                || wild_demographics(
                    res.world_seed,
                    a,
                    policy,
                    res.current_tick,
                    res.ticks_per_day,
                )
                .age_days
                    >= policy.kid_age_days
            {
                return None;
            }
            let (pen, gate) = nearest_pen_with_room(res, a.pos, region)?;
            let from_tool = if let Some(tool) = policy.capture_tool {
                Some(super::hunt::nearest_stack(
                    res,
                    pos,
                    policy.capture_range_tiles,
                    &[tool],
                    true,
                )?)
            } else {
                None
            };
            Some((
                (chebyshev(pos, a.pos), a.pos.1, a.pos.0, a.id),
                CaptureOffer {
                    wild: a.id,
                    pen,
                    gate,
                    from_tool,
                },
            ))
        })
        .min_by_key(|row| row.0)
        .map(|row| row.1)
}
/// Pick an ordered capturable kid in stable distance/tile/id order.
pub fn pick_capture(
    res: &SimResources,
    practice: sim_core::colony::knowledge::KnowledgeMask,
    region: u32,
    pos: Tile,
    radius: u32,
) -> Option<CaptureOffer> {
    capture_candidate(res, practice, region, pos, radius, true)
}
fn recurring_product(policy: &TameableDef, item: ItemKind) -> Option<(&RecurringProduct, bool)> {
    if let Some(milk) = policy
        .products
        .milk
        .as_ref()
        .filter(|milk| item == milk.item)
    {
        return Some((milk, false));
    }
    if let Some(wool) = policy
        .products
        .wool
        .as_ref()
        .filter(|wool| item == wool.item)
    {
        return Some((wool, true));
    }
    // Older saved milk jobs used the ordinary placeholder item.
    policy
        .products
        .milk
        .as_ref()
        .filter(|_| item == ItemKind::role("default_placeholder"))
        .map(|milk| (milk, false))
}

fn product_ready(
    res: &SimResources,
    animal: &sim_core::colony::livestock::Livestock,
    item: ItemKind,
) -> bool {
    let Some(policy) = animal.species.definition().tameable.as_ref() else {
        return false;
    };
    let Some((product, wool)) = recurring_product(policy, item) else {
        return false;
    };
    let today = day(res);
    if animal.hunger_days != 0
        || animal.created_day >= today
        || current_age(animal, res.current_tick, res.ticks_per_day) < policy.kid_age_days
        || res.livestock.slaughter_orders.contains_key(&animal.id)
    {
        return false;
    }
    let last_day = if wool {
        res.livestock
            .wool_last_day
            .get(&animal.id)
            .copied()
            .unwrap_or(animal.created_day)
    } else {
        if !animal.sex.female || !animal.milk_ready {
            return false;
        }
        animal.last_milk_day
    };
    today.saturating_sub(last_day) >= product.every_days
}

/// Pick a nourished adult's ready product using its selected knowledge policy.
pub fn pick_milk(
    res: &SimResources,
    practice: sim_core::colony::knowledge::KnowledgeMask,
    region: u32,
    pos: Tile,
    radius: u32,
) -> Option<MilkOffer> {
    res.livestock
        .animals
        .values()
        .filter_map(|a| {
            let p = a.species.definition().tameable.as_ref()?;
            let product = p
                .products
                .milk
                .iter()
                .chain(p.products.wool.iter())
                .find(|product| {
                    practises(practice, &product.requires_knowledge)
                        && product_ready(res, a, product.item)
                })?;
            if res
                .colony
                .reservations
                .contains_key(&ReserveKey::Livestock(a.id))
            {
                return None;
            }
            let pen = pen_layout(res, a.pen_site).ok()?;
            let gate = pen
                .gates
                .into_iter()
                .filter(|t| res.nav.reaches(region, *t) && chebyshev(pos, *t) <= radius)
                .min_by_key(|t| (chebyshev(pos, *t), t.1, t.0))?;
            Some((
                (chebyshev(pos, gate), gate.1, gate.0, a.id),
                MilkOffer {
                    animal: a.id,
                    gate,
                    item: product.item,
                },
            ))
        })
        .min_by_key(|row| row.0)
        .map(|row| row.1)
}
/// Pick a living, explicitly ordered slaughter target.
pub fn pick_slaughter(
    res: &SimResources,
    _practice: sim_core::colony::knowledge::KnowledgeMask,
    region: u32,
    pos: Tile,
    radius: u32,
) -> Option<SlaughterOffer> {
    res.livestock
        .animals
        .values()
        .filter_map(|a| {
            if !res.livestock.slaughter_orders.contains_key(&a.id)
                || res
                    .colony
                    .reservations
                    .contains_key(&ReserveKey::Livestock(a.id))
            {
                return None;
            }
            let pen = pen_layout(res, a.pen_site).ok()?;
            let gate = pen
                .gates
                .into_iter()
                .filter(|t| res.nav.reaches(region, *t) && chebyshev(pos, *t) <= radius)
                .min_by_key(|t| (chebyshev(pos, *t), t.1, t.0))?;
            Some((
                (chebyshev(pos, gate), gate.1, gate.0, a.id),
                SlaughterOffer { animal: a.id, gate },
            ))
        })
        .min_by_key(|row| row.0)
        .map(|row| row.1)
}

/// Install through the same reservation and seek-target seam as JobAssign.
pub fn start(world: &mut World, res: &mut SimResources, entity: Entity, offer: Offer) -> bool {
    if world.get::<&Job>(entity).is_ok()
        || world.get::<&sim_core::colony::Carry>(entity).is_ok()
        || world.get::<&Child>(entity).is_ok()
        || world
            .get::<&sim_core::colony::people::Drafted>(entity)
            .is_ok()
        || world
            .get::<&sim_core::colony::people::Fleeing>(entity)
            .is_ok()
    {
        return false;
    }
    let Some(id) = world.get::<&Agent>(entity).ok().map(|a| a.id) else {
        return false;
    };
    let Some(pos) = world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)) else {
        return false;
    };
    if super::assign::in_combat(res, id)
        || crate::runtime::mood::broken(world, res, entity)
        || !world.get::<&AgentState>(entity).is_ok_and(|s| {
            matches!(
                *s,
                AgentState::Idle
                    | AgentState::Seeking {
                        target: TargetKind::Activity
                    }
                    | AgentState::Consuming {
                        target: TargetKind::Activity
                    }
            )
        })
    {
        return false;
    }
    let practice = world
        .get::<&Knowledge>(entity)
        .map_or(sim_core::colony::knowledge::KnowledgeMask::ZERO, |k| {
            k.practice()
        });
    let region = res.nav.region_of(pos);
    let current = match offer {
        Offer::Capture(_) => pick_capture(res, practice, region, pos, u32::MAX).map(Offer::Capture),
        Offer::Milk(_) => pick_milk(res, practice, region, pos, u32::MAX).map(Offer::Milk),
        Offer::Slaughter(_) => {
            pick_slaughter(res, practice, region, pos, u32::MAX).map(Offer::Slaughter)
        }
    };
    if current != Some(offer) {
        return false;
    }
    let Some((kind, fetch, item, amount, target)) = job_fields(res, offer) else {
        return false;
    };
    let priority = world.get::<&WorkPriorities>(entity).map_or_else(
        |_| {
            WorkPriorities::defaults(world.get::<&sim_core::components::Elder>(entity).is_ok())
                .level(kind.work())
        },
        |p| p.level(kind.work()),
    );
    if priority == 0 {
        return false;
    }
    let now = res.current_tick;
    let job = Job {
        kind,
        stage: JobStage::ToSource,
        taken_at: now,
        amount,
        item,
        material: None,
        fetch,
        best_distance: chebyshev(pos, target),
        progress_tick: now,
    };
    if job
        .keys()
        .iter()
        .any(|k| res.colony.reservations.contains_key(k))
    {
        return false;
    }
    if world
        .insert(entity, (job, SeekTarget::new(target)))
        .is_err()
    {
        return false;
    }
    for key in job.keys() {
        super::cancel::reserve(res, key, id);
    }
    res.colony.tallies.jobs_taken += 1;
    *res.colony
        .tallies
        .taken_by
        .entry((id, kind.code()))
        .or_default() += 1;
    if let Ok(mut state) = world.get::<&mut AgentState>(entity) {
        *state = AgentState::Seeking {
            target: TargetKind::Job,
        };
    }
    crate::runtime::mind::job_origins::assigned(world, res, id, job);
    true
}
fn job_fields(res: &SimResources, offer: Offer) -> Option<JobFields> {
    let placeholder = ItemKind::role("default_placeholder");
    Some(match offer {
        Offer::Capture(o) => {
            let a = res.colony.wildlife.animals.get(&o.wild)?;
            let p = a.kind.definition().tameable.as_ref()?;
            (
                JobKind::Capture {
                    animal: o.wild,
                    pen: o.pen,
                },
                o.from_tool,
                p.capture_tool.unwrap_or(placeholder),
                u16::from(p.capture_tool.is_some()),
                o.from_tool.unwrap_or(a.pos),
            )
        }
        Offer::Milk(o) => (
            JobKind::Milk {
                animal: o.animal,
                at: o.gate,
            },
            None,
            o.item,
            0,
            o.gate,
        ),
        Offer::Slaughter(o) => (
            JobKind::Slaughter {
                animal: o.animal,
                at: o.gate,
            },
            None,
            placeholder,
            0,
            o.gate,
        ),
    })
}
/// Physical tool identity remains recognizable after a cancelled job is removed.
pub(super) fn capture_tool(kind: ItemKind) -> bool {
    sim_core::colony::wildlife::AnimalKind::all().any(|a| {
        a.definition()
            .tameable
            .as_ref()
            .is_some_and(|p| p.capture_tool == Some(kind))
    })
}
/// Resolve the current physical source, including an optional tool fetch.
pub fn target_tile(res: &SimResources, job: &Job) -> Option<Tile> {
    if let Some(from) = job.fetch {
        return Some(from);
    }
    match job.kind {
        JobKind::Capture { animal, .. } => res.colony.wildlife.animals.get(&animal).map(|a| a.pos),
        JobKind::Milk { at, .. } | JobKind::Slaughter { at, .. } => Some(at),
        _ => None,
    }
}
/// Revalidate continued ownership against real source and physical pen state.
pub fn target_exists(res: &SimResources, owner: AgentId, job: &Job) -> bool {
    match job.kind {
        JobKind::Capture { animal, pen } => {
            let Some(a) = res.colony.wildlife.animals.get(&animal) else {
                return false;
            };
            let Some(p) = a.kind.definition().tameable.as_ref() else {
                return false;
            };
            a.hp > 0
                && res.livestock.capture_orders.contains_key(&animal)
                && res.livestock.capture_ready(animal, day(res))
                && wild_demographics(res.world_seed, a, p, res.current_tick, res.ticks_per_day)
                    .age_days
                    < p.kid_age_days
                && pen_layout(res, pen).is_ok_and(|v| has_capture_slot(res, &v, Some(owner)))
                && job.fetch.is_none_or(|t| {
                    res.colony
                        .ground_items
                        .get(&t)
                        .is_some_and(|s| s.kind == job.item && s.count > 0)
                })
        }
        JobKind::Milk { animal, at } | JobKind::Slaughter { animal, at } => {
            res.livestock
                .animals
                .get(&animal)
                .is_some_and(|a| pen_layout(res, a.pen_site).is_ok_and(|p| p.gates.contains(&at)))
                && (!matches!(job.kind, JobKind::Slaughter { .. })
                    || res.livestock.slaughter_orders.contains_key(&animal))
        }
        _ => false,
    }
}
/// New designations need a real forthcoming command log record, including Steward wrapper.
pub(crate) fn designate(
    res: &mut SimResources,
    command: &Command,
    logged: bool,
    steward_rule: Option<u16>,
) -> CommandResult {
    if !logged {
        return CommandResult::rejected(RejectReason::Blocked);
    }
    let Ok(source_record) = u32::try_from(res.colony.command_log.len()) else {
        return CommandResult::rejected(RejectReason::Blocked);
    };
    let origin = LivestockOrderCause {
        source_record,
        steward_rule,
    };
    let accepted = match *command {
        Command::DesignateCapture { animal } => {
            let eligible = res.colony.wildlife.animals.get(&animal).is_some_and(|a| {
                let d = a.kind.definition();
                a.hp > 0
                    && d.status == ContentStatus::Connected
                    && d.mechanic == AnimalMechanic::Grazer
                    && d.tameable.as_ref().is_some_and(|p| {
                        wild_demographics(res.world_seed, a, p, res.current_tick, res.ticks_per_day)
                            .age_days
                            < p.kid_age_days
                            && res.livestock.capture_ready(animal, day(res))
                            && nearest_pen_with_room(res, a.pos, res.nav.region_of(a.pos)).is_some()
                    })
            });
            if !eligible || res.livestock.capture_orders.contains_key(&animal) {
                false
            } else {
                res.livestock.capture_orders.insert(animal, origin);
                true
            }
        }
        Command::DesignateSlaughter { animal } => {
            if !res.livestock.animals.contains_key(&animal)
                || res.livestock.slaughter_orders.contains_key(&animal)
            {
                false
            } else {
                res.livestock.slaughter_orders.insert(animal, origin);
                true
            }
        }
        _ => false,
    };
    if accepted {
        res.livestock.touch();
        CommandResult::ok(1)
    } else {
        CommandResult::rejected(RejectReason::Nothing)
    }
}
/// Cancel targets before lifecycle removes an animal, including blocked output jobs.
pub fn cancel_for_animal(world: &mut World, res: &mut SimResources, animal: LivestockId) {
    let mut held:Vec<_>=world.query::<(&Agent,&Job)>().iter().filter(|(_,(_,j))|matches!(j.kind,JobKind::Milk{animal:a,..}|JobKind::Slaughter{animal:a,..} if a==animal)).map(|(e,(a,_))|(a.id,e)).collect();
    held.sort_by_key(|r| r.0);
    for (_, e) in held {
        super::cancel::cancel(world, res, e, CancelReason::TargetGone);
    }
    res.livestock.slaughter_orders.remove(&animal);
}
pub(crate) fn cancel_area(
    world: &mut World,
    res: &mut SimResources,
    area: &BTreeSet<Tile>,
) -> usize {
    let captures: BTreeSet<_> = res
        .livestock
        .capture_orders
        .keys()
        .copied()
        .filter(|id| {
            res.colony
                .wildlife
                .animals
                .get(id)
                .is_some_and(|a| area.contains(&a.pos))
        })
        .collect();
    let targeted_jobs: BTreeSet<_> = world
        .query::<&Job>()
        .iter()
        .filter_map(|(_, j)| match j.kind {
            JobKind::Milk { animal, at } | JobKind::Slaughter { animal, at }
                if area.contains(&at) =>
            {
                Some(animal)
            }
            _ => None,
        })
        .collect();
    // Withdrawal uses retained target geometry even when physical admission is broken.
    let livestock: BTreeSet<_> = res
        .livestock
        .animals
        .values()
        .filter(|a| {
            targeted_jobs.contains(&a.id)
                || res
                    .colony
                    .buildings
                    .sites
                    .get(&a.pen_site)
                    .is_some_and(|site| {
                        area.contains(&site.anchor)
                            || site.pieces.iter().any(|piece| {
                                piece.kind.definition().completion
                                    == sim_core::colony::content::CompletionCategory::Door
                                    && !piece.kind.definition().blocks_livestock
                                    && area.iter().any(|t| {
                                        t.0 >= piece.tile.0
                                            && t.1 >= piece.tile.1
                                            && t.0 - piece.tile.0 < piece.size.0
                                            && t.1 - piece.tile.1 < piece.size.1
                                    })
                            })
                    })
        })
        .map(|a| a.id)
        .collect();
    let mut held: Vec<_> = world
        .query::<(&Agent, &Job)>()
        .iter()
        .filter(|(_, (_, j))| match j.kind {
            JobKind::Capture { animal, .. } => captures.contains(&animal),
            JobKind::Milk { animal, .. } | JobKind::Slaughter { animal, .. } => {
                livestock.contains(&animal)
            }
            _ => false,
        })
        .map(|(e, (a, _))| (a.id, e))
        .collect();
    held.sort_by_key(|r| r.0);
    let cancelled_jobs = held.len();
    for (_, e) in held {
        super::cancel::cancel(world, res, e, CancelReason::Command);
    }
    let before = res.livestock.capture_orders.len() + res.livestock.slaughter_orders.len();
    res.livestock
        .capture_orders
        .retain(|id, _| !captures.contains(id));
    res.livestock
        .slaughter_orders
        .retain(|id, _| !livestock.contains(id));
    let removed =
        before - res.livestock.capture_orders.len() - res.livestock.slaughter_orders.len();
    if removed > 0 {
        res.livestock.touch();
    }
    // Automatic milk has no command intent: cancelling its job still changes the world.
    removed + cancelled_jobs
}

#[cfg(test)]
mod tests;
