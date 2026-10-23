//! M5 measurement (goal §11, F1–F10): pressure, animals, apparel, mood and quarrels in a colony probe run.
//!
//! Measurement only: reads the engine after each tick, never writes it. The
//! report is the `m5` block of the colony summary:
//! - `pressures`: every begun pressure `{kind, start_day, end_day, intensity, deaths, items_lost,
//!   work_ticks_lost}` and `skipped` (F1);
//! - `snaps`: per cold snap the lowest hourly firewood the hearth could use (settlement wood +
//!   feedable firewood) and one day's need at the snap's burn (F2);
//! - `deaths`: deaths by cause at the end; `cold_deaths` (exposure, F2/F9);
//! - `beasts`: per pack `{day, wolves, killed, left, bitten, deaths}` while its wolves are on the map or its Beasts
//!   pressure is active (F3; `bitten` counts a person once a pack, so summed over packs it is person-packs);
//! - `wildlife`: the wildlife tallies (F4, F8);
//! - `apparel`: wearers of a hide wrap at the end of day 30 and at the end; winter-night cold
//!   person-hours and person-hours by group (clothed: any top; unclothed) (F5);
//! - `mood`: breakdowns, quarrels, fights, and each quarrel with its "why?" chain length (F6, F7);
//! - `daily`: one row per day end `{day, knives_broken, blades: [count, quality sum], top_knap,
//!   wraps, mood_min, mood_mean, low}` (F8).
use serde_json::{json, Value};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::people::{Apparel, Mood, Top};
use sim_core::colony::{ItemKind, RecipeKind, SkillKind, Skills};
use sim_core::components::{Agent, Child, Cold};
use sim_engine::SimEngine;
use std::collections::BTreeMap;

const DAY: u64 = 1_440;

/// Per-run observer (see the module header).
#[derive(Debug, Default)]
pub struct M5Watch {
    snaps: BTreeMap<u32, (u64, u64)>,
    packs: BTreeMap<u32, Value>,
    daily: Vec<Value>,
    wraps_day30: Option<u64>,
    cold_hours: [u64; 2],
    exposure_hours: [u64; 2],
    bitten_at_pack: BTreeMap<u32, (u64, u64, u64, u64)>,
}

fn blades(engine: &SimEngine) -> (u64, u64) {
    engine
        .resources
        .colony
        .tallies
        .craft_quality
        .iter()
        .filter(|((_, r), _)| *r == RecipeKind::KnapBlade.code())
        .fold((0, 0), |(n, s), (_, (c, q))| (n + c, s + q))
}

fn wraps(engine: &SimEngine) -> u64 {
    engine
        .world
        .query::<&Apparel>()
        .iter()
        .filter(|(_, a)| a.top == Top::HideWrap)
        .count() as u64
}

impl M5Watch {
    /// A fresh observer.
    pub fn new() -> Self {
        Self::default()
    }

    /// Observe the engine after a tick.
    pub fn after_tick(&mut self, engine: &SimEngine) {
        let res = &engine.resources;
        if sim_systems::runtime::story::rule(res).is_none()
            && sim_systems::runtime::mood::rule(res).is_none()
        {
            return;
        }
        let tick = res.current_tick;
        let story = &res.colony.story;
        if tick.is_multiple_of(60) {
            if let Some(p) = story.active_of(sim_core::colony::story::PressureKind::ColdSnap) {
                let have = sim_systems::runtime::story::firewood_have(&engine.resources);
                let need = sim_systems::runtime::story::firewood_day_need(&engine.resources);
                let e = self.snaps.entry(p.start_day).or_insert((u64::MAX, need));
                e.0 = e.0.min(have);
                e.1 = e.1.max(need);
            }
            // Winter-night cold hours by group (F5).
            let minute = (tick + res.rules.clock_offset_ticks) % DAY;
            let winter =
                sim_systems::runtime::resource_regen::season_at(res).is_some_and(|(s, _)| s == 3);
            let night = !(360..1200).contains(&minute);
            if winter && night {
                for (_, (_, apparel, cold)) in engine
                    .world
                    .query::<(&Agent, Option<&Apparel>, Option<&Cold>)>()
                    .without::<&Child>()
                    .iter()
                {
                    let g = usize::from(apparel.is_some());
                    self.exposure_hours[g] += 1;
                    if cold.is_some() {
                        self.cold_hours[g] += 1;
                    }
                }
            }
        }
        // Packs: their wolves and what they did.
        let w = &res.colony.wildlife;
        // A pack's window: from its arrival while its wolves are on the map or its pressure is active
        // (`pack_day` stays set after the pack has gone; M5 numbers review).
        let beasts_day = story
            .active_of(sim_core::colony::story::PressureKind::Beasts)
            .map(|p| p.start_day);
        if let Some(day) = w.pack_day.filter(|d| {
            // ... and once more after its last wolf has gone, so the row ends with the pack's final state.
            beasts_day == Some(*d)
                || w.wolves().next().is_some()
                || self
                    .packs
                    .get(d)
                    .is_some_and(|v| v["alive"].as_u64() != Some(0))
        }) {
            let alive = w.wolves().count() as u64;
            let entry = self.bitten_at_pack.entry(day).or_insert((
                w.tallies.bitten,
                w.tallies.wolves_killed,
                w.tallies.wolves_left,
                sim_systems::runtime::story::deaths_so_far(res),
            ));
            let (b0, k0, l0, d0) = *entry;
            self.packs.insert(
                day,
                json!({
                    "day": day, "wolves": w.pack_size, "alive": alive,
                    "killed": w.tallies.wolves_killed - k0, "left": w.tallies.wolves_left - l0,
                    "bitten": w.tallies.bitten - b0,
                    "deaths": sim_systems::runtime::story::deaths_so_far(res) - d0,
                    "drafted": engine.world.query::<&sim_core::colony::people::Drafted>().iter().count(),
                }),
            );
        }
        if (tick + 1).is_multiple_of(DAY) {
            let day = (tick + 1) / DAY;
            let moods: Vec<i64> = engine
                .world
                .query::<&Mood>()
                .iter()
                .map(|(_, m)| i64::from(m.value))
                .collect();
            let top_knap = engine
                .world
                .query::<&Skills>()
                .iter()
                .map(|(_, s)| s.level(SkillKind::Knap))
                .max()
                .unwrap_or(0);
            let wr = wraps(engine);
            if day == 30 {
                self.wraps_day30 = Some(wr);
            }
            let low = sim_systems::runtime::mood::rule(res).map_or(250, |r| r.low);
            self.daily.push(json!({
                "day": day,
                "knives_broken": w.tallies.knives_broken,
                "knife_uses": w.tallies.knife_uses,
                "blades": blades(engine),
                "top_knap": top_knap,
                "wraps": wr,
                "mood_min": moods.iter().min(),
                "mood_mean": if moods.is_empty() { 0 } else { moods.iter().sum::<i64>() / moods.len() as i64 },
                "low": moods.iter().filter(|v| **v < i64::from(low)).count(),
                "deer": w.deer().count(),
                "deer_killed": w.tallies.deer_killed,
                "meat_eaten": w.tallies.meat_eaten,
            }));
        }
    }

    /// The `m5` block.
    pub fn report(&self, engine: &SimEngine) -> Value {
        let res = &engine.resources;
        let story = &res.colony.story;
        let mut pressures: Vec<Value> = story
            .begun()
            .map(|p| {
                json!({"kind": p.kind.name(), "start_day": p.start_day, "end_day": p.end_day, "intensity": p.intensity,
                    "deaths": p.cost.deaths, "items_lost": p.cost.items_lost, "work_ticks_lost": p.cost.work_ticks_lost,
                    "ended": story.history.iter().any(|h| h.start_day == p.start_day && h.kind == p.kind)})
            })
            .collect();
        pressures.sort_by_key(|p| {
            (
                p["start_day"].as_u64(),
                p["kind"].as_str().map(str::to_string),
            )
        });
        let chronicle = &res.chronicle;
        let quarrels: Vec<Value> = (0..chronicle.len() as u32)
            .filter_map(|i| chronicle.entry(i).map(|e| (i, *e)))
            .filter(|(_, e)| e.kind == ChronicleKind::Quarrel && e.cause >= 1)
            .map(|(i, e)| {
                let chain = sim_systems::runtime::mood::why_chain(chronicle, i);
                json!({"tick": e.tick, "a": e.subject, "b": e.other, "cause": e.cause - 1, "relation": e.value,
                    "chain": chain.len(), "resolvable": chain.len() >= 2})
            })
            .collect();
        let t = &res.tallies;
        let m = res.colony.mood_tallies;
        let w = &res.colony.wildlife;
        json!({
            "pressures": pressures,
            "skipped": story.skipped,
            "snaps": self.snaps.iter().map(|(d, (low, need))| json!({"day": d, "min_firewood": low, "day_need": need})).collect::<Vec<_>>(),
            "deaths": {"starvation": t.deaths_starvation, "dehydration": t.deaths_dehydration, "combat": t.deaths_combat,
                "old_age": t.deaths_old_age, "exposure": t.deaths_exposure},
            "cold_deaths": t.deaths_exposure,
            "beasts": self.packs.values().cloned().collect::<Vec<_>>(),
            "wildlife": serde_json::to_value(&w.tallies).unwrap_or(Value::Null),
            "apparel": {"wraps_day30": self.wraps_day30, "wraps_end": wraps(engine),
                "cold_hours": self.cold_hours, "exposure_hours": self.exposure_hours},
            "mood": {"breakdowns": m.breakdowns, "quarrels": m.quarrels, "fights": m.fights,
                "quarrel_list": quarrels},
            "ledger_meat": {
                "created": res.colony.item_ledger.created_of(ItemKind::Meat),
                "consumed": res.colony.item_ledger.consumed_of(ItemKind::Meat),
            },
            "daily": self.daily,
        })
    }
}
