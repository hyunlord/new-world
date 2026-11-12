//! OreDiscoverySystem: reveal hidden surface deposits from actual personal practice.
//! Priority: 128; interval: 8 ticks; colony only.
//! Reads: positions, knowledge, authored discovery radii and hidden deposits.
//! Writes: ore visibility and immutable witnesses; C2-b mined batches (`mine_batch`, called by
//! JobExecuteSystem) deplete deposits and record miners. Events: OreDiscovered.
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
/// C2-b: one completed ordinary mining batch at `node`: up to the quarry rule's batch of
/// the outcrop's catalog yield lands at the deposit (or beside the worker), the deposit
/// depletes, the ledger creates exactly that many, and the miner is recorded. An
/// exhausted deposit keeps its geology and witness; only its mark is removed.
pub fn mine_batch(
    res: &mut SimResources,
    rule: &sim_core::rules::ColonyRule,
    miner: sim_core::components::AgentId,
    node: (u32, u32),
    at: (u32, u32),
) -> bool {
    let batch = rule.economy.map_or(4, |e| e.quarry_batch).max(1);
    let Some(deposit) = res.ore.deposits.get_mut(&node).filter(|d| d.revealed) else {
        return false;
    };
    let Some(def) = installed_or_bundled().outcrop(deposit.kind) else {
        return false;
    };
    let n = deposit.remaining.min(u16::from(batch));
    if n == 0 {
        return false;
    }
    deposit.remaining -= n;
    if deposit.remaining == 0 {
        res.colony.designations.remove(&node);
    }
    res.ore.touch();
    res.colony.touch(false);
    let spec = sim_core::colony::ItemSpec::raw(def.yield_item, Some(def.material));
    res.colony.item_ledger.create(spec.kind, u64::from(n));
    let drop = if crate::runtime::jobs::items::can_hold_items(res, node) {
        node
    } else {
        at
    };
    crate::runtime::jobs::items::put_items(res, drop, spec, n);
    let mined = res.metal.mined.entry((spec.kind, miner)).or_insert(0);
    *mined = mined.saturating_add(u32::from(n));
    true
}
/// Whether `item` is a metal-chain input whose sources are recorded (`ore`/`ingot` tags).
pub fn tracked(item: sim_core::colony::ItemKind) -> bool {
    item.definition()
        .tags
        .iter()
        .any(|t| t == "ore" || t == "ingot")
}

/// C2-b: after a committed craft, record the first output of each `ingot`-tagged item with
/// its recipe inputs' recorded sources. The line's cause is the first imported input's
/// `Traded` entry, else the caused first-ingot line of an ingot input (tin through a local smelt).
pub fn note_first_ingot(
    res: &mut SimResources,
    recipe: &sim_core::colony::Recipe,
    worker: sim_core::components::AgentId,
    workshop: (u32, u32),
) {
    use sim_core::colony::ore::{MetalInput, MetalWitness};
    let item = recipe.output.0;
    let first = |w: &MetalWitness, kind| matches!(w, MetalWitness::FirstIngot { item, .. } if *item == kind);
    if !item.definition().tags.iter().any(|t| t == "ingot")
        || res.metal.witnesses.values().any(|w| first(w, item))
    {
        return;
    }
    let Ok(index) = u32::try_from(res.chronicle.len()) else {
        return;
    };
    let mut cause = None;
    let mut inputs = Vec::new();
    for &(input, count) in &recipe.inputs {
        let mut record = MetalInput {
            item: input,
            count,
            miners: Vec::new(),
            imports: Vec::new(),
        };
        if tracked(input) {
            record.miners = res
                .metal
                .mined
                .range((input, 0)..=(input, u64::MAX))
                .map(|(&(_, miner), &units)| (miner, units))
                .collect();
            for (i, import) in res.metal.imports.iter().enumerate() {
                if import.item == input {
                    record.imports.push(u32::try_from(i).unwrap_or(u32::MAX));
                    cause = cause.or(Some(CauseRef::Entry(import.traded)));
                }
            }
            if cause.is_none() {
                cause = res
                    .metal
                    .witnesses
                    .iter()
                    .find(|(&i, w)| first(w, input) && res.chronicle.cause_of(i).is_some())
                    .map(|(&i, _)| CauseRef::Entry(i));
            }
        }
        inputs.push(record);
    }
    let miner = inputs
        .iter()
        .flat_map(|i| i.miners.iter().copied())
        .max_by_key(|&(id, units)| (units, std::cmp::Reverse(id)))
        .map(|(id, _)| id);
    let entry = ChronicleEntry::new(res.current_tick, ChronicleKind::FirstIngot)
        .subject(worker)
        .other(miner)
        .cause(recipe.id.code())
        .value(i64::from(item.code()));
    let actual = res.chronicle.push_caused(entry, cause);
    debug_assert_eq!(actual, index);
    res.metal.witnesses.insert(
        actual,
        MetalWitness::FirstIngot {
            worker,
            workshop,
            recipe: recipe.id,
            item,
            inputs,
        },
    );
}

/// C2-b: the first successful chop with a `tool`-tagged item kind, caused by the first-ingot
/// line of the ingot a recipe casts it from (a data lookup, not the used unit's own history).
pub fn note_first_tool_use(
    res: &mut SimResources,
    worker: sim_core::components::AgentId,
    item: sim_core::colony::ItemKind,
) {
    use sim_core::colony::ore::MetalWitness;
    if res.rules.colony.is_none()
        || !item.definition().tags.iter().any(|t| t == "tool")
        || res
            .metal
            .witnesses
            .values()
            .any(|w| matches!(w, MetalWitness::FirstToolUsed { item: used, .. } if *used == item))
    {
        return;
    }
    let ingots: Vec<_> = sim_core::colony::recipes()
        .iter()
        .filter(|r| r.output.0 == item)
        .flat_map(|r| r.inputs.iter().map(|(input, _)| *input))
        .collect();
    let cause = res
        .metal
        .witnesses
        .iter()
        .find(|(_, w)| matches!(w, MetalWitness::FirstIngot { item: ingot, .. } if ingots.contains(ingot)))
        .map(|(&i, _)| CauseRef::Entry(i));
    let entry = ChronicleEntry::new(res.current_tick, ChronicleKind::FirstToolUsed)
        .subject(worker)
        .cause(item.code());
    let actual = res.chronicle.push_caused(entry, cause);
    res.metal
        .witnesses
        .insert(actual, MetalWitness::FirstToolUsed { worker, item });
}

/// Whether no deposit in the world, hidden or exhausted included, yields an item tagged `tag`.
pub fn lacks_outcrop_tag(res: &SimResources, tag: &str) -> bool {
    let catalog = installed_or_bundled();
    !res.ore.deposits.values().any(|d| {
        catalog
            .outcrop(d.kind)
            .is_some_and(|def| def.yield_item.definition().tags.iter().any(|t| t == tag))
    })
}

/// C2-b `tin_shortage` card trigger: an adult practises the alloy and no tin outcrop exists.
pub fn tin_shortage(world: &World, res: &SimResources) -> bool {
    res.rules.colony.is_some()
        && knowledge::defs()
            .id_of("bronze_alloy")
            .is_some_and(|alloy| {
                crate::runtime::knowledge::anyone_practises(world, knowledge::bit(alloy))
            })
        && lacks_outcrop_tag(res, "metal_tin")
}

/// The lowest-id waiting caravan whose goods include a `metal_tin`-tagged item.
pub fn tin_caravan(res: &SimResources) -> Option<u32> {
    res.colony
        .neighbours
        .caravans
        .iter()
        .filter(|c| {
            matches!(
                c.state,
                sim_core::colony::neighbours::VisitState::Waiting { .. }
            )
        })
        .filter(|c| {
            c.goods
                .iter()
                .any(|g| g.count > 0 && g.kind.definition().tags.iter().any(|t| t == "metal_tin"))
        })
        .map(|c| c.id)
        .min()
}

/// C2-b: saved metal records refer to real Chronicle lines of the matching kind, imports to
/// `Traded` entries, mined items to catalog outcrop yields, and each first event is unique.
pub fn validate_metal_at_boundary(res: &SimResources) -> Result<(), String> {
    use sim_core::colony::ore::MetalWitness;
    if res.rules.colony.is_none() && !res.metal.is_empty() {
        return Err("baseline world has metal state".into());
    }
    let catalog = installed_or_bundled();
    let kind_at = |i: u32| res.chronicle.entry(i).map(|e| e.kind);
    for ((item, _), units) in &res.metal.mined {
        let yields = catalog.outcrops().any(|d| d.yield_item == *item);
        if !yields || *units == 0 {
            return Err("invalid mined metal record".into());
        }
    }
    for import in &res.metal.imports {
        if kind_at(import.traded) != Some(ChronicleKind::Traded) || import.count == 0 {
            return Err("metal import lacks its trade".into());
        }
    }
    let mut firsts = BTreeSet::new();
    for (&index, witness) in &res.metal.witnesses {
        let (kind, item) = match witness {
            MetalWitness::FirstIngot { item, inputs, .. } => {
                let imports = u32::try_from(res.metal.imports.len()).unwrap_or(u32::MAX);
                if inputs
                    .iter()
                    .flat_map(|i| &i.imports)
                    .any(|&i| i >= imports)
                {
                    return Err("first ingot cites a missing import".into());
                }
                (ChronicleKind::FirstIngot, *item)
            }
            MetalWitness::FirstToolUsed { item, .. } => (ChronicleKind::FirstToolUsed, *item),
        };
        if kind_at(index) != Some(kind) || !firsts.insert((kind, item)) {
            return Err("invalid metal witness".into());
        }
    }
    Ok(())
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
