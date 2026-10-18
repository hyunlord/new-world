//! Pass 20 (`game_v25`) — `RelationsSystem` (priority 149, every tick):
//! villages that help and villages that quarrel.
//!
//! **Pairs.** Every tick the relation map (`SimResources::settlement_relations`)
//! holds exactly the unordered pairs of living villages: a missing pair is
//! added at its kin base ([`base_relation`]; a daughter's `parent` is set at
//! priority 148 of its founding tick, before this system), a pair with a
//! missing id is removed.
//!
//! **Daily decision** (at `RelationsRule::decision_minute`): every relation
//! drifts toward its base and only then is estrangement re-evaluated
//! (hysteresis: estranged at ≤ `estrange_at`, reconciled at ≥
//! `reconcile_at`). Then every ordered pair (sender g, receiver r) of
//! non-estranged villages within a porter's reach ([`in_reach`]) is taken in
//! id order — first kin aid (food, else wood), then exchange (food for wood,
//! both trips or none) — at most one trip per ordered pair per day and at most
//! `porter_max` porters per sender per day. The conditions read each village's
//! measures taken once at the decision ([`food_days_permille`],
//! [`wood_nights`]); the amounts read the store as debited by earlier
//! departures, above the keep lines ([`food_keep`], [`wood_keep`]).
//!
//! **Trips** (every tick, porters in AgentId order). A porter carries at most
//! ten units; it walks (`Seeking { Agent(own id) }`, the decision system's
//! porter arms) to the receiver's hearth, where its load goes into the
//! receiver's store, then home. A porter not at the receiver
//! `latest_arrival_minute − decision_minute` ticks after departing, or whose
//! receiver dissolved, turns back with its load (a failed trip). At home the
//! remaining load goes back into the sender's store and the marker is removed.
//! A porter that dies, or whose village dissolves, loses its load. Every trip
//! is booked in a [`TripRecord`]: withdrawn = delivered + returned + lost.
//!
//! The system draws nothing; quarrels (the only draw of the pass) are decided
//! by the decision system at a pickup. A no-op without `SocietyRules::relations`.
use std::collections::{BTreeMap, BTreeSet};

use hecs::{Entity, World};
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::components::{
    Agent, AgentId, AgentState, Child, Elder, Founding, Hunger, Inventory, PairDay,
    PorterKind, PorterStage, PorterTrip, Position, RelationState, SeekTarget, Settlement,
    SettlementId, SettlementRelations, TargetKind, Temperament, TripRecord,
    INVENTORY_CAPACITY, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::rules::RelationsRule;
use sim_engine::{RuntimeSystem, SimResources};

use super::firewood::tonight_burn;
use super::food_economy::members_need_per_day;

/// Departure cause: kin aid, food.
pub const CAUSE_AID_FOOD: u8 = 0;
/// Departure cause: kin aid, wood.
pub const CAUSE_AID_WOOD: u8 = 1;
/// Departure cause: exchange, the food leg.
pub const CAUSE_EXCHANGE_FOOD: u8 = 2;
/// Departure cause: exchange, the wood leg.
pub const CAUSE_EXCHANGE_WOOD: u8 = 3;
/// Added to the departure cause of a `PorterDelivered` entry of a failed trip.
pub const CAUSE_FAILED_OFFSET: u8 = 10;

fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

/// `floor(members_need_per_day × 1000)` of settlement `sid` (its members'
/// daily food need in thousandths of a unit).
pub fn need_milli(world: &World, resources: &SimResources, sid: SettlementId) -> u64 {
    (members_need_per_day(world, resources, sid) * 1000.0).floor().max(0.0) as u64
}

/// Food days in permille of a day: `stock × 1,000,000 / max(1, need_milli)`.
pub fn food_days_permille(stock_food: u32, need_milli: u64) -> u64 {
    u64::from(stock_food) * 1_000_000 / need_milli.max(1)
}

/// The food a sender keeps: `ceil(need_milli × keep_days_permille / 1,000,000)`.
pub fn food_keep(need_milli: u64, keep_days_permille: u32) -> u64 {
    (need_milli * u64::from(keep_days_permille)).div_ceil(1_000_000)
}

/// Wood burned tonight in whole units, at least 1 (`tonight_burn` of the
/// firewood rule; 1 without the rule).
pub fn burn_night(resources: &SimResources) -> u32 {
    resources
        .rules
        .society
        .firewood
        .map_or(1, |rule| (tonight_burn(resources, &rule) as u32).max(1))
}

/// Wood nights: `stock_wood / max(1, burn_night)`.
pub fn wood_nights(stock_wood: u32, burn_night: u32) -> u64 {
    u64::from(stock_wood / burn_night.max(1))
}

/// The wood a sender keeps: `wood_keep_nights × burn_night`.
pub fn wood_keep(rule: &RelationsRule, burn_night: u32) -> u64 {
    u64::from(rule.wood_keep_nights) * u64::from(burn_night)
}

/// The kin base of two villages: `relation_kin_parent` if one is the other's
/// parent, `relation_kin_sibling` if they share a parent, else `relation_other`.
pub fn base_relation(rule: &RelationsRule, a: &Settlement, b: &Settlement) -> i32 {
    if a.parent == Some(b.settlement_id) || b.parent == Some(a.settlement_id) {
        rule.relation_kin_parent
    } else if a.parent.is_some() && a.parent == b.parent {
        rule.relation_kin_sibling
    } else {
        rule.relation_other
    }
}

/// Whether a porter reaches hearth `b` from hearth `a` in time: the walk
/// (Chebyshev distance × `move_interval_ticks`) is at most three quarters of
/// the window from the decision to the latest arrival (ticks are minutes).
pub fn in_reach(rule: &RelationsRule, move_interval_ticks: u64, a: (u32, u32), b: (u32, u32)) -> bool {
    let window = u64::from(rule.latest_arrival_minute.saturating_sub(rule.decision_minute));
    u64::from(chebyshev(a, b)) * move_interval_ticks.max(1) <= window * 3 / 4
}

/// One village's measures at the decision minute.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Measure {
    hearth: (u32, u32),
    food: u32,
    food_days: u64,
    food_keep: u64,
    wood: u32,
    wood_nights: u64,
    wood_keep: u64,
}

impl Measure {
    fn row(&self) -> [u64; 4] {
        [self.food_days, u64::from(self.food), self.wood_nights, u64::from(self.wood)]
    }
}

/// The trip in progress and its chronicle cause.
struct Departure {
    from: SettlementId,
    to: SettlementId,
    kind: PorterKind,
    amount: u32,
    cause: u8,
}

/// Pass 20 relations system (see the module docs).
#[derive(Debug, Default)]
pub struct RelationsSystem;

impl RelationsSystem {
    /// Construct the system.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for RelationsSystem {
    fn name(&self) -> &str {
        "RelationsSystem"
    }

    fn priority(&self) -> u32 {
        149
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.society.relations else {
            return;
        };
        let tick = resources.current_tick;
        sync_pairs(resources, &rule);
        progress(world, resources, &rule, tick);
        let minute = (resources.ticks_per_day != 0).then(|| {
            tick.saturating_add(resources.rules.clock_offset_ticks) % resources.ticks_per_day
        });
        if minute == Some(u64::from(rule.decision_minute)) {
            decide(world, resources, &rule, tick);
        }
    }
}

/// Keep exactly the pairs of living villages (missing pairs at their base).
fn sync_pairs(resources: &mut SimResources, rule: &RelationsRule) {
    let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
    ids.sort_unstable();
    let SimResources { settlements, settlement_relations, .. } = resources;
    settlement_relations
        .pairs
        .retain(|(a, b), _| ids.binary_search(a).is_ok() && ids.binary_search(b).is_ok());
    for (index, a) in ids.iter().enumerate() {
        for b in &ids[index + 1..] {
            if settlement_relations.pairs.contains_key(&(*a, *b)) {
                continue;
            }
            let (Some(sa), Some(sb)) = (settlements.get(a), settlements.get(b)) else {
                continue;
            };
            let base = base_relation(rule, sa, sb);
            settlement_relations
                .pairs
                .insert((*a, *b), RelationState { value: base, estranged: false, base });
        }
    }
}

/// The units of a trip not yet delivered, returned or lost.
const fn remaining(record: &TripRecord) -> u32 {
    record
        .trip
        .amount
        .saturating_sub(record.delivered)
        .saturating_sub(record.returned)
        .saturating_sub(record.lost)
}

/// Close `record` at `tick` and move it to the trip log.
fn close(resources: &mut SimResources, porter: AgentId, mut record: TripRecord, tick: u64) {
    record.end_tick = Some(tick);
    resources.settlement_relations.active.remove(&porter);
    resources.settlement_relations.trips.push(record);
}

/// Book the remaining load of `record` as lost.
fn book_lost(resources: &mut SimResources, record: &mut TripRecord) {
    let lost = remaining(record);
    record.lost += lost;
    resources.tallies.porter_units_lost += u64::from(lost);
    resources.tallies.porter_loads_lost += u64::from(lost > 0);
}

/// Whether `state` is a porter's self-targeted walk.
fn walking(state: Option<AgentState>, id: AgentId) -> bool {
    matches!(state, Some(AgentState::Seeking { target: TargetKind::Agent(target) }) if target == id)
}

/// End a porter's trip on its entity: the marker goes and a porter still on
/// its walk becomes Idle without a target.
fn release(world: &mut World, entity: Entity, id: AgentId) {
    let _ = world.remove_one::<PorterTrip>(entity);
    if walking(world.get::<&AgentState>(entity).ok().map(|s| *s), id) {
        let _ = world.insert_one(entity, AgentState::Idle);
        let _ = world.remove_one::<SeekTarget>(entity);
    }
}

/// Point a walking porter at `tile`.
fn head_for(world: &mut World, entity: Entity, id: AgentId, tile: (u32, u32)) {
    if walking(world.get::<&AgentState>(entity).ok().map(|s| *s), id) {
        let _ = world.insert_one(entity, SeekTarget { tile });
    }
}

/// Remove up to `n` units of `kind` from the porter's inventory.
fn unload(world: &World, entity: Entity, kind: PorterKind, n: u32) -> u32 {
    world
        .get::<&mut Inventory>(entity)
        .map_or(0, |mut inventory| inventory.remove(kind.resource(), n))
}

/// Walk every trip forward one tick: dead porters, deliveries, turn-backs,
/// homecomings and dissolved villages.
fn progress(world: &mut World, resources: &mut SimResources, rule: &RelationsRule, tick: u64) {
    if resources.settlement_relations.active.is_empty() {
        return;
    }
    let mut rows: Vec<(AgentId, Entity, (u32, u32), PorterTrip)> = world
        .query::<(&Agent, &Position, &PorterTrip)>()
        .iter()
        .map(|(entity, (agent, pos, trip))| (agent.id, entity, (pos.x, pos.y), *trip))
        .collect();
    rows.sort_by_key(|row| row.0);
    // A porter that died lost its load with it.
    let living: BTreeSet<AgentId> = rows.iter().map(|row| row.0).collect();
    let dead: Vec<AgentId> = resources
        .settlement_relations
        .active
        .keys()
        .copied()
        .filter(|id| !living.contains(id))
        .collect();
    for id in dead {
        if let Some(mut record) = resources.settlement_relations.active.get(&id).copied() {
            book_lost(resources, &mut record);
            close(resources, id, record, tick);
        }
    }
    let deadline = u64::from(rule.latest_arrival_minute.saturating_sub(rule.decision_minute));
    for (id, entity, pos, trip) in rows {
        let Some(mut record) = resources.settlement_relations.active.get(&id).copied() else {
            // A marker without a booking cannot carry anything; drop it.
            release(world, entity, id);
            continue;
        };
        let home = resources.settlements.get(&trip.from).map(|s| s.formation_tile);
        let Some(home) = home else {
            // The sender dissolved: the load is lost.
            let left = remaining(&record);
            unload(world, entity, trip.kind, left);
            book_lost(resources, &mut record);
            release(world, entity, id);
            close(resources, id, record, tick);
            continue;
        };
        match trip.stage {
            PorterStage::Outbound => {
                let receiver = resources.settlements.get(&trip.to).map(|s| s.formation_tile);
                if receiver.is_some_and(|hearth| chebyshev(pos, hearth) <= SETTLEMENT_PROXIMITY_RADIUS) {
                    let load = unload(world, entity, trip.kind, remaining(&record));
                    if let Some(settlement) = resources.settlements.get_mut(&trip.to) {
                        settlement.store(trip.kind.resource(), load);
                    }
                    record.delivered += load;
                    resources.tallies.porter_deliveries += 1;
                    match trip.kind {
                        PorterKind::Food => resources.tallies.porter_food_delivered += u64::from(load),
                        PorterKind::Wood => resources.tallies.porter_wood_delivered += u64::from(load),
                    }
                    // The first porter of a delivered trip credits the pair.
                    if resources
                        .settlement_relations
                        .credited
                        .insert((trip.from, trip.to, trip.depart_tick))
                    {
                        if let Some(state) = resources
                            .settlement_relations
                            .pairs
                            .get_mut(&SettlementRelations::key(trip.from, trip.to))
                        {
                            state.add(rule.delivery_gain);
                        }
                    }
                    turn_home(world, resources, entity, id, trip, record, home);
                    crate::runtime::chronicle_log::record(
                        world,
                        resources,
                        ChronicleEntry::new(tick, ChronicleKind::PorterDelivered)
                            .subject(id)
                            .settlement(Some(trip.to))
                            .settlement2(Some(trip.from))
                            .cause(record.cause)
                            .value(i64::from(load)),
                    );
                } else if receiver.is_none() || tick.saturating_sub(trip.depart_tick) >= deadline {
                    record.failed = true;
                    resources.tallies.porter_turnbacks += 1;
                    turn_home(world, resources, entity, id, trip, record, home);
                    crate::runtime::chronicle_log::record(
                        world,
                        resources,
                        ChronicleEntry::new(tick, ChronicleKind::PorterDelivered)
                            .subject(id)
                            .settlement(Some(trip.to))
                            .settlement2(Some(trip.from))
                            .cause(CAUSE_FAILED_OFFSET + record.cause)
                            .value(0),
                    );
                }
            }
            PorterStage::Return => {
                if chebyshev(pos, home) <= SETTLEMENT_PROXIMITY_RADIUS {
                    let left = remaining(&record);
                    let back = unload(world, entity, trip.kind, left);
                    if let Some(settlement) = resources.settlements.get_mut(&trip.from) {
                        settlement.store(trip.kind.resource(), back);
                    }
                    record.returned += back;
                    resources.tallies.porter_units_returned += u64::from(back);
                    // A shortfall (nothing removes a porter's load today) is lost.
                    book_lost(resources, &mut record);
                    release(world, entity, id);
                    close(resources, id, record, tick);
                }
            }
        }
    }
}

/// Send a porter home: stage `Return`, its target the sender's hearth.
fn turn_home(
    world: &mut World,
    resources: &mut SimResources,
    entity: Entity,
    id: AgentId,
    trip: PorterTrip,
    record: TripRecord,
    home: (u32, u32),
) {
    let back = PorterTrip { stage: PorterStage::Return, ..trip };
    let _ = world.insert_one(entity, back);
    head_for(world, entity, id, home);
    resources
        .settlement_relations
        .active
        .insert(id, TripRecord { trip: back, ..record });
}

/// Drift, estrangement and the day's trips.
fn decide(world: &mut World, resources: &mut SimResources, rule: &RelationsRule, tick: u64) {
    let mut events: Vec<ChronicleEntry> = Vec::new();
    // Drift toward the base, then the hysteresis (only here).
    for ((a, b), state) in resources.settlement_relations.pairs.iter_mut() {
        state.add((state.base - state.value).saturating_mul(rule.relation_drift_permille) / 1000);
        if !state.estranged && state.value <= rule.estrange_at {
            state.estranged = true;
            resources.tallies.villages_estranged += 1;
            events.push(
                ChronicleEntry::new(tick, ChronicleKind::VillagesEstranged)
                    .settlement(Some(*a))
                    .settlement2(Some(*b))
                    .value(i64::from(state.value)),
            );
        } else if state.estranged && state.value >= rule.reconcile_at {
            state.estranged = false;
            resources.tallies.villages_reconciled += 1;
            events.push(
                ChronicleEntry::new(tick, ChronicleKind::VillagesReconciled)
                    .settlement(Some(*a))
                    .settlement2(Some(*b))
                    .value(i64::from(state.value)),
            );
        }
    }
    // Credits older than two days can no longer be claimed by a porter.
    let horizon = tick.saturating_sub(2 * resources.ticks_per_day.max(1));
    resources.settlement_relations.credited.retain(|(_, _, depart)| *depart >= horizon);
    for entry in events {
        resources.chronicle.push(entry);
    }
    let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
    ids.sort_unstable();
    if ids.len() < 2 {
        return;
    }
    let burn = burn_night(resources);
    let mut measures: BTreeMap<SettlementId, Measure> = BTreeMap::new();
    for sid in &ids {
        let Some(settlement) = resources.settlements.get(sid) else { continue };
        let need = need_milli(world, resources, *sid);
        let food = settlement.stockpile.get(&sim_core::components::ResourceKind::Food).copied().unwrap_or(0);
        let wood = settlement.stockpile.get(&sim_core::components::ResourceKind::Wood).copied().unwrap_or(0);
        measures.insert(
            *sid,
            Measure {
                hearth: settlement.formation_tile,
                food,
                food_days: food_days_permille(food, need),
                food_keep: food_keep(need, rule.aid_keep_days_permille),
                wood,
                wood_nights: wood_nights(wood, burn),
                wood_keep: wood_keep(rule, burn),
            },
        );
    }
    let mut candidates = porter_candidates(world, resources, rule, &ids);
    let interval = resources.rules.agent.move_interval_ticks;
    // Every ordered pair's conditions, from the measures above.
    let mut rows: BTreeMap<(SettlementId, SettlementId), PairDay> = BTreeMap::new();
    for g in &ids {
        for r in &ids {
            let (Some(mg), Some(mr)) = (measures.get(g), measures.get(r)) else { continue };
            let Some(state) = resources.settlement_relations.get(*g, *r).copied().filter(|_| g != r) else {
                continue;
            };
            rows.insert((*g, *r), PairDay {
                tick,
                from: *g,
                to: *r,
                value: state.value,
                estranged: state.estranged,
                in_reach: in_reach(rule, interval, mg.hearth, mr.hearth),
                from_measure: mg.row(),
                to_measure: mr.row(),
                aid_food: mg.food_days >= u64::from(rule.aid_giver_days_permille)
                    && mr.food_days < u64::from(rule.aid_receiver_days_permille),
                aid_wood: mg.wood_nights >= u64::from(rule.wood_giver_nights)
                    && mr.wood_nights < u64::from(rule.wood_receiver_nights),
                exchange: mg.food_days >= u64::from(rule.trade_food_surplus_days_permille)
                    && mg.wood_nights < u64::from(rule.trade_wood_short_nights)
                    && mr.wood_nights >= u64::from(rule.trade_wood_surplus_nights)
                    && mr.food_days < u64::from(rule.trade_food_short_days_permille),
                sent: None,
            });
        }
    }
    // The trips: ordered pairs in id order, aid first, then exchange.
    let mut used: BTreeSet<(SettlementId, SettlementId)> = BTreeSet::new();
    let mut porters_used: BTreeMap<SettlementId, u32> = BTreeMap::new();
    let mut entries: Vec<ChronicleEntry> = Vec::new();
    let pairs: Vec<PairDay> = rows.values().copied().collect();
    for row in pairs {
        let (g, r) = (row.from, row.to);
        if row.estranged || !row.in_reach || used.contains(&(g, r)) {
            continue;
        }
        // a. Kin aid: food, else wood.
        let aid = if row.aid_food {
            Some((PorterKind::Food, CAUSE_AID_FOOD))
        } else if row.aid_wood {
            Some((PorterKind::Wood, CAUSE_AID_WOOD))
        } else {
            None
        };
        if let Some((kind, cause)) = aid {
            let amount = trip_amount(resources, rule, &measures, &candidates, &porters_used, g, kind);
            if amount > 0 {
                used.insert((g, r));
                let sent = depart(world, resources, tick, Departure { from: g, to: r, kind, amount, cause }, &mut candidates, &mut porters_used, &mut entries);
                if let Some(day) = rows.get_mut(&(g, r)) {
                    day.sent = Some((kind, sent));
                }
                continue;
            }
        }
        // b. Exchange: g's food for r's wood, both trips or none.
        if row.exchange && !used.contains(&(r, g)) {
            let food = trip_amount(resources, rule, &measures, &candidates, &porters_used, g, PorterKind::Food);
            let wood = trip_amount(resources, rule, &measures, &candidates, &porters_used, r, PorterKind::Wood);
            if food > 0 && wood > 0 {
                used.insert((g, r));
                used.insert((r, g));
                let food_leg = Departure { from: g, to: r, kind: PorterKind::Food, amount: food, cause: CAUSE_EXCHANGE_FOOD };
                let food = depart(world, resources, tick, food_leg, &mut candidates, &mut porters_used, &mut entries);
                let wood_leg = Departure { from: r, to: g, kind: PorterKind::Wood, amount: wood, cause: CAUSE_EXCHANGE_WOOD };
                let wood = depart(world, resources, tick, wood_leg, &mut candidates, &mut porters_used, &mut entries);
                if let Some(day) = rows.get_mut(&(g, r)) {
                    day.sent = Some((PorterKind::Food, food));
                }
                if let Some(day) = rows.get_mut(&(r, g)) {
                    day.sent = Some((PorterKind::Wood, wood));
                }
            }
        }
    }
    resources.settlement_relations.days.extend(rows.into_values());
    for entry in entries {
        crate::runtime::chronicle_log::record(world, resources, entry);
    }
}

/// Members able to carry, per village: adults (no `Child`, no `Elder`)
/// without `Founding` or `PorterTrip`, empty-handed, hunger below
/// `porter_hunger_max`, Idle or in an activity; RD descending, then AgentId.
fn porter_candidates(
    world: &World,
    resources: &SimResources,
    rule: &RelationsRule,
    ids: &[SettlementId],
) -> BTreeMap<SettlementId, Vec<(AgentId, Entity)>> {
    let mut able: BTreeMap<AgentId, (Entity, u16)> = BTreeMap::new();
    for (entity, (agent, state, inventory, hunger, temperament, child, elder, founding, porter)) in world
        .query::<(
            &Agent,
            &AgentState,
            &Inventory,
            Option<&Hunger>,
            Option<&Temperament>,
            Option<&Child>,
            Option<&Elder>,
            Option<&Founding>,
            Option<&PorterTrip>,
        )>()
        .iter()
    {
        let free = matches!(
            state,
            AgentState::Idle
                | AgentState::Seeking { target: TargetKind::Activity }
                | AgentState::Consuming { target: TargetKind::Activity }
        );
        if free
            && child.is_none()
            && elder.is_none()
            && founding.is_none()
            && porter.is_none()
            && inventory.total() == 0
            && hunger.is_none_or(|h| h.value < rule.porter_hunger_max as f32)
        {
            able.insert(agent.id, (entity, temperament.map_or(500, |t| t.rd)));
        }
    }
    let mut out = BTreeMap::new();
    for sid in ids {
        let Some(settlement) = resources.settlements.get(sid) else { continue };
        let mut members: Vec<(u16, AgentId, Entity)> = settlement
            .member_agents
            .iter()
            .filter_map(|id| able.get(id).map(|(entity, rd)| (*rd, *id, *entity)))
            .collect();
        members.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        out.insert(*sid, members.into_iter().map(|(_, id, entity)| (id, entity)).collect());
    }
    out
}

/// The amount sender `g` can send of `kind` now: min(10 × remaining porter
/// slots, the store above its keep line, 10 × remaining candidates).
fn trip_amount(
    resources: &SimResources,
    rule: &RelationsRule,
    measures: &BTreeMap<SettlementId, Measure>,
    candidates: &BTreeMap<SettlementId, Vec<(AgentId, Entity)>>,
    porters_used: &BTreeMap<SettlementId, u32>,
    g: SettlementId,
    kind: PorterKind,
) -> u32 {
    let (Some(measure), Some(settlement)) = (measures.get(&g), resources.settlements.get(&g)) else {
        return 0;
    };
    let held = u64::from(settlement.stockpile.get(&kind.resource()).copied().unwrap_or(0));
    let keep = match kind {
        PorterKind::Food => measure.food_keep,
        PorterKind::Wood => measure.wood_keep,
    };
    let surplus = held.saturating_sub(keep);
    let slots = u64::from(rule.porter_max.saturating_sub(porters_used.get(&g).copied().unwrap_or(0)));
    let able = candidates.get(&g).map_or(0, Vec::len) as u64;
    let cap = u64::from(INVENTORY_CAPACITY);
    u32::try_from(surplus.min(cap * slots).min(cap * able)).unwrap_or(0)
}

/// Send `departure.amount` from the sender's store with the next candidates;
/// returns the units carried.
fn depart(
    world: &mut World,
    resources: &mut SimResources,
    tick: u64,
    departure: Departure,
    candidates: &mut BTreeMap<SettlementId, Vec<(AgentId, Entity)>>,
    porters_used: &mut BTreeMap<SettlementId, u32>,
    entries: &mut Vec<ChronicleEntry>,
) -> u32 {
    let cap = INVENTORY_CAPACITY;
    let count = departure.amount.div_ceil(cap) as usize;
    let Some(pool) = candidates.get_mut(&departure.from) else { return 0 };
    let take = count.min(pool.len());
    let mut chosen: Vec<(AgentId, Entity)> = pool.drain(..take).collect();
    chosen.sort_by_key(|row| row.0);
    *porters_used.entry(departure.from).or_insert(0) += take as u32;
    let mut left = departure.amount;
    let mut sent = 0;
    resources.tallies.porter_trips += 1;
    for (id, entity) in chosen {
        let load = left.min(cap);
        left -= load;
        let withdrawn = resources
            .settlements
            .get_mut(&departure.from)
            .map_or(0, |s| s.withdraw(departure.kind.resource(), load));
        let overflow = world
            .get::<&mut Inventory>(entity)
            .map_or(withdrawn, |mut inventory| inventory.add(departure.kind.resource(), withdrawn));
        // Inventories are empty and loads ≤ capacity: nothing overflows.
        if overflow > 0 {
            if let Some(settlement) = resources.settlements.get_mut(&departure.from) {
                settlement.store(departure.kind.resource(), overflow);
            }
        }
        let carried = withdrawn - overflow;
        let trip = PorterTrip {
            from: departure.from,
            to: departure.to,
            kind: departure.kind,
            amount: carried,
            stage: PorterStage::Outbound,
            depart_tick: tick,
        };
        let _ = world.insert_one(entity, trip);
        // An activity holder is taken: Idle, no target; the routine records
        // the activity ended as interrupted on the next tick.
        let _ = world.insert_one(entity, AgentState::Idle);
        let _ = world.remove_one::<SeekTarget>(entity);
        resources.tallies.porters_departed += 1;
        sent += carried;
        resources.tallies.porter_units_withdrawn += u64::from(carried);
        resources.settlement_relations.active.insert(
            id,
            TripRecord {
                porter: id,
                trip,
                cause: departure.cause,
                delivered: 0,
                returned: 0,
                lost: 0,
                failed: false,
                end_tick: None,
            },
        );
        entries.push(
            ChronicleEntry::new(tick, ChronicleKind::PorterDeparted)
                .subject(id)
                .settlement(Some(departure.from))
                .settlement2(Some(departure.to))
                .cause(departure.cause)
                .value(i64::from(carried)),
        );
    }
    sent
}
