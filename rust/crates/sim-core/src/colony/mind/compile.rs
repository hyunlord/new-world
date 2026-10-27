//! Resolve authored names once, rejecting invalid registries and policy structure.

use super::{model::*, raw, MindLoadError};
use std::collections::{BTreeMap, BTreeSet};

fn ensure(ok: bool, context: &str, message: impl Into<String>) -> Result<(), MindLoadError> {
    if ok {
        Ok(())
    } else {
        Err(MindLoadError::new(context, message))
    }
}
fn hash(bytes: &[u8], mut state: u64) -> u64 {
    for byte in bytes {
        state = (state ^ u64::from(*byte)).wrapping_mul(1_099_511_628_211);
    }
    state
}
const OFFSET: u64 = 14_695_981_039_346_656_037;
fn named_key(name: &str) -> u64 {
    hash(name.as_bytes(), hash(b"worldsim-mind-node-v1\0", OFFSET))
}
fn stage_mask(stages: &[String], context: &str) -> Result<u8, MindLoadError> {
    let mut mask = 0;
    for stage in stages {
        let bit = match stage.as_str() {
            "adult" => LifeStage::Adult.bit(),
            "child" => LifeStage::Child.bit(),
            "elder" => LifeStage::Elder.bit(),
            _ => {
                return Err(MindLoadError::new(
                    context,
                    format!("unknown life stage {stage:?}"),
                ))
            }
        };
        ensure(
            mask & bit == 0,
            context,
            format!("duplicate life stage {stage:?}"),
        )?;
        mask |= bit;
    }
    ensure(mask != 0, context, "life_stages must not be empty")?;
    Ok(mask)
}
fn operation(
    raw: &raw::Predicate,
    kind: FactKind,
    context: &str,
) -> Result<(Op, i64), MindLoadError> {
    let op = match raw.op.as_str() {
        "<" => Op::Lt,
        "<=" => Op::Le,
        ">" => Op::Gt,
        ">=" => Op::Ge,
        "==" => Op::Eq,
        "has" => Op::Has,
        "not" => Op::Not,
        _ => {
            return Err(MindLoadError::new(
                context,
                format!("unknown predicate op {:?}", raw.op),
            ))
        }
    };
    match op {
        Op::Has | Op::Not => {
            ensure(
                kind == FactKind::Bool && raw.value.is_none(),
                context,
                format!("fact {:?}: has/not require Bool and no value", raw.fact),
            )?;
            Ok((op, 0))
        }
        Op::Lt | Op::Le | Op::Gt | Op::Ge | Op::Eq => {
            let value = raw
                .value
                .ok_or_else(|| MindLoadError::new(context, "comparison requires value"))?;
            ensure(
                kind != FactKind::Bool || (op == Op::Eq && matches!(value, 0 | 1)),
                context,
                format!("fact {:?}: Bool comparisons require == 0 or == 1", raw.fact),
            )?;
            Ok((op, value))
        }
    }
}

/// Compile a schema1 RON policy using only explicit caller registration metadata.
/// Neither parsing nor resolution reads or changes the process-global catalogue.
pub fn compile(text: &str, schema: &MindRegistrySchema) -> Result<CompiledMind, MindLoadError> {
    let file: raw::File =
        ron::from_str(text).map_err(|error| MindLoadError::new("parse", error.to_string()))?;
    ensure(
        file.schema_version == 1,
        "schema_version",
        "expected schema 1",
    )?;
    ensure(
        file.bands.iter().map(String::as_str).eq(BAND_NAMES),
        "bands",
        "expected critical,duty,need,work,leisure,idle in order",
    )?;
    let mut registered_facts = BTreeMap::new();
    for spec in &schema.facts {
        ensure(
            !spec.name.is_empty() && registered_facts.insert(spec.name.as_str(), spec).is_none(),
            "registry",
            format!("duplicate/empty fact {:?}", spec.name),
        )?;
    }
    let mut behaviour_names = BTreeMap::new();
    for (index, spec) in schema.behaviours.iter().enumerate() {
        ensure(
            !spec.name.is_empty()
                && behaviour_names
                    .insert(spec.name.as_str(), BehaviourId(index))
                    .is_none(),
            "registry",
            format!("duplicate/empty behaviour {:?}", spec.name),
        )?;
        let mut targets = BTreeSet::new();
        for target in &spec.target_bindings {
            ensure(
                !target.is_empty() && targets.insert(target),
                "registry",
                "duplicate/empty target binding",
            )?;
        }
    }
    let mut used_facts = BTreeSet::new();
    for node in &file.nodes {
        for name in node
            .when
            .iter()
            .map(|p| &p.fact)
            .chain(node.score.terms.iter().map(|t| &t.fact))
        {
            ensure(
                registered_facts.contains_key(name.as_str()),
                &node.id,
                format!("unknown fact {name:?}"),
            )?;
            used_facts.insert(name.as_str());
        }
    }
    let mut fact_names = BTreeMap::new();
    let mut facts = Vec::new();
    for name in used_facts {
        let spec = registered_facts
            .get(name)
            .ok_or_else(|| MindLoadError::new("registry", "fact disappeared"))?;
        fact_names.insert(name.to_owned(), FactId(facts.len()));
        facts.push((*spec).clone());
    }
    // Length framing makes registry boundaries unambiguous even with embedded NULs.
    let frame = |bytes: &[u8], state| hash(bytes, hash(&(bytes.len() as u64).to_le_bytes(), state));
    let mut revision = frame(text.as_bytes(), hash(b"worldsim-mind-policy-v2", OFFSET));
    revision = hash(&(schema.facts.len() as u64).to_le_bytes(), revision);
    for fact in &schema.facts {
        revision = frame(fact.name.as_bytes(), revision);
        revision = hash(
            &[match fact.kind {
                FactKind::Bool => 0,
                FactKind::Signed => 1,
            }],
            revision,
        );
    }
    revision = hash(&(schema.behaviours.len() as u64).to_le_bytes(), revision);
    for behaviour in &schema.behaviours {
        revision = frame(behaviour.name.as_bytes(), revision);
        revision = hash(
            &(behaviour.target_bindings.len() as u64).to_le_bytes(),
            revision,
        );
        for target in &behaviour.target_bindings {
            revision = frame(target.as_bytes(), revision);
        }
    }
    let mut result = CompiledMind {
        revision: MindRevision(revision),
        predicate_count: 0,
        nodes: Vec::new(),
        bands: std::array::from_fn(|_| Vec::new()),
        facts,
        fact_names,
        behaviours: schema.behaviours.clone(),
    };
    let mut shared_predicates = Vec::new();
    let mut names = BTreeSet::new();
    let mut keys = BTreeSet::new();
    let mut fallback_stages = 0;
    for node in file.nodes {
        ensure(
            !node.id.is_empty() && names.insert(node.id.clone()),
            &node.id,
            "duplicate/empty node id",
        )?;
        ensure(
            !node.name_key.is_empty(),
            &node.id,
            "name_key must not be empty",
        )?;
        let band = BAND_NAMES
            .iter()
            .position(|name| *name == node.band)
            .map(BandId)
            .ok_or_else(|| MindLoadError::new(&node.id, format!("unknown band {:?}", node.band)))?;
        let behaviour = *behaviour_names
            .get(node.behaviour.as_str())
            .ok_or_else(|| {
                MindLoadError::new(&node.id, format!("unknown behaviour {:?}", node.behaviour))
            })?;
        if let Some(target) = &node.target {
            ensure(
                schema.behaviours[behaviour.0]
                    .target_bindings
                    .contains(target),
                &node.id,
                format!(
                    "unknown target {target:?} for behaviour {:?}",
                    node.behaviour
                ),
            )?;
        }
        let finish = match node.finish.as_deref() {
            None => None,
            Some("completed") => Some(FinishMode::Completed),
            Some("interrupted") => Some(FinishMode::Interrupted),
            Some(value) => {
                return Err(MindLoadError::new(
                    &node.id,
                    format!("unknown finish mode {value:?}"),
                ))
            }
        };
        ensure(
            finish.is_none() || node.target.is_none(),
            &node.id,
            "finish node must omit target",
        )?;
        ensure(
            node.hysteresis >= 0,
            &node.id,
            "hysteresis must be nonnegative",
        )?;
        ensure(
            node.every_ticks > 0 && (band.0 >= 4 || node.every_ticks == 1),
            &node.id,
            "positive cadence required; critical/duty/need/work require every_ticks 1",
        )?;
        ensure(
            !node.when.is_empty() || !node.score.terms.is_empty(),
            &node.id,
            "node must read at least one explanatory fact",
        )?;
        let stages = stage_mask(&node.life_stages, &node.id)?;
        if band.0 == 5 && node.every_ticks == 1 && finish.is_none() {
            fallback_stages |= stages;
        }
        let random_key = named_key(&node.id);
        ensure(
            keys.insert(random_key),
            &node.id,
            "stable node key collision",
        )?;
        let mut predicates = Vec::new();
        let mut predicate_keys = BTreeSet::new();
        for raw in node.when {
            ensure(
                predicate_keys.insert((raw.fact.clone(), raw.op.clone(), raw.value)),
                &node.id,
                "duplicate predicate",
            )?;
            let fact = *result
                .fact_names
                .get(&raw.fact)
                .ok_or_else(|| MindLoadError::new(&node.id, "unknown fact"))?;
            let (op, value) = operation(&raw, result.facts[fact.0].kind, &node.id)?;
            let identity = (fact, op, value);
            let cache_index = match shared_predicates.iter().position(|key| *key == identity) {
                Some(index) => index,
                None => {
                    let index = shared_predicates.len();
                    shared_predicates.push(identity);
                    index
                }
            };
            predicates.push(Predicate {
                cache_index,
                fact,
                op,
                value,
            });
        }
        let mut terms = Vec::new();
        let mut term_facts = BTreeSet::new();
        for raw in node.score.terms {
            ensure(
                term_facts.insert(raw.fact.clone()),
                &node.id,
                "duplicate score fact",
            )?;
            let fact = *result
                .fact_names
                .get(&raw.fact)
                .ok_or_else(|| MindLoadError::new(&node.id, "unknown score fact"))?;
            terms.push(Term {
                fact,
                weight: raw.weight_permille,
            });
        }
        let id = NodeId(result.nodes.len());
        result.bands[band.0].push(id);
        result.nodes.push(MindNode {
            id,
            name: node.id,
            band,
            behaviour,
            target: node.target,
            finish,
            name_key: node.name_key,
            tags: node.tags,
            hysteresis: node.hysteresis,
            every_ticks: node.every_ticks,
            random_key,
            stages,
            predicates,
            terms,
            constant: node.score.constant,
        });
    }
    ensure(
        fallback_stages == 7,
        "idle",
        "every life stage requires an idle node with every_ticks 1",
    )?;
    result.predicate_count = shared_predicates.len();
    Ok(result)
}
