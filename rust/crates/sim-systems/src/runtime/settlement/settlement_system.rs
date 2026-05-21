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
use sim_core::causal::event::{CausalEvent, DecisionReason, EventId};
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingId, Hunger, Memory, Position, Settlement, SettlementId,
    Sleep, Social, Thirst, SETTLEMENT_FORMATION_AGENT_THRESHOLD,
    SETTLEMENT_FORMATION_BUILDING_THRESHOLD, SETTLEMENT_MAX_POP, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_engine::{RuntimeSystem, SimResources};

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

/// Chebyshev distance between two tile coordinates.
fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    let dx = a.0.abs_diff(b.0);
    let dy = a.1.abs_diff(b.1);
    dx.max(dy)
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
    /// Monotonic side-effect counter incremented every tick the system
    /// runs (plan A28). Observable via [`Self::tick_run_count`].
    tick_run_count: u64,
    /// Settlement→event-id pairs already routed to community history
    /// during the current tick. Prevents double-append within a single
    /// tick (plan A21 1:1 correspondence).
    routed_this_tick: HashSet<(SettlementId, EventId)>,
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
        let new_buildings: BuildingPosList = buildings
            .iter()
            .filter(|(bid, _)| !self.seen_building_ids.contains(bid))
            .copied()
            .collect();
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

    /// Step 4: refresh `member_agents` + `member_buildings` from proximity
    /// to each settlement's formation tile.
    fn sync_membership(
        &self,
        resources: &mut SimResources,
        agent_positions: &HashMap<AgentId, (u32, u32)>,
        buildings: &[BuildingPos],
    ) {
        let live_building_ids: HashSet<BuildingId> =
            buildings.iter().map(|(id, _)| *id).collect();
        for (id, settlement) in resources.settlements.iter_mut() {
            let formation_tile = match self.formation_tiles.get(id).copied() {
                Some(t) => t,
                None => continue,
            };

            // Refresh member_agents from proximity.
            let mut new_members = HashSet::new();
            for (agent_id, pos) in agent_positions {
                if chebyshev(formation_tile, *pos) <= SETTLEMENT_PROXIMITY_RADIUS {
                    new_members.insert(*agent_id);
                }
            }
            settlement.member_agents = new_members;

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
    ) {
        let mut existing_anchors: Vec<(u32, u32)> =
            self.formation_tiles.values().copied().collect();
        let mut candidate_tiles: Vec<(u32, u32)> = agent_positions.values().copied().collect();
        candidate_tiles.sort();
        candidate_tiles.dedup();
        for candidate in candidate_tiles {
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

            self.formation_tiles.insert(new_id, candidate);
            self.formation_event_ids.insert(new_id, formed_id);
            self.birth_last_tick.insert(new_id, tick);
            if settlement.population_stats.current > 0
                && !settlement.member_buildings.is_empty()
            {
                self.last_known_population
                    .insert(new_id, settlement.population_stats.current);
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
        let mut combat_events: Vec<(EventId, AgentId, AgentId)> = Vec::new();
        let mut settlement_decisions: Vec<(EventId, AgentId)> = Vec::new();
        for (_tile_idx, log) in resources.causal_log.iter() {
            for ev in log.iter() {
                if ev.tick() != tick {
                    continue;
                }
                match ev {
                    CausalEvent::CombatCompleted {
                        id,
                        attacker,
                        defender,
                        ..
                    } => {
                        combat_events.push((*id, *attacker, *defender));
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
        for (event_id, attacker, defender) in combat_events {
            // Find the first settlement that contains either party.
            let mut sids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
            sids.sort();
            for sid in sids {
                let is_member = resources
                    .settlements
                    .get(&sid)
                    .map(|s| {
                        s.member_agents.contains(&attacker)
                            || s.member_agents.contains(&defender)
                    })
                    .unwrap_or(false);
                if is_member {
                    self.append_unique(resources, sid, event_id);
                    break;
                }
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
        let mut best: Option<(SettlementId, u32)> = None;
        for (sid, anchor) in &self.formation_tiles {
            let d = chebyshev(*anchor, position);
            if d > SETTLEMENT_PROXIMITY_RADIUS {
                continue;
            }
            match best {
                None => best = Some((*sid, d)),
                Some((_, bd)) if d < bd => best = Some((*sid, d)),
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

    /// Step 7: per-settlement birth trigger.
    fn run_births(
        &mut self,
        world: &mut World,
        resources: &mut SimResources,
        agent_positions: &HashMap<AgentId, (u32, u32)>,
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
            if tick < last_birth.saturating_add(BIRTH_COOLDOWN_TICKS) {
                continue;
            }
            let settlement = match resources.settlements.get(&sid) {
                Some(s) => s,
                None => continue,
            };
            if settlement.population_stats.current >= SETTLEMENT_MAX_POP {
                continue;
            }
            let anchor_agent = match settlement.member_agents.iter().min().copied() {
                Some(a) => a,
                None => continue,
            };
            let spawn_pos = match agent_positions.get(&anchor_agent).copied() {
                Some(p) => p,
                None => continue,
            };
            births.push((sid, spawn_pos));
        }

        for (sid, spawn_pos) in births {
            let new_agent_id = resources.issue_agent_id();
            let entity = world.spawn((
                Position::new(spawn_pos.0, spawn_pos.1),
                Agent { id: new_agent_id },
            ));
            let _ = world.insert(
                entity,
                (
                    AgentState::Idle,
                    Hunger::new(0.0, 0.0),
                    Thirst::new(0.0, 0.0),
                    Sleep::new(0.0, 0.0),
                    Social::new(0.0, 0.0),
                    Memory::new(),
                ),
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
                    tick,
                },
            );

            if let Some(settlement) = resources.settlements.get_mut(&sid) {
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
            resources.causal_log.push(
                tile_idx,
                CausalEvent::SettlementDissolved {
                    id: dissolved_id,
                    parent,
                    settlement_id: sid,
                    final_population,
                    tick,
                },
            );
            resources.settlements.remove(&sid);
            // FULL local-state clear so the same area can re-form.
            // Without clearing `formation_tiles` the area would be
            // permanently suppressed from re-formation (every candidate
            // tile inside the old anchor's radius would be skipped).
            self.formation_tiles.remove(&sid);
            self.formation_event_ids.remove(&sid);
            self.birth_last_tick.remove(&sid);
            self.last_known_population.remove(&sid);
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

        let width = resources.tile_grid.width;
        let height = resources.tile_grid.height;
        if width == 0 || height == 0 {
            return;
        }
        let tick = resources.current_tick;

        self.snapshot_pre_clearing_population(resources);
        let (buildings, new_buildings) = self.snapshot_buildings(resources);
        let agent_positions = Self::snapshot_agents(world);
        self.sync_membership(resources, &agent_positions, &buildings);
        self.run_formation_scan(resources, &agent_positions, &buildings, width, tick);
        // Note: ingest_community_history is called BEFORE run_births so
        // the BuildingPlaced/CombatCompleted/SettlementReason events from
        // upstream systems land before birth events for the same tick.
        // run_births appends AgentBorn directly to history.
        self.ingest_community_history(resources, &new_buildings, tick);
        self.run_births(world, resources, &agent_positions, width, tick);
        self.run_dissolutions(resources, width, tick);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

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
}
