//! ConstructionSystem: advances construction sites worked by co-located agents and turns finished sites into buildings.
//!
//! Priority: 133 · Interval: 1 tick · Gate: none
//! Reads: `Position`, `AgentState`, `ConstructionSite`, `MaterialNeeds`, `ColonySite`; `SimResources.tile_grid`, `causal_log` (`ConstructionStarted`), `site_owner`, `settlements`, `rules`
//! Writes: `ConstructionSite` (progress, despawn), `AgentState`; `SimResources.causal_log`, `building_registry`, `building_kinds`, `building_owner`, `site_owner`, sleep tiles, `colony` (footprints, item ledger, tallies)
//! Events: `CausalEvent::ConstructionCompleted`, `CausalEvent::BuildingPlaced`; chronicle `HutBuilt` (M1)
//!
//! `ConstructionSystem` at priority 133 — the runtime tick of the
//! Phase 6 agent-driven construction loop (V7 Phase 6-β / P6β-1).
//!
//! # Tick ordering
//!
//! ```text
//! 120  AgentMovementSystem
//! 125  AgentDecisionSystem        (Idle→Seeking, Seeking→Consuming, ConstructionStarted emit)
//! 130  HungerDecaySystem
//! 131  ThirstDecaySystem
//! 132  SleepDecaySystem
//! 133  ConstructionSystem         ← this file
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Slotted strictly between `SleepDecaySystem` (132) and
//! `InfluenceVisualizationSystem` (1000), runs every tick.
//!
//! # Responsibilities
//!
//! For each agent in [`AgentState::Consuming`]`{ target: TargetKind::ConstructionSite }`:
//!
//! 1. Look up the co-located [`ConstructionSite`] entity by `(pos.x, pos.y)`.
//! 2. If no site exists at the tile — **absent-site fallback**: reset the
//!    agent to [`AgentState::Idle`] without panic, without progress
//!    mutation, without emitting completion events. This guards against
//!    the race where the site was despawned between the
//!    `AgentDecisionSystem` tick and this one.
//! 3. If a site exists — call [`ConstructionSite::advance`]. Each
//!    co-located [`AgentState::Consuming`]`{ConstructionSite}` agent
//!    advances the site **once** per tick (per-agent progress semantics
//!    per locked P6β-10 / attempt 2). The completion-edge is recorded
//!    at most once per site even if multiple agents are co-located; any
//!    subsequent agents in the same tick observe the saturated site
//!    and contribute no extra completion chain.
//! 4. On completion edge — emit
//!    [`CausalEvent::ConstructionCompleted`] followed by
//!    [`CausalEvent::BuildingPlaced`] (parent-chained,
//!    `radius = 0`), reset all co-located
//!    `Consuming { ConstructionSite }` agents to `Idle`, and despawn the
//!    site entity.
//!
//! # Causal chain closure
//!
//! ```text
//! AgentDecision { reason: ConstructionReason } ← parent
//!   ↑ (ConstructionStarted.parent)
//! ConstructionStarted                          ← parent
//!   ↑ (ConstructionCompleted.parent)
//! ConstructionCompleted                        ← parent
//!   ↑ (BuildingPlaced.parent)
//! BuildingPlaced { radius: 0 }
//! ```
//!
//! `radius = 0` distinguishes the agent-construction emission path from
//! the BSS (`BuildingStampSystem` priority 90) stamping path; the
//! agent-construction path does not stamp influence in Phase 6-β.
//!
//! # M1 colony sites (`MaterialNeeds` + `ColonySite`)
//!
//! A colony site advances only once every material is delivered; until then
//! a co-located `Consuming { ConstructionSite }` agent takes the absent-site
//! fallback. On completion a hut is registered through [`register_dwelling`],
//! the path shelters use (kind, sleep tile, owner), its delivered logs move
//! to `colony.item_ledger.consumed` and its footprint becomes a hut footprint.
//! M3: a space site (wall, door, floor, bedroll) finishes through
//! `jobs::space::complete_site` with no building entry.

use std::collections::HashMap;

use hecs::{Entity, World};
use sim_core::causal::CausalEvent;
use sim_core::chronicle::{ChronicleEntry, ChronicleKind};
use sim_core::colony::{
    Bill, BillMode, BlueprintKind, ColonySite, MaterialNeeds, Occupant, RecipeKind, Workshop,
};
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingBlueprint, BuildingKind, ConstructionSite, Position,
    SettlementId, TargetKind,
};
use sim_engine::{RuntimeSystem, SimResources, RESOURCE_SOURCE_INFINITE};

/// Phase 6-β construction runtime system. Stateless — all per-site state
/// lives in the [`ConstructionSite`] component.
#[derive(Debug, Default)]
pub struct ConstructionSystem;

impl ConstructionSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self
    }
}

impl RuntimeSystem for ConstructionSystem {
    fn name(&self) -> &str {
        "ConstructionSystem"
    }

    fn priority(&self) -> u32 {
        133
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let width = resources.tile_grid.width;
        if width == 0 || resources.tile_grid.height == 0 {
            return;
        }
        let tick = resources.current_tick;

        // (a) Snapshot site positions for fast agent-side lookup. We
        //     additionally track which sites are already complete so the
        //     agent loop below can treat already-complete sites as the
        //     same kind of "no work to do" condition as absent sites:
        //     the agent gets reset to Idle (no progress mutation, no
        //     duplicate completion chain).
        let mut sites_by_pos: HashMap<(u32, u32), Entity> = HashMap::new();
        let mut complete_sites_by_pos: HashMap<(u32, u32), Entity> = HashMap::new();
        {
            let mut q = world.query::<(&ConstructionSite, Option<&MaterialNeeds>)>();
            for (e, (site, needs)) in q.iter() {
                let key = (site.position.x, site.position.y);
                // M1: a colony site waiting for materials is not workable yet.
                let waiting = needs.is_some_and(|n| !n.complete());
                if site.is_complete() || waiting {
                    complete_sites_by_pos.insert(key, e);
                } else {
                    sites_by_pos.insert(key, e);
                }
            }
        }

        // (b) Collect deferred operations from the agent loop. The borrow
        //     checker forbids mutating site components inside the agent
        //     query, so all writes happen below.
        //
        //     Attempt-3 fix: an agent in `Consuming{ConstructionSite}` on
        //     an already-complete site is routed to the same Idle-reset
        //     path as the absent-site fallback — without this, the agent
        //     stays stuck in Consuming forever (advance() returns false
        //     on a saturated site, so no completion edge fires and the
        //     agent never transitions out). No duplicate completion chain
        //     is emitted for the already-complete site.
        let mut absent_resets: Vec<Entity> = Vec::new();
        let mut progress_ops: Vec<Entity> = Vec::new();
        // The reset below assumes the colony plans no shelters (review round 2, L-b).
        let colony =
            resources.rules.colony.is_some() && resources.rules.society.shelter_planning.is_none();
        {
            let mut q = world.query::<(&Position, &AgentState)>();
            for (agent_entity, (pos, state)) in q.iter() {
                // M1 (colony only): the colony has no shelter planning, so a
                // `Seeking { ConstructionSite }` agent is one the Idle cascade
                // sent to the site it stands on. If that site is gone
                // (cancelled, or completed by another builder) nothing else
                // ever moves the agent out of Seeking: reset it to Idle.
                if colony
                    && matches!(
                        *state,
                        AgentState::Seeking {
                            target: TargetKind::ConstructionSite
                        }
                    )
                {
                    let key = (pos.x, pos.y);
                    if !sites_by_pos.contains_key(&key) && !complete_sites_by_pos.contains_key(&key)
                    {
                        absent_resets.push(agent_entity);
                    }
                    continue;
                }
                if !matches!(
                    *state,
                    AgentState::Consuming {
                        target: TargetKind::ConstructionSite
                    }
                ) {
                    continue;
                }
                let key = (pos.x, pos.y);
                if complete_sites_by_pos.contains_key(&key) {
                    // Already-complete site fallback — treat like absent.
                    absent_resets.push(agent_entity);
                    continue;
                }
                match sites_by_pos.get(&key).copied() {
                    None => absent_resets.push(agent_entity),
                    Some(site_entity) => progress_ops.push(site_entity),
                }
            }
        }

        // (c) Absent-site (or already-complete-site) fallback — reset
        //     agent FSM to Idle. Per the locked spec, no progress
        //     mutation, no completion events, no panic on a missing
        //     entity (the agent's site was despawned mid-tick or was
        //     already saturated by direct field write).
        for agent_entity in &absent_resets {
            if let Ok(mut state) = world.get::<&mut AgentState>(*agent_entity) {
                *state = AgentState::Idle;
            }
        }

        // (d) Per-agent progress: each co-located Consuming{ConstructionSite}
        //     agent advances the site once per tick. The completion-edge
        //     is recorded at most once per site — subsequent advances on a
        //     saturated site return `false` and contribute no extra chain.
        let mut completed: HashMap<Entity, (BuildingBlueprint, (u32, u32))> = HashMap::new();
        for site_entity in progress_ops {
            // Scope the mutable borrow tightly so the loop can re-enter
            // hecs queries below.
            let edge: Option<(BuildingBlueprint, (u32, u32))> = {
                let mut site_ref = match world.get::<&mut ConstructionSite>(site_entity) {
                    Ok(s) => s,
                    Err(_) => continue,
                };
                let just_completed = site_ref.advance();
                if just_completed {
                    Some((
                        site_ref.blueprint,
                        (site_ref.position.x, site_ref.position.y),
                    ))
                } else {
                    None
                }
            };
            if let Some((blueprint, position)) = edge {
                // First completion edge wins. Later agents observe the
                // saturated site (advance returns false) and are silently
                // absorbed into the same tick — no duplicate chain.
                completed
                    .entry(site_entity)
                    .or_insert((blueprint, position));
            }
        }

        // (e) For each completion edge: emit ConstructionCompleted then
        //     BuildingPlaced, reset all co-located Consuming agents to
        //     Idle, and despawn the site.
        let mut completed: Vec<_> = completed.into_iter().collect();
        if resources.rules.society.shelter_planning.is_some() || resources.rules.agent.keyed_draws {
            completed.sort_by_key(|(_, (_, position))| *position);
        }
        for (site_entity, (blueprint, position)) in completed {
            // M3: a space site (wall, door, floor, bedroll) worked in place by a
            // V7 builder finishes through the job layer's completion; no
            // building, no BuildingPlaced (ADR 0022).
            let space = world
                .get::<&ColonySite>(site_entity)
                .is_ok_and(|site| site.kind.is_space());
            if space {
                let mut q = world.query::<(&Position, &mut AgentState)>();
                for (_e, (pos, state)) in q.iter() {
                    if (pos.x, pos.y) == position
                        && matches!(
                            *state,
                            AgentState::Consuming {
                                target: TargetKind::ConstructionSite
                            }
                        )
                    {
                        *state = AgentState::Idle;
                    }
                }
                drop(q);
                crate::runtime::jobs::space::complete_site(world, resources, site_entity);
                continue;
            }
            if let Some(rule) = resources.rules.society.shelter_planning {
                if let Some(sid) = resources.site_owner.remove(&position) {
                    let owner =
                        (rule.housing == sim_core::rules::HousingCount::Owned).then_some(sid);
                    register_dwelling(resources, position, BuildingKind::Shelter, owner);
                }
            }
            let colony_site = world
                .get::<&ColonySite>(site_entity)
                .ok()
                .map(|s| *s)
                .zip(world.get::<&MaterialNeeds>(site_entity).ok().map(|n| *n));
            // M2: a workshop is a colony object, not a V7 building (no registry
            // entry, no BuildingPlaced; M2 DECISIONS D22).
            let workshop = colony_site.is_some_and(|(site, _)| site.kind.is_workshop());
            if let Some((site, needs)) = colony_site {
                let mut builders: Vec<AgentId> = world
                    .query::<(&Agent, &Position, &AgentState)>()
                    .iter()
                    .filter(|(_, (_, pos, state))| {
                        (pos.x, pos.y) == position
                            && matches!(
                                state,
                                AgentState::Consuming {
                                    target: TargetKind::ConstructionSite
                                }
                            )
                    })
                    .map(|(_, (agent, _, _))| agent.id)
                    .collect();
                builders.sort_unstable();
                if site.kind.is_workshop() {
                    complete_workshop(resources, site, needs);
                } else {
                    complete_hut(world, resources, site, needs, &builders, tick);
                }
            }
            let tile_idx = position.1 * width + position.0;

            // Parent of ConstructionCompleted: the most recent
            // ConstructionStarted on this tile (if any).
            let started_parent = resources.causal_log.get(tile_idx).and_then(|log| {
                log.as_slice().iter().rev().find_map(|ev| match ev {
                    CausalEvent::ConstructionStarted { id, .. } => Some(*id),
                    _ => None,
                })
            });

            let completed_id = resources.issue_event_id();
            resources.causal_log.push(
                tile_idx,
                CausalEvent::ConstructionCompleted {
                    id: completed_id,
                    parent: started_parent,
                    blueprint,
                    position,
                    tick,
                },
            );

            // Parent of BuildingPlaced (agent-construction path): the
            // ConstructionCompleted we just emitted. radius = 0 —
            // agent-construction does not stamp influence in Phase 6-β.
            if !workshop {
                let placed_id = resources.issue_event_id();
                // V7 Phase 10-β — register the completed building's position
                // for `SettlementSystem` proximity scans. Mirrors the
                // registration in `BuildingStampSystem` for FFI-originated
                // placements.
                resources.building_registry.insert(placed_id, position);
                resources.causal_log.push(
                    tile_idx,
                    CausalEvent::BuildingPlaced {
                        id: placed_id,
                        parent: Some(completed_id),
                        position,
                        radius: 0,
                        tick,
                    },
                );
            }

            // Reset every co-located Consuming{ConstructionSite} agent
            // to Idle — owned by ConstructionSystem itself per the
            // locked Assertion 8 contract.
            {
                let mut q = world.query::<(&Position, &mut AgentState)>();
                for (_e, (pos, state)) in q.iter() {
                    if pos.x == position.0
                        && pos.y == position.1
                        && matches!(
                            *state,
                            AgentState::Consuming {
                                target: TargetKind::ConstructionSite
                            }
                        )
                    {
                        *state = AgentState::Idle;
                    }
                }
            }

            // Despawn the site after agent updates so the query above
            // can still observe co-located agents at this position.
            let _ = world.despawn(site_entity);
        }
    }
}

/// Register a finished dwelling at `position` (its key tile): its kind, an
/// infinite sleep tile there and, when given, its owner. The one path shelters
/// (planned under `shelter_planning`) and M1 huts share.
pub fn register_dwelling(
    resources: &mut SimResources,
    position: (u32, u32),
    kind: BuildingKind,
    owner: Option<SettlementId>,
) {
    resources.building_kinds.insert(position, kind);
    resources.set_sleep_tile(position.0, position.1, RESOURCE_SOURCE_INFINITE);
    if let Some(sid) = owner {
        resources.building_owner.insert(position, sid);
    }
}

/// The settlement whose formation tile lies nearest `tile` (Chebyshev, ties
/// by id): the owner of a finished hut.
fn nearest_settlement(resources: &SimResources, tile: (u32, u32)) -> Option<SettlementId> {
    let mut ids: Vec<_> = resources.settlements.keys().copied().collect();
    ids.sort_unstable();
    ids.into_iter().min_by_key(|id| {
        resources.settlements.get(id).map_or(u32::MAX, |s| {
            let (x, y) = s.formation_tile;
            x.abs_diff(tile.0).max(y.abs_diff(tile.1))
        })
    })
}

/// M2: a finished workshop blueprint becomes a [`Workshop`] in
/// `colony.workshops` (no bills, an empty shelf), its footprint a workshop
/// footprint, its materials consumed. It is not a dwelling: no sleep tile, no
/// owner, no `building_kinds` entry.
fn complete_workshop(resources: &mut SimResources, site: ColonySite, needs: MaterialNeeds) {
    let id = resources.colony.next_workshop_id.max(1);
    resources.colony.next_workshop_id = id + 1;
    resources.colony.site_work_tiles.remove(&site.centre());
    for tile in site.tiles() {
        resources
            .colony
            .occupied
            .insert(tile, Occupant::Workshop(id));
    }
    for line in needs.used() {
        resources
            .colony
            .item_ledger
            .consume(line.kind, u64::from(line.delivered));
    }
    // M2 goal 3.8: a wood bench starts with the hearth's standing firewood bill.
    let mut bills = Vec::new();
    if site.kind == BlueprintKind::WoodBench && resources.rules.society.firewood.is_some() {
        let bill = resources.colony.next_bill_id.max(1);
        resources.colony.next_bill_id = bill + 1;
        bills.push(Bill {
            id: bill,
            recipe: RecipeKind::SplitFirewood,
            mode: BillMode::Maintain(crate::runtime::jobs::hearth::auto_bill_count(resources)),
            done: 0,
            suspended: false,
            auto: true,
        });
    }
    resources.colony.workshops.insert(
        id,
        Workshop {
            id,
            kind: site.kind,
            top_left: site.top_left,
            size: site.size,
            bills,
            shelf: Vec::new(),
            active: None,
        },
    );
    resources.colony.tallies.workshops_built += 1;
    resources.colony.touch(false);
}

/// M1: a finished colony site becomes a hut — registered like a shelter at
/// its centre, owned by the nearest settlement, its logs consumed.
fn complete_hut(
    world: &World,
    resources: &mut SimResources,
    site: ColonySite,
    needs: MaterialNeeds,
    builders: &[AgentId],
    tick: u64,
) {
    let centre = site.centre();
    let owner = nearest_settlement(resources, centre);
    register_dwelling(resources, centre, BuildingKind::Hut, owner);
    for tile in site.tiles() {
        resources
            .colony
            .occupied
            .insert(tile, Occupant::Hut(centre));
    }
    for line in needs.used() {
        resources
            .colony
            .item_ledger
            .consume(line.kind, u64::from(line.delivered));
    }
    resources.colony.tallies.huts_built += 1;
    resources.colony.touch(false);
    let mut entry = ChronicleEntry::new(tick, ChronicleKind::HutBuilt);
    entry.settlement = owner;
    entry.subject = builders.first().copied();
    entry.other = builders.get(1).copied();
    entry.third = builders.get(2).copied();
    entry.value = builders.len() as i64;
    crate::runtime::chronicle_log::record(world, resources, entry);
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    #[test]
    fn metadata() {
        let s = ConstructionSystem::new();
        assert_eq!(s.name(), "ConstructionSystem");
        assert_eq!(s.priority(), 133);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn empty_world_tick_is_no_op() {
        let mut engine = SimEngine::new(32, 32, MaterialRegistry::new());
        let mut sys = ConstructionSystem::new();
        // Must not panic on an empty world.
        sys.tick(&mut engine.world, &mut engine.resources);
    }
}
