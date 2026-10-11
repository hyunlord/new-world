# GOAL: WorldSim — Revitalize Pass 9: counterfactuals you can trust — keyed randomness, stable identities, noise forks and the variance they remove

Repository `hyunlord/new-world`. Continue the Pass 8 line: `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`, both at `4c35e29573bc28878de0cfed9ac67e0df0aa4c48` (draft PR #510 into `lead/main`).
This run is executed by **Claude Code**, unattended, on the Mac. A learned-model run (**WM-1.1**, Claude Code, clone `/Users/rexxa/work/new-world-wm1`, training on the DGX) may work at the same time; it touches nothing of yours. No art run is active.

This is the first simulation pass of the **research track** (planner's research plan v1, "WorldSim 연구 설계 심화"). Its job is to make the counterfactual ground truth trustworthy before any hypothesis is tested. It changes no gameplay balance.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4            # the Pass 4–8 clone, hooks already neutralized
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 4c35e295
cp ~/Downloads/claude_goal_worldsim_pass9.md docs/revitalize/PASS9_GOAL.md
tmux new -s pass9
cd /Users/rexxa/work/new-world-pass4
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS9_GOAL.md>
```
The repository's Claude Code hooks are neutralized **in this clone only**: `.claude/settings.json` holds `{}` and is hidden from git with `--skip-worktree`. Never edit, stage or commit it and never clear that flag. No git hooks are installed. The local `.gitignore` edit and `.ignore` belong to unrelated graft tooling; leave them uncommitted.

**This document is `docs/revitalize/PASS9_GOAL.md`.** Commit it unchanged in Part 0. At the start of every Part, after every compaction or pause, and whenever you are unsure of a number, re-read the relevant section from that file.

The owner drives the run with `/goal`: after each turn a small model reads **only the transcript** and decides whether the condition is met. The evidence it asks for (the section 14 report, the commit list, the `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
CLAUDE.md and the nested `rust/**/CLAUDE.md` files stay authoritative for architecture and conventions (simulation logic in Rust, determinism, no `unwrap()` in production, doc comments, tests per module). These do **not** apply to this run:
1. "Session Startup: run `install_hooks.sh`" — never run it.
2. "Working branch: lead/main", the pre-flight checkout and `git push origin lead/main` — never check out, commit to or push `lead/main` or `main`.
3. The harness pipeline, the `worldsim-harness` skill's pipeline, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS — not used; never create or edit anything under `.harness/audit/`, `.harness/baseline/`, `.harness/reviews/`. This document's gate (4.4) replaces them.
4. If any hook other than the owner's `/goal` blocks a command or a stop, do not get around it: record it in DECISIONS.md, push what you have, end with the report.
5. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`. Expected: no git hooks, a line starting with `S`, `{}`.
6. Never use the repository's `.claude/agents/harness-*` agents; when this document asks for a subagent, use the built-in `general-purpose` type.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 9`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5` and background job logs, then continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.**
6. **Time rule:** record start time and each Part's start and end. Past **10 hours**, finish the current step, push, go to Part 6. Passes 7 and 8 used about 2 hours each: be thorough, especially in the review.
7. **Three strikes** per failing target, then keep the best state and record evidence and hypothesis.
8. **Drop order when time is short:** the divergence-horizon study (8.4) → the v14 single-fork comparison of 8.3 → the held-out half of the v15 health check (9.1, keep the tuning half). Never drop Parts 0–3, the placebo gates (8.1) or the variance-reduction measurement (8.2).
9. **No image generation, no neural-network training.**
10. **Keep bulky output out of context:** probe runs, exports, test logs go to files; read summaries.
11. **Long commands run in the background** (`nohup bash -c '<command>; echo $? > /tmp/p9_<name>.exit' > /tmp/p9_<name>.log 2>&1 &`), polled with short calls; check `pgrep -fl` before starting a job again. Gate logs in `docs/revitalize/verification/p9-<N>/`.

---

# ⚠️ 1. MACHINES
- **The Mac** clone `/Users/rexxa/work/new-world-pass4`. Never read, write or build in `/Users/rexxa/work/new-world-wm1`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe and recorder `--workers 4`.
- Fork experiments write under `/Users/rexxa/work/worldsim-wm-data/p9/` (outside git). `v0/` and `v1/` there are **read-only** (8.3 reads v1's C_cf episodes); never write, move or delete anything in them.
- Python: system `python3` with numpy; no scipy, torch or pandas.
- Headless Godot only (smoke). `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Timing runs (Y8) back to back with nothing heavy running (`pgrep -fl 'Godot|cargo|worldsim_probe|worldsim_wm|python3'`); WM-1.1 keeps its heavy work on the DGX.
- **DGX:** not used by this pass (WM-1.1 owns it).

---

# 2. GIT
- Every commit goes to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.4) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p9-N): <what changed>`. Stage explicit paths only.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`, `docs/worldmodel/**`, `tools/worldmodel/**` — **except** `tools/worldmodel/wm1/**`, `docs/worldmodel/wm1/**` (WM-1's branch) and everything v0/v1 (datasets, manifests, `BENCH_v0/v1.md`, the committed samples).
- **Must not change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, static tests that read `.gd`/`.tscn`/`.gdshader`, except minimal Rust-source re-pins allowed in 4.3.
- **Merge note for later:** the art branch (`codex/art-production-2026-10`, `cd998d38`) added a read-only FFI getter `get_intervention_state` to `rust/crates/sim-bridge/src/ffi/world_node.rs`. Do not add a function of that name; keep your `world_node.rs` changes small (the constructors' version switch), so the later merge is mechanical.

---

# 3. CONTEXT

## 3.1 Where the research stands (planner's review)
- Pass 7–8 built a deterministic village with a hidden winter severity, seven world-level interventions, a recorder and two datasets. A same-seed fork gives a byte-identical prefix and a divergent suffix.
- **WM-1** (first learned models on v0): no model learned any intervention effect (interventions are 0.09 % of training rows); a causal transformer beat every baseline on factual rollouts (1 seed, to be confirmed by WM-1.1 against a calendar baseline).
- The research plan's first gates ask whether a fork's difference is the **effect of the intervention** or partly **noise that moved**. Literature: common random numbers only reduce variance when the same random number drives the same event in both arms (Glasserman & Yao 1992); event-keyed draws make that hold by construction (Klein et al., arXiv 2409.02086, Starsim; Buffalo, Pearson & Klein, arXiv 2603.11084: draws as a pure function of (seed, event id), newborn ids derived from parent and birth index). A placebo fork must be exactly zero; resampling the noise after a fork gives an effect distribution instead of one sample.

## 3.2 The randomness that exists today (planner's survey of `4c35e295`, verify it in Part 0)
1. **Movement:** each agent owns a sequential `MovementRng` (splitmix64) component; `AgentMovementSystem` draws from it only for an **idle** Brownian step (`movement.rs`: one pause draw when `idle_pause_chance > 0`; if the pause succeeds, no dx/dy are drawn; otherwise `next_step` for dx and dy, currently `% 3`). Seeking, hauling home and consuming draw nothing; entities with no `AgentState` also take Brownian steps. So the number of draws an agent has taken depends on how often it was idle: once an intervention changes an agent's idle hours, every later step of that agent uses shifted numbers.
2. **Newborns:** `settlement_system.rs` seeds the newborn's `MovementRng` from `new_agent_id` (`BIRTH_RNG_SALT`); `AgentId`s are issued monotonically (`SimResources::issue_agent_id`). If an intervention moves, adds or removes one birth, every later newborn gets a different id, hence different movement numbers.
3. **Lifespans:** `aging.rs::aging_stream(world_seed, agent_id)` draws each resident's lifespan from its id — same consequence for newborns after a shifted birth.
4. **Worldgen** (`camp.rs` incl. founder needs and the deadwood shuffle, `noise.rs`, `terrain.rs`'s `h64`, and the bootstrap lattice in `sim-bridge/src/ffi/world_node.rs`): fixed at tick 0; identical in both arms.
5. **Weather** (severity, night burn, day fall): pure hashes of (seed, year/day) — already event-keyed; identical in both arms.
6. Decisions (`sim-systems/src/runtime/decision/agent_decision.rs`) draw no random numbers.
7. **Iteration order (hidden noise):** the main decision loop (`agent_decision.rs` ≈ L904, `query.iter()`) visits agents in hecs order, and bush claims (≈ L1404, `claims.entry(bush) += 1`) and food-tile decrements (≈ L1866) are first-come in that order. hecs order depends on history (swap-remove on despawn, archetype moves when components such as `SeekTarget`, `Child`, `Elder`, `Cold` are added or removed), so it can differ between the arms of a fork and is not covered by any key. Other loops in the same file already sort by `AgentId` (≈ L631).
If Part 0 finds any other source of randomness or order dependence (a hash of an `Entity`, an iteration over a `HashMap` that feeds a decision), record it and include it in the keyed scheme.

## 3.3 This pass
`game_v15` = `game_v14` + keyed draws + lineage keys. Same balance, same mechanics; only *which* random number an event receives changes. Plus: noise forks (resample the randomness after a tick), placebo gates, and the first measurement of how much pairing reduces the variance of an intervention's effect. v8–v14 frozen.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Determinism (unchanged)
All simulation logic in Rust; new collections `BTreeMap`/`BTreeSet`; no order-dependent hash iteration; no transcendental f64 in simulation crates; no new crates; no new enum variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType` or `DeathReason`. A keyed draw is a pure function of its key; the run's output never depends on thread count or iteration order.

## 4.2 Versioning — v8 to v14 frozen, v15 new
- `game_v8()` … `game_v14()` keep their behaviour exactly; every new field takes the value that reproduces them bit for bit (the sequential streams stay in the code, used whenever the v15 flag is off).
- Part 1 adds `WorldRules::game_v15()` (= `game_v14()` + the flag of 6.1), `WorldGenConfig::game_v15(seed)`, `game_v15_scene(seed)`, the probe scene `game_v15` wherever `game_v14` appears (`probe/args.rs`, `probe/runner.rs`, `tools/revitalize/probe.sh --help`) and the recorder's scene list. The live game switches to v15 in both `world_node.rs` constructors.

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; all Pass 2–8 tests pass unchanged. Never change `sim-test/src/probe/fingerprint.rs` or `probe/long_run.rs::trace`. Allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 9`: struct-literal field additions with legacy values; probe schema tests moving to schema 9; recorder schema tests moving to schema 3 (the committed v0 sample, schema 1, still loads); Rust-source static assertions pinning code this goal deliberately changes, re-pinned and never weakened.

## 4.4 The gate
Copy `tools/revitalize/gate_pass8.sh` to `gate_pass9.sh` (same steps): debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep (0 hits), `gdscript_strict_check.sh`, `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, both Python unittest discoveries.

## 4.5 v14 guard after every behaviour Part
After Parts 2 and 3: re-run `game_v14` 64 × 10 and 32 × 120 with Pass 8's final flags and compare with Part 0's v14 runs; every shared value identical (only `schema_version` and new keys may differ).

---

# 5. PART 0 — Baseline and the randomness inventory
1. Record `PASS9_BASE_SHA` (`4c35e295`), start time, `uname -a`. Commit `PASS9_GOAL.md`.
2. Release probes with Pass 8's recorded flags: `game_v14` 64 × 10 and 32 × 120 against `p8-4_round1_10day` / `p8-4_round1_120day`; `game_v13` 64 × 10 and 32 × 120 against `p8-7_final_v13_*`; legacy against `p8-7_final_legacy`. Every shared key identical, timing excepted.
3. Create `harness_v14_fingerprints_unchanged` in `harness_revitalize_pass9_frozen.rs` (new file): pin `game_v14` digests for seeds 0–3 after 1,440 **and** 40,320 ticks (at 1,440 ticks, spring day 1, v14 still equals v13; the 40,320-tick pin covers winters, births and lifespans), plus a field-by-field literal test of `WorldRules::game_v14()`.
4. **Randomness inventory** `docs/revitalize/RNG_INVENTORY.md`: every call site that draws a random number or hashes for a pseudo-random choice in `sim-core/src`, `sim-systems/src`, `sim-engine/src` and `sim-bridge/src` (grep `next_u64`, `next_step`, `MovementRng`, `MovementRng::new`, `splitmix`, `mix(`, `h64`, `_SALT`, `world_seed`, `wrapping_mul(0x`), with: file:line, what it decides, its current key/stream, whether the number of draws depends on state, and its v15 key (section 7.1). A second table lists **order dependence**: every loop whose outcome depends on the iteration order of a hecs query or a hash container (first-come claims, shared counters decremented in the loop), with file:line and its v15 treatment (7.1). Confirm or correct 3.2. Commit, push.

---

# 6. PART 1 — The keyed-draw primitive, lineage keys, the v15 skeleton

## 6.1 Rule field
`AgentRules` (`rules.rs`, where `idle_pause_chance` lives; record the choice if you put it elsewhere) gains `keyed_draws: bool` — v8–v14 `false`, v15 `true`.

## 6.2 The primitive (`sim-core/src/rng.rs`, new, pure)
```rust
/// One 64-bit draw that depends only on its key. `stream` names the decision
/// (constants below), `subject` the entity's lineage key or 0 for world draws,
/// `tick` the simulation tick, `slot` the index of the draw within that event,
/// `replicate` the noise replicate (section 7.3; 0 = the canonical run).
pub fn keyed_u64(world_seed: u64, stream: u64, subject: u64, tick: u64, slot: u32, replicate: u32) -> u64
```
- Built from the existing splitmix64 finalizer chained over the fields (e.g. `m(m(m(m(m(seed ^ stream) ^ subject) ^ tick) ^ slot) ^ replicate)` with `m` one splitmix64 step); document the exact chain. No new crate; no Philox needed (record why: counter-keyed splitmix64 is a pure function of the key, which is the property required).
- Helpers: `keyed_unit_f64` (53-bit), `keyed_below(n)` by multiply-shift (no rejection loop, so the number of calls never depends on the value), `keyed_step` (−1/0/+1).
- Stream constants in one place (`STREAM_MOVE_PAUSE`, `STREAM_MOVE_DX`, `STREAM_MOVE_DY`, `STREAM_LIFESPAN`, `STREAM_NEWBORN`, …), all distinct, with a test that they are.

## 6.3 Lineage keys (`sim-core/src/components/lineage.rs`, new)
- Component `Lineage { key: u64, births: u32 }` on every resident in v15 worlds (inert in v8–v14: not attached).
- **Founders:** `key = keyed_u64(seed, STREAM_FOUNDER, 0, 0, founder_index, 0)` where `founder_index` is the worldgen spawn order.
- **Newborns:** with parent A = the parent the birth code already chooses first (the lowest-`AgentId` member — unchanged behaviour), `key = keyed_u64(seed, STREAM_NEWBORN, A.key, 0, A.births, 0)`, then `A.births += 1`. A newborn's key therefore depends on its parent's identity and birth order, never on the global id counter. `AgentId` stays the engine's monotonic id (snapshots, events, beds) and is unchanged.
- **Known limitation, measure it, do not hide it:** in a one-settlement village parent A is nearly always the same founder (`settlement_system.rs` ≈ L757–768 sorts members and takes the two lowest ids), so `A.births` is in practice the village's birth order. A birth added or removed by an intervention then shifts the keys of every later newborn. The keys still remove the dependence on the global `AgentId` counter (which also counts any other spawn), and they become parent-specific once parents differ (several settlements, a dead founder). Record this in DECISIONS.md; Y6 measures how often keys match in the fork data and is reported, not gated.
- Tests: the stability test builds the unrelated birth with a **different parent A** (two settlements, or a direct call of the key function with two parents) and checks that removing it leaves every other newborn's key unchanged; founders' keys depend only on seed and index; keys are distinct over 10,000 founders and 10,000 newborns (no collision).

## 6.4 Wiring and instruments
- v15 skeleton as 4.2, no behaviour change yet (flag read nowhere until Part 2).
- Probe schema 9: `noise_replicate` (0 unless a fork set it) and `keyed_draws` (0/1) in `summary.json`'s `rules` block; no per-row key changes.
- Recorder schema 3: with `--agents`, `agent_keys.npy` (uint64 [N], aligned with `agent_ids.npy`, the lineage key per agent, 0 when the agent has none; marked as an identity field, not an input) and `meta.json` fields `keyed_draws`, `noise_fork` (null or `{tick, replicate}`). `tools/worldmodel/wm_data.py` accepts schema 3 (`SCHEMAS = (1, 2, 3)`); its tests load a schema 1, 2 and 3 episode. Agents are matched across arms by lineage key, never by `AgentId`.
- `evaluate_pass9.py` = `evaluate_pass8.py` reading schema 9 (comparison mode, X targets) plus the fork metrics of section 8 (`tools/revitalize/fork_metrics.py`, stdlib + numpy, with tests on synthetic arrays).
Commit, push.

---

# 7. PART 2 — Keyed draws in v15, noise forks

## 7.1 Switch every draw (only when `keyed_draws`)
- **Movement (idle Brownian):** pause = `keyed_unit_f64(seed, STREAM_MOVE_PAUSE, key, tick, 0, r)`; dx, dy = `keyed_step` on `STREAM_MOVE_DX` / `STREAM_MOVE_DY`. Each is drawn as a pure function of (agent key, tick), whether or not the agent was idle on earlier ticks. The `MovementRng` component stays attached (other code may query it) but is not advanced in v15. (v14's `% 3` and v15's multiply-shift give the same distribution but different values; note it.)
- **Lifespans:** `keyed_u64(seed, STREAM_LIFESPAN, lineage.key, 0, slot, r_b)` with `r_b = noise_replicate_for(birth_tick)` — **no tick in the key**, so a birth shifted by a tick keeps its lifespan. Slots: founders 0 = age, 1–3 = lifespan, 4 = top-up; slot 4 is always computed and used only when needed (`aging.rs` ≈ L43–46); newborns use slots 1–4 the same way. Any other data-dependent draw count is replaced by a fixed count; record it.
- **Iteration order (3.2.7):** in v15 the decision loop collects its agents and processes them sorted by `AgentId` (as the other loops already do), and every other order-dependent loop of the inventory gets the same treatment; v8–v14 keep hecs order (flag-gated). Record the measured effect on v15 health in Part 4.
- **Anything else** found in the inventory (5.4) that decides with randomness.
- `r` is `SimResources.noise_replicate_for(tick)` (7.3); 0 in an unforked run.

## 7.2 Expected consequence
v15 trajectories differ from v14's (different numbers), but every distribution should be the same: v15 is v14 with the same balance. Part 5 checks this.

## 7.3 Noise forks
- `SimResources` gains `noise_fork: Option<NoiseFork { tick: u64, replicate: u32 }>` (inert unless set). `noise_replicate_for(t)` = `replicate` when `t >= fork.tick`, else 0.
- `pub fn apply_noise_fork(engine, replicate)` sets the fork at `engine.current_tick()`; replicate 0 is a no-op by definition.
- **Weather is not resampled in this pass** (severity, night burn, day fall stay keyed on seed and day): the effect distributions of section 8 are over agent noise only. Record it; weather resampling belongs to the Bayes-ceiling pass.
- Recorder: `--fork-tick T --replicates R` (with `--schedule-dir` and `--include-factual`) writes, for every seed and schedule (factual included), R episodes `<scene>_s<seed>_<id>_r<k>` for k = 0..R−1 that share the prefix up to T and use replicate k after it (k = 0 is the canonical run). Episodes sharing a root seed form one **branch family**.

## 7.4 Tests (`harness_revitalize_pass9_rng.rs`)
`harness_keyed_draw_is_pure` (same key, same value; any field change changes it; ~uniform: each of 16 buckets within ±0.01 of 1/16 over 10⁶ draws), `harness_keyed_step_balanced`, `harness_v15_movement_independent_of_history` (an agent's step at tick t is the same whether it was idle or busy before t), `harness_lineage_keys_stable_under_birth_shift`, `harness_noise_fork_replicate0_is_identity` (set `noise_fork = Some(NoiseFork { tick, replicate: 0 })` **directly** — not through `apply_noise_fork`, which may return early — and check every digest is identical), `harness_noise_fork_prefix_identity` (replicate k ≠ 0: digests equal at every 60th tick ≤ T — systems see `current_tick` = completed ticks, so row T is still pre-fork — and different at some tick after), `harness_v14_draws_unchanged` (v14 still uses the sequential streams: fingerprints equal Part 0's). Commit, push; run 4.5; fast-forward green if the gate is green.

---

# 8. PART 3 — The gates: placebo, variance reduction, divergence

All on `game_v15`, seeds **12000–12031** (dataset v1's C_cf seeds, `--seeds 12000..12032` in the tools' half-open notation; recorded under v15 into `WM_DATA_ROOT/p9/`), 35 days, `--every 60`, **with `--agents`**, the five C_cf variants of v1 (gift_wood 60 and rot_wood 60 at tick 29,400; rain 13 h at 33,720; cold_snap 2 nights at 35,160; windfall 20 at 31,920) plus the factual, **R = 8 replicates forked at tick 28,800** (day 21 08:00, before every variant's intervention, so the factual and each variant share replicate k's noise from the same tick).

Budget: (6 schedules × 32 seeds + the one-unit gift × 32 seeds + 7 placebo kinds × 8 seeds) × 8 replicates = 2,240 runs of 35 days; Pass 8's 120-day run took a median 28.4 s (`p8-7_timing_v14_120day`), so ≈ 8 s each, ≈ 75 min with `--workers 4`. Record the real time.

Targets: `bench_v1.c_cf_targets` imported unchanged — dark nights (evenings of days 22–28), cold member-hours from day 22 18:00 to day 29 07:00, food store and deaths at the day-29 06:00 row. Births at day 29 06:00 are reported only.

## 8.1 Placebo gates (G0a)
1. Every intervention kind with amount 0 (the five variants' kinds at their ticks; `gift_food` and `spoil_food` at tick 29,400), on seeds 12000–12007 × 8 replicates: the episode is identical to the factual of the same replicate. **Identity** = the sha256 of every `.npy` file equal, `interventions.json` equal after removing amount-0 records (they are logged), and `meta.json` equal except `episode_id` and `noise_fork`.
2. Replicate 0 of every family equals the unforked run under the same identity (its `meta.noise_fork` is `{tick, replicate: 0}`).
3. A **one-unit gift** (`gift_food 1` at tick 29,400, 32 seeds × 8 replicates): report its effect distribution (the smallest real intervention; most targets should be zero).
Any failure stops the run until explained (it means hidden state leaks across arms).

## 8.2 Variance reduction (G0b) — the central measurement
For each variant v, target y, seed s and replicate k: paired effect `Δ_paired(s,k) = y(v,s,k) − y(f,s,k)`; unpaired effect `Δ_unpaired(s,k) = y(v,s,k) − y(f,s,k')` with k' = (k+1) mod R.
- `V_p = mean_s var_k(Δ_paired(s,·))` and `V_u = mean_s var_k(Δ_unpaired(s,·))`, variances over the 8 replicates **within** a seed (`ddof=1`), averaged over seeds — between-seed differences of the effect never enter. **VRF = V_u / V_p.**
- 95 % interval: percentile cluster bootstrap that resamples seeds with replacement (2,000 resamples, `numpy.random.default_rng(9)`), recomputing VRF each time.
- Edge cases: V_p = 0 < V_u → VRF = ∞ (passes); V_p = V_u = 0 → the target is reported as "no agent noise" and excluded from G0b for that variant (say how many).
- Also report: mean effect per variant and target; **sign stability** = share of seeds where every `Δ_paired(s,k)` has the sign of that seed's mean (0 matches only 0); **spread** = median over seeds of `std_k Δ_paired(s,k)` beside the median `|mean_k Δ_paired(s,k)|`.
- Note in the report: the unpaired arm still shares worldgen, weather and the whole prefix up to the fork, so VRF measures the value of pairing the post-fork agent noise only.
Gate G0b passes when VRF > 1 with its lower interval bound > 1 for every variant on dark nights and cold member-hours that is not excluded as "no agent noise" (record the other targets).

## 8.3 v14 against v15 (single forks)
The v14 single-fork effects of the same seeds and variants come from dataset v1's C_cf episodes (read-only, `WM_DATA_ROOT/v1/C_cf/`; first re-record 2 of them with `game_v14` and confirm byte identity under 8.1's identity). Report per target the mean over seeds of |Δ_v14(s) − mean_k Δ_v15(s,k)| **beside** v15's median within-seed `std_k Δ_paired(s,k)`: the v15 prefix differs from v14's from tick 0 (different movement numbers), so the first number mixes trajectory differences with noise reassignment, and only the comparison with v15's own spread is meaningful.

## 8.4 Divergence horizon
For the one-unit gift and for each variant at replicate 0: per recorded row (60 ticks) after the intervention, the share of living agents (matched by lineage key) whose position differs between arms, and the first row each target differs. Report medians over seeds. A short horizon with small target differences means agent-level noise does not change village outcomes; a long one means it does.

## 8.5 Outputs
`docs/revitalize/probe/p9-3_forks/` (`fork_metrics.json`, `fork_metrics.md`, the scripts' exact commands). Commit, push.

---

# 9. PART 4 — v15 is v14 with different numbers (health and shape)
1. `game_v15` on the tuning seeds (64 × 10, 32 × 120) and the held-out seeds (64–127 × 10, 32–95 × 120), Pass 8's flags, `--check-determinism`; evaluate with `evaluate_pass9.py` in comparison mode against Pass 8's v12 evaluations of the same seeds (`p8-1_v12_tuning_targets`, `p8-1_v12_heldout_targets`).
2. Compare with v14 on the same seeds (`p8-4_round1_*`, `p8-5_heldout_v14_*`): every X target of Pass 8 (X2–X5) must hold for v15 as it did for v14. Additionally report, per X3 clause and per Y-median, the v14 → v15 change with a 95 % bootstrap interval over seeds (2,000 resamples, `default_rng(9)`); an interval that excludes 0 on more than 2 of them is a finding to explain (Y4).
3. **No tuning in this pass.** If a target fails, record it with the failing seeds and the v14 value; do not change knobs.
4. Y8 timing: ms per agent-tick v15 ≤ 1.2 × v14, back to back on one host. Commit, push.

---

# 10. PART 5 — Recorder and dataset tooling for branch families
1. `make_dataset.py` gains a `families` mode: given seeds, schedules, a fork tick and R, it writes branch families under `WM_DATA_ROOT/<name>/` with a manifest listing each family (root seed, fork tick, members) — the format dataset v2 will use. Splits are assigned per **root seed**, never per episode (members of one family never cross splits).
2. `wm_data.py` loads a family as one object (`load_family(path)` → factual and variants by replicate).
3. Tests on a tiny family (2 seeds × 2 variants × 2 replicates × 3 days). No v2 dataset is recorded in this pass.
Commit, push.

---

# 11. TARGETS (game_v15)

| # | target |
|---|---|
| Y1 | determinism 64/64 and 32/32 on tuning and held-out seeds; legacy pin; v8–v14 reproduce Pass 8's final summaries on every shared key |
| Y2 | G0a placebo gates 8.1 (all identical) |
| Y3 | G0b: VRF > 1 with lower 95 % bound > 1 for every variant on dark nights and cold member-hours |
| Y4 | v15 health and shape: Pass 8's X2–X5 hold on tuning and held-out seeds; ≤ 2 unexplained v14→v15 shifts (section 9 item 2) |
| Y5 | every test of 7.4 and 10.3 passes; the randomness inventory is complete (independent reviewer confirms no draw is missed) |
| Y6 | lineage: the stability test of 6.3 passes (unrelated birth with a different parent); in the fork data the share of newborns present in both arms with equal keys is **reported, not gated** (6.3's known limitation) |
| Y7 | recorder schema 3 byte identity (two runs; `--workers 1` vs 4) including fork families |
| Y8 | ms per agent-tick v15 ≤ 1.2 × v14 |

---

# 12. PART 6 — Independent review, final verification, report
1. **Independent review:** a built-in `general-purpose` subagent with fresh context gets `git diff 4c35e295..HEAD -- rust/ tools/revitalize/ tools/worldmodel/`, sections 3.2, 4, 6, 7, 8 and 11, and the inventory; it checks for any draw left on a sequential stream in v15, any key that includes a value that can differ between arms (e.g. an `AgentId` or an `Entity`), data-dependent draw counts, iteration-order dependence, the fork semantics (prefix identity; replicate 0 identity), the pairing in the VRF computation, and v8–v14 invariance, with file:line evidence. Fix confirmed defects (two rounds maximum); record every finding in DECISIONS.md.
2. Full gate on the final WIP head; totals versus Pass 8 (2,005 debug and release).
3. Final probes: legacy and v8–v14 reproduce Pass 8; v15 tuning and held-out runs; the fork experiment of 8 rerun if any code changed after it; timings for Y8.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS9.md` in Korean (section 14 format). Section 13's checks are for the owner in the running game: run the headless smoke only and list them as "주인 확인 필요". Append `## Pass 9` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 9 section (skip and say so if `gh` is not authenticated).

## 12.1 Merge touch-points (list them in the report)
The live game builds `game_v15` (villagers wander along different random paths; balance unchanged); the art branch's `get_intervention_state` getter in `world_node.rs` merges alongside your constructor change.

## 12.2 Out of scope
Consequential interventions (deaths, births), the parameterized intervention space, the Bayes-optimal severity posterior and weather resampling, multiple settlements, scale, dataset v2 — later passes of the research plan. Any balance change. Rendering.

---

# 13. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)
1. 게임이 game_v15로 시작하고 콘솔에 SCRIPT ERROR가 없다.
2. 마을의 모습과 균형은 Pass 8과 같다. 주민이 한가할 때 걷는 경로만 다르다(같은 시드여도 이전 버전과 길이 다름).
3. 겨울의 꺼지는 밤, 추위, 식량, 출생·사망 수준이 Pass 8과 같은 범위다.
4. 신의 개입 결과(비, 선물 등)는 예전처럼 동작한다.

---

# 14. 구현 완료 보고 (REPORT_PASS9.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
난수 목록(어디서 무엇을 뽑는지), 키 기반 추첨 함수와 스트림, 혈통 키, 노이즈 분기(재표본·복제본 0), v8~v14 고정 방식

### 기능 설명
연구 관점에서 무엇이 가능해졌는가? (믿을 수 있는 반사실, 효과 분포, 분기 가족 데이터)

### 변경된 파일 목록
(커밋별)

### 측정 결과
Y1~Y8 표, 가짜 개입 결과, 변형×목표별 VRF와 95% 구간·평균 효과·부호 안정성, v14 단일 분기 대비 차이, 발산 지평, v15 건강·형태(튜닝·검증, v14 대비 변화)

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 백그라운드 작업, 동시에 돌던 작업

### 확인된 제한사항

### Harness 결과
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트 두 묶음)

### 정직한 판단
이제 한 번의 분기 차이를 개입 효과라고 불러도 되는가? 노이즈가 마을 결과를 얼마나 바꾸는가? 다음 단계(결과가 큰 개입, 베이즈 상한)를 위해 무엇이 준비됐고 무엇이 남았는가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, the path of `fork_metrics.md`, and the time spent per Part.
