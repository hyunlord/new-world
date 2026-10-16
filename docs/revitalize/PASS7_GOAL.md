# GOAL: WorldSim — Revitalize Pass 7: a world-model testbed — winters that differ, the god's hand, recorded worlds, a first benchmark

Repository `hyunlord/new-world`. Continue the Pass 6 line: `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`, both at `fe60de21249ad46ad2e2d9ca420d896066277c82` (code head `01fd135d`; draft PR #510 into `lead/main`).
This run is executed by **Claude Code**, unattended. The art run (**Art Pass 5**, Codex, branch `codex/art-production-2026-10`) may be working on the same Mac; section 1 says how to share it.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4            # the Pass 4–6 clone, hooks already neutralized
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # fe60de21
cp ~/Downloads/claude_goal_worldsim_pass7.md docs/revitalize/PASS7_GOAL.md
tmux new -s pass7
cd /Users/rexxa/work/new-world-pass4
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS7_GOAL.md>
```
The repository's Claude Code hooks are neutralized **in this clone only**: `.claude/settings.json` holds `{}` and is hidden from git with `--skip-worktree`. Never edit, stage or commit it and never clear that flag. No git hooks are installed. The local `.gitignore` edit and `.ignore` belong to unrelated graft tooling; leave them uncommitted, as Passes 5 and 6 did.

**This document is `docs/revitalize/PASS7_GOAL.md`.** Commit it unchanged in Part 0. At the start of every Part, after every compaction or pause, and whenever you are unsure of a number, re-read the relevant section from that file.

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
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 7`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5` and background job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.**
6. **Time rule:** record start time and each Part's start and end. Past **10 hours** of working time, finish the current step, push, go to Part 7 (Part 5 must have at least one recorded round before that).
7. **Three strikes** per failing target, then keep the best configuration and record evidence and hypothesis.
8. **Drop order when time is short:** the OOD splits and task T4 of Part 6 → the `cold_snap` and `windfall` interventions (keep the other five kinds; B_act then draws from five kinds, C_cf drops its `cold_snap` variant, Z6 covers five kinds) → the agent, resource, terrain and building tables of the dataset (keep the global and settlement tables). Never drop Parts 0, 1, 2, 5 or 7, nor the global and settlement export of Part 4.
9. **No image generation, no neural-network training** (that is the next step, on the DGX).
10. **Keep bulky output out of context:** probe runs, exports, test logs go to files; read summaries.
11. **Long commands run in the background** (one Bash call is cut off after 10 minutes): `nohup bash -c '<command>; echo $? > /tmp/p7_<name>.exit' > /tmp/p7_<name>.log 2>&1 &`, poll with short calls, check `pgrep -fl` before starting a job again. Keep gate logs in `docs/revitalize/verification/p7-<N>/`.

---

# ⚠️ 1. MACHINES

- **The Mac** clone `/Users/rexxa/work/new-world-pass4`. The art run may be active in `/Users/rexxa/work/new-world-art-prod`; never read, write or build there, never touch `~/github/new-world-wt/lead` or other checkouts, never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe and export `--workers 4`.
- Datasets live outside git: `WM_DATA_ROOT=/Users/rexxa/work/worldsim-wm-data` (create it). Check free disk before Part 6 (the dataset is about 1–1.5 GB; stop and record if less than 15 GiB is free).
- Python: `tools/worldmodel` uses numpy (no scipy, no torch, no pandas). `python3` should already have it (`tools/revitalize/visual_qa.py` imports it); check with `python3 -c 'import numpy; print(numpy.__version__)'`. If it is missing, create `/Users/rexxa/work/worldsim-wm-venv`, install numpy there, use it for every `tools/worldmodel` command and the gate's worldmodel step, and record it.
- Windowed Godot only through `tools/revitalize/with_window_lock.sh` in one command (the art run uses the same lock); this pass needs only headless Godot. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Timing-sensitive runs (Z8) run back to back with nothing else heavy on the Mac: check `pgrep -fl 'Godot|cargo|worldsim_probe|worldsim_wm'` first; if the art run is busy, wait for it or record that it was running.
- **DGX Spark** (`ssh hyunlord@100.70.109.50`, `/home/hyunlord/work/new-world-revitalize`) is optional for long batches (detach to your pushed WIP commit, `CARGO_BUILD_JOBS=10`, `--workers 8`, copy summaries back; no ComfyUI 8188, no dev servers 3100/3200, no other checkouts, never `~/github/worldsim-training`). Never use it for Z8 timings. 12.4 copies the dataset there if reachable.

---

# 2. GIT

- Every commit goes to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.4) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p7-N): <what changed>`. Stage explicit paths only, never `git add -A`.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`, and the new `docs/worldmodel/**` and `tools/worldmodel/**`.
- **Must not change** (the art run owns them): `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, and the static tests that read `.gd`/`.tscn`/`.gdshader` files, except minimal Rust-source re-pins allowed in 4.3.

---

# 3. CONTEXT

## 3.1 Pass 6 result (verified by the planner from the committed evidence)
Gate green on code head `01fd135d` (debug and release 1,941 each); legacy, v8, v9, v10 and v11 reproduce Pass 5 on every shared value. `game_v12` final (`p6-7_final_v12_*`): Y1–Y8 and Y10–Y13 pass; Y9 failed (spring–autumn noon work 0.317, idle 0.619) and stopped after three strikes. Medians by season (spring / summer / autumn / winter): wood stock 26 / 21 / 182 / 106 against targets 39 / 19.5 / 195 / 156; wood gathered per day 30 / 0 / 50 / 6; wood trips per day 15 / 0 / 25 / 12; lit at 02:00 1 / 1 / 1 / 0.856; food store 182 / 280 / 498 / 318; adult work at noon 0.499 / 0.074 / 0.635 / 0.136. Winters (32 seeds × 4 = 128 seed-winters): wood on the first winter day 202; one dark night per winter, on the seventh night in 124 of 128; 96 wood trips and 45 units gathered per winter; cold member-hours 336; food at winter's end 246 (minimum 44). No starvation or dehydration death.

## 3.2 The planner's errors in the Pass 6 brief
- **Summer lost its work by design.** The brief removed summer's wood preparation (`winter_prep_in_summer` false) without giving summer anything else to do; summer noon work fell from 0.609 to 0.074 and no allowed knob could reach Y9. Summer work needs a new mechanism; it is out of scope here (13.2) and Y9 is reported only.
- **The winter was arithmetic.** Autumn prepares four nights, six branches fall a day and seven nights burn 39 each: the shortfall lands on the seventh night in almost every seed. A fixed calendar outcome.

## 3.3 What is still wrong
- **Every winter is the same winter** (3.1): the dark night is a date, not an event.
- **Half of the winter trips find nothing:** 96 trips bring 45 units. An independent check measured why (v12, seeds 0–7): the branch that re-targets a wood gatherer whose spot is empty fires about 49 times a winter, almost always on the way. Claims count only members in `Seeking{GatherWood}`; a gatherer who has arrived is `Consuming{GatherWood}` for a tick before the pickup and holds no claim, so an idle worker claims the same one-unit spot and is re-targeted when the first one takes the branch. The urgent double cap adds only about 12 double claims a winter.
- **Summer noon is idle** (3.2).

## 3.4 Direction: WorldSim as a world-model testbed
A world model is a learned model of how a world changes: given what is observed now and what is done to the world, it predicts what comes next, over hours, seasons and years, and what would have happened under a different action. WorldSim is a good place to build and test one, because the ground truth is cheap and exact: the simulation is deterministic, runs 120 days of a village in seconds, exposes its whole state, has rhythms at four time scales (hour, day, season, year), and can be replayed with and without an intervention to give an exact counterfactual. Rulesets v8–v13 give controlled distribution shifts, and the agent count is a scale knob (tens now, thousands later).

What a testbed needs, and what this pass builds:
1. **Dynamics that are not a calendar.** If the dark night is always the seventh, a model that memorises dates scores perfectly. Winters must differ: a severity drawn per winter (Part 2).
2. **Actions.** A world model must learn how the world answers what is done to it. The actions are the god's hand — gifts, spoilage, rain, cold snaps, windfalls — which are also the game's own player verbs (miracles and disasters; the oracle comes later) (Part 3).
3. **Exact counterfactuals.** The same seed replayed with and without an intervention; identical before it, different after (Part 3).
4. **Recorded worlds.** A stable, documented, byte-reproducible export of what the renderer sees, plus the hidden state the next step depends on (Part 4).
5. **A first benchmark.** Fixed splits and simple baselines, so the learned models of the next step have numbers to beat (Part 6).

What comes after this pass (not here): **WM-1** on the DGX — learned models (a settlement-level sequence model, an agent-level model, and a text world model fine-tuned from the existing Qwen 0.8B/2B pipeline) evaluated on WM-Bench v0; **WM-2** in the game — the god's foresight (preview the predicted outcome of an intervention before casting it) and aggregate surrogates that run far settlements when the map holds thousands of agents. Every choice in this pass keeps those uses possible: settlement-level tables for surrogates, agent-level tables for fine models, interventions shared by the probe, the export and the FFI.

## 3.5 This pass
`game_v13` = `game_v12` + a severity drawn for each winter + claims bounded by what a spot holds. Seven interventions with exact replay. A deterministic world recorder. Dataset v0 and WM-Bench v0 with baselines. v8–v12 frozen.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Architecture and determinism (unchanged)
All simulation logic in Rust; FFI only adds keys and one command; new collections `BTreeMap`/`BTreeSet`; no order-dependent hash iteration; no transcendental f64 in simulation crates; seeded `splitmix64` streams only; no new crates. The winter severity is a pure hash of (world seed, year) and advances no stream; interventions draw no random numbers. No new enum variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType` or `DeathReason`. Time windows use integer minute-of-day (`clock_in_window`, wrap-around allowed). Day index and seasons as in Pass 4.

## 4.2 Versioning — v8 to v12 frozen, v13 new
- `game_v8()` … `game_v12()` and their `WorldGenConfig`s keep their behaviour exactly; every new field takes the value that reproduces them bit-for-bit.
- In Part 1 add `WorldRules::game_v13()` (= `game_v12()` until Part 2 changes it), `WorldGenConfig::game_v13(seed)` (= `game_v12(seed)`), `game_v13_scene(seed)`, and the probe scene `game_v13` wherever `game_v12` appears in the probe: `sim-test/src/probe/args.rs` (usage text and the scene check), `probe/runner.rs` (the scene builder and the determinism checks that list `game_v9` … `game_v12`) and `tools/revitalize/probe.sh --help`.
- The live game switches to v13 in both `world_node.rs` constructors.

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; `harness_population_legacy_scene_pinned`, `harness_population_game_world_survives_3_days` and all Pass 2–6 tests pass unchanged. Never change `sim-test/src/probe/fingerprint.rs` or `probe/long_run.rs::trace`. Allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 7`: struct-literal field additions with legacy values; probe schema tests moving to schema 7; Rust-source static assertions that pin code this goal deliberately changes (the live world constructor, additive FFI keys and the new command), re-pinned and never weakened.

## 4.4 The gate (required before the green branch moves)
Copy `tools/revitalize/gate_pass6.sh` to `gate_pass7.sh` (same steps) and add `python3 -m unittest discover -s tools/worldmodel -p 'test_*.py'`: debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep over `sim-core/src sim-systems/src sim-engine/src` (0 hits), `gdscript_strict_check.sh` (read-only checker), `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, both Python unittest discoveries.

## 4.5 v12 guard after every behaviour Part
After Parts 2 and 3 each: re-run `game_v12` 64 × 10 and 32 × 120 (the arguments of Part 0, step 2) and compare with Part 1's v12 runs; every shared value must be identical. A difference is fixed before the next Part.

## 4.6 Interventions are opt-in
No intervention runs unless a schedule (8.3) or the FFI command (8.4) asks for one. Without them every scene behaves exactly as before; 4.5 is the guard.

---

# 5. PART 0 — Baseline and frozen references

1. Record `PASS7_BASE_SHA` (`fe60de21`), start time and `uname -a`. Commit `PASS7_GOAL.md`.
2. Release probes with exactly the flags METRICS.md records for Pass 6's final runs (`## Pass 6 Part 7`): `game_v12` 64 × 10 against `p6-7_final_v12_10day` and 32 × 120 against `p6-7_final_v12_120day`; `game_v11` 64 × 10 against `p6-7_final_v11_10day`; legacy against `p6-7_final_legacy`. Every shared key identical, timing excepted; a difference stops the run until explained.
3. Record `sim_test::probe::fingerprint` of `game_v12_scene(seed)` for seeds 0..4 after 1,440 ticks; they become the constants of `harness_v12_fingerprints_unchanged` (keep the earlier ones). Add one test that compares `WorldRules::game_v12()` field by field with a literal of its Pass 6 values.

---

# 6. PART 1 — The v13 skeleton, instruments and the evaluator

## 6.1 v13
`game_v13` rules, worldgen, scene and probe wiring as in 4.2, no behaviour beyond v12 yet. The new `FirewoodRule` fields of 7.1 enter `game_v11()`'s literal with their v12 values (v12 is built from v11), and the literals that pin `FirewoodRule` or the tallies are re-pinned with ledger rows: `harness_revitalize_pass6_frozen.rs`, the firewood test literal in `firewood.rs`, `harness_revitalize_tallies.rs`, the collector test in `world_node.rs`; `probe/args.rs` currently asserts that `game_v13` is rejected — flip that test.

## 6.2 Tallies and resource fields (display-only or inert until used)
- `wood_retargets`: firings of the branch that re-targets a wood gatherer with `old = Some(tile)` (its spot is empty); counted in every version, display-only.
- `interventions_applied`: interventions with `applied > 0` (8.2).
- `SimResources` gains, now, the inert fields Part 3 uses: `fire_blocked_until: BTreeMap<SettlementId, u64>`, `cold_snap_nights: u32`, `cold_snap_tonight: bool`, `intervention_log: Vec<InterventionRecord>` (empty and false by default).

## 6.3 Probe schema 7
Every schema-6 key keeps its meaning; never null. New keys: `winter_severity` (0 mild, 1 normal, 2 harsh, −1 outside a winter or without the rule; samples from 00:00 to the burn end follow the night booking, so they read the evening's winter), `wood_retargets_total`, `interventions_applied_total`, `fire_blocked_settlements` (settlements under rain now), `cold_snap_nights_left`.

## 6.4 FFI (additive, all scenes)
`get_time_snapshot`: `winter_severity` (int, as 6.3). A pure-Rust collector with a unit test, as Pass 6 did for its keys.

## 6.5 Evaluator and the v12 baseline
`tools/revitalize/evaluate_pass7.py` (+ tests, stdlib only, reusing the `evaluate_pass6` helpers) computes the Z targets of section 11 that come from schema-7 raw JSONL (Z2–Z5 and Y9; Z1, Z6, Z7 and Z8 are receipts from the probe, the tests, the recorder and the timing runs) and writes `z_targets.json` and `z_targets.md` with the Pass 6 tables (Work by season, Winters) and the Winters table extended by two columns: severity (the `winter_severity` of the winter's first day checkpoint) and the re-target share (winter `wood_retargets` / winter wood trips). Night booking as Pass 6: a 02:00 sample belongs to the evening of the previous display day.
Re-run `game_v12` 64 × 10 and 32 × 120 (Part 0, step 2 arguments) with the Part 1 binary: every shared value must equal Part 0's runs (only `schema_version` and the new keys differ). Evaluate them into `docs/revitalize/probe/p7-1_v12_targets/` so every Z has a v12 value. Commit, push WIP.

---

# 7. PART 2 — Winters that differ, trips that find wood

## 7.1 Rule fields (`FirewoodRule`, `sim-core/src/rules.rs`)
```rust
pub struct FirewoodRule {
    // existing fields unchanged
    /// Draw weights of a winter's severity (mild, normal, harsh), thousandths summing to 1000.
    /// None = every winter as Pass 6.
    pub winter_severity_permille: Option<[u16; 3]>,     // v12 None, v13 Some([300, 400, 300])
    /// Winter burn per hour by severity; read only with weights.
    pub winter_severity_burn_per_hour: [u16; 3],        // v12 [3, 3, 3], v13 [2, 3, 4]
    /// Winter deadwood fall per day by severity; read only with weights and `winter_regen_units_per_day`.
    pub winter_severity_regen_units_per_day: [u32; 3],  // v12 [6, 6, 6], v13 [8, 6, 4]
    /// A deadwood spot takes no more claims than the units it holds.
    pub claims_bounded_by_amount: bool,                 // v12 false, v13 true
}
```
Legacy, v8, v9 and v10 keep `firewood: None`.

## 7.2 The severity of a winter (`settlement/firewood.rs`)
- The year of display day index `d` (0-based) is `d / (4 × days_per_season)`; with 7-day seasons the winter of year `y` is display days `[28y + 21, 28y + 28)` and its nights, booked by the evening they began (`night_season_from_evening`), so the night after the last winter day belongs to that winter.
- `pub fn winter_severity(resources, rule, year) -> u8`: `u = mix(mix(world_seed ^ 0x5749_4E54_4552_5345) ^ year) % 1000`, where `mix(x)` is `MovementRng::new(x).next_u64()` (one splitmix64 step; `worldgen::noise::splitmix64` is not public); 0 when `u < w0`, 1 when `u < w0 + w1`, else 2. Pure: no state, no stream. The double mix matters: a single `seed ^ year` makes seed s, year y collide with seed s ^ y, year 0 (the 128 probe seed-winters would then hold only 32 distinct draws).
- With weights set and seasons on, during a winter:
  - `FireSystem` burns `winter_severity_burn_per_hour[level]` per hour; the night's season and year are those of the evening the night began (`night_season`'s booking);
  - the spread winter regrowth (`resource_regen::regen_wood_winter`, Pass 6's accumulator and cursor) receives `winter_severity_regen_units_per_day[level]` as its units per day instead of `winter_regen_units_per_day` (pass the per-day value, which the accumulator divides exactly); season and year of the current day (`season_at`), as Pass 6's regrowth reads them;
  - the winter burn per night used by the winter base of `wood_target` (current day) and by `urgent_with` (tonight, evening booking) takes the severity burn: add a helper `winter_burn_per_hour(resources, rule, tick) -> u16` used by all three, so the fire, the target and urgency never disagree;
  - the autumn preparation term keeps `burn_per_hour[3]`: villagers do not know the coming winter.
- `winter_severity` is exposed as 6.3 and 6.4 say.

## 7.3 Claims that see the whole trip (`agent_decision.rs`)
With `claims_bounded_by_amount`:
- a member in `Consuming{GatherWood}` counts as one claim on its tile, in both claim maps (the Firewood arm's claims and the post-pass claims used for re-targeting), so a gatherer who has arrived keeps its spot until the pickup;
- in `choose_spot`, a spot qualifies only while `claims < min(cap, amount)`, where `cap` is the per-spot cap after the urgent doubling; first choices and re-targeting alike.
Without the flag, Pass 6's rules. (The independent check measured, v12 seeds 0–7: the bound alone leaves about 42 re-targets a winter; with the `Consuming` claims, about 3, and the re-target share falls from 0.51 to about 0.06 while wood gathered stays 45 a winter.)

## 7.4 Expected shape (planner's estimate; Part 5 measures it)
Mild burns 26 a night (182 a winter) with 8 branches a day: no dark night. Normal is Pass 6: one dark night, on the sixth or seventh. Harsh burns 52 a night (364) with 4 branches a day: dark nights from about the fifth night, three in all; the independent check confirmed this shape. Village health (Z2) is the binding constraint, not Y7: a document-literal run kept Y7 and Y8 comfortably (winter lit minimum 0.61, cold maximum 0.38) but failed Y3 in one seed (replacement −4, or a 6-day stagnation with the claims fix). When Z2 and Z3 conflict, Z2 wins: soften harsh first (its burn or its share), then the other knobs.

## 7.5 Tests — new file `harness_revitalize_pass7_winter.rs`
Read every expected number from `WorldRules::game_v13()` (Part 5 may change it), plus one table test with explicit literals; keep each test short (the gate runs them in debug): `harness_winter_severity_is_a_pure_draw` (same seed and year, same level; over 10,000 years of one seed each level within ±2 % of its weight; the 128 inputs of seeds 0–31 × years 0–3 give 128 distinct draws; v12 rules: no severity anywhere), `harness_winter_burn_by_severity`, `harness_winter_fall_by_severity`, `harness_last_night_keeps_its_winter_severity`, `harness_autumn_prep_ignores_severity`, `harness_claims_bounded_by_amount` (a one-unit spot takes one claim even under the urgent cap; a three-unit spot under a cap of 2 takes two; a `Consuming{GatherWood}` member holds its claim; re-targeting obeys the same bound), `harness_firewood_v12_unchanged`. Commit, push WIP; run 4.5; fast-forward green if the gate is green.

---

# 8. PART 3 — The god's hand

## 8.1 Interventions
Types in `sim-engine/src/intervention.rs` (new; `SimResources` holds the log):
```rust
pub enum Intervention {
    GiftFood { settlement: SettlementId, amount: u32 },
    SpoilFood { settlement: SettlementId, amount: u32 },
    GiftWood { settlement: SettlementId, amount: u32 },
    RotWood { settlement: SettlementId, amount: u32 },
    Rain { settlement: SettlementId, hours: u32 },
    ColdSnap { nights: u32 },
    Windfall { units: u32 },
}
pub struct InterventionRecord { pub tick: u64, pub intervention: Intervention, pub applied: u64 }
```
"Now" is `engine.current_tick()` (completed ticks; the next `tick()` runs systems at this value), and the record's `tick` is the same value.
- `GiftFood`, `SpoilFood`, `GiftWood`, `RotWood`: add to or remove from the settlement's stockpile (`Settlement::store` / `withdraw`, never below 0); `applied` = units changed.
- `Rain`: `fire_blocked_until[settlement] = max(current, now + hours × ticks_per_day / 24)`; while `resources.current_tick < fire_blocked_until[sid]`, `FireSystem` treats that settlement's fire as out and withdraws no wood. Expired entries stay in the map (they block nothing). `applied` = hours.
- `ColdSnap`: `cold_snap_nights += nights`. `FireSystem` keeps the per-night flag `cold_snap_tonight`: at a run inside the burn window, if the flag is false and `cold_snap_nights > 0`, set it (so a cold snap cast in the middle of a night takes that night from the next hourly run); at the first run outside the burn window with the flag set, clear it and decrement `cold_snap_nights`. While the flag is set the fire burns its rate (severity included) + 1 per hour, and inside the cold window members of a settlement whose fire is out get `Cold` in any season. The +1 is not seen by `wood_target` or `urgent_with` (villagers do not foresee it). `applied` = nights.
- `Windfall`: passes over the registered deadwood spots (`wood_source_max`) in key order starting from the first key, giving each spot one unit while its amount is below its ceiling (its max; in winter `min(max, winter_deadwood_cap)`, winter read with `season_at_tick` at "now"), repeating passes until `units` are placed or a pass places none. `wood_winter_cursor` is left alone. `applied` = units placed.
- An unknown settlement or a zero amount changes nothing (`applied` 0).

## 8.2 Applying
`pub fn apply_intervention(engine: &mut SimEngine, intervention: Intervention) -> InterventionRecord` in `sim-systems/src/runtime/intervention.rs` (new; sim-systems can read `season_at_tick` and the firewood helpers, sim-engine cannot) applies at once, between ticks, appends the record to `SimResources.intervention_log` (every call, `applied` 0 included) and adds 1 to `interventions_applied` when `applied > 0`. The log and the tally are display-only and stay out of `probe::fingerprint` and `long_run::trace` (whose files do not change).

## 8.3 Schedules
A JSON file: a list of `{"tick": u64, "kind": "gift_food" | "spoil_food" | "gift_wood" | "rot_wood" | "rain" | "cold_snap" | "windfall", "settlement": "home" | <id>, "amount": u32}`; `"home"` is the lowest settlement id at that tick (none: `applied` 0); `amount` is hours for `rain`, nights for `cold_snap`, units otherwise; `settlement` is ignored for `cold_snap` and `windfall`. Entries apply when `engine.current_tick() == tick`, in file order, after the recorder has taken that tick's row and before `engine.tick()`; so row t is the state before the actions of tick t, and `interventions.json` pairs them. Parser `sim-test/src/worldmodel/schedule.rs` with tests (unknown kinds, malformed rows and ticks out of order are errors).

## 8.4 FFI (additive)
`#[func] fn apply_intervention(&mut self, kind: GString, settlement_id: i64, amount: i64) -> VarDictionary` returning `{ok: bool, applied: int, tick: int}`; `settlement_id < 0` means home (the lowest settlement id now); `ok` is false for an unknown kind, a negative amount or no settlement. Built on a pure `parse_intervention(kind, settlement_id, amount, home) -> Option<Intervention>` with unit tests. Its UI belongs to a later art pass.

## 8.5 State digest and tests — new file `harness_revitalize_pass7_interventions.rs`
`sim-test/src/worldmodel/digest.rs`: `pub fn state_digest(engine) -> u64`, FNV-1a (as `probe::fingerprint`) over the fingerprint string, then in id or key order: every settlement's food and wood stock, `food_tiles` (sorted keys), `wood_tiles`, `fires`, `fire_blocked_until`, `cold_snap_nights`, `cold_snap_tonight`, `food_regen_acc`, `wood_regen_acc`, `wood_winter_acc`, `wood_winter_cursor`. The fingerprint alone misses stockpiles and deadwood, so prefix identity is checked on the digest.
Tests: one per kind (state change and `applied`); `harness_intervention_prefix_identity` (two engines of the same seed, the intervention at tick t in one: digests equal at every 60th tick ≤ t, the digest taken before the intervention is applied; different at some 60th tick after t whenever `applied > 0`; every kind); `harness_intervention_null_identity` (amount 0: identical digests and fingerprints over three days); `harness_rain_blocks_the_fire`; `harness_cold_snap_counts_nights` (cast before the evening: two nights flagged and the counter back to 0; cast at 23:00: that night counts); `harness_windfall_respects_caps` (max outside winter, the winter cap inside; the cursor unchanged); `harness_interventions_are_deterministic`. Commit, push WIP; run 4.5.

---

# 9. PART 4 — Recorded worlds (`worldsim_wm`)

## 9.1 The recorder
Binary `rust/crates/sim-test/src/bin/worldsim_wm.rs` (new `[[bin]]`) and module `sim-test/src/worldmodel/` (`npy.rs`, `sha256.rs`, `digest.rs`, `schema.rs`, `export.rs`, `schedule.rs`). CLI: `worldsim_wm --scene <scene> --seeds a..b --days N --every TICKS [--agents] [--schedule-dir DIR [--include-factual]] --workers K --out DIR`. Without `--schedule-dir` each seed gives one episode with no interventions. With it, each seed runs every file `s<seed>_<id>.json` in the directory (sorted by name) as one episode, plus a no-intervention episode under `--include-factual`; a seed with no file and no `--include-factual` is an error. An episode runs single-threaded; workers parallelise episodes, and output never depends on the worker count.
Values come from the pure collectors the FFI uses (`collect_agent_snapshot`, `collect_agent_colds`, `collect_resource_snapshot`, `collect_time_snapshot`, `collect_terrain_snapshot`, `collect_building_snapshot`), from the settlement store with the firewood helpers (`wood_target`, `wood_in_hand`, `resources.fires`), and from the probe's own functions for the counts (`probe/pass5.rs` metrics, `probe/long_run.rs` row); widen their visibility to `pub(crate)` where needed and never change what they compute. SHA-256 is written by hand: move the test helper of `sim-test/tests/harness_p14_delta_status_panel.rs:174` into `worldmodel/sha256.rs` (the test then calls it) with the "abc" and empty-string test vectors.

## 9.2 An episode directory: `<out>/<scene>_s<seed>_<id>/` (`id` = `none` or the schedule file's `<id>`)
Rows at ticks 0, every, 2·every … days × ticks_per_day inclusive (with `every` 60, 120 days give 2,881 rows). Row t is the state after t completed ticks, taken before the interventions of tick t (8.3).
- `meta.json`: `schema_version` 1, scene, seed, days, every, ticks_per_day, clock_offset_ticks, the git sha, the schedule (verbatim), every array's dtype and shape, every field (name, dtype, unit, meaning), the sha256 of every array file, the final `fingerprint`. No timestamps or timings: wall time goes to stderr and `timing.json`, which no hash or determinism check reads.
- `global.npy`, float32 [T, G]: tick, day index, minute of day, season (−1 without seasons), day of season, year, winter severity (−1, as 6.3), alive, children, elders, adults (= alive − children − elders), births, deaths by the four causes, cold members, deadwood total, food on bushes total, adults working (the numerator of `adult_work_share`), idle adults (`long_run`'s `idle_adults`), the cumulative tallies (meals from store, ground meals, forage trips, wood trips, wood gathered, urgent wood trips, wood re-targets, cold member-hours, interventions applied), settlements under rain now, cold-snap nights left, cold snap tonight.
- `settlements.npy`, float32 [T, S, F], with `settlement_ids.npy` (int64 [S], first-seen order); a settlement exists while it is in `resources.settlements`, and its row is NaN otherwise: members (roster size), food stock, wood stock, wood target, fire lit, wood in hand, formation x and y, rain ticks left.
- `terrain.npy` (uint8 [H, W], tick 0) and `elevation.npy` (uint8 [H, W]); meta says whether the terrain at the last row equals tick 0's.
- `building_keys.npy` (int64 [B, 4]: id, kind, x, y; first-seen order) and `building_present.npy` (uint8 [T, B]).
- With `--agents`: `agent_ids.npy` (int64 [N], first-seen order), `agent_alive.npy` (uint8 [T, N]) and `agents.npy` (float32 [T, N, A], NaN where not alive: x, y, state tag, seek kind, target kind, target x and y, hunger, thirst, fatigue, carried food, carried wood, life stage, cold, settlement index into `settlement_ids` or −1 — all from `collect_agent_snapshot` and `collect_agent_colds`, joined by `agent_id`; then age in days from `Life::age_ticks`, and lifespan in days from `Life::lifespan_ticks`, marked `hidden` in meta because the game never shows it; NaN without `Life`); `resource_keys.npy` (int32 [R, 3]: kind, x, y; first-seen order) and `resources.npy` (float32 [T, R, 3]: amount, max, present), leaving out tiles whose live value is the infinite sentinel `RESOURCE_SOURCE_INFINITE` (water, camp and shelter sleep tiles never change).
- `latent.npy`, float32 [T, L]: state outside the tables that the next state depends on — `food_regen_acc`, `wood_regen_acc`, `wood_winter_acc`, `wood_winter_cursor` (x and y, −1 when none), `cold_snap_nights`, `cold_snap_tonight`; rain lives in the settlement table. Each field in meta.
- `row_digest.npy` (uint64 [T]): `state_digest` (8.5) of each row.
- `interventions.json`: the records of 8.2, in order.

## 9.3 NPY files
Format 1.0, little-endian, C order, header padded with spaces and a final newline to a multiple of 64 bytes, written by hand (no crate). A Rust test of the header bytes and a Python test that numpy loads every file of the sample.

## 9.4 Determinism
Same arguments, byte-identical files: two runs, and `--workers 1` against `--workers 4`, on 4 seeds × 10 days with and without `--agents` and with one schedule directory (`tools/worldmodel/test_export_determinism.py`, which skips with a message when the binary is not built). The first row of every episode equals a fresh engine of that scene and seed.

## 9.5 Tooling and a sample
`tools/worldmodel/export.sh` (build and run, like `probe.sh`; output only under `WM_DATA_ROOT` or `.harness/evidence/worldmodel/`); `tools/worldmodel/wm_data.py` (numpy loader: `load_episode(path)` returns meta and arrays, verifies the sha256 values, looks fields up by name) with `test_wm_data.py`. Commit a sample of two `game_v13` episodes × 10 days with `--agents` under `docs/worldmodel/sample/` (≤ 5 MB) and `docs/worldmodel/README.md` (schema, export, loading, the latent fields, and a plain statement that the tables plus `latent.npy` are not a complete Markov state: agents' decision timers, memories, social bookkeeping and random streams are not exported). Run the Z6 check (section 11) with the Part 4 binary and commit its result. Commit, push WIP.

---

# 10. PART 5 — Measure and tune v13 (three rounds maximum)

Runs (release): `game_v13` 64 × 10 and 32 × 120 with the Part 0, step 2 flags → `docs/revitalize/probe/p7-5_round<N>_{10day,120day}/`; evaluate with `evaluate_pass7.py`. Screening variants on throwaway, uncommitted runs is allowed; only three recorded rounds count, and every screened variant is listed in METRICS.md with its numbers.

Knobs (only these, all in `game_v13()`): severity weights, each 100–600, summing to 1000; burn per hour mild 1–3, normal 3, harsh 3–5; fall per day mild 6–10, normal 6, harsh 2–6; `cold_hunger_permille` 1500–2000 only if Z2 holds. Every Pass 6 value stays as in `game_v12()`. When targets conflict, section 11's table order decides (Z2 before Z3; 7.4 says which knob moves first). "Best" means: the most targets passed in table order, ties broken by Z3's distance to its ranges. Record every round in METRICS.md; keep the best configuration in `game_v13()`. Commit, push WIP; fast-forward green if the gate is green.

---

# 11. TARGETS (game_v13; table order = priority)

Pooling and night booking as Pass 6 (6.5). Seed-winters: 32 seeds × 4 winters = 128.

| # | target |
|---|---|
| Z1 | determinism 64/64 (10 d) and 32/32 (120 d); legacy pin; v8, v9, v10, v11 and v12 reproduce Pass 6's final summaries on every shared key |
| Z2 | Y2 and Y3 as in Pass 6 |
| Z3 | winters differ: share of seed-winters with no dark night in [0.15, 0.60]; with ≥ 2 dark nights in [0.10, 0.50]; among seed-winters with a dark night, the first one falls on ≥ 3 distinct nights and ≤ 0.70 of them on the seventh; each severity's share of the 128 within ±0.10 of its weight |
| Z4 | trips find wood: winter `wood_retargets` / winter wood trips, median over seed-winters ≤ 0.10 (v12 about 0.51, reported beside it); and the fix does not gather less: median winter wood gathered over the normal-severity seed-winters (Pass 6's parameters) ≥ 0.9 × v12's median (45) |
| Z5 | Pass 6 targets kept: Y4, Y5, Y6, Y7, Y8, Y10, Y11 and Y12 with Pass 6's thresholds |
| Z6 | the god's hand: every test of 8.5 passes, and on 8 seeds (0–7) × 7 kinds, recorded with `worldsim_wm` (`--days 30 --every 60 --include-factual`), one intervention at one-based day 23 12:00 (tick 31,920; the second winter day) with amounts gift and spoil food 60, gift and rot wood 30, rain 13 hours, cold snap 1 night, windfall 10: `row_digest` equal to the factual at every row with tick ≤ 31,920, and different at some later row wherever `applied > 0`; kinds that applied 0 in a seed are listed, not failed |
| Z7 | recorded worlds: 9.4 byte-identity, the loader round-trip, every meta field documented |
| Z8 | ms per agent-tick v13 ≤ 1.2 × v12, back to back on one host, recorder off; the recorder's wall time per 120-day episode reported |
| Y9 | reported only (3.2) |

Z6's schedules come from `tools/worldmodel/make_dataset.py z6` (under `WM_DATA_ROOT/z6/`); `tools/worldmodel/check_z6.py` compares the `row_digest.npy` files and writes `docs/revitalize/probe/p7-z6/z6.json` and `z6.md` (per seed and kind: applied, first differing row, verdict).

---

# 12. PART 6 — Dataset v0 and WM-Bench v0

## 12.1 Dataset (`WM_DATA_ROOT/v0/`, outside git)
All splits use `--every 60`. Days below are one-based display days; the tick of day D at minute m is (D − 1) × 1440 + m − 480.

| split | scene | seeds | days | interventions | train / val / test | `--agents` |
|---|---|---|---|---|---|---|
| A_obs | game_v13 | 0–127 | 120 | none | 0–95 / 96–111 / 112–127 | 0–31, 96–99, 112–119 |
| B_act | game_v13 | 1000–1063 | 120 | 8 random each | 1000–1047 / 1048–1055 / 1056–1063 | 1056–1063 |
| C_cf | game_v13 | 2000–2031 | 35 | factual + 4 variants each | — / 2000–2007 / 2008–2031 | none |
| OOD_v12 | game_v12 | 3000–3031 | 120 | none | test | none |
| OOD_long | game_v13 | 4000–4015 | 240 | none | test | none |

- B_act schedules (`numpy.random.default_rng(seed)`, one file `s<seed>_b.json` per seed, rows sorted by tick then draw order): ticks uniform over the hour boundaries from day 2 00:00 to day 118 23:00; kinds uniform over the seven; amounts uniform on grids — food 20–120 step 10, wood 10–80 step 10, rain 6–24 hours, cold snap 1–3 nights, windfall 5–40 step 5; settlement `home`.
- C_cf variants per seed (files `s<seed>_<variant>.json`, recorded with `--include-factual`): `gift_wood` 60 and `rot_wood` 60 at day 21 18:00 (tick 29,400; the last autumn evening); `rain` 13 hours at day 24 18:00 (tick 33,720); `cold_snap` 2 nights at day 25 18:00 (tick 35,160). Effects over the first winter, each against the same seed's factual: Δ dark nights (nights of the evenings of days 22–28, booked by evening, fire out at 02:00), Δ cold member-hours over those nights, Δ food store at day 29 06:00 (tick 40,200), Δ deaths up to that tick.
- `tools/worldmodel/make_dataset.py` writes the schedules, runs `export.sh` per split in the background and resumably (an episode whose meta hashes verify is skipped), and writes `docs/worldmodel/DATASET_v0.md` and `docs/worldmodel/dataset_v0_manifest.json` (every episode: split, role train/val/test, scene, seed, schedule id, sha256 of each file, size; the total size). No seed appears in two splits or two roles.

## 12.2 Tasks and baselines (`tools/worldmodel/bench_v0.py`, numpy only, settlement level)
Variables V: food stock, wood stock, fire lit, cold members, alive, deadwood total, food on bushes, adults working.
- **T1 next hour:** predict V at t + 1 from rows up to t; train on A train (and A + B train for the action-aware variant), test on A test, B test, OOD_v12 and OOD_long. Metric nMAE = MAE / standard deviation of the variable over the test rows.
- **T2 rollout:** from 06:00 of days 3, 10, 17 … of every test episode, predict 24 h and 168 h ahead autoregressively; nMAE at the horizon.
- **T3 dark night:** at 18:00 of every winter evening of A test, the probability that the fire is out at 02:00; Brier score, AUC (rank based), accuracy at 0.5.
- **T4 counterfactual:** on C_cf test seeds, each variant's Δ targets; MAE and sign accuracy.
- Baselines: persistence; daily naive (the value 24 hours earlier); ridge on lags {1, 2, 3, 6, 12, 24} of V plus hour, season and severity one-hots (λ from {0.1, 1, 10, 100} on val), and for the action-aware variant intervention one-hots × amount over the last 24 hours; T3: the rule "wood at 18:00 < tonight's burn" and a logistic regression on (wood / tonight's burn, members, deadwood total, severity one-hot, night index); T4: zero effect and the per-variant mean effect of C_cf val.

## 12.3 Report
`docs/worldmodel/BENCH_v0.md`: every number by task, baseline, variable and split; the ground-truth effect distribution of each C_cf variant; a short "how to beat this" note for WM-1 (inputs, splits, metrics, the numbers to beat). Sanity checks (investigate within three strikes and report either way): ridge beats persistence on T1 food and wood stock; T3 logistic AUC ≥ 0.70; every C_cf variant changes at least one Δ target in ≥ 25 % of test seeds.

## 12.4 DGX (optional)
If reachable: `rsync` `WM_DATA_ROOT/v0` to `hyunlord@100.70.109.50:/home/hyunlord/work/worldsim-wm-data/v0` and verify the manifest hashes there. Say whether it was done.

## 12.5 Tests
`test_bench_v0.py` on synthetic arrays (nMAE, AUC, ridge, rollout, the C_cf effect computation). Commit, push WIP.

---

# 13. PART 7 — Independent review, final verification, report

1. **Independent review:** a built-in `general-purpose` subagent with fresh context gets `git diff fe60de21..HEAD -- rust/ tools/revitalize/ tools/worldmodel/`, sections 4, 7, 8, 9, 11 and 12, and the final Z table; it reports concrete defects (determinism, v8–v12 invariance, the severity draw and its booking, interventions leaking into fingerprints or changing a run without a schedule, recorder rows or fields that do not match the simulation, leakage between dataset splits, metric errors) with file:line evidence. Fix confirmed defects (two rounds maximum); record every finding and disposition in DECISIONS.md. A configuration change after the review goes through the gate and the final probes again and is named in the report.
2. Full gate on the final WIP head; totals versus Pass 6 (1,941 debug and 1,941 release).
3. Final probes: legacy, v8, v9, v10, v11 and v12 reproduce Pass 6; `game_v13` 64 × 10 and 32 × 120 with census and clock samples; the Z6 check again; v12 and v13 timings back to back for Z8. If `game_v13()` changed after the dataset was recorded, re-record the dataset (or say plainly which head it was recorded on).
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS7.md` in Korean (section 15 format), answering each item of section 14 with its evidence level. Append `## Pass 7` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 7 section (`gh pr edit 510 --body-file …`; skip and say so if `gh` is not authenticated). No new PR.

## 13.1 Merge touch-points for the art run (list them in the report)
`winter_severity` in the time snapshot (a HUD or inspector line "this winter: mild / normal / harsh"); `apply_intervention` (the god-powers UI); harsh winters with two or three dark nights in a row; the live game now builds `game_v13`.

## 13.2 Out of scope
Summer work (Y9); neural or LLM world models and any training; rendering and HUD; new death causes; trade; new settlements; map size; merging into `lead/main`; the art-owned paths.

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 겨울마다 다르다. 어떤 겨울은 불이 한 번도 꺼지지 않고, 어떤 겨울은 마지막 밤에만 꺼지고, 혹독한 겨울은 다섯째 밤쯤부터 두세 밤 꺼진다.
2. 혹독한 겨울에는 숲에 가지가 더 드물게 떨어지고 장작이 더 빨리 준다. 온화한 겨울에는 반대다.
3. 겨울에 장작을 주우러 가던 사람이 도중에 방향을 바꾸는 일이 드물다. 가지 하나를 두고 두 사람이 함께 걸어가지 않는다.
4. 그래도 어떤 시드에서도 굶주림·탈수 사망이 없다.
5. 봄·여름·가을은 Pass 6와 같다. 여름 낮이 한가한 것도 그대로다(이번 범위 밖).
6. 신의 개입은 아직 화면에 버튼이 없다. 개입하지 않으면 게임은 이전과 똑같이 돌아간다.
7. 라이브 게임이 game_v13으로 시작하고 콘솔에 SCRIPT ERROR가 없다.

---

# 15. 구현 완료 보고 (REPORT_PASS7.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
겨울의 혹독함(순수 해시 추첨과 효과), 가진 만큼만 받는 자리, 신의 개입(종류·적용 시점·기록·정확한 재현), 세계 기록기(표·잠재 상태·NPY·결정성), 데이터셋과 벤치마크, v8~v12 고정 방식

### 기능 설명
플레이어 관점과 세계 모델 연구 관점에서 각각 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
Z1~Z8 표: v12(Pass 6 그대로) / v13, 중앙값·p10·p90·최소·최대, Y9 참고값
Winters 표(혹독함·꺼진 밤 수·첫 꺼진 밤·이동·주운 장작·재지정 비율·추위·겨울 끝 식량), 조정 라운드별 수치와 걸러 본 변형

### 데이터셋 v0
분할별 에피소드 수·크기·경로·매니페스트, DGX 복사 여부

### WM-Bench v0
과제(T1~T4)·기준선·분할별 수치, 반사실 효과 분포, 건전성 점검 결과, WM-1이 넘어야 할 숫자

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
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트 두 묶음)

### 정직한 판단
겨울이 이제 날짜가 아니라 사건인가? 이 기록과 벤치마크로 세계 모델을 실제로 시험할 수 있는가? 아직 부족한 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, the paths of `DATASET_v0.md`, `dataset_v0_manifest.json` and `BENCH_v0.md`, and the time spent per Part.
