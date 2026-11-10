//! OreDiscoverySystem: reveal hidden surface deposits from actual personal practice.
//! Priority: 128; interval: 8 ticks; colony only.
//! Reads: positions, knowledge, authored discovery radii and hidden deposits.
//! Writes: ore visibility and immutable witnesses. Events: OreDiscovered.
use hecs::{Entity, World};
use sim_core::{
    chronicle::{CauseRef, ChronicleEntry, ChronicleKind},
    colony::{
        content::{installed_or_bundled, OutcropDef},
        knowledge::{self, Knowledge},
        ore::OreWitness,
    },
    components::{Agent, Position},
    spatial::{chebyshev, ChunkIndex},
};
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::{BTreeMap, BTreeSet};
/// Stateless scheduler; all durable state lives in the versioned ore sidecar.
#[derive(Debug, Default)]
pub struct OreDiscoverySystem;
impl RuntimeSystem for OreDiscoverySystem {
    fn name(&self) -> &str {
        "OreDiscoverySystem"
    }
    fn priority(&self) -> u32 {
        128
    }
    fn tick_interval(&self) -> u64 {
        8
    }
    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        discover(world, res);
    }
}
/// Reveal by walking; a read-only observer never calls this simulation transition.
pub fn discover(world: &World, res: &mut SimResources) {
    if res.rules.colony.is_none() || !res.ore.deposits.values().any(|d| !d.revealed) {
        return;
    }
    let workers: BTreeMap<_, _> = world
        .query::<(&Agent, &Position, &Knowledge)>()
        .iter()
        .map(|(entity, (agent, pos, _))| (agent.id, ((pos.x, pos.y), entity)))
        .collect();
    let index = ChunkIndex::from_entries(workers.iter().map(|(&id, &(tile, _))| (tile, id)));
    let hidden: Vec<_> = res
        .ore
        .deposits
        .iter()
        .filter(|(_, d)| !d.revealed)
        .map(|(&t, d)| (t, d.kind))
        .collect();
    for (tile, kind) in hidden {
        let Some(def) = installed_or_bundled().outcrop(kind) else {
            continue;
        };
        let worker = index
            .within_square(tile, def.reveal_radius)
            .filter_map(|(_, id)| workers.get(id).map(|(_, entity)| (*id, *entity)))
            .filter(|(_, entity)| {
                world
                    .get::<&Knowledge>(*entity)
                    .is_ok_and(|k| k.practises(def.knowledge))
            })
            .min_by_key(|(id, _)| *id);
        if let Some((_, entity)) = worker {
            reveal(world, res, entity, tile, def, None);
        }
    }
}
/// Called only after an ordinary quarry batch physically produced stone.
pub fn reveal_from_quarry(
    world: &World,
    res: &mut SimResources,
    worker: Entity,
    quarry: (u32, u32),
) {
    let hidden: Vec<_> = res
        .ore
        .deposits
        .iter()
        .filter(|(_, d)| !d.revealed)
        .map(|(&t, d)| (t, d.kind))
        .collect();
    for (tile, kind) in hidden {
        let Some(def) = installed_or_bundled().outcrop(kind) else {
            continue;
        };
        if chebyshev(tile, quarry) <= def.quarry_radius {
            reveal(world, res, worker, tile, def, Some(quarry));
        }
    }
}
fn reveal(
    world: &World,
    res: &mut SimResources,
    worker: Entity,
    tile: (u32, u32),
    def: &OutcropDef,
    quarry: Option<(u32, u32)>,
) {
    let knowledge = if quarry.is_some() {
        def.quarry_knowledge
    } else {
        def.knowledge
    };
    let Ok(practice) = world.get::<&Knowledge>(worker) else {
        return;
    };
    let Ok(agent) = world.get::<&Agent>(worker) else {
        return;
    };
    let Ok(position) = world.get::<&Position>(worker) else {
        return;
    };
    let worker_tile = (position.x, position.y);
    if !practice.practises(knowledge)
        || quarry.map_or_else(
            || chebyshev(worker_tile, tile) > def.reveal_radius,
            |q| chebyshev(worker_tile, q) > 1,
        )
    {
        return;
    }
    let Ok(index) = u32::try_from(res.chronicle.len()) else {
        return;
    };
    if index == u32::MAX {
        return;
    }
    let Some(deposit) = res.ore.deposits.get_mut(&tile).filter(|d| !d.revealed) else {
        return;
    };
    let witness = OreWitness {
        kind: deposit.kind,
        tile,
        worker: agent.id,
        worker_tile,
        tick: res.current_tick,
        knowledge,
        proficiency: practice.proficiency(knowledge),
        required: knowledge::defs().threshold(knowledge),
        quarry,
    };
    deposit.revealed = true;
    let entry = ChronicleEntry::new(res.current_tick, ChronicleKind::OreDiscovered)
        .subject(agent.id)
        .value(i64::from(deposit.kind.code()));
    let actual = res
        .chronicle
        .push_caused(entry, Some(CauseRef::Ore { witness: index }));
    res.ore.witnesses.insert(actual, witness);
    res.ore.touch();
}
/// Validate saved physical state and the exact one-to-one Chronicle evidence relation.
pub fn validate_state_at_boundary(res: &SimResources) -> Result<(), String> {
    if res.rules.colony.is_none() && !res.ore.is_empty() {
        return Err("baseline world has ore state".into());
    }
    let catalog = installed_or_bundled();
    let in_bounds = |t: (u32, u32)| t.0 < res.tile_grid.width && t.1 < res.tile_grid.height;
    let mut revealed = BTreeSet::new();
    for (&index, w) in &res.ore.witnesses {
        let Some(def) = catalog.outcrop(w.kind) else {
            return Err("unknown witness ore kind".into());
        };
        let Some(deposit) = res.ore.deposits.get(&w.tile) else {
            return Err("ore witness lacks its deposit".into());
        };
        let knowledge = if w.quarry.is_some() {
            def.quarry_knowledge
        } else {
            def.knowledge
        };
        if deposit.kind != w.kind
            || !deposit.revealed
            || !revealed.insert(w.tile)
            || w.knowledge != knowledge
            || w.required != catalog.knowledge_defs().threshold(knowledge)
            || !in_bounds(w.worker_tile)
            || w.quarry.map_or_else(
                || chebyshev(w.worker_tile, w.tile) > def.reveal_radius,
                |q| chebyshev(w.worker_tile, q) > 1,
            )
            || w.proficiency < w.required
            || w.proficiency > 1000
            || w.tick > res.current_tick
            || w.quarry
                .is_some_and(|q| !in_bounds(q) || chebyshev(q, w.tile) > def.quarry_radius)
            || res.chronicle.cause_of(index) != Some(CauseRef::Ore { witness: index })
            || res.chronicle.entry(index).is_none_or(|e| {
                e.kind != ChronicleKind::OreDiscovered
                    || e.subject != Some(w.worker)
                    || e.tick != w.tick
                    || e.value != i64::from(w.kind.code())
            })
        {
            return Err("invalid ore discovery witness".into());
        }
    }
    let mut clusters = BTreeMap::new();
    for (&tile, d) in &res.ore.deposits {
        let Some(def) = catalog.outcrop(d.kind) else {
            return Err("unknown deposit kind".into());
        };
        if def.status != knowledge::ContentStatus::Connected
            || !in_bounds(tile)
            || !in_bounds(d.cluster)
            || chebyshev(tile, d.cluster) > 1
            || d.remaining > def.units
            || d.revealed != revealed.contains(&tile)
            || res
                .ore
                .deposits
                .get(&d.cluster)
                .is_none_or(|anchor| anchor.kind != d.kind || anchor.cluster != d.cluster)
        {
            return Err("invalid physical ore deposit".into());
        }
        *clusters.entry((d.kind, d.cluster)).or_insert(0usize) += 1;
    }
    let mut worlds = BTreeMap::new();
    for ((kind, _), count) in clusters {
        let Some(def) = catalog.outcrop(kind) else {
            return Err("unknown cluster kind".into());
        };
        if !(usize::from(def.cluster.0)..=usize::from(def.cluster.1)).contains(&count) {
            return Err("invalid ore cluster size".into());
        }
        let count = worlds.entry(kind).or_insert(0usize);
        *count += 1;
        if *count > usize::from(def.per_world.1) {
            return Err("too many ore clusters".into());
        }
    }
    for (i, entry) in res.chronicle.entries().iter().enumerate() {
        if entry.kind == ChronicleKind::OreDiscovered
            && u32::try_from(i)
                .ok()
                .is_none_or(|id| !res.ore.witnesses.contains_key(&id))
        {
            return Err("ore event lacks immutable evidence".into());
        }
    }
    Ok(())
}
