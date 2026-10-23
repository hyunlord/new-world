//! M5 bridge (ADR 0027–0030): the storyteller's pressures with the firewood
//! line, the animals and hunt zones, the M5 per-person columns of the agent
//! snapshot, one person's mood, apparel, draft and illness, the drafted list,
//! a chronicle entry's "→ 왜?" chain and the year summary; plus the draft,
//! hunt and dressing orders.
//!
//! Reads are pure collectors (`collect_*`), rows in a documented order:
//! [`collect_story`] (pressures in begin order), [`collect_animals`] (animals by
//! `AnimalId`, zones in the order they were added), [`collect_agent_m5_columns`]
//! (the agent snapshot's row order), [`collect_agent_m5`] (modifiers by size,
//! then code), [`collect_drafted`] (by `AgentId`), [`collect_chronicle_why`]
//! (the chain from the entry to its first cause), [`collect_year_summary`].
//! Ids: people are `AgentId` unless named entity bits, animals `AnimalId`,
//! chronicle entries their index. Every read returns its empty form in the
//! baseline world and under rules without M5.
use godot::prelude::*;
use sim_core::chronicle::{CauseRef, Chronicle, ChronicleKind};
use sim_core::colony::people::{Apparel, DraftOrder, Drafted, Modifier, Mood, Sick};
use sim_core::colony::story::PressureKind;
use sim_core::colony::wildlife::{AnimalKind, Fighter};
use sim_core::colony::{Command, CommandResult, RejectReason, Tool};
use sim_core::components::{Agent, AgentId, BodyHealth, Position};
use sim_engine::{SimEngine, SimResources};
use sim_systems::commands::submit;
use sim_systems::runtime::mood::{self, Link};
use sim_systems::runtime::story;

/// Mood band codes of [`AgentM5View::band`] and the snapshot's `mood_bands`.
pub mod band {
    /// No mood (a child under 10, or not an M5 world).
    pub const NONE: u8 = 0;
    /// Below `low` (250): 우울, tantrum risk.
    pub const LOW: u8 = 1;
    /// Between: 보통.
    pub const NORMAL: u8 = 2;
    /// At or above `high` (750): 흡족.
    pub const CONTENT: u8 = 3;
    /// In a breakdown.
    pub const BREAKDOWN: u8 = 4;
}

/// One pressure in force.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PressureRow {
    /// `PressureKind::code` (0 cold snap, 1 drought, 2 beasts, 3 illness).
    pub kind: u8,
    /// `PressureKind::name`.
    pub name: String,
    /// Display day it began.
    pub start_day: u32,
    /// Display day it ends (exclusive).
    pub end_day: u32,
    /// Which day of it today is (1 = its first day).
    pub day_of: u32,
    /// Its intensity (wolves; extra days).
    pub intensity: u32,
    /// The person it struck (illness), else −1.
    pub subject: i64,
    /// Deaths while it has been in force.
    pub deaths: u64,
}

/// The storyteller and the pressure banner (`get_story`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StoryView {
    /// Whether the world has a storyteller.
    pub on: bool,
    /// Pressures in force, in begin order.
    pub active: Vec<PressureRow>,
    /// Pressures that ended so far.
    pub ended: u32,
    /// Firewood the hearth can burn now (settlement wood + feedable firewood).
    pub firewood: u64,
    /// One night's burn at a cold snap's rate (the banner's "하루 N 필요").
    pub firewood_day_need: u64,
    /// Wolves on the map.
    pub wolves: u32,
    /// The present pack's size (0 when no pack came).
    pub pack_size: u32,
    /// The mean tile of the wolves on the map (−1 without wolves): the banner's pulse.
    pub wolves_x: i32,
    /// See `wolves_x`.
    pub wolves_y: i32,
    /// People sick now.
    pub sick: u32,
    /// Whether the standing order "모두 가죽 옷" is on.
    pub dress_order: bool,
    /// `Storyteller::version`-like change counter: pressures begun + ended.
    pub version: u64,
}

/// The storyteller, the pressures in force and the banner's numbers.
pub fn collect_story(world: &hecs::World, res: &SimResources) -> StoryView {
    let Some(_) = story::rule(res) else {
        return StoryView {
            wolves_x: -1,
            wolves_y: -1,
            dress_order: !res.colony.dress_off,
            ..StoryView::default()
        };
    };
    let today =
        u32::try_from(sim_systems::runtime::resource_regen::day_index(res)).unwrap_or(u32::MAX);
    let s = &res.colony.story;
    let deaths = story::deaths_so_far(res);
    let active = s
        .active
        .iter()
        .map(|p| PressureRow {
            kind: p.kind.code(),
            name: p.kind.name().into(),
            start_day: p.start_day,
            end_day: p.end_day,
            day_of: today.saturating_sub(p.start_day) + 1,
            intensity: p.intensity,
            subject: p.subject.map_or(-1, |a| i64::try_from(a).unwrap_or(-1)),
            deaths: deaths.saturating_sub(p.deaths_at_start),
        })
        .collect();
    let w = &res.colony.wildlife;
    let wolves: Vec<(u32, u32)> = w.wolves().map(|a| a.pos).collect();
    let n = wolves.len() as u64;
    let (wx, wy) = if n == 0 {
        (-1, -1)
    } else {
        let sx: u64 = wolves.iter().map(|p| u64::from(p.0)).sum();
        let sy: u64 = wolves.iter().map(|p| u64::from(p.1)).sum();
        (
            i32::try_from(sx / n).unwrap_or(-1),
            i32::try_from(sy / n).unwrap_or(-1),
        )
    };
    StoryView {
        on: true,
        active,
        ended: s.history.len() as u32,
        firewood: story::firewood_have(res),
        firewood_day_need: story::firewood_day_need(res),
        wolves: n as u32,
        pack_size: if w.pack_day.is_some() { w.pack_size } else { 0 },
        wolves_x: wx,
        wolves_y: wy,
        sick: world.query::<&Sick>().iter().count() as u32,
        dress_order: !res.colony.dress_off,
        version: (s.history.len() + s.active.len()) as u64,
    }
}

/// Marshal [`StoryView`] (pressures as parallel columns).
pub(super) fn story_to_dict(v: &StoryView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("on", v.on);
    let kinds: Vec<i32> = v.active.iter().map(|p| i32::from(p.kind)).collect();
    let names: Vec<GString> = v
        .active
        .iter()
        .map(|p| GString::from(p.name.as_str()))
        .collect();
    let col = |f: &dyn Fn(&PressureRow) -> i64| -> PackedInt64Array {
        v.active
            .iter()
            .map(f)
            .collect::<Vec<i64>>()
            .as_slice()
            .into()
    };
    d.set("kinds", PackedInt32Array::from(kinds.as_slice()));
    d.set("names", PackedStringArray::from(names.as_slice()));
    d.set("start_days", col(&|p| i64::from(p.start_day)));
    d.set("end_days", col(&|p| i64::from(p.end_day)));
    d.set("days_of", col(&|p| i64::from(p.day_of)));
    d.set("intensities", col(&|p| i64::from(p.intensity)));
    d.set("subjects", col(&|p| p.subject));
    d.set(
        "deaths",
        col(&|p| i64::try_from(p.deaths).unwrap_or(i64::MAX)),
    );
    d.set("ended", i64::from(v.ended));
    d.set("firewood", i64::try_from(v.firewood).unwrap_or(i64::MAX));
    d.set(
        "firewood_day_need",
        i64::try_from(v.firewood_day_need).unwrap_or(i64::MAX),
    );
    d.set("wolves", i64::from(v.wolves));
    d.set("pack_size", i64::from(v.pack_size));
    d.set("wolves_x", v.wolves_x);
    d.set("wolves_y", v.wolves_y);
    d.set("sick", i64::from(v.sick));
    d.set("dress_order", v.dress_order);
    d.set("version", i64::try_from(v.version).unwrap_or(i64::MAX));
    d
}

/// The animals on the map and the hunt zones (`get_animals`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AnimalRows {
    /// `AnimalId`s, ascending.
    pub id: Vec<i64>,
    /// `AnimalKind::code` (0 wolf, 1 deer).
    pub kind: Vec<i32>,
    /// Tile x.
    pub x: Vec<i32>,
    /// Tile y.
    pub y: Vec<i32>,
    /// Hit points.
    pub hp: Vec<i32>,
    /// `AnimalState::code` (0 wander, 1 flee, 2 rest, 3 hunt, 4 attack, 5 retreat).
    pub state: Vec<i32>,
    /// 1 when the deer may be hunted (a hunt mark or inside a hunt zone).
    pub huntable: Vec<u8>,
    /// 1 when the deer carries a hunt mark of its own.
    pub marked: Vec<u8>,
    /// Hunt zones `[x0, y0, x1, y1]` flattened, in the order they were added.
    pub zones: Vec<i32>,
    /// `Wildlife::version` (cheap change check).
    pub version: u64,
}

/// Every animal by id and every hunt zone.
pub fn collect_animals(res: &SimResources) -> AnimalRows {
    let w = &res.colony.wildlife;
    let mut out = AnimalRows {
        version: w.version,
        ..AnimalRows::default()
    };
    for a in w.animals.values() {
        out.id.push(i64::from(a.id));
        out.kind.push(i32::from(a.kind.code()));
        out.x.push(i32::try_from(a.pos.0).unwrap_or(-1));
        out.y.push(i32::try_from(a.pos.1).unwrap_or(-1));
        out.hp.push(i32::from(a.hp));
        out.state.push(i32::from(a.state.code()));
        out.huntable.push(u8::from(w.huntable(a)));
        out.marked.push(u8::from(w.marks.contains(&a.id)));
    }
    for &(x0, y0, x1, y1) in &w.zones {
        for v in [x0, y0, x1, y1] {
            out.zones.push(i32::try_from(v).unwrap_or(-1));
        }
    }
    out
}

/// Marshal [`AnimalRows`].
pub(super) fn animals_to_dict(r: &AnimalRows) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("ids", PackedInt64Array::from(r.id.as_slice()));
    d.set("kinds", PackedInt32Array::from(r.kind.as_slice()));
    d.set("xs", PackedInt32Array::from(r.x.as_slice()));
    d.set("ys", PackedInt32Array::from(r.y.as_slice()));
    d.set("hps", PackedInt32Array::from(r.hp.as_slice()));
    d.set("states", PackedInt32Array::from(r.state.as_slice()));
    d.set("huntable", PackedByteArray::from(r.huntable.as_slice()));
    d.set("marked", PackedByteArray::from(r.marked.as_slice()));
    d.set("zones", PackedInt32Array::from(r.zones.as_slice()));
    d.set("version", i64::try_from(r.version).unwrap_or(i64::MAX));
    d
}

/// The mood band of a person (`band::*`).
fn band_of(res: &SimResources, mood: Option<&Mood>) -> u8 {
    let (Some(rule), Some(m)) = (mood::rule(res), mood) else {
        return band::NONE;
    };
    if m.broken(mood::today(res)) {
        band::BREAKDOWN
    } else if m.value < rule.low {
        band::LOW
    } else if m.value >= rule.high {
        band::CONTENT
    } else {
        band::NORMAL
    }
}

/// The M5 per-person columns of the agent snapshot, in its row order (the
/// `(Agent, Position)` archetype iteration, as `collect_agent_colds`):
/// `(top code, mood band, drafted, sick)` — top 0 loincloth, 1 hide wrap, 2 fur
/// cloak; band `band::*`; drafted and sick 0/1.
pub fn collect_agent_m5_columns(world: &hecs::World, res: &SimResources) -> Vec<(u8, u8, u8, u8)> {
    world
        .query::<(
            &Agent,
            &Position,
            Option<&Apparel>,
            Option<&Mood>,
            Option<&Drafted>,
            Option<&Sick>,
        )>()
        .iter()
        .map(|(_, (_, _, apparel, m, drafted, sick))| {
            (
                apparel.map_or(0, |a| a.top.code()),
                band_of(res, m),
                u8::from(drafted.is_some()),
                u8::from(sick.is_some()),
            )
        })
        .collect()
}

/// One person's M5 state (`get_agent_m5`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentM5View {
    /// Whether the person is alive.
    pub found: bool,
    /// Mood 0..=1000 (−1 without a mood).
    pub mood: i32,
    /// `band::*`.
    pub band: u8,
    /// The mood's target now (−1 without a mood).
    pub target: i32,
    /// Modifiers in force: (`Modifier::code`, name, weight after temperament),
    /// largest magnitude first, then code (the inspector shows the first three).
    pub modifiers: Vec<(u8, String, i32)>,
    /// Top code (0 loincloth, 1 hide wrap, 2 fur cloak).
    pub top: u8,
    /// The top's cold factor ‰ (1,000 for the loincloth).
    pub top_cold_permille: u32,
    /// Display day the top was put on (−1 without one).
    pub top_since: i64,
    /// Whether drafted.
    pub drafted: bool,
    /// The draft order: 0 hold, 1 move, 2 attack an animal, 3 attack a person (−1 not drafted).
    pub draft_order: i32,
    /// The order's tile (move) or target id (attack), else −1.
    pub draft_x: i64,
    /// See `draft_x`: the move's y, else −1.
    pub draft_y: i64,
    /// Whether sick, and the ticks left (0 when not).
    pub sick: bool,
    /// Ticks until the illness passes.
    pub sick_ticks_left: i64,
    /// Hit points (0..=100).
    pub hp: i32,
    /// M5.5: running from a wolf.
    pub fleeing: bool,
}

/// Person `agent_id`'s mood, modifiers, apparel, draft and illness.
pub fn collect_agent_m5(world: &hecs::World, res: &SimResources, agent_id: i64) -> AgentM5View {
    let Ok(id) = AgentId::try_from(agent_id) else {
        return AgentM5View::default();
    };
    let Some((entity, _)) = world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == id)
        .map(|(e, a)| (e, a.id))
    else {
        return AgentM5View {
            mood: -1,
            target: -1,
            draft_order: -1,
            draft_x: -1,
            draft_y: -1,
            top_since: -1,
            ..AgentM5View::default()
        };
    };
    let m = world.get::<&Mood>(entity).ok().map(|m| *m);
    let (target, mut modifiers) = match (mood::rule(res), m) {
        (Some(rule), Some(_)) => {
            let facts = mood::facts_of(world, res, entity);
            let mods = mood::modifiers(&facts);
            (i32::from(mood::target(&rule, &mods)), mods)
        }
        _ => (-1, Vec::new()),
    };
    modifiers.sort_by_key(|(m, v)| (std::cmp::Reverse(v.abs()), m.code()));
    let apparel = world.get::<&Apparel>(entity).ok().map(|a| *a);
    let drafted = world.get::<&Drafted>(entity).ok().map(|d| *d);
    let (order, dx, dy) = match drafted.map(|d| d.order) {
        None => (-1, -1, -1),
        Some(DraftOrder::Hold) => (0, -1, -1),
        Some(DraftOrder::MoveTo((x, y))) => (1, i64::from(x), i64::from(y)),
        Some(DraftOrder::Attack(Fighter::Animal(a))) => (2, i64::from(a), -1),
        Some(DraftOrder::Attack(Fighter::Person(p))) => (3, i64::try_from(p).unwrap_or(-1), -1),
    };
    let sick = world.get::<&Sick>(entity).ok().map(|s| s.until);
    AgentM5View {
        found: true,
        mood: m.map_or(-1, |m| i32::from(m.value)),
        band: band_of(res, m.as_ref()),
        target,
        modifiers: modifiers
            .into_iter()
            .map(|(m, v)| (m.code(), m.name().to_string(), v))
            .collect(),
        top: apparel.map_or(0, |a| a.top.code()),
        top_cold_permille: apparel.map_or(1000, |a| a.top.cold_permille()),
        top_since: apparel.map_or(-1, |a| i64::from(a.since_day)),
        drafted: drafted.is_some(),
        draft_order: order,
        draft_x: dx,
        draft_y: dy,
        sick: sick.is_some(),
        sick_ticks_left: sick.map_or(0, |u| {
            i64::try_from(u.saturating_sub(res.current_tick)).unwrap_or(i64::MAX)
        }),
        hp: world
            .get::<&BodyHealth>(entity)
            .map_or(0, |h| h.hp.clamp(0.0, 100.0) as i32),
        fleeing: world
            .get::<&sim_core::colony::people::Fleeing>(entity)
            .is_ok(),
    }
}

/// Marshal [`AgentM5View`] (modifiers as parallel columns).
pub(super) fn agent_m5_to_dict(v: &AgentM5View) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", v.found);
    d.set("mood", v.mood);
    d.set("band", i64::from(v.band));
    d.set("target", v.target);
    let codes: Vec<i32> = v.modifiers.iter().map(|m| i32::from(m.0)).collect();
    let names: Vec<GString> = v
        .modifiers
        .iter()
        .map(|m| GString::from(m.1.as_str()))
        .collect();
    let values: Vec<i32> = v.modifiers.iter().map(|m| m.2).collect();
    d.set("modifier_codes", PackedInt32Array::from(codes.as_slice()));
    d.set("modifier_names", PackedStringArray::from(names.as_slice()));
    d.set("modifier_values", PackedInt32Array::from(values.as_slice()));
    d.set("top", i64::from(v.top));
    d.set("top_cold_permille", i64::from(v.top_cold_permille));
    d.set("top_since", v.top_since);
    d.set("drafted", v.drafted);
    d.set("draft_order", v.draft_order);
    d.set("draft_x", v.draft_x);
    d.set("draft_y", v.draft_y);
    d.set("sick", v.sick);
    d.set("fleeing", v.fleeing);
    d.set("sick_ticks_left", v.sick_ticks_left);
    d.set("hp", v.hp);
    d
}

/// The drafted people (`get_drafted`), by `AgentId`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DraftedRows {
    /// `AgentId`s.
    pub id: Vec<i64>,
    /// Tile x.
    pub x: Vec<i32>,
    /// Tile y.
    pub y: Vec<i32>,
    /// Hit points (0..=100).
    pub hp: Vec<i32>,
    /// The order: 0 hold, 1 move, 2 attack an animal, 3 attack a person.
    pub order: Vec<i32>,
    /// The held tool's `ItemKind::code` (−1 bare hands).
    pub weapon: Vec<i32>,
    /// Ticks since drafted.
    pub ticks: Vec<i64>,
}

/// One drafted person: id, tile, hp, order code, weapon code, ticks drafted.
type DraftedRow = (AgentId, (i32, i32), i32, i32, i32, i64);

/// Every drafted person, by `AgentId`.
pub fn collect_drafted(world: &hecs::World, res: &SimResources) -> DraftedRows {
    let mut rows: Vec<DraftedRow> = world
        .query::<(
            &Agent,
            &Position,
            &Drafted,
            Option<&BodyHealth>,
            Option<&Tool>,
        )>()
        .iter()
        .map(|(_, (a, p, d, h, t))| {
            let order = match d.order {
                DraftOrder::Hold => 0,
                DraftOrder::MoveTo(_) => 1,
                DraftOrder::Attack(Fighter::Animal(_)) => 2,
                DraftOrder::Attack(Fighter::Person(_)) => 3,
            };
            (
                a.id,
                (
                    i32::try_from(p.x).unwrap_or(-1),
                    i32::try_from(p.y).unwrap_or(-1),
                ),
                h.map_or(0, |h| h.hp.clamp(0.0, 100.0) as i32),
                order,
                t.map_or(-1, |t| i32::from(t.item.kind.code())),
                i64::try_from(res.current_tick.saturating_sub(d.since)).unwrap_or(i64::MAX),
            )
        })
        .collect();
    rows.sort_by_key(|r| r.0);
    let mut out = DraftedRows::default();
    for (id, (x, y), hp, order, weapon, ticks) in rows {
        out.id.push(i64::try_from(id).unwrap_or(-1));
        out.x.push(x);
        out.y.push(y);
        out.hp.push(hp);
        out.order.push(order);
        out.weapon.push(weapon);
        out.ticks.push(ticks);
    }
    out
}

/// Marshal [`DraftedRows`].
pub(super) fn drafted_to_dict(r: &DraftedRows) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("ids", PackedInt64Array::from(r.id.as_slice()));
    d.set("xs", PackedInt32Array::from(r.x.as_slice()));
    d.set("ys", PackedInt32Array::from(r.y.as_slice()));
    d.set("hps", PackedInt32Array::from(r.hp.as_slice()));
    d.set("orders", PackedInt32Array::from(r.order.as_slice()));
    d.set("weapons", PackedInt32Array::from(r.weapon.as_slice()));
    d.set("ticks", PackedInt64Array::from(r.ticks.as_slice()));
    d
}

/// A chronicle entry's "→ 왜?" chain (`get_chronicle_why`): one row per link,
/// from the entry itself to its first cause (at most 8 links).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WhyRows {
    /// Whether the entry exists.
    pub found: bool,
    /// Link kind: 0 a chronicle entry, 1 a mood modifier, 2 a pressure.
    pub link: Vec<i32>,
    /// The entry index (link 0), `Modifier::code` (1, 100 = the relation) or `PressureKind::code` (2).
    pub code: Vec<i64>,
    /// For an entry link: its tick, else −1.
    pub tick: Vec<i64>,
    /// For an entry link: `ChronicleKind::as_u8`, else −1.
    pub kind: Vec<i32>,
    /// For an entry link: subject `AgentId`, else −1.
    pub subject: Vec<i64>,
    /// For an entry link: other `AgentId`, else −1.
    pub other: Vec<i64>,
    /// For an entry link: its cause code, else −1.
    pub cause: Vec<i32>,
    /// For an entry link: its value, else 0.
    pub value: Vec<i64>,
}

/// The "why?" chain of chronicle entry `index` (`mood::why_chain`).
pub fn collect_chronicle_why(chronicle: &Chronicle, index: i64) -> WhyRows {
    let Ok(i) = u32::try_from(index) else {
        return WhyRows::default();
    };
    if chronicle.entry(i).is_none() {
        return WhyRows::default();
    }
    let mut out = WhyRows {
        found: true,
        ..WhyRows::default()
    };
    for link in mood::why_chain(chronicle, i) {
        let (kind, code, entry) = match link {
            Link::Entry(e) => (0, i64::from(e), chronicle.entry(e)),
            Link::Modifier(c) => (1, i64::from(c), None),
            Link::Pressure(c) => (2, i64::from(c), None),
        };
        out.link.push(kind);
        out.code.push(code);
        out.tick
            .push(entry.map_or(-1, |e| i64::try_from(e.tick).unwrap_or(-1)));
        out.kind
            .push(entry.map_or(-1, |e| i32::from(e.kind.as_u8())));
        out.subject.push(
            entry
                .and_then(|e| e.subject)
                .map_or(-1, |a| i64::try_from(a).unwrap_or(-1)),
        );
        out.other.push(
            entry
                .and_then(|e| e.other)
                .map_or(-1, |a| i64::try_from(a).unwrap_or(-1)),
        );
        out.cause.push(entry.map_or(-1, |e| i32::from(e.cause)));
        out.value.push(entry.map_or(0, |e| e.value));
    }
    out
}

/// Marshal [`WhyRows`].
pub(super) fn why_to_dict(r: &WhyRows) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", r.found);
    d.set("links", PackedInt32Array::from(r.link.as_slice()));
    d.set("codes", PackedInt64Array::from(r.code.as_slice()));
    d.set("ticks", PackedInt64Array::from(r.tick.as_slice()));
    d.set("kinds", PackedInt32Array::from(r.kind.as_slice()));
    d.set("subjects", PackedInt64Array::from(r.subject.as_slice()));
    d.set("others", PackedInt64Array::from(r.other.as_slice()));
    d.set("causes", PackedInt32Array::from(r.cause.as_slice()));
    d.set("values", PackedInt64Array::from(r.value.as_slice()));
    d
}

/// The link of entry `index` for the chronicle panel's "→" line: (link kind −1
/// none / 0 entry / 1 modifier / 2 pressure, the linked entry index or −1, the
/// modifier or pressure code or −1).
pub fn cause_link(chronicle: &Chronicle, index: u32) -> (i32, i64, i32) {
    match chronicle.cause_of(index) {
        None => (-1, -1, -1),
        Some(CauseRef::Entry(e)) => (0, i64::from(e), -1),
        Some(CauseRef::Modifier { code, entry }) => {
            (1, entry.map_or(-1, i64::from), i32::from(code))
        }
        Some(CauseRef::Pressure { kind, entry }) => {
            (2, entry.map_or(-1, i64::from), i32::from(kind))
        }
        // M5.5: a sampled need reads as its modifier, linked to the pressure behind it.
        Some(CauseRef::Sample { code, sample }) => (
            1,
            chronicle
                .sample(sample)
                .and_then(|s| s.entry)
                .map_or(-1, i64::from),
            i32::from(code),
        ),
    }
}

/// One year's summary (`get_year_summary`; goal 3.4, droppable 9.5).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct YearSummary {
    /// The year asked (0 = the first), valid when `found`.
    pub year: u32,
    /// Whether the year has begun.
    pub found: bool,
    /// Whether the year is over.
    pub complete: bool,
    /// Births.
    pub births: u32,
    /// Deaths by `DeathReason::as_u8` (0 starvation, 1 dehydration, 2 combat, 3 old age, 4 exposure).
    pub deaths: [u32; 5],
    /// Pressures begun by `PressureKind::code` (0 cold snap, 1 drought, 2 beasts, 3 illness).
    pub pressures: [u32; 4],
    /// Knowledge discovered.
    pub discoveries: u32,
    /// Knowledge lost.
    pub lost: u32,
    /// Quarrels.
    pub quarrels: u32,
    /// Fights after a quarrel.
    pub fights: u32,
    /// Breakdowns.
    pub breakdowns: u32,
    /// Deer hunted.
    pub hunted: u32,
    /// Wolves killed.
    pub wolves_killed: u32,
    /// Era progress ‰ now (the tech aggregate's).
    pub era_progress: u16,
    /// M5.5: people who joined from migrant waves.
    pub migrants_joined: u32,
    /// M5.5: trades and gifts.
    pub trades: u32,
    /// M5.5: discoveries helped by an artifact.
    pub eurekas: u32,
}

/// The summary of year `year` (0-based) from the chronicle, by entry tick.
pub fn collect_year_summary(res: &SimResources, year: i64) -> YearSummary {
    let Ok(y) = u32::try_from(year) else {
        return YearSummary::default();
    };
    let per_year = res.rules.calendar.ticks_per_year().max(1);
    let start = u64::from(y) * per_year;
    let end = start + per_year;
    let mut out = YearSummary {
        year: y,
        found: res.current_tick >= start,
        complete: res.current_tick >= end,
        era_progress: res.colony.tech.progress_permille,
        ..YearSummary::default()
    };
    if !out.found {
        return out;
    }
    let bump = |n: &mut u32| *n = n.saturating_add(1);
    for e in res.chronicle.entries() {
        if e.tick < start || e.tick >= end {
            continue;
        }
        match e.kind {
            ChronicleKind::Born => bump(&mut out.births),
            ChronicleKind::Died => {
                if let Some(n) = out.deaths.get_mut(usize::from(e.cause)) {
                    bump(n);
                }
            }
            ChronicleKind::PressureBegan => {
                if let Some(n) = out.pressures.get_mut(usize::from(e.cause)) {
                    bump(n);
                }
            }
            ChronicleKind::KnowledgeDiscovered => bump(&mut out.discoveries),
            ChronicleKind::KnowledgeLost => bump(&mut out.lost),
            ChronicleKind::Quarrel => bump(&mut out.quarrels),
            ChronicleKind::Fight => bump(&mut out.fights),
            ChronicleKind::Breakdown => bump(&mut out.breakdowns),
            ChronicleKind::Hunted if e.cause == AnimalKind::Deer.code() => bump(&mut out.hunted),
            ChronicleKind::WolfKilled => bump(&mut out.wolves_killed),
            ChronicleKind::MigrantsJoined => {
                out.migrants_joined = out.migrants_joined.saturating_add((e.value & 0xFF) as u32)
            }
            ChronicleKind::Traded => bump(&mut out.trades),
            ChronicleKind::Eureka => bump(&mut out.eurekas),
            _ => {}
        }
    }
    out
}

/// Marshal [`YearSummary`].
pub(super) fn year_summary_to_dict(s: &YearSummary) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("year", i64::from(s.year));
    d.set("found", s.found);
    d.set("complete", s.complete);
    d.set("births", i64::from(s.births));
    let deaths: Vec<i32> = s.deaths.iter().map(|n| *n as i32).collect();
    let pressures: Vec<i32> = s.pressures.iter().map(|n| *n as i32).collect();
    d.set("deaths", PackedInt32Array::from(deaths.as_slice()));
    d.set("pressures", PackedInt32Array::from(pressures.as_slice()));
    d.set("discoveries", i64::from(s.discoveries));
    d.set("lost", i64::from(s.lost));
    d.set("quarrels", i64::from(s.quarrels));
    d.set("fights", i64::from(s.fights));
    d.set("breakdowns", i64::from(s.breakdowns));
    d.set("hunted", i64::from(s.hunted));
    d.set("wolves_killed", i64::from(s.wolves_killed));
    d.set("era_progress", i64::from(s.era_progress));
    d.set("migrants_joined", i64::from(s.migrants_joined));
    d.set("trades", i64::from(s.trades));
    d.set("eurekas", i64::from(s.eurekas));
    d
}

/// An M5 order from the bridge (ids as given; a negative id is refused as unknown).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoryCommand {
    /// Draft (`true`) or release a person.
    Draft(i64, bool),
    /// A drafted person walks to a tile.
    DraftMove(i64, i32, i32),
    /// A drafted person attacks an animal.
    DraftAttack(i64, i64),
    /// A drafted person attacks a person.
    DraftAttackPerson(i64, i64),
    /// Mark a deer to be hunted.
    DesignateHunt(i64),
    /// Add a hunt zone over the rectangle.
    AddHuntZone(i32, i32, i32, i32),
    /// Remove the hunt zones and marks in the rectangle.
    CancelHunt(i32, i32, i32, i32),
    /// The standing order "모두 가죽 옷" on or off.
    SetDressOrder(bool),
}

/// Apply a [`StoryCommand`] through `submit` (logged with the next tick, ADR 0012).
pub fn submit_story(engine: &mut SimEngine, cmd: StoryCommand) -> CommandResult {
    let unknown = CommandResult::rejected(RejectReason::Unknown);
    let id = |v: i64| u64::try_from(v).ok();
    let animal = |v: i64| u32::try_from(v).ok();
    let c = |v: i32| u32::try_from(v.max(0)).unwrap_or(0);
    let command = match cmd {
        StoryCommand::Draft(a, on) => id(a).map(|agent| Command::SetDrafted { agent, drafted: on }),
        StoryCommand::DraftMove(a, x, y) => match (id(a), u32::try_from(x), u32::try_from(y)) {
            (Some(agent), Ok(x), Ok(y)) => Some(Command::DraftMove { agent, x, y }),
            _ => None,
        },
        StoryCommand::DraftAttack(a, t) => match (id(a), animal(t)) {
            (Some(agent), Some(animal)) => Some(Command::DraftAttack { agent, animal }),
            _ => None,
        },
        StoryCommand::DraftAttackPerson(a, t) => match (id(a), id(t)) {
            (Some(agent), Some(target)) => Some(Command::DraftAttackPerson { agent, target }),
            _ => None,
        },
        StoryCommand::DesignateHunt(t) => animal(t).map(|animal| Command::DesignateHunt { animal }),
        StoryCommand::AddHuntZone(x0, y0, x1, y1) => Some(Command::AddHuntZone {
            x0: c(x0),
            y0: c(y0),
            x1: c(x1),
            y1: c(y1),
        }),
        StoryCommand::CancelHunt(x0, y0, x1, y1) => Some(Command::CancelHunt {
            x0: c(x0),
            y0: c(y0),
            x1: c(x1),
            y1: c(y1),
        }),
        StoryCommand::SetDressOrder(on) => Some(Command::SetDressOrder { on }),
    };
    command.map_or(unknown, |cmd| submit(engine, cmd))
}

/// Release every drafted person (`cmd_undraft_all`), in `AgentId` order; the
/// result counts the people released.
pub fn submit_undraft_all(engine: &mut SimEngine) -> CommandResult {
    let mut ids: Vec<AgentId> = engine
        .world
        .query::<(&Agent, &Drafted)>()
        .iter()
        .map(|(_, (a, _))| a.id)
        .collect();
    ids.sort_unstable();
    if ids.is_empty() {
        return CommandResult::rejected(RejectReason::Nothing);
    }
    let mut n = 0u32;
    for agent in ids {
        if submit(
            engine,
            Command::SetDrafted {
                agent,
                drafted: false,
            },
        )
        .accepted
        {
            n += 1;
        }
    }
    CommandResult::ok(n)
}

/// Whether any pressure of `kind` is in force (tests and the HUD).
pub fn pressure_active(res: &SimResources, kind: PressureKind) -> bool {
    res.colony.story.is_active(kind)
}

/// Modifier name of a code (the inspector's fallback; `"relation"` for 100).
pub fn modifier_name(code: u8) -> &'static str {
    if code == sim_core::colony::people::RELATION_CAUSE {
        return "relation";
    }
    Modifier::from_code(code).map_or("", Modifier::name)
}
