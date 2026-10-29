//! Colony-only hidden food and physical ownership state for determinism checks.
//! Streams canonical RON directly into FNV; baseline bytes remain untouched.
use sim_core::colony::Carry;
use sim_core::components::Agent;
use sim_engine::SimEngine;
use std::io::{self, Write};

struct HashWriter<'a>(&'a mut u64);
impl Write for HashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        super::fingerprint::feed(self.0, bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

/// FoodState includes plants, ages, pots, meals, all flow/first-event state and
/// pending drying. Physical stores are separate, so bind their counts, shelves,
/// storage-class inputs and reservations too. Carriers use stable AgentId order.
/// Invalid carrier identity is an error, never an apparently equal partial hash.
pub(super) fn append(hash: &mut u64, engine: &SimEngine) -> Result<(), String> {
    if engine.resources.rules.colony.is_none() {
        return Ok(());
    }
    let mut carried = Vec::new();
    for (_, (carry, agent)) in engine.world.query::<(&Carry, Option<&Agent>)>().iter() {
        let agent = agent.ok_or("food fingerprint carrier has no AgentId")?;
        carried.push((agent.id, *carry));
    }
    carried.sort_unstable_by_key(|row| row.0);
    if carried.windows(2).any(|pair| pair[0].0 == pair[1].0) {
        return Err("food fingerprint carrier AgentId is duplicated".into());
    }
    super::fingerprint::feed(hash, b"\0M6a-food-physical-state-v1\0");
    let colony = &engine.resources.colony;
    ron::ser::to_writer(
        HashWriter(hash),
        &(
            &engine.resources.food,
            &colony.ground_items,
            &colony.workshops,
            &colony.zones,
            &colony.rooms,
            &colony.reservations,
            &colony.item_ledger,
            carried,
        ),
    )
    .map_err(|error| format!("food fingerprint serialization failed: {error}"))
}

#[cfg(test)]
#[path = "food_fingerprint_tests.rs"]
mod tests;
