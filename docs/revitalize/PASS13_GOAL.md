# GOAL: WorldSim — Revitalize Pass 13: a split the village builds up to — pressure instead of dice — and a measure of how much of a world's future is its path

Repository `hyunlord/new-world`. Continue the Pass 12 line: `codex/revitalize-2026-10` = `1127455b6a9ca5f969501fa5e7ddb53d18f3f434` (Pass 12's last code commit, gate green), `codex/revitalize-2026-10-wip` = `2aeb22c97617dc5b3a51896ffe7a8c9016647e7f` (docs and probe outputs after it; no Rust change). Draft PR #510 into `lead/main`.
This run is executed by **Claude Code**, unattended, on the Mac. A WM run may compute on the DGX at the same time (it only edits and pushes on the Mac); an art run (Codex) may run Godot on the Mac. Keep that in mind for timings (section 1).

This is the fifth simulation pass of the research track. **It changes gameplay** (v19 only): when a village splits is decided by pressure it builds up from crowding and hunger against a hidden threshold of its own, not by a die rolled every morning.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 2aeb22c9
cp ~/Downloads/claude_goal_worldsim_pass13.md docs/revitalize/PASS13_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS13_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`). Never edit, stage or commit it and never clear that flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. **Never stage gate logs or probe outputs of earlier passes**: stage explicit paths only and check `git diff --cached --stat` before every commit.

**This document is `docs/revitalize/PASS13_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part, after every compaction or pause, and whenever you are unsure of a number.

The owner drives the run with `/goal`: a small model reads **only the transcript**. The evidence it asks for (the section 15 report, the J table, the commit list, the `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
As in Pass 12's 0.2: never run `install_hooks.sh`; never check out, commit to or push `lead/main` or `main`; the harness pipeline, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS and `.harness/audit|baseline|reviews/` are not used — 4.5 is the gate; if a hook other than the owner's `/goal` blocks you, record it in DECISIONS.md, push, end with the report; subagents are the built-in `general-purpose` type, never `.claude/agents/harness-*`. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json` — expect no git hooks, a line starting with `S`, `{}`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 13`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5`, background job logs; continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.** An honest miss is acceptable; a missing number is not.
6. **Time rule:** record start time and each Part's start and end (KST). Past **12 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 7. Keep the Mac awake (`caffeinate -dimsu` in the background, killed at the end).
7. **Three strikes** per failing target: a judged tuning round counts one strike. After three, keep the best state, record evidence and a hypothesis.
8. **Mechanisms change only in Part 2; judged rounds (Part 3) change knobs only** (as Pass 12).
9. **Drop order when time is short:** the `game_v19_nofission` 120-day held-out runs (J6 is then judged on tuning seeds only and J1/J2's held-out clauses for that scene are "not measured", say so) → the J6 remainder bisection (6.2) → the `game_v17` arm of the replicate pilot (11.1) → the held-out 120-day runs of Part 4 (keep the 10-day runs; J1/J3/J5 held-out clauses then "not measured"). Never drop Parts 0–3, the replicate pilot for `game_v16`, `game_v18` and `game_v19`, the 35-day forks (12.1) or the macro forks (12.2) — the macro forks are a gated target in this pass.
10. **No image generation, no neural-network training.**
11. **Keep bulky output out of context:** probe runs, recordings and logs go to files; read summaries.
12. **Long commands run in the background** (`nohup bash -c '<cmd>; echo $? > /tmp/p13_<name>.exit' > /tmp/p13_<name>.log 2>&1 &`), polled with short calls; `pgrep -fl` before starting a job again. Gate logs in `docs/revitalize/verification/p13-<N>/`.

---

# ⚠️ 1. MACHINES
- **The Mac** clone `/Users/rexxa/work/new-world-pass4` only. Never read, write or build in `/Users/rexxa/work/new-world-wm1`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe and recorder `--workers 4`.
- Recordings under `/Users/rexxa/work/worldsim-wm-data/p13/` (outside git). `v0/`, `v1/`, `p9/` … `p12/` are **read-only**. Frozen binaries of this pass in `/Users/rexxa/work/p13bin/`.
- Python: system `python3` with numpy only. Headless Godot only (smoke): `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- **Timing runs (J12)** as Pass 12: before each, `pgrep -fl "Godot|cargo|worldsim_|rustc"` and `uptime`; if foreign heavy processes exist, wait (poll every 5 minutes, at most 60 minutes), then run anyway and record them. Run v16, v19, v16 again, back to back (Pass 12's "before and after" design) and judge on the mean, reporting both.
- **DGX:** not used by this pass.

---

# 2. GIT
- Every commit to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.5) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p13-N): <what changed>`. Stage explicit paths only.
- **May change / must not change:** as Pass 12's section 2 (no `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`; no `tools/worldmodel/wm1/**`, `docs/worldmodel/wm1/**`, `bench_v0.py`, `bench_v1.py`, `calcore_wm11.py`, datasets, manifests, samples).
- **Merge notes (list in the report, do not act):** the art branch (Art Pass 7, not yet merged here) carries the read-only getters `get_intervention_state` and `get_society_snapshot` in `world_node.rs`. Do not add functions of those names; keep `world_node.rs` changes to the two constructors (`game_v19`), the tallies dictionary and test literals. **Do not rename or remove** what `get_society_snapshot` reads: `Settlement::{parent, founded_at, member_agents, formation_tile}`, `Founding` and its fields (`site`, `fission_id`, `leader`, `since_tick`), `SimResources::{wild_regions, fission_log, retired_hearths}`, `FissionRecord`'s fields (`fission_id`, `parent`, `site`, `depart_tick`, `members`, `daughter`, `founded_tick`, `cancelled_tick`) and `Tallies::{fissions, fissions_cancelled, remnant_merges}`. New fields are fine. The art run's own fix for a Pass 11 fixture (`InterventionRecord::soaked_wood: 0` in its test) needs nothing from you.

---

# 3. CONTEXT

## 3.1 What Pass 12 established (planner's review of `2aeb22c9`, with evidence)
- **Fixed:** the J6 cause (region bushes fed the camp store through edge workers until foraging switched off; `local_food_availability` recovers 0.71 of the gap: work at 15:00 0.213 vs v16 0.241); rain now costs wood (deaths effect +1.16, wet burn 35.0 vs dry 35.7 units); daughters get their land and water (held-out starvation + dehydration 0, daughters alive at 28 days 56/56); the first fission is spread (display days 12–114, median 38, IQR 38).
- **The new problem — fission is decided by dice.** The daily hazard is a fresh draw keyed with the noise replicate, with split rates per daily check of 0.015 / 0.024 / 0.045 for h terciles 0–18 / 18–27 / 27–66 thousandths (`REPORT_PASS12.md` L70). Waiting time is then nearly memoryless and the state moves h by a factor of three at most. The 120-day macro forks (`docs/revitalize/probe/p12-7_final_macro/macro.md`, `game_v18`, seeds 12000–12031, R = 2, fork at 28,800):

  | comparison | post-fork first-fission day differs | mean abs Δ of that day | mean abs Δ alive at day 118 |
  |---|---:|---:|---:|
  | factual r0 vs r1 (noise only) | 0.688 | 34.5 days | **18.4** |
  | rot_wood vs factual, same replicate | 0.016 | 0.0 | 0.7 |
  | gift_wood vs factual | 0.016 | 2.4 | 0.7 |
  | spoil_food 60 vs factual | 0.000 | 0.0 | 0.03 |

  In Pass 11 (`game_v17`) the comparable design (whole-run first-fission day rather than post-fork) gave noise 0.094 / 1.6 people and rot_wood 0.27. So in v18 an act of the god moves the world's shape **less** than in v17, and a re-roll of the dice moves it ten times more. For the research this is the wrong kind of variety: the divergence is injected randomness no model can forecast and no intervention can steer.
- **UBS is contaminated by threshold effects.** Pass 12's own synthetic test: a nonlinear static world gives UBS 0.970. v17's alive UBS 0.881 is likely mostly "split or not", a threshold of a static capacity, counted as history. ΔUBS (v18 − v17) intervals contain 0 (alive +0.036 [−0.102, 0.163], food +0.034 [−0.063, 0.140]).
- **Other open items:** the winter-fall floor for settled regions adds 1,935 units and the snow cap destroys 2,637 settled-area units (the two mechanisms cancel); J5 child share 0.717 (bound 0.70) and four settlement-winters over 0.35 (max 0.571); J6 remainder (0.213 vs 0.241, both under the 0.25 bound); J7 wood η² 0.236; daughters now survive their first 28 days in 100 % of cases (Pass 11 0.85) — "daughters that can fail" is gone.

## 3.2 Why these changes
1. **Pressure against a hidden threshold (integrate-and-fire).** Keep the hazard terms of Pass 12 (size, crowding, scarcity) but let a settlement **accumulate** them into a pressure P, and split when P reaches a threshold θ that the settlement carries for life, drawn once from the world seed (never from the replicate). This is the standard "random time change" construction: with θ drawn from an exponential law and P the integrated hazard, the first-passage time has (up to the daily discretization) the distribution of the hazard process — but the randomness sits in one hidden number per settlement instead of a die every morning. Consequences: inside one world the timing is decided by the state's history; a noise fork does not re-roll θ; an act of the god that changes food or crowding moves P and so moves the day; and θ is a hidden parameter a world model can infer from how a village behaves (the same role hardiness and the world variety play). References: first-passage / integrate-and-fire models of decision timing (Ratcliff 1978; Gerstner & Kistler 2002), random time change for point processes (Brown et al. 2002), scalar stress and village fission (Johnson 1982; Chagnon 1975; Kohler et al. 2012).
2. **Measure path dependence directly.** Instead of predicting levels from hidden parameters with a regression (which counts every nonlinearity as history), run the same world several times with different noise replicates from the start and split the variance: what differs between replicates of one world is path; what differs between worlds is the world. This is the number the research needs ("how much of a world's future is its path, how much its parameters").
3. **Make the god's acts count in the macro forks** — now a gated target (J11), because a world model of interventions needs interventions that change outcomes beyond noise.

## 3.3 Code facts (planner's survey of `1127455b`; verify in Part 0)
- `FissionHazard` (`sim-core/src/rules.rs` ≈ L419: `size_lo`, `size_hi`, `crowd_lo_permille`, `crowd_hi_permille`, `scarcity_days`, `weight_crowd_permille`, `weight_scarcity_permille`, `max_daily_permille`, `is_valid`); `WorldRules::V18_HAZARD` ≈ L584 (28, 48, 850, 1050, 2, 500, 500, 120); `game_v18_nofission_with` ≈ L562 (exposure [8, 5, 5], `local_food_availability`, `rain_wet_burn`, `area_fall_floor_permille` 1000); `game_v18_with` ≈ L603 (`min_members` 32, `min_age_days` 10, `cooldown_days` 28, `allowed_seasons` [T, T, T, F], hazard).
- `fission.rs`: `hazard_terms(hazard, members, capacity, food, need_per_member) -> (g, q, s, h)` ≈ L164; the draw `keyed_below(world_seed, STREAM_FISSION_HAZARD, sid, tick, 0, replicate, 1000)` ≈ L468 replacing the crowding gate ≈ L456–470; tests ≈ L626–647.
- `rng.rs`: `STREAMS: [u64; 15]` ≈ L80 (`STREAM_FISSION_HAZARD` L67, `STREAM_REGION_FILL_*` L72–77).
- `variety.rs`: `RANGES_V18` ≈ L56, `draw_v18` ≈ L124.
- Winter wood: `resource_regen/mod.rs` `regen_wood_winter_by_area` ≈ L236–330 (floor doc ≈ L241–247, code ≈ L279–293, sharing one accumulator with the normal fall; settled-area losses `Tallies::winter_wood_lost_settled` ≈ L315; cap `winter_deadwood_cap` ≈ L367–388; with `windfall_over_snow` (true since v14) the snow burial runs once at the first winter pulse ≈ L358–369, before the fall; the 2,637 settled-area losses come from the placement ceiling `min(cap, max)` with cap = 1 (`rules.rs` ≈ L743), counted when no spot is below it ≈ L301–316). `WorldGenConfig` is in `sim-systems/src/worldgen/mod.rs` ≈ L94. Settlements are created only in `SettlementSystem::run_formation_scan` (`settlement_system.rs` ≈ L463–531; the camp and every daughter); ids are monotonic and never reused (`sim-engine/src/lib.rs` ≈ L695–699); `Settlement` derives serde and is built only via `new_with_id` (`settlement.rs` ≈ L144).
- Recorder: `worldmodel/export.rs` `SCHEMA_VERSION` 5 (L33); `--fork-tick T --replicates R` needs `--schedule-dir` and `--include-factual` (≈ L654–671); a seed with no schedule file is allowed under `--include-factual` (≈ L723) — so an existing schedule directory with no files gives, per seed, a factual-only family of R members named `<scene>_s<seed>_none_r<k>` (≈ L691, L720, L727–736), written flat into `--out`; `wm_data.load_family` expects one seed per directory, so `paths_p13.py` groups episodes by that name itself. Fork tick ≥ 1 is enforced (≈ L665–668), so worldgen never sees a replicate.
- Probe schema 12 (`probe/summary.rs` ≈ L175). Pass 12 tools: `diversity_p12.py`, `pilot_p12.py`, `fork_extras_p12.py`, `macro_sensitivity.py`, `bisect_p12.py`, `evaluate_pass12.py`.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Determinism (as Pass 9–12)
All simulation logic in Rust; new collections `BTreeMap`/`BTreeSet`; every new loop in AgentId / SettlementId order; no transcendental f64 in simulation crates (no `ln`, `exp`: an exponential threshold is drawn by a fixed-point table, 7.1); no new crates. New random draws use new stream constants added to `STREAMS`. **World draws never use the replicate — the fission threshold is a world draw.** The group draw (who leaves) keeps using the replicate.

## 4.2 Versioning — v8 to v18 frozen, v19 new
- `game_v8()` … `game_v18(seed)`, `game_v18_nofission(seed)` keep their behaviour exactly; every new field takes the value that reproduces them bit for bit. `RANGES`, `RANGES_V18`, `draw`, `draw_v18` unchanged; v19 uses `draw_v18`'s parameters (no new world-variety slots unless indispensable; if needed, a `RANGES_V19` read through `draw_with`).
- New: `WorldRules::game_v19(seed)`, `game_v19_nofission(seed)`, `WorldGenConfig::game_v19(seed)` (may equal v18's), scenes and scene names `game_v19`, `game_v19_nofission` in every list (probe args + its rejection test, runner, `twin_checked`, recorder `SCENES` / `USAGE` / dispatch, `probe.sh --help`, `scenes.rs`). The live game switches to `game_v19` in both `world_node.rs` constructors.
- No new variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType`, `DeathReason` or `BuildingKind`.
- **Recorder schema 6**: `settlements.npy` gains two **hidden** columns `fission_pressure` and `fission_threshold` (thousandths; the live P and θ under v19, 0 and 0 elsewhere; NaN for empty slots) and one observable column `depart_tick` (the tick its founding group left, from `FissionRecord::depart_tick`; −1 for a settlement not founded by fission). Every place to change: `export.rs` (`[f32; 11]` ≈ L97, `[f32::NAN; 11]` ≈ L363, the row builder ≈ L175, the schema pin ≈ L806, hidden-column asserts ≈ L819), `schema.rs` (`[Field; 11]` ≈ L74), `wm_data.py` (`SCHEMAS` ≈ L47 and the hard-coded family check `(3, 4, 5)` ≈ L148 with its message), and the tests pinning schema 5 (`test_export_determinism.py` ≈ L73/86/101/116/131, `test_hidden_columns.py` ≈ L64, `test_wm_data.py` ≈ L131, its `_rewrite_as_schema_four/three` helpers ≈ L143–171 — add a `_rewrite_as_schema_five` step first — and the last-columns pin ≈ L196). Schemas 1–5 still load; the observable filter drops the two hidden columns.

## 4.3 Legacy invariance and allowed test edits
As Pass 12's 4.3 (ledger rows under `## Pass 13`; the frozen literals of `harness_revitalize_pass7/10/12_frozen.rs` gain the new fields at their legacy values — allowed; probe schema 13; recorder schema 6; the `game_v19`-rejected assertion flipped; Rust-source static pins of code this goal changes re-pinned, never weakened; never touch `fingerprint.rs` or `long_run.rs::trace`).

## 4.4 Tuning discipline
- Tuning seeds 0–63 (10 days) and 0–31 (120 days); held-out 64–127 and 32–95; **replicate pilot: fresh worlds 26000–26063** (never used before Part 5, not even in tests — tests use seeds < 1000). Seeds 30000–33999 are reserved for the WM run. Scratch seeds of this pass: 901000–901031 (reference pilot, 5.4) and 901100–901131 (the per-round macro check, Part 3); 900000–900031 were tuned on in Passes 11–12 and are not used.
- Other scratch exploration on seeds ≥ 902000, disclosed in METRICS.md. Knobs only in Part 3; mechanisms only in Part 2; Parts 4–6 on a frozen commit; a fix after freezing reruns everything it affects.

## 4.5 The gate
`tools/revitalize/gate_pass13.sh` = `gate_pass12.sh` with the new pass number.

## 4.6 Guard after every behaviour Part
After Parts 2 and 3: `game_v18`, `game_v18_nofission`, `game_v17`, `game_v16` 64 × 10 and 32 × 120 with Pass 12's final flags identical to Part 0's runs on every shared value (only `schema_version` and new keys may differ).

---

# 5. PART 0 — Baseline and the replicate measure on old worlds
1. Record `PASS13_BASE_SHA` (`2aeb22c9`), start time, `uname -a`. Commit `PASS13_GOAL.md`.
2. Release probes with Pass 12's final flags: `game_v18`, `game_v18_nofission`, `game_v17`, `game_v16` 64 × 10 and 32 × 120 against Pass 12's final summaries (`p12-7_final_*`); legacy and `game_v15` likewise. Every shared key identical, timing excepted.
3. Frozen pins `harness_revitalize_pass13_frozen.rs`: `game_v18(seed)` and `game_v18_nofission(seed)` digests for seeds 0–3 at 1,440 and 40,320 ticks; field literals of `WorldRules::game_v18(0)` and `V18_HAZARD`.
4. **The replicate measure** (`tools/revitalize/paths_p13.py`; tools only):
   - Data: for a scene, W worlds × R replicates, the replicates forked by the noise fork at tick 1 (`worldsim_wm --fork-tick 1 --replicates R --schedule-dir <empty dir> --include-factual`, 120 days, `--every 60`, no `--agents`; replicate 0 equals the unforked world — check it on 2 worlds).
   - For variable j and display day d ∈ {30, 60, 90, 118}: y_{w,r} = mean of v over the rows of day d (one-based days, `calcore_wm11.row_of`); `food_stock` / `wood_stock` are the nansum over settlement slots as `bench_v1.py` ≈ L89–90 does. Within-world variance Wd_j = mean_w var_r(y_{w,r}) (ddof 1); between-world variance Bd_j = var_w(ȳ_w) (ddof 1) − Wd_j / R, floored at 0. **Path share PS_j(d) = Wd / (Wd + Bd)** (NaN when both are 0); also **path spread PSD_j(d) = sqrt(Wd) / S_j**, S_j = std (ddof 0) of v_j over rows ≥ 24 of all W × R episodes of the scene (the `bench_v1` formula, applied also to `settlements_alive`, which `bench_v1` does not list). Differences between scenes are bootstrapped over worlds **paired** across scenes (same world seeds). Report per d and the mean over d ∈ {60, 90, 118}, for `alive`, `food_stock`, `wood_stock`, `settlements_alive`, `deadwood_total`, `food_on_bushes_total`, `adults_working`. Intervals: bootstrap over worlds (1,000×, `default_rng(0)`), 90 %.
   - Tests on synthetic data: replicates identical → PS = 0; worlds identical with independent noise → PS ≈ 1; a nonlinear static map (Pass 12's synthetic case) → PS = 0 (the property UBS lacked).
   - **Now, as the reference:** record `game_v16`, `game_v17`, `game_v18` for worlds 901000–901031 × R = 4 into `p13/paths_ref/` and commit the table (`docs/revitalize/probe/p13-0_paths/`). Expected: v16 near 0 for `alive`, v18 large. These values anchor J7b. Keep UBS (Pass 12's tool, unchanged) as a reported column only.
5. **Diagnosis of the open J5 cases** (`docs/revitalize/probe/p13-0_j5/`): for the four settlement-winters over 0.35 in Pass 12's held-out and tuning-R3 runs and for the child share: settlement (camp or daughter, age, founding season), dark nights, wood per spot, `winter_wood_lost_settled` that winter, members by stage. Commit, push.

---

# 6. PART 1 — The J6 remainder (tests and tools only; drop second)
## 6.1 What is left
With `local_food_availability` the 15:00 adult work share is 0.213 against v16's 0.241. Using Pass 12's ablation runner (`probe/ablation.rs`, `bisect_p12.py`) and its `adult_act_*` keys, compare `game_v18_nofission` with `game_v16` and with `game_v18_nofission` minus each remaining v17/v18 difference (exposure damage, hunger weight, hardiness, regional wood, region deadwood without bushes, wet burn, area floor) on tuning seeds 0–31 × 120 days.
## 6.2 Report
Which activity takes the missing afternoon share, and which difference explains ≥ half of the remaining gap. If one does and a v19 flag can fix it without breaking J5, Part 2 fixes it (one mechanism); else record the evidence and move on. Outputs `docs/revitalize/probe/p13-1_j6/`. Commit, push.

---

# 7. PART 2 — v19 mechanisms

## 7.1 Fission by accumulated pressure
`FissionRule` gains `pressure: Option<FissionPressure>` (v8–v18: None; with it, `hazard`'s terms are reused and its daily draw is **not** made):
```rust
pub struct FissionPressure {
    pub threshold_table: ThresholdLaw,   // Exponential { mean_permille: u32 } or Uniform { lo_permille: u32, hi_permille: u32 }
    pub decay_permille: u32,             // daily decay of P while h = 0 (0 = none)
    pub reset_on_split: bool,            // P of the parent returns to 0 after its split
}
```
- **Threshold θ.** Drawn once per settlement when it forms (the camp's when the first settlement forms), as a world draw: `u = keyed_below(world_seed, STREAM_FISSION_THRESHOLD, sid, 0, k, 0, 1_000_000)` with k = the settlement's number of splits so far (θ is redrawn after each split, still replicate-free). Exponential law in fixed point: θ = mean × E(u) where E(u) = −ln(1 − u/10⁶) is read from a **committed table of 1,025 entries** (E at u = i/1024, thousandths, generated once by a Python script under `tools/revitalize/` and pasted as a `const` array; linear interpolation between entries in integer arithmetic; the top entry capped at E(1 − 1/2048)). Test: the table's mean over a uniform u grid is 1000 ± 5; 10,000 drawn θ have mean within ± 3 % of `mean_permille` and median within ± 3 % of 0.693 × mean.
- **Daily accumulation** (06:00, settlements in id order, Pass 12's gates checked first in their order — season, world cooldown, `max_settlements`, age, own cooldown, `min_members`, departed group, capacity): on a day every gate passes, P += h (Pass 12's `hazard_terms`, thousandths), or, if h = 0, P = max(0, P − `decay_permille`) (an absolute subtraction in thousandths). **On any day a gate fails — including the gates that end the check for every higher id (world cooldown, `max_settlements`, and a lower id splitting that morning, `fission.rs` ≈ L437–445) — P neither grows nor decays.** **Split when P ≥ θ and P > 0** and a site is free; a split blocked by no free site or by group formation (≈ L519–532) keeps P and θ and counts as today. After a split, P = 0 if `reset_on_split`, and θ is redrawn with the new k.
- **k** = the number of `fission_log` records with `parent == sid`, cancelled ones included. θ = max(1, mean × E / 1000). θ is drawn in `run_formation_scan` when the settlement forms (the only production hook; the camp and every daughter) and after each split. `pressure: Some` requires `hazard: Some` (debug_assert).
- **State**: `Settlement` gains `fission_pressure: u64` and `fission_threshold: u64` (0 under v8–v18; serde default). Recorder schema 6 writes both as hidden columns (4.2).
- v19 start values: Exponential with `mean_permille` 1200 (with typical h about 1.7 × Pass 12's 0–66 under the raised ceiling, a median wait of roughly 11–33 eligible days), `decay_permille` 10, `reset_on_split` true; the hazard terms of `V18_HAZARD` but `max_daily_permille` 200 and `weight_scarcity_permille` 650 / `weight_crowd_permille` 350 (so food matters more than the static capacity); fission gates as v18.
- Instruments (probe schema 13): per settlement per eligible day: P, θ, h, g, q, s; per settlement the day P first exceeded half of θ; per fission the P and θ at the split.
- Tests (`harness_revitalize_pass13_fission.rs`): `harness_pressure_inert_before_v19`; `harness_threshold_is_world_draw` (replicates 0 and 3 forked at tick 1 give identical θ for the same settlement; θ differs across 8 seeds); `harness_threshold_table` (the table checks above); `harness_pressure_accumulates_and_splits` (a constructed settlement with constant h splits on the first day P ≥ θ, not before); `harness_pressure_moves_with_food` (the same world with food removed at day 20 splits no later on ≥ 6 of 8 seeds and earlier on ≥ 3; s = 0 whenever food ≥ members × adult need × `scarcity_days`, ≈ 154 for 32 members, so pick a removal that reaches the scarcity term and say which); `harness_pressure_order_independent` (spawning agents and forming settlements in another order changes neither θ nor any split day); `harness_v18_hazard_unchanged`.

## 7.2 The settled-area floor is not thrown away
The floor already runs after the burial; its units are lost because they share the normal fall's accumulator and its placement ceiling `min(cap, max)` with cap 1. Under v19 (`FirewoodRule::floor_separate: bool`, v8–v18 false): a settled area's extra (floor − normal fall) goes to a separate per-area accumulator whose units are placed with ceiling `max` (not `min(cap, max)`); the normal fall keeps the cap. New tallies `area_floor_placed` and `area_floor_lost_at_max`. Test: under v19 with free spots, `area_floor_placed` > 0 and no floor unit is lost to the cap; v18's tallies unchanged.

## 7.3 J5 and J6 fixes
Only what Part 0.5 and Part 1 point to, one mechanism each at most, behind v19 flags, with tests. Otherwise none.

## 7.4 Daughters that can fail
No mechanism: J4 gains an upper report (section 10) and tuning may use the knobs of section 8. Do not add a mechanism whose only purpose is to kill daughters.

## 7.5 Instruments and evaluation
`evaluate_pass13.py` from `evaluate_pass12.py` (baselines: `game_v18` for J3–J5, `game_v16` for J6); `macro_sensitivity.py` extended with the signed post-fork shift per intervention (with a test). Commit, push; run 4.6; fast-forward if green.

---

# 8. PART 3 — Tuning (tuning seeds only)
`game_v19` and `game_v19_nofission`, 64 × 10 and 32 × 120, `--check-determinism`; J1–J6 and J9 on the tuning seeds (J7, J8 are judged in Parts 5–6), **and the macro check of J11 on scratch seeds 901100–901131** (120 days, R = 2, the three interventions of 12.2) in every judged round, because J11 is what this pass is about. **Knobs:** `mean_permille` 600–3000 (or a uniform law with lo/hi in that range), `decay_permille` 0–50, `max_daily_permille` 80–300, the two weights (sum 1000), `size_lo` 18–32, `size_hi` 32–56, `scarcity_days` 1–4, `min_members` 22–36, `cooldown_days` 21–42; exposure damage child/elder 5–10, adult 3–7; `area_fall_floor_permille` 0–1000. At most three judged rounds; record knobs and results. Freeze, copy binaries to `p13bin/`. Commit, push; run 4.6.

# 9. PART 4 — Held-out health
On the frozen commit: both v19 scenes on held-out seeds; J1–J9; timing for J12. No knob changes. Commit, push.

---

# 10. TARGETS (J)

| # | target |
|---|---|
| J1 | determinism 64/64 and 32/32 for both v19 scenes on tuning and held-out seeds; legacy pin; v8–v18 reproduce Pass 12's final summaries on every shared key |
| J2 | no collapse (Pass 12's J2, both v19 scenes, every 120-day seed) |
| J3 | **fission spread** (`game_v19`, tuning and held-out): share of seeds with ≥ 1 fission in [0.50, 0.90]; first-fission display day IQR ≥ 14, median in [20, 80], no day > 0.25 of first fissions; ≥ 2 settlements alive at day 120 in [0.30, 0.85]; cancellations ≤ 0.25 of attempts |
| J4 | daughters alive 28 days after founding ≥ 0.70; **reported**: the share dissolved or merged within 120 days and within the first winter, by founding season (Pass 12: 1.00 alive, 0 starvation — say whether daughters can still fail) |
| J5 | Pass 12's J5 unchanged (adult share [0.15, 0.50], child ≤ 0.70, per settlement-winter maximum ≤ 0.35 of members at the winter's start, the H3 bounds, starvation + dehydration 0 at 10 days, 120-day median ≤ 1 and max ≤ 6) |
| J6 | village rhythm kept (`game_v19_nofission` vs `game_v16`, Pass 12's J6 mode) |
| J7 | **diversity with paths** (Part 5 replicate pilot): (a) on replicate 0, η² ≥ 0.30 and R²_abs ≤ 0.70 for `food_stock`, `wood_stock`, `alive` (test role); (b) **path share** PS (mean over days 60/90/118) for `alive` in [0.10, 0.50] and for `food_stock` in [0.10, 0.60], **and** PS(`alive`) of v19 below that of v18 on the same worlds (bootstrap 90 % interval of the difference below 0) |
| J8 | interventions have consequences (Part 6, `game_v19`, 35 days): placebo identity 100 %; rain mean deaths effect ≥ 0 and dark-night effect > 0; ≥ 2 of {rain, cold_snap, rot_wood} with deaths effect > 0 in ≥ 0.25 of seeds; gift_wood ≤ 0 |
| J9 | regions realized (Pass 12's J9) |
| J10 | recorder schema 6 byte identity (two runs, `--workers 1` vs 4) for both v19 scenes; schemas 1–5 load; fork tests pass for v19 |
| J11 | **the god moves the timing more than the dice** (12.2, `game_v19`; "post-fork fission" = the first fission whose group **departed** after the fork, from schema 6's `depart_tick`; a fission in one run and none in the other counts as differing, none in both as equal; a mean over pairs needs n ≥ 12, else "not measured"): (a) noise only: share of seeds whose post-fork first-fission day differs by **more than 3 days** ≤ 0.35, mean abs Δ of that day ≤ 7 days (Pass 12: 34.5), and mean abs Δ alive at day 118 ≤ 5 (Pass 12: 18.4); (b) `spoil_food` 60: signed post-fork shift < 0 (earlier) in ≥ 0.60 of the seed-replicates where both split after the fork, `gift_food` 60: shift > 0 in ≥ 0.60 (ties count against; also report the share with shift ≠ 0); (c) noise_c = share of seeds whose r0 and r1 differ in day-118 settlement count or in post-fork day by more than 3 days; variant_c = the same rule per (seed, replicate) against the same-replicate factual; pass if variant_c ≥ noise_c + 0.10 for at least two of `rot_wood`, `spoil_food`, `gift_food` |
| J12 | ms per agent-tick ≤ 1.3 × `game_v16` (mean of the before/after design, section 1) |
| J13 | the J6 remainder report delivered (Part 1) |

---

# 11. PART 5 — The replicate pilot (frozen commit)

## 11.1 Recording
`worldsim_wm` (frozen, schema 6), fresh worlds **26000–26063**, 120 days, `--every 60`, no `--agents`, `--fork-tick 1 --replicates 4 --schedule-dir <empty> --include-factual`, scenes `game_v16`, `game_v18`, `game_v19`, and `game_v17` (drop third) → `/Users/rexxa/work/worldsim-wm-data/p13/pilot/<scene>/`. Roles for the replicate-0 metrics: 26000–26047 train, 26048–26063 test.

## 11.2 Metrics
- On replicate 0: Pass 11/12's diversity block (η², R²_abs, R²_year, settlement counts, calendar-family and persistence nMAE at 24 h and 168 h) — first the Pass 10 implementation check (v1 A_test, WM-1.1's 12 numbers ± 0.002).
- On all replicates: PS and PSD (5.4) per scene, with intervals; the difference v19 − v18 and v19 − v17 per variable; first-fission day per world and replicate (its within-world spread vs between-world spread, the same decomposition).
- **Identification:** the correlation across worlds of the first-fission day with θ (hidden, from the recorder) and with early scarcity (mean of s over the first eligible week): report both (θ should matter; state should matter).
- UBS (Pass 12's tool) as a reported column only.

## 11.3 Outputs
`docs/revitalize/probe/p13-5_pilot/` (`paths.json`, `paths.md`, `diversity.json`, exact commands). J7 is judged here, once.

---

# 12. PART 6 — Forks (frozen commit, `game_v19`)
## 12.1 Consequences (35 days)
Pass 12's `p12_forks.sh` and `fork_extras_p12.py` adapted (`--scene game_v19`, data root `p13/`, frozen binary and `WORLDSIM_GIT_SHA`, schedules regenerated into `p13/schedules`), seeds 12000–12031, the five C_cf variants, the one-unit gift and the factual, R = 4, placebo pass, unforked runs. J8 judged here.
## 12.2 Macro (120 days; gated, J11)
Seeds 12000–12031, R = 2, fork tick 28,800, the factual plus `rot_wood` and `gift_wood` (as Pass 12) and `spoil_food` 60 and **`gift_food` 60** at tick 29,400 on home (write those schedule files explicitly; record them). Compare the first fission founded after the fork. Report Pass 12's columns plus the signed shift per intervention, and beside it the same table for `game_v18` recorded fresh into `p13/macro_v18/` with the same five arms (Pass 12's recordings have no `gift_food`; the schema-6 recorder writes `depart_tick` for v18 too, so both tables use the same post-fork rule). Outputs `docs/revitalize/probe/p13-6_macro/`. Commit, push.

---

# 13. PART 7 — Independent review, final verification, report
1. **Independent review** (fresh `general-purpose` subagent, two rounds maximum): `git diff 2aeb22c9..HEAD -- rust/ tools/revitalize/ tools/worldmodel/`, sections 4, 7, 10, 11, 12, METRICS.md's Pass 13 rounds. With file:line evidence: v8–v18 invariance; θ is a world draw (no replicate anywhere in its key), redrawn only per split; the threshold table (spot-check 5 entries by hand); P accumulates only on eligible days and decays only when h = 0; no transcendental math; schema 6 hidden columns dropped by the observable filter; PS computed as 5.4 says (ddof, the W/R correction, floor 0); J11's comparisons use post-fork fissions; J verdicts follow the rules.
2. Full gate on the final WIP head; totals versus Pass 12 (2,103 debug and release).
3. Final probes: legacy and v8–v18 reproduce Pass 12; both v19 scenes on tuning and held-out; timings for J12; rerun whatever a post-freeze fix affects.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS13.md` in Korean (section 15 format); section 14's in-game checks listed as "주인 확인 필요" (smoke only). Append `## Pass 13` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 13 section (REST API).

## 13.1 Out of scope
Temperament and personality beyond hardiness; family and kinship links; trade or conflict between settlements; dataset v2 itself; weather resampling in forks; any learned model; UI and art (merge notes only).

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)
1. 게임이 game_v19로 시작하고 콘솔에 SCRIPT ERROR가 없다.
2. 같은 시드로 두 번 새 게임을 해도 마을이 갈라지는 날이 같다(이제 아침마다 주사위를 굴리지 않는다).
3. 식량 썩히기를 몇 번 쓰면 마을이 더 일찍 갈라지고, 식량을 내려 주면 늦게 갈라진다.
4. 딸 마을이 대부분 첫 겨울을 넘기지만, 가끔은 흩어지거나 크게 줄어든다.
5. 120일 동안 모든 마을이 통째로 사라지는 일은 없다.

---

# 15. 구현 완료 보고 (REPORT_PASS13.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
복제본 분해 지표(PS·PSD)와 옛 세계 기준값, J5 진단, J6 잔여 분석, 압력 누적 분열(문턱 추첨·고정소수점 지수표·누적·감쇠·재추첨), 눈 상한 뒤 바닥 장작, 스키마 6 숨은 열, v8~v18 고정, 튜닝 라운드(손잡이만, 매 라운드 거시 점검), 복제본 파일럿, 분기 실험, 거시 민감도

### 기능 설명
게임과 연구 관점에서 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
J1~J13 표, 튜닝 라운드별 손잡이·결과(거시 점검 포함), 첫 분열일 분포(v17·v18·v19), 경로 비율 표(v16/v17/v18/v19, 변수·날짜별, 구간), 분열일의 세계 안·세계 간 분해와 θ·초기 부족과의 상관, 거시 민감도(v18 대 v19: 잡음 대 개입, 부호 있는 이동), 딸 마을 생존·실패, 단계별 노출 사망

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 백그라운드 작업, 타이밍 측정 때 다른 작업 유무

### 확인된 제한사항

### Harness 결과
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트 두 묶음)

### 정직한 판단
이제 분열 시점은 세계의 상태와 역사가 정하는가? 신의 개입이 잡음보다 크게 세계의 모양을 움직이는가? 세계의 미래 가운데 경로가 차지하는 몫은 얼마이고, v16~v19에서 어떻게 변했는가? 데이터셋 v2를 v19로 가도 되는가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, the paths of `paths.md`, the J6 and J5 reports, `fork_metrics.md` and the macro report, and the time spent per Part.
