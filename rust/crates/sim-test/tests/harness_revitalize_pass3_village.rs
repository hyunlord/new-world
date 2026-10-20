use sim_core::components::{
    building_kind::shelter_blueprint, AgentState, BuildingKind, ConstructionSite, Position,
    Settlement, TargetKind,
};
use sim_core::material::MaterialRegistry;
use sim_core::rules::{HousingCount, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::construction::ConstructionSystem;
use sim_systems::runtime::settlement::{housing_capacity, SettlementSystem, ShelterPlanningSystem};
use sim_test::baseline_engine;

/// The live game's village rules (`WorldRules::baseline`) copied into
/// `WorldRules::default()` (R1, exact: `shelter_planning` and the hearth
/// anchor are unchanged since v9).
fn village_rules() -> WorldRules {
    let live = WorldRules::baseline(7);
    let mut rules = WorldRules::default();
    rules.society.shelter_planning = live.society.shelter_planning;
    rules.society.settlement_anchor = live.society.settlement_anchor;
    rules
}

#[test]
fn harness_village_one_settlement_per_hearth() {
    // Baseline scene (R3). Two days stay well before the first possible
    // fission (`min_age_days` 10), so one hearth means one village owning
    // every building (8 seeds × 2 days: under 1 s in release).
    for seed in 0..8 {
        let mut engine = baseline_engine(seed);
        let mut formed = false;
        for _ in 0..2880 {
            engine.tick();
            if !formed && !engine.resources.settlements.is_empty() {
                assert!(
                    engine.resources.building_owner.len() >= 5,
                    "ownership at formation"
                );
            }
            formed |= !engine.resources.settlements.is_empty();
            if formed && engine.current_tick().is_multiple_of(60) {
                assert_eq!(
                    engine.resources.settlements.len(),
                    1,
                    "seed {seed} tick {}",
                    engine.current_tick()
                );
                let sid = *engine.resources.settlements.keys().next().expect("village");
                assert!(engine
                    .resources
                    .building_owner
                    .values()
                    .all(|owner| *owner == sid));
                assert!(engine.resources.building_owner.len() >= 5);
            }
        }
        assert!(formed, "seed {seed}");
    }
}

#[test]
fn harness_village_owned_capacity_counts_each_shelter_once() {
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    for sid in [1, 2] {
        let mut settlement = Settlement::new_with_id(sid, 0);
        settlement.formation_tile = (10, 10);
        engine.resources.settlements.insert(sid, settlement);
    }
    for x in 10..14 {
        engine
            .resources
            .building_kinds
            .insert((x, 10), BuildingKind::Shelter);
        engine.resources.building_owner.insert((x, 10), 1);
    }
    let rule = village_rules().society.shelter_planning.expect("planning");
    // Baseline counts owned housing; the v8 `NearFormation` arm is gone.
    assert_eq!(rule.housing, HousingCount::Owned);
    assert_eq!(housing_capacity(&engine.resources, 1, &rule), 32);
    assert_eq!(housing_capacity(&engine.resources, 2, &rule), 0);
}

#[test]
fn harness_village_plans_with_headroom() {
    // One owned shelter (8 places) and baseline headroom 2: six residents
    // (6 + 2 ≥ 8) already plan a second shelter, five (5 + 2 < 8) do not.
    for (members_below_capacity, expected_sites) in [(2, 1), (3, 0)] {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        engine.resources.rules = village_rules();
        let rule = engine
            .resources
            .rules
            .society
            .shelter_planning
            .expect("planning");
        assert_eq!(rule.planning_headroom, 2);
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = (10, 10);
        for id in 0..rule.residents_per_shelter - members_below_capacity {
            settlement.add_member_agent(u64::from(id));
        }
        engine.resources.settlements.insert(1, settlement);
        engine
            .resources
            .building_kinds
            .insert((10, 10), BuildingKind::Shelter);
        engine.resources.building_owner.insert((10, 10), 1);
        let mut planner = ShelterPlanningSystem::default();
        for tick in 1..=rule.plan_interval_ticks {
            engine.resources.current_tick = tick;
            planner.tick(&mut engine.world, &mut engine.resources);
        }
        assert_eq!(
            engine.world.query::<&ConstructionSite>().iter().count(),
            expected_sites,
            "{} residents",
            rule.residents_per_shelter - members_below_capacity
        );
    }
}

#[test]
fn harness_village_completed_site_is_owned() {
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = village_rules();
    let rule = engine
        .resources
        .rules
        .society
        .shelter_planning
        .expect("planning");
    engine
        .resources
        .settlements
        .insert(1, Settlement::new_with_id(1, 0));
    let tile = (10, 10);
    engine.world.spawn((ConstructionSite::new(
        shelter_blueprint(1),
        Position::new(tile.0, tile.1),
    ),));
    engine.world.spawn((
        Position::new(tile.0, tile.1),
        AgentState::Consuming {
            target: TargetKind::ConstructionSite,
        },
    ));
    engine.resources.site_owner.insert(tile, 1);
    let before = housing_capacity(&engine.resources, 1, &rule);
    ConstructionSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert_eq!(engine.resources.building_owner.get(&tile), Some(&1));
    assert_eq!(
        housing_capacity(&engine.resources, 1, &rule),
        before + rule.residents_per_shelter
    );
    assert!(!engine.resources.site_owner.contains_key(&tile));
}

#[test]
fn harness_village_dissolution_releases_only_its_owned_buildings() {
    let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
    engine.resources.rules = village_rules();
    engine
        .resources
        .settlements
        .insert(1, Settlement::new_with_id(1, 0));
    engine.resources.building_owner.insert((10, 10), 1);
    engine.resources.building_owner.insert((20, 20), 2);
    SettlementSystem::new().tick(&mut engine.world, &mut engine.resources);
    assert!(!engine.resources.settlements.contains_key(&1));
    assert!(!engine.resources.building_owner.contains_key(&(10, 10)));
    assert_eq!(engine.resources.building_owner.get(&(20, 20)), Some(&2));
}
