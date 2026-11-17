# Archived testing lessons

Historical evidence retained when the main skill reached its 400-line limit.

| date | symptom | root cause | fix | source |
|---|---|---|---|---|
| 2026-05-25 | 8 runtime errors (FFI calls to missing `#[func]`, parse errors, warnings) appeared only in a windowed run, after 32+ approved pipeline runs | static review and Rust tests never exercise GDScript parsing or the runtime FFI binding | gate the strict check (parse + warnings as errors + FFI name diff) and a headless boot smoke | `0238aef2`; `v7-final-art:CLAUDE.md` "Pipeline GDScript Strict Check" |
| 2026-10-10 | each renderer rewrite broke hundreds of Rust tests that read GDScript text | tests pinned implementation literals with `src.contains(…)`, hashes and line counts | prefer behavioural tests; re-pin only to an equivalent invariant or retire with a reason | `v7-final-art:docs/art/TEST_LEDGER.md`; `v7-final-art:docs/art/production/BRIEF.md` L122-127 |
| 2026-10-10 | per-agent-tick cost differed between 10- and 120-day runs of the same rules | cost grows with run length; the stress world is one camp | compare against the same-host, same-duration, same-build baseline, interleaved; use a natural seed set picked before measuring | `v7-final-art:docs/revitalize/DECISIONS.md` L326; `v7-final-art:docs/revitalize/PERF_BUDGET.md` §1.3 |
| 2026-10-10 | probe runs picked up later edits; an in-place gate had to be stopped | the probe rebuilt from a working tree that kept changing | freeze probe binaries; run base probes and gates from a detached worktree of the committed head | `v7-final-art:docs/revitalize/DECISIONS.md` L296, L358, L1031 |
