# Collector boundary checks

Executed: 2026-10-14T22:30:38.113932+00:00

Command: `python3 tools/art_pipeline/test_summarize_perf_p9.py`

Exit code: 0

These are synthetic correctness checks, not frame-time measurements. No Godot, cargo or rendering workload was launched. The final gate can rerun this command unchanged.

Coverage: 4 accepted baseline/after fixtures; 21 rejected corrupt or incomplete evidence cases; null and even-sized median checks; and a complete four-world aggregate preserving raw samples, pooled tables and explicit optional-world omissions.

```text
PASS valid W500 baseline
PASS valid W0 baseline
PASS valid W1 baseline
PASS valid W500 after
PASS rejects restoration false: Unproven runtime restoration: camera_process_restored
PASS rejects restoration missing: Unproven runtime restoration: vsync_restored
PASS rejects matrix reordered: Missing, duplicate or reordered matrix block in /var/folders/0w/j63n8j5s46j4dl79hrwn0m2w0000gn/T/p9-collector-checks-bperq2b_/session.json
PASS rejects wrong speed: Control mismatch in frozen_Z4_0
PASS rejects wrong fps: Control mismatch in frozen_Z4_0
PASS rejects vsync enabled: Vsync/camera/window controls unproven in frozen_Z4_0
PASS rejects camera processing: Vsync/camera/window controls unproven in frozen_Z4_0
PASS rejects wrong window: Vsync/camera/window controls unproven in frozen_Z4_0
PASS rejects primary mismatch: Primary differs from process + render in frozen_Z4_0
PASS rejects negative measurement: Negative measurement in frozen_Z4_0
PASS rejects missing ablation: Missing mandatory node/shared-layer ablation rows
PASS rejects missing floor: Missing ablation floor or exact restoration
PASS rejects ablation restoration false: Missing ablation floor or exact restoration
PASS rejects unpaired: Unpaired/non-increasing process frame in frozen_Z4_0
PASS rejects frame reuse: Unpaired/non-increasing process frame in frozen_Z4_0
PASS rejects frozen tick: Tick delta/control mismatch in frozen_Z4_0
PASS rejects camera move: Camera/zoom changed in frozen_Z4_0
PASS rejects valid with errors: Session claims valid despite recorded errors
PASS rejects short block: Wrong sample/warmup count in /var/folders/0w/j63n8j5s46j4dl79hrwn0m2w0000gn/T/p9-collector-checks-bperq2b_/session.json: frozen_Z4_0
PASS rejects no tick frames: No tick-carrying frames in running_4x_2
PASS rejects missing after W500 ffi_calls
PASS empty median remains null
PASS independent even median is 5.0
PASS CLI aggregate preserves raw samples, pooled tables and optional omissions
TOTAL: 28 focused checks passed; synthetic evidence only, no performance claims
```

## Gate discovery integration

Added `CollectorBoundaryTests.test_synthetic_measurement_boundaries` so the repository's unittest discovery executes the same 28-check suite; standalone CLI usage remains available.

Verified command: `python3 -m unittest discover -s tools/art_pipeline -p test_summarize_perf_p9.py`

Exit code: 0. Actual unittest output:

```text
.
----------------------------------------------------------------------
Ran 1 test in 1.871s

OK
```

The discovered test also printed all 28 PASS checks above, ending with:

```text
TOTAL: 28 focused checks passed; synthetic evidence only, no performance claims
```
