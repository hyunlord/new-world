//! Forage helpers of the decision system (Pass 4, Pass 18, Pass 20): the
//! per-settlement forage plan, the deterministic bush choice, its candidate
//! filter and the trip-existence predicate. Pure functions over the start-of-tick
//! maps; the decision loop in `system.rs` calls them.

use std::collections::{BTreeMap, HashMap};

use sim_core::components::{AgentId, SettlementId};
use sim_core::rules::ForageRule;

/// Pass 20: a forager at (or heading to) a bush in the start-of-tick snapshot:
/// agent id, village, lineage key, NS and HA.
pub(super) type ForageRival = (AgentId, SettlementId, u64, u16, u16);

/// Per-settlement forage inputs computed once per decision tick (Pass 4).
#[derive(Debug, Clone)]
pub(super) struct ForagePlan {
    /// Settlement formation tile; bushes are measured from here.
    pub(super) home: (u32, u32),
    /// Store target in food units.
    pub(super) target: f64,
    /// Stockpile plus food carried by members.
    pub(super) in_hand: u64,
}

/// Deterministic forage bush choice: among tiles holding food within
/// `forage_radius` of `home` whose claim count is below the cap, minimise
/// `(claims, chebyshev(agent, bush), y, x)`. `None` when no bush qualifies.
/// Pass 20: `avoid` holds the hearths of villages estranged from the
/// forager's (empty in every other scene); see [`bush_eligible`].
pub(super) fn choose_bush(
    rule: &ForageRule,
    home: (u32, u32),
    agent: (u32, u32),
    food_tiles: &HashMap<(u32, u32), u8>,
    claims: &BTreeMap<(u32, u32), u32>,
    exclude: Option<(u32, u32)>,
    avoid: &[(u32, u32)],
) -> Option<(u32, u32)> {
    let chebyshev = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
    food_tiles
        .iter()
        .filter(|(tile, amount)| {
            bush_eligible(rule, home, claims, exclude, avoid, **tile, **amount)
        })
        .map(|(tile, _)| *tile)
        .min_by_key(|tile| {
            (
                claims.get(tile).copied().unwrap_or(0),
                chebyshev(agent, *tile),
                tile.1,
                tile.0,
            )
        })
}

/// The candidate filter of [`choose_bush`]; it does not depend on the person.
/// Pass 20: a bush nearer (Chebyshev) to an `avoid` hearth than to `home` is
/// skipped — a village estranged from another leaves it the bushes on its
/// side.
pub(super) fn bush_eligible(
    rule: &ForageRule,
    home: (u32, u32),
    claims: &BTreeMap<(u32, u32), u32>,
    exclude: Option<(u32, u32)>,
    avoid: &[(u32, u32)],
    tile: (u32, u32),
    amount: u8,
) -> bool {
    let chebyshev = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
    amount >= 1
        && Some(tile) != exclude
        && chebyshev(tile, home) <= rule.forage_radius
        && claims.get(&tile).copied().unwrap_or(0) < rule.max_foragers_per_bush
        && avoid
            .iter()
            .all(|hearth| chebyshev(tile, *hearth) >= chebyshev(tile, home))
}

/// Pass 18: whether [`choose_bush`] (without `exclude`) returns `Some` for
/// this `home` and `claims`, whoever asks.
pub(super) fn bush_exists(
    rule: &ForageRule,
    home: (u32, u32),
    food_tiles: &HashMap<(u32, u32), u8>,
    claims: &BTreeMap<(u32, u32), u32>,
    avoid: &[(u32, u32)],
) -> bool {
    food_tiles
        .iter()
        .any(|(tile, amount)| bush_eligible(rule, home, claims, None, avoid, *tile, *amount))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_bush_prefers_unclaimed_then_near_then_row_major() {
        let rule = ForageRule {
            start_minute: 360,
            end_minute: 1080,
            forage_radius: 16,
            haul_threshold: 4,
            max_foragers_per_bush: 2,
        };
        let mut tiles = HashMap::new();
        for tile in [(12, 10), (10, 12), (8, 10), (40, 40)] {
            tiles.insert(tile, 3u8);
        }
        tiles.insert((11, 11), 0);
        let mut claims = BTreeMap::new();
        // (8,10) and (12,10) tie on distance 2 from (10,10); y then x picks (8,10).
        assert_eq!(
            choose_bush(&rule, (10, 10), (10, 10), &tiles, &claims, None, &[]),
            Some((8, 10))
        );
        claims.insert((8, 10), 1);
        assert_eq!(
            choose_bush(&rule, (10, 10), (10, 10), &tiles, &claims, None, &[]),
            Some((12, 10))
        );
        claims.insert((12, 10), 2);
        claims.insert((10, 12), 2);
        assert_eq!(
            choose_bush(&rule, (10, 10), (10, 10), &tiles, &claims, None, &[]),
            Some((8, 10))
        );
        assert_eq!(
            choose_bush(
                &rule,
                (10, 10),
                (10, 10),
                &tiles,
                &claims,
                Some((8, 10)),
                &[]
            ),
            None
        );
    }

    #[test]
    fn estranged_hearths_remove_the_bushes_on_their_side_at_every_site() {
        // Pass 20: the avoid list of the forage arm (`choose_bush`), the exit
        // predicate (`bush_exists`) and the retarget (`choose_bush` with an
        // exclusion) all go through `bush_eligible`.
        let rule = ForageRule {
            start_minute: 360,
            end_minute: 1080,
            forage_radius: 16,
            haul_threshold: 4,
            max_foragers_per_bush: 2,
        };
        let home = (40, 30);
        let other = (14, 30);
        let claims = BTreeMap::new();
        let mut tiles = HashMap::new();
        tiles.insert((26, 30), 3u8); // 14 from home, 12 from the other: its side
        tiles.insert((27, 30), 3u8); // 13 / 13: a tie stays eligible
        tiles.insert((50, 30), 3u8); // 10 from home, 36 from the other
        assert!(!bush_eligible(
            &rule,
            home,
            &claims,
            None,
            &[other],
            (26, 30),
            3
        ));
        assert!(bush_eligible(&rule, home, &claims, None, &[], (26, 30), 3));
        assert!(bush_eligible(
            &rule,
            home,
            &claims,
            None,
            &[other],
            (27, 30),
            3
        ));
        assert!(bush_eligible(
            &rule,
            home,
            &claims,
            None,
            &[(90, 90), other],
            (50, 30),
            3
        ));
        // Nearest to the agent standing at (28, 30): (27, 30), then (26, 30).
        assert_eq!(
            choose_bush(&rule, home, (28, 30), &tiles, &claims, None, &[]),
            Some((27, 30))
        );
        assert_eq!(
            choose_bush(&rule, home, (28, 30), &tiles, &claims, Some((27, 30)), &[]),
            Some((26, 30))
        );
        assert_eq!(
            choose_bush(
                &rule,
                home,
                (28, 30),
                &tiles,
                &claims,
                Some((27, 30)),
                &[other]
            ),
            Some((50, 30))
        );
        let mut only_theirs = HashMap::new();
        only_theirs.insert((26, 30), 3u8);
        assert!(bush_exists(&rule, home, &only_theirs, &claims, &[]));
        assert!(!bush_exists(&rule, home, &only_theirs, &claims, &[other]));
    }
}
