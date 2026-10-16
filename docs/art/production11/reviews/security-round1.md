# Part 5 independent security / allowed-scope review — round 1

Reviewed HEAD: `90045db04807850be0c5c5c2cb792251c2575da8`.
Baseline: merge `5cdf233409f11afdfde1134565730e21e8807a39`.
Method: read-only git history/diff and source inspection. No cargo, Godot, gate, runtime session, process signals, branch changes, or source modifications. Only this assigned external report was written. Current GOAL sections 0, 1, 2 govern the review; their explicit override forbids the generic reset/lead-main workflow.

## Verdict

**PASS: committed path scope, simulation ownership, shell argument handling, restricted UI links.**
**FAIL (LOW operational issue): identity runner does not bound all lock waiting.**
**NOT_VERIFIED: live signal cleanup, lock races, final gate, remote branch publication, actual runtime scheduling.** No critical/high security issue identified by this bounded review.

## Finding S1 — LOW — unbounded identity waits

`tools/art_pipeline/run_identity_p11.sh:37–40` loops while the quiet lock exists, with no deadline check. At line 53 it invokes `with_window_lock.sh`; that existing helper's lines 5–8 wait indefinitely for the window lock before line 10 enters `run_bounded.sh`. The inner quiet recheck at line 54 is bounded, but neither outer wait is. `P11_CAPTURE_LIMIT` therefore bounds the launched command, not the complete invocation; `P11_DEADLINE_EPOCH` is not read at all by this runner. A long foreign hold or stranded window lock can keep unattended identity collection alive beyond its task cutoff. This is availability/lifecycle risk, not evidence that any foreign process was killed or any measured hold exceeded its duration.

Recommendation: wrap the whole identity invocation in a deadline-aware parent owning its process group, or bound both outer waits and pass remaining duration downstream. Preserve failed attempts and do not remove foreign locks. Runtime reproduction was deliberately not run.

## Evidence supporting passed checks

- `git diff --name-only BASE..HEAD` filtered against GOAL 2.1 allowed roots returns no disallowed path. The only Rust changes are six static-source tests under `rust/crates/sim-test/tests/`; inspected diffs change exact P11 diagnostic path admission, scene resource counts, or the visibility search from an arbitrary byte prefix to the complete function. They do not alter simulation code. Each is recorded at `docs/art/TEST_LEDGER.md:598–603`.
- `git log BASE..HEAD -- .gitignore .ignore sim_bridge.gdextension rust/crates/sim-bridge/src docs/revitalize docs/worldmodel tools/revitalize tools/worldmodel tools/harness .claude .harness` returns no commits. Chronicle getters and all simulation Rust source remain untouched postmerge. The working tree separately retains modified `.gitignore` and untracked `.ignore`; this review did not stage or modify them.
- Shell runners quote checkout/evidence/executable arguments. Their `bash -c` bodies are constant scripts receiving data through positional parameters and `exec "$@"`, not interpolated command strings (`run_identity_p11.sh:51–55`; `run_activity_captures_p11.py:96–101`). Python subprocess calls use argv lists, without `shell=True`.
- `with_quiet_lock_p11.sh:4–15` validates hold duration/deadline; lines 98–147 enforce quiet wait/deadline, cooldown, load/process preflight and atomic mkdir acquisition. Lines 65–95 implement cleanup and check the recorded owner PID before releasing the quiet lock. Lines 155 onward establish separate child job groups and a watchdog. These are static safeguards; actual descendant termination and concurrent stale-lock recovery remain NOT_VERIFIED.
- `run_activity_captures_p11.py:122–139` uses a new session and signals its own process group on timeout. Its extra-activity and village-extension runners reuse this helper. Section profiling likewise starts a session and signals its own process group (`run_agent_sections_p11.py:142–152`). No broad `pkill` or kill-by-name cleanup was identified in the changed runner files.
- UI BBCode is assembled from numeric IDs, fixed generated syllables, and repository-owned localization (`scripts/ui/chronicle_text.gd:27–37`, `scripts/ui/person_names.gd:5–6`). `agent_inspector_panel.gd:578–592` accepts only person/village navigation and does not invoke a browser, shell, external URL, or arbitrary method based on metadata. No current untrusted-name injection route was found; future user-entered names would require escaping.

## Additional observations and limitations

`git diff --check BASE..HEAD` reports two retained evidence-format warnings: `docs/art/production11/gates/p0/cargo_test.log:3293` (blank EOF), and `docs/art/production11/gates/static-repins/a19-failed-2000-byte-window.txt:34` (captured trailing whitespace). These are raw evidence, not executable source or a security failure; do not silently rewrite originals.

This review does not certify runtime test results from log existence, actual quiet-lock discipline across past sessions, remote fast-forward publication, or visual acceptance. No dependency audit or dynamic malicious-input campaign was run. No public-facing authentication, network endpoint, or new dependency surface was introduced in the inspected change set.
