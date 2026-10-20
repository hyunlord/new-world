//! Settlement reads: the settlement snapshot rows, the society snapshot
//! (lineage, founding, fission columns) and per-settlement wood targets.
//! Pure collectors plus the Godot marshallers `node.rs` forwards to.

use godot::prelude::*;
use sim_core::components::{Agent, Position, ResourceKind, Settlement, SettlementId};
use sim_engine::SimResources;

#[cfg(test)]
use super::engine_build::advance_engine_ticks;
#[cfg(test)]
use super::snapshot::collect_agent_snapshot;
#[cfg(test)]
use sim_core::rules::WorldRules;
#[cfg(test)]
use sim_systems::worldgen::{build_world_with, WorldGenConfig};

// ────────────────────────────────────────────────────────────────────────
// V7 Phase 12-γ: Settlement snapshot FFI surface
// ────────────────────────────────────────────────────────────────────────

/// Single row of the settlement snapshot returned by
/// [`collect_settlement_snapshot`].
///
/// V7 Phase 12-γ — surfaces `Settlement` entities to the GDScript renderer
/// with a substrate-derived centroid (mean position of member agents). The
/// substrate has no `Settlement.position` field; this is a derived UI
/// affordance that does NOT add or modify any sim-core data.
///
/// `member_count` is the count of *resolvable* member agents (those whose
/// `AgentId` matches a live `(Agent, Position)` pair in the ECS world).
/// Settlements with zero resolvable members are dropped by the collector
/// rather than emitted with a divide-by-zero centroid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SettlementSnapshotRow {
    /// `hecs::Entity::to_bits().get()` of the Settlement entity.
    pub entity_bits: u64,
    /// `Settlement::settlement_id`.
    pub settlement_id: u32,
    /// Floor of the mean tile-x of resolvable member agents.
    pub centroid_x: i32,
    /// Floor of the mean tile-y of resolvable member agents.
    pub centroid_y: i32,
    /// Count of resolvable member agents.
    pub member_count: u32,
    /// `Settlement::formation_tile.0` — the FIXED formation-anchor tile-x,
    /// set once at formation. Unlike `centroid_x` (live mean of members), this
    /// does not move, so the GDScript marker anchored to it sits still.
    pub formation_x: i32,
    /// `Settlement::formation_tile.1` — the FIXED formation-anchor tile-y.
    pub formation_y: i32,
    /// Settlement `stockpile` Food count (`ResourceKind::Food`), 0 when none
    /// stocked. Additive Direction-2 slice 2-5a field — surfaces the reserve
    /// the gather→deposit→consume chain (2-2/2-3a/2-4) fills and drains.
    pub food_stock: i32,
    /// Revitalize Pass 5 — settlement `stockpile` Wood count, 0 when none.
    pub wood_stock: i32,
}

/// Pure-Rust collector mirroring [`collect_construction_snapshot`] but
/// joining each Settlement to its member agents' Positions.
///
/// Iterates the authoritative `resources.settlements` store (a
/// `HashMap<SettlementId, Settlement>` — settlements are NEVER spawned as
/// ECS world entities), then for each settlement averages the `Position` of
/// every member agent resolvable via the `world` `(Agent, Position)` join.
/// Settlements whose `member_agents` set is empty OR whose members are all
/// stale (not present as `(Agent, Position)` in the world) are skipped —
/// emitting them would require dividing by zero. `entity_bits` carries
/// `settlement_id` (no ECS entity exists; the field is only a stable unique
/// key downstream). Rows are returned sorted by `settlement_id` for
/// deterministic ordering across runs.
pub fn collect_settlement_snapshot(
    world: &hecs::World,
    settlements: &std::collections::HashMap<SettlementId, Settlement>,
) -> Vec<SettlementSnapshotRow> {
    // First pass: build an `Agent.id` → `(x, y)` lookup. Settlement
    // members are referenced by AgentId, not hecs::Entity, so this
    // intermediate index is required.
    let mut agent_positions: std::collections::HashMap<u64, (u32, u32)> =
        std::collections::HashMap::new();
    for (_, (agent, pos)) in world.query::<(&Agent, &Position)>().iter() {
        agent_positions.insert(agent.id, (pos.x, pos.y));
    }

    let mut rows = Vec::new();
    for settlement in settlements.values() {
        let mut sum_x: u64 = 0;
        let mut sum_y: u64 = 0;
        let mut count: u32 = 0;
        for member_id in settlement.member_agents.iter() {
            if let Some(&(x, y)) = agent_positions.get(member_id) {
                sum_x += x as u64;
                sum_y += y as u64;
                count += 1;
            }
        }
        if count == 0 {
            continue;
        }
        let centroid_x = (sum_x / count as u64) as i32;
        let centroid_y = (sum_y / count as u64) as i32;
        rows.push(SettlementSnapshotRow {
            entity_bits: settlement.settlement_id as u64,
            settlement_id: settlement.settlement_id,
            centroid_x,
            centroid_y,
            member_count: count,
            formation_x: settlement.formation_tile.0 as i32,
            formation_y: settlement.formation_tile.1 as i32,
            food_stock: settlement
                .stockpile
                .get(&ResourceKind::Food)
                .copied()
                .unwrap_or(0) as i32,
            wood_stock: settlement
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0) as i32,
        });
    }
    // HashMap iteration order is not stable run-to-run; sort by the unique
    // settlement_id so the snapshot is deterministic (Day-1 invariant +
    // stable furniture-sprite keying downstream).
    rows.sort_by_key(|r| r.settlement_id);
    rows
}

#[derive(Debug, Default, PartialEq, Eq)]
pub(super) struct SocietySnapshot {
    settlement_ids: Vec<i32>,
    parent_ids: Vec<i32>,
    founded_ticks: Vec<i64>,
    member_counts: Vec<i32>,
    fission_pressures: Vec<i64>,
    fission_thresholds: Vec<i64>,
    home_agent_ids: Vec<i64>,
    home_settlement_ids: Vec<i32>,
    founder_agent_ids: Vec<i64>,
    founder_fission_ids: Vec<i32>,
    founder_site_xs: Vec<i32>,
    founder_site_ys: Vec<i32>,
    founder_leader: Vec<u8>,
    founder_since_ticks: Vec<i64>,
    fission_ids: Vec<i32>,
    fission_parents: Vec<i32>,
    fission_site_xs: Vec<i32>,
    fission_site_ys: Vec<i32>,
    fission_sizes: Vec<i32>,
    fission_depart_ticks: Vec<i64>,
    fission_founded_ticks: Vec<i64>,
    fission_cancelled_ticks: Vec<i64>,
    fission_daughters: Vec<i32>,
    fission_food_taken: Vec<i64>,
    retired_hearth_xs: Vec<i32>,
    retired_hearth_ys: Vec<i32>,
    current_tick: i64,
}

pub(super) fn collect_society_snapshot(
    world: &hecs::World,
    resources: &SimResources,
) -> SocietySnapshot {
    use sim_core::components::Founding;

    // Match the existing bridge's integer representation, including u64 AgentId bits.
    let mut snapshot = SocietySnapshot {
        current_tick: resources.current_tick as i64,
        ..SocietySnapshot::default()
    };
    for row in collect_settlement_snapshot(world, &resources.settlements) {
        let settlement = &resources.settlements[&row.settlement_id];
        snapshot
            .settlement_ids
            .push(settlement.settlement_id as i32);
        snapshot
            .parent_ids
            .push(settlement.parent.map_or(-1, |id| id as i32));
        snapshot.founded_ticks.push(settlement.founded_at as i64);
        snapshot
            .member_counts
            .push(settlement.member_agents.len() as i32);
        snapshot
            .fission_pressures
            .push(i64::try_from(settlement.fission_pressure).unwrap_or(i64::MAX));
        snapshot
            .fission_thresholds
            .push(i64::try_from(settlement.fission_threshold).unwrap_or(i64::MAX));
    }
    let mut homes: Vec<_> = resources
        .settlements
        .values()
        .flat_map(|settlement| {
            settlement
                .member_agents
                .iter()
                .map(|&agent_id| (agent_id, settlement.settlement_id))
        })
        .collect();
    homes.sort_unstable();
    for (agent_id, settlement_id) in homes {
        snapshot.home_agent_ids.push(agent_id as i64);
        snapshot.home_settlement_ids.push(settlement_id as i32);
    }
    let mut founders: Vec<_> = world
        .query::<(&Agent, &Founding)>()
        .iter()
        .map(|(_, (agent, founding))| (agent.id, *founding))
        .collect();
    founders.sort_unstable_by_key(|(agent_id, _)| *agent_id);
    for (agent_id, founding) in founders {
        snapshot.founder_agent_ids.push(agent_id as i64);
        snapshot
            .founder_fission_ids
            .push(founding.fission_id as i32);
        snapshot.founder_site_xs.push(founding.site.0 as i32);
        snapshot.founder_site_ys.push(founding.site.1 as i32);
        snapshot
            .founder_leader
            .push(u8::from(agent_id == founding.leader));
        snapshot
            .founder_since_ticks
            .push(founding.since_tick as i64);
    }
    let mut fissions: Vec<_> = resources.fission_log.iter().collect();
    fissions.sort_unstable_by_key(|record| record.fission_id);
    for record in fissions {
        snapshot.fission_ids.push(record.fission_id as i32);
        snapshot.fission_parents.push(record.parent as i32);
        snapshot.fission_site_xs.push(record.site.0 as i32);
        snapshot.fission_site_ys.push(record.site.1 as i32);
        snapshot.fission_sizes.push(record.members.len() as i32);
        snapshot
            .fission_depart_ticks
            .push(record.depart_tick as i64);
        snapshot
            .fission_founded_ticks
            .push(record.founded_tick.map_or(-1, |tick| tick as i64));
        snapshot
            .fission_cancelled_ticks
            .push(record.cancelled_tick.map_or(-1, |tick| tick as i64));
        snapshot
            .fission_daughters
            .push(record.daughter.map_or(-1, |id| id as i32));
        snapshot
            .fission_food_taken
            .push(i64::from(record.food_taken));
    }
    for &(x, y) in &resources.retired_hearths {
        snapshot.retired_hearth_xs.push(x as i32);
        snapshot.retired_hearth_ys.push(y as i32);
    }
    snapshot
}

pub(super) fn society_snapshot_to_dict(snapshot: &SocietySnapshot) -> VarDictionary {
    let mut dict = VarDictionary::new();
    for (key, values) in [
        ("settlement_ids", &snapshot.settlement_ids),
        ("parent_ids", &snapshot.parent_ids),
        ("member_counts", &snapshot.member_counts),
        ("home_settlement_ids", &snapshot.home_settlement_ids),
        ("founder_fission_ids", &snapshot.founder_fission_ids),
        ("founder_site_xs", &snapshot.founder_site_xs),
        ("founder_site_ys", &snapshot.founder_site_ys),
        ("fission_ids", &snapshot.fission_ids),
        ("fission_parents", &snapshot.fission_parents),
        ("fission_site_xs", &snapshot.fission_site_xs),
        ("fission_site_ys", &snapshot.fission_site_ys),
        ("fission_sizes", &snapshot.fission_sizes),
        ("fission_daughters", &snapshot.fission_daughters),
        ("retired_hearth_xs", &snapshot.retired_hearth_xs),
        ("retired_hearth_ys", &snapshot.retired_hearth_ys),
    ] {
        dict.set(key, PackedInt32Array::from(values.as_slice()));
    }
    for (key, values) in [
        ("founded_ticks", &snapshot.founded_ticks),
        ("fission_pressures", &snapshot.fission_pressures),
        ("fission_thresholds", &snapshot.fission_thresholds),
        ("home_agent_ids", &snapshot.home_agent_ids),
        ("founder_agent_ids", &snapshot.founder_agent_ids),
        ("founder_since_ticks", &snapshot.founder_since_ticks),
        ("fission_depart_ticks", &snapshot.fission_depart_ticks),
        ("fission_founded_ticks", &snapshot.fission_founded_ticks),
        ("fission_cancelled_ticks", &snapshot.fission_cancelled_ticks),
        ("fission_food_taken", &snapshot.fission_food_taken),
    ] {
        dict.set(key, PackedInt64Array::from(values.as_slice()));
    }
    dict.set(
        "founder_leader",
        PackedByteArray::from(snapshot.founder_leader.as_slice()),
    );
    dict.set("current_tick", snapshot.current_tick);
    dict
}

/// Marshal a [`SettlementSnapshotRow`] slice into the FFI dictionary
/// shape consumed by `WorldRenderer._update_settlement_furniture`.
/// Eight parallel `PackedArray`s, lengths always equal to `rows.len()`.
///
/// Keys:
/// - `ids`: `PackedInt64Array` — `entity_bits` per row.
/// - `settlement_ids`: `PackedInt32Array` — `settlement_id` per row.
/// - `centroid_xs`: `PackedInt32Array` — tile-x centroid.
/// - `centroid_ys`: `PackedInt32Array` — tile-y centroid.
/// - `member_counts`: `PackedInt32Array` — resolvable member count.
/// - `formation_xs`: `PackedInt32Array` — fixed formation-anchor tile-x.
/// - `formation_ys`: `PackedInt32Array` — fixed formation-anchor tile-y.
/// - `food_stocks`: `PackedInt32Array` — `stockpile` Food count (slice 2-5a).
pub(super) fn settlement_rows_to_dict(
    rows: &[SettlementSnapshotRow],
    fires: &std::collections::BTreeMap<SettlementId, sim_engine::FireState>,
    wood_targets: &[i32],
) -> VarDictionary {
    let n = rows.len();
    let mut ids = PackedInt64Array::new();
    let mut settlement_ids = PackedInt32Array::new();
    let mut centroid_xs = PackedInt32Array::new();
    let mut centroid_ys = PackedInt32Array::new();
    let mut member_counts = PackedInt32Array::new();
    let mut formation_xs = PackedInt32Array::new();
    let mut formation_ys = PackedInt32Array::new();
    let mut food_stocks = PackedInt32Array::new();
    ids.resize(n);
    settlement_ids.resize(n);
    centroid_xs.resize(n);
    centroid_ys.resize(n);
    member_counts.resize(n);
    formation_xs.resize(n);
    formation_ys.resize(n);
    food_stocks.resize(n);
    // Revitalize Pass 5 — wood stock and fire state (1 lit, 0 out or no fire).
    let mut wood_stocks = PackedInt32Array::new();
    let mut fire_lit = PackedByteArray::new();
    wood_stocks.resize(n);
    fire_lit.resize(n);
    for (i, row) in rows.iter().enumerate() {
        wood_stocks[i] = row.wood_stock;
        fire_lit[i] = u8::from(fires.get(&row.settlement_id).is_some_and(|fire| fire.lit));
    }
    for (i, row) in rows.iter().enumerate() {
        ids[i] = row.entity_bits as i64;
        settlement_ids[i] = row.settlement_id as i32;
        centroid_xs[i] = row.centroid_x;
        centroid_ys[i] = row.centroid_y;
        member_counts[i] = row.member_count as i32;
        formation_xs[i] = row.formation_x;
        formation_ys[i] = row.formation_y;
        food_stocks[i] = row.food_stock;
    }
    let mut dict = VarDictionary::new();
    dict.set("ids", ids);
    dict.set("settlement_ids", settlement_ids);
    dict.set("centroid_xs", centroid_xs);
    dict.set("centroid_ys", centroid_ys);
    dict.set("member_counts", member_counts);
    dict.set("formation_xs", formation_xs);
    dict.set("formation_ys", formation_ys);
    dict.set("food_stocks", food_stocks);
    dict.set("wood_stocks", wood_stocks);
    dict.set("fire_lit", fire_lit);
    dict.set("wood_targets", PackedInt32Array::from(wood_targets));
    dict
}

/// Pass 6 — floor of the wood target of every settlement row, in row order;
/// `0` for every row without the firewood rule.
pub fn collect_settlement_wood_targets(
    rows: &[SettlementSnapshotRow],
    resources: &SimResources,
) -> Vec<i32> {
    let target = sim_systems::runtime::settlement::firewood::wood_target(resources)
        .map_or(0, |target| {
            target.max(0.0).floor().min(f64::from(i32::MAX)) as i32
        });
    vec![target; rows.len()]
}

#[cfg(test)]
mod society_snapshot_tests {
    use super::*;
    use sim_core::components::Founding;
    use sim_engine::FissionRecord;

    #[test]
    fn society_joins_built_world_agents_and_preserves_order_filter_and_history() {
        let seed = 20_261_013;
        // Without fission pressure the pressure and threshold columns read 0.
        let mut rules = WorldRules::baseline(seed);
        if let Some(mut fission) = rules.society.fission {
            fission.pressure = None;
            rules.society.fission = Some(fission);
        }
        let mut engine = build_world_with(&WorldGenConfig::baseline(seed), rules);
        advance_engine_ticks(&mut engine, 1_440);
        let agent_rows = collect_agent_snapshot(&engine.world);
        let snapshot = collect_society_snapshot(&engine.world, &engine.resources);
        assert!(!snapshot.home_agent_ids.is_empty());
        assert_eq!(
            snapshot.fission_pressures,
            vec![0; snapshot.settlement_ids.len()]
        );
        assert_eq!(
            snapshot.fission_thresholds,
            vec![0; snapshot.settlement_ids.len()]
        );
        for (&agent_id, &home_id) in snapshot
            .home_agent_ids
            .iter()
            .zip(&snapshot.home_settlement_ids)
        {
            assert!(agent_rows.iter().any(|row| row.agent_id as i64 == agent_id));
            assert!(engine.resources.settlements[&(home_id as u32)]
                .member_agents
                .contains(&(agent_id as u64)));
        }
        assert_eq!(
            snapshot,
            collect_society_snapshot(&engine.world, &engine.resources)
        );

        let first = snapshot.home_agent_ids[0] as u64;
        let second = snapshot.home_agent_ids[1] as u64;
        let parent = snapshot.home_settlement_ids[0] as u32;
        let mut daughter = Settlement::new_with_id(20, 120);
        daughter.parent = Some(parent);
        daughter.fission_pressure = 850;
        daughter.fission_threshold = 1_200;
        daughter.add_member_agent(second);
        let parent_settlement = engine.resources.settlements.get_mut(&parent).unwrap();
        parent_settlement.member_agents.remove(&second);
        parent_settlement.fission_pressure = 325;
        parent_settlement.fission_threshold = 900;
        engine.resources.settlements.insert(20, daughter);
        let mut hidden = Settlement::new_with_id(30, 130);
        hidden.add_member_agent(u64::MAX);
        hidden.fission_pressure = 777;
        hidden.fission_threshold = 888;
        engine.resources.settlements.insert(30, hidden);
        let first_entity = engine
            .world
            .query::<&Agent>()
            .iter()
            .find(|(_, agent)| agent.id == first)
            .map(|(entity, _)| entity)
            .unwrap();
        engine
            .world
            .insert_one(
                first_entity,
                Founding {
                    site: (8, 9),
                    fission_id: 2,
                    leader: first,
                    since_tick: 100,
                },
            )
            .unwrap();
        engine.resources.fission_log = vec![
            FissionRecord {
                fission_id: 2,
                parent,
                site: (8, 9),
                depart_tick: 100,
                members: vec![first],
                adults: 1,
                daughter: None,
                founded_tick: None,
                cancelled_tick: None,
                food_taken: 23,
                wood_taken: 0,
                settled_at_leader: false,
                pressure_at_split: 0,
                threshold_at_split: 0,
            },
            FissionRecord {
                fission_id: 1,
                parent,
                site: (2, 3),
                depart_tick: 80,
                members: vec![second],
                adults: 1,
                daughter: Some(20),
                founded_tick: Some(120),
                cancelled_tick: Some(90),
                food_taken: 17,
                wood_taken: 0,
                settled_at_leader: false,
                pressure_at_split: 0,
                threshold_at_split: 0,
            },
        ];
        engine.resources.retired_hearths.extend([(9, 4), (2, 5)]);
        let snapshot = collect_society_snapshot(&engine.world, &engine.resources);
        let settlement_rows =
            collect_settlement_snapshot(&engine.world, &engine.resources.settlements);
        assert_eq!(
            snapshot.settlement_ids,
            settlement_rows
                .iter()
                .map(|row| row.settlement_id as i32)
                .collect::<Vec<_>>()
        );
        assert_eq!(snapshot.settlement_ids, vec![parent as i32, 20]);
        assert_eq!(snapshot.parent_ids, vec![-1, parent as i32]);
        assert_eq!(snapshot.fission_pressures, [325, 850]);
        assert_eq!(snapshot.fission_thresholds, [900, 1_200]);
        assert_eq!(
            snapshot,
            collect_society_snapshot(&engine.world, &engine.resources)
        );
        assert_eq!(snapshot.founded_ticks[1], 120);
        assert_eq!(
            snapshot.member_counts[0] as usize,
            engine.resources.settlements[&parent].member_agents.len()
        );
        let member_index = snapshot
            .home_agent_ids
            .iter()
            .position(|&id| id == first as i64)
            .unwrap();
        assert_eq!(snapshot.home_settlement_ids[member_index], parent as i32);
        assert_eq!(snapshot.home_agent_ids.last(), Some(&-1));
        assert_eq!(snapshot.home_settlement_ids.last(), Some(&30));
        assert_eq!(snapshot.founder_agent_ids, vec![first as i64]);
        assert_eq!(snapshot.founder_fission_ids, vec![2]);
        assert_eq!(snapshot.founder_site_xs, vec![8]);
        assert_eq!(snapshot.founder_site_ys, vec![9]);
        assert_eq!(snapshot.founder_leader, vec![1]);
        assert_eq!(snapshot.founder_since_ticks, vec![100]);
        assert_eq!(snapshot.fission_ids, vec![1, 2]);
        assert_eq!(snapshot.fission_parents, vec![parent as i32, parent as i32]);
        assert_eq!(snapshot.fission_site_xs, vec![2, 8]);
        assert_eq!(snapshot.fission_site_ys, vec![3, 9]);
        assert_eq!(snapshot.fission_sizes, vec![1, 1]);
        assert_eq!(snapshot.fission_depart_ticks, vec![80, 100]);
        assert_eq!(snapshot.fission_founded_ticks, vec![120, -1]);
        assert_eq!(snapshot.fission_cancelled_ticks, vec![90, -1]);
        assert_eq!(snapshot.fission_daughters, vec![20, -1]);
        assert_eq!(snapshot.fission_food_taken, vec![17, 23]);
        assert_eq!(snapshot.retired_hearth_xs, vec![2, 9]);
        assert_eq!(snapshot.retired_hearth_ys, vec![5, 4]);
        assert_eq!(snapshot.current_tick, engine.resources.current_tick as i64);
    }

    #[test]
    fn society_pressure_columns_read_v19_values_and_saturate_without_mutation() {
        let seed = 20_261_010;
        let mut engine =
            build_world_with(&WorldGenConfig::baseline(seed), WorldRules::baseline(seed));
        advance_engine_ticks(&mut engine, 1_440);
        let snapshot = collect_society_snapshot(&engine.world, &engine.resources);
        assert!(!snapshot.settlement_ids.is_empty());
        assert_eq!(
            snapshot.fission_pressures.len(),
            snapshot.settlement_ids.len()
        );
        assert_eq!(
            snapshot.fission_thresholds.len(),
            snapshot.settlement_ids.len()
        );
        for (index, &id) in snapshot.settlement_ids.iter().enumerate() {
            let settlement = &engine.resources.settlements[&(id as u32)];
            assert_eq!(
                snapshot.fission_pressures[index],
                i64::try_from(settlement.fission_pressure).unwrap()
            );
            assert_eq!(
                snapshot.fission_thresholds[index],
                i64::try_from(settlement.fission_threshold).unwrap()
            );
            assert!(snapshot.fission_thresholds[index] > 0);
        }
        assert_eq!(
            snapshot,
            collect_society_snapshot(&engine.world, &engine.resources)
        );
        let first = snapshot.settlement_ids[0] as u32;
        let settlement = engine.resources.settlements.get_mut(&first).unwrap();
        settlement.fission_pressure = u64::MAX;
        settlement.fission_threshold = u64::MAX;
        let saturated = collect_society_snapshot(&engine.world, &engine.resources);
        assert_eq!(saturated.fission_pressures[0], i64::MAX);
        assert_eq!(saturated.fission_thresholds[0], i64::MAX);
        assert_eq!(
            engine.resources.settlements[&first].fission_pressure,
            u64::MAX
        );
        assert_eq!(
            engine.resources.settlements[&first].fission_threshold,
            u64::MAX
        );
    }
}
