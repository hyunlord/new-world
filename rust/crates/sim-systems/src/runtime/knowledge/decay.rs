//! KnowledgeDecaySystem: what goes unused for six days fades a little every hour; everyday lore is used by living.
//!
//! Priority: 162 (`KNOWLEDGE_DECAY_PRIORITY`). Interval: the rule's `decay_interval` (60 ticks). Gate: `ColonyRule::knowledge`.
//! Reads: `Agent`, `AgentState`, `Knowledge`; `rules` (knowledge, calendar), `current_tick`
//! Writes: `Knowledge` (proficiency, entries at 0 removed, lore stamped used); `SimResources.colony.tallies` (forgotten)
//! Events: none
//!
//! M4 goal 3.3 (ADR 0025). In `AgentId` order: a person seen foraging or
//! eating (`Seeking`/`Consuming` food) uses the forage lore, one seen
//! drinking the water lore (sampled at each step: daily life, no job is
//! gated by them). Then every entry unused for more than `unused_days`
//! loses `decay[source]` ‰, never below `floor[source]`; an entry at 0 is
//! removed. A practitioner bit lost here is counted in
//! `ColonyTallies::forgotten` (and `forgotten_seeded` for seeded entries).
use hecs::World;
use sim_core::colony::knowledge::{bit, gates, ids_of, Knowledge, Lore, Source};
use sim_core::components::{Agent, AgentState, TargetKind};
use sim_engine::{RuntimeSystem, SimResources};

use super::rule;

/// Priority of [`KnowledgeDecaySystem`].
pub const KNOWLEDGE_DECAY_PRIORITY: u32 = 162;

/// See the module header.
#[derive(Debug, Default)]
pub struct KnowledgeDecaySystem;

impl RuntimeSystem for KnowledgeDecaySystem {
    fn name(&self) -> &str {
        "KnowledgeDecaySystem"
    }

    fn priority(&self) -> u32 {
        KNOWLEDGE_DECAY_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        60
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = rule(resources) else {
            return;
        };
        let tick = resources.current_tick;
        let unused = rule.unused_days;
        let g = gates();
        let (forage, water) = (g.lore(Lore::Forage), g.lore(Lore::Water));
        let mut forgotten = 0u64;
        let mut forgotten_seeded = 0u64;
        let mut rows: Vec<(u64, hecs::Entity)> = world
            .query::<(&Agent, &Knowledge)>()
            .iter()
            .map(|(e, (a, _))| (a.id, e))
            .collect();
        rows.sort_unstable();
        for (_, entity) in rows {
            let state = world.get::<&AgentState>(entity).map(|s| *s).ok();
            let Ok(mut k) = world.get::<&mut Knowledge>(entity) else {
                continue;
            };
            let lore = match state.and_then(|s| s.target()) {
                Some(TargetKind::Food | TargetKind::GatherFood) => forage,
                Some(TargetKind::Water) => water,
                _ => 0,
            };
            for id in ids_of(lore) {
                k.touch(id, tick);
            }
            let held = k.practice();
            let seeded: u32 = k
                .entries()
                .iter()
                .filter(|e| e.source == Source::Seeded)
                .fold(0, |m, e| m | bit(e.id));
            if k.decay(tick, unused, |s| rule.decay_of(s), |s| rule.floor_of(s)) {
                let lost = held & !k.practice();
                forgotten += u64::from(lost.count_ones());
                forgotten_seeded += u64::from((lost & seeded).count_ones());
            }
        }
        let t = &mut resources.colony.tallies;
        t.forgotten += forgotten;
        t.forgotten_seeded += forgotten_seeded;
    }
}
