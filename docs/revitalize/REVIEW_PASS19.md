# Pass 19 — independent review

Reviewer: a fresh general-purpose subagent given only the brief (`PASS19_GOAL.md`), the diff `20a1d078..0935cb1a` and the outputs (docs and raw data, read-only), 13:19–13:51 KST, while the first gate and the final-build identity ran. It ran no build; it re-ran the identity comparison, the chronicle and H evaluators (byte-identical results) and short probes with `bin_p3 --workers 1` after checking that no quiet lock existed. Round 1 is reproduced verbatim below, followed by the dispositions.

## Round 1 (verbatim)


Scope: brief `docs/revitalize/PASS19_GOAL.md`, diff `20a1d078..0935cb1a`, outputs under `docs/revitalize`, raw data in `/Users/rexxa/work/p19-data` (read only). Nothing in the repository was modified. My outputs are in `/private/tmp/claude-501/p19-review/`. `/Users/rexxa/work/.mac_quiet.lock` was absent before every binary run.

No BLOCKER. 1 MAJOR, 5 MINOR, 3 NOTE.

The final-build identity check (H6) is still running into `docs/revitalize/probe/p19-4_final`. So far I1, I3, I4 and I2 pass against the base; I5–I7 have not run. H6 is therefore not judged here.

### What I ran

- **Identity, base vs Part 1.** `compare_identity_pass17.py` on the committed summaries, `probe/p19-0_base` vs `probe/p19-1_identity`, for i1_v19, i3_v16, i4_v20, i5_v21, i6_v22 and i7_v23: all six report `PASS: 0 differences`. `diff` of the two `i2_sha256.txt` files is empty (20 hashes).
- **Identity, final build so far.** i1, i3 and i4 PASS against the base; I2 is identical.
- **Pre-rebase commits.** The measurement binaries were built from pre-rebase commits, so I compared each pair's `rust/` tree (`git diff A B -- rust`): 0ca95174 vs 9d6fb0dd, 3322601f vs b0c31465, c2309a70 vs 949d5b8c, and ee6b30c5 vs 44a7d991 all give 0 lines.
- **Binary hash.** `bin_p1/worldsim_probe` sha256 (b0f39dbc…) matches `p19-1_identity/binaries.sha256`.
- **Chronicle checks.** Re-ran `evaluate_pass19_chronicle.py` on `p1/c_A` with twin `p1/c_B`. The result equals the committed `chronicle.json`, and the chronicle sha256s match `chronicle_sha256.txt`.
- **H1–H4.** Re-ran `evaluate_pass19.py` (3 min). The new `eval.md` and `eval.json` are byte-identical to the committed ones.
- **Short probes** with `bin_p3`, `--workers 1`:
  - `game_v23`, seeds 0–1, 8 days, against `bin_base`: identical.
  - The same run with `--chronicle-out`: summary unchanged.
  - `game_v19`, seeds 0–2, 120 days, with chronicle: the `Ranked` cause appears (14 + 14 members), Departed equals the fission trace's members, and deaths match (108 / 58 / 73).

### Verdicts per point

**A. Identity and pins: OK.**
- Both `validation.txt` files support "bit-identical" for I1–I7 and I2 on the Part 1 build, and my re-run agrees.
- The pins (`harness_revitalize_pass19_frozen.rs`) were committed alone in 5663058f, right after the brief and before any Rust change, and never modified afterwards.
- The pin file has one test per seed 0–4 plus a field-by-field rules test (156 `assert_eq!`). Each seed checks the fingerprint and `state_digest` at 1,440, 40,320 and 86,400 ticks.
- H6 on the final build is incomplete, as expected (see above).

**B. Write-only chronicle: OK.**
- The added production lines contain no `causal_log`, `issue_event_id`, `insert_one`, `remove_one`, `world.insert`, `world.remove` or `query_mut`.
- Chronicle reads inside systems are limited to `ensure_person` table lookups and `Chronicle::has_founding_member` (settlement_system.rs ≈L787). `has_founding_member` only chooses between `Joined` cause 1 and `FoundingMember`.
- The new queries are read-only: `chronicle_log::alive_entity`, and the `(&Agent, &Founding)` query in `chronicle_membership`.
- Iteration order is unchanged:
  - `despawn_agent`: `values_mut` → `iter_mut` walks the same `HashMap` and makes the same mutations in the same order. Only `last_roster` is captured, and every read happens before `world.despawn`.
  - `sync_membership` / `resolve_exclusivity`: `insert` and `remove_member_agent` are still called unconditionally; only their return value is captured. `exclusivity_moves += 1` is still unconditional.
  - Pairing: the `min_by` comparator is identical; it now also keeps the familiarity value.
- `FissionSystem::check` calls `select_departure_explained` once with the same arguments, and the `else { continue }` is unchanged.
- `chronicle_log` is `pub(crate)`.

**C. select_departure equivalence: OK.**
- The frozen test copy equals `20a1d078`'s `select_departure` (`diff -w` is clean; only indentation differs).
- `select_departure_explained` keeps the set logic, with `BTreeMap` keys in place of `BTreeSet` and `contains_key` in place of `contains`.
- Each member is labelled on first insertion via `entry().or_insert`:
  - the seed phase's iterated adult → `Seed`;
  - other members of a household taken by the seed or fill → `Household` if ranked, `ElderPartner` if not;
  - top-up singles → `TopUp`.
- The no-family path is labelled `Ranked` in `fission.rs` ≈L778–783.
- `select_departure` is now `explained.map(into_keys)` with the same signature.
- There is a property test over 2,000 cases; existing tests are unchanged; a per-phase label test is added.

**D. C1–C4 by hand on seed 19200: OK.**
- **Birth.** Child 29 at tick 2281, mother 20, father 8, cause 0, settlement 1: equal to `family_events`. The person table has `born_tick` 2281 and parents 20 / 8. A fallback birth (child 27 at tick 720, parents 0 / 1) has cause 1, matching `fallback: true` in the trace.
- **Pairing.** 20 & 8 at tick 2160, settlement 1, value 200.
- **Fission 1** (departs 44520, founds 44721, leader 14):
  - Seed: {14, 27, 30, 38}
  - Household: {29, 35, 40, 41, 44, 45}
  - ElderPartner: {11, 15}

  The ElderPartner count of 2 equals the trace's `elders_departed`. The labels are consistent with the trace's households and couples. Fission 2 (no elders) also matches.
- **Death and widowhood.** 20 dies at tick 31860, cause 3, settlement 1, value 87471 (= tick − born_tick), other 8. The deaths trace says 31861 because it records the completed tick; the evaluator subtracts 1. `Widowed` (subject 8, other 20) follows immediately at the same tick, and 8 dies later at 39300.
- **All 32 worlds:**
  - Widowed exactly when the recorded partner is still alive (1471 / 1471), and correctly absent in the other 68 cases.
  - Every Widowed directly follows its Died.
  - Died settlement = `settlement_at_death` (2891 / 2891).
  - Born cause = trace `fallback` (4359 / 4359).
- **Membership by diff.** Between the brief's snapshot point and post-exclusivity, the only roster mutations are:
  - the `sync_membership` insert (captured);
  - the formation-scan `add_member_agent` (captured via `founding_members`);
  - the `resolve_exclusivity` removal (captured);
  - the `sync_membership` `retain` (not captured; finding 7).

  So the net of admitted minus removed equals the snapshot diff, apart from the `retain` exception. `had_roster` is right (post rosters minus joins, plus leaves). The roster-replay harness test confirms this in three scenes: `game_v23` through a fission, `game_v16` without fission, and legacy.

**E. end_minute readers: OK.**
- `grep -rn day_end_minute rust/crates` finds: the field (routine.rs:13), the rule literal (rules.rs:885), the `None` return inside `end_minute` (routine.rs:106), doc comments, and two readers.
- Both readers go through `end_minute`: `can_enter` (routine/mod.rs ≈L186) and `deadline` (lifecycle.rs ≈L199).
- The formula is brief 7.2's: computed in i64, `/` truncating toward zero, absent temperament = P 500, key term `(key/60) % key_span`. Unit tests cover 1020, 1139, and the truncation edges at P 511, 512, 489 and 488.
- `None` returns `day_end_minute` exactly.
- Harmless additions:
  - `key_span.max(1)`;
  - a clamp to u32;
  - `deadline`'s `key.unwrap_or(id)`, which is documented.

**F. H verdicts: OK.**
- **Registration came first.** The reflog has ee6b30c5 (METRICS registration) at 11:51:15. That is before the only hold (12:39:39, which contains H5) and before the `game_v24` evaluation (12:43:47–13:06:42 per `probe/p19-2_eval/runlog.txt`). METRICS was not edited afterwards: `git diff ee6b30c5 44a7d991` and `git diff 44a7d991 0935cb1a` on METRICS both give 0 lines.
- **The script matches the METRICS definitions:**
  - window [960, 1200) on display minute = (tick + offset) % 1440;
  - `ended` rows only;
  - day-end exit = completed and tick < until_tick;
  - at least 5 exits per person;
  - pooled Spearman;
  - Pass 17 code for H3 and H4;
  - world-resample bootstrap.
- **The failures are reported honestly.**
  - H1 FAIL: 10 vs 31, ratio 0.323 [0.267, 0.375], reported with its registered diagnostics (day-end exits alone give 0.125; the remaining edges are at 16:01 and 18:01).
  - H5 FAIL on the mean: +17.6 %.
- **The passes are justified.**
  - H2: ρ 0.849 [0.840, 0.857] over 4,419 people.
  - H3: all rules pass.
  - H4: 0.1090 vs 0.1100.
- **H5 follows the brief.** Five interleaved repeats (v20, v24, v23 per repeat), medians of five: p99.9 +18.2 %, mean 0.372 ms, slope 0.993. The 18:00-window tick is reported for v20 / v23 / v24: 0.505 / 0.873 / 0.614 ms.

**G. Quiet-lock record: OK.**
- **The hold.** `quiet_hold.log` shows one hold, 12:39:39–12:40:55 (76.1 s), load 3.84, `busy=[]`, uncontended, after a 37-minute wait through the art run's cargo and Godot activity.
- **Timing.** Run timestamps (C6 at 03:39:39Z through scale and subprofile at 03:40:48Z) fit inside the hold.
- **Own jobs were stopped.** The RUNLOG has STOP/CONT lines for groups 37966, 56453 and 91549.
- **The scripts:**
  - `run_watched_p19.sh` stops a job on a foreign lock or on its own PAUSE, polling every 10 s.
  - `quiet_lock_p19.sh` limits a hold to 3600 s, keeps 600 s between holds, polls every 300 s, takes the lock anyway after a 3600 s wait, and requires load < 4.
  - `wait_mac_p19.sh` is identical to the p18 copy apart from renames.
- **Coverage.** Every build, probe, identity probe and gate step goes through the watcher: `watched` in `identity_p19.sh`, `step` in the gate. The gate re-runs `perf_quick` if it was stopped.

**H. Allowed paths and public items.**
- Paths: OK. `git diff --name-only` shows nothing outside `rust/`, `tools/revitalize/` and `docs/revitalize/`, and nothing under `sim-test/src/worldmodel/`.
- Public items: MINOR, see finding 3.

**I. Other.** Findings 1, 2 and 4–9.

### Findings

#### 1. MAJOR — four chronicle write paths have never run in any check

**Paths:**
- `FissionCancelled` / `DepartureCancelled` (fission.rs ≈L252–272);
- the remnant merge, `LeftSettlement` cause 0 followed by `SettlementDissolved` cause 1 (≈L472–501);
- `SettlementDissolved` cause 0 (fission.rs ≈L465, settlement_system.rs ≈L1201);
- `DepartCause::TopUp`, which appears only in a unit test.

**Evidence:**
- `probe/p19-1_chronicle/chronicle.md` shows `fission_cancelled 0/0` and `top_up 0`.
- None of the 32 worlds has a dissolved entry or a left-cause-0 entry.
- No test names these kinds (I grepped sim-systems, sim-test and sim-bridge).
- My 3-seed `game_v19` run had none either.
- So C1's "every FissionRecord … cancel has exactly one matching entry" passes only because there are no cancels.

**Fix:** add a harness test that forces both events:
- a cancel, e.g. by invalidating the destination or killing the leader mid-walk;
- a remnant merge, by keeping a settlement at or below `remnant_members` for a day.

Then assert the entries, their order and the roster replay:
- `FissionCancelled`, then `DepartureCancelled` for living members in AgentId order;
- `LeftSettlement` per member, then `SettlementDissolved` cause 1.

Alternatively, run the C checks on a scene where these events occur.

#### 2. MINOR — identity claim stated before its evidence exists

**Evidence:**
- ROUTINE.md (Pass 19 section) states "game_v21–game_v23 are bit-identical, I5–I7" for the Part 2 code. But the Part 1 identity did not include the `end_minute` change, and H6's I5–I7 have not run.
- Current evidence is only `p2_pins.log` (pins, seeds 0–4) and my 8-day spot check on seeds 0–1.
- The H evaluation also pairs a `game_v23` baseline from `bin_p1` with `game_v24` from `bin_p2`, which relies on the same `None`-path identity.

**Fix:** cite H6 in ROUTINE.md and DECISIONS once it finishes.

#### 3. MINOR — new public items outside the brief §2 list

**Evidence:**
- `WorldRules::V24_ROUTINE` (`pub const`, rules.rs ≈L935).
- sim-test `pub mod probe::chronicle_out`, with `pub fn split`, `document`, `write` and `twins_match`.
- `pub fn probe::run_with_chronicle`, plus its re-export.

Items inside the allowed modules are fine: the `sim_core::chronicle` methods (`has_founding_member`, `person_count`, `approx_bytes`, …) and the `ffi/chronicle.rs` collectors. `scenes::game_v24_scene` counts as scene wiring.

**Fix:** make these private or `pub(crate)` where possible, or record them in DECISIONS as a deviation.

#### 4. MINOR — brief 7.4's trace-window report is missing

**Evidence:** Brief 7.4 asks for a report of what the unchanged probe trace windows show under `game_v24` (`agent_trace.rs` `night_of` ≥ 1080, and the idle tether 360..1080). No Pass 19 doc reports it; I grepped the Pass 19 additions for night_of, tether and "trace window".

**Fix:** add one paragraph to ROUTINE.md or eval.md.

#### 5. MINOR — DECISIONS RUNLOG table is incomplete

**Evidence:** Only Parts 0 and 1 have rows, and Part 1's end reads "(see below)". Parts 2, 3 and 4 have no rows, even though Parts 2 and 3 are pushed. Brief 0.3.1 asks for a (Part, start, end, notes) table.

**Fix:** fill in the rows.

#### 6. MINOR — an attribution without data in PERF_BUDGET

**Evidence:** PERF_BUDGET (Pass 19) says the remaining +17 % mean regression "is the routine's own per-tick work (not attributed further in this pass)". But v23 vs v20 also includes the family rules from v22 and v23 (pairing, households, elder partners).

**Fix:** soften it to "lies in the rules v21–v23 added", or measure it with `subprofile`.

#### 7. NOTE — membership-diff edge cases

- The removals made by `sync_membership`'s `retain(|aid| agent_positions.contains_key(aid))` are not added to `removed`.
  - This is a no-op today: every agent death goes through `despawn_agent`, which cleans rosters first, and the only other `world.despawn` in production is for construction sites.
  - It is still a latent divergence from the brief's snapshot diff.
  - Suggest capturing those removals, or adding a `debug_assert` that `retain` removes nothing.
- A member carrying `Founding` who is admitted to a third roster on the way is never recorded. This follows the brief's skip rule.
- The extra `LeftSettlement` before `FoundingMember`/`Joined`, for a member of a newly formed settlement, is a documented and reasonable extension.

#### 8. NOTE — bridge and UI details

- `collect_person` uses −1 for an unknown `born_tick`, but founders' born ticks are negative, so −1 is ambiguous.
- `alive` scans the whole world on every call, which costs O(N) per lookup if the UI polls it every frame.
- Fallback births record the two lowest member ids as mother and father, and they can both be women. In seed 19200, child 27's "father" (agent 1) is female in the person table. This predates Pass 19, but the UI should read Born cause 1.
- In the legacy scene (`bootstrap_spawn_agents`), founders' records are created lazily, at their first entry. The harness roster-replay test still passes there.

#### 9. NOTE — smaller points

- **C5 memory estimate is a lower bound.** `Chronicle::approx_bytes` counts lengths, not capacity or BTreeMap node overhead, so 12.2 B per person-day (34.2 MB for 10,000 people × 10 years of 28 days) is a minimum.
- **Units in SCALE.md.** Its "MB" values are MiB. Max RSS is flat at 27.7e6 bytes in every raw `time.log`; the data is real and the explanation is given.
- **H5 slope procedure.** The H5 decision-system slope is one run per rule, and v24 runs before v20, against the brief's "old rules then new".
- **Chronicle growth.** The chronicle grows without bound in the live game.
- **C1 checker coverage.** It compares neither the settlement for Born / Paired / Died nor the Born cause. I checked these by hand and they match.
- **DECISIONS record.** The shared-Mac section names "identity I6 of the Part 1 build, run B" as the stopped jobs, but the RUNLOG also shows the base I6 (pgid 37966) stopped 12:36:43–12:41:04. The behaviour was correct; the record is incomplete.
- **Scene error message.** `probe/args.rs` still leaves `game_v24` out of the list of valid scenes.
- **Branch state.** The final commit 0935cb1a is on local branch `p19-part3`, and the gate is running on it; the remote WIP branch is at e2e399cb (Part 3). Part 4 is in progress.

### Overall verdict

The chronicle is behaviour-preserving as far as the evidence reaches:
- Part 1's I1–I7 and the pins are bit-identical, and my re-runs and the final binary's I1, I3, I4 and I2 agree.
- Nothing in the diff pushes to `causal_log`, issues event ids, changes component sets or iteration order, or decides anything from a chronicle read.
- `select_departure_explained` provably chooses Pass 18's set and labels causes as brief 6.3 asks.
- C1–C4 reproduce exactly and survive a hand trace, including widowhood across all 32 worlds.
- `end_minute` is the only path to `day_end_minute`, and its `None` path is exact.
- The H rules were registered before any measurement; the evaluation is byte-reproducible; the H1 and H5-mean failures are reported plainly with diagnostics.
- The quiet-lock procedure and its record satisfy brief 1.2.

What remains open is evidence, not correctness:
- the cancel, remnant-merge, dissolution and TopUp write paths have never run in any check (MAJOR, finding 1);
- the `game_v21`–`game_v23` identity of the Part 2 code is waiting on H6.

Once finding 1 has a harness test, H6 completes, and the minor documentation and visibility fixes are made, Pass 19 is ready for the gate.

## Round 1 dispositions

1. **MAJOR — four write paths never exercised → fixed (test only).** New `rust/crates/sim-test/tests/harness_revitalize_pass19_chronicle_events.rs`, reusing Pass 11's fission setups: (a) a cancelled fission with one member killed on the way (`despawn_agent`) — `FissionStarted`, every member `Departed` with cause `Ranked` in AgentId order, then `FissionCancelled` directly followed by `DepartureCancelled` for exactly the living members in AgentId order, the dead member's `Died` and closed record; (b) a remnant of two members spawned in descending id order and an empty settlement — `SettlementDissolved` cause 0 for the empty one, then `LeftSettlement` cause 0 for 101 and 102 (ascending) and `SettlementDissolved` cause 1, all at the merge tick, members' records present; (c) `SettlementSystem::run_dissolutions` — `SettlementDissolved` cause 0. 3/3 pass in release and debug, clippy clean. `TopUp` stays covered by the `select_departure` unit tests (the per-phase label test and the property test over the same code path); no evaluation world produced one.
2. **MINOR — identity claim before evidence → resolved.** H6 finished: final build (`bin_p3`, tree of `949d5b8c`) I1–I7 zero non-timing differences, I2 equal, DETERMINISM PASS ×6 ([validation](probe/p19-4_final/validation.txt)); ROUTINE.md now cites it. Gate 1 (`0935cb1a`) also ran every pin (debug and release, 2,288/0/0).
3. **MINOR — public items outside the §2 list → recorded as a deviation (not changed).** `WorldRules::V24_ROUTINE` follows the existing `V21_ROUTINE` / `V22_FAMILY` / `V23_FAMILY` constants that `game_v21`–`game_v23` are built from; `sim_test::probe::chronicle_out` (`split`, `document`, `write`, `twins_match`) and `run_with_chronicle` are the plumbing of the allowed `--chronicle-out` flag and are used by the `worldsim_probe` binary, so they must be public to the binary crate. Making them crate-private would change the probe binary after H6.
4. **MINOR — trace-window report missing → added** to ROUTINE.md and as a section of `probe/p19-2_eval/eval.md` (marked as computed after the evaluation from the same runs' `_agents.jsonl`): idle-tether samples +3.2 %, mean distance 5.60 vs 5.62 tiles; mean bed minute 1,326.2 vs 1,325.5, median 1,304 in both.
5. **MINOR — RUNLOG incomplete → filled** (DECISIONS.md, Parts 0–4).
6. **MINOR — attribution without data → reworded** in PERF_BUDGET.md: the remaining +17 % lies in what `game_v21`–`game_v23` added to `game_v20` (routine and family rules), not attributed further.
7. **NOTE — `retain` not captured → recorded** (DECISIONS.md and the report's limitations): a no-op today because every death goes through `despawn_agent`, which empties the rosters first; the roster-replay harness would catch a future divergence. Not changed, to keep the final build's identity binaries.
8. **NOTE — bridge details → recorded** in the report's limitations (−1 for an unknown `born_tick`, `alive` is an O(N) scan, fallback parents can both be female — read `Born` cause 1).
9. **NOTE — smaller points:** the C5 estimate is now called a lower bound (report); SCALE.md's MB are MiB (2^20 bytes; noted); the H5 slope runs ran v24 before v20 inside the slope stage (Pass 18's stage order: new then old), recorded; the shared-Mac record now lists every paused group (all 24 pauses, 1,800 s); the `args.rs` scene error message omits `game_v24` (left, to keep the final build's binaries; noted as a limitation); the branch state was in progress at review time.

## Round 2 (verbatim; fresh general-purpose subagent, 15:01–15:10 KST, the last round)


Scope: brief `docs/revitalize/PASS19_GOAL.md` (read in full), diff `20a1d078..a6371e74`, the docs under `docs/revitalize` (DECISIONS / METRICS Pass 19, CHRONICLE, SCALE, ROUTINE, FAMILY, PERF_BUDGET, REVIEW_PASS19 round 1 + dispositions, probe/p19-*, perf/p19-*, verification/p19_gate/first_0935cb1a), raw data in `/Users/rexxa/work/p19-data` (read only), and the uncommitted draft `REPORT_PASS19.md`. Nothing in the repository was modified; no state-changing git command was run. `/Users/rexxa/work/.mac_quiet.lock` was absent before every run. My scratch outputs are in `/private/tmp/claude-501/p19-review/r2/`.

**Result: 0 BLOCKER, 0 MAJOR, 5 MINOR, 6 NOTE.**

### What I re-ran / verified

- **New test.** `cargo test --release -p sim-test --test harness_revitalize_pass19_chronicle_events`: 3/3 pass, and cargo didn't need to rebuild, so the binary matches HEAD. The release binary run 16× and the debug binary run once also pass 3/3. `rust/` working tree is clean.
- **H6.** I re-ran `compare_identity_pass17.py` for base (`probe/p19-0_base`) against final (`probe/p19-4_final`):
  - i1_v19, i3_v16, i4_v20, i5_v21, i6_v22 and i7_v23 all give `PASS: 0 differences` (timing keys only excluded).
  - `i2_sha256.txt` is identical (20 hashes).
  - `p19-data/final/*.log` contain 6× `DETERMINISM PASS`, and every summary has `determinism_checked: true`.
  - The `bin_p3/worldsim_probe` and `worldsim_perf` sha256 match `binaries.sha256`.
  - `git diff c2309a70 949d5b8c -- rust` is empty; `git diff 949d5b8c a6371e74 -- rust` contains only the new test file; `b0c31465..949d5b8c` touches only `sim-test/src/perf.rs`.
  - So "every later commit changes only tests and documents" holds.
- **Gate 1** (`first_0935cb1a`): 15/15 exit 0, debug and release 2,288 / 0 / 0 each, `stopped_s 0.0` everywhere, allowed paths "outside section 2: none", head 0935cb1a, 13:18:51–15:00:07. The gate logs do not contain the new test, which is consistent with DECISIONS ("copied into the tree only after the gate finished").
- **Quiet-lock record.**
  - Across all six `p19-data/**/RUNLOG.txt` there are 24 unique CONT lines totalling 1,800.3 s. Every STOP is "own quiet hold (PAUSE)", with 0 foreign-lock stops.
  - The last pauses are 250.1 / 260.1 / 260.1 s (Part-1 I6, base I6, run B).
  - `quiet_hold.log`: one hold, 12:39:39–12:40:55, 76.1 s, load 3.84, `busy=[]`, contended=0. Busy evidence at 12:04 / 12:15 / 12:21 is cargo and at 12:10 / 12:26 / 12:31 is Godot.
  - The C6 / H5 / scale command timestamps (03:39:39Z–03:40:48Z) are inside the hold.
  - All of this matches DECISIONS and the report.
- **Evaluators.** I re-ran `evaluate_perf_p19.py` (h5 and c6) and `evaluate_scale_p19.py` on the raw `p19-data/perf/*`. All three `.md` outputs are byte-identical to the committed ones, and `h5.json` is too.
- **Trace windows (round-1 finding 4).** Recomputed from the raw `_agents.jsonl`:
  - v24: 1,379,596 idle samples, mean distance 5.603, 156,358 bed-nights, mean bed minute 1,326.16, median 1,304.
  - v23: 1,337,358 / 5.623 / 156,014 / 1,325.49 / 1,304.
  - These equal eval.md, ROUTINE.md and the report.
- **Environment.** `sysctl`: Apple M4 Max, 14 CPUs, hw.memsize 38,654,705,664. rustc 1.93.1.
- **Report numbers checked against sources:**
  - C1–C6 against chronicle.md and c6.md, including C1's 7,446 = 3,330 + 4,116 and C2's 94 + 330 + 19 = 443.
  - H1–H5 against eval.md and h5.md.
  - The 18:00-window table, the per-repeat p99.9 (+5.7 … +32.2 apart from r3's 1.566 ms) and the day-end bins.
  - The scale table, slopes and decision scopes against SCALE.md and scale.md.
  - S1–S4, including S3: 60 × 0.856 ms = 5.1 %.
  - C5 arithmetic: 20,151 / 211,073 = 0.0955; × 2.8 M = 267,314.
  - The pin count: 151 `assert_eq!` in `assert_v23_fields` (156 in the file, counting the per-seed checker).

  Every number I checked matches its source, apart from the nits in finding 9.

### Round-1 dispositions

| # | disposition | verdict |
|---|---|---|
| 1 MAJOR | new 3-test harness | Done; adequate apart from findings 1–3 below. |
| 2 | H6 cited | Done: ROUTINE.md cites H6 and DECISIONS has the H6 line. |
| 3 | public items as a deviation | Listed in REVIEW_PASS19 and the report. DECISIONS has only a parenthetical "(deviation recorded)" without the item list (finding 8). |
| 4 | trace windows | Done and reproduced exactly; no committed script (finding 9). |
| 5 | RUNLOG | Done; Part 4's end is "(report commit)", which is expected. |
| 6 | PERF_BUDGET wording | Done. |
| 7 | `retain` note | Recorded in the report and briefly in DECISIONS (L1144). |
| 8 | bridge notes | In the report's limitations. |
| 9 | smaller points | Partly. The "lower bound" is in the report but not in CHRONICLE.md. The H5 slope-order note is said to be "recorded" but I find it only in REVIEW_PASS19 itself (finding 8). |

### Findings

#### 1. MINOR — the remnant-merge order assertion is only probabilistic

The test does not reliably detect a missing sort.

**Evidence**
- `harness_chronicle_records_remnant_merges_and_empty_dissolutions` asserts `LeftSettlement` for 101 and then 102.
- The comment and disposition 1 say the members are "spawned in descending id order: the entries must still be ascending". But `Settlement::member_agents` is a std `HashSet` with `RandomState` (`components/settlement.rs:106`), so insertion order does not decide iteration order.
- A 2-element `HashSet{102, 101}` iterates ascending in 421/1000 and 445/1000 constructions (my rustc check, `r2/hs.rs`).
- So if `leaving.sort_unstable()` (fission.rs ≈L479) were removed, this test would still pass about 43 % of the time.
- The cancel test's AgentId order is guaranteed by `group.members: BTreeSet` (fission.rs:122), so it is not affected.

**Fix**
- Repeat the remnant scenario in a loop over many fresh engines (e.g. 32; each `Settlement` gets a fresh RandomState), asserting ascending order every time. Missing a broken sort then has probability below 1e-9.
- Correct the comment and disposition text.

#### 2. NOTE — the cancel test's "skip the dead" goes through `roster.get`, not `is_alive`

**Evidence**
- In `cancel`, a member gets `DepartureCancelled` only if `roster.get(member)` hits *and* `chronicle_log::is_alive(...)` is true.
- `roster()` lists every entity with `Agent + Position` (fission.rs:153), so it includes dead-but-present agents.
- The test kills its member with `despawn_agent`, so the member is absent from the roster and the `is_alive` branch is never reached. Deleting the `is_alive` check would not fail the test.
- Today every death despawns at once, so a dead-but-present member is practically unreachable. This is a coverage nit, not a bug.

**Fix (optional):** also cover a member whose `BodyHealth` is dead but who is still present.

#### 3. MINOR — the new paths are proven only synthetically, and the report's limitations do not say so

**Evidence**
- The three tests drive `FissionSystem` or `SettlementSystem` alone, on `game_v17` (no family rule, so every departure is `Ranked`).
- They do not check the roster replay round 1 suggested, e.g. the re-admission after a remnant merge (`Joined` cause 0 after `LeftSettlement` cause 0).
- No evaluation world contains a cancel, a remnant merge or a dissolution (C1 `fission_cancelled 0/0`, `left_settlement 5`, no dissolved entry).
- The report's limitations mention only TopUp ("단위 테스트에서만"). "정직한 판단" says the chronicle "32세계×120일에서 기존 추적과 한 건도 어긋나지 않았고" without saying that these event kinds never occurred there.

**Fix:** one limitation line: cancel / remnant-merge / dissolution entries are verified only by synthetic harness tests (FissionSystem alone, `game_v17`) and never occurred in the 32 evaluation worlds.

#### 4. NOTE — gate 1 does not cover the final head; H6's pins row

**Evidence**
- Gate 1 ran on 0935cb1a. a6371e74 adds `rust/crates/sim-test/tests/harness_revitalize_pass19_chronicle_events.rs`, so gate 1 neither covers nor counts it: 2,288 tests, and the gate logs have no match for `chronicle_events`.
- Brief 2 lets `codex/revitalize-2026-10` fast-forward only to a WIP commit whose gate is green. A full gate on the final head is therefore required, presumably GATE_TBD; it should report about 2,291 tests.
- The report's H6 row cites only I1–I7, I2 and determinism. H6 also requires "all pins identical on the final build".
- Also, `../new-world-p19-base` still exists. Brief 9.3 requires removing it; the report says "이 패스 끝에 제거".

**Fix**
- Run the final gate on the report-commit head.
- Cite gate 1's 2,288 / 0 / 0 (pins on the final production code) and the final gate in the H6 row.
- Remove the base worktree before the report's claim is finalised.

#### 5. MINOR — "18:00 꼬리 자체는 해결됐다" is stronger than the evidence

**Evidence** (h5.md):
- The 18:00 tick is still 0.614 ms against v20's 0.505 (+22 %).
- The 17:00–19:00 window's largest tick is 0.667 against 0.510 (+31 %).
- v24 r3 has a p99.9 of 1.566 ms (+141 %), which is not explained.
- p99.9 of a 1-day run (1,440 ticks) is about the second-largest tick, so the bar is passed on a median of five noisy samples.
- Pass 18's own record (DECISIONS ≈L1063) had to be corrected for the same kind of over-reading.

The supported claim is that the spike shrank substantially (0.87 → 0.61 ms) and that the p99.9 bar is met on the median of five, with four of five repeats inside it. The registered H1 and the H5 mean both failed.

**Fix:** reword it, e.g. "18:00 틱 비용은 크게 줄었고(0.87 → 0.61 ms, v20 0.505) p99.9 막대는 5회 중앙값으로 통과(r3 +141 % 하나 제외 넷이 막대 안)했지만, 18:00 틱은 아직 v20보다 22 % 높다".

#### 6. MINOR — the H1-edge diagnosis is presented as confirmed, but its evidence was not kept

**Evidence**
- The report says "임시 진단으로 확인한 결과 둘 다 식량 채집 여행이 한꺼번에 가능해져 … (16:01에 214명, 18:01에 53명 …)".
- The only source is DECISIONS L1134: a "temporary uncommitted test, deleted", with no saved output in `p19-data`. The RUNLOG has two 7–17 s "p19 diag tmp" runs at 12:45.
- ROUTINE.md says instead "the trigger at those minutes was not traced further". The two documents disagree.
- `Seeking{GatherFood}` becoming available at 18:01 also sits oddly with brief 7.4's "forage … trips run until 18:00"; the docs do not explain which predicate opens it.
- The 214 is `game_v24`'s count; the report omits that v23 has 219.

**Fix**
- Soften the wording to "임시(미보존) 진단에서 관찰", or keep the diagnostic output in `p19-data` and cite it.
- Align ROUTINE.md with DECISIONS.

#### 7. MINOR — "몇 명까지 부드럽게 돌아가는가" is answered from evidence narrower than the wording

**Evidence**
- SCALE and the report measure simulation CPU only, over 2 days, in one camp:
  - no bridge snapshot or marshalling cost, and no rendering;
  - no long-run state growth (relationships, memory, chronicle, births and fission do not happen in 2 days).
- "2,000명 단일 캠프도 2×에서 한 코어의 12 % … 여유가 크다" and "메모리는 문제가 아니다(16.5 MB)" read as statements about the live game.
- The limitations list only the single-camp caveat.

**Fix:** add "시뮬레이션만, 2일, 브리지 스냅샷·렌더 비용과 장기 상태 증가는 측정 안 함" to that answer and to the limitations.

#### 8. NOTE — two dispositions say "recorded" where the record is thin

**Evidence**
- Disposition 9's H5 slope-stage order (v24 before v20, Pass 18's new-then-old) appears only in REVIEW_PASS19.md. PERF_BUDGET's procedure line says "old rules then new, interleaved" and does not mention that the slope stage is reversed.
- Disposition 3: DECISIONS has only "(deviation recorded)" in L1144, without naming `WorldRules::V24_ROUTINE`, `sim_test::probe::chronicle_out::{split, document, write, twins_match}` or `run_with_chronicle`. The report says "DECISIONS에 기록".

**Fix:** add one explicit DECISIONS line for each.

#### 9. NOTE — small number, unit and reproducibility nits in the report and docs

- **macOS version.** `sw_vers` gives 27.0.1; the report says "macOS 27.0".
- **Memory size.** "메모리 38.65 GB" is 36 GiB (38,654,705,664 bytes); say 36 GiB or keep the decimal deliberately.
- **Units.** The scale table's "MB" are MiB (SCALE.md says so; the report does not), while C5's "34.2 MB / 18.9 MB" are decimal MB. Mark one or the other.
- **Slope bound.** "그 밖 모두 ≤ 1.18": AgentMovementSystem is 1.181.
- **Mixed builds in one percentage.** SCALE finding 1's "778 µs of 1,980 … 39 % of the whole tick" divides the subprofile build's scope by the plain build's tick. Against the subprofile build's own tick (2.051 ms) it is 38 %. The "rebuilt from scratch every tick … most inputs change slowly" attribution is not measured.
- **C5 wording.** CHRONICLE.md does not call C5's memory estimate a lower bound (the report does).
- **Trace windows.** The numbers were computed by hand with no committed script. I reproduced them exactly from the raw data, so this is only a reproducibility nit.

#### 10. NOTE — write-path details beyond round 1 (harmless today)

- **Leader record before the push.** `FissionStarted` (with `other = leader`), `FissionCancelled` and `DaughterFounded` are appended with `chronicle.push`, not `chronicle_log::record`. For `FissionStarted` the leader's record is therefore not ensured *before* the push, as brief 6.1 asks. It is ensured by the `Departed` entries that follow, and founders and newborns always have records already.
- **Cost per death.** `despawn_agent` calls `alive_entity(world, partner)` (an O(N) world scan) for every death with a recorded partner, and `ensure` scans as well whenever a record is missing. This is fine at these sizes, but it is an O(N)-per-death cost the SCALE findings do not mention.

#### 11. NOTE — report placeholders and pending items

FILES_TBD, REVIEW_TBD, RUNLOG_TBD and GATE_TBD are expected. The RUNLOG table's Part 4 end, round 2 of the review, the final gate, the ff of `codex/revitalize-2026-10`, the PR #510 update and the base-worktree removal are still pending.

The report must also carry, as brief 0.1 / 11 require:
- the commit hashes per Part;
- both `git ls-remote` heads;
- the time per Part.

### Verdict

Round 1's dispositions were carried out:
- The MAJOR now has a real test that forces a cancel with a dead member, a remnant merge and both dissolution paths, and it would fail if any of those entries were missing, misordered within the cancel, carried the wrong cause or settlement, or were written for the dead (via the roster).
- Its one weak point is the remnant-merge ordering check, which is only about 57 % sensitive because the roster is a `RandomState` `HashSet` (finding 1). This is cheap to harden.
- H6 reproduces exactly: I1–I7 0 differences, I2 equal, determinism ×6, binaries matching. Gate 1 is green with no stops. The quiet-lock record checks out to the second.
- Every C, H and S number and every scale number in the Korean draft matches its source, and the H5, C6, scale and trace-window evaluations re-run byte-identically.

What remains is wording and completeness, not correctness:
- soften "18:00 꼬리 해결" and the H1-edge "확인" claim;
- state that the cancel, merge and dissolution paths were only exercised synthetically;
- caveat the scale answer as simulation-only, 2-day and single-camp;
- run the final gate on the report head (the gate 1 record predates the new test), then fast-forward the branch and remove the base worktree.

With those, Pass 19 is ready to close.

## Round 2 dispositions

1. **MINOR — remnant-order check only probabilistic → fixed (test only).** The remnant scenario now runs on 32 fresh engines (each roster a new randomly seeded `HashSet`), so a missing sort would pass with probability about 2^-32; comment corrected.
2. **NOTE — `is_alive` branch in `cancel` not reached → accepted.** Every death despawns immediately today, so a dead-but-present member is unreachable; left as is.
3. **MINOR — synthetic-only coverage not stated → fixed** in the report's limitations (cancel, remnant merge and dissolution never occurred in the 32 evaluation worlds; verified only by the `game_v17` harness tests).
4. **NOTE — final gate, pins in H6, base worktree → done in Part 4:** the final gate runs on the report commit (covering the new tests), the H6 row cites the pins (gate 1 and the final gate), and `../new-world-p19-base` is removed.
5. **MINOR — "18:00 꼬리 자체는 해결됐다" too strong → reworded** (the 18:00 tick is still +22 % over v20's, the window maximum +31 %, p99.9 passes on the median of five with r3 unexplained).
6. **MINOR — the edge diagnosis presented as confirmed → reworded** in the report and aligned in ROUTINE.md: observed in a temporary, unsaved diagnostic (v24 214, v23 219 at 16:01; v24 53 at 18:01), the opening predicate not traced.
7. **MINOR — scale answer broader than the evidence → caveated** in the report's answer and limitations and in SCALE.md (simulation only, 2 days, single camp; no bridge/render cost, no long-run growth).
8. **NOTE — thin records → added** to DECISIONS.md (public-item deviations listed by name; H5's slope-stage order).
9. **NOTE — nits → fixed:** macOS 27.0.1; memory 36 GiB; MiB vs decimal MB marked; "≤ 1.19 (movement 1.181)"; SCALE's percentage now uses the subprofile build's own tick (38 %) and no longer claims slowly changing inputs; CHRONICLE.md calls the memory estimate a lower bound. The trace-window numbers stay hand-computed (reproduced exactly by the reviewer).
10. **NOTE — write-path details → accepted:** `FissionStarted`/`FissionCancelled`/`DaughterFounded` are pushed without `ensure` because their only named person (the leader) is ensured by the `Departed` entries in the same step and always has a record already; the O(N) partner scan per death is recorded here.
11. **NOTE — placeholders → filled** in the report.
