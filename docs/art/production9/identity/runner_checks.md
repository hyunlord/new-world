# Identity capture runner checks

Prepared without launching Godot or cargo or touching real shared locks.

## Usage

```bash
bash tools/art_pipeline/run_identity_p9.sh /Users/rexxa/work/art-p9-base /Users/rexxa/work/art-production9-evidence/identity A
bash tools/art_pipeline/run_identity_p9.sh /Users/rexxa/work/art-p9-base /Users/rexxa/work/art-production9-evidence/identity Aprime
bash tools/art_pipeline/run_identity_p9.sh /Users/rexxa/work/new-world-art-prod /Users/rexxa/work/art-production9-evidence/identity B
```

The checkout must already have the debug library, imported assets, and both p9 capture scripts. This runner does not build, import, or copy scripts. Six fresh Godot launches produce `LABEL/P1.png` through `P6.png`, matching JSON sidecars, per-framing logs, `launches.tsv`, the exact P3 plan, and runner facts. P3 retains its additional original FV artifacts. Existing label directories are refused; preserve failures and use a fresh evidence parent for another attempt.

`GODOT_BIN` defaults to the brief's executable. `P9_CAPTURE_LIMIT` defaults to1800 seconds per launch (accepted range 1–3600), passed to `with_window_lock.sh`, which invokes `run_bounded.sh`. Window acquisition retains the existing wrapper's unbounded wait. Before each launch and again after window acquisition, an existing quiet-lock directory delays the run in 300-second polls. No quiet lock is acquired, removed, or claimed by this runner; stale-lock adjudication remains with the owner following GOAL section 1.2. `P9_QUIET_LOCK` exists for isolated fixture tests and defaults to `/Users/rexxa/work/.mac_quiet.lock`; production must use that default. Capture delays inside the acquired window are covered by the per-launch bound.

Any failed command, `SCRIPT ERROR` or `ERROR:` log match, or absent/empty PNG or JSON yields a nonzero final status while remaining framings are attempted and logs retained. Pixel validity, metadata equality and all-six identity verdicts remain the comparator's responsibility. No rendering/performance/pixel-pass claim is made by these runner checks.

## Verification

- `bash -n tools/art_pipeline/run_identity_p9.sh`: exit 0.
- Isolated temporary checkout with fake Godot, fake window wrapper, real `run_bounded.sh`, and a sleep stub. All fixture files were automatically cleaned after the checks. No actual capture files were written outside fixtures.

- PASS: six separate windowed launches, fps 60, 1920x1080, P3 script/plan/env exact; configured bound forwarded six times
- PASS: quiet-lock preflight plus six post-window-acquire rechecks each poll 300 using isolated stub sleep; no real locks touched
- PASS: existing A directory refused, launch count unchanged
- PASS: invalid label rejected with exit 2
- PASS: P2 exit 17 and log retained; remaining framings attempted; missing artifacts recorded; overall exit 1
- PASS: invalid bound rejected before launch

- PASS: isolated fake Godot returned exit 0 and PNG/JSON for every framing while P2 logged `SCRIPT ERROR` and P4 logged `ERROR:`; both TSV rows recorded `log_errors=detected`, all six launches finished, overall exit 1.
- PASS: limit 3600 accepted; 3601, oversized integer, zero and nonnumeric limits rejected with exit 2.
