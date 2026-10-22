//! OralTransmissionSystem: once each evening at the hearth gathering, each listener hears one keyed teller and may learn one thing.
//!
//! Priority: 160 (`ORAL_PRIORITY`). Interval: 1,440 ticks, at the rule's `oral_minute` of the clock (`tick_offset`). Gate: `ColonyRule::knowledge`.
//! Reads: `Agent`, `Position`, `Child`, `Lineage`, `Life`, `Knowledge`; `SimResources.colony` (rooms), `settlements` (hearths), `rules`, `world_seed`
//! Writes: `Knowledge` (listeners' new entries; fire keeping and storytelling stamped used); `SimResources.colony.tallies`
//! Events: none
//!
//! M4 goal 3.3 (ADR 0025). The gathered are the people within
//! `KnowledgeRule::gather_radius` (Chebyshev) of a hearth or standing in the
//! hall. Tellers are adults; listeners are adults and children of at least
//! `learner_min_years`. In `AgentId` order each listener is paired with a
//! keyed random other gathered teller (`STREAM_KNOWLEDGE`, subject = the
//! listener's lineage key, slot 0x1_0000); the teller's candidates are the
//! nodes it practises that the listener does not (below threshold); each is
//! told, in a keyed order (slot 0x1_0200 + id), and takes with
//! `oral_chance_permille` (+ `storytelling_bonus_permille` when the teller
//! practises storytelling; slot 0x1_0100 + id); the first that takes is
//! learned, at most one per pair per evening (goal 3.3). A listener who learns rises to
//! `oral_gain` ‰ (`Source::Oral`), so a node whose threshold is at most that
//! makes a practitioner at once, a harder one a learner; the teller is
//! stamped as using what it told. Everyone reads the
//! evening's snapshot: what is learned tonight is told from tomorrow. Every
//! pair also rehearses one node both practise (keyed, slot 0x1_0003): both
//! are stamped as using it, so shared knowledge lives while people talk
//! (DECISIONS D14). Every
//! gathered holder of fire keeping is stamped as using it (the hearth lore),
//! and a teller who practises storytelling as using that.
use hecs::World;
use sim_core::colony::knowledge::{defs, ids_of, Lore, Source};
use sim_core::rng::{keyed_below, STREAM_KNOWLEDGE};
use sim_engine::{RuntimeSystem, SimResources};

use super::{knowledge_of, people, rule, store, Person};

/// Priority of [`OralTransmissionSystem`].
pub const ORAL_PRIORITY: u32 = 160;

/// See the module header.
#[derive(Debug, Default)]
pub struct OralTransmissionSystem;

/// Whether `p` is at the evening gathering: near a hearth or in the hall.
pub fn gathered(res: &SimResources, hearths: &[(u32, u32)], radius: u32, p: &Person) -> bool {
    hearths
        .iter()
        .any(|h| crate::runtime::jobs::items::chebyshev(*h, p.pos) <= radius)
        || crate::runtime::rooms::roofed_as(res, p.pos, sim_core::rooms::RoomRole::Hall)
}

impl RuntimeSystem for OralTransmissionSystem {
    fn name(&self) -> &str {
        "OralTransmissionSystem"
    }

    fn priority(&self) -> u32 {
        ORAL_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1_440
    }

    fn tick_offset(&self, resources: &SimResources) -> u64 {
        let day = 1_440;
        let minute = rule(resources).map_or(1230, |r| u64::from(r.oral_minute)) % day;
        (minute + day - resources.rules.clock_offset_ticks % day) % day
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = rule(resources) else {
            return;
        };
        let tick = resources.current_tick;
        let seed = resources.world_seed;
        let hearths = crate::runtime::jobs::hearth::hearth_tiles(resources);
        let here: Vec<Person> = people(world, resources)
            .into_iter()
            .filter(|p| gathered(resources, &hearths, rule.gather_radius, p))
            .collect();
        resources.colony.tallies.oral_gathered += here.len() as u64;
        if here.len() < 2 {
            return;
        }
        let d = defs();
        let gates = sim_core::colony::knowledge::gates();
        let hearth_lore = gates.lore(Lore::Hearth);
        let story = gates.oral();
        // The evening's snapshot: every gathered person's knowledge before tonight.
        let before: Vec<_> = here.iter().map(|p| knowledge_of(world, p.entity)).collect();
        let mut after = before.clone();
        let mut told = 0u64;
        let mut learned = 0u64;
        let mut rehearsed = 0u64;
        for (li, listener) in here.iter().enumerate() {
            if !listener.adult && listener.years < rule.learner_min_years {
                continue;
            }
            let tellers: Vec<usize> = (0..here.len())
                .filter(|&i| i != li && here[i].adult)
                .collect();
            if tellers.is_empty() {
                continue;
            }
            let pick = keyed_below(
                seed,
                STREAM_KNOWLEDGE,
                listener.key,
                tick,
                0x1_0000,
                0,
                tellers.len() as u64,
            );
            let ti = tellers[pick as usize];
            let teller = &before[ti];
            // The pair also talk over one node both practise, and both use it
            // (rehearsal; DECISIONS D14).
            let shared: Vec<u8> = ids_of(teller.practice() & before[li].practice()).collect();
            if !shared.is_empty() {
                let pick = keyed_below(
                    seed,
                    STREAM_KNOWLEDGE,
                    listener.key,
                    tick,
                    0x1_0003,
                    0,
                    shared.len() as u64,
                );
                let node = shared[pick as usize];
                after[li].touch(node, tick);
                after[ti].touch(node, tick);
                rehearsed += 1;
            }
            let candidates: Vec<u8> = ids_of(teller.practice() & !before[li].practice()).collect();
            if candidates.is_empty() {
                continue;
            }
            told += 1;
            // Goal 3.3: each candidate is told with its own keyed chance (slot
            // 0x1_0100 + id), tried in a keyed order (slot 0x1_0200 + id); the
            // first that takes is learned — at most one per pair per evening.
            let chance = rule.oral_chance_permille
                + if teller.practice() & story != 0 {
                    rule.storytelling_bonus_permille
                } else {
                    0
                };
            if teller.practice() & story != 0 {
                if let Some(id) = ids_of(story).next() {
                    after[ti].touch(id, tick);
                }
            }
            let mut order: Vec<(u64, u8)> = candidates
                .iter()
                .map(|c| {
                    let rank = sim_core::rng::keyed_u64(
                        seed,
                        STREAM_KNOWLEDGE,
                        listener.key,
                        tick,
                        0x1_0200 + u32::from(*c),
                        0,
                    );
                    (rank, *c)
                })
                .collect();
            order.sort_unstable();
            for (_, node) in order {
                // Telling it is using it, for the teller (DECISIONS D14).
                after[ti].touch(node, tick);
                let roll = keyed_below(
                    seed,
                    STREAM_KNOWLEDGE,
                    listener.key,
                    tick,
                    0x1_0100 + u32::from(node),
                    0,
                    1000,
                );
                if roll < u64::from(chance) && d.get(node).is_some() {
                    after[li].raise_to(node, rule.oral_gain, Source::Oral, tick);
                    learned += 1;
                    break;
                }
            }
        }
        // The hearth lore: everyone gathered who holds fire keeping uses it tonight.
        for k in after.iter_mut() {
            for id in ids_of(hearth_lore) {
                if k.get(id).is_some() {
                    k.touch(id, tick);
                }
            }
        }
        for ((p, b), a) in here.iter().zip(&before).zip(after) {
            if *b != a {
                store(world, p.entity, a);
            }
        }
        let t = &mut resources.colony.tallies;
        t.oral_tellings += told;
        t.oral_learned += learned;
        t.oral_rehearsed += rehearsed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_telling_runs_at_the_oral_minute_of_the_clock() {
        let mut res = SimResources::new(4, 4, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        let offset = OralTransmissionSystem.tick_offset(&res);
        // The colony clock starts at 08:00 (offset 480): 20:30 is raw tick 750 of the day.
        assert_eq!((offset + res.rules.clock_offset_ticks) % 1440, 1230);
        assert_eq!(offset, 750);
    }
}
