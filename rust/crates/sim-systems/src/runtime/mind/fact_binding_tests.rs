//! Fact bindings preserve current row contents, routine overlays, and witness order.
use super::*;
use sim_core::colony::mind::{choose, choose_with_reader, EvalKey, EvalScratch, FactReader};
use sim_core::components::{Activity, Agent, Hunger, Position};

#[test]
fn bound_row_reads_current_person_and_matches_eager_witnesses() {
    let program = crate::runtime::mind::MindProgram::load(r#"(
        schema_version: 1, bands: ["critical", "duty", "need", "work", "leisure", "idle"],
        nodes: [
            (id: "need", band: "need", behaviour: "rest", when: [(fact: "idle", op: "has")],
             score: (constant: 1, terms: [(fact: "hunger", weight_permille: 1000), (fact: "activity_active", weight_permille: 1000)]),
             hysteresis: 0, life_stages: ["adult", "child", "elder"], tags: [], name_key: "NEED"),
            (id: "idle", band: "idle", behaviour: "wander", when: [(fact: "idle", op: "has")],
             score: (constant: 0, terms: []), hysteresis: 0,
             life_stages: ["adult", "child", "elder"], tags: [], name_key: "IDLE")
        ])"#).expect("policy");
    let mut world = hecs::World::new();
    for (id, hunger) in [(90, 90.0), (7, 60.0), (2, 20.0)] {
        world.spawn((
            Agent { id },
            Position::new(1, 1),
            AgentState::Idle,
            Hunger::new(hunger, 0.0),
        ));
    }
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let mut context = prepare_context(&world, &resources).expect("context");
    let mut memo = Vec::new();
    assert!(matches!(
        program
            .providers
            .lazy_row(&mut context, &resources, None, 999, &mut memo),
        Err(FactError::Context(ContextError::UnknownAgent(999)))
    ));
    let mut row = program
        .providers
        .lazy_row(&mut context, &resources, None, 7, &mut memo)
        .expect("row");
    assert_eq!(
        row.get(program.policy.fact_id("hunger").expect("hunger"))
            .expect("value"),
        60000
    );
    row.context
        .set_activity_snapshot(
            7,
            Some(Activity {
                kind: ActivityKind::Play,
                settlement: 1,
                place: (1, 1),
                chosen_tick: 0,
                arrived_tick: None,
                until_tick: None,
            }),
        )
        .expect("current activity");
    assert_eq!(
        row.get(program.policy.fact_id("activity_active").expect("activity"))
            .expect("updated value"),
        1
    );
    let table = program
        .providers
        .build(&program.policy, row.context, &resources)
        .expect("eager");
    let key = EvalKey {
        world_seed: 0,
        agent: 7,
        tick: resources.current_tick,
        replicate: 0,
    };
    let mut eager = EvalScratch::default();
    let mut lazy = EvalScratch::default();
    let expected = choose(
        &program.policy,
        table.row(7).expect("eager row"),
        None,
        key,
        &mut eager,
    )
    .expect("eager choice");
    let actual =
        choose_with_reader(&program.policy, &mut row, None, key, &mut lazy).expect("lazy choice");
    assert_eq!(actual, expected);
    assert_eq!(lazy.witnesses(), eager.witnesses());
    assert!(matches!(
        row.context.row_at(row.person_index, 90),
        Err(ContextError::UnknownAgent(90))
    ));
}

#[test]
fn routine_binding_writes_exact_fact_target_pairs_and_keeps_unknown_noop() {
    let mut world = hecs::World::new();
    world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
    let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    let expected = [
        ("routine_tend_fire", ActivityKind::TendFire),
        ("routine_craft", ActivityKind::Craft),
        ("routine_repair", ActivityKind::Repair),
        ("routine_hearth_circle", ActivityKind::HearthCircle),
        ("routine_explore", ActivityKind::Explore),
        ("routine_play", ActivityKind::Play),
        ("routine_sit_by_fire", ActivityKind::SitByFire),
    ];
    for ((target, kind), binding) in expected.iter().zip(&ROUTINE_BINDINGS) {
        assert_eq!(binding.target, *target);
        assert_eq!(binding.kind, *kind);
        assert_eq!(binding.fact, format!("{target}_available"));
        let mut context = prepare_context(&world, &resources).expect("context");
        prepare_bound_routine_fact(
            &mut context,
            None,
            7,
            binding.fact,
            RoutinePreparation::bind(binding.fact),
        )
        .expect("bound");
        assert_eq!(context.derived(7, binding.fact), Ok(0));
        assert_eq!(context.target(7, binding.target), Ok(None));
        assert!(matches!(
            context.derived(7, "routine_available"),
            Err(ContextError::MissingDerived { .. })
        ));
    }
    let mut context = prepare_context(&world, &resources).expect("context");
    prepare_routine_fact(&mut context, None, 7, "routine_available").expect("all kinds");
    for binding in &ROUTINE_BINDINGS {
        assert_eq!(context.derived(7, binding.fact), Ok(0));
        assert_eq!(context.target(7, binding.target), Ok(None));
    }
    assert_eq!(context.derived(7, "routine_available"), Ok(0));
    prepare_routine_fact(&mut context, None, 7, "routine_weighted_place_available")
        .expect("weighted");
    assert_eq!(
        context.derived(7, "routine_weighted_place_available"),
        Ok(0)
    );
    prepare_routine_fact(&mut context, None, 7, "routine_custom_available")
        .expect("unknown preserves no-op");
    assert!(matches!(
        context.derived(7, "routine_custom_available"),
        Err(ContextError::MissingDerived { .. })
    ));
    assert!(matches!(
        prepare_routine_fact(&mut context, None, 999, "routine_custom_available"),
        Err(FactError::Context(ContextError::UnknownAgent(999)))
    ));
}
