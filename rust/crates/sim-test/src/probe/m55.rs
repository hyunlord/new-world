//! M5.5 measurement (goal §10, F1–F9): flight, wounds, quarrel causes, pressure overlap and population in a colony
//! probe run.
//!
//! Measurement only: reads the engine after each tick, never writes it. The report is the `m55` block of the colony
//! summary:
//! - `flight`: flight episodes (F2c). An episode begins when an undrafted adult outside a roofed room is within
//!   `flee_tiles` of a hunting or biting wolf and was not in the tick before; it **moved away** when, within 30 ticks,
//!   the person stands farther from the wolf's tile at the episode's start than where they stood then (or inside a
//!   roofed room). `episodes`, `moved_away`;
//! - `min_hp`: the lowest hit points any person had while wolves were on the map (100 = untouched; F2b), and
//!   `wolf_deaths` (combat deaths while wolves were on the map);
//! - `quarrels`: each quarrel `{tick, a, b, cause, resolved, reason}` checked by `mood::check_cause` with a 2-day
//!   window (F6), at the end of the run;
//! - `overlap`: minor pressures that began during a major or within 2 days after one ended (F1);
//! - `population`: people at the start and at the end, the most at once and the fewest after day 1 (F1, F7).
use serde_json::{json, Value};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::people::Drafted;
use sim_core::components::{Agent, AgentId, BodyHealth, Child, Position};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

/// Ticks a flight episode has to show movement away (goal §10 F2).
const EPISODE_TICKS: u64 = 30;

/// One open flight episode: start tick, the person's tile then, the wolf's tile then.
type Episode = (u64, (u32, u32), (u32, u32));

/// Per-run observer (see the module header).
#[derive(Debug, Default)]
pub struct M55Watch {
    open: BTreeMap<AgentId, Episode>,
    in_range: BTreeSet<AgentId>,
    episodes: u64,
    moved_away: u64,
    min_hp: Option<f64>,
    wolf_deaths: u64,
    combat_seen: u64,
    start_people: Option<u64>,
    most_people: u64,
    fewest_people: Option<u64>,
}

fn cheb(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

impl M55Watch {
    /// A fresh observer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe the engine after a tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let res = &engine.resources;
        if res.rules.colony.is_none() {
            return;
        }
        let tick = res.current_tick;
        let people = engine.world.query::<&Agent>().iter().count() as u64;
        self.start_people.get_or_insert(people);
        self.most_people = self.most_people.max(people);
        if tick >= 1_440 {
            self.fewest_people = Some(self.fewest_people.map_or(people, |f| f.min(people)));
        }
        let Some(rule) = sim_systems::runtime::wildlife::rule(res) else {
            return;
        };
        let wolves_here = res.colony.wildlife.wolves().next().is_some();
        // Combat deaths while wolves are on the map (a bite killed someone).
        let combat = res.tallies.deaths_combat;
        if wolves_here && combat > self.combat_seen {
            self.wolf_deaths += combat - self.combat_seen;
        }
        self.combat_seen = combat;
        if wolves_here {
            for (_, h) in engine.world.query::<&BodyHealth>().iter() {
                self.min_hp = Some(self.min_hp.map_or(h.hp, |m: f64| m.min(h.hp)));
            }
        }
        let threats = sim_systems::runtime::wildlife::flee::threats(res);
        // Close the open episodes that moved away or ran out of time.
        let pos_of: BTreeMap<AgentId, (u32, u32)> = engine
            .world
            .query::<(&Agent, &Position)>()
            .iter()
            .map(|(_, (a, p))| (a.id, (p.x, p.y)))
            .collect();
        let mut closed = Vec::new();
        for (&id, &(start, from, wolf)) in &self.open {
            let Some(&now) = pos_of.get(&id) else {
                closed.push((id, false));
                continue;
            };
            let away =
                cheb(now, wolf) > cheb(from, wolf) || sim_systems::runtime::rooms::roofed(res, now);
            if away {
                closed.push((id, true));
            } else if tick.saturating_sub(start) >= EPISODE_TICKS {
                closed.push((id, false));
            }
        }
        for (id, ok) in closed {
            self.open.remove(&id);
            if ok {
                self.moved_away += 1;
            }
        }
        // New episodes: undrafted adults outside roofed rooms within reach of a threat.
        let mut now_in: BTreeSet<AgentId> = BTreeSet::new();
        if !threats.is_empty() {
            for (_, (a, p, drafted)) in engine
                .world
                .query::<(&Agent, &Position, Option<&Drafted>)>()
                .without::<&Child>()
                .iter()
            {
                let at = (p.x, p.y);
                if drafted.is_some() || sim_systems::runtime::rooms::roofed(res, at) {
                    continue;
                }
                let Some(&(_, wolf)) = threats
                    .iter()
                    .filter(|(_, w)| cheb(*w, at) <= rule.flee_tiles)
                    .min_by_key(|(id, w)| (cheb(*w, at), *id))
                else {
                    continue;
                };
                now_in.insert(a.id);
                if !self.in_range.contains(&a.id) && !self.open.contains_key(&a.id) {
                    self.open.insert(a.id, (tick, at, wolf));
                    self.episodes += 1;
                }
            }
        }
        self.in_range = now_in;
    }

    /// The `m55` block.
    pub fn report(&self, engine: &SimEngine) -> Value {
        let res = &engine.resources;
        let chronicle = &res.chronicle;
        let window = res.rules.calendar.days_to_ticks(2);
        let mut resolved = 0u64;
        let quarrels: Vec<Value> = (0..chronicle.len() as u32)
            .filter_map(|i| chronicle.entry(i).map(|e| (i, *e)))
            .filter(|(_, e)| e.kind == ChronicleKind::Quarrel && e.cause >= 1)
            .map(|(i, e)| {
                let check = sim_systems::runtime::mood::check_cause(chronicle, i, window);
                if check.is_ok() {
                    resolved += 1;
                }
                json!({"tick": e.tick, "a": e.subject, "b": e.other, "cause": e.cause - 1,
                    "resolved": check.is_ok(), "reason": check.err()})
            })
            .collect();
        // Minor pressures begun during a major or within 2 days after one ended.
        let begun: Vec<_> = res.colony.story.begun().copied().collect();
        let overlap: Vec<Value> = begun
            .iter()
            .filter(|p| !p.kind.is_major())
            .filter(|p| {
                begun.iter().any(|m| {
                    m.kind.is_major()
                        && p.start_day >= m.start_day
                        && p.start_day < m.end_day.saturating_add(2)
                })
            })
            .map(|p| json!({"kind": p.kind.name(), "start_day": p.start_day}))
            .collect();
        let people = engine.world.query::<&Agent>().iter().count() as u64;
        json!({
            "flight": {"episodes": self.episodes, "moved_away": self.moved_away,
                "open_at_end": self.open.len()},
            "min_hp": self.min_hp,
            "wolf_deaths": self.wolf_deaths,
            "quarrels": quarrels,
            "quarrels_resolved": resolved,
            "overlap": overlap,
            "population": {"start": self.start_people, "end": people, "most": self.most_people,
                "fewest_after_day1": self.fewest_people},
        })
    }
}
