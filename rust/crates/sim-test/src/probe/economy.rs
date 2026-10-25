//! M2 measurement (goal §10, E1–E7): the economy's timeline in a colony probe run.
//!
//! Measurement only: reads the engine after each tick, never writes it. The
//! report is the `m2` block of the colony summary.
use serde_json::{json, Value};
use sim_core::colony::{Command, ItemKind, JobKind, RecipeKind, SkillKind, Skills, WorkType};
use sim_core::components::{Agent, AgentState, ResourceKind, TargetKind};
use sim_engine::SimEngine;

const DAY: u64 = 1_440;

/// Per-run observer (see the module header).
#[derive(Debug, Default)]
pub struct Economy {
    blades6_tick: Option<u64>,
    axes2_tick: Option<u64>,
    firewood_stock: Vec<u64>,
    firewood_ground: Vec<u64>,
    settlement_wood: Vec<u64>,
    snapshots: Vec<Value>,
    gather_wood_agent_ticks: u64,
}

/// `(chop = 0 agent, craft = 1 agent)` from the command log (the M2-U priorities).
fn priority_agents(engine: &SimEngine) -> (Option<u64>, Option<u64>) {
    let mut chop0 = None;
    let mut craft1 = None;
    for record in &engine.resources.colony.command_log {
        if let Command::SetWorkPriority { agent, work, level } = record.command {
            if !record.result.accepted {
                continue;
            }
            match (work, level) {
                (WorkType::Chop, 0) => chop0 = Some(agent),
                (WorkType::Craft, 1) => craft1 = Some(agent),
                _ => {}
            }
        }
    }
    (chop0, craft1)
}

fn skills_of(engine: &SimEngine, agent: u64) -> Option<Skills> {
    engine
        .world
        .query::<(&Agent, Option<&Skills>)>()
        .iter()
        .find(|(_, (a, _))| a.id == agent)
        .map(|(_, (_, s))| s.copied().unwrap_or_default())
}

/// Finished crafts whose recipe is of work type Craft, by `agent` (or everyone).
fn craft_crafts(c: &sim_core::colony::ColonyState, agent: Option<u64>) -> u64 {
    c.tallies
        .craft_quality
        .iter()
        .filter(|((a, r), _)| {
            agent.is_none_or(|x| x == *a)
                && sim_core::colony::content::installed_or_bundled()
                    .recipe_by_code(*r)
                    .is_some_and(|recipe| recipe.work == WorkType::Craft)
        })
        .map(|(_, (n, _))| n)
        .sum()
}

impl Economy {
    /// A fresh observer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe the engine after a tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let c = &engine.resources.colony;
        let tick = engine.current_tick();
        // E8: agents whose state targets the decision system's GatherWood (must stay 0 in the colony).
        self.gather_wood_agent_ticks += engine
            .world
            .query::<&AgentState>()
            .iter()
            .filter(|(_, s)| s.target() == Some(TargetKind::GatherWood))
            .count() as u64;
        if self.blades6_tick.is_none() && c.item_ledger.crafted_of(ItemKind::FlintBlade) >= 6 {
            self.blades6_tick = Some(tick);
        }
        if self.axes2_tick.is_none() && c.item_ledger.crafted_of(ItemKind::StoneAxe) >= 2 {
            self.axes2_tick = Some(tick);
        }
        if tick.is_multiple_of(DAY) {
            let stocked: u64 = c
                .ground_items
                .iter()
                .filter(|(t, s)| s.kind == ItemKind::Firewood && c.zone_of_tile.contains_key(t))
                .map(|(_, s)| u64::from(s.count))
                .sum();
            self.firewood_stock.push(stocked);
            self.firewood_ground
                .push(c.ground_total(ItemKind::Firewood));
            let mut ids: Vec<_> = engine.resources.settlements.keys().copied().collect();
            ids.sort_unstable();
            let wood: u64 = ids
                .iter()
                .filter_map(|id| engine.resources.settlements.get(id))
                .map(|s| u64::from(s.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0)))
                .sum();
            self.settlement_wood.push(wood);
            let day = tick / DAY;
            if day == 3 || day == 10 || day == 20 {
                self.snapshots.push(self.snapshot(engine, day));
            }
        }
    }

    fn snapshot(&self, engine: &SimEngine, day: u64) -> Value {
        let (_, craft1) = priority_agents(engine);
        let c = &engine.resources.colony;
        let knap = RecipeKind::KnapBlade.code();
        let (made, sum) = craft1
            .and_then(|a| c.tallies.craft_quality.get(&(a, knap)).copied())
            .unwrap_or((0, 0));
        let skills = craft1.and_then(|a| skills_of(engine, a));
        json!({
            "day": day,
            "craft1_knap_level": skills.map(|s| s.level(SkillKind::Knap)),
            "craft1_woodwork_level": skills.map(|s| s.level(SkillKind::Woodwork)),
            "craft1_blades": made,
            "craft1_blade_quality_sum": sum,
        })
    }

    /// The `m2` block.
    pub fn report(&self, engine: &SimEngine) -> Value {
        let c = &engine.resources.colony;
        let (chop0, craft1) = priority_agents(engine);
        let taken = |agent: Option<u64>, code: u8| {
            agent.map_or(0, |a| {
                c.tallies.taken_by.get(&(a, code)).copied().unwrap_or(0)
            })
        };
        let craft_code = JobKind::Craft {
            workshop: 0,
            bill: 0,
            at: (0, 0),
        }
        .code();
        let chop_code = JobKind::Chop { tree: (0, 0) }.code();
        let crafts_all: u64 = c
            .tallies
            .taken_by
            .iter()
            .filter(|((_, k), _)| *k == craft_code)
            .map(|(_, n)| n)
            .sum();
        json!({
            "blades6_tick": self.blades6_tick,
            "gather_wood_agent_ticks": self.gather_wood_agent_ticks,
            "axes2_tick": self.axes2_tick,
            "chops_with_axe": c.tallies.chops_with_axe,
            "firewood_stock_by_day": self.firewood_stock,
            "firewood_ground_by_day": self.firewood_ground,
            "settlement_wood_by_day": self.settlement_wood,
            "chop0_agent": chop0,
            "chop0_chop_jobs": taken(chop0, chop_code),
            "craft1_agent": craft1,
            "craft1_craft_jobs": taken(craft1, craft_code),
            "craft_jobs_all": crafts_all,
            // Finished crafts of work type Craft (knap, haft), by the Craft = 1
            // adult and by everyone (E4); split firewood is work type Split.
            "craft1_craft_crafts": craft_crafts(c, craft1),
            "craft_crafts_all": craft_crafts(c, None),
            "craft1_taken_by_kind": craft1.map(|a| c.tallies.taken_by.iter()
                .filter(|((x, _), _)| *x == a)
                .map(|((_, k), n)| (k.to_string(), *n))
                .collect::<std::collections::BTreeMap<_, _>>()),
            "supply_jobs_all": c.tallies.taken_by.iter().filter(|((_, k), _)| *k == 6).map(|(_, n)| n).sum::<u64>(),
            "snapshots": self.snapshots,
            "crafted": {
                "flint_blade": c.item_ledger.crafted_of(ItemKind::FlintBlade),
                "stone_axe": c.item_ledger.crafted_of(ItemKind::StoneAxe),
                "firewood": c.item_ledger.crafted_of(ItemKind::Firewood),
            },
            "burned_firewood": c.item_ledger.burned_of(ItemKind::Firewood),
        })
    }
}
