# GOAL: WorldSim — Revitalize Pass 8: winters a model has to read — nights that vary, a hidden severity, held-out seeds, dataset v1 and WM-Bench v1

Repository `hyunlord/new-world`. Continue the Pass 7 line: `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`, both at `ce7acb25d816e6e6fdcc2c8666b90119c42b9121` (code head `6df3a7e0`; draft PR #510 into `lead/main`).
This run is executed by **Claude Code**, unattended, on the Mac. Two other runs may work at the same time: **WM-1** (Claude Code, clone `/Users/rexxa/work/new-world-wm1`, training on the DGX) and possibly an art run (Codex, `/Users/rexxa/work/new-world-art-prod`). Section 1 says how to share.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4            # the Pass 4–7 clone, hooks already neutralized
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # ce7acb25
cp ~/Downloads/claude_goal_worldsim_pass8.md docs/revitalize/PASS8_GOAL.md
tmux new -s pass8
cd /Users/rexxa/work/new-world-pass4
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS8_GOAL.md>
```
The repository's Claude Code hooks are neutralized **in this clone only**: `.claude/settings.json` holds `{}` and is hidden from git with `--skip-worktree`. Never edit, stage or commit it and never clear that flag. No git hooks are installed. The local `.gitignore` edit and `.ignore` belong to unrelated graft tooling; leave them uncommitted, as Passes 5–7 did.

**This document is `docs/revitalize/PASS8_GOAL.md`.** Commit it unchanged in Part 0. At the start of every Part, after every compaction or pause, and whenever you are unsure of a number, re-read the relevant section from that file.

The owner drives the run with `/goal`: after each turn a small model reads **only the transcript** and decides whether the condition is met. The evidence it asks for (the section 14 report, the commit list, the `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
CLAUDE.md and the nested `rust/**/CLAUDE.md` files stay authoritative for architecture and conventions (simulation logic in Rust, determinism, no `unwrap()` in production, doc comments, tests per module). These do **not** apply to this run:
1. "Session Startup: run `install_hooks.sh`" — never run it.
2. "Working branch: lead/main", the pre-flight checkout and `git push origin lead/main` — never check out, commit to or push `lead/main` or `main`.
3. The harness pipeline, the `worldsim-harness` skill's pipeline, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS — not used; never create or edit anything under `.harness/audit/`, `.harness/baseline/`, `.harness/reviews/`. This document's gate (4.4) replaces them.
4. If any hook other than the owner's `/goal` blocks a command or a stop, do not get around it: record it in DECISIONS.md, push what you have, end with the report.
5. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`. Expected: no git hooks, a line starting with `S`, `{}`. Delete a stray git hook and record it; if the settings file is not `{}`, record it and continue.
6. Never use the repository's `.claude/agents/harness-*` agents; when this document asks for a subagent, use the built-in `general-purpose` type.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 8`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5` and background job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.**
6. **Time rule:** record start time and each Part's start and end. Past **10 hours** of working time, finish the current step, push, go to Part 7 (Part 4 must have at least one recorded round and Part 5 must have run before that). Pass 7 used 1 h 50 min of its 10 h: there is room to be thorough; do not hurry the review or the held-out verification.
7. **Three strikes** per failing target, then keep the best configuration and record evidence and hypothesis.
8. **Drop order when time is short:** T5 and the OOD splits of Part 6 → T3b → the agent, resource, terrain and building tables of dataset v1 (keep the global and settlement tables) → the C_cf `windfall` variant. Never drop Parts 0–5 or 7, nor the global and settlement export of Part 6.
9. **No image generation, no neural-network training** (WM-1 does that, separately).
10. **Keep bulky output out of context:** probe runs, exports, test logs go to files; read summaries.
11. **Long commands run in the background** (one Bash call is cut off after 10 minutes): `nohup bash -c '<command>; echo $? > /tmp/p8_<name>.exit' > /tmp/p8_<name>.log 2>&1 &`, poll with short calls, check `pgrep -fl` before starting a job again. Keep gate logs in `docs/revitalize/verification/p8-<N>/`.

---

# ⚠️ 1. MACHINES

- **The Mac** clone `/Users/rexxa/work/new-world-pass4`. Never read, write or build in `/Users/rexxa/work/new-world-wm1` (WM-1), `/Users/rexxa/work/new-world-art-prod` (art), `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe and export `--workers 4`.
- Datasets live outside git under `WM_DATA_ROOT=/Users/rexxa/work/worldsim-wm-data` (v0 is there; v1 goes to `v1/`). Check free disk before Part 6 (v1 is about 1.5 GB; stop and record if less than 15 GiB is free).
- Python: `tools/worldmodel` and `tools/revitalize` use the system `python3` with numpy (Pass 7 confirmed numpy 2.4.3). No scipy, torch or pandas.
- Windowed Godot only through `tools/revitalize/with_window_lock.sh` in one command; this pass needs only headless Godot. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Timing-sensitive runs (X8) run back to back with nothing else heavy on the Mac: check `pgrep -fl 'Godot|cargo|worldsim_probe|worldsim_wm|python3'` first; if another run is busy, wait up to 20 minutes, then measure and record what was running. WM-1 keeps its heavy work on the DGX.
- **DGX Spark:** WM-1 owns its GPU and CPU during this pass. Use it only for the dataset copy of 12.4 (`rsync` and `sha256sum`), nothing else.

---

# 2. GIT

- Every commit goes to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.4) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p8-N): <what changed>`. Stage explicit paths only, never `git add -A`.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`, `docs/worldmodel/**` and `tools/worldmodel/**`, **except** `tools/worldmodel/wm1/**` and `docs/worldmodel/wm1/**` (WM-1 owns them, on its own branch).
- **Must not change** (the art run owns them): `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, and the static tests that read `.gd`/`.tscn`/`.gdshader` files, except minimal Rust-source re-pins allowed in 4.3.
- Dataset v0 stays as it is: never touch `WM_DATA_ROOT/v0`, `docs/worldmodel/DATASET_v0.md`, `dataset_v0_manifest.json`, `BENCH_v0.md` or the committed v0 sample (WM-1 trains on v0). v0 remains reproducible from `ce7acb25`'s recorder; the Part 1+ recorder writes schema 2 and is not expected to reproduce v0's bytes. `bench_v0.py` keeps producing v0's numbers on v0 (comment fixes only).

---

# 3. CONTEXT

## 3.1 Pass 7 result (verified by the planner)
Gate green; v8–v12 reproduce Pass 6. `game_v13` = v12 + a severity drawn per winter (mild / normal / harsh, weights 520 / 280 / 200; burn per hour 2 / 3 / 4; winter deadwood fall per day 8 / 6 / 4) + claims bounded by what a spot holds. Seven interventions with exact replay; the `worldsim_wm` recorder (the planner cross-checked 7 global columns against the probe on 391 ticks: 0 mismatches; a rain counterfactual's digest equal before the intervention and different after); dataset v0 (400 episodes, 1,368 MiB, copied to the DGX); WM-Bench v0. The planner rebuilt `ce7acb25` on Linux and reproduced the v13 32 × 120 summary on all 56,448 values.

## 3.2 What the planner's review found
1. **The winter became a three-valued calendar.** On held-out seeds 32–95 (256 seed-winters), severity alone decides the outcome: mild → 0 dark nights in 152 of 152; normal → 1 dark night, on the seventh, in 62 of 65; harsh → 2 (first on the sixth) or 3 (first on the fifth). The share of the dark-night count's variance explained by severity alone (R², 7.5) is 0.94 on both the tuning and the held-out seeds. And `winter_severity` is an observable column of the recorded world and an input of the baselines, so T3 is solved at AUC 1.000 by arithmetic.
2. **"Every seed" clauses were judged on the tuning seeds.** Round 2 raised the mild weight to 520 to rescue seed 22 (whose draws 752 / 423 / 785 / 508 became two normal and two mild winters). On held-out seeds 32–95 the per-seed health clauses fail for every ruleset, v12 included:

   | held-out seeds 32–95 (120 days) | v12 | v13 |
   |---|---|---|
   | Y3 replacement (births ≥ deaths − 3, days 30–120): failing seeds | 1 (seed 80: −4) | 2 (68: −4, 80: −4) |
   | Y10 evening near hearth ≥ 0.35: failing seeds | 2 (55: 0.267, 75: 0.344) | 2 (55: 0.281, 76: 0.344) |
   | Y3 longest stagnation (≤ 4) | max 4 | max 2 |
   | Z3 no-dark-night share (≤ 0.60) | 0.023 | 0.594 |

   The claims fix does not harm the villages; the clauses are fragile per seed, and judging them on the seeds used for tuning hides that. This was the planner's error in the Pass 7 brief.
3. **Bench v0 defects:** T4's "mean MAE over the four targets" mixes units (cold member-hours dominates); T2 starts only at 06:00, where `cold_members` and `adults_working` are always 0; no intervention changes deaths; windfall applies 0 in most winters because the snow cap is already met (28 of 512 B_act interventions, 6 of 56 Z6 cases); `evaluate_pass7.py` defaults to the start weights 300/400/300.

## 3.3 Measured shape of the fix (planner's experiment, seeds 0–31 × 120 days)
The planner patched `game_v13` with a night-by-night burn change of −d / 0 / +d per hour (weights 250 / 500 / 250, drawn per evening) and a day-by-day winter fall change uniform in [−f, +f], both pure hashes, and measured:

| variant | severity weights | step | fall spread | R² | no dark night | ≥ 2 dark | normal winters: 0 / 1 / 2 / 3 dark | first dark night 4 / 5 / 6 / 7 | per-seed Y3 failures |
|---|---|---|---|---|---|---|---|---|---|
| v13 (Pass 7) | 520/280/200 | 0 | 0 | 0.94 | 0.547 | 0.172 | 0 / 36 / 1 / 0 | 0 / 9 / 13 / 36 | none |
| A | 520/280/200 | 1 | 2 | 0.83 | 0.648 | 0.234 | 13 / 15 / 9 / 0 | 1 / 15 / 14 / 15 | seed 22 (stagnation 6) |
| B | 520/280/200 | 2 | 2 | 0.66 | 0.617 | 0.250 | 15 / 13 / 6 / 3 | 3 / 11 / 20 / 15 | seed 22 (stagnation 5, replacement −4) |
| C | 350/400/250 | 2 | 2 | 0.66 | 0.523 | 0.328 | 19 / 16 / 13 / 2 | 3 / 13 / 27 / 18 | seed 22 (stagnation 6, replacement −5) |

In C, mild winters had 0 / 1 / 2 dark nights in 48 / 3 / 1 cases and harsh winters 2 / 3 / 4 in 12 / 11 / 3; no starvation or dehydration death in any variant; the lowest food store at a winter's end was 31 (C), 5 (B), 48 (v13). Step 1 is not enough (R² 0.83: the between-severity gaps stay large against a ±13-unit night). Seed 22 is a weak village in every variant (its food store ends winters low even when mild); it passes v13 only because the 520 weight made its winters milder. Under 6.4's comparison mode (v12 has no failing tuning seed, so one is allowed) C passes X2 on the tuning seeds and every X3 clause. These are throwaway measurements with a crude patch (env-var knobs, not the 7.2 code); Part 4 measures the real implementation.

## 3.4 This pass
`game_v14` = `game_v13` + a burn that changes night by night + a deadwood fall that changes day by day + windfall that lands on the snow. The severity becomes a hidden variable of the recorded world. Health is judged against v12 on held-out seeds. Dataset v1 and WM-Bench v1 fix the v0 defects. v8–v13 frozen.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Architecture and determinism (unchanged)
All simulation logic in Rust; FFI only adds keys; new collections `BTreeMap`/`BTreeSet`; no order-dependent hash iteration; no transcendental f64 in simulation crates; seeded `splitmix64` streams only; no new crates. The night and day draws of section 7 are pure hashes of (world seed, display day index) like the severity draw and advance no stream. No new enum variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType` or `DeathReason`. Day index and seasons as in Pass 4.

## 4.2 Versioning — v8 to v13 frozen, v14 new
- `game_v8()` … `game_v13()` and their `WorldGenConfig`s keep their behaviour exactly; every new field takes the value that reproduces them bit for bit.
- In Part 1 add `WorldRules::game_v14()` (= `game_v13()` until Part 2), `WorldGenConfig::game_v14(seed)`, `game_v14_scene(seed)`, the probe scene `game_v14` wherever `game_v13` appears (`probe/args.rs` usage and check, `probe/runner.rs` builder and determinism lists, `tools/revitalize/probe.sh --help`), and the recorder's scene list.
- The live game switches to v14 in both `world_node.rs` constructors.
- Interventions keep their v13 meaning in every scene except where a v14 rule field says otherwise (7.3).

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; `harness_population_legacy_scene_pinned`, `harness_population_game_world_survives_3_days` and all Pass 2–7 tests pass unchanged. Never change `sim-test/src/probe/fingerprint.rs` or `probe/long_run.rs::trace`. Allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 8`: struct-literal field additions with legacy values; probe schema tests moving to schema 8; recorder schema tests moving to schema 2 (v0's committed sample stays schema 1 and still loads); Rust-source static assertions that pin code this goal deliberately changes, re-pinned and never weakened.

## 4.4 The gate (required before the green branch moves)
Copy `tools/revitalize/gate_pass7.sh` to `gate_pass8.sh` (same steps): debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep over `sim-core/src sim-systems/src sim-engine/src` (0 hits), `gdscript_strict_check.sh`, `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, both Python unittest discoveries (`tools/revitalize`, `tools/worldmodel`; WM-1's `wm1/` is not on this branch).

## 4.5 v13 guard after every behaviour Part
After Parts 2 and 3 each: re-run `game_v13` 64 × 10 and 32 × 120 with Pass 7's final flags and compare with Part 0's v13 runs; every shared value must be identical (only `schema_version` and new keys may differ). A difference is fixed before the next Part.

## 4.6 Seeds: tuning and held-out
- **Tuning seeds:** 0–63 for 10-day runs, 0–31 for 120-day runs. Every round of Part 4 is judged on them.
- **Held-out seeds:** 64–127 for 10-day runs, 32–95 for 120-day runs. Part 1 runs v12 and v13 on them (the reference); `game_v14` is run on them **once**, in Part 5, after tuning is closed. Nothing in `game_v14()` may change because of what the held-out runs show, except the single corrective round of 10.3.
- **Fresh seeds:** 96–159 (120 days) and 128–191 (10 days), used only by 10.3.
- Dataset v1 seeds (11.1) are disjoint from all of these.

---

# 5. PART 0 — Baseline and frozen references
1. Record `PASS8_BASE_SHA` (`ce7acb25`), start time and `uname -a`. Commit `PASS8_GOAL.md`.
2. Release probes with exactly the flags of Pass 7's recorded runs (METRICS.md `## Pass 7 Part 5` for v13 — the eight Pass 6 clocks, `--check-determinism` — and `## Pass 7 Part 7` for the others): `game_v13` 64 × 10 and 32 × 120 against `p7-5_round2_10day` / `p7-5_round2_120day`; `game_v12` 64 × 10 and 32 × 120 against `p7-7_final_v12_*`; legacy against `p7-7_final_legacy`. Every shared key identical, timing excepted; a difference stops the run until explained.
3. Record `sim_test::probe::fingerprint` of `game_v13_scene(seed)` for seeds 0–3 after 1,440 ticks as the constants of `harness_v13_fingerprints_unchanged`; add one test comparing `WorldRules::game_v13()` field by field with a literal of its Pass 7 values.
4. Rebuild the v0 sample with the Part 0 recorder (still `ce7acb25`'s code) under `.harness/evidence/worldmodel/p8-0_sample/`, with `WORLDSIM_GIT_SHA=dc6760b0ea77370231aca88f3ee6bcc3f8056037` (the sha the committed sample records), and confirm it is byte-identical to `docs/worldmodel/sample/`.

---

# 6. PART 1 — The v14 skeleton, instruments, held-out machinery

## 6.1 v14
`game_v14` rules, worldgen, scene, probe and recorder wiring as in 4.2, no behaviour beyond v13 yet. The new `FirewoodRule` fields of 7.1 enter `game_v11()`'s literal with their v13-reproducing values (v12 and v13 derive from v11), and every literal that pins `FirewoodRule` is re-pinned with a ledger row.

## 6.2 Probe schema 8
Every schema-7 key keeps its meaning. New keys: `night_burn_per_hour` (the rate `FireSystem` used at its last run, 0 outside the burn window), `winter_fall_today` (the winter deadwood fall per day in force, −1 outside a winter). `summary.json` gains a `rules` block: the firewood rule's severity weights, severity burns and falls, and the 7.1 fields of the scene, so evaluators read weights from the run and never from a default.

## 6.3 Recorder schema 2
- `global.npy` gains `night_burn_per_hour` and `winter_fall_today`.
- `winter_severity`, `night_burn_per_hour` and `winter_fall_today` are marked `"hidden": true` in meta (the world's hidden weather); `lifespan_days` stays hidden. Also hidden, because they encode the severity or the draws exactly: the settlement table's `wood_target` (in winter it is `winter_base_days` × the severity's rate × 13) and every field of `latent.npy` (the change of `wood_winter_acc` gives the day's fall). Behaviour that depends on the severity (urgent trips, stock changes) stays observable: inferring the winter from it is the point. `meta.json` lists `"observable_fields"` per array (all non-hidden fields) and keeps `schema_version` 2. `wm_data.py` gains `Episode.observable(array)` and still loads schema 1 (test on the committed v0 sample).
- Fix the Pass 7 review nits: `bench_v0.py` comments that say "applied" for requested amounts and the `make_dataset.py` `--resume` docstring (comments only; v0 numbers unchanged).

## 6.4 `tools/revitalize/evaluate_pass8.py`
Reuses `evaluate_pass7`. Reads the severity weights from the run's `summary.json` `rules` block (an explicit `--severity-permille` overrides and is recorded). Adds:
- **X3 clauses** of section 9 (R², per-severity spread, first-night spread).
- **Held-out comparison mode** `--compare-to <evaluation of v12 on the same seeds>`: for every per-seed clause of Y3, Y7, Y8, Y10, Y11 and Y12 (a clause judged on its minimum or maximum over seeds; for a clause that combines a median with an every-seed bound, only the every-seed part), it counts the failing seeds of both evaluations and passes when this run's count ≤ the reference's count + 1; it lists the failing seeds of both. Y2 is never in comparison mode. Population clauses (medians, shares, X3, X4) are judged as usual. `evaluate_pass8` stores every per-seed value in its JSON (`per_seed`), and `--compare-to` takes an `evaluate_pass8` output (Pass 7's `z_targets.json` holds only quantiles).
- The Winters table gains `night_burn_mean`, `fall_mean` and the dark-night count by severity.
Tests with synthetic rounds (`test_evaluate_pass8.py`).

## 6.5 Baselines on both seed sets
Run, with the Part 1 binary and Pass 7's final flags: `game_v12` and `game_v13` on the tuning seeds (64 × 10, 32 × 120) and on the held-out seeds (64–127 × 10, 32–95 × 120). Evaluate all four with `evaluate_pass8.py` (so they carry per-seed values) into `docs/revitalize/probe/p8-1_*_targets/`. The planner's held-out numbers (3.2) should reproduce; a difference is explained before Part 2. Commit, push WIP.

---

# 7. PART 2 — Winters that vary night by night

## 7.1 Rule fields (`FirewoodRule`, `sim-core/src/rules.rs`)
```rust
pub struct FirewoodRule {
    // existing fields unchanged
    /// Night-by-night change of the winter burn per hour: a night burns its rate
    /// + one of (−step, 0, +step), drawn per evening with these weights (thousandths
    /// summing to 1000). None = every night at its severity's rate (v13).
    pub winter_night_burn_permille: Option<[u16; 3]>,   // v13 None, v14 Some([250, 500, 250])
    /// Size of that change, burn units per hour.
    pub winter_night_burn_step: u16,                    // v13 0, v14 2
    /// Day-by-day change of the winter deadwood fall: the day's fall is the
    /// severity's fall + an integer uniform in [−spread, +spread], floor 0.
    pub winter_fall_spread: u32,                        // v13 0, v14 2
    /// Windfall lands on the snow (7.3).
    pub windfall_over_snow: bool,                       // v13 false, v14 true
}
```
Start values of `game_v14()` (variant C of 3.3): severity weights `[350, 400, 250]`; night weights `[250, 500, 250]`, step 2; fall spread 2; severity burns `[2, 3, 4]` and falls `[8, 6, 4]` as v13.

## 7.2 The draws (`settlement/firewood.rs`)
- `night_burn_delta(resources, rule, evening_day) -> i16`: `u = mix(mix(world_seed ^ 0x4E49_4748_5442_5552) ^ evening_day) % 1000` with `mix` as in Pass 7 (one splitmix64 step); −step when `u < w0`, 0 when `u < w0 + w1`, else +step. `evening_day` is the display day index of the evening the night began (Pass 7's `night_tick_at` booking). Only in a winter with severity weights set; 0 otherwise.
- `winter_fall_delta(resources, rule, day) -> i32`: `u = mix(mix(world_seed ^ 0x4641_4C4C_5350_5244) ^ day)`, delta = `(u % (2·spread + 1)) − spread`; only in winter with severity weights and `spread > 0`.
- `FireSystem`: the hourly rate = `max(1, winter_burn_per_hour(night) + night_burn_delta)` (+1 on a cold-snap night, as Pass 7). `ResourceRegenSystem`: the winter per-day fall = `max(0, severity fall + winter_fall_delta(current day))`, passed to the unchanged accumulator.
- **Villagers do not see the draws:** `wood_target`, `tonight_burn` and urgency keep the severity's rate. Add one helper `night_rate(resources, rule, tick)` — the rate `FireSystem` withdraws per settlement-hour at that tick: severity rate + night delta (floor 1), + 1 while `cold_snap_tonight`, and 0 when the tick is outside the burn window — used by `FireSystem`, the probe key and the recorder, so the three never disagree.

## 7.3 Windfall on the snow
With `windfall_over_snow`: (a) `Windfall` fills every spot up to its max in winter too (not `min(max, winter_deadwood_cap)`); (b) the snow burial (`bury_wood`) runs only at the first regrowth pulse of each winter (the snowfall), not at every winter pulse. Without a windfall nothing ever raises a spot above the cap during winter, so (b) alone changes nothing; prove it: `harness_snow_burial_once_is_inert_without_windfall` (v13 rules with `windfall_over_snow = true` and no interventions, against plain v13: identical `state_digest` every 60 ticks over 35 days, seeds 0–3).

## 7.4 Tests — new file `harness_revitalize_pass8_winter.rs`
Read expected numbers from `WorldRules::game_v14()` (Part 4 may change it), plus one table test of explicit literals computed by an independent Python implementation (as Pass 7 did): `harness_night_burn_is_a_pure_draw` (same seed and evening, same delta; over 10,000 evenings each delta's share within ±0.02 (absolute) of its weight; v13 rules: no delta anywhere), `harness_fall_spread_is_a_pure_draw` (each value's share within ±0.02 absolute of uniform over 10,000 days), `harness_villagers_do_not_see_tonights_draw` (`wood_target` and `tonight_burn` under `game_v14()` equal those of `game_v14()` with `winter_night_burn_permille = None` and `winter_fall_spread = 0`, at every hour of a winter), `harness_night_rate_floor_is_one`, `harness_last_night_keeps_its_evening_draw`, `harness_windfall_lands_on_snow`, the inertness test of 7.3, `harness_firewood_v13_unchanged`. Commit, push WIP; run 4.5; fast-forward green if the gate is green.

---

# 8. PART 3 — Interventions and the recorder under v14
1. Z6 again for v14 (Pass 7's protocol: 8 seeds × 7 kinds at day 23 12:00 → `docs/revitalize/probe/p8-3_z6/`): prefix identity everywhere; windfall now applies a positive amount in at least 6 of 8 seeds.
2. Recorder: hidden columns of 6.3 match the probe keys on 2 seeds × 30 days at every common tick (`tools/worldmodel/test_hidden_columns.py`, skips without the binary); 9.4-style byte identity (two runs, `--workers 1` vs `4`) for v14 episodes.
3. Commit, push WIP; run 4.5.

---

# 9. TARGETS (game_v14; table order = priority)

Pooling and night booking as Pass 6. Seed-winters: 32 tuning seeds × 4 = 128; held-out 64 × 4 = 256.

| # | target |
|---|---|
| X1 | determinism 64/64 (10 d) and 32/32 (120 d) on tuning and held-out seeds; legacy pin; v8–v13 reproduce Pass 7's final summaries on every shared key |
| X2 | village health: Y2 with Pass 6's thresholds (10 days: zero in every seed; 120 days: median 0 and maximum ≤ 2) on every seed set, never in comparison mode; every per-seed clause of Y3, Y7, Y8, Y10, Y11 and Y12 in comparison mode (6.4) against v12 on the same seeds, on the tuning seeds and (Part 5) the held-out seeds; population clauses of Y3 as Pass 6 |
| X3 | winters a model has to read, on tuning and held-out seed-winters: (a) severity-only R² of the dark-night count ≤ 0.75; (b) normal and harsh winters each: the most common dark-night count ≤ 0.75 of them; (c) no-dark-night share in [0.15, 0.60]; (d) ≥ 2 dark nights in [0.10, 0.50]; (e) first dark night on ≥ 4 distinct nights, ≤ 0.50 of them on the seventh; (f) each severity's share within ±0.10 of its weight |
| X4 | Pass 7's Z4 kept: re-target share median ≤ 0.10; normal-severity winter wood gathered ≥ 40.5 |
| X5 | Pass 6 targets Y4, Y5, Y6 and the population clauses of Y7, Y8, Y10, Y11 kept with Pass 6's thresholds |
| X6 | interventions: every Pass 7 intervention test passes; 8.1's Z6 for v14 |
| X7 | recorded worlds: schema 2 byte identity; hidden columns match the probe; the v0 sample still loads (schema 1) and Part 0.4 reproduced it; dataset v1 manifest verified |
| X8 | ms per agent-tick v14 ≤ 1.2 × v13, back to back on one host, recorder off |
| Y9 | reported only (summer noon idle, Pass 6) |

R² (X3a): over the seed-winters, `1 − Σ_s Σ_{w∈s} (n_w − mean_s)² / Σ_w (n_w − mean)²`, where `n_w` is the winter's dark-night count and `mean_s` the mean of its severity; undefined (fail) if every winter has the same count. Pass 7: 0.94.

---

# 10. PART 4 — Tune on the tuning seeds (three rounds maximum), PART 5 — verify on held-out seeds

## 10.1 Part 4
Runs (release): `game_v14` 64 × 10 and 32 × 120 on the tuning seeds with Pass 7's final flags → `docs/revitalize/probe/p8-4_round<N>_{10day,120day}/`; evaluate with `evaluate_pass8.py --compare-to` the Part 1 v12 tuning evaluation. Screening variants on throwaway runs is allowed (only three recorded rounds count; list every screen in METRICS.md with its numbers).
Knobs (only these, all in `game_v14()`): severity weights each 150–600 summing to 1000; the night weights (−, 0, +) each 100–600 summing to 1000; the step 1–3; the fall spread 0–3; severity burns mild 1–3, normal 3, harsh 3–5; severity falls mild 6–10, normal 6, harsh 2–6. Every v13 value not listed stays. When targets conflict, section 9's order decides (X2 before X3). "Best" = the most targets passed in table order, ties broken by X3a's R². Record every round; keep the best in `game_v14()`. Commit, push WIP; fast-forward green if the gate is green.

## 10.2 Part 5 — the held-out look (once)
Run `game_v14` on the held-out seeds (64–127 × 10, 32–95 × 120, `--check-determinism`) and evaluate in comparison mode against Part 1's held-out v12 evaluation → `docs/revitalize/probe/p8-5_heldout_*`. Report every X target on held-out seeds beside the tuning value, with the failing seeds of v12, v13 and v14 per clause.

## 10.3 One corrective round (only if Part 5 fails a target)
At most one change of the knobs, justified by a mechanism (not by a seed), recorded before running; then verify on the **fresh seeds** (96–159 × 120, 128–191 × 10) for v12 and v14 and report tuning, held-out and fresh side by side. No further change after that, whatever the fresh seeds show.

---

# 11. PART 6 — Dataset v1 and WM-Bench v1

## 11.1 Dataset v1 (`WM_DATA_ROOT/v1/`, outside git; `make_dataset.py v1`)
All splits `--every 60`, recorder schema 2.

| split | scene | seeds | days | interventions | train / val / test | `--agents` |
|---|---|---|---|---|---|---|
| A_obs | game_v14 | 10000–10127 | 120 | none | 10000–10095 / 10096–10111 / 10112–10127 | 10000–10031, 10096–10099, 10112–10119 |
| B_act | game_v14 | 11000–11063 | 120 | 8 random each (v0's sampler) | 11000–11047 / 11048–11055 / 11056–11063 | 11056–11063 |
| C_cf | game_v14 | 12000–12031 | 35 | factual + 5 variants | — / 12000–12007 / 12008–12031 | none |
| OOD_v13 | game_v13 | 13000–13031 | 120 | none | test | none |
| OOD_long | game_v14 | 14000–14015 | 240 | none | test | none |

C_cf variants: v0's four (gift_wood 60 and rot_wood 60 at day 21 18:00, tick 29,400; rain 13 hours at day 24 18:00, tick 33,720; cold_snap 2 nights at day 25 18:00, tick 35,160) plus `windfall` 20 at day 23 12:00 (tick 31,920). Write `docs/worldmodel/DATASET_v1.md` and `dataset_v1_manifest.json` as v0's; v0's files stay untouched.

## 11.2 WM-Bench v1 (`tools/worldmodel/bench_v1.py`, numpy only; `docs/worldmodel/BENCH_v1.md`)
Same variables V and baselines as v0, with these changes:
- **Inputs are observable fields only** (`Episode.observable`); `winter_severity`, `wood_target`, the draws and `latent.npy` are never inputs. Report the ridge also with a "severity oracle" input as an upper reference, labelled so.
- **T1** as v0. **T2** from start rows at every hour of days 3, 10, 17 … (24 starts per start day), horizons 24 h and 168 h; v0's 06:00-only number reported beside it.
- **T3** as v0 (labelled "arithmetic: 18:00 stock vs tonight's rate"), and **T3b winter outlook**: at 06:00 of each winter's third day (one-based display days 24, 52, 80, 108), predict the number of dark nights among the five remaining nights; MAE and exact accuracy; baselines: train mean; ridge on (wood stock, deadwood total, the stock drop over each of the two previous nights, the deadwood change over the two previous days, members); the severity oracle (train mean per severity, labelled reference).
- **T4** per variant and target: MAE, sign accuracy and normalized MAE (MAE / mean |true effect| over test seeds; reported as undefined, "—", when that mean is 0); ranking on dark nights, cold member-hours and food store; deaths reported, not ranked; no cross-unit mean anywhere.
- **T5 agent next hour** (A agent episodes; train 10000–10031, val 10096–10099, test 10112–10119): next state tag and target kind accuracy, next position Chebyshev error, next hunger/thirst/fatigue nMAE; baselines persistence and 24 hours earlier.
- Sanity checks (report either way): ridge beats persistence on T1 food and wood stock; on T3b the ridge beats the train mean; every C_cf variant changes at least one ranked target in ≥ 25 % of test seeds; the severity oracle's T3b MAE is not 0 (if it is, X3 failed in the data).
- `BENCH_v1.md` ends with the numbers to beat and a side-by-side of v0 and v1 for every task both have.

## 11.3 DGX copy
`rsync` `WM_DATA_ROOT/v1` to `hyunlord@100.70.109.50:/home/hyunlord/work/worldsim-wm-data/v1` and verify every manifest hash there with `sha256sum -c` (copy and hash only: WM-1 uses the DGX's GPU). Say whether it was done.

## 11.4 Tests
`test_bench_v1.py` on synthetic arrays (observable-only inputs, every-hour starts, T3b counting, normalized T4, T5 metrics). Commit, push WIP.

---

# 12. PART 7 — Independent review, final verification, report
1. **Independent review:** a built-in `general-purpose` subagent with fresh context gets `git diff ce7acb25..HEAD -- rust/ tools/revitalize/ tools/worldmodel/`, sections 4, 6, 7, 9, 10 and 11, the X table (tuning, held-out and fresh if any); it reports concrete defects (determinism, v8–v13 invariance, the draws and their booking, villagers seeing the draws, hidden fields leaking into bench inputs, held-out discipline — did anything in `game_v14()` change after Part 5 outside 10.3 —, the comparison-mode counting, leakage between dataset splits, metric errors) with file:line evidence. Fix confirmed defects (two rounds maximum); record every finding and disposition in DECISIONS.md. A configuration change after the review goes through the gate, the final probes and the held-out look again and is named in the report.
2. Full gate on the final WIP head; totals versus Pass 7 (1,987 debug and 1,987 release).
3. Final probes: legacy, v8–v13 reproduce Pass 7; `game_v14` tuning and held-out runs with census and clock samples; X6 again; v13 and v14 timings back to back for X8. If `game_v14()` changed after dataset v1 was recorded, re-record it.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS8.md` in Korean (section 14 format), answering each item of section 13 with its evidence level. Append `## Pass 8` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 8 section (skip and say so if `gh` is not authenticated).

## 12.1 Merge touch-points for the art run (list them in the report)
The live game builds `game_v14`; `winter_severity` in the time snapshot is unchanged (the god may know it; the recorded world hides it); nights of one winter now differ (an art-side "tonight is bitter" cue could read `night_burn_per_hour` if exposed later — not exposed now); windfall piles can stand above the snow cap.

## 12.2 Out of scope
Summer work (Y9); any neural or LLM model and any training (WM-1); rendering and HUD; new death causes; trade; new settlements; map size; merging into `lead/main`; the art-owned paths; WM-1's paths.

---

# 13. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 같은 혹독함의 겨울이라도 밤마다 장작 줄어드는 속도가 다르다. 보통 겨울에 불이 한 번도 안 꺼지기도 하고, 두세 밤 꺼지기도 한다.
2. 숲에 떨어지는 가지 수가 날마다 조금씩 다르다.
3. 마을 사람들은 오늘 밤이 유난히 추울지 모른다. 장작 목표와 서두르는 아침은 혹독함 기준 그대로다.
4. 겨울에 바람(windfall) 개입을 쓰면 눈 위에 가지가 실제로 쌓인다(FFI로만 확인 가능, 버튼은 아직 없음).
5. 굶주림·탈수 사망은 없어야 한다(Pass 6 기준: 120일 중앙값 0, 최대 2). 봄·여름·가을은 Pass 7과 같다.
6. 라이브 게임이 game_v14로 시작하고 콘솔에 SCRIPT ERROR가 없다.

---

# 14. 구현 완료 보고 (REPORT_PASS8.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
밤별 소모·날별 낙엽 추첨(순수 해시, 저녁 기준 배정), 마을 사람이 모르는 날씨, 눈 위의 바람, 숨은 필드와 기록기 스키마 2, 튜닝·검증 시드 분리와 v12 대비 비교 방식, 데이터셋 v1과 벤치 v1의 변경점, v8~v13 고정 방식

### 기능 설명
플레이어 관점과 세계 모델 연구 관점에서 각각 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
X1~X8 표: 튜닝 시드 / 검증 시드(/ 새 시드) 각각 v12 / v13 / v14, 중앙값·p10·p90·최소·최대, 시드별 조항의 실패 시드 목록
혹독함별 꺼진 밤 분포와 R², Winters 표, 조정 라운드별 수치와 걸러 본 변형

### 데이터셋 v1
분할별 에피소드 수·크기·경로·매니페스트, DGX 복사 여부

### WM-Bench v1
과제(T1~T5, T3b)·기준선·분할별 수치, v0 대비 변화, 건전성 점검, WM-1이 넘어야 할 숫자

### 독립 리뷰
지적 사항과 처리 결과

### 아트 작업과 맞물리는 지점
12.1 목록

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 컨텍스트 압축 뒤 목표 파일을 다시 읽은 시점, 백그라운드 작업, 동시에 돌던 다른 작업(WM-1, 아트)

### 확인된 제한사항

### Harness 결과
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트 두 묶음)

### 정직한 판단
겨울이 이제 라벨이 아니라 읽어 내야 하는 날씨인가? 검증 시드에서도 성립하는가? 이 데이터로 세계 모델의 추론 능력을 시험할 수 있는가? 아직 부족한 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, the paths of `DATASET_v1.md`, `dataset_v1_manifest.json` and `BENCH_v1.md`, and the time spent per Part.
