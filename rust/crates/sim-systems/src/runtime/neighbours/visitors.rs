//! VisitorSystem: walks migrant waves and caravans from their edge to the hearth, ends their stay, and walks them back out.
//!
//! Priority: 175 (`VISITOR_PRIORITY`). Interval: 3 ticks, offset 1. Gate: `ColonyRule::neighbours`.
//! Reads: `SimResources.colony.neighbours`, `nav`, `building_kinds` (the hearth), `rules`
//! Writes: ECS people (a wave accepted on its way joins on arrival, `join::accept`); `SimResources.colony.neighbours` (waves, caravans: position, path, state, answer; neighbours' attitude and last contact; tallies, version), `chronicle`
//! Events: chronicle `MigrantsArrived`, `MigrantsDeclined` (unanswered), `TraderArrived`, `TraderLeft`
//!
//! M5.5 (ADR 0031, 0032). Visitors are not ECS entities (M5.5 DECISIONS
//! D16): a wave or a caravan is one group on one tile that moves one tile a
//! step (every 3 ticks: a person's pace) along a nav path. Arriving, it
//! walks to the meeting tile by the hearth ([`super::meeting_tile`]; with none,
//! or no path there, it waits where it is); there it waits
//! (`VisitState::Waiting`): a wave accepted on its way joins on arrival, else
//! it waits for the player's answer — after
//! `wait_days` unanswered it leaves and its neighbour's attitude drops by
//! `decline_attitude` — and a caravan for `caravan_stay_days`, trading, then
//! it leaves (owed items still undelivered are forgiven). Leaving, it walks
//! back to its edge and is gone.
use hecs::World;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::neighbours::{VisitState, WaveAnswer};
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::jobs::items::chebyshev;

/// Priority of [`VisitorSystem`].
pub const VISITOR_PRIORITY: u32 = 175;

/// A group's tile and the rest of its path.
type Walk = ((u32, u32), Vec<(u32, u32)>);

/// One step of a group at `pos` toward `goal` along `path` (replanned when it does not end at `goal`): its new tile
/// and path; `None` when no path leads there (a group that cannot walk stops where it is).
fn step(
    res: &SimResources,
    pos: (u32, u32),
    goal: (u32, u32),
    mut path: Vec<(u32, u32)>,
) -> Option<Walk> {
    if pos == goal {
        return Some((pos, Vec::new()));
    }
    if path.last() != Some(&goal) {
        path = res.nav.path(pos, goal).map(|p| p.to_vec())?;
    }
    if path.is_empty() {
        return None;
    }
    let next = path.remove(0);
    Some(
        if res.nav.can_step(pos, next) || chebyshev(pos, next) == 0 {
            (next, path)
        } else {
            (pos, Vec::new())
        },
    )
}

/// Where a group of neighbour `id` goes when it leaves: its edge tile.
fn way_out(res: &SimResources, neighbour: u8) -> Option<(u32, u32)> {
    let edge = res.colony.neighbours.neighbour(neighbour)?.edge;
    super::edge_tile(res, edge)
}

/// End wave `index`'s wait: it leaves; `reason` 0 the player declined, 1 nobody answered. The neighbour's attitude
/// drops by `decline_attitude` (droppable 7.5); chronicle `MigrantsDeclined`.
pub fn send_away(res: &mut SimResources, index: usize, reason: u8) {
    let Some(rule) = super::rule(res) else {
        return;
    };
    let Some(w) = res.colony.neighbours.waves.get_mut(index) else {
        return;
    };
    w.state = VisitState::Leaving;
    w.path.clear();
    w.answer = Some(if reason == 0 {
        WaveAnswer::Declined
    } else {
        WaveAnswer::Unanswered
    });
    let (from, id, count) = (w.neighbour, w.id, w.people.len() as i64);
    let nb = &mut res.colony.neighbours;
    if reason == 0 {
        nb.tallies.declined += 1;
    } else {
        nb.tallies.unanswered += 1;
    }
    if let Some(n) = nb.list.iter_mut().find(|n| n.id == from) {
        n.attitude = n.attitude.saturating_sub(rule.decline_attitude);
    }
    nb.version += 1;
    res.chronicle.push(
        ChronicleEntry::new(res.current_tick, ChronicleKind::MigrantsDeclined)
            .cause(u32::from(from))
            .value((i64::from(reason) << 24) | (i64::from(id) << 8) | count),
    );
}

/// See the module header.
#[derive(Debug, Default)]
pub struct VisitorSystem;

impl RuntimeSystem for VisitorSystem {
    fn name(&self) -> &str {
        "VisitorSystem"
    }

    fn priority(&self) -> u32 {
        VISITOR_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        3
    }

    /// Off the hour-aligned ticks (M5 DECISIONS D18).
    fn tick_offset(&self, _resources: &SimResources) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = super::rule(res) else {
            return;
        };
        let busy = res
            .colony
            .neighbours
            .waves
            .iter()
            .any(|w| w.state != VisitState::Gone)
            || res
                .colony
                .neighbours
                .caravans
                .iter()
                .any(|c| c.state != VisitState::Gone);
        if !busy {
            return;
        }
        crate::runtime::nav::ensure(res);
        // With no meeting tile (no hearth, or none free near it) an arriving group waits where it is, so waits still
        // time out and caravans still leave (review F7).
        let meet = super::meeting_tile(res);
        let now = res.current_tick;
        let day_ticks = res.rules.calendar.days_to_ticks(1);
        let day = super::today(res);
        // Waves.
        for i in 0..res.colony.neighbours.waves.len() {
            let w = res.colony.neighbours.waves[i].clone();
            match w.state {
                VisitState::Arriving => {
                    // A group with no way to the hearth stops where it is and waits there.
                    let (pos, path, stuck) = match meet.and_then(|m| step(res, w.pos, m, w.path)) {
                        Some((pos, path)) => (pos, path, false),
                        None => (w.pos, Vec::new(), true),
                    };
                    let arrived = stuck || meet.is_some_and(|m| chebyshev(pos, m) <= 1);
                    let wave = &mut res.colony.neighbours.waves[i];
                    wave.pos = pos;
                    wave.path = path;
                    if arrived {
                        wave.state = VisitState::Waiting { since: now };
                        wave.path.clear();
                        let count = wave.people.len() as i64;
                        if let Some(n) = res
                            .colony
                            .neighbours
                            .list
                            .iter_mut()
                            .find(|n| n.id == w.neighbour)
                        {
                            n.last_contact_day = Some(day);
                        }
                        res.chronicle.push(
                            ChronicleEntry::new(now, ChronicleKind::MigrantsArrived)
                                .cause(u32::from(w.neighbour))
                                .value((i64::from(w.id) << 8) | count),
                        );
                        // Accepted on the way (review F3): they join now, at the hearth.
                        if w.answer == Some(WaveAnswer::Accepted)
                            && super::join::accept(world, res, w.id)
                                == Err(super::join::AcceptError::PopulationCap)
                        {
                            res.colony.neighbours.waves[i].answer = None;
                        }
                    }
                    res.colony.neighbours.version += 1;
                }
                VisitState::Waiting { since } => {
                    if now >= since + day_ticks * u64::from(rule.wait_days) {
                        send_away(res, i, 1);
                    }
                }
                VisitState::Leaving => {
                    let Some(out) = way_out(res, w.neighbour) else {
                        res.colony.neighbours.waves[i].state = VisitState::Gone;
                        continue;
                    };
                    let next = step(res, w.pos, out, w.path);
                    let wave = &mut res.colony.neighbours.waves[i];
                    match next {
                        Some((pos, path)) if pos != out => {
                            wave.pos = pos;
                            wave.path = path;
                        }
                        // At the edge, or no way there: gone.
                        _ => wave.state = VisitState::Gone,
                    }
                    res.colony.neighbours.version += 1;
                }
                VisitState::Gone => {}
            }
        }
        // Caravans.
        for i in 0..res.colony.neighbours.caravans.len() {
            let c = res.colony.neighbours.caravans[i].clone();
            match c.state {
                VisitState::Arriving => {
                    let (pos, path, stuck) = match meet.and_then(|m| step(res, c.pos, m, c.path)) {
                        Some((pos, path)) => (pos, path, false),
                        None => (c.pos, Vec::new(), true),
                    };
                    let arrived = stuck || meet.is_some_and(|m| chebyshev(pos, m) <= 1);
                    let spot = meet
                        .filter(|_| arrived)
                        .and_then(|m| super::trade::trade_spot(res, m));
                    let caravan = &mut res.colony.neighbours.caravans[i];
                    caravan.pos = pos;
                    caravan.path = path;
                    if arrived {
                        caravan.state = VisitState::Waiting { since: now };
                        caravan.path.clear();
                        caravan.spot = spot.or(meet).or(Some(pos));
                        if let Some(n) = res
                            .colony
                            .neighbours
                            .list
                            .iter_mut()
                            .find(|n| n.id == c.neighbour)
                        {
                            n.last_contact_day = Some(day);
                        }
                        res.chronicle.push(
                            ChronicleEntry::new(now, ChronicleKind::TraderArrived)
                                .cause(u32::from(c.neighbour))
                                .value(i64::from(c.id)),
                        );
                    }
                    res.colony.neighbours.version += 1;
                }
                VisitState::Waiting { since } => {
                    if now >= since + day_ticks * u64::from(rule.caravan_stay_days) {
                        let caravan = &mut res.colony.neighbours.caravans[i];
                        caravan.state = VisitState::Leaving;
                        caravan.owed.clear();
                        res.colony.neighbours.version += 1;
                        res.chronicle.push(
                            ChronicleEntry::new(now, ChronicleKind::TraderLeft)
                                .cause(u32::from(c.neighbour))
                                .value(i64::from(c.id)),
                        );
                    }
                }
                VisitState::Leaving => {
                    let Some(out) = way_out(res, c.neighbour) else {
                        res.colony.neighbours.caravans[i].state = VisitState::Gone;
                        continue;
                    };
                    let next = step(res, c.pos, out, c.path);
                    let caravan = &mut res.colony.neighbours.caravans[i];
                    match next {
                        Some((pos, path)) if pos != out => {
                            caravan.pos = pos;
                            caravan.path = path;
                        }
                        _ => caravan.state = VisitState::Gone,
                    }
                    res.colony.neighbours.version += 1;
                }
                VisitState::Gone => {}
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata() {
        let s = VisitorSystem;
        assert_eq!(
            (s.name(), s.priority(), s.tick_interval()),
            ("VisitorSystem", 175, 3)
        );
    }
}
