//! First-generation caravan inputs; legacy and scarcity policies are labelled separately.
use super::{
    manifest::{Config, Policy},
    visits::{goods_rows, GoodsRow},
};
use serde::Serialize;
use sim_core::colony::{
    knowledge::{self, Knowledge},
    ItemKind,
};
use sim_core::components::{Agent, Child};
use sim_core::rules::GoodsPolicy;
use sim_engine::{RuntimeSystem, SimEngine};

#[derive(Debug, Serialize)]
pub struct KnowledgeInput {
    pub node_id: u8,
    pub proficiency: u16,
    pub last_used: u16,
}
#[derive(Debug, Serialize)]
pub struct PractitionerInput {
    pub agent_id: u64,
    pub is_child: bool,
    pub knowledge: Vec<KnowledgeInput>,
}
#[derive(Debug, Serialize)]
pub struct PractitionerRow {
    pub node_id: u8,
    pub count: u64,
}
#[derive(Debug, Serialize)]
pub struct EligibleRow {
    pub authored_index: u32,
    pub item_code: u32,
    pub made_by: Option<u8>,
    pub classification: &'static str,
    pub weight: u32,
    pub count_min: u16,
    pub count_max: u16,
}
#[derive(Debug, Serialize)]
pub struct CaravanGeneration {
    pub caravan_id: u32,
    pub neighbour_id: u8,
    pub executed_tick: u64,
    pub policy: Policy,
    pub neighbour_known_mask: u64,
    #[serde(skip_serializing_if = "super::types::zero_word")]
    pub neighbour_known_mask_high: u64,
    #[serde(skip_serializing_if = "super::types::zero_upper")]
    pub neighbour_known_mask_upper: [u64; 2],
    pub old_all_knowledge_practice_mask: u64,
    #[serde(skip_serializing_if = "super::types::zero_word")]
    pub old_all_knowledge_practice_mask_high: u64,
    #[serde(skip_serializing_if = "super::types::zero_upper")]
    pub old_all_knowledge_practice_mask_upper: [u64; 2],
    pub tech_progress_permille: u16,
    pub practice_days: u64,
    pub practitioner_rows: Vec<PractitionerRow>,
    pub practitioner_inputs: Vec<PractitionerInput>,
    pub eligible_rows: Vec<EligibleRow>,
    pub initial_goods: Vec<GoodsRow>,
    /// C2-b: yield-item tags of every world deposit (hidden/exhausted included), sorted.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outcrop_yield_tags: Vec<String>,
}

/// Pin authored policy independently of the runtime goods classifier.
pub(super) fn policy_weights(
    policy: Policy,
    authored: Option<&GoodsPolicy>,
    legacy: (u32, u32),
) -> Result<(u32, u32), String> {
    let matches = match (policy, authored) {
        (Policy::Old3_1, None) => legacy == (3, 1),
        (Policy::Old3_1, Some(_)) | (Policy::Scarce30_10_1, None) => false,
        (
            Policy::Scarce30_10_1,
            Some(GoodsPolicy::Scarce {
                maximum_practitioners,
                scarce_weight,
                staple_weight,
                rare_weight,
                next_era_cutoff,
                rare_era,
            }),
        ) => {
            (
                *maximum_practitioners,
                *scarce_weight,
                *staple_weight,
                *rare_weight,
                *next_era_cutoff,
                *rare_era,
            ) == (2, 30, 10, 1, 750, knowledge::Era::Neolithic)
        }
    };
    if !matches {
        return Err("manifest policy does not match authored goods policy and legacy rule".into());
    }
    Ok(match policy {
        Policy::Old3_1 => (3, 1),
        Policy::Scarce30_10_1 => (30, 10),
    })
}

pub fn capture(e: &SimEngine, id: u32, config: &Config) -> Result<CaravanGeneration, String> {
    let r = &e.resources;
    let generation = sim_systems::runtime::neighbours::NeighbourSystem;
    let visitor = sim_systems::runtime::neighbours::VisitorSystem;
    if generation.priority() != 174
        || generation.tick_interval() != 1440
        || generation.tick_offset(r) != 0
        || visitor.priority() != 175
        || visitor.tick_interval() != 3
        || visitor.tick_offset(r) != 1
    {
        return Err("caravan capture schedule changed; source proof must be renewed".into());
    }
    let nb = &r.colony.neighbours;
    let c = nb
        .caravans
        .iter()
        .find(|c| c.id == id)
        .ok_or("caravan disappeared")?;
    if !matches!(c.state, sim_core::colony::neighbours::VisitState::Arriving)
        || !r.current_tick.is_multiple_of(1440)
        || r.current_tick % 3 == 1
    {
        return Err("first caravan sighting violates generation/visitor schedule".into());
    }
    let rule = sim_systems::runtime::neighbours::rule(r).ok_or("missing neighbour rule")?;
    let known = nb.neighbour(c.neighbour).ok_or("missing neighbour")?.known;
    let practice = e
        .world
        .query::<&Knowledge>()
        .iter()
        .fold(knowledge::KnowledgeMask::ZERO, |mask, (_, k)| {
            mask | k.practice()
        });
    let window = sim_systems::runtime::knowledge::rule(r).map_or(0, |k| k.practice_days);
    let today = knowledge::day_of(r.current_tick);
    let mut inputs: Vec<_> = e
        .world
        .query::<(&Agent, &Knowledge, Option<&Child>)>()
        .iter()
        .map(|(_, (a, k, child))| PractitionerInput {
            agent_id: a.id,
            is_child: child.is_some(),
            knowledge: k
                .entries()
                .iter()
                .map(|v| KnowledgeInput {
                    node_id: v.id(),
                    proficiency: v.proficiency,
                    last_used: v.last_used,
                })
                .collect(),
        })
        .collect();
    inputs.sort_by_key(|r| r.agent_id);
    let counts: Vec<_> = knowledge::defs()
        .nodes
        .iter()
        .map(|node| PractitionerRow {
            node_id: node.id,
            count: inputs
                .iter()
                .filter(|p| {
                    !p.is_child
                        && p.knowledge.iter().any(|k| {
                            k.node_id == node.id
                                && k.proficiency >= node.threshold
                                && u64::from(today.saturating_sub(k.last_used)) <= window
                        })
                })
                .fold(0, |n, _| n + 1),
        })
        .collect();
    let mut eligible = Vec::new();
    let entries = sim_core::rules::neighbour_data().ok_or("missing authored goods")?;
    let (artifact_weight, staple_weight) = policy_weights(
        config.manifest.policy,
        entries.goods_policy.as_ref(),
        (rule.lack_weight, rule.staple_weight),
    )?;
    for (index, entry) in entries.goods.iter().enumerate() {
        let kind = ItemKind::from_name(&entry.item).ok_or("unresolved authored goods item")?;
        // C2-b gated rows, recomputed from observed adult proficiency and raw deposit yields.
        let known_by_adult = |name: &String| {
            knowledge::id(name).is_some_and(|node| {
                inputs.iter().any(|p| {
                    !p.is_child
                        && p.knowledge.iter().any(|k| {
                            k.node_id == node && k.proficiency >= knowledge::defs().threshold(node)
                        })
                })
            })
        };
        let tag_present = |tag: &String| {
            r.ore.deposits.values().any(|d| {
                sim_core::colony::content::installed_or_bundled()
                    .outcrop(d.kind)
                    .is_some_and(|def| def.yield_item.definition().tags.contains(tag))
            })
        };
        if !entry.requires_knowledge.iter().all(known_by_adult)
            || entry.absent_outcrop_tag.as_ref().is_some_and(tag_present)
        {
            continue;
        }
        let made_by = sim_core::colony::values::item_data(kind)
            .and_then(|d| d.made_by.as_deref())
            .and_then(knowledge::id);
        let weight = match entry.tag.as_str() {
            "staple" => Some((staple_weight, "staple")),
            "lacked" => made_by
                .filter(|&n| known & knowledge::bit(n) != 0)
                .and_then(|node| match config.manifest.policy {
                    Policy::Old3_1 => (practice & knowledge::bit(node) == 0)
                        .then_some((artifact_weight, "lacked")),
                    Policy::Scarce30_10_1 => {
                        let count = counts
                            .iter()
                            .find(|p| p.node_id == node)
                            .map_or(0, |p| p.count);
                        let rare = knowledge::defs()
                            .get(node)
                            .is_some_and(|d| d.era == knowledge::Era::Neolithic)
                            && r.colony.tech.progress_permille < 750
                            && count == 0;
                        if rare {
                            Some((1, "rare"))
                        } else {
                            (count <= 2).then_some((artifact_weight, "scarce"))
                        }
                    }
                }),
            _ => None,
        };
        if let Some((weight, classification)) = weight {
            eligible.push(EligibleRow {
                authored_index: u32::try_from(index).map_err(|_| "goods index exceeds u32")?,
                item_code: kind.code(),
                made_by,
                classification,
                weight,
                count_min: entry.count.0,
                count_max: entry.count.1,
            });
        }
    }
    Ok(CaravanGeneration {
        caravan_id: id,
        neighbour_id: c.neighbour,
        executed_tick: r.current_tick,
        policy: config.manifest.policy,
        neighbour_known_mask: known.words()[0],
        neighbour_known_mask_high: known.words()[1],
        neighbour_known_mask_upper: known.upper(),
        old_all_knowledge_practice_mask: practice.words()[0],
        old_all_knowledge_practice_mask_high: practice.words()[1],
        old_all_knowledge_practice_mask_upper: practice.upper(),
        tech_progress_permille: r.colony.tech.progress_permille,
        practice_days: window,
        practitioner_rows: counts,
        practitioner_inputs: inputs,
        eligible_rows: eligible,
        initial_goods: goods_rows(&c.goods),
        outcrop_yield_tags: r
            .ore
            .deposits
            .values()
            .filter_map(|d| sim_core::colony::content::installed_or_bundled().outcrop(d.kind))
            .flat_map(|def| def.yield_item.definition().tags.iter().cloned())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect(),
    })
}

#[cfg(test)]
mod c1_json_tests {
    use super::*;
    #[test]
    fn c1_probe_trade_mask_json_preserves_both_words() {
        let mut row = CaravanGeneration {
            caravan_id: 1,
            neighbour_id: 0,
            executed_tick: 0,
            policy: Policy::Old3_1,
            neighbour_known_mask: u64::MAX,
            neighbour_known_mask_high: 0,
            neighbour_known_mask_upper: [0, 0],
            old_all_knowledge_practice_mask: u64::MAX,
            old_all_knowledge_practice_mask_high: 0,
            old_all_knowledge_practice_mask_upper: [0, 0],
            tech_progress_permille: 0,
            practice_days: 1,
            practitioner_rows: vec![],
            practitioner_inputs: vec![],
            eligible_rows: vec![],
            initial_goods: vec![],
            outcrop_yield_tags: vec![],
        };
        let low = serde_json::to_value(&row).expect("legacy JSON");
        assert!(low.get("neighbour_known_mask_high").is_none());
        assert!(low.get("old_all_knowledge_practice_mask_high").is_none());
        row.neighbour_known_mask_high = (1_u64 << 63) | 1;
        row.old_all_knowledge_practice_mask_high = u64::MAX;
        let json = serde_json::to_value(&row).expect("full-width JSON");
        assert_eq!(json["neighbour_known_mask"].as_u64(), Some(u64::MAX));
        assert_eq!(
            json["neighbour_known_mask_high"].as_u64(),
            Some((1_u64 << 63) | 1)
        );
        assert_eq!(
            json["old_all_knowledge_practice_mask_high"].as_u64(),
            Some(u64::MAX)
        );
    }
}
