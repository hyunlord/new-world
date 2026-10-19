# Measurement matrix resume checks

The CLI remains `bash tools/art_pipeline/run_perf_matrix_p9.sh baseline|after CHECKOUT EVIDENCE_DIR`.

For each world, the runner examines all existing attempt numbers 1–3. It accepts the earliest complete JSON/log pair only after `summarize_perf_p9.load_session()` independently validates the full session, the world/phase match, `valid` is true, `errors` is empty, and the log contains no ERROR marker. Missing JSON or log, truncated JSON, invalid controls/restoration, and error logs remain rejected evidence. An external completed W0 attempt without an exit receipt can be reused through these checks.

New sessions write an additional `.exit` receipt. Nonzero receipts remain rejected on every later resume even when their JSON is complete. Existing files are never overwritten. If no attempt is accepted, numbering resumes above the highest occupied attempt, through attempt 3; gaps are not recycled. Every existing attempt is retained. Accepted worlds are still collected after the deadline, but no new session launches after it. A directory lock prevents two matrix runners for the same phase/evidence directory; an interrupted runner's stale lock requires inspecting its process before removal.

## Verification

Command: `bash -n tools/art_pipeline/run_perf_matrix_p9.sh` — exit 0.

Command: `python3 -m unittest discover -s tools/art_pipeline -p test_run_perf_matrix_p9.py` — exit 0.

Actual unittest output:

```text
.........
----------------------------------------------------------------------
Ran 9 tests in 9.809s

OK
```

The nine tests cover:

- Complete sessions reused without launches or byte changes.
- Truncated JSON retained; attempt 2 launched.
- ERROR log rejected despite complete JSON.
- Missing log and invalid restoration retained; attempt 3 launched.
- Highest occupied attempt 3 prevents backfilling earlier gaps or further launches.
- Three nonzero exits retained and never accepted on resume.
- World mismatch rejected.
- `valid=false` rejected.
- Deadline still permits reuse of already completed worlds.

Tests copy the runner and real summarizer into a temporary tree. Fake session processes copy synthetic fixtures; the quiet-lock wrapper is replaced inside that temporary tree. No Godot, cargo, real quiet/window lock or timing workload was launched. These checks prove resume/validation behavior, not rendering performance.
