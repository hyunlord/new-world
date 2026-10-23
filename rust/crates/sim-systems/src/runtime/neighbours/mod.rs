//! NeighbourSystem: the two neighbours' day — they are drawn on the first day, their populations drift, and they send migrant waves and caravans on their dates.
//!
//! Priority: 174 (`NEIGHBOUR_PRIORITY`). Interval: 1,440 ticks (the first tick of each day). Gate: `ColonyRule::neighbours`.
//! Reads: `Agent`, `Knowledge`; `SimResources.colony` (neighbours, story, ground items, zones), `settlements` (food), `nav`, `tile_grid`, `building_kinds` (the hearth), `rules`, `world_seed`
//! Writes: `SimResources.colony.neighbours` (list, waves, caravans, plans, tallies, version)
//! Events: none (the visitors' arrival and departure are `VisitorSystem`'s)
//!
//! M5.5 (ADR 0031, 0032). On the first day the two neighbours are drawn from
//! the seed (`STREAM_NEIGHBOURS`, [`generate`]): different map edges, 2–5
//! days away, 20–60 people, attitude 40–60, every Paleolithic node and 2–4
//! Neolithic ones (at least one of the pottery chain between them); waves 1
//! and 2 are planned on keyed days of their windows. On each season's first
//! day a caravan date (its day 2–5) and, after wave 2, a wave roll (its day
//! 2–5) are planned. Each day, in this order: the populations drift ±1 %;
//! the waves due today set out (wave 1 and 2 with their sizes; a roll with
//! chance ‰ min(800, 50 · food-surplus days + stockpile value / 50 + attitude
//! · 2) and 1–4 people) from the neighbour with the higher attitude (ties: the
//! lower id), unless the colony holds 40 people or more; the caravans due
//! today set out from a neighbour with attitude ≥ 30 (the higher one; they
//! alternate when both qualify) unless a major pressure is in force (moved to
//! its end + 1 day, skipped past its season) or it is before day 6.
//! A visitor appears at its neighbour's edge (the reachable tile nearest the
//! edge's middle) and `VisitorSystem` walks it.
use hecs::World;
use sim_core::colony::knowledge::{bit, defs, Era};
use sim_core::colony::neighbours::{Edge, MigrantSpec, Neighbour, VisitState, Wave};
use sim_core::components::Agent;
use sim_core::rng::{keyed_below, keyed_u64, STREAM_NEIGHBOURS};
use sim_core::rules::NeighbourRule;
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::jobs::items::chebyshev;

pub mod join;
pub mod trade;
pub mod visitors;

pub use visitors::{VisitorSystem, VISITOR_PRIORITY};

/// Priority of [`NeighbourSystem`].
pub const NEIGHBOUR_PRIORITY: u32 = 174;

/// Neighbour names in the locale (`NEIGHBOUR_NAME_<n>`, n < this).
pub const NAMES: u8 = 12;

/// The neighbour rule in force (`None` outside the M5.5 colony).
pub fn rule(res: &SimResources) -> Option<NeighbourRule> {
    res.rules.colony.and_then(|c| c.neighbours)
}

/// Today's display day.
pub fn today(res: &SimResources) -> u32 {
    u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX)
}

/// A keyed draw in `lo..=hi` on `STREAM_NEIGHBOURS`.
fn span(seed: u64, subject: u64, tick: u64, slot: u32, (lo, hi): (u64, u64)) -> u64 {
    lo + keyed_below(
        seed,
        STREAM_NEIGHBOURS,
        subject,
        tick,
        slot,
        0,
        hi.saturating_sub(lo) + 1,
    )
}

/// The Neolithic node ids, in id order.
pub fn neolithic() -> Vec<u8> {
    defs()
        .nodes
        .iter()
        .filter(|n| n.era == Era::Neolithic)
        .map(|n| n.id)
        .collect()
}

/// The bits of every Paleolithic node.
pub fn paleolithic_mask() -> u32 {
    defs()
        .nodes
        .iter()
        .filter(|n| n.era == Era::Paleolithic)
        .fold(0, |m, n| m | bit(n.id))
}

/// The pottery chain (a migrant or a caravan may bring it): the nodes of the pottery lane.
pub fn pottery_mask() -> u32 {
    defs()
        .nodes
        .iter()
        .filter(|n| n.lane == sim_core::colony::knowledge::Lane::Pottery)
        .fold(0, |m, n| m | bit(n.id))
}

/// The two neighbours of world `seed` (pure in `(seed, rule)`; see the module header). Subject 0x4E = the
/// neighbours' generation, slot = the neighbour × 16 + the field.
pub fn generate(seed: u64, rule: &NeighbourRule) -> Vec<Neighbour> {
    let draw = |slot: u32, range: (u64, u64)| span(seed, 0x4E, 0, slot, range);
    let first = draw(0, (0, 3)) as u8;
    let second = (first + 1 + draw(1, (0, 2)) as u8) % 4;
    let name0 = draw(2, (0, u64::from(NAMES) - 1)) as u8;
    let name1 = (name0 + 1 + draw(3, (0, u64::from(NAMES) - 2)) as u8) % NAMES;
    let neo = neolithic();
    let mut list: Vec<Neighbour> = (0..2u8)
        .map(|id| {
            let s = u32::from(id) * 16 + 16;
            // 2–4 Neolithic nodes: a keyed shuffle of them, the first k.
            let mut order = neo.clone();
            for i in (1..order.len()).rev() {
                let j = draw(s + 5 + i as u32, (0, i as u64)) as usize;
                order.swap(i, j);
            }
            let (lo, hi) = rule.neolithic_known;
            let k = draw(s + 4, (u64::from(lo), u64::from(hi))) as usize;
            let known = order
                .iter()
                .take(k.min(order.len()))
                .fold(paleolithic_mask(), |m, id| m | bit(*id));
            Neighbour {
                id,
                name_key: if id == 0 { name0 } else { name1 },
                edge: Edge::from_code(if id == 0 { first } else { second }),
                distance_days: draw(
                    s,
                    (
                        u64::from(rule.distance_days.0),
                        u64::from(rule.distance_days.1),
                    ),
                ) as u8,
                population: draw(
                    s + 1,
                    (u64::from(rule.population.0), u64::from(rule.population.1)),
                ) as u16,
                attitude: draw(
                    s + 2,
                    (
                        u64::from(rule.attitude_start.0),
                        u64::from(rule.attitude_start.1),
                    ),
                ) as u8,
                known,
                last_contact_day: None,
            }
        })
        .collect();
    // At least one neighbour knows a node of the pottery chain (goal §3.2): the first takes pot forming.
    if list.iter().all(|n| n.known & pottery_mask() == 0) {
        if let Some(id) = sim_core::colony::knowledge::id("pot_forming") {
            list[0].known |= bit(id);
        }
    }
    list
}

/// The days and sizes of waves 1 and 2 (pure in `(seed, rule)`).
pub fn first_waves(seed: u64, rule: &NeighbourRule) -> Vec<(u32, u8)> {
    let draw = |slot: u32, (lo, hi): (u32, u32)| {
        span(seed, 0x57, 0, slot, (u64::from(lo), u64::from(hi))) as u32
    };
    let size = |slot: u32, (lo, hi): (u8, u8)| draw(slot, (u32::from(lo), u32::from(hi))) as u8;
    vec![
        (draw(0, rule.wave1_days), size(1, rule.wave1_size)),
        (draw(2, rule.wave2_days), size(3, rule.wave2_size)),
    ]
}

/// The tile a visitor of `edge` appears on and leaves from: the passable tile of that edge nearest its middle
/// (ties `(y, x)`) that the hearth's region reaches; else any reachable edge tile nearest the edge's middle.
pub fn edge_tile(res: &SimResources, edge: Edge) -> Option<(u32, u32)> {
    let hearth = crate::worldgen::colony::hearth(res)?;
    let region = res.nav.region_of(hearth);
    let (w, h) = (res.tile_grid.width, res.tile_grid.height);
    let (mx, my) = (w / 2, h / 2);
    let line: Vec<(u32, u32)> = match edge {
        Edge::North => (0..w).map(|x| (x, 0)).collect(),
        Edge::South => (0..w).map(|x| (x, h.saturating_sub(1))).collect(),
        Edge::West => (0..h).map(|y| (0, y)).collect(),
        Edge::East => (0..h).map(|y| (w.saturating_sub(1), y)).collect(),
    };
    let middle = match edge {
        Edge::North => (mx, 0),
        Edge::South => (mx, h.saturating_sub(1)),
        Edge::West => (0, my),
        Edge::East => (w.saturating_sub(1), my),
    };
    let ok = |t: &(u32, u32)| res.nav.passable(*t) && res.nav.reaches(region, *t);
    line.iter()
        .copied()
        .filter(ok)
        .min_by_key(|t| (chebyshev(*t, middle), t.1, t.0))
        .or_else(|| crate::runtime::wildlife::wolf_edge(res, hearth))
}

/// The tile at the hearth where visitors stand: the nearest passable tile to the hearth that holds no building
/// (`(distance, y, x)`), reachable from the hearth's region.
pub fn meeting_tile(res: &SimResources) -> Option<(u32, u32)> {
    let hearth = crate::worldgen::colony::hearth(res)?;
    let region = res.nav.region_of(hearth);
    for d in 1..12u32 {
        let mut best: Option<(u32, u32)> = None;
        for y in hearth.1.saturating_sub(d)..=hearth.1 + d {
            for x in hearth.0.saturating_sub(d)..=hearth.0 + d {
                let t = (x, y);
                if chebyshev(t, hearth) != d
                    || x >= res.tile_grid.width
                    || y >= res.tile_grid.height
                    || !res.nav.passable(t)
                    || !res.nav.reaches(region, t)
                    || crate::runtime::jobs::items::under_building(res, t)
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
    None
}

/// People living in the colony (adults and children: the population cap counts both).
pub fn colony_people(world: &World) -> u32 {
    world.query::<&Agent>().iter().count() as u32
}

/// Food-surplus days (goal §3.2): stored food / (adults × a day's food), whole days.
pub fn surplus_days(world: &World, res: &SimResources, rule: &NeighbourRule) -> u32 {
    use sim_core::components::{Child, ResourceKind};
    let adults = world.query::<&Agent>().without::<&Child>().iter().count() as u32;
    let mut ids: Vec<_> = res.settlements.keys().copied().collect();
    ids.sort_unstable();
    let food: u32 = ids
        .iter()
        .filter_map(|id| res.settlements.get(id))
        .map(|s| s.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0))
        .sum();
    food / (adults.max(1) * rule.food_per_adult_day.max(1))
}

/// A seasonal wave's chance ‰ (goal §3.2).
pub fn roll_permille(
    rule: &NeighbourRule,
    surplus_days: u32,
    stock_value: u64,
    attitude: u8,
) -> u32 {
    let raw = u64::from(rule.surplus_permille) * u64::from(surplus_days)
        + stock_value / u64::from(rule.value_divisor.max(1))
        + u64::from(attitude) * u64::from(rule.attitude_permille);
    raw.min(u64::from(rule.roll_max_permille)) as u32
}

/// The neighbour a wave comes from: the higher attitude, ties the lower id.
fn wave_source(list: &[Neighbour]) -> Option<u8> {
    list.iter()
        .min_by_key(|n| (std::cmp::Reverse(n.attitude), n.id))
        .map(|n| n.id)
}

/// The people of wave `id` from neighbour `n` (pure in `(seed, rule, n.known, id, size, tick)`): lineage keys keyed on
/// the wave and the place in it, their knowledge (every Paleolithic node the neighbour knows at 450–700 ‰; each of its
/// Neolithic nodes with chance ½ at 420–650 ‰), a hide wrap with chance ½; ages from the founders' aging rule.
pub fn migrants(
    seed: u64,
    rule: &NeighbourRule,
    n: &Neighbour,
    wave: u32,
    size: u8,
    aging: Option<sim_core::rules::AgingRule>,
) -> Vec<MigrantSpec> {
    let d = defs();
    (0..u32::from(size))
        .map(|i| {
            let key = keyed_u64(seed, STREAM_NEIGHBOURS, 0x4D49_4752, u64::from(wave), i, 0);
            let draw = |slot: u32, (lo, hi): (u64, u64)| span(seed, key, 0, slot, (lo, hi));
            let mut knowledge = Vec::new();
            for node in d.nodes.iter().filter(|node| n.known & bit(node.id) != 0) {
                let s = 0x100 + u32::from(node.id);
                let p = match node.era {
                    Era::Paleolithic => draw(
                        s,
                        (
                            u64::from(rule.paleo_permille.0),
                            u64::from(rule.paleo_permille.1),
                        ),
                    ),
                    Era::Neolithic => {
                        if draw(s + 0x40, (0, 999)) >= u64::from(rule.neo_chance_permille) {
                            continue;
                        }
                        draw(
                            s,
                            (
                                u64::from(rule.neo_permille.0),
                                u64::from(rule.neo_permille.1),
                            ),
                        )
                    }
                };
                knowledge.push((node.id, p as u16));
            }
            let year = sim_core::rules::CalendarRule::COLONY
                .years_to_ticks(1)
                .max(1);
            let age_years = aging.map_or(25, |rule| {
                let life = crate::runtime::agent::aging::keyed_founder_life(seed, key, 0, &rule);
                ((-life.born_tick).max(0) as u64 / year) as u16
            });
            MigrantSpec {
                key,
                female: sim_core::components::Sex::from_lineage(seed, key).female,
                age_years,
                knowledge,
                wrap: draw(1, (0, 999)) < u64::from(rule.wrap_permille),
            }
        })
        .collect()
}

/// Send wave `size` from the neighbour with the higher attitude today (no-op when the colony is at its cap or no
/// edge is reachable).
fn send_wave(world: &World, res: &mut SimResources, rule: &NeighbourRule, size: u8) {
    if colony_people(world) >= rule.population_cap {
        res.colony.neighbours.tallies.capped += 1;
        return;
    }
    let Some(from) = wave_source(&res.colony.neighbours.list) else {
        return;
    };
    let Some(n) = res.colony.neighbours.neighbour(from).cloned() else {
        return;
    };
    let Some(at) = edge_tile(res, n.edge) else {
        return;
    };
    let id = res.colony.neighbours.waves.len() as u32 + 1;
    let people = migrants(res.world_seed, rule, &n, id, size, res.rules.society.aging);
    let day = today(res);
    let nb = &mut res.colony.neighbours;
    nb.waves.push(Wave {
        id,
        neighbour: from,
        people,
        pos: at,
        path: Vec::new(),
        state: VisitState::Arriving,
        day,
        answer: None,
        joined: Vec::new(),
    });
    nb.tallies.waves += 1;
    nb.version += 1;
}

/// Plan season `index`'s caravan date and, after wave 2, its wave roll (keyed on the season index).
fn plan_season(res: &mut SimResources, rule: &NeighbourRule, index: u32, per_season: u32) {
    let seed = res.world_seed;
    let first_day = index * per_season;
    let days = per_season.saturating_sub(2).max(1);
    let day_of = |slot: u32| {
        first_day + 1 + span(seed, 0x53, u64::from(index), slot, (0, u64::from(days - 1))) as u32
    };
    let wave2 = res
        .colony
        .neighbours
        .planned_waves
        .iter()
        .map(|w| w.0)
        .chain(res.colony.neighbours.waves.iter().map(|w| w.day))
        .max()
        .unwrap_or(rule.wave2_days.1);
    let nb = &mut res.colony.neighbours;
    if rule.caravans {
        nb.planned_caravans.push((day_of(0), 0));
    }
    // Seasonal waves begin with the first season that starts after wave 2 (goal §3.2: "then one roll per season").
    if first_day > wave2.max(rule.wave2_days.1) {
        nb.planned_waves.push((day_of(1), 0));
    }
    nb.planned_waves.sort_unstable();
    nb.planned_caravans.sort_unstable();
}

/// See the module header.
#[derive(Debug, Default)]
pub struct NeighbourSystem;

impl RuntimeSystem for NeighbourSystem {
    fn name(&self) -> &str {
        "NeighbourSystem"
    }

    fn priority(&self) -> u32 {
        NEIGHBOUR_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1_440
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = rule(res) else {
            return;
        };
        crate::runtime::nav::ensure(res);
        let day = today(res);
        if day < res.colony.neighbours.next_day {
            return;
        }
        res.colony.neighbours.next_day = day + 1;
        let seed = res.world_seed;
        if res.colony.neighbours.list.is_empty() {
            res.colony.neighbours.list = generate(seed, &rule);
            res.colony.neighbours.planned_waves = first_waves(seed, &rule);
            res.colony.neighbours.version += 1;
        }
        // Populations drift ±1 % (at least one person), keyed on the neighbour and the day.
        for n in &mut res.colony.neighbours.list {
            let step = (n.population / 100).max(1);
            match keyed_below(
                seed,
                STREAM_NEIGHBOURS,
                0x0050_4F50 + u64::from(n.id),
                u64::from(day),
                0,
                0,
                3,
            ) {
                0 => n.population = n.population.saturating_sub(step).max(1),
                2 => n.population = n.population.saturating_add(step),
                _ => {}
            }
        }
        // Plan the season on its first day.
        let per_season = res.rules.season_days().max(1);
        let index = day / per_season;
        if index >= res.colony.neighbours.seasons_planned {
            res.colony.neighbours.seasons_planned = index + 1;
            plan_season(res, &rule, index, per_season);
        }
        // Waves due today.
        let due: Vec<(u32, u8)> = res
            .colony
            .neighbours
            .planned_waves
            .iter()
            .filter(|w| w.0 <= day)
            .copied()
            .collect();
        res.colony.neighbours.planned_waves.retain(|w| w.0 > day);
        for (_, size) in due {
            if size > 0 {
                send_wave(world, res, &rule, size);
                continue;
            }
            // A seasonal roll.
            if colony_people(world) >= rule.population_cap {
                res.colony.neighbours.tallies.capped += 1;
                continue;
            }
            let attitude = wave_source(&res.colony.neighbours.list)
                .and_then(|id| res.colony.neighbours.neighbour(id))
                .map_or(0, |n| n.attitude);
            let chance = roll_permille(
                &rule,
                surplus_days(world, res, &rule),
                trade::stock_value(res),
                attitude,
            );
            res.colony.neighbours.tallies.rolls += 1;
            if keyed_below(
                seed,
                STREAM_NEIGHBOURS,
                0x524F_4C4C,
                u64::from(day),
                0,
                0,
                1000,
            ) < u64::from(chance)
            {
                res.colony.neighbours.tallies.roll_wins += 1;
                let size = span(
                    seed,
                    0x524F_4C4C,
                    u64::from(day),
                    1,
                    (u64::from(rule.roll_size.0), u64::from(rule.roll_size.1)),
                );
                send_wave(world, res, &rule, size as u8);
            }
        }
        // Caravans due today.
        let due: Vec<(u32, u8)> = res
            .colony
            .neighbours
            .planned_caravans
            .iter()
            .filter(|c| c.0 <= day)
            .copied()
            .collect();
        res.colony.neighbours.planned_caravans.retain(|c| c.0 > day);
        for (planned, _) in due {
            // Never during a major pressure: moved to its end + 1 day inside the season, else skipped.
            if let Some(end) = res
                .colony
                .story
                .active
                .iter()
                .filter(|p| p.kind.is_major())
                .map(|p| p.end_day)
                .max()
            {
                let season_end = (planned / per_season + 1) * per_season;
                if end + 1 < season_end {
                    res.colony.neighbours.planned_caravans.push((end + 1, 0));
                    res.colony.neighbours.planned_caravans.sort_unstable();
                } else {
                    res.colony.neighbours.tallies.caravans_skipped += 1;
                }
                continue;
            }
            if day < rule.caravan_first_day {
                res.colony.neighbours.tallies.caravans_skipped += 1;
                continue;
            }
            trade::send_caravan(world, res, &rule);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata() {
        let s = NeighbourSystem;
        assert_eq!(
            (s.name(), s.priority(), s.tick_interval()),
            ("NeighbourSystem", 174, 1_440)
        );
    }

    #[test]
    fn the_roll_chance_follows_the_goal_formula() {
        let rule = NeighbourRule::M5_5;
        // 50 × 4 + 1,000 / 50 + 50 × 2 = 200 + 20 + 100.
        assert_eq!(roll_permille(&rule, 4, 1_000, 50), 320);
        assert_eq!(roll_permille(&rule, 40, 100_000, 100), 800, "capped");
        assert_eq!(roll_permille(&rule, 0, 0, 0), 0);
    }
}
