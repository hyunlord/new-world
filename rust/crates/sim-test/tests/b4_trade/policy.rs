//! Pure authored policy boundaries and exact historical keyed-vector checks.
use super::*;
use sim_core::colony::content::installed_or_bundled;
use sim_core::rng::{keyed_below, STREAM_TRADE};
use sim_core::rules::{GoodsEntry, GoodsPolicy};

#[test]
fn t_authored_rare_era_and_count_precedence_are_data_driven() {
    let catalog = installed_or_bundled();
    let data = neighbour_data().expect("data");
    for rare_era in [Era::Paleolithic, Era::Neolithic] {
        let policy = Some(GoodsPolicy::Scarce {
            maximum_practitioners: 2,
            scarce_weight: 30,
            staple_weight: 10,
            rare_weight: 1,
            next_era_cutoff: 750,
            rare_era,
        });
        for era in [Era::Paleolithic, Era::Neolithic] {
            let node = node(era);
            let row = data
                .goods
                .iter()
                .find(|r| {
                    r.tag == "lacked"
                        && ItemKind::from_name(&r.item)
                            .expect("item")
                            .definition()
                            .made_by
                            .as_deref()
                            .and_then(knowledge::id)
                            == Some(node)
                })
                .expect("artifact");
            for progress in [249, 250, 749, 750] {
                for count in 0..=3 {
                    let mut input = trade::GoodsInputs {
                        known: knowledge::bit(node),
                        progress_permille: progress,
                        ..Default::default()
                    };
                    input.practitioners[usize::from(node)] = count;
                    let rows = trade::eligible_goods(
                        catalog,
                        std::slice::from_ref(row),
                        &data.rule,
                        policy,
                        &input,
                    );
                    if count == 3 {
                        assert!(rows.is_empty());
                    } else {
                        assert_eq!(rows.len(), 1, "rare and scarce cannot duplicate a row");
                        assert_eq!(
                            rows[0].weight,
                            if count == 0 && progress < 750 && era == rare_era {
                                1
                            } else {
                                30
                            }
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn t_unknown_technique_missing_producer_and_empty_table_are_not_fabricated_offers() {
    let catalog = installed_or_bundled();
    let data = neighbour_data().expect("data");
    let artifact = data
        .goods
        .iter()
        .find(|r| r.tag == "lacked")
        .expect("artifact")
        .clone();
    let no_producer = GoodsEntry {
        item: "firewood".to_owned(),
        tag: "lacked".to_owned(),
        count: (1, 2),
    };
    assert!(catalog
        .item(catalog.item_by_name("firewood").expect("item"))
        .expect("definition")
        .made_by
        .is_none());
    let input = trade::GoodsInputs::default();
    let rows = trade::eligible_goods(
        catalog,
        &[artifact, no_producer],
        &data.rule,
        data.goods_policy,
        &input,
    );
    assert!(rows.is_empty());
    assert!(trade::draw_goods(0, &data.rule, 1, &rows).is_empty());
    let staples: Vec<_> = data
        .goods
        .iter()
        .filter(|r| r.tag == "staple")
        .cloned()
        .collect();
    let rows = trade::eligible_goods(catalog, &staples, &data.rule, data.goods_policy, &input);
    assert_eq!(rows.len(), staples.len());
    assert!(rows.iter().all(|r| r.weight == 10));
    assert!(!trade::draw_goods(0, &data.rule, 1, &rows).is_empty());
}

fn independent_legacy(seed: u64, id: u32, known: u32, practice: u32) -> Vec<Goods> {
    let data = neighbour_data().expect("data");
    let rule = data.rule;
    let table: Vec<_> = data
        .goods
        .iter()
        .filter_map(|row| {
            let kind = ItemKind::from_name(&row.item).expect("item");
            let made = kind.definition().made_by.as_deref().and_then(knowledge::id);
            let weight = match row.tag.as_str() {
                "staple" => 1u64,
                "lacked"
                    if made.is_some_and(|n| {
                        known & knowledge::bit(n) != 0 && practice & knowledge::bit(n) == 0
                    }) =>
                {
                    3
                }
                _ => return None,
            };
            Some((kind, weight, row.count))
        })
        .collect();
    let total: u64 = table.iter().map(|r| r.1).sum();
    let draw = |slot, bound| keyed_below(seed, STREAM_TRADE, u64::from(id), 0, slot, 0, bound);
    let stacks = u32::from(rule.goods_stacks.0)
        + draw(0, u64::from(rule.goods_stacks.1 - rule.goods_stacks.0) + 1) as u32;
    (0..stacks)
        .map(|i| {
            let chosen = draw(0x10 + i, total);
            let mut end = 0;
            let &(kind, _, (lo, hi)) = table
                .iter()
                .find(|r| {
                    end += r.1;
                    chosen < end
                })
                .expect("interval");
            Goods {
                kind,
                count: lo + draw(0x40 + i, u64::from(hi - lo) + 1) as u16,
                quality: rule.goods_quality.0
                    + draw(
                        0x80 + i,
                        u64::from(rule.goods_quality.1 - rule.goods_quality.0) + 1,
                    ) as u8,
            }
        })
        .collect()
}

#[test]
fn t_absent_policy_preserves_exact_legacy_vectors_and_uniform_scaling() {
    let data = neighbour_data().expect("data");
    for seed in 0..8 {
        for id in [1, 2, 9] {
            for (known, practice) in [
                (0, 0),
                (u32::MAX, 0),
                (u32::MAX, u32::MAX),
                (u32::MAX, 0x5555),
            ] {
                assert_eq!(
                    trade::goods(seed, &data.rule, id, known, practice),
                    independent_legacy(seed, id, known, practice)
                );
            }
        }
    }
    let inputs = trade::GoodsInputs {
        known: u32::MAX,
        ..Default::default()
    };
    let mut rows = trade::eligible_goods(
        installed_or_bundled(),
        &data.goods,
        &data.rule,
        None,
        &inputs,
    );
    rows.push(rows[0]);
    let mut scaled = rows.clone();
    for row in &mut scaled {
        row.weight *= 10;
    }
    assert_eq!(
        rows.len(),
        data.goods.len() + 1,
        "duplicate authored rows preserved"
    );
    for seed in 0..8 {
        assert_eq!(
            trade::draw_goods(seed, &data.rule, 1, &rows),
            trade::draw_goods(seed, &data.rule, 1, &scaled)
        );
    }
}
