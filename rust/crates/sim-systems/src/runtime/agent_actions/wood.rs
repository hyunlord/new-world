//! Firewood helpers of the decision system (Pass 5, Pass 6, Pass 7, Pass 18):
//! the per-settlement wood plan, the urgent-mode rule, the deterministic
//! deadwood spot choice, its candidate filter and the trip-existence predicate.
//! Pure functions over the start-of-tick maps; `system.rs` calls them.

use std::collections::BTreeMap;

use sim_core::components::AgentId;
use sim_core::rules::FirewoodRule;

/// A wood gatherer to re-target (Pass 5): agent id, entity, position and its
/// emptied spot (`None` when it had no target).
pub(in crate::runtime) type WoodRetarget = (AgentId, hecs::Entity, (u32, u32), Option<(u32, u32)>);

/// Per-settlement firewood inputs computed once per decision tick (Pass 5).
#[derive(Debug, Clone)]
pub(in crate::runtime) struct WoodPlan {
    /// Settlement formation tile; deadwood is measured from here.
    pub(in crate::runtime) home: (u32, u32),
    /// Wood target in units.
    pub(in crate::runtime) target: f64,
    /// Stockpile plus wood carried by members.
    pub(in crate::runtime) in_hand: u64,
    /// Urgent mode (Pass 6): tonight's burn is not in hand.
    pub(in crate::runtime) urgent: bool,
}

/// The firewood rule a member acts under (Pass 6): in urgent mode the
/// per-spot claim cap doubles; otherwise the rule unchanged.
pub(in crate::runtime) fn urgent_rule(rule: FirewoodRule, urgent: bool) -> FirewoodRule {
    if urgent {
        FirewoodRule {
            max_gatherers_per_spot: rule.max_gatherers_per_spot.saturating_mul(2),
            ..rule
        }
    } else {
        rule
    }
}

/// Deterministic deadwood choice (Pass 5): among spots holding wood within
/// `wood_radius` of `home` whose claim count is below the cap, minimise
/// `(claims, chebyshev(agent, spot), y, x)`. `None` when no spot qualifies.
/// Pass 7: under `claims_bounded_by_amount` the cap is `min(cap, amount)`.
pub(in crate::runtime) fn choose_spot(
    rule: &FirewoodRule,
    home: (u32, u32),
    agent: (u32, u32),
    wood_tiles: &BTreeMap<(u32, u32), u8>,
    claims: &BTreeMap<(u32, u32), u32>,
    exclude: Option<(u32, u32)>,
) -> Option<(u32, u32)> {
    let chebyshev = |a: (u32, u32), b: (u32, u32)| a.0.abs_diff(b.0).max(a.1.abs_diff(b.1));
    wood_tiles
        .iter()
        .filter(|(tile, amount)| spot_eligible(rule, home, claims, exclude, **tile, **amount))
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

/// The candidate filter of [`choose_spot`]; it does not depend on the person.
fn spot_eligible(
    rule: &FirewoodRule,
    home: (u32, u32),
    claims: &BTreeMap<(u32, u32), u32>,
    exclude: Option<(u32, u32)>,
    tile: (u32, u32),
    amount: u8,
) -> bool {
    amount >= 1
        && Some(tile) != exclude
        && tile.0.abs_diff(home.0).max(tile.1.abs_diff(home.1)) <= rule.wood_radius
        && claims.get(&tile).copied().unwrap_or(0)
            < if rule.claims_bounded_by_amount {
                rule.max_gatherers_per_spot.min(u32::from(amount))
            } else {
                rule.max_gatherers_per_spot
            }
}

/// Pass 18: whether [`choose_spot`] (without `exclude`) returns `Some` for
/// this `home` and `claims`, whoever asks.
pub(in crate::runtime) fn spot_exists(
    rule: &FirewoodRule,
    home: (u32, u32),
    wood_tiles: &BTreeMap<(u32, u32), u8>,
    claims: &BTreeMap<(u32, u32), u32>,
) -> bool {
    wood_tiles
        .iter()
        .any(|(tile, amount)| spot_eligible(rule, home, claims, None, *tile, *amount))
}
