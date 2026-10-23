//! DiscoverySystem: curious adults who hold every prerequisite may discover a node whose settlement conditions hold.
//!
//! Priority: 163 (`DISCOVERY_PRIORITY`). Interval: 240 ticks (fixed; `KnowledgeRule::discovery_interval` must match — a test checks). Gate: `ColonyRule::knowledge`.
//! Reads: `Agent`, `Child`, `Lineage`, `Temperament`, `Knowledge`, `Skills`; `SimResources.colony` (ground items, zones, resource nodes), `settlements` (hearths), `world_seed`
//! Writes: `Knowledge` (a discovered node at `discovery_gain` ‰, `Source::Discovered`); `SimResources.colony.tallies` (discoveries), `chronicle`
//! Events: chronicle `KnowledgeDiscovered` (subject the discoverer, value the node id)
//!
//! M4 goal 3.3 (ADR 0025). Each check first evaluates every node's discovery
//! conditions once for the settlement ([`conditions`]): stockpile counts are
//! the items on stockpile cells, skill and knowledge conditions hold when any
//! living person meets them, a clay-bank condition when a clay bank lies
//! within its distance of a hearth. Candidates are adults whose novelty
//! seeking is at least `discovery_ns` or who are among the most curious
//! `discovery_top_permille` of adults (at least one; DECISIONS D19), in
//! `AgentId` order; for each node in id
//! order that has conditions, all of them true, which the candidate does not
//! practise, and whose prerequisites the candidate all practises, one keyed
//! draw (`STREAM_KNOWLEDGE`, subject = the lineage key, slot 0x2_0000 + id)
//! succeeds with odds 1 in `discovery_odds[difficulty − 1]`.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::knowledge::{bit, defs, Cond, Knowledge, Source};
use sim_core::colony::{ItemKind, NodeKind, Skills};
use sim_core::rng::{keyed_below, STREAM_KNOWLEDGE};
use sim_engine::{RuntimeSystem, SimResources};

use super::{knowledge_of, people, rule, store};

/// Priority of [`DiscoverySystem`].
pub const DISCOVERY_PRIORITY: u32 = 163;

/// See the module header.
#[derive(Debug, Default)]
pub struct DiscoverySystem;

/// Items of `kind` lying on stockpile cells (reserved or not).
pub fn stockpiled(res: &SimResources, kind: ItemKind) -> u32 {
    let c = &res.colony;
    c.ground_items
        .iter()
        .filter(|(t, s)| s.kind == kind && c.zone_of_tile.contains_key(t))
        .map(|(_, s)| u32::from(s.count))
        .sum()
}

/// Whether a clay bank lies within `d` (Chebyshev) of a hearth.
pub fn clay_bank_near(res: &SimResources, d: u32) -> bool {
    let hearths = crate::runtime::jobs::hearth::hearth_tiles(res);
    res.colony.nodes.iter().any(|(t, n)| {
        n.kind == NodeKind::ClayBank
            && hearths
                .iter()
                .any(|h| crate::runtime::jobs::items::chebyshev(*h, *t) <= d)
    })
}

/// Whether one condition holds for the settlement.
pub fn holds(world: &World, res: &SimResources, cond: Cond) -> bool {
    match cond {
        Cond::Stockpile(kind, n) => stockpiled(res, kind) >= u32::from(n),
        Cond::Skill(skill, level) => world
            .query::<&Skills>()
            .iter()
            .any(|(_, s)| s.level(skill) >= level),
        Cond::Knows(id, p) => world
            .query::<&Knowledge>()
            .iter()
            .any(|(_, k)| k.proficiency(id) >= p),
        Cond::ClayBankNear(d) => clay_bank_near(res, d),
    }
}

/// The nodes whose discovery conditions all hold now (bit `i` = node `i`;
/// a node without conditions is never set: it is not discovered).
pub fn conditions(world: &World, res: &SimResources) -> u32 {
    defs()
        .nodes
        .iter()
        .filter(|n| !n.discover.is_empty() && n.discover.iter().all(|c| holds(world, res, *c)))
        .fold(0, |m, n| m | bit(n.id))
}

impl RuntimeSystem for DiscoverySystem {
    fn name(&self) -> &str {
        "DiscoverySystem"
    }

    fn priority(&self) -> u32 {
        DISCOVERY_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        240
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = rule(resources) else {
            return;
        };
        let tick = resources.current_tick;
        let seed = resources.world_seed;
        let adults: Vec<_> = people(world, resources)
            .into_iter()
            .filter(|p| p.adult)
            .collect();
        // The top share by novelty seeking (ties by AgentId), at least one.
        let mut ranked: Vec<(std::cmp::Reverse<u16>, u64)> = adults
            .iter()
            .map(|p| (std::cmp::Reverse(p.ns), p.id))
            .collect();
        ranked.sort_unstable();
        let top = (adults.len() as u64 * u64::from(rule.discovery_top_permille))
            .div_ceil(1000)
            .max(1) as usize;
        let curious: std::collections::BTreeSet<u64> =
            ranked.iter().take(top).map(|r| r.1).collect();
        let candidates: Vec<_> = adults
            .into_iter()
            .filter(|p| p.ns >= rule.discovery_ns || curious.contains(&p.id))
            .collect();
        if candidates.is_empty() {
            return;
        }
        let open = conditions(world, resources);
        if open == 0 {
            return;
        }
        let d = defs();
        for p in candidates {
            let mut k = knowledge_of(world, p.entity);
            let mut found = Vec::new();
            for node in d.nodes.iter().filter(|n| open & bit(n.id) != 0) {
                if k.practises(node.id) || !node.prereqs.iter().all(|r| k.practises(*r)) {
                    continue;
                }
                let odds = rule.discovery_odds[usize::from(node.difficulty.clamp(1, 3) - 1)].max(1);
                let draw = keyed_below(
                    seed,
                    STREAM_KNOWLEDGE,
                    p.key,
                    tick,
                    0x2_0000 + u32::from(node.id),
                    0,
                    u64::from(odds),
                );
                if draw == 0 {
                    k.raise_to(node.id, rule.discovery_gain, Source::Discovered, tick);
                    found.push(node.id);
                }
            }
            if found.is_empty() {
                continue;
            }
            store(world, p.entity, k);
            // M5 (ADR 0030): a discovery cheers for days.
            crate::runtime::mood::note(
                world,
                resources,
                p.entity,
                crate::runtime::mood::Fact::Discovered,
            );
            for id in found {
                resources.colony.tallies.discoveries += 1;
                resources.chronicle.push(
                    ChronicleEntry::new(tick, ChronicleKind::KnowledgeDiscovered)
                        .subject(p.id)
                        .value(i64::from(id)),
                );
            }
        }
    }
}
