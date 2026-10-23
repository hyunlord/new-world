//! M5.5 bridge (ADR 0031, 0032): the neighbours and their visitors, one waiting wave's people, one caravan's goods
//! with the colony's stock, the snapshot's `fleeing` column; and the migrant and trade orders.
//!
//! Reads are pure collectors, rows in a documented order: [`collect_neighbours`] (neighbours by id, waves and
//! caravans by number, clues by node id), [`collect_wave`] (the wave's people in their order), [`collect_caravan`]
//! (goods in the caravan's order — the `take` index of a trade — and the colony's stockpiled stacks in `(y, x)`
//! order), [`collect_fleeing_column`] (the agent snapshot's row order). Ids: people `AgentId`, waves and caravans their
//! numbers, nodes `KnowledgeId`, items `ItemKind::code`. Every read returns its empty form in the baseline world and
//! under rules without neighbours.
use godot::prelude::*;
use sim_core::colony::knowledge::{bit, defs, Era};
use sim_core::colony::neighbours::{VisitState, WaveAnswer};
use sim_core::colony::people::Fleeing;
use sim_core::colony::values::value;
use sim_core::colony::{Command, CommandResult, ItemKind, RejectReason};
use sim_core::components::{Agent, Position};
use sim_engine::{SimEngine, SimResources};
use sim_systems::commands::submit;
use sim_systems::runtime::neighbours::{self, trade};

/// One neighbour row: (id, name key index, edge code, distance days, population, attitude, last contact day or −1,
/// Neolithic nodes known as a bitmask).
pub type NeighbourRow = (u8, u8, u8, u8, u16, u8, i64, i64);
/// One wave row: (id, neighbour, state code, x, y, size, deadline tick or −1, answer −1/0/1/2).
pub type WaveRow = (u32, u8, u8, u32, u32, u32, i64, i32);
/// One caravan row: (id, neighbour, state code, x, y, leaving tick or −1, trades).
pub type CaravanRow = (u32, u8, u8, u32, u32, i64, u32);
/// One migrant: (the `AgentId` they will have, female, age in years, arrives wearing a top, top 3 knowledge
/// `(node, ‰, practitioner)`, the worn top's `Top::code` (0 none; the data's `migrant_top`)).
pub type MigrantRow = (u64, bool, u16, bool, [(i32, i32, bool); 3], u8);
/// One goods row: (item code, count, quality, unit value at 100 %, unknown to the colony).
pub type GoodsRow = (u8, u16, u8, u32, bool);
/// One stock row: (x, y, item code, count, quality, unit value at 100 %).
pub type StockRow = (u32, u32, u8, u16, u8, u32);

/// The neighbours and their visitors (`get_neighbours`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NeighboursView {
    /// `Neighbours::version`.
    pub version: u64,
    /// The current tick.
    pub now: u64,
    /// Neighbours by id.
    pub neighbours: Vec<NeighbourRow>,
    /// Waves by number.
    pub waves: Vec<WaveRow>,
    /// Caravans by number.
    pub caravans: Vec<CaravanRow>,
    /// Clues by node id: (node, the artifact's item code).
    pub clues: Vec<(u8, u8)>,
    /// Tallies: migrants joined, trades, gifts, eurekas.
    pub tallies: (u64, u64, u64, u64),
}

fn deadline(state: VisitState, days: u32, day_ticks: u64) -> i64 {
    match state {
        VisitState::Waiting { since } => {
            i64::try_from(since + day_ticks * u64::from(days)).unwrap_or(-1)
        }
        _ => -1,
    }
}

/// The neighbours, waves, caravans and clues (see the module header).
pub fn collect_neighbours(world: &hecs::World, res: &SimResources) -> NeighboursView {
    let Some(rule) = neighbours::rule(res) else {
        return NeighboursView::default();
    };
    let nb = &res.colony.neighbours;
    let day_ticks = res.rules.calendar.days_to_ticks(1);
    let neo = defs()
        .nodes
        .iter()
        .filter(|n| n.era == Era::Neolithic)
        .fold(0u32, |m, n| m | bit(n.id));
    let mut list = nb.list.clone();
    list.sort_by_key(|n| n.id);
    NeighboursView {
        version: nb.version,
        now: res.current_tick,
        neighbours: list
            .iter()
            .map(|n| {
                (
                    n.id,
                    n.name_key,
                    n.edge.code(),
                    n.distance_days,
                    n.population,
                    n.attitude,
                    n.last_contact_day.map_or(-1, i64::from),
                    i64::from(n.known & neo),
                )
            })
            .collect(),
        waves: nb
            .waves
            .iter()
            .map(|w| {
                let answer = match w.answer {
                    None => -1,
                    Some(WaveAnswer::Accepted) => 0,
                    Some(WaveAnswer::Declined) => 1,
                    Some(WaveAnswer::Unanswered) => 2,
                };
                (
                    w.id,
                    w.neighbour,
                    w.state.code(),
                    w.pos.0,
                    w.pos.1,
                    w.people.len() as u32,
                    deadline(w.state, rule.wait_days, day_ticks),
                    answer,
                )
            })
            .collect(),
        caravans: nb
            .caravans
            .iter()
            .map(|c| {
                (
                    c.id,
                    c.neighbour,
                    c.state.code(),
                    c.pos.0,
                    c.pos.1,
                    deadline(c.state, rule.caravan_stay_days, day_ticks),
                    c.trades,
                )
            })
            .collect(),
        clues: trade::clues(world, res)
            .into_iter()
            .map(|(node, kind)| (node, kind.code()))
            .collect(),
        tallies: (
            nb.tallies.migrants_joined,
            nb.tallies.trades,
            nb.tallies.gifts,
            nb.tallies.eurekas,
        ),
    }
}

/// Marshal [`NeighboursView`].
pub(super) fn neighbours_to_dict(v: &NeighboursView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("version", i64::try_from(v.version).unwrap_or(i64::MAX));
    d.set("now", i64::try_from(v.now).unwrap_or(i64::MAX));
    let i32s = |f: &dyn Fn(usize) -> i32, n: usize| {
        PackedInt32Array::from((0..n).map(f).collect::<Vec<_>>().as_slice())
    };
    let n = v.neighbours.len();
    d.set("ids", i32s(&|i| i32::from(v.neighbours[i].0), n));
    // The locale key of each neighbour's name (`data/colony/neighbours.ron`), as `get_tech`'s `name_keys`.
    let keys: Vec<GString> = v
        .neighbours
        .iter()
        .map(|r| GString::from(neighbours::name_key(r.1)))
        .collect();
    d.set("name_keys", PackedStringArray::from(keys.as_slice()));
    d.set("edges", i32s(&|i| i32::from(v.neighbours[i].2), n));
    d.set("distance_days", i32s(&|i| i32::from(v.neighbours[i].3), n));
    d.set("populations", i32s(&|i| i32::from(v.neighbours[i].4), n));
    d.set("attitudes", i32s(&|i| i32::from(v.neighbours[i].5), n));
    d.set("last_contact", i32s(&|i| v.neighbours[i].6 as i32, n));
    let rumoured: Vec<i64> = v.neighbours.iter().map(|r| r.7).collect();
    d.set("rumoured", PackedInt64Array::from(rumoured.as_slice()));
    let w = v.waves.len();
    d.set("wave_ids", i32s(&|i| v.waves[i].0 as i32, w));
    d.set("wave_neighbours", i32s(&|i| i32::from(v.waves[i].1), w));
    d.set("wave_states", i32s(&|i| i32::from(v.waves[i].2), w));
    d.set("wave_x", i32s(&|i| v.waves[i].3 as i32, w));
    d.set("wave_y", i32s(&|i| v.waves[i].4 as i32, w));
    d.set("wave_sizes", i32s(&|i| v.waves[i].5 as i32, w));
    let wd: Vec<i64> = v.waves.iter().map(|r| r.6).collect();
    d.set("wave_deadline", PackedInt64Array::from(wd.as_slice()));
    d.set("wave_answers", i32s(&|i| v.waves[i].7, w));
    let c = v.caravans.len();
    d.set("caravan_ids", i32s(&|i| v.caravans[i].0 as i32, c));
    d.set(
        "caravan_neighbours",
        i32s(&|i| i32::from(v.caravans[i].1), c),
    );
    d.set("caravan_states", i32s(&|i| i32::from(v.caravans[i].2), c));
    d.set("caravan_x", i32s(&|i| v.caravans[i].3 as i32, c));
    d.set("caravan_y", i32s(&|i| v.caravans[i].4 as i32, c));
    let cl: Vec<i64> = v.caravans.iter().map(|r| r.5).collect();
    d.set("caravan_leaves", PackedInt64Array::from(cl.as_slice()));
    d.set("caravan_trades", i32s(&|i| v.caravans[i].6 as i32, c));
    let k = v.clues.len();
    d.set("clue_nodes", i32s(&|i| i32::from(v.clues[i].0), k));
    d.set("clue_items", i32s(&|i| i32::from(v.clues[i].1), k));
    d.set(
        "migrants_joined",
        i64::try_from(v.tallies.0).unwrap_or(i64::MAX),
    );
    d.set("trades", i64::try_from(v.tallies.1).unwrap_or(i64::MAX));
    d.set("gifts", i64::try_from(v.tallies.2).unwrap_or(i64::MAX));
    d.set("eurekas", i64::try_from(v.tallies.3).unwrap_or(i64::MAX));
    d
}

/// One wave and its people (`get_wave`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WaveView {
    /// Whether the wave exists.
    pub found: bool,
    /// Its number.
    pub id: u32,
    /// Its neighbour.
    pub neighbour: u8,
    /// `VisitState::code`.
    pub state: u8,
    /// The tick an unanswered wave leaves (−1 unless waiting).
    pub deadline: i64,
    /// People in order: (the `AgentId` they will have, female, age in years, arrives in a wrap, top 3 knowledge
    /// `(node, ‰, practitioner)` padded with `(−1, 0, false)`).
    pub people: Vec<MigrantRow>,
}

/// Wave `id` (see the module header).
pub fn collect_wave(res: &SimResources, id: i64) -> WaveView {
    let Some(rule) = neighbours::rule(res) else {
        return WaveView::default();
    };
    let Some(w) = u32::try_from(id)
        .ok()
        .and_then(|id| res.colony.neighbours.waves.iter().find(|w| w.id == id))
    else {
        return WaveView::default();
    };
    let d = defs();
    // The worn top's `Top::code` (0 none): the data's `migrant_top` for those who arrive in one.
    let top_code =
        sim_systems::runtime::neighbours::join::migrant_top().map_or(0, |(_, top)| top.code());
    WaveView {
        found: true,
        id: w.id,
        neighbour: w.neighbour,
        state: w.state.code(),
        deadline: deadline(w.state, rule.wait_days, res.rules.calendar.days_to_ticks(1)),
        people: w
            .people
            .iter()
            .map(|p| {
                let mut k = p.knowledge.clone();
                k.sort_by_key(|(node, pm)| (std::cmp::Reverse(*pm), *node));
                let mut top = [(-1, 0, false); 3];
                for (slot, (node, pm)) in top.iter_mut().zip(k.iter()) {
                    *slot = (i32::from(*node), i32::from(*pm), *pm >= d.threshold(*node));
                }
                let worn = if p.wrap { top_code } else { 0 };
                (p.agent, p.female, p.age_years, p.wrap, top, worn)
            })
            .collect(),
    }
}

/// Marshal [`WaveView`].
pub(super) fn wave_to_dict(v: &WaveView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", v.found);
    d.set("id", i64::from(v.id));
    d.set("neighbour", i64::from(v.neighbour));
    d.set("state", i64::from(v.state));
    d.set("deadline", v.deadline);
    let agents: Vec<i64> = v
        .people
        .iter()
        .map(|p| i64::try_from(p.0).unwrap_or(-1))
        .collect();
    d.set("agents", PackedInt64Array::from(agents.as_slice()));
    let flag = |f: &dyn Fn(&MigrantRow) -> i32| {
        PackedInt32Array::from(v.people.iter().map(f).collect::<Vec<_>>().as_slice())
    };
    d.set("female", flag(&|p| i32::from(p.1)));
    d.set("age_years", flag(&|p| i32::from(p.2)));
    d.set("wrap", flag(&|p| i32::from(p.3)));
    // The worn top's `Top::code` per person (0 none): the data's `migrant_top` for those who arrive in one.
    d.set("tops", flag(&|p| i32::from(p.5)));
    let flat = |f: &dyn Fn(&(i32, i32, bool)) -> i32| {
        PackedInt32Array::from(
            v.people
                .iter()
                .flat_map(|p| p.4.iter().map(f))
                .collect::<Vec<_>>()
                .as_slice(),
        )
    };
    d.set("know_nodes", flat(&|k| k.0));
    d.set("know_permille", flat(&|k| k.1));
    d.set("know_practice", flat(&|k| i32::from(k.2)));
    d
}

/// One caravan's goods and the colony's stock (`get_caravan`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CaravanView {
    /// Whether the caravan exists.
    pub found: bool,
    /// Its number.
    pub id: u32,
    /// Its neighbour.
    pub neighbour: u8,
    /// `VisitState::code`.
    pub state: u8,
    /// The tick it leaves (−1 unless trading).
    pub leaves: i64,
    /// The traders' buy and sell ‰.
    pub buy_permille: u32,
    /// See `buy_permille`.
    pub sell_permille: u32,
    /// Goods in order: (item code, count, quality, unit value at 100 %, unknown to the colony).
    pub goods: Vec<GoodsRow>,
    /// The colony's stockpiled stacks in `(y, x)` order: (x, y, item code, count, quality, unit value at 100 %).
    pub stock: Vec<StockRow>,
}

/// Caravan `id` and the colony's stockpiled stacks (see the module header).
pub fn collect_caravan(world: &hecs::World, res: &SimResources, id: i64) -> CaravanView {
    let Some(rule) = neighbours::rule(res) else {
        return CaravanView::default();
    };
    let Some(c) = u32::try_from(id)
        .ok()
        .and_then(|id| res.colony.neighbours.caravans.iter().find(|c| c.id == id))
    else {
        return CaravanView::default();
    };
    let practice = trade::colony_practice(world);
    let unknown = |kind: ItemKind| {
        let nodes = trade::producing_nodes(kind);
        !nodes.is_empty() && nodes.iter().any(|n| practice & bit(*n) == 0)
    };
    let mut stock: Vec<StockRow> = res
        .colony
        .ground_items
        .iter()
        .filter(|(t, s)| res.colony.zone_of_tile.contains_key(*t) && s.count > 0)
        .map(|(t, s)| {
            (
                t.0,
                t.1,
                s.kind.code(),
                s.count,
                s.quality,
                value(s.kind, s.quality),
            )
        })
        .collect();
    stock.sort_by_key(|r| (r.1, r.0));
    CaravanView {
        found: true,
        id: c.id,
        neighbour: c.neighbour,
        state: c.state.code(),
        leaves: deadline(
            c.state,
            rule.caravan_stay_days,
            res.rules.calendar.days_to_ticks(1),
        ),
        buy_permille: rule.buy_permille,
        sell_permille: rule.sell_permille,
        goods: c
            .goods
            .iter()
            .map(|g| {
                (
                    g.kind.code(),
                    g.count,
                    g.quality,
                    value(g.kind, g.quality),
                    unknown(g.kind),
                )
            })
            .collect(),
        stock,
    }
}

/// Marshal [`CaravanView`].
pub(super) fn caravan_to_dict(v: &CaravanView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", v.found);
    d.set("id", i64::from(v.id));
    d.set("neighbour", i64::from(v.neighbour));
    d.set("state", i64::from(v.state));
    d.set("leaves", v.leaves);
    d.set("buy_permille", i64::from(v.buy_permille));
    d.set("sell_permille", i64::from(v.sell_permille));
    let g = |f: &dyn Fn(&GoodsRow) -> i32| {
        PackedInt32Array::from(v.goods.iter().map(f).collect::<Vec<_>>().as_slice())
    };
    d.set("goods_kind", g(&|r| i32::from(r.0)));
    d.set("goods_count", g(&|r| i32::from(r.1)));
    d.set("goods_quality", g(&|r| i32::from(r.2)));
    d.set("goods_value", g(&|r| r.3 as i32));
    d.set("goods_unknown", g(&|r| i32::from(r.4)));
    let s = |f: &dyn Fn(&StockRow) -> i32| {
        PackedInt32Array::from(v.stock.iter().map(f).collect::<Vec<_>>().as_slice())
    };
    d.set("stock_x", s(&|r| r.0 as i32));
    d.set("stock_y", s(&|r| r.1 as i32));
    d.set("stock_kind", s(&|r| i32::from(r.2)));
    d.set("stock_count", s(&|r| i32::from(r.3)));
    d.set("stock_quality", s(&|r| i32::from(r.4)));
    d.set("stock_value", s(&|r| r.5 as i32));
    d
}

/// The snapshot's `fleeing` column: 1 for a person running from a wolf, in the agent snapshot's row order.
pub fn collect_fleeing_column(world: &hecs::World) -> Vec<u8> {
    world
        .query::<(&Agent, &Position, Option<&Fleeing>)>()
        .iter()
        .map(|(_, (_, _, f))| u8::from(f.is_some()))
        .collect()
}

/// An M5.5 order from the bridge (ids as given; a negative id is refused as unknown).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NeighbourCommand {
    /// Accept a wave.
    Accept(i64),
    /// Decline a wave.
    Decline(i64),
    /// Trade with a caravan: give flat `(x, y, count)` triples, take flat `(goods index, count)` pairs.
    Trade(i64, Vec<i32>, Vec<i32>),
}

/// Submit an M5.5 order (unknown ids and malformed lists are refused as `Unknown`).
pub fn submit_neighbours(engine: &mut SimEngine, cmd: NeighbourCommand) -> CommandResult {
    let unknown = CommandResult::rejected(RejectReason::Unknown);
    let command = match cmd {
        NeighbourCommand::Accept(w) => u32::try_from(w)
            .ok()
            .map(|wave| Command::AcceptMigrants { wave }),
        NeighbourCommand::Decline(w) => u32::try_from(w)
            .ok()
            .map(|wave| Command::DeclineMigrants { wave }),
        NeighbourCommand::Trade(c, give, take) => {
            let give: Option<Vec<(u32, u32, u16)>> = (give.len() % 3 == 0)
                .then(|| {
                    give.chunks(3)
                        .map(|t| {
                            Some((
                                u32::try_from(t[0]).ok()?,
                                u32::try_from(t[1]).ok()?,
                                u16::try_from(t[2]).ok()?,
                            ))
                        })
                        .collect()
                })
                .flatten();
            let take: Option<Vec<(u16, u16)>> = (take.len() % 2 == 0)
                .then(|| {
                    take.chunks(2)
                        .map(|t| Some((u16::try_from(t[0]).ok()?, u16::try_from(t[1]).ok()?)))
                        .collect()
                })
                .flatten();
            match (u32::try_from(c).ok(), give, take) {
                (Some(caravan), Some(give), Some(take)) => Some(Command::Trade {
                    caravan,
                    give,
                    take,
                }),
                _ => None,
            }
        }
    };
    match command {
        Some(cmd) => submit(engine, cmd),
        None => unknown,
    }
}
