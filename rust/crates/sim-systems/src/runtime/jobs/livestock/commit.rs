//! Serial physical commits: no animal/tool/readiness mutation before all admission succeeds.
use super::*;
use crate::runtime::food::items;
use crate::runtime::livestock::{emit_event, next_event_index};
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::livestock::{
    CaptureAttempt, KnowledgeWitness, Livestock, LivestockEvent, LivestockEventWitness,
    LivestockYield,
};
use sim_core::colony::livestock_demographics::capture_draw;
use sim_core::colony::{Carry, ItemSpec};

pub(super) enum Outcome {
    Complete,
    Retry,
    Cancel,
}
fn knowledge(world: &World, entity: Entity, name: &str) -> Option<KnowledgeWitness> {
    let id = defs().id_of(name)?;
    let required = defs().threshold(id);
    let proficiency = world.get::<&Knowledge>(entity).ok()?.proficiency(id);
    (proficiency >= required).then_some(KnowledgeWitness {
        knowledge: id,
        proficiency,
        required,
    })
}
pub(super) fn complete(
    world: &mut World,
    res: &mut SimResources,
    owner: AgentId,
    entity: Entity,
    job: &Job,
) -> Outcome {
    if next_event_index(res).is_none() || !target_exists(res, owner, job) {
        return Outcome::Cancel;
    }
    let Some(pos) = world.get::<&Position>(entity).ok().map(|p| (p.x, p.y)) else {
        return Outcome::Cancel;
    };
    let Some(goal) = target_tile(res, job) else {
        return Outcome::Cancel;
    };
    if job.fetch.is_some()
        || (if matches!(job.kind, JobKind::Capture { .. }) {
            chebyshev(pos, goal) > 1
        } else {
            !res.nav.arrived(pos, goal)
        })
    {
        return Outcome::Cancel;
    }
    match job.kind {
        JobKind::Capture { animal, pen } => capture(world, res, owner, entity, animal, pen),
        JobKind::Milk { animal, at } => milk(world, res, owner, entity, animal, at),
        JobKind::Slaughter { animal, at } => slaughter(world, res, owner, animal, at),
        _ => Outcome::Cancel,
    }
}
fn capture(
    world: &mut World,
    res: &mut SimResources,
    owner: AgentId,
    entity: Entity,
    wild: u32,
    pen: u32,
) -> Outcome {
    let Some(source) = res.colony.wildlife.animals.get(&wild).cloned() else {
        return Outcome::Cancel;
    };
    let Some(policy) = source.kind.definition().tameable.as_ref() else {
        return Outcome::Cancel;
    };
    let Some(knows) = knowledge(world, entity, &policy.capture_knowledge) else {
        return Outcome::Cancel;
    };
    let Some(origin) = res.livestock.capture_orders.get(&wild).copied() else {
        return Outcome::Cancel;
    };
    let profile = wild_demographics(
        res.world_seed,
        &source,
        policy,
        res.current_tick,
        res.ticks_per_day,
    );
    if profile.age_days >= policy.kid_age_days {
        return Outcome::Cancel;
    }
    let Some((id, next_id)) = res.livestock.checked_next_id() else {
        return Outcome::Cancel;
    };
    let Some(entry) = next_event_index(res) else {
        return Outcome::Cancel;
    };
    let tool = if let Some(kind) = policy.capture_tool {
        let Some(carry) = world
            .get::<&Carry>(entity)
            .ok()
            .map(|c| *c)
            .filter(|c| c.kind == kind && c.count == 1)
        else {
            return Outcome::Cancel;
        };
        Some(carry.spec())
    } else {
        None
    };
    let today = day(res);
    let Some(attempt) = res.livestock.next_capture_attempt(wild, today) else {
        return Outcome::Cancel;
    };
    if capture_draw(res.world_seed, source.kind, wild, attempt) >= policy.capture_success_permille {
        res.livestock.capture_attempts.insert(
            wild,
            CaptureAttempt {
                completed_attempts: attempt,
                not_before_day: today.saturating_add(policy.capture_retry_days),
            },
        );
        res.livestock.touch();
        return Outcome::Cancel;
    }
    // All following writes are infallible on this serial, fully admitted commit.
    if let Some(spec) = tool {
        let _ = world.remove_one::<Carry>(entity);
        res.colony.item_ledger.consume(spec.kind, 1);
    }
    res.colony.wildlife.animals.remove(&wild);
    res.colony.wildlife.marks.remove(&wild);
    res.colony.wildlife.version = res.colony.wildlife.version.saturating_add(1);
    res.livestock.capture_orders.remove(&wild);
    res.livestock.capture_attempts.remove(&wild);
    res.livestock.next_id = next_id;
    res.livestock.animals.insert(
        id,
        Livestock {
            id,
            species: source.kind,
            age_days: profile.age_days,
            last_age_tick: profile.last_age_tick,
            sex: profile.sex,
            hunger_days: 0,
            pen_site: pen,
            milk_ready: false,
            last_milk_day: today,
            last_bred_day: today,
            created_day: today,
            creation_entry: entry,
        },
    );
    res.livestock.first_capture.get_or_insert(id);
    emit_event(
        res,
        ChronicleEntry::new(res.current_tick, ChronicleKind::LivestockCaptured).subject(owner),
        LivestockEventWitness {
            livestock: id,
            species: source.kind,
            pen_site: pen,
            event: LivestockEvent::Captured {
                wild,
                attempt,
                knowledge: knows,
                tool,
                origin,
            },
        },
    );
    crate::runtime::knowledge::used(world, res, entity, bit(knows.knowledge));
    Outcome::Complete
}
fn milk(
    world: &mut World,
    res: &mut SimResources,
    owner: AgentId,
    entity: Entity,
    id: LivestockId,
    at: Tile,
) -> Outcome {
    let Some(animal) = res.livestock.animals.get(&id).cloned() else {
        return Outcome::Cancel;
    };
    let Some(policy) = animal.species.definition().tameable.as_ref() else {
        return Outcome::Cancel;
    };
    let Some(milk) = policy.products.milk.as_ref() else {
        return Outcome::Cancel;
    };
    let Some(knows) = knowledge(world, entity, &milk.requires_knowledge) else {
        return Outcome::Cancel;
    };
    let today = day(res);
    if !animal.sex.female
        || !animal.milk_ready
        || animal.hunger_days != 0
        || animal.created_day >= today
        || today.saturating_sub(animal.last_milk_day) < milk.every_days
        || current_age(&animal, res.current_tick, res.ticks_per_day) < policy.kid_age_days
        || res.livestock.slaughter_orders.contains_key(&id)
    {
        return Outcome::Cancel;
    }
    let product = LivestockYield {
        item: ItemSpec::raw(milk.item, None),
        count: 1,
    };
    let Ok(batch) = items::new_batch(product.item, product.count, today) else {
        return Outcome::Cancel;
    };
    if items::create_ground_batches(res, at, &[batch], today, items::FoodSource::Harvest).is_err() {
        return Outcome::Retry;
    }
    if let Some(a) = res.livestock.animals.get_mut(&id) {
        a.milk_ready = false;
        a.last_milk_day = today;
    }
    emit_event(
        res,
        ChronicleEntry::new(res.current_tick, ChronicleKind::LivestockMilk).subject(owner),
        LivestockEventWitness {
            livestock: id,
            species: animal.species,
            pen_site: animal.pen_site,
            event: LivestockEvent::Milk {
                knowledge: knows,
                product,
            },
        },
    );
    crate::runtime::knowledge::used(world, res, entity, bit(knows.knowledge));
    Outcome::Complete
}
fn slaughter(
    _world: &mut World,
    res: &mut SimResources,
    owner: AgentId,
    id: LivestockId,
    at: Tile,
) -> Outcome {
    let Some(animal) = res.livestock.animals.get(&id).cloned() else {
        return Outcome::Cancel;
    };
    let Some(policy) = animal.species.definition().tameable.as_ref() else {
        return Outcome::Cancel;
    };
    let Some(origin) = res.livestock.slaughter_orders.get(&id).copied() else {
        return Outcome::Cancel;
    };
    let products: Vec<_> = [policy.products.meat, policy.products.hide]
        .into_iter()
        .map(|(kind, count)| LivestockYield {
            item: ItemSpec::raw(kind, None),
            count,
        })
        .collect();
    let today = day(res);
    let Ok(batches) = products
        .iter()
        .map(|p| items::new_batch(p.item, p.count, today))
        .collect::<Result<Vec<_>, _>>()
    else {
        return Outcome::Cancel;
    };
    if items::create_ground_batches(res, at, &batches, today, items::FoodSource::Harvest).is_err() {
        return Outcome::Retry;
    }
    res.livestock.animals.remove(&id);
    res.livestock.slaughter_orders.remove(&id);
    emit_event(
        res,
        ChronicleEntry::new(res.current_tick, ChronicleKind::LivestockSlaughtered).subject(owner),
        LivestockEventWitness {
            livestock: id,
            species: animal.species,
            pen_site: animal.pen_site,
            event: LivestockEvent::Slaughter { origin, products },
        },
    );
    Outcome::Complete
}
