//! Fixed-name overlays preserve the public observation contract.
use super::*;
use std::mem::size_of;

fn context() -> TickContext {
    let mut world = World::new();
    for id in [42, 7] {
        world.spawn((Agent { id }, Position::new(1, 1), AgentState::Idle));
    }
    let resources = SimResources::new(4, 4, sim_core::MaterialRegistry::new());
    TickContext::capture(&world, &resources).expect("context")
}
const DERIVED_KEYS: [&str; 16] = [
    "social_candidate",
    "social_memory_candidate",
    "social_handshake",
    "routine_admission",
    "invited",
    "routine_deadline",
    "activity_deadline_reached",
    "routine_weighted_place_available",
    "routine_available",
    "routine_tend_fire_available",
    "routine_craft_available",
    "routine_repair_available",
    "routine_hearth_circle_available",
    "routine_explore_available",
    "routine_play_available",
    "routine_sit_by_fire_available",
];
const TARGET_KEYS: [&str; 17] = [
    "food",
    "water",
    "sleep",
    "gather_food",
    "forage_bush",
    "refuge",
    "job",
    "hearth",
    "social_peer",
    "social_memory_peer",
    "routine_tend_fire",
    "routine_craft",
    "routine_repair",
    "routine_hearth_circle",
    "routine_explore",
    "routine_play",
    "routine_sit_by_fire",
];

#[test]
fn all_slots_are_distinct_lazy_and_isolated_by_sorted_person_row() {
    let mut ctx = context();
    assert!(ctx.derived_slots.iter().all(Vec::is_empty));
    assert!(ctx.target_slots.iter().all(Vec::is_empty));
    for (slot, name) in DERIVED_KEYS.iter().enumerate() {
        assert_eq!(derived_slot(name), Some(slot));
        ctx.set_derived(42, name, slot as i64, &[]).expect("write");
        assert_eq!(ctx.derived(42, name), Ok(slot as i64));
        assert!(matches!(
            ctx.derived(7, name),
            Err(ContextError::MissingDerived { .. })
        ));
        assert!(ctx.derived_slots[slot + 1..].iter().all(Vec::is_empty));
        ctx.set_derived(7, name, -1, &[]).expect("write other row");
        assert_eq!(ctx.derived(42, name), Ok(slot as i64));
    }
    for (slot, name) in TARGET_KEYS.iter().enumerate() {
        assert_eq!(target_slot(name), Some(slot));
        ctx.set_target(42, name, None, &[]).expect("write absence");
        assert_eq!(ctx.target(42, name), Ok(None));
        assert!(matches!(
            ctx.target(7, name),
            Err(ContextError::MissingDerived { .. })
        ));
        assert!(ctx.target_slots[slot + 1..].iter().all(Vec::is_empty));
    }
    assert!(ctx.derived.is_empty());
    assert!(ctx.targets.is_empty());
    let fresh = context();
    assert!(fresh.derived_slots.iter().all(Vec::is_empty));
    assert!(fresh.target_slots.iter().all(Vec::is_empty));
}

#[test]
fn known_and_dynamic_names_preserve_errors_and_overwrite_semantics() {
    let mut ctx = context();
    for name in ["social_candidate", "arbitrary-derived"] {
        assert_eq!(
            ctx.set_derived(99, name, 1, &[]),
            Err(ContextError::UnknownAgent(99))
        );
        assert_eq!(
            ctx.derived(99, name),
            Err(ContextError::MissingDerived {
                agent: 99,
                name: name.into()
            })
        );
        ctx.set_derived(7, name, 1, &[Domain::Person])
            .expect("write");
        ctx.invalidate(Domain::Person).expect("invalidate");
        assert_eq!(
            ctx.derived(7, name),
            Err(ContextError::StaleDerived {
                agent: 7,
                name: name.into()
            })
        );
        ctx.set_derived(7, name, 2, &[]).expect("overwrite");
        assert_eq!(ctx.derived(7, name), Ok(2));
    }
    for name in ["food", "arbitrary-target", "social_current_peer"] {
        assert_eq!(
            ctx.set_target(99, name, None, &[]),
            Err(ContextError::UnknownAgent(99))
        );
        assert_eq!(
            ctx.target(99, name),
            Err(ContextError::MissingDerived {
                agent: 99,
                name: name.into()
            })
        );
        let target = TargetCandidate {
            tile: (2, 3),
            agent: Some(42),
        };
        ctx.set_target(7, name, Some(target), &[Domain::Claims])
            .expect("write");
        assert_eq!(ctx.target(7, name), Ok(Some(target)));
        ctx.invalidate(Domain::Claims).expect("invalidate");
        assert_eq!(
            ctx.target(7, name),
            Err(ContextError::StaleDerived {
                agent: 7,
                name: name.into()
            })
        );
        ctx.set_target(7, name, None, &[]).expect("absence");
        assert_eq!(ctx.target(7, name), Ok(None));
        ctx.set_target(7, name, Some(target), &[])
            .expect("present again");
        assert_eq!(ctx.target(7, name), Ok(Some(target)));
    }
    assert_eq!(target_slot("social_current_peer"), None);
    assert!(ctx.targets[&7].contains_key("social_current_peer"));
}

#[test]
fn slot_stamps_preserve_all_domains_empty_masks_and_duplicate_dependencies() {
    let domains = [
        Domain::Person,
        Domain::Claims,
        Domain::FoodSources,
        Domain::WaterSources,
        Domain::SleepSources,
        Domain::RoutineOccupancy,
        Domain::Invitations,
        Domain::Jobs,
        Domain::Threats,
    ];
    for dependency in domains {
        for changed in domains {
            let mut ctx = context();
            ctx.set_derived(7, "social_candidate", 5, &[dependency, dependency])
                .expect("derived");
            ctx.set_target(7, "food", None, &[dependency, dependency])
                .expect("target");
            ctx.set_derived(42, "social_candidate", 6, &[])
                .expect("frozen derived");
            ctx.set_target(42, "food", None, &[])
                .expect("frozen target");
            ctx.invalidate(changed).expect("invalidate");
            assert_eq!(
                ctx.derived(7, "social_candidate").is_ok(),
                dependency != changed
            );
            assert_eq!(ctx.target(7, "food").is_ok(), dependency != changed);
            assert_eq!(ctx.derived(42, "social_candidate"), Ok(6));
            assert_eq!(ctx.target(42, "food"), Ok(None));
        }
    }
}

#[test]
fn report_overlay_slot_layout() {
    println!("OVERLAY_SLOT_LAYOUT versioned_derived={} optional_derived={} target_candidate={} versioned_target={} optional_target={} column_headers={} fully_populated_bytes_per_person={} tick_context={}",
        size_of::<Versioned<i64>>(), size_of::<Option<Versioned<i64>>>(),
        size_of::<TargetCandidate>(), size_of::<Versioned<Option<TargetCandidate>>>(),
        size_of::<Option<Versioned<Option<TargetCandidate>>>>(),
        size_of::<[Vec<Option<Versioned<i64>>>; DERIVED_SLOT_COUNT]>() + size_of::<[Vec<Option<Versioned<Option<TargetCandidate>>>>; TARGET_SLOT_COUNT]>(),
        DERIVED_SLOT_COUNT * size_of::<Option<Versioned<i64>>>() + TARGET_SLOT_COUNT * size_of::<Option<Versioned<Option<TargetCandidate>>>>(),
        size_of::<TickContext>());
}
