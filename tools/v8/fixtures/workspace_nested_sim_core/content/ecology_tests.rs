//! Pure ecology validation and process-isolated installed-ID checks.
use super::*;
use crate::rules::{StoryRule, WildlifeRule};

fn altered(family: &str, old: &str, new: &str) -> RawSources {
    let mut sources = RawSources::bundled();
    let source = sources
        .sources
        .iter_mut()
        .find(|s| s.name == family)
        .expect("family source");
    let text = String::from_utf8(source.bytes.clone()).expect("utf8");
    assert!(text.contains(old), "fixture edit found");
    source.bytes = text.replacen(old, new, 1).into_bytes();
    sources
}
#[test]
fn ecology_base_codes_schedule_and_per_world_overrides_are_exact() {
    let c = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    assert_eq!(
        c.animals().map(|d| d.identity.code).collect::<Vec<_>>(),
        vec![0, 1]
    );
    assert_eq!(
        c.pressures().map(|d| d.identity.code).collect::<Vec<_>>(),
        vec![0, 1, 2, 3]
    );
    let mut sr = StoryRule::M5;
    sr.snap_burn_permille = 2345;
    sr.drought_thirst_permille = 1700;
    sr.sick_days = 7;
    assert_eq!(
        c.pressure_by_code(0)
            .expect("cold")
            .resolve_rule(&sr)
            .snap_burn_permille,
        2345
    );
    assert_eq!(
        c.pressure_by_code(1)
            .expect("drought")
            .resolve_rule(&sr)
            .drought_thirst_permille,
        1700
    );
    assert_eq!(
        c.pressure_by_code(3).expect("illness").duration_span(&sr),
        (7, 7)
    );
    let mut wr = WildlifeRule::M5;
    wr.wolf_hp = 73;
    wr.deer_flee_speed = 4;
    wr.wolf_chase_tiles = 2;
    assert_eq!(c.animal_by_code(0).expect("wolf").hp.wildlife(&wr), 73);
    assert_eq!(
        c.animal_by_code(1)
            .expect("deer")
            .resolve_rule(&wr)
            .deer_flee_speed,
        4
    );
    assert_eq!(
        c.animal_by_code(0)
            .expect("wolf")
            .resolve_rule(&wr)
            .wolf_chase_tiles,
        2
    );
    let s = c.pressure_schedule();
    for (season, expected) in [
        (0, vec![(700, Some(2)), (1000, None)]),
        (1, vec![(600, Some(1)), (1000, Some(2))]),
        (2, vec![(700, Some(2)), (1000, Some(3))]),
        (3, vec![(600, Some(0)), (1000, Some(2))]),
    ] {
        assert_eq!(
            s.seasons[&season]
                .iter()
                .map(|d| (d.upper_exclusive, d.kind.map(|k| k.code())))
                .collect::<Vec<_>>(),
            expected
        );
    }
    assert_eq!(s.minor_draw_slot, 4);
    assert_eq!(
        s.due_order.iter().map(|k| k.code()).collect::<Vec<_>>(),
        vec![0, 2]
    );
}
#[test]
fn ecology_rejects_unknown_selectors_references_and_incoherent_payloads() {
    for (family, old, new, expected) in [
        (
            "animals.ron",
            "\"wolf_hp\"",
            "\"typo_hp\"",
            "unsupported mechanic selector",
        ),
        (
            "animals.ron",
            "item: \"wolf_hide\"",
            "item: \"missing\"",
            "unknown hide item",
        ),
        (
            "animals.ron",
            "item: \"meat\"",
            "item: \"log\"",
            "unsupported perishable meat identity",
        ),
        (
            "animals.ron",
            "category: \"predator\"",
            "category: \"grazer\"",
            "category/mechanic",
        ),
        (
            "pressures.ron",
            "animal: \"wolf\"",
            "animal: \"deer\"",
            "pack requires predator",
        ),
        (
            "pressures.ron",
            "duration_draw_slot: Some(2)",
            "duration_draw_slot: Some(8)",
            "keyed draw slot",
        ),
        ("pressures.ron", "(1000,None)", "(999,None)", "cover 1000"),
        (
            "pressures.ron",
            "\"snap_burn_permille\"",
            "\"bad_factor\"",
            "unsupported mechanic selector",
        ),
    ] {
        let error = Catalog::from_sources(altered(family, old, new), None)
            .expect_err("reject malformed ecology");
        assert!(error.to_string().contains(expected), "{error}");
    }
}
#[test]
fn drought_rejects_authored_growth_above_baseline_without_clipping_world_rules() {
    let original = "regrowth: (value: 400, rule: Some(\"drought_regen_permille\"))";
    for selector in ["None", "Some(\"drought_regen_permille\")"] {
        let replacement = format!("regrowth: (value: 1001, rule: {selector})");
        let error = Catalog::from_sources(altered("pressures.ron", original, &replacement), None)
            .expect_err("reject authored regrowth above baseline");
        assert!(error
            .to_string()
            .contains("drought regrowth exceeds permille"));
    }
    for value in [0, 1000] {
        let replacement = format!("regrowth: (value: {value}, rule: None)");
        let catalog = Catalog::from_sources(altered("pressures.ron", original, &replacement), None)
            .expect("inclusive authored regrowth bounds");
        assert_eq!(
            catalog
                .pressure_by_code(1)
                .expect("drought")
                .resolve_rule(&StoryRule::M5)
                .drought_regen_permille,
            value
        );
    }
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("bundled");
    let mut rule = StoryRule::M5;
    rule.drought_regen_permille = 1001;
    assert_eq!(
        catalog
            .pressure_by_code(1)
            .expect("drought")
            .resolve_rule(&rule)
            .drought_regen_permille,
        1001
    );
}

#[test]
fn ecology_high_codes_resolve_locally_without_installation() {
    let sources = altered("animals.ron", "code: 0", "code: 4294967295");
    let c = Catalog::from_sources(sources, None).expect("high animal");
    assert_eq!(
        c.animal_role("default_predator").expect("role").code(),
        u32::MAX
    );
    assert_eq!(
        c.pressure_by_code(2)
            .expect("pack")
            .pack
            .as_ref()
            .expect("pack effect")
            .animal
            .code(),
        u32::MAX
    );
    let sources = altered("pressures.ron", "code: 0", "code: 4294967294");
    let c = Catalog::from_sources(sources, None).expect("high pressure");
    assert_eq!(
        c.pressure_schedule().seasons[&3][0]
            .kind
            .expect("cold")
            .code(),
        u32::MAX - 1
    );
}
#[test]
fn ecology_id_bootstrap_boundaries_are_isolated() {
    for case in ["missing", "bundled"] {
        let status = std::process::Command::new(std::env::current_exe().expect("exe"))
            .args([
                "--exact",
                "colony::content::ecology_tests::ecology_id_child",
                "--nocapture",
            ])
            .env("WORLDSIM_ECOLOGY_TEST_CASE", case)
            .status()
            .expect("child");
        assert!(status.success(), "{case}");
    }
}
#[test]
fn ecology_id_child() {
    let Ok(case) = std::env::var("WORLDSIM_ECOLOGY_TEST_CASE") else {
        return;
    };
    if case == "missing" {
        assert!(ron::from_str::<PressureKind>("ColdSnap").is_err());
        assert!(ron::from_str::<AnimalKind>("Wolf").is_err());
        assert!(installed().is_err());
        return;
    }
    bootstrap(BootSelection::Bundled).expect("bundled");
    for (name, code) in [
        ("ColdSnap", 0),
        ("Drought", 1),
        ("Beasts", 2),
        ("Illness", 3),
    ] {
        let id: PressureKind = ron::from_str(name).expect("legacy name");
        assert_eq!(id.code(), code);
        assert_eq!(format!("{id:?}"), name);
    }
    for (name, code) in [("Wolf", 0), ("Deer", 1)] {
        let id: AnimalKind = ron::from_str(name).expect("legacy name");
        assert_eq!(id.code(), code);
        assert_eq!(format!("{id:?}"), name);
    }
    assert!(PressureKind::checked(u32::MAX).is_err());
    assert!(AnimalKind::checked(u32::MAX).is_err());
}
