//! Genuine published format14 bytes; fixture generation belongs to the BASE binary.

fn fixture() -> (std::path::PathBuf, Vec<u8>) {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("install bundled catalogue");
    let path = std::path::PathBuf::from(
        std::env::var_os("M6B_BASE_V14_SAVE").expect("genuine BASE save path"),
    );
    let bytes = std::fs::read(&path).expect("read immutable BASE fixture");
    assert_eq!(bytes.get(8..12), Some(14u32.to_le_bytes().as_slice()));
    (path, bytes)
}

#[test]
#[ignore = "requires genuine fbc03bdd save through M6B_BASE_V14_SAVE"]
fn genuine_base_steward_save_loads_without_inventing_extended_state() {
    // Given bytes emitted by fbc03bdd, with active stewardship at tick 360.
    let (path, original) = fixture();
    // When the current reader migrates the historical payload.
    let migrated = sim_systems::save::load(&original).expect("load genuine published format14");
    // Then the historical simulation state remains present and new state is inert.
    assert_eq!(migrated.current_tick(), 360);
    let state = &migrated.resources.steward;
    assert!(state.enabled);
    assert!(
        !state.owned.is_empty(),
        "fixture must contain real automatic orders"
    );
    assert!(
        !state.cooldowns.is_empty(),
        "fixture must exercise map decoding"
    );
    assert!(!state.extended);
    assert!(state.need_since.is_empty());
    assert_eq!(std::fs::read(path).expect("fixture unchanged"), original);
}

#[test]
#[ignore = "requires genuine steward-off BASE save through M6B_BASE_V14_OFF_SAVE"]
fn genuine_base_empty_steward_save_is_not_rejected_as_ambiguous() {
    // Given a genuine BASE colony saved at tick1 with stewardship off.
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("install bundled catalogue");
    let path = std::path::PathBuf::from(
        std::env::var_os("M6B_BASE_V14_OFF_SAVE").expect("genuine off fixture path"),
    );
    let original = std::fs::read(&path).expect("read BASE off fixture");
    assert_eq!(original.get(8..12), Some(14u32.to_le_bytes().as_slice()));
    // When the historical reader checks the potentially empty wire fields.
    let mut migrated = sim_systems::save::load(&original).expect("load empty published steward");
    // Then the valid BASE save survives ambiguity protection and current resave.
    assert_eq!(migrated.current_tick(), 1);
    let state = &migrated.resources.steward;
    assert!(!state.enabled);
    assert!(state.owned.is_empty());
    assert!(state.cooldowns.is_empty());
    assert!(!state.extended);
    assert!(state.need_since.is_empty());
    let bytes = sim_systems::save::save(&mut migrated).expect("resave empty steward");
    let reloaded = sim_systems::save::load(&bytes).expect("reload empty steward");
    assert_eq!(migrated.resources.steward, reloaded.resources.steward);
    assert_eq!(std::fs::read(path).expect("fixture unchanged"), original);
}

#[test]
#[ignore = "requires BASE save and BASE probe summary; run after restoring BASE default rules"]
fn genuine_base_save_continuation_matches_base_and_current_resave() {
    // Given the genuine tick360 save and its BASE binary's day1 fingerprint.
    let (_, original) = fixture();
    let summary_path = std::env::var_os("M6B_BASE_V14_SUMMARY").expect("BASE summary.json path");
    let summary: serde_json::Value = serde_json::from_slice(
        &std::fs::read(summary_path).expect("read BASE continuation summary"),
    )
    .expect("BASE summary JSON");
    let expected = summary["seeds"][0]["fingerprints"]["1"]
        .as_str()
        .expect("BASE day1 fingerprint");
    let mut migrated = sim_systems::save::load(&original).expect("migrate BASE save");
    let saved = sim_systems::save::save(&mut migrated).expect("current resave");
    let mut reloaded = sim_systems::save::load(&saved).expect("current reload");
    assert_eq!(migrated.resources.steward, reloaded.resources.steward);
    // When both the migrated and resaved worlds continue to the same day boundary.
    while migrated.current_tick() < 1440 {
        migrated.tick();
        reloaded.tick();
    }
    // Then both preserve the independently generated BASE continuation.
    assert_eq!(
        sim_test::probe::colony::colony_fingerprint(&migrated),
        expected
    );
    assert_eq!(
        sim_test::probe::colony::colony_fingerprint(&reloaded),
        expected
    );
    assert_eq!(
        migrated.resources.colony.command_log,
        reloaded.resources.colony.command_log
    );
    assert_eq!(
        sim_systems::save::save(&mut migrated).expect("continued migration"),
        sim_systems::save::save(&mut reloaded).expect("continued reload")
    );
}
