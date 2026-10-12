# GOAL: WorldSim — Revitalize Pass 10: a world that can diverge — exposure mortality, hidden world variety, and measured seed diversity

Repository `hyunlord/new-world`. Continue the Pass 9 line: `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`, both at `6aa4dba05171cca04c4cf5360611318ac19fd25f` (draft PR #510 into `lead/main`; the last code commit is `8d3ad643`, later commits are docs).
This run is executed by **Claude Code**, unattended, on the Mac. No other run uses the Mac's CPU heavily at the same time (WM-1.1's leftover evaluations, if any, run on the DGX). No art run is active.

This is the second simulation pass of the research track. **It deliberately changes gameplay balance** (the owner decided: cold and hunger may now kill). Its job is to make seeds produce genuinely different villages, so that a learned world model has dynamics to learn rather than a calendar to memorize.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4            # the Pass 4–9 clone, hooks already neutralized
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 6aa4dba0
cp ~/Downloads/claude_goal_worldsim_pass10.md docs/revitalize/PASS10_GOAL.md
tmux new -s pass10
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS10_GOAL.md>
```
The clone's hooks are neutralized as in Pass 9 (`.claude/settings.json` = `{}`, `--skip-worktree`). Never edit, stage or commit it and never clear that flag. The local `.gitignore` edit and `.ignore` belong to unrelated graft tooling; leave them uncommitted.

**This document is `docs/revitalize/PASS10_GOAL.md`.** Commit it unchanged in Part 0. At the start of every Part, after every compaction or pause, and whenever you are unsure of a number, re-read the relevant section.

The owner drives the run with `/goal`: after each turn a small model reads **only the transcript** and decides whether the condition is met. The evidence it asks for (the section 14 report, the H table, the commit list, the `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
Identical to Pass 9's 0.2: never run `install_hooks.sh`; never check out, commit to or push `lead/main` or `main`; the harness pipeline, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS and `.harness/audit|baseline|reviews/` are not used — section 4.5 is the gate; if any hook other than the owner's `/goal` blocks you, record it in DECISIONS.md, push, end with the report; never use `.claude/agents/harness-*` agents — subagents are the built-in `general-purpose` type. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json` — expect no git hooks, a line starting with `S`, `{}`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 10`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5` and background job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.** An honest miss on a target is acceptable; a missing number is not.
6. **Time rule:** record start time and each Part's start and end (KST). Past **11 hours** of working time (pauses for usage limits excluded, recorded), finish the current step, push, go to Part 7.
7. **Three strikes** per failing target: a tuning round that changes knobs counts one strike. After three, keep the best state, record the evidence and a hypothesis.
8. **Drop order when time is short:** the `game_v16_neutral` arm of the diversity pilot (11.3 attribution) → the second fresh pilot (11.5) → the held-out half of Part 4 (keep the 10-day half) → the fork experiment's replicates beyond R = 2 (Part 6). Never drop Parts 0–3, the first diversity pilot (11.1–11.4, except its v16_neutral arm: if that arm is dropped, H7 reports v15 and v16 only) or the fork experiment's placebo gate.
9. **No image generation, no neural-network training.**
10. **Keep bulky output out of context:** probe runs, recordings and logs go to files; read summaries.
11. **Long commands run in the background** (`nohup bash -c '<command>; echo $? > /tmp/p10_<name>.exit' > /tmp/p10_<name>.log 2>&1 &`), polled with short calls; check `pgrep -fl` before starting a job again. Gate logs in `docs/revitalize/verification/p10-<N>/`.

---

# ⚠️ 1. MACHINES
- **The Mac** clone `/Users/rexxa/work/new-world-pass4` only. Never read, write or build in `/Users/rexxa/work/new-world-wm1`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe and recorder `--workers 4`.
- Recordings go under `/Users/rexxa/work/worldsim-wm-data/p10/` (outside git). `v0/`, `v1/` and `p9/` there are **read-only**.
- Python: system `python3` with numpy; no scipy, torch or pandas.
- Headless Godot only (smoke). `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Timing runs (H10) back to back with nothing heavy running (`pgrep -fl 'Godot|cargo|worldsim_probe|worldsim_wm|python3'`).
- **DGX:** not used by this pass.

---

# 2. GIT
- Every commit goes to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.5) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p10-N): <what changed>`. Stage explicit paths only.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`, `docs/worldmodel/**`, `tools/worldmodel/**` — **except** `tools/worldmodel/wm1/**`, `docs/worldmodel/wm1/**` (the WM branch's) and everything v0/v1 (datasets, manifests, `BENCH_v0/v1.md`, `bench_v0.py`, `bench_v1.py`, the committed samples).
- **Must not change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, static tests that read `.gd`/`.tscn`/`.gdshader`, except minimal Rust-source re-pins allowed in 4.3.
- **Merge notes for later (list them in the report, do not act on them):** (a) the art branch (`cd998d38`) added a read-only FFI getter `get_intervention_state` to `rust/crates/sim-bridge/src/ffi/world_node.rs`; do not add a function of that name and keep your `world_node.rs` changes to the constructors, the tallies dictionary and the `Tallies` struct literal in its test (≈ L2899). (b) `scripts/ui/panels/hud_status_panel.gd` line ≈ 234 sums starvation, dehydration and combat deaths; exposure deaths will be missing from that HUD number and a death notification for exposure has no localization key yet — a UI follow-up for the art/UI line, not this pass. (c) `scripts/ui/renderers/death_viz_renderer.gd` ≈ L100 will draw the new death code (4) in the combat colour — same follow-up.

---

# 3. CONTEXT

## 3.1 Why this pass exists (planner's review of Pass 9 and WM-1.1)
- **WM-1.1:** in dataset v0 the A_train calendar mean (CalAbs: the per-row mean trajectory) explains 0.90–0.98 of the variance of food, wood, fire, alive and deadwood on A_test; the between-seed share of variance η² is 0.02–0.09 for those variables. Dataset v1 (`game_v14`) is no better (food η² 0.195, the rest ≤ 0.06). A plain mean trajectory scores T2 168 h nMAE 0.099 on A_test; every learned model and ridge of WM-1 is worse (M1b 0.122, ridge 0.223). An analog forecast (5 nearest training episodes) scores 0.076. **The v0/v1 benchmark cannot distinguish learned dynamics from a memorized calendar.**
- **Pass 9:** resampling all agent noise after a fork moves 32 of 42 villagers within an hour yet changes the dark-night count in 3 of 32 seeds and deaths in none. In the planner's own reproduction every death in 35 days was old age (6–10 per seed); starvation, dehydration and combat were 0.
- **Diagnosis:** the village has no channel that turns a bad night or a lean season into a lasting difference. Cold only speeds hunger (`FirewoodRule::cold_hunger_permille`); food is plentiful so hunger never saturates; births are gated by a supply-aware cap (`BirthRule::Provisioned`, `SustainRule`) that pins the population near 42; children mature in 4 days (`LifeStageRule::child_ticks` 5,760). Every perturbation is pulled back to the same mean trajectory, and every seed starts from the same world recipe (24 founders, 12 bushes, 20 deadwood spots, the same climate weights).
- **Remedy (this pass):** (A) **exposure** — a cold night damages health, and health can reach zero; (B) **world variety** — each seed draws hidden world parameters (founders, food and wood abundance, terrain, climate); (C) **measure** whether seeds now diverge, with WM-1.1's exact diversity metrics, on fresh seeds after tuning is frozen; (D) check that interventions now have consequences (deaths) in a Pass 9-style fork experiment.

## 3.2 Code facts (planner's survey of `6aa4dba0`; verify in Part 0)
- `BodyHealth { hp, max_hp }` (`sim-core/src/components/body_health.rs`), `DEFAULT_MAX_HP` = 100 (verify). `StarvationSystem` (`sim-systems/src/runtime/survival/mod.rs`, priority 139, every tick): damage 0.08 / tick at saturated hunger, 0.12 at saturated thirst; heals 0.05 / tick (3 hp an hour) when both needs < `SAFE_NEED_CEILING` 50; deaths through `despawn_agent(…, DeathReason::…)`, booked in AgentId order under keyed draws.
- `DeathReason` (`sim-core/src/causal/event.rs`): `Starvation`, `Dehydration`, `Combat`, `OldAge` with `as_str` and a numeric code; tallies `deaths_*` in `Tallies`; FFI tallies dictionary in `world_node.rs` ≈ L2770; recorder `global.npy` columns `deaths_*` (`sim-test/src/worldmodel/export.rs` ≈ L131).
- `Cold` marker (`sim-core/src/components/cold.rs`): attached by `FireSystem` (`runtime/settlement/fire.rs`, interval 60) to every member of a settlement whose fire is out, inside the cold window (22:00–06:00) on winter nights or cold-snap nights; `cold_member_hours` counts marked members per hour.
- Life stages: `Child { born_tick }` (child for `child_ticks` = 5,760 ticks), `Elder` (from `elder_age_ticks` 66,240); lifespans mean 83,520 ± 11,520 ticks (`AgingRule`).
- World recipe: `WorldGenConfig::game_v15(seed)` = v11's: 64×64, 24 founders, 12 bushes of capacity 4, forest 0.30 / hill 0.10 / mountain 0.04, 20 deadwood spots of capacity 6 at radius 5–18. Climate: `FirewoodRule::winter_severity_permille` [350, 400, 250] (v14), burns [2, 3, 4], falls [8, 6, 4], night burn step ±2 (25/50/25 %), day fall spread ±2; `SeasonRule::regen_permille` [750, 1250, 1000, 500].
- Keyed draws: `sim_core::rng::keyed_u64(seed, stream, subject, tick, slot, replicate)` (`const fn`), streams in `STREAMS`.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Determinism (unchanged from Pass 9)
All simulation logic in Rust; new collections `BTreeMap`/`BTreeSet`; no order-dependent hash iteration (process agents in AgentId order in any new loop); no transcendental f64 in simulation crates; no new crates. Every new random number is a keyed draw with a new, distinct stream constant added to `STREAMS`. **World-variety draws never use the replicate** (slot/replicate fixed to the values in 7.1), so a noise fork never changes the world.

## 4.2 Versioning — v8 to v15 frozen, v16 new
- `game_v8()` … `game_v15()` (rules and worldgen) keep their behaviour exactly; every new field takes the value that reproduces them bit for bit.
- New in this pass: `WorldRules::game_v16_neutral()` (= v15 + exposure, Part 1), `WorldRules::game_v16(seed)` and `WorldGenConfig::game_v16(seed)` (+ world variety, Part 2), scenes `game_v16_neutral_scene(seed)` (v15 world, v16_neutral rules) and `game_v16_scene(seed)`; the probe and recorder scene names `game_v16_neutral` and `game_v16` wherever `game_v15` appears (`probe/args.rs` scene list and usage, `probe/runner.rs`, `tools/revitalize/probe.sh --help`, `worldmodel/export.rs` `SCENES` and usage). The live game switches to `game_v16` in both `world_node.rs` constructors (`init` with `DEFAULT_GAME_SEED`, `reset_world(seed)`).
- **Enum freeze lifted for exactly one variant:** `DeathReason::Exposure` (`as_str` `"exposure"`, the next numeric code). No other new variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType` or `DeathReason`. Update the exhaustive `match`es on `DeathReason` (6: `event.rs` ≈ L83 and L99, `survival/mod.rs` ≈ L99 and three test files — the compiler finds them) **and the non-exhaustive death counters the compiler will not find:** `sim-test/src/probe/deaths.rs` (classify `reason == Some(DeathReason::Exposure)` as cause `"exposure"` before the saturation checks, ≈ L258; add `"exposure"`/`deaths_exposure` to `cross_check` and `tallies()`, which become 5 entries, ≈ L311–327 — otherwise every v16 probe aborts with "death census mismatch"), `probe/summary.rs` ≈ L232, `probe/metrics.rs` ≈ L159, `worldmodel/schema.rs` GLOBAL (≈ L44), `worldmodel/export.rs` ≈ L131, `tools/revitalize/fork_metrics.py` ≈ L166.
- In `sim-test/src/probe/runner.rs` add both v16 scenes to `scene_engine` (≈ L97) **and** to both `matches!` allow-lists (≈ L180 observable twin check, ≈ L261 trace/fingerprint twin check); a scene missing there silently skips the determinism checks, so H1 would pass without being checked. Add a test that both v16 scenes go through both checks.

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; all Pass 2–9 tests pass unchanged except the allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 10`: exhaustive `match` arms for `DeathReason::Exposure` in tests; struct-literal field additions with legacy values; probe schema tests moving to schema 10; recorder schema tests moving to schema 4 (schemas 1, 2 and 3 still load); Rust-source static assertions pinning code this goal deliberately changes, re-pinned and never weakened. Never change `sim-test/src/probe/fingerprint.rs` or `probe/long_run.rs::trace`.

## 4.4 Tuning discipline
- **Tuning seeds:** 0–63 (10 days) and 0–31 (120 days). **Held-out:** 64–127 (10 days) and 32–95 (120 days). **Diversity pilot:** fresh seeds 20000–20111, never used before Part 5. **Second pilot (only if 11.5 runs):** 21000–21111.
- Knobs may change only in Part 3, judged on tuning seeds. Part 4 (held-out) and Part 5 (pilot) run on a **frozen** commit recorded in METRICS.md before they start. A held-out or pilot miss is reported, not tuned away (except the single, declared variety adjustment of 11.5).

## 4.5 The gate
Copy `tools/revitalize/gate_pass9.sh` to `gate_pass10.sh` (same steps): debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep (0 hits), `gdscript_strict_check.sh`, `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, both Python unittest discoveries.

## 4.6 v15 guard after every behaviour Part
After Parts 1, 2 and 3: re-run `game_v15` 64 × 10 and 32 × 120 with Pass 9's final flags and compare with Part 0's v15 runs; every shared value identical (only `schema_version` and new keys may differ).

---

# 5. PART 0 — Baseline
1. Record `PASS10_BASE_SHA` (`6aa4dba0`), start time, `uname -a`. Commit `PASS10_GOAL.md`.
2. Release probes with Pass 9's recorded flags: `game_v15` 64 × 10 and 32 × 120 against `docs/revitalize/probe/p9-2_v15_10day` / `p9-2_v15_120day` (held-out: `p9-4_v15_heldout_10day` / `_120day`); `game_v14`, `game_v13` and legacy against `p9-6_final_v14_*`, `p9-6_final_v13_*`, `p9-6_final_legacy`. Every shared key identical, timing excepted.
3. Create `harness_revitalize_pass10_frozen.rs`: pin `game_v15` digests for seeds 0–3 after 1,440 and 40,320 ticks (as Pass 9 did for v14), plus a field-by-field literal test of `WorldRules::game_v15()`.
4. Verify 3.2 (constants, priorities, the `DeathReason` match sites, `DEFAULT_MAX_HP`); correct this file's facts in DECISIONS.md where they differ. Commit, push.

---

# 6. PART 1 — Exposure (`game_v16_neutral`)

## 6.1 Rule
`SocietyRules` gains `exposure: Option<ExposureRule>` (`None` in v8–v15):
```rust
/// Pass 10: a cold night wounds; health can reach zero.
pub struct ExposureRule {
    /// HP lost per hour spent `Cold`, by life stage: [child, adult, elder].
    pub damage_per_cold_hour: [f64; 3],
    /// No healing while `Cold` and for this many hours after the last cold hour.
    pub heal_block_hours: u32,
}
```
Start values: `damage_per_cold_hour` = [9.0, 6.0, 9.0], `heal_block_hours` = 2. (Orientation only, an upper bound on healing: a full dark night is 8 cold hours → adult −48, child/elder −72; healing is 3 hp/h only while both hunger and thirst are < 50, at most ≈ 42 hp between 08:00 and 22:00, and in practice less — agents seek food at need 50, cold nights speed hunger ×1.5, and the 50–90 band is a no-heal gray zone. Expect children and elders to die on the second or third consecutive dark night and adults around the fourth; Part 3 tunes against H3, not against this arithmetic.)

## 6.2 Mechanics
- `FireSystem`, after it attaches/removes `Cold` each hour: for every agent marked `Cold` this hour, in AgentId order, `hp -= damage_per_cold_hour[stage]` (stage: `Child` → 0, `Elder` → 2, otherwise 1); if `hp ≤ 0`, the agent dies through `despawn_agent(…, DeathReason::Exposure)` (deaths booked in AgentId order). Record, per agent, the tick of the last `FireSystem` run that marked it `Cold` in a new component `ExposureClock { last_cold_tick: u64 }` (attached only under the rule).
- `StarvationSystem` healing gains one condition under the rule: no heal while `Cold` or while `current_tick < last_cold_tick + 60 + 60 × heal_block_hours` (the marked hour itself plus the block). Starvation and dehydration damage are unchanged.
- `Tallies::deaths_exposure`; `DeathReason::Exposure` everywhere a death reason is counted or named; FFI tallies dictionary key `deaths_exposure` in `world_node.rs`.
- `WorldRules::game_v16_neutral()` = `game_v15()` + `exposure: Some(start values)`. Nothing else changes.

## 6.3 Instruments
- **Probe schema 10:** per seed `deaths_exposure`, the seed's `world_variety` (all eight fields; `null` for other scenes) and its own `winter_severity_permille`; per seed-winter exposure deaths, members at 18:00 of the winter's first day, dark nights, severity — a seed-winter is **[day D 18:00, day D+7 07:00)** for D = 22, 50, 78, 106 (the night begun on day 28 is a winter night under `night_season_from_evening`, and its after-midnight hours fall on day 29), deaths taken by tick from the death census (`deaths.jsonl`, tick + cause); `min_hp` and `injured_members` (hp < max) at each daily 06:00 sample; alive at every daily sample. `probe/summary.rs` builds its `rules` block from the first seed only (≈ L159), so every per-seed quantity (severity weights, starting agents) must come from the per-seed fields, never from that block.
- **Recorder schema 4:** `global.npy` gains `deaths_exposure` (cumulative) and `injured_members`; `agents.npy` (with `--agents`) gains `hp` **appended as the last field** (observable: a player can see a villager's health; field 16 is pinned by an existing test, `export.rs` ≈ L759); `meta.json` gains `"world_variety": {"hidden": true, "values": {...}}` (Part 2; `null` for other scenes) — hidden is per column elsewhere, so mark it inside the block and teach `wm_data`'s observable filter to skip it. `tools/worldmodel/wm_data.py` accepts schema 4 (`SCHEMAS = (1, 2, 3, 4)`); its tests load schemas 1–4. `bench_v0.py`/`bench_v1.py` stay untouched (their death sums ignore exposure; dataset v2 will use a new loader).
- `evaluate_pass10.py` (from `evaluate_pass9.py`): schema 10, the H targets of section 10, comparison mode as before, with `BASELINE_SCENE = "game_v15"` and the timing factor 1.25 (`evaluate_pass9.py` hard-codes `game_v14` and 1.2, ≈ L32 and L91); severity weights and starting agents per seed.

## 6.4 Tests (`harness_revitalize_pass10_exposure.rs`)
`harness_exposure_inert_before_v16` (v15 digests equal Part 0's), `harness_exposure_kills_a_cold_elder` (a constructed settlement with the fire out on a winter night: an elder at 20 hp dies within 3 cold hours with `DeathReason::Exposure`, tallied once), `harness_exposure_blocks_healing` (no hp gain while `Cold` or within `heal_block_hours` after), `harness_exposure_adult_survives_one_night` (an adult at full hp survives one full dark night), `harness_exposure_order_independent` (two worlds with the same agents spawned in different orders give identical deaths and digests), `harness_exposure_only_from_cold` (every `DeathReason::Exposure` death happens on a `FireSystem` run while the agent is marked `Cold`). Commit, push; run 4.6.

---

# 7. PART 2 — Hidden world variety (`game_v16`)

## 7.1 Parameters (`sim-core/src/variety.rs`, new)
```rust
/// Pass 10: the hidden parameters of one world, drawn from its seed.
pub struct WorldVariety {
    pub starting_agents: u32,        // uniform integer 18..=30   (v15: 24)
    pub berry_bushes: u32,           // uniform integer 9..=15    (12)
    pub berry_capacity: u8,          // uniform integer 3..=5     (4)
    pub deadwood_spots: u32,         // uniform integer 14..=26   (20)
    pub deadwood_capacity: u8,       // uniform integer 5..=7     (6)
    pub forest_permille: u32,        // uniform integer 220..=380 (300)
    pub climate: u8,                 // 0 mild-leaning, 1 neutral, 2 harsh-leaning (CLIMATE_PERMILLE)
    pub food_regen_permille: u32,    // uniform integer 850..=1150 (1000): scales SeasonRule::regen_permille
}
```
- Drawn by `WorldVariety::draw(seed)`: parameter i uses `keyed_below(seed, STREAM_WORLD_VARIETY, 0, 0, i, 0, n_i)` (slot = parameter index 0..7, replicate always 0, tick 0); for the climate, n = 1000 and the climate is the index of the drawn value in the cumulative `CLIMATE_PERMILLE` (start [333, 334, 333]). `STREAM_WORLD_VARIETY` is new, distinct and in `STREAMS`. `WorldVariety::NEUTRAL` holds the v15 values (climate 1, regen 1000).
- Climate → `winter_severity_permille`: mild-leaning [500, 350, 150], neutral [350, 400, 250], harsh-leaning [200, 400, 400]. Burns, falls, night step and day spread unchanged.
- `food_regen_permille` multiplies each entry of `SeasonRule::regen_permille` (integer arithmetic, `x * p / 1000`, documented rounding). The birth gate's `annual_mean_supply` then follows automatically; check that its comment's arithmetic is still true for the neutral world.
- `WorldGenConfig::game_v16(seed)` = v15's config with `starting_agents`, `berry_bushes`, `berry_capacity`, `deadwood_spots`, `deadwood_capacity`, `forest_coverage` (= permille / 1000) from the draw; `WorldRules::game_v16(seed)` = `game_v16_neutral()` with the climate and regen from the draw. Founder lifespans, ages and lineage keys continue to come from Pass 9's keyed streams (founder index order unchanged).
- Worldgen accepts every range above (planner's review). Never redraw a world. Count per seed and report: (a) deadwood spots placed < requested (`camp.rs` ≈ L261–268 places fewer silently), (b) berry-repair terrain swaps (`terrain.rs` ≈ L71–127; these already happen for some v15 seeds). Note: the birth gate's supply estimate counts bushes, not `berry_capacity` (`food_economy.rs` ≈ L14–24, L118), so capacity-3 worlds overstate their supply — a starvation risk to watch in H3; if Part 3 shows it, fixing the estimate for v16 (scaling by capacity / 4) is an allowed knob.

## 7.2 Tests (`harness_revitalize_pass10_variety.rs`)
`harness_variety_neutral_equals_v16_neutral` (a `game_v16` world built with `WorldVariety::NEUTRAL` is digest-identical to `game_v16_neutral`), `harness_variety_ranges` (over seeds 0..10,000 every parameter stays in range, every integer value of each range occurs, the climate shares are the configured `CLIMATE_PERMILLE` ± 0.02), `harness_variety_independent_of_replicate` (a noise fork at tick 0 with replicate 3 leaves the drawn variety and the worldgen digest unchanged), `harness_variety_deterministic` (same seed → same variety, same digest at 1,440 ticks). Commit, push; run 4.6.

---

# 8. PART 3 — Tuning (tuning seeds only)
Run `game_v16_neutral` and `game_v16` on the tuning seeds (64 × 10, 32 × 120, `--check-determinism`), evaluate H1–H6 (section 10) against `game_v15` on the same seeds. **Knobs (and their allowed ranges):** `damage_per_cold_hour` child/elder 5–14, adult 3–10; `heal_block_hours` 0–6; the variety ranges of 7.1 may be narrowed or widened by at most one step each (an integer range end ± 2, `CLIMATE_PERMILLE` entries by ± 50 (sum 1000), regen ends ± 50); `SustainRule::utilization` 0.75–0.90 (v16 only). At most three rounds (one strike each); record every round's knobs and H results in METRICS.md. Freeze: commit the final knobs, record the frozen commit in METRICS.md. Commit, push; run 4.6.

---

# 9. PART 4 — Held-out health
On the frozen commit: `game_v16_neutral` and `game_v16` on the held-out seeds (64–127 × 10, 32–95 × 120), H1–H6 against `game_v15` on the same seeds. No knob changes. Y-style timing for H10. Commit, push.

---

# 10. TARGETS (H)

| # | target |
|---|---|
| H1 | determinism 64/64 and 32/32 for both v16 scenes on tuning and held-out seeds; legacy pin; v8–v15 reproduce Pass 9's final summaries on every shared key |
| H2 | **no collapse** (both v16 scenes, every seed of every set, 120 days): one settlement every day; alive ≥ max(8, 0.4 × starting agents) at every daily sample; alive on day 120 ≥ 0.6 × alive on day 21 |
| H3 | **deaths exist and are bounded** (`game_v16`, per seed-winter over the 120-day runs): share of seed-winters with ≥ 1 exposure death in [0.15, 0.60]; median exposure deaths per seed-winter ≤ 2; maximum ≤ 0.35 × members at the winter's start; mean exposure deaths per seed-winter strictly increasing mild < normal < harsh severity; starvation + dehydration: 0 in every seed at 10 days, 120-day total median ≤ 1 and maximum ≤ 6 |
| H4 | exposure matches the night: every exposure death in the death census (`deaths.jsonl`) has a tick inside a cold window (22:00–06:00) of a winter or cold-snap night, and `harness_exposure_only_from_cold` passes |
| H5 | **village rhythm kept** (`game_v16_neutral` vs `game_v15`, comparison mode as Pass 8–9: failing seeds ≤ v15's + 1): Pass 6 Y4, Y5, Y6, Y9, Y10, Y11, Y12 and Pass 8 X3 (b)–(e), X4. Pass 6 Y3's population clauses are replaced by H2 for v16 (report them anyway) |
| H6 | variety realized: over the 128 seeds of tuning + held-out, every parameter's mean within ± 10 % of its range centre and each climate's share within ± 0.10 of its `CLIMATE_PERMILLE` weight; deadwood shortfalls and berry-repair swaps counted and explained |
| H7 | **diversity** (Part 5 pilot, `game_v16`, A_test-like split of 16 seeds after fitting on 96): η² ≥ 0.30 **and** R²_abs ≤ 0.70 for `food_stock`, `wood_stock` and `alive`; all eight V variables reported for v15, v16_neutral and v16 side by side |
| H8 | **interventions have consequences** (Part 6, `game_v16`): placebo identity 100 %; for at least two of {rain, cold_snap, rot_wood} the replicate-mean deaths effect is > 0 in ≥ 0.25 of seeds and its mean over seeds ≥ 0.25 deaths; gift_wood's mean deaths effect ≤ 0 |
| H9 | recorder schema 4 byte identity (two runs; `--workers 1` vs 4); schemas 1–3 still load; Pass 9's fork tests pass for v16 |
| H10 | ms per agent-tick v16 ≤ 1.25 × v15 (both scenes; back to back on one host) |

---

# 11. PART 5 — The diversity pilot (on the frozen commit)

## 11.1 Recording
`worldsim_wm` (schema 4), 120 days, `--every 60`, no `--agents`, seeds **20000–20111** (`--seeds 20000..20112`), three scenes: `game_v15`, `game_v16_neutral`, `game_v16` → `/Users/rexxa/work/worldsim-wm-data/p10/pilot/<scene>/`. Roles: 20000–20095 "train", 20096–20111 "test". Expected ≈ 336 episodes × ≈ 35 s CPU / 4 workers ≈ 50 min; record the real time.

## 11.2 Metrics — WM-1.1's exact definitions
- Copy `tools/worldmodel/wm1/calcore.py` **verbatim** from `origin/codex/worldmodel-2026-10-wip` at `ecf8c316` (`git show ecf8c316:tools/worldmodel/wm1/calcore.py`) to `tools/revitalize/calcore_wm11.py`, with a header line naming the source commit and path; never edit the copied body. Write `tools/revitalize/diversity_p10.py` around it, loading episodes with `wm_data.load_episode` and the eight V variables exactly as `bench_v1.episode_arrays` builds them (import it read-only) plus `deaths_exposure` in the death sum only where deaths are reported.
- **Implementation check first:** run `diversity_p10.py` on dataset v1 (root `/Users/rexxa/work/worldsim-wm-data` — the manifest's paths begin with `v1/`; A_train fit, A_test scored; WM-1.1's own entry point `git show ecf8c316:tools/worldmodel/wm1/diversity_v1.py` is the reference: `Fit` with `K_MAX = 168`, `bench_v1.split_std` per split, `T_TR = 2881`, `calcore.split_diagnostics`) and reproduce WM-1.1's published v1 A_test numbers to ± 0.002: η² food 0.195, wood 0.049, fire 0.039, cold 0.598, alive 0.061, deadwood 0.027, bushes 0.136, work 0.284; R²_abs food 0.768, wood 0.947, fire 0.959, alive 0.932. If they do not reproduce, stop Part 5, find the difference, record it.
- Per scene, on the pilot test split (fit on its train split): R²_abs, R²_year, η² per variable; divergence day (min / median / max, per variable); between-seed spread at 06:00 of days 30, 60, 90, 118; and the calendar-family T2 168 h nMAE at every-hour starts (`bench_v1.rollout_starts(n, 168, True)`, `bench_v1.split_std` of the pilot test split) for CalAbs, apfit_abs, apfit_year, an5 and an5_off (WM-1.1's strongest members), plus plain persistence (`v[s]`). Read them together: η² and R²_abs say the seeds differ; if persistence or apfit_abs is then nearly as good at 168 h as at 24 h, the differences are mostly constant level offsets (for example the founder count) rather than diverging dynamics — say which in the report.

## 11.3 Attribution
`game_v15` vs `game_v16_neutral` isolates exposure; `game_v16_neutral` vs `game_v16` isolates world variety. Report both differences per variable.

## 11.4 Outputs
`docs/revitalize/probe/p10-5_pilot/` (`diversity.json`, `diversity.md`, exact commands). H7 is judged here, once.

## 11.5 One declared adjustment (only if H7 fails and time allows)
If H7 fails, you may make **one** change to the variety ranges only (not exposure), declared in METRICS.md with its reason **before** any new recording, rerun Part 4's held-out H2–H3 for `game_v16` and record a **second fresh pilot** (seeds 21000–21111). Report both pilots; H7 is judged on the second, and the report states that it followed one adjustment.

---

# 12. PART 6 — Consequential interventions (fork experiment, `game_v16`)
Record exactly as `docs/revitalize/probe/p9-3_forks/p6_forks.sh` with `--scene game_v16 --replicates 4` and output under `p10/`: `--agents`, `--include-factual`, the main families (seeds 12000–12031, 35 days, `--every 60`, fork tick 28,800, the five C_cf variants and the one-unit gift plus the factual), the placebo pass (seven amount-0 kinds on 12000–12007) and the unforked runs into `p10/unforked`; pass `--unforked` to `fork_metrics.py`. `fork_metrics.py` gains `--deaths-with-exposure`, which adds `deaths_exposure` at `row_of(28, 360)` to the death count in `targets_of` (≈ L126–133) and in `series()` (≈ L166) — `bench_v1` itself unchanged; test it on a synthetic episode — and `--v15 <p9/forks> --v15-replicates 8` for the comparison below ("differs" = the replicate-mean dark-night effect of a seed differs between v15 and v16). Report the Pass 9 tables (G0a, VRF with the factual/variant noisy-seed counts, mean effect, sign stability, divergence) and, new, per variant: the share of seeds with replicate-mean deaths effect > 0, the mean deaths effect, and the share of seeds whose dark-night effect differs from v15's (Pass 9 data, `p9/forks`, read-only). Outputs `docs/revitalize/probe/p10-6_forks/`. H8 is judged here. Commit, push.

---

# 13. PART 7 — Independent review, final verification, report
1. **Independent review:** a built-in `general-purpose` subagent with fresh context gets `git diff 6aa4dba0..HEAD -- rust/ tools/revitalize/ tools/worldmodel/`, sections 4, 6, 7, 10, 11 and 12, METRICS.md's Pass 10 rounds and the pilot outputs. It checks, with file:line evidence: v8–v15 invariance; every variety draw keyed with replicate 0; no iteration-order dependence in the new loops; exposure deaths only from `Cold`; healing block correct; `DeathReason::Exposure` counted everywhere deaths are counted; the pilot ran on the frozen commit and its seeds were never used before; `calcore_wm11.py` identical to its source; H verdicts follow the rules. Fix confirmed defects (two rounds maximum); record every finding in DECISIONS.md.
2. Full gate on the final WIP head; totals versus Pass 9 (2,029 debug and release).
3. Final probes: legacy and v8–v15 reproduce Pass 9; both v16 scenes on tuning and held-out seeds; timings for H10; if code changed after Part 5 or Part 6, say what and whether their numbers still hold (rerun what is affected).
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS10.md` in Korean (section 14 format). Section 15's in-game checks are for the owner: run the headless smoke only and list them as "주인 확인 필요". Append `## Pass 10` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 10 section (REST API; skip and say so if `gh` is not authenticated).

## 13.1 Out of scope
Dataset v2 recording and its benchmark; frequent-intervention policies in the data; weather resampling inside noise forks and the Bayes-optimal severity posterior; several settlements; any learned model; the HUD/notification follow-up of 2(b); rendering.

---

# 14. 구현 완료 보고 (REPORT_PASS10.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
노출 사망(피해·회복 차단·사망 처리), 세계 다양성(파라미터·키 기반 추첨·기후), v8~v15 고정 방식, 튜닝 라운드, 다양성 측정(WM-1.1 정의 재현 확인 포함), 분기 실험

### 기능 설명
게임과 연구 관점에서 무엇이 달라졌는가? (마을이 겪는 일, 시드마다 다른 세계, 결과가 남는 개입)

### 변경된 파일 목록
(커밋별)

### 측정 결과
H1~H10 표, 튜닝 라운드별 손잡이와 결과, 시드-겨울별 노출 사망 분포(심각도별), v15 / v16_neutral / v16 다양성 표(η²·R²_abs·갈라지는 날·달력 기준선 nMAE), 귀속(노출 대 다양성), 분기 실험(가짜 개입, 사망 효과, VRF)

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 백그라운드 작업

### 확인된 제한사항

### Harness 결과
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트 두 묶음)

### 정직한 판단
이제 시드마다 다른 마을이 되는가? 그 다양성은 역학(사건의 결과)에서 오는가, 숨은 파라미터의 차이에서 오는가? 게임으로서 더 나아졌는가 나빠졌는가? 데이터셋 v2로 넘어가도 되는가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, the paths of `diversity.md` and `fork_metrics.md`, and the time spent per Part.

---

# 15. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)
1. 게임이 game_v16으로 시작하고 콘솔에 SCRIPT ERROR가 없다.
2. 시드를 바꿔 다시 시작(reset)하면 마을 인원, 덤불·나무 수, 숲의 양이 눈에 띄게 달라진다.
3. 혹독한 겨울에 불이 꺼진 밤이 이어지면 노인·아이가 추위로 죽을 수 있고, 봄이 오면 출생으로 인구가 다시 늘어난다.
4. 마을이 통째로 사라지는 일은 없다(120일).
5. HUD의 사망 숫자에는 추위 사망이 아직 들어가지 않는다(알려진 후속 작업).
