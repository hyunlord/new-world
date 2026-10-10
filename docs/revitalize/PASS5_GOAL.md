# GOAL: WorldSim — Revitalize Pass 5: eat together at the hearth, keep the fire going, work through the day

Repository `hyunlord/new-world`. Continue the Pass 4 line: `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`, both at `5cbcbc4f9bd02bf3f00695bae810368b93fe6300` (draft PR #510 into `lead/main`).
This run is executed by **Claude Code**, unattended. The art run (**Art Pass 3**, Codex, branch `codex/art-production-2026-10`) may be working on the same Mac; section 1 says how to share it.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4            # the Pass 4 clone, hooks already neutralized
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 5cbcbc4f
cp ~/Downloads/claude_goal_worldsim_pass5.md docs/revitalize/PASS5_GOAL.md
tmux new -s pass5
cd /Users/rexxa/work/new-world-pass4
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS5_GOAL.md>
```
The repository's Claude Code hooks are neutralized **in this clone only**: `.claude/settings.json` holds `{}` and is hidden from git with `--skip-worktree`. Never edit, stage or commit it and never clear that flag. No git hooks are installed.

**This document is `docs/revitalize/PASS5_GOAL.md`.** Commit it unchanged in Part 0. At the start of every Part, after every compaction or pause, and whenever you are unsure of a number, re-read the relevant section from that file.

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
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 5`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5` and background job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.**
6. **Time rule:** record start time and each Part's start and end. Past **10 hours** of working time, finish the current step, push, go to Part 7.
7. **Three strikes** per failing target, then keep the best configuration and record evidence and hypothesis.
8. **Drop order when time is short:** cold nights (8.5) first, then the preemption extension (Part 4). Never drop Parts 1, 2, 3 (except 8.5), 5 or 7.
9. **No image generation.**
10. **Keep bulky output out of context:** probe runs, test logs and captures go to files; read summaries.
11. **Long commands run in the background** (one Bash call is cut off after 10 minutes): `nohup bash -c '<command>; echo $? > /tmp/p5_<name>.exit' > /tmp/p5_<name>.log 2>&1 &`, poll with short calls, check `pgrep -fl` before starting a job again. Keep gate logs in `docs/revitalize/verification/p5-<N>/`.

---

# ⚠️ 1. MACHINES

- **The Mac** clone `/Users/rexxa/work/new-world-pass4`. The art run may be active in `/Users/rexxa/work/new-world-art-prod`; never read, write or build there, never touch `~/github/new-world-wt/lead` or other checkouts, never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe `--workers 4`. `tools/revitalize/probe.sh` and `run_bounded.sh` are already portable (Pass 4).
- Windowed Godot only through `tools/revitalize/with_window_lock.sh` in one command; from a background tmux session run that single command in the GUI session with `launchctl asuser $(id -u) …` as Pass 4 did. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- **DGX Spark** (`ssh hyunlord@100.70.109.50`, `/home/hyunlord/work/new-world-revitalize`) is optional for long probe batches (detach to your pushed WIP commit, `CARGO_BUILD_JOBS=10`, `--workers 8`, copy summaries back; no ComfyUI 8188, no dev servers 3100/3200, no other checkouts).

---

# 2. GIT

- Every commit goes to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.4) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p5-N): <what changed>`. Stage explicit paths only, never `git add -A`.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`.
- **Must not change** (the art run owns them): `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, and the static tests that read `.gd`/`.tscn`/`.gdshader` files, except minimal Rust-source re-pins allowed in 4.3.

---

# 3. CONTEXT

## 3.1 Pass 4 result (verified by the planner from the committed evidence)
Gate green at `0dc9a38d` (debug and release 1,881 each). `game_v10`: no stagnation (longest stagnant run 2 days, v9 39), turnover (120 days: 91 births, 73 old-age deaths, alive 37–42), seasons with a winter store (autumn median 492, winter 254), zero starvation and dehydration. W1–W7 and W11 passed; W8, W9, W10 stopped after three rounds.

## 3.2 What is still wrong (planner's reading of `p4-7_final_v10_120day/summary.md`)
- **One job, done by mid-morning.** Purposeful forage trips run only on days 1–2 and around the start of summer (days 7–10: 55–62 a day); once the store passes its target they stop. Opportunistic gathering and hauling (22–32 deliveries a day) keep the store climbing to the 2 × target cap (day 20: 556 against a target of 353). Bushes are empty by mid-morning; in summer 39 units a day of regrowth are lost at the 4-unit cap. Noon idle median is 0.81 (Pass 3: 0.48). About a third of residents (elders 26 %, children 10 %) never work by design.
- **The store is not food.** Hungry residents go to a bush the moment hunger exceeds 50 (`actionable_need_with_fatigue_threshold`, decision priority 125); `StockpileConsumeSystem` (142, every 5 ticks) skips anyone already on the ground-eat path, so the store feeds people only when no bush holds food. That is why evening-near-hearth has a seed minimum of 0.28.
- **Foraging cannot be interrupted.** Critical preemption applies only to `Seeking{Food|Water}` (`agent_decision.rs` around line 1372); a forager in `Seeking{GatherFood}` keeps walking whatever its thirst. No harm yet (dehydration 0), a latent risk.
- **W10 was a bad target.** The v9 baseline itself exceeded "every seed ≤ 10" (max 10.52); v10 median 7.06.
- **Growth is identical in every seed** (exactly 7 births by day 4, 42 alive on day 10): the 720-tick birth cooldown binds, not food. Out of scope this pass; record it.

## 3.3 This pass
Meals at the hearth make the store daily food (and pull people home in the evening). Firewood gives a second job in the afternoon and a reason to stock up for winter: the hearth burns wood at night, more in winter, and a winter night without fire makes residents hungrier. Foraging and wood trips become interruptible by critical needs.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Architecture and determinism (unchanged)
All simulation logic in Rust; FFI only adds keys; new collections `BTreeMap`/`BTreeSet`; no order-dependent hash iteration; no transcendental f64 in simulation crates; seeded `splitmix64` streams only (`MovementRng`); no new crates. **Authorized enum addition:** `TargetKind::GatherWood` (FFI `target_kinds` code 7), with every exhaustive match updated. No new `CausalEvent`, `DecisionReason`, `TerrainType` or `DeathReason` variants. Time windows use integer minute-of-day (`minute_in_window`, wrap-around allowed). Day index as in Pass 4.

## 4.2 Versioning — v8, v9 and v10 frozen, v11 new
- `game_v8()`, `game_v9()`, `game_v10()` and their `WorldGenConfig`s keep their behaviour exactly; new fields take values that reproduce them bit-for-bit (proved in Parts 0 and 7).
- In Part 1 add `WorldRules::game_v11()` (= `game_v10()` until Parts 2–4 add their rules), `WorldGenConfig::game_v11(seed)` (= `game_v10` plus the deadwood fields of 8.1), `game_v11_scene(seed)`, probe scene `game_v11`. Every v11 worldgen addition draws from its **own** stream after all existing steps, so the v10 part of a v11 world is identical to `game_v10(seed)`.
- The live game switches to v11 in both `world_node.rs` constructors.

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; `harness_population_legacy_scene_pinned`, `harness_population_game_world_survives_3_days` and all Pass 2–4 tests pass unchanged. Never change `sim-test/src/probe/fingerprint.rs`. Allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 5`: struct-literal field additions with legacy values; probe schema tests moving to schema 5; Rust-source static assertions that pin code this goal deliberately changes (the live world constructor, additive FFI keys, the `TargetKind` match), re-pinned and never weakened.

## 4.4 The gate (required before the green branch moves)
Same as Pass 4 (`tools/revitalize/gate_pass4.sh` may be copied to `gate_pass5.sh`): debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep over `sim-core/src sim-systems/src sim-engine/src` (0 hits), `gdscript_strict_check.sh` (read-only checker), `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, `python3 -m unittest discover -s tools/revitalize -p 'test_*.py'`.

---

# 5. PART 0 — Baseline and frozen references

1. Record `PASS5_BASE_SHA` (`5cbcbc4f`), start time and `uname -a`. Commit `PASS5_GOAL.md`.
2. Release probes: `game_v10` 64 × 10 and 32 × 120 against `p4-7_final_v10_10day` and `p4-7_final_v10_120day` (every shared key identical, timing excepted); `game_v9` 64 × 10 against `p4-7_final_v9_10day`. A difference stops the run until explained.
3. Record `sim_test::probe::fingerprint` of `game_v10_scene(seed)` for seeds 0..4 after 1,440 ticks; they become the constants of the Pass 5 `*_v10_unchanged` tests (keep the Pass 4 v9 ones).
4. Document the noon-idle mechanism for yourself: `game_v10` seeds 7..9, 30 days, `--clock-samples 06:00,08:00,10:00,12:00,15:00,18:00,19:30,02:00`; write per-hour state shares (idle, foraging, hauling, eating at a bush, social, other) and per-hour bush food totals to METRICS.md under `## Pass 5 baseline`.

---

# 6. PART 1 — Instruments and the v11 skeleton

- `game_v11` rules, worldgen and scene as in 4.2 (no behaviour yet beyond v10; deadwood placement of 8.1 may land here or in Part 3).
- `Tallies` (display-only): `meals_from_store`, `ground_meals` (bush meals resolved), `wood_trips`, `wood_gathered`, `gather_preempted`, `cold_member_hours`.
- Sample schema **version 5** (every v4 key keeps its meaning; never null): `meals_from_store_total`, `ground_meals_total`, `wood_stock`, `wood_target`, `wood_in_hand`, `fire_lit` (0/1 for the settlement), `cold_members`, `wood_trips_total`, `wood_gathered_total`, `deadwood_total`, `gather_preempted_total`, `gather_seekers_critical` (agents in `Seeking{GatherFood|GatherWood}` with hunger or thirst ≥ 90; feeds X11); at clock samples also `adult_work_share` (non-child, non-elder members in `Seeking`/`Consuming` of `GatherFood`, `GatherWood` or `ConstructionSite`, or hauling, divided by non-child, non-elder members) and `idle_share`.
- `summary.md` gains **Meals** (store and ground meals per day by season), **Fire and wood** (stock, target, burn, lit share at 02:00, cold member-hours by season) and **Work by hour** (adult work share and idle share at each clock sample).
- `tools/revitalize/evaluate_pass5.py` (+ tests) computes every X target of section 11 from raw JSONL.
- Baseline: `game_v10` 32 × 120 with the clock samples of 5.4 into `p5-1_v10_120day`. Commit, push WIP.

---

# 7. PART 2 — Meals at the hearth

## 7.1 Rule (`sim-core/src/rules.rs`)
```rust
pub struct SocietyRules {
    // existing fields unchanged
    /// Shared meals from the settlement store. None = Pass 4 behaviour.
    pub meals: Option<MealRule>,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct MealRule {
    pub breakfast_start: u32, // v11 360 (06:00)
    pub breakfast_end: u32,   // v11 480 (08:00)
    pub dinner_start: u32,    // v11 1080 (18:00)
    pub dinner_end: u32,      // v11 1200 (20:00)
    pub meal_hunger: f32,     // v11 25.0 — served at a meal from this hunger up
    pub camp_radius: u32,     // v11 8 — Chebyshev from the formation tile; "at camp"
}
```
Legacy, v8, v9, v10: `None`.

## 7.2 Eat at camp instead of walking to a bush
In the Idle cascade, under the rule: a settlement member whose food need is actionable (`hunger > HUNGER_THRESHOLD`) while it stands within `camp_radius` of its formation tile and its settlement's stockpile holds Food ≥ 1 does **not** enter `Seeking{Food}`; it stays Idle and **no other arm fires for it this tick** except a critical water need (it is waiting for its meal). `StockpileConsumeSystem` serves it within five ticks. Members outside `camp_radius`, or with an empty store, eat at a bush exactly as today.

## 7.3 Serving (`StockpileConsumeSystem`)
Under the rule the serving radius is `camp_radius` and the threshold is `meal_hunger` inside the two meal windows and `HUNGER_THRESHOLD` outside them; one unit (30 hunger) per member per pass, as today; `Tallies.meals_from_store += 1` per unit served. Ground meals count `ground_meals` where a `Consuming{Food}` resolves. Without the rule nothing changes (the relief path and its threshold stay exactly as in Pass 4).

## 7.4 Expected shape
Residents wake in shelters at 06:00 inside `camp_radius` and eat about one unit each at breakfast; the evening gather (18:00–21:00) brings them back for dinner. The store now loses roughly 2 units per resident per day, so the stock target (Pass 4) is crossed every day and purposeful foraging runs every day. The birth gates are unchanged.

## 7.5 Tests — new file `harness_revitalize_pass5_meals.rs`
- `harness_meal_breakfast_from_store`: at 06:30, members at camp with hunger 30 are served from the store; the store drops by the number served; nobody enters `Seeking{Food}`.
- `harness_meal_hungry_at_camp_waits_for_store`: a member at camp with hunger 55 and store ≥ 1 does not seek a bush and is fed within 5 ticks; no other arm sends it away meanwhile.
- `harness_meal_away_eats_at_bush`: the same member 12 tiles from camp seeks a bush.
- `harness_meal_empty_store_falls_back`: store 0 at camp → seeks a bush.
- `harness_meal_v10_unchanged`: fingerprints of Part 0.3 reproduce.

Commit, push WIP; fast-forward green if the gate is green.

---

# 8. PART 3 — Firewood, fire and cold

## 8.1 Deadwood in the world (`WorldGenConfig`, `camp.rs`)
New fields: `deadwood_spots: u32` (v11 20, others 0), `deadwood_capacity: u8` (v11 6), `deadwood_min_radius: u32` (v11 5), `deadwood_max_radius: u32` (v11 18). Under `deadwood_spots > 0`, after every existing worldgen step, choose spots with a dedicated stream (`MovementRng::new(seed ^ DEADWOOD_SALT)`, a new constant distinct from every other salt; record it) among Forest tiles at Chebyshev `[min, max]` from the hearth, then Plain tiles if Forest runs out (candidates sorted by `(y, x)`, then a Fisher–Yates shuffle with that stream), never water, never on a building or bush, and at least 2 tiles (Chebyshev) from another spot. Register them in `SimResources::wood_tiles` / `wood_source_max` (new maps mirroring the food ones) at full capacity.

## 8.2 Regrowth
`ResourceRegenSystem` under the rule adds a wood channel with its own accumulator `SimResources::wood_regen_acc` (pulse: `acc += regen_permille; units = acc / 1000; acc %= 1000`; skip when `units == 0`; never insert zero-valued tiles). With `regen_permille` 500 a spot regrows 6 units a day. The food channel and its seasons are unchanged.

## 8.3 Rule (`SocietyRules::firewood: Option<FirewoodRule>`)
```rust
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FirewoodRule {
    pub gather_start_minute: u32,     // v11 600 (10:00)
    pub gather_end_minute: u32,       // v11 1020 (17:00)
    pub wood_radius: u32,             // v11 18, Chebyshev from the formation tile
    pub haul_threshold: u32,          // v11 2 wood carried → go home
    pub max_gatherers_per_spot: u32,  // v11 1
    pub regen_permille: u32,          // v11 500 per 120-tick pulse
    pub base_days: f64,               // v11 1.5 days of current burn
    pub winter_prep: bool,            // v11 true
    pub burn_start_minute: u32,       // v11 1080 (18:00)
    pub burn_end_minute: u32,         // v11 420 (07:00), wraps midnight
    pub burn_per_hour: [u16; 4],      // v11 [2, 1, 2, 3] spring, summer, autumn, winter (index 0 without seasons)
    pub cold_start_minute: u32,       // v11 1320 (22:00)
    pub cold_end_minute: u32,         // v11 360 (06:00)
    pub cold_hunger_permille: u32,    // v11 1500 (hunger grows ×1.5)
}
```
`burn_per_day(s) = burn_per_hour[s] × 13` (the burn window's hours). Wood target, computed once per settlement per decision tick like the food target: `base_days × burn_per_day(current) + (seasons on and winter_prep and season ∈ {summer, autumn}) × days_per_season × burn_per_day(winter)`. `wood_in_hand` = stockpile Wood + Wood carried by members.

## 8.4 The Firewood arm (`agent_decision.rs`, Idle cascade)
After the Forage arm and before the opportunistic Gather arm; adults only (no `Child`, no `Elder`); inside the gather window; member; carried Wood < `haul_threshold`; inventory room; `wood_in_hand < wood_target`; a deadwood tile with amount ≥ 1 within `wood_radius` of the formation tile and `claims < max_gatherers_per_spot`. Choice minimises `(claims, chebyshev(agent, tile), y, x)`; claims are counted before the agent loop and incremented as assigned (as Pass 4's forage). Enter `Seeking{GatherWood}` with its `SeekTarget`; `wood_trips += 1`. Empty-tile re-targeting as Pass 4's forage. `Consuming{GatherWood}` takes `min(tile amount, haul_threshold − carried Wood, room)` into the inventory, decrements the tile (removing it at 0, regrowth recreates it), `wood_gathered += n`, then Idle.
Haul home generalises Pass 4's `haul_due`: a member carrying Food ≥ the forage threshold or Wood ≥ the wood threshold, or carrying either outside both windows, steps home. `StockpileDepositSystem` drains Wood as well as Food under the rule (Food only without it).

## 8.5 The fire and cold nights
`FireSystem` (priority 147, interval 60, settlements in id order): inside the burn window it withdraws `burn_per_hour[season]` Wood from the stockpile; `lit = (withdrawn == burn_per_hour)`; outside the window `lit = false`. State lives in `SimResources::fires: BTreeMap<SettlementId, FireState { lit: bool }>`. Then, with seasons on and the season winter and the minute inside the cold window and `!lit`, every member gets the zero-sized marker `Cold` (new component, documented in `sim-core/CLAUDE.md`); otherwise members lose it. `cold_member_hours += members with Cold` per run. `HungerDecaySystem` adds `growth_rate × cold_hunger_permille / 1000` instead of `growth_rate` for agents with `Cold`; without `Cold` the arithmetic is byte-for-byte the current one.

## 8.6 FFI (additive)
- `get_agent_snapshot`: `carried_woods: PackedInt32Array`; `target_kinds` may now be 7 (gather wood).
- `get_settlement_snapshot`: `wood_stocks: PackedInt32Array`, `fire_lit: PackedByteArray`.
- `get_resource_snapshot`: rows of kind **3 = deadwood** with amount and max.
- `get_tallies`: the six new tallies of Part 1.

## 8.7 Tests — new file `harness_revitalize_pass5_firewood.rs`
`harness_deadwood_placement` (20 spots within the radii, none on water or buildings, deterministic per seed, v10 tiles untouched), `harness_wood_regen_schedule` (6 a day per emptied spot, no zero tiles), `harness_firewood_trip_and_deposit`, `harness_firewood_stops_at_target`, `harness_firewood_claims_cap`, `harness_firewood_retargets_empty_spot`, `harness_fire_burns_and_goes_out` (store 0 → `lit == false` at 02:00), `harness_fire_cold_raises_hunger` (winter night, fire out → growth ×1.5 for members; fire lit → ×1.0), `harness_firewood_v10_unchanged`.

Commit, push WIP; fast-forward green if the gate is green.

---

# 9. PART 4 — Critical needs interrupt gathering

`AgentRules::preempt_gathering: bool` (v11 true; false elsewhere). Under it, an agent in `Seeking{GatherFood}` or `Seeking{GatherWood}` whose hunger or thirst value is ≥ `critical_preemption` (75.0) becomes Idle and drops its `SeekTarget`; the next decision tick serves the need; `gather_preempted += 1`. Test `harness_gather_preempted_by_thirst` (a wood seeker 15 tiles out with thirst 80 turns back to water within one tick) and `harness_preempt_v10_unchanged`. Commit, push WIP.

---

# 10. PART 5 — Measure and tune (three rounds maximum)

Runs (release): `game_v11` 64 × 10 with `--check-determinism`, and 32 × 120 with `--check-determinism --clock-samples 02:00,06:00,08:00,10:00,12:00,15:00,18:00,19:30` → `docs/revitalize/probe/p5-5_round<N>_{10day,120day}/`; evaluate with `evaluate_pass5.py`.

---

# 11. TARGETS (game_v11; table order = priority)

| # | target |
|---|---|
| X1 | determinism 64/64 (10 d) and 32/32 (120 d); legacy pin; v8, v9 and v10 reproduce Pass 4's final summaries on every shared key |
| X2 | starvation and dehydration: zero in every seed at 10 d; 120 d totals median 0 and max ≤ 2 each |
| X3 | Pass 4 gains kept: one settlement every day; a birth by day 4 and alive ≥ 30 on day 10 in every seed; longest stagnant run ≤ 4 days; an old-age death by day 30 and births ≥ deaths − 3 over days 30–120 in every seed; alive in [32, 44] from day 30; child and elder shares as W6 |
| X4 | meals: from day 3, store meals ≥ 0.60 of all meals (median over seed-days) and ≥ 0.40 in every seed; winter median ≥ 0.40 |
| X5 | foraging every day: forage trips per day median ≥ 8 in every season |
| X6 | fire: lit at the 02:00 sample in ≥ 0.90 of samples from day 3 (median of seeds) and ≥ 0.75 in every seed; wood stock at the last autumn sample median ≥ 0.8 × 7 × winter burn per day |
| X7 | cold: winter cold member-hours ≤ 0.10 of winter member-hours in the cold window (median) and ≤ 0.25 in every seed |
| X8 | daytime: adult work share median ≥ 0.35 at 12:00 and ≥ 0.25 at 15:00 over spring–autumn days; idle share of all residents at 12:00 median ≤ 0.60 (Pass 4: 0.81) |
| X9 | rhythm: night-in-shelter median ≥ 0.80; evening-near-hearth median ≥ 0.55 and ≥ 0.35 in every seed |
| X10 | no regressions: T6a ≤ 1 % (10 d and 120 d medians); social completed per agent-day ≥ 0.40 and > 0 in every seed; day-10 home distance median ≤ 8.1 and every seed ≤ 13 (v10: 7.06 and 11.05; wood trips take people out on purpose) |
| X11 | preemption: no agent in `Seeking{GatherFood\|GatherWood}` with hunger or thirst ≥ 90 at any 120-tick sample in any seed |
| X12 | ms per agent-tick ≤ 1.5 × v10 measured on the same host in this run |

Knobs (only these, all in `game_v11()` / `WorldGenConfig::game_v11`): meal windows ±60 min; `meal_hunger` 15–35; `camp_radius` 6–10; firewood window start 540–720 and end 960–1080; `wood_radius` 12–20; wood `haul_threshold` 1–4; `max_gatherers_per_spot` 1–2; `deadwood_spots` 12–30; `deadwood_capacity` 4–10; wood `regen_permille` 250–1000; each `burn_per_hour` 1–4; wood `base_days` 1.0–2.5; `cold_hunger_permille` 1200–2000; the Pass 4 forage and stock-target knobs within Pass 4's ranges. Record every round with all numbers in METRICS.md; keep the best configuration in `game_v11()`. Commit, push WIP; fast-forward green if the gate is green.

---

# 12. PART 6 — Look at the screen (existing tooling, no GDScript edits)

`CAPTURE_DOCS_DIR=docs/revitalize/shots/pass5 bash tools/revitalize/with_window_lock.sh 1500 bash tools/revitalize/capture.sh final` (GUI session as in section 1). Commit the JPEGs and `visual_qa.md`. Answer in one sentence each from `d2_1200_F2`, `d2_2200_F2N` and `d6_1200_F2`: where people are at noon (bushes, deadwood, camp); whether villagers gather at the hearth in the evening; whether the camp still reads as one village. The renderer does not draw wood, fire state or meals yet; do not fix rendering.

---

# 13. PART 7 — Independent review, final verification, report

1. **Independent review:** a built-in `general-purpose` subagent with fresh context gets `git diff 5cbcbc4f..HEAD -- rust/ tools/revitalize/`, sections 4, 7, 8, 9 and 11 and the final X table; it reports concrete defects (determinism, v8/v9/v10 invariance, formula errors, untested branches, starvation or freeze paths) with file:line evidence. Fix confirmed defects (two rounds maximum); record every finding and disposition in DECISIONS.md.
2. Full gate on the final WIP head; totals versus Pass 4 (1,881 debug and 1,881 release).
3. Final probes: legacy, v8, v9 and v10 reproduce Pass 4; `game_v11` 64 × 10 and 32 × 120 with census and clock samples.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS5.md` in Korean (section 15 format), answering each item of section 14 with its evidence level. Append `## Pass 5` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 5 section (`gh pr edit 510 --body-file …`; skip and say so if `gh` is not authenticated). No new PR.

## 13.1 Merge touch-points for the art run (list them in the report)
`target_kinds == 7` (gathering wood: crouched at deadwood); `carried_woods` (carry sticks instead of the berry basket); `get_resource_snapshot` kind 3 (deadwood piles: fallen logs or stick heaps); `get_settlement_snapshot.wood_stocks` (woodpile size) and `fire_lit` (hearth lit at night or cold ring); meal windows 06:00–08:00 and 18:00–20:00 (seated residents at the hearth may use eat frames); a HUD wood count needs a new localization key.

## 13.2 Out of scope
Rendering and HUD; birth-rate variety (the cooldown-bound growth noted in 3.2); new food types; hunting and fishing; crafting and tools; trade; new settlements; weather beyond the cold-night rule; map size; merging into `lead/main`; the art-owned paths.

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 아침 6~8시와 저녁 6~8시에 주민들이 화덕 주변에서 비축 식량을 먹는다. 캠프에 있는 배고픈 주민은 멀리 덤불까지 걸어가지 않는다.
2. 비축이 매일 줄었다 채워지므로, 어른들이 계절과 상관없이 매일 아침 덤불로 채집을 나간다.
3. 오전 10시부터 오후 5시 사이에 어른 일부가 숲의 마른 나무에서 땔감을 주워 캠프로 나른다(아직 화면에는 땔감이 그려지지 않는다).
4. 밤에는 화덕이 땔감을 태운다. 여름과 가을에는 겨울에 쓸 땔감을 쌓는다. 겨울밤에 땔감이 떨어지면 주민들이 더 빨리 배고파진다.
5. 정오에 아무것도 하지 않는 주민의 비율이 Pass 4(약 80%)보다 확실히 줄어든다.
6. 저녁이면 대부분의 주민이 화덕 주변에 모인다.
7. 채집이나 땔감 줍기 중에 목이 매우 마르거나 배가 고프면 하던 일을 멈추고 마시거나 먹으러 간다.
8. 어떤 시드에서도 굶주림·탈수 사망이 없고, Pass 4의 성장·세대 교체·계절 흐름이 그대로 유지된다.
9. 콘솔에 SCRIPT ERROR가 없다.

---

# 15. 구현 완료 보고 (REPORT_PASS5.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
끼니와 캠프 대기 규칙, 비축 소비, 마른 나무 배치와 재생, 땔감 분기와 귀가, 화덕 연소와 추위, 긴급 필요 끼어들기, v8·v9·v10 고정 방식

### 기능 설명
플레이어 관점에서 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
X1~X12 표: v10(Pass 4 그대로) / v11, 중앙값·p10·p90·최소·최대
시간대별 일하는 어른 비율과 유휴 비율, 계절별 끼니(비축/덤불)·땔감·불·추위, 조정 라운드별 수치

### 독립 리뷰
지적 사항과 처리 결과

### 화면 검증
캡처 경로, V1~V6 수치, 세 질문에 대한 답

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
마을이 하루 종일 살아 움직이는가(끼니, 채집, 땔감, 저녁 모임)? 아직 시뮬레이션처럼 보이지 않는 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, and the time spent per Part.
