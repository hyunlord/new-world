//! Catalogue fixtures for the presentation-only action mapping.

use super::*;
use crate::colony::content::{NamedSource, RawSources};

fn visual_source(bytes: &str) -> NamedSource {
    NamedSource {
        name: SOURCE.into(),
        bytes: bytes.as_bytes().to_vec(),
    }
}

#[test]
fn bundled_visuals_follow_phase_and_existing_recipe_metadata() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("valid data");
    let visuals = catalog.action_visuals();
    for (recipe, key) in [
        ("spin_thread", "spin"),
        ("spin_wool", "spin"),
        ("weave_cloth", "weave"),
        ("make_linen_wrap", "sew"),
        ("sew_wool_tunic", "sew"),
        ("ret_flax", "craft"),
    ] {
        let recipe = catalog.recipe_by_name(recipe).expect("bundled recipe");
        let visual = visuals.resolve(VisibleActionKind::Craft, Phase::Perform, Some(recipe));
        assert_eq!((&*visual.key, &*visual.fallback), (key, "work"));
    }
    let cooked = catalog
        .recipes()
        .find(|recipe| recipe.workshop_tags.iter().any(|tag| tag == "cooking"))
        .expect("bundled cooking recipe");
    for (phase, key, fallback) in [
        (Phase::Idle, "idle", "idle"),
        (Phase::Approach, "walk", "walk"),
        (Phase::Perform, "cook", "work"),
        (Phase::Transport, "carry", "carry"),
    ] {
        let visual = visuals.resolve(VisibleActionKind::Craft, phase, Some(cooked.id));
        assert_eq!((&*visual.key, &*visual.fallback), (key, fallback));
    }
    for (cue, key) in [
        (VisibleCueKind::PickedUp, "pick_up"),
        (VisibleCueKind::Deposited, "set_down"),
        (VisibleCueKind::Equipped, "equip"),
    ] {
        assert_eq!(visuals.cue(cue).key, key);
    }
}

#[test]
fn action_visual_recipe_extension_is_data_only_and_preserves_catalogue_revision() {
    let recipe = NamedSource {
        name: "recipes.ron".into(),
        bytes: br#"(
            schema_version: 1,
            recipes: [(
                identity: (code: 70001, id: "test_baked_flake", name_key: "TEST_BAKED_FLAKE",
                    serde_name: "TestBakedFlake", serde_aliases: [], resolver_aliases: []),
                workshop: "clay_oven", workshop_tags: ["baking"],
                inputs: [("flour", 1)], output: ("bread", 1),
                work_ticks: 7, skill: "knap", work: "craft", quality_add: 0,
                material_term: None, tool: None, auto_bill_role: None,
            )],
        )"#
        .to_vec(),
    };
    let visual = visual_source(
        r#"(
        schema_version: 1,
        recipe_rules: [(
            selector: (workshop_tag: Some("baking")),
            visual: (key: "cook", fallback: "work"),
        )],
    )"#,
    );
    let build = |sources| {
        Catalog::from_sources(RawSources::bundled(), Some(RawSources { sources }))
            .expect("data-only recipe and visual pack")
    };
    let original = build(vec![recipe.clone()]);
    let extended = build(vec![recipe, visual]);
    let resolve = |catalog: &Catalog| {
        let recipe = catalog
            .recipe_by_name("test_baked_flake")
            .expect("new recipe");
        catalog
            .action_visuals()
            .resolve(VisibleActionKind::Craft, Phase::Perform, Some(recipe))
            .clone()
    };
    assert_eq!(resolve(&original).key, "craft");
    assert_eq!(resolve(&extended).key, "cook");
    assert_eq!(original.revision(), extended.revision());
}

#[test]
fn action_visual_validation_rejects_missing_duplicate_and_invalid_rows() {
    for (old, new, message) in [
        (
            "schema_version: 1",
            "schema_version: 2",
            "unsupported schema",
        ),
        ("action: TendFire,", "action: Idle,", "duplicate action"),
        ("cue: Equipped,", "cue: PickedUp,", "duplicate cue"),
        (
            "fallback: \"carry\"",
            "fallback: \"chop\"",
            "unknown fallback",
        ),
        (
            "key: \"tend_fire\"",
            "key: \"tend_fyre\"",
            "unknown visual key",
        ),
        (
            "Some(\"spindle\")",
            "Some(\"wrong_capability\")",
            "matches no recipe",
        ),
    ] {
        let edited = BUNDLED.replacen(old, new, 1);
        assert_ne!(edited, BUNDLED, "test edit for {old}");
        let mut sources = RawSources::bundled();
        sources.sources.retain(|source| source.name != SOURCE);
        sources.sources.push(visual_source(&edited));
        let error = Catalog::from_sources(sources, None).expect_err(message);
        assert_eq!(error.context, SOURCE);
        assert!(error.message.contains(message), "{error}");
    }

    let mut sources = RawSources::bundled();
    let edited = BUNDLED
        .lines()
        .filter(|line| !line.contains("action: TendFire,"))
        .collect::<Vec<_>>()
        .join("\n");
    sources.sources.retain(|source| source.name != SOURCE);
    sources.sources.push(visual_source(&edited));
    let error = Catalog::from_sources(sources, None).expect_err("incomplete table");
    assert!(error.message.contains("missing action TendFire"), "{error}");
}
