# M2 decisions

Numbered decisions of the M2 pass (`docs/v8/m2/M2_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m2-data/` (`$DATA`), outside the repository.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `v8/wip` = `e5594e3f` (checked with `git ls-remote` at the start,
  2026-10-22 08:42 KST).
- Choice: base worktree `../new-world-m2-base` detached at `e5594e3f`, with the main clone's `rust/Cargo.lock`
  copied in. Data in `$DATA` = `/Users/rexxa/work/m2-data/`. An owned `caffeinate -i` runs for the session.
  `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe `--workers 6`. Start 08:42 KST; the 18-hour target is
  02:42 and the 22-hour hard stop 06:42 on 2026-10-23.
- Evidence: `$DATA/TIMELOG.txt`.

## D2 — Part 0: what the art records rest on

- Context: goal §4 asks for ADR 0018 with the art passes' numbers and "the sustainability condition", and for the
  branches by commit. `v8/art-trial2` is running in a parallel session and is **not on the remote** yet
  (`git ls-remote origin 'refs/heads/v8/*'` lists only `v8/art-trial`, `v8/art-concept`, `v8/main`, `v8/wip`).
- Choice: ADR 0018 cites `v8/art-trial` at `c3dafb07` and `v8/art-concept` at `a5406866` (both read with
  `git show origin/<branch>:<path>`), M1-ART's blind reading from `docs/v8/m1art/ART_REPORT.md` 8.2, and the owner's
  choice of 05 from this goal (§2; no record of it exists on any branch). The sustainability condition is written as
  the concept's own "200-asset consistency" risk (`art/concepts/05_ink_and_tone/CONCEPT.md` on `v8/art-concept`: fix
  the marks per material and the main/secondary line ratio), tested by ART-TRIAL2. ART-TRIAL2 is listed as running,
  with no commit.
- Evidence: the ADR's Source section.
