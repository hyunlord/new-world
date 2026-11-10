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
        vec![0, 1, 2, 3, 4, 5, 6]
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

#[test]
fn tameable_data_only_species_resolves_renamed_products_and_category_art() {
    let animals = include_str!("../../../../sim-test/data/ext_pack/animals.ron");
    let items = include_str!("../../../../sim-test/data/ext_pack/items.ron")
        .replace("made_by:Some(\"test_hide_craft\")", "made_by:None")
        .replace("clue_nodes:[\"test_hide_craft\"]", "clue_nodes:[]");
    let extension = RawSources {
        sources: vec![
            NamedSource {
                name: "animals.ron".into(),
                bytes: animals.as_bytes().to_vec(),
            },
            NamedSource {
                name: "items.ron".into(),
                bytes: items.into_bytes(),
            },
            NamedSource {
                name: "blueprints.ron".into(),
                bytes: include_bytes!("../../../../sim-test/data/ext_pack/blueprints.ron").to_vec(),
            },
        ],
    };
    let catalog =
        Catalog::from_sources(RawSources::bundled(), Some(extension)).expect("data-only tameable");
    let animal = catalog
        .animal_by_code(4_294_967_293)
        .expect("extension species");
    let policy = animal.tameable.as_ref().expect("tameable exposed");
    assert_eq!(animal.identity.id, "test_grazer");
    assert_eq!(
        policy.products.hide.0,
        catalog.item_by_name("test_hide").expect("renamed product")
    );
    assert_eq!(policy.products.hide.1, 1);
    assert_eq!(animal.art.shape, ItemShape::Cloth);
    assert_eq!(animal.art.tint_material, "limestone");
    assert_eq!(animal.art.body_length_permille, 510);
    assert_eq!(animal.art.head_direction, -1);
    assert!(animal.art.antlers);
    assert_eq!(animal.art.juvenile_scale_permille, 600);
    assert_eq!(
        animal.domestic_name_key.as_deref(),
        Some("EXT_DOMESTIC_ANIMAL")
    );
    assert_eq!(
        policy.products.wool.as_ref().expect("novel fleece").item,
        policy.products.hide.0
    );
    let old_species = catalog.animal_by_code(1).expect("old species");
    assert!(old_species.tameable.is_none());
    assert!(old_species.art.antlers);
    assert_eq!(
        ron::from_str::<AnimalArt>("()").expect("category fallback"),
        AnimalArt::default()
    );
}

#[test]
fn animal_silhouettes_reject_unrenderable_geometry_and_empty_owned_names() {
    for (old, new) in [
        ("body_length_permille: 720", "body_length_permille: 0"),
        ("leg_count: 4", "leg_count: 9"),
        ("head_direction: 1", "head_direction: 0"),
        ("juvenile_scale_permille: 600", "juvenile_scale_permille: 0"),
        (
            "domestic_name_key: Some(\"C2_ANIMAL_GOAT\")",
            "domestic_name_key: Some(\"\")",
        ),
    ] {
        let error = Catalog::from_sources(altered("animals.ron", old, new), None)
            .expect_err("invalid silhouette");
        assert!(error.to_string().contains("silhouette"), "{error}");
    }
}

#[test]
fn cattle_is_a_plain_origin_and_sheep_has_independent_wool_policy() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    let cattle = catalog
        .animal_by_name("aurochs")
        .expect("plain wild source");
    let cattle = catalog.animal(cattle).expect("cattle definition");
    assert_eq!(
        cattle.domestic_name_key.as_deref(),
        Some("C2_ANIMAL_CATTLE")
    );
    assert_eq!(
        cattle
            .tameable
            .as_ref()
            .expect("capture policy")
            .capture_knowledge,
        "cattle"
    );
    assert_eq!(cattle.origin.terrain, vec![crate::TerrainType::Plain]);
    let sheep = catalog
        .animal_by_code(3)
        .expect("sheep")
        .tameable
        .as_ref()
        .expect("livestock");
    assert!(sheep.products.milk.is_some());
    assert_eq!(
        sheep
            .products
            .wool
            .as_ref()
            .expect("wool")
            .requires_knowledge,
        "wool_spinning"
    );
    assert_eq!(
        catalog.animal_by_code(4).expect("boar").status,
        crate::colony::knowledge::ContentStatus::Inert
    );
}

#[test]
fn tameable_rejects_bad_tools_products_knowledge_jobs_and_numeric_policy() {
    for (old, new, expected) in [
        (
            "item: \"wool\", every_days: 4",
            "item: \"milk\", every_days: 4",
            "ambiguous recurring livestock products",
        ),
        (
            "item: \"wool\", every_days: 4",
            "item: \"log\", every_days: 4",
            "wool output conflicts with legacy milk placeholder",
        ),
        (
            "capture_tool: Some(\"cord\")",
            "capture_tool: Some(\"missing\")",
            "unknown livestock item",
        ),
        (
            "capture_tool: Some(\"cord\")",
            "capture_tool: Some(\"milk\")",
            "food capability mismatch",
        ),
        (
            "meat: (\"meat\", 3)",
            "meat: (\"missing\", 3)",
            "unknown livestock item",
        ),
        (
            "meat: (\"meat\", 3)",
            "meat: (\"cord\", 3)",
            "food capability mismatch",
        ),
        (
            "hide: (\"goat_hide\", 1)",
            "hide: (\"milk\", 1)",
            "food capability mismatch",
        ),
        (
            "capture_knowledge: \"animal_taming\"",
            "capture_knowledge: \"missing\"",
            "unknown livestock knowledge",
        ),
        (
            "requires_knowledge: \"dairy\"",
            "requires_knowledge: \"missing\"",
            "unknown livestock knowledge",
        ),
        (
            "capture_job: \"capture\"",
            "capture_job: \"hunt\"",
            "unsupported livestock job selector",
        ),
        (
            "capture_success_permille: 700",
            "capture_success_permille: 1001",
            "invalid tameable policy",
        ),
        (
            "initial_age_days: (0, 29)",
            "initial_age_days: (29, 0)",
            "invalid tameable policy",
        ),
        (
            "kid_age_days: 30",
            "kid_age_days: 1800",
            "invalid tameable policy",
        ),
        (
            "grass_regrow_days: 1",
            "grass_regrow_days: 0",
            "positive mechanic value required",
        ),
        (
            "milk_work_ticks: 45",
            "milk_work_ticks: 0",
            "positive mechanic value required",
        ),
        (
            "item: \"milk\", every_days: 1",
            "item: \"milk\", every_days: 0",
            "positive mechanic value required",
        ),
    ] {
        let error = Catalog::from_sources(altered("animals.ron", old, new), None)
            .expect_err("reject bad tameable");
        assert!(error.to_string().contains(expected), "{old}: {error}");
    }
}
