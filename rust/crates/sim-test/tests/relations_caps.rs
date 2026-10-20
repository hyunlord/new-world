//! Pass 21: unit and integration tests of the relations rule of the live
//! rules (`WorldRules::baseline`, = Pass 21's `game_v26`) — the rule values, the avoid filter at the Gather arm's selectors
//! (trigger, post-pass, exit predicate and its memo key), the per-village-pair
//! daily caps on disputes and quarrels and their ledger, exchange before aid,
//! the aid keep line, the 30 ‰ drift and its integer stall, the integration
//! case of brief 6.7 and the invariants of brief 9.
//!
//! V8-0: the Pass 20 (`game_v25`) contrast arms are gone (they set the
//! caps to 0 and turned off the avoid lists and exchange-first order, flags
//! removed in V8-0 8.3); where a
//! contrast carries the test, it is made inside the baseline (estranged or
//! not, a carrier or none).
//!
//! The fixtures follow Pass 20's (`harness_revitalize_pass20_relations.rs`):
//! two bare villages on a 96 × 64 map, the camp (settlement 1) at `CAMP` and
//! its daughter (settlement 2, parent 1) at `DAUGHTER`, 26 tiles east.
use hecs::Entity;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::components::{
    Activity, ActivityKind, Agent, AgentId, AgentState, Hunger, Inventory, Lineage, PorterKind,
    PorterTrip, Position, RelationState, ResourceKind, SeekTarget, Settlement, TargetKind,
    Temperament,
};
use sim_core::material::MaterialRegistry;
use sim_core::rng::{keyed_below, STREAM_RELATIONS};
use sim_core::rules::{RelationsRule, WorldRules};
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::decision::AgentDecisionSystem;
use sim_systems::runtime::settlement::food_economy::stock_target;
use sim_systems::runtime::settlement::relations::{burn_night, RelationsSystem};

const SEED: u64 = 20;
const CAMP: (u32, u32) = (20, 30);
const DAUGHTER: (u32, u32) = (46, 30);
/// The live relations rule (`WorldRules::baseline`; = Pass 21's
/// `RULE_RELATIONS`, the grid's configuration G2).
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

/// Two villages under `rules` with the given stores (food, wood) and residents.
fn villages(
    rules: WorldRules,
    stores: [(u32, u32); 2],
    residents: &[Resident],
) -> (SimEngine, Vec<Entity>) {
    let mut engine = SimEngine::new(96, 64, MaterialRegistry::new());
    engine.resources.rules = rules;
    engine.resources.world_seed = SEED;
    let mut list = Vec::new();
    for (index, (tile, parent)) in [(CAMP, None), (DAUGHTER, Some(1))].into_iter().enumerate() {
        let mut settlement = Settlement::new_with_id(index as u32 + 1, 0);
        settlement.formation_tile = tile;
        settlement.parent = parent;
        settlement.store(ResourceKind::Food, stores[index].0);
        settlement.store(ResourceKind::Wood, stores[index].1);
        list.push(settlement);
    }
    let mut entities = Vec::new();
    for &(id, village, rd) in residents {
        let home = if village == 1 { CAMP } else { DAUGHTER };
        entities.push(spawn(&mut engine, id, home, rd));
        list[village as usize - 1].add_member_agent(id);
    }
    for settlement in list {
        engine
            .resources
            .settlements
            .insert(settlement.settlement_id, settlement);
    }
    (engine, entities)
}

fn rules() -> WorldRules {
    WorldRules::baseline(SEED)
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

fn decide_at(engine: &mut SimEngine, tick: u64) {
    engine.resources.current_tick = tick;
    AgentDecisionSystem::new().tick(&mut engine.world, &mut engine.resources);
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

fn state(engine: &SimEngine, entity: Entity) -> AgentState {
    *engine.world.get::<&AgentState>(entity).expect("state")
}

fn target(engine: &SimEngine, entity: Entity) -> Option<(u32, u32)> {
    engine.world.get::<&SeekTarget>(entity).ok().map(|s| s.tile)
}

fn set_pair(engine: &mut SimEngine, value: i32, estranged: bool) {
    engine.resources.settlement_relations.pairs.insert(
        (1, 2),
        RelationState {
            value,
            estranged,
            base: 300,
        },
    );
}

// ---------------------------------------------------------------- the rule

#[test]
fn harness_p21_v26_relations_frozen_at_grid_g2() {
    // Pass 21 7.2: the grid's selected configuration G2, frozen; every field.
    let r = RULE;
    assert_eq!(r.decision_minute, 480);
    assert_eq!(r.latest_arrival_minute, 900);
    assert_eq!(r.porter_max, 2);
    assert_eq!(r.aid_giver_days_permille, 2500);
    assert_eq!(r.aid_keep_days_permille, 2000);
    assert_eq!(r.aid_receiver_days_permille, 2000);
    assert_eq!(r.wood_giver_nights, 3);
    assert_eq!(r.wood_keep_nights, 2);
    assert_eq!(r.wood_receiver_nights, 1);
    assert_eq!(r.trade_food_surplus_days_permille, 3000);
    assert_eq!(r.trade_food_short_days_permille, 2000);
    assert_eq!(r.trade_wood_surplus_nights, 4);
    assert_eq!(r.trade_wood_short_nights, 2);
    assert_eq!(r.relation_kin_parent, 300);
    assert_eq!(r.relation_kin_sibling, 150);
    assert_eq!(r.relation_other, 0);
    assert_eq!(r.relation_drift_permille, 30);
    assert_eq!(r.delivery_gain, 40);
    assert_eq!(r.dispute_loss, 5);
    assert_eq!(r.quarrel_loss, 40);
    assert_eq!(r.estrange_at, -300);
    assert_eq!(r.reconcile_at, -150);
    assert_eq!(r.porter_hunger_max, 30);
    assert_eq!(r.dispute_cap_per_pair_day, 3);
    assert_eq!(r.quarrel_cap_per_pair_day, 1);
}

// ------------------------------------------------- 6.2 the gather avoid

/// A daughter forager (id 11) Idle at (36, 30) at 10:00 of day 3 with the
/// store just at its target (the Forage arm is off, the Gather arm on), the
/// relation estranged, and the given food tiles.
fn gather_world(rules: WorldRules, tiles: &[(u32, u32)]) -> (SimEngine, Entity) {
    let (mut engine, entities) = villages(rules, [(0, 0), (0, 0)], &[(11, 2, 500)]);
    let forager = entities[0];
    *engine.world.get::<&mut Position>(forager).expect("pos") = Position::new(36, 30);
    engine.resources.current_tick = tick_at(3, 600);
    let goal = stock_target(&engine.world, &engine.resources, 2).expect("stock target");
    engine
        .resources
        .settlements
        .get_mut(&2)
        .expect("daughter")
        .store(ResourceKind::Food, goal.ceil() as u32);
    for tile in tiles {
        engine.resources.food_tiles.insert(*tile, 3);
    }
    relations_at(&mut engine, 1);
    set_pair(&mut engine, -400, true);
    (engine, forager)
}

#[test]
fn harness_p21_gather_arm_and_post_pass_skip_the_estranged_side() {
    // (31, 30): 11 from the camp's hearth, 15 from the daughter's — the
    // camp's side, 5 from the forager. (40, 30): own side, 4 away... nearer;
    // so use (41, 30) (5 away, own side) to make the avoided tile the nearer.
    let tiles = [(31, 30), (42, 30)];
    // Estranged: the forager passes the nearer tile on the camp's side.
    let (mut engine, forager) = gather_world(rules(), &tiles);
    decide_at(&mut engine, tick_at(3, 600));
    assert_eq!(
        state(&engine, forager),
        AgentState::Seeking {
            target: TargetKind::GatherFood
        }
    );
    // The post-pass attached the tile the arm chose (one selector).
    assert_eq!(target(&engine, forager), Some((42, 30)));
    assert_eq!(
        engine.resources.tallies.forage_trips, 0,
        "the Gather arm, not the Forage arm"
    );
    // Not estranged: it takes the nearer tile on the camp's side.
    let (mut engine, forager) = gather_world(rules(), &tiles);
    set_pair(&mut engine, -100, false);
    decide_at(&mut engine, tick_at(3, 600));
    assert_eq!(target(&engine, forager), Some((31, 30)));
    // Only the avoided tile: estranged finds nothing, not estranged gathers.
    for (estranged, gathers) in [(true, false), (false, true)] {
        let (mut engine, forager) = gather_world(rules(), &[(31, 30)]);
        if !estranged {
            set_pair(&mut engine, -100, false);
        }
        decide_at(&mut engine, tick_at(3, 600));
        let seeking = state(&engine, forager)
            == AgentState::Seeking {
                target: TargetKind::GatherFood,
            };
        assert_eq!(seeking, gathers, "estranged {estranged}");
    }
}

#[test]
fn harness_p21_post_pass_without_a_target_skips_the_estranged_side() {
    // A gatherer without a target (its arm's tile emptied, the post-pass
    // re-resolves it with the arm's selector): estranged, it goes Idle
    // rather than crossing over; not estranged, it heads to the tile.
    for (estranged, expected) in [(true, None), (false, Some((31, 30)))] {
        let (mut engine, forager) = gather_world(rules(), &[(31, 30)]);
        if !estranged {
            set_pair(&mut engine, -100, false);
        }
        engine
            .world
            .insert_one(
                forager,
                AgentState::Seeking {
                    target: TargetKind::GatherFood,
                },
            )
            .expect("seeking");
        decide_at(&mut engine, tick_at(3, 600));
        assert_eq!(target(&engine, forager), expected);
        if expected.is_none() {
            assert_eq!(state(&engine, forager), AgentState::Idle);
        }
    }
}

#[test]
fn harness_p21_gather_memo_keys_the_avoid_list_owner() {
    // Two villages (2 and 3) share one hearth; 2 is estranged from 1, 3 is
    // not. A member of each, in an activity at the same tile, asks the exit
    // predicate whether a gather tile exists: the only food lies on the
    // camp's side. The member of 3 (lower id, asked first) gets yes and
    // leaves its activity; the member of 2 gets no and stays — a memo keyed
    // only by (tile, catchment) would hand it 3's answer.
    let (mut engine, _) = villages(rules(), [(0, 0), (0, 0)], &[]);
    let mut third = Settlement::new_with_id(3, 0);
    third.formation_tile = DAUGHTER;
    engine.resources.settlements.insert(3, third);
    let first = spawn(&mut engine, 5, (36, 30), 500);
    let second = spawn(&mut engine, 6, (36, 30), 500);
    engine
        .resources
        .settlements
        .get_mut(&3)
        .expect("3")
        .add_member_agent(5);
    engine
        .resources
        .settlements
        .get_mut(&2)
        .expect("2")
        .add_member_agent(6);
    let tick = tick_at(3, 600);
    engine.resources.current_tick = tick;
    for sid in [2, 3] {
        let goal = stock_target(&engine.world, &engine.resources, sid).expect("target");
        engine
            .resources
            .settlements
            .get_mut(&sid)
            .expect("village")
            .store(ResourceKind::Food, goal.ceil() as u32);
    }
    for (entity, sid) in [(first, 3), (second, 2)] {
        let activity = Activity {
            kind: ActivityKind::Craft,
            settlement: sid,
            place: (36, 30),
            chosen_tick: tick - 10,
            arrived_tick: Some(tick - 5),
            until_tick: Some(tick + 60),
        };
        engine
            .world
            .insert(
                entity,
                (
                    AgentState::Consuming {
                        target: TargetKind::Activity,
                    },
                    SeekTarget { tile: (36, 30) },
                    activity,
                ),
            )
            .expect("activity");
    }
    engine.resources.food_tiles.insert((31, 30), 3);
    relations_at(&mut engine, 1);
    for (pair, estranged) in [((1, 2), true), ((1, 3), false), ((2, 3), false)] {
        engine.resources.settlement_relations.pairs.insert(
            pair,
            RelationState {
                value: if estranged { -400 } else { 150 },
                estranged,
                base: 150,
            },
        );
    }
    decide_at(&mut engine, tick);
    assert_eq!(
        state(&engine, first),
        AgentState::Seeking {
            target: TargetKind::GatherFood
        },
        "3's member gathers"
    );
    assert_eq!(target(&engine, first), Some((31, 30)));
    assert_eq!(
        state(&engine, second),
        AgentState::Consuming {
            target: TargetKind::Activity
        },
        "2's member stays"
    );
}

// --------------------------------------------------------- 6.3 the caps

/// The camp forager `a` (id 1) picking up at a bush and a daughter forager
/// (id 11) heading there, under `rule`.
fn dispute_world(rule: RelationsRule, ns: u16, ha: u16) -> (SimEngine, Entity, Entity) {
    let mut rules = rules();
    rules.society.relations = Some(rule);
    let (mut engine, entities) = villages(rules, [(0, 0), (0, 0)], &[(1, 1, 500), (11, 2, 500)]);
    for entity in &entities {
        engine
            .world
            .insert_one(
                *entity,
                Temperament {
                    ns,
                    ha,
                    rd: 500,
                    p: 500,
                },
            )
            .expect("temperament");
    }
    relations_at(&mut engine, 1);
    (engine, entities[0], entities[1])
}

/// Put `a` at the bush (33, 30) picking up and `b` on its way there.
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
    *engine.world.get::<&mut Position>(b).expect("pos") = Position::new(36, 30);
}

/// Forget that the two foragers disputed today (a new forager pair, as far
/// as the once-per-pair-day rule is concerned), keeping the pair's ledger.
fn new_forager_pair(engine: &mut SimEngine) {
    engine.resources.settlement_relations.disputed.clear();
}

fn draw_at(engine: &SimEngine, a: Entity, tick: u64) -> u64 {
    let key = engine.world.get::<&Lineage>(a).expect("lineage").key;
    keyed_below(SEED, STREAM_RELATIONS, key, tick, 0, 0, 1000)
}

/// Daytime ticks of day 3 whose quarrel draw is below `p` (or not).
fn ticks_where(engine: &SimEngine, a: Entity, below: bool, p: u64) -> Vec<u64> {
    (tick_at(3, 600)..tick_at(3, 1000))
        .filter(|t| (draw_at(engine, a, *t) < p) == below)
        .collect()
}

#[test]
fn harness_p21_dispute_cap_stops_the_loss_not_the_count() {
    // NS 0, HA 1000: p = 0, never a quarrel. Five disputes in one day: three
    // cost `dispute_loss`, two are counted uncosted.
    let (mut engine, a, b) = dispute_world(RULE, 0, 1000);
    let start = tick_at(3, 600);
    for n in 0..5 {
        rearm(&mut engine, a, b);
        new_forager_pair(&mut engine);
        decide_at(&mut engine, start + n * 10);
    }
    let t = engine.resources.tallies;
    assert_eq!(
        (
            t.forage_disputes,
            t.disputes_uncosted,
            t.quarrels,
            t.quarrels_capped
        ),
        (5, 2, 0, 0)
    );
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        300 - 3 * RULE.dispute_loss
    );
    assert_eq!(
        engine.resources.settlement_relations.day_counts[&(1, 2)],
        (3, 0)
    );
    let day = engine.resources.settlement_relations.dispute_day;
    assert_eq!(
        engine.resources.settlement_relations.dispute_days[&(day, 1, 2)],
        (5, 0),
        "the log counts every dispute"
    );
    // The next display day the ledger starts again.
    rearm(&mut engine, a, b);
    decide_at(&mut engine, start + 1440);
    assert_eq!(
        engine.resources.settlement_relations.day_counts[&(1, 2)],
        (1, 0)
    );
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        300 - 4 * RULE.dispute_loss
    );
}

#[test]
fn harness_p21_quarrel_cap_skips_the_draw_and_the_pickup_proceeds() {
    // NS 1000, HA 0: p = 500. Two disputes at ticks whose draw is below p:
    // the first quarrels; the second meets the cap — no draw, the pickup
    // happens, `quarrels_capped` counts it.
    let (mut engine, a, b) = dispute_world(RULE, 1000, 0);
    let hot = ticks_where(&engine, a, true, 500);
    rearm(&mut engine, a, b);
    decide_at(&mut engine, hot[0]);
    assert_eq!(engine.resources.tallies.quarrels, 1);
    assert_eq!(
        engine.world.get::<&Inventory>(a).expect("inv").total(),
        0,
        "a quarrel: no pickup"
    );
    rearm(&mut engine, a, b);
    new_forager_pair(&mut engine);
    decide_at(&mut engine, hot[1]);
    let t = engine.resources.tallies;
    assert_eq!(
        (t.forage_disputes, t.quarrels, t.quarrels_capped),
        (2, 1, 1)
    );
    assert_eq!(
        engine
            .world
            .get::<&Inventory>(a)
            .expect("inv")
            .get(ResourceKind::Food),
        3,
        "capped: the pickup happened"
    );
    assert_eq!(
        engine.resources.settlement_relations.day_counts[&(1, 2)],
        (2, 1)
    );
    assert_eq!(
        engine.resources.settlement_relations.pairs[&(1, 2)].value,
        300 - 2 * RULE.dispute_loss - RULE.quarrel_loss
    );
    assert_eq!(of_kind(&engine, ChronicleKind::Quarrel).len(), 1);
    assert_eq!(
        engine
            .resources
            .settlement_relations
            .dispute_days
            .values()
            .copied()
            .collect::<Vec<_>>(),
        vec![(2, 1)]
    );
}

#[test]
fn harness_p21_quarrels_while_estranged_are_counted() {
    for estranged in [false, true] {
        let (mut engine, a, b) = dispute_world(RULE, 1000, 0);
        set_pair(&mut engine, if estranged { -400 } else { 300 }, estranged);
        let hot = ticks_where(&engine, a, true, 500);
        rearm(&mut engine, a, b);
        decide_at(&mut engine, hot[0]);
        assert_eq!(engine.resources.tallies.quarrels, 1);
        assert_eq!(
            engine.resources.tallies.quarrels_while_estranged,
            u64::from(estranged)
        );
    }
}

// ------------------------------------------------- 6.5 exchange first

#[test]
fn harness_p21_exchange_first_takes_the_exchange_over_the_aid() {
    // Camp food 3.3 days and 1 wood night; daughter food 1.6 days and 6 wood
    // nights: both the kin-aid food condition and the exchange hold. The
    // pair exchanges (12 food above the 2-day keep line for the daughter's
    // wood above 2 nights) instead of sending aid.
    let burn = burn_night(
        &villages(rules(), [(0, 0), (0, 0)], &residents())
            .0
            .resources,
    );
    let stores = [(32, burn), (16, 6 * burn)];
    let (mut engine, _) = villages(rules(), stores, &residents());
    relations_at(&mut engine, decision(1));
    let rows = porters(&engine);
    let food_leg: u32 = rows
        .iter()
        .filter(|r| (r.1.from, r.1.kind) == (1, PorterKind::Food))
        .map(|r| r.1.amount)
        .sum();
    let wood_leg: u32 = rows
        .iter()
        .filter(|r| (r.1.from, r.1.kind) == (2, PorterKind::Wood))
        .map(|r| r.1.amount)
        .sum();
    assert_eq!(food_leg, 12, "32 − keep 20");
    assert_eq!(wood_leg, (6 * burn - 2 * burn).min(20));
    assert_eq!(
        rows.len(),
        rows.iter()
            .filter(|r| r.1.kind == PorterKind::Wood || r.1.from == 1)
            .count()
    );
    let causes: Vec<u8> = of_kind(&engine, ChronicleKind::PorterDeparted)
        .iter()
        .map(|e| e.cause)
        .collect();
    assert!(
        causes.iter().all(|c| *c == 2 || *c == 3) && causes.contains(&2) && causes.contains(&3),
        "{causes:?}"
    );
    let days = &engine.resources.settlement_relations.days;
    assert!(days[0].aid_food && days[0].exchange);
    assert_eq!(days[0].sent, Some((PorterKind::Food, 12)));
}

#[test]
fn harness_p21_exchange_first_when_the_higher_id_sends_food() {
    // Mirror: the daughter (2) has the food, the camp (1) the wood and the
    // daughter no wood at all. Both aid conditions hold (wood aid 1 → 2 is
    // checked first for the pair (1, 2)), but the pair's exchange is taken
    // at (1, 2) already: food 2 → 1 for wood 1 → 2, no aid trip.
    let burn = burn_night(
        &villages(rules(), [(0, 0), (0, 0)], &residents())
            .0
            .resources,
    );
    let stores = [(16, 6 * burn), (32, 0)];
    let (mut engine, _) = villages(rules(), stores, &residents());
    relations_at(&mut engine, decision(1));
    let rows = porters(&engine);
    assert!(rows.iter().all(|r| match r.1.kind {
        PorterKind::Food => (r.1.from, r.1.to) == (2, 1),
        PorterKind::Wood => (r.1.from, r.1.to) == (1, 2),
    }));
    let mut causes: Vec<u8> = of_kind(&engine, ChronicleKind::PorterDeparted)
        .iter()
        .map(|e| e.cause)
        .collect();
    causes.sort_unstable();
    causes.dedup();
    assert_eq!(causes, vec![2, 3], "the exchange");
}

#[test]
fn harness_p21_exchange_first_falls_back_to_aid_when_a_leg_cannot_depart() {
    // The daughter has nobody able to carry: no exchange, so the camp sends
    // its food aid (down to the 2-day keep line: 12).
    let burn = burn_night(
        &villages(rules(), [(0, 0), (0, 0)], &residents())
            .0
            .resources,
    );
    let (mut engine, entities) = villages(rules(), [(32, burn), (16, 6 * burn)], &residents());
    for entity in &entities[4..] {
        engine
            .world
            .get::<&mut Hunger>(*entity)
            .expect("hunger")
            .value = 45.0;
    }
    relations_at(&mut engine, decision(1));
    let rows = porters(&engine);
    assert_eq!(
        rows.iter()
            .map(|r| (r.1.from, r.1.kind))
            .collect::<Vec<_>>(),
        vec![(1, PorterKind::Food), (1, PorterKind::Food)]
    );
    assert_eq!(rows.iter().map(|r| r.1.amount).sum::<u32>(), 12);
    assert!(of_kind(&engine, ChronicleKind::PorterDeparted)
        .iter()
        .all(|e| e.cause == 0));
}

// ------------------------------------------------- 6.4 the keep line

#[test]
fn harness_p21_keep_line_below_the_giver_threshold_leaves_a_surplus() {
    // Camp 29 food (3,020 ‰ of a day ≥ the 2,500 ‰ giver line), daughter
    // 10: the camp keeps 2 days (20) and sends 9 (Pass 20's keep line of 3
    // days would have kept all 29). Camp 23 food (2,395 ‰, below the giver
    // line): no aid.
    for (camp, aid, sent) in [(29, true, 9), (23, false, 0)] {
        let (mut engine, _) = villages(rules(), [(camp, 0), (10, 0)], &residents());
        relations_at(&mut engine, decision(1));
        assert_eq!(
            engine.resources.settlement_relations.days[0].aid_food, aid,
            "camp {camp}"
        );
        assert_eq!(
            porters(&engine).iter().map(|r| r.1.amount).sum::<u32>(),
            sent
        );
        assert_eq!(food(&engine, 1), camp - sent);
    }
}

// ------------------------------------------------- drift and its stall

#[test]
fn harness_p21_drift_is_thirty_permille_with_an_integer_stall() {
    // (base − value) × 30 / 1000, truncated toward zero: a gap of 33 or
    // less no longer moves (33 × 30 = 990 < 1,000); 34 moves by 1.
    for (value, after) in [
        (-330, -312),
        (-1000, -961),
        (267, 267),
        (266, 267),
        (270, 270),
        (330, 330),
        (334, 333),
    ] {
        let (mut engine, _) = villages(rules(), [(0, 0), (0, 0)], &residents());
        relations_at(&mut engine, 1);
        set_pair(&mut engine, value, value <= -300);
        relations_at(&mut engine, decision(1));
        assert_eq!(
            engine.resources.settlement_relations.pairs[&(1, 2)].value,
            after,
            "from {value}"
        );
    }
}

// ------------------------------------- 6.7 the integration case

/// One day of daily contact at a bush on the camp's side (31, 30): the
/// decision at 08:00, then at 10:00 a daughter forager (id 11) Idle at
/// (36, 30) with the store at its target (so only the Gather arm can send
/// it) and a camp forager (id 1) heading to the bush. Three decision ticks:
/// choose, arrive, pick up (a dispute when the daughter's forager reached
/// the bush). Returns whether the daughter's forager took the bush.
fn contact_day(engine: &mut SimEngine, day: u64, camp: Entity, daughter: Entity) -> bool {
    relations_at(engine, decision(day));
    let start = tick_at(day, 600);
    engine.resources.current_tick = start;
    engine.resources.food_tiles.insert((31, 30), 3);
    for entity in [camp, daughter] {
        engine
            .world
            .get::<&mut Inventory>(entity)
            .expect("inv")
            .items
            .clear();
        engine
            .world
            .get::<&mut Hunger>(entity)
            .expect("hunger")
            .value = 10.0;
    }
    let goal = stock_target(&engine.world, &engine.resources, 2)
        .expect("target")
        .ceil() as u32;
    let settlement = engine.resources.settlements.get_mut(&2).expect("daughter");
    let held = settlement
        .stockpile
        .get(&ResourceKind::Food)
        .copied()
        .unwrap_or(0);
    settlement.store(ResourceKind::Food, goal.saturating_sub(held));
    *engine.world.get::<&mut Position>(daughter).expect("pos") = Position::new(31, 30);
    engine
        .world
        .insert_one(daughter, AgentState::Idle)
        .expect("idle");
    let _ = engine.world.remove_one::<SeekTarget>(daughter);
    *engine.world.get::<&mut Position>(camp).expect("pos") = Position::new(28, 30);
    engine
        .world
        .insert(
            camp,
            (
                AgentState::Seeking {
                    target: TargetKind::GatherFood,
                },
                SeekTarget { tile: (31, 30) },
            ),
        )
        .expect("camp");
    let mut took = false;
    for offset in 0..3 {
        decide_at(engine, start + offset);
        took |= engine
            .world
            .get::<&SeekTarget>(daughter)
            .is_ok_and(|s| s.tile == (31, 30));
    }
    took
}

#[test]
fn harness_p21_integration_contact_falls_under_avoid_and_the_pair_reconciles() {
    // Brief 6.7: relation −290 with daily contact at a bush on the camp's
    // side. A forced quarrel takes it below −300 (estranged at the next
    // decision). The contact stops under the avoid filter and the pair
    // reconciles (≥ −150) within 40 days (Pass 20's rule, without the
    // filter, kept the forager crossing over and never reconciled).
    let rules = rules();
    let rule = rules.society.relations.expect("rule");
    let (mut engine, entities) = villages(rules, [(0, 0), (0, 0)], &[(1, 1, 500), (11, 2, 500)]);
    let (camp, daughter) = (entities[0], entities[1]);
    for entity in &entities {
        engine
            .world
            .insert_one(
                *entity,
                Temperament {
                    ns: 1000,
                    ha: 0,
                    rd: 500,
                    p: 500,
                },
            )
            .expect("bold");
    }
    relations_at(&mut engine, 1);
    set_pair(&mut engine, -290, false);
    // The forced quarrel: a dispute's and a quarrel's loss.
    state_add(&mut engine, -(rule.quarrel_loss + rule.dispute_loss));
    let mut estranged_on = None;
    let mut contacts_after = 0;
    for day in 3..45 {
        let took = contact_day(&mut engine, day, camp, daughter);
        let state = engine.resources.settlement_relations.pairs[&(1, 2)];
        if state.estranged && estranged_on.is_none() {
            estranged_on = Some(day);
        }
        if estranged_on.is_some_and(|d| day > d) {
            contacts_after += u32::from(took);
        }
    }
    let first = estranged_on.expect("estranged after the forced quarrel");
    assert_eq!(contacts_after, 0, "no contact while estranged");
    let reconciled = of_kind(&engine, ChronicleKind::VillagesReconciled);
    let at = reconciled.first().expect("the pair reconciles").tick;
    assert!(at <= decision(first) + 40 * 1440, "within 40 days: {at}");
}

fn state_add(engine: &mut SimEngine, delta: i32) {
    if let Some(state) = engine.resources.settlement_relations.pairs.get_mut(&(1, 2)) {
        state.add(delta);
    }
}

// ------------------------------------------------- 9 the invariants

#[test]
fn harness_p21_baseline_invariants_and_twin() {
    // Two engines of one baseline world for `DAYS` days: identical at every
    // day; at every tick the ledger respects the caps, the relations exist
    // exactly for living pairs and stay in range; at the end every ended trip
    // balances (withdrawn = delivered + returned + lost) and the tallies
    // match the chronicle.
    //
    // V8-0: world 32 for 36 days instead of `game_v26` world 0 for 60 (15–30
    // CPU-minutes in a debug build). World 32 splits on display day 28 and
    // founds its daughter on day 29 (the earliest split in a scan of seeds
    // 0..48), so the relations rule works on a live village pair for a week.
    const WORLD: u64 = 32;
    const DAYS: u64 = 36;
    let mut a = sim_test::baseline_engine(WORLD);
    let mut b = sim_test::baseline_engine(WORLD);
    let rule = RULE;
    for day in 1..=DAYS {
        for _ in 0..1440 {
            a.tick();
            b.tick();
            let r = &a.resources.settlement_relations;
            for (costed, quarrels) in r.day_counts.values() {
                assert!(
                    *costed <= rule.dispute_cap_per_pair_day
                        && *quarrels <= rule.quarrel_cap_per_pair_day
                );
            }
            let mut ids: Vec<u32> = a.resources.settlements.keys().copied().collect();
            ids.sort_unstable();
            let living: Vec<(u32, u32)> = ids
                .iter()
                .enumerate()
                .flat_map(|(i, x)| ids[i + 1..].iter().map(move |y| (*x, *y)))
                .collect();
            assert_eq!(r.pairs.keys().copied().collect::<Vec<_>>(), living);
            assert!(r.pairs.values().all(|s| (-1000..=1000).contains(&s.value)));
        }
        assert_eq!(
            sim_test::probe::fingerprint(&a),
            sim_test::probe::fingerprint(&b),
            "day {day}"
        );
        assert_eq!(
            a.resources.settlement_relations, b.resources.settlement_relations,
            "day {day}"
        );
        assert_eq!(a.resources.chronicle, b.resources.chronicle, "day {day}");
    }
    let r = &a.resources.settlement_relations;
    for record in &r.trips {
        assert_eq!(
            record.trip.amount,
            record.delivered + record.returned + record.lost
        );
    }
    let t = a.resources.tallies;
    assert_eq!(
        t.porters_departed as usize,
        of_kind(&a, ChronicleKind::PorterDeparted).len()
    );
    assert_eq!(
        t.quarrels as usize,
        of_kind(&a, ChronicleKind::Quarrel).len()
    );
    assert_eq!(
        t.villages_estranged as usize,
        of_kind(&a, ChronicleKind::VillagesEstranged).len()
    );
    assert_eq!(
        t.villages_reconciled as usize,
        of_kind(&a, ChronicleKind::VillagesReconciled).len()
    );
    assert!(t.quarrels_while_estranged <= t.quarrels && t.disputes_uncosted <= t.forage_disputes);
    // The rule acted on the pair (world 32 at day 36, re-measured: 14 daily
    // decision records, 2 porter trips each delivering 10, 27 forage
    // disputes of which 13 uncosted by the daily cap, 3 quarrels and 13
    // capped, the relation at 159 from its base 300).
    assert!(r.days.len() >= 7, "{} decisions", r.days.len());
    assert!(t.porters_departed >= 1 && r.trips.iter().map(|x| x.delivered).sum::<u32>() >= 10);
    assert!(t.forage_disputes >= 10, "{} disputes", t.forage_disputes);
    assert!(
        t.disputes_uncosted >= 1 && t.quarrels_capped >= 1,
        "the caps bind"
    );
    assert!(
        r.pairs[&(1, 2)].value < r.pairs[&(1, 2)].base,
        "losses applied"
    );
    assert!(a.resources.tallies.fissions >= 1, "world {WORLD} splits");
}
