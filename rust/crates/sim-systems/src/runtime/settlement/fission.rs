//! Revitalize Pass 11 — `FissionSystem` (priority 148, every tick; real work
//! hourly and at 06:00).
//!
//! Under `SocietyRules::fission` (`game_v17`) a crowded settlement splits: a
//! group walks to a wild region and founds a daughter settlement. Without the
//! rule the system returns at once, so every earlier world is untouched.
//!
//! **Daily check** (06:00, settlements in id order). A settlement may split
//! when the season is spring or summer, it is at least `min_age_days` old, at
//! least `cooldown_days` have passed since its last fission and since the
//! world's last fission, it holds at least `min_members` members and at least
//! `crowding × capacity` (capacity = the population the `SustainRule` birth
//! gate admits: `floor(utilization × annual_mean_supply / adult_need_per_day)`),
//! fewer than `max_settlements` settlements are alive (≥ 1 member; departed
//! groups count as one each) and a site is free. A site is a wild-region
//! centre whose distance to every formation tile is at least `site_clearance`
//! and that no departed group is heading to; the one nearest the parent's
//! formation tile wins (ties: lower region index). With every other condition
//! met but no free site the check counts `fission_blocked_no_site`.
//!
//! **Group.** k = clamp(round(members × `group_share`), `group_min`,
//! `group_max`). Non-elder members are ranked by
//! `keyed_u64(seed, STREAM_FISSION, lineage key, tick, 0, r)` with r the noise
//! replicate at the check (a fork resamples who leaves; the timing is decided
//! by the state); ties by AgentId. Adults are taken in rank order up to
//! `min_adults`, then the remaining slots in rank order from all non-elders.
//! Fewer than `min_adults` adults: no fission.
//!
//! **Departure.** Each group member gets `Founding { site, fission_id, leader,
//! since_tick }` (leader = the group's lowest-AgentId adult); an Idle or
//! Seeking member is set Idle with its `SeekTarget` dropped, so the decision
//! system starts the walk at once. With `stock_share` the parent's store
//! gives `floor(stock × k / members)` food and wood into a pending store held
//! here (returned to the parent if the fission is cancelled).
//!
//! **Founding** (every tick). When at least three living founders stand within
//! radius 5 of the site — or `settle_timeout_days` after departure, at the
//! leader's tile if it lies at least `site_clearance` from every formation
//! tile (otherwise the fission is cancelled) — a Hearth and two Shelters are
//! stamped as `worldgen::camp::populate` stamps the camp (nearest buildable
//! tiles, `building_event_queue`, shelter sleep tiles). The formation scan of
//! `SettlementSystem` forms the daughter on a later tick; on that tick this
//! system moves every living group member from the parent's roster (and any
//! other) to the daughter's, delivers the pending store, sets
//! `Settlement::parent`, removes `Founding` and counts `Tallies::fissions`.
//! A placed founding with no daughter one day later, or a group with fewer
//! than three living members, is cancelled (`fissions_cancelled`): the
//! markers go, the store returns, a placed hearth is retired.
//!
//! **Remnants and dissolution** (hourly). A settlement with 1 to
//! `remnant_members` members for 24 consecutive hours merges away: its
//! members leave its roster (as non-members they take the existing
//! `SettlementMigrant` arm toward the nearest member of any settlement), its
//! hearth is retired and it is dissolved. A settlement with no member left is
//! dissolved at once although its buildings stay; its hearth is retired.
//! Retired hearths (`SimResources::retired_hearths`) never anchor a new
//! settlement.
//!
//! Every loop runs in SettlementId / AgentId / fission-id order.

use std::collections::{BTreeMap, BTreeSet};

use hecs::World;
use sim_core::causal::event::{CausalEvent, DissolutionCause};
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingKind, Child, Elder, Founding, Lineage, Position, ResourceKind, SeekTarget,
    SettlementId, SETTLEMENT_FORMATION_AGENT_THRESHOLD, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::rng::{keyed_u64, STREAM_FISSION};
use sim_core::rules::{BirthRule, FissionRule};
use sim_core::TerrainType;
use sim_engine::{BuildingPlacedEvent, FissionRecord, RuntimeSystem, SimResources, RESOURCE_SOURCE_INFINITE};

use crate::runtime::resource_regen::season_at_tick;
use crate::runtime::settlement::food_economy::{adult_need_per_day, annual_mean_supply};

/// Engine priority: after `SettlementSystem` (138) and `FireSystem` (147).
pub const FISSION_PRIORITY: u32 = 148;

/// Minute of the daily check (06:00).
const CHECK_MINUTE: u64 = 360;

fn chebyshev(a: (u32, u32), b: (u32, u32)) -> u32 {
    a.0.abs_diff(b.0).max(a.1.abs_diff(b.1))
}

/// A departed group on its way (or waiting for its daughter to form).
#[derive(Debug, Clone)]
struct Group {
    parent: SettlementId,
    site: (u32, u32),
    members: BTreeSet<AgentId>,
    leader: AgentId,
    since: u64,
    food: u32,
    wood: u32,
    /// Hearth tile and tick once the buildings are stamped.
    placed: Option<((u32, u32), u64)>,
    /// Index into `SimResources::fission_log`.
    log: usize,
}

/// Pass 11 village fission.
#[derive(Debug, Default)]
pub struct FissionSystem {
    last_fission: BTreeMap<SettlementId, u64>,
    world_last_fission: Option<u64>,
    next_id: u32,
    groups: BTreeMap<u32, Group>,
    remnant_since: BTreeMap<SettlementId, u64>,
}

impl FissionSystem {
    /// Construct a fresh instance.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Living agents: id → (entity, tile, adult, elder, lineage key).
type Roster = BTreeMap<AgentId, (hecs::Entity, (u32, u32), bool, bool, u64)>;

fn roster(world: &World) -> Roster {
    world
        .query::<(&Agent, &Position, Option<&Child>, Option<&Elder>, Option<&Lineage>)>()
        .iter()
        .map(|(e, (a, p, child, elder, lineage))| {
            (a.id, (e, (p.x, p.y), child.is_none() && elder.is_none(), elder.is_some(), lineage.map_or(a.id, |l| l.key)))
        })
        .collect()
}

/// The population the `SustainRule` birth gate admits in settlement `sid`:
/// `floor(utilization × annual_mean_supply / adult_need_per_day)`; `None`
/// without a provisioned sustain rule.
pub fn sustain_capacity(resources: &SimResources, sid: SettlementId) -> Option<u32> {
    let BirthRule::Provisioned { sustain: Some(sustain), .. } = resources.rules.society.birth else {
        return None;
    };
    let supply = annual_mean_supply(resources, sid, sustain.forage_radius);
    let need = adult_need_per_day(resources.ticks_per_day);
    if need <= 0.0 {
        return None;
    }
    Some((sustain.utilization * supply / need).floor().max(0.0) as u32)
}

fn walkable(resources: &SimResources, p: (u32, u32)) -> bool {
    let t = resources.tile_grid.terrain_type[resources.tile_grid.idx(p.0, p.1)];
    !matches!(t, TerrainType::River | TerrainType::Mountain)
}

/// Stamp a Hearth at the buildable tile nearest `target` and two Shelters
/// beside it (offsets (−2, −1) and (2, −1), as the worldgen camp), each at the
/// nearest buildable tile; returns the hearth tile. Buildable = walkable, no
/// building, water, bush or deadwood spot. `None` when no tile is left.
fn stamp_camp(resources: &mut SimResources, target: (u32, u32)) -> Option<(u32, u32)> {
    let (w, h) = (resources.tile_grid.width, resources.tile_grid.height);
    let mut hearth = None;
    for (dx, dy, kind, radius) in [
        (0i64, 0i64, BuildingKind::Hearth, 6u32),
        (-2, -1, BuildingKind::Shelter, 2),
        (2, -1, BuildingKind::Shelter, 2),
    ] {
        let anchor = hearth.unwrap_or(target);
        let goal = (
            (i64::from(anchor.0) + dx).clamp(0, i64::from(w) - 1) as u32,
            (i64::from(anchor.1) + dy).clamp(0, i64::from(h) - 1) as u32,
        );
        let tile = (0..h)
            .flat_map(|y| (0..w).map(move |x| (x, y)))
            .filter(|&p| {
                walkable(resources, p)
                    && !resources.building_kinds.contains_key(&p)
                    && !resources.water_tiles.contains_key(&p)
                    && !resources.food_source_max.contains_key(&p)
                    && !resources.wood_source_max.contains_key(&p)
            })
            .min_by_key(|&p| (chebyshev(p, goal), p.1, p.0))?;
        resources.building_kinds.insert(tile, kind);
        resources.building_event_queue.push_back(BuildingPlacedEvent { position: tile, radius });
        if kind == BuildingKind::Shelter {
            resources.set_sleep_tile(tile.0, tile.1, RESOURCE_SOURCE_INFINITE);
        }
        if hearth.is_none() {
            hearth = Some(tile);
        }
    }
    hearth
}

impl FissionSystem {
    /// Remove the markers of a group, return its store to the parent, retire
    /// a placed hearth and count the cancellation.
    fn cancel(&mut self, world: &mut World, resources: &mut SimResources, roster: &Roster, id: u32, tick: u64) {
        let Some(group) = self.groups.remove(&id) else { return };
        for member in &group.members {
            if let Some((entity, ..)) = roster.get(member) {
                let _ = world.remove_one::<Founding>(*entity);
            }
        }
        if let Some(parent) = resources.settlements.get_mut(&group.parent) {
            parent.store(ResourceKind::Food, group.food);
            parent.store(ResourceKind::Wood, group.wood);
        }
        if let Some((hearth, _)) = group.placed {
            resources.retired_hearths.insert(hearth);
        }
        resources.tallies.fissions_cancelled += 1;
        if let Some(record) = resources.fission_log.get_mut(group.log) {
            record.cancelled_tick = Some(tick);
        }
    }

    /// The daughter at the group's hearth formed: move the group, deliver the
    /// store, link the parent.
    fn found(&mut self, world: &mut World, resources: &mut SimResources, roster: &Roster, id: u32, daughter: SettlementId, tick: u64) {
        let Some(group) = self.groups.remove(&id) else { return };
        let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        ids.sort_unstable();
        let mut moved_by_scan = 0u64;
        for member in &group.members {
            if !roster.contains_key(member) {
                continue;
            }
            let in_parent = resources.settlements.get(&group.parent).is_some_and(|s| s.member_agents.contains(member));
            let in_daughter = resources.settlements.get(&daughter).is_some_and(|s| s.member_agents.contains(member));
            // The formation scan's exclusivity pass already moved this founder.
            moved_by_scan += u64::from(in_daughter && !in_parent);
            for sid in &ids {
                if *sid != daughter {
                    if let Some(settlement) = resources.settlements.get_mut(sid) {
                        settlement.remove_member_agent(*member);
                    }
                }
            }
            if let Some(settlement) = resources.settlements.get_mut(&daughter) {
                settlement.add_member_agent(*member);
            }
            if let Some((entity, ..)) = roster.get(member) {
                let _ = world.remove_one::<Founding>(*entity);
            }
        }
        // Measurement: `exclusivity_moves` counts moves outside fission.
        resources.tallies.exclusivity_moves = resources.tallies.exclusivity_moves.saturating_sub(moved_by_scan);
        for settlement in resources.settlements.values_mut() {
            settlement.population_stats.current = settlement.member_agents.len() as u32;
        }
        if let Some(settlement) = resources.settlements.get_mut(&daughter) {
            settlement.store(ResourceKind::Food, group.food);
            settlement.store(ResourceKind::Wood, group.wood);
            settlement.parent = Some(group.parent);
        }
        resources.tallies.fissions += 1;
        if let Some(record) = resources.fission_log.get_mut(group.log) {
            record.daughter = Some(daughter);
            record.founded_tick = Some(tick);
        }
    }

    /// Walk every departed group forward one tick (fission-id order).
    fn advance(&mut self, world: &mut World, resources: &mut SimResources, rule: &FissionRule, roster: &Roster, tick: u64) {
        let ids: Vec<u32> = self.groups.keys().copied().collect();
        for id in ids {
            let Some(group) = self.groups.get(&id).cloned() else { continue };
            let alive: Vec<AgentId> = group.members.iter().copied().filter(|m| roster.contains_key(m)).collect();
            match group.placed {
                Some((hearth, placed_tick)) => {
                    let daughter = {
                        let mut found: Vec<SettlementId> = resources
                            .settlements
                            .iter()
                            .filter(|(_, s)| s.formation_tile == hearth)
                            .map(|(sid, _)| *sid)
                            .collect();
                        found.sort_unstable();
                        found.first().copied()
                    };
                    if let Some(daughter) = daughter {
                        self.found(world, resources, roster, id, daughter, tick);
                    } else if tick >= placed_tick + resources.ticks_per_day || alive.is_empty() {
                        self.cancel(world, resources, roster, id, tick);
                    }
                }
                None => {
                    if (alive.len() as u32) < SETTLEMENT_FORMATION_AGENT_THRESHOLD {
                        self.cancel(world, resources, roster, id, tick);
                        continue;
                    }
                    let near = alive
                        .iter()
                        .filter(|m| roster.get(m).is_some_and(|row| chebyshev(row.1, group.site) <= SETTLEMENT_PROXIMITY_RADIUS))
                        .count() as u32;
                    let target = if near >= SETTLEMENT_FORMATION_AGENT_THRESHOLD {
                        Some((group.site, false))
                    } else if tick >= group.since + u64::from(rule.settle_timeout_days) * resources.ticks_per_day {
                        let leader = if roster.contains_key(&group.leader) { group.leader } else { alive[0] };
                        let tile = roster.get(&leader).map_or(group.site, |row| row.1);
                        if resources.settlements.values().all(|s| chebyshev(s.formation_tile, tile) >= rule.site_clearance) {
                            Some((tile, true))
                        } else {
                            self.cancel(world, resources, roster, id, tick);
                            continue;
                        }
                    } else {
                        None
                    };
                    if let Some((tile, at_leader)) = target {
                        match stamp_camp(resources, tile) {
                            Some(hearth) => {
                                if let Some(g) = self.groups.get_mut(&id) {
                                    g.placed = Some((hearth, tick));
                                }
                                if let Some(record) = resources.fission_log.get_mut(group.log) {
                                    record.settled_at_leader = at_leader;
                                }
                            }
                            None => self.cancel(world, resources, roster, id, tick),
                        }
                    }
                }
            }
        }
    }

    /// Dissolve settlement `sid` explicitly (its buildings stay) and retire its hearth.
    fn dissolve(resources: &mut SimResources, sid: SettlementId, tick: u64) {
        let Some(settlement) = resources.settlements.remove(&sid) else { return };
        let tile = settlement.formation_tile;
        resources.retired_hearths.insert(tile);
        resources.dissolutions.push((sid, tick));
        resources.building_owner.retain(|_, owner| *owner != sid);
        resources.site_owner.retain(|_, owner| *owner != sid);
        let id = resources.issue_event_id();
        let tile_idx = tile.1 * resources.tile_grid.width + tile.0;
        resources.causal_log.push(
            tile_idx,
            CausalEvent::SettlementDissolved {
                id,
                parent: None,
                settlement_id: sid,
                final_population: settlement.population_stats.current,
                cause: DissolutionCause::EmptyMembersAndBuildings,
                last_member_id: None,
                last_building_id: None,
                tick,
            },
        );
    }

    /// Hourly: remnants merge away, empty settlements dissolve.
    fn remnants(&mut self, resources: &mut SimResources, rule: &FissionRule, tick: u64) {
        let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        ids.sort_unstable();
        for sid in ids {
            let members = resources.settlements.get(&sid).map_or(0, |s| s.member_agents.len() as u32);
            if members == 0 {
                self.remnant_since.remove(&sid);
                Self::dissolve(resources, sid, tick);
            } else if members <= rule.remnant_members {
                let since = *self.remnant_since.entry(sid).or_insert(tick);
                if tick >= since + resources.ticks_per_day {
                    self.remnant_since.remove(&sid);
                    if let Some(settlement) = resources.settlements.get_mut(&sid) {
                        settlement.member_agents.clear();
                        settlement.population_stats.current = 0;
                    }
                    resources.tallies.remnant_merges += 1;
                    Self::dissolve(resources, sid, tick);
                }
            } else {
                self.remnant_since.remove(&sid);
            }
        }
    }

    /// Daily 06:00 check, settlements in id order.
    fn check(&mut self, world: &mut World, resources: &mut SimResources, rule: &FissionRule, roster: &Roster, tick: u64) {
        let tpd = resources.ticks_per_day;
        if season_at_tick(resources, tick).is_none_or(|(season, _)| season > 1) {
            return;
        }
        let cooldown = u64::from(rule.cooldown_days) * tpd;
        let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        ids.sort_unstable();
        for sid in ids {
            if self.world_last_fission.is_some_and(|last| tick < last + cooldown) {
                return;
            }
            let alive = resources.settlements.values().filter(|s| !s.member_agents.is_empty()).count() as u32
                + self.groups.len() as u32;
            if alive >= rule.max_settlements {
                return;
            }
            let Some(settlement) = resources.settlements.get(&sid) else { continue };
            let members = settlement.member_agents.len() as u32;
            if tick < settlement.founded_at + u64::from(rule.min_age_days) * tpd
                || self.last_fission.get(&sid).is_some_and(|last| tick < last + cooldown)
                || members < rule.min_members
                || self.groups.values().any(|g| g.parent == sid)
            {
                continue;
            }
            let Some(capacity) = sustain_capacity(resources, sid) else { continue };
            if f64::from(members) < rule.crowding * f64::from(capacity) {
                continue;
            }
            // Site: the free wild-region centre nearest the parent.
            let home = settlement.formation_tile;
            let site = resources
                .wild_regions
                .iter()
                .enumerate()
                .filter(|(_, region)| {
                    resources.settlements.values().all(|s| chebyshev(s.formation_tile, region.centre) >= rule.site_clearance)
                        && self.groups.values().all(|g| g.site != region.centre)
                })
                .min_by_key(|(index, region)| (chebyshev(region.centre, home), *index))
                .map(|(_, region)| region.centre);
            let Some(site) = site else {
                resources.tallies.fission_blocked_no_site += 1;
                continue;
            };
            // Group: non-elders ranked by the keyed draw (replicate in force).
            let replicate = resources.noise_replicate_for(tick);
            let mut ranked: Vec<(u64, AgentId, bool)> = settlement
                .member_agents
                .iter()
                .filter_map(|id| roster.get(id).map(|row| (*id, row)))
                .filter(|(_, row)| !row.3)
                .map(|(id, row)| (keyed_u64(resources.world_seed, STREAM_FISSION, row.4, tick, 0, replicate), id, row.2))
                .collect();
            ranked.sort_unstable();
            let k = ((f64::from(members) * rule.group_share + 0.5).floor() as u32).clamp(rule.group_min, rule.group_max);
            let adults: Vec<AgentId> = ranked.iter().filter(|r| r.2).map(|r| r.1).collect();
            if (adults.len() as u32) < rule.min_adults || (ranked.len() as u32) < k {
                continue;
            }
            let mut group: BTreeSet<AgentId> = adults.iter().take(rule.min_adults as usize).copied().collect();
            for (_, id, _) in &ranked {
                if group.len() as u32 >= k {
                    break;
                }
                group.insert(*id);
            }
            let leader = group.iter().copied().find(|id| roster.get(id).is_some_and(|row| row.2)).unwrap_or(adults[0]);
            self.next_id += 1;
            let fission_id = self.next_id;
            let (food, wood) = if rule.stock_share {
                let share = |held: u32| (u64::from(held) * u64::from(k) / u64::from(members.max(1))) as u32;
                let Some(parent) = resources.settlements.get_mut(&sid) else { continue };
                let food = share(parent.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0));
                let wood = share(parent.stockpile.get(&ResourceKind::Wood).copied().unwrap_or(0));
                (parent.withdraw(ResourceKind::Food, food), parent.withdraw(ResourceKind::Wood, wood))
            } else {
                (0, 0)
            };
            for id in &group {
                if let Some((entity, ..)) = roster.get(id) {
                    let _ = world.insert_one(*entity, Founding { site, fission_id, leader, since_tick: tick });
                    let walking = world.get::<&AgentState>(*entity).is_ok_and(|s| !matches!(*s, AgentState::Consuming { .. }));
                    if walking {
                        let _ = world.insert_one(*entity, AgentState::Idle);
                        let _ = world.remove_one::<SeekTarget>(*entity);
                    }
                }
                resources.haulers.remove(id);
            }
            let adults_in_group = group.iter().filter(|id| roster.get(id).is_some_and(|row| row.2)).count() as u32;
            resources.fission_log.push(FissionRecord {
                fission_id,
                parent: sid,
                site,
                depart_tick: tick,
                members: group.iter().copied().collect(),
                adults: adults_in_group,
                daughter: None,
                founded_tick: None,
                cancelled_tick: None,
                food_taken: food,
                wood_taken: wood,
                settled_at_leader: false,
            });
            self.groups.insert(
                fission_id,
                Group { parent: sid, site, members: group, leader, since: tick, food, wood, placed: None, log: resources.fission_log.len() - 1 },
            );
            self.last_fission.insert(sid, tick);
            self.world_last_fission = Some(tick);
        }
    }
}

impl RuntimeSystem for FissionSystem {
    fn name(&self) -> &str {
        "FissionSystem"
    }

    fn priority(&self) -> u32 {
        FISSION_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = resources.rules.society.fission else {
            return;
        };
        let tpd = resources.ticks_per_day;
        if tpd == 0 || resources.tile_grid.width == 0 {
            return;
        }
        let tick = resources.current_tick;
        let clock = tick.saturating_add(resources.rules.clock_offset_ticks) % tpd;
        let hourly = clock.is_multiple_of((tpd / 24).max(1));
        if self.groups.is_empty() && !hourly {
            return;
        }
        let roster = roster(world);
        if !self.groups.is_empty() {
            self.advance(world, resources, &rule, &roster, tick);
        }
        if hourly {
            self.remnants(resources, &rule, tick);
            if clock == CHECK_MINUTE * tpd / 1440 {
                self.check(world, resources, &rule, &roster, tick);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn metadata() {
        let s = FissionSystem::new();
        assert_eq!((s.name(), s.priority(), s.tick_interval()), ("FissionSystem", 148, 1));
    }
}
