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
    resources.nav.install(sim_core::nav::CostMap::new(5, 5));
    crate::runtime::food::items::create_ground(
        &mut resources,
        (1, 1),
        sim_core::colony::ItemSpec::raw(
            sim_core::colony::ItemKind::from_name("berries").expect("food"),
            None,
        ),
        4,
        0,
        crate::runtime::food::items::FoodSource::Harvest,
    )
    .expect("physical food fixture");
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
        &resources.food,
        &resources.colony.ground_items,
        &resources.colony.item_ledger,
    ))
    .unwrap()
}

#[test]
fn authored_multi_person_effects_match_full_refresh_in_primary_invitation_and_replay() {
    for (invited_id, meals, expected_visits) in [
        (2, false, [1, 1, 0]),
        (1, false, [2, 0, 1]),
        (0, true, [2, 0, 0]),
    ] {
        let (mut candidate, mut world, mut resources) = fixture(invited_id, meals);
        let (mut original, mut reference_world, mut reference_resources) =
            fixture(invited_id, meals);
        original.reference_full_refresh = true;
        candidate.try_tick(&mut world, &mut resources).unwrap();
        original
            .try_tick(&mut reference_world, &mut reference_resources)
            .unwrap();
        assert_eq!(
            candidate.refresh_visits, expected_visits,
            "invited={invited_id} meals={meals}"
        );
        assert_eq!(original.refresh_visits, expected_visits);
        assert_eq!(
            state_bytes(&world, &resources),
            state_bytes(&reference_world, &reference_resources)
        );
        if meals {
            assert_eq!(resources.tallies.meals_eaten, 2);
            assert_eq!(resources.food.tallies.meals, 2);
            assert_eq!(resources.colony.ground_items[&(1, 1)].count, 2);
            crate::runtime::food::validate(&world, &resources).expect("conserved meals");
        } else {
            assert_eq!(world.query::<&Activity>().iter().count(), 0);
            assert_eq!(resources.mind.running.len(), 2);
        }
    }
}
