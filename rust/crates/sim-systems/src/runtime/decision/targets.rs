//! Target-tile selectors of the decision system: the nearest resource tile,
//! the local meal tile, the Gather arm's in-radius and catchment-restricted
//! ground-food choice with its avoid-list side test and existence predicate,
//! and the per-tick gather memo types.

use std::collections::BTreeMap;

use sim_core::components::{Position, SettlementId};

/// Pass 18: per-tick `gather_tile_exists` answers by (tile, catchment) with
/// the food-tiles version they were computed from. Pass 21: the key also
/// names whose avoid list was applied (the member's settlement id under a
/// `RelationsRule`, else `None` for the empty list).
pub(super) type GatherMemo = BTreeMap<GatherKey, (u64, bool)>;

/// Pass 21: a [`GatherMemo`] key — (tile, catchment, avoid-list owner).
type GatherKey = ((u32, u32), Option<((u32, u32), u32)>, Option<SettlementId>);

/// Chebyshev radius around an agent within which a ground-food tile is eligible
/// for the `CascadeArm::Gather` arm (Direction-2 slice 2-2).
///
/// Bounds the gather scan to `O(tiles-in-radius)` on the decision hot-path
/// (Gate 6) and keeps satisfied agents from trekking across the map for a
/// low-value pickup. Default `8` — wide enough that the bootstrapped infinite
/// source tiles are reachable from the agent cluster, narrow enough that travel
/// to a target resolves well under the Assertion-13 frozen-streak bound (200).
/// The nearest eligible tile is chosen by the deterministic
/// [`nearest_resource_tile`] `(distance, x, y)` tie-break, then gated on this
/// Chebyshev radius.
pub const GATHER_SCAN_RADIUS: u32 = 8;

/// Nearest resource tile to `pos` by Manhattan distance, with a
/// deterministic `(x, y)` tie-break (V7 Section 16-α).
///
/// `HashMap` iteration order is unspecified, so the `(distance, x, y)`
/// key is what makes the result deterministic across runs and across two
/// independently-built maps. Returns `None` **only** when `tiles` is
/// empty — this is the sole documented `None` case, relied upon by the
/// post-decision pass to skip the attach op for an empty resource map.
pub fn nearest_resource_tile(
    pos: &Position,
    tiles: &std::collections::HashMap<(u32, u32), u8>,
) -> Option<(u32, u32)> {
    tiles
        .keys()
        .min_by_key(|(tx, ty)| {
            let dx = (*tx as i64 - pos.x as i64).abs();
            let dy = (*ty as i64 - pos.y as i64).abs();
            (dx + dy, *tx, *ty)
        })
        .copied()
}

/// Pass 12 (`local_food_availability`): the Manhattan-nearest food tile with
/// food within Chebyshev `radius` of `home` (tie-break `(dist, x, y)`), the
/// meal target of a settlement member; `None` when the catchment is bare.
pub fn nearest_local_food(
    pos: &Position,
    tiles: &std::collections::HashMap<(u32, u32), u8>,
    home: (u32, u32),
    radius: u32,
) -> Option<(u32, u32)> {
    tiles
        .iter()
        .filter(|(t, amount)| {
            **amount > 0 && t.0.abs_diff(home.0).max(t.1.abs_diff(home.1)) <= radius
        })
        .map(|(t, _)| *t)
        .min_by_key(|(tx, ty)| {
            (
                (*tx as i64 - pos.x as i64).abs() + (*ty as i64 - pos.y as i64).abs(),
                *tx,
                *ty,
            )
        })
}

/// Nearest *eligible* food tile for the `CascadeArm::Gather` arm: the
/// Manhattan-nearest tile (deterministic `(dist, x, y)` tie-break) among ONLY
/// those food tiles holding count `> 0` AND within `radius` Chebyshev of `pos`.
///
/// Filtering to the in-radius set BEFORE the nearest-tile reduction is
/// load-bearing: a closer tile OUTSIDE the radius must never mask an eligible
/// tile INSIDE it. `nearest_resource_tile(..).filter(in_radius)` is the WRONG
/// shape — it picks the global nearest first and rejects the whole result if
/// that single tile is out of range, so the Gather arm would never fire even
/// though a reachable tile exists (the radius-confounder bug). Both the Gather
/// arm trigger and the post-decision `SeekTarget` attach call THIS helper so the
/// chosen target tile is identical (determinism + the agent walks to the tile
/// the arm actually selected). Returns `None` when no eligible tile lies within
/// range — the arm then does not fire and the cascade falls through to
/// Settlement.
pub fn nearest_food_tile_in_radius(
    pos: &Position,
    tiles: &std::collections::HashMap<(u32, u32), u8>,
    radius: u32,
) -> Option<(u32, u32)> {
    tiles
        .iter()
        .filter(|((tx, ty), v)| {
            **v > 0 && {
                let dx = (*tx as i64 - pos.x as i64).abs();
                let dy = (*ty as i64 - pos.y as i64).abs();
                dx.max(dy) <= radius as i64
            }
        })
        .map(|(k, _)| *k)
        .min_by_key(|(tx, ty)| {
            let dx = (*tx as i64 - pos.x as i64).abs();
            let dy = (*ty as i64 - pos.y as i64).abs();
            (dx + dy, *tx, *ty)
        })
}

/// Pass 21 (under a `RelationsRule`): whether `tile` lies on the own
/// side of every avoided hearth — its Chebyshev distance to each hearth in
/// `avoid` is at least its distance to `home` (a tie keeps it). The same test
/// as [`bush_eligible`]'s; always true for an empty list.
fn own_side(tile: (u32, u32), home: (u32, u32), avoid: &[(u32, u32)]) -> bool {
    let chebyshev = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
    avoid
        .iter()
        .all(|hearth| chebyshev(tile, *hearth) >= chebyshev(tile, home))
}

/// Pass 12 (`local_food_availability`): [`nearest_food_tile_in_radius`] for
/// the Gather arm, restricted for a settlement member (`local` = its formation
/// tile and catchment radius) to tiles inside its catchment; `None` = as v17.
/// Pass 21: a member also skips tiles nearer (Chebyshev) to a hearth in
/// `avoid` (the villages estranged from its own) than to its own hearth;
/// `avoid` is empty without a `RelationsRule`, and is not
/// read without `local` (a non-member has no hearth to compare).
pub fn nearest_gather_tile(
    pos: &Position,
    tiles: &std::collections::HashMap<(u32, u32), u8>,
    radius: u32,
    local: Option<((u32, u32), u32)>,
    avoid: &[(u32, u32)],
) -> Option<(u32, u32)> {
    let Some((home, catchment)) = local else {
        return nearest_food_tile_in_radius(pos, tiles, radius);
    };
    tiles
        .iter()
        .filter(|((tx, ty), v)| {
            **v > 0
                && (*tx as i64 - pos.x as i64)
                    .abs()
                    .max((*ty as i64 - pos.y as i64).abs())
                    <= radius as i64
                && tx.abs_diff(home.0).max(ty.abs_diff(home.1)) <= catchment
                && own_side((*tx, *ty), home, avoid)
        })
        .map(|(k, _)| *k)
        .min_by_key(|(tx, ty)| {
            (
                (*tx as i64 - pos.x as i64).abs() + (*ty as i64 - pos.y as i64).abs(),
                *tx,
                *ty,
            )
        })
}

/// Pass 18: whether [`nearest_gather_tile`] returns `Some` (the same filters,
/// without choosing the nearest). Pass 21: with the same `avoid` filter.
pub(super) fn gather_tile_exists(
    pos: &Position,
    tiles: &std::collections::HashMap<(u32, u32), u8>,
    radius: u32,
    local: Option<((u32, u32), u32)>,
    avoid: &[(u32, u32)],
) -> bool {
    tiles.iter().any(|((tx, ty), v)| {
        *v > 0
            && (*tx as i64 - pos.x as i64)
                .abs()
                .max((*ty as i64 - pos.y as i64).abs())
                <= radius as i64
            && local.is_none_or(|(home, catchment)| {
                tx.abs_diff(home.0).max(ty.abs_diff(home.1)) <= catchment
                    && own_side((*tx, *ty), home, avoid)
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::decision::forage::bush_eligible;
    use sim_core::rules::ForageRule;
    use std::collections::HashMap;

    #[test]
    fn gather_selectors_skip_tiles_on_an_avoided_side() {
        // Pass 21 (avoid lists): `nearest_gather_tile` and
        // `gather_tile_exists` apply `bush_eligible`'s side test to a member
        // (`local` = home and catchment); an empty list changes nothing, and a
        // person without `local` (a non-member) is not filtered.
        let home = (40, 30);
        let other = (14, 30);
        let local = Some((home, 16));
        let pos = Position::new(30, 30);
        let mut tiles = HashMap::new();
        tiles.insert((26, 30), 3u8); // 14 from home, 12 from the other: its side, nearest
        tiles.insert((27, 30), 3u8); // 13 / 13: a tie stays
        tiles.insert((36, 30), 3u8); // own side
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, local, &[]),
            Some((27, 30))
        );
        tiles.remove(&(27, 30));
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, local, &[]),
            Some((26, 30))
        );
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, local, &[other]),
            Some((36, 30))
        );
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, None, &[other]),
            Some((26, 30)),
            "non-member: unfiltered"
        );
        assert!(gather_tile_exists(&pos, &tiles, 8, local, &[other]));
        tiles.remove(&(36, 30));
        assert_eq!(nearest_gather_tile(&pos, &tiles, 8, local, &[other]), None);
        assert!(!gather_tile_exists(&pos, &tiles, 8, local, &[other]));
        assert!(gather_tile_exists(&pos, &tiles, 8, local, &[]));
        assert!(gather_tile_exists(&pos, &tiles, 8, None, &[other]));
        // Several avoided hearths: every one must be at least as far.
        tiles.insert((36, 30), 3u8);
        assert_eq!(
            nearest_gather_tile(&pos, &tiles, 8, local, &[(90, 90), (34, 30)]),
            None
        );
        assert!(!gather_tile_exists(
            &pos,
            &tiles,
            8,
            local,
            &[(90, 90), (34, 30)]
        ));
        // The side test is `bush_eligible`'s.
        for tile in [(26, 30), (27, 30), (36, 30), (50, 30)] {
            let rule = ForageRule {
                start_minute: 360,
                end_minute: 1080,
                forage_radius: 16,
                haul_threshold: 4,
                max_foragers_per_bush: 2,
            };
            assert_eq!(
                own_side(tile, home, &[other]),
                bush_eligible(&rule, home, &BTreeMap::new(), None, &[other], tile, 3),
                "{tile:?}"
            );
        }
    }
}
