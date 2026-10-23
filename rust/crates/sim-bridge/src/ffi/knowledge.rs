//! M4 knowledge bridge (ADR 0024–0026): the tech map, one node's detail, a
//! person's knowledge, the apprenticeship orders and the orders waiting for a
//! practitioner, plus the teach and clay commands.
//!
//! Reads are pure collectors (`collect_*`), rows in a documented order:
//! [`collect_tech`] (node id order), [`collect_tech_node`] (people by
//! `AgentId`), [`collect_agent_knowledge`] (entries by node id),
//! [`collect_teach_orders`] (order id order), [`collect_knowledge_waits`]
//! (workshops by id and bills in list order, then sites by centre `(x, y)`, then
//! marks by tile `(x, y)`). Ids: people are `AgentId`, nodes `KnowledgeId` (index in
//! `knowledge.ron`), workshops and bills their colony ids. Every read returns
//! its empty form in the baseline world and under rules without knowledge.
use godot::prelude::*;
use sim_core::colony::knowledge::{
    self, defs, era_label, gates, Cond, Gate, Knowledge, KnowledgeId, Lore, TechStatus,
};
use sim_core::colony::{Command, CommandResult, Designation, Job, JobKind};
use sim_core::components::{Agent, AgentId, Child, Life};
use sim_engine::{SimEngine, SimResources};
use sim_systems::commands::submit;
use sim_systems::runtime::knowledge::discover::holds;

/// The tech map (`get_tech`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TechView {
    /// `TechAggregate::version` (0 before the first aggregate).
    pub version: u64,
    /// Era progress ‰.
    pub progress: u16,
    /// Era label: 0 구석기, 1 신석기 전환기, 2 신석기 시대.
    pub era: u8,
    /// Adults at the last aggregate.
    pub adults: u16,
    /// Nodes at risk (one holder): the HUD badge.
    pub at_risk: u32,
    /// One row per node, in id order.
    pub nodes: Vec<TechRow>,
}

/// One node of the tech map.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TechRow {
    /// `KnowledgeId`.
    pub id: u8,
    /// Stable name (`knowledge.ron` id).
    pub name: String,
    /// Locale key of its name.
    pub name_key: String,
    /// `Lane::code` (0 survival … 5 social).
    pub lane: u8,
    /// `Era::code` (0 paleolithic, 1 neolithic).
    pub era: u8,
    /// Threshold ‰.
    pub threshold: u16,
    /// `TechStatus::code` (0 unknown, 1 at risk, 2 known, 3 stable, 4 lost).
    pub status: u8,
    /// Practitioners.
    pub practitioners: u16,
    /// Holders.
    pub holders: u16,
    /// Era-marker weight (0 = not a marker).
    pub marker_weight: u16,
    /// Prerequisite node ids.
    pub prereqs: Vec<u8>,
    /// The first named person (`AgentId`; the one holder of an at-risk node), −1 when none.
    pub first_name: i64,
    /// Whether every discovery condition holds now (false for nodes never discovered).
    pub conditions_met: bool,
}

fn lore_name(l: Lore) -> &'static str {
    match l {
        Lore::Hearth => "hearth",
        Lore::Forage => "forage",
        Lore::Water => "water",
    }
}

fn designation_name(d: Designation) -> &'static str {
    match d {
        Designation::Chop => "chop",
        Designation::Quarry => "quarry",
        Designation::Dig => "dig",
        Designation::Deconstruct => "deconstruct",
        Designation::ClayPit => "clay_pit",
        Designation::Hunt => "hunt",
    }
}

fn enabled(res: &SimResources) -> bool {
    res.rules.colony.and_then(|c| c.knowledge).is_some()
}

/// The tech map (see [`TechView`]).
pub fn collect_tech(world: &hecs::World, res: &SimResources) -> TechView {
    if !enabled(res) {
        return TechView::default();
    }
    let tech = &res.colony.tech;
    let open = sim_systems::runtime::knowledge::discover::conditions(world, res);
    let nodes = defs()
        .nodes
        .iter()
        .map(|n| {
            let s = tech
                .states
                .get(usize::from(n.id))
                .cloned()
                .unwrap_or_default();
            TechRow {
                id: n.id,
                name: n.name.clone(),
                name_key: n.name_key.clone(),
                lane: n.lane.code(),
                era: n.era.code(),
                threshold: n.threshold,
                status: s.status.code(),
                practitioners: s.practitioners,
                holders: s.holders,
                marker_weight: n.marker_weight,
                prereqs: n.prereqs.clone(),
                first_name: s.names.first().map_or(-1, |id| *id as i64),
                conditions_met: open & knowledge::bit(n.id) != 0,
            }
        })
        .collect();
    TechView {
        version: tech.version,
        progress: tech.progress_permille,
        era: era_label(tech.progress_permille),
        adults: tech.adults,
        at_risk: tech.at_risk() as u32,
        nodes,
    }
}

/// One person's view of one node (`get_tech_node` practitioners and learners).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NodePerson {
    /// `AgentId`.
    pub id: AgentId,
    /// Proficiency ‰ (0 for a learner who has none).
    pub proficiency: u16,
    /// Knows instruction (teaches at full fidelity).
    pub instruction: bool,
}

/// One node's detail panel (`get_tech_node`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TechNodeView {
    /// Whether the node exists (and knowledge is on).
    pub found: bool,
    /// The node's row of the tech map.
    pub row: TechRow,
    /// Holders (at threshold) by `AgentId`.
    pub practitioners: Vec<NodePerson>,
    /// Who could learn it: adults and children of the learner age below the threshold, by `AgentId`.
    pub learners: Vec<NodePerson>,
    /// Discovery conditions: `(kind, name, amount, met)`; kind 0 stockpile
    /// (name = item), 1 skill (name = skill, amount = level), 2 knows (name =
    /// node, amount = ‰), 3 clay bank near (amount = tiles).
    pub conditions: Vec<(u8, String, i64, bool)>,
    /// Gates: `(kind, name)`; kind 0 recipe, 1 blueprint, 2 workshop, 3 designation, 4 channel, 5 lore.
    pub gates: Vec<(u8, String)>,
}

/// One node's detail (see [`TechNodeView`]).
pub fn collect_tech_node(world: &hecs::World, res: &SimResources, node: i64) -> TechNodeView {
    let Some(def) = u8::try_from(node).ok().and_then(|n| defs().get(n)) else {
        return TechNodeView::default();
    };
    if !enabled(res) {
        return TechNodeView::default();
    }
    let row = collect_tech(world, res)
        .nodes
        .into_iter()
        .nth(usize::from(def.id))
        .unwrap_or_default();
    let instruction = gates().apprenticeship();
    let min_years = sim_systems::runtime::knowledge::rule(res).map_or(10, |r| r.learner_min_years);
    let year = res.rules.calendar.years_to_ticks(1).max(1);
    let mut people: Vec<(AgentId, bool, u64, Knowledge)> = world
        .query::<(&Agent, Option<&Child>, Option<&Life>, Option<&Knowledge>)>()
        .iter()
        .map(|(_, (a, c, life, k))| {
            let years = life.map_or(0, |l| {
                u64::try_from(l.age_ticks(res.current_tick)).unwrap_or(0) / year
            });
            (a.id, c.is_none(), years, k.cloned().unwrap_or_default())
        })
        .collect();
    people.sort_by_key(|p| p.0);
    let mut practitioners = Vec::new();
    let mut learners = Vec::new();
    for (id, adult, years, k) in &people {
        let person = NodePerson {
            id: *id,
            proficiency: k.proficiency(def.id),
            instruction: k.practice() & instruction != 0,
        };
        if k.practises(def.id) {
            practitioners.push(person);
        } else if *adult || *years >= min_years {
            learners.push(person);
        }
    }
    let conditions = def
        .discover
        .iter()
        .map(|c| {
            let met = holds(world, res, *c);
            match *c {
                Cond::Stockpile(item, n) => (0, item.name().to_string(), i64::from(n), met),
                Cond::Skill(skill, l) => (1, skill.name().to_string(), i64::from(l), met),
                Cond::Knows(id, p) => (
                    2,
                    defs().get(id).map_or(String::new(), |n| n.name.clone()),
                    i64::from(p),
                    met,
                ),
                Cond::ClayBankNear(d) => (3, String::new(), i64::from(d), met),
            }
        })
        .collect();
    let gates = def
        .gates
        .iter()
        .map(|g| match *g {
            Gate::Recipe(r) => (0, r.name().to_string()),
            Gate::Blueprint(b) => (1, b.name().to_string()),
            Gate::Workshop(b) => (2, b.name().to_string()),
            Gate::Designation(d) => (3, designation_name(d).to_string()),
            Gate::Channel(c) => (
                4,
                match c {
                    knowledge::Channel::Apprenticeship => "apprenticeship".to_string(),
                    knowledge::Channel::Oral => "oral".to_string(),
                },
            ),
            Gate::Lore(l) => (5, lore_name(l).to_string()),
        })
        .collect();
    TechNodeView {
        found: true,
        row,
        practitioners,
        learners,
        conditions,
        gates,
    }
}

/// One entry of a person's knowledge tab.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct KnowledgeRow {
    /// `KnowledgeId`.
    pub id: u8,
    /// Proficiency ‰.
    pub proficiency: u16,
    /// The node's threshold ‰.
    pub threshold: u16,
    /// `Source::code` (0 seeded, 1 discovered, 2 oral, 3 observed, 4 apprenticed).
    pub source: u8,
    /// Tick of the start of the day of the last use.
    pub last_used: u64,
    /// Whole days since the last use.
    pub days_unused: u64,
    /// 0 숙련 (practitioner), 1 배우는 중 (below threshold, used lately), 2 잊어 감 (unused past the fading start).
    pub state: u8,
}

/// A person's knowledge tab (`get_agent_knowledge`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AgentKnowledgeView {
    /// Whether the person exists.
    pub found: bool,
    /// Entries by node id.
    pub rows: Vec<KnowledgeRow>,
    /// The person's teach or learn job: `(role 0 teach / 1 learn, order id,
    /// partner AgentId, node)`; `None` without one.
    pub lesson: Option<(u8, u32, AgentId, u8)>,
}

/// A person's knowledge (see [`AgentKnowledgeView`]).
pub fn collect_agent_knowledge(
    world: &hecs::World,
    res: &SimResources,
    agent_id: i64,
) -> AgentKnowledgeView {
    let Some((entity, k)) = world
        .query::<(&Agent, Option<&Knowledge>)>()
        .iter()
        .find(|(_, (a, _))| a.id as i64 == agent_id)
        .map(|(e, (_, k))| (e, k.cloned().unwrap_or_default()))
    else {
        return AgentKnowledgeView::default();
    };
    let today = u64::from(knowledge::day_of(res.current_tick));
    let fade = sim_systems::runtime::knowledge::rule(res).map_or(6, |r| r.unused_days);
    let rows = k
        .entries()
        .iter()
        .map(|e| {
            let threshold = defs().threshold(e.id());
            let days_unused = today.saturating_sub(u64::from(e.last_used));
            let state = if days_unused > fade {
                2
            } else if e.proficiency >= threshold {
                0
            } else {
                1
            };
            KnowledgeRow {
                id: e.id(),
                proficiency: e.proficiency,
                threshold,
                source: e.source().code(),
                last_used: u64::from(e.last_used) * knowledge::DAY_TICKS,
                days_unused,
                state,
            }
        })
        .collect();
    let lesson = world.get::<&Job>(entity).ok().and_then(|j| {
        let (role, order) = match j.kind {
            JobKind::Teach { order, .. } => (0u8, order),
            JobKind::Learn { order, .. } => (1u8, order),
            _ => return None,
        };
        let o = res.colony.teach_orders.iter().find(|o| o.id == order)?;
        let partner = if role == 0 { o.learner } else { o.teacher };
        Some((role, order, partner, o.knowledge))
    });
    AgentKnowledgeView {
        found: true,
        rows,
        lesson,
    }
}

/// One apprenticeship order (`get_teach_orders`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TeachRow {
    /// Order id.
    pub id: u32,
    /// Teacher `AgentId`.
    pub teacher: AgentId,
    /// Learner `AgentId`.
    pub learner: AgentId,
    /// `KnowledgeId`.
    pub knowledge: u8,
    /// Sessions held.
    pub sessions: u32,
    /// The learner's proficiency ‰ now.
    pub proficiency: u16,
    /// The node's threshold ‰ (the goal of the order).
    pub threshold: u16,
    /// A lesson is under way (either partner holds its job).
    pub in_session: bool,
}

/// Every open apprenticeship order, in id order.
pub fn collect_teach_orders(world: &hecs::World, res: &SimResources) -> Vec<TeachRow> {
    let lessons: Vec<u32> = world
        .query::<&Job>()
        .iter()
        .filter_map(|(_, j)| match j.kind {
            JobKind::Teach { order, .. } | JobKind::Learn { order, .. } => Some(order),
            _ => None,
        })
        .collect();
    res.colony
        .teach_orders
        .iter()
        .map(|o| {
            let proficiency = world
                .query::<(&Agent, &Knowledge)>()
                .iter()
                .find(|(_, (a, _))| a.id == o.learner)
                .map_or(0, |(_, (_, k))| k.proficiency(o.knowledge));
            TeachRow {
                id: o.id,
                teacher: o.teacher,
                learner: o.learner,
                knowledge: o.knowledge,
                sessions: o.sessions,
                proficiency,
                threshold: defs().threshold(o.knowledge),
                in_session: lessons.contains(&o.id),
            }
        })
        .collect()
}

/// One order waiting for a practitioner (`get_knowledge_waits`; the "why?" line).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WaitRow {
    /// 0 a bill, 1 a site, 2 a mark.
    pub kind: u8,
    /// Bill: the workshop id; site and mark: the tile x.
    pub a: i64,
    /// Bill: the bill id; site and mark: the tile y.
    pub b: i64,
    /// The first node of the gate no living adult practises.
    pub node: u8,
}

/// Every bill with work left, building site and work mark whose gate no living
/// adult practises.
pub fn collect_knowledge_waits(world: &hecs::World, res: &SimResources) -> Vec<WaitRow> {
    if !enabled(res) {
        return Vec::new();
    }
    let practised: u32 = world
        .query::<&Knowledge>()
        .without::<&Child>()
        .iter()
        .fold(0, |m, (_, k)| m | k.practice());
    // A gate is open when one adult practises all of it; for the line, the first node no adult practises.
    let blocked = |mask: u32| -> Option<u8> {
        if mask == 0 || sim_systems::runtime::knowledge::anyone_practises(world, mask) {
            return None;
        }
        knowledge::ids_of(mask & !practised)
            .next()
            .or_else(|| knowledge::ids_of(mask).next())
    };
    let g = gates();
    let mut out = Vec::new();
    for shop in res.colony.workshops.values() {
        for bill in shop
            .bills
            .iter()
            .filter(|b| !b.suspended && b.remaining() != Some(0))
        {
            if let Some(node) = blocked(g.recipe(bill.recipe)) {
                out.push(WaitRow {
                    kind: 0,
                    a: i64::from(shop.id),
                    b: i64::from(bill.id),
                    node,
                });
            }
        }
    }
    for (centre, view) in sim_systems::runtime::jobs::items::colony_sites(world) {
        if let Some(node) = blocked(g.build(view.site.kind)) {
            out.push(WaitRow {
                kind: 1,
                a: i64::from(centre.0),
                b: i64::from(centre.1),
                node,
            });
        }
    }
    for (tile, mark) in &res.colony.designations {
        if let Some(node) = blocked(g.designation(*mark)) {
            out.push(WaitRow {
                kind: 2,
                a: i64::from(tile.0),
                b: i64::from(tile.1),
                node,
            });
        }
    }
    out
}

fn result_dict(r: CommandResult) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("accepted", r.accepted);
    dict.set("count", i64::from(r.count));
    dict.set("reason", GString::from(r.reason.map_or("", |x| x.name())));
    dict
}

/// The teach dialog's suggested learner for node `node` taught by `teacher` (M4
/// owner decision, M5 DECISIONS D5): among living adults other than the teacher
/// who do not practise the node, the highest novelty seeking (`Temperament.ns`),
/// then the youngest (latest `Life::born_tick`), then the lowest `AgentId`; −1
/// when there is none. The dialog shows it pre-selected and editable.
pub fn collect_teach_suggestion(world: &hecs::World, teacher: i64, node: i64) -> i64 {
    let (Ok(t), Ok(k)) = (u64::try_from(teacher), u8::try_from(node)) else {
        return -1;
    };
    if defs().get(k).is_none() {
        return -1;
    }
    world
        .query::<(
            &Agent,
            Option<&Knowledge>,
            Option<&sim_core::components::Temperament>,
            Option<&Life>,
        )>()
        .without::<&Child>()
        .iter()
        .filter(|(_, (a, kn, _, _))| a.id != t && !kn.is_some_and(|kn| kn.practises(k)))
        .map(|(_, (a, _, temp, life))| {
            (
                std::cmp::Reverse(temp.map_or(0, |x| x.ns)),
                std::cmp::Reverse(life.map_or(i64::MIN, |l| l.born_tick)),
                a.id,
            )
        })
        .min()
        .map_or(-1, |(_, _, id)| i64::try_from(id).unwrap_or(-1))
}

/// `OrderTeach` (ids are `AgentId` / `KnowledgeId`; a negative or oversized id is refused as unknown).
pub fn submit_order_teach(
    engine: &mut SimEngine,
    teacher: i64,
    learner: i64,
    node: i64,
) -> VarDictionary {
    let (Ok(t), Ok(l), Ok(k)) = (
        u64::try_from(teacher),
        u64::try_from(learner),
        u8::try_from(node),
    ) else {
        return result_dict(CommandResult::rejected(
            sim_core::colony::RejectReason::Unknown,
        ));
    };
    result_dict(submit(
        engine,
        Command::OrderTeach {
            teacher: t,
            learner: l,
            knowledge: k,
        },
    ))
}

/// `CancelTeach`.
pub fn submit_cancel_teach(engine: &mut SimEngine, order: i64) -> VarDictionary {
    let Ok(o) = u32::try_from(order) else {
        return result_dict(CommandResult::rejected(
            sim_core::colony::RejectReason::Unknown,
        ));
    };
    result_dict(submit(engine, Command::CancelTeach { order: o }))
}

/// `DesignateClay` over the rectangle (corners clipped at 0).
pub fn submit_designate_clay(
    engine: &mut SimEngine,
    x0: i32,
    y0: i32,
    x1: i32,
    y1: i32,
) -> VarDictionary {
    let c = |v: i32| u32::try_from(v.max(0)).unwrap_or(0);
    result_dict(submit(
        engine,
        Command::DesignateClay {
            x0: c(x0),
            y0: c(y0),
            x1: c(x1),
            y1: c(y1),
        },
    ))
}

fn int_col(values: impl Iterator<Item = i64>) -> PackedInt64Array {
    let v: Vec<i64> = values.collect();
    PackedInt64Array::from(v.as_slice())
}

fn str_col<'a>(values: impl Iterator<Item = &'a str>) -> PackedStringArray {
    let v: Vec<GString> = values.map(GString::from).collect();
    PackedStringArray::from(v.as_slice())
}

pub(super) fn tech_to_dict(v: &TechView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("version", v.version as i64);
    d.set("progress", i64::from(v.progress));
    d.set("era", i64::from(v.era));
    d.set("adults", i64::from(v.adults));
    d.set("at_risk", i64::from(v.at_risk));
    let n = &v.nodes;
    d.set("ids", int_col(n.iter().map(|r| i64::from(r.id))));
    d.set("names", str_col(n.iter().map(|r| r.name.as_str())));
    d.set("name_keys", str_col(n.iter().map(|r| r.name_key.as_str())));
    d.set("lanes", int_col(n.iter().map(|r| i64::from(r.lane))));
    d.set("eras", int_col(n.iter().map(|r| i64::from(r.era))));
    d.set(
        "thresholds",
        int_col(n.iter().map(|r| i64::from(r.threshold))),
    );
    d.set("statuses", int_col(n.iter().map(|r| i64::from(r.status))));
    d.set(
        "practitioners",
        int_col(n.iter().map(|r| i64::from(r.practitioners))),
    );
    d.set("holders", int_col(n.iter().map(|r| i64::from(r.holders))));
    d.set(
        "marker_weights",
        int_col(n.iter().map(|r| i64::from(r.marker_weight))),
    );
    d.set("first_names", int_col(n.iter().map(|r| r.first_name)));
    d.set(
        "conditions_met",
        int_col(n.iter().map(|r| i64::from(r.conditions_met))),
    );
    d.set(
        "prereq_from",
        int_col(
            n.iter()
                .flat_map(|r| r.prereqs.iter().map(|p| i64::from(*p))),
        ),
    );
    d.set(
        "prereq_to",
        int_col(
            n.iter()
                .flat_map(|r| r.prereqs.iter().map(move |_| i64::from(r.id))),
        ),
    );
    d
}

pub(super) fn tech_node_to_dict(v: &TechNodeView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", v.found);
    d.set("id", i64::from(v.row.id));
    d.set("name", GString::from(v.row.name.as_str()));
    d.set("name_key", GString::from(v.row.name_key.as_str()));
    d.set("status", i64::from(v.row.status));
    d.set("threshold", i64::from(v.row.threshold));
    d.set("practitioners", i64::from(v.row.practitioners));
    d.set("holders", i64::from(v.row.holders));
    d.set("lane", i64::from(v.row.lane));
    d.set("era", i64::from(v.row.era));
    d.set("marker_weight", i64::from(v.row.marker_weight));
    d.set(
        "prereqs",
        int_col(v.row.prereqs.iter().map(|p| i64::from(*p))),
    );
    d.set(
        "holder_ids",
        int_col(v.practitioners.iter().map(|p| p.id as i64)),
    );
    d.set(
        "holder_proficiency",
        int_col(v.practitioners.iter().map(|p| i64::from(p.proficiency))),
    );
    d.set(
        "holder_instruction",
        int_col(v.practitioners.iter().map(|p| i64::from(p.instruction))),
    );
    d.set(
        "learner_ids",
        int_col(v.learners.iter().map(|p| p.id as i64)),
    );
    d.set(
        "learner_proficiency",
        int_col(v.learners.iter().map(|p| i64::from(p.proficiency))),
    );
    d.set(
        "cond_kinds",
        int_col(v.conditions.iter().map(|c| i64::from(c.0))),
    );
    d.set(
        "cond_names",
        str_col(v.conditions.iter().map(|c| c.1.as_str())),
    );
    d.set("cond_amounts", int_col(v.conditions.iter().map(|c| c.2)));
    d.set(
        "cond_met",
        int_col(v.conditions.iter().map(|c| i64::from(c.3))),
    );
    d.set(
        "gate_kinds",
        int_col(v.gates.iter().map(|g| i64::from(g.0))),
    );
    d.set("gate_names", str_col(v.gates.iter().map(|g| g.1.as_str())));
    d
}

pub(super) fn agent_knowledge_to_dict(v: &AgentKnowledgeView) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("found", v.found);
    let r = &v.rows;
    d.set("ids", int_col(r.iter().map(|x| i64::from(x.id))));
    d.set(
        "proficiency",
        int_col(r.iter().map(|x| i64::from(x.proficiency))),
    );
    d.set(
        "thresholds",
        int_col(r.iter().map(|x| i64::from(x.threshold))),
    );
    d.set("sources", int_col(r.iter().map(|x| i64::from(x.source))));
    d.set("last_used", int_col(r.iter().map(|x| x.last_used as i64)));
    d.set(
        "days_unused",
        int_col(r.iter().map(|x| x.days_unused as i64)),
    );
    d.set("states", int_col(r.iter().map(|x| i64::from(x.state))));
    match v.lesson {
        Some((role, order, partner, node)) => {
            d.set("lesson_role", i64::from(role));
            d.set("lesson_order", i64::from(order));
            d.set("lesson_partner", partner as i64);
            d.set("lesson_node", i64::from(node));
        }
        None => {
            d.set("lesson_role", -1i64);
            d.set("lesson_order", -1i64);
            d.set("lesson_partner", -1i64);
            d.set("lesson_node", -1i64);
        }
    }
    d
}

pub(super) fn teach_orders_to_dict(rows: &[TeachRow]) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("ids", int_col(rows.iter().map(|r| i64::from(r.id))));
    d.set("teachers", int_col(rows.iter().map(|r| r.teacher as i64)));
    d.set("learners", int_col(rows.iter().map(|r| r.learner as i64)));
    d.set(
        "knowledge",
        int_col(rows.iter().map(|r| i64::from(r.knowledge))),
    );
    d.set(
        "sessions",
        int_col(rows.iter().map(|r| i64::from(r.sessions))),
    );
    d.set(
        "proficiency",
        int_col(rows.iter().map(|r| i64::from(r.proficiency))),
    );
    d.set(
        "thresholds",
        int_col(rows.iter().map(|r| i64::from(r.threshold))),
    );
    d.set(
        "in_session",
        int_col(rows.iter().map(|r| i64::from(r.in_session))),
    );
    d
}

pub(super) fn waits_to_dict(rows: &[WaitRow]) -> VarDictionary {
    let mut d = VarDictionary::new();
    d.set("kinds", int_col(rows.iter().map(|r| i64::from(r.kind))));
    d.set("a", int_col(rows.iter().map(|r| r.a)));
    d.set("b", int_col(rows.iter().map(|r| r.b)));
    d.set("nodes", int_col(rows.iter().map(|r| i64::from(r.node))));
    d
}

/// The status code of `TechStatus::AtRisk` (for tests and GDScript constants).
pub const STATUS_AT_RISK: u8 = TechStatus::AtRisk.code();

/// Unused-import guard for `KnowledgeId` in docs.
pub type NodeId = KnowledgeId;
