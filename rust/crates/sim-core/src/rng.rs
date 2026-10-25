//! Keyed random draws (Pass 9): every number is a pure function of its key.
//!
//! A sequential stream hands an event whichever number is next, so once an
//! intervention changes how many numbers an agent has drawn, every later draw
//! of that agent shifts. A keyed draw instead names its event: the world seed,
//! the decision (`stream`), the entity (`subject`, a lineage key or 0 for a
//! world draw), the tick, the index of the draw within the event (`slot`) and
//! the noise replicate. The same event gets the same number in both arms of a
//! fork, whatever happened elsewhere.
//!
//! The chain is the splitmix64 output function applied once per field:
//!
//! ```text
//! keyed_u64 = m(m(m(m(m(seed ^ stream) ^ subject) ^ tick) ^ slot) ^ replicate)
//! m(x)      = splitmix64 output of state x (add 0x9E3779B97F4A7C15, then the
//!             Stafford variant-13 finalizer), i.e. `MovementRng::new(x).next_u64()`
//! ```
//!
//! `m` is a bijection of `u64`, so changing any single field changes the
//! value. No counter-based generator such as Philox is needed: the property
//! required is that a draw depends on its key alone, which this chain gives
//! by construction (no state is carried between draws).
use serde::{Deserialize, Serialize};

/// Movement: whether an idle agent pauses this movement opportunity.
pub const STREAM_MOVE_PAUSE: u64 = 0x4D4F_5645_5041_5553;
/// Movement: the x step of an idle Brownian move.
pub const STREAM_MOVE_DX: u64 = 0x4D4F_5645_5F44_5800;
/// Movement: the y step of an idle Brownian move.
pub const STREAM_MOVE_DY: u64 = 0x4D4F_5645_5F44_5900;
/// Aging: a resident's founder age, lifespan draws and founder top-up.
pub const STREAM_LIFESPAN: u64 = 0x4C49_4645_5350_414E;
/// Lineage: a founder's key from its worldgen spawn index.
pub const STREAM_FOUNDER: u64 = 0x464F_554E_4445_5252;
/// Lineage: a newborn's key from its first parent's key and birth count.
pub const STREAM_NEWBORN: u64 = 0x4E45_5742_4F52_4E4B;

/// World variety (Pass 10): the hidden parameters of a world, slot = parameter
/// index, subject, tick and replicate always 0 ("WORLDVAR" in ASCII).
pub const STREAM_WORLD_VARIETY: u64 = 0x574F_524C_4456_4152;

/// Hardiness (Pass 11): a resident's frailty factor, subject = its lineage
/// key, tick, slot and replicate always 0 (an identity trait, not noise;
/// "HARDINES" in ASCII).
pub const STREAM_HARDINESS: u64 = 0x4841_5244_494E_4553;

/// Wild regions (Pass 11): the centre of region r, slot = r, subject, tick
/// and replicate 0 ("REGIONCT" in ASCII).
pub const STREAM_REGION: u64 = 0x5245_4749_4F4E_4354;

/// Wild regions (Pass 11): the bush shuffle of region r, slot = r, subject,
/// tick 0, replicate 0 ("REGBUSH_" in ASCII).
pub const STREAM_REGION_BUSH: u64 = 0x5245_4742_5553_485F;

/// Wild regions (Pass 11): the deadwood shuffle of region r, slot = r,
/// subject, tick 0, replicate 0 ("REGWOOD_" in ASCII).
pub const STREAM_REGION_WOOD: u64 = 0x5245_4757_4F4F_445F;

/// Fission (Pass 11): the rank of a member in the departing group, subject =
/// its lineage key, tick = the check's tick, replicate = the noise replicate
/// in force ("FISSION_" in ASCII).
pub const STREAM_FISSION: u64 = 0x4649_5353_494F_4E5F;

/// Fission hazard (Pass 12, `game_v18`): the daily split draw of a
/// settlement, subject = its settlement id, tick = the check's tick, slot 0,
/// replicate = the noise replicate in force ("FISSHAZD" in ASCII).
pub const STREAM_FISSION_HAZARD: u64 = 0x4649_5353_4841_5A44;

/// Region fill (Pass 12, `WorldGenConfig::region_fill`): the shuffle of the
/// extra bush candidates of region r in tier t, slot = r + (t << 16),
/// subject and replicate 0, tick = the swap index ("RFILBUSH" in ASCII).
pub const STREAM_REGION_FILL_BUSH: u64 = 0x5246_494C_4255_5348;

/// Region fill (Pass 12): the shuffle of the extra deadwood candidates of
/// region r, slot = r + (2 << 16) (Forest) or r + (3 << 16) (Plain)
/// ("RFILWOOD" in ASCII).
pub const STREAM_REGION_FILL_WOOD: u64 = 0x5246_494C_574F_4F44;

/// Fission threshold (Pass 13, `game_v19`): the hidden threshold θ of a
/// settlement, subject = its settlement id, tick 0, slot = its number of
/// splits so far, replicate always 0 (a world draw: a noise fork never
/// redraws it; "FISSTHRS" in ASCII).
pub const STREAM_FISSION_THRESHOLD: u64 = 0x4649_5353_5448_5253;

/// Temperament (Pass 16, `game_v20`): a resident's four axes, subject = its
/// lineage key, tick 0, slot = 4·axis + s for the three uniforms s ∈ {1, 2,
/// 3}, replicate always 0 (an identity trait, not noise; "TEMPRMNT" in ASCII).
pub const STREAM_TEMPERAMENT: u64 = 0x5445_4D50_524D_4E54;

/// Routine kind choice, redraw and rest duration (slots zero, one and two).
pub const STREAM_ROUTINE: u64 = 0x524F_5554_494E_455F;
/// Routine place tie rank, keyed by lineage, decision tick and replicate.
pub const STREAM_ROUTINE_PLACE: u64 = 0x5254_4E50_4C41_4345;
/// Children's bounded play steps; slots zero and one are the two axes.
pub const STREAM_ROUTINE_PLAY: u64 = 0x5254_4E50_4C41_595F;
/// Family sex identity; lineage subject and fixed tick, slot and replicate zero.
pub const STREAM_SEX: u64 = 0x5345_585F_4944_454E;

/// Quarrels (Pass 20, `game_v25`): whether a forage dispute between two
/// villages' foragers becomes a quarrel, subject = the lineage key of the
/// lower-AgentId forager, tick = the pickup's tick, slot 0, replicate = the
/// noise replicate in force ("RELATION" in ASCII).
pub const STREAM_RELATIONS: u64 = 0x5245_4C41_5449_4F4E;

/// M1: logs a tree yields, keyed `(seed, STREAM_TREE_LOGS, x, y)` ("tree_logs").
pub const STREAM_TREE_LOGS: u64 = 0x5452_4545_4C4F_4753;
/// Standing tree material, keyed by its admitted coordinates.
pub const STREAM_TREE_SPECIES: u64 = 0x5452_4545_5350_4543;
/// M1: the colony world's bushes beyond the camp ring (shuffle; slot = 0).
pub const STREAM_COLONY_BUSH: u64 = 0x434F_4C42_5553_485F;
/// M1: the colony world's deadwood beyond the camp band (shuffle; slot = terrain pass).
pub const STREAM_COLONY_WOOD: u64 = 0x434F_4C57_4F4F_445F;
/// M1: a replacement adult founder ("colony_founder_fix"; subject = founder slot).
pub const STREAM_COLONY_FOUNDER_FIX: u64 = 0x434F_4C46_4E46_4958;
/// M1.5: a woman's daily conception draw, keyed `(seed, STREAM_CONCEIVE,
/// lineage key, day index)` ("conceive").
pub const STREAM_CONCEIVE: u64 = 0x434F_4E43_4549_5645;

/// M2: boulder placement and stone amounts (`"BOULDERS"`; subject x, tick y, slot 0 =
/// placement, 1 = amount).
pub const STREAM_BOULDER: u64 = 0x424F_554C_4445_5253;
/// M2: flint outcrop choice and amounts (`"OUTCROPS"`; shuffle slot 0, amount slot 1).
pub const STREAM_OUTCROP: u64 = 0x4F55_5443_524F_5053;
/// M2: a craft's quality bonus (`"QUALITY_"`; subject agent, tick, slot bill, replicate 0/1).
pub const STREAM_QUALITY: u64 = 0x5155_414C_4954_595F;

/// M4: knowledge — founders' seeding (slot = node id, 0x100 + id for the
/// proficiency), the evening telling (0x1_0000 teller pick, 0x1_0003 rehearsal,
/// 0x1_0100 + id chance, 0x1_0200 + id order) and discovery (slot
/// 0x2_0000 + id); subject = the lineage key (`"KNOWLEDG"`).
pub const STREAM_KNOWLEDGE: u64 = 0x4B4E_4F57_4C45_4447;

/// M4: clay banks on the river bank (`"CLAYBANK"`; shuffle slot 0, the
/// near-camp top-up slot 1).
pub const STREAM_CLAY: u64 = 0x434C_4159_4241_4E4B;

/// M5: the storyteller's season plans and illness (`"STORYTEL"`; subject = the
/// season index or a lineage key, slot = the draw within it).
pub const STREAM_STORY: u64 = 0x5354_4F52_5954_454C;

/// M5: hunting rolls and animal movement (`"WILDLIFE"`; subject = a hunter's
/// lineage key or an animal id, slot = the animal or the step).
pub const STREAM_WILDLIFE: u64 = 0x5749_4C44_4C49_4645;

/// M5: quarrels (`"QUARRELS"`; subject = the pair's lower lineage key, slot = the other's).
pub const STREAM_QUARREL: u64 = 0x5155_4152_5245_4C53;

/// M5: whether a worn top spares its wearer a cold hour (`"APPAREL!"`; subject =
/// the wearer's lineage key, tick = the hour).
pub const STREAM_APPAREL: u64 = 0x4150_5041_5245_4C21;

/// M5.5 (ADR 0031): the neighbours' generation, migrant waves, migrants' knowledge and population drift.
pub const STREAM_NEIGHBOURS: u64 = 0x4E45_4947_4842_4F52;
/// M5.5 (ADR 0032): caravan dates and their goods.
pub const STREAM_TRADE: u64 = 0x5452_4144_4552_5321;

/// Every stream constant, for the distinctness test and the inventory.
pub const STREAMS: [u64; 39] = [
    STREAM_MOVE_PAUSE,
    STREAM_MOVE_DX,
    STREAM_MOVE_DY,
    STREAM_LIFESPAN,
    STREAM_FOUNDER,
    STREAM_NEWBORN,
    STREAM_WORLD_VARIETY,
    STREAM_HARDINESS,
    STREAM_REGION,
    STREAM_REGION_BUSH,
    STREAM_REGION_WOOD,
    STREAM_FISSION,
    STREAM_FISSION_HAZARD,
    STREAM_REGION_FILL_BUSH,
    STREAM_REGION_FILL_WOOD,
    STREAM_FISSION_THRESHOLD,
    STREAM_TEMPERAMENT,
    STREAM_ROUTINE,
    STREAM_ROUTINE_PLACE,
    STREAM_ROUTINE_PLAY,
    STREAM_SEX,
    STREAM_RELATIONS,
    STREAM_TREE_LOGS,
    STREAM_TREE_SPECIES,
    STREAM_COLONY_BUSH,
    STREAM_COLONY_WOOD,
    STREAM_COLONY_FOUNDER_FIX,
    STREAM_CONCEIVE,
    STREAM_BOULDER,
    STREAM_OUTCROP,
    STREAM_QUALITY,
    STREAM_KNOWLEDGE,
    STREAM_CLAY,
    STREAM_STORY,
    STREAM_WILDLIFE,
    STREAM_QUARREL,
    STREAM_APPAREL,
    STREAM_NEIGHBOURS,
    STREAM_TRADE,
];

/// One splitmix64 output for state `x` (identical to `MovementRng::new(x).next_u64()`).
pub const fn mix(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// One 64-bit draw that depends only on its key. `stream` names the decision
/// (constants above), `subject` the entity's lineage key or 0 for world draws,
/// `tick` the simulation tick, `slot` the index of the draw within that event,
/// `replicate` the noise replicate (0 = the canonical run).
pub const fn keyed_u64(
    world_seed: u64,
    stream: u64,
    subject: u64,
    tick: u64,
    slot: u32,
    replicate: u32,
) -> u64 {
    mix(mix(mix(mix(mix(world_seed ^ stream) ^ subject) ^ tick) ^ slot as u64) ^ replicate as u64)
}

/// A uniform value in `[0, 1)` from the top 53 bits of a keyed draw.
pub fn keyed_unit_f64(
    world_seed: u64,
    stream: u64,
    subject: u64,
    tick: u64,
    slot: u32,
    replicate: u32,
) -> f64 {
    (keyed_u64(world_seed, stream, subject, tick, slot, replicate) >> 11) as f64
        / (1u64 << 53) as f64
}

/// A value in `0..n` by multiply-shift (`n = 0` gives 0). One draw, no
/// rejection loop, so the number of draws never depends on the value.
pub const fn below(draw: u64, n: u64) -> u64 {
    ((draw as u128 * n as u128) >> 64) as u64
}

/// A keyed value in `0..n` (see [`below`]).
pub const fn keyed_below(
    world_seed: u64,
    stream: u64,
    subject: u64,
    tick: u64,
    slot: u32,
    replicate: u32,
    n: u64,
) -> u64 {
    below(
        keyed_u64(world_seed, stream, subject, tick, slot, replicate),
        n,
    )
}

/// A keyed Brownian axis step in `{-1, 0, +1}`.
pub const fn keyed_step(
    world_seed: u64,
    stream: u64,
    subject: u64,
    tick: u64,
    slot: u32,
    replicate: u32,
) -> i32 {
    keyed_below(world_seed, stream, subject, tick, slot, replicate, 3) as i32 - 1
}

/// A noise fork (Pass 9): from `tick` on, every keyed draw uses `replicate`
/// instead of 0. Replicate 0 is the canonical run by definition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoiseFork {
    /// First simulation tick (as systems see `current_tick`) that uses the replicate.
    pub tick: u64,
    /// The noise replicate after the fork.
    pub replicate: u32,
}

impl NoiseFork {
    /// The replicate in force at simulation tick `tick`.
    pub const fn replicate_at(&self, tick: u64) -> u32 {
        if tick >= self.tick {
            self.replicate
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn streams_are_distinct() {
        for (i, a) in STREAMS.iter().enumerate() {
            for b in &STREAMS[i + 1..] {
                assert_ne!(a, b);
            }
        }
    }

    #[test]
    fn mix_is_the_splitmix64_reference() {
        // splitmix64 seeded with 0: first output 0xE220A8397B1DCDAF (Vigna's reference).
        assert_eq!(mix(0), 0xE220_A839_7B1D_CDAF);
    }

    #[test]
    fn every_field_changes_the_draw() {
        let base = keyed_u64(1, STREAM_MOVE_DX, 2, 3, 4, 5);
        assert_eq!(base, keyed_u64(1, STREAM_MOVE_DX, 2, 3, 4, 5));
        for other in [
            keyed_u64(9, STREAM_MOVE_DX, 2, 3, 4, 5),
            keyed_u64(1, STREAM_MOVE_DY, 2, 3, 4, 5),
            keyed_u64(1, STREAM_MOVE_DX, 9, 3, 4, 5),
            keyed_u64(1, STREAM_MOVE_DX, 2, 9, 4, 5),
            keyed_u64(1, STREAM_MOVE_DX, 2, 3, 9, 5),
            keyed_u64(1, STREAM_MOVE_DX, 2, 3, 4, 9),
        ] {
            assert_ne!(base, other);
        }
    }

    #[test]
    fn below_stays_in_range_and_steps_are_unit() {
        assert_eq!(below(u64::MAX, 3), 2);
        assert_eq!(below(0, 3), 0);
        assert_eq!(below(123, 0), 0);
        for t in 0..1000 {
            let s = keyed_step(7, STREAM_MOVE_DX, 11, t, 0, 0);
            assert!((-1..=1).contains(&s));
            let u = keyed_unit_f64(7, STREAM_MOVE_PAUSE, 11, t, 0, 0);
            assert!((0.0..1.0).contains(&u));
        }
    }

    #[test]
    fn fork_replicate_starts_at_its_tick() {
        let fork = NoiseFork {
            tick: 100,
            replicate: 3,
        };
        assert_eq!(
            (
                fork.replicate_at(99),
                fork.replicate_at(100),
                fork.replicate_at(500)
            ),
            (0, 3, 3)
        );
    }
}
