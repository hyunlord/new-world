//! Read-only visible surface deposits and immutable geological discovery witnesses.
use godot::prelude::*;
use sim_core::colony::content::installed_or_bundled;
use sim_engine::SimResources;
/// One revealed physical deposit; hidden deposits never cross this renderer boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutcropRow {
    /// Physical tile, also the stable renderer key.
    pub tile: (u32, u32),
    /// Selected ore family identity.
    pub kind: u32,
    /// Remaining extractable units.
    pub remaining: u16,
}
/// Collect in stable tile order without touching the simulation or its navigation cache.
pub fn collect_outcrops(res: &SimResources) -> Vec<OutcropRow> {
    if res.rules.colony.is_none() {
        return Vec::new();
    }
    res.ore
        .deposits
        .iter()
        .filter(|(_, d)| d.revealed && d.remaining > 0)
        .map(|(&tile, d)| OutcropRow {
            tile,
            kind: d.kind.code(),
            remaining: d.remaining,
        })
        .collect()
}
pub(super) fn outcrops_to_dict(res: &SimResources) -> VarDictionary {
    let rows = collect_outcrops(res);
    let ints = |v: Vec<i64>| PackedInt64Array::from(v.as_slice());
    let mut d = VarDictionary::new();
    d.set("schema_version", 1i64);
    d.set("enabled", res.rules.colony.is_some());
    d.set("tick", i64::try_from(res.current_tick).unwrap_or(i64::MAX));
    d.set(
        "version",
        i64::try_from(res.ore.version).unwrap_or(i64::MAX),
    );
    d.set(
        "xs",
        ints(rows.iter().map(|r| i64::from(r.tile.0)).collect()),
    );
    d.set(
        "ys",
        ints(rows.iter().map(|r| i64::from(r.tile.1)).collect()),
    );
    d.set(
        "kinds",
        ints(rows.iter().map(|r| i64::from(r.kind)).collect()),
    );
    d.set(
        "remaining",
        ints(rows.iter().map(|r| i64::from(r.remaining)).collect()),
    );
    d
}
pub(super) fn evidence_to_dict(res: &SimResources, index: Option<u32>) -> VarDictionary {
    let mut d = VarDictionary::new();
    let Some(w) = index.and_then(|i| res.ore.witnesses.get(&i)) else {
        return index
            .and_then(|i| metal_causes(res, i))
            .map_or(d, metal_to_dict);
    };
    let Some(def) = installed_or_bundled().outcrop(w.kind) else {
        return d;
    };
    d.set("event", "discovered");
    d.set("kind", i64::from(w.kind.code()));
    d.set("name_key", def.identity.name_key.as_str());
    d.set("worker", i64::try_from(w.worker).unwrap_or(i64::MAX));
    d.set("x", i64::from(w.tile.0));
    d.set("y", i64::from(w.tile.1));
    d.set("worker_x", i64::from(w.worker_tile.0));
    d.set("worker_y", i64::from(w.worker_tile.1));
    d.set("tick", i64::try_from(w.tick).unwrap_or(i64::MAX));
    d.set("knowledge", i64::from(w.knowledge));
    d.set("proficiency", i64::from(w.proficiency));
    d.set("required", i64::from(w.required));
    d.set(
        "method",
        if w.quarry.is_some() {
            "quarry"
        } else {
            "prospecting"
        },
    );
    if let Some((x, y)) = w.quarry {
        d.set("quarry_x", i64::from(x));
        d.set("quarry_y", i64::from(y));
    }
    d
}

/// C3-b: what a first-ingot or first-tool line names besides its people — the knowledge gating
/// the recipe and its station, the place (the template housing that station, else the station)
/// and, for an ingot, the plain inputs (fuel) whose sources are not recorded. Generic per metal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MetalCauses {
    /// `first_ingot` or `first_tool`.
    pub event: &'static str,
    /// Knowledge name keys, station gate first, without repeats.
    pub knowledge: Vec<String>,
    /// Template or station name key.
    pub place: String,
    /// Item codes of untracked inputs (not `ore`/`ingot`).
    pub inputs: Vec<u32>,
}
/// Read the immutable metal witness at a Chronicle index; absent for every other line.
pub fn metal_causes(res: &SimResources, index: u32) -> Option<MetalCauses> {
    use sim_core::colony::{knowledge::KnowledgeMask, ore::MetalWitness};
    let (event, recipe, station) = match res.metal.witnesses.get(&index)? {
        MetalWitness::FirstIngot {
            recipe, workshop, ..
        } => {
            let def = recipe.recipe()?;
            let station = res
                .colony
                .workshops
                .values()
                .find(|shop| shop.work_tile() == *workshop)
                .map_or(def.workshop, |shop| shop.kind);
            ("first_ingot", def, station)
        }
        MetalWitness::FirstToolUsed { item, .. } => {
            let def = sim_core::colony::recipes()
                .iter()
                .find(|r| r.output.0 == *item)?;
            ("first_tool", def, def.workshop)
        }
    };
    let catalog = installed_or_bundled();
    let gates = catalog.knowledge_gates();
    let nodes = &catalog.knowledge_defs().nodes;
    let mut knowledge = Vec::new();
    // A foreign-catalogue id requires every bit; it names nothing.
    for mask in [gates.build(station), gates.recipe(recipe.id)]
        .into_iter()
        .filter(|&mask| mask != KnowledgeMask::MAX)
    {
        for (id, node) in nodes.iter().enumerate() {
            let held = u8::try_from(id).is_ok_and(|id| mask.contains_all(KnowledgeMask::bit(id)));
            if held && !knowledge.contains(&node.name_key) {
                knowledge.push(node.name_key.clone());
            }
        }
    }
    let station_def = station.definition();
    let place = catalog
        .building_templates()
        .buildings
        .iter()
        .find(|t| t.stations.contains(&station_def.identity.id))
        .map_or_else(|| station_def.identity.name_key.clone(), |t| t.name_key.clone());
    let inputs = if event == "first_ingot" {
        recipe
            .inputs
            .iter()
            .filter(|(item, _)| !sim_systems::runtime::ore::tracked(*item))
            .map(|(item, _)| item.code())
            .collect()
    } else {
        Vec::new()
    };
    Some(MetalCauses {
        event,
        knowledge,
        place,
        inputs,
    })
}
fn metal_to_dict(causes: MetalCauses) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("event", causes.event);
    d.set(
        "knowledge_keys",
        PackedStringArray::from_iter(causes.knowledge.iter().map(GString::from)),
    );
    d.set("place_key", causes.place.as_str());
    d.set(
        "inputs",
        PackedInt64Array::from_iter(causes.inputs.iter().map(|&code| i64::from(code))),
    );
    d
}
