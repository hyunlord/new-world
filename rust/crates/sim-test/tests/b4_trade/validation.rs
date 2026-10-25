//! Strict local policy/reference parsing, independent of process-global catalog initialization.
use super::*;
use sim_core::colony::content::{Catalog, RawSources};
use sim_core::rules::{parse_neighbour_data_with_catalog, GoodsPolicy};

const DATA: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../sim-core/data/colony/neighbours.ron"
));

#[test]
fn t_strict_policy_parser_rejects_bad_fields_ranges_tags_and_references() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("local catalog");
    let data = parse_neighbour_data_with_catalog(DATA, &catalog).expect("valid authored policy");
    assert_eq!(
        (data.rule.lack_weight, data.rule.staple_weight),
        (3, 1),
        "saved rule values preserved"
    );
    assert_eq!(
        data.goods_policy,
        Some(GoodsPolicy::Scarce {
            maximum_practitioners: 2,
            scarce_weight: 30,
            staple_weight: 10,
            rare_weight: 1,
            next_era_cutoff: 750,
            rare_era: Era::Neolithic
        })
    );
    for (old, new) in [
        ("maximum_practitioners:", "maximum_practitioners_typo:"),
        ("Scarce(", "Unsupported("),
        ("rare_era: Neolithic", "rare_era: FutureEra"),
        ("scarce_weight: 30", "scarce_weight: 0"),
        ("staple_weight: 10", "staple_weight: 0"),
        ("rare_weight: 1", "rare_weight: 0"),
        ("next_era_cutoff: 750", "next_era_cutoff: 1001"),
        ("tag: \"lacked\"", "tag: \"typo\""),
        ("item: \"clay_pot\"", "item: \"missing_item\""),
        ("count: (1, 2)", "count: (2, 1)"),
        ("count: (1, 2)", "count: (0, 2)"),
        (
            "one_neighbour_knows: [\"pot_firing\"]",
            "one_neighbour_knows: [\"missing_node\"]",
        ),
        (
            "migrant_top: Some(\"hide_wrap\")",
            "migrant_top: Some(\"stone\")",
        ),
        ("wait_days:", "wait_dayz:"),
    ] {
        assert!(DATA.contains(old), "mutation exists: {old}");
        let changed = DATA.replacen(old, new, 1);
        assert!(
            parse_neighbour_data_with_catalog(&changed, &catalog).is_none(),
            "rejected {old} -> {new}"
        );
    }
}

#[test]
fn t_missing_policy_is_legacy_and_unknown_fields_do_not_default_it() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    let rule = ron::to_string(&neighbour_data().expect("data").rule).expect("rule");
    let old = format!("(schema_version:1,rule:{rule},names:[],goods:[])");
    let parsed = parse_neighbour_data_with_catalog(&old, &catalog).expect("legacy data");
    assert_eq!(parsed.goods_policy, None);
    assert!(parse_neighbour_data_with_catalog(
        &old.replacen("names:[]", "names:[],goods_polciy:None", 1),
        &catalog
    )
    .is_none());
}

#[test]
fn t_registered_full_width_item_uses_local_catalog_without_global_fallback() {
    let mut sources = RawSources::bundled();
    let items = sources
        .sources
        .iter_mut()
        .find(|s| s.name == "items.ron")
        .expect("items");
    let text = String::from_utf8(items.bytes.clone()).expect("utf8");
    let needle = "code: 1, id: \"firewood\"";
    assert!(text.contains(needle));
    items.bytes = text
        .replacen(needle, "code: 4294967295, id: \"firewood\"", 1)
        .into_bytes();
    let catalog = Catalog::from_sources(sources, None).expect("registered full-width catalog");
    let data = parse_neighbour_data_with_catalog(DATA, &catalog).expect("all references local");
    let item = catalog.item_by_name("firewood").expect("local firewood");
    assert_eq!(item.code(), u32::MAX);
    let row = data
        .goods
        .iter()
        .find(|r| r.item == "firewood")
        .expect("authored staple")
        .clone();
    let table = trade::eligible_goods(
        &catalog,
        &[row],
        &data.rule,
        data.goods_policy,
        &trade::GoodsInputs::default(),
    );
    assert_eq!(table.len(), 1);
    assert_eq!((table[0].kind.code(), table[0].weight), (u32::MAX, 10));
    let goods = trade::draw_goods(0, &data.rule, 1, &table);
    assert!(!goods.is_empty());
    assert!(goods.iter().all(|g| g.kind.code() == u32::MAX));
}
