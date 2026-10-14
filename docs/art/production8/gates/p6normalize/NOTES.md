# P6 evidence normalizer

Run:

```
python3 tools/art_pipeline/normalize_evidence_p8.py RAW_GOD_MANIFEST A30_RAW CALENDAR docs/art/production8/qa_evidence.json P7_PACING1 P7_PACING3
```

Inputs are read-only. JSON/gzip JSON are accepted. Output must remain inside `docs/art/production8`; source paths cannot be reused as output. Existing blind and unrelated assessment fields survive update. The tool does not modify acceptance thresholds or call an evaluator; exit 1 means normalization found inconsistencies, with the evidence/errors still written for audit. Structural malformed input raises a typed error.

Performance retains all raw CPU/load/restoration data, maps load_before uptime/process arrays into the existing evaluator fields, supplies first_three_alive_tick from the measured calendar, and includes unchanged P7 pacing source records labelled historical wall-clock evidence. `normalization_errors` records source/tick/seed validity failures. The evaluator must honor these errors and raw invalid status.

Paired-world identity is `(run_id, world_epoch)` and can span distinct P/G processes. Three requested casts per world must match measured days and each other's ticks, seed, amount 60, exact spoil/gift kind, original target and successful replies. Actual food and source-derived scarcity lines are preserved and checked for arithmetic consistency; the tool does not require post-intervention G food to remain below the baseline line. Three 06:00 morning frames and one day120 final per world are source-bound by SHA256. Last-morning actual P/theta is retained; final getter departure ticks are filtered to the original parent and the earliest is reported. Missing departures stay null. Missing/failed targets, mismatched sidecars/worlds, mixed code SHAs, and invalid final days prevent the paired contract from being marked valid.

Tests: 11 passed, including distinct run IDs with same epoch, wrong world/day/amount/CPU tick, missing target, mixed code, source overwrite prevention, read-only old pacing, and blind-field preservation. Basedpyright: zero errors/warnings. Ruff essential error checks: PASS. Raw logs in this directory. No final capture inputs were available for a real normalization run in this slice.
