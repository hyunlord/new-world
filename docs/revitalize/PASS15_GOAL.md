# GOAL: WorldSim — Revitalize Pass 15: how many people can this world hold? A performance baseline and a per-agent budget before the game grows (no rule changes)

Repository `hyunlord/new-world`. Continue the simulation line: `codex/revitalize-2026-10` = `f5373ab2b7679c4a0d3a5c86b256971c86f93d41` (Pass 14, gate green), `codex/revitalize-2026-10-wip` = `736c71971b90166038ac461e3b332a1e56693f38` (Pass 14's report and gate logs after it). Draft PR #510.
This run is executed by **Claude Code on the Mac** in the simulation clone, unattended. **Everything runs on the Mac** this time — the Mac Studio is the machine the game is played on, so its numbers are the ones that matter. At the same time a world-model run (WM-2) works in `/Users/rexxa/work/new-world-wm1`; its Mac side is light (git, small unit tests) and its heavy jobs are on the DGX. No art run is active.

**This pass changes no game rule.** Every existing scene (`game_v8` … `game_v19`) must behave bit-for-bit as before; the research dataset is frozen on `game_v19`. It adds measurement only: a per-system timer, a stress world, a benchmark binary and a budget document. Its product is `docs/revitalize/PERF_BUDGET.md`: how much a tick costs today, which systems grow faster than the population, how many people the current code can simulate inside a frame budget, and the rule every later game pass must meet.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 736c7197
cp ~/Downloads/claude_goal_worldsim_pass15.md docs/revitalize/PASS15_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS15_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only; never stage outputs or gate logs of earlier passes.

**This document is `docs/revitalize/PASS15_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript, so the section 12 report, the P table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As Pass 14's 0.2: no `install_hooks.sh`, never `lead/main` or `main`, no `harness_pipeline.sh` (section 4.4 is the gate), built-in `general-purpose` subagents only, a blocking hook other than `/goal` → record, push, report. CLAUDE.md's "Current target ~500 entities" is this pass's target population. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 15`; times in the same section.
2. **Do not end your turn between Parts or while a timed batch runs.** Poll long batches every 10–20 minutes with short calls.
3. Review loops: two rounds maximum. Push after every Part.
4. On failure: work around, note it, continue. A failed budget is a valid result; a missing number is not.
5. **Time rule:** past **10 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 6.
6. **Drop order:** the map sweep (7.3) → the `raw` stress variant (7.2) → the n = 2,000 stress size. Never drop Parts 0–1, the natural sweep (7.1), the `fed` stress sweep up to n = 1,000, the snapshot timings (6.5), `PERF_BUDGET.md` or Part 6.

---

# ⚠️ 1. MACHINE AND TIMING HYGIENE
- **Mac** (`/Users/rexxa/work/new-world-pass4`): edit, build, test, benchmark. Never touch `/Users/rexxa/work/new-world-wm1`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout; never stop a process you did not start. **No DGX use** in this pass (the WM-2 run owns it).
- **Timed runs** (everything in Part 3 and the overhead check): release build only; **one timed process at a time** (no parallel workers); never at the same time as a cargo build, a cargo test or the gate. Before every timed batch record `date`, `sysctl -n vm.loadavg`, `pgrep -fl "Godot|cargo|rustc|worldsim_"`; start only when the 1-minute load average is < 4 and no `Godot` process runs (the owner may be playtesting the game on this Mac). Otherwise wait 10 minutes and check again; after 2 hours of waiting, run anyway and mark the batch `contended` in its output and in RUNLOG.
- Record once: `uname -a`, `sysctl -n machdep.cpu.brand_string hw.ncpu hw.memsize`, `rustc -V`, `cargo -V`, the git SHA of every build.
- The simulation is single-threaded (no rayon in `sim-engine` / `sim-systems`); report that and do not add threads.

---

# 2. GIT
- Commits to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after.
- Commit messages `revitalize(p15-N): …`.
- **May change:** `rust/crates/sim-engine/src/lib.rs` (the profiled tick, 6.1), `rust/crates/sim-systems/src/worldgen/mod.rs` (the stress configuration, 6.2), `rust/crates/sim-test/**` (the benchmark binary and its tests), `rust/crates/sim-bridge/src/ffi/world_node.rs` (one debug `#[func]`, 6.4), the one optional fix of Part 5, `tools/revitalize/**`, `docs/revitalize/**`. **Must not change:** every other path — `scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `docs/art/**`, `tools/art_pipeline/**`, `tools/worldmodel/**`, `docs/worldmodel/**`, every `rules.rs` constant and scene constructor.
- **No new field in any existing public struct** (`WorldGenConfig`, `WorldRules`, `SimResources`, `SimEngine`, the snapshot row types). The art branch builds some of them as struct literals; a new field broke its build in Art Pass 7. New types are fine.

---

# 3. CONTEXT (planner's review)
- **How the game runs** (`sim-bridge/src/ffi/world_node.rs` L146–215): `WorldSimNode` builds `game_v19(DEFAULT_GAME_SEED)` and drives the engine from Godot's `process` with a Gaffer accumulator: fixed step `FIXED_DT = 1/30` s (30 ticks per second at speed 1), speed multiplier clamped to 0–4 (keys 1–4 give 0.25× / 0.5× / 1× / 2×; the API allows 4×), at most `MAX_ITERS_PER_FRAME = 5` ticks per frame. One tick is one game minute: a day is 1,440 ticks, 48 s at 1×, 12 s at 4×. At 4× and 60 FPS the engine runs 2 ticks per frame on average.
- **What one tick costs today:** Pass 13's round-3 probe of `game_v19` (seeds 0–32, 120 days; `docs/revitalize/probe/p13-3_round3_v19_120day/summary.json` `timing`) reports `ms_per_agent_tick` median 0.0025 (p90 0.0031, max 0.0040; it excludes the determinism twin and observers, which `wall_seconds` includes). With a median of 25–66 people alive over the year that is **≈ 0.1 ms per tick** (≈ 0.4 ms in the rare ~150-person world). That is an average: it hides which systems cost what, how the cost grows with the population, and the occasional slow tick that drops a frame. (There are no daily-interval systems — intervals are 1, 5, 6, 10, 60 and 120 — so day-boundary work happens inside interval-1 systems at clock boundaries.)
- **What is not known:** the cost at the ~500 people CLAUDE.md names as the current target, whether any system is quadratic in the population (the V7 reset of May 2026 was caused by O(n²) interactions and 59 systems, "30 people below 60 FPS"), and the cost of the UI's reads: several GDScript files call the same snapshot getters every frame (the planner's review found `get_agent_snapshot()` called from `_process` in six files on this branch — `agent_renderer.gd`, `need_bar_renderer.gd`, `seek_viz_renderer.gd`, `activity_trail_renderer.gd`, `panels/hud_status_panel.gd`, `panels/hud_topbar.gd` — plus `camera_controller.gd` when auto-track is on, and `get_settlement_snapshot()` ×3, `get_construction_snapshot()` ×2, `get_building_snapshot()`, `get_recent_deaths()`, `get_world_info()` per frame; the UI lives on the art branch, which may differ — 6.5 inventories both).
- **Why now:** the game track's next systems (temperament, family, chronicle UI, inter-village trade; `WorldSim_통합_로드맵_2026-10.md` section 6) all add per-agent work. A budget measured before them lets each later pass check its own cost.
- **The world grows only to ~150 people by itself** (one camp of 18–30 founders, `max_settlements` 4, villages of ~30–45). So the population sweep needs a **stress world** (6.2): the `game_v19` world and rules with more founders. It is a pessimistic stress, not a forecast of a real game: worldgen puts every founder on the ring 1–3 tiles from the camp (≤ 43 tiles, `camp.rs` L105–119), so people per tile grows with n and same-tile peer scans (`agent_decision.rs` ≈ L1106–1141, L1283) look superlinear because of crowding, not code; and the first settlement admits everyone in radius (`settlement_system.rs` ≈ L480) although `SETTLEMENT_MAX_POP` is 50. Stress slopes are therefore **density-confounded upper bounds**. The natural sweep (7.1) gives the realistic slopes over its own range (≈ 20–150 alive, per day), and 7.4 reports both.

---

# 4. RULES

## 4.1 Nothing changes behaviour
Every change of this pass is either new code that no existing scene calls (the stress configuration, the benchmark binary, the debug `#[func]`) or a refactor proved behaviour-neutral (the profiled tick, the optional fix). Identity checks I1–I3 (section 5 item 3) on the base build and on the final build must agree.

## 4.2 Seeds
Natural sweep: **15000–15031** (32 worlds). Stress and map sweeps: **15100–15107** (8 worlds). Not used before, not in tests (tests use seeds < 1000). Identity checks use the seeds named in section 5 item 3. Seeds 12000–12031, 14000–14063 (Pass 14), 24000–26063 (pilots), 30000–33999 (WM-1.3), 40000–40559 and 42000–42559 (WM-2) are not used.

## 4.3 The budget (what the verdicts measure)
- **Frame:** 60 FPS = 16.67 ms. The simulation may use at most half of a frame at the maximum speed 4× (2 ticks per frame on average; V7's budget of ~4 ms of simulation per frame is the same order). Hence:
- **B1 (mean tick):** mean tick time ≤ **4.0 ms**.
- **B2 (slow ticks):** the 99.9th percentile of tick times ≤ **8.0 ms**; also report the maximum and the number of ticks > 16.67 ms (each is a dropped frame at any speed).
- **B3 (UI reads):** one call of every per-frame getter's Rust-side body (6.5), summed, ≤ **2.0 ms**; reported a second time multiplied by each getter's per-frame call count (6.5 inventory).
- **Target population:** 500 alive.
- All numbers are this Mac's; the report states them as such and recommends a factor-2 margin for slower machines (`N_max/2`).

## 4.4 The gate
`tools/revitalize/gate_pass15.sh <log_dir>` = `gate_pass13.sh`'s steps unchanged (prepare, debug and release `cargo test --workspace`, both clippy runs, the forbidden-math grep, bridge build, Godot import, strict check, smoke, both Python discoveries — `tools/revitalize` and `tools/worldmodel` with `WM_BIN=/nonexistent PROBE_BIN=/nonexistent` so no recorder runs inside the gate; record the skip counts), plus: `worldsim_perf quick` (6.3; release build, output to `rust/target/p15_quick/`, not committed) exits 0 and writes valid JSON; `git diff --name-only f5373ab2..HEAD` lists only paths section 2 allows. Never run the gate during a timed batch.

---

# 5. PART 0 — Setup and identity at the base
1. Record start time and section 1's machine facts. Commit `PASS15_GOAL.md`; add `## Pass 15` to DECISIONS.md and METRICS.md; create the RUNLOG table in DECISIONS.md's Pass 15 section (Part, start, end, notes).
2. Build the **base** binaries in a separate worktree `../new-world-p15-base` detached at `f5373ab2` (`git worktree add --detach`; remove it at the end of the pass): `cargo build --release -p sim-test` there; record the sha256 of `worldsim_probe` and `worldsim_wm`.
3. **Identity references** (with the base build; probe outputs under `docs/revitalize/probe/p15-0_base/` — commit the `summary.json` files only; recorder episodes under `/Users/rexxa/work/p15-data/` outside the repository — commit only the list of their array sha256 values as `docs/revitalize/probe/p15-0_base/i2_sha256.txt`):
   - **I1:** `worldsim_probe --scene game_v19 --seeds 0..32 --days 120 --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30 --sample-every 120 --workers 4 --check-determinism --out docs/revitalize/probe/p15-0_base/i1_v19` (Pass 13 round 3's flags; `--out` is required — the default is `.harness/evidence/…`). Compare with `docs/revitalize/probe/p13-3_round3_v19_120day/summary.json` using `tools/revitalize/compare_summaries.py` (timing excluded) and record the result; a difference there means a portability difference between machines (record it; it does not stop the pass, because the base-vs-final comparison below is the one that decides).
   - **I2:** `worldsim_wm --scene game_v19 --seeds 12000..12002 --days 120 --every 60 --workers 2 --out …`; record every array file's sha256.
   - **I3:** the I1 probe for `--scene game_v16`, `--out docs/revitalize/probe/p15-0_base/i3_v16` (compare with `docs/revitalize/probe/p13-3_guard_v16_120day/summary.json` likewise).
   These are not timed runs; `--workers 4` is fine. Push.

---

# 6. PART 1 — Instruments (Rust)

## 6.1 The profiled tick (`sim-engine/src/lib.rs`)
- Add `pub struct TickProfile` (new type; every pub item documented — the crate has `#![warn(missing_docs)]` and clippy runs with `-D warnings`): per system **in execution order** (the order `register_system` sorts them into, the order `system_names()` returns): `name: String`, `priority: u32`, `interval: u64`, `calls: u64`, `total_ns: u64`, `max_ns: u64`; plus `ticks: u64`, `tick_us: Vec<u32>` (wall time of every whole tick in microseconds, saturating), `prune_ns: u64` (the `recent_deaths.retain` step), `last_tick_ns: Vec<u64>` (each system's time in the most recent tick, 0 when not due — what the benchmark needs to explain a slow tick).
- Refactor `SimEngine::tick` into a private `tick_inner(&mut self, profile: Option<&mut TickProfile>)` keeping today's statements in today's order; `pub fn tick(&mut self)` calls `tick_inner(None)`; add `pub fn tick_profiled(&mut self, profile: &mut TickProfile)` and `pub fn new_tick_profile(&self) -> TickProfile`. If a profile's system count differs from the engine's, `tick_profiled` rebuilds it from the engine before timing (documented; no panic). `std::time::Instant` is read only when a profile is given. No other change in the file except tests in its `#[cfg(test)]` module.
- `sim-engine` cannot build a seeded world (it depends only on hecs, serde, sim-core), so its unit tests cover the profile's bookkeeping on a bare `SimEngine::new` with two dummy systems (`calls` = due ticks, `tick_us.len() == ticks`, `last_tick_ns` zero for a system not due). The **equivalence test** lives in `sim-test` (`tests/` or `perf.rs`): a profiled and an unprofiled `game_v19` engine of seed 7 reach the same probe fingerprint after 3,000 ticks (`probe/mod.rs` declares `mod fingerprint;` privately: make it `pub(crate) mod fingerprint;` — a visibility change only; `fingerprint(engine: &SimEngine) -> String` hashes every agent's id, position, state, needs and hp, the tallies and the settlements).

## 6.2 The stress world (`sim-systems/src/worldgen/mod.rs`)
- An associated function in `impl WorldGenConfig`: `pub const fn stress_v19(seed: u64, agents: u32, side: u32) -> Self` = `Self { starting_agents: agents, width: side, height: side, ..WorldGenConfig::game_v19(seed) }` (a new constructor; no new field). Rules: `WorldRules::game_v19(seed)` unchanged. Doc comment: "benchmark-only; not a game scene; never used by the recorder or the probe's scenes".
- Unit tests: `WorldGenConfig::stress_v19(s, n, 64)` equals `WorldGenConfig::game_v19(s)` except `starting_agents`; `build_world_with(&WorldGenConfig::stress_v19(15100, 500, 64), WorldRules::game_v19(15100))` has 500 agents at tick 0; `WorldGenConfig::stress_v19(s, WorldGenConfig::game_v19(s).starting_agents, 64) == WorldGenConfig::game_v19(s)`. Worldgen can fail per seed (`camp.rs` `choose_camp` expect ≈ L80, the berry asserts ≈ L204–225, `terrain.rs` repair expects ≈ L119–123): a seed that fails at a size is dropped for that size and recorded (not a bug of this pass); a size with fewer than 6 surviving seeds is reported as such.

## 6.3 The benchmark binary (`sim-test/src/bin/worldsim_perf.rs`, logic in `sim-test/src/perf.rs`)
No new crate dependency (`rust/Cargo.lock` must not change). Subcommands (each writes one JSON file per run into `--out`, named `<mode>_<scene>_s<seed>_n<agents>_side<side>_<feed>.json`, a `runs.json` index, and a sidecar `<same name>.tick_us.bin` — the run's `tick_us` as little-endian u32 — which is **not** committed except for the one run the review recomputes):
- `natural --scene game_v19 --seeds A..B --days D --out DIR` — the scene exactly as the probe builds it.
- `stress --seeds A..B --agents 25,50,… --side 64 --days 2 --feed fed|raw --out DIR` — the stress world. **`fed`:** after each tick, if a home settlement exists (`sim_systems::runtime::intervention::home_settlement(&engine.resources)`) and either no gift has been given yet or the clock now reads 06:00 (`(engine.current_tick() + clock_offset_ticks) % 1440 == 360` after the tick — the probe's `minute_of_day` convention, `long_run.rs` ≈ L105; take `home` before calling `apply_intervention` to avoid a borrow conflict), apply `Intervention::GiftFood { settlement: home, amount: 3 × alive }` (`sim_engine::Intervention`) through `sim_systems::runtime::intervention::apply_intervention(&mut engine, …)` between ticks; record each gift's tick, requested and applied amounts. (No settlement exists before the first tick — worldgen creates none; `SettlementSystem` founds it — so the first gift lands right after the first tick that founds one, then every 06:00.) **`raw`:** no gifts.
- `map --seeds A..B --sides 64,128,256 --days 2 --out DIR` — `stress_v19(seed, game_v19(seed).starting_agents, side)`, fed.
- `overhead --seed S --days D --repeats 3 --out DIR` — the natural world run with `tick()` and with `tick_profiled()`, alternating, `repeats` times each; wall time of each run.
- `quick --out DIR` — `stress` with seed 15100, n = 500, side 64, 1 day, fed (the gate's smoke and the later passes' regression check).

Each run's JSON: mode, scene, seed, agents requested, side, feed, days, git SHA (env `WORLDSIM_GIT_SHA`, as the probe), build profile, machine facts, `contended` (env `PERF_CONTENDED=1`); per system {name, priority, interval, calls, total_ms, mean_us_per_call, max_us, share_of_tick_time}; **per day** per system total ms with that day's alive at 06:00, where day d is the ticks whose clock day `(tick + clock_offset_ticks) / 1440` equals d (the same day the 06:00 sample belongs to; for the natural slopes of 7.4); tick-time summary {mean_ms, p50, p90, p99, p999, max_ms, n_over_8ms, n_over_16_67ms}; **`slowest_ticks`**: the 20 slowest ticks after the first 60, kept by `perf.rs` as a running top-20 updated after every tick from `tick_us` and a copy of `last_tick_ns` — tick number, display day and clock, total ms and the 5 systems with the largest time in that tick; **`fingerprint`**: `probe::fingerprint(&engine)` at the end of the run (Part 5's stress proof uses it); alive and settlements at 06:00 of each day and their means; gifts (fed); for `stress` and `natural`, the snapshot timings of 6.5 at the end of the run. The first 60 ticks of every run (tick 0 runs every system) are excluded from the tick-time summary and reported separately.

## 6.4 One debug `#[func]` (`sim-bridge/src/ffi/world_node.rs`)
In `world_node.rs`, a pure builder `pub fn build_stress_engine(seed: u64, agents: u32) -> SimEngine` = `build_world_with(&WorldGenConfig::stress_v19(seed, agents, 64), WorldRules::game_v19(seed))`, and `#[func] fn reset_world_stress(&mut self, seed: i64, agents: i64) -> bool` that does what `reset_world` does (store `seed as u64` as the world seed, replace the engine, zero the accumulator, emit `world_reset` deferred) with `build_stress_engine(seed as u64, agents.clamp(1, 5000) as u32)`. Doc comment: "benchmark only — lets the art line measure frame times on a crowded world; never called by the game's UI". Nothing in `scripts/` calls it in this pass. A unit test that `build_stress_engine(15100, 500)` has 500 agents.

## 6.5 Snapshot timings (in `perf.rs`)
- **Inventory first** (read-only, both branches): `grep -rn` over `scripts/` (recursive — the HUD panels are in `scripts/ui/panels/`) on this branch and `git grep` on `origin/codex/art-production-2026-10` (the UI's real home), for every `WorldSimNode` getter. Classify each call site as **per frame** (reached from `_process` / `_physics_process` unconditionally), **per tick change** (reached when the simulation tick changed — at 30 TPS and 60 FPS about every other frame at 1×, every frame at 2× and above; count it as per frame), **conditional per frame** (e.g. `camera_controller.gd` only with auto-track on), or **event** (click, signal, reset); and mark calls **inside a loop** with what the loop runs over (e.g. the art branch's `building_renderer.gd` ≈ L133–136 and L345–357 call `get_settlement_snapshot()` once per building or sprite) — their per-frame count is the loop's length at n = 500 (from the stress run's building count). Write the table (getter, file:line, class, loop, calls per frame at n = 500) into `scaling.md`.
- **Time the whole body of each per-frame getter, Rust side:** every pure collector that the `#[func]`'s body calls, together — e.g. `get_agent_snapshot` = `collect_agent_snapshot` + `collect_agent_ages_days` + `collect_agent_colds`; `get_settlement_snapshot` = `collect_settlement_snapshot(&engine.world, &engine.resources.settlements)` + `collect_settlement_wood_targets`; also `get_world_info` (`collect_world_info`) and every other getter the inventory marks per frame — 100 repetitions each at the end of a run, median and p90 in µs. The `*_to_dict` conversions need a Godot runtime and are the art line's measurement (6.4 gives them the world).
- **B3** sums one call of each per-frame getter; the **×callers** figure multiplies each getter by its per-frame call count from the inventory (art branch's counts if they differ; both reported).

Commit, push. Run the debug and release `cargo test --workspace` before the commit.

---

# 7. PART 2 — Behaviour proof, and PART 3 — the sweeps

## 7.0 Behaviour proof (Part 2, before any timed run)
Build the final binaries at the pushed commit; rerun I1–I3 (section 5 item 3, into `docs/revitalize/probe/p15-0_final/`) and compare with the base: I1 and I3 summaries identical except `timing` (`compare_summaries.py`), I2 array sha256 identical. A difference stops the pass (revert the offending change, record it, report). Then the overhead check: `worldsim_perf overhead --seed 15000 --days 30 --repeats 3`; report profiled vs unprofiled wall time (median) and the overhead %; if it exceeds 5 %, say so in every table header. Push.

## 7.1 Natural sweep
`worldsim_perf natural --scene game_v19 --seeds 15000..15032 --days 120` (sequential; ≈ 12–20 min). Per world and pooled: tick summary, the top 10 systems by share, the slowest single ticks (tick number, display day and hour, which systems were due), mean alive and settlements.

## 7.2 Stress sweep
`worldsim_perf stress --seeds 15100..15108 --agents 25,50,100,200,500,1000,2000 --side 64 --days 2 --feed fed`, then the same with `--feed raw`. Smallest sizes first; if one run of a size exceeds 20 minutes, stop that size after the run, skip larger sizes and record it. **Repeatability:** seed 15100 at n = 500 fed three times; report the coefficient of variation of mean tick time.

## 7.3 Map sweep
`worldsim_perf map --seeds 15100..15108 --sides 64,128,256 --days 2` (fed, founders as `game_v19`). It separates costs that grow with tiles (influence grid, terrain scans) from costs that grow with people.

## 7.4 Scaling analysis (`tools/revitalize/perf_p15.py`, standard library only)
- For every system, two slopes, each by ordinary least squares with a 95 % interval (t-based, from the residuals): the **natural slope** — log(that system's ms per day) on log(alive at 06:00) over the days of the natural sweep (all 32 worlds; days with alive ≥ 15), the realistic growth over ≈ 20–150 people (it pools days, so it also absorbs time trends — more buildings and settlements, winter firewood; say so beside it); and the **stress slope** — log(mean µs per call) on log(mean alive) over the fed stress runs with mean alive ≥ 80, a density-confounded upper bound (section 3). Likewise against log(tiles) on the map sweep. A system whose stress slope is ≥ 0.5 above its natural slope is marked **crowding-driven**. Classes (applied to each slope): b < 1.2 **linear or better**, 1.2 ≤ b < 1.7 **superlinear**, b ≥ 1.7 **near-quadratic**. Systems that cost < 1 % of tick time at every size are listed but not classified.
- The same fit for the whole tick's mean time.
- **N_max:** the largest n whose median (over seeds) fed mean tick ≤ 4.0 ms; if every size passes, "≥ 2,000"; also the n interpolated on the fitted log-log line where the mean tick reaches 4.0 ms.
- **Static audit** (by reading the code of every system with share ≥ 5 % at n = 500 or class superlinear / near-quadratic): which loops make it grow (a query nested in a query, a scan of all agents or all tiles per agent, a sort per tick, allocation per tick), with file and line.
- Unit tests of the fitting on synthetic data (exact power laws give the slope to 1e-9; noisy ones within the interval).
Commit the per-run JSON files (small) under `docs/revitalize/probe/p15-1_natural/`, `p15-2_stress/`, `p15-3_map/`, and the tables `docs/revitalize/probe/p15-4_scaling/scaling.md`. Push after each sweep.

## 7.5 The P table (pre-registered)

| # | question | rule |
|---|---|---|
| P1 | the game as it is (`game_v19`, natural, 32 worlds) | PASS if every world meets B1 and B2 |
| P2 | 500 people (fed stress, n = 500, 8 worlds) | PASS if the median world meets B1 and the p99.9 of all 8 worlds' ticks pooled meets B2 (a 2-day run has ≈ 2,820 timed ticks, so one world's p99.9 is its ~3rd-slowest tick — too jitter-prone alone); per-world maxima and ticks > 16.67 ms reported |
| P3 | the UI's reads at 500 people | PASS if B3 holds (single calls); the ×callers figure reported |
| P4 | headroom | N_max (both forms) reported; "comfortable" if N_max ≥ 1,000 (twice the target) |
| P5 | growth | the list of superlinear and near-quadratic systems (natural and stress slopes, intervals, crowding-driven marks); "clean" if no system whose **natural** slope is superlinear or worse costs ≥ 5 % of tick time at n = 500 |

---

# 8. PART 4 — `docs/revitalize/PERF_BUDGET.md`
1. **Today:** the P table; mean tick, p99.9 and max per sweep; the top 10 systems at n = 50, 500, 2,000 with shares; the per-agent cost (µs per agent-tick at n = 500); the slow-tick causes; the snapshot timings and the per-frame caller inventory.
2. **The rule for later game passes** (copied into every future pass goal): a pass that adds or changes a system runs `worldsim_perf quick` on this Mac before and after, and reports (a) the change in mean tick at n = 500 fed — it fails if the mean tick rises by more than **15 %** or above 4.0 ms; (b) the new system's stress slope from a three-size stress run (n = 100, 500, 1,000; one seed), compared with this pass's stress slopes of comparable existing systems — near-quadratic fails unless it is no steeper than the crowding-driven slopes measured here, or the pass's goal accepts it explicitly.
3. **For the art line** (a short list they can act on): measure frame times on `reset_world_stress(seed, 500)`; read each per-frame getter once per frame and share the result (the 6.5 inventory lists today's duplicate callers); the Rust-side costs from 6.5.
4. **Next optimisation candidates**, ordered by share × slope, each with its file and line.

---

# 9. PART 5 — One optional fix
Allowed only if P2 fails or a system is near-quadratic — by its natural slope, or by its stress slope when not marked crowding-driven — with ≥ 10 % of tick time at n = 500. Then fix **that one system** only if: the fix changes ≤ 300 lines in that system's module; I1–I3 stay identical to the base (7.0's procedure) — so it must not change iteration order, random draws or floating-point operation order anywhere — **and**, because I1–I3 never reach crowded paths, the probe fingerprint of `stress_v19(15100, 500, 64)` (fed) after 2 days is identical between the pre-fix commit of this pass and the fix; debug and release `cargo test --workspace` are green; and the stress sweep at n = 100, 500, 1,000 (8 seeds, fed) is rerun before and after, with the gain reported. If any condition fails, revert and record the attempt as the first candidate in PERF_BUDGET.md section 4. If neither trigger holds, write "no fix needed" in DECISIONS and skip this Part.

---

# 10. PART 6 — Review, gate, report
1. Independent review (fresh `general-purpose` subagent, two rounds maximum): the refactor of `tick` (statement order unchanged; `Instant` only under a profile); I1–I3 base vs final; that no existing scene calls the stress code; no new field in an existing public struct; the overhead number; one run's JSON recomputed by hand from its `.tick_us.bin` sidecar (mean, p99.9; commit that one sidecar); one slope recomputed by hand; the P verdicts follow 7.5; the allowed paths. Findings and dispositions in `docs/revitalize/REVIEW_PASS15.md`.
2. Gate (4.4) → `docs/revitalize/verification/p15_gate/`; fast-forward `codex/revitalize-2026-10` if green; add a Pass 15 section to PR #510's description.
3. Remove the base worktree (`git worktree remove ../new-world-p15-base`).
4. `docs/revitalize/REPORT_PASS15.md` in Korean (section 12 format) and the same text in the final message.

---

# 11. 인게임 확인사항 (오너용)
이 패스는 게임 규칙을 바꾸지 않으므로 게임 화면에 보이는 변화는 없어야 한다.
- 게임을 실행해 기존과 똑같이 마을이 생기고 움직이는지(시드 같으면 같은 장면), 콘솔에 새 에러가 없는지.
- 숫자 키 1~4로 속도를 바꿔도 이전과 같은 느낌인지. 이 패스의 보고서가 말하는 "느려지는 지점"(N_max)과 체감이 맞는지는 다음 아트 패스가 `reset_world_stress`로 붐비는 세계를 띄워 프레임 시간을 잰 뒤 확인한다.

---

# 12. 구현 완료 보고 (REPORT_PASS15.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
무엇을 만들었는가? (계측 틱, 스트레스 세계, 벤치마크 바이너리, 디버그 FFI, 분석 스크립트, 예산 문서)

### 구현 방법
틱 계측 방식과 오버헤드, 동작 불변 증명(I1~I3), 스트레스 세계 구성(fed/raw), 기울기 적합, N_max 산출

### 기능 설명
지금 게임은 몇 명까지 프레임 안에 돌아가는가? 무엇이 인구보다 빨리 비싸지는가? 이후 게임 패스는 어떤 규칙을 지켜야 하는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
P1~P5 표, 스윕별 평균·p99.9·최대 틱 시간, 크기별 상위 시스템과 점유율, 시스템별 기울기와 구간, N_max(두 방식), 스냅샷 수집 시간(단일·호출자 수 배), 느린 틱의 원인, 반복 측정 변동계수, 계측 오버헤드

### 환경
맥 사양, rustc 버전, 각 배치의 부하 평균과 contended 여부

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md Pass 15 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 대기 구간

### 확인된 제한사항
(스트레스 세계는 한 캠프에 모두 모인 비관적 조건이라는 점 포함)

### 게이트 결과
(단계별 결과, 테스트 수치, 허용 경로 밖 변경 없음)

### 정직한 판단
지금 코드로 v1.0(수백 명, 마을 여럿)이 가능한가? 성격·가족을 얹기 전에 고쳐야 할 것이 있는가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #510 update, and the time spent per Part.
