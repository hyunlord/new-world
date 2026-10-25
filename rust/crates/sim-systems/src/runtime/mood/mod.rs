//! MoodSystem: moves each person's mood toward the target its modifiers set, notes the night's sleep and cold, and starts breakdowns.
//!
//! Priority: 172 (`MOOD_PRIORITY`). Interval: 60 ticks, offset 7. Gate: `ColonyRule::mood`.
//! Reads: `Agent`, `Child`, `Life`, `AgentState`, `Position`, `Cold`, `Hunger`, `Thirst`, `Sick`, `Drafted`, `Temperament`; `SimResources.colony` (rooms, story), `rules` (mood, calendar, clock offset)
//! Writes: `Mood` (inserted for adults and children of 10 years or more; value, nights, breakdown), `Job` (cancelled at a breakdown); `SimResources.colony` (reservations), `chronicle`
//! Events: chronicle `Breakdown` (the three largest negative modifiers; cause table → the largest)
//!
//! M5 (ADR 0030). Each interval, in `AgentId` order: during the night window
//! (the colony's sleep hours, 20:00–06:00) a sleeper in a roofed bedroom, a
//! sleeper outside a roofed room and a cold person are noted for tonight; in
//! the 8 o'clock hour tonight becomes last night (the system runs at minute 7). The target is `base` + Σ modifiers
//! ([`modifiers`]; harm avoidance scales the negative ones ×(800 + ha/5) ‰,
//! reward dependence the social ones), clamped to 0–1,000, and the value moves
//! 1/`step_divisor` of the gap toward it (at least 1 while there is a gap). A
//! value below `breakdown` starts a breakdown: the person's job is cancelled
//! and the job layer gives them none through the next day; chronicle `Breakdown`.
//!
//! The other systems note the facts the modifiers read through [`note`]:
//! teaching and learning (`jobs::teach`, `knowledge::oral`), a gated craft
//! (`knowledge::used`), a discovery (`knowledge::discover`), a quarrel
//! (`quarrel`), and a kin's death (`survival::despawn_agent` → [`kin_died`]).
use hecs::{Entity, World};
use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
use sim_core::colony::content::PressureMechanic;
use sim_core::colony::people::{Drafted, Modifier, Mood, Sick, NO_DAY};
use sim_core::colony::story::PressureKind;
use sim_core::colony::CancelReason;
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Cold, Hunger, Kin, Life, Position, TargetKind, Temperament,
    Thirst,
};
use sim_core::rules::MoodRule;
use sim_engine::{RuntimeSystem, SimResources};

pub mod quarrel;

pub use quarrel::{QuarrelSystem, QUARREL_PRIORITY};

/// A breakdown found this interval: the person and their modifiers.
type Breakdown = (AgentId, Entity, Vec<(Modifier, i32)>);

/// Priority of [`MoodSystem`].
pub const MOOD_PRIORITY: u32 = 172;

/// Night facts (`Mood::nights`): last night in bits 0–2, tonight in bits 3–5.
const ROOFED: u8 = 1;
const OUTSIDE: u8 = 2;
const COLD: u8 = 4;
const TONIGHT: u8 = 3;

/// The mood rule in force (`None` outside the M5 colony).
pub fn rule(res: &SimResources) -> Option<MoodRule> {
    res.rules.colony.and_then(|c| c.mood)
}

/// Today's display day as a mood day.
pub fn today(res: &SimResources) -> u16 {
    crate::runtime::resource_regen::day_index(res).min(u64::from(NO_DAY - 1)) as u16
}

/// A fact the other systems note for a person's mood.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Fact {
    /// They taught or learned today.
    Learned,
    /// They worked a gated craft as its practitioner today.
    Crafted,
    /// They discovered something.
    Discovered,
}

/// Note `fact` for `entity` (no-op without the mood rule or a `Mood`).
pub fn note(world: &mut World, res: &SimResources, entity: Entity, fact: Fact) {
    let Some(rule) = rule(res) else {
        return;
    };
    let day = today(res);
    if let Ok(mut m) = world.get::<&mut Mood>(entity) {
        match fact {
            Fact::Learned => m.learned_day = day,
            Fact::Crafted => m.crafted_day = day,
            Fact::Discovered => m.joy_until = day.saturating_add(rule.joy_days.saturating_sub(1)),
        }
    }
}

/// A person died (`dead`, chronicle entry `entry`): their parents, children and
/// partner grieve for `grief_days`, the grief pointing at the death.
pub fn kin_died(
    world: &mut World,
    res: &SimResources,
    dead: AgentId,
    kin: Option<Kin>,
    entry: u32,
) {
    let Some(rule) = rule(res) else {
        return;
    };
    let day = today(res);
    let close: Vec<Entity> = world
        .query::<(&Agent, Option<&Kin>)>()
        .with::<&Mood>()
        .iter()
        .filter(|(_, (a, k))| {
            kin.is_some_and(|d| {
                d.mother == Some(a.id) || d.father == Some(a.id) || d.partner == Some(a.id)
            }) || k.is_some_and(|k| {
                k.mother == Some(dead) || k.father == Some(dead) || k.partner == Some(dead)
            })
        })
        .map(|(e, _)| e)
        .collect();
    for e in close {
        if let Ok(mut m) = world.get::<&mut Mood>(e) {
            m.grief_until = day.saturating_add(rule.grief_days.saturating_sub(1));
            m.grief_entry = entry;
        }
    }
}

/// The work-speed factor of `entity`'s mood, ‰: `low_work_permille` under tantrum
/// risk, `high_work_permille` when content, else 1,000 (and 1,000 without a mood).
pub fn work_permille(world: &World, res: &SimResources, entity: Entity) -> u32 {
    let Some(rule) = rule(res) else {
        return 1000;
    };
    match world.get::<&Mood>(entity).map(|m| m.value) {
        Ok(v) if v < rule.low => rule.low_work_permille,
        Ok(v) if v >= rule.high => rule.high_work_permille,
        _ => 1000,
    }
}

/// Whether `entity` is in a breakdown today (the job layer gives them no job).
pub fn broken(world: &World, res: &SimResources, entity: Entity) -> bool {
    rule(res).is_some()
        && world
            .get::<&Mood>(entity)
            .is_ok_and(|m| m.broken(today(res)))
}

/// What a person's mood reads this interval.
#[derive(Debug, Clone, Copy, Default)]
pub struct Facts {
    /// `Mood::nights` (last night in the low bits).
    pub nights: u8,
    /// Hunger at or past its seek threshold.
    pub hungry: bool,
    /// Thirst at or past its seek threshold.
    pub thirsty: bool,
    /// Sick.
    pub sick: bool,
    /// Drafted a day or more.
    pub drafted_long: bool,
    /// A pressure is active.
    pub pressure: bool,
    /// The day fields of the mood.
    pub mood: Option<Mood>,
    /// Today.
    pub day: u16,
    /// Harm avoidance and reward dependence ‰ (500 without a temperament).
    pub ha: u16,
    /// See [`Self::ha`].
    pub rd: u16,
    /// The weights of the pressure-related modifiers (M5.5 calibration knobs; `MoodRule::cold_weight` …).
    pub weights: Weights,
}

/// The rule's weights of the pressure-related modifiers (M5.5, goal §3.1.6): cold, hungry, sick, pressure, grief.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Weights {
    /// `MoodRule::cold_weight`.
    pub cold: i16,
    /// `MoodRule::hungry_weight`.
    pub hungry: i16,
    /// `MoodRule::sick_weight`.
    pub sick: i16,
    /// `MoodRule::pressure_weight`.
    pub pressure: i16,
    /// `MoodRule::grief_weight`.
    pub grief: i16,
}

impl Default for Weights {
    fn default() -> Self {
        Self::of(&MoodRule::M5)
    }
}

impl Weights {
    /// The weights of `rule`.
    pub const fn of(rule: &MoodRule) -> Self {
        Self {
            cold: rule.cold_weight,
            hungry: rule.hungry_weight,
            sick: rule.sick_weight,
            pressure: rule.pressure_weight,
            grief: rule.grief_weight,
        }
    }

    /// The weight of `m`: the rule's for the pressure-related modifiers, else the table's (`Modifier::weight`).
    pub const fn weight(&self, m: Modifier) -> i32 {
        match m {
            Modifier::Cold => self.cold as i32,
            Modifier::Hungry => self.hungry as i32,
            Modifier::Sick => self.sick as i32,
            Modifier::Pressure => self.pressure as i32,
            Modifier::Grief => self.grief as i32,
            other => other.weight(),
        }
    }
}

/// The active modifiers of a person with `f`, each with its weight after temperament.
pub fn modifiers(f: &Facts) -> Vec<(Modifier, i32)> {
    let mut out = Vec::with_capacity(8);
    let last = f.nights & 7;
    let mut push = |m: Modifier| {
        let w = f.weights.weight(m);
        let scaled = if m.social() {
            w * (800 + i32::from(f.rd) / 5) / 1000
        } else if w < 0 {
            w * (800 + i32::from(f.ha) / 5) / 1000
        } else {
            w
        };
        out.push((m, scaled));
    };
    if last & ROOFED != 0 {
        push(Modifier::SleptRoofed);
    } else if last & OUTSIDE != 0 {
        push(Modifier::SleptOutside);
    }
    push(if f.hungry {
        Modifier::Hungry
    } else {
        Modifier::Fed
    });
    if f.thirsty {
        push(Modifier::Thirsty);
    }
    if last & COLD != 0 {
        push(Modifier::Cold);
    }
    if let Some(m) = f.mood {
        if m.quarrel_day == f.day {
            push(Modifier::Quarrelled);
        }
        if m.learned_day == f.day {
            push(Modifier::Learned);
        }
        if Mood::within(m.joy_until, f.day) {
            push(Modifier::Discovered);
        }
        if Mood::within(m.grief_until, f.day) {
            push(Modifier::Grief);
        }
        if m.crafted_day == f.day {
            push(Modifier::Crafted);
        }
    }
    if f.sick {
        push(Modifier::Sick);
    }
    if f.drafted_long {
        push(Modifier::Drafted);
    }
    if f.pressure {
        push(Modifier::Pressure);
    }
    out
}

/// The mood target of `mods`: `base` + Σ, clamped to 0–1,000.
pub fn target(rule: &MoodRule, mods: &[(Modifier, i32)]) -> i16 {
    (i32::from(rule.base) + mods.iter().map(|(_, w)| w).sum::<i32>()).clamp(0, 1000) as i16
}

/// One step of `value` toward `target`: 1/`divisor` of the gap, at least 1 while there is one.
pub fn step(value: i16, target: i16, divisor: i16) -> i16 {
    let gap = i32::from(target) - i32::from(value);
    if gap == 0 {
        return value;
    }
    let mut d = gap / i32::from(divisor.max(1));
    if d == 0 {
        d = gap.signum();
    }
    (i32::from(value) + d).clamp(0, 1000) as i16
}

/// The facts of `entity` now (the bridge's mood line reads the same).
pub fn facts_of(world: &World, res: &SimResources, entity: Entity) -> Facts {
    use crate::runtime::decision::needs as n;
    let day = today(res);
    let day_ticks = res.rules.calendar.days_to_ticks(1);
    let mood = world.get::<&Mood>(entity).ok().map(|m| *m);
    let t = world.get::<&Temperament>(entity).ok().map(|t| *t);
    Facts {
        nights: mood.map_or(0, |m| m.nights),
        hungry: world
            .get::<&Hunger>(entity)
            .is_ok_and(|h| h.value >= n::HUNGER_THRESHOLD),
        thirsty: world
            .get::<&Thirst>(entity)
            .is_ok_and(|t| t.value >= n::THIRST_THRESHOLD),
        sick: world.get::<&Sick>(entity).is_ok(),
        drafted_long: world
            .get::<&Drafted>(entity)
            .is_ok_and(|d| res.current_tick.saturating_sub(d.since) >= day_ticks),
        pressure: crate::runtime::story::any_active(res),
        mood,
        day,
        ha: t.map_or(500, |t| t.ha),
        rd: t.map_or(500, |t| t.rd),
        weights: rule(res).map_or_else(Weights::default, |r| Weights::of(&r)),
    }
}

/// The chronicle cause of modifier `m` for a person with `mood`: the modifier,
/// with the entry behind it when there is one (a grief's death, a cold night's
/// or a hunger's pressure).
pub fn cause_of(res: &SimResources, m: Modifier, mood: Option<Mood>) -> CauseRef {
    let pressure = |mechanic| {
        crate::runtime::story::active_mechanic(res, mechanic)
            .and_then(|active| crate::runtime::story::cause_ref(res, active.kind))
            .and_then(|c| match c {
                CauseRef::Pressure { entry, .. } => entry,
                _ => None,
            })
    };
    let entry = match m {
        Modifier::Grief => mood.map(|m| m.grief_entry).filter(|e| *e != u32::MAX),
        Modifier::Cold | Modifier::SleptOutside => pressure(PressureMechanic::Cold),
        Modifier::Hungry | Modifier::Thirsty => pressure(PressureMechanic::Drought),
        Modifier::Sick => pressure(PressureMechanic::Illness),
        Modifier::Pressure | Modifier::Drafted => {
            res.colony.story.active.first().and_then(|p| p.began_entry)
        }
        _ => None,
    };
    CauseRef::Modifier {
        code: m.code(),
        entry,
    }
}

/// Resolve a person's modifier cause, retaining a lingering novel illness's origin.
pub fn cause_for_person(
    res: &SimResources,
    id: AgentId,
    m: Modifier,
    mood: Option<Mood>,
) -> CauseRef {
    if m == Modifier::Sick && res.content_state.illness_origins.contains_key(&id) {
        return CauseRef::Modifier {
            code: m.code(),
            entry: crate::runtime::story::illness_origin(res, id)
                .and_then(|kind| crate::runtime::story::illness_cause(res, kind))
                .and_then(|cause| match cause {
                    CauseRef::Pressure { entry, .. } => entry,
                    _ => None,
                }),
        };
    }
    cause_of(res, m, mood)
}

/// M5.5 (ADR 0030 amendment): the concrete record behind modifier `m` as a quarrel's cause for the person `id`
/// (`entity`, facts `f`): hunger, thirst, a cold or an outdoor night are sampled now (`NeedSample`, kept in the
/// chronicle; with the pressure behind them for the "why?" chain); grief points at the death, sickness, drafting and a
/// pressure at the pressure's `PressureBegan`. Anything else has no record (`entry: None`).
pub fn cause_record(
    world: &World,
    res: &mut SimResources,
    entity: Entity,
    id: AgentId,
    m: Modifier,
    f: &Facts,
) -> CauseRef {
    match m {
        Modifier::Hungry | Modifier::Thirsty | Modifier::Cold | Modifier::SleptOutside => {
            let entry = match cause_of(res, m, f.mood) {
                CauseRef::Modifier { entry, .. } => entry,
                _ => None,
            };
            let tenths = |v: f64| (v.clamp(0.0, 100.0) * 10.0) as u16;
            let sample = sim_core::chronicle::NeedSample {
                agent: id,
                tick: res.current_tick,
                hunger: world
                    .get::<&Hunger>(entity)
                    .map_or(0, |h| tenths(f64::from(h.value))),
                thirst: world.get::<&Thirst>(entity).map_or(0, |t| tenths(t.value)),
                nights: f.nights & 7,
                entry,
            };
            CauseRef::Sample {
                code: m.code(),
                sample: res.chronicle.push_sample(sample),
            }
        }
        _ => cause_for_person(res, id, m, f.mood),
    }
}

/// What record a cause must point at (review F4: the record must fit the cause, not only exist).
#[derive(Clone, Copy)]
enum Expect {
    /// Any record kind the checker knows.
    Any,
    /// A death (grief).
    Died,
    /// A `PressureBegan`, of this `PressureKind` code when given.
    Pressure(Option<u32>),
    /// A pressure with the expected registered mechanic.
    Mechanic(PressureMechanic),
    /// An earlier quarrel of the same pair (the relation).
    PairQuarrel,
}

/// The record modifier code `code` must point at.
fn expect_of(code: u8) -> Expect {
    if code == sim_core::colony::people::RELATION_CAUSE {
        return Expect::PairQuarrel;
    }
    match Modifier::from_code(code) {
        Some(Modifier::Grief) => Expect::Died,
        Some(Modifier::Sick) => Expect::Mechanic(PressureMechanic::Illness),
        Some(Modifier::Cold | Modifier::SleptOutside) => Expect::Mechanic(PressureMechanic::Cold),
        Some(Modifier::Hungry | Modifier::Thirsty) => Expect::Mechanic(PressureMechanic::Drought),
        Some(Modifier::Pressure | Modifier::Drafted) => Expect::Pressure(None),
        _ => Expect::Any,
    }
}

/// M5.5 (ADR 0030 amendment, goal §3.1.7): whether the cause of quarrel entry `index` resolves — its record exists,
/// fits the cause (grief → a death, sickness → an illness, cold → a cold snap, hunger or thirst → a drought, a
/// pressure or drafting → a pressure, the relation → the same pair's earlier quarrel), belongs to the initiator (the
/// entry's subject; the world for a pressure) and lies within `window` ticks before the quarrel (a state — a pressure
/// — lies within it when it was in force at some tick of it). A need sample must also show the need (hunger or
/// thirst at its seek threshold, the night's cold or outdoor sleep). `Err` says why not. A partner is recognised
/// through the dead's recorded partner (`Died.other`) only: the chronicle keeps no partner of the living.
pub fn check_cause(
    chronicle: &sim_core::chronicle::Chronicle,
    index: u32,
    window: u64,
) -> Result<(), &'static str> {
    use crate::runtime::decision::needs as n;
    let q = chronicle.entry(index).ok_or("no such entry")?;
    if q.kind != ChronicleKind::Quarrel {
        return Err("not a quarrel");
    }
    let who = q.subject.ok_or("no initiator")?;
    let pair = {
        let (a, b) = (Some(who), q.other);
        if a <= b {
            (a, b)
        } else {
            (b, a)
        }
    };
    let from = q.tick.saturating_sub(window);
    let within = |t: u64| (from..=q.tick).contains(&t);
    let entry_ok = |e: u32, expect: Expect| -> Result<(), &'static str> {
        let rec = chronicle.entry(e).ok_or("record missing")?;
        let fits = match (expect, rec.kind) {
            (Expect::Any, _) => true,
            (Expect::Died, ChronicleKind::Died) => true,
            (Expect::Pressure(kind), ChronicleKind::PressureBegan) => {
                kind.is_none_or(|k| rec.cause == k)
            }
            (Expect::Mechanic(mechanic), ChronicleKind::PressureBegan) => {
                PressureKind::from_code(rec.cause)
                    .is_some_and(|kind| kind.definition().mechanic == mechanic)
            }
            (Expect::PairQuarrel, ChronicleKind::Quarrel) => {
                let p = if rec.subject <= rec.other {
                    (rec.subject, rec.other)
                } else {
                    (rec.other, rec.subject)
                };
                p == pair
            }
            _ => false,
        };
        if !fits {
            return Err("the record does not fit the cause");
        }
        match rec.kind {
            ChronicleKind::PressureBegan => {
                if rec.tick > q.tick {
                    return Err("pressure after the quarrel");
                }
                let ended = (0..chronicle.len() as u32).find(|&i| {
                    chronicle
                        .entry(i)
                        .is_some_and(|x| x.kind == ChronicleKind::PressureEnded)
                        && chronicle.cause_of(i) == Some(CauseRef::Entry(e))
                });
                match ended.and_then(|i| chronicle.entry(i)) {
                    Some(end) if end.tick < from => Err("pressure ended before the window"),
                    _ => Ok(()),
                }
            }
            ChronicleKind::Died => {
                let dead = rec.subject.ok_or("death without a person")?;
                let parents = |id| chronicle.person(id).map(|p| (p.mother, p.father));
                let kin = rec.other == Some(who)
                    || parents(dead).is_some_and(|(m, f)| m == Some(who) || f == Some(who))
                    || parents(who).is_some_and(|(m, f)| m == Some(dead) || f == Some(dead));
                if !kin {
                    Err("the death is not the initiator's kin")
                } else if !within(rec.tick) {
                    Err("the death lies outside the window")
                } else {
                    Ok(())
                }
            }
            ChronicleKind::Quarrel => {
                if rec.subject != Some(who) && rec.other != Some(who) {
                    Err("the earlier quarrel is not the initiator's")
                } else if !within(rec.tick) || e >= index {
                    Err("the earlier quarrel lies outside the window")
                } else {
                    Ok(())
                }
            }
            _ => Err("a record of another kind"),
        }
    };
    match chronicle.cause_of(index) {
        None => Err("no cause"),
        Some(CauseRef::Entry(e)) => entry_ok(e, Expect::Any),
        Some(CauseRef::Modifier {
            code,
            entry: Some(e),
        }) => entry_ok(e, expect_of(code)),
        Some(CauseRef::Modifier { entry: None, .. }) => Err("no record behind the modifier"),
        Some(CauseRef::Pressure {
            kind,
            entry: Some(e),
        }) => entry_ok(e, Expect::Pressure(Some(kind))),
        Some(CauseRef::Pressure { entry: None, .. }) => Err("no record behind the pressure"),
        Some(CauseRef::Sample { code, sample }) => {
            let s = chronicle.sample(sample).ok_or("sample missing")?;
            if s.agent != who {
                return Err("the sample is not the initiator's");
            }
            if !within(s.tick) {
                return Err("the sample lies outside the window");
            }
            let shows = match Modifier::from_code(code) {
                Some(Modifier::Hungry) => {
                    f64::from(s.hunger) >= f64::from(n::HUNGER_THRESHOLD) * 10.0
                }
                Some(Modifier::Thirsty) => f64::from(s.thirst) >= n::THIRST_THRESHOLD * 10.0,
                Some(Modifier::Cold) => s.nights & COLD != 0,
                Some(Modifier::SleptOutside) => s.nights & OUTSIDE != 0,
                _ => false,
            };
            if shows {
                Ok(())
            } else {
                Err("the sample does not show the need")
            }
        }
    }
}

/// One link of a "→ 왜?" chain (ADR 0030).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Link {
    /// A chronicle entry (its index).
    Entry(u32),
    /// A mood modifier (`Modifier::code`).
    Modifier(u8),
    /// A pressure (`PressureKind::code`).
    Pressure(u32),
}

/// The "why?" chain of chronicle entry `index`: the entry, then its cause, the
/// entry behind that cause, and so on (at most 8 links; a loop stops it).
pub fn why_chain(chronicle: &sim_core::chronicle::Chronicle, index: u32) -> Vec<Link> {
    let mut out = vec![Link::Entry(index)];
    let mut at = index;
    let mut seen = std::collections::BTreeSet::from([index]);
    while out.len() < 8 {
        let next = match chronicle.cause_of(at) {
            None => break,
            Some(CauseRef::Entry(e)) => Some(e),
            Some(CauseRef::Pressure { kind, entry }) => {
                out.push(Link::Pressure(kind));
                entry
            }
            Some(CauseRef::Modifier { code, entry }) => {
                out.push(Link::Modifier(code));
                entry
            }
            // M5.5: a modifier backed by a need sample; the chain goes on to the pressure behind it, if any.
            Some(CauseRef::Sample { code, sample }) => {
                out.push(Link::Modifier(code));
                chronicle.sample(sample).and_then(|s| s.entry)
            }
        };
        match next {
            Some(e) if seen.insert(e) && chronicle.entry(e).is_some() => {
                out.push(Link::Entry(e));
                at = e;
            }
            _ => break,
        }
    }
    out
}

/// Whether the clock is in the night window (20:00–06:00).
fn night(res: &SimResources) -> bool {
    let minute = res
        .current_tick
        .saturating_add(res.rules.clock_offset_ticks)
        % res.ticks_per_day.max(1);
    !(360..1200).contains(&minute)
}

/// See the module header.
#[derive(Debug, Default)]
pub struct MoodSystem;

impl RuntimeSystem for MoodSystem {
    fn name(&self) -> &str {
        "MoodSystem"
    }

    fn priority(&self) -> u32 {
        MOOD_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        60
    }

    /// Minute 7 of each hour: off the hour-aligned ticks where the 60-, 120- and 240-tick systems run
    /// (DECISIONS D18).
    fn tick_offset(&self, _resources: &SimResources) -> u64 {
        7
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = rule(res) else {
            return;
        };
        let tick = res.current_tick;
        let year = res.rules.calendar.years_to_ticks(1).max(1);
        // Adults and children of `child_years` or more get a mood.
        let mut fresh: Vec<(AgentId, Entity)> = world
            .query::<(&Agent, Option<&Child>, Option<&Life>)>()
            .without::<&Mood>()
            .iter()
            .filter(|(_, (_, child, life))| {
                child.is_none()
                    || life.is_some_and(|l| {
                        u64::try_from(l.age_ticks(tick)).unwrap_or(0) / year >= rule.child_years
                    })
            })
            .map(|(e, (a, ..))| (a.id, e))
            .collect();
        fresh.sort_by_key(|row| row.0);
        for (_, e) in fresh {
            let _ = world.insert_one(
                e,
                Mood {
                    value: rule.base,
                    ..Mood::default()
                },
            );
        }
        let minute = tick.saturating_add(res.rules.clock_offset_ticks) % res.ticks_per_day.max(1);
        // The run in the 8 o'clock hour (one a day; the system runs at minute 7 of each hour, D18).
        let morning = minute / 60 == 8;
        let is_night = night(res);
        let day = today(res);
        let mut rows: Vec<(AgentId, Entity)> = world
            .query::<&Agent>()
            .with::<&Mood>()
            .iter()
            .map(|(e, a)| (a.id, e))
            .collect();
        rows.sort_by_key(|row| row.0);
        let mut breakdowns: Vec<Breakdown> = Vec::new();
        for (id, e) in rows {
            // The night's facts.
            let mut nights = world.get::<&Mood>(e).map_or(0, |m| m.nights);
            if is_night {
                let asleep = world.get::<&AgentState>(e).is_ok_and(|s| {
                    matches!(
                        *s,
                        AgentState::Consuming {
                            target: TargetKind::Sleep
                        }
                    )
                });
                let pos = world.get::<&Position>(e).map(|p| (p.x, p.y)).ok();
                if asleep {
                    let roofed_bed = pos.is_some_and(|p| {
                        crate::runtime::rooms::roofed_as(res, p, sim_core::rooms::RoomRole::Bedroom)
                    });
                    let roofed = pos.is_some_and(|p| crate::runtime::rooms::roofed(res, p));
                    if roofed_bed {
                        nights |= ROOFED << TONIGHT;
                    } else if !roofed {
                        nights |= OUTSIDE << TONIGHT;
                    }
                }
                if world.get::<&Cold>(e).is_ok() {
                    nights |= COLD << TONIGHT;
                }
            }
            if morning {
                nights = (nights >> TONIGHT) & 7;
            }
            if let Ok(mut m) = world.get::<&mut Mood>(e) {
                m.nights = nights;
            }
            let facts = facts_of(world, res, e);
            let mods = modifiers(&facts);
            let goal = target(&rule, &mods);
            let mut start = false;
            if let Ok(mut m) = world.get::<&mut Mood>(e) {
                m.value = step(m.value, goal, rule.step_divisor);
                if m.value < rule.breakdown && !m.broken(day) {
                    m.breakdown_until = day.saturating_add(rule.breakdown_days);
                    start = true;
                }
            }
            if start {
                breakdowns.push((id, e, mods));
            }
        }
        for (id, e, mut mods) in breakdowns {
            if world.get::<&sim_core::colony::Job>(e).is_ok() {
                crate::runtime::jobs::cancel(world, res, e, CancelReason::Needs);
            }
            mods.retain(|(_, w)| *w < 0);
            mods.sort_by_key(|(m, w)| (*w, *m));
            mods.truncate(3);
            let code = mods.iter().enumerate().fold(0i64, |acc, (i, (m, _))| {
                acc | (i64::from(m.code()) << (8 * i))
            });
            let mood = world.get::<&Mood>(e).ok().map(|m| *m);
            let cause = mods
                .first()
                .map(|(m, _)| cause_for_person(res, id, *m, mood));
            res.colony.mood_tallies.breakdowns += 1;
            res.chronicle.push_caused(
                ChronicleEntry::new(tick, ChronicleKind::Breakdown)
                    .subject(id)
                    .cause(u32::from(mods.len() as u8))
                    .value(code),
                cause,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_value_converges_to_the_target_without_stalling() {
        let mut v = 500;
        for _ in 0..80 {
            v = step(v, 230, 8);
        }
        assert_eq!(v, 230, "reaches the target exactly");
        assert_eq!(
            step(500, 503, 8),
            501,
            "a gap below the divisor still moves"
        );
        assert_eq!(step(500, 420, 8), 490);
    }

    #[test]
    fn modifiers_follow_the_goal_table_and_temperament() {
        let rule = MoodRule::M5;
        let base = Facts {
            nights: ROOFED,
            ha: 1000,
            rd: 1000,
            day: 5,
            ..Facts::default()
        };
        let m = modifiers(&base);
        assert_eq!(m, vec![(Modifier::SleptRoofed, 80), (Modifier::Fed, 40)]);
        assert_eq!(target(&rule, &m), 620);
        let cold = Facts {
            nights: OUTSIDE | COLD,
            hungry: true,
            ha: 0,
            ..base
        };
        // ha 0: negatives ×800 ‰: −96, −120, −96.
        assert_eq!(
            modifiers(&cold),
            vec![
                (Modifier::SleptOutside, -96),
                (Modifier::Hungry, -120),
                (Modifier::Cold, -96)
            ]
        );
        assert_eq!(target(&rule, &modifiers(&cold)), 188);
    }

    /// M5.5 (goal §3.1.7): a quarrel's cause resolves only when its record exists, is the initiator's (or the
    /// world's) and lies within the window — each failing case fails.
    #[test]
    fn quarrel_causes_resolve_only_to_a_fitting_record() {
        use sim_core::chronicle::{Chronicle, NeedSample, PersonRecord};
        const DAY: u64 = 1_440;
        let window = 2 * DAY;
        let quarrel =
            |c: &mut Chronicle, tick: u64, who: AgentId, other: AgentId, cause: CauseRef| {
                c.push_caused(
                    ChronicleEntry::new(tick, ChronicleKind::Quarrel)
                        .subject(who)
                        .other(Some(other))
                        .cause(1),
                    Some(cause),
                )
            };
        let sample = |c: &mut Chronicle, who: AgentId, tick: u64, hunger: u16, nights: u8| {
            c.push_sample(NeedSample {
                agent: who,
                tick,
                hunger,
                thirst: 0,
                nights,
                entry: None,
            })
        };
        let mut c = Chronicle::new();
        let hungry = Modifier::Hungry.code();
        // A hungry initiator sampled at the quarrel: resolves.
        let s = sample(&mut c, 1, 10 * DAY, 700, 0);
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Sample {
                code: hungry,
                sample: s,
            },
        );
        assert_eq!(check_cause(&c, q, window), Ok(()));
        // Someone else's sample, an old sample, a sample that is not hungry: each fails.
        let s = sample(&mut c, 2, 10 * DAY, 700, 0);
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Sample {
                code: hungry,
                sample: s,
            },
        );
        assert!(check_cause(&c, q, window).is_err());
        let s = sample(&mut c, 1, 7 * DAY, 700, 0);
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Sample {
                code: hungry,
                sample: s,
            },
        );
        assert!(check_cause(&c, q, window).is_err());
        let s = sample(&mut c, 1, 10 * DAY, 100, 0);
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Sample {
                code: hungry,
                sample: s,
            },
        );
        assert!(check_cause(&c, q, window).is_err());
        // Last night's cold: the sample must carry the cold bit.
        let cold = Modifier::Cold.code();
        let s = sample(&mut c, 1, 10 * DAY, 0, COLD);
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Sample {
                code: cold,
                sample: s,
            },
        );
        assert_eq!(check_cause(&c, q, window), Ok(()));
        let s = sample(&mut c, 1, 10 * DAY, 0, OUTSIDE);
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Sample {
                code: cold,
                sample: s,
            },
        );
        assert!(check_cause(&c, q, window).is_err());
        // A pressure: in force within the window resolves; one that ended before it does not.
        let began = c.push_caused(
            ChronicleEntry::new(5 * DAY, ChronicleKind::PressureBegan).cause(3),
            None,
        );
        let pressure = Modifier::Pressure.code();
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Modifier {
                code: pressure,
                entry: Some(began),
            },
        );
        assert_eq!(check_cause(&c, q, window), Ok(()), "still in force");
        c.push_caused(
            ChronicleEntry::new(6 * DAY, ChronicleKind::PressureEnded).cause(3),
            Some(CauseRef::Entry(began)),
        );
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Modifier {
                code: pressure,
                entry: Some(began),
            },
        );
        assert!(
            check_cause(&c, q, window).is_err(),
            "ended four days before"
        );
        // Grief: the death of a child of the initiator, within the window; a stranger's death fails.
        c.ensure_person(7, || {
            Some(PersonRecord {
                mother: Some(1),
                ..PersonRecord::default()
            })
        });
        let died = c.push_caused(
            ChronicleEntry::new(9 * DAY, ChronicleKind::Died).subject(7),
            None,
        );
        let grief = Modifier::Grief.code();
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Modifier {
                code: grief,
                entry: Some(died),
            },
        );
        assert_eq!(check_cause(&c, q, window), Ok(()));
        let q = quarrel(
            &mut c,
            10 * DAY,
            3,
            2,
            CauseRef::Modifier {
                code: grief,
                entry: Some(died),
            },
        );
        assert!(check_cause(&c, q, window).is_err(), "not person 3's kin");
        // The relation: the pair's earlier quarrel within the window resolves; no record fails.
        let earlier = quarrel(
            &mut c,
            9 * DAY,
            2,
            1,
            CauseRef::Modifier {
                code: 0,
                entry: None,
            },
        );
        let rel = sim_core::colony::people::RELATION_CAUSE;
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Modifier {
                code: rel,
                entry: Some(earlier),
            },
        );
        assert_eq!(check_cause(&c, q, window), Ok(()));
        let q = quarrel(
            &mut c,
            10 * DAY,
            1,
            2,
            CauseRef::Modifier {
                code: rel,
                entry: None,
            },
        );
        assert!(check_cause(&c, q, window).is_err());
    }
}
