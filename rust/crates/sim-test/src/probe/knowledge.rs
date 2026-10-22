//! M4 measurement (goal §10, F1–F9): knowledge in a colony probe run.
//!
//! Measurement only: reads the engine after each tick, never writes it. The
//! report is the `m4` block of the colony summary:
//! - `start`: the aggregate after tick 0 — the at-risk nodes (one holder) and
//!   their holder, the badge count, the era progress (F1);
//! - `daily`: one row per day end — per node `[practitioners, holders, status]`,
//!   the era progress ‰, adults, adult holders of flint knapping (F2), the top
//!   knapper's level (F8), the cumulative blade `(count, quality sum)` (F8), the
//!   seeded entries forgotten so far (F5), whether a once-known gate stood
//!   without a living adult practitioner while the colony had an order for it
//!   (F5 "stall"), clay pots made so far (F7);
//! - `discovered`: the first discovery tick of each node (chronicle);
//! - `learned`: every apprenticeship completion `[tick, learner, teacher, node]`;
//! - `teach_done_ticks`: the tick each apprenticeship order closed done (F3);
//! - `lost`: every `KnowledgeLost` `[tick, node, last holder]` (F10);
//! - `max_stall_days`: the longest run of consecutive stalled days (F5);
//! - `bytes_per_person`: the mean `Knowledge` size at the end (memory plan);
//! - `waits_end`: the orders waiting for a practitioner at the end
//!   (`[kind, a, b, node]`, as the bridge's `get_knowledge_waits`; F10), and
//!   `status_end`: every node's status code at the end.
use serde_json::{json, Value};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::knowledge::{self, defs, gates, Knowledge, TechStatus};
use sim_core::colony::{Designation, ItemKind, RecipeKind, SkillKind, Skills};
use sim_core::components::Child;
use sim_engine::SimEngine;

const DAY: u64 = 1_440;

/// Per-run observer (see the module header).
#[derive(Debug, Default)]
pub struct KnowledgeWatch {
    start: Value,
    daily: Vec<Value>,
    stall_run: u64,
    max_stall: u64,
    teach_done: u64,
    teach_done_ticks: Vec<u64>,
}

fn blade_quality(engine: &SimEngine) -> (u64, u64) {
    engine
        .resources
        .colony
        .tallies
        .craft_quality
        .iter()
        .filter(|((_, r), _)| *r == RecipeKind::KnapBlade.code())
        .fold((0, 0), |(n, s), (_, (c, q))| (n + c, s + q))
}

/// Whether a gate of a node that was once held now has no living adult
/// practitioner while the colony has an order that needs it (a bill with work
/// left, a site, or a mark).
fn stalled(engine: &SimEngine) -> bool {
    let res = &engine.resources;
    let c = &res.colony;
    let g = gates();
    let known_once = |mask: u32| {
        knowledge::ids_of(mask).any(|id| {
            c.tech
                .states
                .get(usize::from(id))
                .is_some_and(|s| s.status != TechStatus::Unknown)
        })
    };
    let missing = |mask: u32| {
        mask != 0
            && known_once(mask)
            && !sim_systems::runtime::knowledge::anyone_practises(&engine.world, mask)
    };
    let bills = c
        .workshops
        .values()
        .flat_map(|s| s.bills.iter())
        .any(|b| !b.suspended && b.remaining() != Some(0) && missing(g.recipe(b.recipe)));
    let sites = sim_systems::runtime::jobs::items::colony_sites(&engine.world)
        .values()
        .any(|v| missing(g.build(v.site.kind)));
    let marks = [Designation::Quarry, Designation::Dig, Designation::ClayPit]
        .into_iter()
        .any(|d| c.designations.values().any(|m| *m == d) && missing(g.designation(d)));
    bills || sites || marks
}

impl KnowledgeWatch {
    /// A fresh observer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe the state after one tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let res = &engine.resources;
        let now = res.current_tick;
        if res.rules.colony.and_then(|c| c.knowledge).is_none() {
            return;
        }
        let tech = &res.colony.tech;
        // F3: an order closes done when its learner practises the node (by
        // sessions or because the learner heard it at the fire first).
        if res.colony.tallies.teach_done > self.teach_done {
            self.teach_done = res.colony.tallies.teach_done;
            self.teach_done_ticks.push(now);
        }
        if now == 0 {
            let at_risk: Vec<Value> = tech
                .states
                .iter()
                .enumerate()
                .filter(|(_, s)| s.status == TechStatus::AtRisk)
                .map(|(i, s)| json!([defs().nodes[i].name, s.names.first()]))
                .collect();
            self.start = json!({
                "at_risk": at_risk,
                "badge": tech.at_risk(),
                "progress": tech.progress_permille,
                "adults": tech.adults,
            });
        }
        if !(now + 1).is_multiple_of(DAY) {
            return;
        }
        let knap = knowledge::id("flint_knapping").unwrap_or(0);
        let knappers = engine
            .world
            .query::<&Knowledge>()
            .without::<&Child>()
            .iter()
            .filter(|(_, k)| k.practises(knap))
            .count();
        let top_knap = engine
            .world
            .query::<&Skills>()
            .iter()
            .map(|(_, s)| s.level(SkillKind::Knap))
            .max()
            .unwrap_or(0);
        let stall = stalled(engine);
        if stall {
            self.stall_run += 1;
            self.max_stall = self.max_stall.max(self.stall_run);
        } else {
            self.stall_run = 0;
        }
        let nodes: Vec<Value> = tech
            .states
            .iter()
            .map(|s| json!([s.practitioners, s.holders, s.status.code()]))
            .collect();
        self.daily.push(json!({
            "day": (now + 1) / DAY,
            "nodes": nodes,
            "progress": tech.progress_permille,
            "adults": tech.adults,
            "knappers": knappers,
            "top_knap": top_knap,
            "blades": blade_quality(engine),
            "forgotten_seeded": res.colony.tallies.forgotten_seeded,
            "stall": stall,
            "clay_pots": res.colony.item_ledger.crafted_of(ItemKind::ClayPot),
            "raw_pots": res.colony.item_ledger.crafted_of(ItemKind::RawPot),
            "clay": res.colony.item_ledger.created_of(ItemKind::Clay),
            "ground_axes": res.colony.item_ledger.crafted_of(ItemKind::GroundAxe),
            "conditions": sim_systems::runtime::knowledge::discover::conditions(&engine.world, res),
        }));
    }

    /// The `m4` block (see the module header).
    pub fn report(&self, engine: &SimEngine) -> Value {
        let res = &engine.resources;
        let mut discovered = serde_json::Map::new();
        let mut learned = Vec::new();
        let mut lost = Vec::new();
        let mut at_risk = Vec::new();
        for e in res.chronicle.entries() {
            let name = usize::try_from(e.value)
                .ok()
                .and_then(|i| defs().nodes.get(i))
                .map_or("?", |n| n.name.as_str());
            match e.kind {
                ChronicleKind::KnowledgeDiscovered => {
                    discovered.entry(name.to_string()).or_insert(json!(e.tick));
                }
                ChronicleKind::KnowledgeLearned => {
                    learned.push(json!([e.tick, e.subject, e.other, name]));
                }
                ChronicleKind::KnowledgeLost => lost.push(json!([e.tick, name, e.subject])),
                ChronicleKind::KnowledgeAtRisk => at_risk.push(json!([e.tick, name, e.subject])),
                _ => {}
            }
        }
        let sizes: Vec<usize> = engine
            .world
            .query::<&Knowledge>()
            .iter()
            .map(|(_, k)| k.bytes())
            .collect();
        let t = &res.colony.tallies;
        json!({
            "start": self.start,
            "daily": self.daily,
            "discovered": discovered,
            "learned": learned,
            "lost": lost,
            "at_risk_events": at_risk,
            "max_stall_days": self.max_stall,
            "teach_done_ticks": self.teach_done_ticks,
            "teach": {"orders_open": res.colony.teach_orders.len(), "done": t.teach_done, "cancelled": t.teach_cancelled, "sessions": t.teach_sessions},
            "oral": {"gathered": t.oral_gathered, "tellings": t.oral_tellings, "learned": t.oral_learned, "rehearsed": t.oral_rehearsed},
            "forgotten": t.forgotten,
            "forgotten_seeded": t.forgotten_seeded,
            "observed": t.observed,
            "discoveries": t.discoveries,
            "tools_broken": t.tools_broken,
            "ford_tiles": t.ford_tiles,
            "bytes_per_person": if sizes.is_empty() { 0 } else { sizes.iter().sum::<usize>() / sizes.len() },
            "waits_end": sim_bridge::ffi::collect_knowledge_waits(&engine.world, res)
                .iter()
                .map(|w| json!([w.kind, w.a, w.b, defs().get(w.node).map_or("?", |n| n.name.as_str())]))
                .collect::<Vec<_>>(),
            "status_end": res.colony.tech.states.iter().map(|s| s.status.code()).collect::<Vec<_>>(),
            "node_names": defs().nodes.iter().map(|n| n.name.clone()).collect::<Vec<_>>(),
        })
    }
}
