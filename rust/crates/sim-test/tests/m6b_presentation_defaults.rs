//! Stage 1 policy changes retain historical defaults and inert presentation machinery.
use sim_core::colony::steward::{parse_rules, rules, Action, StewardScope};
use sim_core::colony::Command;
use sim_systems::commands::submit;

#[test]
fn stage1_policy_changes_are_explicit_against_published_base() {
    let _engine = sim_test::colony_engine(0);
    let base =
        parse_rules(include_str!("fixtures/m6b_base_steward.ron")).expect("published BASE policy");
    let current = rules().expect("bundled policy");
    assert_eq!(
        base.rules.iter().map(|r| r.id).collect::<Vec<_>>(),
        (1..=8).collect::<Vec<_>>()
    );
    assert_eq!(
        current.rules.iter().map(|r| r.id).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5, 7, 9, 10, 8]
    );
    for id in [1, 3, 5, 7] {
        assert_eq!(
            current.rules.iter().find(|r| r.id == id),
            base.rules.iter().find(|r| r.id == id)
        );
    }
    assert!(base
        .rules
        .iter()
        .all(|r| !r.extended && r.grace_ticks.is_none()));
    assert!(current
        .rules
        .iter()
        .all(|r| !r.extended && r.grace_ticks.is_none()));
    assert!(matches!(
        base.rules[1].action,
        Action::Wood {
            cap_log_margin: None,
            ..
        }
    ));
    assert!(matches!(base.rules[5].action, Action::Teach));
    assert!(current
        .rules
        .iter()
        .all(|r| !matches!(r.action, Action::Teach)));
    assert!(
        matches!(&current.rules[1].action, Action::Wood { items, cap_log_margin: Some(20), .. } if items == &[sim_core::colony::ItemKind::Log])
    );
    assert!(
        matches!(&current.rules[3].action, Action::Bills { recipes, days: 0, fuel: None, .. }
        if recipes == &[(sim_core::colony::RecipeKind::HaftAxe, 2), (sim_core::colony::RecipeKind::KnapBlade, 4)])
    );
    assert!(
        matches!(&current.rules[6].action, Action::Bills { recipes, fuel: Some(_), .. } if recipes.is_empty())
    );
    assert!(matches!(
        current.rules[7].action,
        Action::House {
            completion_role: Some(sim_core::rooms::RoomRole::Storehouse),
            ..
        }
    ));
    let Action::FoodPlan(mut food) = current.rules[8].action.clone() else {
        panic!("food planning remains rule 8");
    };
    assert!(food.cultivation.take().is_some());
    assert_eq!(Action::FoodPlan(food), base.rules[7].action);
}

#[test]
fn default_steward_does_not_accumulate_experimental_grace_state() {
    // Given an ordinary enabled colony with the Stage 1 policy.
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
    assert!(rules_fired
        .iter()
        .all(|rule| [1, 2, 3, 4, 5, 7, 8, 9, 10].contains(rule)));
}
