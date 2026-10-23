---
name: worldsim-testing
description: Use when testing or verifying any WorldSim change, before claiming "no behaviour change", "deterministic", "faster" or "done" — identity (tools/v8/identity.sh), determinism, the 30-step gate (tools/v8/gate.sh), the economy soak, perf comparison (tools/v8/perf.sh), test layout by domain on sim_test::baseline_engine, and how to write an assertion that can fail.
---

# WorldSim testing

What proves what. A green test proves only what its assertion can catch;
identity proves "no behaviour change"; the gate proves a commit is fit to
fast-forward `v8/main`. Code rules live in `worldsim-architecture`; Godot
headless specifics in `worldsim-godot`.

## Rules

### Claims
- **Do** run identity (`tools/v8/identity.sh <base_data_dir>`) before claiming "no behaviour change" (refactor, rename, move, deletion). *Reason:* only full summaries + raw samples + chronicle by content + determinism show that nothing moved; a green test suite does not.
- **Don't** explain an identity mismatch away. The failing commit is not pushed until it is fixed or reverted, and the cause goes in DECISIONS. *Reason:* every "harmless" difference is unproven until the identical run exists.
- **Don't** add a key to `tools/v8/identity_removed_keys.txt` to make a mismatch pass. A key goes there only when a commit removed it on purpose, with that commit area as the reason; `identity.sh` drops it from the base side only where the base value is zero/false/null/empty and fails on any non-zero value. *Reason:* the allowlist exists for removed dead fields, not for changed values.
- **Do** write a claim only after its artefact exists (log, summary, commit); mark it PENDING until then. *Reason:* V7 records repeatedly cited checks before they ran.
- **Don't** take a green structural test suite as proof that the game works. *Reason:* tests that read source text, run on `WorldRules::default()` (which switched 8 of V7's 28 systems off), or set the fixture value themselves stay green while the live loop is broken.

### Gate and determinism
- **Do** run the gate (`tools/v8/gate.sh <log_dir>`) on one committed head with nothing edited, built or committed while it runs. *Reason:* a gate whose steps saw different trees tested no commit.
- **Do** keep raw probe and gate data outside the repository (identity refuses an output dir inside it). *Reason:* raw outputs are gigabytes; commit summaries only.
- **Do** check determinism with the probe's `--check-determinism` and with in-test twins (two engines, same seed, lockstep, compare `sim_test::probe::fingerprint`). Extend the twin when you add state. *Reason:* twins only cover the state they compare.
- **Do** treat probe seed ranges as half-open (`--seeds 0..32` = seeds 0–31). *Reason:* `rust/crates/sim-test/src/probe/args.rs` parses `a..b` half-open; `a..b` with b inclusive silently drops a seed.
- **Don't** run Godot steps (import, strict check, smoke) concurrently with each other or with Godot tests. *Reason:* the strict check temporarily edits `project.godot`, and parallel runs see the edit.

### Perf
- **Do** compare perf only against base numbers measured the same way: same machine, same build profile (release), same procedure (`tools/v8/perf.sh`), same duration. *Reason:* cost per tick changes with run length, world and host load.
- **Do** treat a natural mean more than +10 % over base as a WARN to investigate, and report it; perf is report-only in the gate. *Reason:* mean tick is the stable statistic; the step never blocks, so the report must say it.
- **Don't** judge a change on p99 / p99.9 from three repeats. *Reason:* tails of a few runs are single-tick machine spikes (Pass 21 U6: per-repeat ratios ×1.023 / ×1.490 / ×0.516); report them as noise.
- **Don't** compare timings taken under different host load, and never call a contended number a result. *Reason:* on the shared Mac one layer "cost" +62 % because the host load fell 176 → 36 between samples.
- **Don't** use debug builds, debug dylibs or headless dummy-renderer timings as performance evidence. *Reason:* debug W500 was 51 ms against 29 ms release; the dummy renderer draws nothing.

### Benchmark, save, second renderer (M1.5, ADR 0016)
- **Do** measure colony-scale cost with the standing benchmark (`worldsim_perf bench`, gate step `bench` = `tools/v8/bench.sh`): five colony scenes (N = 200/1,000/2,000, N = 200 with `tools/v8/bench_script.json`, N = 200 on 64²), each in its own process (clean RSS), 3 repeats, medians; the gate compares with the latest pass in `docs/perf/PERF_LEDGER.md` (> +10 % on tick mean, p99 or RSS/agent fails unless `--justified` quotes the CHANGELOG). *Reason:* C3; the natural `perf.sh` worlds hold 10–40 people and cannot show O(N²) work.
- **Do** judge memory on the allocator counter (`alloc_per_agent`, `alloc_per_tile`; the regression rule and, since M5 D19, the C3 memory ceilings), keeping RSS as information. *Reason:* RSS is bimodal on macOS: one binary read 83–118 B per tile across sessions and the M4 base missed its own ceiling once; the allocator read 63.1 B every time.
- **Do** append a pass's ledger rows only from a session where base and candidate ran back to back, interleaved (`bench --repeats 1` per binary per round, then `bench-summarise`), with the host load recorded. *Reason:* timings taken under different load are not comparable (perf lessons below).
- **Do** keep the benchmark world alive: it is fed like `quick` (3 food per person at 06:00); check `alive_end` and the deaths in every run file. *Reason:* without the gift N = 2,000 lost 1,611 people inside the run, and a world that dies is cheap.
- **Do** prove view independence with `tools/v8/renderer_agreement.sh` (text renderer = Godot agent layer, seed 100, tick 900) and keep `docs/v8/snapshot_schema.md` complete (`python3 tools/v8/check_docs_helpers.py schema .`, gate step `schema_docs`). *Reason:* C2; the schema check fails on any undocumented getter or key.

### Assertions
- **Do** assert a threshold on a measured invariant, with the observed value and margin in a comment. *Reason:* `> 0` passes when a value collapses from 368 to 1.
- **Do** ask of every test "which bug still passes?", and make it fail once (break the code or revert the fix) before trusting it. *Reason:* V7 had tests that passed with the feature deleted.
- **Do** require that the checked event actually happened (count > 0) before asserting about it, and fail on missing inputs. *Reason:* "every cancel has one entry" passed with 0 cancels.
- **Do** account for censoring in duration and time-to-event metrics: count the runs that end before the event and report them, or measure only events with enough remaining window. *Reason:* Pass 21 U3 counted late reconciliations that the 120-day run could never see as failures.
- **Do** tune on some seeds and assert on held-out seeds picked (and grepped for earlier use) before measuring. *Reason:* a threshold tuned on its own seed tests the tuning, not the mechanism.
- **Do** compute expected literals independently of the code under test (by hand, a Python re-implementation, or the rule's own fields), never by printing the function's output. *Reason:* a literal produced by the code under test proves nothing.
- **Don't** write tests that read source text (`.gd`, `.tscn`, `.rs`, localization JSON). *Reason:* every renderer rewrite broke hundreds of them while proving no behaviour; V8-0 deleted them.
- **Don't** re-pin, `#[ignore]`, weaken or delete a failing assertion to turn the gate green. *Reason:* the failure is the information; fix the code or keep the work local.

## Procedures

Longer versions (assertion recipes, perf and contention protocol):
`references/assertions.md`, `references/perf.md`.

### A. Identity
1. Base data exists once per base commit (V8-0's is `/Users/rexxa/work/v8-0-data/base`, captured at `b1ca032f`). To capture a new base: build the release probe at that commit (a detached worktree), then `bash tools/v8/identity.sh --capture <bin_dir> <out_dir>` with `<out_dir>` outside the repo.
2. Candidate: `bash tools/v8/identity.sh <base_data_dir> [<candidate_out_dir>]`. It builds `worldsim_probe` (release) from the working tree, runs scene `game_v26` (the probe's alias of `baseline`), seeds 0..32, 120 days, clock samples 02:00…19:30, `--sample-every 120`, `--chronicle-out`, then `--check-determinism` on seeds 0..4.
3. It prints four lines; all must be PASS:
   - `summary`: `compare_summaries.py` — every simulation value equal, timing keys (`wall_seconds`, `ms_per_agent_tick`, `timing`) and `git_sha` excluded, 0 differences and 0 one-sided paths;
   - `chronicle`: every JSON in the chronicle dir (`chronicle_N.json`, `relations_N.json`) equal by content except `git_sha`;
   - `raw samples`: sha256 of every per-seed `*.jsonl` with `git_sha` removed;
   - `determinism`: `DETERMINISM PASS`.
4. Any FAIL: stop. Do not push the commit; find the cause (bisect the commit's hunks), fix or revert, rerun. Record the cause in DECISIONS.
5. Intended behaviour change: identity fails by design. Say so in the commit and CHANGELOG, show which values moved and why, and capture a new base at that commit for later refactors.

### B. Determinism
1. Probe: `cd rust && cargo build --release -p sim-test --bin worldsim_probe && ./target/release/worldsim_probe --scene baseline --seeds 0..4 --days 120 --workers 4 --check-determinism --out <dir>` → `DETERMINISM PASS`.
2. In tests: build two `baseline_engine(seed)` engines, tick both in lockstep, compare `sim_test::probe::fingerprint(&engine)` at checkpoints and at the end; put twins in `rust/crates/sim-test/tests/determinism.rs`.
3. For an order-sensitive fix, run the twin over many fresh engines (32) so hash-order luck cannot pass it.

### C. The gate (`tools/v8/gate.sh <log_dir> [--base-data DIR]`)
Each step logs to `<log_dir>/<step>.log`; the summary with start time, duration and exit code is `<log_dir>/gate.txt`. The gate passes only when every step except `perf` exits 0 (20 steps since M1, 21 since V8-PLAY, 25 since M1.5, 27 since M3, 28 since M4, 29 since M5, 30 since M5.5; `nav_bench` is report only too).

| # | step | checks |
|---|---|---|
| 1 | `fmt` | `cargo fmt --all -- --check` |
| 2 | `bridge_build` | `cargo build -p sim-bridge` (the dylib Godot loads) |
| 3 | `godot_import` | bounded headless `--import --editor --quit`, twice (a fresh import logs font errors) |
| 4 | `test_debug` | `cargo test --workspace --no-fail-fast -- --test-threads=8` |
| 5 | `test_release` | the same in `--release` |
| 6 | `clippy` | `cargo clippy --workspace --all-targets -- -D warnings` |
| 7 | `forbidden_math` | float transcendental grep over sim-core, sim-systems, sim-engine `src` |
| 8 | `gdscript_strict` | `tools/v8/gdscript_strict_check.sh` (parse, warnings as errors, FFI names) |
| 9 | `smoke` | `tools/v8/smoke.sh`: 900 headless frames; `SCRIPT ERROR`, `ERROR:`, `Locale:` and raw-key counts ≤ base |
| 10 | `identity` | `tools/v8/identity.sh <base data>` |
| 11 | `determinism` | probe `--check-determinism`, seeds 0..4 |
| 12 | `perf` | `tools/v8/perf.sh` + `perf_summary.py --compare`; report only, > +10 % mean = WARN |
| 13 | `bridge_api` | `tools/v8/bridge_api.py --check` against the expected list |
| 14 | `docs` | `tools/v8/check_docs.sh` |
| 15 | `inventory_closure` | `tools/v8/inventory.py check` |
| 16 | `python_unittest` | `python3 -m unittest discover -s tools/v8 -p 'test_*.py'` |
| 17 | `m1_tests` | the M1 tests by name (`colony_commands`, `colony_jobs`, `colony_world`, `engine_colony_ffi`, `probe::colony`, `jobs::`, `colony::`) + `tools/v8/test_locale_trf.gd` |
| 18 | `colony_determinism` | colony probe, seeds 0..4, 10 days, `tools/v8/u_script.json`, `--check-determinism` |
| 19 | `replay` | a scripted colony run's recorded command log replays to a byte-identical summary |
| 20 | `uid_check` | `tools/v8/check_docs_helpers.py uid` (every Godot source has its committed `.uid`) |
| 21 | `play_check` | `tools/v8/test_play.sh`: `tools/play.sh` exit codes (unknown version 3, no Godot 2, bad argument 4) and `--headless-check` of `main`, `m1`, `v7` (0 `SCRIPT ERROR` in 300 headless frames) |
| 22 | `schema_docs` | `tools/v8/check_docs_helpers.py schema .`: every bridge getter and key is in `docs/v8/snapshot_schema.md` (C2) |
| 23 | `renderer_agreement` | `tools/v8/renderer_agreement.sh`: the text renderer's people = the Godot agent layer's (seed 100, U-script, tick 900; C2) |
| 24 | `save_roundtrip` | `tools/v8/save_roundtrip.sh`: `save_load.rs` (colony seeds 0–7 save → load → tick = tick; C1) |
| 25 | `bench` | `tools/v8/bench.sh`: the standing benchmark against `docs/perf/PERF_LEDGER.md` (> +10 % fails; ceilings report only; C3) |
| 26 | `soak` | `tools/v8/soak.sh`: the economy soak fixture, colony seeds 0..4 × 20 days with `tools/v8/soak_script.json` (stone, flint, workshops, bills, axes, a house, a storehouse, a dug pocket; since M4 also 0 `KnowledgeLost`; since M5 with `--story no-beasts`): ledger and reservations exact every tick, 0 deaths, determinism (M3 goal 3.4) |
| 27 | `nav_bench` | `worldsim_perf nav-bench`: N = 1,000 with the soak script; movement + pathfinding µs per tick against the 600 µs budget, requests, memo hits, nodes expanded (report only) |
| 28 | `knowledge` | `cargo test --release -p sim-test --test colony_knowledge` + the loss fixture `tools/v8/m4_f10_script.json` (seeds 0..8, 4 days) scored by `docs/v8/m4/m4_score.py --f10`: 8/8 lose the doorway within 240 ticks of the kill, end Lost, and leave door orders waiting on it |
| 29 | `story` | colony seeds 0..2 × 48 days with `tools/v8/m5_u_script.json` (no panic), scored by `docs/v8/m5/m5_score.py --story`: F1's schedule, 0 ledger and 0 reservation violation ticks on every seed |
| 30 | `neighbours` | colony seeds 0..2 × 48 days with `tools/v8/m55_u_script.json`, scored by `docs/v8/m5_5/m55_score.py --neighbours`: two neighbours, waves 1 and 2 in their windows and accepted, ≥ 1 trade, 0 ledger (with traded in and out) and 0 reservation violation ticks |

1. Commit first; run the gate on that head; do not touch the tree until it finishes (it records `head` and `dirty` in `gate.txt`).
2. Read `gate.txt`, not only the exit code: test counts, perf WARN lines, determinism line.
3. A red step: read its log, fix the cause, commit, rerun the whole gate. A stopped or partial gate is not evidence.
4. Bound every long command with `tools/v8/run_bounded.sh <seconds> <cmd…>` (exit 124 on timeout).

### D. Perf comparison (summary; full protocol in `references/perf.md`)
1. Release build: `cd rust && cargo build --release -p sim-test`.
2. `bash tools/v8/perf.sh rust/target/release <out_dir> candidate` — natural `worldsim_perf natural --scene game_v26 --days 120` on seeds 19600, 19603, 19605, 19606, three repeats (each repeat runs the four seeds in turn), then `quick` (500 founders, 64×64, fed, 1 day). It logs host load per repeat in `RUNLOG.txt`.
3. `python3 tools/v8/perf_summary.py --compare <base>/perf/perf_summary.json <out_dir>/perf_summary.json` — WARN when a mean rises > 10 %. V8-0 base: natural mean median 0.2773 ms (`docs/v8/V8_0_BASE_NUMBERS.md`, contended Mac).
4. Report the mean; list p99 / p99.9 per repeat labelled as noise; state the start/end load and whether the run was contended.
5. A perf claim (faster/slower) needs an admitted quiet session (calibrated probe, attempts ≥ 30 min apart) or it is reported as contended.

### E. Test layout
- Integration tests: `rust/crates/sim-test/tests/` by domain — `agents_*.rs`, `settlement_*.rs`, `family_*.rs`, `relations_*.rs`, `chronicle_*.rs`, `engine_*.rs` (engine and bridge), `determinism.rs`. Add to the matching file; a new file only for a new domain.
- Unit tests: `#[cfg(test)] mod tests` inside the module, for pure logic and truncation edges.
- Start V7-system tests from `sim_test::baseline_engine(seed)` (the frozen identity world: `WorldRules::baseline(seed)` + `WorldGenConfig::baseline(seed)`) and colony tests from `sim_test::colony_engine(seed)` (the game since M1, ADR 0014; Procedure F). Use `SimEngine::new` + `WorldRules::default()` only for rule-independent mechanics, and never to cover a rules-gated system.
- Advance with `engine.tick()`; 1 tick = 1 simulated minute, 1,440 ticks = 1 simulated day. Read state from the pub fields `engine.world` and `engine.resources`.
- Bridge data: test the pure collectors headless (`sim_bridge::ffi::collect_<thing>` on a built engine) in `rust/crates/sim-test/tests/engine_bridge_ffi.rs` (the bridge-surface domain file) or a new `engine_<concern>_ffi.rs`. sim-test links sim-bridge as an `rlib`. Unit tests inside sim-bridge (`cargo test -p sim-bridge`) are for pure marshalling helpers only. GDScript parse and FFI binding are covered by gate steps 8–9, not by sim-test.
- Name tests by what they assert (`settlement_fire_keeps_winter_hearths_lit`), not by pass or phase.

### F. Colony tests: ledger, reservations, replay (M1 patterns)
1. Start colony tests from `sim_test::colony_engine(seed)` (or `scenes::colony_scene`) and helpers in `rust/crates/sim-test/tests/colony_support/mod.rs`; give orders with `sim_systems::commands::submit`, never by writing `SimResources.colony` directly.
2. **Ledger:** after every tick of a test run, check `sim_systems::runtime::jobs::invariants::check(&world, &resources)` (per-kind conservation since M2: created = ground + carried + held as tools + delivered to sites + on workshop shelves + consumed, from `invariants::ledger_of(world, res, kind)`; reservations; stacks). Assert that every kind the test is about was created at all (`ledger_of(..).created > 0` for logs, stone, flint, …) so the check cannot pass on an idle world; for crafts assert the transform count (`item_ledger.transforms`) and the `crafted`/`craft_inputs` split.
3. **Reservations:** cancel by each `CancelReason` (a command, a death, a need, the target gone) and assert `colony.reservations` holds no key of the cancelled agent afterwards; tick once more when a release happens on the next stage.
4. **Replay:** run with a command script (`worldsim_probe --scene colony --commands <file> --commands-out <dir>`), then replay the written log with `--commands <dir>/…` and compare `summary.json` byte for byte (timings live in `timing.json`, outside the comparison). The unit test in `rust/crates/sim-test/src/probe/colony.rs` and gate step 19 do this.
5. **Twins:** two colony engines with the same seed and the same commands, ticked in lockstep, compared with `probe::colony::colony_fingerprint` (it covers the colony state, jobs and carries, not only the V7 fingerprint).

### G. The economy soak fixture (M3; gate step 26)
1. `bash tools/v8/soak.sh <out_dir> <worldsim_probe>` runs colony seeds 0..4 for 20 days with `tools/v8/soak_script.json` (the M2 economy plus a walled house, a walled storehouse and a dug pocket) and `--check-determinism`; it prints `SOAK PASS` only when every seed has 0 ledger and 0 reservation violation ticks, 0 deaths, and the determinism line, plus one `seed N:` line (crafts, felled, workshops, deaths, alive).
2. Run it on every pass's candidate (the gate does), not only on M3 code: a change anywhere in the job layer shows up first as a ledger hole or a death in a long scripted run.
3. A script entry may wait (`"after": "workshop"`, `"after": "pocket_dug"`) and renew (`"every_days"`); its resolver picks tiles from the live world (`rust/crates/sim-test/src/probe/colony.rs`). Check a resolver's spot with the same predicates the command applies (people, reservations, other buildings' doors and beds — DECISIONS M3 D17); a resolver that picks a spot the command refuses in part measures the script, not the sim.
4. Read the failing seeds before believing a bar: `docs/v8/m3/m3_score.py --out appendix.json` lists per-seed values; a bar that fails on a few seeds usually has one cause per seed, found by rendering that seed (`worldsim_probe --ascii --rect`).

### H. Knowledge fixtures (M4)
1. Unit-level knowledge tests start from `job_world_m4` (the M2 job world under `ColonyRule::M4`, knowledge systems not registered, so nothing spreads or fades by itself) and give people nodes with `know(engine, e, names, ‰)`; channel tests register only the systems they test (`knowledge_world(seed, systems)`) and pin the agents in place.
2. Loss: `Command::DebugKill { agent }` (a probe and test order, logged and replayed like every command) kills one person through the normal death path; the script resolver `holders_of` kills every holder of a node. Assert the chronicle `KnowledgeLost` within one aggregate interval (240 ticks), the node's state Lost, and a waiting order in `collect_knowledge_waits`.
3. A gate test must keep a non-practitioner on the work in every way the world can advance it — a job and the V7 arms in place (stand the person on the site) — and assert progress 0; then make them a practitioner and assert the work completes (D22).
4. Score scripted runs with `docs/v8/m4/m4_score.py` (`--out` for the per-seed appendix; `--f10` for the loss fixture); the bars are registered in its docstring before the first run.

### I. Pressure and control runs (M5)
1. A behaviour that answers pressure is measured twice on the same seeds and script: with the storyteller (`--story on`, the default) and without (`--story off`, the control); a third run removes the player's answer (`m5_u_nodraft_script.json`: no draft) to show the answer matters (F3b).
2. Bars that say "unchanged by M5" are read on the control and compared with the previous pass measured the same way (M4's probe built in a detached worktree at its report commit, its own U-script, the same seeds and days): F9a pooled 0.322 against M4's 0.336.
3. Bars that say "the pressure is felt" are read on the pressure run (F2a, F6b); bars that say "a prepared colony survives" on the same run (F2b, F3a).
4. Rates pooled over worlds when a world holds a handful of events (births, breakdowns), never per world; register the pooling in the scorer's docstring before the held-out run.
5. The gate's `story` step runs two worlds × 48 days with the storyteller and scores the schedule and the ledger (`m5_score.py --story`); the soak runs `--story no-beasts` (weather and illness, no wolves) and still asks 0 deaths.

### J. Controls and the arithmetic before a bar (M5.5)
1. A behaviour that answers the player is measured against the same seeds without the answer: M5.5 runs the script
   (pressure), the script without drafting (no-draft), without the storyteller (story-off), without neighbours
   (no-neighbours) and with neither (quiet, the demographic control). Register which run each bar reads in the
   scorer's docstring before the held-out run.
2. A bar that depends on rates states its expected value from the rates before it is registered (M5.5 F4: ~1 kill in
   7 days → ~4 wraps by day 30; F9: quality 30 + 3L + 2.5). M5's F5 and F8 failed on the planner's own arithmetic (8 kills
   for 4 wraps in 48 days that give ~6; 80 knife uses where worlds had 6–22); write the arithmetic next to the bar and
   check it against a calibration run before trusting the bar — the premise too, not only the product: M5.5 F9
   assumed knapping level ~1 in days 1–12, and the held-out worlds had their top knapper at level ≥ 6 by day 2–10, so
   the "quality gain ≥ 8" could not happen (M5.5 D24).
3. Make every check a record can fail (M5.5 F6: a quarrel's cause must be a record that exists, fits the cause — grief
   a death, cold a cold snap, the relation the same pair's quarrel — belongs to the initiator and lies within 2 days;
   `mood::check_cause`, tested on each failing case; the review found the first version accepted mismatched records).
4. A ratio bar is unmeasured, not passed, in a world where the compared quantity is 0 in both groups: M5.5's F4 scorer
   counted 0 ≤ 0.8 × 0 as a pass in 29 of 32 worlds (nobody cold at night: housed people lose the marker), and the 3
   measurable worlds all failed. Count the measurable worlds and report the bar on them.

## Lessons

| date | symptom | root cause | fix | source |
|---|---|---|---|---|
| 2026-10-23 | M5's F3a booked a cold snap's 8 exposure deaths to a wolf pack that had left a week earlier | the probe keyed each pack's window on `wildlife.pack_day`, which nothing ever clears, so a window ran until the next pack | bound a measurement window by the event's own end (its pressure active or its wolves on the map), and check a window's deaths against the event's own cost field | `rust/crates/sim-test/src/probe/m5.rs`; `docs/v8/m5/M5_EVALUATION.md` |
| 2026-10-23 | F7's "every quarrel has a resolvable why-chain" passed 210 of 210 although 175 chains stopped at a mood modifier with no chronicle entry behind it | the registered reading (≥ 2 links) holds by construction: every quarrel is chronicled with its cause, which is itself a link | before registering a bar, ask which world would fail it; a bar no world can fail measures the code path, not the behaviour | `docs/v8/m5/m5_score.py`; `docs/v8/m5/M5_EVALUATION.md` |
| 2026-10-23 | a quarrel test read cause "relation" where it had set up a hunger quarrel | its helper returned the chronicle's *last* quarrel, and the world had chronicled a quarrel between two other people at setup | a helper that waits for an event looks only at entries made after it started (`chronicle.len()` before), and the test asserts the event's subjects are its own pair | `rust/crates/sim-test/tests/colony_mood.rs` |
| 2026-10-23 | the M5 F9a bar (births / woman-year 0.20–0.30 "unchanged by M5") failed at 8 calibration worlds with 0.32–0.44 | the band was M1.5's no-orders 3-year measure; the scripted 2-year colony reads higher even before M5 (M4 measured the same way: 0.336) | read an "unchanged" bar against the previous pass run the same way (detached worktree at its report commit, its own script, same seeds and days) and report both numbers | `docs/v8/m5/M5_DECISIONS.md` D14 |
| 2026-10-22 | the M3 evaluation on seeds 400–431 failed F1, F4, F5 and F9, while calibration seeds 0–7 passed them all | one world (403) sealed its camp in a hill basin (undug hill impassable since M3), and the script's spot pickers checked fewer conditions than the commands they fed (no people, no other building's door) | keep the failing held-out result as the result of record; fix the causes; measure the fixed build again on a second, never-used seed set (600–631) and report both | `docs/v8/m3/M3_DECISIONS.md` D17; `rust/crates/sim-test/src/probe/colony.rs` |
| 2026-10-22 | the M3 skill bar F4 (sleepers in a bedroom) read 0 on nights when people slept there | one sample at 02:00 missed workers who walked home late from the forest and fell asleep after it | count distinct sleepers over hourly samples across the night window instead of one instant | `docs/v8/m3/M3_DECISIONS.md` D15; `rust/crates/sim-test/src/probe/space.rs` |
| 2026-10-22 | the M1.5 gate's identity step failed with "No such file … base/probe/summary.json" and perf with no base summary | `--base-data` was given the data root (`$DATA/base`), while the identity capture lives in `$DATA/base/identity` and the gate also reads `<base>/perf/perf_summary.json` | pass the identity capture directory itself and put the base `perf.sh` output in its `perf/` subfolder before the gate; a gate with a wrong base path is rerun whole | `tools/v8/gate.sh`; `docs/v8/m1_5/M1_5_DECISIONS.md` D19 |
| 2026-10-21 | a probe exposure counter (fertile, paired, fed woman-days) disagreed with the system it measured, and a probe-side job-release observer attributed only 50 of 131 releases | the observer saw the state after the whole tick, when a released worker had already been set `Idle` by a later system | count an event where the code makes the decision (a measurement-only counter at the seam, read by nothing), and check the count against the system's own tally (`counted == needs_tally`) | `rust/crates/sim-systems/src/runtime/jobs/seam.rs`; `docs/v8/m1_5/M1_5_DECISIONS.md` D3 |
| 2026-10-21 | a measurement patch applied to the base worktree (benchmark, chronicle-out, churn counter) had to be proved inert | measurement code inside sim crates could change behaviour | run the patched base probe and `cmp` its colony `summary.json` with the unpatched base's (byte-identical on 32 seeds, with and without the U-script) before trusting its numbers | `docs/v8/m1_5/M1_5_DECISIONS.md` D3 |
| 2026-10-15 | claims of "no behaviour change" needed proof; shared-key comparisons could miss added or dropped keys | — | identity before claiming no behaviour change: full summaries minus timing including key sets, raw sample hashes, chronicle by content, determinism twins, on the exact final build | `v7-final-art:docs/revitalize/DECISIONS.md` L870, L1202, L846; `v7-final-art:docs/revitalize/REVIEW_PASS21.md` Round 1 M2 |
| 2026-10-18 | the identity validator passed a corrupted chronicle with exit 0 | a review fix put `2>&1` on its own line after the heredoc inside `$( … )`: an empty command | redirect on the command itself; add a negative test (one changed value must FAIL); show every comparator can fail | `0ade8ffd`; `v7-final-art:docs/revitalize/DECISIONS.md` L1204; `v7-final-art:docs/revitalize/REVIEW_PASS20.md` Round 2 MAJOR A |
| 2026-10-18 | chronicle files were never byte-identical across builds | each file carries the `git_sha` label and a `kinds` vocabulary that grew 17 → 22 | compare by content; require the base kind list to be a prefix of the candidate's; say "identical in content" | `v7-final-art:docs/revitalize/DECISIONS.md` L1202; `v7-final-art:docs/revitalize/REVIEW_PASS20.md` Round 1 N1 |
| 2026-05-25 | 8 runtime errors (FFI calls to missing `#[func]`, parse errors, warnings) appeared only in a windowed run, after 32+ approved pipeline runs | static review and Rust tests never exercise GDScript parsing or the runtime FFI binding | gate the strict check (parse + warnings as errors + FFI name diff) and a headless boot smoke | `0238aef2`; `v7-final-art:CLAUDE.md` "Pipeline GDScript Strict Check" |
| 2026-10-20 | the V7 test suite was green, yet none of its default-rule tests exercised 8 of the 28 live systems with their logic on | V7-era tests ran on `WorldRules::default()`, where every `SocietyRules` option is `None` and those systems return at once | test live behaviour on the baseline rules (`baseline_engine`); count a system as covered only when it ticks with its rule present | `83a4f2dd:docs/v8/harvest/test_triage.md` §1, §3; `v7-final-art:rust/crates/sim-core/src/rules.rs` |
| 2026-10-11 | metadata tests passed while the renderer never wrote the field; a tint test passed before the change it guarded | the fixture set the metadata itself; the assertion was vacuous | drive the real code path; confirm each new test fails first | `v7-final-art:docs/art/production5/runtime_review.md` L7; `v7-final-art:docs/art/production3/DECISIONS.md` L20 |
| 2026-10-10 | each renderer rewrite broke hundreds of Rust tests that read GDScript text | tests pinned implementation literals with `src.contains(…)`, hashes and line counts | prefer behavioural tests; re-pin only to an equivalent invariant or retire with a reason | `v7-final-art:docs/art/TEST_LEDGER.md`; `v7-final-art:docs/art/production/BRIEF.md` L122-127 |
| 2026-10-12 | checks passed with nothing checked (0 cancels; a missing run skipped; a scene missing from an allow-list skipped determinism) | absence treated as success | require expected counts > 0, fail on missing inputs, one shared list for both call sites, reject unknown labels | `v7-final-art:docs/revitalize/REVIEW_PASS19.md` Round 1 finding 1; `v7-final-art:docs/revitalize/DECISIONS.md` L572, L579, L601 |
| 2026-10-12 | tests could not fail as described (a CLI test accepting either exit code; a twin on a seed with no second village) | assertions weaker than their names | ask "which bug still passes?"; assert exact values, ids, exit codes; pick seeds that exercise the path | `v7-final-art:docs/revitalize/DECISIONS.md` L566, L636, L638; `v7-final-art:docs/revitalize/REVIEW_PASS20.md` Round 1 m6 |
| 2026-10-11 | test literals computed by the code under test proved nothing | — | compute expected literals independently (Python re-implementation; frozen copy of the old function plus a property test) | `v7-final-art:docs/revitalize/DECISIONS.md` L462, L1121 |
| 2026-10-10 | the required 3-day survival test failed (11 dehydration deaths vs 0) and tuning could not make it green | a real behaviour defect | keep the failing assertion; never re-pin, ignore or delete it to make a gate green | `v7-final-art:docs/revitalize/DECISIONS.md` L28, L101–115 |
| 2026-10-16 | rule paths that never occurred in play (fission cancel, remnant merge, porter loss) were claimed verified | evaluation worlds never produced those events | synthetic tests that force each write path; say "verified only synthetically" | `v7-final-art:docs/revitalize/REVIEW_PASS19.md` Round 1 finding 1, Round 2 finding 3; `v7-final-art:docs/revitalize/REPORT_PASS20.md` |
| 2026-10-16 | determinism twins did not cover new state | twins compared only older state | extend the twin to every new resource (relations and chronicles each tick and at the end) | `v7-final-art:docs/revitalize/DECISIONS.md` L1122, L1200 |
| 2026-10-12 | measurement-only state risked entering behaviour or identity | new measurement fields live in engine resources | never read them in systems; keep them out of digests and compared outputs | `v7-final-art:docs/revitalize/DECISIONS.md` L599, L704, L1255 |
| 2026-10-19 | Pass 21 U3 failed its reconciliation bar (8 of 21 reconciled by day 120; 7 still recovering) | censoring: a "by day 120" metric cannot count events after the run ends; late estrangements need 10–15 days | report censored counts; measure time-to-event only with enough remaining window, or use survival statistics registered in advance | `v7-final-art:docs/revitalize/METRICS.md` L3212; `v7-final-art:docs/revitalize/REPORT_PASS21.md`; `v7-final-art:docs/revitalize/DAUGHTER_DIAGNOSIS.md` |
| 2026-10-19 | Pass 21 U6 natural p99.9 ×1.400 > 1.35 (MISS); Pass 18's p99.9 PASS rested on one repeat's hiccup | tail percentiles from 3 repeats are noise: single 20–174 ms ticks, per-repeat ratios ×1.023 / ×1.490 / ×0.516 | report as noise; use ≥ 5 interleaved repeats or a trimmed statistic registered in advance; list every repeat | `v7-final-art:docs/revitalize/PERF_BUDGET.md` L215–217, L158; `v7-final-art:docs/revitalize/REVIEW_PASS21.md` Round 1 N7 |
| 2026-10-15 | successive builds moved the mean ratio 1.166 → 1.150 → 1.167 while the code got cheaper | hold-to-hold noise (±2 points) as large as the improvement | declare one official run on the final build; report development runs beside it; never select favourable repeats | `v7-final-art:docs/revitalize/DECISIONS.md` L932, L1048–1049; `v7-final-art:docs/revitalize/REVIEW_PASS18.md` Round 1 D5 |
| 2026-10-10 | per-agent-tick cost differed between 10- and 120-day runs of the same rules | cost grows with run length; the stress world is one camp | compare against the same-host, same-duration, same-build baseline, interleaved; use a natural seed set picked before measuring | `v7-final-art:docs/revitalize/DECISIONS.md` L326; `v7-final-art:docs/revitalize/PERF_BUDGET.md` §1.3 |
| 2026-10-15 | timed sweeps ran under contention while their JSON said `contended: false` | the load gate was checked once per batch; foreign and own load rose during the run | gate every sweep, record start loads, no subagents during holds, rerun contended measurements | `v7-final-art:docs/revitalize/DECISIONS.md` L786, L793, L1071; `v7-final-art:docs/revitalize/REVIEW_PASS15.md` Round 1 M1 |
| 2026-10-13 | Art 7 layer cost read +62 % (9.77 → 15.83 ms) and could not be attributed | host 1-minute load fell 176.7 → 36.2 between the two samples | never compare timings taken under different load; use admission control | `v7-final-art:docs/art/production7/DECISIONS.md` L25 |
| 2026-10-18 | Art 12 measured nothing: 21+21 admission holds refused, three attempts burned within seconds | a "no foreign process above 50 % CPU" rule was too strict for a 14-core Mac; retries had no spacing | replace process-list rules with a measured interference probe | `v7-final-art:docs/art/production12/REPORT.md` L79-81; `v7-final-art:docs/art/production12/DECISIONS.md` L7 |
| 2026-10-19 | needed a reproducible "is the Mac quiet enough" test | a busy core, not foreign processes, competes with a single-threaded loop | calibrated admission probe (Art 13): ~2 s fixed single-thread work; R = median of the 5 fastest of 15; valid iff probe before and after ≤ 1.05 R and 1-min load < 8; retry ≥ 30 min after the last attempt ended, ≤ 6 attempts, ≤ 3 h | `v7-final-art:docs/art/production13/GOAL.md` L59-78; `v7-final-art:docs/art/production13/DECISIONS.md` L10-11 |
| 2026-10-20 | an earlier run's calibration would have admitted the wrong sessions | Art 13b's R was 2.762 s against Art 13's 2.009 s; quiet-machine speed drifts | recalibrate in every run; never normalise timings by R | `v7-final-art:docs/art/production13b/REPORT.md` L223, L301-303 |
| 2026-10-17 | debug numbers were reported as performance (W500 51 ms debug vs 29 ms release) | Godot loads the debug-path dylib and the gate rebuilds it | measure release builds only; debug captures are never perf evidence | `v7-final-art:docs/art/production11/REPORT.md` L17, L235 |
| 2026-10-14 | `cargo test --workspace` exceeded the 1,800 s gate bound (one fission test 764 s) | the test profile was unoptimised | `CARGO_PROFILE_TEST_OPT_LEVEL=1` or `2`, keeping debug assertions and overflow checks on | `v7-final-art:docs/art/production8/DECISIONS.md` L10-12 |
| 2026-10-14 | CPU stayed busy after a gate timeout and contaminated later measurements | the bounded runner killed `cargo` but left the test binary orphaned | kill the whole owned process group on timeout; check for owned orphans before measuring | `v7-final-art:docs/art/production8/DECISIONS.md` L10; `v7-final-art:docs/art/production9/DECISIONS.md` L18 |
| 2026-10-10 | probe runs picked up later edits; an in-place gate had to be stopped | the probe rebuilt from a working tree that kept changing | freeze probe binaries; run base probes and gates from a detached worktree of the committed head | `v7-final-art:docs/revitalize/DECISIONS.md` L296, L358, L1031 |
| 2026-10-12 | a green gate did not test any single commit | debug tests ran on one tree, release on another, later steps on the moving working tree | run the final gate on the report commit with nothing edited or committed while it runs | `v7-final-art:docs/revitalize/DECISIONS.md` L622, L1143; `v7-final-art:docs/revitalize/REVIEW_PASS20.md` Round 2 MAJOR B |
| 2026-10-11 | empty Python test discovery exited 5 and broke the gate | `unittest discover` with no tests exits 5 | keep at least one test (or an explicit skip) in `tools/v8/test_*.py` | `v7-final-art:docs/revitalize/DECISIONS.md` L458 |
| 2026-10-20 | M1 hut test failed: a site reservation was still held one tick after the hut finished | the finishing builder releases its `Site` key in the job stage after the site's last progress tick; a haul in flight may hold a `Cell` legitimately | assert reservation release after one more tick, and check per agent and per key kind, not "reservations empty" | `rust/crates/sim-test/tests/colony_jobs.rs` |
| 2026-10-20 | the colony world test expected 18 wild regions and saw 11–14 | regions must lie within `region_water_max` of water; the 192 × 192 map cannot always fit 18 | assert the measured floor (≥ 10, observed 11–14 on seeds 0–7) and record the rule, not the request | `rust/crates/sim-test/tests/colony_world.rs`; `docs/v8/m1/M1_DECISIONS.md` D17 |
| 2026-10-22 | an identity run with `CARGO_TARGET_DIR` set to a scratch directory would have compared the base with a stale probe | `tools/v8/identity.sh` builds with cargo but runs `rust/target/release/worldsim_probe` by path | run identity (and every script that runs a binary by path) with the default target directory; give a parallel worker its own target dir only for `cargo test` | `tools/v8/identity.sh`; `docs/v8/m2/M2_DECISIONS.md` D19 |
| 2026-10-22 | `engine_ascii.rs` failed late in M2 ("no carrier to check at tick 600") after several green targeted runs of the `colony_*` files | a colony behaviour change moved the moment the first loads are carried; a test outside the colony files pinned that moment, and the full suite was not run until the end | after a colony behaviour change run the whole `cargo test -p sim-test --no-fail-fast`, not only the files you touched; a test that needs an event should step to the first tick it happens instead of pinning a tick | `rust/crates/sim-test/tests/engine_ascii.rs`; `docs/v8/m2/M2_DECISIONS.md` D22 |
| 2026-10-22 | M2 gate 1 failed only on `bench` (N = 1,000 ×1.14, N = 2,000 ×1.22) while every other step passed | the 1-minute load was 19.7 during the step against 3 when the ledger rows were measured; the baseline-only perf scene read ×1.83 in the same gate | before a gate, check `uptime` and start it in a quiet window (the bench step runs last, ~70 min in); read a red bench next to the step's recorded load and confirm with an interleaved base/candidate pair before touching code | `tools/v8/bench.sh`; `docs/v8/m2/M2_DECISIONS.md` D23 |

