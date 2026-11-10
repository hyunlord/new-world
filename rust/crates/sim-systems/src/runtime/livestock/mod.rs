//! LivestockSystem: physical forage, biological age, hunger, natural death and birth.
//! Priority: 176; interval: 60 ticks; gate: colony only.
//! Reads: authored tameable policy, actual pen pieces, clock, canonical jobs/reservations.
//! Writes: livestock sidecar, cancelled target jobs, immutable chronicle witnesses.
//! Events: LivestockBorn, LivestockDied; jobs emit capture/milk/slaughter after commit.
pub mod pens;
#[cfg(test)]
pub(crate) mod tests;
mod validation;

pub use pens::{has_capture_slot, nearest_pen_with_room, pen_layout, PenLayout, PenUnavailable};
pub use validation::validate_state_at_boundary;

use hecs::World;
use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
use sim_core::colony::livestock::{
    advance_age, current_age, GrassState, Livestock, LivestockDeathReason, LivestockEvent,
    LivestockEventWitness,
};
use sim_core::colony::livestock_demographics::newborn_sex;
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::BTreeMap;

/// Livestock follows wildlife and visitors, before diagnosis.
pub const LIVESTOCK_PRIORITY: u32 = 176;

/// The sole livestock lifecycle scheduler; all durable clocks live in the sidecar.
#[derive(Debug, Default)]
pub struct LivestockSystem;
impl LivestockSystem {
    /// Create the stateless scheduler.
    pub fn new() -> Self {
        Self
    }
}
impl RuntimeSystem for LivestockSystem {
    fn name(&self) -> &str {
        "LivestockSystem"
    }
    fn priority(&self) -> u32 {
        LIVESTOCK_PRIORITY
    }
    fn tick_interval(&self) -> u64 {
        60
    }
    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        advance(world, res);
    }
}

/// Current display day, including the established simulation clock offset.
pub fn day(res: &SimResources) -> u32 {
    u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX)
}

/// Restore the derived product cooldown index after chronicle and livestock load.
pub fn rebuild_product_cache(res: &mut SimResources) {
    res.livestock.wool_last_day = wool_days(res);
}

fn wool_days(res: &SimResources) -> BTreeMap<u32, u32> {
    res.livestock
        .witnesses
        .iter()
        .filter_map(|(&index, witness)| {
            let LivestockEvent::Milk { product, .. } = &witness.event else {
                return None;
            };
            let wool = witness
                .species
                .definition()
                .tameable
                .as_ref()?
                .products
                .wool
                .as_ref()?;
            if product.item.kind != wool.item {
                return None;
            }
            let entry = res.chronicle.entry(index)?;
            let day = u32::try_from(crate::runtime::resource_regen::day_index_at(
                res, entry.tick,
            ))
            .unwrap_or(u32::MAX);
            Some((witness.livestock, day))
        })
        .collect()
}

/// The next usable witness index; call before any irreversible physical commit.
pub fn next_event_index(res: &SimResources) -> Option<u32> {
    u32::try_from(res.chronicle.len())
        .ok()
        .filter(|&id| id != u32::MAX)
}

/// Append an already committed fact and its immutable typed witness.
/// The producer must preflight next_event_index before its physical mutation.
pub(crate) fn emit_event(
    res: &mut SimResources,
    entry: ChronicleEntry,
    witness: LivestockEventWitness,
) -> u32 {
    let index = u32::try_from(res.chronicle.len()).unwrap_or(u32::MAX);
    let actual = res
        .chronicle
        .push_caused(entry, Some(CauseRef::Livestock { witness: index }));
    if matches!(&witness.event, LivestockEvent::Milk { product, .. }
        if witness.species.definition().tameable.as_ref()
            .and_then(|p| p.products.wool.as_ref())
            .is_some_and(|wool| wool.item == product.item.kind))
    {
        res.livestock
            .wool_last_day
            .insert(witness.livestock, day(res));
    }
    res.livestock.witnesses.insert(actual, witness);
    res.livestock.touch();
    actual
}

fn advance(world: &mut World, res: &mut SimResources) {
    if res.rules.colony.is_none() {
        return;
    }
    let today = day(res);
    let mut changed = false;
    for animal in res.livestock.animals.values_mut() {
        let old = (animal.age_days, animal.last_age_tick);
        advance_age(animal, res.current_tick, res.ticks_per_day);
        changed |= old != (animal.age_days, animal.last_age_tick);
    }
    // Dead wild sources cannot retain pending orders/cooldowns forever.
    let wild = &res.colony.wildlife.animals;
    let before = (
        res.livestock.capture_orders.len(),
        res.livestock.capture_attempts.len(),
    );
    res.livestock
        .capture_orders
        .retain(|id, _| wild.contains_key(id));
    res.livestock
        .capture_attempts
        .retain(|id, _| wild.contains_key(id));
    changed |= before
        != (
            res.livestock.capture_orders.len(),
            res.livestock.capture_attempts.len(),
        );
    let previous = res.livestock.last_advanced_day.unwrap_or(today);
    if res.livestock.last_advanced_day != Some(today) {
        for at in previous.saturating_add(1)..=today {
            daily(world, res, at);
        }
        res.livestock.last_advanced_day = Some(today);
        changed = true;
    }
    if changed {
        res.livestock.touch();
    }
}

fn daily(world: &mut World, res: &mut SimResources, today: u32) {
    // Availability regenerates by comparing the saved next_regrow_day; keep its history.
    let layouts: BTreeMap<_, _> = res
        .colony
        .buildings
        .sites
        .keys()
        .filter_map(|&id| pen_layout(res, id).ok().map(|pen| (id, pen)))
        .collect();
    let ids: Vec<_> = res.livestock.animals.keys().copied().collect();
    let mut deaths = Vec::new();
    for id in ids {
        let Some(animal) = res.livestock.animals.get(&id).cloned() else {
            continue;
        };
        let Some(policy) = animal.species.definition().tameable.as_ref() else {
            continue;
        };
        let fed = if today <= animal.created_day {
            None
        } else {
            let cells: Vec<_> = layouts
                .get(&animal.pen_site)
                .into_iter()
                .flat_map(|p| &p.interior)
                .copied()
                .filter(|tile| {
                    res.livestock
                        .grass
                        .get(tile)
                        .is_none_or(|g| today >= g.next_regrow_day)
                })
                .take(policy.graze_tiles_per_head as usize)
                .collect();
            let enough = cells.len() == policy.graze_tiles_per_head as usize;
            if enough {
                for cell in cells {
                    res.livestock.grass.insert(
                        cell,
                        GrassState {
                            next_regrow_day: today.saturating_add(policy.grass_regrow_days),
                        },
                    );
                }
            }
            Some(enough)
        };
        let age = current_age(&animal, res.current_tick, res.ticks_per_day);
        if let Some(live) = res.livestock.animals.get_mut(&id) {
            if let Some(fed) = fed {
                live.hunger_days = if fed {
                    0
                } else {
                    live.hunger_days.saturating_add(1)
                };
            }
            live.milk_ready = layouts.contains_key(&animal.pen_site)
                && today > animal.created_day
                && live.hunger_days == 0
                && live.sex.female
                && age >= policy.kid_age_days
                && policy.products.milk.as_ref().is_some_and(|milk| {
                    today.saturating_sub(live.last_milk_day) >= milk.every_days
                });
            if age >= policy.max_age_days {
                deaths.push((id, LivestockDeathReason::OldAge));
            } else if live.hunger_days >= policy.starvation_days {
                deaths.push((id, LivestockDeathReason::Starvation));
            }
        }
    }
    for (id, reason) in deaths {
        if next_event_index(res).is_none() {
            break;
        }
        let Some(animal) = res.livestock.animals.get(&id).cloned() else {
            continue;
        };
        crate::runtime::jobs::livestock::cancel_for_animal(world, res, id);
        res.livestock.animals.remove(&id);
        res.livestock.slaughter_orders.remove(&id);
        emit_event(
            res,
            ChronicleEntry::new(res.current_tick, ChronicleKind::LivestockDied),
            LivestockEventWitness {
                livestock: id,
                species: animal.species,
                pen_site: animal.pen_site,
                event: LivestockEvent::Died { reason },
            },
        );
    }
    births(res, today, &layouts);
    res.livestock.touch();
}

fn births(res: &mut SimResources, today: u32, layouts: &BTreeMap<u32, PenLayout>) {
    let mothers: Vec<_> = res
        .livestock
        .animals
        .values()
        .filter(|a| a.sex.female)
        .cloned()
        .collect();
    for mother in mothers {
        let Some(policy) = mother.species.definition().tameable.as_ref() else {
            continue;
        };
        let Some(pen) = layouts.get(&mother.pen_site) else {
            continue;
        };
        if mother.created_day >= today
            || mother.hunger_days != 0
            || current_age(&mother, res.current_tick, res.ticks_per_day) < policy.kid_age_days
            || today.saturating_sub(mother.last_bred_day) < policy.breeding_interval_days
            || res
                .livestock
                .animals
                .values()
                .filter(|a| a.pen_site == pen.site)
                .count()
                >= usize::from(pen.capacity)
        {
            continue;
        }
        let father = res
            .livestock
            .animals
            .values()
            .find(|a| {
                a.species == mother.species
                    && a.pen_site == mother.pen_site
                    && !a.sex.female
                    && a.hunger_days == 0
                    && a.created_day < today
                    && current_age(a, res.current_tick, res.ticks_per_day) >= policy.kid_age_days
            })
            .map(|a| a.id);
        let Some(father) = father else {
            continue;
        };
        // Parents already ate; admit another head only if physical forage can support it.
        let remaining = pen
            .interior
            .iter()
            .filter(|tile| {
                res.livestock
                    .grass
                    .get(tile)
                    .is_none_or(|g| today >= g.next_regrow_day)
            })
            .count();
        if remaining < policy.graze_tiles_per_head as usize {
            continue;
        }
        let support_needed: u64 = res
            .livestock
            .animals
            .values()
            .filter(|a| a.pen_site == pen.site)
            .filter_map(|a| a.species.definition().tameable.as_ref())
            .map(|p| u64::from(p.graze_tiles_per_head))
            .sum();
        if support_needed.saturating_add(u64::from(policy.graze_tiles_per_head))
            > pen.interior.len() as u64
        {
            continue;
        }
        let Some((id, next)) = res.livestock.checked_next_id() else {
            break;
        };
        let Some(entry) = next_event_index(res) else {
            break;
        };
        let newborn = Livestock {
            id,
            species: mother.species,
            age_days: 0,
            last_age_tick: res.current_tick,
            sex: newborn_sex(res.world_seed, mother.species, id),
            hunger_days: 0,
            pen_site: pen.site,
            milk_ready: false,
            last_milk_day: today,
            last_bred_day: today,
            created_day: today,
            creation_entry: entry,
        };
        res.livestock.next_id = next;
        res.livestock.animals.insert(id, newborn);
        if let Some(live) = res.livestock.animals.get_mut(&mother.id) {
            live.last_bred_day = today;
        }
        if res.livestock.first_kid_id.is_none() {
            res.livestock.first_kid_id = Some(id);
        }
        emit_event(
            res,
            ChronicleEntry::new(res.current_tick, ChronicleKind::LivestockBorn),
            LivestockEventWitness {
                livestock: id,
                species: mother.species,
                pen_site: pen.site,
                event: LivestockEvent::Born {
                    mother: mother.id,
                    father,
                },
            },
        );
    }
}
