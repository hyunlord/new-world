//! M4: the knowledge bridge collectors (`ffi/knowledge.rs`) on built engines —
//! content and row order, and their empty form in the baseline world.

use sim_bridge::ffi::{
    collect_agent_knowledge, collect_knowledge_waits, collect_teach_orders, collect_tech,
    collect_tech_node,
};
use sim_core::colony::knowledge::{self, defs, Knowledge, TechStatus};
use sim_core::colony::Command;
use sim_core::components::{Agent, Child};
use sim_systems::commands::submit;

fn id(name: &str) -> u8 {
    knowledge::id(name).unwrap_or_else(|| panic!("no node {name}"))
}

#[test]
fn knowledge_reads_are_empty_in_the_baseline_world() {
    let mut engine = sim_test::baseline_engine(3);
    engine.tick();
    let tech = collect_tech(&engine.world, &engine.resources);
    assert_eq!((tech.version, tech.nodes.len(), tech.at_risk), (0, 0, 0));
    assert!(!collect_tech_node(&engine.world, &engine.resources, 0).found);
    assert!(collect_teach_orders(&engine.world, &engine.resources).is_empty());
    assert!(collect_knowledge_waits(&engine.world, &engine.resources).is_empty());
    let someone = engine
        .world
        .query::<&Agent>()
        .iter()
        .map(|(_, a)| a.id)
        .min()
        .expect("an agent");
    let view = collect_agent_knowledge(&engine.world, &engine.resources, someone as i64);
    assert!(
        view.rows.is_empty(),
        "nobody knows anything in the baseline"
    );
}

#[test]
fn the_tech_view_lists_every_node_in_id_order_with_the_aggregate() {
    let mut engine = sim_test::colony_engine(1);
    for _ in 0..241 {
        engine.tick();
    }
    let res = &engine.resources;
    let tech = collect_tech(&engine.world, res);
    assert_eq!(tech.nodes.len(), defs().nodes.len());
    for (i, row) in tech.nodes.iter().enumerate() {
        assert_eq!(usize::from(row.id), i, "id order");
        let state = &res.colony.tech.states[i];
        assert_eq!(row.status, state.status.code());
        assert_eq!(
            (row.practitioners, row.holders),
            (state.practitioners, state.holders)
        );
        assert_eq!(row.threshold, defs().threshold(row.id));
    }
    let at_risk = tech
        .nodes
        .iter()
        .filter(|r| r.status == TechStatus::AtRisk.code())
        .count();
    assert!(
        at_risk >= 2,
        "the doorway and hide scraping start with one holder: {at_risk}"
    );
    assert_eq!(
        tech.at_risk as usize, at_risk,
        "the badge counts the at-risk rows"
    );
    // Two of ten founders know timber walls (weight 2 of 14): 2/10 × 2/14 = 28 ‰.
    assert_eq!(tech.progress, 28);
    let door = &tech.nodes[usize::from(id("doorway"))];
    assert_eq!((door.status, door.holders), (TechStatus::AtRisk.code(), 1));
    assert!(door.first_name >= 0, "the holder is named");
}

#[test]
fn node_and_person_views_agree_with_the_component() {
    let mut engine = sim_test::colony_engine(1);
    for _ in 0..241 {
        engine.tick();
    }
    let door = id("doorway");
    let holder = engine.resources.colony.tech.states[usize::from(door)].names[0];
    let node = collect_tech_node(&engine.world, &engine.resources, i64::from(door));
    assert!(node.found);
    assert_eq!(node.row.id, door);
    let named: Vec<u64> = node.practitioners.iter().map(|p| p.id).collect();
    assert_eq!(named, vec![holder]);
    assert!(node.practitioners[0].proficiency >= defs().threshold(door));
    assert!(
        !node.gates.is_empty(),
        "the doorway gates the door blueprint"
    );
    assert!(!collect_tech_node(&engine.world, &engine.resources, 99).found);

    let view = collect_agent_knowledge(&engine.world, &engine.resources, holder as i64);
    assert!(view.found);
    let k = engine
        .world
        .query::<(&Agent, &Knowledge)>()
        .iter()
        .find(|(_, (a, _))| a.id == holder)
        .map(|(_, (_, k))| k.clone())
        .expect("the holder's knowledge");
    let rows: Vec<(u8, u16, u8)> = view
        .rows
        .iter()
        .map(|r| (r.id, r.proficiency, r.source))
        .collect();
    let want: Vec<(u8, u16, u8)> = k
        .entries()
        .iter()
        .map(|e| (e.id(), e.proficiency, e.source().code()))
        .collect();
    assert_eq!(rows, want, "one row per entry, in id order");
    assert!(rows.windows(2).all(|w| w[0].0 < w[1].0));
    let door_row = view
        .rows
        .iter()
        .find(|r| r.id == door)
        .expect("doorway row");
    assert_eq!(door_row.state, 0, "a fresh seeded entry is practised");
    assert!(!collect_agent_knowledge(&engine.world, &engine.resources, -1).found);
}

#[test]
fn teach_orders_are_listed_until_cancelled() {
    let mut engine = sim_test::colony_engine(1);
    for _ in 0..241 {
        engine.tick();
    }
    let door = id("doorway");
    let teacher = engine.resources.colony.tech.states[usize::from(door)].names[0];
    let learner = engine
        .world
        .query::<(&Agent, Option<&Knowledge>)>()
        .without::<&Child>()
        .iter()
        .filter(|(_, (a, k))| a.id != teacher && !k.is_some_and(|k| k.practises(door)))
        .map(|(_, (a, _))| a.id)
        .min()
        .expect("a learner");
    let r = submit(
        &mut engine,
        Command::OrderTeach {
            teacher,
            learner,
            knowledge: door,
        },
    );
    assert!(r.accepted, "{r:?}");
    engine.tick();
    let orders = collect_teach_orders(&engine.world, &engine.resources);
    assert_eq!(orders.len(), 1);
    let o = &orders[0];
    assert_eq!(
        (o.teacher, o.learner, o.knowledge),
        (teacher, learner, door)
    );
    assert_eq!(o.threshold, defs().threshold(door));
    let r = submit(&mut engine, Command::CancelTeach { order: o.id });
    assert!(r.accepted, "{r:?}");
    engine.tick();
    assert!(collect_teach_orders(&engine.world, &engine.resources).is_empty());
}
