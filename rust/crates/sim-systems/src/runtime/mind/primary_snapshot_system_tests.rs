//! Actual authored effects compared with the original full-refresh coordinator path.
use super::*;
use sim_core::components::{
    Activity, ActivityKind, Agent, Hunger, Inventory, Position, ResourceKind, SeekTarget,
    Settlement, Social, TargetKind,
};
use std::collections::BTreeMap;

const POLICY: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
(id:"eat",band:"critical",behaviour:"eat",target:Some("food"),when:[(fact:"hunger",op:">",value:Some(80000))],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"EAT"),
(id:"activity",band:"duty",behaviour:"routine_sit_by_fire",when:[(fact:"activity_active",op:"has")],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"ACTIVITY"),
(id:"social",band:"need",behaviour:"socialise",target:Some("social_peer"),when:[(fact:"idle",op:"has"),(fact:"social_candidate",op:"has")],score:(constant:100,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"SOCIAL"),
(id:"idle",band:"idle",behaviour:"wander",when:[],score:(constant:0,terms:[(fact:"hunger",weight_permille:1)]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;

fn fixture(invited_id: u64, meals: bool) -> (ColonyMindSystem, World, SimResources) {
    let mut resources = SimResources::new(5, 5, sim_core::MaterialRegistry::new());
    resources.rules = sim_core::rules::WorldRules::colony(1);
    resources.food_tiles.insert((1, 1), 4);
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (2, 1);
    settlement.member_agents.extend([1, 2]);
    resources.settlements.insert(1, settlement);
    let mut world = World::new();
    for id in [1, 2] {
        let entity = world.spawn((
            Agent { id },
            Position::new(1, 1),
            AgentState::Idle,
            Hunger::new(if meals { 90.0 } else { 0.0 }, 0.0),
            Social::new(if meals { 0.0 } else { 90.0 }, 0.0),
            Inventory::default(),
        ));
        if meals {
            world
                .insert_one(
                    entity,
                    AgentState::Consuming {
                        target: TargetKind::Food,
                    },
                )
                .unwrap();
        } else if id == invited_id {
            world
                .insert(
                    entity,
                    (
                        AgentState::Seeking {
                            target: TargetKind::Activity,
                        },
                        Activity {
                            kind: ActivityKind::SitByFire,
                            settlement: 1,
                            place: (2, 1),
                            chosen_tick: 0,
                            arrived_tick: None,
                            until_tick: None,
                        },
                        SeekTarget { tile: (2, 1) },
                    ),
                )
                .unwrap();
        }
    }
    (
        ColonyMindSystem::new(MindProgram::load(POLICY).unwrap()),
        world,
        resources,
    )
}

fn state_bytes(world: &World, resources: &SimResources) -> Vec<u8> {
    let mut rows: Vec<_> = world
        .query::<(
            &Agent,
            &Position,
            &AgentState,
            &Hunger,
            &Social,
            &Inventory,
            Option<&Activity>,
            Option<&sim_core::components::ActivityRest>,
            Option<&SeekTarget>,
        )>()
        .iter()
        .map(
            |(_, (a, p, s, h, social, inventory, activity, rest, seek))| {
                (
                    a.id,
                    *p,
                    *s,
                    h.value.to_bits(),
                    social.loneliness.to_bits(),
                    inventory.clone(),
                    activity.copied(),
                    rest.copied(),
                    seek.copied(),
                )
            },
        )
        .collect();
    rows.sort_by_key(|r| r.0);
    let food: BTreeMap<_, _> = resources.food_tiles.iter().map(|(k, v)| (*k, *v)).collect();
    let stocks: BTreeMap<_, _> = resources
        .settlements
        .iter()
        .map(|(id, s)| (*id, s.stockpile.get(&ResourceKind::Food).copied()))
        .collect();
    bincode::serialize(&(
        rows,
        &resources.mind,
        food,
        stocks,
        resources.tallies.meals_eaten,
        resources.tallies.meals_without_food,
        &resources.social_chase_since,
        &resources.social_retry_after,
    ))
    .unwrap()
}

#[test]
fn acknowledged_primaries_preserve_actual_effects_and_full_invitation_replay() {
    for (invited_id, meals, expected) in [(2, false, 1), (1, false, 2), (0, true, 2)] {
        let (mut candidate, mut world, mut resources) = fixture(invited_id, meals);
        let (mut original, mut reference_world, mut reference_resources) =
            fixture(invited_id, meals);
        original.reference_full_refresh = true;
        candidate.try_tick(&mut world, &mut resources).unwrap();
        original
            .try_tick(&mut reference_world, &mut reference_resources)
            .unwrap();
        assert_eq!(candidate.snapshot_refresh_visits, [expected, 0]);
        assert_eq!(original.snapshot_refresh_visits, [0, 0]);
        assert_eq!(candidate.refresh_visits, original.refresh_visits);
        assert_eq!(
            state_bytes(&world, &resources),
            state_bytes(&reference_world, &reference_resources)
        );
        // A fresh tick builds a new context; previous one-shot metadata cannot survive.
        resources.current_tick += 1;
        reference_resources.current_tick += 1;
        candidate.try_tick(&mut world, &mut resources).unwrap();
        original
            .try_tick(&mut reference_world, &mut reference_resources)
            .unwrap();
        assert_eq!(
            state_bytes(&world, &resources),
            state_bytes(&reference_world, &reference_resources)
        );
    }
}

#[test]
fn malformed_secondary_alias_uses_eight_query_in_actual_coordinator() {
    let (mut candidate, mut world, mut resources) = fixture(0, true);
    let (mut original, mut reference_world, mut reference_resources) = fixture(0, true);
    original.reference_full_refresh = true;
    for world in [&mut world, &mut reference_world] {
        world.spawn((
            Agent { id: 2 },
            sim_core::components::Child { born_tick: 0 },
        ));
    }
    candidate.try_tick(&mut world, &mut resources).unwrap();
    original
        .try_tick(&mut reference_world, &mut reference_resources)
        .unwrap();
    assert_eq!(candidate.snapshot_refresh_visits, [1, 1]);
    assert_eq!(
        state_bytes(&world, &resources),
        state_bytes(&reference_world, &reference_resources)
    );
}
