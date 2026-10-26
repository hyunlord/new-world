//! Actual cross-settlement forage consumption and its node-keyed quarrel draw.
use super::*;
use crate::runtime::agent_actions::needs::NeedAvailability;
use hecs::World;
use sim_core::components::{
    Agent, AgentState, Inventory, Lineage, Position, RelationState, ResourceKind, Settlement,
    TargetKind, Temperament,
};
use sim_core::rng::{keyed_below, STREAM_RELATIONS};
use sim_core::rules::WorldRules;
use sim_engine::SimResources;

fn run_dispute(seed: u64, node_key: u64) -> bool {
    let mut world = World::new();
    let entities: Vec<_> = [11, 22]
        .into_iter()
        .map(|id| {
            world.spawn((
                Agent { id },
                Position::new(2, 2),
                AgentState::Consuming {
                    target: TargetKind::GatherFood,
                },
                Inventory::default(),
                Lineage {
                    key: 1000 + id,
                    births: 0,
                },
                Temperament::from_axes([500; 4]),
            ))
        })
        .collect();
    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    resources.rules = WorldRules::baseline(seed);
    resources.world_seed = seed;
    resources.current_tick = 600;
    let rule = resources
        .rules
        .society
        .relations
        .as_mut()
        .expect("baseline relations");
    rule.dispute_cap_per_pair_day = 0;
    rule.quarrel_cap_per_pair_day = 0;
    let rule = *rule;
    for (sid, agent) in [(1, 11), (2, 22)] {
        let mut settlement = Settlement::new_with_id(sid, 0);
        settlement.formation_tile = (sid, 2);
        settlement.member_agents.insert(agent);
        resources.settlements.insert(sid, settlement);
    }
    resources.settlement_relations.pairs.insert(
        (1, 2),
        RelationState {
            value: 300,
            estranged: false,
            base: 300,
        },
    );
    resources.food_tiles.insert((2, 2), 30);
    let mut effects = EffectState::prepare(&world, &mut resources);
    assert_eq!(
        effects
            .forage_spots
            .get(&(2, 2))
            .expect("actual rival snapshot")
            .len(),
        2
    );
    let expected = keyed_below(
        seed,
        STREAM_RELATIONS ^ node_key,
        11,
        600,
        0,
        resources.noise_replicate_for(600),
        1000,
    ) < 250;
    for (entity, agent, member) in [(entities[0], 11, 1), (entities[1], 22, 2)] {
        let mut context = EffectContext {
            world: &mut world,
            resources: &mut resources,
            effects: &mut effects,
            routine: None,
            entity,
            agent,
            position: Position::new(2, 2),
            target: ActionTarget::Tile((2, 2)),
            availability: NeedAvailability {
                food: true,
                water: false,
                sleep: false,
            },
            night_rest: None,
            interrupted: false,
            social_handshake: false,
            cause: None,
            lineage_key: Some(1000 + agent),
            member: Some(member),
            node_random_key: node_key,
            finish_reason: FinishReason::Interrupted,
            authored_completion: false,
        };
        assert_eq!(
            gather::continue_running(&mut context),
            Ok(EffectStatus::Finished)
        );
    }
    let actual = resources.tallies.quarrels == 1;
    assert_eq!(
        actual, expected,
        "seed={seed}, node={node_key}: actual registered consume must use agent/tick/node"
    );
    assert_eq!(resources.tallies.forage_disputes, 1);
    assert_eq!(effects.quarrel_entries.len(), usize::from(expected));
    assert_eq!(
        resources.settlement_relations.pairs[&(1, 2)].value,
        300 - rule.dispute_loss - if expected { rule.quarrel_loss } else { 0 }
    );
    for entity in entities {
        assert_eq!(
            *world.get::<&AgentState>(entity).expect("state"),
            AgentState::Idle
        );
        let food = world
            .get::<&Inventory>(entity)
            .expect("inventory")
            .get(ResourceKind::Food);
        assert_eq!(
            food > 0,
            !expected,
            "quarrel suppresses both foragers' actual pickups"
        );
    }
    assert_eq!(resources.food_tiles[&(2, 2)] == 30, expected);
    actual
}

#[test]
fn cross_settlement_forage_quarrel_uses_selected_node_and_agent_identity() {
    let mut changed_by_node = false;
    let mut saw_quarrel = false;
    let mut saw_pickup = false;
    for seed in 0..32 {
        let first = run_dispute(seed, 7);
        assert_eq!(first, run_dispute(seed, 7), "same input repeats exactly");
        let second = run_dispute(seed, 91);
        changed_by_node |= first != second;
        saw_quarrel |= first || second;
        saw_pickup |= !first || !second;
    }
    assert!(changed_by_node && saw_quarrel && saw_pickup);
}
