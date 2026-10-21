# M1.5 decisions

Numbered decisions of the M1.5 pass (`docs/v8/m1_5/M1_5_GOAL.md`). Each: context, choice, reason, evidence.
Raw data lives in `/Users/rexxa/work/m1_5-data/` (`$DATA`), outside the repository.

## D1 — Machine, base and data

- Context: goal §1. Base `v8/main` = `v8/wip` = `3183e77d` (checked with `git ls-remote` at the start,
  2026-10-21 21:21 KST).
- Choice: base worktree `../new-world-m15-base` detached at `3183e77d`, with the main clone's `rust/Cargo.lock`
  copied in (pitfalls: a fresh worktree resolves newer crates). Data in `$DATA` = `/Users/rexxa/work/m1_5-data/`.
  An owned `caffeinate -i` runs for the session. `CARGO_BUILD_JOBS=8`, tests `--test-threads=8`, probe
  `--workers 6`.
- Evidence: `$DATA/TIMELOG.txt`.

## D2 — The U-script and the M1 fixtures

- Context: goal §2 says the M1 U-script lives in `docs/v8/m1/`; it is in fact `tools/v8/u_script.json` (M1 D16,
  `docs/v8/m1/M1_EVALUATION.md` §10.1). `docs/v8/m1/` holds only its description.
- Choice: the U-script is `tools/v8/u_script.json`. The M1 colony fixtures are
  `worldsim_probe --scene colony --commands tools/v8/u_script.json --days 10 --seeds 100..132 --workers 6
  --out … --chronicle-out …` with the base binaries (`$DATA/base/fixtures/`).
