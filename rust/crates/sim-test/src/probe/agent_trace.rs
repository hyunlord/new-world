//! Pass 16 per-person trace (`worldsim_probe --agent-trace`, off by default).
//!
//! An observer of components and state transitions after every tick — it
//! changes no system and draws nothing. For every person it keeps the
//! temperament (the component when present; otherwise, e.g. in `game_v19`,
//! the value `Temperament::founder` / `Temperament::child` *would* give,
//! computed here from the lineage key and the `AgentBorn` parents, never
//! inserted) and, over **non-elder adult** time only (no `Child`, no
//! `Elder`), these counters:
//!
//! - `idle_dist_sum` / `idle_samples`: Chebyshev distance from the person's
//!   settlement hearth (`formation_tile` of the first settlement in id order
//!   that lists it), sampled at system ticks divisible by 10 while Idle with
//!   the clock in 06:00–18:00;
//! - `bed_minutes`: per night, the clock minute of the first system tick
//!   between 18:00 and 03:00 at which the person is `Consuming{Sleep}`
//!   (18:00–23:59 as 1,080…1,439, after midnight as 1,440 + m);
//! - `social_completed`: transitions out of `Consuming{Agent}` with a
//!   loneliness drop ≥ min(25, loneliness before the tick);
//! - `deposit_food_*` / `deposit_wood_*`: at a tick where the carried amount
//!   drops while the person stands within `SETTLEMENT_PROXIMITY_RADIUS` of its
//!   settlement's hearth, the amount carried on the tick before (a drop
//!   elsewhere is not a deposit);
//! - `departed`: whether the person left in a completed fission.
//!
//! The clock minute is that of the **system tick** — the tick the systems ran
//! at, `resources.current_tick` after `engine.tick()` — plus
//! `clock_offset_ticks`, modulo the day.
use serde_json::{json, Value};
use sim_core::causal::CausalEvent;
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Elder, Inventory, Lineage, Position, ResourceKind, Social, TargetKind,
    Temperament, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};

/// Probe options outside `ProbeArgs` (Pass 16): flags this pass adds without
/// a new field in an existing public struct.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProbeOptions {
    /// Write the per-person files `<scene>_s<seed>_agents.jsonl`,
    /// `_people.jsonl`, `_fissions.jsonl` and `_trace_meta.json`.
    pub agent_trace: bool,
}

impl ProbeOptions {
    /// Removes this pass's flags (`--agent-trace`) from `args` and returns the
    /// rest for `ProbeArgs::parse` with the options found.
    pub fn split(args: impl IntoIterator<Item = String>) -> (Vec<String>, Self) {
        let mut options = Self::default();
        let rest = args
            .into_iter()
            .filter(|arg| {
                if arg == "--agent-trace" {
                    options.agent_trace = true;
                    false
                } else {
                    true
                }
            })
            .collect();
        (rest, options)
    }
}

/// One person's record.
#[derive(Clone, Debug, Default)]
struct Person {
    key: u64,
    born_tick: Option<u64>,
    temperament: Option<Temperament>,
    from_component: bool,
    parents: Vec<AgentId>,
    parent_temperaments: [Option<Temperament>; 2],
    born_event_found: bool,
    adult_ticks: u64,
    idle_dist_sum: u64,
    idle_samples: u64,
    bed_minutes: BTreeMap<u64, u32>,
    social_completed: u64,
    deposit_food: (u64, u64),
    deposit_wood: (u64, u64),
    alive: bool,
}

/// The previous tick's observed values of one person.
#[derive(Clone, Copy, Debug)]
struct Prev {
    state: AgentState,
    loneliness: f64,
    food: u32,
    wood: u32,
}

/// One fission seen in `fission_log`, with the eligible members at its tick.
#[derive(Clone, Debug)]
struct FissionSeen {
    index: usize,
    eligible: BTreeSet<AgentId>,
}

/// The per-person observer of one run.
#[derive(Debug, Default)]
pub(crate) struct AgentTracer {
    seed: u64,
    people: BTreeMap<AgentId, Person>,
    prev: BTreeMap<AgentId, Prev>,
    fissions: Vec<FissionSeen>,
    /// Newborns whose `AgentBorn` event was not in their tile's ring.
    born_event_misses: u64,
    /// People whose computed temperament differs from their component.
    component_mismatches: u64,
    routine: super::pass17::RoutineTracer,
}

/// The clock minute of the engine's last system tick.
fn clock_minute(engine: &SimEngine) -> Option<u64> {
    let r = &engine.resources;
    (r.ticks_per_day != 0).then(|| r.current_tick.saturating_add(r.rules.clock_offset_ticks) % r.ticks_per_day)
}

/// The night a clock reading belongs to and its bed minute: 18:00–23:59 →
/// (day, minute), 00:00–02:59 → (day − 1, 1,440 + minute); else None.
pub(crate) fn night_of(abs_tick: u64, ticks_per_day: u64) -> Option<(u64, u32)> {
    if ticks_per_day != 1440 {
        return None;
    }
    let (day, minute) = (abs_tick / 1440, (abs_tick % 1440) as u32);
    if minute >= 1080 {
        Some((day, minute))
    } else if minute < 180 {
        day.checked_sub(1).map(|d| (d, 1440 + minute))
    } else {
        None
    }
}

impl AgentTracer {
    /// Starts the trace on a freshly built engine (tick 0): every person
    /// present is a founder.
    pub(crate) fn start(engine: &SimEngine) -> Self {
        let mut tracer = Self { seed: engine.resources.world_seed, routine: super::pass17::RoutineTracer::start(engine), ..Self::default() };
        for (_, (agent, lineage, temperament)) in
            engine.world.query::<(&Agent, Option<&Lineage>, Option<&Temperament>)>().iter()
        {
            let key = lineage.map_or(agent.id, |l| l.key);
            let computed = lineage.map(|l| Temperament::founder(tracer.seed, l.key));
            let person = tracer.person(key, None, computed, temperament, Vec::new(), [None; 2], true);
            tracer.people.insert(agent.id, person);
        }
        tracer.record_prev(engine);
        tracer
    }

    #[allow(clippy::too_many_arguments)]
    fn person(
        &mut self,
        key: u64,
        born_tick: Option<u64>,
        computed: Option<Temperament>,
        component: Option<&Temperament>,
        parents: Vec<AgentId>,
        parent_temperaments: [Option<Temperament>; 2],
        born_event_found: bool,
    ) -> Person {
        if let (Some(c), Some(t)) = (computed, component) {
            if c != *t {
                self.component_mismatches += 1;
            }
        }
        Person {
            key,
            born_tick,
            temperament: component.copied().or(computed),
            from_component: component.is_some(),
            parents,
            parent_temperaments,
            born_event_found,
            alive: true,
            ..Person::default()
        }
    }

    fn record_prev(&mut self, engine: &SimEngine) {
        self.prev.clear();
        for (_, (agent, state, social, inventory)) in
            engine.world.query::<(&Agent, &AgentState, Option<&Social>, Option<&Inventory>)>().iter()
        {
            self.prev.insert(
                agent.id,
                Prev {
                    state: *state,
                    loneliness: social.map_or(0.0, |s| s.loneliness),
                    food: inventory.map_or(0, |i| i.get(ResourceKind::Food)),
                    wood: inventory.map_or(0, |i| i.get(ResourceKind::Wood)),
                },
            );
        }
    }

    /// Observes the engine right after one `engine.tick()`.
    pub(crate) fn after_tick(&mut self, engine: &SimEngine) {
        let r = &engine.resources;
        let sys_tick = r.current_tick;
        let minute = clock_minute(engine);
        let night = night_of(sys_tick.saturating_add(r.rules.clock_offset_ticks), r.ticks_per_day);
        let width = r.tile_grid.width;
        // Hearths: the first settlement in id order that lists the person.
        let mut hearth: BTreeMap<AgentId, (u32, u32)> = BTreeMap::new();
        let mut ids: Vec<_> = r.settlements.keys().copied().collect();
        ids.sort_unstable();
        for sid in &ids {
            if let Some(s) = r.settlements.get(sid) {
                for id in &s.member_agents {
                    hearth.entry(*id).or_insert(s.formation_tile);
                }
            }
        }
        let alive_before: BTreeSet<AgentId> = self.prev.keys().copied().collect();
        let mut seen: BTreeSet<AgentId> = BTreeSet::new();
        let mut elders: BTreeSet<AgentId> = BTreeSet::new();
        type Row<'a> = (
            &'a Agent,
            &'a Position,
            &'a AgentState,
            Option<&'a Social>,
            Option<&'a Inventory>,
            Option<&'a Child>,
            Option<&'a Elder>,
            Option<&'a Lineage>,
            Option<&'a Temperament>,
        );
        let mut query = engine.world.query::<Row>();
        let mut rows: Vec<_> = query.iter().map(|(_, row)| row).collect();
        rows.sort_by_key(|row| row.0.id);
        for (agent, pos, state, social, inventory, child, elder, lineage, temperament) in rows {
            let id = agent.id;
            seen.insert(id);
            if elder.is_some() {
                elders.insert(id);
            }
            if !self.people.contains_key(&id) {
                // A newborn: its parents from the AgentBorn event at its tile.
                let key = lineage.map_or(id, |l| l.key);
                let parents = r
                    .causal_log
                    .get(pos.y * width + pos.x)
                    .and_then(|log| {
                        log.as_slice().iter().find_map(|event| match event {
                            CausalEvent::AgentBorn { agent: born, parent_ids, .. } if *born == id => Some(parent_ids.clone()),
                            _ => None,
                        })
                    });
                let found = parents.is_some();
                if !found {
                    self.born_event_misses += 1;
                }
                let parents = parents.unwrap_or_default();
                let mut parent_temperaments = [None; 2];
                for (slot, pid) in parents.iter().take(2).enumerate() {
                    if alive_before.contains(pid) {
                        parent_temperaments[slot] = self.people.get(pid).and_then(|p| p.temperament);
                    }
                }
                let computed = lineage.map(|l| Temperament::child(self.seed, l.key, parent_temperaments));
                let person = self.person(key, Some(sys_tick), computed, temperament, parents, parent_temperaments, found);
                self.people.insert(id, person);
            }
            let adult = child.is_none() && elder.is_none();
            let prev = self.prev.get(&id).copied();
            let Some(person) = self.people.get_mut(&id) else { continue };
            if !adult {
                continue;
            }
            person.adult_ticks += 1;
            let home = hearth.get(&id).copied();
            if let (Some(home), Some(m)) = (home, minute) {
                if sys_tick.is_multiple_of(10) && (360..1080).contains(&m) && matches!(state, AgentState::Idle) {
                    person.idle_dist_sum += u64::from(pos.x.abs_diff(home.0).max(pos.y.abs_diff(home.1)));
                    person.idle_samples += 1;
                }
            }
            if let (Some((n, bed)), AgentState::Consuming { target: TargetKind::Sleep }) = (night, state) {
                person.bed_minutes.entry(n).or_insert(bed);
            }
            if let Some(prev) = prev {
                let lonely = social.map_or(0.0, |s| s.loneliness);
                let was_social = matches!(prev.state, AgentState::Consuming { target: TargetKind::Agent(_) });
                let is_social = matches!(state, AgentState::Consuming { target: TargetKind::Agent(_) });
                if was_social && !is_social && prev.loneliness - lonely >= prev.loneliness.min(25.0) {
                    person.social_completed += 1;
                }
                let near = home.is_some_and(|h| pos.x.abs_diff(h.0).max(pos.y.abs_diff(h.1)) <= SETTLEMENT_PROXIMITY_RADIUS);
                let (food, wood) = (
                    inventory.map_or(0, |i| i.get(ResourceKind::Food)),
                    inventory.map_or(0, |i| i.get(ResourceKind::Wood)),
                );
                if near && food < prev.food {
                    person.deposit_food.0 += u64::from(prev.food);
                    person.deposit_food.1 += 1;
                }
                if near && wood < prev.wood {
                    person.deposit_wood.0 += u64::from(prev.wood);
                    person.deposit_wood.1 += 1;
                }
            }
        }
        for (id, person) in &mut self.people {
            if person.alive && !seen.contains(id) {
                person.alive = false;
            }
        }
        // New fission records: the eligible (non-elder) members of the parent
        // at the departure tick (the departers included).
        for index in self.fissions.len()..r.fission_log.len() {
            let record = &r.fission_log[index];
            let mut eligible: BTreeSet<AgentId> = r
                .settlements
                .get(&record.parent)
                .map(|s| s.member_agents.iter().copied().filter(|id| seen.contains(id) && !elders.contains(id)).collect())
                .unwrap_or_default();
            eligible.extend(record.members.iter().copied());
            self.fissions.push(FissionSeen { index, eligible });
        }
        self.record_prev(engine);
        self.routine.after_tick(engine);
    }

    /// The output lines: (agents, people, fissions, meta).
    pub(crate) fn finish(&self, engine: &SimEngine) -> (Vec<Value>, Vec<Value>, Vec<Value>, Value) {
        let r = &engine.resources;
        let tpd = r.ticks_per_day.max(1) as f64;
        let mut departed: BTreeSet<AgentId> = BTreeSet::new();
        let mut fissions = Vec::new();
        let (mut completed, mut cancelled, mut pending) = (0u64, 0u64, 0u64);
        for seen in &self.fissions {
            let record = &r.fission_log[seen.index];
            let status = if record.cancelled_tick.is_some() {
                cancelled += 1;
                "cancelled"
            } else if record.founded_tick.is_some() {
                completed += 1;
                departed.extend(record.members.iter().copied());
                "completed"
            } else {
                pending += 1;
                "pending"
            };
            fissions.push(json!({
                "fission_id": record.fission_id,
                "parent": record.parent,
                "depart_tick": record.depart_tick,
                "status": status,
                "members": record.members,
                "eligible": seen.eligible.iter().collect::<Vec<_>>(),
            }));
        }
        let axes = |t: &Option<Temperament>| t.map(|t| json!({"ns": t.ns, "ha": t.ha, "rd": t.rd, "p": t.p}));
        let mut agents = Vec::new();
        let mut people = Vec::new();
        for (id, p) in &self.people {
            let temperament = axes(&p.temperament);
            let source = if p.from_component { "component" } else { "computed" };
            people.push(json!({
                "id": id,
                "lineage_key": p.key,
                "founder": p.born_tick.is_none(),
                "born_tick": p.born_tick,
                "temperament": temperament,
                "temperament_source": source,
                "parents": p.parents,
                "born_event_found": p.born_event_found,
                "parent_temperaments": [axes(&p.parent_temperaments[0]), axes(&p.parent_temperaments[1])],
                "alive_end": p.alive,
                "ever_adult": p.adult_ticks > 0,
                "routine": self.routine.person(*id, false),
            }));
            if p.adult_ticks == 0 {
                continue;
            }
            agents.push(json!({
                "id": id,
                "lineage_key": p.key,
                "founder": p.born_tick.is_none(),
                "born_tick": p.born_tick,
                "temperament": temperament,
                "temperament_source": source,
                "adult_days": p.adult_ticks as f64 / tpd,
                "adult_ticks": p.adult_ticks,
                "routine": self.routine.person(*id, true),
                "idle_dist_sum": p.idle_dist_sum,
                "idle_samples": p.idle_samples,
                "bed_minutes": p.bed_minutes.values().collect::<Vec<_>>(),
                "social_completed": p.social_completed,
                "deposit_food_sum": p.deposit_food.0,
                "deposit_food_count": p.deposit_food.1,
                "deposit_wood_sum": p.deposit_wood.0,
                "deposit_wood_count": p.deposit_wood.1,
                "departed": departed.contains(id),
            }));
        }
        let meta = json!({
            "people": self.people.len(),
            "traced_adults": agents.len(),
            "born_event_misses": self.born_event_misses,
            "component_mismatches": self.component_mismatches,
            "fissions_completed": completed,
            "fissions_cancelled": cancelled,
            "fissions_pending": pending,
            "end_tick": engine.current_tick(),
            "pass17": self.routine.meta(engine),
        });
        (agents, people, fissions, meta)
    }

    pub(crate) fn routine_output(&self) -> [(&'static str, Vec<Value>); 2] {
        self.routine.output()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{Settlement, Sleep};
    use sim_core::material::MaterialRegistry;

    #[test]
    fn options_split_removes_only_the_trace_flag() {
        let (rest, options) = ProbeOptions::split(["--scene", "game_v20", "--agent-trace", "--days", "3"].map(String::from));
        assert!(options.agent_trace);
        assert_eq!(rest, ["--scene", "game_v20", "--days", "3"]);
        let (rest, options) = ProbeOptions::split(["--days", "3"].map(String::from));
        assert!(!options.agent_trace);
        assert_eq!(rest, ["--days", "3"]);
    }

    #[test]
    fn nights_run_from_six_pm_to_three_am() {
        assert_eq!(night_of(1440 * 2 + 1080, 1440), Some((2, 1080)));
        assert_eq!(night_of(1440 * 2 + 1439, 1440), Some((2, 1439)));
        assert_eq!(night_of(1440 * 3 + 10, 1440), Some((2, 1450)));
        assert_eq!(night_of(1440 * 3 + 179, 1440), Some((2, 1619)));
        assert_eq!(night_of(1440 * 3 + 180, 1440), None);
        assert_eq!(night_of(1440 * 3 + 1079, 1440), None);
        assert_eq!(night_of(10, 1440), None);
    }

    /// A scripted engine (no systems registered: `tick()` only advances the
    /// clock) with one settlement at (10, 10) and one adult founder, whose
    /// components the test sets between ticks; each counter checked by hand.
    #[test]
    fn scripted_adult_counters() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        e.resources.world_seed = 5;
        // Clock offset so the first system tick (0) reads 05:59.
        e.resources.rules.clock_offset_ticks = 359;
        let a = e.spawn_agent(12, 10);
        e.world
            .insert(a, (AgentState::Idle, Social::new(40.0, 0.0), Inventory::default(), Sleep::new(0.0, 0.0), Lineage { key: 77, births: 0 }))
            .unwrap();
        let id = e.world.get::<&Agent>(a).unwrap().id;
        let mut s = Settlement::new_with_id(1, 0);
        s.formation_tile = (10, 10);
        s.member_agents.insert(id);
        e.resources.settlements.insert(1, s);
        let mut t = AgentTracer::start(&e);
        // System tick 0 (05:59): Idle but before 06:00 — no idle sample.
        e.tick();
        t.after_tick(&e);
        // System tick 1 (06:00): not divisible by 10 — no sample.
        e.tick();
        t.after_tick(&e);
        // Ticks 2..=10: at system tick 10 (06:09) Idle at distance 2 → one sample.
        for _ in 2..=10 {
            e.tick();
            t.after_tick(&e);
        }
        // Tick 11: walk to (16, 13) carrying 5 food and 2 wood (far: no deposit).
        {
            let mut p = e.world.get::<&mut Position>(a).unwrap();
            (p.x, p.y) = (16, 13);
            let mut inv = e.world.get::<&mut Inventory>(a).unwrap();
            inv.add(ResourceKind::Food, 5);
            inv.add(ResourceKind::Wood, 2);
        }
        e.tick();
        t.after_tick(&e);
        // Tick 12: eats 1 food far from home — a drop elsewhere, not a deposit.
        e.world.get::<&mut Inventory>(a).unwrap().remove(ResourceKind::Food, 1);
        e.tick();
        t.after_tick(&e);
        // Tick 13: home (11, 11), within the radius: food 4 → 0 and wood 2 → 0 deposited.
        {
            let mut p = e.world.get::<&mut Position>(a).unwrap();
            (p.x, p.y) = (11, 11);
            let mut inv = e.world.get::<&mut Inventory>(a).unwrap();
            inv.remove(ResourceKind::Food, 4);
            inv.remove(ResourceKind::Wood, 2);
        }
        e.tick();
        t.after_tick(&e);
        // Tick 14: social starts (Consuming{Agent}); tick 15 ends with a drop 40 → 12 (28 ≥ 25).
        *e.world.get::<&mut AgentState>(a).unwrap() = AgentState::Consuming { target: TargetKind::Agent(99) };
        e.tick();
        t.after_tick(&e);
        *e.world.get::<&mut AgentState>(a).unwrap() = AgentState::Idle;
        e.world.get::<&mut Social>(a).unwrap().loneliness = 12.0;
        e.tick();
        t.after_tick(&e);
        // Tick 16–17: a combat-like Consuming{Agent} that ends with no drop — not counted.
        *e.world.get::<&mut AgentState>(a).unwrap() = AgentState::Consuming { target: TargetKind::Agent(98) };
        e.tick();
        t.after_tick(&e);
        *e.world.get::<&mut AgentState>(a).unwrap() = AgentState::Idle;
        e.tick();
        t.after_tick(&e);
        // Run to system tick 860 (20:19) Idle at (11, 11): idle samples every 10
        // ticks while the clock is before 18:00 (system tick 721 = 18:00).
        while e.resources.current_tick < 860 {
            e.tick();
            t.after_tick(&e);
        }
        // System tick 861 (20:20): asleep — the night's first sleeping minute is 1220.
        *e.world.get::<&mut AgentState>(a).unwrap() = AgentState::Consuming { target: TargetKind::Sleep };
        for _ in 0..5 {
            e.tick();
            t.after_tick(&e);
        }
        let (agents, people, fissions, meta) = t.finish(&e);
        assert_eq!(people.len(), 1);
        assert!(fissions.is_empty());
        assert_eq!(meta["born_event_misses"], 0);
        let row = &agents[0];
        assert_eq!(row["lineage_key"], 77);
        assert_eq!(row["temperament"]["ns"], Temperament::founder(5, 77).ns);
        assert_eq!(row["temperament_source"], "computed");
        // Idle samples: system ticks 10, 20, …, 720 that were Idle: tick 10 at
        // distance 2; ticks 20..=720 at (11, 11), distance 1 (71 samples).
        assert_eq!(row["idle_samples"], 1 + 71);
        assert_eq!(row["idle_dist_sum"], 2 + 71);
        assert_eq!(row["deposit_food_count"], 1);
        assert_eq!(row["deposit_food_sum"], 4);
        assert_eq!(row["deposit_wood_count"], 1);
        assert_eq!(row["deposit_wood_sum"], 2);
        assert_eq!(row["social_completed"], 1);
        assert_eq!(row["bed_minutes"], json!([1220]));
        assert_eq!(row["departed"], false);
        assert!((row["adult_days"].as_f64().unwrap() - 866.0 / 1440.0).abs() < 1e-12);
    }

    #[test]
    fn children_and_elders_are_not_counted() {
        let mut e = SimEngine::new(32, 32, MaterialRegistry::new());
        let a = e.spawn_agent(5, 5);
        e.world.insert(a, (AgentState::Idle, Child { born_tick: 0 })).unwrap();
        let mut t = AgentTracer::start(&e);
        for _ in 0..20 {
            e.tick();
            t.after_tick(&e);
        }
        let (agents, people, _, _) = t.finish(&e);
        assert!(agents.is_empty());
        assert_eq!(people.len(), 1);
        assert_eq!(people[0]["ever_adult"], false);
    }
}
