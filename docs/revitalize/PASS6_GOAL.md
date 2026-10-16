# GOAL: WorldSim — Revitalize Pass 6: an autumn for wood, a winter that bites

Repository `hyunlord/new-world`. Continue the Pass 5 line: `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`, both at `682b442617d3f4bb9b151c29d324b17dfe82da93` (draft PR #510 into `lead/main`).
This run is executed by **Claude Code**, unattended. The art run (**Art Pass 4**, Codex, branch `codex/art-production-2026-10`) may be working on the same Mac; section 1 says how to share it.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4            # the Pass 4/5 clone, hooks already neutralized
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 682b4426
cp ~/Downloads/claude_goal_worldsim_pass6.md docs/revitalize/PASS6_GOAL.md
tmux new -s pass6
cd /Users/rexxa/work/new-world-pass4
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS6_GOAL.md>
```
The repository's Claude Code hooks are neutralized **in this clone only**: `.claude/settings.json` holds `{}` and is hidden from git with `--skip-worktree`. Never edit, stage or commit it and never clear that flag. No git hooks are installed. The local `.gitignore` edit and `.ignore` belong to unrelated graft tooling; leave them uncommitted, as Pass 5 did.

**This document is `docs/revitalize/PASS6_GOAL.md`.** Commit it unchanged in Part 0. At the start of every Part, after every compaction or pause, and whenever you are unsure of a number, re-read the relevant section from that file.

The owner drives the run with `/goal`: after each turn a small model reads **only the transcript** and decides whether the condition is met. The evidence it asks for (the section 15 report, the commit list, the `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
CLAUDE.md and the nested `rust/**/CLAUDE.md` files stay authoritative for architecture and conventions (simulation logic in Rust, determinism, no `unwrap()` in production, doc comments, tests per module). These do **not** apply to this run:
1. "Session Startup: run `install_hooks.sh`" — never run it.
2. "Working branch: lead/main", the pre-flight checkout and `git push origin lead/main` — never check out, commit to or push `lead/main` or `main`.
3. The harness pipeline, the `worldsim-harness` skill's pipeline, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS — not used; never create or edit anything under `.harness/audit/`, `.harness/baseline/`, `.harness/reviews/`. This document's gate (4.4) replaces them.
4. If any hook other than the owner's `/goal` blocks a command or a stop, do not get around it: record it in DECISIONS.md, push what you have, end with the report.
5. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`. Expected: no git hooks, a line starting with `S`, `{}`. Delete a stray git hook and record it; if the settings file is not `{}`, record it and continue.
6. Never use the repository's `.claude/agents/harness-*` agents; when this document asks for a subagent, use the built-in `general-purpose` type.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 6`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5` and background job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.**
6. **Time rule:** record start time and each Part's start and end. Past **10 hours** of working time, finish the current step, push, go to Part 7.
7. **Three strikes** per failing target, then keep the best configuration and record evidence and hypothesis.
8. **Drop order when time is short:** the winter story trace (Part 6) first, then urgent mornings (Part 3), then elders at the evening fire (Part 4). Never drop Parts 0, 1, 2, 5 or 7.
9. **No image generation.**
10. **Keep bulky output out of context:** probe runs, test logs and captures go to files; read summaries.
11. **Long commands run in the background** (one Bash call is cut off after 10 minutes): `nohup bash -c '<command>; echo $? > /tmp/p6_<name>.exit' > /tmp/p6_<name>.log 2>&1 &`, poll with short calls, check `pgrep -fl` before starting a job again. Keep gate logs in `docs/revitalize/verification/p6-<N>/`.

---

# ⚠️ 1. MACHINES

- **The Mac** clone `/Users/rexxa/work/new-world-pass4`. The art run may be active in `/Users/rexxa/work/new-world-art-prod`; never read, write or build there, never touch `~/github/new-world-wt/lead` or other checkouts, never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe `--workers 4`.
- Windowed Godot only through `tools/revitalize/with_window_lock.sh` in one command (the art run uses the same lock); from a background tmux session run that single command in the GUI session with `launchctl asuser $(id -u) …` as Pass 5 did. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Timing-sensitive runs (Y13) run back to back with nothing else heavy on the Mac: check `pgrep -fl 'Godot|cargo|worldsim_probe'` for other runs first; if the art run is busy, wait for it or record that it was running.
- **DGX Spark** (`ssh hyunlord@100.70.109.50`, `/home/hyunlord/work/new-world-revitalize`) is optional for long probe batches (detach to your pushed WIP commit, `CARGO_BUILD_JOBS=10`, `--workers 8`, copy summaries back; no ComfyUI 8188, no dev servers 3100/3200, no other checkouts). Never use it for Y13 timings.

---

# 2. GIT

- Every commit goes to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.4) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p6-N): <what changed>`. Stage explicit paths only, never `git add -A`.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`.
- **Must not change** (the art run owns them): `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, and the static tests that read `.gd`/`.tscn`/`.gdshader` files, except minimal Rust-source re-pins allowed in 4.3.

---

# 3. CONTEXT

## 3.1 Pass 5 result (verified by the planner from the committed evidence)
Gate green on code head `c77ee27c` (debug and release 1,912 each); final probes reproduce legacy, v8, v9 and v10 on every shared value. `game_v11` (`p5-5_round3_*`): X1–X4, X6–X8 and X10–X12 pass; X5 and X9 stopped after three rounds. Medians by season from `docs/revitalize/probe/p5-5_round3_120day/summary.md`:

| | spring | summer | autumn | winter |
|---|---:|---:|---:|---:|
| wood stock | 28 | 287 | 311 | 158 |
| wood target | 39 | 292.5 | 312 | 58.5 |
| wood gathered / day | 28 | 44 | 42 | 0 |
| wood trips / day | 14 | 22 | 21 | 0 |
| fire lit at 02:00 | 1.0 | 1.0 | 1.0 | 1.0 |
| cold member-hours / day | 0 | 0 | 0 | 0 |
| forage trips / day | 32.5 | 83 | 0 | 0 |
| food store | 202 | 340 | 528 | 289.5 |

Noon adult work share: 0.500 over spring–autumn, 0.285 over all seasons. Evening near the hearth: median 0.590, seed minimum 0.333. Food is already tight for the worst seed: at the day checkpoints around the end of the second, third and fourth winters its store is 0 (median about 200).

## 3.2 The planner's errors in the Pass 5 brief
- "The store is not food" was wrong: v10 already served 90 % of meals from the store (X4 v10 0.903); meals at the hearth moved meals to breakfast and dinner (98 %) rather than to the store.
- X5 asked for foraging in every season without a mechanism: purposeful foraging runs only while the store is under its target, and autumn and winter stores sit above it. X5 is retired; this pass measures wood work by season instead (Y5, Y6).
- `idle_share` was not defined; the run counted haulers as working and recomputed v10 the same way (0.774 → 0.512). That definition stays. Pass 4's definition (haulers idle) gives 0.829 → 0.738.

## 3.3 What is still wrong
- **Wood is never scarce.** Deadwood regrowth is one global accumulator: at `regen_permille` 500 every spot below capacity gains 1 unit every second 120-tick pulse, 6 a day per spot, 120 a day over 20 spots; the hearth burns at most 39 a day. The autumn target (`1.5 × 26 + 7 × 39 = 312`) covers the whole winter (7 × 39 = 273); the last autumn sample holds a median 335 (minimum 301). Every 02:00 sample from day 3 in all 32 seeds is lit and winter cold member-hours are 0 in every seed.
- **Winter days are empty.** No wood trips and no forage trips in winter.
- **Autumn is not a season of its own.** Summer already stocks the winter's wood (summer stock 287, gathered 44 a day; autumn 311, 42 a day).
- **The last winter night counts as spring.** `FireSystem` takes the season of the current display day, so after midnight of the last winter evening the burn rate is spring's and no one gets `Cold`; the probe books that 02:00 sample to spring.
- **Elders keep away from the evening fire.** In `movement.rs` an elder's idle tether is `AgingRule.elder_tether_radius` (4) at all hours and overrides the evening tether (3), while the evening measure counts Chebyshev ≤ 3 from home (`probe/long_run.rs`).

## 3.4 This pass
Autumn becomes the wood season: only autumn prepares for winter, and only for part of it. In winter most deadwood lies under snow and a few branches fall a day, one here and one there, so a few adults go out every winter day. When tonight's wood is short, adults go out at dawn. In some seeds the fire goes out on a late-winter night, and that night everyone grows hungry faster. Elders follow the evening tether. The village must still never starve.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Architecture and determinism (unchanged)
All simulation logic in Rust; FFI only adds keys; new collections `BTreeMap`/`BTreeSet`; no order-dependent hash iteration; no transcendental f64 in simulation crates; seeded `splitmix64` streams only (`MovementRng`); no new crates. No new enum variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType` or `DeathReason`. Time windows use integer minute-of-day (`clock_in_window`, wrap-around allowed). Day index and seasons as in Pass 4.

## 4.2 Versioning — v8, v9, v10 and v11 frozen, v12 new
- `game_v8()` … `game_v11()` and their `WorldGenConfig`s keep their behaviour exactly; every new field takes the value that reproduces them bit-for-bit.
- In Part 1 add `WorldRules::game_v12()` (= `game_v11()` until Parts 2–4 change it), `WorldGenConfig::game_v12(seed)` (= `game_v11(seed)`; no worldgen change this pass), `game_v12_scene(seed)`, and the probe scene `game_v12` wherever `game_v11` appears in the probe: `sim-test/src/probe/args.rs` (usage text and the scene check), `probe/runner.rs` (the scene builder and the two v9–v11 determinism checks) and `tools/revitalize/probe.sh --help`.
- The live game switches to v12 in both `world_node.rs` constructors.

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; `harness_population_legacy_scene_pinned`, `harness_population_game_world_survives_3_days` and all Pass 2–5 tests pass unchanged. Never change `sim-test/src/probe/fingerprint.rs`. Allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 6`: struct-literal field additions with legacy values; probe schema tests moving to schema 6; Rust-source static assertions that pin code this goal deliberately changes (the live world constructor, additive FFI keys), re-pinned and never weakened.

## 4.4 The gate (required before the green branch moves)
Copy `tools/revitalize/gate_pass5.sh` to `gate_pass6.sh` (same steps): debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep over `sim-core/src sim-systems/src sim-engine/src` (0 hits), `gdscript_strict_check.sh` (read-only checker), `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, `python3 -m unittest discover -s tools/revitalize -p 'test_*.py'`.

## 4.5 v11 guard after every behaviour Part
After Parts 2, 3 and 4 each: re-run `game_v11` 64 × 10 and 32 × 120 (the arguments of Part 0, step 2) and compare with Part 1's v11 runs; every shared value must be identical. A 1-day fingerprint cannot see summer, autumn, winter or elders, so this probe comparison is the guard. A difference is fixed before the next Part.

---

# 5. PART 0 — Baseline and frozen references

1. Record `PASS6_BASE_SHA` (`682b4426`), start time and `uname -a`. Commit `PASS6_GOAL.md`.
2. Release probes with exactly the flags METRICS.md records for Pass 5's final rounds (`## Pass 5 Part 5 — tuning`: `--check-determinism` and the clocks `02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30` on **both** runs): `game_v11` 64 × 10 against `p5-5_round3_10day` and 32 × 120 against `p5-5_round3_120day`; `game_v10` 64 × 10 against `p5-7_final_v10_10day`. Every shared key identical, timing excepted; a difference stops the run until explained.
3. Record `sim_test::probe::fingerprint` of `game_v11_scene(seed)` for seeds 0..4 after 1,440 ticks; they become the constants of the Pass 6 `*_v11_unchanged` tests (keep the earlier ones). Add one test that compares `WorldRules::game_v11()` field by field with a literal of its Pass 5 values.

---

# 6. PART 1 — Instruments, FFI and the v12 skeleton

## 6.1 v12
`game_v12` rules, worldgen, scene and probe wiring as in 4.2, no behaviour beyond v11 yet.

## 6.2 Tallies (display-only)
`urgent_wood_trips` (wood trips started in urgent mode, 8.1); `winter_wood_lost` (winter regrowth units that found no spot below the cap, 7.3).

## 6.3 Probe schema 6
Every v5 key keeps its meaning; never null. New keys:
- `wood_gatherers_now` — members in `Seeking{GatherWood}` or `Consuming{GatherWood}` (the wood counterpart of `foraging_now`);
- `urgent_wood` — 1 when the settlement is in urgent mode (8.1). It is a state flag, independent of the clock, so it may read 1 at a night sample;
- `urgent_wood_trips_total`, `winter_wood_lost_total`;
- at clock samples `idle_elders` and `idle_elders_near_hearth` (idle elders, and those within Chebyshev 3 of home; diagnostic for Part 4).

## 6.4 FFI (additive, all scenes)
- `get_agent_snapshot`: `colds: PackedByteArray` (1 when the agent has the `Cold` marker).
- `get_settlement_snapshot`: `wood_targets: PackedInt32Array` (floor of the settlement's wood target; 0 without the firewood rule).
Pure-Rust collectors with unit tests, as Pass 5 did for its keys.

## 6.5 Evaluator and the v11 baseline
`tools/revitalize/evaluate_pass6.py` (+ tests, stdlib only, reusing `evaluate_pass4`/`evaluate_pass5` helpers) computes every Y target of section 11 from schema-6 raw JSONL and writes `y_targets.json`, `y_targets.md`, plus two tables in `y_targets.md`: **Work by season** (adult work share and idle share at 12:00 and 15:00 per season; per-seed medians first) and **Winters** (per seed-winter: wood on hand at the winter's first day checkpoint, nights out at 02:00, the first such night, trip-days, wood trips, urgent trips, cold member-hours, food store at the winter's last periodic sample; then medians and extremes). Night booking for every night measure: a 02:00 sample belongs to the season of the previous display day (the evening the night began).
Re-run `game_v11` 64 × 10 and 32 × 120 (Part 0, step 2 arguments) with the Part 1 binary: every shared value must equal Part 0's runs (only `schema_version` and the new keys differ). Evaluate them into `docs/revitalize/probe/p6-1_v11_targets/` so every Y has a v11 value. Commit, push WIP.

---

# 7. PART 2 — Autumn for wood, a lean winter

## 7.1 Rule fields (`FirewoodRule`, `sim-core/src/rules.rs`)
```rust
pub struct FirewoodRule {
    // existing fields unchanged
    /// Days of winter burn added while preparing for winter. None = `days_per_season` (Pass 5).
    pub winter_prep_days: Option<f64>,          // v11 None, v12 Some(3.0)
    /// Summer also prepares for winter (Pass 5). v12: autumn only.
    pub winter_prep_in_summer: bool,            // v11 true, v12 false
    /// Days of current burn kept in store during winter. None = `base_days`.
    pub winter_base_days: Option<f64>,          // v11 None, v12 Some(4.0)
    /// Winter deadwood regrowth, units per day over all spots, one spot at a time (7.3). None = Pass 5 regrowth.
    pub winter_regen_units_per_day: Option<u32>, // v11 None, v12 Some(6)
    /// Most wood a spot holds in winter; the rest lies under snow. None = no cap.
    pub winter_deadwood_cap: Option<u8>,        // v11 None, v12 Some(1)
    /// Burn and cold use the season of the evening the night began (3.3). Pass 5: false.
    pub night_season_from_evening: bool,        // v11 false, v12 true
    /// Urgent mode start (8.1). None = off.
    pub urgent_start_minute: Option<u32>,       // v11 None, v12 Some(420)
}
```
Legacy, v8, v9 and v10 keep `firewood: None`.

## 7.2 Wood target (`settlement/firewood.rs::wood_target`)
`target = base(s) × burn_per_day(s) + prep(s)`. `base(s)` is `winter_base_days` in winter when set, else `base_days`. `prep(s) = prep_days × burn_per_day(winter)` when seasons are on, `winter_prep` is true and the season is autumn, or summer with `winter_prep_in_summer`; `prep_days = winter_prep_days.unwrap_or(days_per_season as f64)`. v11 values give exactly Pass 5's formula. v12 start values: spring 39, summer 19.5, autumn 1.5 × 26 + 3 × 39 = 156, winter 4.0 × 39 = 156 (after the first night or two of winter, gathering runs whenever wood is on the ground).

## 7.3 Deadwood in winter (`ResourceRegenSystem`)
With seasons on and the season winter, each rule below applies only when its field is set:
- **Cap:** every pulse first lowers each spot above `winter_deadwood_cap` to the cap (those units are gone); a spot lowered to 0 is removed from `wood_tiles` (its `wood_source_max` registration stays; no zero-valued tile is ever stored).
- **Spread regrowth** instead of the Pass 5 per-spot rule: a separate accumulator `SimResources::wood_winter_acc` gains `winter_regen_units_per_day × 1000 / 12` per pulse (12 pulses a day); each whole unit goes to one spot — the next registered spot after `SimResources::wood_winter_cursor` in key order, wrapping, whose amount is below `min(capacity, cap)`; the cursor moves to it. A unit that finds no such spot is lost (`winter_wood_lost += 1`). With 6 a day, one branch appears somewhere every 4 hours.
- In spring the Pass 5 regrowth returns, with the Pass 5 accumulator untouched by winter.

## 7.4 Night season (`settlement/fire.rs`)
Under `night_season_from_evening`, `FireSystem` evaluates the season at `current_tick − 1440` when the minute of day is before `burn_end_minute`, and uses that season for `burn_per_hour` and for the cold check; otherwise the current season, as in Pass 5. The last winter night is then a winter night to its end.

## 7.5 Expected shape (planner's estimate; Part 5 measures it)
Summer gathers only for summer nights (target 19.5); autumn gathers to 156 (autumn bursts can overshoot by up to 2 per gatherer in flight, so 156–196). A winter begins with that in hand plus at most 20 on the ground and gains about 42 more over seven days, against 7 × 39 = 273 burned: the planner expects about one dark late-winter night in many seeds and none in others. If that is too harsh or too mild, Part 5 moves `winter_prep_days` first (each 1.0 is 39 units, one night).

## 7.6 Tests — new file `harness_revitalize_pass6_winter.rs`
Read every expected number from `WorldRules::game_v12()` (Part 5 may change it), plus one table test of the formula with explicit literals: `harness_wood_target_by_season`, `harness_autumn_only_prep`, `harness_winter_cap_buries_deadwood` (a full spot drops to the cap at the first winter pulse; a cap-0 spot is removed and stays registered), `harness_winter_spread_regrowth` (units arrive one spot at a time in key order, wrap, skip full spots, count lost units), `harness_spring_restores_deadwood`, `harness_last_winter_night_is_winter` (02:00 after the last winter evening burns the winter rate and applies `Cold` when out), `harness_firewood_v11_unchanged`. Commit, push WIP; run 4.5; fast-forward green if the gate is green.

---

# 8. PART 3 — Urgent mornings and a colder night

## 8.1 Urgent mode (`agent_decision.rs`, Firewood arm)
Once per decision tick and settlement: `urgent = urgent_start_minute.is_some() && wood_in_hand < burn_per_day(night season)` (tonight's burn is not on hand). For members of an urgent settlement the firewood window is `[urgent_start_minute, gather_end_minute)` and the per-spot claim cap is `2 × max_gatherers_per_spot`, also when an empty-tile gatherer is re-targeted; otherwise everything is as in Pass 5. A trip started in urgent mode also adds `urgent_wood_trips += 1`. No change to `hauls_home` is needed: the forage window (06:00–18:00) already keeps a partly loaded gatherer out at 07:00.

## 8.2 Colder nights (a knob, not a new rule)
`cold_hunger_permille` stays 1500 at the start (the worst seed's winter store is already near 0, 3.1); Part 5 may raise it within 1500–2000 only if Y2, Y3 and Y8 hold. `FireSystem` and the `Cold` marker are unchanged.

## 8.3 Tests — new file `harness_revitalize_pass6_urgent.rs`
Values from `game_v12()`: `harness_urgent_window_opens_at_dawn` (wood in hand below tonight's burn: an adult starts a wood trip at the urgent start; above it: not before 10:00), `harness_urgent_two_per_spot` (claim cap doubled, also on re-targeting), `harness_urgent_ends_when_wood_is_in_hand` (the flag clears once wood in hand reaches tonight's burn), `harness_urgent_v11_unchanged`. Commit, push WIP; run 4.5.

---

# 9. PART 4 — Elders at the evening fire

`AgingRule.elder_evening_tether: bool` (v10 and v11 false, v12 true). In `movement.rs`, when it is true and the evening-gather window is open, an idle elder's tether radius is `min(elder_tether_radius, evening tether radius)`; outside the window elders keep `elder_tether_radius`. Tests: `harness_elder_evening_tether_v12` (an idle elder at 19:00 stays within Chebyshev 3 of its formation tile over 120 ticks; at 12:00 it may stand at 4), `harness_elder_tether_v11_unchanged`. Commit, push WIP; run 4.5.

---

# 10. PART 5 — Measure and tune (three rounds maximum)

Runs (release): `game_v12` 64 × 10 and 32 × 120 with the Part 0, step 2 flags → `docs/revitalize/probe/p6-5_round<N>_{10day,120day}/`; evaluate with `evaluate_pass6.py` (winter burn per day and cold window from `game_v12()`). Screening variants on throwaway, uncommitted runs is allowed (Pass 5 did it); only three recorded rounds count, and every screened variant is listed in METRICS.md with its numbers.

---

# 11. TARGETS (game_v12; table order = priority)

Pooling: per-day quantities (wood gathered, trips) are medians over seed-days of the season; shares (work, idle, lit, cold) take each seed's median or share first, then quantiles across seeds; nights are booked as in 6.5.

| # | target |
|---|---|
| Y1 | determinism 64/64 (10 d) and 32/32 (120 d); legacy pin; v8, v9, v10 and v11 reproduce Pass 5's final summaries on every shared key |
| Y2 | starvation and dehydration: zero in every seed at 10 d; 120 d totals median 0 and max ≤ 2 each |
| Y3 | village health as Pass 5's X3: one settlement every day; a birth by day 4 and alive ≥ 30 on day 10 in every seed; longest stagnant run ≤ 4 days; an old-age death by day 30 and births ≥ deaths − 3 over days 30–120 in every seed; alive in [32, 44] from day 30; child and elder shares as W6 |
| Y4 | meals as X4: from day 3, store meals ≥ 0.60 of all meals (median over seed-days) and ≥ 0.40 in every seed; winter median ≥ 0.40 |
| Y5 | autumn is the wood season: autumn wood gathered per day median ≥ 1.5 × summer's; autumn adult work share at 12:00 ≥ v11's autumn value − 0.05 (Part 1 baseline) |
| Y6 | winter has work: wood trips on ≥ 4 of the 7 days of a winter (median over seed-winters); wood trips per winter median ≥ 20 |
| Y7 | the fire can go out in winter: per seed, the share of winter nights lit at 02:00 — median across seeds in [0.70, 0.97] and ≥ 0.50 in every seed; outside winter, 02:00 lit share median ≥ 0.95 and ≥ 0.85 in every seed |
| Y8 | the cold is survivable: winter cold members / members in the cold window median ≤ 0.25 and ≤ 0.50 in every seed; food store at each winter's last periodic sample median ≥ 0.8 × v11's (Part 1 baseline) |
| Y9 | daytime as X8 (spring–autumn): adult work share median ≥ 0.35 at 12:00 and ≥ 0.25 at 15:00; idle share at 12:00 median ≤ 0.60 |
| Y10 | rhythm as X9: night-in-shelter median ≥ 0.80; evening-near-hearth median ≥ 0.55 and ≥ 0.35 in every seed |
| Y11 | no regressions as X10: T6a ≤ 1 %; social completed per agent-day ≥ 0.40 and > 0 in every seed; day-10 home distance median ≤ 8.1 and every seed ≤ 13 |
| Y12 | preemption as X11: no agent in `Seeking{GatherFood\|GatherWood}` with hunger or thirst ≥ 90 at any 120-tick sample |
| Y13 | ms per agent-tick ≤ 1.2 × v11, both measured back to back on the same host in this run (section 1) |

Y7's lower bound and Y8's cold bound together allow about one dark night per winter on average; aim there.

Knobs (only these, all in `game_v12()`): `winter_prep_days` 1.0–5.0; `winter_deadwood_cap` 0–3; `winter_regen_units_per_day` 0–12; `winter_base_days` 2.0–5.0; `cold_hunger_permille` 1500–2000; `urgent_start_minute` 360–540. Every Pass 5 value stays as in `game_v11()`. Record every round with all numbers in METRICS.md; keep the best configuration in `game_v12()`. Commit, push WIP; fast-forward green if the gate is green.

---

# 12. PART 6 — A winter in detail

`tools/revitalize/winter_trace.py` (+ test) reads the final `game_v12` 120-day raw JSONL and picks two seeds: the one with the most winter nights out and the one with the fewest (ties: lower seed). For each, the first winter (display days 22–28, nights booked as in 6.5) at the 120-tick samples: wood in hand, fire lit, cold members, `wood_gatherers_now`, `urgent_wood`, food store. Write `docs/revitalize/winter_trace.md` with both tables and five plain sentences per seed telling what happened (for example: when the wood ran out, how many went out the next morning, how much food the cold night cost). The renderer on this branch draws no fire state (Art Pass 4 adds it); no capture this pass.

---

# 13. PART 7 — Independent review, final verification, report

1. **Independent review:** a built-in `general-purpose` subagent with fresh context gets `git diff 682b4426..HEAD -- rust/ tools/revitalize/`, sections 4, 7, 8, 9 and 11 and the final Y table; it reports concrete defects (determinism, v8–v11 invariance, formula errors, untested branches, starvation or freeze paths, an urgent mode that never ends, night booking) with file:line evidence. Fix confirmed defects (two rounds maximum); record every finding and disposition in DECISIONS.md. A configuration change after the review goes through the gate and the final probes again and is named in the report.
2. Full gate on the final WIP head; totals versus Pass 5 (1,912 debug and 1,912 release).
3. Final probes: legacy, v8, v9, v10 and v11 reproduce Pass 5; `game_v12` 64 × 10 and 32 × 120 with census and clock samples; v11 and v12 timings back to back for Y13.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS6.md` in Korean (section 15 format), answering each item of section 14 with its evidence level. Append `## Pass 6` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 6 section (`gh pr edit 510 --body-file …`; skip and say so if `gh` is not authenticated). No new PR.

## 13.1 Merge touch-points for the art run (list them in the report)
`colds` per agent (huddled or shivering villagers on a cold night); `wood_targets` per settlement (a HUD "wood / target"); winter nights with `fire_lit == 0` and `wood_stocks == 0` (cold ring, no glow; Art Pass 4 already maps this); deadwood spots at the winter cap (mostly bare, single branches appearing); adults leaving for wood at the urgent start; elders inside the evening camp; the live game now builds `game_v12`.

## 13.2 Out of scope
Rendering and HUD; food scarcity (bush regrowth, the food stock target, the order of the Forage and Firewood arms); birth-rate variety (the cooldown-bound growth); new death causes (the cold never kills directly); weather beyond the season rules; new settlements; trade; map size; merging into `lead/main`; the art-owned paths.

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 여름에는 땔감을 그날 밤 쓸 만큼만 줍고, 가을에 어른들이 숲으로 땔감을 주우러 많이 나간다.
2. 겨울이 되면 숲의 마른 나무 더미가 거의 사라지고, 몇 시간마다 한 곳에 가지가 하나씩 떨어진다.
3. 겨울 낮에도 몇몇 어른이 새로 떨어진 가지를 주우러 나간다.
4. 대부분의 시드에서 늦겨울에 장작이 떨어져 불이 꺼지는 밤이 하루쯤 있고, 그 밤에는 주민들이 더 빨리 배고파진다. 겨울 마지막 밤도 겨울 밤으로 친다.
5. 오늘 밤 땔 장작이 모자라면 다음 날 아침 7시부터 어른들이 땔감을 주우러 나간다.
6. 그래도 어떤 시드에서도 굶주림·탈수 사망이 없다.
7. 저녁에는 노인도 다른 주민처럼 화덕 3칸 안에 모인다.
8. 봄이 오면 마른 나무가 다시 자라고 불도 다시 매일 밤 켜진다.
9. 콘솔에 SCRIPT ERROR가 없다.

---

# 15. 구현 완료 보고 (REPORT_PASS6.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
가을 대비와 겨울 목표, 겨울 마른 나무(눈 상한과 한 곳씩 떨어지는 재생), 밤의 계절, 급한 아침, 추운 밤, 노인 저녁 반경, v8~v11 고정 방식

### 기능 설명
플레이어 관점에서 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
Y1~Y13 표: v11(Pass 5 그대로) / v12, 중앙값·p10·p90·최소·최대
계절별 땔감(비축·목표·주운 양·이동 횟수·급한 이동)·불·추위·식량, 계절별 정오·15시 일하는 어른 비율, Winters 표, 조정 라운드별 수치와 걸러 본 변형

### 겨울 이야기
winter_trace.md의 두 시드 요약

### 독립 리뷰
지적 사항과 처리 결과

### 아트 작업과 맞물리는 지점
13.1 목록

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 컨텍스트 압축 뒤 목표 파일을 다시 읽은 시점, 백그라운드 작업

### 확인된 제한사항

### Harness 결과
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트)

### 정직한 판단
겨울이 실제로 버티는 계절이 되었는가? 가을은 준비하는 계절로 보이는가? 아직 시뮬레이션처럼 보이지 않는 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, and the time spent per Part.
