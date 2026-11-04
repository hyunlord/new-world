//! D-minus1: presentation release preserves the published steward policy.
use sim_core::colony::steward::{parse_rules, rules, StewardScope};
use sim_core::colony::Command;
use sim_systems::commands::submit;

#[test]
fn default_steward_policy_matches_published_base() {
    // Given the exact policy from fbc03bddb, decoded with retained optional fields.
    let _engine = sim_test::colony_engine(0);
    let base =
        parse_rules(include_str!("fixtures/m6b_base_steward.ron")).expect("published BASE policy");
    // When the bundled default policy is resolved.
    let current = rules().expect("bundled policy");
    // Then no experimental rule, cap, grace, priority or parameter is enabled.
    assert_eq!(current, &base);
}

#[test]
fn default_steward_does_not_accumulate_experimental_grace_state() {
    // Given an ordinary enabled colony with the presentation defaults.
    let mut engine = sim_test::colony_engine(0);
    assert!(
        submit(
            &mut engine,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All
            },
        )
        .accepted
    );
    // When the first full day executes without player orders.
    for _ in 0..1440 {
        engine.tick();
    }
    // Then retained optional machinery remains inert, while normal rules act.
    assert!(engine.resources.steward.need_since.is_empty());
    assert!(!engine.resources.steward.extended);
    let rules_fired: Vec<_> = engine
        .resources
        .colony
        .command_log
        .iter()
        .filter_map(|record| match record.command {
            Command::Steward { rule, .. } => Some(rule),
            _ => None,
        })
        .collect();
    assert!(!rules_fired.is_empty());
    assert!(rules_fired.iter().all(|rule| (1..=8).contains(rule)));
}
