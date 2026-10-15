# WM-2 — independent review

GOAL.md 12.1: a fresh `general-purpose` subagent, two rounds at most. The reviewer read GOAL.md, METRICS.md, DECISIONS.md,
RUNLOG.md, the code under `tools/worldmodel/wm2/` and the `wm1/` changes, the copied run files, and the DGX data / run
directories read-only; it modified nothing.

## Round 1 (2026-10-15 20:10 KST, on `7fb6b707` + the split explanation; W6 and the `game_v16` floor still pending)

**Verified (no finding):** the merge (`26b9af1f` = `git merge-tree` of `cbecf7a6` and `f5373ab2`; changes after it only
under `tools/worldmodel/wm1/`, `tools/worldmodel/wm2/`, `docs/worldmodel/wm2/`); identity (every `.npy` hashed from the
files, `interventions.json` parsed, `meta.json` key by key with only `git_sha` exempt; 52 / 52, 468 files); leakage (all
five run configs: WM-1.3's 67 names exactly plus the 7 action columns; `fission_threshold` read only by
`split_task explain`, written 19:53:06 after the last `split.json` 19:52:05); roles in both manifests (0 mismatches;
floor families are test-only splits) and the registry against GOAL 7.1 (3,780 train / 386 val episodes); the
pre-registration order (`b57ef7bb` committed 02:15:36, fetched by the DGX at 02:15:48; every test output 02:15:57 or
later, every one with `code_commit` / `eval_code_commit` `b57ef7bb`; `git diff a34c8396 b57ef7bb -- tools` empty; 9
copied JSONs equal to the DGX originals by sha256); the floor estimator by hand (world 40490, 3 starts × 4 replicates ×
8 V: 168 h 0.0189485 = `floor.json`, 24 h exact); the effect alignment by hand (family 40500, 8 cases: casts on rows,
shared prefixes, true effects at +168 / +720, restricted split days and shifts equal to `effects_base.json`); baseline
selection on validation only; every W verdict against the METRICS 7 rule (and an independent recomputation of the W3 /
W4 statistics); `burn_in_states`, the full-burn-in evaluation, the rollout extension's row indexing.

| id | severity | finding | resolution |
|---|---|---|---|
| R1 | MAJOR | WM2.md §6 claim 3 (M1b-act a better factual forecaster) lacked the qualifier that it holds on every-hour starts; at the three 06:00 floor starts the model is worse than `an5_off` (reviewer's paired bootstrap, not pre-registered: 168 h +0.023 [+0.015, +0.031] — corrected in round 2 from +0.016 —, 24 h +0.007 [+0.004, +0.009]) | claim 3 qualified ("on every-hour starts") with that caveat and its intervals; added to "Not supported" |
| R2 | MAJOR | The floor families share the whole simulator state at the fork (θ, `latent`, `world_variety`, agents), so the floor is a full-state floor; headroom does not bound what an observable-history forecaster can gain | WM2.md §1 and §6 say so; "Not supported" lists it |
| R3 | MINOR | `ridge_day` θ figure −0.07 → 0.28 should be 0.27 | fixed |
| R4 | MINOR | "a third of the timing error … a bound" overstated a leave-one-out R² gain of the signed error | reworded: share of variance of the signed error, slightly optimistic, no bound |
| R5 | MINOR | "M1-long predicts deaths after a gift": it predicts fewer deaths | fixed (−0.05 / −0.29 vs true −0.04) |
| R6 | MINOR | "factual and four interventions", "reproduces … bit for bit" overstated | "up to four (489 / 560 worlds have all four)"; "52 of 52 Stage A group-F episodes (2 worlds)" |
| R7 | MINOR | `split_explain.json` carries no `code_commit` / `argv` | provenance recorded: run from the `b57ef7bb` worktree at 19:53:05 (RUNLOG Part 5, 7.7); not re-run |
| R8 | MINOR | `judge_wm2.py` changed after pre-registration without a dated note | METRICS.md amendment 1 (report-only fields; diagnostics and launchers new; no scoring change) |
| R9 | MINOR | RUNLOG: m1long_s0 s/step inconsistent; Part 3 row out of time order | fixed (rows reordered, s/step taken from the end row) |
| N1 | NOTE | the "why" diagnostics cover few runs, validation only | WM2.md names the runs and world counts |
| N2 | NOTE | the models do not tell act kinds apart (food drop after rotted wood and after a gift); the action wiring is correct | added to WM2.md §1 as evidence for the mechanism |
| N3 | NOTE | 28.4 % of `sp240L` casts were only partly applied (mean 0.951 of 240) while the model sees the requested amount | WM2.md limitations |
| N4 | NOTE | a bootstrap draw with no case in a cell drops that cell (`nanmean`); immaterial here | none |
| N5 | NOTE | W6, the `game_v16` floor, WM1_3.md's top line and this file pending | completed after the control run (round 2) |

## Round 2 (2026-10-16 00:00 KST, on `a971add3`, final)

**Verified (no finding):** the judge re-run reproduces the committed `judge.json` / `judge.md` byte for byte; the round-1
resolutions R3–R8, N1–N3, N5; W6's verdict and its provenance (`metrics.json` `eval_code_commit` and `floor_v16/floor.json`
`code_commit` `b57ef7bb`, written 23:56:07 / 23:55:44, after the pre-registration; the `b57ef7bb` worktree clean; copies
equal to the DGX originals by sha256); the partial-application figure of N3 recomputed on the DGX (978 `sp240L` casts,
28.43 % partial, mean 0.951); about 400 numbers of WM2.md (all 288 cells of the effects table, the W, factual, floor and
runs tables, the 7-day side analysis and the other non-pre-registered comparisons recomputed with `effects.py`, θ, the
diagnostics); the changed paths (183, all under the four allowed trees); commit ids and run times in RUNLOG.

| id | severity | finding | resolution |
|---|---|---|---|
| M1 | MAJOR | WM2.md §6 never used W6: claim 3 said "the same direction as WM-1.3's `game_v16` result" although the pre-registered replication on fresh `game_v16` worlds tied (model 0.210 vs WM-1.3's 0.193–0.195; `an5` 0.207 both times) | claim 3 states the W6 tie; "Not supported" lists the replication; §1 gives the model's number beside WM-1.3's |
| m1 | MINOR | two per-seed numbers in §3 (168 h s2 −0.015 → −0.014; 24 h s0 upper −0.008 → −0.007) | fixed |
| m2 | MINOR | R1's caveat interval lower bound +0.016 → +0.015 (`bs.paired` on the seed-averaged floor-matched per-world scores) | fixed in WM2.md and the round-1 row above |
| m3 | MINOR | "mean (L1-weighted) predictor … the diagnostics show why": an L1 term seeks the median, and the cause was not tested | reworded as a hypothesis consistent with the diagnostics (median-seeking loss, rare cast rows / splits) |
| m4 | MINOR | §6 claim 2 "room for better forecasters" ignored the full-state floor (R2) | "room for forecasters that know more of the state; what an observable-history forecaster can gain is not established" |
| m5 | MINOR | sign accuracy 88–97 % holds for `sp240L` and `gf240E` only (`sp120L` 77–78 %) | fixed |
| m6 | MINOR | W6 at 24 h is not pre-registered | labelled so |
| m7 | MINOR | W4's TIE is against `an5_arm`, not the zero predictor | fixed ("predictions identical to the zero predictor's") |
| m8 | MINOR | RUNLOG `floor_v19` times 19:31–19:53 → 19:29:45–19:51:18 | fixed |
| n1 | NOTE | `ridge_day` within ±7 d 0.088 vs judge 0.087 | fixed (0.087) |
| n2 | NOTE | `judge.json` W4 `censored_pred` always empty | field removed from `judge_wm2.py` (report only; the censored shares are in WM2.md) |
| n3 | NOTE | s/step figures (1.56 vs 1.57; 0.40 vs 0.41) and "a quarter of the compute" (≈ 0.29 of wall-clock) | WM2.md §2 and RUNLOG tidied |
| n4 | NOTE | §3's floor list omits `alive` (0.0007 at 168 h) | left as is (the list names the non-zero and the exactly-zero variables) |
| n5 | NOTE | the models' 45 % within ±7 d = 34 censored worlds + 2 late splits | reworded |
| n6 | NOTE | gate at HEAD, REPORT.md, fast-forward, PR comment still open | done after this round (RUNLOG Part 7) |
