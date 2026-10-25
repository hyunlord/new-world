//! Genuine format10 defence migration; old-binary fixtures remain external and immutable.
#[test]
#[ignore = "requires real format10 saves through MUX2_V10_FIXTURES"]
fn genuine_v10_saves_gain_defence_defaults_and_continue_identically_after_resave() {
    sim_core::colony::content::bootstrap(sim_core::colony::content::BootSelection::Bundled)
        .expect("install bundled catalogue before historical save decoding");
    let directory = std::path::PathBuf::from(
        std::env::var_os("MUX2_V10_FIXTURES").expect("external v10 directory"),
    );
    let mut paths: Vec<_> = std::fs::read_dir(&directory)
        .expect("fixture directory")
        .map(|entry| entry.expect("fixture entry").path())
        .filter(|path| {
            matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("save" | "wsave")
            )
        })
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "real old-binary fixtures are required");
    for path in paths {
        let original = std::fs::read(&path).expect("read old bytes");
        assert_eq!(
            original.get(8..12),
            Some(10u32.to_le_bytes().as_slice()),
            "{} must be genuine v10",
            path.display()
        );
        let mut migrated = sim_systems::save::load(&original)
            .unwrap_or_else(|error| panic!("migrate {}: {error}", path.display()));
        assert_eq!(migrated.resources.steward.defence, Default::default());
        let bytes = sim_systems::save::save(&mut migrated).expect("v11 resave");
        let mut reloaded = sim_systems::save::load(&bytes).expect("load v11");
        assert_eq!(migrated.resources.steward, reloaded.resources.steward);
        assert_eq!(
            bytes,
            sim_systems::save::save(&mut reloaded).expect("stable v11")
        );
        for _ in 0..60 {
            migrated.tick();
            reloaded.tick();
        }
        assert_eq!(
            sim_systems::save::save(&mut migrated).expect("continued migrated"),
            sim_systems::save::save(&mut reloaded).expect("continued reloaded"),
            "{} continuation",
            path.display()
        );
        assert_eq!(std::fs::read(&path).expect("unchanged fixture"), original);
    }
}
