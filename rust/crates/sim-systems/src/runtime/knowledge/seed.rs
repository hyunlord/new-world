//! The founders' knowledge (M4 goal 3.1, ADR 0024): who knows what at the start.
//!
//! Called by the colony worldgen after the founders are made adults. Every
//! founder gets a `Knowledge`. Nodes seeded `All` go to every founder at 600 ‰.
//! A node seeded `Count(n)` goes to `n` of every 10 founders (rounded, at
//! least 1; adults only): founders are ranked by a keyed draw
//! (`STREAM_KNOWLEDGE`, subject = lineage key, slot = node id), those who
//! already practise every prerequisite of the node first (so a hafter knaps,
//! a door-maker builds log walls; M4 DECISIONS D9), and the first `n` get it
//! at 500 + a keyed draw below 201 ‰ (slot 0x100 + node id). Nodes are seeded
//! in id order, so a prerequisite listed earlier is already placed.
use sim_core::colony::knowledge::{defs, Knowledge, Seeding, Source};
use sim_core::rng::{keyed_below, keyed_u64, STREAM_KNOWLEDGE};
use sim_engine::SimEngine;

use super::people;

/// Proficiency ‰ of a node seeded to every founder.
pub const SEED_ALL: u16 = 600;
/// Lowest proficiency ‰ of a node seeded to some founders (the highest is +200).
pub const SEED_MIN: u16 = 500;

/// How many of `founders` know a node seeded `Count(n)` (n per 10, rounded, ≥ 1).
pub const fn seeded_count(n: u8, founders: usize) -> usize {
    let c = (n as usize * founders + 5) / 10;
    if c == 0 && n > 0 {
        1
    } else {
        c
    }
}

/// Give every founder of a fresh colony world its knowledge (see the module header).
pub fn seed_founders(engine: &mut SimEngine) {
    if super::rule(&engine.resources).is_none() {
        return;
    }
    let seed = engine.resources.world_seed;
    let founders = people(&engine.world, &engine.resources);
    let mut know: Vec<Knowledge> = vec![Knowledge::default(); founders.len()];
    for node in &defs().nodes {
        match node.seed {
            Seeding::Nobody => {}
            Seeding::All => {
                for k in &mut know {
                    k.set(node.id, SEED_ALL, Source::Seeded, 0);
                }
            }
            Seeding::Count(n) => {
                let mut ranked: Vec<(bool, u64, usize)> = founders
                    .iter()
                    .enumerate()
                    .filter(|(_, p)| p.adult)
                    .map(|(i, p)| {
                        let has_prereqs = node.prereqs.iter().all(|r| know[i].practises(*r));
                        let draw =
                            keyed_u64(seed, STREAM_KNOWLEDGE, p.key, 0, u32::from(node.id), 0);
                        (!has_prereqs, draw, i)
                    })
                    .collect();
                ranked.sort_unstable();
                for &(_, _, i) in ranked.iter().take(seeded_count(n, founders.len())) {
                    let extra = keyed_below(
                        seed,
                        STREAM_KNOWLEDGE,
                        founders[i].key,
                        0,
                        0x100 + u32::from(node.id),
                        0,
                        201,
                    );
                    know[i].set(node.id, SEED_MIN + extra as u16, Source::Seeded, 0);
                }
            }
        }
    }
    for (p, k) in founders.iter().zip(know) {
        let _ = engine.world.insert_one(p.entity, k);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seeded_counts_scale_with_the_founders() {
        assert_eq!(
            (
                seeded_count(3, 10),
                seeded_count(1, 10),
                seeded_count(1, 3),
                seeded_count(2, 1000)
            ),
            (3, 1, 1, 200)
        );
        assert_eq!(seeded_count(0, 10), 0);
    }
}
