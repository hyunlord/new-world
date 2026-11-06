//! Exact predecessor17 migration, baseline byte identity and persisted thermal clocks.
use super::*;
use sim_core::colony::thermal::{DoorThermal, ThermalRules, ThermalState};

#[test]
fn baseline_writer_retains_format17_header_and_absent_thermal_state() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    let original = save(&mut engine).expect("baseline");
    assert_eq!(&original[8..12], &17u32.to_le_bytes());
    let mut loaded = load_with(&original, |_| {}).expect("baseline reload");
    assert!(loaded.resources.thermal.is_none());
    assert_eq!(save(&mut loaded).expect("baseline resave"), original);
}

#[test]
fn colony18_preserves_partial_door_time_and_explicit_empty_history() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M3);
    let mut state = ThermalState::new(ThermalRules::bundled().expect("rules"));
    state.doors.insert(
        (2, 2),
        DoorThermal {
            open_until_tick: 2,
            ..Default::default()
        },
    );
    engine.resources.thermal = Some(state.clone());
    let bytes = save(&mut engine).expect("colony18");
    assert_eq!(&bytes[8..12], &18u32.to_le_bytes());
    let mut loaded = load_with(&bytes, |_| {}).expect("colony reload");
    assert_eq!(loaded.resources.thermal, Some(state));
    assert_eq!(save(&mut loaded).expect("colony resave"), bytes);
}

#[test]
#[ignore = "mandatory original17 migration: WORLDSIM_M6B_FORMAT17_FIXTURES"]
fn genuine_format17_baseline_and_physical_owners_survive_migration() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("catalog");
    let root = std::path::PathBuf::from(
        std::env::var_os("WORLDSIM_M6B_FORMAT17_FIXTURES").expect("original17 root"),
    );
    let baseline = std::fs::read(root.join("baseline17.wsave")).expect("original baseline");
    let mut loaded = load_with(&baseline, |_| {}).expect("genuine baseline17");
    assert_eq!(save(&mut loaded).expect("same baseline bytes"), baseline);
    let original =
        std::fs::read(root.join("colony17-owners-processing.wsave")).expect("original colony");
    let mut loaded = load_with(&original, |_| {}).expect("genuine colony17");
    use sim_core::colony::{people::Apparel, Carry};
    assert_eq!(
        loaded.resources.colony.ground_items[&(1, 1)]
            .spec()
            .wear_remaining_hours,
        Some(731)
    );
    assert_eq!(
        loaded.resources.colony.workshops[&1].shelf[0]
            .spec()
            .wear_remaining_hours,
        Some(730)
    );
    let carried: Vec<_> = loaded
        .world
        .query::<&Carry>()
        .iter()
        .map(|(_, c)| c.wear_remaining_hours)
        .collect();
    assert_eq!(carried, vec![Some(729)]);
    let worn: Vec<_> = loaded
        .world
        .query::<&Apparel>()
        .iter()
        .map(|(_, a)| a.wear_remaining_hours)
        .collect();
    assert_eq!(worn, vec![Some(725)]);
    assert!(loaded.resources.processing.pending[0].remaining_ticks > 0);
    assert_eq!(loaded.resources.processing.pending[1].remaining_ticks, 0);
    // This synthetic fixture has no saved mind and uses an empty registrar;
    // it retains the declared numeric migration rather than a historical policy.
    assert!(loaded.resources.thermal_enabled);
    let state = loaded
        .resources
        .thermal
        .as_ref()
        .expect("declared migration");
    assert!(state.paid_hearths.is_empty() && state.dwellings.is_empty());
    let current = save(&mut loaded).expect("migrated colony18");
    let mut again = load_with(&current, |_| {}).expect("current reload");
    assert_eq!(save(&mut again).expect("current exact"), current);
    for entry in std::fs::read_dir(root.join("scene")).expect("scene saves") {
        let path = entry.expect("entry").path();
        if path.extension().is_some_and(|e| e == "wsave") {
            let bytes = std::fs::read(&path).expect("real scene bytes");
            let envelope: Envelope = decode(&bytes[12..]).expect("original envelope");
            let original: Body = decode(&envelope.payload).expect("original17 body");
            let original_mind = encode(&original.mind).expect("original mind bytes");
            let mut scene = load(&bytes).expect("genuine scene17 load");
            assert_eq!(encode(&scene.resources.mind).unwrap(), original_mind);
            assert!(!scene.resources.thermal_enabled);
            assert!(scene.resources.thermal.is_none());
            crate::runtime::thermal::initialize(&mut scene.resources);
            assert!(scene.resources.thermal.is_none());
            let next = save(&mut scene).expect("scene18");
            assert_eq!(&next[8..12], &18u32.to_le_bytes());
            let mut again = load(&next).expect("scene18 load");
            assert!(!again.resources.thermal_enabled);
            assert!(again.resources.thermal.is_none());
            assert_eq!(encode(&again.resources.mind).unwrap(), original_mind);
            assert_eq!(
                save(&mut again).expect("scene exact"),
                next,
                "{}",
                path.display()
            );
            scene.tick();
            again.tick();
            assert!(scene.resources.thermal.is_none());
            assert!(again.resources.thermal.is_none());
            assert_eq!(save(&mut scene).unwrap(), save(&mut again).unwrap());
        }
    }
}

#[test]
fn thermal_tail_rejects_out_of_world_coordinates_before_runtime() {
    use sim_core::colony::thermal::{DwellingThermal, PaidHeat, RoomThermal};
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M3);
    engine.resources.thermal = Some(ThermalState::new(ThermalRules::bundled().unwrap()));
    let valid = save(&mut engine).unwrap();
    for defect in 0..4 {
        let mut envelope: Envelope = decode(&valid[12..]).unwrap();
        let mut body: thermal_codec::BodyV18 = decode(&envelope.payload).unwrap();
        let state = body.thermal.as_mut().unwrap();
        match defect {
            0 => {
                state.doors.insert((8, 0), DoorThermal::default());
            }
            1 => {
                state.rooms.insert(
                    1,
                    RoomThermal {
                        last_integrated_tick: None,
                        temperature_d10: 0,
                        tiles: [(0, 8)].into_iter().collect(),
                    },
                );
            }
            2 => {
                state.paid_hearths.insert(
                    1,
                    PaidHeat {
                        tick: 0,
                        tile: (u32::MAX, 0),
                        requested_wood: 1,
                        debited_wood: 1,
                    },
                );
            }
            3 => {
                state
                    .dwellings
                    .insert((0, u32::MAX), DwellingThermal::default());
            }
            _ => unreachable!(),
        }
        envelope.payload = encode(&body).unwrap();
        let mut corrupt = valid[..12].to_vec();
        corrupt.extend(encode(&envelope).unwrap());
        assert!(
            matches!(load_with(&corrupt, |_| {}), Err(SaveError::Decode(_))),
            "defect {defect}"
        );
    }
}

#[test]
fn paid_heat_cannot_be_reassigned_to_another_real_hearth() {
    use sim_core::colony::thermal::PaidHeat;
    use sim_core::components::{BuildingKind, Settlement};
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M3);
    for (id, tile) in [(1, (2, 2)), (2, (5, 5))] {
        let mut home = Settlement::new_with_id(id, 0);
        home.formation_tile = tile;
        engine.resources.settlements.insert(id, home);
        engine
            .resources
            .building_kinds
            .insert(tile, BuildingKind::Hearth);
    }
    let mut state = ThermalState::new(ThermalRules::bundled().unwrap());
    state.paid_hearths.insert(
        1,
        PaidHeat {
            tick: 0,
            tile: (2, 2),
            requested_wood: 1,
            debited_wood: 1,
        },
    );
    engine.resources.thermal = Some(state);
    let valid = save(&mut engine).expect("correct physical owner");
    let mut envelope: Envelope = decode(&valid[12..]).unwrap();
    let mut body: thermal_codec::BodyV18 = decode(&envelope.payload).unwrap();
    body.thermal
        .as_mut()
        .unwrap()
        .paid_hearths
        .get_mut(&1)
        .unwrap()
        .tile = (5, 5);
    envelope.payload = encode(&body).unwrap();
    let mut corrupt = valid[..12].to_vec();
    corrupt.extend(encode(&envelope).unwrap());
    assert!(matches!(
        load_with(&corrupt, |_| {}),
        Err(SaveError::Decode(_))
    ));
    // Destroyed owners can leave receipts until the existing runtime prune.
    engine.resources.settlements.remove(&1);
    let stale = save(&mut engine).expect("stale owner receipt");
    assert!(load_with(&stale, |_| {}).is_ok());
}

#[test]
fn current_custom_policy_keeps_numeric_thermal_through_registration_and_reload() {
    use crate::runtime::mind::MindProgram;
    use sim_core::colony::mind_records::MindHistory;
    const SOURCE: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
    for custom in [false, true] {
        let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
        engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M3);
        if custom {
            let program = MindProgram::load(SOURCE).expect("current custom policy");
            engine.resources.mind.policy_source = Some(SOURCE.into());
            engine.resources.mind.history = Some(MindHistory::new(&program.policy).unwrap());
        }
        crate::register_colony_mind_runtime_systems(&mut engine);
        assert!(engine.resources.thermal_enabled);
        crate::runtime::thermal::initialize(&mut engine.resources);
        assert!(engine.resources.thermal.is_some());
        let bytes = save(&mut engine).expect("current18 save");
        let mut again = load(&bytes).expect("current18 reload");
        assert!(again.resources.thermal_enabled);
        assert!(again.resources.thermal.is_some());
        assert_eq!(save(&mut again).unwrap(), bytes);
    }
}

#[test]
fn disabled_thermal_activation_discards_stale_numeric_state() {
    let mut engine = SimEngine::new(8, 8, MaterialRegistry::new());
    engine.resources.rules.colony = Some(sim_core::rules::ColonyRule::M3);
    engine.resources.thermal = Some(ThermalState::new(ThermalRules::bundled().unwrap()));
    engine.resources.thermal_enabled = false;
    crate::runtime::thermal::initialize(&mut engine.resources);
    assert!(engine.resources.thermal.is_none());
    crate::runtime::thermal::initialize(&mut engine.resources);
    assert!(engine.resources.thermal.is_none());
}
