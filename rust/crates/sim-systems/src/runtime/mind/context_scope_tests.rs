//! Differential target-scope storage tests against the previous map implementation.
use super::*;
use sim_core::components::Settlement;

fn fixture() -> (World, SimResources) {
    let mut world = World::new();
    for id in [9, 2, 7] {
        world.spawn((Agent { id }, Position::new(1, 1), AgentState::Idle));
    }
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    for (sid, home, members) in [(8, (6, 6), vec![2, 9]), (3, (2, 2), vec![2, 7])] {
        let mut settlement = Settlement::new_with_id(sid, 0);
        settlement.formation_tile = home;
        for agent in members {
            settlement.add_member_agent(agent);
        }
        settlement.stockpile.insert(ResourceKind::Food, sid);
        resources.settlements.insert(sid, settlement);
    }
    resources.food_tiles.insert((2, 2), 4);
    (world, resources)
}

fn same_scope(a: &TargetScope, b: &TargetScope) {
    assert_eq!(a.settlement, b.settlement);
    assert_eq!(a.home, b.home);
    assert_eq!(a.local_food, b.local_food);
    assert_eq!(a.avoid_hearths, b.avoid_hearths);
    assert_eq!(a.food_available, b.food_available);
    assert_eq!(a.camp_stocked, b.camp_stocked);
    assert_eq!(a.stock_shortfall, b.stock_shortfall);
    assert_eq!(a.stock_in_hand, b.stock_in_hand);
    assert_eq!(
        a.stock_hoard_limit.map(f64::to_bits),
        b.stock_hoard_limit.map(f64::to_bits)
    );
}

fn old_get(
    scopes: &BTreeMap<AgentId, TargetScope>,
    agent: AgentId,
) -> Result<&TargetScope, ContextError> {
    scopes
        .get(&agent)
        .ok_or_else(|| ContextError::MissingDerived {
            agent,
            name: "target_scope".into(),
        })
}

#[test]
fn scope_column_matches_old_capture_with_overlapping_membership() {
    let (world, resources) = fixture();
    let mut context = TickContext::capture(&world, &resources).unwrap();
    let mut old = old_map_capture(&context, &world, &resources);
    assert_eq!(context.target_scope(2).unwrap().settlement, Some(3));
    for active in [9, 2, 7] {
        context.begin_visit(active).unwrap();
        for agent in [2, 7, 9] {
            same_scope(
                context.target_scope(agent).unwrap(),
                old_get(&old, agent).unwrap(),
            );
        }
        assert_eq!(
            context.target_scope(99).unwrap_err(),
            old_get(&old, 99).unwrap_err()
        );
    }
    let index = context.row_index(7).unwrap();
    context.scopes[index] = None;
    old.remove(&7);
    assert_eq!(
        context.target_scope(7).unwrap_err(),
        old_get(&old, 7).unwrap_err()
    );
    assert_eq!(
        context.target_scope(99).unwrap_err(),
        old_get(&old, 99).unwrap_err()
    );
}

#[test]
fn scope_column_preserves_set_errors_stamps_and_row_independence() {
    let (world, resources) = fixture();
    let mut context = TickContext::capture(&world, &resources).unwrap();
    let mut oracle = TickContext::capture(&world, &resources).unwrap();
    let mut old = old_map_capture(&oracle, &world, &resources);
    for c in [&mut context, &mut oracle] {
        c.set_derived(2, "scope_stamp", 17, &[Domain::Person])
            .unwrap();
    }
    let replacement = TargetScope {
        home: Some((7, 7)),
        avoid_hearths: vec![(5, 5)],
        stock_hoard_limit: Some(-0.0),
        ..TargetScope::default()
    };
    for agent in [99, 2, 7, 2] {
        let expected = oracle
            .row(agent)
            .map(|_| ())
            .and_then(|()| oracle.invalidate(Domain::Person))
            .map(|()| {
                old.insert(agent, replacement.clone());
            });
        assert_eq!(
            context.set_target_scope(agent, replacement.clone()),
            expected
        );
        assert_eq!(context.versions, oracle.versions);
        assert_eq!(
            context.derived(2, "scope_stamp"),
            oracle.derived(2, "scope_stamp")
        );
        for id in [2, 7, 9] {
            same_scope(
                context.target_scope(id).unwrap(),
                old_get(&old, id).unwrap(),
            );
        }
    }
    context.versions[Domain::Person as usize] = u64::MAX;
    oracle.versions[Domain::Person as usize] = u64::MAX;
    let before = context.target_scope(9).unwrap().clone();
    assert_eq!(
        context.set_target_scope(9, replacement),
        oracle.invalidate(Domain::Person)
    );
    same_scope(context.target_scope(9).unwrap(), &before);
    assert_eq!(
        context.set_target_scope(99, TargetScope::default()),
        Err(ContextError::UnknownAgent(99))
    );
    assert_eq!(context.versions, oracle.versions);
}

fn old_map_capture(
    context: &TickContext,
    world: &World,
    resources: &SimResources,
) -> BTreeMap<AgentId, TargetScope> {
    let mut result = BTreeMap::new();
    let mut settlements: Vec<_> = resources.settlements.iter().collect();
    settlements.sort_unstable_by_key(|(id, _)| **id);
    let radius = crate::runtime::settlement::food_economy::catchment_radius(resources);
    let walking: BTreeSet<_> = world
        .query::<(&Agent, &Founding)>()
        .iter()
        .map(|(_, (a, _))| a.id)
        .chain(
            world
                .query::<(&Agent, &PorterTrip)>()
                .iter()
                .filter(|_| resources.rules.society.relations.is_some())
                .map(|(_, (a, _))| a.id),
        )
        .collect();
    let stock: BTreeMap<_, _> = settlements
        .iter()
        .map(|(sid, _)| {
            let in_hand =
                crate::runtime::settlement::food_economy::stock_in_hand(world, resources, **sid);
            let target =
                crate::runtime::settlement::food_economy::stock_target(world, resources, **sid)
                    .unwrap_or(0.0);
            let cap = resources
                .rules
                .agent
                .forage
                .and(resources.rules.society.stock_target)
                .map(|rule| rule.hoard_cap_factor * target);
            (**sid, ((in_hand as f64) < target, in_hand, cap))
        })
        .collect();
    let local_food: BTreeMap<_, _> = settlements
        .iter()
        .map(|(sid, settlement)| {
            let home = settlement.formation_tile;
            (
                **sid,
                resources.food_tiles.iter().any(|(tile, amount)| {
                    *amount > 0 && tile.0.abs_diff(home.0).max(tile.1.abs_diff(home.1)) <= radius
                }),
            )
        })
        .collect();
    for row in &context.rows {
        let member = settlements
            .iter()
            .find(|(_, settlement)| settlement.member_agents.contains(&row.agent));
        let mut scope = TargetScope {
            food_available: context.availability.food,
            ..TargetScope::default()
        };
        if let Some((sid, settlement)) = member {
            scope.settlement = Some(**sid);
            scope.home = Some(settlement.formation_tile);
            scope.camp_stocked = settlement
                .stockpile
                .get(&ResourceKind::Food)
                .copied()
                .unwrap_or(0)
                >= 1;
            if let Some((shortfall, in_hand, limit)) = stock.get(*sid) {
                scope.stock_shortfall = *shortfall;
                scope.stock_in_hand = *in_hand;
                scope.stock_hoard_limit = *limit;
            }
            if resources.rules.agent.local_food_availability && !walking.contains(&row.agent) {
                scope.local_food = Some((settlement.formation_tile, radius));
                scope.food_available = local_food.get(*sid).copied().unwrap_or(false);
            }
            if resources.rules.society.relations.is_some() {
                for ((a, b), state) in &resources.settlement_relations.pairs {
                    if !state.estranged {
                        continue;
                    }
                    let other = if a == *sid {
                        Some(b)
                    } else if b == *sid {
                        Some(a)
                    } else {
                        None
                    };
                    if let Some(other) = other.and_then(|id| resources.settlements.get(id)) {
                        scope.avoid_hearths.push(other.formation_tile);
                    }
                }
                scope.avoid_hearths.sort_unstable();
            }
        }
        result.insert(row.agent, scope);
    }
    result
}

#[test]
fn initial_forage_stock_matches_uncached_scope_oracle() {
    use crate::runtime::mind::{behaviours::EffectState, facts};
    use sim_core::rules::{ForageRule, StockTargetRule};

    for forage_enabled in [false, true] {
        for target_enabled in [false, true] {
            let (mut world, mut resources) = fixture();
            resources.rules.agent.forage = forage_enabled.then_some(ForageRule {
                start_minute: 0,
                end_minute: 1440,
                forage_radius: 16,
                haul_threshold: 5,
                max_foragers_per_bush: 2,
            });
            resources.rules.society.stock_target = target_enabled.then_some(StockTargetRule {
                base_days: 2.75,
                hoard_cap_factor: 1.375,
            });
            let mut inventory = Inventory::default();
            inventory.add(ResourceKind::Food, 3);
            // This member contributes stock and hunger but has no captured person row.
            world.spawn((Agent { id: 42 }, inventory, Hunger::new(13.0, 0.037)));
            resources
                .settlements
                .get_mut(&3)
                .unwrap()
                .add_member_agent(42);
            let mut effects = EffectState::prepare(&world, &mut resources);
            assert_eq!(
                effects.forage_plans.len(),
                if forage_enabled && target_enabled {
                    2
                } else {
                    0
                }
            );
            if !forage_enabled || !target_enabled {
                // Disabled rules must ignore even a supplied entry.
                effects.forage_plans.insert(
                    3,
                    ForagePlan {
                        home: (0, 0),
                        target: f64::NAN,
                        in_hand: u64::MAX,
                    },
                );
            }
            let oracle_context = TickContext::capture_continuation(&world, &resources).unwrap();
            let oracle = old_map_capture(&oracle_context, &world, &resources);
            assert_eq!(oracle[&2].stock_in_hand, 6);
            assert_eq!(oracle[&2].settlement, Some(3));
            for missing in [None, Some(3), Some(8)] {
                if let Some(sid) = missing {
                    effects.forage_plans.remove(&sid);
                }
                let captured = facts::prepare_continuation_context_with_claims(
                    &world,
                    &resources,
                    effects.claims.clone(),
                    Some(&effects.forage_plans),
                )
                .unwrap();
                let uncached = facts::prepare_continuation_context_with_claims(
                    &world,
                    &resources,
                    effects.claims.clone(),
                    None,
                )
                .unwrap();
                assert_eq!(captured.versions, uncached.versions);
                assert_eq!(captured.claims, uncached.claims);
                for row in captured.rows() {
                    same_scope(
                        captured.target_scope(row.agent).unwrap(),
                        &oracle[&row.agent],
                    );
                    same_scope(
                        captured.target_scope(row.agent).unwrap(),
                        uncached.target_scope(row.agent).unwrap(),
                    );
                }
            }
        }
    }
}

#[test]
fn initial_stock_capture_retains_duplicate_error_before_scope_work() {
    let (mut world, resources) = fixture();
    world.spawn((Agent { id: 2 }, Position::new(0, 0), AgentState::Idle));
    let plans = BTreeMap::new();
    let cached = TickContext::capture_continuation_with_claims(
        &world,
        &resources,
        BTreeMap::new(),
        Some(&plans),
    )
    .err();
    let uncached = TickContext::capture_continuation(&world, &resources).err();
    assert_eq!(cached, uncached);
    assert_eq!(cached, Some(ContextError::DuplicateAgent(2)));
}
