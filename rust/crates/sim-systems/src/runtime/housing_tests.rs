//! Unit tests for HOUSE1 households: founding, residence rules (data-only), orphans.
use super::*;
use sim_core::components::{Agent, Child, Kin, Life, Settlement, Sex};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;

fn fixture(people: &[(AgentId, bool, bool, Kin)]) -> (World, SimResources) {
    let mut world = World::new();
    let mut res = SimResources::new(32, 32, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    let mut settlement = Settlement::new_with_id(1, 0);
    for &(id, female, adult, kin) in people {
        let life = Life {
            born_tick: -(i64::try_from(id).unwrap_or(0)) * 10,
            lifespan_ticks: u64::MAX,
        };
        if adult {
            world.spawn((Agent { id }, kin, Sex { female }, life));
        } else {
            world.spawn((
                Agent { id },
                kin,
                Sex { female },
                life,
                Child { born_tick: 0 },
            ));
        }
        settlement.add_member_agent(id);
    }
    res.settlements.insert(1, settlement);
    (world, res)
}

fn partner(of: AgentId) -> Kin {
    Kin {
        partner: Some(of),
        ..Kin::default()
    }
}

fn set_partner(world: &mut World, id: AgentId, partner: AgentId) {
    for (_, (agent, kin)) in world.query_mut::<(&Agent, &mut Kin)>() {
        if agent.id == id {
            kin.partner = Some(partner);
        }
    }
}

fn kinds(res: &SimResources, kind: ChronicleKind) -> usize {
    res.chronicle
        .entries()
        .iter()
        .filter(|entry| entry.kind == kind)
        .count()
}

#[test]
fn founding_groups_families_without_events_and_heads_the_oldest_adult() {
    let child = Kin {
        mother: Some(1),
        father: Some(2),
        ..Kin::default()
    };
    let (world, mut res) = fixture(&[
        (1, true, true, partner(2)),
        (2, false, true, partner(1)),
        (3, false, true, Kin::default()),
        (9, true, false, child),
    ]);
    reconcile_with(&world, &mut res, ResidenceRule::Neolocal);
    let state = &res.colony.buildings.housing;
    assert!(state.founded);
    assert_eq!(state.households.len(), 2);
    let family = &state.households[&HouseholdId(0)];
    assert_eq!(family.members, BTreeSet::from([1, 2, 9]));
    // Born earliest = highest id in this fixture.
    assert_eq!(family.head, 2);
    assert_eq!(res.chronicle.entries().len(), 0, "founding emits nothing");
}

#[test]
fn neolocal_pairing_forms_a_new_household_with_the_lower_parent() {
    let (mut world, mut res) = fixture(&[
        (1, true, true, Kin::default()),
        (2, false, true, Kin::default()),
    ]);
    reconcile_with(&world, &mut res, ResidenceRule::Neolocal);
    set_partner(&mut world, 1, 2);
    set_partner(&mut world, 2, 1);
    reconcile_with(&world, &mut res, ResidenceRule::Neolocal);
    let state = &res.colony.buildings.housing;
    assert_eq!(state.households.len(), 1);
    let couple = &state.households[&HouseholdId(2)];
    assert_eq!(couple.members, BTreeSet::from([1, 2]));
    assert_eq!(couple.parent, Some(HouseholdId(0)));
    assert_eq!(kinds(&res, ChronicleKind::HouseholdFormed), 1);
}

#[test]
fn residence_rule_from_data_moves_the_wife_or_the_husband_without_a_new_id() {
    for (rule, stays) in [
        (ResidenceRule::Patrilocal, 2),
        (ResidenceRule::Matrilocal, 1),
    ] {
        let (mut world, mut res) = fixture(&[
            (1, true, true, Kin::default()),
            (2, false, true, Kin::default()),
        ]);
        reconcile_with(&world, &mut res, rule);
        let home = res.colony.buildings.housing.household_of(stays);
        set_partner(&mut world, 1, 2);
        set_partner(&mut world, 2, 1);
        reconcile_with(&world, &mut res, rule);
        let state = &res.colony.buildings.housing;
        assert_eq!(state.households.len(), 1, "{rule:?}");
        assert_eq!(state.household_of(1), home);
        assert_eq!(state.household_of(2), home);
        assert_eq!(state.next_household, 2, "{rule:?}: no new id");
        assert_eq!(kinds(&res, ChronicleKind::HouseholdFormed), 0);
    }
}

#[test]
fn orphaned_minors_join_their_grandparents_household() {
    let parent = Kin {
        mother: Some(5),
        ..Kin::default()
    };
    let orphan = Kin {
        mother: Some(3),
        ..Kin::default()
    };
    let (mut world, mut res) = fixture(&[
        (5, true, true, Kin::default()),
        (3, true, true, parent),
        (9, false, false, orphan),
    ]);
    reconcile_with(&world, &mut res, ResidenceRule::Neolocal);
    let state = &res.colony.buildings.housing;
    let grand = state.household_of(5);
    // Founding put 3 (adult daughter, no partner) on her own with child 9.
    assert_ne!(state.household_of(3), grand);
    assert_eq!(state.household_of(9), state.household_of(3));
    let dead: Vec<_> = world
        .query::<&Agent>()
        .iter()
        .filter(|(_, agent)| agent.id == 3)
        .map(|(entity, _)| entity)
        .collect();
    for entity in dead {
        world.despawn(entity).expect("despawn");
    }
    // Death keeps the person record (the death path writes it before despawn).
    res.chronicle.ensure_person(3, || {
        Some(sim_core::chronicle::PersonRecord {
            mother: Some(5),
            ..Default::default()
        })
    });
    reconcile_with(&world, &mut res, ResidenceRule::Neolocal);
    let state = &res.colony.buildings.housing;
    assert_eq!(state.household_of(9), grand);
    assert!(state.validate().is_ok());
}

#[test]
fn heirless_minors_keep_their_household_and_never_claim() {
    let orphan = Kin {
        mother: Some(3),
        ..Kin::default()
    };
    let (mut world, mut res) =
        fixture(&[(3, true, true, Kin::default()), (9, false, false, orphan)]);
    reconcile_with(&world, &mut res, ResidenceRule::Neolocal);
    let home = res.colony.buildings.housing.household_of(9);
    assert!(home.is_some());
    let dead: Vec<_> = world
        .query::<&Agent>()
        .iter()
        .filter(|(_, agent)| agent.id == 3)
        .map(|(entity, _)| entity)
        .collect();
    for entity in dead {
        world.despawn(entity).expect("despawn");
    }
    let before = res.colony.buildings.housing.revision;
    reconcile_with(&world, &mut res, ResidenceRule::Neolocal);
    let state = &res.colony.buildings.housing;
    assert_eq!(
        state.household_of(9),
        home,
        "no kin: the minor keeps the household"
    );
    assert_eq!(state.households[&home.expect("home")].head, 9);
    assert!(state.validate().is_ok());
    assert!(
        state.revision > before,
        "a membership change moves the revision"
    );
    assert_eq!(
        housing_demand(&world, &res),
        0,
        "an adult-less household is no demand"
    );
}
