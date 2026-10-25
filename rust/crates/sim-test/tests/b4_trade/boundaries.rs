//! Fresh-count saturation and saved-offer compatibility boundaries.
use super::*;

#[test]
fn t_generation_counts_future_use_saturated_days_and_missing_components() {
    let node = node(Era::Paleolithic);
    let threshold = knowledge::defs().threshold(node);
    let mut e = fixture(0, node, 750);
    e.resources
        .rules
        .colony
        .as_mut()
        .expect("colony")
        .knowledge
        .as_mut()
        .expect("knowledge rule")
        .practice_days = 0;
    holder(&mut e, node, threshold, TODAY + 1, false, true);
    holder(&mut e, node, threshold + 1, TODAY, false, true);
    holder(&mut e, node, threshold - 1, TODAY, false, true);
    holder(&mut e, node, 1000, TODAY - 1, false, true);
    holder(&mut e, node, 1000, TODAY, true, true);
    holder(&mut e, node, 1000, TODAY, false, false);
    e.spawn_agent(5, 5); // Real Agent without Knowledge is not a practitioner.
    let inputs = trade::generation_inputs(&e.world, &e.resources, knowledge::bit(node));
    assert_eq!(
        inputs.practitioners[usize::from(node)],
        2,
        "inclusive zero-window and future saturated age0"
    );
    let mut e = fixture(0, node, 750);
    e.resources
        .rules
        .colony
        .as_mut()
        .expect("colony")
        .knowledge
        .as_mut()
        .expect("knowledge rule")
        .practice_days = 0;
    e.resources.current_tick = (u64::from(u16::MAX) + 10) * 1440;
    holder(&mut e, node, threshold, u16::MAX, false, true);
    holder(&mut e, node, threshold, u16::MAX - 1, false, true);
    assert_eq!(knowledge::day_of(e.resources.current_tick), u16::MAX);
    assert_eq!(
        trade::generation_inputs(&e.world, &e.resources, 0).practitioners[usize::from(node)],
        1
    );
}

#[test]
fn t_saved_caravan_keeps_goods_and_legacy_rule_layout_values() {
    let node = node(Era::Paleolithic);
    let mut e = fixture(0, node, 750);
    holder(&mut e, node, 1000, TODAY, false, true);
    let expected = expected(&e, node, 1, 750);
    let goods = generate(&mut e);
    assert_eq!(goods, expected);
    let before = e.resources.colony.neighbours.clone();
    let rules = e.resources.rules;
    let bytes = sim_systems::save::save(&mut e).expect("save generated goods");
    let mut loaded = sim_systems::save::load(&bytes).expect("load generated goods");
    assert_eq!(loaded.resources.colony.neighbours, before);
    assert_eq!(loaded.resources.rules, rules);
    let rule = neighbours::rule(&loaded.resources).expect("saved rule");
    assert_eq!((rule.lack_weight, rule.staple_weight), (3, 1));
    assert_eq!(loaded.resources.colony.neighbours.caravans[0].goods, goods);
    assert_eq!(sim_systems::save::save(&mut loaded).expect("resave"), bytes);
}
