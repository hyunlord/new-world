//! Indexed target queries retain live source, claim, scope and insertion semantics.
use super::*;
use crate::runtime::agent_actions::forage::choose_bush;
use crate::runtime::mind::context::TargetScope;
use sim_core::components::{Agent, Position};
use sim_core::rules::{ForageRule, StockTargetRule};

fn fixture() -> (TickContext, SimResources) {
    let mut world = hecs::World::new();
    world.spawn((Agent { id: 7 }, Position::new(16, 16), AgentState::Idle));
    let mut resources = SimResources::new(48, 48, sim_core::MaterialRegistry::new());
    resources.rules.agent.forage = Some(ForageRule {
        start_minute: 0,
        end_minute: 1440,
        forage_radius: 16,
        haul_threshold: 5,
        max_foragers_per_bush: 2,
    });
    resources.rules.society.stock_target = Some(StockTargetRule {
        base_days: 1.0,
        hoard_cap_factor: 2.0,
    });
    for y in 0..48 {
        for x in 0..48 {
            if (x * 13 + y * 7) % 11 == 0 {
                resources.food_tiles.insert((x, y), ((x + y) % 5) as u8);
            }
        }
    }
    resources
        .food_tiles
        .insert((15, 16), sim_engine::RESOURCE_SOURCE_INFINITE);
    resources.food_tiles.insert((16, 15), 1);
    let mut context = TickContext::capture(&world, &resources).expect("context");
    context
        .set_target_scope(
            7,
            TargetScope {
                home: Some((16, 16)),
                ..TargetScope::default()
            },
        )
        .expect("scope");
    (context, resources)
}

fn assert_live_oracle(context: &mut TickContext, resources: &SimResources) {
    let person = context.row(7).expect("person");
    let scope = context.target_scope(7).expect("scope");
    let gather = targets::nearest_gather_tile(
        &person.position,
        &resources.food_tiles,
        targets::GATHER_SCAN_RADIUS,
        scope.local_food,
        &scope.avoid_hearths,
    );
    let forage = choose_bush(
        &resources.rules.agent.forage.expect("rule"),
        scope.home.expect("home"),
        (person.position.x, person.position.y),
        &resources.food_tiles,
        context.claims(),
        None,
        &scope.avoid_hearths,
    );
    assert_eq!(
        context.gather_available(7, resources).expect("available"),
        gather.is_some()
    );
    assert_eq!(
        resolve_target(context, resources, 7, "gather_food")
            .expect("gather")
            .map(|t| t.tile),
        gather
    );
    assert_eq!(
        resolve_target(context, resources, 7, "forage_bush")
            .expect("forage")
            .map(|t| t.tile),
        forage
    );
}

#[test]
fn indexed_targets_match_live_oracle_across_scopes_and_claim_changes() {
    let (mut context, resources) = fixture();
    for local_food in [None, Some(((16, 16), 6)), Some(((16, 16), 16))] {
        for avoid_hearths in [vec![], vec![(20, 16)], vec![(12, 16), (16, 20)]] {
            context
                .set_target_scope(
                    7,
                    TargetScope {
                        home: Some((16, 16)),
                        local_food,
                        avoid_hearths,
                        ..TargetScope::default()
                    },
                )
                .expect("scope");
            assert_live_oracle(&mut context, &resources);
            for tile in [(15, 16), (16, 15), (17, 16)] {
                context.adjust_claim(tile, 2).expect("claim cap");
                assert_live_oracle(&mut context, &resources);
                context.adjust_claim(tile, -1).expect("partial release");
                assert_live_oracle(&mut context, &resources);
                context.adjust_claim(tile, -1).expect("release");
            }
        }
    }
}

#[test]
fn source_invalidation_rebuilds_membership_after_arbitrary_insertions() {
    let (mut context, mut resources) = fixture();
    assert_live_oracle(&mut context, &resources);
    resources.food_tiles.clear();
    resources.food_tiles.insert((17, 16), 1);
    context
        .invalidate(Domain::FoodSources)
        .expect("replace sources");
    assert_live_oracle(&mut context, &resources);
    assert_eq!(
        context.gather_candidate(7, &resources).expect("candidate"),
        Some((17, 16))
    );
    // Same map size does not imply unchanged membership.
    resources.food_tiles.remove(&(17, 16));
    resources.food_tiles.insert((16, 16), 2);
    context
        .invalidate(Domain::FoodSources)
        .expect("insert closer source");
    assert_live_oracle(&mut context, &resources);
    assert_eq!(
        context.gather_candidate(7, &resources).expect("candidate"),
        Some((16, 16))
    );
    for amount in [1, 0, sim_engine::RESOURCE_SOURCE_INFINITE] {
        resources.food_tiles.insert((16, 16), amount);
        context
            .invalidate(Domain::FoodSources)
            .expect("amount change");
        assert_live_oracle(&mut context, &resources);
        let mut observed = Vec::new();
        context.visit_food_in_square(&resources, (16, 16), 0, |tile, value| {
            observed.push((tile, value));
        });
        let expected = if amount == 0 {
            vec![]
        } else {
            vec![((16, 16), amount)]
        };
        assert_eq!(observed, expected);
    }
    resources.food_tiles.clear();
    context.invalidate(Domain::FoodSources).expect("depletion");
    assert_live_oracle(&mut context, &resources);
}
