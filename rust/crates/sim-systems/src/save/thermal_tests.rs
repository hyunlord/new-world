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
            let mut scene = load(&bytes).expect("genuine scene17 load");
            let next = save(&mut scene).expect("scene18");
            let mut again = load(&next).expect("scene18 load");
            assert_eq!(
                save(&mut again).expect("scene exact"),
                next,
                "{}",
                path.display()
            );
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
