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

/// Every stream constant, for the distinctness test and the inventory.
pub const STREAMS: [u64; 12] = [
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
pub const fn keyed_u64(world_seed: u64, stream: u64, subject: u64, tick: u64, slot: u32, replicate: u32) -> u64 {
    mix(mix(mix(mix(mix(world_seed ^ stream) ^ subject) ^ tick) ^ slot as u64) ^ replicate as u64)
}

/// A uniform value in `[0, 1)` from the top 53 bits of a keyed draw.
pub fn keyed_unit_f64(world_seed: u64, stream: u64, subject: u64, tick: u64, slot: u32, replicate: u32) -> f64 {
    (keyed_u64(world_seed, stream, subject, tick, slot, replicate) >> 11) as f64 / (1u64 << 53) as f64
}

/// A value in `0..n` by multiply-shift (`n = 0` gives 0). One draw, no
/// rejection loop, so the number of draws never depends on the value.
pub const fn below(draw: u64, n: u64) -> u64 {
    ((draw as u128 * n as u128) >> 64) as u64
}

/// A keyed value in `0..n` (see [`below`]).
pub const fn keyed_below(world_seed: u64, stream: u64, subject: u64, tick: u64, slot: u32, replicate: u32, n: u64) -> u64 {
    below(keyed_u64(world_seed, stream, subject, tick, slot, replicate), n)
}

/// A keyed Brownian axis step in `{-1, 0, +1}`.
pub const fn keyed_step(world_seed: u64, stream: u64, subject: u64, tick: u64, slot: u32, replicate: u32) -> i32 {
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
        let fork = NoiseFork { tick: 100, replicate: 3 };
        assert_eq!((fork.replicate_at(99), fork.replicate_at(100), fork.replicate_at(500)), (0, 3, 3));
    }
}
