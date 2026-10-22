//! HearthFeedSystem: feeds each hearth from firewood items, nearest first, so `FireSystem` burns them (1 firewood = 1 wood unit).
//!
//! Priority: 146 (`HEARTH_FEED_PRIORITY`). Interval: 60 ticks. Gate: `WorldRules.colony` with `economy` and `society.firewood`.
//! Reads: `SimResources.colony` (ground items, zones, reservations), `settlements` (formation tile, stockpile), `rules.society.firewood`, the clock
//! Writes: `SimResources.settlements` (stockpile wood), `colony` (ground items, item ledger `burned`)
//! Events: none
//!
//! M2 goal 3.8, the first extraction from the decision tick: in the colony the
//! firewood no longer reaches the hearth through the decision system's
//! `GatherWood` arm (switched off by [`decision_firewood`]); it is made by jobs
//! (the wood bench's firewood bill, or gathering deadwood when no bill can run)
//! and lies as items in stockpiles or on the hearth pile (the tiles within
//! [`HEARTH_PILE_REACH`] of the hearth). Every hour, before `FireSystem` (147)
//! burns, each settlement in id order whose wood is below tonight's burn takes
//! firewood stacks — on stockpile cells or the hearth pile, unreserved, nearest
//! the hearth first `(distance, y, x)` — into `stockpile[Wood]`, one wood unit
//! per item, logged as `burned` (consumed) in the item ledger. `FireSystem` is
//! unchanged.
use hecs::World;
use sim_core::colony::{ItemKind, ReserveKey};
use sim_core::components::ResourceKind;
use sim_core::rules::FirewoodRule;
use sim_engine::{RuntimeSystem, SimResources};

use super::items::chebyshev;
use crate::runtime::settlement::firewood::{burn_hours, tonight_burn, wood_target};

/// Priority of [`HearthFeedSystem`]: just before `FireSystem` (147).
pub const HEARTH_FEED_PRIORITY: u32 = 146;
/// Interval of [`HearthFeedSystem`]: hourly, like `FireSystem`.
pub const HEARTH_FEED_INTERVAL: u64 = 60;
/// Firewood within this Chebyshev distance of a hearth is its pile.
pub const HEARTH_PILE_REACH: u32 = 2;

/// Whether the colony's firewood goes through jobs: the economy and the
/// firewood rule are both on (the M2 colony; never the baseline).
pub fn firewood_by_jobs(res: &SimResources) -> bool {
    res.rules.colony.is_some_and(|c| c.economy.is_some()) && res.rules.society.firewood.is_some()
}

/// The firewood rule the decision system acts under: none when firewood goes
/// through jobs (its `GatherWood` arm, wood plans and wood trips are then off).
pub fn decision_firewood(res: &SimResources) -> Option<FirewoodRule> {
    res.rules
        .society
        .firewood
        .filter(|_| !firewood_by_jobs(res))
}

/// The wood units the colony wants on hand (the decision's former target:
/// `wood_target`, rounded up; 0 without the rule).
pub fn hearth_need(res: &SimResources) -> u32 {
    wood_target(res).map_or(0, |t| t.ceil().max(0.0) as u32)
}

/// The hearth's auto-bill count (goal 3.8): `firewood_days × burn_per_day` =
/// the winter's stored days (`winter_base_days`, else `base_days`) × the
/// winter burn per hour × the burn hours.
pub fn auto_bill_count(res: &SimResources) -> u16 {
    res.rules.society.firewood.map_or(0, |rule| {
        let days = rule.winter_base_days.unwrap_or(rule.base_days);
        let per_day = f64::from(rule.burn_per_hour[3]) * f64::from(burn_hours(&rule));
        (days * per_day).ceil().clamp(0.0, f64::from(u16::MAX)) as u16
    })
}

/// Wood units held by settlements (their stockpiles), in total.
pub fn settlement_wood(res: &SimResources) -> u32 {
    res.settlements
        .values()
        .map(|s| s.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0))
        .sum()
}

/// The hearths firewood is brought to: every settlement's formation tile in
/// settlement id order (the tile `feed` measures its pile from); without a
/// settlement, the colony's hearth building.
pub fn hearth_tiles(res: &SimResources) -> Vec<(u32, u32)> {
    let mut ids: Vec<_> = res.settlements.keys().copied().collect();
    ids.sort_unstable();
    let tiles: Vec<(u32, u32)> = ids
        .iter()
        .filter_map(|id| res.settlements.get(id).map(|s| s.formation_tile))
        .collect();
    if tiles.is_empty() {
        crate::worldgen::colony::hearth(res).into_iter().collect()
    } else {
        tiles
    }
}

/// Firewood items a hearth can use: on stockpile cells or within
/// [`HEARTH_PILE_REACH`] of a hearth (reserved stacks included: they are on
/// their way to a use).
pub fn feedable_firewood(res: &SimResources) -> u32 {
    let hearths = hearth_tiles(res);
    let c = &res.colony;
    c.ground_items
        .iter()
        .filter(|(tile, stack)| {
            stack.kind == ItemKind::Firewood
                && (c.zone_of_tile.contains_key(tile)
                    || hearths
                        .iter()
                        .any(|h| chebyshev(**tile, *h) <= HEARTH_PILE_REACH))
        })
        .map(|(_, stack)| u32::from(stack.count))
        .sum()
}

/// Feed every hearth (see the module header).
pub fn feed(res: &mut SimResources) {
    let Some(rule) = res.rules.society.firewood else {
        return;
    };
    let tonight = tonight_burn(res, &rule).ceil().max(0.0) as u32;
    let mut ids: Vec<_> = res.settlements.keys().copied().collect();
    ids.sort_unstable();
    for sid in ids {
        let Some((hearth, have)) = res.settlements.get(&sid).map(|s| {
            (
                s.formation_tile,
                s.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0),
            )
        }) else {
            continue;
        };
        if have >= tonight {
            continue;
        }
        let c = &res.colony;
        let mut stacks: Vec<(u32, u32, u32, u16)> = c
            .ground_items
            .iter()
            .filter(|(tile, stack)| {
                stack.kind == ItemKind::Firewood
                    && (c.zone_of_tile.contains_key(tile)
                        || chebyshev(**tile, hearth) <= HEARTH_PILE_REACH)
                    && !c.reservations.contains_key(&ReserveKey::Stack(**tile))
            })
            .map(|(tile, stack)| (chebyshev(*tile, hearth), tile.1, tile.0, stack.count))
            .collect();
        stacks.sort_unstable();
        let mut need = tonight - have;
        let mut fed = 0u32;
        for (_, y, x, count) in stacks {
            if need == 0 {
                break;
            }
            let take = u32::from(count).min(need) as u16;
            let (taken, _) = super::items::take_items(res, (x, y), ItemKind::Firewood, take);
            need -= u32::from(taken);
            fed += u32::from(taken);
        }
        if fed > 0 {
            res.colony
                .item_ledger
                .burn(ItemKind::Firewood, u64::from(fed));
            if let Some(s) = res.settlements.get_mut(&sid) {
                *s.stockpile.entry(ResourceKind::Wood).or_insert(0) += fed;
            }
        }
    }
}

/// See the module header.
#[derive(Debug, Default)]
pub struct HearthFeedSystem;

impl HearthFeedSystem {
    /// A fresh system.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for HearthFeedSystem {
    fn name(&self) -> &str {
        "HearthFeedSystem"
    }

    fn priority(&self) -> u32 {
        HEARTH_FEED_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        HEARTH_FEED_INTERVAL
    }

    fn tick(&mut self, _world: &mut World, resources: &mut SimResources) {
        if !firewood_by_jobs(resources) {
            return;
        }
        feed(resources);
    }
}
