//! StorytellerSystem: plans each season's pressure, begins and ends it, and runs illness.
//!
//! Priority: 170 (`STORYTELLER_PRIORITY`). Interval: 1,440 ticks (the first tick of each day). Gate: `ColonyRule::story`.
//! Reads: `Agent`, `Child`, `Sick`, `Lineage`; `SimResources.colony` (story, ground items, zones, rooms, beds), `bed_of`, `tallies` (deaths), `rules` (story, calendar, seasons), `world_seed`
//! Writes: `Sick` (inserted, removed); `SimResources.colony.story` (planned, active, history, version), `chronicle`, `causal_log`
//! Events: chronicle `PressureBegan`, `PressureEnded`, `Sick`, `Recovered`; causal `PressureCause` (illness onset)
//!
//! M5 (ADR 0027). On the first day of each season the season is planned with
//! keyed draws (`STREAM_STORY`, subject = the season's index since the start):
//! one **major** on a day drawn among the season's days 2–5 (0-based 1–4) of a
//! kind drawn by season (winter ColdSnap 600 / Beasts 400, summer Drought 600 /
//! Beasts 400, spring Beasts 700 / none, autumn Beasts 700 / Illness 300), and in
//! autumn and winter one **minor** Illness. A kind that has not come for a while
//! is *due* and replaces the draw (ColdSnap after a winter without one, Beasts
//! after three seasons); mercy rules and the gaps move a day forward inside the
//! season or drop the pressure (M5 DECISIONS D7). Each day the planned
//! pressures of the day begin (intensity from the colony's wealth and size)
//! and the active ones whose end day came end with their cost; beasts also end
//! when their pack is gone (`end_pressure`, called by the animal system).
//!
//! M5.5 (ADR 0027 amendment, `StoryRule::no_stack`): pressures do not stack.
//! A planned illness whose day comes while a major is active or within
//! `stack_gap_days` after one ended, and a planned major whose day comes while
//! `major_wait_sick` or more people are sick, wait a day at a time inside their
//! season ([`held_back`]); one that cannot begin before its season's last
//! allowed day is dropped (`skipped`).
//!
//! Effects live where the behaviour lives: `FireSystem` (cold snap burn,
//! exposure, roofed sleepers), `ResourceRegenSystem` and `ThirstDecaySystem`
//! (drought), `SleepDecaySystem` and the job layer's work ticks (illness, the
//! snap's outdoor work), the animal system (beasts). They ask [`active`].
use hecs::{Entity, World};
use sim_core::causal::CausalEvent;
use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
use sim_core::colony::content::{installed_or_bundled, PressureIntensity, PressureMechanic};
use sim_core::colony::people::Sick;
use sim_core::colony::story::{Pressure, PressureCost, PressureKind, Storyteller};
use sim_core::components::{Agent, AgentId, Child, Lineage, Position};
use sim_core::rng::{keyed_below, STREAM_STORY};
use sim_core::rules::StoryRule;
use sim_engine::{RuntimeSystem, SimResources};

/// Priority of [`StorytellerSystem`].
pub const STORYTELLER_PRIORITY: u32 = 170;

/// Season indices of the calendar (0 spring … 3 winter).
const WINTER: u32 = 3;

/// The story rule in force (`None` outside the M5 colony).
pub fn rule(res: &SimResources) -> Option<StoryRule> {
    res.rules.colony.and_then(|c| c.story)
}

/// The story rule when a pressure of `kind` is in force, else `None`.
pub fn active(res: &SimResources, kind: PressureKind) -> Option<StoryRule> {
    rule_for(res, kind).filter(|_| res.colony.story.is_active(kind))
}

/// Actual active identity and effects resolved against this world's rules.
#[derive(Debug, Clone, Copy)]
pub struct ActivePressure {
    /// Registered identity, retained in costs and causes.
    pub kind: PressureKind,
    /// Current world rules projected through the pressure definition.
    pub rule: StoryRule,
}

/// Resolve definition effects without freezing per-world rule overrides.
pub fn rule_for(res: &SimResources, kind: PressureKind) -> Option<StoryRule> {
    rule(res).map(|rule| kind.definition().resolve_rule(&rule))
}

/// Find the active mechanic; admission rejects distinct identities sharing one mechanic.
pub fn active_mechanic(res: &SimResources, mechanic: PressureMechanic) -> Option<ActivePressure> {
    let pressure = res
        .colony
        .story
        .active
        .iter()
        .filter(|p| p.kind.definition().mechanic == mechanic)
        .min_by_key(|p| p.kind)?;
    Some(ActivePressure {
        kind: pressure.kind,
        rule: rule_for(res, pressure.kind)?,
    })
}

/// Illness provenance; bundled illness deliberately uses no sparse map entry.
pub fn illness_origin(res: &SimResources, id: AgentId) -> Option<PressureKind> {
    res.content_state
        .illness_origins
        .get(&id)
        .copied()
        .or_else(|| installed_or_bundled().pressure_role("illness_default"))
}

/// Per-person illness effects, including an illness whose pressure has ended.
pub fn illness_rule(res: &SimResources, id: AgentId) -> Option<StoryRule> {
    rule_for(res, illness_origin(res, id)?)
}

/// Firewood the hearth can burn now: the settlements' wood plus feedable firewood items
/// (the pressure banner's "장작 N" and F2).
pub fn firewood_have(res: &SimResources) -> u64 {
    u64::from(crate::runtime::jobs::hearth::settlement_wood(res))
        + u64::from(crate::runtime::jobs::hearth::feedable_firewood(res))
}

/// One night's burn at a cold snap's rate (the banner's "하루 N 필요" and F2): the burn
/// window's whole hours × tonight's hourly rate × the hearths × `snap_burn_permille`
/// (1,000 outside the M5 colony). 0 without the firewood rule.
pub fn firewood_day_need(res: &SimResources) -> u64 {
    let Some(fw) = res.rules.society.firewood else {
        return 0;
    };
    let factor = active_mechanic(res, PressureMechanic::Cold)
        .map(|p| p.rule)
        .or_else(|| {
            installed_or_bundled()
                .pressure_role("cold_default")
                .and_then(|kind| rule_for(res, kind))
        })
        .map_or(1000, |s| s.snap_burn_permille);
    let (s, e) = (fw.burn_start_minute, fw.burn_end_minute);
    let span = if e > s { e - s } else { 1440 - s + e };
    let hours = u64::from(span) / 60;
    let day = res.current_tick / res.ticks_per_day.max(1);
    let midnight = day * res.ticks_per_day.max(1) + 960;
    let per_hour = u64::from(crate::runtime::settlement::firewood::night_rate(
        res, &fw, midnight,
    ));
    let hearths = res.settlements.len().max(1) as u64;
    hours * per_hour * hearths * u64::from(factor) / 1000
}

/// Whether any pressure is in force (mood's "pressure active").
pub fn any_active(res: &SimResources) -> bool {
    rule(res).is_some() && !res.colony.story.active.is_empty()
}

/// The work-speed factor of `entity` working at `tile`, ‰ of its normal work
/// ticks (1,000 = unchanged): a cold snap outdoors (`snap_outdoor_work_permille`,
/// not for a fur-cloak wearer; outdoors = not in a roofed room), illness
/// (`sick_work_permille`), and mood (ADR 0030, `mood::work_permille`). Factors multiply.
pub fn work_permille(world: &World, res: &SimResources, entity: Entity, tile: (u32, u32)) -> u32 {
    if rule(res).is_none() {
        return 1000;
    }
    let mut factor: u64 = 1000;
    if let Some(cold) = active_mechanic(res, PressureMechanic::Cold).filter(|_| {
        !crate::runtime::rooms::roofed(res, tile)
            && !world
                .get::<&sim_core::colony::people::Apparel>(entity)
                .is_ok_and(|a| a.top.cold_snap_work_immune())
    }) {
        factor = factor * u64::from(cold.rule.snap_outdoor_work_permille) / 1000;
    }
    if world.get::<&Sick>(entity).is_ok() {
        let illness = world
            .get::<&Agent>(entity)
            .ok()
            .and_then(|a| illness_rule(res, a.id))
            .or_else(|| rule(res));
        if let Some(illness) = illness {
            factor = factor * u64::from(illness.sick_work_permille) / 1000;
        }
    }
    factor = factor * u64::from(crate::runtime::mood::work_permille(world, res, entity)) / 1000;
    factor.min(u64::from(u32::MAX)) as u32
}

/// `ticks` scaled by [`work_permille`] (rounded, at least 1).
pub fn scale_work(
    world: &World,
    res: &SimResources,
    entity: Entity,
    tile: (u32, u32),
    ticks: u32,
) -> u32 {
    let permille = work_permille(world, res, entity, tile);
    if permille == 1000 {
        return ticks;
    }
    ((u64::from(ticks) * u64::from(permille) + 500) / 1000).clamp(1, u64::from(u32::MAX)) as u32
}

/// Every death counted so far, of any cause.
pub fn deaths_so_far(res: &SimResources) -> u64 {
    let t = &res.tallies;
    t.deaths_starvation
        + t.deaths_dehydration
        + t.deaths_combat
        + t.deaths_old_age
        + t.deaths_exposure
}

/// Record a `PressureCause` causal event for `agent` at `pos` the first time the
/// active pressure of `kind` does `effect` to them (0 cold, 1 a wound, 2 sickness, 3 thirst).
pub fn note_cause(
    res: &mut SimResources,
    agent: AgentId,
    pos: (u32, u32),
    kind: PressureKind,
    effect: u8,
) {
    if !(res.colony.story.is_active(kind)
        || effect == 2 && res.content_state.illness_origins.get(&agent) == Some(&kind))
        || !res
            .colony
            .story
            .touched
            .insert((kind.code(), agent, effect))
    {
        return;
    }
    let width = res.tile_grid.width;
    let id = res.issue_event_id();
    res.causal_log.push(
        pos.1 * width + pos.0,
        CausalEvent::PressureCause {
            id,
            parent: None,
            agent,
            position: pos,
            kind: kind.code(),
            effect,
            tick: res.current_tick,
        },
    );
}

/// Add to the cost of the active pressure of `kind` (no-op when none is active).
pub fn add_cost(res: &mut SimResources, kind: PressureKind, items: u32, work_ticks: u64) {
    if let Some(p) = res.colony.story.active.iter_mut().find(|p| p.kind == kind) {
        p.cost.items_lost = p.cost.items_lost.saturating_add(items);
        p.cost.work_ticks_lost = p.cost.work_ticks_lost.saturating_add(work_ticks);
    }
}

/// End the active pressure of `kind` now (no-op when none is active): its cost is
/// closed, `PressureEnded` logged (cause → its `PressureBegan`), and it moves to the history.
pub fn end_pressure(res: &mut SimResources, kind: PressureKind) {
    let Some(at) = res.colony.story.active.iter().position(|p| p.kind == kind) else {
        return;
    };
    let mut done = res.colony.story.active.remove(at);
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    done.end_day = done.end_day.min(day.max(done.start_day));
    done.cost.deaths =
        u32::try_from(deaths_so_far(res).saturating_sub(done.deaths_at_start)).unwrap_or(u32::MAX);
    res.chronicle.push_caused(
        ChronicleEntry::new(res.current_tick, ChronicleKind::PressureEnded)
            .cause(kind.code())
            .value(i64::from(done.cost.deaths)),
        done.began_entry.map(CauseRef::Entry),
    );
    let code = kind.code();
    res.colony.story.touched.retain(|(k, ..)| *k != code);
    res.colony.story.history.push(done);
    res.colony.story.version += 1;
}

/// The `CauseRef` of the active pressure of `kind` (for mood modifiers and chronicle lines).
pub fn cause_ref(res: &SimResources, kind: PressureKind) -> Option<CauseRef> {
    res.colony
        .story
        .active_of(kind)
        .map(|p| CauseRef::Pressure {
            kind: kind.code(),
            entry: p.began_entry,
        })
}

/// A lingering novel illness keeps its pressure provenance after the active window.
pub fn illness_cause(res: &SimResources, kind: PressureKind) -> Option<CauseRef> {
    cause_ref(res, kind).or_else(|| {
        if Some(kind) == installed_or_bundled().pressure_role("illness_default") {
            return None;
        }
        res.colony
            .story
            .history
            .iter()
            .rev()
            .find(|p| p.kind == kind)
            .map(|p| CauseRef::Pressure {
                kind: kind.code(),
                entry: p.began_entry,
            })
    })
}

/// The season plan's inputs.
#[derive(Debug, Clone, Copy)]
pub struct SeasonSlot {
    /// Seasons since the start (0 = the first spring); the draws' subject.
    pub index: u32,
    /// The season (0 spring … 3 winter).
    pub season: u32,
    /// The year (0 = the first).
    pub year: u32,
    /// Its first display day.
    pub first_day: u32,
    /// Its length in days.
    pub days: u32,
}

/// The major kind a season draws (`u` in 0..1000), `None` for a quiet draw.
pub fn drawn_kind(season: u32, u: u64) -> Option<PressureKind> {
    installed_or_bundled()
        .pressure_schedule()
        .seasons
        .get(&season)
        .and_then(|rows| rows.iter().find(|row| u < u64::from(row.upper_exclusive)))
        .and_then(|row| row.kind)
}

/// The earliest and latest day of `slot` on which a major of `kind` may begin
/// (mercy and gaps; `None` when the season has no such day).
fn window(
    rule: &StoryRule,
    story: &Storyteller,
    slot: SeasonSlot,
    kind: PressureKind,
) -> Option<(u32, u32)> {
    let mut first = slot.first_day + 1;
    let mut last = slot.first_day + slot.days.saturating_sub(2).max(1);
    let mercy = &kind.definition().mercy;
    if let Some(value) = &mercy.first_day {
        first = first.max(value.story(rule));
    }
    if slot.year == 0 {
        if let Some(value) = &mercy.first_winter_days {
            first = first.max(slot.first_day + value.story(rule));
        }
    }
    if let (Some(value), Some(prev)) = (&mercy.repeat_gap_days, story.last_began(kind)) {
        first = first.max(prev + value.story(rule));
    }
    if slot.season == WINTER {
        if let Some(value) = &mercy.winter_end_days {
            last = last.min(slot.first_day + slot.days.saturating_sub(1 + value.story(rule)));
        }
    }
    if kind.is_major() {
        if let Some(prev) = story.last_major_day {
            first = first.max(prev + rule.major_gap_days);
        }
    }
    (first <= last).then_some((first, last))
}

/// Whether `kind` is due in `slot`: none came in its due window (ADR 0027).
fn due(rule: &StoryRule, story: &Storyteller, slot: SeasonSlot, kind: PressureKind) -> bool {
    let Some(days) = kind
        .definition()
        .due_days
        .as_ref()
        .map(|value| value.story(rule))
    else {
        return false;
    };
    if slot.first_day < days {
        return false;
    }
    story
        .last_began(kind)
        .is_none_or(|day| day + days <= slot.first_day)
}

/// Plan `slot`'s pressures: (the major, the minor) — pure in `(seed, slot, story)`.
pub fn plan_season(
    seed: u64,
    rule: &StoryRule,
    story: &Storyteller,
    slot: SeasonSlot,
) -> (Option<Pressure>, Option<Pressure>, bool) {
    let draw = |slot_no: u32, n: u64| {
        keyed_below(seed, STREAM_STORY, u64::from(slot.index), 0, slot_no, 0, n)
    };
    let mut kind = drawn_kind(slot.season, draw(1, 1000));
    kind = kind.filter(|kind| kind.definition().enabled(rule));
    let schedule = installed_or_bundled().pressure_schedule();
    for &candidate in &schedule.due_order {
        let def = candidate.definition();
        if def.enabled(rule)
            && def.due_seasons.contains(&slot.season)
            && due(rule, story, slot, candidate)
            && (def.mercy.repeat_gap_days.is_none()
                || window(rule, story, slot, candidate).is_some())
        {
            kind = Some(candidate);
            break;
        }
    }
    let offset = 1 + draw(0, u64::from(slot.days.saturating_sub(2).max(1))) as u32;
    let mut skipped = false;
    let major = kind.and_then(|kind| match window(rule, story, slot, kind) {
        Some((first, last)) => {
            let day = (slot.first_day + offset).max(first);
            if day > last {
                skipped = true;
                return None;
            }
            Some(planned(kind, day, duration(rule, kind, draw)))
        }
        None => {
            skipped = true;
            None
        }
    });
    let minor = schedule
        .minor
        .filter(|minor| {
            minor.definition().enabled(rule)
                && schedule.minor_seasons.contains(&slot.season)
                && kind != Some(*minor)
        })
        .and_then(|minor| {
            let day = slot.first_day
                + 1
                + draw(
                    schedule.minor_draw_slot,
                    u64::from(slot.days.saturating_sub(2).max(1)),
                ) as u32;
            let first = minor
                .definition()
                .mercy
                .first_day
                .as_ref()
                .map_or(0, |value| value.story(rule));
            (day >= first).then(|| planned(minor, day, duration(rule, minor, draw)))
        });
    (major, minor, skipped)
}

fn duration(rule: &StoryRule, kind: PressureKind, draw: impl Fn(u32, u64) -> u64) -> u32 {
    let def = kind.definition();
    let (lo, hi) = def.duration_span(rule);
    def.duration_draw_slot.map_or(lo, |slot| {
        lo + draw(slot, u64::from(hi.saturating_sub(lo) + 1)) as u32
    })
}

const fn planned(kind: PressureKind, day: u32, days: u32) -> Pressure {
    Pressure {
        kind,
        start_day: day,
        end_day: day + days,
        began_entry: None,
        deaths_at_start: 0,
        intensity: 0,
        subject: None,
        cost: PressureCost {
            deaths: 0,
            items_lost: 0,
            work_ticks_lost: 0,
        },
    }
}

/// Items lying on stockpile cells (the wolves' wealth measure).
pub fn stockpiled_items(res: &SimResources) -> u64 {
    res.colony
        .ground_items
        .iter()
        .filter(|(tile, _)| res.colony.zone_of_tile.contains_key(tile))
        .map(|(_, stack)| u64::from(stack.count))
        .sum()
}

/// Living adults (no `Child` marker), in `AgentId` order, with their keys.
fn adults(world: &World) -> Vec<(AgentId, Entity, u64, bool)> {
    let mut out: Vec<(AgentId, Entity, u64, bool)> = world
        .query::<(&Agent, Option<&Child>, Option<&Lineage>, Option<&Sick>)>()
        .iter()
        .filter(|(_, (_, child, ..))| child.is_none())
        .map(|(e, (agent, _, lineage, sick))| {
            (
                agent.id,
                e,
                lineage.map_or(agent.id, |l| l.key),
                sick.is_some(),
            )
        })
        .collect();
    out.sort_by_key(|row| row.0);
    out
}

/// Make `entity` sick until `until`; chronicle `Sick` (other = who passed it on).
fn fall_ill(
    world: &mut World,
    res: &mut SimResources,
    entity: Entity,
    id: AgentId,
    until: u64,
    from: Option<AgentId>,
    kind: PressureKind,
) {
    if world.insert_one(entity, Sick { until }).is_err() {
        return;
    }
    if Some(kind) == installed_or_bundled().pressure_role("illness_default") {
        res.content_state.illness_origins.remove(&id);
    } else {
        res.content_state.illness_origins.insert(id, kind);
    }
    let cause = illness_cause(res, kind);
    res.chronicle.push_caused(
        ChronicleEntry::new(res.current_tick, ChronicleKind::Sick)
            .subject(id)
            .other(from),
        cause,
    );
    let pos = world
        .get::<&Position>(entity)
        .map_or((0, 0), |p| (p.x, p.y));
    note_cause(res, id, pos, kind, 2);
}

/// The daily illness step: recoveries, then last night's spread between people
/// whose beds lie in one bedroom (keyed `rule.sick_spread_permille`), then the
/// person-ticks lost.
fn illness_day(world: &mut World, res: &mut SimResources) {
    let now = res.current_tick;
    let mut sick: Vec<(AgentId, Entity, u64)> = world
        .query::<(&Agent, &Sick)>()
        .iter()
        .map(|(e, (a, s))| (a.id, e, s.until))
        .collect();
    sick.sort_by_key(|row| row.0);
    for &(id, entity, until) in &sick {
        if until <= now {
            let _ = world.remove_one::<Sick>(entity);
            res.content_state.illness_origins.remove(&id);
            res.chronicle
                .push(ChronicleEntry::new(now, ChronicleKind::Recovered).subject(id));
        }
    }
    if active_mechanic(res, PressureMechanic::Illness).is_some()
        || !res.content_state.illness_origins.is_empty()
    {
        let room_of = |res: &SimResources, id: AgentId| -> Option<sim_core::rooms::RoomId> {
            let bed = res.bed_of.get(&id)?;
            let room = *res.colony.room_of_tile.get(bed)?;
            res.colony
                .rooms
                .get(&room)
                .filter(|r| r.roofed && r.role == sim_core::rooms::RoomRole::Bedroom)
                .map(|_| room)
        };
        let carriers: Vec<(AgentId, sim_core::rooms::RoomId)> = sick
            .iter()
            .filter(|(_, _, until)| *until > now)
            .filter_map(|(id, ..)| room_of(res, *id).map(|room| (*id, room)))
            .collect();
        let day = crate::runtime::resource_regen::day_index(res);
        for (id, entity, key, is_sick) in adults(world) {
            if is_sick || sick.iter().any(|(s, ..)| *s == id) {
                continue;
            }
            let Some(room) = room_of(res, id) else {
                continue;
            };
            let Some(&(from, _)) = carriers.iter().find(|(_, r)| *r == room) else {
                continue;
            };
            let Some(kind) = illness_origin(res, from) else {
                continue;
            };
            if res.colony.story.touched.contains(&(kind.code(), id, 2)) {
                continue;
            }
            let Some(rule) = rule_for(res, kind) else {
                continue;
            };
            let until = now + res.rules.calendar.days_to_ticks(u64::from(rule.sick_days));
            if keyed_below(res.world_seed, STREAM_STORY, key, day, 6, 0, 1000)
                < u64::from(rule.sick_spread_permille)
            {
                fall_ill(world, res, entity, id, until, Some(from), kind);
            }
        }
    }
    let mut counts = std::collections::BTreeMap::<PressureKind, u64>::new();
    for (_, (agent, _)) in world.query::<(Option<&Agent>, &Sick)>().iter() {
        if let Some(kind) = agent
            .and_then(|agent| illness_origin(res, agent.id))
            .or_else(|| installed_or_bundled().pressure_role("illness_default"))
        {
            *counts.entry(kind).or_default() += 1;
        }
    }
    for (kind, count) in counts {
        add_cost(res, kind, 0, count * res.ticks_per_day);
    }
}

/// Begin `p` today: its intensity, the `PressureBegan` entry, and the onset of an illness.
fn begin(world: &mut World, res: &mut SimResources, rule: &StoryRule, mut p: Pressure) {
    let day = u64::from(p.start_day);
    let adults = adults(world);
    let definition = p.kind.definition();
    let resolved = definition.resolve_rule(rule);
    match &definition.intensity {
        PressureIntensity::PackByWealth {
            base,
            per_items,
            max,
        } => {
            let wealth = stockpiled_items(res) / u64::from(per_items.story(rule).max(1));
            p.intensity =
                (u64::from(base.story(rule)) + wealth).min(u64::from(max.story(rule))) as u32;
        }
        PressureIntensity::ExtendForAdults {
            threshold,
            extra_days,
        } => {
            if adults.len() as u32 >= threshold.story(rule) {
                p.intensity = 1;
                p.end_day += extra_days;
            }
        }
        PressureIntensity::None => {}
    }
    p.deaths_at_start = deaths_so_far(res);
    let index = res.chronicle.push_caused(
        ChronicleEntry::new(res.current_tick, ChronicleKind::PressureBegan)
            .cause(p.kind.code())
            .value(i64::from(p.intensity)),
        None,
    );
    p.began_entry = Some(index);
    if p.kind.is_major() {
        res.colony.story.last_major_day = Some(p.start_day);
    }
    res.colony.story.active.push(p);
    res.colony.story.version += 1;
    if definition.mechanic == PressureMechanic::Illness {
        let healthy: Vec<_> = adults.iter().filter(|row| !row.3).collect();
        if healthy.is_empty() {
            return;
        }
        let pick = keyed_below(
            res.world_seed,
            STREAM_STORY,
            day,
            0,
            5,
            0,
            healthy.len() as u64,
        ) as usize;
        let (id, entity, ..) = *healthy[pick];
        let until = res.current_tick
            + res
                .rules
                .calendar
                .days_to_ticks(u64::from(resolved.sick_days));
        if let Some(active) = res
            .colony
            .story
            .active
            .iter_mut()
            .find(|a| a.kind == p.kind)
        {
            active.subject = Some(id);
        }
        fall_ill(world, res, entity, id, until, None, p.kind);
    }
}

/// Begin a pressure of `kind` today for `days` days, outside the plan (tests, the
/// probe's forced pressures, and the F3 fixture); a kind already in force is left alone.
pub fn force(world: &mut World, res: &mut SimResources, kind: PressureKind, days: u32) {
    let _ = try_force(world, res, kind, days);
}

/// Rejected pressure admission, before any state or keyed draw is changed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForceError {
    /// Another identity already owns the requested mechanic.
    MechanicConflict {
        /// Identity already owning this mechanic.
        active: PressureKind,
        /// Identity whose admission was rejected.
        requested: PressureKind,
    },
}
impl std::fmt::Display for ForceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MechanicConflict { active, requested } => write!(
                f,
                "pressure {} conflicts with active {}",
                requested.code(),
                active.code()
            ),
        }
    }
}
impl std::error::Error for ForceError {}

/// Force a registered pressure, rejecting a distinct active identity of the same mechanic.
/// An already active exact identity and worlds without story retain their legacy no-op behavior.
pub fn try_force(
    world: &mut World,
    res: &mut SimResources,
    kind: PressureKind,
    days: u32,
) -> Result<(), ForceError> {
    let Some(rule) = rule(res) else {
        return Ok(());
    };
    if res.colony.story.is_active(kind) {
        return Ok(());
    }
    if let Some(active) = active_mechanic(res, kind.definition().mechanic) {
        return Err(ForceError::MechanicConflict {
            active: active.kind,
            requested: kind,
        });
    }
    let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    begin(world, res, &rule, planned(kind, day, days));
    Ok(())
}

/// Whether pressure `kind` may not begin on `day` because pressures do not stack (M5.5, `StoryRule::no_stack`):
/// an illness while a major is active or within `stack_gap_days` after one ended (the end day is exclusive: a major
/// ending on day E lets an illness begin on E + `stack_gap_days`); a major while `sick` ≥ `major_wait_sick`.
pub fn stack_blocked(
    rule: &StoryRule,
    story: &Storyteller,
    kind: PressureKind,
    day: u32,
    sick: u32,
) -> bool {
    if !rule.no_stack {
        return false;
    }
    if kind.is_major() {
        return sick >= rule.major_wait_sick;
    }
    story.active.iter().any(|p| p.kind.is_major())
        || story
            .history
            .iter()
            .any(|p| p.kind.is_major() && day < p.end_day.saturating_add(rule.stack_gap_days))
}

/// The last day of its season on which held-back pressure `p` may still begin (M5.5): an illness until the season's
/// last day, a major until its window's last day (mercy rules: winter's last days for beasts).
fn last_wait_day(rule: &StoryRule, p: &Pressure, per_season: u32, per_year: u32) -> u32 {
    let index = p.start_day / per_season.max(1);
    let first_day = index * per_season;
    if !p.kind.is_major() {
        return first_day + per_season.saturating_sub(1);
    }
    // The window's last day (`window`); a held-back major only moves later, so its first day does not matter.
    let mut last = first_day + per_season.saturating_sub(2).max(1);
    if index % per_year.max(1) == WINTER {
        if let Some(value) = &p.kind.definition().mercy.winter_end_days {
            last = last.min(first_day + per_season.saturating_sub(1 + value.story(rule)));
        }
    }
    last
}

/// What to do with planned pressure `p` on `day` (M5.5): `None` = begin it; `Some(Some(next))` = it waits for `next`;
/// `Some(None)` = it cannot begin this season (dropped).
pub fn held_back(
    rule: &StoryRule,
    story: &Storyteller,
    p: &Pressure,
    day: u32,
    sick: u32,
    per_season: u32,
    per_year: u32,
) -> Option<Option<Pressure>> {
    if !stack_blocked(rule, story, p.kind, day, sick) {
        return None;
    }
    let next = day + 1;
    if next > last_wait_day(rule, p, per_season, per_year) {
        return Some(None);
    }
    let days = p.end_day.saturating_sub(p.start_day);
    Some(Some(Pressure {
        start_day: next,
        end_day: next + days,
        ..*p
    }))
}

/// The storyteller's schedule alone, without a world: every pressure that
/// would begin in `days` days, planned season by season as the system does
/// (every pressure lasts its planned days; beasts their longest stay). Pure in
/// `(seed, rule, days, days_per_season)`; the F1 check and the tests use it.
pub fn simulate_schedule(seed: u64, rule: &StoryRule, days: u32, per_season: u32) -> Vec<Pressure> {
    let mut story = Storyteller::default();
    for day in 0..days {
        let index = day / per_season.max(1);
        if index >= story.seasons_planned {
            story.seasons_planned = index + 1;
            let slot = SeasonSlot {
                index,
                season: index % 4,
                year: index / 4,
                first_day: index * per_season,
                days: per_season,
            };
            let (major, minor, skipped) = plan_season(seed, rule, &story, slot);
            story.skipped += u32::from(skipped);
            story.planned.extend(major);
            story.planned.extend(minor);
            story.planned.sort_by_key(|p| (p.start_day, p.kind));
        }
        let over: Vec<usize> = (0..story.active.len())
            .filter(|&i| story.active[i].end_day <= day)
            .collect();
        for i in over.into_iter().rev() {
            let done = story.active.remove(i);
            story.history.push(done);
        }
        let today: Vec<Pressure> = story
            .planned
            .iter()
            .filter(|p| p.start_day <= day)
            .copied()
            .collect();
        story.planned.retain(|p| p.start_day > day);
        for p in today {
            if story.is_active(p.kind) {
                story.skipped += 1;
                continue;
            }
            // M5.5: no stacking (nobody is sick in a schedule without a world).
            match held_back(rule, &story, &p, day, 0, per_season, 4) {
                Some(Some(later)) => {
                    story.planned.push(later);
                    story.planned.sort_by_key(|q| (q.start_day, q.kind));
                    continue;
                }
                Some(None) => {
                    story.skipped += 1;
                    continue;
                }
                None => {}
            }
            if p.kind.is_major() {
                story.last_major_day = Some(p.start_day);
            }
            story.active.push(p);
        }
    }
    let mut all: Vec<Pressure> = story.begun().copied().collect();
    all.sort_by_key(|p| (p.start_day, p.kind));
    all
}

/// See the module header.
#[derive(Debug, Default)]
pub struct StorytellerSystem;

impl RuntimeSystem for StorytellerSystem {
    fn name(&self) -> &str {
        "StorytellerSystem"
    }

    fn priority(&self) -> u32 {
        STORYTELLER_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1_440
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = rule(res) else {
            return;
        };
        let day = u32::try_from(crate::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
        if day < res.colony.story.next_check_day {
            return;
        }
        res.colony.story.next_check_day = day + 1;
        // Plan the season on its first day.
        let per_season = res.rules.season_days().max(1);
        let per_year = res.rules.calendar.seasons_per_year.max(1);
        let index = day / per_season;
        if index >= res.colony.story.seasons_planned && res.rules.seasons.is_some() {
            res.colony.story.seasons_planned = index + 1;
            let slot = SeasonSlot {
                index,
                season: index % per_year,
                year: index / per_year,
                first_day: index * per_season,
                days: per_season,
            };
            let (major, minor, skipped) =
                plan_season(res.world_seed, &rule, &res.colony.story, slot);
            if skipped {
                res.colony.story.skipped += 1;
            }
            res.colony.story.planned.extend(major);
            res.colony.story.planned.extend(minor);
            res.colony
                .story
                .planned
                .sort_by_key(|p| (p.start_day, p.kind));
        }
        // End what is over (beasts after their longest stay).
        let over: Vec<PressureKind> = res
            .colony
            .story
            .active
            .iter()
            .filter(|p| p.end_day <= day)
            .map(|p| p.kind)
            .collect();
        for kind in over {
            if kind.definition().mechanic == PressureMechanic::Illness
                && world
                    .query::<(Option<&Agent>, &Sick)>()
                    .iter()
                    .any(|(_, (a, _))| {
                        a.and_then(|a| illness_origin(res, a.id))
                            .or_else(|| installed_or_bundled().pressure_role("illness_default"))
                            == Some(kind)
                    })
            {
                continue;
            }
            end_pressure(res, kind);
        }
        // Begin what is due today; a kind already in force waits (dropped as skipped).
        let today: Vec<Pressure> = res
            .colony
            .story
            .planned
            .iter()
            .filter(|p| p.start_day <= day)
            .copied()
            .collect();
        res.colony.story.planned.retain(|p| p.start_day > day);
        let sick = world.query::<&Sick>().iter().count() as u32;
        for p in today {
            if active_mechanic(res, p.kind.definition().mechanic).is_some() {
                res.colony.story.skipped += 1;
                continue;
            }
            // M5.5: pressures do not stack (a held-back pressure waits a day, or is dropped).
            match held_back(
                &rule,
                &res.colony.story,
                &p,
                day,
                sick,
                per_season,
                per_year,
            ) {
                Some(Some(later)) => {
                    let story = &mut res.colony.story;
                    story.planned.push(later);
                    story.planned.sort_by_key(|q| (q.start_day, q.kind));
                    continue;
                }
                Some(None) => {
                    res.colony.story.skipped += 1;
                    continue;
                }
                None => {}
            }
            begin(world, res, &rule, p);
        }
        if rule.illness || !res.content_state.illness_origins.is_empty() {
            illness_day(world, res);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn slot(index: u32) -> SeasonSlot {
        SeasonSlot {
            index,
            season: index % 4,
            year: index / 4,
            first_day: index * 6,
            days: 6,
        }
    }

    #[test]
    fn bundled_illness_keeps_sparse_provenance_empty_and_honors_world_overrides() {
        // Given a world with intentionally non-default illness timing and work cost.
        let mut engine =
            sim_engine::SimEngine::new(8, 8, sim_core::material::MaterialRegistry::new());
        let mut colony = sim_core::rules::ColonyRule::M5;
        let mut story = StoryRule::M5;
        story.sick_days = 7;
        story.sick_work_permille = 1730;
        colony.story = Some(story);
        engine.resources.rules.colony = Some(colony);
        let entity = engine.spawn_agent(1, 1);
        // When the existing public force boundary begins the bundled illness.
        force(
            &mut engine.world,
            &mut engine.resources,
            PressureKind::Illness,
            2,
        );
        // Then old per-world tuning remains authoritative without new sparse state.
        assert!(engine.resources.content_state.illness_origins.is_empty());
        assert_eq!(
            engine.world.get::<&Sick>(entity).expect("sick adult").until,
            engine.resources.rules.calendar.days_to_ticks(7)
        );
        assert_eq!(
            work_permille(&engine.world, &engine.resources, entity, (1, 1)),
            1730
        );
    }

    #[test]
    fn forcing_an_active_exact_identity_does_not_restart_it() {
        // Given an active pressure and its established history reference.
        let mut engine =
            sim_engine::SimEngine::new(8, 8, sim_core::material::MaterialRegistry::new());
        engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M5);
        force(
            &mut engine.world,
            &mut engine.resources,
            PressureKind::Drought,
            2,
        );
        let before = engine.resources.colony.story.clone();
        // When the generic checked boundary receives that same identity again.
        let result = try_force(
            &mut engine.world,
            &mut engine.resources,
            PressureKind::Drought,
            99,
        );
        // Then it retains the original timing, cost, and entry.
        assert_eq!(result, Ok(()));
        assert_eq!(engine.resources.colony.story, before);
    }

    #[test]
    fn metadata() {
        let s = StorytellerSystem;
        assert_eq!(
            (s.name(), s.priority(), s.tick_interval()),
            ("StorytellerSystem", 170, 1_440)
        );
    }

    #[test]
    fn the_first_spring_and_summer_are_quiet_and_the_mercy_days_hold() {
        let rule = StoryRule::M5;
        let story = Storyteller::default();
        for seed in 0..64 {
            for index in [0, 1] {
                let (major, minor, _) = plan_season(seed, &rule, &story, slot(index));
                assert!(
                    major.is_none() && minor.is_none(),
                    "seed {seed} season {index}"
                );
            }
            // The first winter: no cold snap on its first two days (18, 19); illness only from day 18.
            if let (Some(p), _, _) = plan_season(seed, &rule, &story, slot(3)) {
                assert!(p.start_day >= 19 && p.start_day <= 22, "seed {seed}: {p:?}");
                if p.kind == PressureKind::ColdSnap {
                    assert!(p.start_day >= 20);
                    assert!((2..=4).contains(&(p.end_day - p.start_day)));
                }
            }
        }
    }

    #[test]
    fn a_draw_inside_the_major_gap_moves_or_drops() {
        let rule = StoryRule::M5;
        let story = Storyteller {
            last_major_day: Some(40),
            ..Storyteller::default()
        };
        // Winter of year 1 starts at day 42; the gap allows day 46 only.
        for seed in 0..32 {
            if let (Some(p), _, _) = plan_season(seed, &rule, &story, slot(7)) {
                assert_eq!(p.start_day, 46, "seed {seed}");
            }
        }
    }
}
