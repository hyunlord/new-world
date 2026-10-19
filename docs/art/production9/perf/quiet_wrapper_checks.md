# Quiet-session wrapper checks

Verified 2026-10-14 22:22–22:24 UTC. These are isolated shell fixtures, **not measured sessions**. No production quiet/window lock, Godot, cargo, build, test workload, or gate was started. Fixture timing is not performance evidence.

## Invocation

```bash
bash tools/art_pipeline/with_quiet_lock_p9.sh baseline-W0 3600 \
  "$GODOT_BIN" --path /Users/rexxa/work/art-p9-base [measurement arguments]
```

The second argument is the entire quiet-hold budget (1–3600 seconds), including the existing window-lock wrapper's wait and child cleanup. The child inherits stdout/stderr directly. A timed-out child normally returns 143; a child failure retains its exit code. The child receives `P9_CONTENDED` (`0` or `1`), `P9_HOLD_START` (Unix epoch seconds), and `P9_HOLD_OWNER=codex-art-pass9`.

## Fixture method and results

For every fixture, the script was copied beneath a fresh `/tmp/art-p9-*` directory using:

```bash
sed "s|lock=/Users/rexxa/work/.mac_quiet.lock|lock=$fixture/quiet.lock|" \
  tools/art_pipeline/with_quiet_lock_p9.sh > "$fixture/tools/art_pipeline/with_quiet_lock_p9.sh"
```

A stub `with_window_lock.sh` created `fixture-window.lock`, installed an EXIT cleanup trap, and ran the supplied child command. PATH-local `pgrep` returned 1 (no matching processes); PATH-local `sysctl` returned `{ 0.20 0.10 0.10 }`. These substitutions affect only copies under `/tmp`.

| Check | Actual command / condition | Observed result |
|---|---|---|
| Shell syntax | `bash -n tools/art_pipeline/with_quiet_lock_p9.sh` | Exit 0 |
| Whitespace | `git diff --check -- tools/art_pipeline/with_quiet_lock_p9.sh` | Exit 0; file was untracked, so syntax and fixture checks are the substantive checks |
| Success | `bash "$fixture/tools/art_pipeline/with_quiet_lock_p9.sh" harmless 10 /usr/bin/true` | Exit 0; HOLD_START/HOLD_END with load 0.20 and contended=false; both fixture locks absent |
| Deadline | `bash "$fixture/tools/art_pipeline/with_quiet_lock_p9.sh" deadline 3 /bin/sleep 30` | Exit 143; recorded elapsed=3; both fixture locks absent |
| Stale lock | Fixture quiet lock set to `touch -t 202001010000`; wrapper ran `/usr/bin/true` | STALE_REMOVED recorded age=214212185; fixture DECISIONS.md appended; subsequent command succeeded |
| Own cooldown | Immediate subsequent invocation; fixture `sleep` printed argument and exited 77 | WAIT_COOLDOWN remaining=600s; sleep argument 600; no hold |
| Other lock | Fresh fixture quiet lock; same fail-fast mock sleep | WAIT_OTHER_LOCK age=0; sleep argument 300; no hold |
| Child failure/output/environment | Child `/bin/sh -c 'echo child-output-retained; echo "$P9_CONTENDED $P9_HOLD_START $P9_HOLD_OWNER"; exit 7'` | Output retained verbatim; `0 1792016634 codex-art-pass9`; wrapper exit 7; quiet lock absent |
| Contended timeout | Separate fixture load 4.20; mock epoch first 1792017000, then 1792020601; child checks `test "$P9_CONTENDED" = 1` | `contended-env-pass`; HOLD_START waited=3601s and contended=true; exit 0 |

Fixture locations: `/tmp/art-p9-quiet.bYDCfh`, `/tmp/art-p9-env.fzhtuG`, `/tmp/art-p9-contended.9PrsAp`. They retain generated logs and mock tools. Earlier quiet fixture cases were run sequentially with its log reset between isolated cases; the table records tool-observed output.

## Limits

No real 60-minute run or 10-minute wait was performed. Cooldown/poll delays were checked by intercepting `sleep`; contention used a fake epoch. The actual existing window wrapper was inspected, but its shared lock was not exercised. Process-name matching conservatively treats any matching `cargo`, `rustc`, `Godot`, or `worldsim_*` as contention, including an unexpected leftover process from this run. Termination targets only child process groups created by this invocation; it does not signal foreign processes.
