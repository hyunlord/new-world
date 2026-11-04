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
    assert_eq!(third.yields[0].count, 10);
    assert_eq!(catalog.crops().len(), 4);
    assert!(
        build(&extension.replace("seed_item: \"emmer_seed\"", "seed_item: \"absent_seed\""))
            .is_err()
    );
    assert!(build(&extension.replace("min_fertility: 400", "min_fertility: 1001")).is_err());
}

#[test]
fn emmer_seed_is_durable_and_harvest_preserves_the_original_net_grain() {
    let catalog = Catalog::from_sources(RawSources::bundled(), None).expect("catalog");
    let crop = catalog
        .crop(catalog.crop_by_name("emmer").expect("emmer"))
        .expect("crop");
    let seed = catalog.item(crop.seed_item).expect("seed");
    assert_eq!(seed.identity.code, 31);
    assert!(seed.food.is_none());
    assert_eq!(
        catalog.item_by_name("flax_seed").expect("flax seed").code(),
        30
    );
    let grain = catalog.item_by_name("wild_grain").expect("grain");
    assert_eq!(
        crop.yields,
        vec![
            CropYield {
                item: grain,
                count: 10
            },
            CropYield {
                item: crop.seed_item,
                count: 2
            }
        ]
    );
    assert_eq!(crop.seeds_per_tile, 2);
    let recipe = catalog
        .recipe(
            catalog
                .recipe_by_name("extract_emmer_seed")
                .expect("recipe"),
        )
        .expect("definition");
    assert_eq!(recipe.identity.code, 19);
    assert_eq!(recipe.inputs, vec![(grain, 2)]);
    assert_eq!(recipe.output, (crop.seed_item, 2));
    assert_eq!(recipe.work_ticks, 30);
}
