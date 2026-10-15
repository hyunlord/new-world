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
//!
//! **Pass 12 (`game_v18`).** The season gate reads `allowed_seasons` (v17:
//! spring and summer). With `hazard` set, the crowding gate is replaced, after
//! the capacity and before the site choice, by a daily draw: in thousandths,
//! g = clamp((members − size_lo) × 1000 / (size_hi − size_lo), 0, 1000),
//! c = members × 1000 / max(capacity, 1), q = clamp((c − crowd_lo) × 1000 /
//! (crowd_hi − crowd_lo), 0, 1000), need = max(1, floor(members ×
//! adult_need_per_day × scarcity_days)), s = clamp(1000 − food × 1000 / need,
//! 0, 1000) with food the settlement's own stockpile Food, h = max_daily × g ×
//! (weight_crowd × q + weight_scarcity × s) / 10^9; the settlement splits when
//! `keyed_below(seed, STREAM_FISSION_HAZARD, sid, tick, 0, r, 1000) < h`, r the
//! noise replicate in force. Every check that reaches the draw is logged in
//! `SimResources::fission_checks` (measurement only).
//!
//! **Pass 13 (`game_v19`).** With `pressure` set (it requires `hazard`) the
//! daily draw is not made: on a morning every gate before the site choice
//! passes (season, world cooldown, `max_settlements`, age, own cooldown,
//! `min_members`, departed group, capacity), the settlement's pressure P
//! (`Settlement::fission_pressure`, thousandths) grows by h, or, when h = 0,
//! falls by `decay_permille` (floor 0); on a morning any gate fails —
//! including the gates that end the check for every higher id (world
//! cooldown, `max_settlements`, a lower id splitting that morning) — P
//! neither grows nor decays. The settlement splits when P ≥ θ
//! (`Settlement::fission_threshold`) and P > 0 and a site is free; a split
//! blocked by no free site or by group formation keeps P and θ. After a
//! split P returns to 0 (`reset_on_split`) and θ is redrawn with k = the
//! parent's fission records (`fission_threshold`, a world draw: never the
//! replicate). θ is drawn when the settlement forms (`SettlementSystem`).

use std::collections::{BTreeMap, BTreeSet};

use hecs::World;
use sim_core::causal::event::{CausalEvent, DissolutionCause};
use sim_core::components::{
    Agent, AgentId, AgentState, BuildingKind, Child, Elder, Founding, Lineage, Position, ResourceKind, SeekTarget,
    SettlementId, Temperament, SETTLEMENT_FORMATION_AGENT_THRESHOLD, SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::fission_threshold::fission_threshold;
use sim_core::rng::{keyed_below, keyed_u64, keyed_unit_f64, STREAM_FISSION, STREAM_FISSION_HAZARD};
use sim_core::rules::{BirthRule, FissionHazard, FissionRule};
use sim_core::TerrainType;
use sim_engine::{BuildingPlacedEvent, FissionCheck, FissionRecord, RuntimeSystem, SimResources, RESOURCE_SOURCE_INFINITE};

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

/// Pass 12: the hazard terms (g, q, s, h) in thousandths for a settlement of
/// `members` with `capacity` (`sustain_capacity`) and `food` in its store;
/// `need_per_member` is `adult_need_per_day` (the only f64, floored once).
pub fn hazard_terms(hazard: &FissionHazard, members: u32, capacity: u32, food: u32, need_per_member: f64) -> (i64, i64, i64, i64) {
    let clamp = |x: i64| x.clamp(0, 1000);
    let m = i64::from(members);
    let g = clamp((m - i64::from(hazard.size_lo)) * 1000 / (i64::from(hazard.size_hi) - i64::from(hazard.size_lo)).max(1));
    let c = m * 1000 / i64::from(capacity.max(1));
    let q = clamp(
        (c - i64::from(hazard.crowd_lo_permille)) * 1000
            / (i64::from(hazard.crowd_hi_permille) - i64::from(hazard.crowd_lo_permille)).max(1),
    );
    let need = ((f64::from(members) * need_per_member * f64::from(hazard.scarcity_days)).floor() as i64).max(1);
    let s = clamp(1000 - i64::from(food) * 1000 / need);
    let h = i64::from(hazard.max_daily_permille) * g
        * (i64::from(hazard.weight_crowd_permille) * q + i64::from(hazard.weight_scarcity_permille) * s)
        / 1_000_000_000;
    (g, q, s, h)
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
                                    // Review round 1, M1: a group settling where its
                                    // leader stands walks to the new hearth, not to
                                    // the region centre it could not reach.
                                    if at_leader {
                                        g.site = hearth;
                                        for member in &alive {
                                            if let Some((entity, ..)) = roster.get(member) {
                                                if let Ok(mut founding) = world.get::<&mut Founding>(*entity) {
                                                    founding.site = hearth;
                                                }
                                            }
                                        }
                                    }
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
    fn dissolve(resources: &mut SimResources, sid: SettlementId, final_population: u32, tick: u64) {
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
                final_population,
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
                Self::dissolve(resources, sid, 0, tick);
            } else if members <= rule.remnant_members {
                let since = *self.remnant_since.entry(sid).or_insert(tick);
                if tick >= since + resources.ticks_per_day {
                    self.remnant_since.remove(&sid);
                    if let Some(settlement) = resources.settlements.get_mut(&sid) {
                        settlement.member_agents.clear();
                        settlement.population_stats.current = 0;
                    }
                    resources.tallies.remnant_merges += 1;
                    // Review round 1, L4: the event carries the members before the merge.
                    Self::dissolve(resources, sid, members, tick);
                }
            } else {
                self.remnant_since.remove(&sid);
            }
        }
    }

    /// Daily 06:00 check, settlements in id order.
    fn check(&mut self, world: &mut World, resources: &mut SimResources, rule: &FissionRule, roster: &Roster, tick: u64) {
        let tpd = resources.ticks_per_day;
        // Pass 12: the allowed seasons (v17: spring and summer, i.e. season <= 1).
        if season_at_tick(resources, tick).is_none_or(|(season, _)| !rule.allowed_seasons.get(season).copied().unwrap_or(false)) {
            return;
        }
        let mut checks: Vec<FissionCheck> = Vec::new();
        let cooldown = u64::from(rule.cooldown_days) * tpd;
        let mut ids: Vec<SettlementId> = resources.settlements.keys().copied().collect();
        ids.sort_unstable();
        for sid in ids {
            // Both ends of the check for every higher id (Pass 12: `break`
            // so the logged checks are kept; nothing follows the loop).
            if self.world_last_fission.is_some_and(|last| tick < last + cooldown) {
                break;
            }
            let alive = resources.settlements.values().filter(|s| !s.member_agents.is_empty()).count() as u32
                + self.groups.len() as u32;
            if alive >= rule.max_settlements {
                break;
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
            // Pass 12: the hazard draw replaces the crowding gate.
            let mut check = None;
            match rule.hazard {
                None => {
                    if f64::from(members) < rule.crowding * f64::from(capacity) {
                        continue;
                    }
                }
                Some(hazard) => {
                    let food = settlement.stockpile.get(&ResourceKind::Food).copied().unwrap_or(0);
                    let (g, q, s, h) = hazard_terms(&hazard, members, capacity, food, adult_need_per_day(tpd));
                    let (draw, split, pressure, threshold) = match rule.pressure {
                        None => {
                            let replicate = resources.noise_replicate_for(tick);
                            let draw = keyed_below(resources.world_seed, STREAM_FISSION_HAZARD, u64::from(sid), tick, 0, replicate, 1000);
                            (draw, (draw as i64) < h, 0, 0)
                        }
                        // Pass 13: accumulate (every gate passed today), no draw.
                        Some(law) => {
                            let k = resources.fission_log.iter().filter(|r| r.parent == sid).count() as u32;
                            let Some(settlement) = resources.settlements.get_mut(&sid) else { continue };
                            if settlement.fission_threshold == 0 {
                                // A settlement formed without its draw (e.g. deserialized
                                // from before Pass 13) draws it now, with the same key.
                                settlement.fission_threshold =
                                    fission_threshold(law.threshold_table, resources.world_seed, u64::from(sid), k);
                            }
                            settlement.fission_pressure = if h > 0 {
                                settlement.fission_pressure.saturating_add(h as u64)
                            } else {
                                settlement.fission_pressure.saturating_sub(u64::from(law.decay_permille))
                            };
                            let (p, theta) = (settlement.fission_pressure, settlement.fission_threshold);
                            (0, p > 0 && p >= theta, p, theta)
                        }
                    };
                    check = Some(FissionCheck {
                        tick,
                        settlement: sid,
                        members,
                        capacity,
                        food,
                        g,
                        q,
                        s,
                        h,
                        draw,
                        split,
                        site_free: false,
                        pressure,
                        threshold,
                    });
                }
            }
            let Some(settlement) = resources.settlements.get(&sid) else { continue };
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
            if let Some(mut check) = check {
                check.site_free = site.is_some();
                checks.push(check);
                if !check.split {
                    continue;
                }
            }
            let Some(site) = site else {
                resources.tallies.fission_blocked_no_site += 1;
                continue;
            };
            // Group: non-elders ranked by the keyed draw (replicate in force).
            let replicate = resources.noise_replicate_for(tick);
            // Pass 16 (`game_v20`, NS → who leaves): with temperament effects
            // the rank key is (unit draw − weight × ns / 1,000,000, AgentId),
            // sorted with `total_cmp` and the id tie-break (a member without
            // the component counts as NS 500); the u64 below is then the rank.
            // Without effects the v17–v19 path stays as it was.
            let departure = resources.rules.agent.temperament.filter(|rule| rule.effects);
            let ranked: Vec<(u64, AgentId, bool)> = if let Some(lean) = departure {
                let temperaments: BTreeMap<AgentId, Temperament> = world
                    .query::<(&Agent, &Temperament)>()
                    .iter()
                    .filter(|(_, (agent, _))| settlement.member_agents.contains(&agent.id))
                    .map(|(_, (agent, t))| (agent.id, *t))
                    .collect();
                let neutral = Temperament { ns: 500, ha: 500, rd: 500, p: 500 };
                let mut keyed: Vec<(f64, AgentId, bool)> = settlement
                    .member_agents
                    .iter()
                    .filter_map(|id| roster.get(id).map(|row| (*id, row)))
                    .filter(|(_, row)| !row.3)
                    .map(|(id, row)| {
                        let unit = keyed_unit_f64(resources.world_seed, STREAM_FISSION, row.4, tick, 0, replicate);
                        (lean.departure_key(unit, temperaments.get(&id).unwrap_or(&neutral)), id, row.2)
                    })
                    .collect();
                keyed.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
                keyed.into_iter().enumerate().map(|(rank, (_, id, adult))| (rank as u64, id, adult)).collect()
            } else {
                let mut ranked: Vec<(u64, AgentId, bool)> = settlement
                    .member_agents
                    .iter()
                    .filter_map(|id| roster.get(id).map(|row| (*id, row)))
                    .filter(|(_, row)| !row.3)
                    .map(|(id, row)| (keyed_u64(resources.world_seed, STREAM_FISSION, row.4, tick, 0, replicate), id, row.2))
                    .collect();
                ranked.sort_unstable();
                ranked
            };
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
            let Some(leader) = group.iter().copied().find(|id| roster.get(id).is_some_and(|row| row.2)) else {
                continue; // no adult in the group (only with `min_adults` 0)
            };
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
            let (pressure_at_split, threshold_at_split) =
                resources.settlements.get(&sid).map_or((0, 0), |s| (s.fission_pressure, s.fission_threshold));
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
                pressure_at_split,
                threshold_at_split,
            });
            // Pass 13: the parent's pressure resets and θ is redrawn (k now
            // counts this record).
            if let Some(law) = rule.pressure {
                let k = resources.fission_log.iter().filter(|r| r.parent == sid).count() as u32;
                let theta = fission_threshold(law.threshold_table, resources.world_seed, u64::from(sid), k);
                if let Some(parent) = resources.settlements.get_mut(&sid) {
                    if law.reset_on_split {
                        parent.fission_pressure = 0;
                    }
                    parent.fission_threshold = theta;
                }
            }
            self.groups.insert(
                fission_id,
                Group { parent: sid, site, members: group, leader, since: tick, food, wood, placed: None, log: resources.fission_log.len() - 1 },
            );
            self.last_fission.insert(sid, tick);
            self.world_last_fission = Some(tick);
        }
        resources.fission_checks.extend(checks);
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
    fn hazard_terms_follow_the_formula() {
        // The goal's start values (7.1), so the numbers do not move with tuning.
        let hz = FissionHazard {
            size_lo: 20,
            size_hi: 40,
            crowd_lo_permille: 850,
            crowd_hi_permille: 1050,
            scarcity_days: 2,
            weight_crowd_permille: 500,
            weight_scarcity_permille: 500,
            max_daily_permille: 120,
        };
        // need per member 1.0: 30 members, 2 days -> need 60.
        // g = (30 - 20) * 1000 / 20 = 500; c = 30000 / 30 = 1000; q = 150 * 1000 / 200 = 750;
        // s = 1000 - 30 * 1000 / 60 = 500; h = 120 * 500 * (500 * 750 + 500 * 500) / 1e9 = 37.
        assert_eq!(hazard_terms(&hz, 30, 30, 30, 1.0), (500, 750, 500, 37));
        // Capacity 0 counts as 1 (q = 1000); an empty store s = 1000; size at size_hi.
        assert_eq!(hazard_terms(&hz, 40, 0, 0, 1.0), (1000, 1000, 1000, 120));
        // At or below size_lo the hazard is 0 whatever else.
        assert_eq!(hazard_terms(&hz, 20, 1, 0, 1.0).3, 0);
        // A full store and room: s = 0, q = 0.
        assert_eq!(hazard_terms(&hz, 30, 100, 1000, 1.0), (500, 0, 0, 0));
    }

    #[test]
    fn metadata() {
        let s = FissionSystem::new();
        assert_eq!((s.name(), s.priority(), s.tick_interval()), ("FissionSystem", 148, 1));
    }
}
