# Literal F3 audit preparation

The script is ready for an explicit parent trigger after the original frozen run terminates and a corrected evaluator report exists. Preparation must not open incomplete cohort outputs. The script itself checks the final `RUNLOG.txt` workload-end row and process exit files before reading any report or raw cohort data.

It imports no evaluator module. It reuses only typed JSON accessors from the SHA-pinned external `family_trace_recompute.py`; both previous family audit scripts are hash-checked and never modified. NumPy 2.4.3 is already installed in the local Python environment; no installation was performed.

## Invocation after completion

Use the corrected analysis report path supplied by the parent. Keep the original evaluator output intact.

```sh
PYTHONDONTWRITEBYTECODE=1 OPENBLAS_NUM_THREADS=1 OMP_NUM_THREADS=1 nice -n 19 \
  python3 /Users/rexxa/work/p17-data/f3_literal_audit.py evaluation \
  /Users/rexxa/work/p17-data/b-evaluation \
  /absolute/path/to/corrected/eval.json --json \
  > /Users/rexxa/work/p17-data/b_eval_f3_literal_audit.json \
  2> /Users/rexxa/work/p17-data/b_eval_f3_literal_audit.log

PYTHONDONTWRITEBYTECODE=1 python3 /Users/rexxa/work/p17-data/f3_literal_audit.py \
  --render-json /Users/rexxa/work/p17-data/b_eval_f3_literal_audit.json \
  > /Users/rexxa/work/p17-data/b_eval_f3_literal_audit.md
```

For tuning, replace `evaluation` with `tuning` and the root with `/Users/rexxa/work/p17-data/b-tuning/r1`; supply that cohort's corrected report. Tuning requires seeds 18100–18115 over 60 days. Evaluation requires seeds 18000–18031 over 120 days.

## Reconstruction and comparison

- Every full-sibling pair in the lifetime records is included when both known mother/father slots match in order. Residence, coexistence and life stage never filter sibling eligibility. Unknown founder parentage is not inferred.
- A sibling pair uses the first hourly snapshot where both people share the same non-null settlement when that context exists; otherwise `tick`, `settlement` and `stages` are all null. No reference context is invented for nonconcurrent siblings or siblings that never shared a settlement.
- Every unrelated pair is unique within its world and requires a shared non-null settlement. It uses the first such hourly snapshot; life stage is recorded there and never filters eligibility.
- Unrelated pairs exclude direct parent–child relations and any shared known parent across either parental slot. Half-siblings and cross-slot shared parents are excluded.
- Both `(a,b)` and `(b,a)` temperament observations are explicitly constructed. Pearson is calculated separately for NS, HA, RD and P; group means average the four correlations; F3 subtracts unrelated from sibling mean.
- Bootstrap uses 2,000 whole-world draws, retaining empty-pair worlds, with a fresh `numpy.random.default_rng(0)` for every statistic. It combines independently centered within-world covariance blocks, checks the point estimate against direct oriented observations, excludes undefined draws and records their valid count. CI endpoints use linear 2.5/97.5 percentiles.
- Every reported pair ID, first tick, settlement and stage pair is compared, including fully null sibling context. Missing, extra, duplicate and altered pairs are retained as discrepancies; per-world counts distinguish pairs without shared context. Pair counts, excluded-kin counts, unassigned-person counts, all 11 F3 point estimates/CIs and bootstrap valid counts are checked. Float tolerance is absolute `1e-12`, relative `1e-11`.
- Output hashes bind the raw people, hourly snapshots, family metadata, corrected report, terminal evidence and audit scripts. Raw files are rehashed at completion to detect changes during the audit.

Exit 0 means raw/report consistency. Exit 1 means discrepancies. Exit 2 means malformed, missing or nonterminal inputs. No exit code is an F1–F8 acceptance verdict; this is evidence extraction, not the fresh Part 8 review. No simulation, probe, build, evaluator rerun, source change or report overwrite is performed by the script.

## Preparation verification

Nineteen synthetic tests passed after the resolved full-sibling definition was applied. Before the change, the two added cases for nonconcurrent and different-settlement siblings failed as expected. Coverage includes lifetime-only siblings; unknown and reversed parent slots; cross-slot half-siblings and direct parent–child exclusions; all life-stage combinations; first shared non-null settlement; nullable report context; missing, duplicated, extra and altered pairs; explicit orientations; empty/constant samples; and the terminal guard. All 11 numeric statistics were checked against independently constructed oriented observations across 2,000 explicit whole-world resamples, and a separate check confirmed 11 fresh seed-0 generators.

Four additional synthetic integration tests passed for the complete loader/report/human-output path, a missing-pair discrepancy through that path, the registered real cohort contracts, and CLI exit/render behavior. All filesystem/report reads in these integration tests were mocked to synthetic records. The fixtures include siblings without shared context and an original terminal exit 1 followed by a corrected report, preserving that legitimate workflow.

This preparation has not read any B evaluation outputs and establishes no cohort result. Both earlier family audit scripts retain their pinned hashes.
