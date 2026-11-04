use super::*;
#[test]
fn third_crop_requires_only_data_and_rejects_unknown_physical_seed() {
    let source = include_str!("../../../data/colony/crops.ron");
    let extension = source
        .replace("code: 0", "code: 20")
        .replace("code: 1", "code: 21")
        .replace("id: \"emmer\"", "id: \"third_grain\"")
        .replace("id: \"flax\"", "id: \"third_fibre\"")
        .replace("serde_name: \"Emmer\"", "serde_name: \"ThirdGrain\"")
        .replace("serde_name: \"Flax\"", "serde_name: \"ThirdFibre\"");
    let build = |text: &str| {
        Catalog::from_sources(
            RawSources::bundled(),
            Some(RawSources {
                sources: vec![NamedSource {
                    name: "crops.ron".into(),
                    bytes: text.as_bytes().to_vec(),
                }],
            }),
        )
    };
    let catalog = build(&extension).expect("data-only crops");
    let third = catalog
        .crop(catalog.crop_by_name("third_grain").expect("id"))
        .expect("crop");
    assert_eq!(third.seeds_per_tile, 2);
    assert_eq!(third.yields[0].count, 12);
    assert_eq!(catalog.crops().len(), 4);
    assert!(
        build(&extension.replace("seed_item: \"wild_grain\"", "seed_item: \"absent_seed\""))
            .is_err()
    );
    assert!(build(&extension.replace("min_fertility: 400", "min_fertility: 1001")).is_err());
}
