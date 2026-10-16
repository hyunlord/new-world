# GOAL: WorldSim — Revitalize Pass 12: a split that comes when the village is under pressure, daughters that can live, rain that hurts, and a diversity measure that tells levels from histories

Repository `hyunlord/new-world`. Continue the Pass 11 line: `codex/revitalize-2026-10-wip` = `codex/revitalize-2026-10` = `31559232e0a24af12d29f490d440d1c5ee5f3661` (Pass 11 final; gate green on its last code commit `e6dedd34`, later commits are docs and probe outputs). Draft PR #510 into `lead/main`.
This run is executed by **Claude Code**, unattended, on the Mac. At the same time a WM run (WM-1.3) works in `/Users/rexxa/work/new-world-wm1` and computes **on the DGX** (it only edits and pushes on the Mac), and an art run (Art Pass 7, **Codex**) may work in `/Users/rexxa/work/new-world-art-prod` and run Godot on the Mac. Keep that in mind for timings (section 1).

This is the fourth simulation pass of the research track. **It changes gameplay** (v18 only): the moment a village splits becomes a daily chance that rises with crowding and hunger, daughter villages get the land they were promised, and rain costs wood instead of saving it. The owner approved these directions.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 31559232
cp ~/Downloads/claude_goal_worldsim_pass12.md docs/revitalize/PASS12_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS12_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`). Never edit, stage or commit it and never clear that flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. **Never stage gate logs or probe outputs of earlier passes**: stage explicit paths only and check `git diff --cached --stat` before every commit.

**This document is `docs/revitalize/PASS12_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part, after every compaction or pause, and whenever you are unsure of a number.

The owner drives the run with `/goal`: a small model reads **only the transcript**. The evidence it asks for (the section 15 report, the J table, the commit list, the `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
As in Pass 11's 0.2: never run `install_hooks.sh`; never check out, commit to or push `lead/main` or `main`; the harness pipeline, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS and `.harness/audit|baseline|reviews/` are not used — 4.5 is the gate; if a hook other than the owner's `/goal` blocks you, record it in DECISIONS.md, push, end with the report; subagents are the built-in `general-purpose` type, never `.claude/agents/harness-*`. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json` — expect no git hooks, a line starting with `S`, `{}`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 12`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5`, background job logs; continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.** An honest miss is acceptable; a missing number is not.
6. **Time rule:** record start time and each Part's start and end (KST). Past **12 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 7. Keep the Mac awake (`caffeinate -dimsu` in the background, killed at the end).
7. **Three strikes** per failing target: a judged tuning round counts one strike. After three, keep the best state, record evidence and a hypothesis.
8. **Mechanisms change only in Part 2; judged rounds (Part 3) change knobs only.** Pass 11 changed three mechanisms inside its judged rounds and could not attribute its J changes (review M5). If a judged round shows a mechanism is wrong, record it, finish the rounds with knobs, and list the mechanism change as follow-up work — do not slip it in.
9. **Drop order when time is short:** the 120-day macro-sensitivity forks (12.2) → the 120-day runs of Part 4 (keep the 10-day runs; J1/J3/J5 held-out clauses are then reported "not measured") → the `game_v18_nofission` pilot arm (11.3 attribution) → the forward half of the J6 bisection (6.2, keep the reverse half). Never drop Parts 0–3, the daughter diagnosis (5.3), the pilot (11.1 except its `game_v18_nofission` arm, 11.2, 11.4) or the 35-day fork experiment (12.1).
10. **No image generation, no neural-network training.**
11. **Keep bulky output out of context:** probe runs, recordings and logs go to files; read summaries.
12. **Long commands run in the background** (`nohup bash -c '<cmd>; echo $? > /tmp/p12_<name>.exit' > /tmp/p12_<name>.log 2>&1 &`), polled with short calls; `pgrep -fl` before starting a job again. Gate logs in `docs/revitalize/verification/p12-<N>/`.

---

# ⚠️ 1. MACHINES
- **The Mac** clone `/Users/rexxa/work/new-world-pass4` only. Never read, write or build in `/Users/rexxa/work/new-world-wm1`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start (a Godot or cargo process from the art clone is not yours).
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe and recorder `--workers 4`.
- Recordings under `/Users/rexxa/work/worldsim-wm-data/p12/` (outside git). `v0/`, `v1/`, `p9/`, `p10/`, `p11/` are **read-only**. Frozen binaries of this pass outside the clone in `/Users/rexxa/work/p12bin/` (as Pass 11's `p11bin/`).
- Python: system `python3` with numpy only. Headless Godot only (smoke): `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- **Timing runs (J12)** back to back with nothing heavy running: before each timing run check `pgrep -fl "Godot|cargo|worldsim_"` for processes not started by you; if any exist, wait (poll every 5 minutes, at most 60 minutes), then run anyway and record `uptime` and the foreign processes beside the numbers. Timing ratios are judged only between runs made back to back under the same conditions.
- **DGX:** not used by this pass.

---

# 2. GIT
- Every commit to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.5) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p12-N): <what changed>`. Stage explicit paths only.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`, `docs/worldmodel/**`, `tools/worldmodel/**` — **except** `tools/worldmodel/wm1/**`, `docs/worldmodel/wm1/**`, the v0/v1 datasets, manifests, `BENCH_v0/v1.md`, `bench_v0.py`, `bench_v1.py`, `calcore_wm11.py` and the committed samples.
- **Must not change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, static tests that read `.gd`/`.tscn`/`.gdshader`, except minimal Rust-source re-pins allowed in 4.3.
- **Merge notes (list in the report, do not act):** the art branch carries `get_intervention_state` (Art Pass 6) and, from Art Pass 7, a read-only getter `get_society_snapshot` in `world_node.rs`. Do not add functions of those names; keep `world_node.rs` changes to the two constructors (`game_v18`), the tallies dictionary and test literals. **Do not rename or remove** what that getter reads: `Settlement::parent`, `Settlement::founded_at`, `Settlement::member_agents`, `Settlement::formation_tile`, the `Founding` component and its fields (`site`, `fission_id`, `leader`, `since_tick`), `SimResources::wild_regions`, `SimResources::fission_log` with `FissionRecord`'s fields (`fission_id`, `parent`, `site`, `depart_tick`, `members`, `daughter`, `founded_tick`, `cancelled_tick`), `SimResources::retired_hearths` and `Tallies::{fissions, fissions_cancelled, remnant_merges}`. New fields are fine.

---

# 3. CONTEXT

## 3.1 What Pass 11 established (planner's review, with an independent reproduction)
- **Fission works but is pinned in time.** On held-out seeds 72 % of worlds split, 64 % hold ≥ 2 villages at day 120, 85 % of daughters live 28 days. But the first fission is the daughter's formation on probe day 28 (display day 29) in 32 of 46 held-out worlds (IQR 1 day). The planner's reproduction on the pilot test seeds: a world splits in its first year iff its population at day 29 is ≥ 33 (≤ 31 never splits; two worlds at 49–50 did not split because they were below capacity). Cause: seasons last 7 display days (spring 1–7, summer 8–14, autumn 15–21, winter 22–28); the birth gate holds every village at its capacity from about day 14; the check needs spring or summer (`fission.rs` L394) and age ≥ 14 days, so the first eligible day is day 29, where the static capacity alone decides. **Whether** a world splits is set at worldgen; only what happens after differs.
- **Diversity:** pilot (seeds 22000–22111) game_v17 test η² food 0.672, wood 0.269, alive 0.908 (reproduced). Fission adds food +0.24 and wood +0.20 η². 120-day macro forks: rot_wood changes the settlement count or first-fission day in 0.27 of seed-replicates vs 0.094 for noise alone.
- **dyn-η² does not measure what it was meant to.** It subtracts each episode's level over days 3–10 and computes η² after day 11. For `alive` it stays 0.79–0.999 in every scene (v15 included), because later levels are set by each world's static capacity, and as a ratio it inflates when the remaining variance is tiny (v15 alive 0.79 with almost no variance). A measure in absolute units, against what the hidden parameters already explain, is needed (11.2).
- **Rain still saves lives** (J8): the 30 % soak (≈ 10–18 wood) is less than the 13 hours of burn it saves (≈ 40–50). Mean deaths effect −0.36, dark-night effect +0.70.
- **Daughters starve or freeze** (J5): starvation + dehydration max 9 at 120 days; one winter killed 0.48 of a world's members (every case a harsh winter with 4–5 dark nights, deaths in a daughter on 2–3 consecutive nights; e.g. tuning seed 24: 35 deaths, 27 in a daughter of 50). Region bushes realized 1,546 of 2,292 requested (0.67): the bush filter needs a Forest tile with a Plain neighbour inside a ring of radius 2–6 near the map edge (`worldgen/camp.rs` L363–372); 93 of 128 worlds fell short. A daughter has 8–14 deadwood spots against the camp's 14–26.
- **The afternoon work regression (J6) is unexplained.** `game_v17_nofission` adult work share at 15:00 (Pass 6 Y9 `work_1500`) is 0.116 against `game_v16` 0.241 on held-out seeds (both under 0.25; v15 0.417, v16_neutral 0.388 on the same seeds). Pass 11 tied most of it to the wild regions in a scratch run, moved region resources beyond the camp's forage and wood radii (round 3), and the regression stayed.

## 3.2 Why these changes
1. **Fission as a hazard.** In ethnography and ABMs a village splits when scalar stress (group size beyond what consensus can carry; Johnson 1982 "Organizational structure and scalar stress"; Dunbar's ≈ 50 layer) and resource stress build, and the timing of the split is contingent (Chagnon 1975; Kohler et al. 2012 VEP). A daily probability that rises with size, crowding against capacity, and a shortfall in the store turns "whether and when" into a state-dependent process: two worlds with the same capacity split at different times because their stores ran differently, and an act of the god (spoiling food, gifting wood) shifts the hazard. The draw is keyed with the noise replicate (as the group draw already is), so a fork can also resample when; the macro forks measure how much of the timing is noise and how much is state.
2. **Daughters get their land.** A region that delivers two thirds of its promised bushes is a broken promise of worldgen, not a game choice. Fill the shortfall with looser candidate tiers.
3. **Rain costs wood.** The fire is kept fed while it rains; the wood burns wet and gives no warmth. Rain then always costs at least what a dry night costs, plus the soak.
4. **A measure for "histories diverge".** The research question of dataset v2 is whether worlds differ by their history and not only by their hidden parameters. Measure the between-world spread that the hidden parameters and the early level do **not** predict, in units of the variable's own spread.

## 3.3 Code facts (planner's survey of `31559232`; verify in Part 0)
- `FissionSystem` (`sim-systems/src/runtime/settlement/fission.rs`, priority 148, interval 1; daily check at `CHECK_MINUTE` 360 = 06:00, L543): `check` L392; season gate L394 (`season > 1` → no fission); world cooldown L401 and `max_settlements` L406 (both `return`, ending the check for every higher id), age, own cooldown, `min_members`, departed group L411–416; capacity L418; `sustain_capacity` L136 (= `floor(utilization × annual_mean_supply / adult_need_per_day)`); crowding L419; site choice L423–434; group draw with `noise_replicate_for(tick)` L439.
- `FissionRule` (`sim-core/src/rules.rs` L361) and `game_v17_with` L470 (min_members 32, crowding 1.0, group_share 0.35, 6–14, 4 adults, cooldown 42, min_age 14, site_clearance 16, stock_share, max 4, timeout 2, remnant 2). `game_v17_nofission_with` L444 (exposure [9, 5, 6], hunger weight 0.5, hardiness on, heal block 2, `rain_soak_permille` 300, `regional_wood` true).
- Rain: `sim-systems/src/runtime/intervention.rs` L62–80 (sets `fire_blocked_until`, soaks under `rain_soak_permille`); `fire.rs` L109–119 (a raining settlement's fire is not lit and **withdraws nothing**), cold marking L122–139 (Founding agents excluded from the parent's cold).
- Regions: `worldgen/camp.rs::regions` L321 (centres at Chebyshev 22–28 from the camp hearth, separation 20, ring 2–6, `beyond_camp` filter, bush filter Forest + Plain neighbour, deadwood Forest then Plain, spacing 2); constants L285–293; `WildRegion { centre, bushes, deadwood, water_distance }` recorded in `SimResources::wild_regions`.
- Variety: `sim-core/src/variety.rs` `RANGES` L38 (shared by v16 and v17: never edit it), `draw_with(seed, ranges, climate)`, slots 0–10, `without_regions`.
- RNG streams: `sim-core/src/rng.rs`, `STREAMS: [u64; 12]` L65.
- Probe scene lists: `probe/args.rs` usage L45, `matches!` L69, error L70, and its test L145 asserts `game_v18` is **rejected** (flip it: allowed edit, ledger row); `probe/runner.rs` `scene_engine` L117–118, `twin_checked` L378 (a scene missing there silently skips the determinism twin), its test L390; recorder `worldmodel/export.rs` `SCENES` L36–37, dispatch L53–54, `USAGE` L596, `SCHEMA_VERSION` 5 L33; `scenes.rs` v17 scenes L106–133.
- Interventions target `"home"` = the lowest settlement id.
- Winter deadwood by area: `resource_regen/mod.rs` `regen_wood_winter_by_area` L236–275 — the camp area gets exactly the day's `per_day` and each region area `per_day × n_area / n_camp` **on top** (there is no world total to split); `wood_spot_area` maps spots to areas (0 = camp, r + 1 = region r), filled only when regions exist (L335).
- The probe already writes `fission_log` (depart, founded, cancelled ticks, members) and per-settlement `founded_tick` / `parent` (`probe/pass11.rs` L31–55); members per settlement only at `MEMBER_DAYS` [30, 60, 90, 118] (L11); `first_fission_day` is **zero-based** (`(tick + offset) / tpd`, L68, L139; display day = value + 1); the death census (`probe/deaths.rs` ≈ L281) has `settlement_at_death` and `settlement_food_stock_at_death` but no per-settlement stock or fire history.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Determinism (as Pass 9–11)
All simulation logic in Rust; new collections `BTreeMap`/`BTreeSet`; every new loop in AgentId / SettlementId order; no transcendental f64 in simulation crates; no new crates. New random draws are keyed with new, distinct stream constants added to `STREAMS`. **World draws never use the replicate**; the fission hazard draw does (7.1).

## 4.2 Versioning — v8 to v17 frozen, v18 new
- `game_v8()` … `game_v17(seed)` and `game_v17_nofission(seed)` (rules and worldgen) keep their behaviour exactly; every new field takes the value that reproduces them bit for bit. `RANGES` and `WorldVariety::draw` stay unchanged; v18's ranges are a new `RANGES_V18` read through `draw_with` (same slots, so a seed's other parameters are identical across v17 and v18).
- New: `WorldRules::game_v18(seed)`, `WorldRules::game_v18_nofission(seed)`, `WorldGenConfig::game_v18(seed)` (shared by both), scenes `game_v18_scene`, `game_v18_nofission_scene`; scene names `game_v18`, `game_v18_nofission` in every list of 3.3 (probe args, runner, twin, recorder `SCENES`/`USAGE`/dispatch, `probe.sh --help`). The live game switches to `game_v18` in both `world_node.rs` constructors.
- No new variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType`, `DeathReason` or `BuildingKind`.
- Recorder schema stays **5** unless a new array column is indispensable; if you add one, move to schema 6, mark hidden columns, update `wm_data.py` (`SCHEMAS`) with a test, and say why.

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; all Pass 2–11 tests pass unchanged except allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 12`: struct-literal field additions with legacy values; probe schema tests moving to schema 12; the `game_v18`-rejected assertion (3.3); Rust-source static assertions pinning code this goal deliberately changes, re-pinned and never weakened. Never change `sim-test/src/probe/fingerprint.rs` or `probe/long_run.rs::trace`.

## 4.4 Tuning discipline
- Tuning seeds 0–63 (10 days) and 0–31 (120 days); held-out 64–127 (10 days) and 32–95 (120 days); **diversity pilot: fresh seeds 24000–24111** (never used before Part 5, not even in unit tests — use seeds < 1000 or ≥ 900000 in tests). Seeds 30000–33999 are reserved for the WM run; never use them.
- Scratch exploration on seeds ≥ 900000 is allowed before a judged round; disclose every scratch configuration in METRICS.md.
- Knobs change only in Part 3 (judged on tuning seeds); mechanisms only in Part 2 (0.3 rule 8). Parts 4–6 run on a **frozen** commit recorded in METRICS.md before they start. A fix after freezing (review findings) → rerun everything it affects and compare, as Pass 11 did.

## 4.5 The gate
`tools/revitalize/gate_pass12.sh` = `gate_pass11.sh` with the new pass number (debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep, `gdscript_strict_check.sh`, `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, both Python unittest discoveries).

## 4.6 v17 guard after every behaviour Part
After Parts 2 and 3: re-run `game_v17`, `game_v17_nofission`, `game_v16` 64 × 10 and 32 × 120 with Pass 11's final flags and compare with Part 0's runs; every shared value identical (only `schema_version` and new keys may differ).

---

# 5. PART 0 — Baseline, re-scored diversity, daughter diagnosis

## 5.1 Baseline
1. Record `PASS12_BASE_SHA` (`31559232`), start time, `uname -a`. Commit `PASS12_GOAL.md`.
2. Release probes with Pass 11's final flags: `game_v17`, `game_v17_nofission`, `game_v16` 64 × 10 and 32 × 120 against Pass 11's final summaries (Part 7 receipts), legacy and `game_v15` against `p11-7_final_*`. Every shared key identical, timing excepted.
3. Frozen pins `harness_revitalize_pass12_frozen.rs`: `game_v17(seed)` and `game_v17_nofission(seed)` digests for seeds 0–3 at 1,440 and 40,320 ticks; field-by-field literals of `WorldRules::game_v17(0)` and of `RANGES`.

## 5.2 The new diversity measure, computed first on old data (tools only)
In `tools/revitalize/diversity_p12.py` (keep `calcore_wm11.py` and `diversity_p10.py`/`diversity_p11.py` byte-identical):
- For a scene's pilot (all 112 episodes), variable j and display day d ∈ {30, 60, 90, 118}: y_e = mean of v_e over the rows r of day d, i.e. rows with (60 r + 480) // 1440 + 1 = d (`calcore_wm11.row_of`, one-based; `bench_v1.row_of` is zero-based — do not mix them). Features x_e = the episode's hidden world parameters (`meta.json` `world_variety` values: every slot except `climate`, plus a `climate` one-hot; for `game_v15` there are none) plus m_e = mean of v_e over display days 3–10; drop features with zero std in the scene. Standardize features; ridge with λ chosen by leave-one-out MSE from {0.1, 1, 10, 100, 1000}; LOO predictions ŷ_e. A linear ridge counts a nonlinear static effect as unexplained: say so in the report, and add a synthetic test with a nonlinear static map (report its UBS).
- **R²_static(j, d)** = 1 − Σ(y − ŷ)² / Σ(y − ȳ)².
- **UBS(j, d)** (unexplained between-world spread) = sqrt(mean_e (y_e − ŷ_e)²) / S_j, with S_j the scene's split std of v_j (`bench_v1`'s definition: std over rows ≥ 24 of all 112 episodes). It is in units of the variable's own spread, is near 0 when worlds differ only by what their parameters and early level fix, and grows when histories diverge.
- Report both, per day and as the mean over days 60/90/118, for `food_stock`, `wood_stock`, `alive`, `deadwood_total`, `food_on_bushes_total`, `adults_working`; beside them the old dyn-η² (for continuity). Tests on synthetic episodes: static-only (linear) worlds give UBS ≈ 0; worlds with an independent random walk after day 10 give UBS > 0.3. Paired bootstrap of ΔUBS between two scenes on the same seeds: resample seeds with replacement (1,000×, `default_rng(0)`), recompute both UBS with the λ already chosen, report the 90 % percentile interval.
- Compute it now, read-only, on `p10/pilot/{game_v15,game_v16}` and `p11/pilot/{game_v16,game_v17_nofission,game_v17}` and commit the table (`docs/revitalize/probe/p12-0_ubs/`). These are continuity values (J7b's reference is `game_v17` recorded on the Part 5 seeds, 11.2). For `game_v17` also report UBS within the worlds that split and within those that did not (ridge refit per subset).

## 5.3 Daughter diagnosis (`docs/revitalize/DAUGHTER_DIAGNOSIS.md`)
From Pass 11's held-out and tuning-R3 v17 probe outputs (deaths census, `fission_log`, per-settlement keys, variety block; the probe has no per-settlement stock or fire history, so record the seeds concerned with Pass 11's final recorder `/Users/rexxa/work/p11bin/wm_final` — schema 5 `settlements.npy`: members, food, wood, fire per slot, hourly — into `/Users/rexxa/work/worldsim-wm-data/p12/diag/`; match a daughter to its region by `fission_log` site = `wild_regions` centre), for every death by starvation, dehydration and exposure in a daughter, and for every seed-winter over the J5 0.35 bound: the settlement and its founding (tick, season, at a region centre or at a timeout leader tile), its region's requested/placed bushes and deadwood, `water_distance`, members at founding and at death, the stock and fire history of the preceding three days. Answer with counts: (a) how many starvation deaths are in daughters with a bush shortfall, (b) how many are dehydration and how far the water was, (c) how many daughters were founded off a region centre, (d) for the over-bound winters, the dark nights per settlement and the wood per spot. Commit, push. Part 2's daughter fixes follow from these counts.

---

# 6. PART 1 — Why the afternoon work fell (J6 bisection; tools and test scenes only)

## 6.1 The differences
List every behaviour difference between `game_v16` and `game_v17_nofission` by reading `git diff 14372ea2..31559232 -- rust/crates/sim-core rust/crates/sim-engine rust/crates/sim-systems` and grepping every new rule field and flag (`hunger_weight`, `use_hardiness`, `Hardiness`, `rain_soak_permille`, `regional_wood`, `wood_spot_area`, wild regions in worldgen, and any multi-settlement audit fix gated on a v17 flag). For each: the flag or condition that gates it, and whether it can act without interventions.

## 6.2 The ablation
Add **test-only** ablation scenes in `sim-test` (e.g. a probe option `--ablate <name>` or scene names `abl_<name>`; no new constructors in `sim-core`), each built from the v17_nofission rules and worldgen with one difference reverted to v16 (reverse half: `no_regions` via `WorldVariety::without_regions` in worldgen — note it also turns regional wood off, because `wood_spot_area` stays empty without regions; `v16_damage` ([9, 5, 6] → [10, 4, 5]), `no_hunger_weight`, `no_hardiness`, `no_regional_wood`, and one per remaining gated difference), and each built from v16 with one v17 difference added (forward half). The planner's reading of the diff: everything else new in `food_economy`, `settlement_system`, `movement`, `agent_decision` and `stockpile_consume` is gated on `fission.is_some()` or on `Founding` and is inert in v17_nofission — verify, do not assume. Run every ablation on the tuning seeds 0–31 × 120 days (the runs J6 is judged on) and compute Pass 6 Y9 `work_1200`, `work_1500`, `idle_1200` with `evaluate_pass11.py`'s J6 code; also the adult count at 15:00 and the share of adults by activity at 15:00 (forage, wood, haul, build, idle, rest, eat, social, walking) for v16, v17_nofission and the decisive ablation — there is no probe key for this today: add it as a schema-12 clock key (or a test-only output of the ablation runner) and say which.
**Report:** which single reversion restores ≥ half of the gap (0.241 − 0.116 on held-out; recompute on tuning seeds), which pairs interact, and the activity that took the afternoon. If one difference explains it, Part 2 fixes it for v18 behind a flag (J6 is judged on v18_nofission). If none does, record the evidence and a hypothesis and move on (one strike). Outputs `docs/revitalize/probe/p12-1_bisect/`. Commit, push.

---

# 7. PART 2 — v18 mechanisms

## 7.1 Fission as a daily hazard
`FissionRule` gains `allowed_seasons: [bool; 4]` (v17: `[true, true, false, false]`, which is today's L394 gate) and `hazard: Option<FissionHazard>` (v17: None, which keeps the crowding gate of L419 exactly):
```rust
pub struct FissionHazard {
    pub size_lo: u32,               // 22: size factor 0 at or below
    pub size_hi: u32,               // 40: size factor 1 at or above
    pub crowd_lo_permille: u32,     // 850: crowding factor 0 at or below members/capacity
    pub crowd_hi_permille: u32,     // 1050: crowding factor 1 at or above
    pub scarcity_days: u32,         // 2: a store below members × adult need × this many days is short
    pub weight_crowd_permille: u32, // 500 (the two weights sum to 1000)
    pub weight_scarcity_permille: u32, // 500
    pub max_daily_permille: u32,    // 120: the hazard's ceiling per daily check
}
```
With `hazard` Some, the daily check at 06:00 keeps Pass 11's order (season gate — now reading `allowed_seasons` —, world cooldown, `max_settlements`, age, own cooldown, `min_members`, departed group, capacity) and **replaces the L419 crowding gate by the hazard draw, before the site choice**. It computes, in i64 thousandths, each factor clamped to [0, 1000] before use (f64 only inside the existing `adult_need_per_day` and `sustain_capacity`):
```
g = clamp((members − size_lo) × 1000 / (size_hi − size_lo), 0, 1000)
c = members × 1000 / max(capacity, 1)                       (capacity = sustain_capacity; None → no fission)
q = clamp((c − crowd_lo) × 1000 / (crowd_hi − crowd_lo), 0, 1000)
need = max(1, floor(members × adult_need_per_day × scarcity_days))
s = clamp(1000 − food_stock × 1000 / need, 0, 1000)        (food_stock = the settlement's own stockpile Food; a departed group's pending store is not counted)
h = max_daily × g × (weight_crowd × q + weight_scarcity × s) / 10^9   (thousandths; at most 3 × 10^11 before the division)
split ⇔ keyed_below(world_seed, STREAM_FISSION_HAZARD, sid, tick, 0, r, 1000) < h
```
with r = `noise_replicate_for(tick)` and `STREAM_FISSION_HAZARD` a new stream. `size_hi > size_lo`, `crowd_hi_permille > crowd_lo_permille` and the two weights summing to 1000 are asserted when the rule is built (debug_assert plus a unit test). A successful draw with no free site counts `fission_blocked_no_site` as today. The group, departure, founding and remnant rules stay Pass 11's. v18 start values: `size_lo` 20 and the rest of the struct above, `allowed_seasons` `[true, true, true, false]` (autumn departures allowed: a daughter founded before its first winter must survive it), `min_members` 22, `min_age_days` 10, `cooldown_days` 28, the rest as v17.
Instruments (probe schema 12): per seed, every daily check of every settlement that reached the draw: tick, sid, members, capacity, food stock, g, q, s, h, the draw, split, `site_free`; and per settlement the first day it reached the draw.

## 7.2 Regions that deliver (worldgen, v18 flag `region_fill: bool` in `WorldGenConfig`, v17 false)
Inside the per-region loop, right after that region's bushes are placed (camp.rs ≈ L373–379) and before its deadwood: while the region has fewer bushes than `region_bushes`, take candidates in tiers, each tier shuffled by new keyed streams `STREAM_REGION_FILL_BUSH` / `STREAM_REGION_FILL_WOOD` (added to `STREAMS`; slot = `region + (tier << 16)`, replicate 0), never on water, a building, a food source or a deadwood spot, always `beyond_camp`: tier 2 Forest in the ring 2–6 without the Plain-neighbour condition; tier 3 Plain with a Forest neighbour in the ring; tier 4 the same two filters in ring 7–9. Deadwood after the existing pass: tier 2 ring 7–9, Forest then Plain (slots `region + (2 << 16)` and `region + (3 << 16)`), with the existing 2-tile spacing. Count per tier in the worldgen report. Never edit terrain. Tests: v17 worlds unchanged (digest); v18 realized bushes and deadwood ≥ 0.95 of requested over seeds 0–127.

## 7.3 Daughter survival (from 5.3)
Implement, behind v18 flags, the fixes 5.3's counts point to, from this list only: (a) `WorldGenConfig::region_water_max: Option<u32>` (v17 None; v18 start Some(12)): a centre candidate needs a water tile within that Chebyshev distance and is filtered out before the draw; (b) `RANGES_V18.region_deadwood` raised (start 12–18) and/or `region_bushes` (start 8–12); (c) a winter-fall floor for settled regions, `FirewoodRule::area_fall_floor_permille` (v8–v17 0 = off; v18 start 600): region area r + 1 is **settled** while a settlement with ≥ 1 member has its formation tile within Chebyshev 9 of region r's centre; a settled area's per-pulse accumulator adds `max(per_day × 1000 × n_area, per_day × area_fall_floor_permille × n_camp)` (in the units of `regen_wood_winter_by_area`; `per_unit` unchanged) — this is extra wood, the camp's own fall is never reduced; the snow cap (`winter_deadwood_cap`) may turn part of it into `winter_wood_lost`, report how much; (d) a timeout founding off a region centre counts as cancelled when no bush lies within the daughter's forage radius. Record which you implemented and why, citing 5.3. Tests per flag.

## 7.4 Rain that costs wood (`FirewoodRule::rain_wet_burn: bool`, v8–v17 false, v18 true)
Under it, in `fire.rs`, a raining settlement inside the burn window still withdraws `per_hour` wood (`Settlement::withdraw` returns `min(n, held)`, so all it has, if less) and stays unlit; nothing else changes. New `Tallies::wet_burn_wood` (the units withdrawn that way), a probe key and an entry in the FFI tallies dictionary. Test: v17 rain unchanged (digest); v18 rain over a winter night burns what a dry night burns, the fire is never lit and members are `Cold`; J8's rain effect is judged in Part 6.

## 7.5 J6 fix
If Part 1 found the cause, fix it for v18 behind a flag with a test. Otherwise nothing.

## 7.6 Instruments and evaluation
Probe schema 12 (new keys only; v17 scenes write the new keys as absent or empty). `evaluate_pass12.py` from `evaluate_pass11.py`: J targets of section 10, `game_v17` as the baseline scene for J3–J5 and `game_v16` for J6. Tests: `harness_revitalize_pass12_fission.rs` (`harness_hazard_inert_in_v17`, `harness_hazard_formula` on constructed settlements for 4 cases including capacity 0 and an empty store, `harness_hazard_keyed_with_replicate` (replicate 0 equals the unforked run; replicate 3 changes at least one split day in 8 seeds with constant h), `harness_hazard_rises_with_shortfall`, `harness_allowed_seasons`, `harness_hazard_order_independent` — the draw for (sid, tick) does not depend on which other settlements exist or were checked, and spawn order changes nothing), `harness_revitalize_pass12_world.rs` (region fill, water rule, area floor, wet burn). Commit, push; run 4.6; fast-forward if green.

---

# 8. PART 3 — Tuning (tuning seeds only)
Run `game_v18` and `game_v18_nofission` on the tuning seeds (64 × 10, 32 × 120, `--check-determinism`), evaluate J1–J9 against the stated baselines on the same seeds. **Knobs and ranges:** hazard `size_lo` 18–28, `size_hi` 32–48, `crowd_lo_permille` 700–940, `crowd_hi_permille` 960–1150, `scarcity_days` 1–4, the two weights (0–1000, sum 1000), `max_daily_permille` 50–300; `allowed_seasons` `[T,T,T,F]` or `[T,T,F,F]`; `min_members` 18–32, `min_age_days` 7–21, `cooldown_days` 21–42; `RANGES_V18` region bushes and deadwood ± 3 at either end; `region_water_max` 8–16; `area_fall_floor_permille` 0–1000; `damage_per_cold_hour` child/elder 5–12, adult 3–9; `hunger_weight` 0–1; `rain_soak_permille` 150–500. At most three judged rounds; record each round's knobs and J results in METRICS.md. Freeze: commit, record the frozen commit, copy the probe and recorder binaries to `p12bin/`. Commit, push; run 4.6.

# 9. PART 4 — Held-out health
On the frozen commit: both v18 scenes on held-out seeds (64–127 × 10, 32–95 × 120), J1–J9 against the same baselines on the same seeds; timing for J12 (section 1). No knob changes. Commit, push.

---

# 10. TARGETS (J)

| # | target |
|---|---|
| J1 | determinism 64/64 and 32/32 for both v18 scenes on tuning and held-out seeds; legacy pin; v8–v17 reproduce Pass 11's final summaries on every shared key |
| J2 | **no collapse** (both v18 scenes, every 120-day seed): total alive ≥ max(8, 0.4 × starting agents) at every daily sample; total alive on day 120 ≥ 0.6 × day 21; no settlement lingers with 1–2 members for more than 48 h |
| J3 | **fission comes when the village is under pressure** (`game_v18`, 120-day seeds, tuning and held-out separately): share of seeds with ≥ 1 fission in [0.50, 0.90]; first-fission day (daughter formation, **display day = the probe's zero-based `first_fission_day` + 1**; `evaluate_pass12.py` converts before every statistic): **IQR ≥ 14 days, median in [20, 80], and no single day holds more than 0.25 of the first fissions**; share of seeds with ≥ 2 settlements alive at day 120 in [0.30, 0.85]; no seed above `max_settlements`; cancellations ≤ 0.25 of attempts; reported: the split rate per daily check by tercile of h, and the first-fission day's correlation with each hidden parameter |
| J4 | daughters alive 28 days after founding ≥ 0.70 (Pass 11: 0.60 bound, 0.85 measured); report the share that dissolved or merged, by founding season |
| J5 | **the cold can take anyone, but no winter takes a village** (pooled tuning + held-out seed-winters with ≥ 1 exposure death): adult share in [0.15, 0.50], child share ≤ 0.70, elder share reported; the median over settlement-nights with a death of the maximum exposure deaths in one hour ≤ 2; share of seed-winters with ≥ 1 exposure death in [0.15, 0.60], median per seed-winter ≤ 2, **maximum over settlement-winters of exposure deaths ≤ 0.35 × that settlement's members at the winter's start** (probe schema 12 adds per settlement `members_at_winter_start`, sampled at 18:00 of display days 22, 50, 78, 106 — the evening the first winter night begins (nights are booked to the season of their evening) — and exposure deaths per settlement-winter; a settlement founded inside the winter uses its founders; Pass 11's world-level ratio reported beside it), mild < normal < harsh; starvation + dehydration 0 at 10 days in every seed, 120-day median ≤ 1, **maximum ≤ 6** |
| J6 | **village rhythm kept** (`game_v18_nofission` vs `game_v16`, Pass 11's J6 comparison mode and escape table): Pass 6 Y4, Y5, Y6, Y9, Y10, Y11, Y12 and Pass 8 X3 (b)–(e), X4. `game_v18` reported per agent against its own settlement's hearth |
| J7 | **diversity with histories** (Part 5 pilot, `game_v18`, test split): η² ≥ 0.30 **and** R²_abs ≤ 0.70 for `food_stock`, `wood_stock` and `alive`; **J7b** (all 112 episodes, 5.2; reference = `game_v17` recorded on the **same** seeds 24000–24111): for `alive` and for `food_stock`, the paired-bootstrap 90 % interval of ΔUBS (v18 − v17, mean over days 60/90/118) lies above 0, and mean R²_static(`alive`) of v18 ≤ that of v17 |
| J8 | **interventions have consequences** (Part 6, `game_v18`): placebo identity 100 %; **rain: mean deaths effect ≥ 0** and mean dark-night effect > 0; at least two of {rain, cold_snap, rot_wood} have replicate-mean deaths effect > 0 in ≥ 0.25 of seeds; gift_wood's mean deaths effect ≤ 0 |
| J9 | variety and regions **realized**: realized region bushes and deadwood ≥ 0.95 of requested over the 128 tuning + held-out seeds; every drawn parameter's mean within ± 10 % of its range centre; region placement failures 0 (or counted and explained) |
| J10 | recorder byte identity (two runs; `--workers 1` vs 4) for both v18 scenes; schemas 1–5 still load; v18 cases added to `tools/worldmodel/test_export_determinism.py` (as `test_v17_fork_families_are_byte_identical`) and to the harness fork tests, all passing |
| J11 | macro sensitivity reported (12.2; no threshold), with the noise-only baseline |
| J12 | ms per agent-tick ≤ 1.3 × `game_v16` (both v18 scenes, back to back on one host, section 1) |
| J13 | the J6 bisection delivered (Part 1): the decisive difference with its share of the gap, or the ruled-out list and a hypothesis |

---

# 11. PART 5 — The diversity pilot (frozen commit)

## 11.1 Recording
`worldsim_wm` (frozen binary), 120 days, `--every 60`, no `--agents`, seeds **24000–24111** (`--seeds 24000..24112`), scenes `game_v16`, `game_v17`, `game_v18_nofission`, `game_v18` → `/Users/rexxa/work/worldsim-wm-data/p12/pilot/<scene>/`. Roles: 24000–24095 train, 24096–24111 test.

## 11.2 Metrics
Pass 11's `diversity_p11.py` outputs for all four scenes (η², R²_abs, R²_year, divergence day, spread, calendar-family and persistence nMAE at 24 h and 168 h, settlement counts at days 30/60/90/118 with their distribution), plus 5.2's R²_static and UBS for each scene on these seeds. First run the Pass 10 implementation check again (v1 A_test, WM-1.1's 12 numbers to ± 0.002).

## 11.3 Attribution
v17 → v18_nofission isolates rain wet burn, region fill, daughter fixes, the J6 fix (if any) and exposure knobs; v18_nofission → v18 isolates hazard fission (with the region fill). Report Δη² and ΔUBS per variable.

## 11.4 Outputs
`docs/revitalize/probe/p12-5_pilot/` (`diversity.json`, `diversity.md`, exact commands). J7 is judged here, once.

---

# 12. PART 6 — Forks (frozen commit, `game_v18`)

## 12.1 Consequences (35 days)
Pass 11's `docs/revitalize/probe/p11-6_forks/p11_forks.sh` copied to `p12_forks.sh` (schedules regenerated with `tools/revitalize/pass9_fork_schedules.py` into `p12/schedules`; `p11/` is read-only) with `--scene game_v18`, data root `p12/`, the frozen binary and its `WORLDSIM_GIT_SHA`: seeds 12000–12031, 35 days, `--every 60`, fork tick 28,800, the five C_cf variants and the one-unit gift plus the factual, `--replicates 4`, `--agents`, `--include-factual`; the placebo pass (seven amount-0 kinds on 12000–12007); unforked runs; `fork_metrics.py --deaths-with-exposure --home-slot-fire --deaths-by-stage --unforked …`. Report Pass 11's tables plus `wet_burn_wood` per variant (or, from the recorder, the drop in home wood over rain hours inside the burn window) and first-fission day per variant. J8 is judged here.

## 12.2 Macro sensitivity (120 days; drop first)
Pass 11's design (seeds 12000–12031, `rot_wood` and `gift_wood` plus the factual, fork tick 28,800, `--replicates 2`), adding `spoil_food` 60 at tick 29,400 on home (it acts on the hazard's scarcity term; `pass9_fork_schedules.py` writes `spoil_food` only as a zero-amount placebo today, so write this schedule file explicitly and record it). With autumn departures many worlds split before the fork: compare **the first fission founded after the fork tick**. Extend `macro_sensitivity.py` (with a test) accordingly. Report for (a) factual r0 vs r1 (noise only) and (b) each intervention vs the factual at the same replicate: the share of seeds whose settlement count at day 60/90/118 or post-fork first-fission day differs; the mean |Δ post-fork first-fission day| among seeds where both split after the fork; the mean |Δ| of total alive at day 118. State plainly whether interventions move the timing more than noise does. Outputs `docs/revitalize/probe/p12-6_macro/`. Commit, push.

---

# 13. PART 7 — Independent review, final verification, report
1. **Independent review** (fresh `general-purpose` subagent, two rounds maximum): `git diff 31559232..HEAD -- rust/ tools/revitalize/ tools/worldmodel/`, sections 4, 7, 10, 11, 12, METRICS.md's Pass 12 rounds, the bisection, the diagnosis, the pilot and fork outputs. With file:line evidence it checks: v8–v17 invariance; keyed draws and their replicate use (hazard and group with, worldgen never); the hazard formula against 7.1 on two logged checks recomputed by hand; no mechanism change inside a judged round; region fill tiers never touch terrain or water; wet burn only in v18; the pilot ran on the frozen commit with fresh seeds; UBS computed as 5.2 says (LOO, split std); J verdicts follow the rules.
2. Full gate on the final WIP head; totals versus Pass 11 (2,078 debug and release).
3. Final probes: legacy and v8–v17 reproduce Pass 11; both v18 scenes on tuning and held-out seeds; timings for J12; say whether any code changed after Parts 5–6 and rerun what is affected.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS12.md` in Korean (section 15 format). Section 14's in-game checks are for the owner: smoke only, list them as "주인 확인 필요". Append `## Pass 12` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 12 section (REST API).

## 13.1 Out of scope
Temperament and personality beyond hardiness; family and kinship links; trade or conflict between settlements; dataset v2 and its benchmark; weather resampling in forks; any learned model; UI and art (merge notes only).

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)
1. 게임이 game_v18로 시작하고 콘솔에 SCRIPT ERROR가 없다.
2. 처음 마을이 붐비거나 식량이 모자라 보일 때, 가을이나 봄·여름의 어느 날 아침 무리가 떠난다 — 새 게임마다 떠나는 날이 다르고, 늘 29일차 아침은 아니다.
3. 야생 지역에 덤불과 나무가 실제로 모여 있고, 딸 마을 근처에 물이 있다.
4. 딸 마을이 첫 겨울을 대부분 넘기고, 한 겨울에 마을 사람 3분의 1 넘게 얼어 죽는 일은 드물다.
5. 비를 내리면 그날 밤 불은 꺼진 채인데 장작은 줄어든다(비가 장작을 아껴 주지 않는다).
6. 120일 동안 모든 마을이 통째로 사라지는 일은 없다.

---

# 15. 구현 완료 보고 (REPORT_PASS12.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
새 다양성 지표(R²_static·UBS)와 옛 데이터 재채점, 딸 마을 진단, 오후 노동 이분 탐색(어떤 차이가 원인인가), 분열 위험도(공식·키 추첨·허용 계절), 지역 채움 단계, 딸 마을 생존 수정, 젖은 연소 비, v8~v17 고정, 튜닝 라운드(손잡이만), 파일럿, 분기 실험, 거시 민감도

### 기능 설명
게임과 연구 관점에서 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
J1~J13 표, 튜닝 라운드별 손잡이와 결과, 첫 분열일 분포(v17 대 v18)와 위험도 삼분위별 분열률, 딸 마을 생존·사망 원인, 단계별 노출 사망, v16/v17/v18_nofission/v18 다양성 표(η²·R²_abs·R²_static·UBS·옛 dyn-η²·정착지 수·기준선 nMAE), 귀속, 분기 실험(비의 사망 효과·젖은 연소량·가짜 개입), 거시 민감도(잡음 대 개입)

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
분열 시점이 이제 세계의 상태와 역사에 따라 갈라지는가, 아니면 주사위인가? 숨은 파라미터로 설명되지 않는 세계 간 차이(UBS)는 v17보다 커졌는가? 신의 작은 개입이 분열 시점을 잡음보다 크게 움직이는가? 게임으로서 나아졌는가? 데이터셋 v2를 v18로 가도 되는가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, the paths of `DAUGHTER_DIAGNOSIS.md`, the bisection report, `diversity.md`, `fork_metrics.md` and the macro-sensitivity report, and the time spent per Part.
