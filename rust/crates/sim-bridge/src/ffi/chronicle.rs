//! Pass 19: read-only chronicle lookups for the art line's chronicle and
//! "why?" UI. Pure collectors over `SimResources::chronicle` (and the world
//! for entity-bit resolution and liveness); the `#[func]` wrappers in
//! `node.rs` only marshal. Every id is an i64 (AgentId bits as the
//! snapshot's `agent_ids`); `None` is -1.

use godot::prelude::*;
use sim_core::chronicle::{Chronicle, ChronicleEntry};
use sim_core::components::{Agent, AgentId, BodyHealth, SettlementId};

/// Largest page `get_chronicle_since` returns.
pub const CHRONICLE_PAGE_MAX: i64 = 512;

fn signed(id: u64) -> i64 {
    i64::from_ne_bytes(id.to_ne_bytes())
}

fn unsigned(id: i64) -> u64 {
    u64::from_ne_bytes(id.to_ne_bytes())
}

/// The AgentId of the entity with snapshot bits `entity_bits`, if it is an agent.
pub fn collect_agent_id(world: &hecs::World, entity_bits: i64) -> Option<AgentId> {
    let entity = hecs::Entity::from_bits(unsigned(entity_bits))?;
    world.get::<&Agent>(entity).ok().map(|agent| agent.id)
}

/// One person from the person table (works for the dead).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PersonView {
    /// Whether the person table has this id.
    pub found: bool,
    /// Whether a living entity with this id exists.
    pub alive: bool,
    /// -1 unknown, 0 male, 1 female.
    pub female: i64,
    /// Birth tick (founders before zero); -1 unknown.
    pub born_tick: i64,
    /// Death tick; -1 while alive.
    pub died_tick: i64,
    /// `DeathReason::as_u8`; -1 while alive.
    pub death_reason: i64,
    /// Mother's AgentId; -1 unknown.
    pub mother: i64,
    /// Father's AgentId; -1 unknown.
    pub father: i64,
    /// Temperament axes NS, HA, RD, P; -1 without a temperament.
    pub temperament: [i64; 4],
}

/// The person-table record of `agent_id` (i64 bits) and whether they live.
pub fn collect_person(world: &hecs::World, chronicle: &Chronicle, agent_id: i64) -> PersonView {
    let id = unsigned(agent_id);
    let Some(person) = chronicle.person(id) else {
        return PersonView {
            female: -1,
            born_tick: -1,
            died_tick: -1,
            death_reason: -1,
            mother: -1,
            father: -1,
            temperament: [-1; 4],
            ..PersonView::default()
        };
    };
    let alive = person.died_tick.is_none()
        && world
            .query::<(&Agent, Option<&BodyHealth>)>()
            .iter()
            .any(|(_, (agent, health))| agent.id == id && !health.is_some_and(BodyHealth::is_dead));
    PersonView {
        found: true,
        alive,
        female: person.female.map_or(-1, i64::from),
        born_tick: person.born_tick.unwrap_or(-1),
        died_tick: person.died_tick.map_or(-1, signed),
        death_reason: person.death_reason.map_or(-1, i64::from),
        mother: person.mother.map_or(-1, signed),
        father: person.father.map_or(-1, signed),
        temperament: person.temperament.map_or([-1; 4], |t| t.map(i64::from)),
    }
}

/// Chronicle entries as parallel columns (chronicle order).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ChronicleRows {
    /// Whether the asked person / settlement is known (or the page request valid).
    pub found: bool,
    /// Entry ticks.
    pub tick: Vec<i64>,
    /// `ChronicleKind::as_u8`.
    pub kind: Vec<i32>,
    /// Subject AgentIds (-1 none).
    pub subject: Vec<i64>,
    /// Other AgentIds (-1 none).
    pub other: Vec<i64>,
    /// Third AgentIds (-1 none).
    pub third: Vec<i64>,
    /// Settlement ids (-1 none).
    pub settlement: Vec<i64>,
    /// Second settlement ids (-1 none).
    pub settlement2: Vec<i64>,
    /// Kind-specific cause codes.
    pub cause: Vec<i32>,
    /// Kind-specific values.
    pub value: Vec<i64>,
    /// M5: each entry's chronicle index (the key of `get_chronicle_why`).
    pub index: Vec<i64>,
    /// M5: the "→" link of each entry: −1 none, 0 another entry, 1 a mood modifier, 2 a pressure.
    pub link: Vec<i32>,
    /// M5: the entry the link points at (−1 none).
    pub link_entry: Vec<i64>,
    /// M5: the link's modifier code (100 = the relation) or pressure code (−1 none).
    pub link_code: Vec<i32>,
    /// `get_chronicle_since` only: the index after the last returned entry.
    pub next: i64,
    /// `get_chronicle_since` only: total entries.
    pub total: i64,
}

impl ChronicleRows {
    fn push(&mut self, chronicle: &Chronicle, index: u32, entry: &ChronicleEntry) {
        let (link, link_entry, link_code) = super::story::cause_link(chronicle, index);
        self.index.push(i64::from(index));
        self.link.push(link);
        self.link_entry.push(link_entry);
        self.link_code.push(link_code);
        self.tick.push(signed(entry.tick));
        self.kind.push(i32::from(entry.kind.as_u8()));
        self.subject.push(entry.subject.map_or(-1, signed));
        self.other.push(entry.other.map_or(-1, signed));
        self.third.push(entry.third.map_or(-1, signed));
        self.settlement.push(entry.settlement.map_or(-1, i64::from));
        self.settlement2
            .push(entry.settlement2.map_or(-1, i64::from));
        self.cause.push(i32::from(entry.cause));
        self.value.push(entry.value);
    }

    fn from_indices(
        chronicle: &Chronicle,
        found: bool,
        indices: impl Iterator<Item = u32>,
    ) -> Self {
        let mut rows = Self {
            found,
            ..Self::default()
        };
        for index in indices {
            if let Some(entry) = chronicle.entry(index) {
                rows.push(chronicle, index, entry);
            }
        }
        rows
    }
}

/// Every entry naming person `agent_id`, in chronicle order.
pub fn collect_person_chronicle(chronicle: &Chronicle, agent_id: i64) -> ChronicleRows {
    let id = unsigned(agent_id);
    let found = chronicle.person(id).is_some() || chronicle.of_agent(id).next().is_some();
    ChronicleRows::from_indices(
        chronicle,
        found,
        chronicle.indices_of_agent(id).iter().copied(),
    )
}

/// Every entry naming settlement `settlement_id`, in chronicle order.
pub fn collect_settlement_chronicle(chronicle: &Chronicle, settlement_id: i64) -> ChronicleRows {
    let Ok(id) = SettlementId::try_from(settlement_id) else {
        return ChronicleRows::default();
    };
    let found = chronicle.of_settlement(id).next().is_some();
    ChronicleRows::from_indices(
        chronicle,
        found,
        chronicle.indices_of_settlement(id).iter().copied(),
    )
}

/// Entries `start ..` (at most `max`, clamped to 1..=512) plus `next` and `total`.
pub fn collect_chronicle_since(chronicle: &Chronicle, start: i64, max: i64) -> ChronicleRows {
    let total = chronicle.len();
    let start = usize::try_from(start.max(0))
        .unwrap_or(usize::MAX)
        .min(total);
    let max = max.clamp(1, CHRONICLE_PAGE_MAX) as usize;
    let end = start.saturating_add(max).min(total);
    let mut rows = ChronicleRows::from_indices(
        chronicle,
        true,
        (start..end).map(|i| u32::try_from(i).unwrap_or(u32::MAX)),
    );
    rows.next = end as i64;
    rows.total = total as i64;
    rows
}

/// Marshal a [`PersonView`] as `{found, alive, female, born_tick, died_tick,
/// death_reason, mother, father, ns, ha, rd, p}`.
pub(super) fn person_to_dict(view: PersonView) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("found", view.found);
    dict.set("alive", view.alive);
    dict.set("female", view.female);
    dict.set("born_tick", view.born_tick);
    dict.set("died_tick", view.died_tick);
    dict.set("death_reason", view.death_reason);
    dict.set("mother", view.mother);
    dict.set("father", view.father);
    for (key, value) in ["ns", "ha", "rd", "p"].into_iter().zip(view.temperament) {
        dict.set(key, value);
    }
    dict
}

/// Marshal [`ChronicleRows`] as packed columns; `with_page` adds `next` and `total`.
pub(super) fn rows_to_dict(rows: ChronicleRows, with_page: bool) -> VarDictionary {
    let mut dict = VarDictionary::new();
    dict.set("found", rows.found);
    dict.set("tick", PackedInt64Array::from(rows.tick.as_slice()));
    dict.set("kind", PackedInt32Array::from(rows.kind.as_slice()));
    dict.set("subject", PackedInt64Array::from(rows.subject.as_slice()));
    dict.set("other", PackedInt64Array::from(rows.other.as_slice()));
    dict.set("third", PackedInt64Array::from(rows.third.as_slice()));
    dict.set(
        "settlement",
        PackedInt64Array::from(rows.settlement.as_slice()),
    );
    dict.set(
        "settlement2",
        PackedInt64Array::from(rows.settlement2.as_slice()),
    );
    dict.set("cause", PackedInt32Array::from(rows.cause.as_slice()));
    dict.set("value", PackedInt64Array::from(rows.value.as_slice()));
    dict.set("index", PackedInt64Array::from(rows.index.as_slice()));
    dict.set("link", PackedInt32Array::from(rows.link.as_slice()));
    dict.set(
        "link_entry",
        PackedInt64Array::from(rows.link_entry.as_slice()),
    );
    dict.set(
        "link_code",
        PackedInt32Array::from(rows.link_code.as_slice()),
    );
    if with_page {
        dict.set("next", rows.next);
        dict.set("total", rows.total);
    }
    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::chronicle::{ChronicleKind, PersonRecord};

    fn chronicle() -> Chronicle {
        let mut chronicle = Chronicle::new();
        // Founder 1 (no parents), founder 2, child 5 of 1 and 2; 2 dies.
        chronicle.ensure_person(1, || {
            Some(PersonRecord {
                female: Some(true),
                born_tick: Some(-9000),
                temperament: Some([100, 200, 300, 400]),
                ..PersonRecord::default()
            })
        });
        chronicle.ensure_person(2, || {
            Some(PersonRecord {
                female: Some(false),
                born_tick: Some(-8000),
                ..PersonRecord::default()
            })
        });
        chronicle.push(
            ChronicleEntry::new(0, ChronicleKind::SettlementFormed)
                .settlement(Some(1))
                .value(2),
        );
        chronicle.push(
            ChronicleEntry::new(0, ChronicleKind::FoundingMember)
                .subject(1)
                .settlement(Some(1)),
        );
        chronicle.push(
            ChronicleEntry::new(0, ChronicleKind::FoundingMember)
                .subject(2)
                .settlement(Some(1)),
        );
        chronicle.ensure_person(5, || {
            Some(PersonRecord {
                mother: Some(1),
                father: Some(2),
                born_tick: Some(70),
                ..PersonRecord::default()
            })
        });
        chronicle.push(
            ChronicleEntry::new(70, ChronicleKind::Born)
                .subject(5)
                .other(Some(1))
                .third(Some(2))
                .settlement(Some(1)),
        );
        chronicle.close_person(2, 90, 3, PersonRecord::default);
        chronicle.push(
            ChronicleEntry::new(90, ChronicleKind::Died)
                .subject(2)
                .other(Some(1))
                .settlement(Some(1))
                .cause(3)
                .value(8090),
        );
        chronicle.push(
            ChronicleEntry::new(90, ChronicleKind::Widowed)
                .subject(1)
                .other(Some(2)),
        );
        chronicle
    }

    #[test]
    fn agent_id_resolves_from_entity_bits() {
        let mut world = hecs::World::new();
        let entity = world.spawn((Agent { id: 7 },));
        let other = world.spawn((1u8,));
        assert_eq!(
            collect_agent_id(&world, signed(entity.to_bits().get())),
            Some(7)
        );
        assert_eq!(
            collect_agent_id(&world, signed(other.to_bits().get())),
            None
        );
        assert_eq!(collect_agent_id(&world, 0), None);
        world.despawn(entity).expect("despawn");
        assert_eq!(
            collect_agent_id(&world, signed(entity.to_bits().get())),
            None
        );
    }

    #[test]
    fn a_dead_person_keeps_their_record_and_history() {
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 1 },));
        let chronicle = chronicle();
        let dead = collect_person(&world, &chronicle, 2);
        assert!(dead.found && !dead.alive);
        assert_eq!(
            (
                dead.female,
                dead.born_tick,
                dead.died_tick,
                dead.death_reason
            ),
            (0, -8000, 90, 3)
        );
        assert_eq!(dead.temperament, [-1; 4]);
        let history = collect_person_chronicle(&chronicle, 2);
        assert!(history.found);
        assert_eq!(history.kind, vec![1, 0, 11, 10]);
        assert_eq!(history.tick, vec![0, 70, 90, 90]);
        assert_eq!(history.third, vec![-1, 2, -1, -1]);
    }

    #[test]
    fn a_founder_has_no_parents_and_a_child_names_both() {
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 1 },));
        let chronicle = chronicle();
        let founder = collect_person(&world, &chronicle, 1);
        assert!(founder.found && founder.alive);
        assert_eq!(
            (founder.mother, founder.father, founder.female),
            (-1, -1, 1)
        );
        assert_eq!(founder.temperament, [100, 200, 300, 400]);
        let child = collect_person(&world, &chronicle, 5);
        assert!(
            child.found && !child.alive,
            "no living entity with id 5 in this world"
        );
        assert_eq!(
            (child.mother, child.father, child.born_tick, child.died_tick),
            (1, 2, 70, -1)
        );
    }

    #[test]
    fn unknown_ids_are_not_found() {
        let world = hecs::World::new();
        let chronicle = chronicle();
        let nobody = collect_person(&world, &chronicle, 999);
        assert!(!nobody.found && !nobody.alive);
        assert_eq!(
            (nobody.female, nobody.born_tick, nobody.mother),
            (-1, -1, -1)
        );
        assert_eq!(
            collect_person_chronicle(&chronicle, 999),
            ChronicleRows::default()
        );
        assert!(!collect_settlement_chronicle(&chronicle, 42).found);
        assert!(!collect_settlement_chronicle(&chronicle, -1).found);
        assert!(!collect_person(&world, &chronicle, -1).found);
        let village = collect_settlement_chronicle(&chronicle, 1);
        assert!(village.found);
        assert_eq!(village.kind, vec![12, 1, 1, 0, 11]);
    }

    #[test]
    fn the_feed_clamps_its_page_and_reports_next_and_total() {
        let chronicle = chronicle();
        let all = collect_chronicle_since(&chronicle, 0, 10_000);
        assert_eq!((all.tick.len(), all.next, all.total), (6, 6, 6));
        let one = collect_chronicle_since(&chronicle, 2, 0);
        assert_eq!(
            (one.tick.len(), one.next),
            (1, 3),
            "max below 1 reads one entry"
        );
        let negative = collect_chronicle_since(&chronicle, -5, 2);
        assert_eq!((negative.kind, negative.next), (vec![12, 1], 2));
        let past = collect_chronicle_since(&chronicle, 99, 5);
        assert_eq!((past.tick.len(), past.next, past.total), (0, 6, 6));
        let mut big = Chronicle::new();
        for tick in 0..600 {
            big.push(ChronicleEntry::new(tick, ChronicleKind::CameOfAge).subject(tick));
        }
        let page = collect_chronicle_since(&big, 0, 600);
        assert_eq!((page.tick.len(), page.next, page.total), (512, 512, 600));
    }
}
