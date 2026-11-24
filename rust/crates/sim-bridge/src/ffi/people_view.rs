//! PERF2-1 B: the people layer's packed per-tick view (`WorldSimNode::get_people_view`).
//!
//! One read per tick: every column the 3D people layer draws from, struct-of-arrays in AgentId order, under the
//! same keys as `get_agent_snapshot` (plus the ACTION1 visual keys and the elder log seat of `mocap3d.gd`), and
//! the rows that changed since the previously built view: `moved` (only the tile changed), `changed` (anything
//! else, or new) and `removed` (AgentIds gone). `serial` / `base_serial` let a reader that missed a view fall
//! back to a full refresh; `full` is true for the first view of a world. Building it never touches the simulation.

use super::activity::collect_activity_snapshot;
use super::appearance::collect_appearance;
use super::colony::collect_agent_jobs;
use super::snapshot::{
    collect_agent_ages_years, collect_agent_infancy_and_pregnancy,
    collect_agent_snapshot_with_resources,
};
use super::story::collect_agent_m5_columns;
use godot::prelude::*;
use sim_core::colony::content::installed_or_bundled;
use sim_engine::SimEngine;
use std::collections::{HashMap, HashSet};

/// The activity kind of an elder's fire seat (`get_activity_snapshot` kind 6, SitByFire).
const SIT_BY_FIRE: u8 = 6;

/// One person's drawn inputs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub(crate) struct PeopleRow {
    pub agent_id: u64,
    pub bits: u64,
    pub x: u32,
    pub y: u32,
    pub state: u8,
    pub target_kind: u8,
    pub carried_wood: i32,
    pub life_stage: u8,
    pub age_years: i32,
    pub infant: u8,
    pub job: u8,
    pub carry_kind: i64,
    pub top: u8,
    pub sex: u8,
    pub look: i32,
    pub variant: u8,
    pub visual: String,
    pub fallback: String,
    pub log_seat: u8,
}

impl PeopleRow {
    /// Every field but the tile equal.
    fn same_but_tile(&self, other: &PeopleRow) -> bool {
        self.agent_id == other.agent_id
            && self.bits == other.bits
            && self.state == other.state
            && self.target_kind == other.target_kind
            && self.carried_wood == other.carried_wood
            && self.life_stage == other.life_stage
            && self.age_years == other.age_years
            && self.infant == other.infant
            && self.job == other.job
            && self.carry_kind == other.carry_kind
            && self.top == other.top
            && self.sex == other.sex
            && self.look == other.look
            && self.variant == other.variant
            && self.visual == other.visual
            && self.fallback == other.fallback
            && self.log_seat == other.log_seat
    }
}

/// Row indices of `rows` that moved (tile only) or changed (anything else, or new), and the AgentIds of
/// `previous` that are gone.
pub(crate) fn people_diff(
    previous: &HashMap<u64, PeopleRow>,
    rows: &[PeopleRow],
) -> (Vec<i32>, Vec<i32>, Vec<i64>) {
    let mut moved = Vec::new();
    let mut changed = Vec::new();
    for (i, row) in rows.iter().enumerate() {
        let index = i32::try_from(i).unwrap_or(i32::MAX);
        match previous.get(&row.agent_id) {
            Some(old) if old == row => {}
            Some(old) if old.same_but_tile(row) => moved.push(index),
            _ => changed.push(index),
        }
    }
    let present: HashSet<u64> = rows.iter().map(|row| row.agent_id).collect();
    let mut removed: Vec<i64> = previous
        .keys()
        .filter(|id| !present.contains(id))
        .map(|&id| i64::try_from(id).unwrap_or(i64::MAX))
        .collect();
    removed.sort_unstable();
    (moved, changed, removed)
}

/// Every person's row, in AgentId order (the collectors share the snapshot's row order).
pub(crate) fn collect_people_rows(engine: &SimEngine) -> Vec<PeopleRow> {
    let world = &engine.world;
    let resources = &engine.resources;
    let tick = engine.current_tick();
    let base = collect_agent_snapshot_with_resources(world, resources);
    let years = collect_agent_ages_years(world, tick, resources.rules.calendar.ticks_per_year());
    let (infants, _) = collect_agent_infancy_and_pregnancy(world, resources, tick);
    let jobs = collect_agent_jobs(world);
    let m5 = collect_agent_m5_columns(world, resources);
    let looks = collect_appearance(world);
    let visuals = installed_or_bundled().action_visuals();
    let keys: HashMap<u64, (String, String)> = sim_systems::visible_action_rows(world, resources)
        .iter()
        .map(|row| {
            let visual = visuals.resolve(row.action, row.phase, row.recipe);
            (
                row.agent_id,
                (visual.key.to_string(), visual.fallback.to_string()),
            )
        })
        .collect();
    let seats: HashSet<u64> = collect_activity_snapshot(world)
        .iter()
        .filter(|row| row.kind == SIT_BY_FIRE && row.until_tick.is_some_and(|until| until > tick))
        .map(|row| row.entity_bits)
        .collect();
    let mut rows: Vec<PeopleRow> = base
        .iter()
        .enumerate()
        .map(|(i, row)| {
            let (visual, fallback) = keys.get(&row.agent_id).cloned().unwrap_or_default();
            PeopleRow {
                agent_id: row.agent_id,
                bits: row.entity_bits,
                x: row.x,
                y: row.y,
                state: row.state_tag,
                target_kind: row.target_kind,
                carried_wood: row.carried_wood,
                life_stage: row.life_stage,
                age_years: years.get(i).copied().unwrap_or(-1),
                infant: infants.get(i).copied().unwrap_or(0),
                job: jobs.get(i).map_or(0, |job| job.0),
                carry_kind: jobs.get(i).map_or(-1, |job| job.1),
                top: m5.get(i).map_or(0, |columns| columns.0),
                sex: looks.get(i).map_or(0, |look| look.sex),
                look: looks.get(i).map_or(0, |look| look.look as i32),
                variant: looks.get(i).map_or(0, |look| look.variant),
                visual,
                fallback,
                log_seat: u8::from(seats.contains(&row.entity_bits)),
            }
        })
        .collect();
    rows.sort_by_key(|row| row.agent_id);
    rows
}

/// The cached view and the rows it was built from.
#[derive(Default)]
pub(crate) struct PeopleViewCache {
    key: Option<(u64, i64, usize)>,
    view: Option<VarDictionary>,
    serial: i64,
    generation: Option<i64>,
    rows: HashMap<u64, PeopleRow>,
}

/// The view of `engine` now, from `cache`; built once per (tick, world generation, command count).
pub(crate) fn cached_people_view_to_dict(
    cache: &mut PeopleViewCache,
    engine: &SimEngine,
    generation: i64,
) -> VarDictionary {
    let tick = engine.current_tick();
    let key = (tick, generation, engine.resources.colony.command_log.len());
    if cache.key == Some(key) {
        if let Some(view) = &cache.view {
            return view.clone();
        }
    }
    let full = cache.generation != Some(generation);
    if full {
        cache.rows.clear();
    }
    let rows = collect_people_rows(engine);
    let (moved, changed, removed) = people_diff(&cache.rows, &rows);
    let base_serial = if full { -1 } else { cache.serial };
    cache.serial += 1;
    let view = people_view_to_dict(
        &rows,
        &moved,
        &changed,
        &removed,
        tick,
        cache.serial,
        base_serial,
        full,
    );
    cache.rows = rows.into_iter().map(|row| (row.agent_id, row)).collect();
    cache.generation = Some(generation);
    cache.key = Some(key);
    cache.view = Some(view.clone());
    view
}

#[allow(clippy::too_many_arguments)]
fn people_view_to_dict(
    rows: &[PeopleRow],
    moved: &[i32],
    changed: &[i32],
    removed: &[i64],
    tick: u64,
    serial: i64,
    base_serial: i64,
    full: bool,
) -> VarDictionary {
    let as_i64 = |v: u64| i64::from_ne_bytes(v.to_ne_bytes());
    let mut visual_keys = PackedStringArray::new();
    let mut fallback_keys = PackedStringArray::new();
    for row in rows {
        visual_keys.push(&GString::from(row.visual.as_str()));
        fallback_keys.push(&GString::from(row.fallback.as_str()));
    }
    let i64s = |f: &dyn Fn(&PeopleRow) -> i64| -> PackedInt64Array {
        rows.iter().map(f).collect::<Vec<i64>>().as_slice().into()
    };
    let i32s = |f: &dyn Fn(&PeopleRow) -> i32| -> PackedInt32Array {
        rows.iter().map(f).collect::<Vec<i32>>().as_slice().into()
    };
    let u8s = |f: &dyn Fn(&PeopleRow) -> u8| -> PackedByteArray {
        rows.iter().map(f).collect::<Vec<u8>>().as_slice().into()
    };
    let mut dict = VarDictionary::new();
    dict.set("tick", i64::try_from(tick).unwrap_or(i64::MAX));
    dict.set("serial", serial);
    dict.set("base_serial", base_serial);
    dict.set("full", full);
    dict.set("ids", i64s(&|r| as_i64(r.bits)));
    dict.set("agent_ids", i64s(&|r| as_i64(r.agent_id)));
    dict.set("xs", i32s(&|r| i32::try_from(r.x).unwrap_or(i32::MAX)));
    dict.set("ys", i32s(&|r| i32::try_from(r.y).unwrap_or(i32::MAX)));
    dict.set("states", u8s(&|r| r.state));
    dict.set("target_kinds", u8s(&|r| r.target_kind));
    dict.set("carried_woods", i32s(&|r| r.carried_wood));
    dict.set("life_stages", u8s(&|r| r.life_stage));
    dict.set("ages_years", i32s(&|r| r.age_years));
    dict.set("infants", u8s(&|r| r.infant));
    dict.set("jobs", u8s(&|r| r.job));
    dict.set("carry_kinds", i64s(&|r| r.carry_kind));
    dict.set("tops", u8s(&|r| r.top));
    dict.set("sexes", u8s(&|r| r.sex));
    dict.set("look_ids", i32s(&|r| r.look));
    dict.set("variants", u8s(&|r| r.variant));
    dict.set("visual_keys", visual_keys);
    dict.set("fallback_keys", fallback_keys);
    dict.set("log_seats", u8s(&|r| r.log_seat));
    dict.set("moved", PackedInt32Array::from(moved));
    dict.set("changed", PackedInt32Array::from(changed));
    dict.set("removed", PackedInt64Array::from(removed));
    dict
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(id: u64, x: u32, look: i32) -> PeopleRow {
        PeopleRow {
            agent_id: id,
            bits: id + 100,
            x,
            y: 1,
            look,
            ..PeopleRow::default()
        }
    }

    #[test]
    fn diff_splits_moved_changed_and_removed() {
        let previous: HashMap<u64, PeopleRow> =
            [row(1, 5, 0), row(2, 5, 0), row(3, 5, 0), row(4, 5, 0)]
                .into_iter()
                .map(|r| (r.agent_id, r))
                .collect();
        let now = vec![row(1, 5, 0), row(2, 6, 0), row(3, 5, 7), row(5, 1, 0)];
        let (moved, changed, removed) = people_diff(&previous, &now);
        assert_eq!(moved, vec![1]);
        assert_eq!(changed, vec![2, 3]);
        assert_eq!(removed, vec![4]);
    }

    #[test]
    fn diff_moved_and_changed_together_is_changed() {
        let previous: HashMap<u64, PeopleRow> = [(1, row(1, 5, 0))].into_iter().collect();
        let mut next = row(1, 6, 0);
        next.visual = "chop".into();
        let (moved, changed, removed) = people_diff(&previous, &[next]);
        assert!(moved.is_empty() && removed.is_empty());
        assert_eq!(changed, vec![0]);
    }

    #[test]
    fn first_view_is_all_changed() {
        let (moved, changed, removed) = people_diff(&HashMap::new(), &[row(1, 0, 0), row(2, 0, 0)]);
        assert!(moved.is_empty() && removed.is_empty());
        assert_eq!(changed, vec![0, 1]);
    }
}
