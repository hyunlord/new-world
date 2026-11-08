//! Save-boundary validation of live ownership and retained immutable livestock facts.
use hecs::World;
use sim_core::chronicle::{CauseRef, ChronicleKind};
use sim_core::colony::content::{installed_or_bundled, TameableDef};
use sim_core::colony::knowledge::ContentStatus;
use sim_core::colony::livestock::{
    KnowledgeWitness, LivestockEvent, LivestockEventWitness, LivestockOrderCause,
};
use sim_core::colony::{Carry, Command, Job, JobKind, ReserveKey};
use sim_core::components::Agent;
use sim_engine::SimResources;
use std::collections::{BTreeMap, BTreeSet};

/// Validate persisted semantics without repairing state or rejecting damaged pens.
/// Only accepted pending commands may use `command_boundary`; events use current_tick.
pub fn validate_state_at_boundary(
    world: &World,
    res: &SimResources,
    command_boundary: u64,
) -> Result<(), String> {
    let state = &res.livestock;
    if res.rules.colony.is_none() {
        return if state == &Default::default() {
            Ok(())
        } else {
            Err("livestock outside colony".into())
        };
    }
    let today = super::day(res);
    if state.next_id == 0 || state.last_advanced_day.is_some_and(|d| d > today) {
        return Err("invalid livestock allocator/daily clock".into());
    }
    let mut creations = BTreeMap::new();
    let mut closed = BTreeSet::new();
    let mut captured_sources = BTreeSet::new();
    let mut first_capture = None;
    let mut first_kid = None;
    let catalog = installed_or_bundled();
    for (&index, witness) in &state.witnesses {
        if witness.livestock == 0 || witness.livestock >= state.next_id {
            return Err("livestock witness reuses/exceeds allocator".into());
        }
        let entry = res
            .chronicle
            .entry(index)
            .ok_or("missing livestock chronicle entry")?;
        if entry.tick > res.current_tick
            || entry.kind != event_kind(&witness.event)
            || res.chronicle.cause_of(index) != Some(CauseRef::Livestock { witness: index })
            || entry.other.is_some()
            || entry.third.is_some()
        {
            return Err("livestock witness/chronicle mismatch".into());
        }
        let policy = policy(res, witness)?;
        match &witness.event {
            LivestockEvent::Captured {
                wild,
                attempt,
                knowledge,
                tool,
                origin,
            } => {
                if !captured_sources.insert(*wild)
                    || *attempt == 0
                    || *wild == 0
                    || tool.map(|item| item.kind) != policy.capture_tool
                    || entry.subject.is_none()
                    || creations.insert(witness.livestock, index).is_some()
                {
                    return Err("invalid or duplicate captured livestock".into());
                }
                validate_knowledge(knowledge, &policy.capture_knowledge)?;
                validate_origin(res, *origin, *wild, false, entry.tick)?;
                if first_capture.is_none() {
                    first_capture = Some(witness.livestock);
                }
            }
            LivestockEvent::Born { mother, father } => {
                if entry.subject.is_some()
                    || mother == father
                    || *mother >= witness.livestock
                    || *father >= witness.livestock
                    || creations.insert(witness.livestock, index).is_some()
                {
                    return Err("invalid newborn identity/subjects".into());
                }
                for parent in [mother, father] {
                    let parent_index = creations
                        .get(parent)
                        .ok_or("newborn parent has no creation witness")?;
                    let parent_witness = state
                        .witnesses
                        .get(parent_index)
                        .ok_or("missing parent creation")?;
                    if parent_witness.species != witness.species
                        || parent_witness.pen_site != witness.pen_site
                        || closed.contains(parent)
                    {
                        return Err("newborn parent species/pen/lifetime mismatch".into());
                    }
                }
                if first_kid.is_none() {
                    first_kid = Some(witness.livestock);
                }
            }
            LivestockEvent::Milk { knowledge, product } => {
                let milk = policy
                    .products
                    .milk
                    .as_ref()
                    .ok_or("milk witness without policy")?;
                validate_knowledge(knowledge, &milk.requires_knowledge)?;
                if entry.subject.is_none() || product.item.kind != milk.item || product.count != 1 {
                    return Err("invalid milk production witness".into());
                }
                existing_subject(res, witness, &creations, &closed)?;
            }
            LivestockEvent::Slaughter { origin, products } => {
                existing_subject(res, witness, &creations, &closed)?;
                validate_origin(res, *origin, witness.livestock, true, entry.tick)?;
                if entry.subject.is_none()
                    || products.len() != 2
                    || (products[0].item.kind, products[0].count) != policy.products.meat
                    || (products[1].item.kind, products[1].count) != policy.products.hide
                {
                    return Err("invalid slaughter production witness".into());
                }
                closed.insert(witness.livestock);
            }
            LivestockEvent::Died { .. } => {
                existing_subject(res, witness, &creations, &closed)?;
                if entry.subject.is_some() {
                    return Err("natural livestock death has human subject".into());
                }
                closed.insert(witness.livestock);
            }
        }
    }
    for (index, entry) in res.chronicle.entries().iter().enumerate() {
        if matches!(
            entry.kind,
            ChronicleKind::LivestockCaptured
                | ChronicleKind::LivestockBorn
                | ChronicleKind::LivestockMilk
                | ChronicleKind::LivestockSlaughtered
                | ChronicleKind::LivestockDied
        ) && u32::try_from(index)
            .ok()
            .is_none_or(|id| !state.witnesses.contains_key(&id))
        {
            return Err("livestock chronicle entry lacks witness".into());
        }
    }
    if state.first_capture != first_capture || state.first_kid_id != first_kid {
        return Err("first livestock facts are not pinned to earliest events".into());
    }
    for (&id, animal) in &state.animals {
        let creation = creations
            .get(&id)
            .and_then(|index| state.witnesses.get(index));
        if id != animal.id
            || id == 0
            || id >= state.next_id
            || animal.last_age_tick > res.current_tick
            || animal.created_day > today
            || animal.last_milk_day < animal.created_day
            || animal.last_milk_day > today
            || animal.last_bred_day < animal.created_day
            || animal.last_bred_day > today
            || closed.contains(&id)
            || creations.get(&id) != Some(&animal.creation_entry)
            || creation.is_none_or(|w| w.species != animal.species || w.pen_site != animal.pen_site)
        {
            return Err("live livestock identity/creation/clock mismatch".into());
        }
        let witness = creation.ok_or("missing live creation")?;
        let policy = policy(res, witness)?;
        let created_tick = res
            .chronicle
            .entry(animal.creation_entry)
            .ok_or("missing creation entry")?
            .tick;
        if animal.last_age_tick < created_tick.saturating_sub(res.ticks_per_day.max(1) - 1)
            || animal.created_day
                != u32::try_from(crate::runtime::resource_regen::day_index_at(
                    res,
                    created_tick,
                ))
                .unwrap_or(u32::MAX)
        {
            return Err("livestock biological/display creation phase mismatch".into());
        }
        let day_ticks = res.ticks_per_day.max(1);
        let elapsed = res.current_tick.saturating_sub(created_tick);
        let elapsed_days = u32::try_from(elapsed / day_ticks).unwrap_or(u32::MAX);
        let age = sim_core::colony::livestock::current_age(animal, res.current_tick, day_ticks);
        let age_valid = match witness.event {
            LivestockEvent::Born { .. } => {
                age == elapsed_days
                    && animal.last_age_tick >= created_tick
                    && (animal.last_age_tick - created_tick).is_multiple_of(day_ticks)
            }
            LivestockEvent::Captured { .. } => {
                age >= elapsed_days
                    && age
                        <= policy
                            .kid_age_days
                            .saturating_sub(1)
                            .saturating_add(elapsed_days)
                            .saturating_add(u32::from(!elapsed.is_multiple_of(day_ticks)))
            }
            _ => false,
        };
        if !age_valid
            || animal.hunger_days > today.saturating_sub(animal.created_day)
            || (animal.milk_ready
                && (!animal.sex.female
                    || animal.hunger_days != 0
                    || age < policy.kid_age_days
                    || today <= animal.created_day
                    || policy.products.milk.as_ref().is_none_or(|milk| {
                        today.saturating_sub(animal.last_milk_day) < milk.every_days
                    })))
        {
            return Err("livestock age/hunger/product readiness is impossible".into());
        }
    }
    if creations
        .keys()
        .any(|id| !closed.contains(id) && !state.animals.contains_key(id))
    {
        return Err("created livestock disappeared without removal event".into());
    }
    let max_regrow = catalog
        .animals()
        .filter_map(|a| a.tameable.as_ref())
        .map(|p| p.grass_regrow_days)
        .max()
        .unwrap_or(0);
    for (&(x, y), grass) in &state.grass {
        if x >= res.tile_grid.width
            || y >= res.tile_grid.height
            || grass.next_regrow_day > today.saturating_add(max_regrow)
        {
            return Err("invalid livestock grass clock/tile".into());
        }
    }
    let max_capture_retry = catalog
        .animals()
        .filter_map(|species| species.tameable.as_ref())
        .map(|policy| policy.capture_retry_days)
        .max()
        .unwrap_or(0);
    for (&wild, attempt) in &state.capture_attempts {
        let retry_days = match res.colony.wildlife.animals.get(&wild) {
            Some(source) => {
                catalog
                    .animal(source.kind)
                    .and_then(|species| species.tameable.as_ref())
                    .ok_or("capture attempt has untameable source")?
                    .capture_retry_days
            }
            // A source can disappear after JobExecute and before the hourly prune.
            None => max_capture_retry,
        };
        if wild == 0
            || wild >= res.colony.wildlife.next_id
            || attempt.completed_attempts == 0
            || attempt.not_before_day > today.saturating_add(retry_days)
        {
            return Err("invalid completed capture attempt".into());
        }
    }
    for (&wild, &origin) in &state.capture_orders {
        validate_origin(res, origin, wild, false, command_boundary)?;
    }
    for (&animal, &origin) in &state.slaughter_orders {
        if !state.animals.contains_key(&animal) {
            return Err("slaughter order has no living target".into());
        }
        validate_origin(res, origin, animal, true, command_boundary)?;
    }
    for (&kind, &count) in &state.lost_capture_tools {
        if count == 0
            || !catalog
                .animals()
                .filter_map(|a| a.tameable.as_ref())
                .any(|p| p.capture_tool == Some(kind))
        {
            return Err("invalid lost livestock capture tool".into());
        }
    }
    validate_jobs(world, res)
}

fn existing_subject(
    res: &SimResources,
    witness: &LivestockEventWitness,
    creations: &BTreeMap<u32, u32>,
    closed: &BTreeSet<u32>,
) -> Result<(), String> {
    let created = creations
        .get(&witness.livestock)
        .and_then(|index| res.livestock.witnesses.get(index));
    if closed.contains(&witness.livestock)
        || created.is_none_or(|w| w.species != witness.species || w.pen_site != witness.pen_site)
    {
        Err("livestock event outside subject lifetime/provenance".into())
    } else {
        Ok(())
    }
}

fn policy(
    res: &SimResources,
    witness: &LivestockEventWitness,
) -> Result<&'static TameableDef, String> {
    let catalog = installed_or_bundled();
    let species = catalog
        .animal(witness.species)
        .ok_or("unknown livestock species")?;
    if species.status != ContentStatus::Connected {
        return Err("inactive livestock species".into());
    }
    let policy = species
        .tameable
        .as_ref()
        .ok_or("livestock species has no tameable policy")?;
    let pen = res
        .colony
        .buildings
        .sites
        .get(&witness.pen_site)
        .ok_or("livestock pen provenance is missing")?;
    if catalog
        .building_templates()
        .get(&pen.template)
        .is_none_or(|t| t.function.livestock_capacity == 0)
    {
        return Err("livestock pen lacks capacity capability".into());
    }
    Ok(policy)
}

fn validate_knowledge(witness: &KnowledgeWitness, name: &str) -> Result<(), String> {
    let defs = installed_or_bundled().knowledge_defs();
    if defs.id_of(name) != Some(witness.knowledge)
        || witness.required != defs.threshold(witness.knowledge)
        || witness.proficiency < witness.required
        || witness.proficiency > 1000
    {
        Err("invalid livestock knowledge witness".into())
    } else {
        Ok(())
    }
}

fn validate_origin(
    res: &SimResources,
    origin: LivestockOrderCause,
    target: u32,
    slaughter: bool,
    boundary: u64,
) -> Result<(), String> {
    let record = res
        .colony
        .command_log
        .get(origin.source_record as usize)
        .ok_or("missing livestock command source")?;
    if !record.result.accepted || record.tick > boundary {
        return Err("unaccepted/future livestock command".into());
    }
    let (command, rule) = match &record.command {
        Command::Steward { rule, command } => (command.as_ref(), Some(*rule)),
        command => (command, None),
    };
    let matches = match command {
        Command::DesignateCapture { animal } => !slaughter && *animal == target,
        Command::DesignateSlaughter { animal } => slaughter && *animal == target,
        _ => false,
    };
    if !matches || rule != origin.steward_rule {
        Err("livestock command target/wrapper mismatch".into())
    } else {
        Ok(())
    }
}

fn validate_jobs(world: &World, res: &SimResources) -> Result<(), String> {
    let mut keys = BTreeMap::new();
    for (entity, (agent, job)) in world.query::<(&Agent, &Job)>().iter() {
        if !matches!(
            job.kind,
            JobKind::Capture { .. } | JobKind::Milk { .. } | JobKind::Slaughter { .. }
        ) {
            continue;
        }
        let expected: BTreeSet<_> = job.keys().into_iter().collect();
        let actual: BTreeSet<_> = res
            .colony
            .reservations
            .iter()
            .filter(|(_, holder)| **holder == agent.id)
            .map(|(key, _)| *key)
            .collect();
        if expected != actual {
            return Err("livestock job reservation mismatch".into());
        }
        if let JobKind::Milk { animal, .. } | JobKind::Slaughter { animal, .. } = job.kind {
            if !res.livestock.animals.contains_key(&animal) {
                return Err("livestock job target removed without cancellation".into());
            }
        }
        if let JobKind::Capture { animal, .. } = job.kind {
            let tool = res
                .colony
                .wildlife
                .animals
                .get(&animal)
                .and_then(|wild| wild.kind.definition().tameable.as_ref())
                .and_then(|policy| policy.capture_tool);
            if let Some(tool) = tool.filter(|_| job.fetch.is_none()) {
                if world
                    .get::<&Carry>(entity)
                    .ok()
                    .is_none_or(|carry| carry.kind != tool || carry.count != 1)
                {
                    return Err("capture job lost its physical carried tool".into());
                }
            }
        }
        for key in expected {
            if keys.insert(key, agent.id).is_some() {
                return Err("duplicate livestock job owner".into());
            }
        }
    }
    for (key, holder) in &res.colony.reservations {
        if matches!(key, ReserveKey::Livestock(_) | ReserveKey::PenCapture(_))
            && keys.get(key) != Some(holder)
        {
            return Err("orphaned livestock reservation".into());
        }
    }
    Ok(())
}

pub(super) fn event_kind(event: &LivestockEvent) -> ChronicleKind {
    match event {
        LivestockEvent::Captured { .. } => ChronicleKind::LivestockCaptured,
        LivestockEvent::Born { .. } => ChronicleKind::LivestockBorn,
        LivestockEvent::Milk { .. } => ChronicleKind::LivestockMilk,
        LivestockEvent::Slaughter { .. } => ChronicleKind::LivestockSlaughtered,
        LivestockEvent::Died { .. } => ChronicleKind::LivestockDied,
    }
}
