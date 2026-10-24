//! Read-only family lookup with entity-bit inputs and AgentId-valued links.

use godot::prelude::*;
use sim_core::components::{Agent, AgentId, BodyHealth, Kin, Position, Sex};

/// Family data for one existing agent; links use the snapshot's `agent_ids` domain.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AgentFamilyRow {
    /// The person's recorded sex, drawn once when their identity is created.
    pub female: bool,
    /// Recorded mother, retained even when that person is no longer alive.
    pub mother: Option<AgentId>,
    /// Recorded father, retained even when that person is no longer alive.
    pub father: Option<AgentId>,
    /// Recorded partner; this observer does not repair or clear simulation state.
    pub partner: Option<AgentId>,
    /// Living offspring of any life stage, sorted by AgentId.
    pub children: Vec<AgentId>,
}

/// Resolve `entity_bits` from snapshot `ids`, then collect stable AgentId links.
///
/// Invalid, stale, non-agent or pre-family entities return `None`. Children must
/// be live agents whose recorded mother or father is the selected person's id;
/// parent and partner links retain their recorded values, including absent ids.
/// A child's BodyHealth, when present, must not report death. The target lookup
/// itself requires Agent, Sex and Kin components regardless of pending death.
pub fn collect_agent_family(world: &hecs::World, entity_bits: u64) -> Option<AgentFamilyRow> {
    let entity = hecs::Entity::from_bits(entity_bits)?;
    let (agent_id, female, kin) = {
        let mut query = world.query_one::<(&Agent, &Sex, &Kin)>(entity).ok()?;
        let (agent, sex, kin) = query.get()?;
        (agent.id, sex.female, *kin)
    };
    let mut children: Vec<_> = world
        .query::<(&Agent, &Kin, Option<&BodyHealth>)>()
        .iter()
        .filter_map(|(_, (child, child_kin, health))| {
            (!health.is_some_and(BodyHealth::is_dead)
                && (child_kin.mother == Some(agent_id) || child_kin.father == Some(agent_id)))
            .then_some(child.id)
        })
        .collect();
    children.sort_unstable();
    Some(AgentFamilyRow {
        female,
        mother: kin.mother,
        father: kin.father,
        partner: kin.partner,
        children,
    })
}

/// Every person's recorded mother, father and partner (AgentId, `None` unknown or without `Kin`), one row per entity
/// with `Agent` and `Position` in `collect_agent_snapshot`'s order (M-ART2.1: the renderer groups people into
/// households for the LOD marker rings). Pure: reads components only.
pub fn collect_kin_columns(world: &hecs::World) -> Vec<[Option<AgentId>; 3]> {
    world
        .query::<(&Agent, &Position, Option<&Kin>)>()
        .iter()
        .map(|(_, (_, _, kin))| kin.map_or([None; 3], |k| [k.mother, k.father, k.partner]))
        .collect()
}

/// An AgentId as the snapshot's signed bits (-1 for none).
pub fn signed_or_none(id: Option<AgentId>) -> i64 {
    id.map_or(-1, |id| i64::from_ne_bytes(id.to_ne_bytes()))
}

/// Marshal exactly six keys; absent links are -1 and children are packed i64s.
/// AgentId bit patterns match the existing snapshot's signed `agent_ids` array.
pub(super) fn agent_family_to_dict(row: Option<AgentFamilyRow>) -> VarDictionary {
    let found = row.is_some();
    let row = row.unwrap_or_default();
    let signed_id = |id: AgentId| i64::from_ne_bytes(id.to_ne_bytes());
    let children: Vec<_> = row.children.into_iter().map(signed_id).collect();
    let mut dict = VarDictionary::new();
    dict.set("found", found);
    dict.set("female", row.female);
    dict.set("mother", row.mother.map_or(-1, signed_id));
    dict.set("father", row.father.map_or(-1, signed_id));
    dict.set("partner", row.partner.map_or(-1, signed_id));
    dict.set("children", PackedInt64Array::from(children.as_slice()));
    dict
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::Child;

    #[test]
    fn missing_family_returns_none_for_invalid_stale_and_incomplete_entities() {
        // Given invalid bits, partial components, a non-agent and a despawned id.
        let mut world = hecs::World::new();
        let old_scene = world.spawn((Agent { id: 10 },));
        let missing_kin = world.spawn((Agent { id: 11 }, Sex { female: true }));
        let missing_sex = world.spawn((Agent { id: 12 }, Kin::default()));
        let non_agent = world.spawn((Sex { female: true }, Kin::default()));
        let stale = world.spawn((Agent { id: 13 }, Sex { female: false }, Kin::default()));
        world.despawn(stale).expect("despawn fixture");
        world.spawn((Agent { id: 14 }, Sex { female: true }, Kin::default()));
        // When querying each missing or incomplete family.
        for bits in [
            0,
            u64::MAX,
            old_scene.to_bits().get(),
            missing_kin.to_bits().get(),
            missing_sex.to_bits().get(),
            non_agent.to_bits().get(),
            stale.to_bits().get(),
        ] {
            // Then no row is fabricated, even when an entity slot is reused.
            assert_eq!(collect_agent_family(&world, bits), None);
        }
    }

    #[test]
    fn founder_family_has_no_fabricated_links() {
        // Given a founder with no recorded ancestry or partner.
        let mut world = hecs::World::new();
        let founder = world.spawn((Agent { id: 70 }, Sex { female: true }, Kin::default()));
        // When reading their family through entity bits.
        let row = collect_agent_family(&world, founder.to_bits().get());
        // Then only the recorded sex is populated.
        assert_eq!(
            row,
            Some(AgentFamilyRow {
                female: true,
                ..AgentFamilyRow::default()
            })
        );
    }

    #[test]
    fn links_are_agent_ids_and_preserve_absent_recorded_relatives() {
        // Given a person whose recorded relatives are not alive in this world.
        let mut world = hecs::World::new();
        let person = world.spawn((
            Agent { id: 700 },
            Sex { female: false },
            Kin {
                mother: Some(100),
                father: Some(200),
                partner: Some(800),
                last_birth_tick: Some(123),
            },
        ));
        assert_ne!(person.to_bits().get(), 700);
        // When looking up the entity-bit key, rather than its simulation id.
        let row = collect_agent_family(&world, person.to_bits().get());
        // Then links remain AgentIds; absent relatives do not become unknown.
        assert_eq!(
            row,
            Some(AgentFamilyRow {
                female: false,
                mother: Some(100),
                father: Some(200),
                partner: Some(800),
                children: vec![],
            })
        );
        assert_eq!(collect_agent_family(&world, 700), None);
    }

    #[test]
    fn children_are_living_offspring_of_either_parent_sorted_across_life_stages() {
        // Given offspring in both parent slots and different archetypes.
        let mut world = hecs::World::new();
        let parent = world.spawn((Agent { id: 500 }, Sex { female: true }, Kin::default()));
        let maternal = Kin {
            mother: Some(500),
            ..Kin::default()
        };
        let paternal = Kin {
            father: Some(500),
            ..Kin::default()
        };
        world.spawn((Agent { id: 30 }, maternal, Child { born_tick: 0 }));
        world.spawn((Agent { id: 10 }, paternal));
        world.spawn((
            Agent { id: 20 },
            Kin {
                father: Some(500),
                ..maternal
            },
        ));
        world.spawn((Agent { id: 40 }, Kin::default()));
        world.spawn((maternal,));
        let dead_child = world.spawn((Agent { id: 5 }, maternal));
        world
            .despawn(dead_child)
            .expect("despawn offspring fixture");
        let size_before = world.len();
        // When collecting twice from the unchanged world.
        let first = collect_agent_family(&world, parent.to_bits().get()).expect("parent family");
        let second = collect_agent_family(&world, parent.to_bits().get()).expect("parent family");
        // Then each living offspring appears once, including an adult child.
        assert_eq!(first.children, vec![10, 20, 30]);
        assert_eq!(first, second);
        assert_eq!(world.len(), size_before);
        assert_eq!(
            *world.get::<&Kin>(parent).expect("parent Kin"),
            Kin::default()
        );
    }

    #[test]
    fn pending_dead_children_are_excluded_without_changing_target_lookup() {
        // Given present dead and healthy offspring, plus one without BodyHealth.
        let mut world = hecs::World::new();
        let mut dead_health = BodyHealth::new();
        dead_health.apply_damage(dead_health.max_hp);
        let parent = world.spawn((
            Agent { id: 500 },
            Sex { female: true },
            Kin::default(),
            dead_health,
        ));
        let kin = Kin {
            mother: Some(500),
            ..Kin::default()
        };
        let pending_dead = world.spawn((Agent { id: 10 }, kin, dead_health));
        world.spawn((Agent { id: 20 }, kin, BodyHealth::new()));
        world.spawn((Agent { id: 30 }, kin));
        // When collecting a still-present target, even if its own death is pending.
        let row = collect_agent_family(&world, parent.to_bits().get()).expect("present target");
        // Then dead children are omitted without removing or changing any entity.
        assert_eq!(row.children, vec![20, 30]);
        assert!(world.contains(pending_dead));
        assert!(world
            .get::<&BodyHealth>(pending_dead)
            .expect("child health")
            .is_dead());
    }
}
