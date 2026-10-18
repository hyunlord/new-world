# GOAL: WorldSim — Revitalize Pass 4: villagers forage for the winter, grow old, and make room for the next generation

Repository `hyunlord/new-world`. Continue the Pass 3 line: `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`, both at `a8611e07aabbb4043f5fac2e8dd14b375a0ada5d` (draft PR #510 into `lead/main`).
This run is executed by **Claude Code**, unattended. Another unattended run (**Art Production**, Codex, branch `codex/art-production-2026-10`) may still be working on the same Mac; section 1 says how to share it.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
git clone https://github.com/hyunlord/new-world /Users/rexxa/work/new-world-pass4
cd /Users/rexxa/work/new-world-pass4
git checkout -b codex/revitalize-2026-10-wip origin/codex/revitalize-2026-10-wip
mkdir -p docs/revitalize && cp ~/Downloads/claude_goal_worldsim_pass4.md docs/revitalize/PASS4_GOAL.md
git update-index --skip-worktree .claude/settings.json && echo '{}' > .claude/settings.json
rm -f .git/hooks/pre-commit .git/hooks/post-commit
tmux new -s pass4
cd /Users/rexxa/work/new-world-pass4
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS4_GOAL.md>
```
The repository's Claude Code hooks (the PreToolUse commit gate and the Stop gate) are neutralized **in this clone only**: the owner replaced `.claude/settings.json` with `{}` and hid that change from git with `--skip-worktree`. Never edit, stage or commit `.claude/settings.json` and never clear its skip-worktree flag. No git hooks are installed in this clone.

The owner drives the run with `/goal`. After each of your turns a small model reads **only the transcript** (it cannot open files or run commands) and decides whether the condition is met; until it is, you get another turn. So the evidence the condition asks for (the section 16 report, the commit list, the `git ls-remote` head) must appear in your final message itself, not only in files.

**This document is `docs/revitalize/PASS4_GOAL.md`.** Commit it unchanged in Part 0. A long run gets its context compacted several times: at the start of every Part, after every compaction or pause, and whenever you are unsure of a number, re-read the relevant section from that file. Never work from a remembered number.

## 0.2 This document overrides CLAUDE.md where they conflict
CLAUDE.md and the nested `rust/**/CLAUDE.md` files stay authoritative for architecture and coding conventions (all simulation logic in Rust, determinism, no `unwrap()` in production, doc comments, tests per module). These instructions in CLAUDE.md **do not apply to this run**:
1. "Session Startup: run `bash tools/harness/install_hooks.sh`" — **never run it**.
2. "Working branch: lead/main (always)", the pre-flight `git checkout lead/main`, and `git push origin lead/main` — **never check out, commit to, or push `lead/main` or `main`**. Work only on the branches in section 2.
3. "Harness Pipeline (Mandatory for All Code Changes)", the `worldsim-harness` skill, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS and STRUCTURAL markers — **do not run the pipeline, do not invoke that skill's pipeline, never create or edit anything under `.harness/audit/`, `.harness/baseline/` or `.harness/reviews/`**. This document's gate (4.4) and tests replace them.
4. If any hook other than the owner's `/goal` ever blocks a command or a stop (it should not; the repository's hooks are neutralized), do not try to satisfy or get around it. Record it in DECISIONS.md, push what you have, and end with the report.
5. Git hooks are separate from Claude Code hooks. First command of the run: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`. Expected: no git hooks, the flag line starting with `S`, and `{}`. If a git hook exists (only possible if the startup script ran by mistake), delete it from this clone and note it in DECISIONS.md. If the settings file is not `{}`, do not edit it: record what you found in DECISIONS.md and continue; if a commit is then blocked, follow item 4.
6. Do not use the repository's `.claude/agents/harness-*` agents for anything; they write harness verdict files. When this document asks for a subagent, use the built-in `general-purpose` type.

## 0.3 Autonomy
1. **Never ask the owner anything.** Do not use AskUserQuestion; do not stop to offer options. Decide, implement, and record the decision with its reason in `docs/revitalize/DECISIONS.md` under `## Pass 4`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part, sub-tasks as you like) and work straight through to Part 7. If a usage limit or anything else pauses the session, on resume: re-read this file's section for the current Part, check the task list, `git log --oneline -5` and any background job logs, then continue; everything up to the last Part is already pushed.
3. **Cap review loops at two rounds.** Fix once, re-review once; on a second objection accept, record, move on.
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.** A partial delivery with an honest report beats a stalled run.
6. **Time rule.** Record the start time and every Part's start and end time in DECISIONS.md. At every Part boundary check elapsed working time (time paused by a usage limit does not count); past **10 hours**, finish the current step, push, go to Part 7.
7. **Three strikes.** If one target still fails after three fix or tuning rounds, stop on it, record evidence and your best hypothesis, keep the best configuration, move on.
8. **Drop order when time is short:** Part 4 (seasons) first, then the elder behaviours of 8.4. Never drop Parts 1, 2, 3 (except 8.4), 5 or 7.
9. **No image generation.** The art run owns that.
10. **Keep bulky output out of your context.** Probe runs, test logs and captures go to files; read summaries, not raw JSONL.
11. **Long commands run in the background.** One Bash tool call is cut off after 10 minutes at most (2 by default). Run anything that can take longer (the full gate, release test runs, probe batches, captures, anything behind the window lock) detached, with a log and an exit-code file, for example `nohup bash -c '<command>; echo $? > /tmp/p4_<name>.exit' > /tmp/p4_<name>.log 2>&1 &`, then poll with short calls (`sleep 240; tail -5 /tmp/p4_<name>.log; cat /tmp/p4_<name>.exit 2>/dev/null`). Before starting a job, check with `pgrep -fl` that the same job is not already running (after a pause or compaction it may be). Copy gate logs you want to keep into `docs/revitalize/verification/p4-<N>/` like Pass 3 did.

---

# ⚠️ 1. MACHINES

**The Mac** (user `rexxa`), clone `/Users/rexxa/work/new-world-pass4`, is where you edit, build, test, probe and capture.
- The art run may be active in `/Users/rexxa/work/new-world-art-prod`. Never read, write or build there; never touch `~/github/new-world-wt/lead`, `/Users/rexxa/work/new-world-capture` or any other checkout, and never kill a process you did not start.
- Share the CPU: `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe `--workers 4`.
- macOS has no `timeout` command. Add `tools/revitalize/run_bounded.sh <seconds> <command...>` (a Python `subprocess.run(..., timeout=…)` wrapper returning 124 on timeout, like `run_bounded` in `capture.sh`) and use it wherever a limit is needed.
- `tools/revitalize/probe.sh` was written for the DGX and fails on macOS: `free -g` does not exist, `df -Pk /home` reports the empty autofs mount, `realpath -m` is GNU-only, and it overwrites `CARGO_BUILD_JOBS` with 10. Before the first probe (Part 0), make it portable: on Darwin skip the memory wait and check free disk with `df -Pk .`; resolve the output path with Python (`os.path.realpath`) instead of `realpath -m`; keep an already exported `CARGO_BUILD_JOBS`; accept `game_v10` later. Linux behaviour stays identical. Raw output keeps going to the ignored `.harness/evidence/revitalize/probe/<label>/`; only `summary.json`/`summary.md` (and evaluator output) are committed under `docs/revitalize/probe/<label>/`, as in Pass 3.
- `capture.sh` and `with_window_lock.sh` already handle macOS (Pass 3 captured on this Mac).
- Windowed Godot only through the existing `tools/revitalize/with_window_lock.sh` (shared lock with the art run), in one command. Headless Godot does not need the lock.
- `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot` (else the first Godot 4.6 from `mdfind "kMDItemCFBundleIdentifier == 'org.godotengine.godot'"`).

**DGX Spark** (`ssh hyunlord@100.70.109.50`) is optional, only for the long probe batches if the Mac is busy: `/home/hyunlord/work/new-world-revitalize`, `git fetch && git checkout --detach <your pushed WIP commit>`, `CARGO_BUILD_JOBS=10`, `--workers 8`, then copy the summaries back. Same safety rules as Pass 3 (no ComfyUI 8188, no dev servers 3100/3200, no other checkouts).

---

# 2. GIT

- Every commit goes to `codex/revitalize-2026-10-wip` immediately; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.4) is green. (The branch names keep their history; the run being Claude Code does not matter.)
- Never force-push or rewrite history. Before every push `git remote -v && git branch --show-current`; after every push `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages: `revitalize(p4-N): <what changed>` (not CLAUDE.md's `[t-000]` format).
- Stage explicit paths only (never `git add -A` or `git add .`); run `git status --short` before every commit and make sure no raw probe output, build output or log outside `docs/revitalize/` is staged.
- **Paths you may change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`.
- **Paths you must not change** (the art run owns them): `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, and the static tests that read `.gd`/`.tscn`/`.gdshader` files, except the minimal struct-literal and Rust-source re-pins allowed in 4.3 (ledger rows marked `shared file`).

---

# 3. CONTEXT

## 3.1 Pass 3 result (verified by the planner from `docs/revitalize/probe/p3-7_final_*`)
Every U target passed on `game_v9`: one settlement per seed every day; zero deaths of any cause in 64 × 10 days and 32 × 60 days; night-in-shelter 97.6 %; free meals closed (v8 had a median 762 of 6,072 meals per seed without food, max 2,146; v9 has 0); v8 reproduction identical.

## 3.2 What the medians hid (planner reruns of `game_v9` seeds 7 and 8 for 60 days at `a8611e07`)
- **Stagnation by chance.** Seed 8 stays at the 24 founders for 45 days with zero births. Harvest ≈ meals ≈ 58 units/day against 120 possible; stock oscillates 0–16, so the reserve gate (10 + 0.5 × 57.6 = 38.8) never opens. The ten bushes together hold 24.5 of their 40 units on average the whole time (capacity 4 each): the food is there, nobody fetches it. It is not distance: seed 8 has the closest bushes of the seeds checked (6, 7, 7 tiles from the hearth). In the 64-seed final run, seeds 8, 21 and 22 had 24–26 people at day 10; the U6 stability maximum of 17 is seed 8 catching up between day 45 and 60.
- **Hoarding by chance.** Seed 7 (all bushes 12–16 tiles away) carries ~25 food at any time, mean home distance 11.6, stock 659 on day 60 and still rising ~10/day, evening-near-hearth share 0.13.
- **A frozen village.** All 32 seeds end at exactly 42 people with exactly 18 births; nobody dies of anything, so after the cap is reached (day ~10 in the median seed) nothing changes.
- **Half the village idle.** Noon idle share median 0.596 at 10 days, 0.488 at 60 days.

## 3.3 Root causes in the code
- Gathering for the store is purely opportunistic: the Gather arm fires only when a food tile lies within `GATHER_SCAN_RADIUS` (8) of the agent's **current** position (`agent_decision.rs`, around line 1147), and nothing sends anyone to fetch food on purpose.
- Depositing is passive: `StockpileDepositSystem` empties inventories only when the carrier happens to be within `SETTLEMENT_PROXIMITY_RADIUS` (5) of the formation tile. Its header says the active "return home to deposit" run ("slice 2-3b") was never built.
- There is no stock target: gathering never stops when the store is full, and never starts when it is empty.
- No death except need saturation or combat, so the plateau is permanent.
- The food regenerates the same amount every day of the year (`FOOD_REGEN_AMOUNT` 1 per `REGEN_INTERVAL` 120 ticks), so a store has no purpose beyond smoothing.
- Bush mechanics that shape the design: each bush holds at most `berry_capacity` = 4 units (`WorldGenConfig::game_default`), so 12 hours of night regrowth mostly hits the cap and is lost; a `Consuming{GatherFood}` pickup takes everything on the tile up to the free room of `INVENTORY_CAPACITY` (10) and removes the tile at 0; `StaleSeekTargetSystem` (126) re-routes only `Seeking{Food|Water|Sleep}` agents whose tile vanished, not `Seeking{GatherFood}`. Check whether a GatherFood seeker whose bush is emptied before it arrives can stand on the empty tile in `Seeking` (the arrival check needs food > 0 and the post-pass keeps an existing `SeekTarget`). With purposeful foraging that case becomes common, so section 7.3 requires re-targeting.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Architecture and determinism (unchanged)
All simulation logic in Rust. New FFI only **adds** keys to existing dictionaries. New collections `BTreeMap`/`BTreeSet`; no order-dependent hash iteration; no transcendental f64 in simulation crates; seeded `splitmix64` streams only; no new crates; no new `CausalEvent`, `DecisionReason`, `TargetKind` or `TerrainType` variants. **One authorized enum addition:** `DeathReason::OldAge` (code 3, `"old_age"`) in `sim-core/src/causal/event.rs`, with every exhaustive match updated. Time windows use integer minute-of-day as in Pass 3. Day index `d = (current_tick + clock_offset_ticks) / ticks_per_day`.

## 4.2 Versioning — v8 and v9 frozen, v10 new
- `WorldRules::game_v8()`, `game_v9()`, `WorldGenConfig::game_default()` and `game_v9()` keep their behaviour exactly; new fields take values that reproduce them bit-for-bit (proved in Parts 0 and 7).
- Add in Part 1 `WorldRules::game_v10()` = a copy of `game_v9()` except sustain `utilization: 0.705`, `WorldGenConfig::game_v10(seed)` (= `game_v9` with `berry_bushes: 12`, everything else equal), `game_v10_scene(seed)`, probe scene `game_v10`. With 12 bushes × 12 = 144 nominal units/day, 0.705 keeps the plateau at 42 exactly like v9 (`(m + 1) × 2.4 ≤ 101.52`, away from a floating-point boundary). Parts 2–4 then add their rules to `game_v10()` only; Part 4 changes `utilization` to 0.81 against the annual mean supply. If Part 4 is dropped, 0.705 stays.
- The live game switches to v10 in both `world_node.rs` constructors (`build_world_with(&WorldGenConfig::game_v10(..), WorldRules::game_v10())`).

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged and draws no extra RNG. `harness_population_legacy_scene_pinned`, `harness_population_game_world_survives_3_days`, all Pass 2 and Pass 3 tests pass unchanged. **Never change `sim-test/src/probe/fingerprint.rs`**: it is the yardstick for legacy/v8/v9 invariance (`harness_village_v8_unchanged` and the Pass 4 `*_v9_unchanged` tests); assert v10-only state (old-age deaths, `Life`, `Elder`, stock target) separately. Allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 4`: struct-literal field additions with legacy values (rules, `Tallies`); probe schema tests moving to schema 4; Rust-source static assertions that pin code this goal deliberately changes (the live world constructor, additive FFI keys, the `DeathReason` match), re-pinned and never weakened.

## 4.4 The gate (required before the green branch moves)
```bash
export CARGO_BUILD_JOBS=6
cd rust
cargo test --workspace -- --test-threads=6
cargo test --workspace --release -- --test-threads=6
cargo clippy --workspace -- -D warnings
cargo clippy --workspace --all-targets -- -D warnings
grep -rnE '\.(exp|exp2|ln|log2|log10|powf|powi|sin|cos|tan|tanh|atan2|hypot|cbrt)\(' \
  crates/sim-core/src crates/sim-systems/src crates/sim-engine/src     # expect 0 hits
cd ..
GODOT_BIN=$GODOT_BIN bash tools/harness/gdscript_strict_check.sh      # exit 0 (read-only checker; allowed)
(cd rust && cargo build -p sim-bridge)
bash tools/revitalize/run_bounded.sh 600 $GODOT_BIN --headless --path . --quit-after 900 > /tmp/pass4_smoke.log 2>&1; echo "exit $?"
grep -E 'SCRIPT ERROR|ERROR:' /tmp/pass4_smoke.log | grep -viE 'alsa|audio'   # expect no output; exit line 0
python3 -m unittest discover -s tools/revitalize -p 'test_*.py'
```

---

# 5. PART 0 — Baseline and frozen references

1. `git status --short && git log --oneline -3` at `a8611e07`; record `PASS4_BASE_SHA`, the start time and `uname -a` in DECISIONS.md. Commit `docs/revitalize/PASS4_GOAL.md` unchanged together with the `probe.sh` portability fix and `run_bounded.sh` (section 1).
2. Release build; `--scene game_v9 --seeds 0..64 --days 10 --check-determinism` with label `p4-0_v9_repro`; compare with `docs/revitalize/probe/p3-7_final_10day/summary.json`: every simulation metric identical (timing excepted). Same for `--scene game --seeds 0..64 --days 10` (label `p4-0_v8_repro`) against `p3-7_final_v8`, after checking in its `summary.md` that it was run with the same seeds and days; if not, use whichever stored v8 summary matches and say which. Pass 3's summaries were produced on the DGX (Linux aarch64) and yours on the Mac (macOS aarch64); the simulation is meant to be bit-identical across both. If a metric differs, rerun the same command on the DGX at `a8611e07`: if the DGX reproduces Pass 3, record a cross-platform determinism defect in DECISIONS.md with the first differing seed and metric, use your Mac numbers as the reference from then on, and continue; if the DGX also differs, stop until explained.
3. Record `sim_test::probe::fingerprint(&engine)` of `game_v9_scene(seed)` for seeds 0..4 after 1,440 ticks, before any code change (the same pattern as `harness_village_v8_unchanged`); they become the constants of the Pass 4 `*_v9_unchanged` tests.
4. Rerun the planner's evidence for yourself: `--scene game_v9 --seeds 7..9 --days 60 --clock-samples 02:00,12:00,19:30` and write the seed 7 and seed 8 trajectories (alive, births, stock, harvested/day, meals/day, carried food, `food_tiles_total`) in METRICS.md under `## Pass 4 baseline`.

---

# 6. PART 1 — Instruments

- Probe `--scene game_v10`; `--days` up to 150.
- `Tallies` (display-only, never read by simulation logic): `deaths_old_age`, `forage_trips` (Forage arm firings), `haul_deposits` (deposit events by haulers), `regen_lost_at_cap` (food regrowth units discarded by the `min(max)` clamp in `regen_channel`, food channel only; counted in every scene, so the v9 baseline shows how much night regrowth the 4-unit cap wastes).
- Sample schema **version 4** (every v3 key keeps its meaning; never emit null): `season` (0 spring, 1 summer, 2 autumn, 3 winter; 0 when seasons are off), `day_of_season`, `elders`, `stock_target`, `stock_in_hand` (stockpile + food carried by members), `foraging_now` (agents on a forage trip), `hauling_now`, `deaths_old_age`, `forage_trips_total`, `regen_lost_at_cap_total`, `mean_age_days` (0 without aging).
- The death census reads the actual `DeathReason` from the despawn path (add it to the observer), cross-checks with `Tallies`, and reports `old_age` as a cause with `age_days_at_death`.
- `summary.md` gains **Seasons** (per-season medians of supply, harvest, need, stock, regrowth lost at the cap), **Demography** (births, deaths by cause, children/adults/elders, mean age) and **Foraging** sections.
- `tools/revitalize/evaluate_pass4.py` (+ `test_evaluate_pass4.py`) computes every W target of section 10 from raw JSONL, like `evaluate_pass3.py`.

Baseline: `--scene game_v9 --seeds 0..32 --days 120 --check-determinism --clock-samples 02:00,12:00,19:30` → `docs/revitalize/probe/p4-1_v9_120day/` (expected: frozen at 42, stagnant seeds). Commit, push WIP.

---

# 7. PART 2 — Forage on purpose, haul home, stop when the store is full

## 7.1 Rules (`sim-core/src/rules.rs`)
```rust
pub struct AgentRules {
    // existing fields unchanged
    /// Purposeful daytime foraging and hauling. None = Pass 3 behaviour.
    pub forage: Option<ForageRule>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForageRule {
    pub start_minute: u32,          // v10 360  (06:00)
    pub end_minute: u32,            // v10 1080 (18:00)
    pub forage_radius: u32,         // v10 16, Chebyshev from the settlement formation tile
    pub haul_threshold: u32,        // v10 4 food carried → go home (one full bush at capacity 4)
    pub max_foragers_per_bush: u32, // v10 1 (hard cap; a 4-unit bush is emptied by one pickup)
}
pub struct SocietyRules {
    // existing fields unchanged
    /// Store target that drives foraging. None = Pass 3 behaviour.
    pub stock_target: Option<StockTargetRule>,
}
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StockTargetRule {
    pub base_days: f64,        // v10 1.5 days of current need
    pub winter_prep: bool,     // v10 true (uses Part 4 seasons; ignored when seasons are off)
    pub hoard_cap_factor: f64, // v10 2.0
}
```
Legacy, v8 and v9: `None`.

## 7.2 Target
In the settlement module (next to `food_economy.rs`), one public function used by the decision system, the probe and tests:
```
need      = members_need_per_day(world, resources, sid)                 // Pass 3 helper, actual rates
target    = base_days × need
          + (if seasons on and winter_prep and season ∈ {summer, autumn})
              days_per_season × max(0, need − winter_supply_per_day(sid))   // Part 4
stock_in_hand = stockpile Food + Σ Food carried by live members (sorted AgentId)
```
Compute `target` and `stock_in_hand` once per settlement per decision tick, before the agent loop (sorted by settlement id), not per agent; W11 limits the cost.

## 7.3 The Forage arm (`agent_decision.rs`, Idle cascade)
New arm after Combat and **before** the existing Gather arm, for agents with no `Child` and no `Elder` marker, when all hold: the minute is inside the window; the agent is a settlement member; carried Food < `haul_threshold`; inventory has room; `stock_in_hand < target`; at least one food tile with amount ≥ 1 lies within `forage_radius` of the formation tile.
- Bush choice, deterministic: among food tiles with amount ≥ 1 within `forage_radius` of the formation tile and with `claims < max_foragers_per_bush`, minimise `(claims, chebyshev(agent, bush), y, x)`, where `claims` = agents already in `Seeking{GatherFood}` whose `SeekTarget` is that bush (computed once before the agent loop in sorted order and incremented as this tick assigns foragers). If no bush qualifies, the arm does not fire and the cascade continues.
- The agent enters `Seeking{GatherFood}` with its `SeekTarget` on that bush; the GatherFood target post-pass must keep a forage target beyond `GATHER_SCAN_RADIUS` (today it resolves a missing target within 8 of the agent). `Tallies.forage_trips += 1`. No causal event (the Gather arm emits none either).
- **Re-target, never freeze.** Under the rule, every `Seeking{GatherFood}` agent whose `SeekTarget` tile holds no food re-targets in the post-pass by the same choice rule (claims recomputed, that tile excluded); if nothing qualifies it becomes `Idle` and drops the `SeekTarget`. A forager may visit several bushes before its load reaches `haul_threshold`.

## 7.4 Bound the opportunistic Gather arm
Under the rule, the existing Gather arm also requires carried Food < `haul_threshold` and `stock_in_hand < hoard_cap_factor × target`. Without the rule it is unchanged.

## 7.5 Haul home (`AgentMovementSystem`)
Under the rule, an `Idle` member without `Child` whose carried Food ≥ `haul_threshold`, or ≥ 1 outside the forage window, and who is farther than `SETTLEMENT_PROXIMITY_RADIUS` from its formation tile, takes a **directed** signum step toward the formation tile instead of a Brownian step (no RNG draw for that step; this changes RNG consumption only under v10). `StockpileDepositSystem` then drains the load as today; count `haul_deposits` there when the depositor arrived by hauling.

## 7.6 Tests — new file `harness_revitalize_pass4_forage.rs`
- `harness_forage_seed8_no_longer_stagnates`: the Pass 3 seed-8 world with only this Part's rules added (`build_world_with(&WorldGenConfig::game_v9(8), rules)` where `rules` = `game_v9()` plus v10's `forage` and `stock_target`), so the map is exactly the one that stagnated: stock reaches ≥ 39 by day 3 and a birth happens by day 4.
- `harness_forage_stops_at_target`: stock set to 3 × target → no Forage arm firing for one day, and no opportunistic gathering while `stock_in_hand ≥ 2 × target`.
- `harness_haul_home_deposits`: an Idle member carrying 6 Food 12 tiles away arrives within the deposit radius and its Food reaches the stockpile within 16 × `move_interval_ticks` + 20 ticks.
- `harness_forage_claims_cap`: with `max_foragers_per_bush = 1`, three eligible adults and two stocked bushes → exactly two forage, one per bush, and the third does not; with 2, four adults → two per bush.
- `harness_forage_retargets_when_bush_empties`: a forager's bush is emptied before it arrives → it re-targets another stocked bush (or becomes Idle when none qualifies) and is never in `Seeking{GatherFood}` on an empty tile for more than one tick.
- `harness_forage_v9_unchanged`: fingerprints of Part 0.3 reproduce.

Commit, push WIP; fast-forward green if the gate is green.

---

# 8. PART 3 — Grow old, die, be replaced

## 8.1 Rule and components
```rust
pub struct SocietyRules {
    // existing fields unchanged
    /// Aging and natural death. None = nobody ages (legacy, v8, v9).
    pub aging: Option<AgingRule>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AgingRule {
    pub elder_age_ticks: u64,             // v10 46 days = 66_240
    pub lifespan_mean_ticks: u64,         // v10 58 days = 83_520
    pub lifespan_spread_ticks: u64,       // v10  8 days = 11_520
    pub founder_age_min_ticks: u64,       // v10  6 days =  8_640
    pub founder_age_max_ticks: u64,       // v10 40 days = 57_600
    pub founder_min_remaining_ticks: u64, // v10  3 days =  4_320
    pub elder_tether_radius: u32,         // v10 4
}
```
New components in `sim-core/src/components/` (documented in `rust/crates/sim-core/CLAUDE.md`): `Life { born_tick: i64, lifespan_ticks: u64 }` and the zero-sized marker `Elder`. Pass 3's `Child` stays as it is.

## 8.2 Deterministic lifespans (integer only)
Stream per agent, following the newborn-seed convention in `settlement_system.rs` (there is no free `splitmix64` function; `MovementRng::next_u64` is splitmix64): `MovementRng::new(world_seed ^ agent_id.wrapping_mul(0x9E37_79B9_7F4A_7C15).wrapping_add(AGING_RNG_SALT))` with a new constant `AGING_RNG_SALT = 0x4C1F_E5A1_0000_0001` (distinct from `BIRTH_RNG_SALT` and every other salt; check with grep and record it). This stream is separate from the agent's movement `MovementRng` component and never advances it. Three draws `dᵢ = next_u64() % (2·spread + 1)`; `lifespan = mean − spread + (d₁ + d₂ + d₃) / 3` (integer division), so lifespans lie in `[mean − spread, mean + spread]` and cluster near the mean.
- **Founders** (worldgen, under the rule): `age = min + next_u64() % (max − min + 1)` from the same stream, before the lifespan draws; `born_tick = −(age as i64)`; if `lifespan < age + min_remaining` then `lifespan = age + min_remaining + next_u64() % 2880`.
- **Newborns**: `born_tick = tick as i64`, lifespan from the draws; `Child` exactly as in Pass 3.

## 8.3 `AgingSystem` (priority 146, interval 60)
In `AgentId` order: `age = tick as i64 − born_tick`. If `age ≥ lifespan` → `survival::despawn_agent(…, DeathReason::OldAge, tick)` (`Tallies.deaths_old_age += 1`, recent deaths reason 3, the usual roster and bed cleanup). Else if `age ≥ elder_age` and no `Elder` → insert `Elder`. Structural changes go through a collected list after the query.

## 8.4 Elders
Elders skip the Construction, Forage, opportunistic Gather and Combat arms; their idle tether is `elder_tether_radius` around the formation tile at all hours (they stay at the camp); everything else as adults (needs, sleep, social, eating, drinking). Elders still count in `members_need_per_day` and in the sustain and reserve gates.

## 8.5 FFI (additive)
- `get_agent_snapshot`: `life_stages` gains value **2 = elder** (0 adult, 1 child unchanged); new `ages_days: PackedInt32Array` (whole days, −1 without `Life`).
- `get_tallies`: `deaths_old_age`.
- `get_recent_deaths`: reason 3 = old age.

## 8.6 Tests — new file `harness_revitalize_pass4_aging.rs`
- `harness_aging_lifespans_bounded_and_deterministic`: 1,000 agent ids → every lifespan in `[mean − spread, mean + spread]`, mean within ±1 day of 58; twins identical.
- `harness_aging_founders`: every founder age in [6, 40] days; no founder dies before day 3.
- `harness_aging_old_age_death_and_replacement`: `game_v10_scene(1)` for 60 days → at least one `OldAge` death, a birth after it, and bed and roster cleanup verified.
- `harness_aging_elders_stay_home`: an `Elder` never enters Construction, Forage or opportunistic Gather and stays within `elder_tether_radius` while Idle.
- `harness_aging_v9_unchanged`: fingerprints of Part 0.3 reproduce.

Commit, push WIP; fast-forward green if the gate is green.

---

# 9. PART 4 — Seasons that make the store matter

## 9.1 Rule
```rust
pub struct WorldRules {
    // existing fields unchanged
    pub seasons: Option<SeasonRule>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeasonRule {
    pub days_per_season: u32,     // v10 7 (a year is 28 days)
    pub regen_permille: [u16; 4], // v10 [750, 1250, 1000, 500] spring, summer, autumn, winter
}
```
`season(d) = (d / days_per_season) % 4`; day 1 (d = 0) is the first day of spring.

## 9.2 Regeneration
Under the rule, `ResourceRegenSystem` replaces the fixed food amount with an exact integer accumulator `SimResources::food_regen_acc: u32` (0 otherwise, untouched by legacy): each pulse `acc += regen_permille[season]; units = acc / 1000; acc %= 1000;` and every registered food source gains `units` (capped at its max as today). When `units == 0` skip the food channel entirely: `regen_channel` with amount 0 would insert zero-valued tiles, and everywhere else in the code an absent tile means empty. The season comes from `resources.current_tick` and the clock offset as in 4.1. Water and sleep unchanged. Per bush per full day: spring 9, summer 15, autumn 12, winter 6 units (every day's total is a multiple of 1000, so the carried remainder never changes a day's count).

## 9.3 Supply in the formulas
`supply_per_day(sid)` gains a season-aware form: current-season supply = bushes × 12 × permille[season] / 1000; `annual_mean_supply` = bushes × 12 × mean(permille) / 1000; `winter_supply_per_day` = bushes × 12 × permille[3] / 1000. The Provisioned sustain gate uses `annual_mean_supply` when seasons are on. `game_v10()`: `utilization: 0.81` (12 bushes → annual mean 126/day → `(members + 1) × 2.4 ≤ 102.06` → the village plateaus at 42; 0.81 rather than 0.80 keeps the plateau off an exact floating-point boundary), `reserve_days: 0.5`, `forage_radius: 16`, `cooldown_ticks: 720`.

These are nominal numbers. The harvestable supply is lower because a bush saturates at 4 units overnight (in summer about 7.5 units of night regrowth meet a 4-unit cap). The planner's rough estimate for 12 bushes kept picked during the day: spring ≈ 102, summer ≈ 138, autumn ≈ 120, winter ≈ 72 units/day against a need of 100.8 at 42 people, so a summer-and-autumn surplus of about 395 against a winter deficit of about 202. Spring is nearly break-even, which is why the base store matters. Use `regen_lost_at_cap` to see the real loss; if W3 or W7 fail because of it, the knobs are `berry_capacity` and `berry_bushes` (section 10).

## 9.4 FFI (additive)
`get_time_snapshot`: `season` (0–3), `day_of_season` (1-based), `days_per_season` (0 without seasons).

## 9.5 Tests — new file `harness_revitalize_pass4_seasons.rs`
- `harness_season_regen_schedule`: 28 full days on one bush kept empty by a test drain → units per day exactly [9, 15, 12, 6] per season, and no zero-valued food tile ever appears.
- `harness_season_target_includes_winter`: in summer and autumn the target equals base + 7 × max(0, need − winter supply); in winter and spring it equals base.
- `harness_season_survives_first_winter`: `game_v10_scene(1)` for 35 days → zero starvation and stock > 0 at the end of winter.
- `harness_season_v9_unchanged`: fingerprints of Part 0.3 reproduce.

Commit, push WIP; fast-forward green if the gate is green.

---

# 10. PART 5 — Measure and tune (three rounds maximum)

Runs (release): `--scene game_v10 --seeds 0..64 --days 10 --check-determinism` and `--scene game_v10 --seeds 0..32 --days 120 --check-determinism --clock-samples 02:00,12:00,19:30` → `docs/revitalize/probe/p4-5_round<N>_{10day,120day}/`; evaluate with `evaluate_pass4.py`.

Targets (game_v10; table order = priority):

| # | target |
|---|---|
| W1 | determinism 64/64 (10 d) and 32/32 (120 d); legacy pin; v8 and v9 reproductions identical |
| W2 | dehydration: 10 d median 0, max ≤ 1; 120 d total median 0, max ≤ 2 |
| W3 | famine: 10 d zero in every seed; 120 d total median 0, max ≤ 3 (four winters) |
| W4 | one settlement at every day sample from day 1, every seed |
| W5 | no stagnation: a birth by day 4 in every seed; day-10 alive ≥ 30 in every seed; longest run of days with `members ≤ sustainable_adults − 3` and no birth ≤ 4 (days 2–120) |
| W6 | turnover: ≥ 1 old-age death by day 30 in every seed; over days 30–120 births ≥ deaths − 3 in every seed; every day sample from day 30 has alive in [32, 44]; day-120 child share median in [0.03, 0.20], elder share median in [0.08, 0.35] |
| W7 | store: stock at the last sample of each autumn median ≥ 0.8 × 7 × max(0, need − winter supply); max stock in every seed ≤ 3 × the largest target seen in that seed |
| W8 | activity: noon idle share median ≤ 0.45 over spring–autumn days (Pass 3: 0.596 / 0.488); forage trips per day median ≥ 10 in summer and autumn |
| W9 | rhythm: night-in-shelter median ≥ 0.70; evening-near-hearth median ≥ 0.50 and ≥ 0.30 in every seed (Pass 3 minimum 0.13) |
| W10 | no regressions: T6a ≤ 1 %; day-10 home distance median ≤ 7.0 and every seed ≤ 10 (foraging takes people out on purpose); social completed per agent-day ≥ 0.40 and > 0 in every seed |
| W11 | ms per agent-tick ≤ 1.5 × the Pass 3 game_v9 value measured on the same host in this run |

Knobs (only these, all in `game_v10()` / `WorldGenConfig::game_v10`): `berry_bushes` 10–14; `berry_capacity` 4–8; `utilization` 0.70–0.90 (re-derive the plateau whenever bushes or permille change, keep it at 42 and off a floating-point boundary); `reserve_days` 0.25–1.0; `base_days` 1.0–2.5; `hoard_cap_factor` 1.5–3.0; `haul_threshold` 2–8; `max_foragers_per_bush` 1–2; forage window start 300–480 and end 960–1140; `regen_permille[3]` (winter) 300–700 with the four values' mean kept at 875; `days_per_season` 5–10; lifespan mean 50–70 and spread 4–12 days; `elder_age` 40–52 days. Record every round with all numbers in METRICS.md; keep the best configuration in `game_v10()`. Commit, push WIP; fast-forward green if the gate is green.

---

# 11. PART 6 — Look at the screen (existing tooling, no GDScript edits)

In your own clone, after building `sim-bridge`, in one command:
```bash
CAPTURE_DOCS_DIR=docs/revitalize/shots/pass4 bash tools/revitalize/with_window_lock.sh 1500 bash tools/revitalize/capture.sh final
```
This waits for the window lock if the art run holds it, so run it in the background (0.3 item 11). Commit the 960 × 540 JPEGs and `visual_qa.md` (V1–V6). The renderer is still the Pass 2 renderer (carried food, elders and seasons are not drawn); do not fix rendering. Answer in the report, one sentence each, from `d2_1200_F2`, `d2_2200_F2N` and `d6_1200_F2`: where people are at noon (at bushes, on the way, at the camp); roughly how many stand at the camp at noon compared with Pass 3's `docs/revitalize/shots/pass3`; whether the camp still reads as one village at 22:00.

---

# 12. PART 7 — Independent review, final verification, report

1. **Independent review.** Spawn a review subagent (built-in `general-purpose` type, fresh context, not a `harness-*` agent). Give it `git diff a8611e07..HEAD -- rust/ tools/revitalize/`, sections 4 and 7–10 of this document, and the final W table; ask for concrete defects (determinism, v8/v9 invariance, formula errors, untested branches) with file:line evidence. Fix confirmed defects (two rounds maximum) and record every finding and its disposition in DECISIONS.md.
2. Full gate (4.4) on the final WIP head; totals versus Pass 3 (1,850 debug and 1,850 release passed).
3. Final probes: legacy reproduces Pass 1 numbers; `game` (v8) and `game_v9` reproduce Pass 3 exactly on every key both summaries have (schema 4 only adds keys); `game_v10` 64 × 10 and 32 × 120 with census and clock samples.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise leave it and say exactly what is red.
5. `docs/revitalize/REPORT_PASS4.md` in Korean (section 16 format), answering each item of section 15 with its evidence level (simulation test, probe metric, capture, or not verified). Append `## Pass 4` to METRICS.md and DECISIONS.md.
6. Update the PR #510 description with a short Pass 4 section (`gh pr edit 510 --body-file …`). If `gh` is not authenticated, skip and say so. Do not open a new PR.

---

# 13. LOCALIZATION AND THE ART RUN

No GDScript and no user-visible text change in this run. List these as **merge touch-points for the art run** in the report: elders arrive as `life_stages == 2` (draw them older; the art run currently treats any value other than 1 as an adult); the HUD death count must add `deaths_old_age`; `season` and `day_of_season` are available for a seasonal tint and a HUD season label (new localization keys then); `ages_days` is available for the inspector.

# 14. OUT OF SCOPE

Rendering and HUD; families and kinship; names; disease, accidents and combat changes; new food types (fishing, hunting, farming); food spoilage; weather; new settlements or migration; map size; merging into `lead/main`; the paths in section 2 owned by the art run.

---

# 15. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)

1. 낮(06~18시)에는 어른 주민 여럿이 먼 덤불까지 걸어가 열매를 따고 모닥불 쪽으로 돌아온다. 한 덤불에 여러 명이 몰리지 않고 흩어진다(들고 있는 열매는 아직 그려지지 않는다).
2. 비축이 넉넉하면 채집하러 나가는 주민이 줄고, 모자라면 다시 늘어난다. 비축이 끝없이 쌓이지 않는다.
3. 어떤 시드로 시작해도 처음 며칠 안에 아이가 태어나고 마을이 커진다. 창립 인원 그대로 몇십 일 멈춰 있는 마을이 없다.
4. 계절이 7일마다 바뀌고, 여름과 가을에는 비축이 크게 늘며 겨울에는 줄어든다. 겨울에도 굶어 죽는 주민이 없다(화면에 계절 표시는 아직 없으니 프로브 요약의 계절 표로 확인한다).
5. 주민은 나이를 먹는다. 46일쯤부터 노인이 되어 모닥불 근처에 머물고 일하러 나가지 않으며, 50~66일 사이에 늙어서 죽는다(노인 외형은 아직 어른과 같다).
6. 누군가 늙어 죽으면 그 뒤에 아이가 태어나 자리를 채운다. 인구는 40명 안팎에서 오르내리며 고정되지 않는다.
7. 정오에 서 있기만 하는 주민의 비율이 이전(약 절반)보다 줄어든다.
8. 밤에는 여전히 쉼터에서 자고, 저녁에는 모닥불 주변에 모인다.
9. 렌더링은 아직 이전과 같다(노인 외형, 계절 표시, 늙어 죽은 주민 수의 HUD 반영은 아트 작업에서 한다).
10. 콘솔에 SCRIPT ERROR가 없다.

---

# 16. 구현 완료 보고 (REPORT_PASS4.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
비축 목표 공식, 채집 분기와 덤불 선택, 빈 덤불 재지정, 귀가 운반, 수명 추첨과 노인 단계, 계절 재생 누산기, v8·v9 고정 방식

### 기능 설명
플레이어 관점에서 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
W1~W11 표: v9(Pass 3 그대로) / v10, 중앙값·p10·p90·최소·최대
120일 인구·출생·사망 원인·연령 구성 추이, 계절별 공급·수확·비축, 시드 7·8 전후 비교, 조정 라운드별 수치

### 독립 리뷰
지적 사항과 처리 결과

### 화면 검증
캡처 경로, V1~V6 수치, 세 질문에 대한 답

### 아트 작업과 맞물리는 지점
13절 목록

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 사용량 한도나 다른 이유로 멈춘 구간, 컨텍스트 압축 뒤 목표 파일을 다시 읽은 시점, 백그라운드로 돌린 긴 작업 목록

### 확인된 제한사항

### Harness 결과
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트)

### 정직한 판단
120일 동안 마을이 살아 움직이는가(성장, 비축, 세대 교체)? 아직 시뮬레이션처럼 보이지 않는 부분은 무엇인가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, and the time spent per Part.
