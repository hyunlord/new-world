# Pass 21 independent review

Fresh built-in `general-purpose` subagents, two rounds at most (brief 0.3.4, 8.2). Each round's findings are copied verbatim below, followed by the dispositions.

## Review round 1

Reviewer: independent `general-purpose` subagent, 2026-10-19 02:00–02:20 KST, at HEAD `e2b318af` (branch `codex/revitalize-2026-10-wip`). Read-only: no repository file was modified. No cargo build or test was run (the release test log `p21-data/p2_freeze_tests.log` already shows the pins 6/6 and the Pass 21 relations tests 16/16 on the freeze tree, so a rebuild would have added no evidence). Python tools were re-run on the raw data into a scratch directory only.

### What was checked and holds (evidence, no finding)

- **Commit order.** `git log --stat 6bb092d5..HEAD`: `637f63b1` (brief) → `69205067` (pins file alone, `harness_revitalize_pass21_frozen.rs` only) → `5c74612f` (tools) → `93c0f0bb` (first other Rust change). Pins were recorded before any other Rust change. The pins pass on the final Rust tree (`p2_freeze_tests.log`: 6 passed, release).
- **Allowed paths.** `git diff --name-only 6bb092d5..HEAD` touches only `rust/`, `tools/revitalize/`, `docs/revitalize/`; nothing under `sim-test/src/worldmodel/`; no `*_p20*` tool edited. The `_p21` shared-Mac and gate scripts differ from the `_p20` ones only in names, paths, owner string and the `6bb092d5` base (diffed).
- **v25 gating of the new code paths.**
  - `agent_decision.rs:1077-1083` `gather_avoid` returns `(&[], None)` unless `rule.avoid_gather`, so in `game_v25` every `nearest_gather_tile` / `gather_tile_exists` call gets the empty list (`own_side` is `all()` over an empty slice = true) and the memo key's third element is always `None`.
  - Dispute block `agent_decision.rs:2819-2873`: with caps 0, `capped=false`, `costed=true`, `may_quarrel=true`; the keyed draw has the same arguments; the loss order is unchanged.
  - `relations.rs:509` exchange-first block runs only under `rule.exchange_first`; `relations.rs:543` calls the new `exchange()` helper at the same point with the same arguments as the old inline code. The helper (`relations.rs:555-587`) is a faithful extraction: `used` is inserted before `depart`, and `rows` is updated in the same way.
  - `day_counts` is written only when `capped`. `pass21::block` returns `None` unless `dispute_cap_per_pair_day > 0` (`pass21.rs:25-27`).
  - Neither `Tallies` nor `SettlementRelations` is hashed beyond Pass 20's fields: `probe/fingerprint.rs:65-73` uses four tallies, and the pins' relations digest lists the Pass 20 fields explicitly (`frozen.rs:283-293`).
- **Every food-target selector (6.2).** `grep TargetKind::GatherFood` finds two state writes, `agent_decision.rs:2112` (Forage arm → `choose_bush` with `avoid_for(sid)`) and `:2207` (Gather arm → `nearest_gather_tile(..., gather_avoid(id).0)`). The post-pass has four paths:
  - `:3142` keeps the Forage-arm bush;
  - `:3146` keeps a held target that still has food (documented as unchanged);
  - `:3152` re-resolves a seeker that has no target, with the avoid list;
  - `:3234` is the no-forage-rule fallback, with the avoid list.

  The emptied-target retarget `:3326` uses `choose_bush` with the avoid list. The existence checks are `:1441` `bush_exists(avoid_for)` and `:1525` `gather_tile_exists(avoid)`. I found no other selector. The trigger (`:2190`) and the post-pass (`:3152`) call the same function with the same `local` and the same list, and `member_of` / `local_home` both take the first settlement in id order, so the radius-confounder rule holds. The `GatherMemo` key is `(tile, local, owner)` (`:1520-1521`), and `harness_p21_gather_memo_keys_the_avoid_list_owner` proves the split.
- **Caps (6.3).** For each dispute:
  - It is tallied first (`:2818`).
  - `costed = cap==0 || costed_today < cap` and `may_quarrel = cap==0 || quarrels_today < cap` are read before the increment.
  - The draw runs only if `may_quarrel`.
  - The ledger is incremented by `costed` and by `quarrel`.
  - It is cleared with `disputed` on a new display day (`:1067`).

  I found no off-by-one. When the quarrel cap is reached, `quarrelled` is not set, so the pickup proceeds. Tests `dispute_cap_stops_the_loss_not_the_count` and `quarrel_cap_skips_the_draw_and_the_pickup_proceeds` cover both, plus the day reset.
- **Exchange first (6.5).** The exchange is tried at the first row of the unordered pair in either orientation (`relations.rs:509-521`). On success it uses both ordered pairs and skips aid; on failure aid runs as in v25. At most one orientation can meet the condition (the surplus line is above the short line). The deviation is recorded in DECISIONS:1237 with tests for both orientations and the fallback.
- **Keep line and V26 values.** `rules.rs:1006-1019` sets giver 2500, keep 2000, receiver 2000, drift 30, dispute 5, quarrel 40, flags true, caps 3/1, and everything else from `V25_RELATIONS`. This equals G2. `harness_p21_v26_relations_frozen_at_grid_g2` pins every field.
- **Grid (7.2).**
  - `grid_p21.py` re-run on `p21-data/grid` reproduces `grid.json` byte-for-byte (parsed equal), and `test_grid_p21.py` passes 4/4.
  - Recomputing from `grid.json` independently: G5, G10, G11 and G12 fail the reconciliation bar (2/5 each) and the other eight qualify. The top delivering share among the qualifiers is 0.714, shared by G2, G3, G6, G7, G8 and G9, so the earliest wins → **G2**, as committed.
  - The configuration order matches the brief (`CONFIGS`, unit-tested).
  - The grid started after the diagnostic ended (diag end 23:08:20, G1 start 23:08:28).
- **Registration before evaluation.** METRICS `## Pass 21` was committed at 00:17:15 (`8f681805`). The eval build ran 00:17:24–00:17:44 (`p21-data/RUNLOG.txt`) and the first eval run started at 00:17:51 (`p21-2_eval/times.txt`).
- **Evaluation numbers.**
  - `evaluate_pass21.py --no-u4` re-run on `p21-data/p2/A` reproduces `eval.json`'s U1, U2, U3, U4_deaths and U7 exactly.
  - Independent recomputation from the raw summaries gives 19 multi-village worlds, 13 delivering, and 45 pairs at the end of which 13 are estranged.
  - Quarrels while estranged, by the 08:00 day state: v25 768/864 (0.889) and v26 56/322 (0.174); the tally gives 51/322. Both are below 0.2, and RELATIONS' "81–89 % → 10–16 %" holds.
  - Reconciliation durations: evaluation [10, 11, 11, 11, 12, 13, 15, 29] days, comparison [10–14 ×12, 39], which supports "19 of 21, the other two 29 and 39".
  - The U6 natural seed pick (`settlements_alive_daily[59] ≥ 2`) reproduces the DECISIONS list.
  - DETERMINISM PASS appears in the diag, A and C logs.
- **Diagnostic.** `diag_p21.py` re-run on `p21-data/diag` reproduces `diag.json` exactly. On the eval seeds it gives pooled λ before = 1.454 and estranged-pair contact = 0.204, as RELATIONS states.
- **U6.** `evaluate_perf_p21.py` re-run on `p21-data/perf/u6` reproduces `u6.json` and `u6.md` exactly, and `provenance.json` equals the raw copy. The p21 evaluator is a faithful port of p20's with the candidate moved to v26 (diffed).
- **Quiet lock.**
  - `quiet_RUNLOG.txt`: request 00:50:24, PAUSE, settled after 393 s at load 3.34 (threshold < 4), hold 00:56:57–01:25:56 (1,738.7 s), contended 0.
  - The watcher logs show STOP of all four own groups (53521, 55050, 55073, 60525) at 00:50:27–32 and CONT at 01:25:58–01:26:03.
  - No job of this pass started inside the window (grep of every `RUNLOG.txt` under `p21-data`).
  - `commands.jsonl` has 55 rows, all exit 0, with `sha_before == sha_after`.
- **Part 1 identity, partial (not committed).** I compared the completed Part 1 identity summaries (`p21-data/p1id/probe/i{1,3..8}_*`) with `probe/p21-0_base` using `compare_summaries.py`: I1 and I3–I8 show 0 differences and 0 one-sided paths. **I9 (`game_v25`) on the Part 1 build was still running** (started 01:55:04), and the final identity had only started (02:02:55, I1). See M2.

### Findings

#### M1 — MAJOR — U7 claims a C1–C4 pass that was not run
- **Where:** `docs/revitalize/probe/p21-2_eval/eval.md:21`; `tools/revitalize/evaluate_pass21.py:256` (hard-coded text `(C1–C4: chronicle.md)`) and `:200` (U7 verdict from T7 and the 6.3 checks only).
- **Evidence:**
  - METRICS `## Pass 21` U7 requires "C1–C4 via `evaluate_pass20_chronicle.py` with C4 against a second `game_v26` run".
  - `docs/revitalize/probe/p21-2_eval/` holds no `chronicle.md` (`ls`: `eval.json`, `eval.md`, `eval_binary.sha256`, `times.txt`).
  - The second `game_v26` run B started at 02:00:20 (`p21-data/p2/RUNLOG.txt`) and at 02:08 had finished 4 of 32 seeds (`game_v26_B.log`). Yet `ac412643` (the evaluation commit) is dated 02:04:24, so C4 cannot have been evaluated.
  - The row nevertheless reads "PASS (C1–C4: chronicle.md)".
- **Fix:** After run B finishes, run `evaluate_pass20_chronicle.py` (C1–C3 on A, C4 A vs B) and commit `chronicle.md`. Make U7's verdict depend on it, or change the text to "T7 + 6.3: PASS; C1–C4 pending" until it exists. Remove the hard-coded link from the script.

#### M2 — MAJOR — "`game_v25` unchanged / bit-identical (I9)" is stated as established, but the Part 1 and final I9 runs are not finished
- **Where:**
  - `docs/revitalize/RELATIONS.md:74` ("`game_v25` and every older scene are unchanged (identity I1–I9 and all pins)");
  - `docs/revitalize/probe/p21-2_compare/compare.md:3` ("bit-identical in this pass's build, I8/I9");
  - `docs/revitalize/DECISIONS.md:1249` ("`game_v25` and older are bit-identical, I9/I8").
- **Evidence:**
  - At HEAD, `docs/revitalize/probe/p21-1_identity/` is untracked and holds logs only. `p21-data/p1id` I9 started at 01:55:04 and had not finished; `p21-data/finalid` holds I1 only.
  - The Part 1 `cargo test --workspace` (watched group 60525, started 22:48:39, debug, `--no-fail-fast`) is still running at 02:13. It was alive with a test binary at 63 % CPU, and has no `end:` line in `p21-data/RUNLOG.txt`.
  - Brief 6.7 lists "Identity: I1–I9 identical on the Part 1 build" among the checks of Part 1 before its push. The Part 1 commits (`93c0f0bb`, `b1b1cab2`) are on the pushed branch. The DECISIONS "Ordering" bullet (1246) discloses that Part 2 started early; it does not say that the Part 1 push preceded the Part 1 identity and workspace tests.
  - What is established: I1 and I3–I8 equal on the Part 1 build (my check above); the pins pass on the final tree; the code reading in this review finds every new path gated.
- **Fix:** Finish the Part 1 I9 run and validate it with `validate_identity_p21.sh` (all 8 labels, I2, determinism), then commit `probe/p21-1_identity/` with its validation file. Do the same for the final identity. Until then, change the three sentences to "pending (I1–I8 equal on the Part 1 build; I9 running)". Record in DECISIONS that the Part 1 push preceded its identity and workspace tests.

#### m1 — MINOR — the `game_v25` run state and outputs change, though no compared output does
- **Where:** `agent_decision.rs:2853-2855` (`dispute_days` written under every relations rule); `agent_decision.rs:2871` (`quarrels_while_estranged` counted under every rule); `chronicle_out.rs:84-96` (`relations_<seed>.json` written for every relations scene); `components/relations.rs:105-109`.
- **Evidence:** `p21-data/p2/A/chronicle_game_v25` holds 64 files, against 32 for `chronicle_game_v24`. In `game_v25`, `SimResources.settlement_relations.dispute_days` and `tallies.quarrels_while_estranged` now differ from the base build.
  - Identity still holds, because the summary, chronicle file, fingerprint, digest and pins exclude these. The identity validators compare only `chronicle_*.json` (`validate_identity_p21.sh:33`).
  - Brief §2 says "`game_v25` and older scenes are bit-identical", and 6.6 limits new outputs to capped rules.
  - DECISIONS:1236 and :1241 disclose both, the latter as a §2 public-item deviation. Neither mentions the extra output file for `game_v25`, nor that `dispute_days` grows by one entry per disputing pair-day without bound in the live game, where it is never read.
- **Fix:** Record the extra `game_v25` output file and the unbounded growth in DECISIONS. Consider pruning `dispute_days` (or gating it behind a probe-only flag) in a later pass.

#### m2 — MINOR — the stated reason why exchange never happens misdescribes the condition
- **Where:** `docs/revitalize/RELATIONS.md:106` ("no village had ≥ 4 nights of wood beside a partner short of both wood and food"); `docs/revitalize/probe/p21-2_diag/diag.md:43` ("no village had a wood surplus beside a food-poor partner short of wood").
- **Evidence:** The condition (`relations.rs:477-480`) needs a food sender with ≥ 3,000 ‰ food days and < 2 wood nights, and a wood sender with ≥ 4 wood nights and < 2,000 ‰ food days. Over the 3,550 in-reach ordered pair-days of the `game_v26` evaluation (`p2/A/game_v26`, `pass20.days` columns 6/8/10/12):
  - the food-sender side held on 1,072;
  - the wood-sender side (wood-rich and food-short) held on only 15;
  - both together held on 0.

  The food shortage belongs to the wood-rich village, not to its partner, and the binding constraint is that wood-rich villages are almost never food-short.
- **Fix:** Reword both sentences with these counts.

#### m3 — MINOR — a count in RELATIONS is ambiguous
- **Where:** `docs/revitalize/RELATIONS.md:104`.
- **Evidence:** "8 of 21 estrangements were reconciled … after 10–15 days in 19 of 21 cases". The second "21" counts reconciliations pooled over the evaluation (8) and the comparison (13), not the 21 estrangements of the first clause. The numbers themselves are correct (durations above).
- **Fix:** Write "of the 21 reconciliations on both seed sets, 19 came 10–15 days after the estrangement (the other two: 29 and 39)".

#### N1 — NOTE — the pins exercise `game_v25`'s relations code weakly
- **Where:** `harness_revitalize_pass21_frozen.rs:247-270`, `:332-412`.
- **Evidence:**
  - Seeds 1–4 never split by day 60 (DECISIONS:1225), so only seed 0's relations digest covers any v25 relations code, over 60 days.
  - The rule-leaf asserts predate the four new `V25_RELATIONS` fields. Those are asserted only in `harness_revitalize_pass21_relations.rs:149-150`, which is adequate but not in the frozen file.
  - I9 (32 seeds, 120 days) is the real guard; see M2.

#### N2 — NOTE — the diagnostic's "the model holds" rests on two pairs and on an outcome-derived input
- **Where:** `probe/p21-2_diag/diag.md:37-39`; `diag_p21.py:226-235`.
- **Evidence:**
  - Only 19509 (3.34 vs 3) and 19510 (1.07 vs 1) have non-zero predictions. The other replayed pairs are 0 vs 0.
  - Each pair's contact while estranged is that pair's own measured while/before ratio. That ratio is measured on the days the pair actually spent estranged, so the replay is partly fitted to the outcome it predicts.
- **Fix:** Add one sentence stating this limitation.

#### N3 — NOTE — the comparison's causal reading of the dispute increase is not tested
- **Where:** `probe/p21-2_compare/compare.md:35` ("Disputes are more frequent (1.7×) because estranged pairs now reconcile and meet again").
- **Evidence:** It is plausible, but a second mechanism is just as consistent with the data. Under v26 a capped quarrel no longer idles the rival, and the pooled λ on non-estranged pair-days rises (19500–19515: 0.088 → 0.561; 19600–19631: 0.279 → 1.454), so foragers meet more per open pair-day.
- **Fix:** Soften to "consistent with".

#### N4 — NOTE — grid provenance
- **Where:** `probe/p21-2_grid/times.txt`.
- **Evidence:** G1 has no end time (its first driver was killed while its probe continued), and G2 started at 23:11:41 while G1 was still running. The G1-equals-diagnostic cross-check (grid.md) covers G1's integrity; the overlap matters only for timing.

#### N5 — NOTE — pre-existing forage memo key (Pass 20, not changed)
- **Where:** `agent_decision.rs:1438`.
- **Evidence:** `bush_memo` is keyed by `plan.home` only, while `bush_exists` takes `avoid_for(sid)`. Two villages sharing one formation tile, the setup of the new gather-memo test, would share answers. It is unreachable while hearths are distinct; this is outside Pass 21's scope.

#### N6 — NOTE — U1 failed-share definition
- **Where:** `evaluate_pass21.py:155,163` against METRICS (`turned back ÷ departed`).
- **Evidence:** The script uses the trip column `failed`, not `turned_back`. Both are 0 here, so the verdict is unaffected.

#### N7 — NOTE — the U6 natural p99.9 MISS
- **Evidence:** The miss is reported as registered (×1.400 > 1.35, not re-measured). The hold started at load 3.34 with another session's Python job of about 0.8 core running. The protocol's < 4 threshold was met, but the per-repeat ratios (×1.023, ×1.490, ×0.516) show the statistic has no power at three repeats. PERF_BUDGET's recommendation follows from this; no defect.

### Verdict

I found no correctness bug in the Pass 21 code:
- every food-target selector and both existence checks apply the avoid list under `avoid_gather`, and the memo key is sound;
- the caps and ledgers behave as 6.3 specifies;
- exchange first is implemented in a defensible reading that DECISIONS records;
- every new path is gated or measurement-only, so `game_v25` should be unchanged;
- the grid selection (G2), the diagnostic, U1–U3, U4's daughter rows and U6 reproduce exactly from the raw data, and the rule table was registered before the evaluation began.

The verdicts as reported are U1 PASS; U2, U3, U4 (one bar) and U6 (one bar) FAIL; they are honest. Two records are not yet supported by evidence:
- U7's "PASS (C1–C4)" cites a file and a twin run that did not exist at the commit (M1);
- the `game_v25` identity (I9) is asserted in three documents while the Part 1 and final I9 runs, and the Part 1 workspace tests, were still running (M2).

Both must be closed, or the wording corrected, before the gate and the report.

### Dispositions of round 1

| id | disposition |
|---|---|
| M1 | **Fixed.** `evaluate_pass21.py` no longer prints a C1–C4 pass it has not seen: U7's verdict takes `--chronicle-json` (the `evaluate_pass20_chronicle.py` output) and is `PENDING` without it, `FAIL` if any of C1–C4 fails. C1–C3 on run A and C4 against the second `game_v26` run B were run when run B finished; `probe/p21-2_eval/chronicle.md` / `chronicle.json` committed and `eval.md` / `eval.json` regenerated with them. |
| M2 | **Fixed.** The Part 1 identity (I9 included) and the final-build identity were validated with `validate_identity_p21.sh` when they finished and committed (`probe/p21-1_identity/`, `probe/p21-3_final/` with their `validation.txt`); the statements in RELATIONS.md, compare.md and DECISIONS stand on them. DECISIONS records that the Part 1 push preceded its identity and workspace tests, and the workspace result (2,351 / 0 / 0). |
| m1 | **Recorded** in DECISIONS (the extra `relations_<seed>.json` of a `game_v25` run, the unbounded measurement-only `dispute_days` in the live game); no code change in this pass (it would change the final build after its identity). |
| m2 | **Fixed** in RELATIONS.md and diag.md with the reviewer's counts (verified: food-sender side 1,072, wood-sender side 15, both 0 of 3,550 in-reach pair-days). |
| m3 | **Fixed** (RELATIONS.md: "of these 21 reconciliations on both seed sets, 19 came 10–15 days after …"). |
| N1 | Acknowledged; I9 (32 worlds × 120 days) is the guard, and the four new `V25_RELATIONS` fields are asserted in the Pass 21 relations test. |
| N2 | **Fixed:** diag.md states the replay's limitation. |
| N3 | **Fixed:** compare.md says "consistent with" and names both mechanisms. |
| N4 | Recorded in DECISIONS. |
| N5 | Recorded in DECISIONS as out of scope (Pass 20 code, unreachable while hearths are distinct). |
| N6 | **Fixed:** U1's failed share uses the tallies; unchanged value (0 of 170). |
| N7 | No action (reported as registered). |
