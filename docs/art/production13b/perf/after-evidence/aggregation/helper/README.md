# Offline raw capture aggregation

Run with Python 3.12 or newer; standard library only:

```sh
python3 /Users/rexxa/work/art-production13b-evidence/after-aggregation-helper/aggregate_after.py FUTURE_AFTER_FOLDER EXTERNAL_OUTPUT_FOLDER after
```

Produces `after-detail.json` and `after-detail.md`. Reads existing files only; never launches Godot, probes, builds, validators, or measurements. No timings are renormalized and no acceptance bars are changed. Every attempt remains separate. Source SHA256, raw metadata, and adjacent recorded validity sidecars are retained. Sidecars are NOT revalidated: official admission must use each capture's own calibration. Run-level snapshots and generated timer manifests remain separate provenance evidence.

Statistics use counted frames only: median, nearest-rank p90/p99, maximum, mean; primary/process/render/setup/pacing/ticks distributions; total ticks and tick-carrying frame counts/distributions; counted and block population/tick endpoints. Population-loss blocks remain visible but are excluded from eligible pooled statistics. Exclusive section distributions are checked for alignment, completeness and exact sum. Getter call/time means include zero-call frames; getter times are nested, not additive to exclusive renderer totals. GPU status is retained rather than treating unavailable GPU time as measured zero.

`COMPLETE_RAW_SHAPE` means expected frame/block counts only, not admission or acceptance. `pending` tracks raw-shape coverage; a capture can fill coverage while being invalid, so inspect `capture_valid`, `capture_errors`, `admission`, and the official verdicts. Missing input generates seven pending matrix/section entries and no timing values. Empty distributions have null statistics. Errors are retained in output; callers must inspect them (the CLI writes a report even with errors).

Matrix baseline files are rejected in `after` mode. Section captures have no phase field: the operator must supply the correct after-only input folder. Historical section files cannot be identified as historical from their schema alone. Referenced split fragments are suppressed when their merged capture is present. Do not place unrelated prior attempts inside the input folder unless they should be listed separately.

Historical smoke uses `/Users/rexxa/work/art-production13-evidence/baseline` and is explicitly labelled `historical-smoke`. It is not a Part4 after result. `verify_historical.py` compares 1,104 existing before-reference statistics/means across five matrix worlds and three section captures (absolute tolerance 1e-12 for serialized timings), verifies exclusion of W500 population-loss pooled values, and verifies seven pending entries for missing input. `test_math.py` checks nearest-rank percentiles, even medians and null empty statistics. Historical smoke admission is not rerun with fresh calibration.
