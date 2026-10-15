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
| R1 | MAJOR | WM2.md §6 claim 3 (M1b-act a better factual forecaster) lacked the qualifier that it holds on every-hour starts; at the three 06:00 floor starts the model is worse than `an5_off` (reviewer's paired bootstrap, not pre-registered: 168 h +0.023 [+0.016, +0.031], 24 h +0.007 [+0.004, +0.009]) | claim 3 qualified ("on every-hour starts") with that caveat and its intervals; added to "Not supported" |
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
