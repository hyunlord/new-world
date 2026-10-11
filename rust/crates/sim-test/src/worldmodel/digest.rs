//! A digest of the whole simulation state a row depends on (Pass 7).
//!
//! `probe::fingerprint` hashes agents, four tallies and each settlement's
//! food, but not wood stocks, deadwood, fires or the intervention state, so
//! prefix identity (a run with an intervention equals the factual run up to
//! the intervention) is checked on this digest instead.
use sim_core::components::ResourceKind;
use sim_engine::SimEngine;

use crate::probe::fingerprint;

const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const PRIME: u64 = 0x0100_0000_01b3;

fn feed(hash: &mut u64, bytes: &[u8]) {
    for byte in bytes {
        *hash = (*hash ^ u64::from(*byte)).wrapping_mul(PRIME);
    }
}

/// FNV-1a (as `probe::fingerprint`) over the fingerprint string, then, in id
/// or key order with a length before every collection: every settlement's
/// food and wood stock, `food_tiles`, `wood_tiles`, `fires`,
/// `fire_blocked_until`, `cold_snap_nights`, `cold_snap_tonight`,
/// `food_regen_acc`, `wood_regen_acc`, `wood_winter_acc` and
/// `wood_winter_cursor`. Integers are little-endian.
pub fn state_digest(engine: &SimEngine) -> u64 {
    let r = &engine.resources;
    let mut hash = OFFSET;
    feed(&mut hash, fingerprint(engine).as_bytes());
    let mut settlements: Vec<_> = r.settlements.values().collect();
    settlements.sort_by_key(|s| s.settlement_id);
    feed(&mut hash, &(settlements.len() as u64).to_le_bytes());
    for settlement in settlements {
        feed(&mut hash, &settlement.settlement_id.to_le_bytes());
        for kind in [ResourceKind::Food, ResourceKind::Wood] {
            feed(&mut hash, &settlement.stockpile.get(&kind).copied().unwrap_or(0).to_le_bytes());
        }
    }
    let mut food: Vec<_> = r.food_tiles.iter().map(|(k, v)| (*k, *v)).collect();
    food.sort_unstable();
    for tiles in [food, r.wood_tiles.iter().map(|(k, v)| (*k, *v)).collect()] {
        feed(&mut hash, &(tiles.len() as u64).to_le_bytes());
        for ((x, y), amount) in tiles {
            feed(&mut hash, &x.to_le_bytes());
            feed(&mut hash, &y.to_le_bytes());
            feed(&mut hash, &[amount]);
        }
    }
    feed(&mut hash, &(r.fires.len() as u64).to_le_bytes());
    for (sid, fire) in &r.fires {
        feed(&mut hash, &sid.to_le_bytes());
        feed(&mut hash, &[u8::from(fire.lit)]);
    }
    feed(&mut hash, &(r.fire_blocked_until.len() as u64).to_le_bytes());
    for (sid, until) in &r.fire_blocked_until {
        feed(&mut hash, &sid.to_le_bytes());
        feed(&mut hash, &until.to_le_bytes());
    }
    feed(&mut hash, &r.cold_snap_nights.to_le_bytes());
    feed(&mut hash, &[u8::from(r.cold_snap_tonight)]);
    feed(&mut hash, &r.food_regen_acc.to_le_bytes());
    feed(&mut hash, &r.wood_regen_acc.to_le_bytes());
    feed(&mut hash, &r.wood_winter_acc.to_le_bytes());
    match r.wood_winter_cursor {
        Some((x, y)) => {
            feed(&mut hash, &[1]);
            feed(&mut hash, &x.to_le_bytes());
            feed(&mut hash, &y.to_le_bytes());
        }
        None => feed(&mut hash, &[0]),
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_matches_known_vector() {
        let mut hash = OFFSET;
        feed(&mut hash, b"hello");
        assert_eq!(hash, 0xa430d84680aabd0b);
    }

    #[test]
    fn digest_sees_what_the_fingerprint_misses() {
        let mut engine = crate::scenes::game_v12_scene(1);
        let base = state_digest(&engine);
        assert_eq!(base, state_digest(&crate::scenes::game_v12_scene(1)));
        let before = fingerprint(&engine);
        if let Some(spot) = engine.resources.wood_tiles.values_mut().next() {
            *spot -= 1;
        }
        assert_eq!(fingerprint(&engine), before);
        assert_ne!(state_digest(&engine), base);
        let mut engine = crate::scenes::game_v12_scene(1);
        engine.resources.cold_snap_tonight = true;
        assert_ne!(state_digest(&engine), base);
    }
}
