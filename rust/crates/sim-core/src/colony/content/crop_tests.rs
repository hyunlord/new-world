use super::*;
#[test]
fn third_crop_requires_only_data_and_rejects_unknown_physical_seed() {
    let source = include_str!("../../../data/colony/crops.ron");
    let mut extension = source.to_owned();
    for (code, name, replacement, serde_name, serde_replacement) in [
        (0, "emmer", "third_grain", "Emmer", "ThirdGrain"),
        (1, "flax", "third_fibre", "Flax", "ThirdFibre"),
        (2, "barley", "third_barley", "Barley", "ThirdBarley"),
        (3, "rye", "third_rye", "Rye", "ThirdRye"),
        (4, "lentil", "third_lentil", "Lentil", "ThirdLentil"),
        (5, "pea", "third_pea", "Pea", "ThirdPea"),
        (6, "millet", "third_millet", "Millet", "ThirdMillet"),
    ] {
        extension = extension
            .replace(&format!("code: {code},"), &format!("code: {},", code + 20))
            .replace(
                &format!("id: \"{name}\""),
                &format!("id: \"{replacement}\""),
            )
            .replace(
                &format!("serde_name: \"{serde_name}\""),
                &format!("serde_name: \"{serde_replacement}\""),
            );
    }
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
    assert_eq!(catalog.crops().len(), 14);
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
