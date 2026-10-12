//! The deterministic world recorder behind `worldsim_wm` (Pass 7 section 9).
//!
//! An episode runs one scene and seed single-threaded for `days`, optionally
//! under an intervention schedule, and takes a row every `every` ticks
//! (ticks 0, every, 2 x every, ... days x ticks_per_day). Row t is the state
//! after t completed ticks, taken before the interventions of tick t.
//! Values come from the pure collectors the FFI uses, the firewood helpers and
//! the probe's own counts; nothing here feeds the simulation. Files are
//! written into `<dir>.tmp` and renamed when complete, so a partial episode
//! never looks finished. Wall time goes to stderr and `timing.json` only.
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use serde_json::{json, Value};
use sim_bridge::ffi::world_node::{
    collect_agent_colds, collect_agent_snapshot, collect_building_snapshot, collect_resource_snapshot,
    collect_terrain_snapshot, collect_time_snapshot,
};
use sim_core::components::{Agent, BodyHealth, Life, Lineage, ResourceKind, SettlementId};
use sim_engine::{Intervention, NoiseFork, SimEngine, RESOURCE_SOURCE_INFINITE};
use sim_systems::runtime::settlement::firewood::{wood_in_hand, wood_target, year_at_tick};

use super::digest::state_digest;
use super::npy::{self, Dtype};
use super::schedule::{apply_due, parse_schedule, ScheduleEntry};
use super::schema::{Field, AGENT, GLOBAL, LATENT, RESOURCE, SETTLEMENT};
use super::sha256::sha256_hex;
use crate::probe::fingerprint;

/// Recorder format version written into `meta.json`.
pub const SCHEMA_VERSION: u64 = 4;

/// Scenes the recorder can build.
pub const SCENES: [&str; 9] =
    ["game", "game_v9", "game_v10", "game_v11", "game_v12", "game_v13", "game_v14", "game_v15", "game_v16_neutral"];

/// Build a fresh engine of `scene` and `seed`.
pub fn make_scene(scene: &str, seed: u64) -> Result<SimEngine, String> {
    Ok(match scene {
        "game" => crate::scenes::game_scene(seed),
        "game_v9" => crate::scenes::game_v9_scene(seed),
        "game_v10" => crate::scenes::game_v10_scene(seed),
        "game_v11" => crate::scenes::game_v11_scene(seed),
        "game_v12" => crate::scenes::game_v12_scene(seed),
        "game_v13" => crate::scenes::game_v13_scene(seed),
        "game_v14" => crate::scenes::game_v14_scene(seed),
        "game_v15" => crate::scenes::game_v15_scene(seed),
        "game_v16_neutral" => crate::scenes::game_v16_neutral_scene(seed),
        _ => return Err(format!("scene must be one of {}", SCENES.join(", "))),
    })
}

/// One episode to record.
#[derive(Debug, Clone)]
pub struct EpisodeSpec {
    /// Scene name.
    pub scene: String,
    /// World seed.
    pub seed: u64,
    /// Display days to run.
    pub days: u64,
    /// Ticks between rows.
    pub every: u64,
    /// Record the agent and resource tables.
    pub agents: bool,
    /// `none`, or the schedule file's id.
    pub id: String,
    /// Schedule text (verbatim JSON) when the episode has one.
    pub schedule: Option<String>,
    /// Noise fork of a branch-family member (Pass 9): replicate `k` from tick
    /// `T` on; `None` for an unforked episode.
    pub fork: Option<NoiseFork>,
}

impl EpisodeSpec {
    /// Directory name `<scene>_s<seed>_<id>`, plus `_r<k>` for a fork member.
    pub fn dir_name(&self) -> String {
        match self.fork {
            Some(fork) => format!("{}_s{}_{}_r{}", self.scene, self.seed, self.id, fork.replicate),
            None => format!("{}_s{}_{}", self.scene, self.seed, self.id),
        }
    }
}

/// Every value of one row before the tables are assembled.
struct Row {
    global: [f32; 37],
    settlements: BTreeMap<SettlementId, [f32; 9]>,
    agents: BTreeMap<u64, [f32; 18]>,
    keys: BTreeMap<u64, u64>,
    resources: BTreeMap<(u8, u32, u32), (f32, f32)>,
    buildings: BTreeMap<u64, (u8, u32, u32)>,
    latent: [f32; 7],
    digest: u64,
}

fn num(row: &Value, key: &str) -> f32 {
    row[key].as_f64().unwrap_or(f64::NAN) as f32
}

/// Take one row of `engine` (Pass 7 section 9.2).
fn take_row(engine: &SimEngine, scene: &str, seed: u64, agents: bool, settlement_order: &mut Vec<SettlementId>) -> Row {
    let r = &engine.resources;
    let now = engine.current_tick();
    let sample = crate::probe::metrics::sample(engine, scene, seed, 0);
    let homes = crate::probe::pass5::member_homes(engine);
    let work = crate::probe::pass5::work_counts(engine, &homes);
    let time = collect_time_snapshot(engine);
    let minute = if r.ticks_per_day == 0 { 0 } else { now.saturating_add(r.rules.clock_offset_ticks) % r.ticks_per_day };
    let seasons_on = r.rules.seasons.is_some();
    let alive = num(&sample, "alive");
    let children = num(&sample, "children");
    let elders = num(&sample, "elders");
    let (night_burn, winter_fall) = crate::probe::pass8::values(engine);
    let global = [
        now as f32,
        time.day_index as f32,
        minute as f32,
        if seasons_on { time.season as f32 } else { -1.0 },
        if seasons_on { time.day_of_season as f32 } else { 0.0 },
        year_at_tick(r, now) as f32,
        time.winter_severity as f32,
        alive,
        children,
        elders,
        alive - children - elders,
        num(&sample, "births"),
        num(&sample, "deaths_starvation"),
        num(&sample, "deaths_dehydration"),
        num(&sample, "deaths_combat"),
        num(&sample, "deaths_old_age"),
        num(&sample, "cold_members"),
        num(&sample, "deadwood_total"),
        num(&sample, "food_tiles_finite_total"),
        work.working as f32,
        num(&sample, "idle_adults"),
        num(&sample, "meals_from_store_total"),
        num(&sample, "ground_meals_total"),
        num(&sample, "forage_trips_total"),
        num(&sample, "wood_trips_total"),
        num(&sample, "wood_gathered_total"),
        num(&sample, "urgent_wood_trips_total"),
        num(&sample, "wood_retargets_total"),
        num(&sample, "cold_member_hours_total"),
        num(&sample, "interventions_applied_total"),
        num(&sample, "fire_blocked_settlements"),
        num(&sample, "cold_snap_nights_left"),
        f32::from(u8::from(r.cold_snap_tonight)),
        night_burn as f32,
        winter_fall as f32,
        num(&sample, "deaths_exposure"),
        num(&sample, "injured_members"),
    ];
    let mut ids: Vec<SettlementId> = r.settlements.keys().copied().collect();
    ids.sort_unstable();
    let target = wood_target(r).map_or(f32::NAN, |t| t as f32);
    let mut settlements = BTreeMap::new();
    for sid in &ids {
        if !settlement_order.contains(sid) {
            settlement_order.push(*sid);
        }
        let Some(s) = r.settlements.get(sid) else { continue };
        let stock = |kind| s.stockpile.get(&kind).copied().unwrap_or(0) as f32;
        settlements.insert(*sid, [
            s.member_agents.len() as f32,
            stock(ResourceKind::Food),
            stock(ResourceKind::Wood),
            target,
            f32::from(u8::from(r.fires.get(sid).is_some_and(|fire| fire.lit))),
            wood_in_hand(&engine.world, r, *sid) as f32,
            s.formation_tile.0 as f32,
            s.formation_tile.1 as f32,
            r.fire_blocked_until.get(sid).map_or(0, |until| until.saturating_sub(now)) as f32,
        ]);
    }
    let mut agent_rows = BTreeMap::new();
    let mut keys = BTreeMap::new();
    let mut resources = BTreeMap::new();
    if agents {
        keys = engine.world.query::<(&Agent, &Lineage)>().iter().map(|(_, (a, l))| (a.id, l.key)).collect();
        let member_of: BTreeMap<u64, SettlementId> = {
            let mut map = BTreeMap::new();
            for sid in &ids {
                if let Some(s) = r.settlements.get(sid) {
                    for id in &s.member_agents {
                        map.entry(*id).or_insert(*sid);
                    }
                }
            }
            map
        };
        let lives: BTreeMap<u64, Life> = engine.world.query::<(&Agent, &Life)>().iter().map(|(_, (a, l))| (a.id, *l)).collect();
        let hps: BTreeMap<u64, f64> =
            engine.world.query::<(&Agent, &BodyHealth)>().iter().map(|(_, (a, h))| (a.id, h.hp)).collect();
        let tpd = r.ticks_per_day.max(1) as f64;
        let snapshot = collect_agent_snapshot(&engine.world);
        let colds = collect_agent_colds(&engine.world);
        for (row, cold) in snapshot.iter().zip(colds) {
            let settlement = member_of
                .get(&row.agent_id)
                .and_then(|sid| settlement_order.iter().position(|s| s == sid))
                .map_or(-1.0, |index| index as f32);
            let life = lives.get(&row.agent_id);
            agent_rows.insert(row.agent_id, [
                row.x as f32,
                row.y as f32,
                f32::from(row.state_tag),
                f32::from(row.seek_kind),
                f32::from(row.target_kind),
                row.target_x as f32,
                row.target_y as f32,
                row.hunger,
                row.thirst,
                row.sleep,
                row.carried_food as f32,
                row.carried_wood as f32,
                f32::from(row.life_stage),
                f32::from(cold),
                settlement,
                life.map_or(f32::NAN, |l| (l.age_ticks(now) as f64 / tpd) as f32),
                life.map_or(f32::NAN, |l| (l.lifespan_ticks as f64 / tpd) as f32),
                hps.get(&row.agent_id).map_or(f32::NAN, |hp| *hp as f32),
            ]);
        }
        for row in collect_resource_snapshot(r) {
            let live = match row.kind {
                0 => r.food_tiles.get(&(row.x, row.y)),
                1 => r.water_tiles.get(&(row.x, row.y)),
                2 => r.sleep_tiles.get(&(row.x, row.y)),
                _ => None,
            };
            if live == Some(&RESOURCE_SOURCE_INFINITE) {
                continue;
            }
            resources.insert((row.kind, row.x, row.y), (f32::from(row.amount), f32::from(row.max)));
        }
    }
    let buildings = collect_building_snapshot(r).into_iter().map(|b| (b.id, (b.kind, b.x, b.y))).collect();
    let cursor = r.wood_winter_cursor;
    let latent = [
        r.food_regen_acc as f32,
        r.wood_regen_acc as f32,
        r.wood_winter_acc as f32,
        cursor.map_or(-1.0, |c| c.0 as f32),
        cursor.map_or(-1.0, |c| c.1 as f32),
        r.cold_snap_nights as f32,
        f32::from(u8::from(r.cold_snap_tonight)),
    ];
    Row { global, settlements, agents: agent_rows, keys, resources, buildings, latent, digest: state_digest(engine) }
}

/// Keys in first-seen order over all rows (rows are each sorted by key).
fn first_seen<K: Ord + Copy, V>(rows: impl Iterator<Item = impl IntoIterator<Item = (K, V)>>) -> Vec<K> {
    let mut seen = BTreeSet::new();
    let mut order = Vec::new();
    for row in rows {
        for (key, _) in row {
            if seen.insert(key) {
                order.push(key);
            }
        }
    }
    order
}

fn fields_json(fields: &[Field], dtype: &str) -> Value {
    Value::Array(
        fields
            .iter()
            .map(|f| json!({"name": f.name, "dtype": dtype, "unit": f.unit, "meaning": f.meaning, "hidden": f.hidden}))
            .collect(),
    )
}

fn kind_name(intervention: &Intervention) -> (&'static str, Option<SettlementId>, u32) {
    match *intervention {
        Intervention::GiftFood { settlement, amount } => ("gift_food", Some(settlement), amount),
        Intervention::SpoilFood { settlement, amount } => ("spoil_food", Some(settlement), amount),
        Intervention::GiftWood { settlement, amount } => ("gift_wood", Some(settlement), amount),
        Intervention::RotWood { settlement, amount } => ("rot_wood", Some(settlement), amount),
        Intervention::Rain { settlement, hours } => ("rain", Some(settlement), hours),
        Intervention::ColdSnap { nights } => ("cold_snap", None, nights),
        Intervention::Windfall { units } => ("windfall", None, units),
    }
}

/// The recorded files of one episode: (name, bytes, dtype, shape) for arrays
/// plus `meta.json` and `interventions.json`.
pub struct EpisodeFiles {
    /// Array files in write order.
    pub arrays: Vec<(String, Vec<u8>, Dtype, Vec<usize>)>,
    /// `meta.json` contents.
    pub meta: Value,
    /// `interventions.json` contents.
    pub interventions: Value,
}

/// Run one episode in memory and assemble its files.
pub fn record(spec: &EpisodeSpec) -> Result<EpisodeFiles, String> {
    let mut engine = make_scene(&spec.scene, spec.seed)?;
    let entries: Vec<ScheduleEntry> = match &spec.schedule {
        Some(text) => parse_schedule(text)?,
        None => Vec::new(),
    };
    let tpd = engine.resources.ticks_per_day;
    if tpd == 0 || spec.every == 0 {
        return Err("recorder requires positive ticks_per_day and every".into());
    }
    let total = spec.days.checked_mul(tpd).ok_or("days overflow")?;
    let terrain0 = collect_terrain_snapshot(&engine.resources);
    let mut order: Vec<SettlementId> = Vec::new();
    let mut rows = Vec::new();
    let mut next = 0usize;
    for tick in 0..=total {
        if tick.is_multiple_of(spec.every) {
            rows.push(take_row(&engine, &spec.scene, spec.seed, spec.agents, &mut order));
        }
        if tick == total {
            break;
        }
        if let Some(fork) = spec.fork.filter(|fork| fork.tick == tick) {
            engine.apply_noise_fork(fork.replicate);
        }
        apply_due(&mut engine, &entries, &mut next);
        engine.tick();
    }
    let terrain_end = collect_terrain_snapshot(&engine.resources);
    let t = rows.len();
    let mut arrays = Vec::new();
    let mut push = |name: &str, bytes: Vec<u8>, dtype: Dtype, shape: Vec<usize>| arrays.push((name.to_string(), bytes, dtype, shape));

    let global: Vec<f32> = rows.iter().flat_map(|r| r.global).collect();
    push("global.npy", npy::f32_file(&[t, GLOBAL.len()], &global), Dtype::F32, vec![t, GLOBAL.len()]);
    let s = order.len();
    let mut table = Vec::with_capacity(t * s * SETTLEMENT.len());
    for row in &rows {
        for sid in &order {
            table.extend(row.settlements.get(sid).copied().unwrap_or([f32::NAN; 9]));
        }
    }
    push("settlements.npy", npy::f32_file(&[t, s, SETTLEMENT.len()], &table), Dtype::F32, vec![t, s, SETTLEMENT.len()]);
    let ids: Vec<i64> = order.iter().map(|id| i64::from(*id)).collect();
    push("settlement_ids.npy", npy::i64_file(&[s], &ids), Dtype::I64, vec![s]);
    let (h, w) = (terrain0.height as usize, terrain0.width as usize);
    push("terrain.npy", npy::u8_file(&[h, w], &terrain0.terrain), Dtype::U8, vec![h, w]);
    push("elevation.npy", npy::u8_file(&[h, w], &terrain0.elevation), Dtype::U8, vec![h, w]);
    let building_order = first_seen(rows.iter().map(|r| r.buildings.iter().map(|(k, v)| (*k, *v))));
    let mut building_keys = Vec::new();
    for id in &building_order {
        let (kind, x, y) = rows.iter().find_map(|r| r.buildings.get(id).copied()).unwrap_or((0, 0, 0));
        building_keys.extend([*id as i64, i64::from(kind), i64::from(x), i64::from(y)]);
    }
    let b = building_order.len();
    push("building_keys.npy", npy::i64_file(&[b, 4], &building_keys), Dtype::I64, vec![b, 4]);
    let present: Vec<u8> = rows.iter().flat_map(|r| building_order.iter().map(|id| u8::from(r.buildings.contains_key(id)))).collect();
    push("building_present.npy", npy::u8_file(&[t, b], &present), Dtype::U8, vec![t, b]);
    if spec.agents {
        let agent_order = first_seen(rows.iter().map(|r| r.agents.iter().map(|(k, v)| (*k, *v))));
        let n = agent_order.len();
        let agent_ids: Vec<i64> = agent_order.iter().map(|id| *id as i64).collect();
        push("agent_ids.npy", npy::i64_file(&[n], &agent_ids), Dtype::I64, vec![n]);
        // Schema 3: the lineage key of each agent (0 without one), aligned with
        // agent_ids; an identity field for matching agents across arms.
        let agent_keys: Vec<u64> =
            agent_order.iter().map(|id| rows.iter().find_map(|r| r.keys.get(id).copied()).unwrap_or(0)).collect();
        push("agent_keys.npy", npy::u64_file(&[n], &agent_keys), Dtype::U64, vec![n]);
        let alive: Vec<u8> = rows.iter().flat_map(|r| agent_order.iter().map(|id| u8::from(r.agents.contains_key(id)))).collect();
        push("agent_alive.npy", npy::u8_file(&[t, n], &alive), Dtype::U8, vec![t, n]);
        let mut table = Vec::with_capacity(t * n * AGENT.len());
        for row in &rows {
            for id in &agent_order {
                table.extend(row.agents.get(id).copied().unwrap_or([f32::NAN; 18]));
            }
        }
        push("agents.npy", npy::f32_file(&[t, n, AGENT.len()], &table), Dtype::F32, vec![t, n, AGENT.len()]);
        let resource_order = first_seen(rows.iter().map(|r| r.resources.iter().map(|(k, v)| (*k, *v))));
        let rr = resource_order.len();
        let keys: Vec<i32> = resource_order.iter().flat_map(|(k, x, y)| [i32::from(*k), *x as i32, *y as i32]).collect();
        push("resource_keys.npy", npy::i32_file(&[rr, 3], &keys), Dtype::I32, vec![rr, 3]);
        let mut table = Vec::with_capacity(t * rr * 3);
        for row in &rows {
            for key in &resource_order {
                match row.resources.get(key) {
                    Some((amount, max)) => table.extend([*amount, *max, 1.0]),
                    None => table.extend([f32::NAN, f32::NAN, 0.0]),
                }
            }
        }
        push("resources.npy", npy::f32_file(&[t, rr, 3], &table), Dtype::F32, vec![t, rr, 3]);
    }
    let latent: Vec<f32> = rows.iter().flat_map(|r| r.latent).collect();
    push("latent.npy", npy::f32_file(&[t, LATENT.len()], &latent), Dtype::F32, vec![t, LATENT.len()]);
    let digests: Vec<u64> = rows.iter().map(|r| r.digest).collect();
    push("row_digest.npy", npy::u64_file(&[t], &digests), Dtype::U64, vec![t]);

    let interventions = Value::Array(
        engine
            .resources
            .intervention_log
            .iter()
            .map(|rec| {
                let (kind, settlement, amount) = kind_name(&rec.intervention);
                json!({"tick": rec.tick, "kind": kind, "settlement": settlement, "amount": amount, "applied": rec.applied})
            })
            .collect(),
    );
    let schedule: Value = match &spec.schedule {
        Some(text) => serde_json::from_str(text).map_err(|e| e.to_string())?,
        None => Value::Null,
    };
    let mut array_meta = serde_json::Map::new();
    for (name, bytes, dtype, shape) in &arrays {
        array_meta.insert(name.clone(), json!({"dtype": dtype.name(), "shape": shape, "sha256": sha256_hex(bytes)}));
    }
    let mut fields = serde_json::Map::new();
    fields.insert("global.npy".into(), fields_json(&GLOBAL, "float32"));
    fields.insert("settlements.npy".into(), fields_json(&SETTLEMENT, "float32"));
    fields.insert("latent.npy".into(), fields_json(&LATENT, "float32"));
    if spec.agents {
        fields.insert("agents.npy".into(), fields_json(&AGENT, "float32"));
        fields.insert("resources.npy".into(), fields_json(&RESOURCE, "float32"));
        fields.insert("agent_keys.npy".into(), json!([
            {"name": "lineage_key", "dtype": "uint64", "unit": "id", "meaning": "lineage key of the agent in agent_ids order (0 without one): an identity field for matching agents across the arms of a fork, not an input", "hidden": true, "identity": true}]));
        fields.insert("resource_keys.npy".into(), json!([
            {"name": "kind", "dtype": "int32", "unit": "index", "meaning": "0 food, 1 water, 2 sleep, 3 deadwood", "hidden": false},
            {"name": "x", "dtype": "int32", "unit": "tiles", "meaning": "tile x", "hidden": false},
            {"name": "y", "dtype": "int32", "unit": "tiles", "meaning": "tile y", "hidden": false}]));
    }
    fields.insert("building_keys.npy".into(), json!([
        {"name": "id", "dtype": "int64", "unit": "id", "meaning": "BuildingId", "hidden": false},
        {"name": "kind", "dtype": "int64", "unit": "index", "meaning": "BuildingKind::code (0 hearth)", "hidden": false},
        {"name": "x", "dtype": "int64", "unit": "tiles", "meaning": "tile x", "hidden": false},
        {"name": "y", "dtype": "int64", "unit": "tiles", "meaning": "tile y", "hidden": false}]));
    let observable: serde_json::Map<String, Value> = fields
        .iter()
        .map(|(name, list)| {
            let names: Vec<Value> = list
                .as_array()
                .map(|cols| cols.iter().filter(|c| c["hidden"] != Value::Bool(true)).map(|c| c["name"].clone()).collect())
                .unwrap_or_default();
            (name.clone(), Value::Array(names))
        })
        .collect();
    let meta = json!({
        "schema_version": SCHEMA_VERSION,
        "observable_fields": observable,
        "scene": spec.scene,
        "seed": spec.seed,
        "episode_id": spec.id,
        "days": spec.days,
        "every": spec.every,
        "rows": t,
        "ticks_per_day": tpd,
        "clock_offset_ticks": engine.resources.rules.clock_offset_ticks,
        "git_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
        "agents_recorded": spec.agents,
        "keyed_draws": engine.resources.rules.agent.keyed_draws,
        "noise_fork": spec.fork.map_or(Value::Null, |fork| json!({"tick": fork.tick, "replicate": fork.replicate})),
        // Schema 4: the world's hidden parameters (game_v16 only), marked
        // hidden inside the block; never an input of a model of what a player sees.
        "world_variety": engine.resources.world_variety.map_or(Value::Null, |v| json!({
            "hidden": true, "values": crate::probe::pass10::variety_json(&v)})),
        "schedule": schedule,
        "arrays": array_meta,
        "fields": fields,
        "terrain_static": terrain0 == terrain_end,
        "final_fingerprint": fingerprint(&engine),
        "row_rule": "row t is the state after t completed ticks, taken before the interventions of tick t",
        "missing_value": "NaN in float tables where an entity does not exist",
        "not_markov": "the tables plus latent.npy are not a complete Markov state: agents' decision timers, memories, social bookkeeping and random streams are not exported",
    });
    Ok(EpisodeFiles { arrays, meta, interventions })
}

/// Whether `dir` holds a finished episode whose meta hashes verify.
pub fn verify_episode(dir: &Path) -> bool {
    let Ok(text) = fs::read_to_string(dir.join("meta.json")) else { return false };
    let Ok(meta) = serde_json::from_str::<Value>(&text) else { return false };
    let Some(arrays) = meta["arrays"].as_object() else { return false };
    !arrays.is_empty()
        && arrays.iter().all(|(name, entry)| {
            fs::read(dir.join(name)).is_ok_and(|bytes| entry["sha256"].as_str() == Some(sha256_hex(&bytes).as_str()))
        })
}

/// Whether the finished episode in `dir` is the one `spec` asks for: its hashes
/// verify and its meta records the same scene, seed, days, every, agent flag,
/// schedule and (when `WORLDSIM_GIT_SHA` is set) git sha (review round 1, L1).
pub fn episode_matches(dir: &Path, spec: &EpisodeSpec) -> bool {
    let Ok(text) = fs::read_to_string(dir.join("meta.json")) else { return false };
    let Ok(meta) = serde_json::from_str::<Value>(&text) else { return false };
    let schedule = match &spec.schedule {
        Some(text) => serde_json::from_str::<Value>(text).unwrap_or(Value::Null),
        None => Value::Null,
    };
    let sha_ok = std::env::var("WORLDSIM_GIT_SHA").map_or(true, |sha| meta["git_sha"].as_str() == Some(sha.as_str()));
    verify_episode(dir)
        && meta["scene"].as_str() == Some(spec.scene.as_str())
        && meta["seed"].as_u64() == Some(spec.seed)
        && meta["days"].as_u64() == Some(spec.days)
        && meta["every"].as_u64() == Some(spec.every)
        && meta["agents_recorded"].as_bool() == Some(spec.agents)
        && meta["schedule"] == schedule
        && meta["noise_fork"] == spec.fork.map_or(Value::Null, |fork| json!({"tick": fork.tick, "replicate": fork.replicate}))
        && sha_ok
}

/// Record `spec` into `<out>/<dir_name>` (via a `.tmp` directory). With
/// `resume`, a finished episode that [`episode_matches`] `spec` is kept. Returns the wall
/// seconds spent (0 when skipped).
pub fn write_episode(spec: &EpisodeSpec, out: &Path, resume: bool) -> Result<f64, String> {
    let dir = out.join(spec.dir_name());
    if resume && episode_matches(&dir, spec) {
        eprintln!("SKIP {} (verified)", spec.dir_name());
        return Ok(0.0);
    }
    let started = Instant::now();
    let files = record(spec)?;
    let tmp = out.join(format!("{}.tmp", spec.dir_name()));
    let _ = fs::remove_dir_all(&tmp);
    fs::create_dir_all(&tmp).map_err(|e| e.to_string())?;
    for (name, bytes, ..) in &files.arrays {
        fs::write(tmp.join(name), bytes).map_err(|e| e.to_string())?;
    }
    let pretty = |v: &Value| serde_json::to_string_pretty(v).map(|s| s + "\n").map_err(|e| e.to_string());
    fs::write(tmp.join("meta.json"), pretty(&files.meta)?).map_err(|e| e.to_string())?;
    fs::write(tmp.join("interventions.json"), pretty(&files.interventions)?).map_err(|e| e.to_string())?;
    let seconds = started.elapsed().as_secs_f64();
    fs::write(tmp.join("timing.json"), format!("{{\"wall_seconds\": {seconds:.3}}}\n")).map_err(|e| e.to_string())?;
    let _ = fs::remove_dir_all(&dir);
    fs::rename(&tmp, &dir).map_err(|e| e.to_string())?;
    eprintln!("EPISODE {} rows={} wall_seconds={seconds:.2}", spec.dir_name(), files.meta["rows"]);
    Ok(seconds)
}

/// Validated `worldsim_wm` arguments.
#[derive(Debug, Clone)]
pub struct WmArgs {
    /// Scene name.
    pub scene: String,
    /// Half-open seed range.
    pub seeds: std::ops::Range<u64>,
    /// Display days.
    pub days: u64,
    /// Ticks between rows.
    pub every: u64,
    /// Record agents and resources.
    pub agents: bool,
    /// Directory of `s<seed>_<id>.json` schedules.
    pub schedule_dir: Option<PathBuf>,
    /// Also record the no-intervention episode of every seed.
    pub include_factual: bool,
    /// Episode workers (1..=8).
    pub workers: usize,
    /// Output directory.
    pub out: PathBuf,
    /// Keep finished episodes whose hashes verify.
    pub resume: bool,
    /// Branch families (Pass 9): fork tick T; every seed and schedule gets
    /// `replicates` episodes sharing the prefix up to T.
    pub fork_tick: Option<u64>,
    /// Replicates per branch-family member (with `fork_tick`).
    pub replicates: u32,
}

/// Usage text of `worldsim_wm`.
pub const USAGE: &str = "usage: worldsim_wm --scene game_v13|game_v14|game_v15|game_v16_neutral|... --seeds a..b --days N --every TICKS [--agents] [--schedule-dir DIR [--include-factual]] [--fork-tick T --replicates R] [--resume] --workers K --out DIR";

impl WmArgs {
    /// Parse argv excluding the executable.
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, String> {
        let mut parsed = Self {
            scene: String::new(),
            seeds: 0..0,
            days: 0,
            every: 60,
            agents: false,
            schedule_dir: None,
            include_factual: false,
            workers: 1,
            out: PathBuf::new(),
            resume: false,
            fork_tick: None,
            replicates: 0,
        };
        let mut args = args.into_iter();
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--agents" => parsed.agents = true,
                "--include-factual" => parsed.include_factual = true,
                "--resume" => parsed.resume = true,
                "--help" => return Err(USAGE.into()),
                _ => {
                    let value = args.next().ok_or_else(|| format!("missing value for {flag}"))?;
                    match flag.as_str() {
                        "--scene" => parsed.scene = value,
                        "--seeds" => {
                            let (a, b) = value.split_once("..").ok_or("seeds must be a..b (half-open)")?;
                            parsed.seeds = a.parse().map_err(|_| "invalid seed start")?..b.parse().map_err(|_| "invalid seed end")?;
                        }
                        "--days" => parsed.days = value.parse().map_err(|_| "invalid days")?,
                        "--every" => parsed.every = value.parse().map_err(|_| "invalid every")?,
                        "--schedule-dir" => parsed.schedule_dir = Some(value.into()),
                        "--workers" => parsed.workers = value.parse().map_err(|_| "invalid workers")?,
                        "--out" => parsed.out = value.into(),
                        "--fork-tick" => parsed.fork_tick = Some(value.parse().map_err(|_| "invalid fork tick")?),
                        "--replicates" => parsed.replicates = value.parse().map_err(|_| "invalid replicates")?,
                        _ => return Err(format!("unknown argument {flag}\n{USAGE}")),
                    }
                }
            }
        }
        if !SCENES.contains(&parsed.scene.as_str()) {
            return Err(format!("scene must be one of {}", SCENES.join(", ")));
        }
        if parsed.seeds.is_empty() || !(1..=400).contains(&parsed.days) || parsed.every == 0 {
            return Err("seeds must be nonempty, days 1..=400 and every positive".into());
        }
        if !(1..=8).contains(&parsed.workers) || parsed.out.as_os_str().is_empty() {
            return Err("workers must be 1..=8 and --out is required".into());
        }
        if parsed.include_factual && parsed.schedule_dir.is_none() {
            return Err("--include-factual needs --schedule-dir".into());
        }
        match parsed.fork_tick {
            Some(tick) => {
                if !(1..=64).contains(&parsed.replicates) {
                    return Err("--fork-tick needs --replicates 1..=64".into());
                }
                if parsed.schedule_dir.is_none() || !parsed.include_factual {
                    return Err("--fork-tick needs --schedule-dir and --include-factual".into());
                }
                // Tick 0 is rejected: founders' lives are drawn at worldgen, before
                // the recorder could set the fork (review round 1, L3).
                if tick == 0 || tick >= parsed.days * 1440 {
                    return Err("--fork-tick must lie inside the run (1..days x 1440)".into());
                }
            }
            None if parsed.replicates != 0 => return Err("--replicates needs --fork-tick".into()),
            None => {}
        }
        Ok(parsed)
    }

    /// The episodes, seed by seed: schedule files sorted by name, then the
    /// factual episode under `--include-factual`.
    pub fn episodes(&self) -> Result<Vec<EpisodeSpec>, String> {
        let base = |seed: u64, id: String, schedule: Option<String>| EpisodeSpec {
            scene: self.scene.clone(),
            seed,
            days: self.days,
            every: self.every,
            agents: self.agents,
            id,
            schedule,
            fork: None,
        };
        let mut files: Vec<(String, PathBuf)> = Vec::new();
        if let Some(dir) = &self.schedule_dir {
            for entry in fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))? {
                let path = entry.map_err(|e| e.to_string())?.path();
                if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                    if name.starts_with('s') && name.ends_with(".json") {
                        files.push((name.to_string(), path.clone()));
                    }
                }
            }
            files.sort();
        }
        let mut episodes = Vec::new();
        for seed in self.seeds.clone() {
            if self.schedule_dir.is_none() {
                episodes.push(base(seed, "none".into(), None));
                continue;
            }
            let prefix = format!("s{seed}_");
            let mut found = 0;
            for (name, path) in &files {
                let Some(id) = name.strip_prefix(&prefix).and_then(|rest| rest.strip_suffix(".json")) else { continue };
                if id.is_empty() || id == "none" || !id.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-') {
                    return Err(format!("{name}: invalid schedule id"));
                }
                let text = fs::read_to_string(path).map_err(|e| format!("{name}: {e}"))?;
                parse_schedule(&text).map_err(|e| format!("{name}: {e}"))?;
                episodes.push(base(seed, id.to_string(), Some(text)));
                found += 1;
            }
            if self.include_factual {
                episodes.push(base(seed, "none".into(), None));
            } else if found == 0 {
                return Err(format!("seed {seed} has no schedule file and --include-factual is not set"));
            }
        }
        // Pass 9: every episode becomes a branch family of `replicates` members.
        if let Some(tick) = self.fork_tick {
            episodes = episodes
                .into_iter()
                .flat_map(|spec| {
                    (0..self.replicates).map(move |replicate| EpisodeSpec {
                        fork: Some(NoiseFork { tick, replicate }),
                        ..spec.clone()
                    })
                })
                .collect();
        }
        Ok(episodes)
    }
}

/// Record every episode of `args` with a scoped worker pool; output never
/// depends on the worker count. Returns (dir name, wall seconds) per episode.
pub fn run(args: &WmArgs) -> Result<Vec<(String, f64)>, String> {
    let episodes = args.episodes()?;
    fs::create_dir_all(&args.out).map_err(|e| e.to_string())?;
    let results = std::thread::scope(|scope| {
        let mut handles = Vec::new();
        for worker in 0..args.workers {
            let episodes = &episodes;
            handles.push(scope.spawn(move || {
                let mut done = Vec::new();
                for spec in episodes.iter().skip(worker).step_by(args.workers) {
                    done.push((spec.dir_name(), write_episode(spec, &args.out, args.resume)?));
                }
                Ok::<_, String>(done)
            }));
        }
        let mut all = Vec::new();
        for handle in handles {
            all.extend(handle.join().map_err(|_| "recorder worker panicked".to_string())??);
        }
        Ok::<_, String>(all)
    })?;
    let mut results = results;
    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(days: u64, agents: bool, schedule: Option<&str>) -> EpisodeSpec {
        EpisodeSpec {
            scene: "game_v13".into(),
            seed: 3,
            days,
            every: 60,
            agents,
            id: if schedule.is_some() { "t".into() } else { "none".into() },
            schedule: schedule.map(str::to_string),
            fork: None,
        }
    }

    #[test]
    fn first_row_is_the_fresh_engine_and_shapes_match_the_fields() {
        let files = record(&spec(1, true, None)).expect("record");
        let fresh = make_scene("game_v13", 3).expect("scene");
        let digest_file = &files.arrays.iter().find(|a| a.0 == "row_digest.npy").expect("digests").1;
        let start = npy::header(Dtype::U64, &[25]).len();
        assert_eq!(&digest_file[start..start + 8], &state_digest(&fresh).to_le_bytes());
        let shape = |name: &str| files.arrays.iter().find(|a| a.0 == name).expect(name).3.clone();
        assert_eq!(shape("global.npy"), vec![25, GLOBAL.len()]);
        assert_eq!(shape("latent.npy"), vec![25, LATENT.len()]);
        assert_eq!(shape("agents.npy")[2], AGENT.len());
        assert_eq!(files.meta["rows"], 25);
        assert_eq!(files.meta["fields"]["global.npy"].as_array().expect("fields").len(), GLOBAL.len());
        assert!(files.meta["fields"]["agents.npy"][16]["hidden"].as_bool().expect("bool"));
    }

    #[test]
    fn schema_two_hides_the_weather_and_lists_observable_fields() {
        let files = record(&spec(1, true, None)).expect("record");
        assert_eq!(files.meta["schema_version"], 4);
        let hidden = |array: &str, name: &str| {
            files.meta["fields"][array]
                .as_array()
                .expect("fields")
                .iter()
                .find(|c| c["name"] == name)
                .map(|c| c["hidden"].as_bool().expect("bool"))
                .expect(name)
        };
        for name in ["winter_severity", "night_burn_per_hour", "winter_fall_today"] {
            assert!(hidden("global.npy", name), "{name}");
        }
        assert!(hidden("settlements.npy", "wood_target"));
        assert!(hidden("agents.npy", "lifespan_days"));
        assert!(LATENT.iter().all(|f| f.hidden));
        assert!(!hidden("global.npy", "deadwood_total"));
        let observable = files.meta["observable_fields"].as_object().expect("observable");
        for (array, fields) in files.meta["fields"].as_object().expect("fields") {
            let want: Vec<&Value> =
                fields.as_array().expect("list").iter().filter(|c| c["hidden"] == false).map(|c| &c["name"]).collect();
            let got: Vec<&Value> = observable[array].as_array().expect("names").iter().collect();
            assert_eq!(got, want, "{array}");
        }
        assert_eq!(observable["latent.npy"], json!([]));
        assert_eq!(GLOBAL[33].name, "night_burn_per_hour");
        assert_eq!(GLOBAL[34].name, "winter_fall_today");
        // Schema 4 appends its columns after every earlier one.
        assert_eq!(GLOBAL[35].name, "deaths_exposure");
        assert_eq!(GLOBAL[36].name, "injured_members");
        assert_eq!(AGENT[17].name, "hp");
        assert!(!hidden("agents.npy", "hp"));
        assert_eq!(files.meta["world_variety"], Value::Null);
    }

    #[test]
    fn schedules_are_recorded_and_change_the_rows_after_their_tick() {
        let text = r#"[{"tick": 600, "kind": "gift_wood", "settlement": "home", "amount": 30}]"#;
        let with = record(&spec(1, false, Some(text))).expect("record");
        let without = record(&spec(1, false, None)).expect("record");
        let digests = |f: &EpisodeFiles| f.arrays.iter().find(|a| a.0 == "row_digest.npy").expect("d").1.clone();
        let (a, b) = (digests(&with), digests(&without));
        let h = npy::header(Dtype::U64, &[25]).len();
        // Rows 0..=10 (ticks 0..=600) equal; row 11 (tick 660) differs.
        assert_eq!(a[h..h + 11 * 8], b[h..h + 11 * 8]);
        assert_ne!(a[h + 11 * 8..h + 12 * 8], b[h + 11 * 8..h + 12 * 8]);
        assert_eq!(with.interventions[0]["applied"], 30);
        assert_eq!(with.meta["schedule"][0]["kind"], "gift_wood");
        assert_eq!(without.interventions, json!([]));
    }

    #[test]
    fn resume_keeps_only_the_requested_episode() {
        let out = std::env::temp_dir().join(format!("wm_resume_test_{}", std::process::id()));
        let _ = fs::remove_dir_all(&out);
        let base = spec(1, false, None);
        write_episode(&base, &out, false).expect("write");
        let dir = out.join(base.dir_name());
        assert!(episode_matches(&dir, &base));
        assert!(!episode_matches(&dir, &EpisodeSpec { agents: true, ..base.clone() }));
        assert!(!episode_matches(&dir, &EpisodeSpec { days: 2, ..base.clone() }));
        assert!(!episode_matches(&dir, &EpisodeSpec { every: 120, ..base.clone() }));
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn arguments_validate() {
        let parse = |s: &str| WmArgs::parse(s.split_whitespace().map(String::from));
        assert!(parse("--scene game_v13 --seeds 0..2 --days 10 --every 60 --workers 2 --out x").is_ok());
        assert!(parse("--scene legacy --seeds 0..2 --days 10 --workers 2 --out x").is_err());
        assert!(parse("--scene game_v13 --seeds 0..2 --days 10 --workers 9 --out x").is_err());
        assert!(parse("--scene game_v13 --seeds 0..2 --days 10 --include-factual --out x").is_err());
        assert!(parse("--scene game_v13 --seeds 0..2 --days 401 --out x").is_err());
        let fork = |t: &str| parse(&format!("--scene game_v15 --seeds 0..2 --days 2 --schedule-dir d --include-factual --fork-tick {t} --replicates 2 --out x"));
        assert!(fork("1440").is_ok());
        assert!(fork("0").is_err());
        assert!(fork("2880").is_err());
        assert!(parse("--scene game_v15 --seeds 0..2 --days 2 --fork-tick 10 --replicates 2 --out x").is_err());
    }
}
