//! Required genuine formats6–9: preserved fields and continuation under current policy.
//! Explicitly run with WORLDSIM_MUX_FIXTURES=<stage-b>; no old-future oracle is replaced.
#[path = "legacy_migration_support/inventory.rs"]
mod inventory;
#[path = "legacy_migration_support/wire.rs"]
mod wire;
use serde::Deserialize;
use sim_core::colony::materials::OAK;
use sim_core::colony::{
    content::{bootstrap, BootSelection},
    BillMode, Command, RecipeKind,
};
use sim_engine::SimEngine;
use sim_systems::{commands::submit, save};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Bill {
    cmd: String,
    tick: u64,
    workshop: u32,
    recipe: String,
    mode: String,
    count: u16,
    result: CapturedResult,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CapturedResult {
    accepted: bool,
    count: u32,
    reason: Option<String>,
}
fn command(row: &Bill) -> Command {
    assert_eq!(row.cmd, "add_bill");
    let mode = match row.mode.as_str() {
        "make" => BillMode::Make(row.count),
        "maintain" => BillMode::Maintain(row.count),
        other => panic!("unregistered bill mode {other}"),
    };
    Command::AddBill {
        workshop: row.workshop,
        recipe: RecipeKind::from_name(&row.recipe).expect("legacy recipe"),
        mode,
    }
}
fn encoded(engine: &mut SimEngine) -> Vec<u8> {
    let bytes = save::save(engine).expect("current save");
    assert_eq!(&bytes[8..12], &save::FORMAT_VERSION.to_le_bytes());
    bytes
}
#[test]
#[ignore = "mandatory explicit external nine-file suite: WORLDSIM_MUX_FIXTURES"]
fn genuine_legacy_inventory_preserves_fields_and_current_policy_continuation() {
    bootstrap(BootSelection::Bundled).expect("bundled historical catalogue");
    let root = std::path::PathBuf::from(
        std::env::var_os("WORLDSIM_MUX_FIXTURES").expect("required external fixture root"),
    );
    assert!(root.is_absolute() && root.is_dir());
    inventory::verify_provenance(&root);
    assert_eq!(inventory::FIXTURES.len(), 9);
    for fixture in inventory::FIXTURES {
        let path = root.join(fixture.path);
        inventory::check_hash(&path, fixture.sha256);
        let original = std::fs::read(&path).expect("required genuine file");
        assert_eq!(&original[..8], b"WORLDSIM");
        assert_eq!(&original[8..12], &fixture.version.to_le_bytes());
        let expected = wire::read(&original, fixture.version);
        let mut migrated = save::load(&original).expect("migrate genuine legacy file");
        assert_eq!(migrated.current_tick(), fixture.tick);
        assert_eq!(expected.tick, fixture.tick);
        assert_eq!(migrated.resources.world_seed, fixture.seed);
        assert_eq!(migrated.resources.steward.enabled, fixture.enabled);
        assert_eq!(migrated.resources.steward.defence, Default::default());
        match expected.steward {
            Some(old) => old.check(&migrated.resources.steward),
            None => assert_eq!(migrated.resources.steward, Default::default()),
        }
        let old_history = (
            &expected.resources.chronicle,
            &expected.resources.causal_log,
            &expected.resources.colony.command_log,
        );
        let new_history = (
            &migrated.resources.chronicle,
            &migrated.resources.causal_log,
            &migrated.resources.colony.command_log,
        );
        assert_eq!(
            save::encode(&old_history).expect("old history"),
            save::encode(&new_history).expect("preserved history"),
            "{} history",
            fixture.path
        );
        assert_eq!(
            migrated.resources.content_state.completed,
            expected.content.completed
        );
        assert_eq!(
            migrated.resources.content_state.workshop_cutters,
            expected.content.workshop_cutters
        );
        assert_eq!(
            migrated.resources.content_state.illness_origins,
            expected.content.illness_origins
        );
        assert!(migrated.resources.content_state.tree_materials.is_empty());
        assert_eq!(
            migrated.resources.content_state.tree_material_default,
            (!migrated.resources.colony.trees.is_empty()).then_some(OAK)
        );
        let first = encoded(&mut migrated);
        let mut twin = save::load(&first).expect("current reload");
        assert_eq!(first, encoded(&mut twin), "{} stable resave", fixture.path);
        // Exercise an unsaved migrated branch against the current-format reloaded branch.
        migrated = save::load(&original).expect("fresh unsaved migration");
        let bills: Vec<Bill> = if fixture.version == 6 && fixture.tick == 2000 {
            let parent = path.parent().expect("historical directory");
            serde_json::from_slice(
                &std::fs::read(parent.join("continuation-2000-3000.json")).expect("captured bills"),
            )
            .expect("strict bill schema")
        } else {
            Vec::new()
        };
        if fixture.version == 6 && fixture.tick == 2000 {
            assert_eq!(bills.len(), 2);
            assert!(bills.windows(2).all(|pair| pair[0].tick <= pair[1].tick));
            assert!(bills.iter().all(|row| (2000..3000).contains(&row.tick)));
        }
        let mut submitted = 0;
        let duration = if fixture.version == 9 { 24 } else { 1000 };
        for tick in fixture.tick..fixture.tick + duration {
            assert_eq!(migrated.current_tick(), tick);
            assert_eq!(twin.current_tick(), tick);
            for row in bills.iter().filter(|row| row.tick == tick) {
                // Historical result stays a recorded observation; current policy must agree between twins.
                let _historical = (&row.result.accepted, &row.result.count, &row.result.reason);
                let a = submit(&mut migrated, command(row));
                let b = submit(&mut twin, command(row));
                assert_eq!(
                    (a.accepted, a.count, a.reason),
                    (b.accepted, b.count, b.reason)
                );
                submitted += 1;
            }
            migrated.tick();
            twin.tick();
        }
        assert_eq!(submitted, bills.len());
        assert_eq!(
            encoded(&mut migrated),
            encoded(&mut twin),
            "{} current-policy continuation",
            fixture.path
        );
        assert_eq!(std::fs::read(&path).expect("immutable fixture"), original);
        inventory::check_hash(&path, fixture.sha256);
        println!(
            "legacy migration verified: {} format{} tick{}",
            fixture.path, fixture.version, fixture.tick
        );
    }
    inventory::verify_provenance(&root);
}
