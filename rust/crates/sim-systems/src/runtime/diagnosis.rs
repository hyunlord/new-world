//! DiagnosisSystem: evaluate settlement diagnosis checks once per in-game hour.
//!
//! Priority: 250 (after all gameplay systems). Interval: 60 ticks (1 hour).
//! Reads: World (Settlement), SimResources (colony, fires, current_tick, rules)
//! Writes: SimResources.diagnosis_cache, SimResources.requests_cache (presentation only)
//! Events: none
//!
//! Read-only fact providers for settlement subjects.
//! They never write, reserve or draw randomness (ADR 0016 C2, ADR 0055).

use sim_core::colony::diagnosis::{
    self, DiagCheck, DiagnosisData, NextSubject, SubjectKind, MAX_CHAIN_DEPTH,
};
use sim_core::colony::requests;
use sim_core::colony::{Designation, ZoneKind};
use sim_core::components::Settlement;
use sim_engine::{
    DiagnosisCache, DiagnosisChainEntry, DiagnosisResultEntry, RequestsCache, RuntimeSystem,
    SimResources,
};
use std::collections::BTreeMap;

/// Fact reader function type (clippy type_complexity).
type FactReaderBox<'a> = Box<dyn Fn(&str) -> Option<i64> + 'a>;

// ── Settlement fact reader ─────────────────────────────────────────────

/// Read a named fact for a settlement from the colony state and world.
/// Returns `None` for unknown fact names; every value is an integer.
fn settlement_fact(world: &hecs::World, resources: &SimResources, fact_name: &str) -> Option<i64> {
    if resources.rules.colony.is_none() {
        return Some(0);
    }
    let col = &resources.colony;
    match fact_name {
        "food_stored" => {
            let stockpile_tiles: std::collections::BTreeSet<(u32, u32)> = col
                .zones
                .values()
                .filter(|z| z.kind == ZoneKind::Stockpile)
                .flat_map(|z| z.tiles.iter().copied())
                .collect();
            let mut count: i64 = 0;
            for (pos, stack) in &col.ground_items {
                if stockpile_tiles.contains(pos) {
                    count += i64::from(stack.count);
                }
            }
            Some(count)
        }
        "winter_need" => {
            let members = first_settlement_member_count(world);
            let days = i64::from(resources.rules.season_days());
            Some(i64::from(members) * 2 * days)
        }
        "winter_food_short" => {
            let stored = settlement_fact(world, resources, "food_stored").unwrap_or(0);
            let need = settlement_fact(world, resources, "winter_need").unwrap_or(0);
            Some(if stored < need && need > 0 { 1 } else { 0 })
        }
        "firewood_stock" => {
            let mut count: i64 = 0;
            for stack in col.ground_items.values() {
                if is_firewood_kind(stack.kind) {
                    count += i64::from(stack.count);
                }
            }
            Some(count)
        }
        "adult_count" => Some(i64::from(first_settlement_member_count(world))),
        "idle_adult_count" => {
            let members = i64::from(first_settlement_member_count(world));
            let busy = col.reservations.len() as i64;
            Some((members - busy).max(0))
        }
        "is_winter" => {
            let tpd = resources.ticks_per_day;
            if tpd == 0 {
                return Some(0);
            }
            let tpy = resources.rules.calendar.ticks_per_year();
            if tpy == 0 {
                return Some(0);
            }
            let day_in_year = (resources.current_tick / tpd) % (tpy / tpd);
            let dps = u64::from(resources.rules.season_days());
            if dps == 0 {
                return Some(0);
            }
            let season = day_in_year / dps;
            Some(if season == 3 { 1 } else { 0 })
        }
        "ripe_plant_count" => {
            let count = col
                .designations
                .values()
                .filter(|d| matches!(d, Designation::Harvest))
                .count();
            Some(count as i64)
        }
        "stockpile_count" => {
            let count = col
                .zones
                .values()
                .filter(|z| z.kind == ZoneKind::Stockpile)
                .count();
            Some(count as i64)
        }
        "ground_items_nearby" => {
            let stockpile_tiles: std::collections::BTreeSet<(u32, u32)> = col
                .zones
                .values()
                .filter(|z| z.kind == ZoneKind::Stockpile)
                .flat_map(|z| z.tiles.iter().copied())
                .collect();
            let count: i64 = col
                .ground_items
                .iter()
                .filter(|(pos, _)| !stockpile_tiles.contains(pos))
                .map(|(_, s)| i64::from(s.count))
                .sum();
            Some(count)
        }
        "haulers_assigned" => Some(col.reservations.len() as i64),
        "free_cells" => {
            let stockpile_tiles: std::collections::BTreeSet<(u32, u32)> = col
                .zones
                .values()
                .filter(|z| z.kind == ZoneKind::Stockpile)
                .flat_map(|z| z.tiles.iter().copied())
                .collect();
            let occupied = stockpile_tiles
                .iter()
                .filter(|pos| col.ground_items.contains_key(pos))
                .count();
            Some((stockpile_tiles.len() - occupied) as i64)
        }
        "lit" => {
            let any_lit = resources.fires.values().any(|f| f.lit);
            Some(if any_lit { 1 } else { 0 })
        }
        "fuel_at_hearth" => Some(resources.fires.values().filter(|f| f.lit).count() as i64),
        "fuel_stock_days" => {
            let fw = settlement_fact(world, resources, "firewood_stock").unwrap_or(0);
            Some(fw)
        }
        "roofed" => {
            let has_roof = col.rooms.values().any(|r| !r.tiles.is_empty() && r.roofed);
            Some(if has_roof { 1 } else { 0 })
        }
        "bed_count" => Some(col.beds.len() as i64),
        _ => None,
    }
}

/// Count members of the first settlement in the world.
fn first_settlement_member_count(world: &hecs::World) -> u32 {
    world
        .query::<&Settlement>()
        .iter()
        .next()
        .map(|(_, s)| s.member_agents.len() as u32)
        .unwrap_or(0)
}

/// Check if an item kind is firewood by name.
fn is_firewood_kind(kind: sim_core::colony::ItemKind) -> bool {
    let name = kind.name();
    name == "Firewood" || name == "Log"
}

// ── Person fact reader ─────────────────────────────────────────────────

/// Read a named fact for a person from ECS components. Reuses the same fact
/// names as the mind's BUILTIN_PROVIDERS (hunger_tenths, thirst_tenths, cold,
/// etc.) by reading the corresponding ECS components directly.
fn person_fact(world: &hecs::World, entity: hecs::Entity, fact_name: &str) -> Option<i64> {
    match fact_name {
        "hunger_tenths" => {
            let hunger = world.get::<&sim_core::components::Hunger>(entity).ok()?;
            Some((f64::from(hunger.value) * 10.0).round() as i64)
        }
        "thirst_tenths" => {
            let thirst = world.get::<&sim_core::components::Thirst>(entity).ok()?;
            Some((thirst.value * 10.0).round() as i64)
        }
        "cold" => {
            let has_cold = world.get::<&sim_core::components::Cold>(entity).is_ok();
            Some(if has_cold { 1 } else { 0 })
        }
        "fleeing" => {
            let has_flee = world
                .get::<&sim_core::colony::people::Fleeing>(entity)
                .is_ok();
            Some(if has_flee { 1 } else { 0 })
        }
        "drafted" => {
            let has_draft = world
                .get::<&sim_core::colony::people::Drafted>(entity)
                .is_ok();
            Some(if has_draft { 1 } else { 0 })
        }
        "idle" => {
            let state = world
                .get::<&sim_core::components::AgentState>(entity)
                .ok()?;
            Some(if *state == sim_core::components::AgentState::Idle {
                1
            } else {
                0
            })
        }
        _ => None,
    }
}

/// Evaluate all person checks for an entity, returning the failing ones with chains.
pub fn evaluate_person(
    world: &hecs::World,
    resources: &SimResources,
    entity: hecs::Entity,
    data: &DiagnosisData,
) -> Vec<DiagnosisResultEntry> {
    let checks = match data.subjects.get(&SubjectKind::Person) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let reader = |f: &str| person_fact(world, entity, f);
    let mut results = Vec::new();
    for check in checks {
        if let Some((params, true)) = evaluate_check(check, &reader) {
            let chain = build_chain(world, resources, data, &check.next, 0);
            results.push(DiagnosisResultEntry {
                check_id: check.id.clone(),
                text_key: check.text_key.clone(),
                severity: check.severity.code(),
                params,
                tags: check.tags.clone(),
                chain,
            });
        }
    }
    results
}

// ── Stockpile fact reader ──────────────────────────────────────────────

/// Read a named fact for a stockpile zone by zone id.
fn stockpile_fact(resources: &SimResources, zone_id: u32, fact_name: &str) -> Option<i64> {
    let col = &resources.colony;
    let zone = col.zones.get(&zone_id)?;
    if zone.kind != ZoneKind::Stockpile {
        return None;
    }
    match fact_name {
        "food_stored" => {
            let mut count: i64 = 0;
            for pos in &zone.tiles {
                if let Some(stack) = col.ground_items.get(pos) {
                    count += i64::from(stack.count);
                }
            }
            Some(count)
        }
        "free_cells" => {
            let occupied = zone
                .tiles
                .iter()
                .filter(|pos| col.ground_items.contains_key(pos))
                .count();
            Some((zone.tiles.len() - occupied) as i64)
        }
        "ground_items_nearby" => {
            // Items on ground within 8 tiles of the zone but not in any stockpile.
            let stockpile_tiles: std::collections::BTreeSet<(u32, u32)> = col
                .zones
                .values()
                .filter(|z| z.kind == ZoneKind::Stockpile)
                .flat_map(|z| z.tiles.iter().copied())
                .collect();
            let center = zone.tiles.iter().next().copied().unwrap_or((0, 0));
            let count: i64 = col
                .ground_items
                .iter()
                .filter(|(pos, _)| {
                    !stockpile_tiles.contains(pos)
                        && (pos.0 as i64 - center.0 as i64).unsigned_abs() <= 8
                        && (pos.1 as i64 - center.1 as i64).unsigned_abs() <= 8
                })
                .map(|(_, s)| i64::from(s.count))
                .sum();
            Some(count)
        }
        "haulers_assigned" => Some(col.reservations.len() as i64),
        _ => None,
    }
}

/// Evaluate stockpile checks for a zone.
pub fn evaluate_stockpile(
    world: &hecs::World,
    resources: &SimResources,
    zone_id: u32,
    data: &DiagnosisData,
) -> Vec<DiagnosisResultEntry> {
    let checks = match data.subjects.get(&SubjectKind::Stockpile) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let reader = |f: &str| stockpile_fact(resources, zone_id, f);
    let mut results = Vec::new();
    for check in checks {
        if let Some((params, true)) = evaluate_check(check, &reader) {
            let chain = build_chain(world, resources, data, &check.next, 0);
            results.push(DiagnosisResultEntry {
                check_id: check.id.clone(),
                text_key: check.text_key.clone(),
                severity: check.severity.code(),
                params,
                tags: check.tags.clone(),
                chain,
            });
        }
    }
    results
}

// ── Hearth fact reader ────────────────────────────────────────────────

/// Read a named fact for a hearth by settlement id.
fn hearth_fact(resources: &SimResources, settlement_id: u32, fact_name: &str) -> Option<i64> {
    let fire = resources.fires.get(&settlement_id);
    match fact_name {
        "lit" => Some(if fire.is_some_and(|f| f.lit) { 1 } else { 0 }),
        "fuel_at_hearth" => Some(fire.map_or(0, |_| 1)),
        "fuel_stock_days" => {
            let mut fw: i64 = 0;
            for stack in resources.colony.ground_items.values() {
                if is_firewood_kind(stack.kind) {
                    fw += i64::from(stack.count);
                }
            }
            Some(fw)
        }
        _ => None,
    }
}

/// Evaluate hearth checks for a settlement.
pub fn evaluate_hearth(
    world: &hecs::World,
    resources: &SimResources,
    settlement_id: u32,
    data: &DiagnosisData,
) -> Vec<DiagnosisResultEntry> {
    let checks = match data.subjects.get(&SubjectKind::Hearth) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let reader = |f: &str| hearth_fact(resources, settlement_id, f);
    let mut results = Vec::new();
    for check in checks {
        if let Some((params, true)) = evaluate_check(check, &reader) {
            let chain = build_chain(world, resources, data, &check.next, 0);
            results.push(DiagnosisResultEntry {
                check_id: check.id.clone(),
                text_key: check.text_key.clone(),
                severity: check.severity.code(),
                params,
                tags: check.tags.clone(),
                chain,
            });
        }
    }
    results
}

// ── Room fact reader ──────────────────────────────────────────────────

/// Read a named fact for a room by room id.
fn room_fact(resources: &SimResources, room_id: u32, fact_name: &str) -> Option<i64> {
    let col = &resources.colony;
    let room = col.rooms.get(&room_id)?;
    match fact_name {
        "roofed" => Some(if room.roofed { 1 } else { 0 }),
        "bed_count" => {
            let count = room
                .tiles
                .iter()
                .filter(|tile| col.beds.contains_key(tile))
                .count();
            Some(count as i64)
        }
        _ => None,
    }
}

/// Evaluate room checks for a room.
pub fn evaluate_room(
    world: &hecs::World,
    resources: &SimResources,
    room_id: u32,
    data: &DiagnosisData,
) -> Vec<DiagnosisResultEntry> {
    let checks = match data.subjects.get(&SubjectKind::Room) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let reader = |f: &str| room_fact(resources, room_id, f);
    let mut results = Vec::new();
    for check in checks {
        if let Some((params, true)) = evaluate_check(check, &reader) {
            let chain = build_chain(world, resources, data, &check.next, 0);
            results.push(DiagnosisResultEntry {
                check_id: check.id.clone(),
                text_key: check.text_key.clone(),
                severity: check.severity.code(),
                params,
                tags: check.tags.clone(),
                chain,
            });
        }
    }
    results
}

// ── Check evaluation ───────────────────────────────────────────────────

/// Evaluate a single check against a fact reader.
fn evaluate_check(
    check: &DiagCheck,
    fact_reader: &dyn Fn(&str) -> Option<i64>,
) -> Option<(BTreeMap<String, String>, bool)> {
    for pred in &check.when {
        let actual = fact_reader(&pred.fact)?;
        if !pred.op.matches(actual, pred.threshold) {
            return Some((BTreeMap::new(), false));
        }
    }
    let mut params = BTreeMap::new();
    for (name, fact_name) in &check.params {
        if let Some(value) = fact_reader(fact_name) {
            params.insert(name.clone(), value.to_string());
        }
    }
    Some((params, true))
}

/// Evaluate all settlement checks, returning the failing ones with chains.
pub fn evaluate_settlement(
    world: &hecs::World,
    resources: &SimResources,
    data: &DiagnosisData,
) -> Vec<DiagnosisResultEntry> {
    let checks = match data.subjects.get(&SubjectKind::Settlement) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let reader = |f: &str| settlement_fact(world, resources, f);
    let mut results = Vec::new();
    for check in checks {
        if let Some((params, true)) = evaluate_check(check, &reader) {
            let chain = build_chain(world, resources, data, &check.next, 0);
            results.push(DiagnosisResultEntry {
                check_id: check.id.clone(),
                text_key: check.text_key.clone(),
                severity: check.severity.code(),
                params,
                tags: check.tags.clone(),
                chain,
            });
        }
    }
    results
}

/// Follow a diagnosis chain to depth <= MAX_CHAIN_DEPTH.
fn build_chain(
    world: &hecs::World,
    resources: &SimResources,
    data: &DiagnosisData,
    next: &Option<NextSubject>,
    depth: usize,
) -> Vec<DiagnosisChainEntry> {
    if depth >= MAX_CHAIN_DEPTH {
        return Vec::new();
    }
    let next = match next {
        Some(n) => n,
        None => return Vec::new(),
    };
    let checks = match data.subjects.get(&next.kind) {
        Some(c) => c,
        None => return Vec::new(),
    };
    let reader: FactReaderBox<'_> = Box::new(|f: &str| settlement_fact(world, resources, f));
    let mut entries = Vec::new();
    for check in checks {
        if let Some((params, true)) = evaluate_check(check, reader.as_ref()) {
            entries.push(DiagnosisChainEntry {
                subject_kind: next.kind.code(),
                check_id: check.id.clone(),
                text_key: check.text_key.clone(),
                severity: check.severity.code(),
                params,
            });
            let deeper = build_chain(world, resources, data, &check.next, depth + 1);
            entries.extend(deeper);
            break;
        }
    }
    entries
}

// ── Systems ────────────────────────────────────────────────────────────

/// Hourly settlement diagnosis evaluation.
pub struct DiagnosisSystem;

impl RuntimeSystem for DiagnosisSystem {
    fn name(&self) -> &str {
        "DiagnosisSystem"
    }
    fn priority(&self) -> u32 {
        250
    }
    fn tick_interval(&self) -> u64 {
        60
    }
    fn tick_offset(&self, _resources: &SimResources) -> u64 {
        7
    }
    fn tick(&mut self, world: &mut hecs::World, resources: &mut SimResources) {
        if resources.rules.colony.is_none() {
            return;
        }
        let data = match diagnosis::diagnosis_data() {
            Some(d) => d,
            None => return,
        };
        let entries = evaluate_settlement(world, resources, data);
        let mut results = BTreeMap::new();
        results.insert(0u32, entries);
        resources.diagnosis_cache = DiagnosisCache {
            tick: resources.current_tick,
            results,
        };
    }
}

/// Hourly request aggregation from why-not records.
pub struct RequestsSystem;

impl RuntimeSystem for RequestsSystem {
    fn name(&self) -> &str {
        "RequestsSystem"
    }
    fn priority(&self) -> u32 {
        251
    }
    fn tick_interval(&self) -> u64 {
        60
    }
    fn tick_offset(&self, _resources: &SimResources) -> u64 {
        8
    }
    fn tick(&mut self, _world: &mut hecs::World, resources: &mut SimResources) {
        if resources.rules.colony.is_none() {
            return;
        }
        let request_data = match requests::requests_data() {
            Some(d) => d,
            None => return,
        };
        let steward_on = resources.steward.enabled;
        let current_hour = resources.current_tick / 60;

        let mut rule_counts: BTreeMap<usize, (u32, sim_core::components::AgentId)> =
            BTreeMap::new();
        for (agent_id, wn) in &resources.mind.why_not {
            if !wn.is_active(
                current_hour,
                requests::WINDOW_HOURS as u32,
                requests::ACTIVE_THRESHOLD,
            ) {
                continue;
            }
            for entry in wn.latest().iter().take(2) {
                if let Some(indices) = request_data.fact_index.get(&entry.fact_name) {
                    for &idx in indices {
                        let e = rule_counts.entry(idx).or_insert((0, *agent_id));
                        e.0 += 1;
                    }
                }
            }
        }

        let mut active: Vec<sim_engine::ActiveRequest> = Vec::new();
        for (idx, (count, voice)) in &rule_counts {
            if *count == 0 {
                continue;
            }
            let rule = &request_data.rules[*idx];
            let is_steward = steward_on && !rule.steward_tags.is_empty();
            active.push(sim_engine::ActiveRequest {
                rule_id: rule.id.clone(),
                text_key: rule.text_key.clone(),
                action_key: rule.action_key.clone(),
                priority: rule.priority,
                count: *count,
                voice: *voice,
                voice_name: String::new(),
                steward_active: is_steward,
                tags: rule.steward_tags.clone(),
            });
        }
        active.sort_by(|a, b| a.priority.cmp(&b.priority).then(b.count.cmp(&a.count)));
        active.truncate(requests::MAX_SHOWN);

        let mut results = BTreeMap::new();
        results.insert(0u32, active);
        resources.requests_cache = RequestsCache {
            tick: resources.current_tick,
            results,
        };
    }
}
