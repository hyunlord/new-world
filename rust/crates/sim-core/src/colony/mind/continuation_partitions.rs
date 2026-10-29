//! Immutable integer transition points for conservative predicate-equivalent replay.
use super::{CompiledMind, FactId, Op, Predicate};

#[derive(Debug, Clone)]
pub(super) struct PredicatePartitions {
    boundaries: Vec<Vec<i64>>,
}
impl PredicatePartitions {
    pub(super) fn new(policy: &CompiledMind) -> Self {
        let mut boundaries = vec![Vec::new(); policy.facts.len()];
        for predicate in policy.nodes.iter().flat_map(|node| &node.predicates) {
            add_boundaries(&mut boundaries[predicate.fact.index()], predicate);
        }
        for points in &mut boundaries {
            points.sort_unstable();
            points.dedup();
        }
        Self { boundaries }
    }

    pub(super) fn equivalent(&self, fact: FactId, previous: i64, current: i64) -> bool {
        let points = &self.boundaries[fact.index()];
        let lower = previous.min(current);
        let upper = previous.max(current);
        let next = points.partition_point(|&point| point <= lower);
        points.get(next).is_none_or(|&point| point > upper)
    }
}

fn add_boundaries(points: &mut Vec<i64>, predicate: &Predicate) {
    let threshold = predicate.value;
    match predicate.op {
        Op::Lt | Op::Ge => {
            if threshold != i64::MIN {
                points.push(threshold);
            }
        }
        Op::Le | Op::Gt => {
            if let Some(next) = threshold.checked_add(1) {
                points.push(next);
            }
        }
        Op::Eq => {
            if threshold != i64::MIN {
                points.push(threshold);
            }
            if let Some(next) = threshold.checked_add(1) {
                points.push(next);
            }
        }
        Op::Has => points.extend([1, 2]),
        Op::Not => points.extend([0, 1]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::colony::mind::continuation::matches_predicate;

    #[test]
    fn every_operator_and_extreme_threshold_preserves_predicate_truth() {
        let thresholds = [i64::MIN, i64::MIN + 1, -1, 0, 1, 2, i64::MAX - 1, i64::MAX];
        for op in [Op::Lt, Op::Le, Op::Gt, Op::Ge, Op::Eq, Op::Has, Op::Not] {
            for threshold in thresholds {
                let predicate = Predicate {
                    cache_index: 0,
                    fact: FactId(0),
                    op,
                    value: threshold,
                };
                let mut points = Vec::new();
                add_boundaries(&mut points, &predicate);
                points.sort_unstable();
                points.dedup();
                let mut values = thresholds.to_vec();
                for &point in &points {
                    values.push(point);
                    if let Some(before) = point.checked_sub(1) {
                        values.push(before);
                    }
                    if let Some(after) = point.checked_add(1) {
                        values.push(after);
                    }
                    assert_ne!(
                        matches_predicate(point - 1, &predicate),
                        matches_predicate(point, &predicate),
                        "every emitted boundary must be an actual transition",
                    );
                }
                let partitions = PredicatePartitions {
                    boundaries: vec![points],
                };
                for &old in &values {
                    for &new in &values {
                        if partitions.equivalent(FactId(0), old, new) {
                            assert_eq!(
                                matches_predicate(old, &predicate),
                                matches_predicate(new, &predicate)
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn extrema_constants_and_equality_disjoint_regions_remain_conservative() {
        for (op, threshold) in [
            (Op::Lt, i64::MIN),
            (Op::Ge, i64::MIN),
            (Op::Le, i64::MAX),
            (Op::Gt, i64::MAX),
        ] {
            let mut points = Vec::new();
            add_boundaries(
                &mut points,
                &Predicate {
                    cache_index: 0,
                    fact: FactId(0),
                    op,
                    value: threshold,
                },
            );
            assert!(points.is_empty());
        }
        let partitions = PredicatePartitions {
            boundaries: vec![vec![1, 2]],
        };
        assert!(!partitions.equivalent(FactId(0), 0, 2));
        assert!(partitions.equivalent(FactId(0), i64::MIN, 0));
        assert!(partitions.equivalent(FactId(0), 2, i64::MAX));
    }
    #[test]
    fn compiled_partitions_include_predicates_from_every_policy_band() {
        use crate::colony::mind::{
            compile,
            tests::{node, policy, schema},
        };
        let custom = |name, band, predicate| {
            node(name, band, 0, "", 0).replace("(fact: \"available\", op: \"has\")", predicate)
        };
        let compiled = compile(
            &policy(&[
                custom(
                    "urgent",
                    "critical",
                    "(fact:\"need\",op:\">=\",value:Some(0))",
                ),
                custom("owner", "need", "(fact:\"need\",op:\"<\",value:Some(10))"),
                custom("other", "work", "(fact:\"need\",op:\"==\",value:Some(5))"),
            ]),
            &schema(),
        )
        .unwrap();
        let partitions = PredicatePartitions::new(&compiled);
        let fact = compiled.fact_id("need").unwrap();
        assert_eq!(partitions.boundaries[fact.index()], [0, 5, 6, 10]);
        assert!(partitions.equivalent(fact, 1, 4));
        assert!(!partitions.equivalent(fact, 4, 5));
        assert!(!partitions.equivalent(fact, 5, 6));
        assert!(!partitions.equivalent(fact, 9, 10));
    }
}
