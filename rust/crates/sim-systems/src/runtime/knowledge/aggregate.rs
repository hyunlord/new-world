//! TechAggregateSystem: counts each node's practitioners and holders, sets its state, warns of risk and loss, and computes the era progress.
//!
//! Priority: 164 (`TECH_AGGREGATE_PRIORITY`). Interval: 240 ticks (fixed; `KnowledgeRule::aggregate_interval` must match — a test checks). Gate: `ColonyRule::knowledge`.
//! Reads: `Agent`, `Child`, `Knowledge`; `rules` (knowledge, calendar), `current_tick`
//! Writes: `SimResources.colony.tech` (states, era progress, version), `chronicle`
//! Events: chronicle `KnowledgeAtRisk` (subject the one holder, value the node id), `KnowledgeLost` (subject the last holder, value the node id)
//!
//! M4 goal 3.4 (ADR 0026). Per node: **holders** are living people at or
//! above its threshold; **practitioners** the adult holders who used it (or
//! gained it) within `practice_days`. State: no holder → `Lost` if it was ever
//! held, else `Unknown`; one holder → `AtRisk`; else `Stable` with at least
//! `stable_at` practitioners, `Known` below (M4 DECISIONS D7: the at-risk state
//! means one holder, so the HUD names one person). Moving into `AtRisk` from
//! `Known` or `Stable` logs `KnowledgeAtRisk`; moving into `Lost` logs
//! `KnowledgeLost` with the last named holder. `names` keeps up to five
//! practitioners (holders when none practise) by `AgentId`.
//!
//! Each era's progress divides living adult coverage by its authored marker weights.
//! Empty eras stay at zero; unavailable markers do not add a hidden denominator.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::knowledge::{defs, Knowledge, TechAggregate, TechState, TechStatus};
use sim_core::components::{Agent, AgentId, Child};
use sim_engine::{RuntimeSystem, SimResources};

use super::rule;

/// Priority of [`TechAggregateSystem`].
pub const TECH_AGGREGATE_PRIORITY: u32 = 164;

/// See the module header.
#[derive(Debug, Default)]
pub struct TechAggregateSystem;

/// Compute the aggregate of `world` now, against the previous one (`prev`).
/// Returns the new aggregate and the chronicle entries of its transitions.
pub fn compute(
    world: &World,
    res: &SimResources,
    prev: &TechAggregate,
) -> (TechAggregate, Vec<ChronicleEntry>) {
    let tick = res.current_tick;
    let d = defs();
    let window = rule(res).map_or(0, |r| r.practice_days);
    let today = sim_core::colony::knowledge::day_of(tick);
    let stable_at = rule(res).map_or(3, |r| r.stable_at);
    let mut rows: Vec<(AgentId, bool, Knowledge)> = world
        .query::<(&Agent, &Knowledge, Option<&Child>)>()
        .iter()
        .map(|(_, (a, k, c))| (a.id, c.is_none(), k.clone()))
        .collect();
    rows.sort_by_key(|r| r.0);
    let adults = world.query::<&Agent>().without::<&Child>().iter().count() as u32;
    let mut out = TechAggregate {
        states: Vec::with_capacity(d.nodes.len()),
        adults: u16::try_from(adults).unwrap_or(u16::MAX),
        ..prev.clone()
    };
    out.states.clear();
    let mut events = Vec::new();
    let mut weighted = vec![0u64; d.eras.eras.len()];
    let mut totals = vec![0u64; d.eras.eras.len()];
    for node in &d.nodes {
        let mut holders = Vec::new();
        let mut practitioners = Vec::new();
        for (id, adult, k) in &rows {
            if !k.practises(node.id) {
                continue;
            }
            holders.push(*id);
            let recent = k
                .get(node.id)
                .is_some_and(|e| u64::from(today.saturating_sub(e.last_used)) <= window);
            if *adult && recent {
                practitioners.push(*id);
            }
        }
        let before = prev
            .states
            .get(usize::from(node.id))
            .cloned()
            .unwrap_or_default();
        let status = match holders.len() {
            0 if before.status != TechStatus::Unknown => TechStatus::Lost,
            0 => TechStatus::Unknown,
            1 => TechStatus::AtRisk,
            _ if practitioners.len() >= usize::from(stable_at) => TechStatus::Stable,
            _ => TechStatus::Known,
        };
        let names: Vec<AgentId> = if practitioners.is_empty() {
            &holders
        } else {
            &practitioners
        }
        .iter()
        .take(5)
        .copied()
        .collect();
        let changed = status != before.status;
        if changed
            && status == TechStatus::AtRisk
            && matches!(before.status, TechStatus::Known | TechStatus::Stable)
        {
            if let Some(holder) = holders.first() {
                events.push(
                    ChronicleEntry::new(tick, ChronicleKind::KnowledgeAtRisk)
                        .subject(*holder)
                        .value(i64::from(node.id)),
                );
            }
        }
        if changed && status == TechStatus::Lost {
            let mut entry =
                ChronicleEntry::new(tick, ChronicleKind::KnowledgeLost).value(i64::from(node.id));
            if let Some(last) = before.names.first() {
                entry = entry.subject(*last);
            }
            events.push(entry);
        }
        if node.marker_weight > 0 {
            let order = usize::from(node.era.code());
            if order < totals.len() {
                let weight = u64::from(node.marker_weight);
                totals[order] += weight;
                if adults > 0 {
                    let coverage =
                        (1000 * practitioners.len() as u64 / u64::from(adults)).min(1000);
                    weighted[order] += weight * coverage;
                }
            }
        }
        out.states.push(TechState {
            practitioners: u16::try_from(practitioners.len()).unwrap_or(u16::MAX),
            holders: u16::try_from(holders.len()).unwrap_or(u16::MAX),
            status,
            last_change_tick: if changed {
                tick
            } else {
                before.last_change_tick
            },
            names,
        });
    }
    out.era_progress = weighted
        .iter()
        .zip(&totals)
        .map(|(weight, total)| u16::try_from(weight / total.max(&1)).unwrap_or(1000))
        .collect();
    out.current_era = d.eras.current(&out.era_progress);
    // The historical scalar remains the first post-founder era's progress.
    out.progress_permille = out.era_progress.get(1).copied().unwrap_or(0);
    // The first measurement establishes an era; only later measurements can change it.
    if !prev.era_progress.is_empty() && out.current_era != prev.current_era {
        events.push(
            ChronicleEntry::new(tick, ChronicleKind::EraChanged)
                .cause(u32::from(out.current_era.1))
                .value(i64::from(out.current_era.0)),
        );
    }
    if out.states != prev.states
        || out.progress_permille != prev.progress_permille
        || out.adults != prev.adults
        || out.era_progress != prev.era_progress
        || out.current_era != prev.current_era
    {
        out.version = prev.version + 1;
    }
    (out, events)
}

impl RuntimeSystem for TechAggregateSystem {
    fn name(&self) -> &str {
        "TechAggregateSystem"
    }

    fn priority(&self) -> u32 {
        TECH_AGGREGATE_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        240
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        if rule(resources).is_none() {
            return;
        }
        let (next, events) = compute(world, resources, &resources.colony.tech);
        resources.colony.tech = next;
        for e in events {
            resources.chronicle.push(e);
        }
    }
}
