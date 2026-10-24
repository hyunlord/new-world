# M-ART2.1 decisions

Goal: `docs/v8/m_art2_1/MART21_GOAL.md`. The goal says "never ask the owner": every choice the goal leaves open is
recorded here with its reason. Raw data lives in `/Users/rexxa/work/mart21-data/` (`$DATA`), outside the repository.

## D1 — intake (Part 0, 2026-10-24 17:2x KST)

- **Base:** `v8/wip` = `origin/v8/wip` = `a1c20220b5dc1d4c86fc983950152b7691ae6a41` (D19 on `v8/main` `d796fcac`),
  as the goal states.
- **Clean checkout:** `git status --porcelain` showed only the three items the goal names as left alone
  (` M .gitignore`, `?? .ignore`, `?? docs/revitalize/`) plus this pass's own `docs/v8/m_art2_1/` (the goal file).
  No other Claude Code process had this checkout as its working directory (`lsof -d cwd` over every `claude`
  process: ten sessions, all in other repositories). Start condition met.
- **Art source:** `origin/v8/art-trees` head **`f182769fe06b1cb60105e4d0bf127b6524277838`**
  ("v8-arttrees(p3): bind owner review artifacts to the verified remote commit"); it contains
  `docs/v8/arttrees/TREES_REPORT.md` (verdict "오너 판단 대기", i.e. the art line hands the files over with the owner
  review open). The branch descends from the current import commit `c8ed8608` (art-polish2); outside `art/flora/`,
  `art/boards/` and `docs/v8/arttrees/` it changes only `art/style05/palette_05.json`, which the import does not copy.
  So moving the import's `source_commit` to `f182769f` leaves every already-imported file byte-identical (checked by
  `art_import.py --check` after the move, Part 2). The branch is not merged; files are read from its git objects.
- **Owner notes:** `docs/v8/m_art2_1/OWNER_FEEDBACK.md` does not exist at start, so §7's owner-notes scope is
  empty (nothing to triage; nothing goes to M-EXT on that account).
- **Gate on `a1c20220`:** run once, unchanged code, in a detached worktree (`$DATA/gate0wt`, the main clone's
  `Cargo.lock` copied in) so Part 1 can proceed in this checkout meanwhile:
  `BENCH_PASS=M-ART2.1 bash tools/v8/gate.sh $DATA/gate0 --base-data /Users/rexxa/work/m55-data/base/identity`
  (the M-ART2 base). Host load at start 9.2 (contended; this pass's own work runs beside it), so a red `bench`
  there is read as contention unless the final gate repeats it. Result: PENDING.
