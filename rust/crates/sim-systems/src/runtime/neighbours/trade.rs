//! M5.5 (ADR 0032): caravans, their goods, barter and the clue an artifact gives (eureka).
//!
//! - A caravan sets out from a neighbour with attitude ≥ `caravan_attitude` (the higher one; when both qualify they
//!   alternate) with 6–10 stacks of goods drawn from a weighted table (`STREAM_TRADE`): items of a node the neighbour
//!   knows and no colonist practises weigh `lack_weight` (3) — clay pot, ground axe, cured hide, spear, hide wrap, fur
//!   cloak — and staples `staple_weight` (1) — meat, flint, clay, hide, firewood; an artifact stack holds 1–2 items,
//!   a staple stack 2–6, at quality 30–70.
//! - Values come from `data/colony/values.ron` (`colony::values::value`). A `Trade` is accepted iff
//!   value(give) × `buy_permille` ≥ value(take) × `sell_permille`. Taken goods are put at the caravan's spot by the
//!   hearth (ledger `traded_in`) and hauled to the stockpiles as any loose stack; given stacks become the caravan's
//!   `owed` list, which haulers carry to the spot (`JobKind::Trade`, ledger `traded_out` on arrival). A trade raises
//!   the neighbour's attitude by `trade_attitude`; a gift (give without take) by one per `gift_value_per_attitude` of
//!   value. Chronicle `Traded`.
//! - Eureka: an artifact the colony holds (anywhere: ground, hands, shelves) whose producing node no living colonist
//!   practises makes that node a clue ([`clues`]): `DiscoverySystem` counts its stockpile conditions as met and
//!   multiplies its chance by `eureka_permille` / 1,000 for adults who practise its prerequisites.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::knowledge::{bit, Knowledge};
use sim_core::colony::neighbours::{Caravan, Goods, VisitState};
use sim_core::colony::values::value;
use sim_core::colony::ItemKind;
use sim_core::rng::{keyed_below, STREAM_TRADE};
use sim_core::rules::NeighbourRule;
use sim_engine::SimResources;
use std::collections::BTreeMap;

use crate::runtime::jobs::items::chebyshev;

/// The artifacts that weigh as "lacked" goods (goal §3.3) with the node that makes each.
pub const ARTIFACTS: [(ItemKind, &str); 6] = [
    (ItemKind::ClayPot, "pot_firing"),
    (ItemKind::GroundAxe, "stone_grinding"),
    (ItemKind::CuredHide, "hide_scraping"),
    (ItemKind::Spear, "hafting"),
    (ItemKind::HideWrap, "hide_scraping"),
    (ItemKind::FurCloak, "hide_scraping"),
];

/// The staples every caravan may bring (goal §3.3).
pub const STAPLES: [ItemKind; 5] = [
    ItemKind::Meat,
    ItemKind::Flint,
    ItemKind::Clay,
    ItemKind::Hide,
    ItemKind::Firewood,
];

/// The knowledge nodes that make `kind` (an artifact's clue): a clay pot the pottery chain that makes it (clay
/// preparation, pot forming, pot firing), a raw pot clay preparation and pot forming, a ground axe stone grinding, a
/// spear or a stone axe hafting, hide goods hide scraping; empty otherwise (M5.5 DECISIONS).
pub fn producing_nodes(kind: ItemKind) -> Vec<u8> {
    let names: &[&str] = match kind {
        ItemKind::ClayPot => &["clay_prep", "pot_forming", "pot_firing"],
        ItemKind::RawPot => &["clay_prep", "pot_forming"],
        ItemKind::GroundAxe => &["stone_grinding"],
        ItemKind::Spear | ItemKind::StoneAxe => &["hafting"],
        ItemKind::CuredHide | ItemKind::HideWrap | ItemKind::FurCloak => &["hide_scraping"],
        _ => &[],
    };
    names
        .iter()
        .filter_map(|n| sim_core::colony::knowledge::id(n))
        .collect()
}

/// The nodes some living colonist practises (bit `i` = node `i`).
pub fn colony_practice(world: &World) -> u32 {
    world
        .query::<&Knowledge>()
        .iter()
        .fold(0, |m, (_, k)| m | k.practice())
}

/// Items of `kind` the colony holds anywhere (created + traded in − consumed − traded out).
pub fn held(res: &SimResources, kind: ItemKind) -> u64 {
    let l = &res.colony.item_ledger;
    (l.created_of(kind) + l.traded_in_of(kind))
        .saturating_sub(l.consumed_of(kind) + l.traded_out_of(kind))
}

/// The clues of the moment (eureka): node → the artifact that hints at it, for each artifact the colony holds whose
/// producing node no living colonist practises (the first artifact by `ItemKind` order when several hint at one node).
pub fn clues(world: &World, res: &SimResources) -> BTreeMap<u8, ItemKind> {
    let practice = colony_practice(world);
    let mut out = BTreeMap::new();
    for kind in ItemKind::ALL {
        if held(res, kind) == 0 {
            continue;
        }
        for node in producing_nodes(kind) {
            if practice & bit(node) == 0 {
                out.entry(node).or_insert(kind);
            }
        }
    }
    out
}

/// The value of every stack on a stockpile cell (the seasonal wave roll's stockpile value).
pub fn stock_value(res: &SimResources) -> u64 {
    res.colony
        .ground_items
        .iter()
        .filter(|(t, _)| res.colony.zone_of_tile.contains_key(*t))
        .map(|(_, s)| u64::from(value(s.kind, s.quality)) * u64::from(s.count))
        .sum()
}

/// The tile by the hearth where a caravan hands over goods: the meeting tile's nearest tile that can hold items and
/// holds none (`(distance, y, x)`), else the meeting tile.
pub fn trade_spot(res: &SimResources, meet: (u32, u32)) -> Option<(u32, u32)> {
    let region = res.nav.region_of(meet);
    for d in 0..8u32 {
        let mut best: Option<(u32, u32)> = None;
        for y in meet.1.saturating_sub(d)..=meet.1 + d {
            for x in meet.0.saturating_sub(d)..=meet.0 + d {
                let t = (x, y);
                if chebyshev(t, meet) != d
                    || !crate::runtime::jobs::items::can_hold_items(res, t)
                    || res.colony.ground_items.contains_key(&t)
                    || res.colony.zone_of_tile.contains_key(&t)
                    || !res.nav.reaches(region, t)
                {
                    continue;
                }
                if best.is_none_or(|b| (t.1, t.0) < (b.1, b.0)) {
                    best = Some(t);
                }
            }
        }
        if best.is_some() {
            return best;
        }
    }
    Some(meet)
}

/// The goods of caravan `id` from a neighbour knowing `known`, the colony practising `practice` (pure in its inputs;
/// see the module header).
pub fn goods(seed: u64, rule: &NeighbourRule, id: u32, known: u32, practice: u32) -> Vec<Goods> {
    let draw =
        |slot: u32, n: u64| keyed_below(seed, STREAM_TRADE, u64::from(id), 0, slot, 0, n.max(1));
    // The weighted table: artifacts the neighbour can make and the colony cannot, then the staples.
    let mut table: Vec<(ItemKind, u32, bool)> = Vec::new();
    for (kind, node) in ARTIFACTS {
        let Some(n) = sim_core::colony::knowledge::id(node) else {
            continue;
        };
        if known & bit(n) != 0 && practice & bit(n) == 0 {
            table.push((kind, rule.lack_weight, true));
        }
    }
    for kind in STAPLES {
        table.push((kind, rule.staple_weight, false));
    }
    let total: u64 = table.iter().map(|t| u64::from(t.1)).sum();
    let (lo, hi) = rule.goods_stacks;
    let stacks = u32::from(lo) + draw(0, u64::from(hi.saturating_sub(lo)) + 1) as u32;
    let (qlo, qhi) = rule.goods_quality;
    let mut out: Vec<Goods> = Vec::new();
    for i in 0..stacks {
        let mut pick = draw(0x10 + i, total);
        let Some(&(kind, _, artifact)) = table.iter().find(|t| {
            if pick < u64::from(t.1) {
                true
            } else {
                pick -= u64::from(t.1);
                false
            }
        }) else {
            continue;
        };
        let count = if artifact {
            1 + draw(0x40 + i, 2) as u16
        } else {
            2 + draw(0x40 + i, 5) as u16
        };
        let quality = qlo + draw(0x80 + i, u64::from(qhi.saturating_sub(qlo)) + 1) as u8;
        out.push(Goods {
            kind,
            count,
            quality,
        });
    }
    out
}

/// The neighbour a caravan comes from: attitude ≥ `caravan_attitude`, the higher one (ties the lower id); when both
/// qualify, not the one that sent the last caravan.
fn caravan_source(res: &SimResources, rule: &NeighbourRule) -> Option<u8> {
    let nb = &res.colony.neighbours;
    let mut ok: Vec<(std::cmp::Reverse<u8>, u8)> = nb
        .list
        .iter()
        .filter(|n| n.attitude >= rule.caravan_attitude)
        .map(|n| (std::cmp::Reverse(n.attitude), n.id))
        .collect();
    ok.sort_unstable();
    if ok.len() >= 2 {
        if let Some(last) = nb.last_caravan_from {
            if let Some(other) = ok.iter().find(|n| n.1 != last) {
                return Some(other.1);
            }
        }
    }
    ok.first().map(|n| n.1)
}

/// Send a caravan today (no-op when no neighbour qualifies or its edge cannot be reached).
pub fn send_caravan(world: &World, res: &mut SimResources, rule: &NeighbourRule) {
    let Some(from) = caravan_source(res, rule) else {
        res.colony.neighbours.tallies.caravans_skipped += 1;
        return;
    };
    let Some(n) = res.colony.neighbours.neighbour(from).cloned() else {
        return;
    };
    let Some(at) = super::edge_tile(res, n.edge) else {
        return;
    };
    let id = res.colony.neighbours.caravans.len() as u32 + 1;
    let goods = goods(res.world_seed, rule, id, n.known, colony_practice(world));
    let day = super::today(res);
    let nb = &mut res.colony.neighbours;
    nb.caravans.push(Caravan {
        id,
        neighbour: from,
        goods,
        pos: at,
        path: Vec::new(),
        state: VisitState::Arriving,
        day,
        spot: None,
        owed: Vec::new(),
        trades: 0,
    });
    nb.last_caravan_from = Some(from);
    nb.tallies.caravans += 1;
    nb.version += 1;
}

/// `amount` owed items of the stack at `from` are on their way to the trading caravan: they leave its `owed` list.
pub fn take_owed(res: &mut SimResources, from: (u32, u32), amount: u16) {
    let Some(c) = res
        .colony
        .neighbours
        .caravans
        .iter_mut()
        .find(|c| matches!(c.state, VisitState::Waiting { .. }))
    else {
        return;
    };
    let mut left = amount;
    for entry in c.owed.iter_mut().filter(|e| e.0 == from) {
        let take = entry.2.min(left);
        entry.2 -= take;
        left -= take;
    }
    c.owed.retain(|e| e.2 > 0);
}

/// Why a trade was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TradeRefusal {
    /// No caravan of that number is trading at the hearth.
    NoCaravan,
    /// A given stack is not the colony's stockpiled stack of that size, or a taken stack is not the caravan's.
    BadItems,
    /// Nothing given and nothing taken.
    Empty,
    /// value(give) × 80 % < value(take) × 120 %.
    Balance,
}

/// The value the traders count for `give` (stacks `(tile, count)`) and `take` (`(goods index, count)`) of caravan
/// `c`: `(give at 100 %, take at 100 %)`, or `None` when an item is not there.
pub fn values_of(
    res: &SimResources,
    c: &Caravan,
    give: &[((u32, u32), u16)],
    take: &[(u16, u16)],
) -> Option<(u64, u64)> {
    let mut given = 0u64;
    let mut owed_on: BTreeMap<(u32, u32), u16> = BTreeMap::new();
    for (t, _, n) in &c.owed {
        *owed_on.entry(*t).or_insert(0) += n;
    }
    let mut asked: BTreeMap<(u32, u32), u16> = BTreeMap::new();
    for &(tile, count) in give {
        let stack = res.colony.ground_items.get(&tile)?;
        let total = asked.entry(tile).or_insert(0);
        *total = total.saturating_add(count);
        let free = stack
            .count
            .saturating_sub(owed_on.get(&tile).copied().unwrap_or(0));
        if count == 0 || *total > free || !res.colony.zone_of_tile.contains_key(&tile) {
            return None;
        }
        given += u64::from(value(stack.kind, stack.quality)) * u64::from(count);
    }
    let mut taken = 0u64;
    let mut asked_goods: BTreeMap<u16, u16> = BTreeMap::new();
    for &(stack, count) in take {
        let g = c.goods.get(usize::from(stack))?;
        let total = asked_goods.entry(stack).or_insert(0);
        *total = total.saturating_add(count);
        if count == 0 || *total > g.count {
            return None;
        }
        taken += u64::from(value(g.kind, g.quality)) * u64::from(count);
    }
    Some((given, taken))
}

/// Whether the traders accept: value(give) × `buy_permille` ≥ value(take) × `sell_permille`.
pub const fn balanced(rule: &NeighbourRule, give: u64, take: u64) -> bool {
    give * rule.buy_permille as u64 >= take * rule.sell_permille as u64
}

/// Apply a `Trade` with caravan `caravan` (see the module header); `Ok(items given + taken)`. `practice` is the nodes
/// the colony practises now (the unknown artifacts bought are counted, F8).
pub fn trade(
    res: &mut SimResources,
    practice: u32,
    caravan: u32,
    give: &[((u32, u32), u16)],
    take: &[(u16, u16)],
) -> Result<u32, TradeRefusal> {
    let rule = super::rule(res).ok_or(TradeRefusal::NoCaravan)?;
    let index = res
        .colony
        .neighbours
        .caravans
        .iter()
        .position(|c| c.id == caravan && matches!(c.state, VisitState::Waiting { .. }))
        .ok_or(TradeRefusal::NoCaravan)?;
    if give.is_empty() && take.is_empty() {
        return Err(TradeRefusal::Empty);
    }
    let c = res.colony.neighbours.caravans[index].clone();
    let (given, taken) = values_of(res, &c, give, take).ok_or(TradeRefusal::BadItems)?;
    if !balanced(&rule, given, taken) {
        res.colony.neighbours.tallies.trades_refused += 1;
        return Err(TradeRefusal::Balance);
    }
    let spot = c.spot.unwrap_or(c.pos);
    // Taken goods: handed over at the spot (they enter the colony's ledger as traded in).
    for &(stack, count) in take {
        let Some(g) = res.colony.neighbours.caravans[index]
            .goods
            .get_mut(usize::from(stack))
        else {
            continue;
        };
        g.count -= count;
        let nodes = producing_nodes(g.kind);
        if !nodes.is_empty() && nodes.iter().any(|n| practice & bit(*n) == 0) {
            res.colony.neighbours.tallies.unknown_bought += u64::from(count);
        }
        let spec = sim_core::colony::ItemSpec {
            kind: g.kind,
            material: None,
            quality: g.quality,
        };
        res.colony.item_ledger.trade_in(g.kind, u64::from(count));
        crate::runtime::jobs::items::put_items(res, spot, spec, count);
    }
    res.colony.neighbours.caravans[index]
        .goods
        .retain(|g| g.count > 0);
    // Given stacks: owed, carried to the spot by haulers.
    for &(tile, count) in give {
        if let Some(kind) = res.colony.ground_items.get(&tile).map(|s| s.kind) {
            res.colony.neighbours.caravans[index]
                .owed
                .push((tile, kind, count));
        }
    }
    let give_value = given * u64::from(rule.buy_permille) / 1000;
    let take_value = taken * u64::from(rule.sell_permille) / 1000;
    let nb = &mut res.colony.neighbours;
    nb.caravans[index].trades += 1;
    let gain = if take.is_empty() {
        nb.tallies.gifts += 1;
        (give_value / u64::from(rule.gift_value_per_attitude.max(1))).min(100) as u8
    } else {
        nb.tallies.trades += 1;
        rule.trade_attitude
    };
    if let Some(n) = nb.list.iter_mut().find(|n| n.id == c.neighbour) {
        n.attitude = n.attitude.saturating_add(gain).min(100);
    }
    nb.tallies.value_given += give_value;
    nb.tallies.value_taken += take_value;
    nb.version += 1;
    res.colony.touch(false);
    res.chronicle.push(
        ChronicleEntry::new(res.current_tick, ChronicleKind::Traded)
            .cause(c.neighbour)
            .value((give_value.min(65_535) << 16 | take_value.min(65_535)) as i64),
    );
    let items: u32 = give.iter().map(|g| u32::from(g.1)).sum::<u32>()
        + take.iter().map(|t| u32::from(t.1)).sum::<u32>();
    Ok(items)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_balance_rule_is_eighty_against_one_hundred_twenty() {
        let rule = NeighbourRule::M5_5;
        // Give 30 for take 20: 30 × 800 = 24,000 ≥ 20 × 1,200 = 24,000 (accepted at the line).
        assert!(balanced(&rule, 30, 20));
        assert!(!balanced(&rule, 29, 20));
        assert!(balanced(&rule, 5, 0), "a gift is always balanced");
    }

    #[test]
    fn a_clay_pot_hints_at_the_pottery_chain() {
        let nodes = producing_nodes(ItemKind::ClayPot);
        let id = |n: &str| sim_core::colony::knowledge::id(n).expect("node");
        assert_eq!(
            nodes,
            vec![id("clay_prep"), id("pot_forming"), id("pot_firing")]
        );
        assert!(producing_nodes(ItemKind::Log).is_empty());
    }
}
