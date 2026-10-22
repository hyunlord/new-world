//! The ECS world in a save file: the component registry, capture and rebuild.
//!
//! Exactness matters beyond component values: every hecs query iterates
//! archetypes in creation order and entities in dense order (both depend on
//! history: swap-removes, archetype moves), entity ids appear in components
//! and the bridge, and `World::spawn` reuses the most recently freed id with
//! its bumped generation. So a save records:
//! - archetypes in creation order, the empty ones too (an empty archetype
//!   that fills later keeps its place before younger ones);
//! - each archetype's entities (id and generation) in dense order and one
//!   column of values per component type;
//! - the allocator: its id range (`meta_len`) and its free list in stack
//!   order with each free id's generation.
//!
//! [`build`] recreates the archetypes empty in order, spawns every entity
//! with `spawn_at` in archetype and dense order, then rebuilds the free list
//! (see the steps in [`build`]). [`capture`] checks that every archetype's
//! types are all in [`registry`]; an unregistered component type fails the
//! save rather than losing data.

use std::any::TypeId;
use std::collections::{BTreeMap, BTreeSet};

use hecs::{Archetype, ColumnBatchType, Component, Entity, EntityBuilder, World};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

use sim_core::colony::knowledge::Knowledge;
use sim_core::colony::{Carry, ColonySite, Job, MaterialNeeds, Skills, Tool, WorkPriorities};
use sim_core::components::{
    Activity, ActivityRest, Agent, AgentState, BodyHealth, Child, Cold, ConstructionSite, Elder,
    ExposureClock, FamilyObservations, Founding, Hardiness, Hunger, Inventory, Kin, Life, Lineage,
    Memory, PorterTrip, Position, Pregnant, RoutineObservation, RoutineObservationBatch,
    SeekTarget, SettlementMigrant, Sex, Sleep, Social, Temperament, Thirst,
};

use crate::runtime::agent::MovementRng;
use crate::runtime::influence::agent_sample::InfluenceSample;

/// How one component type is written, read and declared.
pub struct ComponentCodec {
    /// Stable name in save files (the type's name).
    pub name: &'static str,
    type_id: fn() -> TypeId,
    save: fn(&Archetype) -> Result<Vec<u8>, String>,
    load: fn(&[u8], &mut [EntityBuilder]) -> Result<(), String>,
    declare: fn(&mut ColumnBatchType),
}

fn codec<T: Component + Serialize + DeserializeOwned>(name: &'static str) -> ComponentCodec {
    ComponentCodec {
        name,
        type_id: TypeId::of::<T>,
        save: save_column::<T>,
        load: load_column::<T>,
        declare: declare_column::<T>,
    }
}

fn save_column<T: Component + Serialize>(archetype: &Archetype) -> Result<Vec<u8>, String> {
    let column = archetype
        .get::<&T>()
        .ok_or_else(|| "archetype lacks a registered column".to_string())?;
    super::encode(&*column)
}

fn load_column<T: Component + DeserializeOwned>(
    bytes: &[u8],
    builders: &mut [EntityBuilder],
) -> Result<(), String> {
    let values: Vec<T> = super::decode(bytes)?;
    if values.len() != builders.len() {
        return Err(format!(
            "column holds {} values for {} entities",
            values.len(),
            builders.len()
        ));
    }
    for (builder, value) in builders.iter_mut().zip(values) {
        builder.add(value);
    }
    Ok(())
}

fn declare_column<T: Component>(batch: &mut ColumnBatchType) {
    batch.add::<T>();
}

/// Every component type the simulation puts in the world. A new component
/// type must be added here (the save fails on an unregistered type, and the
/// round-trip tests catch it).
pub fn registry() -> Vec<ComponentCodec> {
    vec![
        codec::<Activity>("Activity"),
        codec::<ActivityRest>("ActivityRest"),
        codec::<Agent>("Agent"),
        codec::<AgentState>("AgentState"),
        codec::<BodyHealth>("BodyHealth"),
        codec::<Carry>("Carry"),
        codec::<Child>("Child"),
        codec::<Cold>("Cold"),
        codec::<ColonySite>("ColonySite"),
        codec::<ConstructionSite>("ConstructionSite"),
        codec::<Elder>("Elder"),
        codec::<ExposureClock>("ExposureClock"),
        codec::<FamilyObservations>("FamilyObservations"),
        codec::<Founding>("Founding"),
        codec::<Hardiness>("Hardiness"),
        codec::<Hunger>("Hunger"),
        codec::<InfluenceSample>("InfluenceSample"),
        codec::<Inventory>("Inventory"),
        codec::<Job>("Job"),
        codec::<Kin>("Kin"),
        codec::<Knowledge>("Knowledge"),
        codec::<Life>("Life"),
        codec::<Lineage>("Lineage"),
        codec::<MaterialNeeds>("MaterialNeeds"),
        codec::<Memory>("Memory"),
        codec::<MovementRng>("MovementRng"),
        codec::<PorterTrip>("PorterTrip"),
        codec::<Position>("Position"),
        codec::<Pregnant>("Pregnant"),
        codec::<RoutineObservation>("RoutineObservation"),
        codec::<RoutineObservationBatch>("RoutineObservationBatch"),
        codec::<SeekTarget>("SeekTarget"),
        codec::<Skills>("Skills"),
        codec::<SettlementMigrant>("SettlementMigrant"),
        codec::<Sex>("Sex"),
        codec::<Sleep>("Sleep"),
        codec::<Social>("Social"),
        codec::<Temperament>("Temperament"),
        codec::<Thirst>("Thirst"),
        codec::<Tool>("Tool"),
        codec::<WorkPriorities>("WorkPriorities"),
    ]
}

/// One archetype: component names (indices into [`WorldSave::components`]),
/// entity bits in dense order, one encoded column per component.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArchetypeSave {
    types: Vec<u16>,
    entities: Vec<u64>,
    columns: Vec<Vec<u8>>,
}

/// The saved world.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorldSave {
    /// Component names used by `archetypes` (the registry, in its order).
    components: Vec<String>,
    /// Archetypes in creation order; index 0 is hecs's empty archetype.
    archetypes: Vec<ArchetypeSave>,
    /// The allocator's id range: every id below it has been allocated once.
    meta_len: u32,
    /// Free ids in free-list (stack) order, bottom first, as entity bits
    /// (the generation the id gets when it is reused).
    free: Vec<u64>,
}

impl WorldSave {
    /// Number of free ids in the saved allocator (ids a later spawn reuses).
    pub fn free_len(&self) -> usize {
        self.free.len()
    }

    /// Number of saved live entities.
    pub fn live_len(&self) -> usize {
        self.archetypes.iter().map(|a| a.entities.len()).sum()
    }
}

/// Capture `world`. Reads the free list by reserving ids, so `world` must be
/// replaced afterwards (as `save` does) — it then holds reserved ids.
pub fn capture(world: &World) -> Result<WorldSave, String> {
    let codecs = registry();
    let ids: Vec<TypeId> = codecs.iter().map(|c| (c.type_id)()).collect();
    let handles: BTreeMap<u32, Entity> = world
        .iter()
        .map(|e| (e.entity().id(), e.entity()))
        .collect();
    let mut archetypes = Vec::new();
    for (index, archetype) in world.archetypes().enumerate() {
        let types: Vec<usize> = (0..codecs.len())
            .filter(|&i| archetype.has_dynamic(ids[i]))
            .collect();
        if types.len() != archetype.component_types().len() {
            return Err(format!(
                "archetype {index} has {} component types, {} registered (add the type to save::world::registry)",
                archetype.component_types().len(),
                types.len()
            ));
        }
        let entities = archetype
            .ids()
            .iter()
            .map(|id| {
                handles
                    .get(id)
                    .map(|e| e.to_bits().get())
                    .ok_or_else(|| format!("entity id {id} of archetype {index} is not live"))
            })
            .collect::<Result<Vec<u64>, String>>()?;
        let columns = types
            .iter()
            .map(|&i| (codecs[i].save)(archetype))
            .collect::<Result<Vec<_>, String>>()?;
        archetypes.push(ArchetypeSave {
            types: types
                .iter()
                .map(|&i| u16::try_from(i).unwrap_or(u16::MAX))
                .collect(),
            entities,
            columns,
        });
    }
    // The free list: `reserve_entity` hands out free ids in pop order (top of
    // the stack first) with their stored generation, then fresh ids (from
    // `meta_len`, generation 1). Freed ids always carry generation ≥ 2.
    let live_max = handles.keys().next_back().copied();
    let mut popped = Vec::new();
    let meta_len = loop {
        let entity = world.reserve_entity();
        if generation(entity) == 1 {
            let fresh = entity.id();
            let seen_max = popped.iter().map(|e: &Entity| e.id()).chain(live_max).max();
            if seen_max.is_some_and(|m| m >= fresh) {
                return Err(format!(
                    "free id {fresh} has generation 1 (unsupported allocator state)"
                ));
            }
            break fresh;
        }
        popped.push(entity);
    };
    let live = u32::try_from(handles.len()).unwrap_or(u32::MAX);
    if u64::from(meta_len) != u64::from(live) + popped.len() as u64 {
        return Err(format!(
            "allocator: {meta_len} ids, {live} live, {} free",
            popped.len()
        ));
    }
    let free = popped.iter().rev().map(|e| e.to_bits().get()).collect();
    Ok(WorldSave {
        components: codecs.iter().map(|c| c.name.to_string()).collect(),
        archetypes,
        meta_len,
        free,
    })
}

/// An entity's generation (the high 32 bits of `Entity::to_bits`).
fn generation(e: Entity) -> u32 {
    u32::try_from(e.to_bits().get() >> 32).unwrap_or(0)
}

fn entity(bits: u64) -> Result<Entity, String> {
    Entity::from_bits(bits).ok_or_else(|| format!("bad entity bits {bits:#x}"))
}

/// The entity with `id` and `generation` (generation ≥ 1).
fn handle(id: u32, generation: u32) -> Result<Entity, String> {
    entity((u64::from(generation) << 32) | u64::from(id))
}

/// Rebuild the world `save` describes.
///
/// 1. Create every archetype after hecs's empty one, empty, in saved order.
/// 2. Spawn every live entity with `spawn_at` in archetype then dense order
///    (ids skipped over enter hecs's free list with generation 1).
/// 3. If the top id is free, spawn a placeholder there so the id range ends
///    at `meta_len`.
/// 4. Pop every remaining free id with a placeholder spawn.
/// 5. For each free id in saved stack order, give its placeholder the
///    generation below the saved one and despawn it: the despawn bumps the
///    generation to the saved value and pushes the id, so the free list ends
///    up in saved order. Placeholders live only in the empty archetype after
///    every saved component-less entity, so that archetype's order is kept.
pub fn build(save: &WorldSave) -> Result<World, String> {
    let codecs = registry();
    let by_name: BTreeMap<&str, usize> = codecs
        .iter()
        .enumerate()
        .map(|(i, c)| (c.name, i))
        .collect();
    let map: Vec<usize> = save
        .components
        .iter()
        .map(|n| {
            by_name
                .get(n.as_str())
                .copied()
                .ok_or_else(|| format!("unknown component {n} in save"))
        })
        .collect::<Result<_, String>>()?;
    let codec_of = |t: u16| -> Result<&ComponentCodec, String> {
        map.get(usize::from(t))
            .map(|&i| &codecs[i])
            .ok_or_else(|| format!("component index {t} out of range"))
    };
    let mut world = World::new();
    match save.archetypes.first() {
        Some(first) if first.types.is_empty() => {}
        _ => return Err("archetype 0 must be the empty archetype".to_string()),
    }
    // 1. archetypes in order.
    for (index, archetype) in save.archetypes.iter().enumerate().skip(1) {
        let mut batch = ColumnBatchType::new();
        for &t in &archetype.types {
            (codec_of(t)?.declare)(&mut batch);
        }
        let built = batch
            .into_batch(0)
            .build()
            .map_err(|_| format!("archetype {index}: empty batch incomplete"))?;
        world.spawn_column_batch(built);
        if world.archetypes().len() != index + 1 {
            return Err(format!("archetype {index} duplicates an earlier one"));
        }
    }
    // 2. live entities.
    let mut live: BTreeSet<u32> = BTreeSet::new();
    for (index, archetype) in save.archetypes.iter().enumerate() {
        if archetype.columns.len() != archetype.types.len() {
            return Err(format!("archetype {index}: columns and types differ"));
        }
        let mut builders: Vec<EntityBuilder> = archetype
            .entities
            .iter()
            .map(|_| EntityBuilder::new())
            .collect();
        for (&t, bytes) in archetype.types.iter().zip(&archetype.columns) {
            let codec = codec_of(t)?;
            (codec.load)(bytes, &mut builders)
                .map_err(|e| format!("archetype {index} column {}: {e}", codec.name))?;
        }
        for (&bits, builder) in archetype.entities.iter().zip(builders.iter_mut()) {
            let e = entity(bits)?;
            if !live.insert(e.id()) {
                return Err(format!("entity id {} saved twice", e.id()));
            }
            world.spawn_at(e, builder.build());
        }
    }
    for (index, (have, want)) in world.archetypes().zip(&save.archetypes).enumerate() {
        if have.ids().len() != want.entities.len() {
            return Err(format!("archetype {index} rebuilt with other entities"));
        }
    }
    // 3. the id range.
    let top = save.meta_len.checked_sub(1);
    if let Some(top) = top {
        if !live.contains(&top) {
            world.spawn_at(handle(top, 1)?, ());
        }
    }
    if live.last().is_some_and(|&m| m >= save.meta_len) {
        return Err("a live id lies beyond the saved id range".to_string());
    }
    // 4. pop the free list into placeholders.
    let placeholders = save
        .meta_len
        .checked_sub(world.len())
        .ok_or_else(|| "more live entities than saved ids".to_string())?;
    for _ in 0..placeholders {
        let e = world.spawn(());
        if e.id() >= save.meta_len || live.contains(&e.id()) {
            return Err(format!("placeholder took id {}", e.id()));
        }
    }
    // 5. the free list in saved order with saved generations.
    if save.free.len() != (save.meta_len as usize).saturating_sub(live.len()) {
        return Err("free list does not cover the free ids".to_string());
    }
    for &bits in &save.free {
        let saved = entity(bits)?;
        if generation(saved) < 2 {
            return Err(format!("free id {} with generation 1", saved.id()));
        }
        let before = handle(saved.id(), generation(saved) - 1)?;
        world.spawn_at(before, ());
        world
            .despawn(before)
            .map_err(|_| format!("placeholder {} vanished", saved.id()))?;
    }
    if world.len() as usize != live.len() {
        return Err("rebuilt world has the wrong entity count".to_string());
    }
    Ok(world)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A world with history: swap-removes, an id reused, an archetype move
    /// and an archetype that is empty at the end.
    fn history() -> World {
        let mut w = World::new();
        let a: Vec<Entity> = (0..6u32).map(|i| w.spawn((Position::new(i, 0),))).collect();
        w.spawn((Position::new(9, 9), Cold));
        let gone = w.spawn((Cold,));
        let _ = w.despawn(gone);
        let _ = w.despawn(a[1]);
        let _ = w.despawn(a[4]);
        let _ = w.despawn(a[2]);
        w.spawn((Position::new(7, 7),));
        let _ = w.insert_one(a[0], Cold);
        w
    }

    fn order(w: &World) -> Vec<(u64, u32, u32, bool)> {
        w.query::<(&Position, Option<&Cold>)>()
            .iter()
            .map(|(e, (p, c))| (e.to_bits().get(), p.x, p.y, c.is_some()))
            .collect()
    }

    #[test]
    fn rebuild_keeps_order_ids_and_free_list() {
        let save = capture(&history()).expect("capture");
        assert_eq!((save.meta_len, save.free.len()), (8, 3));
        let mut rebuilt = build(&save).expect("build");
        let mut original = history();
        assert_eq!(order(&rebuilt), order(&original));
        assert_eq!(rebuilt.archetypes().len(), original.archetypes().len());
        // The next spawns reuse the same ids with the same generations, and
        // land in the archetype that was empty in the same query position.
        for i in 0..5u32 {
            let x = original.spawn((Cold, Position::new(100 + i, 0)));
            let y = rebuilt.spawn((Cold, Position::new(100 + i, 0)));
            assert_eq!(x.to_bits(), y.to_bits(), "spawn {i}");
        }
        assert_eq!(order(&rebuilt), order(&original));
        assert_eq!(capture(&rebuilt), capture(&original));
    }

    #[test]
    fn unregistered_component_fails_the_capture() {
        let mut w = World::new();
        w.spawn((Position::new(1, 1), 5u8));
        assert!(capture(&w).is_err());
    }
}
