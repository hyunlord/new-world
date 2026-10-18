//! Revitalize Pass 15 — `worldsim_perf`: per-system tick timings of the game
//! world and of the stress world, and the Rust-side cost of the UI's snapshot
//! getters. Measurement only: nothing here feeds the simulation, and the
//! stress world is never a game, recorder or probe scene.
//!
//! Conventions (PASS15_GOAL.md 6.3):
//! - a run's ticks are numbered from 0; tick `t`'s display day is
//!   `(t + clock_offset_ticks) / ticks_per_day` and its clock the remainder;
//! - the first [`WARMUP_TICKS`] ticks (tick 0 runs every system) are left out
//!   of the tick-time summary, the per-system figures and the slowest ticks,
//!   and reported separately;
//! - percentiles are nearest-rank: the `p`-th of `n` sorted values is the
//!   value at index `ceil(p / 100 × n) − 1`.
use serde_json::{json, Value};
use sim_core::components::Agent;
use sim_core::rules::WorldRules;
use sim_engine::{Intervention, SimEngine, TickProfile};
use sim_systems::runtime::intervention::{apply_intervention, home_settlement};
use sim_systems::worldgen::{build_world_with, WorldGenConfig};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

/// Ticks at the start of every run kept out of the steady-state figures.
pub const WARMUP_TICKS: u64 = 60;
/// How many of the slowest steady ticks a run keeps.
pub const SLOWEST_KEPT: usize = 20;
/// Systems named per slow tick.
pub const SLOW_TICK_SYSTEMS: usize = 5;
/// Clock minute of the daily sample and of the fed world's daily gift (06:00).
pub const SAMPLE_MINUTE: u64 = 360;
/// Food per living person in each fed gift.
pub const GIFT_PER_ALIVE: u64 = 3;
/// Repetitions of each snapshot collector at the end of a run.
pub const SNAPSHOT_REPEATS: usize = 100;
/// A stress size whose run exceeds this many seconds is the last size run.
pub const SIZE_WALL_LIMIT_SECONDS: f64 = 1200.0;

/// Whether the stress world gets the daily food gift.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feed {
    /// `GiftFood` of 3 × alive to the home settlement after the first tick
    /// that founds one and then after every tick that ends at 06:00.
    Fed,
    /// No gifts.
    Raw,
    /// The natural game world (no gifts; named `none` in file names).
    None,
}

impl Feed {
    /// The name used in file names and JSON.
    pub const fn as_str(self) -> &'static str {
        match self {
            Feed::Fed => "fed",
            Feed::Raw => "raw",
            Feed::None => "none",
        }
    }
}

/// One benchmark run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunSpec {
    /// `natural`, `stress`, `map` or `quick`.
    pub mode: String,
    /// The probe scene name, or the stress world's label: `stress_v19`
    /// (`game_v19` rules), `stress_v20` or `stress_v20_inert` (Pass 16
    /// `--rules`; the same world under `game_v20` / `game_v20_inert` rules).
    pub scene: String,
    /// World seed.
    pub seed: u64,
    /// Founders asked for (stress worlds); `None` for the natural scene.
    pub agents: Option<u32>,
    /// Map side in tiles.
    pub side: u32,
    /// Gift policy.
    pub feed: Feed,
    /// Simulated ticks.
    pub ticks: u64,
}

impl RunSpec {
    /// `<mode>_<scene>_s<seed>_n<agents>_side<side>_<feed>` (`founders` fills
    /// in the natural scene's agents).
    pub fn stem(&self, founders: u64) -> String {
        format!(
            "{}_{}_s{}_n{}_side{}_{}",
            self.mode,
            self.scene,
            self.seed,
            self.agents.map_or(founders, u64::from),
            self.side,
            self.feed.as_str()
        )
    }
}

/// Build the world of `spec`; a worldgen panic (a seed that cannot host the
/// camp at that size) comes back as `Err` with its message.
pub fn build_engine(spec: &RunSpec) -> Result<SimEngine, String> {
    let scene = spec.scene.clone();
    let (seed, agents, side) = (spec.seed, spec.agents, spec.side);
    std::panic::catch_unwind(move || match agents {
        Some(n) => build_world_with(&WorldGenConfig::stress_v19(seed, n, side), stress_rules(&scene, seed)),
        None => crate::probe::scene_engine(&scene, seed),
    })
    .map_err(|payload| panic_text(payload.as_ref()))
}

/// Pass 16: the rules of a stress world by its label (`stress_v20`,
/// `stress_v20_inert`; anything else is today's `game_v19`).
pub fn stress_rules(label: &str, seed: u64) -> WorldRules {
    match label {
        "stress_v20" => WorldRules::game_v20(seed),
        "stress_v21" => WorldRules::game_v21(seed),
        "stress_v22" => WorldRules::game_v22(seed),
        "stress_v23" => WorldRules::game_v23(seed),
        "stress_v24" => WorldRules::game_v24(seed),
        "stress_v25" => WorldRules::game_v25(seed),
        "stress_v26" => WorldRules::game_v26(seed),
        "stress_v20_inert" => WorldRules::game_v20_inert(seed),
        _ => WorldRules::game_v19(seed),
    }
}

/// Pass 16: the stress label of the `--rules` choice stored in
/// `PerfArgs::scene` for `stress` / `quick` / `map`.
pub fn stress_label(rules: &str) -> &'static str {
    match rules {
        "game_v20" => "stress_v20",
        "game_v21" => "stress_v21",
        "game_v22" => "stress_v22",
        "game_v23" => "stress_v23",
        "game_v24" => "stress_v24",
        "game_v25" => "stress_v25",
        "game_v26" => "stress_v26",
        "game_v20_inert" => "stress_v20_inert",
        _ => "stress_v19",
    }
}

fn panic_text(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| payload.downcast_ref::<&str>().map(|s| (*s).to_string()))
        .unwrap_or_else(|| "panic".into())
}

/// Living agents (entities with `Agent`; the dead are despawned).
pub fn alive(engine: &SimEngine) -> u64 {
    engine.world.query::<&Agent>().iter().count() as u64
}

/// Nearest-rank percentile of `sorted` (ascending); 0 for an empty slice.
pub fn percentile(sorted: &[u32], p: f64) -> u32 {
    if sorted.is_empty() {
        return 0;
    }
    sorted[nearest_rank(p, sorted.len()) - 1]
}

/// The 1-based nearest rank `ceil(p / 100 × n)` of percentile `p` (given to
/// 0.001) among `n` values, in integer arithmetic so 99.9 of 1,000 is 999.
pub fn nearest_rank(p: f64, n: usize) -> usize {
    let milli = (p * 1000.0).round().clamp(0.0, 100_000.0) as u64;
    let rank = (milli * n as u64).div_ceil(100_000) as usize;
    rank.clamp(1, n.max(1))
}

/// Nearest-rank percentile of `sorted` f64 values; 0 for an empty slice.
pub fn percentile_f64(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[nearest_rank(p, sorted.len()) - 1]
}

/// Mean, nearest-rank percentiles, maximum and frame-overrun counts of tick
/// times in microseconds.
pub fn tick_summary(tick_us: &[u32]) -> Value {
    let mut sorted = tick_us.to_vec();
    sorted.sort_unstable();
    let n = sorted.len();
    let sum: u64 = sorted.iter().map(|&v| u64::from(v)).sum();
    let ms = |us: u32| f64::from(us) / 1000.0;
    json!({
        "ticks": n,
        "mean_ms": if n == 0 { 0.0 } else { sum as f64 / n as f64 / 1000.0 },
        "p50_ms": ms(percentile(&sorted, 50.0)),
        "p90_ms": ms(percentile(&sorted, 90.0)),
        "p99_ms": ms(percentile(&sorted, 99.0)),
        "p999_ms": ms(percentile(&sorted, 99.9)),
        "max_ms": ms(sorted.last().copied().unwrap_or(0)),
        "n_over_8ms": sorted.iter().filter(|&&v| v > 8_000).count(),
        "n_over_16_67ms": sorted.iter().filter(|&&v| f64::from(v) > 16_670.0).count(),
    })
}

/// One of the slowest steady ticks.
#[derive(Debug, Clone, PartialEq)]
pub struct SlowTick {
    /// Tick number (0-based).
    pub tick: u64,
    /// Whole-tick wall time, µs.
    pub us: u32,
    /// The systems with the largest time in that tick: (name, µs).
    pub top: Vec<(String, f64)>,
    /// Systems with interval > 1 that were due in that tick.
    pub due_periodic: Vec<String>,
}

/// Keep the `SLOWEST_KEPT` slowest ticks seen (ties: the earlier tick stays).
#[derive(Debug, Default)]
pub struct SlowestTicks {
    /// Slowest first.
    pub ticks: Vec<SlowTick>,
}

impl SlowestTicks {
    /// Whether a tick of `us` would enter the list.
    pub fn admits(&self, us: u32) -> bool {
        self.ticks.len() < SLOWEST_KEPT || self.ticks.last().is_some_and(|last| us > last.us)
    }

    /// Insert `entry` (call only when [`Self::admits`]).
    pub fn insert(&mut self, entry: SlowTick) {
        let at = self.ticks.partition_point(|t| t.us >= entry.us);
        self.ticks.insert(at, entry);
        self.ticks.truncate(SLOWEST_KEPT);
    }
}

fn slow_tick_entry(profile: &TickProfile, tick: u64, us: u32) -> SlowTick {
    let mut order: Vec<usize> = (0..profile.systems.len()).collect();
    order.sort_by(|&a, &b| profile.last_tick_ns[b].cmp(&profile.last_tick_ns[a]).then(a.cmp(&b)));
    SlowTick {
        tick,
        us,
        top: order
            .iter()
            .take(SLOW_TICK_SYSTEMS)
            .filter(|&&i| profile.last_tick_ns[i] > 0)
            .map(|&i| (profile.systems[i].name.clone(), profile.last_tick_ns[i] as f64 / 1000.0))
            .collect(),
        due_periodic: profile
            .systems
            .iter()
            .filter(|s| s.interval > 1 && tick.is_multiple_of(s.interval))
            .map(|s| s.name.clone())
            .collect(),
    }
}

/// Display day and `HH:MM` clock of tick `tick`.
pub fn display_clock(tick: u64, offset: u64, ticks_per_day: u64) -> (u64, String) {
    if ticks_per_day == 0 {
        return (0, "00:00".into());
    }
    let shifted = tick + offset;
    let minute = (shifted % ticks_per_day) * 1440 / ticks_per_day;
    (shifted / ticks_per_day, format!("{:02}:{:02}", minute / 60, minute % 60))
}

#[derive(Debug, Default, Clone)]
struct DayBucket {
    ticks: u64,
    tick_us: u64,
    system_ns: Vec<u64>,
    alive_0600: Option<u64>,
    settlements_0600: Option<u64>,
}

/// The result of one run: its JSON and its whole `tick_us` series.
#[derive(Debug, Clone)]
pub struct RunOutput {
    /// The run's JSON document.
    pub json: Value,
    /// Wall time of every tick, µs (warm-up included).
    pub tick_us: Vec<u32>,
}

/// Pass 19: each settlement's member count, largest first.
fn settlement_sizes(engine: &SimEngine) -> Vec<usize> {
    let mut sizes: Vec<usize> = engine.resources.settlements.values().map(|s| s.member_agents.len()).collect();
    sizes.sort_unstable_by(|a, b| b.cmp(a));
    sizes
}

/// Run `spec` on `engine` (built by [`build_engine`]) for `spec.ticks` ticks.
pub fn run(spec: &RunSpec, mut engine: SimEngine) -> RunOutput {
    let started = Instant::now();
    let founders = alive(&engine);
    let offset = engine.resources.rules.clock_offset_ticks;
    let tpd = engine.resources.ticks_per_day.max(1);
    let mut profile = engine.new_tick_profile();
    let systems = profile.systems.len();
    let mut warm_systems = profile.systems.clone();
    let mut warm_prune_ns = 0u64;
    let mut steady_max_ns = vec![0u64; systems];
    let mut days: BTreeMap<u64, DayBucket> = BTreeMap::new();
    let mut slowest = SlowestTicks::default();
    let mut gifts: Vec<Value> = Vec::new();
    #[cfg(feature = "subprofile")]
    let mut subprofile = crate::subprofile_report::SubprofileLog::new();
    for _ in 0..spec.ticks {
        let tick = engine.current_tick();
        engine.tick_profiled(&mut profile);
        let us = *profile.tick_us.last().unwrap_or(&0);
        #[cfg(feature = "subprofile")]
        subprofile.record(tick, us);
        if tick + 1 == WARMUP_TICKS {
            warm_systems = profile.systems.clone();
            warm_prune_ns = profile.prune_ns;
        }
        let bucket = days.entry((tick + offset) / tpd).or_insert_with(|| DayBucket { system_ns: vec![0; systems], ..DayBucket::default() });
        bucket.ticks += 1;
        bucket.tick_us += u64::from(us);
        for (sum, ns) in bucket.system_ns.iter_mut().zip(&profile.last_tick_ns) {
            *sum += ns;
        }
        if tick >= WARMUP_TICKS {
            for (max, ns) in steady_max_ns.iter_mut().zip(&profile.last_tick_ns) {
                *max = (*max).max(*ns);
            }
            if slowest.admits(us) {
                slowest.insert(slow_tick_entry(&profile, tick, us));
            }
        }
        let now = engine.current_tick();
        let at_sample = (now + offset) % tpd == SAMPLE_MINUTE * tpd / 1440;
        if at_sample {
            let day = days.entry((now + offset) / tpd).or_insert_with(|| DayBucket { system_ns: vec![0; systems], ..DayBucket::default() });
            day.alive_0600 = Some(alive(&engine));
            day.settlements_0600 = Some(engine.resources.settlements.len() as u64);
        }
        if spec.feed == Feed::Fed && (gifts.is_empty() || at_sample) {
            if let Some(home) = home_settlement(&engine.resources) {
                let requested = GIFT_PER_ALIVE * alive(&engine);
                let record = apply_intervention(
                    &mut engine,
                    Intervention::GiftFood { settlement: home, amount: u32::try_from(requested).unwrap_or(u32::MAX) },
                );
                gifts.push(json!({"tick": record.tick, "settlement": home, "requested": requested, "applied": record.applied}));
            }
        }
    }
    let wall_seconds = started.elapsed().as_secs_f64();
    let warm = (WARMUP_TICKS as usize).min(profile.tick_us.len());
    let steady_us: u64 = profile.tick_us[warm..].iter().map(|&v| u64::from(v)).sum();
    let steady_ns = steady_us * 1000;
    let mut accounted_ns = 0u64;
    let system_rows: Vec<Value> = profile
        .systems
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let (calls, total_ns) = if profile.ticks >= WARMUP_TICKS {
                (s.calls - warm_systems[i].calls, s.total_ns - warm_systems[i].total_ns)
            } else {
                (0, 0)
            };
            accounted_ns += total_ns;
            json!({
                "name": s.name, "priority": s.priority, "interval": s.interval,
                "calls": calls, "total_ms": total_ns as f64 / 1e6,
                "mean_us_per_call": if calls == 0 { 0.0 } else { total_ns as f64 / calls as f64 / 1000.0 },
                "max_us": steady_max_ns[i] as f64 / 1000.0,
                "share_of_tick_time": if steady_ns == 0 { 0.0 } else { total_ns as f64 / steady_ns as f64 },
                "calls_incl_warmup": s.calls, "total_ms_incl_warmup": s.total_ns as f64 / 1e6,
                "max_us_incl_warmup": s.max_ns as f64 / 1000.0,
            })
        })
        .collect();
    let prune_steady_ns = if profile.ticks >= WARMUP_TICKS { profile.prune_ns - warm_prune_ns } else { 0 };
    let per_day: Vec<Value> = days
        .iter()
        .map(|(day, b)| {
            json!({"day": day, "ticks": b.ticks, "alive_0600": b.alive_0600, "settlements_0600": b.settlements_0600,
                   "tick_ms": b.tick_us as f64 / 1000.0,
                   "systems_ms": b.system_ns.iter().map(|&ns| ns as f64 / 1e6).collect::<Vec<_>>()})
        })
        .collect();
    let samples: Vec<(u64, u64)> = days.values().filter_map(|b| b.alive_0600.zip(b.settlements_0600)).collect();
    let mean_of = |f: fn(&(u64, u64)) -> u64| if samples.is_empty() { 0.0 } else { samples.iter().map(f).sum::<u64>() as f64 / samples.len() as f64 };
    let slow: Vec<Value> = slowest
        .ticks
        .iter()
        .map(|t| {
            let (day, clock) = display_clock(t.tick, offset, tpd);
            json!({"tick": t.tick, "display_day": day, "clock": clock, "total_ms": f64::from(t.us) / 1000.0,
                   "top_systems": t.top.iter().map(|(n, us)| json!({"name": n, "ms": us / 1000.0})).collect::<Vec<_>>(),
                   "due_periodic": t.due_periodic})
        })
        .collect();
    let warm_us = &profile.tick_us[..warm];
    let snapshots = matches!(spec.mode.as_str(), "natural" | "stress" | "quick").then(|| snapshot_timings(&engine));
    let json = json!({
        "pass": 15, "mode": spec.mode, "scene": spec.scene, "seed": spec.seed,
        "agents_requested": spec.agents, "founders": founders, "side": spec.side, "feed": spec.feed.as_str(),
        "ticks": spec.ticks, "days": spec.ticks as f64 / tpd as f64, "ticks_per_day": tpd, "clock_offset_ticks": offset,
        "git_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
        "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
        "machine": machine_facts(),
        "contended": std::env::var("PERF_CONTENDED").is_ok_and(|v| v == "1"),
        "wall_seconds": wall_seconds,
        "warmup": {"ticks": warm, "tick0_ms": warm_us.first().map_or(0.0, |&v| f64::from(v) / 1000.0),
                   "summary": tick_summary(warm_us)},
        "tick_summary": tick_summary(&profile.tick_us[warm..]),
        "steady_tick_ms_total": steady_us as f64 / 1000.0,
        "prune_ms": prune_steady_ns as f64 / 1e6,
        "unaccounted_ms": steady_ns.saturating_sub(accounted_ns + prune_steady_ns) as f64 / 1e6,
        "systems": system_rows,
        "per_day": per_day,
        "slowest_ticks": slow,
        "fingerprint": crate::probe::fingerprint(&engine),
        "alive_end": alive(&engine),
        "settlements_end": engine.resources.settlements.len(),
        // Pass 19 (scale check): each settlement's members at the end
        // (largest first) and the chronicle's size (measurement only).
        "settlement_sizes_end": settlement_sizes(&engine),
        "chronicle_entries_end": engine.resources.chronicle.len(),
        "chronicle_people_end": engine.resources.chronicle.person_count(),
        "chronicle_approx_bytes_end": engine.resources.chronicle.approx_bytes(),
        "buildings_end": sim_bridge::ffi::world_node::collect_building_snapshot(&engine.resources).len(),
        "mean_alive_0600": mean_of(|s| s.0),
        "mean_settlements_0600": mean_of(|s| s.1),
        "gifts": gifts,
        "snapshot_timings": snapshots,
    });
    #[cfg(feature = "subprofile")]
    let json = {
        let mut json = json;
        json["subprofile"] = subprofile.summary(offset, tpd);
        json
    };
    RunOutput { json, tick_us: profile.tick_us }
}

/// Median and p90 (nearest rank) of `SNAPSHOT_REPEATS` timed calls of `f`, µs.
fn time_collector<T>(mut f: impl FnMut() -> T) -> (f64, f64) {
    let mut us: Vec<f64> = Vec::with_capacity(SNAPSHOT_REPEATS);
    for _ in 0..SNAPSHOT_REPEATS {
        let start = Instant::now();
        let out = std::hint::black_box(f());
        us.push(start.elapsed().as_nanos() as f64 / 1000.0);
        drop(out);
    }
    us.sort_by(f64::total_cmp);
    (percentile_f64(&us, 50.0), percentile_f64(&us, 90.0))
}

/// The Rust-side body of every `WorldSimNode` getter (all pure collectors
/// its `#[func]` calls, together; the `*_to_dict` conversions need Godot and
/// are not timed), [`SNAPSHOT_REPEATS`] calls each: median and p90 in µs, and
/// the row count where the getter returns rows.
pub fn snapshot_timings(engine: &SimEngine) -> Value {
    use sim_bridge::ffi::world_node as wn;
    let world = &engine.world;
    let r = &engine.resources;
    let rows = |n: usize| json!(n);
    let mut out = serde_json::Map::new();
    let mut put = |name: &str, (median, p90): (f64, f64), count: Value| {
        out.insert(name.into(), json!({"median_us": median, "p90_us": p90, "rows": count}));
    };
    let agents = wn::collect_agent_snapshot(world);
    put(
        "get_agent_snapshot",
        time_collector(|| {
            (
                wn::collect_agent_snapshot(world),
                wn::collect_agent_ages_days(world, engine.current_tick(), r.ticks_per_day),
                wn::collect_agent_colds(world),
            )
        }),
        rows(agents.len()),
    );
    let settlements = wn::collect_settlement_snapshot(world, &r.settlements);
    put(
        "get_settlement_snapshot",
        time_collector(|| {
            let rows = wn::collect_settlement_snapshot(world, &r.settlements);
            let targets = wn::collect_settlement_wood_targets(&rows, r);
            (rows, targets)
        }),
        rows(settlements.len()),
    );
    put("get_construction_snapshot", time_collector(|| wn::collect_construction_snapshot(world)), rows(wn::collect_construction_snapshot(world).len()));
    put("get_building_snapshot", time_collector(|| wn::collect_building_snapshot(r)), rows(wn::collect_building_snapshot(r).len()));
    put("get_resource_snapshot", time_collector(|| wn::collect_resource_snapshot(r)), rows(wn::collect_resource_snapshot(r).len()));
    put("get_recent_deaths", time_collector(|| wn::collect_recent_deaths(r)), rows(wn::collect_recent_deaths(r).len()));
    put("get_relationship_snapshot", time_collector(|| wn::collect_relationship_snapshot(r)), rows(wn::collect_relationship_snapshot(r).len()));
    put("get_world_info", time_collector(|| wn::collect_world_info(r)), Value::Null);
    put("get_time_snapshot", time_collector(|| wn::collect_time_snapshot(engine)), Value::Null);
    put("get_tallies", time_collector(|| wn::collect_tallies(r)), Value::Null);
    put("get_terrain_snapshot", time_collector(|| wn::collect_terrain_snapshot(r)), Value::Null);
    // `get_influence_overlay` hands Godot a copy of one channel's buffer; the
    // copy into a Vec stands in for the PackedByteArray copy.
    put(
        "get_influence_overlay",
        time_collector(|| r.influence_grid.current_buf(sim_core::influence::InfluenceChannel::all()[0]).to_vec()),
        Value::Null,
    );
    if let Some(first) = agents.first() {
        put("get_agent_detail", time_collector(|| wn::collect_agent_detail(world, first.entity_bits)), Value::Null);
        let (x, y) = (first.x as i32, first.y as i32);
        put("get_tile_causal_history", time_collector(|| wn::try_collect_tile_causal_history(r, x, y)), Value::Null);
    }
    // The renderer asks for the causal history of every distinct agent tile,
    // twice per frame (inventory): one whole pass over those tiles.
    let mut tiles: Vec<(i32, i32)> = agents.iter().map(|a| (a.x as i32, a.y as i32)).collect();
    tiles.sort_unstable();
    tiles.dedup();
    put(
        "get_tile_causal_history_all_agent_tiles",
        time_collector(|| tiles.iter().map(|&(x, y)| wn::try_collect_tile_causal_history(r, x, y).len()).sum::<usize>()),
        rows(tiles.len()),
    );
    let buildings = wn::collect_building_snapshot(r);
    out.insert(
        "context".into(),
        json!({"agents": agents.len(), "agent_tiles": tiles.len(), "buildings": buildings.len(),
               "hearths": buildings.iter().filter(|b| b.kind == sim_core::components::BuildingKind::Hearth.code()).count(),
               "settlements": settlements.len(), "construction_sites": wn::collect_construction_snapshot(world).len(),
               "repeats": SNAPSHOT_REPEATS}),
    );
    Value::Object(out)
}

/// Machine facts recorded in every run (`uname -a`, CPU, cores, memory,
/// `rustc -V`), gathered once per process.
pub fn machine_facts() -> Value {
    static FACTS: std::sync::OnceLock<Value> = std::sync::OnceLock::new();
    FACTS
        .get_or_init(|| {
            let cmd = |prog: &str, args: &[&str]| {
                std::process::Command::new(prog)
                    .args(args)
                    .output()
                    .ok()
                    .filter(|o| o.status.success())
                    .map_or_else(|| "unknown".to_string(), |o| String::from_utf8_lossy(&o.stdout).trim().to_string())
            };
            json!({
                "uname": cmd("uname", &["-a"]),
                "cpu": cmd("sysctl", &["-n", "machdep.cpu.brand_string"]),
                "ncpu": cmd("sysctl", &["-n", "hw.ncpu"]),
                "memsize": cmd("sysctl", &["-n", "hw.memsize"]),
                "rustc": cmd("rustc", &["-V"]),
            })
        })
        .clone()
}

/// Write a run's JSON (`<stem>.json`) and its `<stem>.tick_us.bin` sidecar
/// (every tick's µs as little-endian u32) into `dir`; returns the JSON file name.
pub fn write_run(dir: &Path, stem: &str, output: &RunOutput) -> Result<String, String> {
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let name = format!("{stem}.json");
    let text = serde_json::to_string_pretty(&output.json).map_err(|e| e.to_string())?;
    std::fs::write(dir.join(&name), text + "\n").map_err(|e| format!("{name}: {e}"))?;
    let bytes: Vec<u8> = output.tick_us.iter().flat_map(|v| v.to_le_bytes()).collect();
    std::fs::write(dir.join(format!("{stem}.tick_us.bin")), bytes).map_err(|e| format!("{stem}: {e}"))?;
    Ok(name)
}

/// Merge `entries` into `dir/runs.json` (entries with the same `file` or,
/// for failed runs, the same `stem` are replaced; order kept).
pub fn update_index(dir: &Path, entries: &[Value]) -> Result<(), String> {
    let path = dir.join("runs.json");
    let mut runs: Vec<Value> = std::fs::read_to_string(&path)
        .ok()
        .and_then(|text| serde_json::from_str::<Value>(&text).ok())
        .and_then(|v| v["runs"].as_array().cloned())
        .unwrap_or_default();
    for entry in entries {
        let key = |v: &Value| v["stem"].as_str().map(str::to_string);
        match runs.iter().position(|r| key(r) == key(entry)) {
            Some(i) => runs[i] = entry.clone(),
            None => runs.push(entry.clone()),
        }
    }
    let text = serde_json::to_string_pretty(&json!({"pass": 15, "runs": runs})).map_err(|e| e.to_string())?;
    std::fs::write(&path, text + "\n").map_err(|e| format!("{}: {e}", path.display()))
}

/// The index entry of a finished run.
pub fn index_entry(stem: &str, output: &RunOutput) -> Value {
    let j = &output.json;
    json!({"stem": stem, "file": format!("{stem}.json"), "status": "ok", "mode": j["mode"], "scene": j["scene"], "seed": j["seed"],
           "agents_requested": j["agents_requested"], "side": j["side"], "feed": j["feed"], "ticks": j["ticks"],
           "mean_ms": j["tick_summary"]["mean_ms"], "p999_ms": j["tick_summary"]["p999_ms"], "max_ms": j["tick_summary"]["max_ms"],
           "mean_alive_0600": j["mean_alive_0600"], "wall_seconds": j["wall_seconds"], "contended": j["contended"]})
}

/// Build and run `spec`, write its files into `dir`, and return its index
/// entry (a worldgen failure is recorded as `status: worldgen_failed`, a
/// panic during the run as `status: run_failed`).
pub fn run_and_write(dir: &Path, spec: &RunSpec) -> Result<Value, String> {
    let engine = match build_engine(spec) {
        Ok(engine) => engine,
        Err(error) => {
            let stem = spec.stem(0);
            eprintln!("{stem}: worldgen failed: {error}");
            return Ok(json!({"stem": stem, "status": "worldgen_failed", "error": error, "mode": spec.mode, "seed": spec.seed,
                             "agents_requested": spec.agents, "side": spec.side, "feed": spec.feed.as_str()}));
        }
    };
    let stem = spec.stem(alive(&engine));
    let spec_copy = spec.clone();
    let output = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(move || run(&spec_copy, engine))) {
        Ok(output) => output,
        Err(payload) => {
            let error = panic_text(payload.as_ref());
            eprintln!("{stem}: run failed: {error}");
            return Ok(json!({"stem": stem, "status": "run_failed", "error": error, "mode": spec.mode, "seed": spec.seed,
                             "agents_requested": spec.agents, "side": spec.side, "feed": spec.feed.as_str()}));
        }
    };
    write_run(dir, &stem, &output)?;
    let entry = index_entry(&stem, &output);
    eprintln!(
        "{stem}: mean {:.4} ms p99.9 {:.3} ms max {:.3} ms alive {} wall {:.1} s",
        output.json["tick_summary"]["mean_ms"].as_f64().unwrap_or(0.0),
        output.json["tick_summary"]["p999_ms"].as_f64().unwrap_or(0.0),
        output.json["tick_summary"]["max_ms"].as_f64().unwrap_or(0.0),
        output.json["alive_end"],
        output.json["wall_seconds"].as_f64().unwrap_or(0.0)
    );
    Ok(entry)
}

/// Overhead check (7.0): the natural world run with `tick()` and with
/// `tick_profiled()`, alternating, `repeats` times each.
pub fn overhead(scene: &str, seed: u64, ticks: u64, repeats: u32) -> Value {
    let mut plain = Vec::new();
    let mut profiled = Vec::new();
    let mut fingerprints = Vec::new();
    for _ in 0..repeats {
        let mut engine = crate::probe::scene_engine(scene, seed);
        let start = Instant::now();
        for _ in 0..ticks {
            engine.tick();
        }
        plain.push(start.elapsed().as_secs_f64());
        let plain_fp = crate::probe::fingerprint(&engine);
        let mut engine = crate::probe::scene_engine(scene, seed);
        let mut profile = engine.new_tick_profile();
        let start = Instant::now();
        for _ in 0..ticks {
            engine.tick_profiled(&mut profile);
        }
        profiled.push(start.elapsed().as_secs_f64());
        fingerprints.push(plain_fp == crate::probe::fingerprint(&engine));
    }
    let median = |v: &[f64]| {
        let mut s = v.to_vec();
        s.sort_by(f64::total_cmp);
        if s.is_empty() { 0.0 } else if s.len() % 2 == 1 { s[s.len() / 2] } else { (s[s.len() / 2 - 1] + s[s.len() / 2]) / 2.0 }
    };
    let (mp, mq) = (median(&plain), median(&profiled));
    json!({"pass": 15, "mode": "overhead", "scene": scene, "seed": seed, "ticks": ticks, "repeats": repeats,
           "plain_wall_seconds": plain, "profiled_wall_seconds": profiled,
           "median_plain_seconds": mp, "median_profiled_seconds": mq,
           "overhead_pct": if mp > 0.0 { (mq / mp - 1.0) * 100.0 } else { 0.0 },
           "same_fingerprint": fingerprints,
           "git_sha": std::env::var("WORLDSIM_GIT_SHA").unwrap_or_else(|_| "unknown".into()),
           "build_profile": if cfg!(debug_assertions) { "debug" } else { "release" },
           "machine": machine_facts(), "contended": std::env::var("PERF_CONTENDED").is_ok_and(|v| v == "1")})
}

/// Parsed `worldsim_perf` command line.
#[derive(Debug, Clone, PartialEq)]
pub struct PerfArgs {
    /// Subcommand.
    pub mode: String,
    /// Scene for `natural` / `overhead`.
    pub scene: String,
    /// Seeds (half-open range).
    pub seeds: std::ops::Range<u64>,
    /// Founder counts for `stress`.
    pub agents: Vec<u32>,
    /// Map sides (`map`), or the one side of `stress`.
    pub sides: Vec<u32>,
    /// Simulated days.
    pub days: u64,
    /// Gift policy for `stress`.
    pub feed: Feed,
    /// Overhead repeats.
    pub repeats: u32,
    /// Output directory.
    pub out: PathBuf,
}

/// The usage line.
pub const USAGE: &str = "usage: worldsim_perf natural --scene game_v19 --seeds A..B --days D --out DIR\n       worldsim_perf stress --seeds A..B --agents 25,50,... --side 64 --days 2 --feed fed|raw [--rules game_v19|game_v20|game_v20_inert|game_v21|game_v22|game_v23|game_v24|game_v25|game_v26] --out DIR\n       worldsim_perf map --seeds A..B --sides 64,128,256 --days 2 --out DIR\n       worldsim_perf overhead --seed S --days D --repeats 3 --out DIR\n       worldsim_perf quick --out DIR";

impl PerfArgs {
    /// Parse `args` (without the program name).
    pub fn parse(args: &[String]) -> Result<Self, String> {
        let mode = args.first().cloned().ok_or(USAGE)?;
        let mut parsed = PerfArgs {
            mode: mode.clone(),
            scene: "game_v19".into(),
            seeds: 15100..15101,
            agents: vec![500],
            sides: vec![64],
            days: 1,
            feed: Feed::Fed,
            repeats: 3,
            out: PathBuf::new(),
        };
        let list = |v: &str| v.split(',').map(|x| x.trim().parse::<u32>().map_err(|e| format!("{v}: {e}"))).collect::<Result<Vec<_>, _>>();
        // Pass 16: `--rules` picks the stress world's rules (default game_v19).
        let mut rules: Option<String> = None;
        let mut i = 1;
        while i < args.len() {
            let value = args.get(i + 1).ok_or_else(|| format!("{} needs a value", args[i]))?;
            match args[i].as_str() {
                "--scene" => parsed.scene = value.clone(),
                "--rules" => {
                    if !matches!(value.as_str(), "game_v19" | "game_v20" | "game_v20_inert" | "game_v21" | "game_v22" | "game_v23" | "game_v24" | "game_v25" | "game_v26") {
                        return Err(format!("--rules game_v19|game_v20|game_v20_inert|game_v21|game_v22|game_v23|game_v24|game_v25|game_v26, not {value}"));
                    }
                    rules = Some(value.clone());
                }
                "--seeds" => {
                    let (a, b) = value.split_once("..").ok_or("--seeds takes A..B")?;
                    parsed.seeds = a.parse::<u64>().map_err(|e| e.to_string())?..b.parse::<u64>().map_err(|e| e.to_string())?;
                }
                "--seed" => {
                    let s = value.parse::<u64>().map_err(|e| e.to_string())?;
                    parsed.seeds = s..s + 1;
                }
                "--agents" => parsed.agents = list(value)?,
                "--side" | "--sides" => parsed.sides = list(value)?,
                "--days" => parsed.days = value.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
                "--repeats" => parsed.repeats = value.parse().map_err(|e: std::num::ParseIntError| e.to_string())?,
                "--feed" => {
                    parsed.feed = match value.as_str() {
                        "fed" => Feed::Fed,
                        "raw" => Feed::Raw,
                        other => return Err(format!("--feed fed|raw, not {other}")),
                    }
                }
                "--out" => parsed.out = PathBuf::from(value),
                other => return Err(format!("unknown flag {other}\n{USAGE}")),
            }
            i += 2;
        }
        if !matches!(mode.as_str(), "natural" | "stress" | "map" | "overhead" | "quick") {
            return Err(format!("unknown subcommand {mode}\n{USAGE}"));
        }
        if mode == "quick" {
            parsed.seeds = 15100..15101;
            parsed.agents = vec![500];
            parsed.sides = vec![64];
            parsed.days = 1;
            parsed.feed = Feed::Fed;
        }
        if parsed.out.as_os_str().is_empty() {
            return Err(format!("--out is required\n{USAGE}"));
        }
        // Stress, quick and map ignore `--scene` (as before); `--rules` is
        // kept in `scene` for them, and only for them.
        match (mode.as_str(), rules) {
            ("stress" | "quick" | "map", rules) => parsed.scene = rules.unwrap_or_else(|| "game_v19".into()),
            (_, Some(_)) => return Err(format!("--rules applies to stress, quick and map\n{USAGE}")),
            (_, None) => {}
        }
        if matches!(mode.as_str(), "natural" | "overhead") && !parsed.scene.starts_with("game") {
            return Err(format!("--scene must be a game scene, not {}", parsed.scene));
        }
        if parsed.seeds.is_empty() || parsed.days == 0 || parsed.agents.is_empty() || parsed.sides.is_empty() {
            return Err("empty seeds, days, agents or sides".into());
        }
        Ok(parsed)
    }

    /// Run the subcommand; returns the number of runs written.
    pub fn execute(&self) -> Result<usize, String> {
        let ticks = self.days * 1440;
        let dir = self.out.as_path();
        std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        let mut written = 0usize;
        let mut record = |entry: Value| -> Result<(), String> {
            written += usize::from(entry["status"] == "ok");
            update_index(dir, &[entry])
        };
        match self.mode.as_str() {
            "overhead" => {
                let result = overhead(&self.scene, self.seeds.start, ticks, self.repeats);
                let name = format!("overhead_{}_s{}_d{}.json", self.scene, self.seeds.start, self.days);
                let text = serde_json::to_string_pretty(&result).map_err(|e| e.to_string())?;
                std::fs::write(dir.join(&name), text + "\n").map_err(|e| e.to_string())?;
                eprintln!("{name}: overhead {:.2} %", result["overhead_pct"].as_f64().unwrap_or(0.0));
                return Ok(1);
            }
            "natural" => {
                for seed in self.seeds.clone() {
                    let spec = RunSpec { mode: "natural".into(), scene: self.scene.clone(), seed, agents: None, side: 64, feed: Feed::None, ticks };
                    record(run_and_write(dir, &spec)?)?;
                }
            }
            "stress" | "quick" => {
                let side = self.sides[0];
                'sizes: for &agents in &self.agents {
                    for seed in self.seeds.clone() {
                        let spec = RunSpec { mode: self.mode.clone(), scene: stress_label(&self.scene).into(), seed, agents: Some(agents), side, feed: self.feed, ticks };
                        let entry = run_and_write(dir, &spec)?;
                        let slow = entry["wall_seconds"].as_f64().is_some_and(|w| w > SIZE_WALL_LIMIT_SECONDS);
                        record(entry)?;
                        if slow {
                            eprintln!("n = {agents}: a run exceeded {SIZE_WALL_LIMIT_SECONDS} s; the remaining seeds of this size and every larger size are skipped");
                            record(json!({"stem": format!("size_limit_n{agents}_{}", self.feed.as_str()), "status": "size_limit", "agents_requested": agents, "after_seed": seed}))?;
                            break 'sizes;
                        }
                    }
                }
            }
            "map" => {
                for &side in &self.sides {
                    for seed in self.seeds.clone() {
                        let founders = WorldGenConfig::game_v19(seed).starting_agents;
                        let spec = RunSpec { mode: "map".into(), scene: stress_label(&self.scene).into(), seed, agents: Some(founders), side, feed: Feed::Fed, ticks };
                        record(run_and_write(dir, &spec)?)?;
                    }
                }
            }
            _ => return Err(USAGE.into()),
        }
        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiled_and_plain_game_v19_reach_the_same_fingerprint() {
        // Pass 15 6.1 equivalence: seed 7, 3,000 ticks.
        let mut plain = crate::scenes::game_v19_scene(7);
        let mut profiled = crate::scenes::game_v19_scene(7);
        let mut profile = profiled.new_tick_profile();
        for _ in 0..3_000 {
            plain.tick();
            profiled.tick_profiled(&mut profile);
        }
        assert_eq!(crate::probe::fingerprint(&plain), crate::probe::fingerprint(&profiled));
        assert_eq!(profile.ticks, 3_000);
        assert_eq!(profile.tick_us.len(), 3_000);
        let names: Vec<&str> = profile.systems.iter().map(|s| s.name.as_str()).collect();
        assert_eq!(names, plain.system_names());
        // Interval-1 systems ran every tick.
        assert!(profile.systems.iter().filter(|s| s.interval == 1).all(|s| s.calls == 3_000));
    }

    #[test]
    fn nearest_rank_percentiles() {
        let v: Vec<u32> = (1..=1000).collect();
        assert_eq!((percentile(&v, 50.0), percentile(&v, 90.0), percentile(&v, 99.9), percentile(&v, 100.0)), (500, 900, 999, 1000));
        assert_eq!(percentile(&[7], 99.9), 7);
        assert_eq!(percentile(&[], 50.0), 0);
        let s = tick_summary(&[1000, 9000, 20_000, 3000]);
        assert_eq!(s["mean_ms"], 8.25);
        assert_eq!((s["n_over_8ms"].as_u64(), s["n_over_16_67ms"].as_u64(), s["max_ms"].as_f64()), (Some(2), Some(1), Some(20.0)));
    }

    #[test]
    fn slowest_ticks_keep_the_twenty_largest() {
        let mut slow = SlowestTicks::default();
        for (tick, us) in [5u32, 9, 1, 30, 7, 30, 2, 8, 50, 4, 3, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 0, 60].iter().enumerate() {
            if slow.admits(*us) {
                slow.insert(SlowTick { tick: tick as u64, us: *us, top: vec![], due_periodic: vec![] });
            }
        }
        let kept: Vec<u32> = slow.ticks.iter().map(|t| t.us).collect();
        assert_eq!(kept.len(), SLOWEST_KEPT);
        assert_eq!(&kept[..4], &[60, 50, 30, 30]);
        assert!(kept.windows(2).all(|w| w[0] >= w[1]));
        assert!(!kept.contains(&0) && !kept.contains(&1) && !kept.contains(&2));
        // Ties keep the earlier tick first.
        assert_eq!((slow.ticks[2].tick, slow.ticks[3].tick), (3, 5));
    }

    #[test]
    fn display_clock_uses_the_offset() {
        assert_eq!(display_clock(0, 480, 1440), (0, "08:00".into()));
        assert_eq!(display_clock(960, 480, 1440), (1, "00:00".into()));
        assert_eq!(display_clock(1319, 480, 1440), (1, "05:59".into()));
    }

    #[test]
    fn a_short_fed_stress_run_reports_consistent_numbers() {
        let spec = RunSpec { mode: "stress".into(), scene: "stress_v19".into(), seed: 15100, agents: Some(40), side: 64, feed: Feed::Fed, ticks: 1500 };
        let engine = build_engine(&spec).expect("worldgen");
        let out = run(&spec, engine);
        let j = &out.json;
        assert_eq!(out.tick_us.len(), 1500);
        assert_eq!(j["founders"], 40);
        assert_eq!(j["tick_summary"]["ticks"], 1500 - WARMUP_TICKS);
        assert_eq!(j["warmup"]["ticks"], WARMUP_TICKS);
        // Pass 19: settlement sizes sum to the members; the chronicle recorded the founders.
        let sizes = j["settlement_sizes_end"].as_array().expect("sizes");
        assert_eq!(sizes.len() as u64, j["settlements_end"].as_u64().expect("count"));
        assert!(j["chronicle_people_end"].as_u64().expect("people") >= 40);
        assert!(j["chronicle_entries_end"].as_u64().expect("entries") >= 40, "founding members");
        assert!(j["chronicle_approx_bytes_end"].as_u64().expect("bytes") > 0);
        // Per-day ticks cover every tick, and the per-day system sums equal the whole-run sums.
        let days = j["per_day"].as_array().expect("days");
        assert_eq!(days.iter().map(|d| d["ticks"].as_u64().unwrap_or(0)).sum::<u64>(), 1500);
        let systems = j["systems"].as_array().expect("systems");
        for (i, s) in systems.iter().enumerate() {
            let per_day: f64 = days.iter().map(|d| d["systems_ms"][i].as_f64().unwrap_or(0.0)).sum();
            let whole = s["total_ms_incl_warmup"].as_f64().unwrap_or(0.0);
            assert!((per_day - whole).abs() <= 1e-6 * whole.max(1.0), "{}: {per_day} vs {whole}", s["name"]);
        }
        let shares: f64 = systems.iter().map(|s| s["share_of_tick_time"].as_f64().unwrap_or(0.0)).sum();
        assert!(shares > 0.0 && shares <= 1.0 + 1e-9, "shares {shares}");
        // Fed: the first gift right after the first settlement, then one per 06:00 (day 1 at tick 1320).
        let gifts = j["gifts"].as_array().expect("gifts");
        assert!(!gifts.is_empty());
        assert!(gifts.iter().skip(1).all(|g| (g["tick"].as_u64().unwrap_or(0) + 480) % 1440 == 360), "{gifts:?}");
        assert!(gifts.iter().all(|g| g["requested"].as_u64() > Some(0)));
        assert!(j["slowest_ticks"].as_array().is_some_and(|s| s.len() == SLOWEST_KEPT));
        assert!(j["slowest_ticks"][0]["tick"].as_u64() >= Some(WARMUP_TICKS));
        assert!(j["snapshot_timings"]["get_agent_snapshot"]["median_us"].as_f64().is_some());
        assert!(j["per_day"][1]["alive_0600"].as_u64().is_some());
        // The fed run differs from a raw run of the same world only through the gifts.
        let raw = RunSpec { feed: Feed::Raw, ..spec.clone() };
        let raw_out = run(&raw, build_engine(&raw).expect("worldgen"));
        assert!(raw_out.json["gifts"].as_array().is_some_and(Vec::is_empty));
        // Determinism: the same spec gives the same fingerprint.
        let again = run(&spec, build_engine(&spec).expect("worldgen"));
        assert_eq!(again.json["fingerprint"], j["fingerprint"]);
    }

    #[test]
    fn files_and_index_round_trip() {
        let dir = std::env::temp_dir().join(format!("worldsim_perf_test_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let spec = RunSpec { mode: "map".into(), scene: "stress_v19".into(), seed: 3, agents: Some(24), side: 64, feed: Feed::Fed, ticks: 70 };
        let entry = run_and_write(&dir, &spec).expect("run");
        assert_eq!(entry["status"], "ok");
        assert_eq!(entry["file"], "map_stress_v19_s3_n24_side64_fed.json");
        update_index(&dir, std::slice::from_ref(&entry)).expect("index");
        update_index(&dir, &[entry]).expect("index");
        let index: Value = serde_json::from_str(&std::fs::read_to_string(dir.join("runs.json")).expect("read")).expect("json");
        assert_eq!(index["runs"].as_array().map(Vec::len), Some(1));
        let bin = std::fs::read(dir.join("map_stress_v19_s3_n24_side64_fed.tick_us.bin")).expect("bin");
        assert_eq!(bin.len(), 70 * 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn natural_spec_names_the_scene_founders() {
        let spec = RunSpec { mode: "natural".into(), scene: "game_v19".into(), seed: 15000, agents: None, side: 64, feed: Feed::None, ticks: 1 };
        let engine = build_engine(&spec).expect("scene");
        assert!(engine.resources.world_variety.is_some());
        assert_eq!(spec.stem(alive(&engine)), format!("natural_game_v19_s15000_n{}_side64_none", alive(&engine)));
    }

    #[test]
    fn cli_parses_each_subcommand() {
        let a = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        let p = PerfArgs::parse(&a("stress --seeds 15100..15108 --agents 25,50 --side 64 --days 2 --feed raw --out x")).expect("stress");
        assert_eq!((p.seeds, p.agents, p.sides, p.days, p.feed), (15100..15108, vec![25, 50], vec![64], 2, Feed::Raw));
        let q = PerfArgs::parse(&a("quick --out y")).expect("quick");
        assert_eq!((q.seeds, q.agents, q.days, q.feed), (15100..15101, vec![500], 1, Feed::Fed));
        assert!(PerfArgs::parse(&a("natural --scene legacy --seeds 0..1 --days 1 --out z")).is_err());
        assert!(PerfArgs::parse(&a("quick")).is_err());
        assert!(PerfArgs::parse(&a("bogus --out z")).is_err());
        assert!(PerfArgs::parse(&a("stress --seeds 1..2 --frobnicate 3 --out z")).is_err());
        let o = PerfArgs::parse(&a("overhead --seed 15000 --days 30 --repeats 3 --out w")).expect("overhead");
        assert_eq!((o.seeds, o.days, o.repeats), (15000..15001, 30, 3));
    }

    #[test]
    fn rules_flag_picks_the_stress_rules_and_label() {
        // Pass 16 8.5: --rules (default game_v19, today's behaviour; --scene stays ignored by stress).
        let a = |s: &str| s.split_whitespace().map(String::from).collect::<Vec<_>>();
        for (flag, label) in [("", "stress_v19"), ("--rules game_v19", "stress_v19"), ("--rules game_v20", "stress_v20"), ("--rules game_v20_inert", "stress_v20_inert"), ("--rules game_v21", "stress_v21"), ("--rules game_v22", "stress_v22"), ("--rules game_v23", "stress_v23"), ("--rules game_v24", "stress_v24"), ("--rules game_v25", "stress_v25"), ("--rules game_v26", "stress_v26")] {
            let p = PerfArgs::parse(&a(&format!("stress --seeds 1..2 {flag} --out x"))).expect("stress");
            assert_eq!(stress_label(&p.scene), label, "{flag}");
        }
        let ignored = PerfArgs::parse(&a("stress --scene game_v16 --seeds 1..2 --out x")).expect("stress");
        assert_eq!(stress_label(&ignored.scene), "stress_v19");
        assert!(PerfArgs::parse(&a("stress --rules game_v18 --out x")).is_err());
        // Pass 19 7.6: game_v24 maps to stress_v24 and WorldRules::game_v24
        // (an unknown label would silently fall back to game_v19).
        assert_eq!(stress_rules(stress_label("game_v24"), 4), WorldRules::game_v24(4));
        assert_ne!(stress_rules(stress_label("game_v24"), 4), WorldRules::game_v23(4));
        // Pass 20: game_v25 maps to stress_v25 and WorldRules::game_v25.
        assert_eq!(stress_rules(stress_label("game_v25"), 4), WorldRules::game_v25(4));
        assert_ne!(stress_rules(stress_label("game_v25"), 4), WorldRules::game_v24(4));
        // Pass 21: game_v26 maps to stress_v26 and WorldRules::game_v26; the
        // next version is still rejected.
        assert!(PerfArgs::parse(&a("stress --rules game_v27 --out x")).is_err());
        assert_eq!(stress_rules(stress_label("game_v26"), 4), WorldRules::game_v26(4));
        assert_ne!(stress_rules(stress_label("game_v26"), 4), WorldRules::game_v25(4));
        assert!(PerfArgs::parse(&a("natural --rules game_v20 --seeds 0..1 --days 1 --out z")).is_err());
        let q = PerfArgs::parse(&a("quick --out y")).expect("quick");
        assert_eq!(stress_label(&q.scene), "stress_v19");
        for (label, rules) in [("stress_v19", WorldRules::game_v19(9)), ("stress_v20", WorldRules::game_v20(9)), ("stress_v20_inert", WorldRules::game_v20_inert(9)), ("stress_v21", WorldRules::game_v21(9)), ("stress_v22", WorldRules::game_v22(9)), ("stress_v23", WorldRules::game_v23(9)), ("stress_v24", WorldRules::game_v24(9)), ("stress_v25", WorldRules::game_v25(9)), ("stress_v26", WorldRules::game_v26(9))] {
            let spec = RunSpec { mode: "stress".into(), scene: label.into(), seed: 9, agents: Some(30), side: 64, feed: Feed::Fed, ticks: 1 };
            assert_eq!(build_engine(&spec).expect("engine").resources.rules, rules);
            assert!(spec.stem(30).starts_with(&format!("stress_{label}_s9_n30")));
        }
    }
}
