//! Format22 upper-mask roundtrips and genuine pre-ore format21 migration.
use super::*;
use sim_core::colony::knowledge::{bit, ids_of, Knowledge, KnowledgeMask, Source};
use sim_core::colony::neighbours::{Edge, Neighbour};

fn bootstrap() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundled catalog");
}

#[test]
fn current_save_preserves_authoritative_neighbour_and_derived_practitioner_upper_bits() {
    bootstrap();
    // Given independent knowledge at every word boundary, including unused authored slots.
    let ids = [0, 63, 64, 127, 128, 191, 192, 255];
    let mask = ids
        .into_iter()
        .fold(KnowledgeMask::ZERO, |mask, id| mask | bit(id));
    let mut engine = sim_engine::SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    let mut knowledge = Knowledge::default();
    for id in ids {
        knowledge.set(id, 1000, Source::Migrant, 0);
    }
    let entity = engine.world.spawn((knowledge,));
    engine.resources.colony.neighbours.list.push(Neighbour {
        id: 7,
        name_key: 0,
        edge: Edge::North,
        distance_days: 2,
        population: 10,
        attitude: 50,
        known: mask,
        last_contact_day: None,
    });
    // When the actual save boundary writes and reloads format22.
    let bytes = super::super::save(&mut engine).expect("save22");
    let mut restored = super::super::load_with(&bytes, |_| {}).expect("load22");
    // Then both authoritative and rebuilt masks survive without aliases or truncation.
    assert_eq!(restored.resources.colony.neighbours.list[0].known, mask);
    let restored_knowledge = restored
        .world
        .get::<&Knowledge>(entity)
        .expect("same entity");
    assert_eq!(
        ids_of(restored_knowledge.practice()).collect::<Vec<_>>(),
        ids
    );
    drop(restored_knowledge);
    assert_eq!(super::super::save(&mut restored).expect("stable22"), bytes);
    let encoded = super::super::encode(&mask).expect("standalone mask");
    assert_eq!(
        super::super::decode::<KnowledgeMask>(&encoded).expect("full binary mask"),
        mask
    );
}

#[test]
fn upper_tail_rejects_duplicate_unordered_zero_and_orphan_rows() {
    #[derive(Deserialize)]
    struct Tail(
        #[serde(deserialize_with = "super::super::knowledge_upper::deserialize")]
        super::super::knowledge_upper::UpperWords,
    );
    // Given malformed rows at the versioned serialization boundary.
    for rows in [
        vec![(1_u8, [1_u64, 0]), (1, [2, 0])],
        vec![(2, [1, 0]), (1, [2, 0])],
        vec![(1, [0, 0])],
    ] {
        // When decoding into the ordered map, duplicate and noncanonical rows reject.
        let wire = super::super::encode(&rows).expect("malformed wire fixture");
        assert!(super::super::decode::<Tail>(&wire).is_err());
    }
    let wire = super::super::encode(&vec![(7_u8, [1_u64, 0])]).expect("orphan wire");
    let tail: Tail = super::super::decode(&wire).expect("ordered row");
    let mut resources =
        sim_engine::SimEngine::new(8, 8, sim_core::MaterialRegistry::new()).resources;
    // Then a structurally valid row still cannot invent its neighbour.
    assert!(super::super::knowledge_upper::restore(&mut resources, tail.0).is_err());
}

#[test]
fn only_authenticated_pre22_ore_slot_is_inserted() {
    let registered = vec![
        ("Before".into(), vec![1]),
        ("OreDiscoverySystem".into(), vec![]),
        ("After".into(), vec![2]),
    ];
    let old = vec![registered[0].clone(), registered[2].clone()];
    let additions = |ore| super::super::LegacySystemAdditions {
        steward: false,
        defence: false,
        food: false,
        buildings: false,
        livestock: false,
        ore,
        diagnosis: false,
    };
    let mut restored = old.clone();
    super::super::insert_legacy_systems(&mut restored, &registered, additions(true))
        .expect("historical slot");
    assert_eq!(restored, registered);
    assert!(
        super::super::insert_legacy_systems(&mut restored, &registered, additions(true)).is_err()
    );
    let mut current_missing = old.clone();
    super::super::insert_legacy_systems(&mut current_missing, &registered, additions(false))
        .expect("no current repair");
    assert_eq!(current_missing, old);
}

#[test]
fn format21_requires_exact_published_revision_and_preserved_identity_prefix() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    assert!(accepts_catalog(REVISION, &catalog).expect("genuine published revision"));
    assert!(!accepts_catalog(catalog.revision(), &catalog).expect("current cannot be relabelled21"));
    assert!(!accepts_catalog("unknown21", &catalog).expect("unknown revision"));
}

#[test]
fn genuine_base21_keeps_history_and_starts_with_empty_ore() {
    let Ok(path) = std::env::var("C2_BASE_SAVE_PATH") else {
        return;
    };
    bootstrap();
    // Given the retained unmodified C1-b binary's actual format21 output.
    let bytes = std::fs::read(path).expect("genuine BASE21 fixture");
    assert_eq!(&bytes[8..12], &21_u32.to_le_bytes());
    let envelope: super::super::Envelope =
        super::super::decode(super::super::read_header(&bytes).expect("header")).expect("envelope");
    assert_eq!(envelope.revision, REVISION);
    let old: LegacyBodyV21 = super::super::decode(&envelope.payload).expect("genuine old body");
    // When the production loader authenticates and migrates it.
    let mut restored = super::super::load(&bytes).expect("genuine21 migration");
    // Then no ore is invented, old system state/history survives, and current saves are stable.
    assert_eq!(restored.current_tick(), old.current_tick);
    assert_eq!(
        restored.resources.ore,
        sim_core::colony::ore::OreState::default()
    );
    assert_eq!(
        super::super::encode(&restored.resources.chronicle).expect("restored history"),
        super::super::encode(&old.resources.chronicle).expect("original history")
    );
    let slots = restored.system_states().expect("systems");
    assert_eq!(
        slots
            .iter()
            .filter(|row| row.0 == "OreDiscoverySystem")
            .count(),
        1
    );
    assert_eq!(
        slots
            .into_iter()
            .filter(|row| row.0 != "OreDiscoverySystem")
            .collect::<Vec<_>>(),
        old.systems
    );
    let current = super::super::save(&mut restored).expect("migrated22 save");
    assert_eq!(&current[8..12], &super::super::FORMAT_VERSION.to_le_bytes());
    let mut roundtrip = super::super::load(&current).expect("current22 load");
    assert_eq!(
        super::super::save(&mut roundtrip).expect("immediate22 save"),
        current
    );
    for _ in 0..60 {
        restored.tick();
        roundtrip.tick();
    }
    assert_eq!(
        super::super::save(&mut restored).expect("continued migration"),
        super::super::save(&mut roundtrip).expect("continued restored")
    );
}
