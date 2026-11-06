//! Exact differential observations against candidate4's frozen full scan.
use super::*;
use crate::colony::mind::tests::{key, node, policy, schema};
use crate::colony::mind::{compile, LifeStage, MindRevision};
use std::cell::RefCell;
#[path = "selector_jump_reference.rs"]
mod reference;

struct Reader<'a, const STABLE: bool> {
    policy: &'a CompiledMind,
    tick: u64,
    values: Vec<i64>,
    fail: Option<FactId>,
    reads: Vec<FactId>,
    stages: RefCell<Vec<LifeStage>>,
    changing_stage: bool,
}
impl<const STABLE: bool> FactReader for Reader<'_, STABLE> {
    const STABLE_READS: bool = STABLE;
    type Error = MindEvalError;
    fn revision(&self) -> MindRevision {
        self.policy.revision()
    }
    fn agent(&self) -> u64 {
        7
    }
    fn tick(&self) -> u64 {
        self.tick
    }
    fn life_stage(&self) -> LifeStage {
        let mut trace = self.stages.borrow_mut();
        let stage = if self.changing_stage && trace.len() % 3 == 1 {
            LifeStage::Child
        } else {
            LifeStage::Adult
        };
        trace.push(stage);
        stage
    }
    fn get(&mut self, fact: FactId) -> Result<i64, Self::Error> {
        self.reads.push(fact);
        if self.fail == Some(fact) {
            return Err(MindEvalError::InvalidIndex("injected fact"));
        }
        let value = self.values[fact.index()];
        Ok(if STABLE {
            value
        } else {
            value.saturating_add((self.reads.len() % 2) as i64)
        })
    }
}
#[allow(clippy::too_many_arguments)]
fn compare<const STABLE: bool>(
    p: &CompiledMind,
    tick: u64,
    values: Vec<i64>,
    fail: Option<FactId>,
    running: Option<RunningChoice>,
    changing_stage: bool,
    scratch: &mut EvalScratch,
    old_scratch: &mut EvalScratch,
) {
    let make = || Reader::<STABLE> {
        policy: p,
        tick,
        values: values.clone(),
        fail,
        reads: vec![],
        stages: RefCell::new(vec![]),
        changing_stage,
    };
    let mut actual = make();
    let mut old = make();
    assert_eq!(
        choose_with_reader(p, &mut actual, running, key(tick), scratch),
        reference::choose_with_reader(p, &mut old, running, key(tick), old_scratch)
    );
    assert_eq!(scratch.witnesses(), old_scratch.witnesses());
    assert_eq!(actual.reads, old.reads);
    assert_eq!(actual.stages.into_inner(), old.stages.into_inner());
}
fn custom(name: &str, band: &str, predicates: &str, cadence: u32, stage: &str) -> String {
    node(name, band, 1, "(fact: \"other\", weight_permille: 1000)", 3)
        .replace("(fact: \"available\", op: \"has\")", predicates)
        .replace("tags: []", &format!("tags: [], every_ticks: {cadence}"))
        .replace("[\"adult\", \"child\", \"elder\"]", stage)
}
#[test]
fn jumps_preserve_full_result_roles_reads_and_structural_calls() {
    let terms = [
        "(fact: \"available\", op: \"has\")",
        "(fact: \"need\", op: \">=\", value: Some(0))",
        "(fact: \"cold\", op: \"not\")",
        "(fact: \"need\", op: \">=\", value: Some(-1))",
        "(fact: \"need\", op: \">=\", value: Some(-2))",
        "(fact: \"need\", op: \">=\", value: Some(-3))",
        "(fact: \"need\", op: \">=\", value: Some(-4))",
        "(fact: \"need\", op: \">=\", value: Some(-5))",
    ];
    let mut scratch = EvalScratch::default();
    let mut old = EvalScratch::default();
    for length in 0..=terms.len() {
        let prefix = terms[..length].join(",");
        let mut nodes = vec![];
        for index in 0..6 {
            nodes.push(custom(
                &format!("shared_{index}"),
                "leisure",
                &prefix,
                if index % 2 == 0 { 1 } else { 3 },
                if index == 2 {
                    "[\"child\"]"
                } else {
                    "[\"adult\", \"child\", \"elder\"]"
                },
            ));
        }
        let tail = "(fact: \"hearth_lit\", op: \"has\")";
        nodes.push(custom(
            "new_tail",
            "leisure",
            &if prefix.is_empty() {
                tail.to_owned()
            } else {
                format!("{prefix},{tail}")
            },
            1,
            "[\"adult\"]",
        ));
        nodes.push(custom("short", "leisure", terms[0], 1, "[\"adult\"]"));
        nodes.push(custom(
            "reordered",
            "leisure",
            &terms.iter().rev().copied().collect::<Vec<_>>().join(","),
            1,
            "[\"adult\"]",
        ));
        nodes.push(custom("again", "idle", &prefix, 1, "[\"adult\"]"));
        let p = compile(&policy(&nodes), &schema()).unwrap();
        for tick in 0..6 {
            for bits in 0..16 {
                let values = p
                    .facts()
                    .iter()
                    .map(|f| match f.name.as_str() {
                        "available" => bits & 1,
                        "cold" => (bits >> 1) & 1,
                        "hearth_lit" => 1,
                        "need" => {
                            if bits & 4 == 0 {
                                -1
                            } else {
                                1
                            }
                        }
                        _ => {
                            if bits & 8 == 0 {
                                2
                            } else {
                                i64::MAX
                            }
                        }
                    })
                    .collect::<Vec<_>>();
                for running in std::iter::once(None).chain(p.nodes().iter().flat_map(|n| {
                    [false, true].map(|finished| {
                        Some(RunningChoice {
                            revision: p.revision(),
                            node: n.id,
                            finished,
                        })
                    })
                })) {
                    compare::<true>(
                        &p,
                        tick,
                        values.clone(),
                        None,
                        running,
                        true,
                        &mut scratch,
                        &mut old,
                    );
                    compare::<false>(
                        &p,
                        tick,
                        values.clone(),
                        None,
                        running,
                        false,
                        &mut scratch,
                        &mut old,
                    );
                }
                for fact in p.facts().iter().map(|f| p.fact_id(&f.name).unwrap()) {
                    compare::<true>(
                        &p,
                        tick,
                        values.clone(),
                        Some(fact),
                        None,
                        true,
                        &mut scratch,
                        &mut old,
                    );
                }
            }
        }
    }
}
#[test]
fn compiled_jump_destinations_are_exact_contiguous_prefix_boundaries() {
    let prefix = "(fact: \"available\", op: \"has\"),(fact: \"need\", op: \">=\", value: Some(0)),(fact: \"cold\", op: \"not\")";
    let changed = prefix.replace("op: \"not\"", "op: \"has\"");
    let p = compile(
        &policy(&[
            custom("a", "need", prefix, 1, "[\"adult\"]"),
            custom("b", "need", prefix, 1, "[\"adult\"]"),
            custom("c", "need", &changed, 1, "[\"adult\"]"),
            custom("d", "need", prefix, 1, "[\"adult\"]"),
        ]),
        &schema(),
    )
    .unwrap();
    for band in &p.bands {
        for (position, id) in band.iter().enumerate() {
            let predicates = &p.nodes[id.index()].predicates;
            let offset = p.node_predicate_offsets[id.index()];
            let ends = &p.false_prefix_end[offset..offset + predicates.len()];
            assert!(ends.windows(2).all(|pair| pair[0] >= pair[1]));
            for next in position + 1..band.len() {
                let shared = (position + 1..=next)
                    .map(|index| {
                        predicates
                            .iter()
                            .zip(&p.nodes[band[index].index()].predicates)
                            .take_while(|(a, b)| a.cache_index == b.cache_index)
                            .count()
                    })
                    .min()
                    .unwrap();
                for proved in 0..=predicates.len() {
                    assert_eq!(
                        ends[..proved].partition_point(|end| *end > next),
                        proved.min(shared)
                    );
                }
            }
            for depth in 0..predicates.len() {
                let prefix = &predicates[..=depth];
                let end = (position + 1..band.len())
                    .find(|next| {
                        let other = &p.nodes[band[*next].index()].predicates;
                        other.len() <= depth
                            || !prefix
                                .iter()
                                .zip(other)
                                .all(|(a, b)| a.cache_index == b.cache_index)
                    })
                    .unwrap_or(band.len());
                assert_eq!(
                    p.false_prefix_end[p.node_predicate_offsets[id.index()] + depth],
                    end
                );
            }
        }
    }
}

#[test]
fn true_prefix_keeps_absolute_rejections_cap_order_and_clears_reused_scratch() {
    let nodes: Vec<_> = (0..20).map(|index| custom(&format!("reject_{index}"), "need",
        &format!("(fact: \"available\", op: \"has\"),(fact: \"need\", op: \">=\", value: Some(0)),(fact: \"other\", op: \">=\", value: Some({}))", index + 1),
        1, "[\"adult\"]")).collect();
    let p = compile(&policy(&nodes), &schema()).unwrap();
    let mut scratch = EvalScratch::default();
    for value in [0, 100, 0] {
        let mut reader = Reader::<true> {
            policy: &p,
            tick: 0,
            values: p
                .facts()
                .iter()
                .map(|f| if f.name == "other" { value } else { 1 })
                .collect(),
            fail: None,
            reads: vec![],
            stages: RefCell::new(vec![]),
            changing_stage: false,
        };
        let selected = choose_with_reader(&p, &mut reader, None, key(0), &mut scratch).unwrap();
        if value == 0 {
            assert_eq!(scratch.rejected.len(), REJECTED_CAP);
            for (index, rejected) in scratch.rejected.iter().enumerate() {
                assert_eq!(
                    rejected.node,
                    p.node_id(&format!("reject_{index}")).unwrap()
                );
                assert_eq!(rejected.depth, 2);
                assert_eq!(rejected.fact, p.fact_id("other").unwrap());
                assert_eq!(rejected.op_str, ">=");
                assert_eq!(rejected.threshold, index as i64 + 1);
                assert_eq!(rejected.actual, 0);
            }
            let blocked = scratch.blocked_candidates(selected.selected.band);
            assert_eq!(blocked.len(), 2);
            for (index, blocked) in blocked.iter().enumerate() {
                assert_eq!(blocked.node, p.node_id(&format!("reject_{index}")).unwrap());
                assert_eq!(blocked.predicate_depth, 2);
                assert_eq!(blocked.threshold, index as i64 + 1);
            }
        } else {
            assert!(scratch.rejected.is_empty());
            assert!(scratch
                .blocked_candidates(selected.selected.band)
                .is_empty());
        }
    }
}
