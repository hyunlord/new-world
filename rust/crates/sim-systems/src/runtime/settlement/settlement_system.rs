//! V7 Phase 10-β — `SettlementSystem` (priority 138, every tick).
//!
//! Sole runtime owner of [`Settlement`] formation, sync, growth, and
//! dissolution. Runs strictly after [`CombatSystem`](crate::runtime::combat::CombatSystem)
//! (priority 137) so deaths from this tick are reflected in proximity
//! membership BEFORE the formation/dissolution check runs.
//!
//! # Tick pipeline (in order)
//!
//! 1. **Pre-clearing population snapshot** — for every healthy settlement
//!    (`current > 0` AND `!member_buildings.is_empty()`), stamp the
//!    population into `last_known_population`. Used at dissolution time so
//!    `SettlementDissolved.final_population` carries the pre-clearing
//!    population (plan A4) instead of the trivial zero captured at the
//!    dissolution tick.
//!
//! 2. **Snapshot building positions** — pulled from
//!    `SimResources::building_registry`. The registry is authoritative;
//!    the causal log alone cannot serve this purpose (same-tile
//!    `StampDirty` / `InfluenceChanged` evict `BuildingPlaced` within a
//!    single tick under normal stamp + propagation load).
//!
//! 3. **Snapshot agent positions** — single hecs query over `(Agent,
//!    Position)`. Used for both formation scan and membership sync.
//!
//! 4. **Membership sync** — for every existing settlement, refresh
//!    `member_agents` and `member_buildings` from proximity to the
//!    settlement's formation tile. Update `population_stats.current`.
//!
//! 5. **Formation scan** — find unregistered clusters meeting both
//!    formation thresholds. Skip candidates inside any existing
//!    settlement's proximity bubble. Mint a `SettlementId`, populate
//!    members, emit `CausalEvent::SettlementFormed` with `founding_members`
//!    sorted ascending.
//!
//! 6. **Community history ingestion** — append routed events to each
//!    settlement's `community_history`:
//!      * `BuildingPlaced` whose position is inside settlement boundary;
//!      * `CombatCompleted` involving a member;
//!      * `AgentDecision { SettlementReason }` whose target_agent is a
//!        member of this settlement.
//!
//!    `AgentBorn` is appended directly at birth-emit time (step 7).
//!
//! 7. **Birth trigger** — for each settlement that (a) was founded at
//!    least `BIRTH_COOLDOWN_TICKS` ago, (b) has `population_stats.current
//!    < SETTLEMENT_MAX_POP`, spawn a fresh agent at the lowest-id
//!    member's position. Emit `CausalEvent::AgentBorn` parented to the
//!    settlement's `SettlementFormed` event id. Append the AgentBorn id
//!    to the settlement's community history.
//!
//! 8. **Dissolution check** — settlements satisfying BOTH
//!    `population_stats.current == 0` AND `member_buildings.is_empty()`
//!    are removed. Emit `CausalEvent::SettlementDissolved` parented to
//!    the formation event id. Clear ALL local state for the dissolved
//!    settlement (formation_tile, formation_event_id, birth_last_tick,
//!    last_known_population) so the same area can be re-formed later
//!    with a fresh `SettlementId` (plan A10 forbids id reuse but does
//!    not forbid re-formation in the same area).

use std::collections::{HashMap, HashSet};

use hecs::World;
use sim_core::causal::event::{CausalEvent, DecisionReason, DissolutionCause, EventId};
use sim_core::components::{
    Agent, AgentId, AgentState, BodyHealth, BuildingId, BuildingKind, Hunger, Inventory, Memory, Position,
    ResourceKind,
    Settlement, SettlementId, Sleep, Social, Thirst, SETTLEMENT_FORMATION_AGENT_THRESHOLD,
    SETTLEMENT_FORMATION_BUILDING_THRESHOLD, SETTLEMENT_MAX_POP, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::rules::BirthRule;
use sim_engine::{RuntimeSystem, SimResources};

use crate::runtime::agent::MovementRng;

/// `(BuildingId, (x, y))` pair — one entry per known building. Used by the
/// formation scan + membership sync as a flat, copy-cheap view of
/// `SimResources::building_registry`.
type BuildingPos = (BuildingId, (u32, u32));

/// Flat list of `(BuildingId, position)` snapshots produced once per tick.
type BuildingPosList = Vec<BuildingPos>;

/// Pair of `(all_buildings, newly_added_buildings)` returned by
/// [`SettlementSystem::snapshot_buildings`].
type BuildingSnapshot = (BuildingPosList, BuildingPosList);

/// Number of ticks a settlement must exist before another birth is
/// considered. Locked by the Phase 10-β plan (P10Plan-5). One reproductive
/// "season" per ~200-tick window keeps the population from ballooning in
/// the harness window while still exercising the birth path multiple
/// times in a 1000-tick smoke run.
pub const BIRTH_COOLDOWN_TICKS: u64 = 200;

// V7 Stage 1.5 — newborn need growth rates + movement seed. Mirror the
// bootstrap rates (BOOTSTRAP_HUNGER_RATE etc. in sim-bridge world_node.rs);
// duplicated as local consts because that crate depends on sim-systems and
// cannot be imported here. Keep in sync with the bootstrap rates.
const BIRTH_HUNGER_RATE: f32 = 0.05;
const BIRTH_THIRST_RATE: f64 = 0.08;
const BIRTH_SLEEP_RATE: f64 = 0.03;
const BIRTH_SOCIAL_RATE: f64 = 0.04;
/// Salt so a newborn's Brownian stream does not alias a bootstrap agent's.
const BIRTH_RNG_SALT: u64 = 0xB117_0000_5EED_0001;

/// Chebyshev distance between two tile coordinates.
fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    dx.max(dy)
}

/// Manhattan distance between two tile coordinates. Used by the membership
/// exclusivity pass to pick the single owning settlement for an agent that
/// sits inside two overlapping proximity bubbles (belonging-model reform).
fn manhattan(a: (u32, u32), b: (u32, u32)) -> u64 {
    a.0.abs_diff(b.0) as u64 + a.1.abs_diff(b.1) as u64
}

/// Phase 10-β settlement runtime system.
///
/// Holds per-settlement local state (formation tile + formation event id +
/// last-birth tick + pre-clearing population snapshot) that is not part of
/// the canonical [`Settlement`] component. This keeps the Phase 10-α data
/// struct stable while still allowing the system to recompute proximity
/// and gate births deterministically.
#[derive(Debug, Default)]
pub struct SettlementSystem {
    /// Formation tile per settlement — used as the proximity anchor for
    /// membership sync and community-history routing.
    formation_tiles: HashMap<SettlementId, (u32, u32)>,
    /// Formation `EventId` per settlement — parent of
    /// `SettlementDissolved` and `AgentBorn` events.
    formation_event_ids: HashMap<SettlementId, EventId>,
    /// Tick at which the last `AgentBorn` fired for each settlement.
    /// Initialised to `founded_at` so the first birth requires
    /// `BIRTH_COOLDOWN_TICKS` after formation.
    birth_last_tick: HashMap<SettlementId, u64>,
    /// Snapshot of `building_registry` keys seen at the previous tick.
    /// Used to compute the set of buildings newly added this tick for
    /// community-history routing. The causal log alone cannot serve this
    /// purpose because the per-tile ring buffer evicts `BuildingPlaced`
    /// events within a single tick under normal stamp + influence
    /// propagation load.
    seen_building_ids: HashSet<BuildingId>,
    /// Per-settlement pre-clearing population snapshot — the population
    /// captured at the LAST tick where both `current > 0` AND
    /// `!member_buildings.is_empty()` held. Used at dissolution to
    /// populate `SettlementDissolved.final_population` (plan A4).
    last_known_population: HashMap<SettlementId, u32>,
    /// Last observed member id per settlement (lowest-id member at the
    /// snapshot moment). Captured at the same snapshot point as
    /// `last_known_population` and used to populate
    /// `SettlementDissolved.last_member_id` (V7 Phase 10-γ / P10γ-A17).
    last_known_member_id: HashMap<SettlementId, AgentId>,
    /// Last observed building id per settlement (lowest-id building at
    /// the snapshot moment). Captured at the same snapshot point as
    /// `last_known_population` and used to populate
    /// `SettlementDissolved.last_building_id` (V7 Phase 10-γ / P10γ-A17).
    last_known_building_id: HashMap<SettlementId, BuildingId>,
    /// Monotonic side-effect counter incremented every tick the system
    /// runs (plan A28). Observable via [`Self::tick_run_count`].
    tick_run_count: u64,
    /// Settlement→event-id pairs already routed to community history
    /// during the current tick. Prevents double-append within a single
    /// tick (plan A21 1:1 correspondence).
    routed_this_tick: HashSet<(SettlementId, EventId)>,
    /// Settlements minted by `run_formation_scan` during the current tick
    /// (V7 Phase 10-γ / P10γ-A6). Stored as `(settlement_id, formed_id)`
    /// pairs so the formation event can be seeded into community_history
    /// AFTER `ingest_community_history` runs, making the SettlementFormed
    /// entry the most-recent entry at formation tick.
    newly_formed_this_tick: Vec<(SettlementId, EventId)>,
}

impl SettlementSystem {
    /// Construct a fresh instance with empty local state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Observable per-tick execution side-effect (plan A28). Increments
    /// each time [`RuntimeSystem::tick`] runs.
    pub fn tick_run_count(&self) -> u64 {
        self.tick_run_count
    }

    /// Step 1: snapshot the pre-clearing population for every healthy
    /// settlement so `SettlementDissolved.final_population` can carry
    /// the last non-zero value (plan A4).
    fn snapshot_pre_clearing_population(&mut self, resources: &SimResources) {
        for (sid, settlement) in resources.settlements.iter() {
            if settlement.population_stats.current > 0
                && !settlement.member_buildings.is_empty()
            {
                self.last_known_population
                    .insert(*sid, settlement.population_stats.current);
                if let Some(min_member) = settlement.member_agents.iter().min().copied() {
                    self.last_known_member_id.insert(*sid, min_member);
                }
                if let Some(min_building) = settlement.member_buildings.iter().min().copied() {
                    self.last_known_building_id.insert(*sid, min_building);
                }
            }
        }
    }

    /// Step 2: snapshot building positions from the authoritative registry.
    /// Returns the list of `(BuildingId, (x, y))` pairs and the set of
    /// newly-added `BuildingId`s vs the previous tick (used for history
    /// routing in step 6).
    fn snapshot_buildings(&mut self, resources: &SimResources) -> BuildingSnapshot {
        let buildings: BuildingPosList = resources
            .building_registry
            .iter()
            .map(|(id, pos)| (*id, *pos))
            .collect();
        let mut new_buildings: BuildingPosList = buildings
            .iter()
            .filter(|(bid, _)| !self.seen_building_ids.contains(bid))
            .copied()
            .collect();
        // Pass 9: under keyed draws new buildings are routed in BuildingId order
        // (HashMap order otherwise; history records only).
        if resources.rules.agent.keyed_draws {
            new_buildings.sort_by_key(|(bid, _)| *bid);
        }
        self.seen_building_ids = resources.building_registry.keys().copied().collect();
        (buildings, new_buildings)
    }

    /// Step 3: snapshot agent positions from the ECS world.
    fn snapshot_agents(world: &World) -> HashMap<AgentId, (u32, u32)> {
        world
            .query::<(&Agent, &Position)>()
            .iter()
            .map(|(_, (a, p))| (a.id, (p.x, p.y)))
            .collect()
    }

    /// Step 4: refresh `member_agents` + `member_buildings` for each settlement
    /// under the BELONGING MODEL (membership-reform).
    ///
    /// Membership is no longer "currently standing in the proximity bubble".
    /// Per settlement we:
    ///   * DROP only DEAD members — `retain` ids still resolvable to a live
    ///     `(Agent, Position)` (an id absent from `agent_positions` has
    ///     despawned). A far-but-LIVE member is KEPT, so a forager taken away
    ///     from the settlement (Direction-2 carry/store) no longer collapses
    ///     the roster (the 2-2 Gather 75→0 bug).
    ///   * ADD in-radius live agents, deterministically (sorted by `AgentId`)
    ///     and respecting `SETTLEMENT_MAX_POP` so a join cannot push the roster
    ///     past the cap. The sort makes the capped admission set
    ///     iteration-order-independent.
    ///
    /// Leaving the radius is NOT a leave path — only death (here, via the
    /// retain step) or migration (the exclusivity pass) removes a member.
    fn sync_membership(
        &self,
        resources: &mut SimResources,
        agent_positions: &HashMap<AgentId, (u32, u32)>,
        buildings: &[BuildingPos],
        admitted: &mut Vec<(AgentId, SettlementId)>,
    ) {
        let live_building_ids: HashSet<BuildingId> =
            buildings.iter().map(|(id, _)| *id).collect();
        let cap = SETTLEMENT_MAX_POP as usize;
        // Pass 11 (`game_v17`, under `FissionRule`): membership is sticky — the
        // proximity sync admits only agents that belong to no settlement, so a
        // member foraging near another village's hearth stays home (Part 0
        // audit g8). Without the rule the set is empty and nothing changes.
        let settled: HashSet<AgentId> = if resources.rules.society.fission.is_some() {
            resources.settlements.values().flat_map(|s| s.member_agents.iter().copied()).collect()
        } else {
            HashSet::new()
        };
        for (id, settlement) in resources.settlements.iter_mut() {
            let formation_tile = match self.formation_tiles.get(id).copied() {
                Some(t) => t,
                None => continue,
            };

            // DROP only DEAD members (retain ids resolvable to a live Position).
            settlement
                .member_agents
                .retain(|aid| agent_positions.contains_key(aid));

            // ADD in-radius live agents — deterministic (sorted) + cap-respecting.
            let mut in_radius: Vec<AgentId> = agent_positions
                .iter()
                .filter(|(aid, pos)| {
                    chebyshev(formation_tile, **pos) <= SETTLEMENT_PROXIMITY_RADIUS
                        && (!settled.contains(aid) || settlement.member_agents.contains(aid))
                })
                .map(|(aid, _)| *aid)
                .collect();
            in_radius.sort_unstable();
            for aid in in_radius {
                if settlement.member_agents.len() >= cap {
                    break;
                }
                // Pass 19: actual admissions are noted for the chronicle diff.
                if settlement.member_agents.insert(aid) {
                    admitted.push((aid, *id));
                }
            }

            // Prune stale member_buildings; admit nearby live ones.
            settlement
                .member_buildings
                .retain(|bid| live_building_ids.contains(bid));
            for (bid, bpos) in buildings {
                if chebyshev(formation_tile, *bpos) <= SETTLEMENT_PROXIMITY_RADIUS {
                    settlement.member_buildings.insert(*bid);
                }
            }

            settlement.population_stats.current = settlement.member_agents.len() as u32;
        }
    }

    /// Membership exclusivity (belonging-model reform): an agent belongs to AT
    /// MOST ONE settlement. After the proximity sync + formation scan, an agent
    /// listed in two or more rosters (because it sits inside overlapping
    /// bubbles, or a founder was already a member elsewhere) is kept ONLY in the
    /// settlement whose `formation_tile` is Manhattan-nearest to the agent's
    /// current position, tie-broken by the LOWER `SettlementId` — never by
    /// HashMap iteration order. It is removed from every other roster.
    ///
    /// An agent currently in NO settlement's radius keeps its single existing
    /// membership (it is not reassigned just for wandering) — this pass only
    /// resolves genuine double-membership. `population_stats.current` is
    /// recomputed for every settlement AFTER the resolution so it always equals
    /// the live `member_agents.len()`.
    ///
    /// Determinism: settlement ids and member ids are collected and SORTED
    /// before use, and the keeper is chosen by `min_by_key((manhattan, id))`, so
    /// the result is independent of `HashMap`/`HashSet` iteration order.
    fn resolve_exclusivity(
        &self,
        resources: &mut SimResources,
        agent_positions: &HashMap<AgentId, (u32, u32)>,
        removed: &mut Vec<(AgentId, SettlementId)>,
    ) {
        // Build agent → rosters map in sorted (settlement, member) order.
        let mut settlement_ids: Vec<SettlementId> =
            resources.settlements.keys().copied().collect();
        settlement_ids.sort_unstable();
        let mut memberships: HashMap<AgentId, Vec<SettlementId>> = HashMap::new();
        for sid in &settlement_ids {
            if let Some(settlement) = resources.settlements.get(sid) {
                let mut members: Vec<AgentId> =
                    settlement.member_agents.iter().copied().collect();
                members.sort_unstable();
                for aid in members {
                    memberships.entry(aid).or_default().push(*sid);
                }
            }
        }

        // Resolve only genuine double-membership, in sorted agent order.
        let mut multi: Vec<(AgentId, Vec<SettlementId>)> = memberships
            .into_iter()
            .filter(|(_, rosters)| rosters.len() > 1)
            .collect();
        multi.sort_unstable_by_key(|(aid, _)| *aid);

        for (aid, rosters) in multi {
            let pos = match agent_positions.get(&aid).copied() {
                Some(p) => p,
                None => continue, // dead agent already dropped by retain; defensive
            };
            let keeper = rosters.iter().copied().min_by_key(|sid| {
                let ft = resources
                    .settlements
                    .get(sid)
                    .map(|s| s.formation_tile)
                    .unwrap_or((0, 0));
                (manhattan(ft, pos), *sid)
            });
            if let Some(keep) = keeper {
                for sid in rosters {
                    if sid != keep {
                        if let Some(settlement) = resources.settlements.get_mut(&sid) {
                            // Pass 19: removals are noted for the chronicle diff.
                            if settlement.remove_member_agent(aid) {
                                removed.push((aid, sid));
                            }
                            // Pass 11 measurement (display only, every version).
                            resources.tallies.exclusivity_moves += 1;
                        }
                    }
                }
            }
        }

        // Recompute live-member counts AFTER exclusivity resolution.
        for settlement in resources.settlements.values_mut() {
            settlement.population_stats.current = settlement.member_agents.len() as u32;
        }
    }

    /// Step 5: scan for unregistered clusters meeting both formation
    /// thresholds. Mint a new `SettlementId`, populate the settlement,
    /// and emit `SettlementFormed`.
    fn run_formation_scan(
        &mut self,
        resources: &mut SimResources,
        agent_positions: &HashMap<AgentId, (u32, u32)>,
        buildings: &[BuildingPos],
        width: u32,
        tick: u64,
        admitted: &mut Vec<(AgentId, SettlementId)>,
    ) {
        // Pass 11: only anchors of settlements that still exist block a
        // candidate (`FissionSystem` dissolves settlements outside this
        // system; before Pass 11 both maps always hold the same ids).
        let mut existing_anchors: Vec<(u32, u32)> = self
            .formation_tiles
            .iter()
            .filter(|(sid, _)| resources.settlements.contains_key(sid))
            .map(|(_, tile)| *tile)
            .collect();
        let mut candidate_tiles: Vec<(u32, u32)> = agent_positions.values().copied().collect();
        candidate_tiles.sort();
        candidate_tiles.dedup();
        if resources.rules.society.shelter_planning.is_some()
            || resources.rules.society.settlement_anchor == sim_core::rules::SettlementAnchor::Hearth
        {
            // Generated camps form around their real Hearth, so movement before
            // formation cannot anchor the village on its westernmost resident.
            // Default rules retain the exact legacy candidate sequence above.
            let mut hearths: Vec<_> = buildings.iter().filter_map(|(_, position)| {
                (resources.building_kinds.get(position) == Some(&BuildingKind::Hearth))
                    .then_some(*position)
            }).collect();
            hearths.sort_by_key(|&(x, y)| (y, x));
            hearths.dedup();
            if resources.rules.society.settlement_anchor == sim_core::rules::SettlementAnchor::AnyCluster {
                candidate_tiles.retain(|position| !hearths.contains(position));
                hearths.extend(candidate_tiles);
            }
            candidate_tiles = hearths;
        }
        for candidate in candidate_tiles {
            // Pass 11: a retired hearth never anchors a settlement again
            // (empty outside `game_v17`).
            if resources.retired_hearths.contains(&candidate) {
                continue;
            }
            // Skip candidates inside an existing settlement's radius.
            if existing_anchors
                .iter()
                .any(|a| chebyshev(*a, candidate) <= SETTLEMENT_PROXIMITY_RADIUS)
            {
                continue;
            }
            let agent_count = agent_positions
                .values()
                .filter(|p| chebyshev(candidate, **p) <= SETTLEMENT_PROXIMITY_RADIUS)
                .count() as u32;
            let building_count = buildings
                .iter()
                .filter(|(_, p)| chebyshev(candidate, *p) <= SETTLEMENT_PROXIMITY_RADIUS)
                .count() as u32;
            if agent_count < SETTLEMENT_FORMATION_AGENT_THRESHOLD
                || building_count < SETTLEMENT_FORMATION_BUILDING_THRESHOLD
            {
                continue;
            }

            let new_id = resources.issue_settlement_id();
            let mut settlement = Settlement::new_with_id(new_id, tick);
            // Pin the fixed formation anchor — the same `candidate` tile stored
            // in `self.formation_tiles` below. Fixed for the settlement's
            // lifetime so the FFI marker does not drift with the live centroid.
            settlement.formation_tile = candidate;
            // Pass 13 (`game_v19`): the hidden fission threshold, a world draw
            // keyed by the new id and its splits so far (none for a fresh id).
            if let Some(pressure) = resources.rules.society.fission.and_then(|f| f.pressure) {
                let k = resources.fission_log.iter().filter(|r| r.parent == new_id).count() as u32;
                settlement.fission_threshold = sim_core::fission_threshold::fission_threshold(
                    pressure.threshold_table,
                    resources.world_seed,
                    u64::from(new_id),
                    k,
                );
            }
            let mut founding_members: Vec<AgentId> = Vec::new();
            for (aid, pos) in agent_positions {
                if chebyshev(candidate, *pos) <= SETTLEMENT_PROXIMITY_RADIUS {
                    settlement.add_member_agent(*aid);
                    founding_members.push(*aid);
                }
            }
            for (bid, bpos) in buildings {
                if chebyshev(candidate, *bpos) <= SETTLEMENT_PROXIMITY_RADIUS {
                    settlement.add_member_building(*bid);
                }
            }
            settlement.population_stats.current = settlement.member_agents.len() as u32;
            founding_members.sort();
            // Pass 19: the founders are admissions for the chronicle diff.
            admitted.extend(founding_members.iter().map(|aid| (*aid, new_id)));

            let formed_id = resources.issue_event_id();
            let tile_idx = candidate.1 * width + candidate.0;
            resources.causal_log.push(
                tile_idx,
                CausalEvent::SettlementFormed {
                    id: formed_id,
                    parent: None,
                    settlement_id: new_id,
                    founding_members,
                    tick,
                },
            );

            // V7 Phase 10-γ: track newly-formed settlements so the
            // formation event can be seeded into community_history AFTER
            // step 6 (ingest_community_history) routes the founding-tick
            // building events. This makes the SettlementFormed entry the
            // most recent entry at formation tick (plan A6 "most recent
            // or only"), while still allowing the founding-tick
            // BuildingPlaced events to route into history (plan A21).
            self.newly_formed_this_tick.push((new_id, formed_id));

            self.formation_tiles.insert(new_id, candidate);
            self.formation_event_ids.insert(new_id, formed_id);
            self.birth_last_tick.insert(new_id, tick);
            if settlement.population_stats.current > 0
                && !settlement.member_buildings.is_empty()
            {
                self.last_known_population
                    .insert(new_id, settlement.population_stats.current);
                if let Some(m) = settlement.member_agents.iter().min().copied() {
                    self.last_known_member_id.insert(new_id, m);
                }
                if let Some(b) = settlement.member_buildings.iter().min().copied() {
                    self.last_known_building_id.insert(new_id, b);
                }
            }
            if resources.rules.society.shelter_planning.is_some_and(|rule|
                rule.housing == sim_core::rules::HousingCount::Owned)
            {
                for tile in resources.building_kinds.keys() {
                    if chebyshev(*tile, candidate) <= SETTLEMENT_PROXIMITY_RADIUS + 3 {
                        resources.building_owner.entry(*tile).or_insert(new_id);
                    }
                }
            }
            existing_anchors.push(candidate);
            resources.settlements.insert(new_id, settlement);
        }
    }

    /// Step 6: route eligible same-tick events into per-settlement
    /// community history.
    ///
    /// Routes (1:1 correspondence — each event id appears in at most one
    /// settlement's history):
    ///   * `BuildingPlaced` whose position is inside settlement boundary
    ///     — picked from the seen-vs-current registry diff so the routing
    ///     remains valid even after the per-tile ring evicts the event;
    ///   * `CombatCompleted` involving a settlement member (attacker or
    ///     defender);
    ///   * `AgentDecision { SettlementReason }` whose target_agent is a
    ///     member.
    fn ingest_community_history(
        &mut self,
        resources: &mut SimResources,
        new_buildings: &[BuildingPos],
        tick: u64,
    ) {
        // (a) BuildingPlaced — route by position to the closest
        //     settlement whose formation tile is within radius. Use the
        //     registry diff so events evicted from the per-tile ring are
        //     still attributable.
        for (bid, position) in new_buildings {
            // The `BuildingPlaced.id` is the same as the `BuildingId` (the
            // event id is stored as the building's stable id in the
            // registry — see BuildingStampSystem / ConstructionSystem).
            self.route_history_event(resources, *bid, *position);
        }

        // (b) CombatCompleted involving a member — scan the causal log
        //     for events from THIS tick. Membership check uses the
        //     post-sync `member_agents` snapshot.
        // (c) AgentDecision{SettlementReason} where the target_agent is a
        //     member of some settlement — extracted from the same scan.
        // V7 Phase 10-γ / P10γ-A13: combat routing is gated by the
        // explicit `settlement_link` tag on `CombatCompleted`, NOT by a
        // pure membership check. Events whose `settlement_link == None`
        // are NOT routed — even if a combatant happens to be a member
        // at scan time. This makes the routing predicate auditable on
        // the event itself.
        let mut combat_events: Vec<(EventId, SettlementId)> = Vec::new();
        let mut settlement_decisions: Vec<(EventId, AgentId)> = Vec::new();
        for (_tile_idx, log) in resources.causal_log.iter() {
            for ev in log.iter() {
                if ev.tick() != tick {
                    continue;
                }
                match ev {
                    CausalEvent::CombatCompleted {
                        id,
                        settlement_link: Some(sid),
                        ..
                    } => {
                        combat_events.push((*id, *sid));
                    }
                    CausalEvent::AgentDecision {
                        id,
                        reason: DecisionReason::SettlementReason,
                        agent: _emitting_agent,
                        ..
                    } => {
                        // We need the target_agent — but AgentDecision
                        // doesn't carry the target directly. The decision
                        // sets `AgentState::Seeking{Agent(target_agent)}`.
                        // We use the emitting agent's current AgentState
                        // at scan time as the proxy. The route_settlement_
                        // decision_to_target() helper resolves the target
                        // by reading the agent's state.
                        settlement_decisions.push((*id, *_emitting_agent));
                    }
                    _ => {}
                }
            }
        }
        for (event_id, sid) in combat_events {
            // Route ONLY to the settlement carried in the explicit link.
            // If that settlement still exists, append. Untagged events
            // (settlement_link == None) were filtered out above.
            if resources.settlements.contains_key(&sid) {
                self.append_unique(resources, sid, event_id);
            }
        }
        for (event_id, emitting_agent) in settlement_decisions {
            // The cascade arm's `target_agent` is the lowest-id member of
            // the lowest-id capacity settlement. Reproduce that here
            // (deterministic) — but only as a routing hint. The
            // authoritative route is "the settlement that contains the
            // target agent referenced by the emitter's current Seeking
            // state". Without re-reading the world we use the same
            // selection rule as agent_decision.rs.
            //
            // Fallback: route by membership — emitting_agent is a NON-
            // member by the cascade arm's check, so we route to the
            // settlement that owns the target (lowest-id capacity
            // settlement, lowest-id member).
            let mut candidate_ids: Vec<SettlementId> = resources
                .settlements
                .iter()
                .filter(|(_, s)| {
                    s.population_stats.current < SETTLEMENT_MAX_POP
                        && !s.member_agents.is_empty()
                        && !s.member_agents.contains(&emitting_agent)
                })
                .map(|(id, _)| *id)
                .collect();
            candidate_ids.sort();
            if let Some(sid) = candidate_ids.first().copied() {
                self.append_unique(resources, sid, event_id);
            }
        }
    }

    /// Route a (event_id, position) pair to the closest settlement whose
    /// formation tile is within `SETTLEMENT_PROXIMITY_RADIUS`. Used for
    /// position-bearing events (BuildingPlaced).
    fn route_history_event(
        &mut self,
        resources: &mut SimResources,
        event_id: EventId,
        position: (u32, u32),
    ) {
        let keyed = resources.rules.agent.keyed_draws;
        let mut best: Option<(SettlementId, u32)> = None;
        for (sid, anchor) in &self.formation_tiles {
            let d = chebyshev(*anchor, position);
            if d > SETTLEMENT_PROXIMITY_RADIUS {
                continue;
            }
            match best {
                None => best = Some((*sid, d)),
                Some((_, bd)) if d < bd => best = Some((*sid, d)),
                // Pass 9: under keyed draws an exact tie goes to the lower
                // settlement id instead of the first in HashMap order.
                Some((bs, bd)) if keyed && d == bd && *sid < bs => best = Some((*sid, d)),
                _ => {}
            }
        }
        if let Some((sid, _)) = best {
            self.append_unique(resources, sid, event_id);
        }
    }

    /// Append `event_id` to `sid`'s community history if not already
    /// routed this tick. Enforces plan A21 1:1 correspondence.
    fn append_unique(
        &mut self,
        resources: &mut SimResources,
        sid: SettlementId,
        event_id: EventId,
    ) {
        if !self.routed_this_tick.insert((sid, event_id)) {
            return;
        }
        if let Some(settlement) = resources.settlements.get_mut(&sid) {
            // Guard against duplicate appends if the event was already in
            // history from a prior tick (defensive — should not happen in
            // practice with `routed_this_tick`).
            if !settlement.community_history.contains(&event_id) {
                settlement.append_history(event_id);
            }
        }
    }

    /// Pass 19 chronicle (write-only): formations and roster changes of this
    /// tick, by diff. `admitted` holds every (agent, settlement) pair the
    /// proximity sync or the formation scan actually added and `removed`
    /// every pair exclusivity actually removed, so a pair in both is
    /// unchanged (the non-fission re-admit/drop cycle) and the net joins and
    /// leaves equal the before/after roster difference. Members carrying
    /// `Founding` are skipped (their move is `Settled`). Entries are appended
    /// per new settlement in id order, then per agent in id order.
    fn chronicle_membership(
        &self,
        world: &World,
        resources: &mut SimResources,
        admitted: &[(AgentId, SettlementId)],
        removed: &[(AgentId, SettlementId)],
        tick: u64,
    ) {
        use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
        use std::collections::{BTreeMap, BTreeSet};
        if self.newly_formed_this_tick.is_empty() && admitted.is_empty() && removed.is_empty() {
            return;
        }
        let added: BTreeSet<(AgentId, SettlementId)> = admitted.iter().copied().collect();
        let gone: BTreeSet<(AgentId, SettlementId)> = removed.iter().copied().collect();
        let mut joins: BTreeMap<AgentId, BTreeSet<SettlementId>> = BTreeMap::new();
        let mut leaves: BTreeMap<AgentId, BTreeSet<SettlementId>> = BTreeMap::new();
        for &(aid, sid) in added.difference(&gone) {
            joins.entry(aid).or_default().insert(sid);
        }
        for &(aid, sid) in gone.difference(&added) {
            leaves.entry(aid).or_default().insert(sid);
        }
        if self.newly_formed_this_tick.is_empty() && joins.is_empty() && leaves.is_empty() {
            return;
        }
        let founding: BTreeSet<AgentId> = world
            .query::<(&Agent, &sim_core::components::Founding)>()
            .iter()
            .map(|(_, (agent, _))| agent.id)
            .collect();
        let mut handled: BTreeSet<AgentId> = BTreeSet::new();
        let mut formed: Vec<SettlementId> = self.newly_formed_this_tick.iter().map(|(sid, _)| *sid).collect();
        formed.sort_unstable();
        for sid in formed {
            let Some(settlement) = resources.settlements.get(&sid) else { continue };
            let mut members: Vec<AgentId> = settlement.member_agents.iter().copied().collect();
            members.sort_unstable();
            let daughter = members.iter().any(|id| founding.contains(id));
            resources.chronicle.push(
                ChronicleEntry::new(tick, ChronicleKind::SettlementFormed)
                    .settlement(Some(sid))
                    .value(members.len() as i64),
            );
            for id in members {
                if founding.contains(&id) {
                    continue;
                }
                handled.insert(id);
                for old in leaves.get(&id).into_iter().flatten() {
                    crate::runtime::chronicle_log::record(
                        world,
                        resources,
                        ChronicleEntry::new(tick, ChronicleKind::LeftSettlement)
                            .subject(id)
                            .settlement(Some(*old))
                            .cause(2),
                    );
                }
                let entry = if daughter {
                    ChronicleEntry::new(tick, ChronicleKind::Joined).cause(3)
                } else if resources.chronicle.has_founding_member(id) {
                    ChronicleEntry::new(tick, ChronicleKind::Joined).cause(1)
                } else {
                    ChronicleEntry::new(tick, ChronicleKind::FoundingMember)
                };
                crate::runtime::chronicle_log::record(world, resources, entry.subject(id).settlement(Some(sid)));
            }
        }
        let changed: BTreeSet<AgentId> = joins.keys().chain(leaves.keys()).copied().collect();
        for id in changed {
            if handled.contains(&id) || founding.contains(&id) {
                continue;
            }
            let now: BTreeSet<SettlementId> = resources
                .settlements
                .iter()
                .filter(|(_, s)| s.member_agents.contains(&id))
                .map(|(sid, _)| *sid)
                .collect();
            let joined = joins.get(&id).cloned().unwrap_or_default();
            let left = leaves.get(&id).cloned().unwrap_or_default();
            let had_roster = now.difference(&joined).next().is_some() || !left.is_empty();
            for old in &left {
                crate::runtime::chronicle_log::record(
                    world,
                    resources,
                    ChronicleEntry::new(tick, ChronicleKind::LeftSettlement)
                        .subject(id)
                        .settlement(Some(*old))
                        .cause(2),
                );
            }
            for new in &joined {
                crate::runtime::chronicle_log::record(
                    world,
                    resources,
                    ChronicleEntry::new(tick, ChronicleKind::Joined)
                        .subject(id)
                        .settlement(Some(*new))
                        .cause(u8::from(had_roster)),
                );
            }
        }
    }

    /// Step 7: per-settlement birth trigger.
    fn run_births(
        &mut self,
        world: &mut World,
        resources: &mut SimResources,
        width: u32,
        tick: u64,
    ) {
        let mut births: Vec<(SettlementId, (u32, u32))> = Vec::new();
        let mut settlement_ids: Vec<SettlementId> =
            resources.settlements.keys().copied().collect();
        settlement_ids.sort();
        for sid in settlement_ids {
            let last_birth = match self.birth_last_tick.get(&sid).copied() {
                Some(t) => t,
                None => continue,
            };
            let cooldown = match resources.rules.society.birth {
                BirthRule::Legacy => BIRTH_COOLDOWN_TICKS,
                BirthRule::Provisioned { cooldown_ticks, .. } => cooldown_ticks,
            };
            if tick < last_birth.saturating_add(cooldown) {
                continue;
            }
            let settlement = match resources.settlements.get(&sid) {
                Some(s) => s,
                None => continue,
            };
            if settlement.population_stats.current >= SETTLEMENT_MAX_POP {
                continue;
            }
            if let BirthRule::Provisioned {
                food_cost, max_mean_hunger, max_mean_thirst, sustain, ..
            } = resources.rules.society.birth {
                let capacity = resources.rules.society.shelter_planning.map_or(
                    SETTLEMENT_MAX_POP,
                    |rule| super::housing_capacity(resources, sid, &rule),
                );
                if settlement.member_agents.len() as u32 >= capacity
                    || settlement.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0) < food_cost
                {
                    continue;
                }
                let mut members: Vec<_> = world.query::<(&Agent, &Hunger, &Thirst)>()
                    .iter().filter(|(_, (agent, _, _))| settlement.member_agents.contains(&agent.id))
                    .map(|(_, (agent, hunger, thirst))| (agent.id, hunger.value, thirst.value)).collect();
                members.sort_by_key(|(id, _, _)| *id);
                if members.is_empty() {
                    continue;
                }
                let count = members.len() as f64;
                let hunger = members.iter().map(|(_, h, _)| *h as f64).sum::<f64>() / count;
                let thirst = members.iter().map(|(_, _, t)| *t).sum::<f64>() / count;
                if hunger > max_mean_hunger || thirst > max_mean_thirst {
                    continue;
                }
                if let Some(sustain) = sustain {
                    // Pass 4: with seasons the gate plans against the annual mean
                    // (identical to the fixed supply without seasons).
                    let supply = super::food_economy::annual_mean_supply(
                        resources, sid, sustain.forage_radius,
                    );
                    let adult_need = super::food_economy::adult_need_per_day(resources.ticks_per_day);
                    let future_need = (settlement.member_agents.len() as f64 + 1.0) * adult_need;
                    let reserve = sustain.reserve_days
                        * super::food_economy::members_need_per_day(world, resources, sid);
                    let stock = settlement.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0);
                    if future_need > sustain.utilization * supply
                        || f64::from(stock) < f64::from(food_cost) + reserve
                    {
                        continue;
                    }
                }
            }
            // Belonging-model birth anchor (membership-model-reform): a member
            // may now roam far outside SETTLEMENT_PROXIMITY_RADIUS, so the
            // newborn must spawn at the FIXED formation_tile (the settlement's
            // nursery) rather than at a member's now-possibly-distant position
            // (which would scatter newborns across the map and break the
            // "newborn spawns inside the radius" invariant). The live-member
            // presence check stays — a settlement with no live members cannot
            // give birth — but only the spawn SITE moves to the formation tile.
            if settlement.member_agents.is_empty() {
                continue;
            }
            let spawn_pos = match self.formation_tiles.get(&sid).copied() {
                Some(p) => p,
                None => continue,
            };
            births.push((sid, spawn_pos));
        }

        for (sid, spawn_pos) in births {
            let family_parents = resources.rules.society.family.and_then(|_| {
                let members = &resources.settlements.get(&sid)?.member_agents;
                crate::runtime::agent::pairing::birth_parents(world, members)
            });
            // A qualifying family supplies mother then father; the original
            // two-lowest-member parent chain remains the disabled/fallback path.
            let parent_ids: Vec<AgentId> = if let Some((_, mother, father)) = family_parents {
                vec![mother, father]
            } else {
                match resources.settlements.get(&sid) {
                Some(s) => {
                    let mut members: Vec<AgentId> =
                        s.member_agents.iter().copied().collect();
                    members.sort();
                    members.into_iter().take(2).collect()
                }
                None => Vec::new(),
                }
            };

            // Pass 9: under keyed draws the newborn's lineage key comes from the
            // first parent's key and birth count (never from the id counter);
            // the parent's count advances. Without a first parent carrying a
            // lineage (not expected in a keyed world) the key falls back to the
            // settlement and tick of the birth.
            let lineage = resources.rules.agent.keyed_draws.then(|| {
                let parent = parent_ids.first().and_then(|first| {
                    world.query_mut::<(&Agent, &mut sim_core::components::Lineage)>()
                        .into_iter()
                        .find(|(_, (agent, _))| agent.id == *first)
                        .map(|(_, (_, lineage))| lineage.child(resources.world_seed))
                });
                parent.unwrap_or(sim_core::components::Lineage {
                    key: sim_core::rng::keyed_u64(
                        resources.world_seed, sim_core::rng::STREAM_NEWBORN, u64::from(sid), tick, u32::MAX, 0,
                    ),
                    births: 0,
                })
            });
            if let Some((mother, _, _)) = family_parents {
                if let Ok(mut kin) = world.get::<&mut sim_core::components::Kin>(mother) {
                    kin.last_birth_tick = Some(tick);
                }
            }
            // Pass 16 inheritance uses the selected named parents' temperaments.
            let parent_temperaments: [Option<sim_core::components::Temperament>; 2] =
                if resources.rules.agent.temperament.is_some() && lineage.is_some() {
                    let mut found = [None; 2];
                    for (_, (agent, temperament)) in world.query::<(&Agent, &sim_core::components::Temperament)>().iter() {
                        for (slot, id) in parent_ids.iter().take(2).enumerate() {
                            if agent.id == *id {
                                found[slot] = Some(*temperament);
                            }
                        }
                    }
                    found
                } else {
                    [None; 2]
                };
            let new_agent_id = resources.issue_agent_id();
            let entity = world.spawn((
                Position::new(spawn_pos.0, spawn_pos.1),
                Agent { id: new_agent_id },
            ));
            // V7 Stage 1.5 — match bootstrap's component set so the newborn can
            // move (MovementRng → AgentMovementSystem iterates it) and its needs
            // rise so it joins the Seeking→Consuming loop. Seed derived
            // deterministically from the unique new_agent_id (splitmix64
            // multiply + birth salt) → reproducible, and distinct from
            // bootstrap streams. Initial need values stay 0.0 (newborn not
            // hungry at birth; births are time-staggered by the cooldown).
            let birth_seed = new_agent_id
                .wrapping_mul(0x9E37_79B9_7F4A_7C15)
                .wrapping_add(BIRTH_RNG_SALT);
            let (hunger_rate, thirst_rate, sleep_rate) = resources.rules.society.life_stage
                .map_or((BIRTH_HUNGER_RATE, BIRTH_THIRST_RATE, BIRTH_SLEEP_RATE),
                    |rule| (rule.hunger_rate, rule.thirst_rate, rule.sleep_rate));
            let _ = world.insert(
                entity,
                (
                    MovementRng::new(birth_seed),
                    AgentState::Idle,
                    Hunger::new(0.0, hunger_rate),
                    Thirst::new(0.0, thirst_rate),
                    Sleep::new(0.0, sleep_rate),
                    Social::new(0.0, BIRTH_SOCIAL_RATE),
                    Memory::new(),
                    // add-starvation-death — newborns carry BodyHealth too, so a
                    // newborn is never treated as instantly-dead by combat and is
                    // subject to the same starvation/recovery rules as adults.
                    BodyHealth::new(),
                    // Direction-2 slice 2-2 — newborns carry an empty Inventory so
                    // the Gather cascade arm can fill it once they are satisfied.
                    Inventory::default(),
                ),
            );

            if let Some(lineage) = lineage {
                let _ = world.insert_one(entity, lineage);
                // Pass 11: a newborn's hardiness, keyed on its lineage
                // (replicate 0: an identity trait, not noise).
                if resources.rules.society.exposure.is_some_and(|rule| rule.use_hardiness) {
                    let _ = world.insert_one(entity, sim_core::components::Hardiness::draw(resources.world_seed, lineage.key));
                }
                // Pass 16: a newborn's temperament from its lineage key and the
                // named parents' (`game_v20`); with effects its loneliness rate
                // leans with RD (a mutation: no archetype change).
                if let Some(rule) = resources.rules.agent.temperament {
                    let temperament = sim_core::components::Temperament::child(resources.world_seed, lineage.key, parent_temperaments);
                    let _ = world.insert_one(entity, temperament);
                    if rule.effects {
                        if let Ok(mut social) = world.get::<&mut Social>(entity) {
                            social.growth_rate = rule.social_rate(social.growth_rate, &temperament);
                        }
                    }
                }
                if resources.rules.society.family.is_some() {
                    let sex =
                        sim_core::components::Sex::from_lineage(resources.world_seed, lineage.key);
                    let kin = sim_core::components::Kin {
                        mother: parent_ids.first().copied(),
                        father: parent_ids.get(1).copied(),
                        ..sim_core::components::Kin::default()
                    };
                    let _ = world.insert_one(entity, sex);
                    let _ = world.insert_one(entity, kin);
                    let temperament = world
                        .get::<&sim_core::components::Temperament>(entity)
                        .ok()
                        .map(|t| *t);
                    if let Some(temperament) = temperament {
                        crate::runtime::agent::pairing::record_family_event(
                            world,
                            tick,
                            sim_core::components::FamilyEvent::Born {
                                child: new_agent_id,
                                mother: kin.mother,
                                father: kin.father,
                                fallback: family_parents.is_none(),
                                lineage_key: lineage.key,
                                female: sex.female,
                                temperament,
                            },
                        );
                    }
                }
            }
            if resources.rules.society.life_stage.is_some() {
                let _ = world.insert_one(entity, sim_core::components::Child { born_tick: tick });
            }
            if let Some(rule) = resources.rules.society.aging {
                let life = match lineage {
                    Some(lineage) => crate::runtime::agent::aging::keyed_newborn_life(
                        resources.world_seed, lineage.key, tick, resources.noise_replicate_for(tick), &rule,
                    ),
                    None => crate::runtime::agent::aging::newborn_life(
                        resources.world_seed, new_agent_id, tick, &rule,
                    ),
                };
                let _ = world.insert_one(entity, life);
            }
            // Pass 19 chronicle (write-only): the newborn's record from its
            // components, the parents' records, and the birth.
            crate::runtime::chronicle_log::ensure_entity(world, resources, new_agent_id, entity);
            crate::runtime::chronicle_log::record(
                world,
                resources,
                sim_core::chronicle::ChronicleEntry::new(tick, sim_core::chronicle::ChronicleKind::Born)
                    .subject(new_agent_id)
                    .other(parent_ids.first().copied())
                    .third(parent_ids.get(1).copied())
                    .settlement(Some(sid))
                    .cause(u8::from(family_parents.is_none())),
            );

            let parent = self.formation_event_ids.get(&sid).copied();
            let event_id = resources.issue_event_id();
            let tile_idx = spawn_pos.1 * width + spawn_pos.0;
            resources.causal_log.push(
                tile_idx,
                CausalEvent::AgentBorn {
                    id: event_id,
                    parent,
                    agent: new_agent_id,
                    parent_ids,
                    tick,
                },
            );

            resources.tallies.births += 1;

            if let Some(settlement) = resources.settlements.get_mut(&sid) {
                if let BirthRule::Provisioned { food_cost, .. } = resources.rules.society.birth {
                    settlement.withdraw(ResourceKind::Food, food_cost);
                }
                settlement.add_member_agent(new_agent_id);
                settlement.population_stats.current = settlement.member_agents.len() as u32;
                settlement.population_stats.total_births =
                    settlement.population_stats.total_births.saturating_add(1);
                // Append the AgentBorn id to history once (1:1 with the
                // emitted event). Guarded by `routed_this_tick` so the
                // step-6 ingestion pass never re-adds it.
                if !settlement.community_history.contains(&event_id) {
                    settlement.append_history(event_id);
                }
                self.routed_this_tick.insert((sid, event_id));
            }
            self.birth_last_tick.insert(sid, tick);
        }
    }

    /// Step 8: dissolve settlements that satisfy BOTH zero-population
    /// AND no-buildings. Emit `SettlementDissolved` and FULLY clear local
    /// state so the same area can be re-formed later with a fresh id
    /// (plan: id not reused; area can re-form).
    ///
    /// Belonging-model reform note: after the retain-live step in
    /// [`Self::sync_membership`], `population_stats.current == 0` now means
    /// "every member is DEAD", NOT "no member is currently nearby". A
    /// settlement with a live but far-away member keeps `current > 0` and is
    /// therefore NEVER dissolved here — it dissolves only once that last live
    /// member despawns AND no buildings remain.
    fn run_dissolutions(
        &mut self,
        resources: &mut SimResources,
        width: u32,
        tick: u64,
    ) {
        let mut to_dissolve: Vec<SettlementId> = Vec::new();
        for (sid, settlement) in resources.settlements.iter() {
            if settlement.population_stats.current == 0 && settlement.member_buildings.is_empty()
            {
                to_dissolve.push(*sid);
            }
        }
        to_dissolve.sort();
        for sid in to_dissolve {
            let parent = self.formation_event_ids.get(&sid).copied();
            let tile = self
                .formation_tiles
                .get(&sid)
                .copied()
                .unwrap_or((0, 0));
            let tile_idx = tile.1 * width + tile.0;
            let dissolved_id = resources.issue_event_id();
            let final_population = self
                .last_known_population
                .get(&sid)
                .copied()
                .unwrap_or_else(|| {
                    resources
                        .settlements
                        .get(&sid)
                        .map(|s| s.population_stats.current)
                        .unwrap_or(0)
                });
            let last_member_id = self.last_known_member_id.get(&sid).copied();
            let last_building_id = self.last_known_building_id.get(&sid).copied();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::SettlementDissolved {
                    id: dissolved_id,
                    parent,
                    settlement_id: sid,
                    final_population,
                    cause: DissolutionCause::EmptyMembersAndBuildings,
                    last_member_id,
                    last_building_id,
                    tick,
                },
            );
            resources.settlements.remove(&sid);
            // Pass 11 measurement (display only).
            resources.dissolutions.push((sid, tick));
            // Pass 19 chronicle (write-only).
            resources.chronicle.push(
                sim_core::chronicle::ChronicleEntry::new(tick, sim_core::chronicle::ChronicleKind::SettlementDissolved)
                    .settlement(Some(sid)),
            );
            resources.building_owner.retain(|_, owner| *owner != sid);
            // FULL local-state clear so the same area can re-form.
            // Without clearing `formation_tiles` the area would be
            // permanently suppressed from re-formation (every candidate
            // tile inside the old anchor's radius would be skipped).
            self.formation_tiles.remove(&sid);
            self.formation_event_ids.remove(&sid);
            self.birth_last_tick.remove(&sid);
            self.last_known_population.remove(&sid);
            self.last_known_member_id.remove(&sid);
            self.last_known_building_id.remove(&sid);
        }
    }
}

impl RuntimeSystem for SettlementSystem {
    fn name(&self) -> &str {
        "SettlementSystem"
    }

    fn priority(&self) -> u32 {
        138
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        self.tick_run_count = self.tick_run_count.saturating_add(1);
        self.routed_this_tick.clear();
        self.newly_formed_this_tick.clear();

        let width = resources.tile_grid.width;
        let height = resources.tile_grid.height;
        if width == 0 || height == 0 {
            return;
        }
        let tick = resources.current_tick;

        self.snapshot_pre_clearing_population(resources);
        let (buildings, new_buildings) = self.snapshot_buildings(resources);
        let agent_positions = Self::snapshot_agents(world);
        // Pass 19: roster admissions and removals of this tick, for the
        // chronicle's membership diff (recorded after exclusivity).
        let mut admitted: Vec<(AgentId, SettlementId)> = Vec::new();
        let mut removed: Vec<(AgentId, SettlementId)> = Vec::new();
        self.sync_membership(resources, &agent_positions, &buildings, &mut admitted);
        self.run_formation_scan(resources, &agent_positions, &buildings, width, tick, &mut admitted);
        // Belonging-model reform: enforce one-settlement-per-agent AFTER both the
        // proximity sync and the formation scan, so a founder added by formation
        // that was already a member elsewhere is moved (not duplicated). Also
        // recomputes population_stats.current, so ingest/births/dissolution below
        // read post-exclusivity live-member counts.
        self.resolve_exclusivity(resources, &agent_positions, &mut removed);
        self.chronicle_membership(world, resources, &admitted, &removed, tick);
        // Note: ingest_community_history is called BEFORE run_births so
        // the BuildingPlaced/CombatCompleted/SettlementReason events from
        // upstream systems land before birth events for the same tick.
        // run_births appends AgentBorn directly to history.
        self.ingest_community_history(resources, &new_buildings, tick);
        // V7 Phase 10-γ: seed SettlementFormed events into community_history
        // AFTER step 6. This makes the SettlementFormed entry the MOST
        // RECENT entry at formation tick (plan A6), while the founding-tick
        // BuildingPlaced events that step 6 already routed remain in the
        // history at the start of the buffer.
        for (sid, formed_id) in self.newly_formed_this_tick.clone() {
            if let Some(settlement) = resources.settlements.get_mut(&sid) {
                if !settlement.community_history.contains(&formed_id) {
                    settlement.append_history(formed_id);
                }
            }
            self.routed_this_tick.insert((sid, formed_id));
        }
        self.run_births(world, resources, width, tick);
        self.run_dissolutions(resources, width, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    #[test]
    fn hearth_first_formation_is_opt_in() {
        for generated in [false, true] {
            let mut resources = SimResources::new(64, 64, sim_core::MaterialRegistry::new());
            if generated { resources.rules = sim_core::rules::WorldRules::game_v8(); }
            resources.building_kinds.insert((10, 10), BuildingKind::Hearth);
            let positions = HashMap::from([(0, (7, 7)), (1, (10, 10)), (2, (13, 13)), (3, (10, 9))]);
            let buildings = vec![(0, (10, 10)), (1, (9, 9)), (2, (11, 9))];
            let mut system = SettlementSystem::new();
            system.run_formation_scan(&mut resources, &positions, &buildings, 64, 0, &mut Vec::new());
            let first = resources.settlements.get(&1).expect("formation threshold met");
            assert_eq!(first.formation_tile, if generated { (10, 10) } else { (7, 7) });
            if generated { assert_eq!(first.member_agents.len(), 4); }
        }
    }

    #[test]
    fn metadata() {
        let s = SettlementSystem::new();
        assert_eq!(s.name(), "SettlementSystem");
        assert_eq!(s.priority(), 138);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn empty_world_tick_is_no_op() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        let mut sys = SettlementSystem::new();
        sys.tick(&mut engine.world, &mut engine.resources);
        assert!(engine.resources.settlements.is_empty());
    }

    #[test]
    fn chebyshev_distance_is_symmetric() {
        assert_eq!(chebyshev((0, 0), (5, 3)), 5);
        assert_eq!(chebyshev((5, 3), (0, 0)), 5);
        assert_eq!(chebyshev((10, 10), (10, 10)), 0);
    }

    #[test]
    fn birth_cooldown_constant_is_200() {
        assert_eq!(BIRTH_COOLDOWN_TICKS, 200);
    }

    #[test]
    fn tick_run_count_increments_each_tick() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        let mut sys = SettlementSystem::new();
        for _ in 0..5 {
            sys.tick(&mut engine.world, &mut engine.resources);
        }
        assert_eq!(sys.tick_run_count(), 5);
    }
    #[test]
    fn newborn_life_stage_rates_are_rule_gated() {
        use sim_core::components::Child;
        use sim_core::rules::WorldRules;
        for children in [false, true] {
            let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
            engine.resources.rules = if children { WorldRules::game_v9() } else { WorldRules::game_v8() };
            if let BirthRule::Provisioned { cooldown_ticks, .. } = &mut engine.resources.rules.society.birth {
                *cooldown_ticks = 720;
            }
            for _ in 0..3 {
                let entity = engine.spawn_agent(10, 10);
                engine.world.insert(entity, (Hunger::new(0.0, 0.05), Thirst::new(0.0, 0.08))).expect("founder needs");
            }
            engine.resources.building_registry.insert(100, (10, 10));
            engine.resources.building_registry.insert(101, (11, 10));
            engine.resources.building_kinds.insert((10, 10), BuildingKind::Hearth);
            engine.resources.building_kinds.insert((11, 10), BuildingKind::Shelter);
            for x in 10..20 { engine.resources.food_source_max.insert((x, 11), 4); }
            let mut system = SettlementSystem::new();
            system.tick(&mut engine.world, &mut engine.resources);
            let sid = *engine.resources.settlements.keys().next().expect("formed");
            engine.resources.settlements.get_mut(&sid).expect("settlement").store(ResourceKind::Food, 500);
            engine.resources.current_tick = 720;
            system.tick(&mut engine.world, &mut engine.resources);
            assert_eq!(engine.resources.tallies.births, 1);
            let mut query = engine.world.query::<(&Agent, &Hunger, &Thirst, &Sleep, Option<&Child>)>();
            let (_, (_, hunger, thirst, sleep, child)) = query.iter().next().expect("newborn with sleep");
            let expected = if children { (0.03, 0.048, 0.03) } else { (BIRTH_HUNGER_RATE, BIRTH_THIRST_RATE, BIRTH_SLEEP_RATE) };
            assert_eq!((hunger.growth_rate, thirst.growth_rate, sleep.growth_rate), expected);
            assert_eq!(child.map(|child| child.born_tick), children.then_some(720));
        }
    }

}
