//! Fresh generation inputs and authored-order goods selection; no persistent state.
use hecs::World;
use sim_core::colony::content::Catalog;
use sim_core::colony::knowledge::{bit, day_of, Knowledge};
use sim_core::colony::neighbours::Goods;
use sim_core::colony::ItemKind;
use sim_core::components::{Agent, Child};
use sim_core::rng::{keyed_below, STREAM_TRADE};
use sim_core::rules::{GoodsEntry, GoodsPolicy, NeighbourRule};
use sim_engine::SimResources;

/// All variable eligibility inputs at the instant goods are generated.
#[derive(Debug, Clone, Copy, Default)]
pub struct GoodsInputs {
    /// Knowledge practiced by the source neighbour.
    pub known: u32,
    /// Historical all-Knowledge practice mask, used only by legacy policy.
    pub legacy_practice: u32,
    /// Current live adult practitioner counts by knowledge ID.
    pub practitioners: [u32; 32],
    /// Actual stored era progress at generation.
    pub progress_permille: u16,
}

/// One eligible authored row; duplicates and order remain significant.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WeightedGoods {
    /// Registered full-width item identity.
    pub kind: ItemKind,
    /// Positive draw weight.
    pub weight: u32,
    /// Inclusive stack-size range.
    pub count: (u16, u16),
}

/// Count current adult holders whose proficiency and inclusive day recency meet the rules.
pub fn generation_inputs(world: &World, res: &SimResources, known: u32) -> GoodsInputs {
    let mut inputs = GoodsInputs {
        known,
        legacy_practice: super::colony_practice(world),
        progress_permille: res.colony.tech.progress_permille,
        ..GoodsInputs::default()
    };
    let defs = sim_core::colony::knowledge::defs();
    let today = day_of(res.current_tick);
    let window = crate::runtime::knowledge::rule(res).map_or(0, |rule| rule.practice_days);
    for (_, (_, knowledge)) in world
        .query::<(&Agent, &Knowledge)>()
        .without::<&Child>()
        .iter()
    {
        for entry in knowledge.entries() {
            if entry.proficiency >= defs.threshold(entry.id())
                && u64::from(today.saturating_sub(entry.last_used)) <= window
            {
                let count = &mut inputs.practitioners[usize::from(entry.id())];
                *count = count.saturating_add(1);
            }
        }
    }
    inputs
}

/// Resolve eligible rows against an explicit catalog, preserving their authored order.
pub fn eligible_goods(
    catalog: &Catalog,
    entries: &[GoodsEntry],
    rule: &NeighbourRule,
    policy: Option<GoodsPolicy>,
    inputs: &GoodsInputs,
) -> Vec<WeightedGoods> {
    let mut table = Vec::new();
    for entry in entries {
        let Some(kind) = catalog.item_by_name(&entry.item) else {
            continue;
        };
        let made_by = catalog
            .item(kind)
            .and_then(|item| item.made_by.as_deref())
            .and_then(|name| catalog.knowledge_defs().id_of(name));
        let weight = match (entry.tag.as_str(), policy) {
            ("staple", None) => Some(rule.staple_weight),
            ("staple", Some(GoodsPolicy::Scarce { staple_weight, .. })) => Some(staple_weight),
            ("lacked", None) => made_by
                .filter(|&node| {
                    inputs.known & bit(node) != 0 && inputs.legacy_practice & bit(node) == 0
                })
                .map(|_| rule.lack_weight),
            (
                "lacked",
                Some(GoodsPolicy::Scarce {
                    maximum_practitioners,
                    scarce_weight,
                    rare_weight,
                    next_era_cutoff,
                    rare_era,
                    ..
                }),
            ) => made_by
                .filter(|&node| inputs.known & bit(node) != 0)
                .and_then(|node| {
                    let count = inputs.practitioners[usize::from(node)];
                    if count == 0
                        && inputs.progress_permille < next_era_cutoff
                        && catalog
                            .knowledge_defs()
                            .get(node)
                            .is_some_and(|def| def.era == rare_era)
                    {
                        Some(rare_weight)
                    } else if count <= maximum_practitioners {
                        Some(scarce_weight)
                    } else {
                        None
                    }
                }),
            _ => None,
        };
        if let Some(weight) = weight {
            table.push(WeightedGoods {
                kind,
                weight,
                count: entry.count,
            });
        }
    }
    table
}

/// Draw stacks with the historical keyed identity, slots and inclusive ranges.
pub fn draw_goods(seed: u64, rule: &NeighbourRule, id: u32, table: &[WeightedGoods]) -> Vec<Goods> {
    let draw =
        |slot: u32, n: u64| keyed_below(seed, STREAM_TRADE, u64::from(id), 0, slot, 0, n.max(1));
    let total: u64 = table.iter().map(|row| u64::from(row.weight)).sum();
    let (lo, hi) = rule.goods_stacks;
    let stacks = u32::from(lo) + draw(0, u64::from(hi.saturating_sub(lo)) + 1) as u32;
    let (qlo, qhi) = rule.goods_quality;
    let mut out = Vec::new();
    for i in 0..stacks {
        let mut pick = draw(0x10 + i, total);
        let Some(row) = table.iter().find(|row| {
            if pick < u64::from(row.weight) {
                true
            } else {
                pick -= u64::from(row.weight);
                false
            }
        }) else {
            continue;
        };
        let (clo, chi) = row.count;
        let count = clo + draw(0x40 + i, u64::from(chi.saturating_sub(clo)) + 1) as u16;
        let quality = qlo + draw(0x80 + i, u64::from(qhi.saturating_sub(qlo)) + 1) as u8;
        out.push(Goods {
            kind: row.kind,
            count,
            quality,
        });
    }
    out
}

/// Legacy pure entry point: absent policy uses the saved rule's historical weights.
pub fn goods(seed: u64, rule: &NeighbourRule, id: u32, known: u32, practice: u32) -> Vec<Goods> {
    let catalog = sim_core::colony::content::installed_or_bundled();
    let entries = sim_core::rules::neighbour_data().map_or(&[][..], |data| data.goods.as_slice());
    let inputs = GoodsInputs {
        known,
        legacy_practice: practice,
        ..GoodsInputs::default()
    };
    let table = eligible_goods(catalog, entries, rule, None, &inputs);
    draw_goods(seed, rule, id, &table)
}
