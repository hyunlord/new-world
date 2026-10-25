//! Process-isolated recipe/tool proof through ordinary bills, supply and craft jobs.
mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, run_until, tick_checked, worker};
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::content::{bootstrap, BootSelection, NamedSource, RawSources};
use sim_core::colony::{BillMode, BlueprintKind, Command, ItemKind, ItemSpec, RecipeKind};
use sim_core::rules::{ColonyRule, WildlifeRule};
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::invariants::ledger_of;

#[test]
fn bundled_cutter_breaks_at_the_world_rule_limit() {
    bootstrap(BootSelection::Bundled).expect("bundled catalog");
    for limit in [2u32, 4] {
        let mut engine = job_world_m2(32, 32);
        engine.resources.rules.colony = Some(ColonyRule {
            wildlife: Some(WildlifeRule {
                knife_uses: limit,
                ..WildlifeRule::M5
            }),
            ..ColonyRule::M2
        });
        worker(&mut engine, 3, 3);
        let shop = build_workshop(&mut engine, BlueprintKind::DryingRack, (12, 12), (4, 4));
        let count = u16::try_from(limit + 1).expect("small fixture count");
        drop_items(
            &mut engine,
            (20, 20),
            ItemSpec::raw(ItemKind::Hide, None),
            count,
        );
        drop_items(
            &mut engine,
            (21, 20),
            ItemSpec::raw(ItemKind::FlintBlade, None),
            1,
        );
        assert!(
            submit(
                &mut engine,
                Command::AddBill {
                    workshop: shop,
                    recipe: RecipeKind::ScrapeHide,
                    mode: BillMode::Make(count),
                }
            )
            .accepted
        );
        assert!(
            run_until(&mut engine, 8_000, |e| e
                .resources
                .colony
                .tallies
                .crafts_done
                >= u64::from(limit))
                < 8_000
        );
        assert_eq!(
            engine.resources.colony.wildlife.tallies.knife_uses,
            u64::from(limit)
        );
        assert_eq!(
            ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade).consumed,
            1,
            "configured limit {limit} must consume the reusable blade"
        );
        assert!(!engine.resources.colony.knife_wear.contains_key(&shop));
        assert!(engine
            .resources
            .chronicle
            .entries()
            .iter()
            .any(|entry| entry.kind == ChronicleKind::ToolBroke
                && entry.cause == ItemKind::FlintBlade.code()
                && entry.value == i64::from(limit)));
        for _ in 0..180 {
            tick_checked(&mut engine);
        }
        assert_eq!(
            engine.resources.colony.tallies.crafts_done,
            u64::from(limit),
            "the next ordinary bill craft must wait for a replacement blade"
        );
    }
}

fn extension() -> RawSources {
    RawSources { sources: vec![
        NamedSource { name: "items.ron".into(), bytes: br#"
        (schema_version: 1, roles: {}, items: [(
            identity: (code: 300, id: "fixture_cutter", name_key: "FIXTURE_CUTTER",
                serde_name: "FixtureCutter", serde_aliases: [], resolver_aliases: []),
            category: "tool", material_format_key: "", tags: [], stack_max: 20, carry: 5,
            stack_rule: None, carry_rule: None, value: 1, made_by: None, clue_nodes: [],
            chop: None, weapon: None,
            cutter: Some((wear_rule: None, wear_default: 2, break_tally: Some("knives_broken"))),
            food: None, fuel_units: None, apparel: None,
        )])"#.to_vec() },
        NamedSource { name: "recipes.ron".into(), bytes: br#"
        (schema_version: 1, roles: {}, recipes: [(
            identity: (code: 4294967295, id: "fixture_scrape", name_key: "FIXTURE_SCRAPE",
                serde_name: "FixtureScrape", serde_aliases: [], resolver_aliases: []),
            workshop: "drying_rack", workshop_tags: ["hideworking"],
            inputs: [("hide", 1)], output: ("cured_hide", 1),
            work_ticks: 7, skill: "knap", work: "craft", quality_add: 0,
            material_term: None,
            tool: Some((capability: "cutter", preferred_item: "flint_blade", count: 1, reusable: true)),
            auto_bill_role: None,
        )])"#.to_vec() },
    ] }
}

#[test]
fn alternate_cutter_runs_in_an_isolated_catalogue_process() {
    let status = std::process::Command::new(std::env::current_exe().expect("test binary"))
        .args(["--exact", "alternate_cutter_child", "--nocapture"])
        .env("WORLDSIM_RECIPE_CUTTER_CHILD", "1")
        .status()
        .expect("child");
    assert!(status.success());
}

#[test]
fn alternate_cutter_child() {
    if std::env::var_os("WORLDSIM_RECIPE_CUTTER_CHILD").is_none() {
        return;
    }
    bootstrap(BootSelection::Explicit {
        request_id: "recipe-cutter-regression".into(),
        sources: extension(),
    })
    .expect("extension bootstrap");
    let cutter = ItemKind::checked(300).expect("extension cutter");
    let recipe = RecipeKind::checked(u32::MAX).expect("extension recipe");
    let mut engine = job_world_m2(32, 32);
    engine.resources.rules.colony = Some(ColonyRule {
        wildlife: Some(WildlifeRule::M5),
        ..ColonyRule::M2
    });
    worker(&mut engine, 3, 3);
    let shop = build_workshop(&mut engine, BlueprintKind::DryingRack, (12, 12), (4, 4));
    drop_items(
        &mut engine,
        (20, 20),
        ItemSpec::raw(ItemKind::Hide, None),
        3,
    );
    let accepted = submit(
        &mut engine,
        Command::AddBill {
            workshop: shop,
            recipe,
            mode: BillMode::Make(3),
        },
    );
    assert!(accepted.accepted);
    for _ in 0..180 {
        tick_checked(&mut engine);
    }
    assert_eq!(
        engine.resources.colony.tallies.crafts_done, 0,
        "missing cutter cannot craft"
    );
    // A single reusable tool must start this three-craft bill; it is not multiplied by the craft count.
    drop_items(&mut engine, (21, 20), ItemSpec::raw(cutter, None), 1);
    let elapsed = run_until(&mut engine, 8_000, |e| {
        e.resources.colony.tallies.crafts_done >= 2
    });
    assert!(
        elapsed < 8_000,
        "alternate cutter must be supplied and used"
    );
    assert_eq!(
        ledger_of(&engine.world, &engine.resources, cutter).consumed,
        1
    );
    assert_eq!(
        ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade).consumed,
        0
    );
    assert_eq!(engine.resources.colony.wildlife.tallies.knife_uses, 2);
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|entry| entry.kind == ChronicleKind::ToolBroke
            && entry.cause == cutter.code()
            && entry.value == 2));
    assert!(engine
        .resources
        .colony
        .tallies
        .craft_quality
        .keys()
        .any(|(_, code)| *code == u32::MAX));
    for _ in 0..180 {
        tick_checked(&mut engine);
    }
    assert_eq!(
        engine.resources.colony.tallies.crafts_done, 2,
        "broken cutter blocks next craft"
    );
    assert!(!engine
        .resources
        .content_state
        .workshop_cutters
        .contains_key(&shop));
    drop_items(
        &mut engine,
        (21, 20),
        ItemSpec::raw(ItemKind::FlintBlade, None),
        1,
    );
    assert!(
        run_until(&mut engine, 8_000, |e| e
            .resources
            .colony
            .tallies
            .crafts_done
            >= 3)
            < 8_000
    );
    assert_eq!(
        ledger_of(&engine.world, &engine.resources, ItemKind::FlintBlade).consumed,
        0,
        "replacement default cutter starts unworn"
    );
    assert!(
        engine.resources.content_state.workshop_cutters.is_empty(),
        "base cutter does not create extension state"
    );
}
