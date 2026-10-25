//! Current-public-API fixtures and independent registered T weighted-draw oracle.
pub(super) use sim_core::colony::knowledge::{self, Era, Knowledge, KnowledgeEntry, Source};
pub(super) use sim_core::colony::neighbours::Goods;
pub(super) use sim_core::colony::ItemKind;
pub(super) use sim_core::components::Child;
use sim_core::rng::{keyed_below, STREAM_TRADE};
pub(super) use sim_core::rules::{neighbour_data, NeighbourData};
pub(super) use sim_engine::SimEngine;
pub(super) use sim_systems::runtime::neighbours::{self, trade};

pub(super) const TODAY: u16 = 20;

pub(super) fn node(era: Era) -> u8 {
    neighbour_data()
        .expect("authored goods")
        .goods
        .iter()
        .filter(|row| row.tag == "lacked")
        .filter_map(|row| ItemKind::from_name(&row.item))
        .filter_map(|kind| kind.definition().made_by.as_deref().and_then(knowledge::id))
        .find(|&id| knowledge::defs().get(id).expect("resolved node").era == era)
        .expect("bundled artifact in selected era")
}

pub(super) fn fixture(seed: u64, node: u8, progress: u16) -> SimEngine {
    let mut e = sim_test::colony_engine(seed);
    let entities: Vec<_> = e
        .world
        .query::<&Knowledge>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in entities {
        e.world
            .remove_one::<Knowledge>(entity)
            .expect("remove founder knowledge");
    }
    let rule = neighbours::rule(&e.resources).expect("neighbour rule");
    e.resources.colony.neighbours.list = neighbours::generate(seed, &rule);
    for n in &mut e.resources.colony.neighbours.list {
        n.known = knowledge::bit(node);
        n.attitude = 100;
    }
    e.resources.current_tick = u64::from(TODAY) * 1440;
    e.resources.colony.tech.progress_permille = progress;
    e.resources.colony.tech.states.clear();
    assert!(e.resources.colony.neighbours.caravans.is_empty());
    sim_systems::runtime::nav::ensure(&mut e.resources);
    assert!(e
        .resources
        .colony
        .neighbours
        .list
        .iter()
        .all(|n| neighbours::edge_tile(&e.resources, n.edge).is_some()));
    e
}

pub(super) fn holder(
    e: &mut SimEngine,
    node: u8,
    proficiency: u16,
    last_used: u16,
    child: bool,
    agent: bool,
) {
    let k = Knowledge::from_entries(vec![KnowledgeEntry::new(
        node,
        Source::Seeded,
        proficiency,
        last_used,
    )]);
    if agent {
        let entity = e.spawn_agent(5, 5);
        e.world.insert_one(entity, k).expect("holder knowledge");
        if child {
            e.world
                .insert_one(entity, Child { born_tick: 0 })
                .expect("child");
        }
    } else {
        e.world.spawn((k,));
    }
}

/// Inputs are explicit registered counts, not a call to the production classifier.
pub(super) fn expected(e: &SimEngine, node: u8, count: u32, progress: u16) -> Vec<Goods> {
    let rule = neighbours::rule(&e.resources).expect("rule");
    let era = knowledge::defs().get(node).expect("node").era;
    let mut table = Vec::new();
    for row in &neighbour_data().expect("goods").goods {
        let kind = ItemKind::from_name(&row.item).expect("item");
        let made_by = kind.definition().made_by.as_deref().and_then(knowledge::id);
        let weight = match row.tag.as_str() {
            "staple" => Some(10u64),
            "lacked" if made_by == Some(node) => {
                if era == Era::Neolithic && progress < 750 && count == 0 {
                    Some(1)
                } else if count <= 2 {
                    Some(30)
                } else {
                    None
                }
            }
            "lacked" => None,
            _ => panic!("unexpected authored tag"),
        };
        if let Some(weight) = weight {
            table.push((kind, weight, row.count));
        }
    }
    let total: u64 = table.iter().map(|r| r.1).sum();
    assert!(total > 0);
    let draw = |slot, n| keyed_below(e.resources.world_seed, STREAM_TRADE, 1, 0, slot, 0, n);
    let stacks = u32::from(rule.goods_stacks.0)
        + draw(0, u64::from(rule.goods_stacks.1 - rule.goods_stacks.0) + 1) as u32;
    (0..stacks)
        .map(|i| {
            let pick = draw(0x10 + i, total);
            let mut cumulative = 0;
            let &(kind, _, (lo, hi)) = table
                .iter()
                .find(|r| {
                    cumulative += r.1;
                    pick < cumulative
                })
                .expect("weighted interval");
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

pub(super) fn generate(e: &mut SimEngine) -> Vec<Goods> {
    let rule = neighbours::rule(&e.resources).expect("rule");
    trade::send_caravan(&e.world, &mut e.resources, &rule);
    assert_eq!(
        e.resources.colony.neighbours.caravans.len(),
        1,
        "real caravan generated"
    );
    assert_eq!(e.resources.colony.neighbours.tallies.caravans, 1);
    e.resources.colony.neighbours.caravans[0].goods.clone()
}

pub(super) fn has_artifact(goods: &[Goods], node: u8) -> bool {
    goods.iter().any(|g| {
        g.kind
            .definition()
            .made_by
            .as_deref()
            .and_then(knowledge::id)
            == Some(node)
    })
}
