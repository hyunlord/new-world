//! Behavioural schema and fact-table boundaries, independent of global bootstrap.

use super::*;

pub(super) fn schema() -> MindRegistrySchema {
    MindRegistrySchema {
        facts: vec![
            FactSpec {
                name: "available".into(),
                kind: FactKind::Bool,
            },
            FactSpec {
                name: "cold".into(),
                kind: FactKind::Bool,
            },
            FactSpec {
                name: "hearth_lit".into(),
                kind: FactKind::Bool,
            },
            FactSpec {
                name: "need".into(),
                kind: FactKind::Signed,
            },
            FactSpec {
                name: "other".into(),
                kind: FactKind::Signed,
            },
        ],
        behaviours: vec![
            BehaviourSpec {
                name: "rest".into(),
                target_bindings: vec!["hearth".into()],
            },
            BehaviourSpec {
                name: "wander".into(),
                target_bindings: vec![],
            },
        ],
    }
}
pub(super) fn node(name: &str, band: &str, constant: i64, terms: &str, margin: i64) -> String {
    format!(
        r#"(id: "{name}", band: "{band}", behaviour: "rest",
        when: [(fact: "available", op: "has")],
        score: (constant: {constant}, terms: [{terms}]), hysteresis: {margin},
        life_stages: ["adult", "child", "elder"], tags: [], name_key: "NODE_{name}")"#
    )
}
pub(super) fn policy(nodes: &[String]) -> String {
    let fallback =
        node("idle", "idle", -100, "", 0).replace("behaviour: \"rest\"", "behaviour: \"wander\"");
    format!(
        r#"(schema_version: 1, bands: ["critical", "duty", "need", "work", "leisure", "idle"], nodes: [{}])"#,
        nodes
            .iter()
            .map(String::as_str)
            .chain(std::iter::once(fallback.as_str()))
            .collect::<Vec<_>>()
            .join(",")
    )
}
pub(super) fn facts(policy: &CompiledMind, tick: u64, need: i64, other: i64) -> PersonFacts {
    let mut table = PersonFacts::new(policy, vec![7], tick).expect("sorted agents");
    table.set_life_stage(0, LifeStage::Adult).expect("stage");
    for spec in policy.facts() {
        let value = match spec.name.as_str() {
            "need" => need,
            "other" => other,
            _ => 1,
        };
        table
            .set(policy.fact_id(&spec.name).expect("fact"), 0, value)
            .expect("value");
    }
    table
}
pub(super) const fn key(tick: u64) -> EvalKey {
    EvalKey {
        world_seed: 5,
        agent: 7,
        tick,
        replicate: 0,
    }
}

#[test]
fn strict_loading_names_and_structure_fail_with_context() {
    // Given a valid, locally compiled policy independent of installed content.
    let valid = policy(&[node("food", "need", 10, "", 0)]);
    let bad = [
        (
            valid.replace("schema_version: 1", "schema_version: 2"),
            "schema",
        ),
        (
            valid.replace("schema_version: 1", "schema_version: 1, surprise: true"),
            "surprise",
        ),
        (
            valid.replace("fact: \"available\"", "fact: \"missing\""),
            "unknown fact",
        ),
        (
            valid.replace("behaviour: \"rest\"", "behaviour: \"missing\""),
            "unknown behaviour",
        ),
        (
            valid.replace("band: \"need\"", "band: \"missing\""),
            "unknown band",
        ),
        (
            valid.replace(
                "name_key: \"NODE_food\"",
                "name_key: \"NODE_food\", target: Some(\"missing\")",
            ),
            "unknown target",
        ),
        (
            valid.replace("hysteresis: 0", "hysteresis: -1"),
            "hysteresis",
        ),
        (
            valid.replace(
                "name_key: \"NODE_food\"",
                "name_key: \"NODE_food\", every_ticks: 0",
            ),
            "cadence",
        ),
        (
            valid.replace(
                "name_key: \"NODE_food\"",
                "name_key: \"NODE_food\", every_ticks: 2",
            ),
            "cadence",
        ),
        (
            valid.replace("op: \"has\"", "op: \"has\", value: Some(1)"),
            "has/not",
        ),
        (
            valid.replace("op: \"has\"", "op: \">\", value: Some(0)"),
            "Bool",
        ),
        (
            valid.replace("\"adult\", \"child\", \"elder\"", "\"unknown\""),
            "life stage",
        ),
    ];
    for (text, expected) in bad {
        // When invalid authoring crosses the parse/resolve boundary.
        let error = compile(&text, &schema()).expect_err("invalid policy");
        // Then it is rejected with the concrete source/field reason.
        assert!(error.to_string().contains(expected), "{error}");
    }
}

#[test]
fn duplicate_registry_and_node_names_are_rejected() {
    let text = policy(&[node("a", "need", 1, "", 0)]);
    let mut registry = schema();
    registry.facts.push(registry.facts[0].clone());
    assert!(compile(&text, &registry)
        .expect_err("duplicate")
        .message
        .contains("duplicate"));
    let duplicate = policy(&[node("a", "need", 1, "", 0), node("a", "work", 2, "", 0)]);
    assert!(compile(&duplicate, &schema())
        .expect_err("duplicate")
        .message
        .contains("duplicate"));
}

#[test]
fn columnar_rows_require_sorted_identity_explicit_cells_and_life_stage() {
    let compiled = compile(&policy(&[node("a", "need", 1, "", 0)]), &schema()).expect("policy");
    assert!(matches!(
        PersonFacts::new(&compiled, vec![8, 7], 0),
        Err(MindEvalError::AgentOrder)
    ));
    assert!(matches!(
        PersonFacts::new(&compiled, vec![7, 7], 0),
        Err(MindEvalError::AgentOrder)
    ));
    let mut table = PersonFacts::new(&compiled, vec![7, 8], 0).expect("sorted");
    assert!(matches!(
        table.row(7),
        Err(MindEvalError::MissingLifeStage(0))
    ));
    table.set_life_stage(0, LifeStage::Child).expect("stage");
    assert!(matches!(
        table.row(7),
        Err(MindEvalError::UnwrittenFact { .. })
    ));
    let fact = compiled.fact_id("available").expect("fact");
    assert!(matches!(
        table.set(fact, 0, 2),
        Err(MindEvalError::InvalidBoolean(_))
    ));
    table.set(fact, 0, 1).expect("bool");
    assert_eq!(table.row_index(8), Some(1));
    assert_eq!(
        table.row_at(0).expect("filled").life_stage(),
        LifeStage::Child
    );
    assert_eq!(table.get(fact, 0), Ok(1));
}

#[test]
fn named_random_keys_survive_insertion_while_revisions_bind_changed_dictionaries() {
    let before = compile(&policy(&[node("a", "need", 1, "", 0)]), &schema()).expect("policy");
    let after = compile(
        &policy(&[
            node("other_node", "work", 2, "", 0),
            node("a", "need", 1, "", 0),
        ]),
        &schema(),
    )
    .expect("policy");
    let key_of = |policy: &CompiledMind| {
        policy
            .node(policy.node_id("a").expect("node"))
            .expect("node")
            .random_key
    };
    assert_eq!(key_of(&before), key_of(&after));
    assert_ne!(before.revision(), after.revision());
}

#[test]
fn data_only_warming_node_selects_registered_rest_target_with_exact_witnesses() {
    let warm = node("warm_at_hearth", "need", 20, "", 0)
        .replace(
            "(fact: \"available\", op: \"has\")",
            "(fact: \"cold\", op: \"has\"), (fact: \"hearth_lit\", op: \"has\")",
        )
        .replace(
            "name_key: \"NODE_warm_at_hearth\"",
            "name_key: \"NODE_warm_at_hearth\", target: Some(\"hearth\")",
        );
    let compiled = compile(&policy(&[warm]), &schema()).expect("data-only node");
    let table = facts(&compiled, 0, 0, 0);
    let mut scratch = EvalScratch::default();
    let selected = choose(
        &compiled,
        table.row(7).expect("row"),
        None,
        key(0),
        &mut scratch,
    )
    .expect("choice");
    let node = compiled.node(selected.selected.node).expect("node");
    assert_eq!(node.name, "warm_at_hearth");
    assert_eq!(node.target.as_deref(), Some("hearth"));
    for name in ["cold", "hearth_lit"] {
        assert!(scratch
            .witnesses()
            .iter()
            .any(|w| w.fact == compiled.fact_id(name).expect("fact") && w.value == 1));
    }
    let removed = compile(&policy(&[]), &schema()).expect("remove by data");
    let table = facts(&removed, 0, 0, 0);
    let choice = choose(
        &removed,
        table.row(7).expect("row"),
        None,
        key(0),
        &mut scratch,
    )
    .expect("choice");
    assert_eq!(
        removed.node(choice.selected.node).expect("node").name,
        "idle"
    );
}

#[path = "selection_tests.rs"]
mod selection;

#[test]
fn revision_frames_registry_names_instead_of_delimiters() {
    let text = policy(&[node("a", "need", 1, "", 0)]);
    let mut split = schema();
    split.behaviours.splice(
        0..0,
        ["a", "b"].map(|name| BehaviourSpec {
            name: name.into(),
            target_bindings: vec![],
        }),
    );
    let mut joined = schema();
    joined.behaviours.insert(
        0,
        BehaviourSpec {
            name: "a\0behaviour\0b".into(),
            target_bindings: vec![],
        },
    );
    let first = compile(&text, &split).expect("split registry");
    let second = compile(&text, &joined).expect("joined registry");
    assert_ne!(first.behaviour_id("rest"), second.behaviour_id("rest"));
    assert_ne!(first.revision(), second.revision());
}

#[test]
fn explicit_finish_is_typed_and_cannot_hide_target_or_replace_idle_fallback() {
    let ordinary = policy(&[node("a", "need", 1, "", 0)]);
    let text = ordinary.replace("id: \"a\"", "id: \"a\", finish: Some(\"interrupted\")");
    let compiled = compile(&text, &schema()).expect("explicit finish");
    let node = compiled
        .node(compiled.node_id("a").expect("node"))
        .expect("node");
    assert_eq!(node.finish, Some(FinishMode::Interrupted));
    assert!(compile(&text.replace("interrupted", "unknown"), &schema()).is_err());
    assert!(compile(
        &text.replace(
            "finish: Some(\"interrupted\")",
            "finish: Some(\"completed\"), target: Some(\"hearth\")"
        ),
        &schema()
    )
    .is_err());
    let only_finish =
        policy(&[]).replace("id: \"idle\"", "id: \"idle\", finish: Some(\"completed\")");
    assert!(compile(&only_finish, &schema()).is_err());
}
