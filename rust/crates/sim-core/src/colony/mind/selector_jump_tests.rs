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
    ];
    let mut scratch = EvalScratch::default();
    let mut old = EvalScratch::default();
    for length in 0..=3 {
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
    let p = compile(
        &policy(&[
            node("a", "need", 0, "", 0),
            node("b", "need", 0, "", 0),
            node("c", "need", 0, "", 0).replace("op: \"has\"", "op: \"not\""),
            node("d", "need", 0, "", 0),
        ]),
        &schema(),
    )
    .unwrap();
    for band in &p.bands {
        for (position, id) in band.iter().enumerate() {
            let predicates = &p.nodes[id.index()].predicates;
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
