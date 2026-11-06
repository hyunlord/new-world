//! Format17 physical wear, passive processing and genuine predecessor migration regressions.
use super::*;
use sim_core::colony::people::{Apparel, Top};
use sim_core::colony::processing::PassiveBatch;
use sim_core::colony::{Carry, ItemKind, ItemSpec, ItemStack, Tool};

#[test]
fn format17_roundtrip_preserves_exact_wear_in_all_physical_owners() {
    // Given distinct finite wear values in each independently serialized owner.
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    let item = ItemSpec {
        wear_remaining_hours: Some(731),
        quality: 63,
        ..ItemSpec::raw(ItemKind::HideWrap, None)
    };
    engine
        .resources
        .colony
        .ground_items
        .insert((1, 1), ItemStack::of(item, 1));
    let carry = Carry::of(
        ItemSpec {
            wear_remaining_hours: Some(729),
            ..item
        },
        1,
    );
    let tool = Tool {
        item: ItemSpec {
            wear_remaining_hours: Some(727),
            ..item
        },
        uses: 9,
    };
    let apparel = Apparel {
        top: Top::HideWrap,
        since_day: 7,
        material: item.material,
        quality: item.quality,
        wear_remaining_hours: Some(725),
    };
    let owner = engine.world.spawn((carry, tool, apparel));
    // When a real save is loaded and saved again.
    let bytes = save(&mut engine).expect("save format17");
    let mut loaded = load_with(&bytes, |_| {}).expect("load format17");
    // Then each exact condition survives, with stable bytes and entity identities.
    assert_eq!(loaded.resources.colony.ground_items[&(1, 1)].spec(), item);
    assert_eq!(*loaded.world.get::<&Carry>(owner).expect("carry"), carry);
    assert_eq!(*loaded.world.get::<&Tool>(owner).expect("tool"), tool);
    assert_eq!(
        *loaded.world.get::<&Apparel>(owner).expect("apparel"),
        apparel
    );
    assert_eq!(apparel.spec().wear_remaining_hours, Some(725));
    assert_eq!(save(&mut loaded).expect("resave"), bytes);
}

fn engine_with_pending_processing() -> SimEngine {
    use sim_core::colony::Workshop;
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    let recipe = sim_core::colony::recipes()
        .iter()
        .find(|recipe| recipe.process.is_some())
        .expect("authored passive recipe");
    let process = recipe.process.as_ref().expect("process");
    for (id, tile) in [(1, (2, 2)), (2, (4, 2))] {
        let shop = Workshop {
            id,
            kind: recipe.workshop,
            top_left: tile,
            size: (1, 1),
            bills: vec![],
            shelf: vec![],
            active: None,
        };
        let output = ItemSpec::raw(recipe.output.0, None);
        engine.resources.processing.pending.push(PassiveBatch {
            workshop: id,
            bill: id,
            tile: shop.work_tile(),
            output,
            count: recipe.output.1,
            remaining_ticks: if id == 1 {
                engine
                    .resources
                    .rules
                    .calendar
                    .days_to_ticks(u64::from(process.duration_days))
                    - 1
            } else {
                0
            },
            requires_river: process.requires_river,
        });
        engine.resources.colony.workshops.insert(id, shop);
    }
    engine.resources.colony.next_workshop_id = 3;
    engine.resources.colony.next_bill_id = 3;
    engine
}

#[test]
fn processing_tail_roundtrip_preserves_qualifying_time_and_blocked_ready_output() {
    // Given one wet-time batch and one completed-but-not-yet-placed batch.
    let mut engine = engine_with_pending_processing();
    let expected = engine.resources.processing.clone();
    // When the explicit format17 tail is round-tripped without ticking.
    let bytes = save(&mut engine).expect("save processing tail");
    let mut loaded = load_with(&bytes, |_| {}).expect("load processing tail");
    // Then no wall-clock progress, output emission or condition reset occurs.
    assert_eq!(loaded.resources.processing, expected);
    assert!(loaded.resources.colony.ground_items.is_empty());
    assert_eq!(save(&mut loaded).expect("resave"), bytes);
}

#[test]
fn processing_save_rejects_unauthored_output_duration_and_overcapacity() {
    // Given otherwise valid pending state, corrupt one authoritative property.
    for defect in 0..5 {
        let mut engine = engine_with_pending_processing();
        let valid_bytes = save(&mut engine).expect("valid precursor");
        match defect {
            0 => engine.resources.processing.pending[0].count = 0,
            1 => engine.resources.processing.pending[0].remaining_ticks = u64::MAX,
            2 => {
                engine.resources.processing.pending[0].requires_river =
                    !engine.resources.processing.pending[0].requires_river
            }
            3 => engine.resources.processing.pending[0].output.kind = ItemKind::Stone,
            4 => engine
                .resources
                .processing
                .pending
                .push(engine.resources.processing.pending[0].clone()),
            _ => unreachable!(),
        }
        // Corrupt the explicit tail independently of the guarded writer.
        let mut envelope: Envelope = decode(&valid_bytes[12..]).unwrap();
        let (mut body, thermal) = thermal_codec::decode_test(&envelope.payload).unwrap();
        body.processing = engine.resources.processing.clone();
        envelope.payload = thermal_codec::encode_test(&body, &thermal).unwrap();
        let mut corrupt = valid_bytes[..12].to_vec();
        corrupt.extend(encode(&envelope).unwrap());
        assert!(matches!(
            load_with(&corrupt, |_| {}),
            Err(SaveError::Decode(_))
        ));
        // When encoding the corrupt candidate.
        let result = save(&mut engine);
        // Then the save boundary rejects it instead of authorizing output creation.
        assert!(
            matches!(result, Err(SaveError::Encode(_))),
            "defect {defect}"
        );
    }
}

#[test]
fn processing_removed_bill_and_destroyed_workshop_survive_command_boundary_save() {
    // Given consumed inputs after bill removal and one destroyed workshop.
    let mut engine = engine_with_pending_processing();
    engine.resources.colony.workshops.remove(&1);
    let expected = engine.resources.processing.clone();
    // When saved at that command boundary, before the next executor tick.
    let bytes = save(&mut engine).expect("save pending loss");
    let mut loaded = load_with(&bytes, |_| {}).expect("load pending loss");
    // Then pending ownership remains exact; advancing consumes the loss only once.
    assert_eq!(loaded.resources.processing, expected);
    loaded
        .resources
        .processing
        .pending
        .retain(|batch| batch.workshop == 1);
    let lost = loaded.resources.colony.tallies.crafts_lost;
    crate::runtime::jobs::processing::advance(&mut loaded.resources);
    crate::runtime::jobs::processing::advance(&mut loaded.resources);
    assert!(loaded.resources.processing.pending.is_empty());
    assert!(loaded.resources.colony.ground_items.is_empty());
    assert_eq!(loaded.resources.colony.tallies.crafts_lost, lost + 1);
}

#[test]
fn arbitrary_predecessor_revision_cannot_use_format16_admission() {
    // Given the installed bundled catalog and an unrecognized revision.
    let catalog = sim_core::colony::content::Catalog::from_sources(
        sim_core::colony::content::RawSources::bundled(),
        None,
    )
    .expect("bundled catalog");
    // When the narrowly bounded old-reader admission is asked.
    let accepted = legacy_weaving_v16::accepts_catalog("unknown-format16-catalog", &catalog)
        .expect("admission");
    // Then an unrelated revision cannot acquire old-reader privileges.
    assert!(!accepted);
}

#[test]
#[ignore = "mandatory external predecessor suite: WORLDSIM_M6B_FORMAT16_FIXTURES"]
fn genuine_format16_physical_and_pending_fixtures_migrate_without_loss() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("installed catalog before historical decode");
    // Given genuine bytes exported before production format/type edits.
    let root = std::path::PathBuf::from(
        std::env::var_os("WORLDSIM_M6B_FORMAT16_FIXTURES").expect("external fixture root"),
    );
    let physical = std::fs::read(root.join("format16-physical-owners.wsave"))
        .expect("genuine physical fixture");
    assert_eq!(&physical[8..12], &16u32.to_le_bytes());
    // When resources and all three opaque ECS physical columns migrate.
    let mut loaded = load_with(&physical, crate::register_default_runtime_systems)
        .expect("migrate physical fixture");
    // Then historical physical properties survive and new mechanics are inert.
    assert_eq!(loaded.resources.colony.ground_items[&(20, 20)].count, 3);
    assert!(loaded
        .resources
        .colony
        .ground_items
        .values()
        .all(|item| item.wear_remaining_hours.is_none()));
    let carried: Vec<_> = loaded
        .world
        .query::<&Carry>()
        .iter()
        .map(|(_, item)| *item)
        .collect();
    assert_eq!(carried.len(), 1);
    assert_eq!(carried[0].count, 2);
    assert_eq!(carried[0].wear_remaining_hours, None);
    let tools: Vec<_> = loaded
        .world
        .query::<&Tool>()
        .iter()
        .map(|(_, item)| *item)
        .collect();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].uses, 3);
    assert_eq!(tools[0].item.wear_remaining_hours, None);
    let mut worn: Vec<_> = loaded
        .world
        .query::<&Apparel>()
        .iter()
        .map(|(_, item)| *item)
        .collect();
    worn.sort_by_key(|item| item.top);
    assert_eq!(worn.len(), 2);
    assert_eq!([worn[0].top, worn[1].top], [Top::HideWrap, Top::FurCloak]);
    assert!(worn.iter().all(|item| item.since_day == 1
        && item.material.is_none()
        && item.quality == 50
        && item.wear_remaining_hours.is_none()));
    assert_eq!(loaded.resources.colony.workshops.len(), 1);
    assert_eq!(
        loaded
            .resources
            .colony
            .workshops
            .values()
            .next()
            .expect("workshop")
            .shelf
            .len(),
        2
    );
    assert!(loaded.resources.processing.pending.is_empty());
    crate::runtime::jobs::invariants::check(&loaded.world, &loaded.resources)
        .expect("historical physical ledger");
    let current = save(&mut loaded).expect("save migrated format17");
    let mut again =
        load_with(&current, crate::register_default_runtime_systems).expect("reload current");
    assert_eq!(save(&mut again).expect("stable current bytes"), current);

    // Given the separately exported synthetic craft/origin predecessor fixture.
    let craft =
        std::fs::read(root.join("format16-craft-origins.wsave")).expect("genuine craft fixture");
    assert_eq!(&craft[8..12], &16u32.to_le_bytes());
    // When its old food output spec and cultivation tail migrate.
    let mut loaded = load_with(&craft, |_| {}).expect("migrate pending fixture");
    // Then active work, passive drying and origin accounting remain represented.
    assert_eq!(loaded.resources.food.pending_drying.len(), 1);
    assert_eq!(loaded.resources.food.pending_drying[0].count, 2);
    assert_eq!(loaded.resources.food.pending_drying[0].due_tick, 100);
    assert_eq!(
        loaded.resources.food.pending_drying[0]
            .output
            .wear_remaining_hours,
        None
    );
    assert_eq!(loaded.resources.cultivation.drying_origins.len(), 1);
    assert_eq!(loaded.resources.cultivation.craft_origins.len(), 1);
    assert_eq!(
        loaded
            .world
            .query::<&sim_core::colony::Job>()
            .iter()
            .count(),
        1
    );
    assert!(loaded.resources.processing.pending.is_empty());
    let current = save(&mut loaded).expect("save migrated pending");
    let mut again = load_with(&current, |_| {}).expect("reload pending");
    assert_eq!(save(&mut again).expect("stable pending bytes"), current);
}
