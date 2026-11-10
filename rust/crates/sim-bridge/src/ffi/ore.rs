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
        return d;
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
