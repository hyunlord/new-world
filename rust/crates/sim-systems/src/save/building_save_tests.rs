//! Format18 boundary and format19 building-tail tests; integrate after runtime schema.
use super::*;
use sim_core::colony::{
    building_site::*,
    knowledge::{Knowledge, Source},
    JobKind, Skills,
};

fn bootstrap() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("bundle");
}

#[test]
fn format18_keeps_expanded_knowledge_practice_and_era_state() {
    bootstrap();
    let mut engine = sim_engine::SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    let mut knowledge = Knowledge::default();
    knowledge.set(42, 777, Source::Discovered, 99);
    let mut skills = Skills::default();
    skills.complete(JobKind::Build { site: (2, 2) });
    engine.world.spawn((knowledge, skills));
    engine.resources.colony.tech.era_progress = vec![500, 250, 0, 0];
    engine.resources.colony.tech.current_era = (0, 1);
    let world = super::super::world::capture(&engine.world).expect("world");
    let old = LegacyBodyV18 {
        current_tick: engine.current_tick(),
        world,
        systems: vec![],
        steward: engine.resources.steward.clone(),
        content_state: engine.resources.content_state.clone(),
        mind: engine.resources.mind.clone(),
        food: engine.resources.food.clone(),
        cultivation: engine.resources.cultivation.clone(),
        processing: engine.resources.processing.clone(),
        resources: engine.resources,
    };
    let body = decode_body(&super::super::encode(&old).expect("old wire")).expect("upgrade");
    let restored = super::super::world::build(&body.world).expect("expanded columns unchanged");
    let mut query = restored.query::<(&Knowledge, &Skills)>();
    let (_, (knowledge, skills)) = query.iter().next().expect("person");
    assert!(knowledge.get(42).is_some());
    assert_eq!(skills.completed.iter().sum::<u32>(), 1);
    assert_eq!(body.resources.colony.tech.era_progress, [500, 250, 0, 0]);
    assert_eq!(body.resources.colony.tech.current_era, (0, 1));
    assert!(body.buildings.sites.is_empty());
}

#[test]
fn building_tail_retains_station_pieces_causes_allocators_and_housing() {
    bootstrap();
    let mut state = BuildingState::default();
    for (id, status, phase, cause) in [
        (
            0,
            BuildingStatus::Building,
            PiecePhase::Replacing,
            BuildCause::Command,
        ),
        (
            1,
            BuildingStatus::Complete,
            PiecePhase::Complete,
            BuildCause::Zone(8),
        ),
        (
            2,
            BuildingStatus::Cancelled,
            PiecePhase::Cancelled,
            BuildCause::Steward(9),
        ),
    ] {
        state.sites.insert(
            id,
            BuildingSite {
                id,
                template: "workshop_shed".into(),
                station: Some("workbench".into()),
                anchor: (1, 1),
                rotation: 1,
                footprint: (3, 3),
                pieces: vec![BuildingPiece {
                    id,
                    kind: sim_core::colony::BlueprintKind::Hut,
                    tile: (1, 1),
                    size: (3, 3),
                    phase,
                }],
                cause,
                source_record: 19 + id,
                placed_tick: 77,
                status,
                completion_entry: if id == 1 { Some(20) } else { None },
            },
        );
    }
    state.next_site_id = 3;
    state.next_piece_id = 3;
    state
        .housing_zones
        .insert(8, [(1, 1), (1, 2)].into_iter().collect());
    let bytes = super::super::encode(&state).expect("tail");
    let loaded: BuildingState = super::super::decode(&bytes).expect("restore");
    assert_eq!(super::super::encode(&loaded).expect("stable"), bytes);
    assert_eq!(loaded.sites, state.sites);
    super::super::building_validation::validate(&loaded).expect("valid counters");
    let mut corrupt = loaded.clone();
    corrupt.next_site_id = 2;
    assert!(super::super::building_validation::validate(&corrupt).is_err());
    corrupt = loaded.clone();
    corrupt.next_piece_id = 2;
    assert!(super::super::building_validation::validate(&corrupt).is_err());
    assert_eq!(loaded.next_ids(2), Some((4, 5)));
    assert_eq!(loaded.housing_zones, state.housing_zones);
}

#[test]
fn building_sidecar_does_not_enter_nested_resources_wire() {
    bootstrap();
    let mut engine = sim_engine::SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    let original = super::super::encode(&engine.resources).expect("nested wire");
    engine
        .resources
        .colony
        .buildings
        .housing_zones
        .insert(99, [(1, 1)].into_iter().collect());
    engine.resources.colony.buildings.next_site_id = 101;
    assert_eq!(
        super::super::encode(&engine.resources).expect("nested wire"),
        original
    );
}

#[test]
fn stale_building_counter_is_rejected_without_repair() {
    bootstrap();
    let mut state = BuildingState::default();
    state.next_piece_id = u32::MAX;
    assert!(state.next_piece_after(1).is_none());
}

fn slot_additions(buildings: bool) -> super::super::LegacySystemAdditions {
    super::super::LegacySystemAdditions {
        steward: false,
        defence: false,
        food: false,
        diagnosis: false,
        buildings,
    }
}

#[test]
fn historical_building_slot_insertion_preserves_every_existing_state_byte() {
    let registered: Vec<_> = [
        ("RoomSystem", vec![11]),
        ("BuildingTemplateSystem", vec![]),
        ("OralTransmissionSystem", vec![22]),
    ]
    .into_iter()
    .map(|(name, state)| (name.to_owned(), state))
    .collect();
    let mut old = vec![registered[0].clone(), registered[2].clone()];
    super::super::insert_legacy_systems(&mut old, &registered, slot_additions(true))
        .expect("known historical slot");
    assert_eq!(old, registered);
    assert!(
        super::super::insert_legacy_systems(&mut old, &registered, slot_additions(true)).is_err(),
        "duplicate legacy building slot must reject"
    );
}

#[test]
fn current19_missing_building_slot_is_never_repaired() {
    let registered: Vec<_> = [
        "RoomSystem",
        "BuildingTemplateSystem",
        "OralTransmissionSystem",
    ]
    .into_iter()
    .map(|name| (name.to_owned(), vec![]))
    .collect();
    let mut saved = vec![registered[0].clone(), registered[2].clone()];
    let original = saved.clone();
    super::super::insert_legacy_systems(&mut saved, &registered, slot_additions(false))
        .expect("current format has no migration");
    assert_eq!(saved, original);
    assert_ne!(
        saved, registered,
        "install_saved exact list check must reject"
    );
}

#[test]
fn baseline_registry_stays51_and_never_receives_building_slot() {
    bootstrap();
    let mut engine = sim_engine::SimEngine::new(8, 8, sim_core::MaterialRegistry::new());
    crate::register_world_runtime_systems(
        &mut engine,
        sim_engine::registration::WorldKind::Baseline,
    );
    let registered = engine.system_states().expect("baseline registry");
    assert_eq!(registered.len(), 51);
    assert!(!registered
        .iter()
        .any(|(name, _)| name == "BuildingTemplateSystem"));
    let mut historical = registered.clone();
    super::super::insert_legacy_systems(&mut historical, &registered, slot_additions(false))
        .expect("baseline unchanged");
    assert_eq!(historical, registered);
}

#[test]
fn building_command_boundary_accepts_pending_but_rejects_future_and_forged_sources() {
    bootstrap();
    let mut engine = sim_engine::SimEngine::new(16, 16, sim_core::MaterialRegistry::new());
    let res = &mut engine.resources;
    res.rules.colony = Some(sim_core::rules::ColonyRule::m5_5());
    res.current_tick = 7;
    let catalog = sim_core::colony::content::installed_or_bundled();
    let template = catalog.building_templates().get("hut").expect("hut");
    let plan = template.expand(catalog, None, 0).expect("plan");
    let pieces: Vec<_> = plan
        .iter()
        .enumerate()
        .map(|(index, piece)| BuildingPiece {
            id: index as u32,
            kind: piece.kind,
            tile: (1 + piece.offset.0, 1 + piece.offset.1),
            size: piece.size,
            phase: PiecePhase::Cancelled,
        })
        .collect();
    res.colony.buildings.next_piece_id = pieces.len() as u32;
    res.colony.buildings.next_site_id = 1;
    res.colony.buildings.sites.insert(
        0,
        BuildingSite {
            id: 0,
            template: "hut".into(),
            station: None,
            anchor: (1, 1),
            rotation: 0,
            footprint: template.footprint,
            pieces,
            cause: BuildCause::Command,
            source_record: 0,
            placed_tick: 8,
            status: BuildingStatus::Cancelled,
            completion_entry: None,
        },
    );
    res.colony
        .command_log
        .push(sim_core::colony::CommandRecord {
            tick: 8,
            command: sim_core::colony::Command::BuildTemplate {
                id: "hut".into(),
                anchor: (1, 1),
                rotation: 0,
                station: None,
            },
            result: sim_core::colony::CommandResult::ok(1),
        });
    let validate = crate::runtime::building_templates::validate_state_at_boundary;
    validate(&engine.world, res, 8).expect("accepted command at paused admission boundary");
    assert!(validate(&engine.world, res, 7).is_err(), "future command");
    res.colony
        .buildings
        .sites
        .get_mut(&0)
        .expect("site")
        .placed_tick = 7;
    assert!(
        validate(&engine.world, res, 8).is_err(),
        "forged generation tick"
    );
    res.colony
        .buildings
        .sites
        .get_mut(&0)
        .expect("site")
        .placed_tick = 8;
    res.colony.command_log[0].result.accepted = false;
    assert!(
        validate(&engine.world, res, 8).is_err(),
        "rejected source command"
    );
}
