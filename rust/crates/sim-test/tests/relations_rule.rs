//! Pass 20: unit and integration tests of the relations rule —
//! measures, reach, kin aid, exchange (both or none), keep lines, porter
//! choice, the trip (delivery, turn-back, loss), the deposit exemption,
//! drift / clamp / hysteresis, forage disputes and quarrels, the avoid
//! filter, the chronicle kinds and the integration case of brief 6.9.
//!
//! The fixtures are two bare villages on a 96 × 64 map under
//! `WorldRules::baseline(SEED)`: the camp (settlement 1) at `CAMP` and its
//! daughter (settlement 2, parent 1) at `DAUGHTER`, 26 tiles east.
//!
//! V8-0: rebased from `game_v25` (`V25_RELATIONS`) on the baseline, whose
//! relations rule is Pass 21's (`V26_RELATIONS`): aid giver 2,500 and keep
//! 2,000 permille of a day, drift 30 permille, dispute loss 5, quarrel loss
//! 40, the avoid filter at every food target, per-pair daily caps on what
//! disputes (3) and quarrels (1) cost, and exchange tried before aid. The
//! expected numbers below are re-derived from it (`RULE`).
use hecs::Entity;
use sim_core::causal::event::DeathReason;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::components::{
    Activity, ActivityKind, Agent, AgentId, AgentState, Child, Elder, Founding, Hunger, Inventory,
    Lineage, PorterKind, PorterStage, PorterTrip, Position, RelationState, ResourceKind,
    SeekTarget, Settlement, SettlementRelations, TargetKind, Temperament,
};
use sim_core::material::MaterialRegistry;
use sim_core::rng::{keyed_below, STREAM_RELATIONS};
use sim_core::rules::{RelationsRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::{AgentMovementSystem, MovementRng};
use sim_systems::runtime::decision::agent_decision::AgentDecisionSystem;
use sim_systems::runtime::settlement::relations::{
    base_relation, burn_night, food_days_permille, food_keep, in_reach, need_milli, wood_keep,
    wood_nights, RelationsSystem,
};
use sim_systems::runtime::settlement::StockpileDepositSystem;
use sim_systems::runtime::survival::despawn_agent;

const SEED: u64 = 20;
const CAMP: (u32, u32) = (20, 30);
const DAUGHTER: (u32, u32) = (46, 30);
/// The baseline relations rule (`V26_RELATIONS` at `v7-final-art`).
const RULE: RelationsRule = match WorldRules::baseline(SEED).society.relations {
    Some(rule) => rule,
    None => panic!("the baseline has a relations rule"),
};

/// The tick of display day `day`, minute `minute` (clock offset 480).
fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// The decision tick of day `day`.
fn decision(day: u64) -> u64 {
    tick_at(day, 480)
}

/// One resident: id, village, RD.
type Resident = (AgentId, u32, u16);

fn spawn(engine: &mut SimEngine, id: AgentId, at: (u32, u32), rd: u16) -> Entity {
    engine.world.spawn((
        Position::new(at.0, at.1),
        Agent { id },
        AgentState::Idle,
        Hunger::new(10.0, 0.05),
        Inventory::default(),
        Temperament {
            ns: 500,
            ha: 500,
            rd,
            p: 500,
        },
        Lineage {
            key: id.wrapping_mul(0x9E37_79B9_7F4A_7C15),
            births: 0,
        },
        MovementRng::new(id),
    ))
}

/// Two villages with the given stores (food, wood) and residents.
fn two_villages(stores: [(u32, u32); 2], residents: &[Resident]) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(96, 64, MaterialRegistry::new());
    engine.resources.rules = WorldRules::baseline(SEED);
    engine.resources.world_seed = SEED;
    let mut villages = Vec::new();
    for (index, (tile, parent)) in [(CAMP, None), (DAUGHTER, Some(1))].into_iter().enumerate() {
        let mut settlement = Settlement::new_with_id(index as u32 + 1, 0);
        settlement.formation_tile = tile;
        settlement.parent = parent;
        settlement.store(ResourceKind::Food, stores[index].0);
        settlement.store(ResourceKind::Wood, stores[index].1);
        villages.push(settlement);
    }
    let mut entities = Vec::new();
    for &(id, village, rd) in residents {
        let home = if village == 1 { CAMP } else { DAUGHTER };
        entities.push(spawn(&mut engine, id, home, rd));
        villages[village as usize - 1].add_member_agent(id);
    }
    for settlement in villages {
        engine
            .resources
            .settlements
            .insert(settlement.settlement_id, settlement);
    }
    (engine, entities)
}

/// Four adults per village (RD 500 except where given), ids 1–4 and 11–14.
fn residents() -> Vec<Resident> {
    vec![
        (1, 1, 400),
        (2, 1, 900),
        (3, 1, 700),
        (4, 1, 700),
        (11, 2, 500),
        (12, 2, 500),
        (13, 2, 500),
        (14, 2, 500),
    ]
}

fn relations_at(engine: &mut SimEngine, tick: u64) {
    engine.resources.current_tick = tick;
    RelationsSystem::new().tick(&mut engine.world, &mut engine.resources);
}

fn of_kind(engine: &SimEngine, kind: ChronicleKind) -> Vec<ChronicleEntry> {
    engine
        .resources
        .chronicle
        .entries()
        .iter()
        .copied()
        .filter(|e| e.kind == kind)
        .collect()
}

fn food(engine: &SimEngine, sid: u32) -> u32 {
    engine.resources.settlements[&sid]
        .stockpile
        .get(&ResourceKind::Food)
        .copied()
        .unwrap_or(0)
}

fn wood(engine: &SimEngine, sid: u32) -> u32 {
    engine.resources.settlements[&sid]
        .stockpile
        .get(&ResourceKind::Wood)
        .copied()
        .unwrap_or(0)
}

fn porters(engine: &SimEngine) -> Vec<(AgentId, PorterTrip)> {
    let mut rows: Vec<_> = engine
        .world
        .query::<(&Agent, &PorterTrip)>()
        .iter()
        .map(|(_, (agent, trip))| (agent.id, *trip))
        .collect();
    rows.sort_by_key(|row| row.0);
    rows
}

/// Run the movement, decision, deposit and relations systems over `ticks`.
fn walk(engine: &mut SimEngine, ticks: std::ops::Range<u64>) {
    let mut movement = AgentMovementSystem::new();
    let mut decide = AgentDecisionSystem::new();
    let mut deposit = StockpileDepositSystem::new();
    let mut relations = RelationsSystem::new();
    for tick in ticks {
        engine.resources.current_tick = tick;
        let (world, resources) = (&mut engine.world, &mut engine.resources);
        if tick.is_multiple_of(movement.tick_interval()) {
            movement.tick(world, resources);
        }
        decide.tick(world, resources);
        if tick.is_multiple_of(deposit.tick_interval()) {
            deposit.tick(world, resources);
        }
        relations.tick(world, resources);
    }
}

#[test]
fn harness_relations_measures_and_keep_lines() {
    // 4 members × 0.05 × 1440 / 30 = 9.6 units a day → need_milli 9,600.
    let (engine, _) = two_villages([(40, 0), (0, 0)], &residents());
    assert_eq!(need_milli(&engine.world, &engine.resources, 1), 9600);
    assert_eq!(food_days_permille(40, 9600), 4166);
    assert_eq!(food_days_permille(14, 9600), 1458);
    assert_eq!(food_days_permille(7, 0), 7_000_000, "need floor of 1");
    assert_eq!(food_keep(9600, 3000), 29, "ceil(28.8)");
    assert_eq!(food_keep(10_000, 3000), 30, "exact");
    assert_eq!(food_keep(0, 3000), 0);
    assert_eq!(wood_nights(37, 13), 2);
    assert_eq!(wood_nights(5, 0), 5, "burn floor of 1");
    assert_eq!(wood_keep(&RULE, 13), 26);
    let burn = burn_night(&engine.resources);
    assert!(
        (13..=52).contains(&burn),
        "one night burns 13–52 units: {burn}"
    );
}

#[test]
fn harness_relations_reach_is_three_quarters_of_the_window() {
    // (900 − 480) × 3 / 4 = 315 ticks; one step per 8 ticks → 39 tiles.
    assert!(in_reach(&RULE, 8, (0, 0), (39, 10)));
    assert!(!in_reach(&RULE, 8, (0, 0), (40, 10)));
    assert!(in_reach(&RULE, 8, CAMP, DAUGHTER));
    assert!(!in_reach(&RULE, 8, (0, 0), (0, 56)));
}

#[test]
fn harness_relations_pairs_follow_living_villages_at_their_kin_base() {
    let (mut engine, _) = two_villages([(0, 0), (0, 0)], &residents());
    let mut sibling = Settlement::new_with_id(3, 0);
    sibling.formation_tile = (20, 56);
    sibling.parent = Some(1);
    let mut stranger = Settlement::new_with_id(4, 0);
    stranger.formation_tile = (80, 10);
    assert_eq!(
        base_relation(
            &RULE,
            &engine.resources.settlements[&1],
            &engine.resources.settlements[&2]
        ),
        300
    );
    assert_eq!(
        base_relation(&RULE, &engine.resources.settlements[&2], &sibling),
        150
    );
    assert_eq!(
        base_relation(&RULE, &engine.resources.settlements[&1], &stranger),
        0
    );
    assert_eq!(
        base_relation(&RULE, &engine.resources.settlements[&2], &stranger),
        0
    );
    engine.resources.settlements.insert(3, sibling);
    engine.resources.settlements.insert(4, stranger);
    relations_at(&mut engine, 100);
    let pairs = &engine.resources.settlement_relations.pairs;
    assert_eq!(
        pairs.keys().copied().collect::<Vec<_>>(),
        vec![(1, 2), (1, 3), (1, 4), (2, 3), (2, 4), (3, 4)]
    );
    assert_eq!(
        pairs[&(1, 2)],
        RelationState {
            value: 300,
            estranged: false,
            base: 300
        }
    );
    assert_eq!(pairs[&(2, 3)].base, 150);
    assert_eq!(pairs[&(3, 4)].base, 0);
    engine.resources.settlements.remove(&3);
    relations_at(&mut engine, 101);
    assert_eq!(
        engine
            .resources
            .settlement_relations
            .pairs
            .keys()
            .copied()
            .collect::<Vec<_>>(),
        vec![(1, 2), (1, 4), (2, 4)],
        "pairs with a missing id are removed"
    );
}

#[test]
fn harness_relations_drift_clamp_and_hysteresis() {
    let (mut engine, _) = two_villages([(0, 0), (0, 0)], &[]);
    relations_at(&mut engine, 1);
    let set = |engine: &mut SimEngine, value: i32, estranged: bool| {
        engine.resources.settlement_relations.pairs.insert(
            (1, 2),
            RelationState {
                value,
                estranged,
                base: 150,
            },
        );
    };
    // Drift: value += (base − value) × 30 / 1000, truncated toward zero.
    assert_eq!(RULE.relation_drift_permille, 30);
    set(&mut engine, -315, false);
    relations_at(&mut engine, decision(1));
    let state = engine.resources.settlement_relations.pairs[&(1, 2)];
    assert_eq!(state.value, -315 + 465 * 30 / 1000);
    assert!(state.estranged, "−302 ≤ −300 after the drift");
    let estranged = of_kind(&engine, ChronicleKind::VillagesEstranged);
    assert_eq!(estranged.len(), 1);
    assert_eq!(
        (
            estranged[0].settlement,
            estranged[0].settlement2,
            estranged[0].value
        ),
        (Some(1), Some(2), -302)
    );
    // Still estranged between the two lines (no flapping).
    set(&mut engine, -160, true);
    relations_at(&mut engine, decision(2));
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)],
        RelationState {
            value: -151,
            estranged: true,
            base: 150
        }
    );
    // Reconciled at ≥ −150 after the drift.
    set(&mut engine, -152, true);
    relations_at(&mut engine, decision(3));
    assert!(!engine.resources.settlement_relations.pairs[&(1, 2)].estranged);
    let reconciled = of_kind(&engine, ChronicleKind::VillagesReconciled);
    assert_eq!(reconciled.len(), 1);
    assert_eq!(reconciled[0].value, -143);
    // Not estranged until the line is reached; evaluated only at the decision.
    set(&mut engine, -299, false);
    relations_at(&mut engine, decision(4) + 1);
    assert!(
        !engine.resources.settlement_relations.pairs[&(1, 2)].estranged,
        "only at the decision minute"
    );
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        -299,
        "no drift off the minute"
    );
    // Clamp.
    let mut state = RelationState {
        value: 995,
        estranged: false,
        base: 300,
    };
    state.add(RULE.delivery_gain);
    assert_eq!(state.value, 1000);
    state.add(-3000);
    assert_eq!(state.value, -1000);
    assert_eq!(engine.resources.tallies.villages_estranged, 1);
    assert_eq!(engine.resources.tallies.villages_reconciled, 1);
}

#[test]
fn harness_relations_kin_aid_sends_food_with_the_most_social_adults() {
    // Camp food days 100 × 1e6 / 9600 = 10,416 ≥ 2,500; daughter 10 → 1,041 < 2,000.
    let (mut engine, entities) = two_villages([(100, 0), (10, 0)], &residents());
    relations_at(&mut engine, decision(1) - 1);
    relations_at(&mut engine, decision(1));
    // min(10 × 2 slots, 100 − keep 20, 10 × 4 candidates) = 20 → two porters, RD 900 then 700 (id 3).
    let rows = porters(&engine);
    assert_eq!(rows.iter().map(|r| r.0).collect::<Vec<_>>(), vec![2, 3]);
    for (_, trip) in &rows {
        assert_eq!(
            (trip.from, trip.to, trip.kind, trip.amount, trip.stage),
            (1, 2, PorterKind::Food, 10, PorterStage::Outbound)
        );
        assert_eq!(trip.depart_tick, decision(1));
    }
    assert_eq!(food(&engine, 1), 80);
    for id in [2, 3] {
        let entity = entities[id as usize - 1];
        assert_eq!(
            engine
                .world
                .get::<&Inventory>(entity)
                .expect("inv")
                .get(ResourceKind::Food),
            10
        );
        assert_eq!(
            *engine.world.get::<&AgentState>(entity).expect("state"),
            AgentState::Idle
        );
    }
    let departed = of_kind(&engine, ChronicleKind::PorterDeparted);
    assert_eq!(departed.len(), 2, "one entry per porter");
    for (entry, id) in departed.iter().zip([2, 3]) {
        assert_eq!(
            (
                entry.subject,
                entry.settlement,
                entry.settlement2,
                entry.cause,
                entry.value
            ),
            (Some(id), Some(1), Some(2), 0, 10)
        );
    }
    let t = engine.resources.tallies;
    assert_eq!(
        (t.porter_trips, t.porters_departed, t.porter_units_withdrawn),
        (1, 2, 20)
    );
    // The day's row records the conditions and what was sent.
    let days = &engine.resources.settlement_relations.days;
    assert_eq!(days.len(), 2);
    assert!(days[0].aid_food && days[0].in_reach && days[0].sent == Some((PorterKind::Food, 20)));
    assert!(!days[1].aid_food && days[1].sent.is_none());
}

#[test]
fn harness_relations_keep_line_bounds_the_load() {
    // The keep line is 2,000 ‰ of a day: ceil(9.6 × 2) = 20. Camp 23 food: days
    // 2,395 < 2,500 → no aid; 28: 2,916, surplus 28 − 20 = 8 → one porter; 31:
    // 3,229, surplus 11 → 2 porters (10 + 1).
    assert_eq!(food_keep(9600, RULE.aid_keep_days_permille), 20);
    for (stock, sent) in [(23, 0), (28, 8), (31, 11)] {
        let (mut engine, _) = two_villages([(stock, 0), (10, 0)], &residents());
        relations_at(&mut engine, decision(1));
        let rows = porters(&engine);
        assert_eq!(
            rows.iter().map(|r| r.1.amount).sum::<u32>(),
            sent,
            "stock {stock}"
        );
        if sent > 10 {
            assert_eq!(
                rows.iter().map(|r| r.1.amount).collect::<Vec<_>>(),
                vec![10, 1]
            );
        }
        if sent > 0 {
            assert_eq!(food(&engine, 1), stock - sent);
        }
    }
}

#[test]
fn harness_relations_aid_wood_when_food_does_not_qualify() {
    let probe = two_villages([(0, 0), (0, 0)], &residents()).0;
    let burn = burn_night(&probe.resources);
    // Camp 5 nights of wood, daughter none; food balanced (no food aid).
    let (mut engine, _) = two_villages([(30, 5 * burn), (30, 0)], &residents());
    relations_at(&mut engine, decision(1));
    let rows = porters(&engine);
    let sent: u32 = rows.iter().map(|r| r.1.amount).sum();
    assert_eq!(sent, (5 * burn - 2 * burn).min(20));
    assert!(rows.iter().all(|r| r.1.kind == PorterKind::Wood));
    assert_eq!(of_kind(&engine, ChronicleKind::PorterDeparted)[0].cause, 1);
}

#[test]
fn harness_relations_exchange_is_both_trips_or_none() {
    let probe = two_villages([(0, 0), (0, 0)], &residents()).0;
    let burn = burn_night(&probe.resources);
    // Camp: food 3.3 days, wood 1 night; daughter: food 1.6 days (no aid:
    // 3.3 < 4), wood 6 nights (aid wood would need the camp < 1 night).
    let stores = [(32, burn), (16, 6 * burn)];
    // The brief's kin-aid thresholds (4,000 / 1,500), so that kin aid does not
    // qualify either: the baseline tries the exchange first and falls back to
    // aid when a leg cannot depart (pass 21), so only then is "none" visible.
    let brief = RelationsRule {
        aid_giver_days_permille: 4000,
        aid_receiver_days_permille: 1500,
        ..RULE
    };
    let (mut engine, _) = two_villages(stores, &residents());
    engine.resources.rules.society.relations = Some(brief);
    relations_at(&mut engine, decision(1));
    let rows = porters(&engine);
    let food_leg: Vec<_> = rows
        .iter()
        .filter(|r| r.1.kind == PorterKind::Food)
        .collect();
    let wood_leg: Vec<_> = rows
        .iter()
        .filter(|r| r.1.kind == PorterKind::Wood)
        .collect();
    assert_eq!(
        food_leg.iter().map(|r| r.1.amount).sum::<u32>(),
        12,
        "32 − keep 20"
    );
    assert!(food_leg.iter().all(|r| (r.1.from, r.1.to) == (1, 2)));
    assert_eq!(
        wood_leg.iter().map(|r| r.1.amount).sum::<u32>(),
        (6 * burn - 2 * burn).min(20)
    );
    assert!(wood_leg.iter().all(|r| (r.1.from, r.1.to) == (2, 1)));
    let wood_sent: u32 = wood_leg.iter().map(|r| r.1.amount).sum();
    assert_eq!(
        (food(&engine, 1), wood(&engine, 2)),
        (20, 6 * burn - wood_sent),
        "both stores debited"
    );
    let causes: Vec<u8> = of_kind(&engine, ChronicleKind::PorterDeparted)
        .iter()
        .map(|e| e.cause)
        .collect();
    assert!(causes.contains(&2) && causes.contains(&3));
    // None: the daughter has nobody able to carry → the camp sends nothing either.
    let mut tired = residents();
    for row in tired.iter_mut().filter(|row| row.1 == 2) {
        row.2 = 500;
    }
    let (mut engine, entities) = two_villages(stores, &tired);
    engine.resources.rules.society.relations = Some(brief);
    for entity in &entities[4..] {
        engine
            .world
            .get::<&mut Hunger>(*entity)
            .expect("hunger")
            .value = 45.0;
    }
    relations_at(&mut engine, decision(1));
    assert!(porters(&engine).is_empty(), "both or none");
    assert_eq!(food(&engine, 1), 32);
}

#[test]
fn harness_relations_candidates_are_free_fed_empty_handed_adults() {
    let mut people = residents();
    people.extend([
        (5, 1, 1000),
        (6, 1, 1000),
        (7, 1, 1000),
        (8, 1, 1000),
        (9, 1, 1000),
        (10, 1, 950),
    ]);
    let (mut engine, entities) = two_villages([(200, 0), (5, 0)], &people);
    let entity =
        |id: AgentId| entities[people.iter().position(|row| row.0 == id).expect("resident")];
    engine
        .world
        .insert_one(entity(5), Child { born_tick: 0 })
        .expect("child");
    engine.world.insert_one(entity(6), Elder).expect("elder");
    engine
        .world
        .insert_one(
            entity(7),
            Founding {
                site: DAUGHTER,
                fission_id: 1,
                leader: 7,
                since_tick: 0,
            },
        )
        .expect("founding");
    engine
        .world
        .get::<&mut Inventory>(entity(8))
        .expect("inv")
        .add(ResourceKind::Wood, 1);
    engine
        .world
        .get::<&mut Hunger>(entity(9))
        .expect("hunger")
        .value = 30.0;
    // RD 950, in an activity: taken (Idle, target dropped, the Activity stays
    // for the routine to end as interrupted).
    let activity = Activity {
        kind: ActivityKind::Craft,
        settlement: 1,
        place: (21, 31),
        chosen_tick: 0,
        arrived_tick: None,
        until_tick: None,
    };
    engine
        .world
        .insert(
            entity(10),
            (
                AgentState::Seeking {
                    target: TargetKind::Activity,
                },
                SeekTarget { tile: (21, 31) },
                activity,
            ),
        )
        .expect("activity");
    // RD 900 (id 2) is seeking food: not free.
    engine
        .world
        .insert_one(
            entity(2),
            AgentState::Seeking {
                target: TargetKind::Food,
            },
        )
        .expect("food");
    relations_at(&mut engine, decision(1));
    assert_eq!(
        porters(&engine).iter().map(|r| r.0).collect::<Vec<_>>(),
        vec![3, 10],
        "RD 950 then RD 700 (id 3 before 4)"
    );
    assert_eq!(
        *engine.world.get::<&AgentState>(entity(10)).expect("state"),
        AgentState::Idle
    );
    assert!(engine.world.get::<&SeekTarget>(entity(10)).is_err());
    assert!(engine.world.get::<&Activity>(entity(10)).is_ok());
}

#[test]
fn harness_relations_no_trips_when_estranged_or_out_of_reach() {
    let (mut engine, _) = two_villages([(100, 0), (10, 0)], &residents());
    relations_at(&mut engine, 1);
    engine.resources.settlement_relations.pairs.insert(
        (1, 2),
        RelationState {
            value: -900,
            estranged: true,
            base: 300,
        },
    );
    relations_at(&mut engine, decision(1));
    assert!(porters(&engine).is_empty());
    assert!(
        engine.resources.settlement_relations.days[0].aid_food
            && engine.resources.settlement_relations.days[0].estranged
    );
    let (mut engine, _) = two_villages([(100, 0), (10, 0)], &residents());
    engine
        .resources
        .settlements
        .get_mut(&2)
        .expect("daughter")
        .formation_tile = (61, 30);
    relations_at(&mut engine, decision(1));
    assert!(porters(&engine).is_empty(), "41 tiles: out of reach");
    assert!(!engine.resources.settlement_relations.days[0].in_reach);
}

#[test]
fn harness_relations_trip_delivers_returns_and_is_not_redeposited() {
    let (mut engine, entities) = two_villages([(100, 0), (10, 0)], &residents());
    let start = decision(1);
    walk(&mut engine, start - 1..start + 1);
    assert_eq!(food(&engine, 1), 80);
    // Ten ticks at home: the deposit system would have taken the load back.
    walk(&mut engine, start + 1..start + 12);
    assert_eq!(food(&engine, 1), 80, "the deposit skips porters");
    assert_eq!(porters(&engine).len(), 2);
    let mut delivered_at = None;
    for tick in start + 12..start + 900 {
        walk(&mut engine, tick..tick + 1);
        if delivered_at.is_none() && food(&engine, 2) > 10 {
            delivered_at = Some(tick);
        }
        if porters(&engine).is_empty() {
            break;
        }
    }
    assert!(
        delivered_at.is_some_and(|tick| tick - start < 315),
        "{delivered_at:?}"
    );
    assert_eq!(food(&engine, 2), 30, "both loads in the daughter's store");
    assert_eq!(food(&engine, 1), 80, "nothing came back");
    assert!(porters(&engine).is_empty(), "both porters are home");
    let pair = engine.resources.settlement_relations.pairs[&(1, 2)];
    assert_eq!(pair.value, 300 + RULE.delivery_gain, "one trip, one credit");
    let trips = &engine.resources.settlement_relations.trips;
    assert_eq!(trips.len(), 2);
    for record in trips {
        assert_eq!(
            (
                record.trip.amount,
                record.delivered,
                record.returned,
                record.lost,
                record.failed
            ),
            (10, 10, 0, 0, false)
        );
        assert!(record.end_tick.is_some());
    }
    assert!(engine.resources.settlement_relations.active.is_empty());
    let delivered = of_kind(&engine, ChronicleKind::PorterDelivered);
    assert_eq!(delivered.len(), 2);
    for entry in &delivered {
        assert_eq!(
            (
                entry.settlement,
                entry.settlement2,
                entry.cause,
                entry.value
            ),
            (Some(2), Some(1), 0, 10)
        );
    }
    for id in [2_u64, 3] {
        let entity = entities[id as usize - 1];
        assert!(engine.world.get::<&PorterTrip>(entity).is_err());
        assert!(
            engine.resources.settlements[&1].member_agents.contains(&id),
            "still the camp's"
        );
        let pos = *engine.world.get::<&Position>(entity).expect("pos");
        assert!(
            pos.x.abs_diff(CAMP.0).max(pos.y.abs_diff(CAMP.1)) <= 5,
            "home"
        );
    }
    let t = engine.resources.tallies;
    assert_eq!(
        (
            t.porter_deliveries,
            t.porter_food_delivered,
            t.porter_turnbacks,
            t.porter_units_returned,
            t.porter_units_lost
        ),
        (2, 20, 0, 0, 0)
    );
}

#[test]
fn harness_relations_turn_back_at_the_deadline_and_when_the_receiver_dissolves() {
    // Deadline: the porters never move (only the relations system runs).
    let (mut engine, _) = two_villages([(100, 0), (10, 0)], &residents());
    let start = decision(1);
    relations_at(&mut engine, start);
    relations_at(&mut engine, start + 419);
    assert!(porters(&engine)
        .iter()
        .all(|r| r.1.stage == PorterStage::Outbound));
    relations_at(&mut engine, start + 420);
    assert!(porters(&engine)
        .iter()
        .all(|r| r.1.stage == PorterStage::Return));
    let failed = of_kind(&engine, ChronicleKind::PorterDelivered);
    assert_eq!(failed.len(), 2);
    assert!(failed.iter().all(|e| e.cause == 10 && e.value == 0));
    // At home (they never left): the loads go back into the camp's store.
    relations_at(&mut engine, start + 421);
    assert_eq!(food(&engine, 1), 100);
    assert!(porters(&engine).is_empty());
    for record in &engine.resources.settlement_relations.trips {
        assert_eq!(
            (
                record.delivered,
                record.returned,
                record.lost,
                record.failed
            ),
            (0, 10, 0, true)
        );
    }
    assert_eq!(engine.resources.tallies.porter_turnbacks, 2);
    assert_eq!(engine.resources.tallies.porter_units_returned, 20);
    // Receiver dissolved: turn back at once.
    let (mut engine, _) = two_villages([(100, 0), (10, 0)], &residents());
    relations_at(&mut engine, start);
    engine.resources.settlements.remove(&2);
    relations_at(&mut engine, start + 1);
    assert!(of_kind(&engine, ChronicleKind::PorterDelivered)
        .iter()
        .all(|e| e.cause == 10));
    assert!(engine.resources.settlement_relations.pairs.is_empty());
}

#[test]
fn harness_relations_loads_are_lost_with_a_dead_porter_or_a_dissolved_sender() {
    let (mut engine, entities) = two_villages([(100, 0), (10, 0)], &residents());
    let start = decision(1);
    relations_at(&mut engine, start);
    despawn_agent(
        &mut engine.world,
        &mut engine.resources,
        entities[1],
        2,
        CAMP,
        DeathReason::Starvation,
        start + 5,
    );
    relations_at(&mut engine, start + 6);
    let trips = &engine.resources.settlement_relations.trips;
    assert_eq!(trips.len(), 1);
    assert_eq!(
        (
            trips[0].porter,
            trips[0].lost,
            trips[0].delivered,
            trips[0].returned
        ),
        (2, 10, 0, 0)
    );
    assert_eq!(engine.resources.tallies.porter_units_lost, 10);
    assert_eq!(engine.resources.tallies.porter_loads_lost, 1);
    // The sender dissolves: the other porter's load is lost too (and leaves its hands).
    engine.resources.settlements.remove(&1);
    relations_at(&mut engine, start + 7);
    assert!(porters(&engine).is_empty());
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(entities[2])
            .expect("inv")
            .total(),
        0
    );
    let ended = &engine.resources.settlement_relations.trips;
    assert_eq!(ended.len(), 2);
    for record in ended {
        assert_eq!(
            record.trip.amount,
            record.delivered + record.returned + record.lost
        );
    }
    assert_eq!(engine.resources.tallies.porter_units_lost, 20);
}

/// A forager `a` of the camp picking up at `bush` and a forager `b` of the
/// daughter heading there; returns the engine and both entities.
fn dispute_world(ns: u16, ha: u16) -> (SimEngine, Entity, Entity) {
    let (mut engine, entities) = two_villages([(0, 0), (0, 0)], &[(1, 1, 500), (11, 2, 500)]);
    let bush = (33, 30);
    engine.resources.food_tiles.insert(bush, 3);
    for (entity, state) in [
        (
            entities[0],
            AgentState::Consuming {
                target: TargetKind::GatherFood,
            },
        ),
        (
            entities[1],
            AgentState::Seeking {
                target: TargetKind::GatherFood,
            },
        ),
    ] {
        engine
            .world
            .insert(
                entity,
                (
                    state,
                    Temperament {
                        ns,
                        ha,
                        rd: 500,
                        p: 500,
                    },
                ),
            )
            .expect("forager");
    }
    *engine.world.get::<&mut Position>(entities[0]).expect("pos") = Position::new(bush.0, bush.1);
    *engine.world.get::<&mut Position>(entities[1]).expect("pos") =
        Position::new(bush.0 + 3, bush.1);
    engine
        .world
        .insert_one(entities[1], SeekTarget { tile: bush })
        .expect("target");
    relations_at(&mut engine, 1);
    (engine, entities[0], entities[1])
}

/// The quarrel draw of forager `a` (the lower AgentId) at `tick`.
fn draw_at(engine: &SimEngine, a: Entity, tick: u64) -> u64 {
    let key = engine.world.get::<&Lineage>(a).expect("lineage").key;
    keyed_below(SEED, STREAM_RELATIONS, key, tick, 0, 0, 1000)
}

/// A daytime tick (10:00 onwards) whose quarrel draw is below / at least `p`.
fn tick_where(engine: &SimEngine, a: Entity, below: bool, p: u64) -> u64 {
    (tick_at(3, 600)..tick_at(3, 1000))
        .find(|t| (draw_at(engine, a, *t) < p) == below)
        .expect("a tick")
}

/// Put the two foragers back at (`a`) and on the way to (`b`) a full bush.
fn rearm(engine: &mut SimEngine, a: Entity, b: Entity) {
    engine.resources.food_tiles.insert((33, 30), 3);
    engine
        .world
        .insert_one(
            a,
            AgentState::Consuming {
                target: TargetKind::GatherFood,
            },
        )
        .expect("a");
    engine
        .world
        .get::<&mut Inventory>(a)
        .expect("inv")
        .items
        .clear();
    *engine.world.get::<&mut Position>(a).expect("pos") = Position::new(33, 30);
    engine
        .world
        .insert(
            b,
            (
                AgentState::Seeking {
                    target: TargetKind::GatherFood,
                },
                SeekTarget { tile: (33, 30) },
            ),
        )
        .expect("b");
}

fn decide_at(engine: &mut SimEngine, tick: u64) {
    engine.resources.current_tick = tick;
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
}

#[test]
fn harness_relations_dispute_costs_relation_once_per_forager_pair_and_day() {
    // NS 0, HA 1000: p = clamp((0 + 0 − 2000 + 1000) / 4) = 0 → never a quarrel.
    let (mut engine, a, b) = dispute_world(0, 1000);
    let tick = tick_at(3, 600);
    decide_at(&mut engine, tick);
    assert_eq!(engine.resources.tallies.forage_disputes, 1);
    assert_eq!(engine.resources.tallies.quarrels, 0);
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        300 - RULE.dispute_loss
    );
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(a)
            .expect("inv")
            .get(ResourceKind::Food),
        3,
        "the pickup happened"
    );
    assert!(engine
        .resources
        .settlement_relations
        .disputed
        .contains(&(1, 11)));
    // The same pair again the same day: no second dispute (the pickup happens).
    rearm(&mut engine, a, b);
    decide_at(&mut engine, tick + 30);
    assert_eq!(engine.resources.tallies.forage_disputes, 1);
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(a)
            .expect("inv")
            .get(ResourceKind::Food),
        3
    );
    // The next day it counts again.
    rearm(&mut engine, a, b);
    decide_at(&mut engine, tick + 1440);
    assert_eq!(engine.resources.tallies.forage_disputes, 2);
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        300 - 2 * RULE.dispute_loss
    );
    assert!(
        of_kind(&engine, ChronicleKind::Quarrel).is_empty(),
        "disputes are counted, not chronicled"
    );
}

#[test]
fn harness_relations_no_rival_no_dispute() {
    // The rival of a pickup must belong to another village; foragers of one
    // village, or a village without a relation entry, never dispute.
    let (mut engine, _, _) = dispute_world(1000, 0);
    engine
        .resources
        .settlements
        .get_mut(&2)
        .expect("daughter")
        .remove_member_agent(11);
    engine
        .resources
        .settlements
        .get_mut(&1)
        .expect("camp")
        .add_member_agent(11);
    decide_at(&mut engine, tick_at(3, 600));
    assert_eq!(engine.resources.tallies.forage_disputes, 0);
    // `pairs` is synced by the relations system only; the decision skips a
    // pair without an entry.
    let (mut engine, _, _) = dispute_world(1000, 0);
    engine.resources.settlement_relations.pairs.clear();
    decide_at(&mut engine, tick_at(3, 600));
    assert_eq!(engine.resources.tallies.forage_disputes, 0);
}

#[test]
fn harness_relations_quarrel_is_a_keyed_draw_that_stops_both_pickups() {
    // NS 1000, HA 0: p = clamp((2000 + 1000) / 4, 0, 500) = 500.
    let (mut engine, a, b) = dispute_world(1000, 0);
    let tick = tick_where(&engine, a, true, 500);
    // The rival has arrived too (its pickup this tick must be a no-op).
    *engine.world.get::<&mut Position>(b).expect("pos") = Position::new(33, 30);
    engine
        .world
        .insert_one(
            b,
            AgentState::Consuming {
                target: TargetKind::GatherFood,
            },
        )
        .expect("arrived");
    decide_at(&mut engine, tick);
    assert_eq!(engine.resources.tallies.quarrels, 1);
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        300 - RULE.dispute_loss - RULE.quarrel_loss
    );
    for forager in [a, b] {
        assert_eq!(
            engine
                .world
                .get::<&Inventory>(forager)
                .expect("inv")
                .total(),
            0,
            "no pickup"
        );
        assert_eq!(
            *engine.world.get::<&AgentState>(forager).expect("state"),
            AgentState::Idle
        );
    }
    assert_eq!(engine.resources.food_tiles.get(&(33, 30)), Some(&3));
    let quarrels = of_kind(&engine, ChronicleKind::Quarrel);
    assert_eq!(quarrels.len(), 1);
    let q = quarrels[0];
    assert_eq!(
        (
            q.subject,
            q.other,
            q.settlement,
            q.settlement2,
            q.value,
            q.tick
        ),
        (
            Some(1),
            Some(11),
            Some(1),
            Some(2),
            i64::from(300 - RULE.dispute_loss - RULE.quarrel_loss),
            tick
        )
    );
    // The same world at a tick whose draw is ≥ p: a dispute only.
    let (mut engine, a, _) = dispute_world(1000, 0);
    let calm = tick_where(&engine, a, false, 500);
    decide_at(&mut engine, calm);
    assert_eq!(
        (
            engine.resources.tallies.forage_disputes,
            engine.resources.tallies.quarrels
        ),
        (1, 0)
    );
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(a)
            .expect("inv")
            .get(ResourceKind::Food),
        3
    );
    // Without temperament p = 250: the draw decides with the same key.
    let (mut engine, a, _) = dispute_world(500, 500);
    let tick = tick_where(&engine, a, true, 250);
    decide_at(&mut engine, tick);
    assert_eq!(engine.resources.tallies.quarrels, 1);
}

#[test]
fn harness_relations_estranged_foragers_avoid_the_others_side() {
    // A daughter forager, one bush 15 tiles west of its hearth (11 from the
    // camp's): eligible normally, avoided when the villages are estranged.
    for estranged in [false, true] {
        let (mut engine, entities) = two_villages([(0, 0), (0, 0)], &[(11, 2, 500)]);
        engine.resources.food_tiles.insert((31, 30), 3);
        relations_at(&mut engine, 1);
        engine.resources.settlement_relations.pairs.insert(
            (1, 2),
            RelationState {
                value: -400,
                estranged,
                base: 300,
            },
        );
        decide_at(&mut engine, tick_at(3, 600));
        let state = *engine.world.get::<&AgentState>(entities[0]).expect("state");
        let foraging = state
            == AgentState::Seeking {
                target: TargetKind::GatherFood,
            }
            && engine
                .world
                .get::<&SeekTarget>(entities[0])
                .is_ok_and(|s| s.tile == (31, 30));
        assert_eq!(foraging, !estranged, "estranged {estranged}: {state:?}");
        assert_eq!(engine.resources.tallies.forage_trips, u64::from(!estranged));
    }
    // A bush equally far from both hearths stays eligible (ties keep it).
    let (mut engine, entities) = two_villages([(0, 0), (0, 0)], &[(11, 2, 500)]);
    engine.resources.food_tiles.insert((33, 30), 3);
    relations_at(&mut engine, 1);
    engine.resources.settlement_relations.pairs.insert(
        (1, 2),
        RelationState {
            value: -400,
            estranged: true,
            base: 300,
        },
    );
    decide_at(&mut engine, tick_at(3, 600));
    assert_eq!(
        *engine.world.get::<&AgentState>(entities[0]).expect("state"),
        AgentState::Seeking {
            target: TargetKind::GatherFood
        }
    );
}

#[test]
fn harness_relations_integration_quarrel_estranges_and_stops_trips() {
    // Brief 6.9: relation −290, one forced quarrel (−5 −40 = −335), the
    // next decision drifts (+19 toward the parent base 300) to −316 and
    // estranges the pair; with an aid condition met no trip departs, and the
    // avoid filter applies.
    let (mut engine, a, _) = dispute_world(1000, 0);
    engine.resources.settlement_relations.pairs.insert(
        (1, 2),
        RelationState {
            value: -290,
            estranged: false,
            base: 300,
        },
    );
    let tick = tick_where(&engine, a, true, 500);
    decide_at(&mut engine, tick);
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        -290 - RULE.dispute_loss - RULE.quarrel_loss
    );
    assert!(
        !engine.resources.settlement_relations.pairs[&(1, 2)].estranged,
        "only at the decision"
    );
    // Aid would qualify: camp rich, daughter poor.
    engine
        .resources
        .settlements
        .get_mut(&1)
        .expect("camp")
        .store(ResourceKind::Food, 100);
    engine.world.insert_one(a, AgentState::Idle).expect("idle");
    engine
        .world
        .get::<&mut Inventory>(a)
        .expect("inv")
        .remove(ResourceKind::Food, 10);
    relations_at(&mut engine, decision(4));
    let state = engine.resources.settlement_relations.pairs[&(1, 2)];
    assert_eq!(state.value, -335 + 635 * 30 / 1000);
    assert!(state.estranged);
    let estranged = of_kind(&engine, ChronicleKind::VillagesEstranged);
    assert_eq!(estranged.len(), 1);
    assert_eq!(estranged[0].value, -316);
    assert!(engine
        .resources
        .settlement_relations
        .days
        .iter()
        .any(|d| d.aid_food && d.estranged));
    assert!(
        porters(&engine).is_empty(),
        "no trips between estranged villages"
    );
    assert_eq!(engine.resources.tallies.porters_departed, 0);
    // The avoid filter: the daughter's forager no longer takes the bush on the camp's side.
    let (mut engine2, entities) = two_villages([(0, 0), (0, 0)], &[(11, 2, 500)]);
    engine2.resources.settlement_relations = engine.resources.settlement_relations.clone();
    engine2.resources.food_tiles.insert((31, 30), 3);
    decide_at(&mut engine2, tick_at(4, 600));
    assert_ne!(
        *engine2
            .world
            .get::<&AgentState>(entities[0])
            .expect("state"),
        AgentState::Seeking {
            target: TargetKind::GatherFood
        }
    );
    assert_eq!(engine2.resources.tallies.forage_trips, 0);
    // The chronicle: Quarrel then VillagesEstranged.
    let kinds: Vec<ChronicleKind> = engine
        .resources
        .chronicle
        .entries()
        .iter()
        .map(|e| e.kind)
        .collect();
    assert_eq!(
        kinds,
        vec![ChronicleKind::Quarrel, ChronicleKind::VillagesEstranged]
    );
}

#[test]
fn harness_relations_pair_key_and_estranged_lookup() {
    assert_eq!(SettlementRelations::key(9, 4), (4, 9));
}

#[test]
fn harness_relations_trips_and_quarrels_are_deterministic() {
    // Review round 1, m6: a twin over the relations code itself — porters
    // walking out and back, a forced dispute — compared field by field.
    let run = || {
        let (mut engine, _) = two_villages([(100, 0), (10, 0)], &residents());
        let start = decision(1);
        walk(&mut engine, start - 1..start + 700);
        let (mut quarrel, a, b) = dispute_world(1000, 0);
        let tick = tick_where(&quarrel, a, true, 500);
        *quarrel.world.get::<&mut Position>(b).expect("pos") = Position::new(33, 30);
        decide_at(&mut quarrel, tick);
        let positions: Vec<(AgentId, u32, u32)> = {
            let mut rows: Vec<_> = engine
                .world
                .query::<(&Agent, &Position)>()
                .iter()
                .map(|(_, (a, p))| (a.id, p.x, p.y))
                .collect();
            rows.sort_unstable();
            rows
        };
        (
            engine.resources.settlement_relations.clone(),
            engine.resources.chronicle.clone(),
            engine.resources.tallies,
            positions,
            quarrel.resources.settlement_relations.clone(),
            quarrel.resources.chronicle.clone(),
        )
    };
    let (first, second) = (run(), run());
    assert_eq!(
        first.0.trips.len(),
        2,
        "the twin exercises two porter trips"
    );
    assert_eq!(
        first.4.pairs[&(1, 2)].value,
        300 - RULE.dispute_loss - RULE.quarrel_loss,
        "and a quarrel"
    );
    assert_eq!(first, second);
}
